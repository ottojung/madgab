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
    pub(crate) syllables: usize,
    pub(crate) rarity: Option<f64>,
    /// Cached `lexical::is_closed_class(&word)`.  The span shortlist
    /// sorts call the lexical test inside their comparators, and the test
    /// lowercases, so it is not free enough to re-run there.
    pub(crate) closed: bool,
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

            if matches.len() > MATCHES_PER_SPAN {
                const CHEAP_KEEP: usize = 96;
                const COST_BANDS: usize = 4;
                const BAND_KEEP: usize =
                    (MATCHES_PER_SPAN - CHEAP_KEEP) / COST_BANDS;

                let ordered = matches.clone();
                let mut selected = Vec::with_capacity(MATCHES_PER_SPAN);
                let mut seen = HashSet::new();

                for m in ordered.iter().take(CHEAP_KEEP) {
                    if seen.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                let mut bands: Vec<Vec<FuzzyMatch>> =
                    (0..COST_BANDS).map(|_| Vec::new()).collect();
                let scale = budget.max(1e-9);
                for &m in &ordered {
                    let band = (((m.cost / scale) * COST_BANDS as f64)
                        .floor() as usize)
                        .min(COST_BANDS - 1);
                    bands[band].push(m);
                }

                for band in &mut bands {
                    band.sort_by(|a, b| {
                        rarity_key(self.words[a.word_idx].rarity)
                            .cmp(&rarity_key(self.words[b.word_idx].rarity))
                            .then_with(|| {
                                a.cost
                                    .partial_cmp(&b.cost)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .then_with(|| {
                                self.words[a.word_idx]
                                    .word
                                    .cmp(&self.words[b.word_idx].word)
                            })
                    });
                    for m in band.iter().take(BAND_KEEP) {
                        if seen.insert(m.word_idx) {
                            selected.push(*m);
                        }
                    }
                }

                if selected.len() < MATCHES_PER_SPAN {
                    for m in &ordered {
                        if seen.insert(m.word_idx) {
                            selected.push(*m);
                            if selected.len() == MATCHES_PER_SPAN {
                                break;
                            }
                        }
                    }
                }

                selected.sort_by(|a, b| {
                    a.consumed
                        .cmp(&b.consumed)
                        .then_with(|| {
                            a.cost
                                .partial_cmp(&b.cost)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .then_with(|| {
                            rarity_key(self.words[a.word_idx].rarity)
                                .cmp(&rarity_key(self.words[b.word_idx].rarity))
                        })
                });
                *matches = selected;
            }

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

/// Vowel pairs that form a single syllabic diphthong in the corpus
/// inventory.  Everything else that looks like two adjacent vowels
/// ("uə" in *accrual*, "iə" in *academia*) really is two syllables.
const DIPHTHONGS: &[(char, char)] = &[
    ('a', 'ɪ'),
    ('a', 'ʊ'),
    ('e', 'ɪ'),
    ('o', 'ʊ'),
    ('ɔ', 'ɪ'),
    ('ɑ', 'ɪ'),
    ('ɑ', 'ʊ'),
    ('ɪ', 'ɚ'),
    ('ʊ', 'ɚ'),
];

fn is_vowel_symbol(c: char) -> bool {
    phonetics::vowels::INVENTORY
        .iter()
        .any(|v| v.starts_with(c) && v.chars().count() == 1)
        || c == 'ɚ'
}

/// Count syllable nuclei in a stress-mark-free IPA transcription.
///
/// A Mad Gab clue only works if it can be *spoken* with the target's
/// rhythm, so the search needs a cheap syllable estimate for both clue
/// words and targets.  Syllable count is not derivable from raw IPA
/// length (which is why the length proxy is only a weak signal), but it
/// is cheap: count vowel nuclei and collapse diphthongs.
///
/// Words that contain no vowel symbol at all ("mm" -> /m/) are still
/// spoken with one syllable, so they count as one.
pub(crate) fn ipa_syllables(ipa: &str) -> usize {
    let chars: Vec<char> = ipa.chars().collect();
    let mut nuclei = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        if !is_vowel_symbol(chars[i]) {
            i += 1;
            continue;
        }
        nuclei += 1;
        if i + 1 < chars.len()
            && DIPHTHONGS
                .iter()
                .any(|(a, b)| chars[i] == *a && chars[i + 1] == *b)
        {
            i += 2;
            continue;
        }
        i += 1;
    }
    nuclei.max(usize::from(!ipa.is_empty()))
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
        let closed = crate::lexical::is_closed_class(&word);
        words.push(FuzzyWord {
            word,
            syllables: ipa_syllables(&ipa),
            ipa,
            ipa_len: chars.len(),
            rarity: entry.rarity,
            closed,
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
            syllables: 1,
            rarity: Some(100.0),
            closed: false,
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

    // ZZ_PROBE (scratch/c4e8d7-measure only): line-level diagnosis of where a
    // dictionary word is dropped on the default approximate path.
    #[test]
    fn zz_probe_span() {
        let json = open_english_pronouncing_dictionary::CORPUS_JSON;
        let raw: HashMap<String, RawEntry> =
            serde_json::from_str(json).unwrap();
        let corpus = Corpus::from_json(json, Some(50_000.0)).unwrap();

        let target_ipa = crate::transcribe_with_boundaries(
            &corpus,
            "It's just a stupid game",
            true,
        )
        .unwrap()
        .0;
        let chars: Vec<char> = target_ipa.chars().collect();
        println!("ZZ target_ipa = {target_ipa}  n = {}", chars.len());
        for (i, c) in chars.iter().enumerate() {
            println!("ZZ   {i:2} {c}");
        }

        // 1. Is the word in the raw corpus at all, and at what rarity?
        for w in ["dupe", "hits", "justice", "hid", "came", "tup", "group"] {
            let entry = raw.get(w);
            let ipa = corpus.preferred_ipa(w);
            println!(
                "ZZ raw {w:10} rarity = {:?}  ipa = {:?}",
                entry.and_then(|e| e.rarity),
                ipa
            );
        }

        // 2. Does build_lexicon keep it?
        let lex = build_lexicon(json, &corpus, Some(50_000.0));
        println!("ZZ lexicon words = {}", lex.words.len());
        for w in ["dupe", "hits", "justice", "hid", "came"] {
            let idx = lex.words.iter().position(|f| f.word == w);
            println!(
                "ZZ lex {w:10} -> {:?}",
                idx.map(|i| (lex.words[i].ipa.clone(), lex.words[i].rarity))
            );
        }

        // 3. Does matches_at offer it for the span, and where in the list?
        for start in [10usize, 3, 13, 15] {
            let hits = lex.matches_at(&chars, start, 0.5, 1);
            let mut n = 0;
            for (i, m) in hits.iter().enumerate() {
                let f = &lex.words[m.word_idx];
                if f.word == "dupe" {
                    println!(
                        "ZZ span {start}-? out_idx {i} consumed {} cost {:.6} ipa {}",
                        m.consumed, m.cost, f.ipa
                    );
                    n += 1;
                }
            }
            if n == 0 {
                println!("ZZ span {start}-? : dupe ABSENT from matches_at");
            }
            println!(
                "ZZ span {start}-? : matches_at returned {} entries, consumed histogram {:?}",
                hits.len(),
                {
                    let mut h = std::collections::BTreeMap::new();
                    for m in &hits {
                        *h.entry(m.consumed).or_insert(0usize) += 1;
                    }
                    h
                }
            );
        }
    }

    #[test]
    fn indel_trie_respects_budget() {
        let mut best = HashMap::new();
        let mut stack = Vec::new();
        push_state(&mut stack, &mut best, 1, 1, 0.6, 0.5);
        assert!(stack.is_empty());
    }

    #[test]
    fn syllable_count_collapses_diphthongs() {
        for (ipa, expected) in [
            ("hɪts", 1),
            ("dʒʌstəs", 2),
            ("keɪm", 1),
            ("naɪs", 1),
            ("naɪʒ", 1),
            ("ɡoʊ", 1),
            ("baʊt", 1),
            ("bɔɪ", 1),
            ("stʊpəd", 2),
            ("ə", 1),
            ("m", 1),
            ("ɹɛkəɡnaɪzspitʃ", 4),
            ("ɪtsdʒʌstəstʊpədɡeɪm", 6),
            // Hiatus in CMUdict spelling really is two syllables.
            ("əkɹuəl", 3),
        ] {
            assert_eq!(
                ipa_syllables(ipa),
                expected,
                "syllable count for /{ipa}/"
            );
        }
    }
}
