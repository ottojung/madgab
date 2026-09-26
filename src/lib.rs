//! Mad Gab puzzle generator.
//!
//! The generator searches for English word sequences whose connected
//! pronunciation is close to a target phrase while preferring a
//! genuinely different lexical/word-boundary parse.

use std::collections::{HashMap, HashSet};

use phonetics::transcriptions::{Corpus, Pronunciation};
use serde::Serialize;

mod approx;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// One candidate Mad Gab clue.
#[derive(Debug, Clone, Serialize)]
pub struct Clue {
    pub phrase: String,
    pub ipa: String,
    pub words: Vec<ClueWord>,
    /// Composite score in [0, 1] — higher is better.
    pub score: f64,
}

/// One word inside a candidate clue.
#[derive(Debug, Clone, Serialize)]
pub struct ClueWord {
    pub word: String,
    pub ipa: String,
    pub rarity: Option<f64>,
    /// Phonetic edit cost against the target span consumed by this word.
    pub sub_cost: f64,
}

/// Search behavior.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    Exact,
    Approximate {
        per_word_budget: f64,
        total_budget: f64,
    },
}

impl SearchMode {
    pub fn approximate() -> Self {
        Self::Approximate {
            per_word_budget: 0.5,
            total_budget: 1.5,
        }
    }
}

/// Search configuration.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub beam_width: usize,
    pub top_n: usize,
    pub max_rarity: Option<f64>,
    pub mode: SearchMode,
    pub min_word_ipa_chars: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            beam_width: 64,
            top_n: 10,
            max_rarity: Some(50_000.0),
            mode: SearchMode::Exact,
            min_word_ipa_chars: 1,
        }
    }
}

/// A reusable Mad Gab generator.
pub struct Generator {
    corpus: Corpus,
    config: GeneratorConfig,
    /// Iteration-friendly view of the same preferred pronunciations.
    /// The corpus trie remains authoritative for Exact mode.
    fuzzy_lexicon: approx::FuzzyLexicon,
}

impl Generator {
    pub fn from_json(
        json: &str,
        config: GeneratorConfig,
    ) -> Result<Self, phonetics::transcriptions::Error> {
        let corpus = Corpus::from_json(json, config.max_rarity)?;

        // The common/default configuration is rarity-bounded, so this
        // stays around 50k words. Avoid building a second 280k-word view
        // for callers that explicitly request an unfiltered Exact-only
        // corpus (the integration corpus probes do this).
        let fuzzy_lexicon = if config.max_rarity.is_some()
            || matches!(config.mode, SearchMode::Approximate { .. })
        {
            approx::build_lexicon(json, &corpus, config.max_rarity)
        } else {
            approx::FuzzyLexicon::empty()
        };

        Ok(Self {
            corpus,
            config,
            fuzzy_lexicon,
        })
    }

    pub fn corpus(&self) -> &Corpus {
        &self.corpus
    }

    pub fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: GeneratorConfig) {
        self.config = config;
    }

    /// Generate ranked clue candidates for target.
    pub fn generate(&self, target: &str) -> Vec<Clue> {
        match self.config.mode {
            SearchMode::Exact => self.generate_exact(target),
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => self.generate_approximate(target, per_word_budget, total_budget),
        }
    }

    fn generate_exact(&self, target: &str) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_with_boundaries(&self.corpus, target, false)
        else {
            return Vec::new();
        };
        let target_words = target_word_set(target);
        let target_letters = normalized_phrase_letters(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        let mut beam: Vec<Vec<Partial>> = vec![Vec::new(); n + 1];
        beam[0].push(Partial::empty());

        for p in 0..n {
            if beam[p].is_empty() {
                continue;
            }
            let here = std::mem::take(&mut beam[p]);
            for partial in &here {
                for (consumed, pronunciation) in self.corpus.trie.words_starting_at(&chars, p) {
                    if pronunciation.ipa.chars().count() < self.config.min_word_ipa_chars {
                        continue;
                    }
                    let next = partial.extend_pronunciation(pronunciation, consumed, 0.0);
                    insert_top_k(&mut beam[p + consumed], next, self.config.beam_width);
                }
            }
        }

        self.finish(
            std::mem::take(&mut beam[n]),
            &target_ipa,
            &target_boundaries,
            &target_words,
            &target_letters,
        )
    }

    fn generate_approximate(
        &self,
        target: &str,
        per_word_budget: f64,
        total_budget: f64,
    ) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_with_boundaries(&self.corpus, target, true)
        else {
            return Vec::new();
        };
        let target_words = target_word_set(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let (target_letters, ipa_to_letter) =
            project_target_letters(target, &target_boundaries, chars.len());
        let n = chars.len();
        if n == 0 || self.fuzzy_lexicon.is_empty() {
            return Vec::new();
        }

        // Candidate word/span alignments depend only on the target and
        // per-word edit budget, not on a particular beam hypothesis.
        // Build this expensive lattice once.
        let perf_start = std::time::Instant::now();
        let lattice: Vec<Vec<approx::FuzzyMatch>> = (0..n)
            .map(|p| {
                self.fuzzy_lexicon.matches_at(
                    &chars,
                    p,
                    per_word_budget,
                    self.config.min_word_ipa_chars,
                )
            })
            .collect();
        eprintln!("PERF lattice {:?}", perf_start.elapsed());
        // Approximate mode now uses the segmentation-first recovery as
        // its sole phrase search. Running the historical lexical beam
        // first duplicated the same lattice work and dominated runtime
        // on longer phrases without adding unique future information.
        let perf_recovery = std::time::Instant::now();
        let mut completed: Vec<Partial> = Vec::new();

        // Recovery search: decouple segmentation survival from lexical
        // survival.  The ordinary beam is deliberately tight and fast,
        // but a locally mediocre word can otherwise erase an excellent
        // global resegmentation.  First retain a small set of promising
        // target-span structures; only then explore lexical alternatives
        // inside each retained structure.
        #[derive(Clone)]
        struct SpanEdge {
            end: usize,
            matches: Vec<approx::FuzzyMatch>,
            min_cost: f64,
            max_familiarity: f64,
            min_reused: usize,
            max_ipa_len: usize,
        }

        #[derive(Clone)]
        struct SegPath {
            spans: Vec<(usize, usize)>,
            min_cost: f64,
            max_familiarity_sum: f64,
            min_reused: usize,
            max_ipa_len_sum: usize,
            rank: f64,
        }

        const SPAN_AXIS_KEEP: usize = 32;
        const SEG_STATE_KEEP: usize = 8;
        const SEGMENTATION_KEEP: usize = 96;

        let target_inner: HashSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|&b| b < n)
            .collect();

        let mut span_lattice: Vec<Vec<SpanEdge>> = (0..n).map(|_| Vec::new()).collect();

        for p in 0..n {
            let mut grouped: std::collections::BTreeMap<usize, Vec<approx::FuzzyMatch>> =
                std::collections::BTreeMap::new();
            for &m in &lattice[p] {
                let end = p + m.consumed;
                if end <= n {
                    grouped.entry(end).or_default().push(m);
                }
            }

            for (end, matches) in grouped {
                let quality = |m: &approx::FuzzyMatch| {
                    let word = self.fuzzy_lexicon.word(m.word_idx);
                    let familiarity = word_familiarity(word.rarity);
                    let reused = target_words.contains(&normalized_word(&word.word));
                    let letter_start = ipa_to_letter[p];
                    let letter_end = ipa_to_letter[end];
                    let orthographic = orthographic_similarity(
                        &word.word,
                        &target_letters[letter_start..letter_end],
                    );
                    -0.1125 * m.cost + 0.15 * familiarity - if reused { 0.10 } else { 0.0 }
                        + 0.01 * (word.ipa_len.min(8) as f64)
                        + 0.25 * orthographic
                };

                // A span shortlist is a portfolio, not simply the
                // cheapest N words.  This preserves near-homophones that
                // are strong on a different quality axis.
                let mut selected = Vec::new();
                let mut seen_words = HashSet::new();

                let mut by_cost = matches.clone();
                by_cost.sort_by(|a, b| {
                    a.cost
                        .partial_cmp(&b.cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for m in by_cost.iter().take(SPAN_AXIS_KEEP) {
                    if seen_words.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                let mut by_familiarity = matches.clone();
                by_familiarity.sort_by(|a, b| {
                    cmp_desc(
                        word_familiarity(self.fuzzy_lexicon.word(a.word_idx).rarity),
                        word_familiarity(self.fuzzy_lexicon.word(b.word_idx).rarity),
                    )
                });
                for m in by_familiarity.iter().take(SPAN_AXIS_KEEP) {
                    if seen_words.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                let mut by_quality = matches;
                by_quality.sort_by(|a, b| cmp_desc(quality(a), quality(b)));
                for m in by_quality.iter().take(SPAN_AXIS_KEEP) {
                    if seen_words.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }
                selected.sort_by(|a, b| cmp_desc(quality(a), quality(b)));

                if selected.is_empty() {
                    continue;
                }

                let min_cost = selected
                    .iter()
                    .map(|m| m.cost)
                    .fold(f64::INFINITY, f64::min);
                let max_familiarity = selected
                    .iter()
                    .map(|m| word_familiarity(self.fuzzy_lexicon.word(m.word_idx).rarity))
                    .fold(0.0, f64::max);
                let min_reused = usize::from(selected.iter().all(|m| {
                    let word = self.fuzzy_lexicon.word(m.word_idx);
                    target_words.contains(&normalized_word(&word.word))
                }));
                let max_ipa_len = selected
                    .iter()
                    .map(|m| self.fuzzy_lexicon.word(m.word_idx).ipa_len)
                    .max()
                    .unwrap_or(0);

                span_lattice[p].push(SpanEdge {
                    end,
                    matches: selected,
                    min_cost,
                    max_familiarity,
                    min_reused,
                    max_ipa_len,
                });
            }
        }

        // Structural DP.  For a fixed (position, word count, number of
        // shared target boundaries), all future structural possibilities
        // are identical.  Keep only a handful of strongest lexical
        // upper-bound representatives in each such state.
        eprintln!("PERF span_lattice {:?}", perf_recovery.elapsed());
        let perf_struct = std::time::Instant::now();
        let max_words = n.min(
            target_boundaries
                .len()
                .saturating_mul(3)
                .saturating_add(2)
                .max(4),
        );
        let mut seg_states: Vec<HashMap<(usize, usize), Vec<SegPath>>> =
            (0..=n).map(|_| HashMap::new()).collect();
        seg_states[0].insert(
            (0, 0),
            vec![SegPath {
                spans: Vec::new(),
                min_cost: 0.0,
                max_familiarity_sum: 0.0,
                min_reused: 0,
                max_ipa_len_sum: 0,
                rank: 0.0,
            }],
        );

        for p in 0..n {
            let here = std::mem::take(&mut seg_states[p]);
            for ((word_count, shared), paths) in here {
                for path in paths {
                    for edge in &span_lattice[p] {
                        let next_words = word_count + 1;
                        if next_words > max_words
                            || path.min_cost + edge.min_cost > total_budget + 1e-9
                        {
                            continue;
                        }

                        let next_shared =
                            shared + usize::from(edge.end < n && target_inner.contains(&edge.end));
                        let mut spans = path.spans.clone();
                        spans.push((p, edge.end));

                        let min_cost = path.min_cost + edge.min_cost;
                        let max_familiarity_sum = path.max_familiarity_sum + edge.max_familiarity;
                        let min_reused = path.min_reused + edge.min_reused;
                        let max_ipa_len_sum = path.max_ipa_len_sum + edge.max_ipa_len;
                        let denom = next_words as f64;
                        let rank = -0.1125 * min_cost + 0.15 * max_familiarity_sum / denom
                            - 0.10 * min_reused as f64 / denom
                            + 0.05 * (max_ipa_len_sum as f64 / (4.0 * denom)).min(1.0);

                        let bucket = seg_states[edge.end]
                            .entry((next_words, next_shared))
                            .or_default();
                        bucket.push(SegPath {
                            spans,
                            min_cost,
                            max_familiarity_sum,
                            min_reused,
                            max_ipa_len_sum,
                            rank,
                        });
                        bucket.sort_by(|a, b| cmp_desc(a.rank, b.rank));
                        bucket.truncate(SEG_STATE_KEEP);
                    }
                }
            }
        }

        let target_inner_count = target_inner.len();
        let mut segmentations: Vec<(f64, SegPath)> = Vec::new();
        for ((word_count, shared), paths) in std::mem::take(&mut seg_states[n]) {
            if word_count == 0 {
                continue;
            }
            let clue_inner = word_count.saturating_sub(1);
            let union = target_inner_count + clue_inner - shared;
            let novelty = if union == 0 {
                0.0
            } else {
                1.0 - shared as f64 / union as f64
            };
            let denom = word_count as f64;
            for path in paths {
                let upper = 0.45 * (1.0 - path.min_cost / 4.0).clamp(0.0, 1.0)
                    + 0.25 * novelty
                    + 0.10 * (1.0 - path.min_reused as f64 / denom)
                    + 0.15 * path.max_familiarity_sum / denom
                    + 0.05 * (path.max_ipa_len_sum as f64 / (4.0 * denom)).min(1.0);
                segmentations.push((upper, path));
            }
        }
        segmentations.sort_by(|a, b| cmp_desc(a.0, b.0));

        // Keep the final structural shortlist diverse instead of letting
        // one high-scoring segmentation family consume every slot.
        // Paths in the same (word-count, shared-boundaries, cost-band)
        // family have very similar global structure, so select them
        // round-robin across families, preserving score order within
        // each family.
        let mut families: std::collections::BTreeMap<(usize, usize, usize), Vec<(f64, SegPath)>> =
            std::collections::BTreeMap::new();
        for item in segmentations {
            let path = &item.1;
            let word_count = path.spans.len();
            let shared = path
                .spans
                .iter()
                .filter(|(_, end)| *end < n && target_inner.contains(end))
                .count();
            let cost_band = ((path.min_cost / 0.25) + 1e-9).floor() as usize;
            families
                .entry((word_count, shared, cost_band))
                .or_default()
                .push(item);
        }
        for members in families.values_mut() {
            members.sort_by(|a, b| cmp_desc(a.0, b.0));
        }

        let mut segmentations = Vec::with_capacity(SEGMENTATION_KEEP);
        let mut rank = 0usize;
        while segmentations.len() < SEGMENTATION_KEEP {
            let mut added = false;
            for members in families.values() {
                if let Some(item) = members.get(rank) {
                    segmentations.push(item.clone());
                    added = true;
                    if segmentations.len() == SEGMENTATION_KEEP {
                        break;
                    }
                }
            }
            if !added {
                break;
            }
            rank += 1;
        }
        segmentations.sort_by(|a, b| cmp_desc(a.0, b.0));
        eprintln!(
            "DEBUG_SEG classic={:?} recognize={:?}",
            segmentations
                .iter()
                .position(|(_, s)| s.spans == vec![(0, 3), (3, 10), (10, 13), (13, 15), (15, 19)]),
            segmentations
                .iter()
                .position(|(_, s)| s.spans == vec![(0, 3), (3, 4), (4, 10), (10, 14)])
        );

        eprintln!("PERF structural {:?}", perf_struct.elapsed());
        let perf_lexical = std::time::Instant::now();
        let target_stems: Vec<String> = target_words
            .iter()
            .map(|word| stem_word(&normalized_word(word)))
            .collect();

        const DEEP_SEGMENTATIONS: usize = 40;
        const DEEP_BUCKET_KEEP: usize = 32;
        const SHALLOW_BUCKET_KEEP: usize = 8;
        const ALTERNATIVES_PER_SPAN: usize = 48;
        const COST_STEP: f64 = 0.10;

        #[derive(Clone)]
        struct LexState {
            choices: Vec<usize>,
            cost: f64,
            rank: f64,
            last_word: Option<usize>,
            previous_word: Option<usize>,
        }

        let prune_lexical_bucket = |mut states: Vec<LexState>, keep: usize| {
            if states.len() <= keep {
                return states;
            }
            states.sort_by(|a, b| cmp_desc(a.rank, b.rank));

            let mut chosen = HashSet::new();

            let mut best_last: HashMap<usize, usize> = HashMap::new();
            let mut best_pair: HashMap<(usize, usize), usize> = HashMap::new();
            for (index, state) in states.iter().enumerate() {
                if let Some(last) = state.last_word {
                    best_last.entry(last).or_insert(index);
                    if let Some(previous) = state.previous_word {
                        best_pair.entry((previous, last)).or_insert(index);
                    }
                }
            }

            let mut last_reps: Vec<usize> = best_last.into_values().collect();
            last_reps.sort_by(|&a, &b| cmp_desc(states[a].rank, states[b].rank));
            for index in last_reps.into_iter().take(keep / 2) {
                chosen.insert(index);
            }

            let mut pair_reps: Vec<usize> = best_pair.into_values().collect();
            pair_reps.sort_by(|&a, &b| cmp_desc(states[a].rank, states[b].rank));
            for index in pair_reps {
                if chosen.len() >= keep.saturating_mul(3) / 4 {
                    break;
                }
                chosen.insert(index);
            }

            for index in 0..states.len() {
                if chosen.len() >= keep {
                    break;
                }
                chosen.insert(index);
            }

            let mut result: Vec<LexState> = chosen
                .into_iter()
                .map(|index| states[index].clone())
                .collect();
            result.sort_by(|a, b| cmp_desc(a.rank, b.rank));
            result
        };

        let mut recovered = Vec::new();
        for (segmentation_rank, (_, segmentation)) in segmentations.into_iter().enumerate() {
            let diagnostic_classic = vec![(0, 3), (3, 10), (10, 13), (13, 15), (15, 19)];
            if std::env::var_os("MADGAB_CANON_ONLY").is_some()
                && segmentation.spans != diagnostic_classic
            {
                continue;
            }
            let word_count = segmentation.spans.len() as f64;
            let lexical_rank = |m: &approx::FuzzyMatch, start: usize, end: usize| {
                let word = self.fuzzy_lexicon.word(m.word_idx);
                let stem = stem_word(&normalized_word(&word.word));
                let reused = target_stems.iter().any(|target| stems_match(&stem, target));
                let letter_start = ipa_to_letter[start];
                let letter_end = ipa_to_letter[end];
                let orthographic =
                    orthographic_similarity(&word.word, &target_letters[letter_start..letter_end]);
                -0.1125 * m.cost + 0.10 * word_familiarity(word.rarity) / word_count
                    - if reused { 0.08 / word_count } else { 0.0 }
                    + 0.08 * lexical_shape_quality(&word.word) / word_count
                    + 0.15 * orthographic / word_count
            };

            let mut alternatives = Vec::with_capacity(segmentation.spans.len());
            let mut valid = true;
            for &(start, end) in &segmentation.spans {
                let Some(edge) = span_lattice[start].iter().find(|edge| edge.end == end) else {
                    valid = false;
                    break;
                };
                let mut matches = edge.matches.clone();
                matches.sort_by(|a, b| {
                    cmp_desc(lexical_rank(a, start, end), lexical_rank(b, start, end)).then_with(
                        || {
                            self.fuzzy_lexicon
                                .word(a.word_idx)
                                .word
                                .cmp(&self.fuzzy_lexicon.word(b.word_idx).word)
                        },
                    )
                });
                matches.truncate(ALTERNATIVES_PER_SPAN.min(matches.len()));
                alternatives.push(matches);
            }
            if !valid || alternatives.iter().any(Vec::is_empty) {
                continue;
            }

            if segmentation.spans == vec![(0, 3), (3, 10), (10, 13), (13, 15), (15, 19)] {
                let wanted = ["hits", "justice", "dupe", "hid", "came"];
                let ranks: Vec<Option<usize>> = alternatives
                    .iter()
                    .zip(wanted)
                    .map(|(slot, wanted)| {
                        slot.iter()
                            .position(|m| self.fuzzy_lexicon.word(m.word_idx).word == wanted)
                    })
                    .collect();
                eprintln!("ORTHO_CANON ranks={ranks:?}");
            }

            let bucket_count = (total_budget / COST_STEP).ceil() as usize + 1;
            let mut buckets: Vec<Vec<LexState>> = (0..bucket_count).map(|_| Vec::new()).collect();
            buckets[0].push(LexState {
                choices: Vec::with_capacity(segmentation.spans.len()),
                cost: 0.0,
                rank: 0.0,
                last_word: None,
                previous_word: None,
            });

            let bucket_keep = if segmentation_rank < DEEP_SEGMENTATIONS {
                DEEP_BUCKET_KEEP
            } else {
                SHALLOW_BUCKET_KEEP
            };

            for (slot, &(start, end)) in segmentation.spans.iter().enumerate() {
                let mut next_buckets: Vec<Vec<LexState>> =
                    (0..bucket_count).map(|_| Vec::new()).collect();

                for bucket in &buckets {
                    for state in bucket {
                        for (choice, m) in alternatives[slot].iter().enumerate() {
                            let total_cost = state.cost + m.cost;
                            if total_cost > total_budget + 1e-9 {
                                continue;
                            }

                            let cost_bucket = ((total_cost / COST_STEP) + 1e-9).floor() as usize;
                            let cost_bucket = cost_bucket.min(bucket_count - 1);
                            let mut choices = state.choices.clone();
                            choices.push(choice);

                            next_buckets[cost_bucket].push(LexState {
                                choices,
                                cost: total_cost,
                                rank: state.rank + lexical_rank(m, start, end),
                                last_word: Some(m.word_idx),
                                previous_word: state.last_word,
                            });
                        }
                    }
                }

                for bucket in &mut next_buckets {
                    if bucket.len() > bucket_keep {
                        *bucket = prune_lexical_bucket(std::mem::take(bucket), bucket_keep);
                    }
                }
                buckets = next_buckets;
                if segmentation.spans == diagnostic_classic {
                    let wanted = ["hits", "justice", "dupe", "hid", "came"];
                    let survives = buckets.iter().flatten().any(|state| {
                        state.choices.len() == slot + 1
                            && state.choices.iter().enumerate().all(|(i, choice)| {
                                let m = &alternatives[i][*choice];
                                self.fuzzy_lexicon.word(m.word_idx).word == wanted[i]
                            })
                    });
                    let state_count: usize = buckets.iter().map(Vec::len).sum();
                    eprintln!(
                        "CANON_STAGE slot={} survives={} states={}",
                        slot + 1,
                        survives,
                        state_count
                    );
                }
            }

            for bucket in buckets {
                for state in bucket {
                    let mut partial = Partial::empty();
                    for (slot, choice) in state.choices.into_iter().enumerate() {
                        let m = &alternatives[slot][choice];
                        let word = self.fuzzy_lexicon.word(m.word_idx);
                        let (start, end) = segmentation.spans[slot];
                        let orthographic = orthographic_similarity(
                            &word.word,
                            &target_letters[ipa_to_letter[start]..ipa_to_letter[end]],
                        );
                        partial =
                            partial.extend_fuzzy_scored(word, m.consumed, m.cost, orthographic);
                    }
                    recovered.push(partial);
                }
            }
        }

        eprintln!(
            "PERF lexical {:?} recovered={}",
            perf_lexical.elapsed(),
            recovered.len()
        );
        completed.extend(recovered);
        eprintln!(
            "PERF prefinish total={:?} candidates={}",
            perf_start.elapsed(),
            completed.len()
        );
        self.finish(
            completed,
            &target_ipa,
            &target_boundaries,
            &target_words,
            &target_letters,
        )
    }

    fn finish(
        &self,
        completed: Vec<Partial>,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
        target_letters: &[char],
    ) -> Vec<Clue> {
        let perf_finish = std::time::Instant::now();
        let mut clues: Vec<Clue> = completed
            .into_iter()
            .map(|p| p.into_clue(target_ipa, target_boundaries, target_words, target_letters))
            .collect();

        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.phrase.cmp(&b.phrase))
        });

        let mut seen = HashSet::new();
        clues.retain(|c| seen.insert(c.phrase.to_lowercase()));
        for wanted in ["wreck a nice beach", "hits justice dupe hid came"] {
            if let Some(rank) = clues
                .iter()
                .position(|c| c.phrase.eq_ignore_ascii_case(wanted))
            {
                eprintln!(
                    "DEBUG_FINAL raw wanted={wanted:?} rank={rank} score={}",
                    clues[rank].score
                );
            }
        }
        eprintln!(
            "DEBUG_FINAL cutoffs={:?}",
            [0usize, 49, 99, 499]
                .into_iter()
                .filter_map(|i| clues.get(i).map(|c| (i, c.score, c.phrase.as_str())))
                .collect::<Vec<_>>()
        );
        eprintln!(
            "PERF finish presel {:?} candidates={}",
            perf_finish.elapsed(),
            clues.len()
        );
        let perf_select = std::time::Instant::now();
        let selected = select_diverse(clues, self.config.top_n);
        eprintln!("PERF select {:?}", perf_select.elapsed());
        for wanted in ["wreck a nice beach", "hits justice dupe hid came"] {
            eprintln!(
                "DEBUG_FINAL selected wanted={wanted:?} rank={:?}",
                selected
                    .iter()
                    .position(|c| c.phrase.eq_ignore_ascii_case(wanted))
            );
        }
        selected
    }
}

// -----------------------------------------------------------------
// Transcription and scoring
// -----------------------------------------------------------------

fn transcribe_with_boundaries(
    corpus: &Corpus,
    phrase: &str,
    normalize: bool,
) -> Option<(String, Vec<usize>)> {
    let mut out = String::new();
    let mut boundaries = Vec::new();
    for word in phrase.split_whitespace() {
        let key = clean_input_word(word);
        let ipa = corpus.preferred_ipa(&key)?;
        if normalize {
            out.push_str(&approx::normalize_ipa(ipa));
        } else {
            out.push_str(ipa);
        }
        boundaries.push(out.chars().count());
    }
    Some((out, boundaries))
}

fn clean_input_word(word: &str) -> String {
    word.to_lowercase()
        .trim_end_matches(['.', ',', '!', '?', ';', ':'])
        .to_string()
}

fn normalized_word(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn stem_word(normed: &str) -> String {
    let mut s: String = normed
        .chars()
        .map(|c| if c == 'z' { 's' } else { c })
        .collect();
    for suffix in ["ies", "es", "ed", "ing", "s", "d"] {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            if suffix == "ies" {
                s.push('y');
            }
            break;
        }
    }
    s
}

fn stems_match(a: &str, b: &str) -> bool {
    a == b || format!("{a}e") == b || a == format!("{b}e")
}

fn lexical_shape_quality(word: &str) -> f64 {
    let letters: String = word
        .chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(|c| c.to_lowercase())
        .collect();
    if letters.is_empty() {
        return 0.0;
    }
    let has_vowel = letters
        .chars()
        .any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'));
    let mut quality = if letters.len() == 1 {
        if has_vowel {
            1.0
        } else {
            0.2
        }
    } else if letters.len() <= 2 && !has_vowel {
        0.2
    } else {
        1.0
    };

    let apostrophes = word.chars().filter(|&c| c == '\'').count();
    let odd_punctuation = word
        .chars()
        .any(|c| !c.is_alphabetic() && c != '\'' && c != '-');
    if odd_punctuation {
        quality *= 0.25;
    } else if apostrophes > 1 {
        quality *= 0.5;
    }
    quality
}

fn project_target_letters(
    target: &str,
    ipa_boundaries: &[usize],
    total_ipa_len: usize,
) -> (Vec<char>, Vec<usize>) {
    let words: Vec<String> = target.split_whitespace().map(normalized_word).collect();
    let letters: Vec<char> = words.iter().flat_map(|word| word.chars()).collect();
    let mut map = vec![0usize; total_ipa_len + 1];

    let mut ipa_start = 0usize;
    let mut letter_start = 0usize;
    for (word, &ipa_end) in words.iter().zip(ipa_boundaries) {
        let ipa_len = ipa_end.saturating_sub(ipa_start);
        let letter_len = word.chars().count();
        if ipa_len > 0 {
            for offset in 0..=ipa_len {
                let projected = (offset * letter_len + ipa_len / 2) / ipa_len;
                map[ipa_start + offset] = letter_start + projected.min(letter_len);
            }
        }
        ipa_start = ipa_end;
        letter_start += letter_len;
    }

    for slot in &mut map[ipa_start.min(total_ipa_len)..] {
        *slot = letters.len();
    }
    (letters, map)
}

fn orthographic_similarity(word: &str, target_letters: &[char]) -> f64 {
    let word_chars: Vec<char> = normalized_word(word).chars().collect();
    let denominator = word_chars.len().max(target_letters.len());
    if denominator == 0 {
        return 0.0;
    }

    let mut previous: Vec<usize> = (0..=target_letters.len()).collect();
    for (i, &word_char) in word_chars.iter().enumerate() {
        let mut current = Vec::with_capacity(target_letters.len() + 1);
        current.push(i + 1);
        for (j, &target_char) in target_letters.iter().enumerate() {
            let substitution = previous[j] + usize::from(word_char != target_char);
            current.push((current[j] + 1).min(previous[j + 1] + 1).min(substitution));
        }
        previous = current;
    }

    1.0 - previous[target_letters.len()] as f64 / denominator as f64
}

fn normalized_phrase_letters(phrase: &str) -> Vec<char> {
    phrase
        .split_whitespace()
        .flat_map(|word| normalized_word(word).chars().collect::<Vec<_>>())
        .collect()
}

fn target_word_set(target: &str) -> HashSet<String> {
    target.split_whitespace().map(normalized_word).collect()
}

#[derive(Debug, Clone)]
struct Partial {
    words: Vec<ClueWord>,
    sub_cost_total: f64,
    cheap_score: f64,
    orthographic_sum: f64,
    /// Target-stream offsets consumed at clue word boundaries.
    cuts: Vec<usize>,
    /// Stable path key for duplicate suppression in approximate beams.
    key: String,
}

impl Partial {
    fn empty() -> Self {
        Self {
            words: Vec::new(),
            sub_cost_total: 0.0,
            cheap_score: 0.0,
            orthographic_sum: 0.0,
            cuts: Vec::new(),
            key: String::new(),
        }
    }

    fn extend_pronunciation(&self, p: &Pronunciation, consumed: usize, word_sub_cost: f64) -> Self {
        self.extend_parts(&p.word, &p.ipa, p.rarity, consumed, word_sub_cost, 0.0)
    }

    fn extend_fuzzy(&self, word: &approx::FuzzyWord, consumed: usize, word_sub_cost: f64) -> Self {
        self.extend_parts(
            &word.word,
            &word.ipa,
            word.rarity,
            consumed,
            word_sub_cost,
            0.0,
        )
    }

    fn extend_fuzzy_scored(
        &self,
        word: &approx::FuzzyWord,
        consumed: usize,
        word_sub_cost: f64,
        orthographic: f64,
    ) -> Self {
        self.extend_parts(
            &word.word,
            &word.ipa,
            word.rarity,
            consumed,
            word_sub_cost,
            orthographic,
        )
    }

    fn extend_parts(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        consumed: usize,
        word_sub_cost: f64,
        orthographic: f64,
    ) -> Self {
        let len = ipa.chars().count();
        let word_bonus = (len as f64).min(6.0) / 6.0;
        let rarity_penalty = match rarity {
            Some(r) if r > 5_000.0 => -((r / 50_000.0).min(1.0)),
            _ => 0.0,
        };

        let mut words = self.words.clone();
        words.push(ClueWord {
            word: word.to_string(),
            ipa: ipa.to_string(),
            rarity,
            sub_cost: word_sub_cost,
        });

        let mut cuts = self.cuts.clone();
        let end = cuts.last().copied().unwrap_or(0) + consumed;
        cuts.push(end);

        let step_key = format!("{}\u{1f}{}", word.to_lowercase(), ipa);
        let key = if self.key.is_empty() {
            step_key
        } else {
            format!("{} {}", self.key, step_key)
        };

        Self {
            words,
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            cheap_score: self.cheap_score + word_bonus + rarity_penalty - word_sub_cost,
            orthographic_sum: self.orthographic_sum + orthographic,
            cuts,
            key,
        }
    }

    fn metrics(
        &self,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
        total_len: usize,
        partial: bool,
    ) -> Metrics {
        let similarity = (1.0 - self.sub_cost_total / 4.0).clamp(0.0, 1.0);
        let novelty = boundary_novelty(&self.cuts, target_boundaries, total_len, partial);

        let target_stems: HashSet<String> = target_words
            .iter()
            .map(|w| stem_word(&normalized_word(w)))
            .collect();
        let reused = self
            .words
            .iter()
            .filter(|w| {
                let stem = stem_word(&normalized_word(&w.word));
                target_stems.iter().any(|target| stems_match(&stem, target))
            })
            .count() as f64;
        let word_novelty = 1.0 - reused / self.words.len().max(1) as f64;

        let familiarity = if self.words.is_empty() {
            0.0
        } else {
            self.words
                .iter()
                .map(|w| word_familiarity(w.rarity))
                .sum::<f64>()
                / self.words.len() as f64
        };

        let covered = if partial {
            self.cuts.last().copied().unwrap_or(0)
        } else {
            total_len
        };
        let covered_per_word = covered as f64 / self.words.len().max(1) as f64;
        let length_signal = (covered_per_word / 4.0).min(1.0);
        let shape_quality = if self.words.is_empty() {
            0.0
        } else {
            self.words
                .iter()
                .map(|w| lexical_shape_quality(&w.word))
                .sum::<f64>()
                / self.words.len() as f64
        };
        let orthographic = if self.words.is_empty() {
            0.0
        } else {
            self.orthographic_sum / self.words.len() as f64
        };

        let combined = 0.40 * similarity
            + 0.15 * novelty
            + 0.12 * word_novelty
            + 0.10 * familiarity
            + 0.05 * length_signal
            + 0.08 * shape_quality
            + 0.10 * orthographic;

        Metrics {
            combined,
            novelty,
            familiarity,
            word_novelty,
            length_signal,
            shape_quality,
            orthographic,
        }
    }

    fn into_clue(
        self,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
        target_letters: &[char],
    ) -> Clue {
        let total_len = target_ipa.chars().count();
        let metrics = self.metrics(target_boundaries, target_words, total_len, false);
        let normalized_similarity =
            (1.0 - self.sub_cost_total / total_len.max(1) as f64).clamp(0.0, 1.0);
        let clue_spelling = self
            .words
            .iter()
            .map(|word| normalized_word(&word.word))
            .collect::<String>();
        let spelling_novelty = 1.0 - orthographic_similarity(&clue_spelling, target_letters);
        let score = 0.52 * normalized_similarity
            + 0.05 * metrics.novelty
            + 0.08 * metrics.word_novelty
            + 0.06 * metrics.familiarity
            + 0.02 * metrics.length_signal
            + 0.12 * metrics.shape_quality
            + 0.15 * spelling_novelty;
        Clue {
            phrase: self
                .words
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            ipa: target_ipa.to_string(),
            words: self.words,
            score,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Metrics {
    combined: f64,
    novelty: f64,
    familiarity: f64,
    word_novelty: f64,
    length_signal: f64,
    shape_quality: f64,
    orthographic: f64,
}

/// Symmetric segmentation novelty: Jaccard distance between target
/// inner word boundaries and clue inner boundaries.
///
/// The old score only asked which target boundaries disappeared. It
/// therefore gave zero novelty to a useful split that preserved an
/// original boundary (for example splitting one target word into
/// several clue words). Jaccard distance rewards both added and
/// removed boundaries.
fn boundary_novelty(
    cuts: &[usize],
    target_boundaries: &[usize],
    total_len: usize,
    partial: bool,
) -> f64 {
    let covered = cuts.last().copied().unwrap_or(0);
    let target_inner: HashSet<usize> = target_boundaries
        .iter()
        .copied()
        .filter(|&b| b < total_len && (!partial || b <= covered))
        .collect();
    let clue_inner: HashSet<usize> = cuts.iter().copied().filter(|&c| c < total_len).collect();

    if target_inner.is_empty() {
        return if clue_inner.is_empty() { 0.0 } else { 1.0 };
    }
    let shared = target_inner.intersection(&clue_inner).count();
    1.0 - shared as f64 / target_inner.len() as f64
}

fn word_familiarity(rarity: Option<f64>) -> f64 {
    let Some(r) = rarity.filter(|r| r.is_finite() && *r > 0.0) else {
        return 0.0;
    };
    let lo = 100.0_f64.log10();
    let hi = 50_000.0_f64.log10();
    (1.0 - (r.max(1.0).log10() - lo) / (hi - lo)).clamp(0.0, 1.0)
}

// -----------------------------------------------------------------
// Beam retention
// -----------------------------------------------------------------

/// Exact mode keeps its original cheap top-K behavior.
fn insert_top_k(beam: &mut Vec<Partial>, candidate: Partial, k: usize) {
    if k == 0 {
        return;
    }
    if beam.len() < k {
        beam.push(candidate);
        return;
    }
    let (worst_idx, worst_score) = beam
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.cheap_score
                .partial_cmp(&b.cheap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, p)| (i, p.cheap_score))
        .unwrap();
    if candidate.cheap_score > worst_score {
        beam[worst_idx] = candidate;
    }
}

/// Approximate mode keeps a bounded *portfolio* rather than a single
/// scalar top-K. This matters because a locally expensive word can be
/// the key to a globally excellent resegmentation.
///
/// We reserve representatives across segmentation-depth / cost cells,
/// then fill the rest round-robin from independent objective rankings:
/// overall score, boundary novelty, lexical familiarity, phonetic cost,
/// and target-word novelty.
fn prune_partials(
    candidates: Vec<Partial>,
    k: usize,
    target_boundaries: &[usize],
    target_words: &HashSet<String>,
    total_len: usize,
) -> Vec<Partial> {
    if k == 0 || candidates.is_empty() {
        return Vec::new();
    }

    // Exact path duplicates (same words + same pronunciations) can be
    // generated through multiple edit alignments. Keep the better one.
    let mut dedup: HashMap<String, Partial> = HashMap::new();
    for candidate in candidates {
        match dedup.get(&candidate.key) {
            Some(old)
                if old
                    .metrics(target_boundaries, target_words, total_len, true)
                    .combined
                    >= candidate
                        .metrics(target_boundaries, target_words, total_len, true)
                        .combined => {}
            _ => {
                dedup.insert(candidate.key.clone(), candidate);
            }
        }
    }

    let items: Vec<Partial> = dedup.into_values().collect();
    if items.len() <= k {
        return items;
    }

    let metrics: Vec<Metrics> = items
        .iter()
        .map(|p| p.metrics(target_boundaries, target_words, total_len, true))
        .collect();

    let mut selected = HashSet::new();

    // First protect up to two representatives from each structural
    // (word-count, acoustic-cost-band) cell. This prevents the huge
    // family of zero-cost/local optima from erasing every moderately
    // edited resegmentation.
    let mut cells: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (i, p) in items.iter().enumerate() {
        let band = ((p.sub_cost_total / 0.25) + 1e-9).floor() as usize;
        cells
            .entry((p.words.len().min(16), band.min(16)))
            .or_default()
            .push(i);
    }
    let mut protected = Vec::new();
    for members in cells.values_mut() {
        members.sort_by(|&a, &b| {
            metrics[b]
                .combined
                .partial_cmp(&metrics[a].combined)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        protected.extend(members.iter().take(2).copied());
    }
    protected.sort_by(|&a, &b| {
        metrics[b]
            .combined
            .partial_cmp(&metrics[a].combined)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in protected.into_iter().take(k / 2) {
        selected.insert(i);
    }

    let mut orders: Vec<Vec<usize>> = Vec::new();
    let indices: Vec<usize> = (0..items.len()).collect();

    let mut combined = indices.clone();
    combined.sort_by(|&a, &b| cmp_desc(metrics[a].combined, metrics[b].combined));
    orders.push(combined);

    let mut novelty = indices.clone();
    novelty.sort_by(|&a, &b| cmp_desc(metrics[a].novelty, metrics[b].novelty));
    orders.push(novelty);

    let mut familiarity = indices.clone();
    familiarity.sort_by(|&a, &b| cmp_desc(metrics[a].familiarity, metrics[b].familiarity));
    orders.push(familiarity);

    let mut acoustic = indices.clone();
    acoustic.sort_by(|&a, &b| {
        items[a]
            .sub_cost_total
            .partial_cmp(&items[b].sub_cost_total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    orders.push(acoustic);

    let mut lexical = indices;
    lexical.sort_by(|&a, &b| cmp_desc(metrics[a].word_novelty, metrics[b].word_novelty));
    orders.push(lexical);

    let mut rank = 0;
    while selected.len() < k {
        let mut added = false;
        for order in &orders {
            if let Some(&i) = order.get(rank) {
                added |= selected.insert(i);
                if selected.len() == k {
                    break;
                }
            }
        }
        if !added && orders.iter().all(|o| rank >= o.len()) {
            break;
        }
        rank += 1;
    }

    let mut out: Vec<Partial> = selected.into_iter().map(|i| items[i].clone()).collect();
    out.sort_by(|a, b| {
        let am = a.metrics(target_boundaries, target_words, total_len, true);
        let bm = b.metrics(target_boundaries, target_words, total_len, true);
        cmp_desc(am.combined, bm.combined)
    });
    out
}

fn cmp_desc(a: f64, b: f64) -> std::cmp::Ordering {
    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
}

// -----------------------------------------------------------------
// Final proposal diversity
// -----------------------------------------------------------------

const MMR_LAMBDA: f64 = 0.25;

fn select_diverse(clues: Vec<Clue>, top_n: usize) -> Vec<Clue> {
    if top_n == 0 || clues.is_empty() {
        return Vec::new();
    }
    if clues.len() <= top_n {
        return clues;
    }

    let words: Vec<Vec<String>> = clues
        .iter()
        .map(|c| c.phrase.split_whitespace().map(normalized_word).collect())
        .collect();

    let bigrams: Vec<HashSet<String>> = words
        .iter()
        .map(|ws| {
            ws.windows(2)
                .map(|w| format!("{} {}", w[0], w[1]))
                .collect()
        })
        .collect();

    let word_sets: Vec<HashSet<String>> = words
        .iter()
        .map(|ws| ws.iter().cloned().collect())
        .collect();

    // Keep the strongest score-ranked core intact. Diversity is useful
    // for the tail of the proposal set, but should not evict a candidate
    // that already ranks near the top on the generator's actual score.
    let diversity_slots = ((top_n + 9) / 10).max(1).min(top_n);
    let core_len = top_n.saturating_sub(diversity_slots).min(clues.len());

    let mut picked: Vec<usize> = (0..core_len).collect();
    let mut remaining: Vec<usize> = (core_len..clues.len()).collect();
    let mut max_overlap = vec![0.0_f64; clues.len()];

    for &j in &picked {
        for &i in &remaining {
            let word_overlap = directional_overlap(&word_sets[i], &word_sets[j]);
            let bigram_overlap = directional_overlap(&bigrams[i], &bigrams[j]);
            let overlap = 0.5 * word_overlap + 0.5 * bigram_overlap;
            max_overlap[i] = max_overlap[i].max(overlap);
        }
    }

    while picked.len() < top_n && !remaining.is_empty() {
        let mut best_pos = 0;
        let mut best_value = f64::NEG_INFINITY;
        for (pos, &i) in remaining.iter().enumerate() {
            let value = clues[i].score - MMR_LAMBDA * max_overlap[i];
            if value > best_value {
                best_value = value;
                best_pos = pos;
            }
        }

        let chosen = remaining.remove(best_pos);
        picked.push(chosen);

        for &i in &remaining {
            let word_overlap = directional_overlap(&word_sets[i], &word_sets[chosen]);
            let bigram_overlap = directional_overlap(&bigrams[i], &bigrams[chosen]);
            let overlap = 0.5 * word_overlap + 0.5 * bigram_overlap;
            max_overlap[i] = max_overlap[i].max(overlap);
        }
    }

    let mut slots: Vec<Option<Clue>> = clues.into_iter().map(Some).collect();
    picked
        .into_iter()
        .map(|i| slots[i].take().expect("picked once"))
        .collect()
}

fn directional_overlap(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() {
        return 0.0;
    }
    a.intersection(b).count() as f64 / a.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = r#"{
        "cat":  { "rarity": 100, "ipa": { "cmu": "kæt" }, "alt_display": "CAT" },
        "kit":  { "rarity": 200, "ipa": { "cmu": "kɪt" } },
        "at":   { "rarity": 80,  "ipa": { "cmu": "æt" } },
        "ka":   { "rarity": 5000,"ipa": { "cmu": "kæ" } }
    }"#;

    #[test]
    fn transcribes_known_phrase() {
        let g = Generator::from_json(TINY, GeneratorConfig::default()).unwrap();
        assert_eq!(g.corpus().transcribe("cat"), Some("kæt".to_string()));
    }

    #[test]
    fn generates_completions_for_a_tiny_corpus() {
        let cfg = GeneratorConfig {
            min_word_ipa_chars: 2,
            ..GeneratorConfig::default()
        };
        let g = Generator::from_json(TINY, cfg).unwrap();
        let clues = g.generate("cat");
        assert!(!clues.is_empty());
        assert!(clues.iter().any(|c| c.phrase == "cat"));
    }

    #[test]
    fn empty_when_target_word_unknown() {
        let g = Generator::from_json(TINY, GeneratorConfig::default()).unwrap();
        assert!(g.generate("orange").is_empty());
    }

    #[test]
    fn jaccard_boundary_novelty_rewards_added_cuts() {
        let target = vec![9, 14];
        let clue = vec![3, 4, 9, 14];
        let n = boundary_novelty(&clue, &target, 14, false);
        assert!((n - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn diagnostic_scores_acceptance_sequences() {
        use open_english_pronouncing_dictionary::CORPUS_JSON;

        fn score(target: &str, expected: &[&str]) {
            let g = Generator::from_json(
                CORPUS_JSON,
                GeneratorConfig {
                    mode: SearchMode::approximate(),
                    top_n: 50,
                    ..GeneratorConfig::default()
                },
            )
            .unwrap();
            let (ipa, boundaries) = transcribe_with_boundaries(g.corpus(), target, true).unwrap();
            let target_words = target_word_set(target);
            let chars: Vec<char> = ipa.chars().collect();

            let mut states = vec![(0usize, Partial::empty())];
            for &wanted in expected {
                let mut next = Vec::new();
                for (pos, partial) in states {
                    if pos >= chars.len() {
                        continue;
                    }
                    for m in g.fuzzy_lexicon.matches_at(&chars, pos, 0.5, 1) {
                        let word = g.fuzzy_lexicon.word(m.word_idx);
                        if word.word.eq_ignore_ascii_case(wanted)
                            && partial.sub_cost_total + m.cost <= 1.5 + 1e-9
                        {
                            next.push((
                                pos + m.consumed,
                                partial.extend_fuzzy(word, m.consumed, m.cost),
                            ));
                        }
                    }
                }
                states = next;
            }

            let mut full: Vec<_> = states
                .into_iter()
                .filter(|(pos, _)| *pos == chars.len())
                .map(|(_, p)| p)
                .collect();
            full.sort_by(|a, b| {
                let am = a.metrics(&boundaries, &target_words, chars.len(), false);
                let bm = b.metrics(&boundaries, &target_words, chars.len(), false);
                cmp_desc(am.combined, bm.combined)
            });
            for (rank, p) in full.iter().take(8).enumerate() {
                let m = p.metrics(&boundaries, &target_words, chars.len(), false);
                eprintln!(
                    "DIAG_SCORE target={target:?} phrase={:?} alignment_rank={rank} cost={} score={} novelty={} familiarity={} word_novelty={} cuts={:?} words={:?}",
                    expected.join(" "),
                    p.sub_cost_total,
                    m.combined,
                    m.novelty,
                    m.familiarity,
                    m.word_novelty,
                    p.cuts,
                    p.words.iter().map(|w| (&w.word, w.rarity, w.sub_cost)).collect::<Vec<_>>()
                );
            }
        }

        score(
            "It's just a stupid game",
            &["hits", "justice", "dupe", "hid", "came"],
        );
        score("recognize speech", &["wreck", "a", "nice", "beach"]);
    }

    #[test]
    fn acceptance_sequences_are_reachable_in_fuzzy_lattice() {
        use open_english_pronouncing_dictionary::CORPUS_JSON;

        fn check(target: &str, expected: &[&str]) {
            let g = Generator::from_json(
                CORPUS_JSON,
                GeneratorConfig {
                    mode: SearchMode::approximate(),
                    ..GeneratorConfig::default()
                },
            )
            .unwrap();

            let (ipa, _) = transcribe_with_boundaries(g.corpus(), target, true).unwrap();
            let chars: Vec<char> = ipa.chars().collect();

            // Keep every acoustically valid alignment of the required
            // word sequence. This checks matcher/budget reachability
            // independently of phrase-beam pruning.
            let mut states = vec![(0usize, 0.0f64, Vec::new())];
            for &wanted in expected {
                let mut next = Vec::new();
                let mut seen = HashSet::new();

                for (pos, total, history) in &states {
                    if *pos >= chars.len() {
                        continue;
                    }
                    for m in g.fuzzy_lexicon.matches_at(&chars, *pos, 0.5, 1) {
                        let word = g.fuzzy_lexicon.word(m.word_idx);
                        let end = pos + m.consumed;
                        if word.word.eq_ignore_ascii_case(wanted)
                            && total + m.cost <= 1.5 + 1e-9
                            && seen.insert((end, (total + m.cost).to_bits()))
                        {
                            let mut full = history.clone();
                            full.push((*pos, end));
                            next.push((end, total + m.cost, full));
                        }
                    }
                }

                assert!(
                    !next.is_empty(),
                    "{target:?}: required word {wanted:?} has no continuation after states {states:?}"
                );
                states = next;
            }

            eprintln!("TRACE_ACCEPT {target:?} states={states:?}");
            assert!(
                states.iter().any(|(pos, _, _)| *pos == chars.len()),
                "{target:?}: required sequence does not reach target end"
            );
        }

        check(
            "It's just a stupid game",
            &["hits", "justice", "dupe", "hid", "came"],
        );
        check("recognize speech", &["wreck", "a", "nice", "beach"]);
    }
}
