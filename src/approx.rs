use std::collections::{HashMap, HashSet};

use phonetics::transcriptions::Corpus;
use serde::Deserialize;

/// Cost of inserting or deleting one IPA character while aligning a
/// clue word to a target span.
const GAP_COST: f64 = 0.20;

/// Keep each target-span bucket bounded after ranking by edit cost and
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

#[derive(Debug)]
pub(crate) struct FuzzyLexicon {
    words: Vec<FuzzyWord>,
    by_len: Vec<Vec<usize>>,
    substitution_costs: HashMap<(char, char), f64>,
}

impl FuzzyLexicon {
    pub(crate) fn empty() -> Self {
        Self {
            words: Vec::new(),
            by_len: vec![Vec::new()],
            substitution_costs: HashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub(crate) fn word(&self, index: usize) -> &FuzzyWord {
        &self.words[index]
    }

    /// Enumerate words that can align to a prefix beginning at start.
    ///
    /// Weighted Levenshtein alignment lets clue words consume a target
    /// span that is shorter or longer than their own IPA when the edit
    /// budget permits it. DP buffers and phoneme-pair distances are
    /// reused/precomputed because this is the dominant hot path.
    pub(crate) fn matches_at(
        &self,
        target: &[char],
        start: usize,
        budget: f64,
        min_word_ipa_chars: usize,
    ) -> Vec<FuzzyMatch> {
        if start >= target.len() || budget < 0.0 || self.words.is_empty() {
            return Vec::new();
        }

        let remaining = target.len() - start;
        let max_gaps = (budget / GAP_COST + 1e-9).floor() as usize;
        let mut by_span: Vec<Vec<FuzzyMatch>> = vec![Vec::new(); remaining + 1];

        let max_word_len = self
            .by_len
            .len()
            .saturating_sub(1)
            .min(remaining.saturating_add(max_gaps));
        let scratch_len = max_word_len.saturating_add(1).max(2);
        let mut prev = vec![0.0; scratch_len];
        let mut curr = vec![0.0; scratch_len];

        for word_len in min_word_ipa_chars.max(1)..=max_word_len {
            let Some(indices) = self.by_len.get(word_len) else {
                continue;
            };
            if indices.is_empty() {
                continue;
            }

            let min_span = word_len.saturating_sub(max_gaps).max(1);
            let max_span = word_len.saturating_add(max_gaps).min(remaining);
            if min_span > max_span {
                continue;
            }

            for &word_idx in indices {
                let word = &self.words[word_idx];
                for consumed in min_span..=max_span {
                    let length_lb = word_len.abs_diff(consumed) as f64 * GAP_COST;
                    if length_lb > budget + 1e-9 {
                        continue;
                    }

                    let span = &target[start..start + consumed];
                    if let Some(cost) = weighted_distance_bounded(
                        span,
                        &word.chars,
                        budget,
                        &self.substitution_costs,
                        &mut prev,
                        &mut curr,
                    ) {
                        by_span[consumed].push(FuzzyMatch {
                            consumed,
                            word_idx,
                            cost,
                        });
                    }
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
                        rarity_key(self.words[a.word_idx].rarity)
                            .cmp(&rarity_key(self.words[b.word_idx].rarity))
                    })
                    .then_with(|| {
                        self.words[a.word_idx]
                            .word
                            .cmp(&self.words[b.word_idx].word)
                    })
            });
            if matches.len() > MATCHES_PER_SPAN {
                matches.truncate(MATCHES_PER_SPAN);
            }
            out.extend(matches.iter().copied());
        }
        out
    }
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    rarity: Option<f64>,
}

pub(crate) fn normalize_ipa(ipa: &str) -> String {
    ipa.chars()
        .filter(|&c| c != 'ˈ' && c != 'ˌ')
        .collect()
}

/// Build an iteration-friendly view of the preferred OpenEPD
/// pronunciations plus the small lookup tables needed by fuzzy search.
pub(crate) fn build_lexicon(
    json: &str,
    corpus: &Corpus,
    max_rarity: Option<f64>,
) -> FuzzyLexicon {
    let raw: HashMap<String, RawEntry> =
        serde_json::from_str(json).expect("Corpus::from_json already validated this JSON");

    let mut words = Vec::new();
    let mut alphabet = HashSet::new();

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
        let chars: Vec<char> = ipa.chars().collect();
        alphabet.extend(chars.iter().copied());
        words.push(FuzzyWord {
            word,
            ipa,
            chars,
            rarity: entry.rarity,
        });
    }

    words.sort_by(|a, b| a.word.cmp(&b.word).then_with(|| a.ipa.cmp(&b.ipa)));

    let max_len = words.iter().map(|w| w.chars.len()).max().unwrap_or(0);
    let mut by_len = vec![Vec::new(); max_len + 1];
    for (index, word) in words.iter().enumerate() {
        by_len[word.chars.len()].push(index);
    }

    let alphabet: Vec<char> = alphabet.into_iter().collect();
    let mut substitution_costs = HashMap::new();
    for &a in &alphabet {
        for &b in &alphabet {
            let cost = if a == b {
                0.0
            } else {
                phonetics::distance(&a.to_string(), &b.to_string())
            };
            substitution_costs.insert((a, b), cost);
        }
    }

    FuzzyLexicon {
        words,
        by_len,
        substitution_costs,
    }
}

fn rarity_key(rarity: Option<f64>) -> u64 {
    match rarity {
        Some(r) if r.is_finite() && r >= 0.0 => r.round() as u64,
        _ => u64::MAX,
    }
}

fn weighted_distance_bounded(
    a: &[char],
    b: &[char],
    budget: f64,
    substitution_costs: &HashMap<(char, char), f64>,
    prev: &mut [f64],
    curr: &mut [f64],
) -> Option<f64> {
    if a.len().abs_diff(b.len()) as f64 * GAP_COST > budget + 1e-9 {
        return None;
    }
    debug_assert!(prev.len() > b.len() && curr.len() > b.len());

    for (j, slot) in prev.iter_mut().take(b.len() + 1).enumerate() {
        *slot = j as f64 * GAP_COST;
    }

    for (i, &ac) in a.iter().enumerate() {
        curr[0] = (i + 1) as f64 * GAP_COST;
        let mut row_min = curr[0];

        for (j, &bc) in b.iter().enumerate() {
            let sub = *substitution_costs.get(&(ac, bc)).unwrap_or(&1.0);
            let delete = prev[j + 1] + GAP_COST;
            let insert = curr[j] + GAP_COST;
            let replace = prev[j] + sub;
            let value = delete.min(insert).min(replace);
            curr[j + 1] = value;
            row_min = row_min.min(value);
        }

        if row_min > budget + 1e-9 {
            return None;
        }

        for j in 0..=b.len() {
            std::mem::swap(&mut prev[j], &mut curr[j]);
        }
    }

    let cost = prev[b.len()];
    (cost <= budget + 1e-9).then_some(cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(chars: &[char]) -> HashMap<(char, char), f64> {
        let mut out = HashMap::new();
        for &a in chars {
            for &b in chars {
                out.insert(
                    (a, b),
                    if a == b {
                        0.0
                    } else {
                        phonetics::distance(&a.to_string(), &b.to_string())
                    },
                );
            }
        }
        out
    }

    #[test]
    fn weighted_distance_allows_boundary_insertions() {
        let target: Vec<char> = "ɪts".chars().collect();
        let clue: Vec<char> = "hɪts".chars().collect();
        let mut all = target.clone();
        all.extend(clue.iter().copied());
        let costs = table(&all);
        let mut prev = vec![0.0; clue.len() + 1];
        let mut curr = vec![0.0; clue.len() + 1];
        let cost = weighted_distance_bounded(
            &target,
            &clue,
            0.5,
            &costs,
            &mut prev,
            &mut curr,
        )
        .unwrap();
        assert!((cost - GAP_COST).abs() < 1e-9);
    }

    #[test]
    fn weighted_distance_rejects_too_many_gaps() {
        let target: Vec<char> = "a".chars().collect();
        let clue: Vec<char> = "abcd".chars().collect();
        let costs = table(&['a', 'b', 'c', 'd']);
        let mut prev = vec![0.0; clue.len() + 1];
        let mut curr = vec![0.0; clue.len() + 1];
        assert!(weighted_distance_bounded(
            &target,
            &clue,
            0.5,
            &costs,
            &mut prev,
            &mut curr,
        )
        .is_none());
    }
}
