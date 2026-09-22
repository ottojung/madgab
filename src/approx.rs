use std::collections::HashMap;

use phonetics::transcriptions::Corpus;
use serde::Deserialize;

/// Cost of inserting or deleting one IPA character while aligning a
/// clue word to a target span. This is intentionally cheaper than a
/// maximally-different substitution: connected speech routinely gains
/// or loses weak segments at word boundaries.
const GAP_COST: f64 = 0.20;

/// Keep the fuzzy lattice bounded. Matches are capped independently
/// for every target-span length, after sorting by acoustic cost and
/// lexical frequency.
const MATCHES_PER_SPAN: usize = 128;

#[derive(Debug, Clone)]
pub(crate) struct FuzzyWord {
    pub(crate) word: String,
    pub(crate) ipa: String,
    pub(crate) chars: Vec<char>,
    pub(crate) rarity: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FuzzyMatch {
    pub(crate) consumed: usize,
    pub(crate) word_idx: usize,
    pub(crate) cost: f64,
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    rarity: Option<f64>,
}

/// Stress marks are useful for display but too brittle for connected-
/// speech matching. OpenEPD mixes broad and narrow sources, so search
/// over the segment stream and leave stress out of the edit lattice.
pub(crate) fn normalize_ipa(ipa: &str) -> String {
    ipa.chars()
        .filter(|&c| c != 'ˈ' && c != 'ˌ')
        .collect()
}

/// Build the word list used by the edit-distance lattice.
///
/// Corpus::from_json has already validated the same JSON before this
/// function is called. Parsing only rarity here gives us an iterable
/// list of words while still delegating pronunciation preference to the
/// corpus itself.
pub(crate) fn build_lexicon(
    json: &str,
    corpus: &Corpus,
    max_rarity: Option<f64>,
) -> Vec<FuzzyWord> {
    let raw: HashMap<String, RawEntry> =
        serde_json::from_str(json).expect("Corpus::from_json already validated this JSON");

    let mut words = Vec::new();
    for (word, entry) in raw {
        if let (Some(max), Some(rarity)) = (max_rarity, entry.rarity) {
            if rarity > max {
                continue;
            }
        }
        let Some(ipa) = corpus.preferred_ipa(&word) else {
            continue;
        };
        let ipa = normalize_ipa(ipa);
        if ipa.is_empty() {
            continue;
        }
        words.push(FuzzyWord {
            word,
            chars: ipa.chars().collect(),
            ipa,
            rarity: entry.rarity,
        });
    }

    // Determinism matters for ties and for browser/native builds.
    words.sort_by(|a, b| a.word.cmp(&b.word).then_with(|| a.ipa.cmp(&b.ipa)));
    words
}

/// Enumerate words that can align to a prefix beginning at start.
///
/// Unlike phonetics-rs' substitution-only trie walk, this matcher uses
/// weighted Levenshtein alignment, so a word may consume a target span
/// one or two segments shorter/longer when the edit budget permits it.
/// The expensive lattice is computed once per target position, not once
/// per beam hypothesis.
pub(crate) fn matches_at(
    words: &[FuzzyWord],
    target: &[char],
    start: usize,
    budget: f64,
    min_word_ipa_chars: usize,
) -> Vec<FuzzyMatch> {
    if start >= target.len() || budget < 0.0 {
        return Vec::new();
    }

    let remaining = target.len() - start;
    let max_gaps = (budget / GAP_COST + 1e-9).floor() as usize;
    let mut by_span: Vec<Vec<FuzzyMatch>> = vec![Vec::new(); remaining + 1];

    for (word_idx, word) in words.iter().enumerate() {
        let word_len = word.chars.len();
        if word_len < min_word_ipa_chars || word_len == 0 {
            continue;
        }

        let min_span = word_len.saturating_sub(max_gaps).max(1);
        let max_span = word_len.saturating_add(max_gaps).min(remaining);
        if min_span > max_span {
            continue;
        }

        for consumed in min_span..=max_span {
            let length_lb = word_len.abs_diff(consumed) as f64 * GAP_COST;
            if length_lb > budget + 1e-9 {
                continue;
            }
            let span = &target[start..start + consumed];
            if let Some(cost) = weighted_distance_bounded(span, &word.chars, budget) {
                by_span[consumed].push(FuzzyMatch {
                    consumed,
                    word_idx,
                    cost,
                });
            }
        }
    }

    let mut out = Vec::new();
    for matches in by_span.iter_mut().skip(1) {
        matches.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    rarity_key(words[a.word_idx].rarity)
                        .cmp(&rarity_key(words[b.word_idx].rarity))
                })
                .then_with(|| words[a.word_idx].word.cmp(&words[b.word_idx].word))
        });
        if matches.len() > MATCHES_PER_SPAN {
            matches.truncate(MATCHES_PER_SPAN);
        }
        out.extend(matches.iter().copied());
    }
    out
}

fn rarity_key(rarity: Option<f64>) -> u64 {
    match rarity {
        Some(r) if r.is_finite() && r >= 0.0 => r.round() as u64,
        _ => u64::MAX,
    }
}

/// Weighted Levenshtein distance between a target span and one clue
/// pronunciation. Substitutions use phonetics-rs' calibrated
/// per-segment distance; insertions/deletions use a small fixed gap
/// cost appropriate for connected-speech elision.
fn weighted_distance_bounded(a: &[char], b: &[char], budget: f64) -> Option<f64> {
    if a.len().abs_diff(b.len()) as f64 * GAP_COST > budget + 1e-9 {
        return None;
    }

    let mut prev: Vec<f64> = (0..=b.len()).map(|j| j as f64 * GAP_COST).collect();
    let mut curr = vec![0.0; b.len() + 1];

    for (i, &ac) in a.iter().enumerate() {
        curr[0] = (i + 1) as f64 * GAP_COST;
        let mut row_min = curr[0];

        for (j, &bc) in b.iter().enumerate() {
            let sub = if ac == bc {
                0.0
            } else {
                phonetics::distance(&ac.to_string(), &bc.to_string())
            };
            let delete = prev[j + 1] + GAP_COST;
            let insert = curr[j] + GAP_COST;
            let replace = prev[j] + sub;
            let v = delete.min(insert).min(replace);
            curr[j + 1] = v;
            row_min = row_min.min(v);
        }

        if row_min > budget + 1e-9 {
            return None;
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let cost = prev[b.len()];
    (cost <= budget + 1e-9).then_some(cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighted_distance_allows_boundary_insertions() {
        let target: Vec<char> = "ɪts".chars().collect();
        let clue: Vec<char> = "hɪts".chars().collect();
        let cost = weighted_distance_bounded(&target, &clue, 0.5).unwrap();
        assert!((cost - GAP_COST).abs() < 1e-9);
    }

    #[test]
    fn weighted_distance_rejects_too_many_gaps() {
        let target: Vec<char> = "a".chars().collect();
        let clue: Vec<char> = "abcd".chars().collect();
        assert!(weighted_distance_bounded(&target, &clue, 0.5).is_none());
    }
}
