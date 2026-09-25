use std::collections::{BTreeMap, HashMap, HashSet};

use phonetics::transcriptions::Corpus;
use serde::Deserialize;

/// Cost of inserting or deleting one IPA character while aligning a
/// clue word to a target span.
const GAP_COST: f64 = 0.20;

/// Bound the number of word alternatives exposed for any one target
/// span after trie traversal. Search still sees multiple acoustic and
/// lexical alternatives, but pathological homophone clusters cannot
/// swamp the phrase beam.
const MATCHES_PER_SPAN: usize = 256;

#[derive(Debug, Clone)]
pub(crate) struct FuzzyWord {
    pub(crate) word: String,
    pub(crate) ipa: String,
    pub(crate) ipa_len: usize,
    pub(crate) rarity: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FuzzyMatch {
    pub(crate) consumed: usize,
    pub(crate) word_idx: usize,
    pub(crate) cost: f64,
}

#[derive(Debug, Default)]
struct TrieNode {
    children: BTreeMap<char, usize>,
    terminations: Vec<usize>,
}

/// A compact pronunciation trie used only by Approximate mode.
///
/// phonetics-rs already has a trie, but its public approximate walk
/// supports substitutions only. Mad Gab connected speech also needs
/// small insertions/deletions near word boundaries, so this trie runs
/// a bounded weighted edit search over the same preferred
/// pronunciations.
#[derive(Debug)]
pub(crate) struct FuzzyLexicon {
    words: Vec<FuzzyWord>,
    nodes: Vec<TrieNode>,
    substitution_costs: HashMap<(char, char), f64>,
}

impl FuzzyLexicon {
    pub(crate) fn empty() -> Self {
        Self {
            words: Vec::new(),
            nodes: vec![TrieNode::default()],
            substitution_costs: HashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub(crate) fn word(&self, index: usize) -> &FuzzyWord {
        &self.words[index]
    }

    /// Find all words whose pronunciation is within the edit budget of
    /// a target prefix at start.
    ///
    /// States are (trie node, target chars consumed). For any such
    /// state only the cheapest alignment matters; a more expensive
    /// route has exactly the same possible suffixes and can be
    /// discarded. This keeps insertion/deletion support close to the
    /// cost of an ordinary trie walk instead of scanning the lexicon.
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
        let mut best: HashMap<(usize, usize), f64> = HashMap::new();
        let mut stack = vec![(0usize, 0usize, 0.0f64)];
        best.insert((0, 0), 0.0);

        // A word can be reached through more than one edit alignment.
        // Keep the cheapest cost for each (consumed span, word).
        let mut found: HashMap<(usize, usize), f64> = HashMap::new();

        while let Some((node_idx, consumed, cost)) = stack.pop() {
            if cost > budget + 1e-9 {
                continue;
            }
            if best
                .get(&(node_idx, consumed))
                .is_some_and(|&known| cost > known + 1e-9)
            {
                continue;
            }

            let node = &self.nodes[node_idx];
            if consumed > 0 {
                for &word_idx in &node.terminations {
                    if self.words[word_idx].ipa_len < min_word_ipa_chars {
                        continue;
                    }
                    found
                        .entry((consumed, word_idx))
                        .and_modify(|old| {
                            if cost < *old {
                                *old = cost;
                            }
                        })
                        .or_insert(cost);
                }
            }

            // Delete one target segment: the listener effectively
            // loses a weak segment while the clue pronunciation stays
            // at the same trie node.
            if consumed < remaining {
                push_state(
                    &mut stack,
                    &mut best,
                    node_idx,
                    consumed + 1,
                    cost + GAP_COST,
                    budget,
                );
            }

            for (&clue_char, &child_idx) in &node.children {
                // Insert one clue segment: extra material in the clue
                // pronunciation that consumes no target segment.
                push_state(
                    &mut stack,
                    &mut best,
                    child_idx,
                    consumed,
                    cost + GAP_COST,
                    budget,
                );

                // Match/substitute one segment.
                if consumed < remaining {
                    let target_char = target[start + consumed];
                    let sub = if target_char == clue_char {
                        0.0
                    } else {
                        self.substitution_costs
                            .get(&(target_char, clue_char))
                            .copied()
                            .unwrap_or_else(|| {
                                phonetics::distance(
                                    &target_char.to_string(),
                                    &clue_char.to_string(),
                                )
                            })
                    };
                    push_state(
                        &mut stack,
                        &mut best,
                        child_idx,
                        consumed + 1,
                        cost + sub,
                        budget,
                    );
                }
            }
        }

        let mut by_span: Vec<Vec<FuzzyMatch>> = vec![Vec::new(); remaining + 1];
        for ((consumed, word_idx), cost) in found {
            by_span[consumed].push(FuzzyMatch {
                consumed,
                word_idx,
                cost,
            });
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
            matches.truncate(MATCHES_PER_SPAN.min(matches.len()));
            out.extend(matches.iter().copied());
        }
        out
    }
}

fn push_state(
    stack: &mut Vec<(usize, usize, f64)>,
    best: &mut HashMap<(usize, usize), f64>,
    node: usize,
    consumed: usize,
    cost: f64,
    budget: f64,
) {
    if cost > budget + 1e-9 {
        return;
    }
    let key = (node, consumed);
    if best.get(&key).is_some_and(|&old| old <= cost + 1e-9) {
        return;
    }
    best.insert(key, cost);
    stack.push((node, consumed, cost));
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

/// Build the fuzzy trie from the same preferred pronunciations used by
/// the main corpus. The extra JSON parse gives us an iterable word
/// list; lookup/source preference remains delegated to Corpus.
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
            ipa_len: chars.len(),
            rarity: entry.rarity,
        });
    }

    words.sort_by(|a, b| a.word.cmp(&b.word).then_with(|| a.ipa.cmp(&b.ipa)));

    let mut nodes = vec![TrieNode::default()];
    for (word_idx, word) in words.iter().enumerate() {
        let mut node_idx = 0usize;
        for ch in word.ipa.chars() {
            let child_idx = if let Some(&existing) = nodes[node_idx].children.get(&ch) {
                existing
            } else {
                let new_idx = nodes.len();
                nodes.push(TrieNode::default());
                nodes[node_idx].children.insert(ch, new_idx);
                new_idx
            };
            node_idx = child_idx;
        }
        nodes[node_idx].terminations.push(word_idx);
    }

    // Precompute the small IPA-symbol substitution table once. Target
    // symbols normally come from the same corpus alphabet; the search
    // has a safe fallback for anything outside it.
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
        nodes,
        substitution_costs,
    }
}

fn rarity_key(rarity: Option<f64>) -> u64 {
    match rarity {
        Some(r) if r.is_finite() && r >= 0.0 => r.round() as u64,
        _ => u64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indel_trie_finds_inserted_initial_segment() {
        // Build a tiny trie manually: /hɪts/ should match target /ɪts/
        // by one inserted clue segment.
        let words = vec![FuzzyWord {
            word: "hits".into(),
            ipa: "hɪts".into(),
            ipa_len: 4,
            rarity: Some(100.0),
        }];
        let mut nodes = vec![TrieNode::default()];
        let mut node = 0;
        for ch in "hɪts".chars() {
            let next = nodes.len();
            nodes.push(TrieNode::default());
            nodes[node].children.insert(ch, next);
            node = next;
        }
        nodes[node].terminations.push(0);

        let alphabet: Vec<char> = "hɪts".chars().collect();
        let mut substitution_costs = HashMap::new();
        for &a in &alphabet {
            for &b in &alphabet {
                substitution_costs.insert(
                    (a, b),
                    if a == b {
                        0.0
                    } else {
                        phonetics::distance(&a.to_string(), &b.to_string())
                    },
                );
            }
        }

        let lexicon = FuzzyLexicon {
            words,
            nodes,
            substitution_costs,
        };
        let target: Vec<char> = "ɪts".chars().collect();
        let hits = lexicon.matches_at(&target, 0, 0.5, 1);
        assert!(hits
            .iter()
            .any(|m| m.word_idx == 0 && (m.cost - GAP_COST).abs() < 1e-9));
    }

    #[test]
    fn indel_trie_respects_budget() {
        let mut best = HashMap::new();
        let mut stack = Vec::new();
        push_state(&mut stack, &mut best, 1, 1, 0.6, 0.5);
        assert!(stack.is_empty());
    }
}
