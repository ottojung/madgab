//! Mad Gab puzzle generator.
//!
//! The generator searches for English word sequences whose connected
//! pronunciation is close to a target phrase while preferring a
//! genuinely different lexical/word-boundary parse.

use std::collections::{HashMap, HashSet};

use phonetics::transcriptions::{Corpus, Pronunciation};
use serde::Serialize;

mod approx;
pub mod lexical;

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
        let Some((target_ipa, target_boundaries, target_syllables)) =
            transcribe_with_boundaries(&self.corpus, target, false)
        else {
            return Vec::new();
        };
        let target_phrase = TargetPhrase::new(target);
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

            // The corpus trie is third-party: `Corpus::from_json` inserts
            // pronunciations while iterating a `HashMap`, so the
            // terminations sharing a trie node come out in hash-seed
            // order.  `insert_top_k` keeps the first arrival on an
            // equal-`cheap_score` tie, so that order decides which
            // hypothesis survives the beam — which is why exact mode used
            // to return a different clue set from one process to the next.
            //
            // The walk itself is independent of the beam hypothesis, so it
            // is hoisted out of the inner loop and the sort is paid once
            // per target position.  Two entries that compare equal here
            // (same span, spelling and IPA) extend every partial into the
            // same `Partial`, so the key is total for our purposes.
            let mut options: Vec<(usize, &Pronunciation)> = self
                .corpus
                .trie
                .words_starting_at(&chars, p)
                .filter(|(_, pronunciation)| {
                    pronunciation.ipa.chars().count()
                        >= self.config.min_word_ipa_chars
                })
                .collect();
            options.sort_by(|(consumed_a, a), (consumed_b, b)| {
                consumed_a
                    .cmp(consumed_b)
                    .then_with(|| a.word.cmp(&b.word))
                    .then_with(|| a.ipa.cmp(&b.ipa))
            });

            for partial in &here {
                for &(consumed, pronunciation) in &options {
                    let next = partial.extend_pronunciation(pronunciation, consumed, 0.0);
                    insert_top_k(&mut beam[p + consumed], next, self.config.beam_width);
                }
            }
        }

        self.finish(
            std::mem::take(&mut beam[n]),
            &target_ipa,
            &target_boundaries,
            &target_phrase,
            target_syllables,
        )
    }

    fn generate_approximate(
        &self,
        target: &str,
        per_word_budget: f64,
        total_budget: f64,
    ) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries, target_syllables)) =
            transcribe_with_boundaries(&self.corpus, target, true)
        else {
            return Vec::new();
        };
        let target_phrase = TargetPhrase::new(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 || self.fuzzy_lexicon.is_empty() {
            return Vec::new();
        }

        // Candidate word/span alignments depend only on the target and
        // per-word edit budget, not on a particular beam hypothesis.
        // Build this expensive lattice once.
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

        let mut beam: Vec<Vec<Partial>> = vec![Vec::new(); n + 1];
        beam[0].push(Partial::empty());

        for p in 0..n {
            if beam[p].is_empty() {
                continue;
            }
            let here = prune_partials(
                std::mem::take(&mut beam[p]),
                self.config.beam_width,
                &target_boundaries,
                &target_phrase,
                target_syllables,
                n,
            );

            for partial in &here {
                let remaining = total_budget - partial.sub_cost_total;
                if remaining < -1e-9 {
                    continue;
                }
                for m in &lattice[p] {
                    if m.cost > remaining + 1e-9 {
                        continue;
                    }
                    let word = self.fuzzy_lexicon.word(m.word_idx);
                    let next = partial.extend_fuzzy(word, m.consumed, m.cost);
                    let q = p + m.consumed;
                    if q > n {
                        continue;
                    }
                    beam[q].push(next);

                    // Intermediate hypotheses must stay tight for
                    // interactive search. Completed hypotheses are
                    // different: they will never be expanded again, so
                    // retain a much larger bounded pool and let the real
                    // final scorer + diversity selector decide among
                    // them. Premature completion pruning loses exactly
                    // the globally-good parses beam search is meant to
                    // approximate.
                    let keep = if q == n {
                        self.config
                            .top_n
                            .saturating_mul(128)
                            .max(1024)
                            .min(8192)
                    } else {
                        self.config.beam_width.max(1)
                    };
                    if beam[q].len() > keep.saturating_mul(2) {
                        let reduced = prune_partials(
                            std::mem::take(&mut beam[q]),
                            keep,
                            &target_boundaries,
                            &target_phrase,
                            target_syllables,
                            n,
                        );
                        beam[q] = reduced;
                    }
                }
            }
        }

        let final_keep = self
            .config
            .top_n
            .saturating_mul(128)
            .max(1024)
            .min(8192);
        let mut completed = prune_partials(
            std::mem::take(&mut beam[n]),
            final_keep,
            &target_boundaries,
            &target_phrase,
            target_syllables,
            n,
        );

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
            min_closed: usize,
            min_reused: usize,
            min_syllables: usize,
            max_syllables: usize,
        }

        /// One word available to fill a span, with the score components
        /// it contributes.
        struct SlotAlt {
            match_ref: approx::FuzzyMatch,
            cost: f64,
            reused: usize,
            familiarity: f64,
            closed: usize,
            shape: f64,
            syllables: usize,
        }

        impl SlotAlt {
            /// Additive share of the final score this word brings on its
            /// own.  Only used to order a span's alternatives; the
            /// enumeration itself is scored on the real objective.
            fn contribution(&self, word_count: f64) -> f64 {
                axes::SIMILARITY_PER_WORD * (-self.cost)
                    - axes::WORD_NOVELTY * self.reused as f64 / word_count
                    + axes::FAMILIARITY * self.familiarity / word_count
                    + axes::CLOSED_CLASS
                        * closed_class_penalty(
                            self.closed as f64,
                            word_count,
                        )
                    + axes::SHAPE * self.shape / word_count
            }
        }

        #[derive(Clone)]
        struct SegPath {
            spans: Vec<(usize, usize)>,
            /// Target inner boundaries this segmentation reproduces.
            shared: usize,
            min_cost: f64,
            max_familiarity_sum: f64,
            min_closed_sum: usize,
            min_reused: usize,
            min_syllables_sum: usize,
            max_syllables_sum: usize,
            rank: f64,
        }

        const SPAN_AXIS_KEEP: usize = 16;
        const SPAN_BAND_KEEP: usize = 4;
        const SPAN_RARITY_KEEP: usize = 4;
        const SPAN_SHORTLIST: usize = 160;
        const SEG_STATE_KEEP: usize = 32;
        const SEGMENTATION_KEEP: usize = 256;
        const LEXICAL_COMBINATIONS_PER_SEGMENTATION: usize = 64;
        const LEXICAL_BRANCH_KEEP: usize = 10;
        const LEXICAL_HEAP_POP_LIMIT: usize = 4_000;

        let target_inner: HashSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|&b| b < n)
            .collect();

        let mut span_lattice: Vec<Vec<SpanEdge>> =
            (0..n).map(|_| Vec::new()).collect();

        for p in 0..n {
            let mut grouped: std::collections::BTreeMap<
                usize,
                Vec<approx::FuzzyMatch>,
            > = std::collections::BTreeMap::new();
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
                    let reused =
                        target_phrase.reuse.reuses(&word.word);
                    axes::SIMILARITY_PER_WORD * (-m.cost)
                        + axes::FAMILIARITY * familiarity
                        - if reused { axes::WORD_NOVELTY } else { 0.0 }
                        + axes::SHAPE
                            * lexical_shape_quality(
                                &word.word,
                                familiarity,
                            )
                    + axes::CLOSED_CLASS
                        * closed_class_penalty(
                            f64::from(word.closed),
                            1.0,
                        )
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
                        word_familiarity(
                            self.fuzzy_lexicon.word(a.word_idx).rarity,
                        ),
                        word_familiarity(
                            self.fuzzy_lexicon.word(b.word_idx).rarity,
                        ),
                    )
                });
                for m in by_familiarity.iter().take(SPAN_AXIS_KEEP) {
                    if seen_words.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                // Every other axis above prefers cheap, familiar words,
                // which is precisely the region the exact search already
                // owns.  A resegmentation that needs an uncommon word is
                // then silently unreachable: no axis ever retains it.
                // Reserve explicit slots for the *least* familiar
                // candidates so approximate mode can still reach wordings
                // the common core never produces.
                let mut by_rarity = matches.clone();
                by_rarity.sort_by(|a, b| {
                    rarity_rank(self.fuzzy_lexicon.word(a.word_idx).rarity)
                        .cmp(&rarity_rank(
                            self.fuzzy_lexicon.word(b.word_idx).rarity,
                        ))
                        .then_with(|| {
                            a.cost.partial_cmp(&b.cost).unwrap_or(
                                std::cmp::Ordering::Equal,
                            )
                        })
                });
                for m in by_rarity.iter().take(SPAN_RARITY_KEEP) {
                    if seen_words.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                let mut cost_bands:
                    std::collections::BTreeMap<usize, Vec<approx::FuzzyMatch>> =
                    std::collections::BTreeMap::new();
                let budget_scale = per_word_budget.max(1e-9);
                for &m in &matches {
                    let band = ((m.cost / budget_scale) * 4.0)
                        .floor()
                        .clamp(0.0, 3.0) as usize;
                    cost_bands.entry(band).or_default().push(m);
                }
                for bucket in cost_bands.values() {
                    let mut by_band_quality = bucket.clone();
                    by_band_quality
                        .sort_by(|a, b| cmp_desc(quality(a), quality(b)));
                    for m in by_band_quality.iter().take(SPAN_BAND_KEEP) {
                        if seen_words.insert(m.word_idx) {
                            selected.push(*m);
                        }
                    }

                    let mut by_band_familiarity = bucket.clone();
                    by_band_familiarity.sort_by(|a, b| {
                        cmp_desc(
                            word_familiarity(
                                self.fuzzy_lexicon.word(a.word_idx).rarity,
                            ),
                            word_familiarity(
                                self.fuzzy_lexicon.word(b.word_idx).rarity,
                            ),
                        )
                    });
                    for m in by_band_familiarity.iter().take(SPAN_BAND_KEEP) {
                        if seen_words.insert(m.word_idx) {
                            selected.push(*m);
                        }
                    }

                    let mut by_band_rarity = bucket.clone();
                    by_band_rarity.sort_by(|a, b| {
                        rarity_rank(
                            self.fuzzy_lexicon.word(a.word_idx).rarity,
                        )
                        .cmp(&rarity_rank(
                            self.fuzzy_lexicon.word(b.word_idx).rarity,
                        ))
                        .then_with(|| {
                            a.cost.partial_cmp(&b.cost).unwrap_or(
                                std::cmp::Ordering::Equal,
                            )
                        })
                    });
                    for m in by_band_rarity.iter().take(SPAN_BAND_KEEP) {
                        if seen_words.insert(m.word_idx) {
                            selected.push(*m);
                        }
                    }
                }

                let mut by_quality = matches;
                by_quality.sort_by(|a, b| cmp_desc(quality(a), quality(b)));
                for m in by_quality.iter().take(SPAN_AXIS_KEEP) {
                    if seen_words.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                // The shortlist exists to bound the per-span alternative
                // count, not to re-rank it.  Every axis above favours the
                // cheap-and-familiar corner, so once the enumeration
                // below is exact there is no reason to stop there: fill
                // the remaining budget from the full quality order, so a
                // word that only becomes the right choice in combination
                // with the other spans is actually reachable.
                for m in by_quality {
                    if selected.len() >= SPAN_SHORTLIST {
                        break;
                    }
                    if seen_words.insert(m.word_idx) {
                        selected.push(m);
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
                    .map(|m| {
                        word_familiarity(
                            self.fuzzy_lexicon.word(m.word_idx).rarity,
                        )
                    })
                    .fold(0.0, f64::max);
                // The structural DP can only pick from the shortlist, so
                // the best it may assume for this span is "no
                // closed-class word is forced here", which is achievable
                // exactly when at least one alternative is a content
                // word.  A higher value would be a bound the enumeration
                // below could not reach.
                let min_closed = usize::from(
                    selected
                        .iter()
                        .all(|m| self.fuzzy_lexicon.word(m.word_idx).closed),
                );
                let min_reused = usize::from(selected.iter().all(|m| {
                    let word = self.fuzzy_lexicon.word(m.word_idx);
                    target_phrase
                        .words
                        .contains(&normalized_word(&word.word))
                }));
                // Syllable bookkeeping lets the structural DP judge a
                // resegmentation on the same rhythm axis the final scorer
                // uses, instead of treating a nineteen-syllable shred of
                // the target as equivalent to a well-paced one.
                let mut min_syllables = usize::MAX;
                let mut max_syllables = 0usize;
                for m in &selected {
                    let syllables = self.fuzzy_lexicon.word(m.word_idx).syllables;
                    min_syllables = min_syllables.min(syllables);
                    max_syllables = max_syllables.max(syllables);
                }

                span_lattice[p].push(SpanEdge {
                    end,
                    matches: selected,
                    min_cost,
                    max_familiarity,
                    min_closed,
                    min_reused,
                    min_syllables,
                    max_syllables,
                });
            }
        }

        // Structural DP.  For a fixed (position, word count, number of
        // shared target boundaries), all future structural possibilities
        // are identical.  Keep only a handful of strongest lexical
        // upper-bound representatives in each such state.
        let max_words = n.min(
            target_boundaries
                .len()
                .saturating_mul(3)
                .saturating_add(2)
                .max(4),
        );
        let mut seg_states: Vec<
            HashMap<(usize, usize), Vec<SegPath>>,
        > = (0..=n).map(|_| HashMap::new()).collect();
        seg_states[0].insert(
            (0, 0),
            vec![SegPath {
                spans: Vec::new(),
                shared: 0,
                min_cost: 0.0,
                max_familiarity_sum: 0.0,
                min_closed_sum: 0,
                min_reused: 0,
                min_syllables_sum: 0,
                max_syllables_sum: 0,
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
                            || path.min_cost + edge.min_cost
                                > total_budget + 1e-9
                        {
                            continue;
                        }

                        let next_shared = shared
                            + usize::from(
                                edge.end < n
                                    && target_inner.contains(&edge.end),
                            );
                        let mut spans = path.spans.clone();
                        spans.push((p, edge.end));

                        let min_cost = path.min_cost + edge.min_cost;
                        let max_familiarity_sum =
                            path.max_familiarity_sum
                                + edge.max_familiarity;
                        let min_closed_sum =
                            path.min_closed_sum + edge.min_closed;
                        let min_reused =
                            path.min_reused + edge.min_reused;
                        let min_syllables_sum = path.min_syllables_sum
                            + edge.min_syllables;
                        let max_syllables_sum = path.max_syllables_sum
                            + edge.max_syllables;
                        let denom = next_words as f64;
                        let rank = axes::SIMILARITY_PER_WORD_RANK
                            * (-min_cost)
                            + axes::FAMILIARITY
                                * max_familiarity_sum
                                / denom
                            - axes::WORD_NOVELTY
                                * min_reused as f64
                                / denom
                            + axes::CLOSED_CLASS
                                * closed_class_penalty(
                                    min_closed_sum as f64,
                                    denom,
                                )
                            + axes::RHYTHM
                                * rhythm_match_in(
                                    min_syllables_sum,
                                    max_syllables_sum,
                                    target_syllables,
                                );

                        let bucket = seg_states[edge.end]
                            .entry((next_words, next_shared))
                            .or_default();
                        bucket.push(SegPath {
                            spans,
                            shared: next_shared,
                            min_cost,
                            max_familiarity_sum,
                            min_closed_sum,
                            min_reused,
                            min_syllables_sum,
                            max_syllables_sum,
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
        for ((word_count, shared), paths) in
            std::mem::take(&mut seg_states[n])
        {
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
                let upper = axes::SIMILARITY
                    * (1.0 - path.min_cost / 4.0).clamp(0.0, 1.0)
                    + axes::NOVELTY * novelty
                    + axes::WORD_NOVELTY
                        * (1.0
                            - path.min_reused as f64 / denom)
                    + axes::FAMILIARITY
                        * path.max_familiarity_sum / denom
                    + axes::CLOSED_CLASS
                        * closed_class_penalty(
                            path.min_closed_sum as f64,
                            denom,
                        )
                    + axes::RHYTHM
                        * rhythm_match_in(
                            path.min_syllables_sum,
                            path.max_syllables_sum,
                            target_syllables,
                        )
                    + axes::SHAPE;
                segmentations.push((upper, path));
            }
        }
        segmentations.sort_by(|a, b| cmp_desc(a.0, b.0));
        segmentations.truncate(SEGMENTATION_KEEP);

        #[cfg(not(target_arch = "wasm32"))]
        if let (Ok(span_spec), Ok(word_spec)) = (
            std::env::var("MADGAB_TRACE_SPANS"),
            std::env::var("MADGAB_TRACE_WORDS"),
        ) {
            let spans: Vec<(usize, usize)> = span_spec
                .split(',')
                .filter_map(|part| {
                    let (a, b) = part.split_once('-')?;
                    Some((a.parse().ok()?, b.parse().ok()?))
                })
                .collect();
            let words: Vec<&str> =
                word_spec.split(',').filter(|s| !s.is_empty()).collect();
            let in_range =
                spans.iter().all(|&(start, end)| start < n && end <= n);
            let seg_rank = if in_range {
                segmentations
                    .iter()
                    .position(|(_, path)| path.spans == spans)
            } else {
                None
            };
            eprintln!(
                "MADGAB_TRACE segmentation={spans:?} rank={seg_rank:?}"
            );
            if in_range && spans.len() == words.len() {
                for (slot, (&(start, end), wanted)) in
                    spans.iter().zip(words.iter()).enumerate()
                {
                    let edge = span_lattice[start]
                        .iter()
                        .find(|edge| edge.end == end);
                    let word_rank = edge.and_then(|edge| {
                        edge.matches.iter().position(|m| {
                            self.fuzzy_lexicon
                                .word(m.word_idx)
                                .word
                                .eq_ignore_ascii_case(wanted)
                        })
                    });
                    let word_cost = edge.and_then(|edge| {
                        edge.matches
                            .iter()
                            .find(|m| {
                                self.fuzzy_lexicon
                                    .word(m.word_idx)
                                    .word
                                    .eq_ignore_ascii_case(wanted)
                            })
                            .map(|m| m.cost)
                    });
                    eprintln!(
                        "MADGAB_TRACE span_slot={slot} span={start}-{end} word={wanted:?} rank={word_rank:?} cost={word_cost:?}"
                    );
                }
            }
        }

        // For a fixed segmentation, boundary novelty and word count are
        // fixed, so the only remaining choice is which word fills each
        // span.  Enumerate that Cartesian product exactly, in descending
        // order of the *real* final score, with a bounded best-first
        // search over prefixes.
        //
        // The previous enumeration ranked a cost-dominated proxy, so it
        // only ever explored the corner of the product where every word
        // was independently cheap and familiar.  A resegmentation that
        // is excellent overall but needs one locally expensive word — the
        // normal case for a real Mad Gab answer — was never reachable.
        let mut recovered = Vec::new();
        for (_, segmentation) in segmentations {
            let word_count = segmentation.spans.len().max(1) as f64;
            let mut slots: Vec<Vec<SlotAlt>> =
                Vec::with_capacity(segmentation.spans.len());
            let mut possible = true;

            for &(start, end) in &segmentation.spans {
                let Some(edge) = span_lattice[start]
                    .iter()
                    .find(|edge| edge.end == end)
                else {
                    possible = false;
                    break;
                };

                let mut alts: Vec<SlotAlt> = edge
                    .matches
                    .iter()
                    .map(|&m| {
                        let word = self.fuzzy_lexicon.word(m.word_idx);
                        let familiarity = word_familiarity(word.rarity);
                        SlotAlt {
                            match_ref: m,
                            cost: m.cost,
                            reused: usize::from(
                                target_phrase.reuse.reuses(&word.word),
                            ),
                            familiarity,
                            closed: usize::from(word.closed),
                            shape: lexical_shape_quality(
                                &word.word,
                                familiarity,
                            ),
                            syllables: word.syllables,
                        }
                    })
                    .collect();
                alts.sort_by(|a, b| {
                    cmp_desc(
                        a.contribution(word_count),
                        b.contribution(word_count),
                    )
                    .then_with(|| {
                        self.fuzzy_lexicon
                            .word(a.match_ref.word_idx)
                            .word
                            .cmp(&self.fuzzy_lexicon.word(b.match_ref.word_idx).word)
                    })
                });
                slots.push(alts);
            }

            if !possible || slots.iter().any(Vec::is_empty) {
                continue;
            }

            // Suffix bounds make the best-first key an admissible upper
            // bound on the score of any completion of a prefix, so the
            // emission order really is descending in final score.
            let depth = slots.len();
            let mut suf_min_cost = vec![0.0_f64; depth + 1];
            let mut suf_min_reused = vec![0usize; depth + 1];
            let mut suf_max_fam = vec![0.0_f64; depth + 1];
            let mut suf_min_closed = vec![0usize; depth + 1];
            let mut suf_max_shape = vec![0.0_f64; depth + 1];
            let mut suf_min_syl = vec![0usize; depth + 1];
            let mut suf_max_syl = vec![0usize; depth + 1];
            for k in (0..depth).rev() {
                let here = &slots[k];
                suf_min_cost[k] = suf_min_cost[k + 1]
                    + here.iter().map(|a| a.cost).fold(f64::INFINITY, f64::min);
                suf_min_reused[k] = suf_min_reused[k + 1]
                    + usize::from(here.iter().all(|a| a.reused == 1));
                suf_max_fam[k] = suf_max_fam[k + 1]
                    + here
                        .iter()
                        .map(|a| a.familiarity)
                        .fold(0.0_f64, f64::max);
                suf_min_closed[k] = suf_min_closed[k + 1]
                    + usize::from(here.iter().all(|a| a.closed == 1));
                suf_max_shape[k] = suf_max_shape[k + 1]
                    + here.iter().map(|a| a.shape).fold(0.0_f64, f64::max);
                suf_min_syl[k] = suf_min_syl[k + 1]
                    + here.iter().map(|a| a.syllables).min().unwrap_or(0);
                suf_max_syl[k] = suf_max_syl[k + 1]
                    + here.iter().map(|a| a.syllables).max().unwrap_or(0);
            }

            let clue_inner = depth.saturating_sub(1);
            let union = target_inner_count + clue_inner - segmentation.shared;
            let novelty = if union == 0 {
                0.0
            } else {
                1.0 - segmentation.shared as f64 / union as f64
            };

            let bound = |prefix: &[usize]| -> f64 {
                let (mut cost, mut reused, mut fam, mut closed, mut shape, mut syl) =
                    (0.0_f64, 0usize, 0.0_f64, 0usize, 0.0_f64, 0usize);
                for (k, &i) in prefix.iter().enumerate() {
                    let a = &slots[k][i];
                    cost += a.cost;
                    reused += a.reused;
                    fam += a.familiarity;
                    closed += a.closed;
                    shape += a.shape;
                    syl += a.syllables;
                }
                let k = prefix.len();
                axes::SIMILARITY
                    * (1.0 - (cost + suf_min_cost[k]) / 4.0).clamp(0.0, 1.0)
                    + axes::NOVELTY * novelty
                    + axes::WORD_NOVELTY
                        * (1.0
                            - (reused + suf_min_reused[k]) as f64 / word_count)
                    + axes::FAMILIARITY
                        * (fam + suf_max_fam[k]) / word_count
                    + axes::CLOSED_CLASS
                        * closed_class_penalty(
                            (closed + suf_min_closed[k]) as f64,
                            word_count,
                        )
                    + axes::RHYTHM
                        * rhythm_match_in(
                            syl + suf_min_syl[k],
                            syl + suf_max_syl[k],
                            target_syllables,
                        )
                    + axes::SHAPE * (shape + suf_max_shape[k]) / word_count
            };

            let quantized =
                |score: f64| -> i64 { (score * 1_000_000_000.0).round() as i64 };
            let mut heap = std::collections::BinaryHeap::new();
            heap.push((quantized(bound(&[])), 0usize, Vec::<usize>::new()));
            let mut seen: HashSet<(usize, Vec<usize>)> = HashSet::new();
            seen.insert((0, Vec::new()));

            let mut emitted = 0usize;
            let mut popped = 0usize;
            while let Some((_key, k, prefix)) = heap.pop() {
                popped += 1;
                if k == depth {
                    let total_cost: f64 = prefix
                        .iter()
                        .enumerate()
                        .map(|(slot, &i)| slots[slot][i].cost)
                        .sum();
                    if total_cost <= total_budget + 1e-9 {
                        let mut partial = Partial::empty();
                        for (slot, &i) in prefix.iter().enumerate() {
                            let a = &slots[slot][i];
                            let word =
                                self.fuzzy_lexicon.word(a.match_ref.word_idx);
                            partial = partial.extend_fuzzy(
                                word,
                                a.match_ref.consumed,
                                a.cost,
                            );
                        }
                        recovered.push(partial);
                        emitted += 1;
                        if emitted >= LEXICAL_COMBINATIONS_PER_SEGMENTATION {
                            break;
                        }
                    }
                    continue;
                }

                if popped >= LEXICAL_HEAP_POP_LIMIT {
                    break;
                }

                for i in 0..slots[k].len().min(LEXICAL_BRANCH_KEEP) {
                    let mut next = prefix.clone();
                    next.push(i);
                    if seen.insert((k + 1, next.clone())) {
                        heap.push((quantized(bound(&next)), k + 1, next));
                    }
                }
            }
        }

        completed.extend(recovered);
        self.finish(
            completed,
            &target_ipa,
            &target_boundaries,
            &target_phrase,
            target_syllables,
        )
    }

    fn finish(
        &self,
        completed: Vec<Partial>,
        target_ipa: &str,
        target_boundaries: &[usize],
        target: &TargetPhrase,
        target_syllables: usize,
    ) -> Vec<Clue> {
        let mut clues: Vec<Clue> = completed
            .into_iter()
            .map(|p| {
                p.into_clue(
                    target_ipa,
                    target_boundaries,
                    target,
                    target_syllables,
                )
            })
            .collect();

        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.phrase.cmp(&b.phrase))
        });

        // Two spellings of one clue ("this peach" / "this' peach") are
        // one proposal.  Deduplicating on the raw phrase let a single
        // resegmentation occupy most of the result list and crowded out
        // genuinely different ones.
        let mut seen = HashSet::new();
        clues.retain(|c| seen.insert(phrase_signature(&c.phrase)));

        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(wanted) = std::env::var("MADGAB_TRACE_PHRASES") {
            for phrase in wanted.split('|').filter(|s| !s.is_empty()) {
                match clues
                    .iter()
                    .position(|c| c.phrase.eq_ignore_ascii_case(phrase))
                {
                    Some(rank) => eprintln!(
                        "MADGAB_TRACE raw phrase={phrase:?} rank={rank} score={:.9}",
                        clues[rank].score
                    ),
                    None => eprintln!(
                        "MADGAB_TRACE raw phrase={phrase:?} missing candidates={}",
                        clues.len()
                    ),
                }
            }
            if let Some(cutoff) = clues.get(self.config.top_n.saturating_sub(1)) {
                eprintln!(
                    "MADGAB_TRACE raw_cutoff rank={} score={:.9} phrase={:?}",
                    self.config.top_n.saturating_sub(1),
                    cutoff.score,
                    cutoff.phrase
                );
            }
        }

        select_diverse(clues, self.config.top_n)
    }
}

// -----------------------------------------------------------------
// Transcription and scoring
// -----------------------------------------------------------------

fn transcribe_with_boundaries(
    corpus: &Corpus,
    phrase: &str,
    normalize: bool,
) -> Option<(String, Vec<usize>, usize)> {
    let mut out = String::new();
    let mut boundaries = Vec::new();
    let mut syllables = 0usize;
    for word in phrase.split_whitespace() {
        let key = clean_input_word(word);
        let ipa = corpus.preferred_ipa(&key)?;
        if normalize {
            out.push_str(&approx::normalize_ipa(ipa));
        } else {
            out.push_str(ipa);
        }
        syllables += approx::ipa_syllables(&approx::normalize_ipa(ipa));
        boundaries.push(out.chars().count());
    }
    Some((out, boundaries, syllables))
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

fn novelty_stem(word: &str) -> String {
    let mut s = normalized_word(word);
    for (from, to) in [
        ("isation", "ization"),
        ("ising", "izing"),
        ("ised", "ized"),
        ("ises", "izes"),
        ("ise", "ize"),
        ("yse", "yze"),
    ] {
        if s.len() > from.len() + 2 && s.ends_with(from) {
            s.truncate(s.len() - from.len());
            s.push_str(to);
            break;
        }
    }
    for suffix in ["ing", "ies", "ed", "es", "s", "d"] {
        if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
            if suffix == "ies" {
                s.push('y');
            }
            break;
        }
    }
    if s.len() > 4 && s.ends_with('e') {
        s.pop();
    }
    s
}

fn lexical_shape_quality(word: &str, familiarity: f64) -> f64 {
    let w = normalized_word(word);
    match w.chars().count() {
        0 => 0.0,
        1 if w == "a" || w == "i" => 1.0,
        1 => 0.05,
        2 => 0.35 + 0.55 * familiarity,
        _ => 1.0,
    }
}

/// The target phrase, preprocessed once per search.
#[derive(Debug)]
struct TargetPhrase {
    /// Target words in order, normalized.
    words: Vec<String>,
    /// Lexical-family reuse test against [`Self::words`].
    reuse: ReuseIndex,
}

impl TargetPhrase {
    fn new(target: &str) -> Self {
        let words: Vec<String> =
            target.split_whitespace().map(normalized_word).collect();
        let stems: HashSet<String> =
            words.iter().map(|w| novelty_stem(w)).collect();
        Self { words, reuse: ReuseIndex::new(stems) }
    }
}

/// Cached lexical-family reuse test.
///
/// The reuse test normalizes and stems two strings, and approximate mode
/// evaluates it once per clue word for every candidate in every beam
/// comparison.  The target stem set is tiny and fixed for a whole
/// search, so it is precomputed once and per-word results are memoized
/// behind an interior-mutability cell.
#[derive(Debug)]
struct ReuseIndex {
    target_stems: HashSet<String>,
    memo: std::cell::RefCell<HashMap<String, bool>>,
}

impl ReuseIndex {
    fn new(target_stems: HashSet<String>) -> Self {
        Self {
            target_stems,
            memo: std::cell::RefCell::new(HashMap::new()),
        }
    }

    fn reuses(&self, word: &str) -> bool {
        if let Some(&hit) = self.memo.borrow().get(word) {
            return hit;
        }
        let hit = self.target_stems.contains(&novelty_stem(word));
        self.memo.borrow_mut().insert(word.to_string(), hit);
        hit
    }
}

#[derive(Debug, Clone)]
struct Partial {
    words: Vec<ClueWord>,
    sub_cost_total: f64,
    /// Cached syllable total; `metrics` runs in every beam comparison.
    syllables: usize,
    /// Cached closed-class word count; `metrics` runs in every beam
    /// comparison and the test allocates, so it is not re-done there.
    closed: usize,
    cheap_score: f64,
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
            syllables: 0,
            closed: 0,
            cheap_score: 0.0,
            cuts: Vec::new(),
            key: String::new(),
        }
    }

    fn extend_pronunciation(
        &self,
        p: &Pronunciation,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        self.extend_parts(
            &p.word,
            &p.ipa,
            p.rarity,
            lexical::is_closed_class(&p.word),
            consumed,
            word_sub_cost,
        )
    }

    fn extend_fuzzy(
        &self,
        word: &approx::FuzzyWord,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        self.extend_parts(
            &word.word,
            &word.ipa,
            word.rarity,
            word.closed,
            consumed,
            word_sub_cost,
        )
    }

    fn extend_parts(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        closed: bool,
        consumed: usize,
        word_sub_cost: f64,
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
            syllables: self.syllables + approx::ipa_syllables(ipa),
            closed: self.closed + usize::from(closed),
            cheap_score: self.cheap_score + word_bonus + rarity_penalty - word_sub_cost,
            cuts,
            key,
        }
    }

    fn metrics(
        &self,
        target: &TargetPhrase,
        target_boundaries: &[usize],
        target_syllables: usize,
        total_len: usize,
        partial: bool,
    ) -> Metrics {
        let similarity = (1.0 - self.sub_cost_total / 4.0).clamp(0.0, 1.0);
        let novelty =
            boundary_novelty(&self.cuts, target_boundaries, total_len, partial);

        let reused = self
            .words
            .iter()
            .filter(|w| target.reuse.reuses(&w.word))
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

        // A Mad Gab clue has to be *sayable* with the target's rhythm, not
        // merely built from similar phones.  Without this axis the search
        // happily "wins" by shredding the target into one- and two-phone
        // words: that matches acoustically, but it is not a resegmentation
        // anybody can say out loud.
        let rhythm = rhythm_match(self.syllables, target_syllables);

        let shape_quality = if self.words.is_empty() {
            0.0
        } else {
            self.words
                .iter()
                .map(|w| {
                    let familiarity = word_familiarity(w.rarity);
                    lexical_shape_quality(&w.word, familiarity)
                })
                .sum::<f64>()
                / self.words.len() as f64
        };

        // A Mad Gab clue is a *puzzle answer*, so it also has to be
        // readable.  Every other lexical axis here is a function of
        // frequency or phone content, and frequency points the wrong
        // way: `the`, `a`, `it` and `each` are among the commonest words
        // in English, so the familiarity axis rewards precisely the
        // determiner salad no human would use as an answer.  This is the
        // one axis that asks which *class* of word was used, and it is
        // close to anti-correlated with `familiarity` by construction.
        let content = if self.words.is_empty() {
            0.0
        } else {
            1.0 - self.closed as f64 / self.words.len() as f64
        };
        let closed_penalty = closed_class_penalty(
            self.closed as f64,
            self.words.len() as f64,
        );

        let combined = axes::SIMILARITY * similarity
            + axes::NOVELTY * novelty
            + axes::WORD_NOVELTY * word_novelty
            + axes::FAMILIARITY * familiarity
            + axes::RHYTHM * rhythm
            + axes::SHAPE * shape_quality
            + axes::CLOSED_CLASS * closed_penalty;

        Metrics {
            combined,
            novelty,
            familiarity,
            word_novelty,
            rhythm,
            content,
        }
    }

    fn into_clue(
        self,
        target_ipa: &str,
        target_boundaries: &[usize],
        target: &TargetPhrase,
        target_syllables: usize,
    ) -> Clue {
        let total_len = target_ipa.chars().count();
        let score = self
            .metrics(target, target_boundaries, target_syllables, total_len, false)
            .combined;
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

/// Penalty, per unit of closed-class word share, that the clue score
/// applies.  See [`lexical`] for what "closed class" means here and why
/// it is not already covered by the other axes.
///
/// The share is squared by [`closed_class_penalty`] first, so this is the
/// penalty for a clue that is *nothing but* function words; a clue with
/// one function word in four pays a twentieth of it.  That convexity is
/// the point: a determiner inside an otherwise ordinary phrase is
/// idiomatic English and must stay cheap, while a clue that is mostly
/// determiners and pronouns is a word salad and must not.
const CLOSED_CLASS_WEIGHT: f64 = 0.15;

/// The clue score's axes in one place, so the final score and every
/// internal proxy that shadows part of it stay in step.
///
/// `CLOSED_CLASS` is signed and *subtracted*: a clue whose words are all
/// content words pays nothing.
mod axes {
    /// Phonetic similarity of the clue's word sequence to the target.
    pub const SIMILARITY: f64 = 0.25;
    /// Boundary novelty against the target's own word boundaries.
    pub const NOVELTY: f64 = 0.15;
    /// Fraction of clue words that are not a target word.
    pub const WORD_NOVELTY: f64 = 0.15;
    /// Mean per-word corpus familiarity.
    pub const FAMILIARITY: f64 = 0.10;
    /// Agreement between clue and target syllable counts.
    pub const RHYTHM: f64 = 0.30;
    /// Per-word orthographic shape.
    pub const SHAPE: f64 = 0.05;
    /// Closed-class (function) word share, subtracted.  The share is
    /// squared by `closed_class_penalty` before it gets here.
    pub const CLOSED_CLASS: f64 = -super::CLOSED_CLASS_WEIGHT;

    /// Per-word share of the similarity axis, used by the single-word
    /// ranking proxies in `generate_approximate` (`quality`,
    /// `SlotAlt::contribution`): a one-word clue pays the full axis, and
    /// its edit cost is divided by that axis's own normaliser.
    pub const SIMILARITY_PER_WORD: f64 = SIMILARITY / 4.0;

    /// Acoustic-cost coefficient of the structural DP's `rank`.  The DP
    /// only ever compares candidate *segmentations*, and it must not let
    /// a cheap-but-poorly-fitting segmentation outrank an excellent one
    /// that happens to need an edited word, so it deliberately weights
    /// cost more heavily than the final score does.
    pub const SIMILARITY_PER_WORD_RANK: f64 = 0.1125;
}

#[derive(Debug, Clone, Copy)]
struct Metrics {
    combined: f64,
    novelty: f64,
    familiarity: f64,
    word_novelty: f64,
    rhythm: f64,
    /// Share of the clue's words that are content words, in [0, 1].
    content: f64,
}

/// Symmetric segmentation novelty: Jaccard distance between the
/// target's inner word boundaries and the clue's inner word boundaries.
///
/// Counting only the boundaries that disappeared would give zero novelty
/// to a useful split that preserved an original boundary, so the measure
/// is a Jaccard distance and rewards both added and removed boundaries.
///
/// Both inputs are ascending runs of distinct offsets, so union and
/// intersection are counted by merge.  This runs once per candidate in
/// every beam comparison, where a set-based implementation dominated the
/// whole approximate search.
fn boundary_novelty(
    cuts: &[usize],
    target_boundaries: &[usize],
    total_len: usize,
    partial: bool,
) -> f64 {
    let covered = cuts.last().copied().unwrap_or(0);

    let mut a = cuts.iter().copied().filter(|&c| c < total_len);
    let mut b = target_boundaries
        .iter()
        .copied()
        .filter(|&x| x < total_len && (!partial || x <= covered));

    let mut shared = 0usize;
    let mut union = 0usize;
    let mut next_a = a.next();
    let mut next_b = b.next();
    while let (Some(x), Some(y)) = (next_a, next_b) {
        union += 1;
        if x == y {
            shared += 1;
            next_a = a.next();
            next_b = b.next();
        } else if x < y {
            next_a = a.next();
        } else {
            next_b = b.next();
        }
    }
    union += usize::from(next_a.is_some()) + a.count();
    union += usize::from(next_b.is_some()) + b.count();

    if union == 0 {
        return 0.0;
    }
    1.0 - shared as f64 / union as f64
}

/// The content-word penalty, given `closed` closed-class words out of
/// `words` total.
///
/// Readability is a threshold phenomenon, not a linear one, so the
/// penalty is convex in the closed-class share.  One function word in a
/// four-word clue is ordinary English; three function words in a
/// five-word clue is not a phrase anybody would use.  A linear penalty
/// charges the idiomatic clue exactly as much as the salad, which forces
/// the weight low enough to be useless; squaring the share separates the
/// two cases by more than a factor of four at the same weight.
///
/// Squaring also keeps the axis cheap to bound from below.  Every
/// internal proxy that only knows the *minimum* closed-class count a
/// span could be filled with stays a valid lower bound on the penalty
/// after the square, which is not true of a concave or linear map.
fn closed_class_penalty(closed: f64, words: f64) -> f64 {
    if words <= 0.0 {
        return 0.0;
    }
    let share = (closed / words).clamp(0.0, 1.0);
    share * share
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
/// We reserve representatives across structural
/// (word-count, acoustic-cost-band, rhythm-band) cells, then fill the
/// rest round-robin from independent objective rankings: overall score,
/// boundary novelty, lexical familiarity, phonetic cost, target-word
/// novelty, rhythmic agreement, and content-word share.
fn prune_partials(
    candidates: Vec<Partial>,
    k: usize,
    target_boundaries: &[usize],
    target: &TargetPhrase,
    target_syllables: usize,
    total_len: usize,
) -> Vec<Partial> {
    if k == 0 || candidates.is_empty() {
        return Vec::new();
    }

    let score_of = |p: &Partial| {
        p.metrics(
            target,
            target_boundaries,
            target_syllables,
            total_len,
            true,
        )
        .combined
    };

    // Exact path duplicates (same words + same pronunciations) can be
    // generated through multiple edit alignments. Keep the better one.
    let mut dedup: HashMap<String, Partial> = HashMap::new();
    for candidate in candidates {
        match dedup.get(&candidate.key) {
            Some(old) if score_of(old) >= score_of(&candidate) => {}
            _ => {
                dedup.insert(candidate.key.clone(), candidate);
            }
        }
    }

    // HashMap iteration order varies per process; sorting by the path key
    // keeps beam retention (and therefore the whole search) reproducible.
    let mut items: Vec<Partial> = dedup.into_values().collect();
    items.sort_by(|a, b| a.key.cmp(&b.key));
    if items.len() <= k {
        return items;
    }

    let metrics: Vec<Metrics> = items
        .iter()
        .map(|p| {
            p.metrics(
                target,
                target_boundaries,
                target_syllables,
                total_len,
                true,
            )
        })
        .collect();

    let mut selected = HashSet::new();

    // First protect up to two representatives from each structural
    // (word-count, acoustic-cost-band, rhythm-band) cell. This prevents
    // the huge family of zero-cost/local optima from erasing every
    // moderately edited resegmentation.
    let mut cells: HashMap<(usize, usize, usize), Vec<usize>> = HashMap::new();
    for (i, p) in items.iter().enumerate() {
        let band = ((p.sub_cost_total / 0.25) + 1e-9).floor() as usize;
        let rhythm_band =
            ((1.0 - metrics[i].rhythm) * 4.0 + 1e-9).floor().min(4.0) as usize;
        cells
            .entry((p.words.len().min(16), band.min(16), rhythm_band))
            .or_default()
            .push(i);
    }
    let mut cell_keys: Vec<(usize, usize, usize)> =
        cells.keys().copied().collect();
    cell_keys.sort_unstable();
    let mut protected = Vec::new();
    for key in cell_keys {
        let members = cells.get_mut(&key).expect("key came from cells");
        members.sort_by(|&a, &b| {
            cmp_desc(metrics[a].combined, metrics[b].combined).then(a.cmp(&b))
        });
        protected.extend(members.iter().take(2).copied());
    }
    protected.sort_by(|&a, &b| {
        cmp_desc(metrics[a].combined, metrics[b].combined).then(a.cmp(&b))
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

    let mut lexical = indices.clone();
    lexical.sort_by(|&a, &b| cmp_desc(metrics[a].word_novelty, metrics[b].word_novelty));
    orders.push(lexical);

    let mut rhythm = indices.clone();
    rhythm.sort_by(|&a, &b| cmp_desc(metrics[a].rhythm, metrics[b].rhythm));
    orders.push(rhythm);

    // Content words first.  This is the only order in the list that runs
    // *against* the familiarity order, which is why it is listed
    // separately rather than folded into it: beam retention keeps a
    // portfolio, and without a slot reserved for the axis the portfolio
    // only ever contains the cheap-and-common corner that every other
    // order already covers.
    let mut content = indices;
    content.sort_by(|&a, &b| cmp_desc(metrics[a].content, metrics[b].content));
    orders.push(content);

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

    let mut out: Vec<Partial> = selected
        .into_iter()
        .map(|i| items[i].clone())
        .collect();
    out.sort_by(|a, b| {
        cmp_desc(
            score_of(a),
            score_of(b),
        )
    });
    out
}

/// Canonical identity of a clue: its words without case or punctuation.
/// Orthographic variants of the same spoken clue collapse onto one key.
fn phrase_signature(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .map(normalized_word)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Sort key ordering candidates from rarest to most common.
fn rarity_rank(rarity: Option<f64>) -> u64 {
    match rarity {
        Some(r) if r.is_finite() && r >= 0.0 => r.round() as u64,
        _ => u64::MAX,
    }
}

fn cmp_desc(a: f64, b: f64) -> std::cmp::Ordering {
    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
}

/// Agreement between a clue's syllable count and the target's.  One
/// extra or missing syllable already costs half the axis; a
/// two-syllable error costs all of it.  `lo..=hi` is the reachable range
/// of syllable totals, which lets the structural search use the same
/// function as an upper bound.
fn rhythm_match_in(lo: usize, hi: usize, target_syllables: usize) -> f64 {
    let closest = if target_syllables < lo {
        lo - target_syllables
    } else if target_syllables > hi {
        target_syllables - hi
    } else {
        0
    };
    (1.0 - (closest as f64 / 2.0).min(1.0)).max(0.0)
}

fn rhythm_match(clue_syllables: usize, target_syllables: usize) -> f64 {
    rhythm_match_in(clue_syllables, clue_syllables, target_syllables)
}

// -----------------------------------------------------------------
// Final proposal diversity
// -----------------------------------------------------------------

/// Trade-off between raw score and novelty of the proposal set.
const MMR_LAMBDA: f64 = 0.35;

/// Pick `top_n` proposals, trading score against redundancy with the
/// proposals already chosen.
///
/// The penalty used to be measured in the score spread *inside the
/// requested top-N* — a few thousandths for any real search — so
/// maximal redundancy was effectively free and the "diverse" set was
/// fifty spellings of one clue.  Two things are fixed here: the penalty
/// is measured against the spread of the whole candidate pool, and
/// redundancy is the maximum over everything picked so far rather than
/// only the previous pick.  A clue is redundant when it repeats another
/// proposal's words *or* its word boundaries; the latter matters most,
/// because two spellings of one resegmentation are the same puzzle.
fn select_diverse(clues: Vec<Clue>, top_n: usize) -> Vec<Clue> {
    if top_n == 0 || clues.is_empty() {
        return Vec::new();
    }
    if clues.len() <= top_n {
        return clues;
    }

    let words: Vec<Vec<String>> = clues
        .iter()
        .map(|c| {
            c.phrase
                .split_whitespace()
                .map(normalized_word)
                .collect()
        })
        .collect();

    let bigrams: Vec<HashSet<String>> = words
        .iter()
        .map(|ws| {
            ws.windows(2)
                .map(|w| format!("{} {}", w[0], w[1]))
                .collect()
        })
        .collect();

    let word_sets: Vec<HashSet<String>> =
        words.iter().map(|ws| ws.iter().cloned().collect()).collect();

    let boundaries: Vec<HashSet<usize>> = clues
        .iter()
        .map(|c| {
            let mut cuts: Vec<usize> = Vec::new();
            let mut at = 0usize;
            for w in c.words.iter().skip(1) {
                at += w.ipa.chars().count();
                cuts.push(at);
            }
            cuts.into_iter().collect()
        })
        .collect();

    let mut remaining: Vec<usize> = (0..clues.len()).collect();
    let mut picked = Vec::with_capacity(top_n.min(clues.len()));
    let mut max_overlap = vec![0.0_f64; clues.len()];

    // Scale the penalty by the full spread of the candidate pool.  This
    // states the policy directly: showing a structurally different
    // resegmentation may cost as much score as the difference between
    // the best and the worst candidate the search found.  Scaling by the
    // spread *inside* the requested top-N instead — which is what this
    // used to do — makes that spread a few thousandths for any real
    // search, so the penalty vanishes and "diverse" does nothing.
    let best = clues[0].score;
    let worst = clues
        .iter()
        .map(|c| c.score)
        .fold(f64::INFINITY, f64::min);
    let score_scale = (best - worst).max(1e-6);

    while picked.len() < top_n && !remaining.is_empty() {
        if let Some(&last) = picked.last() {
            for &i in &remaining {
                let word_overlap = jaccard(&word_sets[i], &word_sets[last]);
                let bigram_overlap = jaccard(&bigrams[i], &bigrams[last]);
                let cut_overlap =
                    jaccard_usize(&boundaries[i], &boundaries[last]);
                let overlap = 0.3 * word_overlap
                    + 0.2 * bigram_overlap
                    + 0.5 * cut_overlap;
                max_overlap[i] = max_overlap[i].max(overlap);
            }
        }

        let mut best_pos = 0;
        let mut best_value = f64::NEG_INFINITY;
        for (pos, &i) in remaining.iter().enumerate() {
            let value =
                clues[i].score - MMR_LAMBDA * score_scale * max_overlap[i];
            if value > best_value {
                best_value = value;
                best_pos = pos;
            }
        }

        picked.push(remaining.remove(best_pos));
    }

    let mut slots: Vec<Option<Clue>> = clues.into_iter().map(Some).collect();
    picked
        .into_iter()
        .map(|i| slots[i].take().expect("picked once"))
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let union = a.len() + b.len();
    if union == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / union as f64
}

fn jaccard_usize(a: &HashSet<usize>, b: &HashSet<usize>) -> f64 {
    let union = a.len() + b.len();
    if union == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / union as f64
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
        // "recognize speech" split as rec / og / nize / spitch: two new
        // inner cuts, one of which (after "rec") keeps the target's own
        // word break.
        let n = boundary_novelty(&[3, 4, 9, 14], &[9], 14, false);
        assert!((n - 2.0 / 3.0).abs() < 1e-9, "novelty {n}");
    }

    fn clue(phrase: &str, score: f64) -> Clue {
        let words: Vec<ClueWord> = phrase
            .split_whitespace()
            .map(|w| ClueWord {
                word: w.to_string(),
                ipa: "b".repeat(3),
                rarity: None,
                sub_cost: 0.0,
            })
            .collect();
        Clue {
            phrase: phrase.to_string(),
            ipa: "bbb".to_string(),
            words,
            score,
        }
    }

    /// Many spellings of one resegmentation must not crowd out every
    /// other resegmentation the search found.
    #[test]
    fn proposal_list_covers_distinct_resegmentations() {
        // One excellent clue for a word-boundary pattern, forty mediocre
        // siblings of it, six good clues with six *different* patterns,
        // and a low-scoring tail so the pool has a realistic spread.
        let mut pool: Vec<Clue> = vec![clue("aa0 b b", 0.930)];
        for i in 1..40 {
            pool.push(clue(&format!("aa{i} b b"), 0.880));
        }
        for (i, extra) in [0usize, 1, 3, 4, 5, 6].iter().enumerate() {
            let words: Vec<&str> = std::iter::once("dd")
                .chain(std::iter::repeat("b").take(*extra))
                .collect();
            pool.push(clue(&words.join(" "), 0.909 - i as f64 * 1e-3));
        }
        for i in 0..40 {
            pool.push(clue(&format!("ee{i} b b b b"), 0.82 + i as f64 * 1e-4));
        }
        let picked = select_diverse(pool, 10);
        assert_eq!(picked.len(), 10);
        let distinct: HashSet<Vec<usize>> =
            picked.iter().map(boundaries_of).collect();
        assert!(
            distinct.len() >= 6,
            "expected proposals spanning several resegmentations, got {distinct:?}"
        );
    }

    fn boundaries_of(c: &Clue) -> Vec<usize> {
        let mut cuts = Vec::new();
        let mut at = 0usize;
        for w in c.words.iter().skip(1) {
            at += w.ipa.chars().count();
            cuts.push(at);
        }
        cuts
    }

    /// The content-word axis is not decoration: two clues that are
    /// identical on every other axis must be separated by it, with the
    /// content-word one ahead.
    #[test]
    fn closed_class_axis_prefers_content_words_at_equal_cost() {
        // Same target stream, same number of words, same edit cost, same
        // syllables, same reuse: the only difference is which *class* of
        // word each span is filled with.
        let content = scored(1.0, 4, &[0, 3, 7, 11], 4, false, 0);
        let function = scored(1.0, 4, &[0, 3, 7, 11], 4, false, 4);
        assert!(
            content > function,
            "content-word clue scored {content}, closed-class clue {function}"
        );

        // The gap must be real but bounded: the axis may not be able to
        // outweigh every other consideration on its own.
        assert!(
            content - function <= CLOSED_CLASS_WEIGHT + 1e-9,
            "the closed-class penalty exceeded its own weight"
        );
        assert!(
            content - function > 0.5 * CLOSED_CLASS_WEIGHT,
            "the closed-class penalty is not doing its job"
        );
    }

    /// The penalty is convex in the closed-class share, so an idiomatic
    /// clue with one function word in four is far cheaper than a salad
    /// that is mostly function words.  This is the property that lets the
    /// weight be large enough to matter at all.
    #[test]
    fn closed_class_penalty_is_convex_in_the_closed_share() {
        let idiomatic = closed_class_penalty(1.0, 4.0);
        let salad = closed_class_penalty(3.0, 5.0);
        assert!(idiomatic < salad, "{idiomatic} !< {salad}");
        // Squaring the share separates them by more than the factor a
        // linear penalty would give, and keeps the idiomatic case cheap.
        assert!(idiomatic <= 0.0625 + 1e-12, "{idiomatic}");
        assert!(salad >= 0.30, "{salad}");
    }

    /// The axis is orthogonal to `word_novelty` (which asks whether a
    /// word differs from the target's) and anti-correlated with
    /// `familiarity` (which rewards common words, and the commonest
    /// English words are function words).  Nothing else in the score
    /// carries this information, which is why the axis earns its place
    /// rather than restating an existing one.
    #[test]
    fn closed_class_axis_is_not_covered_by_the_existing_axes() {
        // Corpus rarity of `a` and of `beach`.  A determiner is the more
        // frequent word by an order of magnitude, so `familiarity`
        // actively rewards the exact word the new axis penalises.
        let a_rarity = 4.0;
        let beach_rarity = 1_933.0;
        assert!(a_rarity < beach_rarity);
        assert!(
            word_familiarity(Some(a_rarity))
                > word_familiarity(Some(beach_rarity)),
            "familiarity is expected to reward the determiner"
        );
        assert!(lexical::is_closed_class("a"));
        assert!(!lexical::is_closed_class("beach"));
    }

    /// Score a synthetic clue directly through `Partial::metrics`, with
    /// every axis held fixed except the one under test.
    fn scored(
        sub_cost_total: f64,
        words: usize,
        cuts: &[usize],
        syllables: usize,
        partial: bool,
        closed: usize,
    ) -> f64 {
        let target = TargetPhrase::new("a b c d e");
        let target_boundaries = [2usize, 4, 6, 8];
        let mut p = Partial::empty();
        p.sub_cost_total = sub_cost_total;
        p.syllables = syllables;
        p.closed = closed;
        p.cuts = cuts.to_vec();
        p.words = (0..words)
            .map(|i| ClueWord {
                word: format!("w{i}"),
                ipa: "abc".to_string(),
                rarity: Some(1_000.0),
                sub_cost: 0.0,
            })
            .collect();
        p.metrics(
            &target,
            &target_boundaries,
            syllables,
            12,
            partial,
        )
        .combined
    }

    /// Orthographic variants of one clue are one proposal.
    #[test]
    fn phrase_signature_collapses_spelling_variants() {
        assert_eq!(
            phrase_signature("This' peach, wrecking"),
            phrase_signature("this peach wrecking")
        );
        assert_ne!(
            phrase_signature("wreck a nice beach"),
            phrase_signature("wreck a nice each")
        );
    }

    #[test]
    fn rhythm_axis_rewards_matching_syllable_counts() {
        assert!((rhythm_match(6, 6) - 1.0).abs() < 1e-9);
        assert!((rhythm_match(7, 6) - 0.5).abs() < 1e-9);
        assert!(rhythm_match(9, 6).abs() < 1e-9);
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

            let (ipa, _, _) = transcribe_with_boundaries(g.corpus(), target, true)
                .unwrap();
            let chars: Vec<char> = ipa.chars().collect();

            // Keep the cheapest acoustically valid alignment of the
            // required word sequence.  This checks matcher/budget
            // reachability independently of phrase-beam pruning.
            let mut states: HashMap<(usize, String), f64> =
                HashMap::from([((0usize, String::new()), 0.0_f64)]);
            for &wanted in expected {
                let mut next: HashMap<(usize, String), f64> = HashMap::new();
                for (&(pos, _), &total) in &states {
                    if pos >= chars.len() {
                        continue;
                    }
                    for m in g
                        .fuzzy_lexicon
                        .matches_at(&chars, pos, 0.5, 1)
                    {
                        let word = g.fuzzy_lexicon.word(m.word_idx);
                        if !word.word.eq_ignore_ascii_case(wanted) {
                            continue;
                        }
                        let accumulated = total + m.cost;
                        if accumulated > 1.5 + 1e-9 {
                            continue;
                        }
                        next
                            .entry((pos + m.consumed, word.word.clone()))
                            .and_modify(|best| *best = best.min(accumulated))
                            .or_insert(accumulated);
                    }
                }
                assert!(
                    !next.is_empty(),
                    "{expected:?} is not reachable within the default budgets for {target:?}"
                );
                states = next;
            }
            assert!(
                states.keys().any(|(pos, _)| *pos == chars.len()),
                "{expected:?} does not cover the whole target for {target:?}"
            );
        }

        check("recognize speech", &["wreck", "a", "nice", "beach"]);
        check(
            "It's just a stupid game",
            &["hits", "justice", "dupe", "hid", "came"],
        );
    }
}
