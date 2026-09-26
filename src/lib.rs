//! Mad Gab puzzle generator.
//!
//! The generator searches for English word sequences whose connected
//! pronunciation is close to a target phrase while preferring a
//! genuinely different lexical/word-boundary parse.

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use std::rc::Rc;

use phonetics::transcriptions::{Corpus, Pronunciation};
use serde::Serialize;

mod adjacency;
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
    /// Phoneme offsets into the target's IPA stream at which this clue's
    /// word boundaries fall, the last one being the end of the stream.
    ///
    /// This is the clue's *resegmentation*: the word-boundary structure
    /// the search aligned it at.  It is reported rather than re-derived
    /// from [`Self::words`] because under an approximate alignment
    /// the two disagree — a clue word consumes a run of the target's
    /// phonemes, which is not the same run as its own transcription — and
    /// every consumer that wanted the structure was therefore reading a
    /// corrupted one.  See [`clue_structure`].
    pub cuts: Vec<usize>,
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

// -----------------------------------------------------------------
// The per-segmentation emission allowance
// -----------------------------------------------------------------
//
// The lexical phase enumerates, for one segmentation, the Cartesian
// product of its slots' candidate lists, and every segmentation is given
// `LEXICAL_COMBINATIONS_PER_SEGMENTATION` wordings to spend.  A
// best-first walk spends that allowance on the tuples with the best
// bound, which is exactly the corner where every slot takes a cheap
// index 0.  A wording that is excellent overall but locally bad in one
// or two slots is therefore not merely late in that order: it is
// *absent*, and no ordering of the same order can bring it inside an
// allowance that small.
//
// So part of the allowance buys a *spread* of the index-tuple space
// instead: representatives of depth profiles — which slots are deep —
// that the cost-best corner never produces.  These three constants are
// the whole of that spend, and they are module scope so the bound they
// imply is testable without running a search.

/// How many alternatives a span's shortlist retains.
const SPAN_SHORTLIST: usize = 160;
/// The wordings one segmentation may emit, profiles included.
const LEXICAL_COMBINATIONS_PER_SEGMENTATION: usize = 64;
/// The width every slot of the per-segmentation traversal is *opened* at,
/// and the factor by which it is widened when — and only when — the
/// traversal has exhausted the current width and still wants wordings.
///
/// This is not a ceiling.  A candidate at any rank of any slot is pushed as
/// soon as the traversal reaches a level that can afford it, and a level the
/// traversal never reaches is never paid for, so the uniform pre-filter that
/// used to sit in front of the traversal — and made a word at walk rank 99
/// of a slot unreachable by any visit order — is gone.  See
/// [`next_branch_stage`] and `docs/work/items/w-9d4e17.md`.
///
/// The first stage is the number the traversal used to be capped at, so the
/// first stage costs exactly what the cap cost, and the growth factor is the
/// smallest that gets from it to a `SPAN_SHORTLIST`-wide list in three
/// stages (10 -> 40 -> 160).
const LEXICAL_BRANCH_STAGE_0: usize = 10;
const LEXICAL_BRANCH_STAGE_GROWTH: usize = 4;

/// Part of every segmentation's allowance reserved for depth profiles.
///
/// It is carved out *before* the traversal starts and the traversal is
/// handed only what the profiles did not use, so the per-segmentation
/// total is unchanged and the global budgets still hold; ordinary
/// quality keeps the rest.
const EMIT_PROFILE_RESERVE: usize = 16;
/// Part of every segmentation's allowance reserved for the **adjacency**
/// operator (w-c1d3a7), the neighbourhood walk over the wordings the
/// traversal already emitted.
///
/// The profile reserve above samples the index-tuple space *from the corner*:
/// a profile representative keeps index 0 in every slot outside the profile,
/// so it can be deep in at most [`EMIT_PROFILE_MAX_DEEP`] slots and it cannot
/// reach a wording that is deep in several slots *and* off the corner in the
/// rest.  The adjacency operator spends its share differently: it starts from
/// wordings the pool already holds and substitutes **one slot at a time**, so
/// the cost of being deep in one slot is additive rather than
/// multiplicative.
///
/// This is the one per-segmentation spend that is *not* carved out of the
/// traversal's allowance.  Carving it out there is what the profile reserve
/// does, and doing the same here measurably costs a deep-in-a-span match
/// (`approximate_pool_reaches_matches_deep_in_a_span`), because the
/// traversal's own emissions are what the depth tests are written against.
/// It is bounded by `LEXICAL_GLOBAL_EMISSION_BUDGET` instead — the ceiling the
/// whole search already respects, and one the baseline leaves slack (14,239
/// of 16,384 spent).  So the operator's spend is bounded by the same
/// constant that bounds everything else, and the global bound is unchanged.
const ADJACENCY_RESERVE: usize = 8;
/// The adjacency operator's share of the **whole search**, across every
/// segmentation, as a reserved slice rather than a per-segmentation spend.
///
/// The per-segmentation [`ADJACENCY_RESERVE`] is a cap; this is the ceiling
/// that stops the caps from adding up to more than the traversal can afford.
/// Without it the two spends compete for the same
/// `LEXICAL_GLOBAL_EMISSION_BUDGET`, and because the operator runs
/// interleaved with the traversal in schedule order it wins the argument
/// early and the *tail* of the schedule is truncated instead — which is
/// exactly what happened: with the per-segmentation guarantee in place and no
/// slice of its own, the whole search spent 14,239 + 256 * 8 = 16,287 of the
/// 16,384 global budget and
/// `approximate_pool_reaches_matches_deep_in_a_span` lost a wording.
///
/// `LEXICAL_GLOBAL_EMISSION_BUDGET / 16` = `SEGMENTATION_KEEP *
/// LEXICAL_COMBINATIONS_PER_SEGMENTATION / 16` = **1,024**, which is a
/// sixteenth of the search's whole emission budget: enough for the operator to
/// matter on a pool of ~180 structures (5-6 admissions each) and small enough
/// that the traversal's own 14,239 is never at risk.  The same fraction is
/// written out rather than divided, because `SEGMENTATION_KEEP` is scoped
/// inside the search function; if that constant moves, this one must move with
/// it.
const ADJACENCY_GLOBAL_RESERVE: usize = 1_024;
/// How many of a node's children in one slot the adjacency walk retains, and
/// therefore how many of them it allocates and pushes.
///
/// The seeds the walk expands are the pool's own wordings, so the pop budget
/// is *derived* from them by [`adjacency::pop_budget`] rather than chosen
/// here: a hand-picked pop count promises nothing once it falls below the
/// seed count, which is the normal case.  The reserve is
/// `ADJACENCY_RESERVE`, so the walk is guaranteed that many admissions per
/// segmentation and the caller's cap and the walk's reach are the same
/// number by construction.  See `docs/work/items/w-6f3a91.md`.
const ADJACENCY_PER_SLOT: usize = 2;
/// How many slots of a profile may be deep at once.  A `k`-deep profile
/// class has `C(depth, k)` members per sweep step, so beyond two the
/// classes outnumber the reserve and the sweep is not widened to
/// compensate.
///
/// That reason expired with the class *order*.  It counted classes against a
/// reserve spent breadth-before-depth, so a deep class was only reached once
/// every shallower class had been paid in full — which is what made the class
/// count, and not the reserve, the thing that bounded the depth.  The order is
/// now interleaved by depth (see [`coverage_tuples`]), so the deepest classes
/// are funded from the *first* tuple of the reserve whatever the class count
/// is, and the count no longer bounds anything.
const EMIT_PROFILE_MAX_DEEP: usize = 4;

/// The per-slot rotation of a profile class's sweep.
///
/// Every slot of a class sweeps *its own* candidate list, at its own width and
/// at its own rotation of the phase.  The rotation is what makes those
/// coordinates independent: with one shared rotation a class's coordinates
/// differ only by `slot * COVERAGE_SLOT_ROTATION`, so a class can only ever
/// place *one* alternative per slot no matter how it is swept, and every
/// wording the reserve emits is a diagonal of the class's index rectangle
/// rather than a point of it.
///
/// The value is only required to be non-zero and to keep the per-slot
/// rotations distinct, which is what it does for every slot count the search
/// produces (a run of targets has no segmentation five slots wide and no
/// segmentations above ten).  It is coprime with the spans the sweep actually
/// meets — 150, 83, 151, 81 — so the union of a run's rotations still tiles
/// each slot's list rather than only a residue class of it.
const COVERAGE_SLOT_ROTATION: usize = 37;

/// The `k`-subsets of `0..len`, in lexicographic order.
fn slot_combinations(len: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if k == 0 || k > len {
        return out;
    }
    let mut combo: Vec<usize> = (0..k).collect();
    loop {
        out.push(combo.clone());
        let mut i = k;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if combo[i] != i + len - k {
                combo[i] += 1;
                for j in i + 1..k {
                    combo[j] = combo[j - 1] + 1;
                }
                break;
            }
        }
    }
}

/// The next per-slot width the per-segmentation traversal should open, or
/// `None` when it has already read every alternative every slot has.
///
/// A slot's depth is a property of the traversal, not of the list: the walk
/// opens at [`LEXICAL_BRANCH_STAGE_0`], reads that width to exhaustion, and
/// asks for the next one *only* if it is still hungry.  So the width is
/// geometric (three stages span a `SPAN_SHORTLIST`-wide list from 10) and
/// saturates at the widest list present, which means no candidate is ever
/// removed by a width rule — a word at any rank of any slot is pushed as
/// soon as the traversal reaches the level that can afford it, and a level
/// the traversal never reaches is never paid for.
fn next_branch_stage(current: usize, widest: usize) -> Option<usize> {
    // `max(1)` only matters for the degenerate `current == 0`, which the
    // traversal cannot reach (an empty slot is skipped before it runs); it
    // keeps the schedule from stalling at zero width.
    let wider = current.max(1).saturating_mul(LEXICAL_BRANCH_STAGE_GROWTH);
    if widest <= current || wider <= current {
        None
    } else {
        Some(wider.min(widest))
    }
}

/// The `nth` index a uniform sweep of one slot's alternatives visits.
///
/// The sweep covers `floor..width`, where `floor` is the width the traversal
/// *opens* at.  The step is uniform — `span` indices taken `per` at a time,
/// rounded **up**, with the whole sweep rotated by `phase` — and `None` when
/// the slot is no wider than that floor, so the rule can never walk off the
/// end of a list or invent an index.
///
/// # Why one call already tiles the list
///
/// `stride = span.div_ceil(per)`, so the `per` indices of a single sweep
/// advance by at least `span / per` and the *last* of them lands at or past
/// `span`, i.e. at or past the end of the range.  Because the start is
/// rotated by `phase` and taken modulo `span`, the sweep wraps, so a sweep
/// is an exact cover when `per` divides `span` and an exact cover with
/// `per - span % per` repeats when it does not — never a gap.  That is what
/// makes the rotation a rotation of a *cover* rather than of a sample: one
/// segmentation's reserve already visits every index its slots have, and the
/// phase exists so that a structure's successive segmentations do not all
/// spend on the same tiling.
///
/// # The coupling this floor has, stated honestly
///
/// `floor` is the traversal's *first* stage, so the claim "every index below
/// the floor is one the traversal generates" is a claim about the stage the
/// traversal opens at, not about every width it can open.  [`next_branch_stage`]
/// can open 40 and 160 too, and if a traversal ever *drains* at a narrow
/// width with appetite left, indices in `(floor, wider)` are then reachable
/// by the traversal as well and the reserve's spend on them is duplicated
/// rather than additive.  Two things bound that.  The widening is asked for
/// only when the heap empties, and the per-segmentation emission allowance is
/// exhausted long before a `10^depth` product does; and
/// `docs/work/items/w-9d4e17.md` measures the widening firing **zero** times
/// across the six real multi-clause targets.  So on real targets the floor
/// is the traversal's real ceiling and the reserve's spend is pure coverage —
/// but that is a measurement, not a theorem, and this comment does not claim
/// it as one.  The assertion in
/// `tests::the_coverage_sweep_starts_where_the_traversal_stops` is scoped to
/// the first stage for the same reason.
fn sweep_index(width: usize, per: usize, nth: usize, phase: usize) -> Option<usize> {
    let floor = LEXICAL_BRANCH_STAGE_0;
    if width <= floor {
        return None;
    }
    let span = width - floor;
    let stride = span.div_ceil(per.max(1));
    Some(floor + (nth * stride + phase) % span)
}

/// The index tuples the coverage reserve emits for one segmentation.
///
/// The reserve exists because the best-first walk orders by an *admissible
/// bound*, so it spends its allowance in descending bound order and that
/// order is concentrated in the corner where every slot sits at its own
/// best alternative.  A word that is a real alternative of its span but not
/// its best one is therefore not late in that order: it is absent, and no
/// budget reaches it, because the number of better-bound tuples in front of
/// it grows with the product of the other slots' widths.
///
/// So the reserve is a *systematic sample* of the index-tuple space, and a
/// systematic sample of a list is uniform over the list.  The rule used to
/// sample four geometric rungs (10, 30, 80, 200), deepest rung first.  That
/// is coverage of four points: everything between two rungs was unsampled,
/// and because the reserve was spent deepest-first it was spent entirely on
/// the top two.  A dictionary word at rank 13 of a 160-wide slot was
/// unreachable in every cell of the search.
///
/// The rule walks slot subsets in an order that **interleaves the depths**:
/// the whole of the one-deep classes first — so a reserve too small to afford
/// every shape still touches every slot, as it always did — and then, round by
/// round, one class of every *deeper* shape in turn, with the classes of a
/// depth in [`slot_combinations`] order.  So a reserve too small to afford
/// every class pays for a spread of *depths* rather than for the whole of one
/// of them, and a pairing shape is funded from the reserve's second tuple
/// onwards instead of after every one-deep and two-deep shape has been paid
/// for in full.  (Depth-then-breadth, which this replaces, bought the
/// every-slot guarantee by making the depth of a funded shape a function of
/// how many shallower classes there were, and that is what confined a deep
/// alternative to a shape the reserve never reached: with a reserve of 16 and
/// the class counts a five-slot segmentation produces, the four-deep shape was
/// the sixteenth class tried and the sixteenth tuple spent.)
///
/// Every slot of a class takes its **own** coordinate, from its own list, at
/// its own width and its own rotation of the phase ([`profile_tuple`]); the
/// slots outside the class keep the traversal's own best index, so a
/// representative is the cheapest wording of that shape rather than a
/// general-purpose regression, and `build` still compares the tuple's total
/// substitution cost against `total_budget` additively, so the reserve is
/// bounded by the same bound as every other emission.
///
/// What the class order and the per-slot rotation buy together is the
/// difference between sampling a *diagonal* of a class's index rectangle and
/// sampling *points of* it.  A wording whose slots are all off the traversal's
/// best index at ranks unrelated to one another — the shape every real
/// resegmentation has, and the shape the acceptance corpus is made of — is a
/// point of that rectangle and not a diagonal of it, so under the shared-index
/// rule it is not late in the reserve's order: it is absent, exactly as a deep
/// word is absent from the traversal's.
///
/// `phase` is a counter the search advances once per segmentation, so the
/// sweeps of one run tile the lists between them.  It is a pure function of
/// the deterministic schedule, so the enumeration stays reproducible.
fn coverage_tuples(
    slot_widths: &[usize],
    reserve: usize,
    max_deep: usize,
    phase: usize,
) -> Vec<Vec<usize>> {
    let depth = slot_widths.len();
    let max_deep = max_deep.min(depth);
    let mut out: Vec<Vec<usize>> = Vec::new();
    if reserve == 0 || depth == 0 {
        return out;
    }
    let by_depth: Vec<Vec<Vec<usize>>> = (1..=max_deep)
        .map(|deep| slot_combinations(depth, deep))
        .collect();
    // The whole of the one-deep classes is walked first, so a reserve too
    // small to afford every shape still touches every slot, as it always did.
    // Every later round then offers one class of each *deeper* shape in turn,
    // so a reserve that cannot afford all of them pays for a spread of depths
    // rather than for the whole of one of them.  A class's `nth` draw widens
    // its own sweep (`per = nth + 1`), so a class that survives several
    // rounds gets several distinct points rather than a repeat of its first.
    for combo in &by_depth[0] {
        if out.len() >= reserve {
            return out;
        }
        if let Some(tuple) = profile_tuple(slot_widths, combo, 0, phase) {
            out.push(tuple);
        }
    }
    let mut drawn: Vec<usize> = vec![0; by_depth.len()];
    'rounds: loop {
        if (1..by_depth.len()).all(|k| drawn[k] >= by_depth[k].len()) {
            break 'rounds;
        }
        for k in 1..by_depth.len() {
            if out.len() >= reserve {
                break 'rounds;
            }
            let Some(combo) = by_depth[k].get(drawn[k]) else {
                continue;
            };
            if let Some(tuple) = profile_tuple(slot_widths, combo, drawn[k], phase) {
                out.push(tuple);
            }
            drawn[k] += 1;
        }
    }
    out
}

/// One point of one shape class: the class's `nth` sweep, with every slot of
/// the class sweeping *its own* list at *its own* width and rotation.
///
/// `None` when any member slot is no wider than the traversal's opening width,
/// which is the same "drop out rather than invent an index" rule
/// [`sweep_index`] applies — and the reason a class is not partially emitted:
/// a representative of a class is a wording of *that* shape, and a shape with
/// one of its slots at the traversal's own best index is a different shape.
fn profile_tuple(
    slot_widths: &[usize],
    combo: &[usize],
    nth: usize,
    phase: usize,
) -> Option<Vec<usize>> {
    let per = nth + 1;
    let mut tuple = vec![0usize; slot_widths.len()];
    for &slot in combo {
        let at = sweep_index(
            slot_widths[slot],
            per,
            nth,
            phase.wrapping_add(slot.wrapping_mul(COVERAGE_SLOT_ROTATION)),
        )?;
        tuple[slot] = at;
    }
    Some(tuple)
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
                    let next = partial.extend_pronunciation(
                        &target_phrase,
                        pronunciation,
                        consumed,
                        0.0,
                    );
                    insert_top_k(&mut beam[p + consumed], next, self.config.beam_width);
                }
            }
        }

        self.finish(
            std::mem::take(&mut beam[n]),
            &target_ipa,
            &target_boundaries,
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
                    let next = partial
                        .extend_fuzzy(&target_phrase, word, m.consumed, m.cost);
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
            target_syllables,
            n,
        );

        // Recovery search: decouple segmentation survival from lexical
        // survival.  The ordinary beam is deliberately tight and fast,
        // but a locally mediocre word can otherwise erase an excellent
        // global resegmentation.  First retain a small set of promising
        // target-span structures; only then explore lexical alternatives
        // inside each retained structure.
        /// One retained target span: the words that can fill it, and
        /// the extremums of the score components they contribute.  The
        /// extremums are what a span path is summarized by, so they are
        /// computed once here rather than per candidate.
        #[derive(Clone)]
        struct SpanEdge {
            end: usize,
            matches: Vec<approx::FuzzyMatch>,
            extremes: SpanExtremes,
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

        /// A chain of target spans.  `extremes` summarizes every score
        /// component the chain has committed to, and `shared` counts
        /// the target inner boundaries it reproduces.
        #[derive(Clone)]
        struct SegPath {
            spans: Vec<(usize, usize)>,
            shared: usize,
            extremes: SpanExtremes,
            rank: f64,
        }

        const SPAN_AXIS_KEEP: usize = 16;
        const SPAN_BAND_KEEP: usize = 4;
        const SPAN_RARITY_KEEP: usize = 4;
        const SEG_STATE_KEEP: usize = 32;
        const SEGMENTATION_KEEP: usize = 256;
        const LEXICAL_HEAP_POP_LIMIT: usize = 4_000;

        // The lexical phase's budget is *global*.  These are the same two
        // products the old per-segmentation caps implied when multiplied
        // out by `SEGMENTATION_KEEP` — they were always the real worst
        // case and were only ever reached implicitly, by multiplying two
        // constants that were written to bound one segmentation each.
        // Naming them changes who the budget belongs to, not how much of
        // it there is: the arithmetic is deliberately identical, so the
        // time and memory bound of the search is unchanged by this
        // commit and only its *distribution* moves.
        //
        // `LEXICAL_GLOBAL_EMISSION_BUDGET` bounds how many wordings reach
        // the pool; `LEXICAL_GLOBAL_POP_BUDGET` bounds the heap work that
        // produces them, and is the one that actually bounds wall clock.
        const LEXICAL_GLOBAL_EMISSION_BUDGET: usize =
            SEGMENTATION_KEEP * LEXICAL_COMBINATIONS_PER_SEGMENTATION;
        const LEXICAL_GLOBAL_POP_BUDGET: usize =
            SEGMENTATION_KEEP * LEXICAL_HEAP_POP_LIMIT;

        // These two are NON-BINDING and cannot be made to bind, and that
        // is a property of the traversal rather than of their values: the
        // whole search spends at most `SEGMENTATION_KEEP *
        // LEXICAL_COMBINATIONS_PER_SEGMENTATION` = 256 * 64 = 16_384
        // emissions, because `emit_allowance` is clamped per segmentation
        // to at most `LEXICAL_COMBINATIONS_PER_SEGMENTATION`, so the
        // global constant is already at its maximum reachable value at
        // 1x. Measured inert across 1x..256x (w-be6d21). Do not spend
        // another pass tuning them; the binding constraint is per-slot
        // width, `LEXICAL_BRANCH_KEEP` (w-9d4e17).

        // The score of the worst clue the ordinary beam already put in
        // the pool.  This is the bar the search already commits to — it
        // is the same pool size `final_keep` fixes above, not a new one.
        // A span path is discarded outright only when its admissible
        // bound is under it, which is sound: nothing the bound covers
        // could have entered the output anyway.
        let mut incumbent: Vec<f64> = completed
            .iter()
            .map(|p| {
                p.metrics(
                    &target_boundaries,
                    target_syllables,
                    n,
                    false,
                )
                .combined
            })
            .collect();
        incumbent.sort_by(|a, b| cmp_desc(*a, *b));
        let incumbent = incumbent
            .get(final_keep - 1)
            .copied()
            .unwrap_or(f64::NEG_INFINITY);

        let target_inner: HashSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|&b| b < n)
            .collect();
        let target_inner_count = target_inner.len();

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
                let max_shape = selected
                    .iter()
                    .map(|m| {
                        let word = self.fuzzy_lexicon.word(m.word_idx);
                        lexical_shape_quality(
                            &word.word,
                            word_familiarity(word.rarity),
                        )
                    })
                    .fold(0.0, f64::max);
                // The structural DP can only pick from the shortlist, so
                // the best it may assume for this span is "no
                // closed-class word is forced here", which is achievable
                // exactly when at least one alternative is a content
                // word.  A higher value would be a bound the enumeration
                // below could not reach, and a lower one a bound the
                // final scorer could beat.
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
                    extremes: SpanExtremes {
                        min_cost,
                        max_familiarity,
                        max_shape,
                        min_closed,
                        min_reused,
                        min_syllables,
                        max_syllables,
                    },
                });
            }
        }

        // Suffix relaxation over the span DAG: from every target offset,
        // the least that finishing the target from there can be worth.
        // A completion has to follow one path, so the relaxation joins
        // the alternatives by taking, per axis, the value that holds
        // whichever one it picks.  This is what makes
        // [`span_score_bound`] admissible over a *partial* span path.
        let mut tail: Vec<Option<SpanExtremes>> = vec![None; n + 1];
        tail[n] = Some(SpanExtremes::ZERO);
        for p in (0..n).rev() {
            let mut acc: Option<SpanExtremes> = None;
            for edge in &span_lattice[p] {
                let Some(after) = tail[edge.end] else {
                    continue;
                };
                acc = Some(match acc {
                    None => edge.extremes.upper_plus(after),
                    Some(so_far) => so_far.best_of(
                        edge.extremes.upper_plus(after),
                    ),
                });
            }
            tail[p] = acc;
        }

        // Three keys, all built from the final scorer's own weights and
        // all derived from the same per-span extremums:
        //
        // * [`partial_span_score`] ranks the representatives kept inside
        //   a DP state,
        // * [`complete_span_score`] orders the finished structures the
        //   lexical enumeration expands,
        // * [`span_score_bound`] is the admissible one, and is the only
        //   key allowed to discard a path outright.
        let span_partial = |words: usize, s: &SegPath| -> f64 {
            partial_span_score(s.extremes, words, target_syllables)
        };

        let span_objective = |words: usize, s: &SegPath| -> f64 {
            complete_span_score(
                s.extremes,
                words,
                s.shared,
                target_inner_count,
                target_syllables,
            )
        };

        let span_bound = |at: usize, words: usize, s: &SegPath| -> f64 {
            let Some(tail) = tail[at] else {
                return f64::NEG_INFINITY;
            };
            span_score_bound(
                s.extremes,
                tail,
                words,
                s.shared,
                n - at,
                target_inner_count,
                target_syllables,
            )
        };

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
                extremes: SpanExtremes::ZERO,
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
                            || path.extremes.min_cost + edge.extremes.min_cost
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

                        let extended = SegPath {
                            spans: Vec::new(),
                            shared: next_shared,
                            extremes: path.extremes
                                .upper_plus(edge.extremes),
                            rank: 0.0,
                        };

                        // A path whose admissible bound is already under
                        // the incumbent cannot enter the output, so it is
                        // not carried forward at all.
                        if span_bound(edge.end, next_words, &extended)
                            < incumbent
                        {
                            continue;
                        }

                        let bucket = seg_states[edge.end]
                            .entry((next_words, next_shared))
                            .or_default();
                        bucket.push(SegPath {
                            spans,
                            rank: span_partial(next_words, &extended),
                            ..extended
                        });
                        bucket.sort_by(|a, b| cmp_desc(a.rank, b.rank));
                        bucket.truncate(SEG_STATE_KEEP);
                    }
                }
            }
        }

        let mut segmentations: Vec<(f64, SegPath)> = Vec::new();
        for ((word_count, _shared), paths) in
            std::mem::take(&mut seg_states[n])
        {
            if word_count == 0 {
                continue;
            }
            for path in paths {
                // The path is complete here, so novelty is exact and
                // the scorer's own weights apply without reservation.
                segmentations.push((span_objective(word_count, &path), path));
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
        //
        // The *global* budget is what this loop spends, and it is spent on
        // boundary structures rather than on segmentations.  Measured on
        // real targets at `top_n = 50`, the retained pool holds 137-180
        // distinct boundary structures while the visible list shows 4-12
        // of them.  The old loop gave every one of the up-to-256 retained
        // segmentations its own full `LEXICAL_COMBINATIONS_PER_SEGMENTATION`
        // wordings, so a structure realized by thirty alignments was
        // funded 1920 deep while a structure realized by one was funded 64
        // deep — the budget went almost entirely on wordings competing for
        // the same handful of structure slots, and the rare resegmentations
        // were the ones starved.
        //
        // So the budget is now shared out per *structure*:
        //
        // * `breadth` is `structure_wording_allowance` — the number of
        //   wordings of one structure the display policy can admit at all
        //   (see that function).  Every structure's best alignment is
        //   funded that far, and the schedule is breadth-first over
        //   structures, so no structure can be starved by a stronger one;
        // * whatever is left of `LEXICAL_GLOBAL_EMISSION_BUDGET` becomes
        //   each structure's *equal share* of the whole budget
        //   (`structure_depth_ceiling`), spent as extra depth on the same
        //   structures once breadth is paid for.
        //
        // The total is unchanged: `structure_depth_ceiling` sums to at most
        // the global budget, which is the product the old per-segmentation
        // caps already implied.  This moves spend between structures; it
        // does not buy more of it.
        let mut recovered = Vec::new();
        let breadth = structure_wording_allowance(self.config.top_n);
        // A structure's wordings, and how many of its retained
        // segmentations have been enumerated.  The key is the
        // segmentation's own span list, which is exactly what `Clue::cuts`
        // later reports, so the search and the display policy count the
        // same structure rather than two re-derived versions of it.
        let mut funded: HashMap<&[(usize, usize)], usize> = HashMap::new();
        // Breadth-first schedule over structures: every structure's best
        // alignment is enumerated before any structure's second.  Without
        // it a share rule starves the tail for exactly the reason the
        // per-slot cap did — the late-ranked resegmentations never get a
        // turn.
        let mut schedule: Vec<usize> = Vec::with_capacity(segmentations.len());
        let depth_ceiling = {
            let mut by_structure: Vec<Vec<usize>> = Vec::new();
            let mut where_: HashMap<&[(usize, usize)], usize> = HashMap::new();
            for (i, (_, seg)) in segmentations.iter().enumerate() {
                match where_.get(seg.spans.as_slice()) {
                    Some(&g) => by_structure[g].push(i),
                    None => {
                        where_.insert(seg.spans.as_slice(), by_structure.len());
                        by_structure.push(vec![i]);
                    }
                }
            }
            let ceiling = structure_depth_ceiling(
                LEXICAL_GLOBAL_EMISSION_BUDGET,
                by_structure.len(),
                breadth,
            );
            let mut round = 0usize;
            loop {
                let before = schedule.len();
                for group in &by_structure {
                    if let Some(&i) = group.get(round) {
                        schedule.push(i);
                    }
                }
                if schedule.len() == before {
                    break;
                }
                round += 1;
            }
            ceiling
        };
        let mut spent_emissions = 0usize;
        // The coverage reserve's rotation, advanced once per segmentation.
        // See [`coverage_tuples`].
        let mut coverage_phase = 0usize;
        // The adjacency operator's reserved slice of the emission budget, see
        // `ADJACENCY_GLOBAL_RESERVE`.  It is drawn down here so that the
        // operator's spend is bounded independently of the traversal's.
        let mut adjacency_spend = ADJACENCY_GLOBAL_RESERVE;
        let mut spent_pops = 0usize;
        for &index in &schedule {
            if spent_emissions >= LEXICAL_GLOBAL_EMISSION_BUDGET
                || spent_pops >= LEXICAL_GLOBAL_POP_BUDGET
            {
                break;
            }
            let (_, segmentation) = &segmentations[index];
            let structure: &[(usize, usize)] = &segmentation.spans;
            // This segmentation may be funded whatever its structure has
            // not already been funded, up to the per-segmentation ceiling:
            // the first alignment of a structure gets the breadth the
            // display policy can use, and later alignments of the same
            // structure share what is left of the structure's depth.
            let held = funded.get(structure).copied().unwrap_or(0);
            let emit_allowance = depth_ceiling
                .saturating_sub(held)
                .clamp(1, LEXICAL_COMBINATIONS_PER_SEGMENTATION);
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

            // The depth-profile reserve, spent before the traversal so it
            // is genuinely reserved rather than left over: the traversal
            // below is handed whatever the profiles did not use.  Both
            // halves count against the same per-segmentation allowance
            // and the same global budgets, so the total is unchanged.
            let build = |tuple: &[usize]| -> Option<Partial> {
                let total_cost: f64 = tuple
                    .iter()
                    .enumerate()
                    .map(|(slot, &i)| slots[slot][i].cost)
                    .sum();
                if total_cost > total_budget + 1e-9 {
                    return None;
                }
                let mut partial = Partial::empty();
                for (slot, &i) in tuple.iter().enumerate() {
                    let a = &slots[slot][i];
                    let word = self.fuzzy_lexicon.word(a.match_ref.word_idx);
                    partial = partial.extend_fuzzy(
                        &target_phrase,
                        word,
                        a.match_ref.consumed,
                        a.cost,
                    );
                }
                Some(partial)
            };
            let profile_allowance =
                EMIT_PROFILE_RESERVE.min(emit_allowance);
            let mut profile_emitted = 0usize;
            // The index tuples this segmentation has actually put in the
            // pool, in emission order.  The adjacency operator below is
            // seeded from exactly this, so it starts from what the pool
            // holds rather than from the index-tuple origin.
            let mut pooled: Vec<Vec<usize>> = Vec::new();
            let widths: Vec<usize> =
                slots.iter().map(Vec::len).collect();
            for tuple in coverage_tuples(
                &widths,
                profile_allowance,
                EMIT_PROFILE_MAX_DEEP,
                coverage_phase,
            ) {
                if profile_emitted >= profile_allowance
                    || spent_emissions >= LEXICAL_GLOBAL_EMISSION_BUDGET
                {
                    break;
                }
                let Some(partial) = build(&tuple) else {
                    continue;
                };
                #[cfg(test)]
                counters::note_depth(
                    &counters::DEEPEST_PROFILE,
                    tuple.iter().copied().max().unwrap_or(0),
                );
                pooled.push(tuple);
                recovered.push(partial);
                profile_emitted += 1;
                spent_emissions += 1;
                *funded.entry(structure).or_default() += 1;
            }
            // The next segmentation sweeps a rotated window of the same
            // lists, so the union of a run's sweeps is the list rather than
            // one arithmetic progression of it.  Advanced once per
            // segmentation, on the deterministic schedule, so the
            // enumeration stays reproducible.
            coverage_phase = coverage_phase.wrapping_add(1);
            let emit_allowance =
                emit_allowance.saturating_sub(profile_emitted);
            // The adjacency operator's share.  It is *not* carved out of the
            // traversal's allowance: the traversal's own emissions are the
            // ones the depth tests are written against, and taking eight of
            // them measurably costs a deep-in-a-span match
            // (`approximate_pool_reaches_matches_deep_in_a_span`).  The
            // operator is instead bounded by the *global* emission budget,
            // which is the search's real ceiling and which the baseline
            // leaves slack (14,239 of 16,384 spent), so the operator's spend
            // is bounded by the same constant that bounds everything else.
            // `.min(adjacency_spend)` draws on the reserved slice, so the
            // operator runs out of budget rather than the traversal.
            let adjacency_allowance = ADJACENCY_RESERVE
                .min(emit_allowance)
                .min(adjacency_spend);

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
            // The prefix is shared rather than copied: every branch of the
            // search extends the same path, and this loop pushes millions
            // of them.  `Rc<[usize]>` compares and hashes exactly like the
            // `Vec` it replaces, so the heap's pop order and the `seen`
            // membership test are unchanged — only the allocation count
            // per branch drops from two to one.
            let empty: Rc<[usize]> = Rc::from(Vec::<usize>::new());
            heap.push((quantized(bound(&[])), 0usize, empty.clone()));
            let mut seen: HashSet<(usize, Rc<[usize]>), BuildHasherDefault<FxHasher>> =
                HashSet::default();
            seen.insert((0, empty.clone()));

            // How deep any single slot may be read is *not* a property of
            // the list: it is a property of what this traversal is still
            // willing to spend.  `cap` opens at `LEXICAL_BRANCH_STAGE_0`
            // and is widened — geometrically, up to the widest list here —
            // only after the current width has been walked to exhaustion
            // *and* the traversal still wants wordings.  So a level the
            // traversal never needs to visit costs nothing, and no candidate
            // can be missing merely because it sat at rank 99 of a slot that
            // a uniform pre-filter had already truncated.
            let widest = widths.iter().copied().max().unwrap_or(0);
            let mut cap = LEXICAL_BRANCH_STAGE_0.min(widest);
            let mut emitted = 0usize;
            let mut popped = 0usize;
            // The traversal runs in passes over the heap.  A walk is the
            // search itself: pop, expand, emit.  When a walk drains without
            // filling its allowance, the only thing the next stage needs is
            // the list of nodes the walk already expanded, so that it can
            // open their newly legal children instead of re-expanding the
            // lattice.  That list is written only by a *replay* — a second,
            // identical pass over a freshly seeded heap that records what it
            // expands and emits nothing — and a replay is scheduled only
            // once the traversal has already decided it wants a wider stage.
            // So the pass that runs on every segmentation of every target
            // allocates nothing per pop, which matters because the
            // measurement in w-9d4e17 is that a widening never happens at
            // all on a multi-clause real target.
            let mut replaying = false;
            let mut opened = cap;
            let mut expanded: Vec<(usize, Rc<[usize]>)> = Vec::new();
            let root: Rc<[usize]> = empty.clone();
            'stages: loop {
                let mut finished = false;
                while let Some((_key, k, prefix)) = heap.pop() {
                    if !replaying {
                        popped += 1;
                    }
                    if k == depth {
                        if let Some(partial) = build(&prefix) {
                            if !replaying {
                                #[cfg(test)]
                                counters::note_depth(
                                    &counters::DEEPEST_TRAVERSAL,
                                    prefix.iter().copied().max().unwrap_or(0),
                                );
                                pooled.push(prefix.to_vec());
                                recovered.push(partial);
                                emitted += 1;
                                spent_emissions += 1;
                                *funded.entry(structure).or_default() += 1;
                            }
                            // This segmentation's share of its structure's
                            // depth, and the global budget.  The old code
                            // had only the first shape of limit and applied
                            // it per segmentation, so `SEGMENTATION_KEEP`
                            // alignments each bought 64 wordings of a
                            // resegmentation the display policy fills after
                            // 17.  `emitted` counts this segmentation's
                            // wordings across every stage, and a replay
                            // advances none of the counters it is mirroring,
                            // so it stops in exactly the place the walk it
                            // replays stopped.
                            if emitted >= emit_allowance
                                || spent_emissions
                                    >= LEXICAL_GLOBAL_EMISSION_BUDGET
                            {
                                finished = true;
                                break;
                            }
                        }
                        continue;
                    }

                    if !replaying && popped >= LEXICAL_HEAP_POP_LIMIT {
                        finished = true;
                        break;
                    }
                    if replaying {
                        // Only a node that was expanded is worth widening: a
                        // leaf has no children, and its index tuple is
                        // already a wording.
                        expanded.push((k, prefix.clone()));
                    }
                    for i in 0..slots[k].len().min(cap) {
                        let mut next = prefix.to_vec();
                        next.push(i);
                        let next: Rc<[usize]> = Rc::from(next);
                        if seen.insert((k + 1, next.clone())) {
                            heap.push((quantized(bound(&next)), k + 1, next));
                        }
                    }
                }
                if finished {
                    break 'stages;
                }

                // The heap is empty.  Either the replay is done and the
                // wider stage is waiting for its new children, or this width
                // could not fill the traversal's appetite and the question
                // is whether to open the next one at all.
                if replaying {
                    cap = next_branch_stage(opened, widest)
                        .expect("a stage was opened only when one was available");
                    for (k, prefix) in expanded.drain(..) {
                        for i in opened..slots[k].len().min(cap) {
                            let mut next = prefix.to_vec();
                            next.push(i);
                            let next: Rc<[usize]> = Rc::from(next);
                            if seen.insert((k + 1, next.clone())) {
                                heap.push((quantized(bound(&next)), k + 1, next));
                            }
                        }
                    }
                    replaying = false;
                    continue 'stages;
                }
                if next_branch_stage(cap, widest).is_none() {
                    break 'stages;
                }
                if popped >= LEXICAL_HEAP_POP_LIMIT
                    || spent_pops + popped >= LEXICAL_GLOBAL_POP_BUDGET
                    || spent_emissions >= LEXICAL_GLOBAL_EMISSION_BUDGET
                {
                    break 'stages;
                }
                // Re-seed and replay at the *current* width, so the replay
                // expands exactly the nodes the exhausted walk expanded and
                // leaves exactly the frontier it left.  Only then is the
                // wider width opened, and only the newly legal children are
                // added — so the wider stage pops only nodes the narrow stage
                // never reached, instead of re-walking the lattice.
                opened = cap;
                heap.clear();
                seen.clear();
                seen.insert((0, root.clone()));
                heap.push((quantized(bound(&[])), 0, root.clone()));
                replaying = true;
            }
            spent_pops += popped;

            // ---- w-c1d3a7: the adjacency / neighbourhood operator ----
            //
            // The traversal above moves by *extending a prefix*, so a wording
            // that is jointly excellent but locally expensive in one slot
            // costs the sum of every better-bound node in front of it, and a
            // per-segmentation allowance of that size never reaches it.  This
            // operator moves by *substituting one slot of a complete wording*
            // instead, seeded from the wordings this segmentation has already
            // put in the pool, and re-scoring each child with the same
            // admissible `bound` the traversal orders by — so depth in one
            // slot is additive rather than multiplicative, and a slot index
            // far outside any width-capped prefix walk is open to it.  It
            // reads no vocabulary and names no phrase: the same call serves
            // every target and every segmentation.  The measurement that
            // motivates it is in `docs/work/items/w-c1d3a7.md`.

            let mut adjacency_emitted = 0usize;
            if adjacency_allowance > 0
                && spent_emissions < LEXICAL_GLOBAL_EMISSION_BUDGET
            {
                for tuple in adjacency::admit(
                    adjacency::Neighbourhood {
                        pops: adjacency::pop_budget(
                            pooled.len(),
                            adjacency_allowance,
                        ),
                        per_slot: ADJACENCY_PER_SLOT,
                    },
                    &pooled,
                    &widths,
                    &bound,
                ) {
                    if adjacency_emitted >= adjacency_allowance
                        || spent_emissions
                            >= LEXICAL_GLOBAL_EMISSION_BUDGET
                    {
                        break;
                    }
                    let Some(partial) = build(&tuple) else {
                        continue;
                    };
                    #[cfg(test)]
                    counters::note_depth(
                        &counters::DEEPEST_ADJACENCY,
                        tuple.iter().copied().max().unwrap_or(0),
                    );
                    pooled.push(tuple);
                    recovered.push(partial);
                    adjacency_emitted += 1;
                    spent_emissions += 1;
                    adjacency_spend -= 1;
                    // Deliberately *not* charged to `funded`.  That map is
                    // the traversal's per-structure depth account: it is what
                    // `depth_ceiling` is drawn against, so a word the
                    // traversal never emitted must not shrink a later
                    // segmentation of the same structure.  Charging it here
                    // double-counted the operator against the very budget it
                    // is extra, and cost a deep-in-a-span match
                    // (`approximate_pool_reaches_matches_deep_in_a_span`).
                    // The operator is bounded by `ADJACENCY_RESERVE` and by
                    // `LEXICAL_GLOBAL_EMISSION_BUDGET` instead, which is where
                    // its spend belongs.
                }
            }
        }

        completed.extend(recovered);
        self.finish(
            completed,
            &target_ipa,
            &target_boundaries,
            target_syllables,
        )
    }

    fn finish(
        &self,
        completed: Vec<Partial>,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_syllables: usize,
    ) -> Vec<Clue> {
        let mut clues: Vec<Clue> = completed
            .into_iter()
            .map(|p| {
                p.into_clue(
                    target_ipa,
                    target_boundaries,
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

        // ZZ_POOL_OUT (SCRATCH, scratch/2f7a10-verify only): dump the
        // deduplicated pool exactly as it stands here, i.e. after the sort and
        // after the `phrase_signature` dedup and immediately before
        // `select_diverse` admits or drops anything.  This is the pool the
        // release binary actually produces on the default path, so every
        // pool-membership number taken from this dump is admissible.  The
        // hunk is read-only, allocates only when the variable is set, touches
        // no counter and cannot be reached by the search; visible output is
        // byte-identical with it unset and set (proved in
        // docs/measure-2f7a10-verify.md).  Ported from scratch/2f7a10-base;
        // nothing was merged.
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(path) = std::env::var("ZZ_POOL_OUT") {
            use std::fmt::Write as _;
            let mut out = String::new();
            for (i, c) in clues.iter().enumerate() {
                let _ = writeln!(out, "{}\t{:.17}\t{:?}\t{:?}", i, c.score, c.phrase, c.cuts);
            }
            std::fs::write(&path, out).expect("zz pool dump");
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

/// Test-only instrumentation. Approximate search is a constant-factor
/// problem, so the regression tests count calls rather than trusting the
/// wall clock. Compiled out of release builds entirely.
#[cfg(test)]
mod counters {
    use std::cell::Cell;

    thread_local! {
        pub static METRICS: Cell<u64> = const { Cell::new(0) };
        pub static NOVELTY_STEM: Cell<u64> = const { Cell::new(0) };
        /// The deepest slot index the lexical traversal's own emissions
        /// reach, and the deepest one the reserved depth-profile emissions
        /// reach.  The two are the before/after of the emission-spread
        /// front in one place.
        pub static DEEPEST_TRAVERSAL: Cell<usize> = const { Cell::new(0) };
        pub static DEEPEST_PROFILE: Cell<usize> = const { Cell::new(0) };
        /// The deepest slot index the adjacency operator's admissions
        /// reach: the third of the three emission-spread counters.
        pub static DEEPEST_ADJACENCY: Cell<usize> = const { Cell::new(0) };
    }

    pub fn bump(counter: &'static std::thread::LocalKey<Cell<u64>>) {
        counter.with(|c| c.set(c.get() + 1));
    }

    pub fn take(counter: &'static std::thread::LocalKey<Cell<u64>>) -> u64 {
        counter.with(|c| c.replace(0))
    }

    /// Record `index` as the deepest slot seen, for one of the two
    /// depth counters.
    pub fn note_depth(
        counter: &'static std::thread::LocalKey<Cell<usize>>,
        index: usize,
    ) {
        counter.with(|c| {
            if index > c.get() {
                c.set(index);
            }
        });
    }

    pub fn take_depth(
        counter: &'static std::thread::LocalKey<Cell<usize>>,
    ) -> usize {
        counter.with(|c| c.replace(0))
    }
}

fn novelty_stem(word: &str) -> String {
    #[cfg(test)]
    counters::bump(&counters::NOVELTY_STEM);
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
    // This runs once per clue word for every hypothesis the search builds,
    // so it counts the normalized characters in place instead of
    // materializing the normalized string just to measure its length.
    let mut count = 0usize;
    let mut single: Option<char> = None;
    for c in word.chars() {
        if !c.is_alphanumeric() {
            continue;
        }
        for lower in c.to_lowercase() {
            if count == 0 {
                single = Some(lower);
            }
            count += 1;
        }
    }
    match count {
        0 => 0.0,
        1 if single == Some('a') || single == Some('i') => 1.0,
        1 => 0.05,
        2 => 0.35 + 0.55 * familiarity,
        _ => 1.0,
    }
}

/// The pre-optimization definition of [`lexical_shape_quality`], kept as
/// the reference the allocation-free version is tested against.
#[cfg(test)]
fn lexical_shape_quality_reference(word: &str, familiarity: f64) -> f64 {
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

/// One link of a `Partial`'s persistent clue-word list.
///
/// The beam extends a candidate by appending one word, and every
/// extension used to copy the whole `Vec<ClueWord>` — two `String`s per
/// clue word — so extending a path of W words cost O(W) allocations and
/// made a whole path cost O(W^2).  A linked list makes extension O(1)
/// and the beam shares the untouched prefix with every sibling.
#[derive(Debug)]
struct WordNode {
    word: ClueWord,
    /// The prefix this candidate had before `word` was appended.
    prev: Option<Rc<WordNode>>,
    /// Clue words up to and including this one.
    len: usize,
}

/// Iterator over a `Partial`'s clue words in clue order.
///
/// The list is built back to front, so the first `next` walks it from the
/// last word to the empty prefix onto `pending`; later calls pop from the
/// other end.  Only `into_clue` and the unit tests materialise the words
/// at all — the search itself only ever asks for the count.
struct WordIter<'a> {
    next: Option<&'a WordNode>,
    pending: Vec<&'a ClueWord>,
}

impl<'a> Iterator for WordIter<'a> {
    type Item = &'a ClueWord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pending.is_empty() {
            let mut node = self.next.take();
            while let Some(n) = node {
                self.pending.push(&n.word);
                node = n.prev.as_deref();
            }
        }
        self.pending.pop()
    }
}

#[derive(Debug, Clone)]
struct Partial {
    /// Tail of the persistent clue-word list, or `None` when empty.
    words: Option<Rc<WordNode>>,
    sub_cost_total: f64,
    /// Cached syllable total; `metrics` runs in every beam comparison.
    syllables: usize,
    /// Cached closed-class word count; `metrics` runs in every beam
    /// comparison and the test allocates, so it is not re-done there.
    closed: usize,
    cheap_score: f64,
    /// Target-stream offsets consumed at clue word boundaries.  Shared
    /// for the same reason as `words`: a candidate only ever grows this
    /// vector by one push.
    cuts: Rc<Vec<usize>>,
    /// Stable path key for duplicate suppression in approximate beams.
    key: String,
    /// Incremental per-word aggregates, so `metrics` does not re-derive
    /// them from `words` on every beam comparison. Each is the running
    /// total in clue-word order, which keeps the sums bit-identical to
    /// folding `words` at scoring time.
    reused_count: usize,
    familiarity_sum: f64,
    shape_sum: f64,
}

impl Partial {
    fn empty() -> Self {
        Self {
            words: None,
            sub_cost_total: 0.0,
            syllables: 0,
            closed: 0,
            cheap_score: 0.0,
            cuts: Rc::new(Vec::new()),
            key: String::new(),
            reused_count: 0,
            familiarity_sum: 0.0,
            shape_sum: 0.0,
        }
    }

    /// Clue words in this candidate, in clue order.
    fn words(&self) -> WordIter<'_> {
        WordIter {
            next: self.words.as_deref(),
            pending: Vec::new(),
        }
    }

    fn word_count(&self) -> usize {
        self.words.as_ref().map_or(0, |n| n.len)
    }

    fn extend_pronunciation(
        &self,
        target: &TargetPhrase,
        p: &Pronunciation,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        self.extend_parts(
            target,
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
        target: &TargetPhrase,
        word: &approx::FuzzyWord,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        self.extend_parts(
            target,
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
        target: &TargetPhrase,
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

        // Share the prefix instead of copying it: the one new `ClueWord`
        // is built here and every sibling of this candidate reuses the
        // whole list before it through the `Rc`.
        let word_node = Rc::new(WordNode {
            word: ClueWord {
                word: word.to_string(),
                ipa: ipa.to_string(),
                rarity,
                sub_cost: word_sub_cost,
            },
            prev: self.words.clone(),
            len: self.word_count() + 1,
        });

        let mut cuts = Vec::with_capacity(self.cuts.len() + 1);
        cuts.extend_from_slice(&self.cuts);
        let end = cuts.last().copied().unwrap_or(0) + consumed;
        cuts.push(end);

        // One allocation, built in place. `format!` built the same bytes
        // through two temporaries and grew the key from empty on every
        // extension.
        let lower = word.to_lowercase();
        let step_len = lower.len() + 1 + ipa.len() + 1;
        let mut key = String::with_capacity(if self.key.is_empty() {
            step_len - 1
        } else {
            self.key.len() + 1 + step_len - 1
        });
        if !self.key.is_empty() {
            key.push_str(&self.key);
            key.push(' ');
        }
        key.push_str(&lower);
        key.push('\u{1f}');
        key.push_str(ipa);

        // The aggregates below are the same terms metrics() used to fold
        // out of the word vector on every call. Appending one word to a running
        // total in clue-word order is bit-identical to re-folding the whole
        // vector, and costs O(1) instead of O(W).
        let familiarity = word_familiarity(rarity);
        let reuses = target.reuse.reuses(word);
        let shape = lexical_shape_quality(word, familiarity);

        Self {
            words: Some(word_node),
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            syllables: self.syllables + approx::ipa_syllables(ipa),
            closed: self.closed + usize::from(closed),
            cheap_score: self.cheap_score + word_bonus + rarity_penalty - word_sub_cost,
            cuts: Rc::new(cuts),
            key,
            reused_count: self.reused_count + usize::from(reuses),
            familiarity_sum: self.familiarity_sum + familiarity,
            shape_sum: self.shape_sum + shape,
        }
    }

    fn metrics(
        &self,
        target_boundaries: &[usize],
        target_syllables: usize,
        total_len: usize,
        partial: bool,
    ) -> Metrics {
        #[cfg(test)]
        counters::bump(&counters::METRICS);
        let similarity = (1.0 - self.sub_cost_total / 4.0).clamp(0.0, 1.0);
        let novelty =
            boundary_novelty(&self.cuts, target_boundaries, total_len, partial);

        let reused = self.reused_count as f64;
        let words = self.word_count();
        let word_novelty = 1.0 - reused / words.max(1) as f64;

        let familiarity = if words == 0 {
            0.0
        } else {
            self.familiarity_sum / words as f64
        };

        // A Mad Gab clue has to be *sayable* with the target's rhythm, not
        // merely built from similar phones.  Without this axis the search
        // happily "wins" by shredding the target into one- and two-phone
        // words: that matches acoustically, but it is not a resegmentation
        // anybody can say out loud.
        let rhythm = rhythm_match(self.syllables, target_syllables);

        let shape_quality = if words == 0 {
            0.0
        } else {
            self.shape_sum / words as f64
        };

        // A Mad Gab clue is a *puzzle answer*, so it also has to be
        // readable.  Every other lexical axis here is a function of
        // frequency or phone content, and frequency points the wrong
        // way: `the`, `a`, `it` and `each` are among the commonest words
        // in English, so the familiarity axis rewards precisely the
        // determiner salad no human would use as an answer.  This is the
        // one axis that asks which *class* of word was used, and it is
        // close to anti-correlated with `familiarity` by construction.
        let content = if words == 0 {
            0.0
        } else {
            1.0 - self.closed as f64 / words as f64
        };
        let closed_penalty = closed_class_penalty(self.closed as f64, words as f64);

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
        target_syllables: usize,
    ) -> Clue {
        let total_len = target_ipa.chars().count();
        let score = self
            .metrics(target_boundaries, target_syllables, total_len, false)
            .combined;
        let words: Vec<ClueWord> = self.words().cloned().collect();
        Clue {
            phrase: words
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            ipa: target_ipa.to_string(),
            words,
            score,
            // The cut vector is shared with every sibling of this
            // candidate, so the clue takes its own copy. This runs once
            // per retained clue, not once per beam extension.
            cuts: self.cuts.to_vec(),
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
    /// `SlotAlt::contribution`) and by the structural DP's own keys
    /// (`partial_span_score`): a one-word clue pays the full axis, and
    /// its edit cost is divided by that axis's own normaliser.
    pub const SIMILARITY_PER_WORD: f64 = SIMILARITY / 4.0;
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

/// The search's hot membership tests — the recovery enumeration's
/// `seen` prefixes and the approximate dedup keys — run millions of
/// times on short byte strings, where the default SipHash is the bulk of
/// the cost.  This is the FxHash mixing function: a multiply-and-rotate
/// over machine words.
///
/// It is a pure function of the bytes, with no per-process seed, so a
/// `HashSet` using it still iterates in the same order from one process
/// to the next.  It is only used for sets whose *iteration order is
/// never observed*; anything that lets a hash value reach the output
/// must keep the default hasher.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FxHasher {
    hash: u64,
}

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FxHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
    }
}

impl std::hash::Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while let Some((chunk, tail)) = rest.split_first_chunk::<8>() {
            self.add(u64::from_le_bytes(*chunk));
            rest = tail;
        }
        if !rest.is_empty() {
            let mut buf = [0u8; 8];
            buf[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(buf));
        }
        self.add(bytes.len() as u64);
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.add(n as u64);
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.add(n as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

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
    target_syllables: usize,
    total_len: usize,
) -> Vec<Partial> {
    if k == 0 || candidates.is_empty() {
        return Vec::new();
    }

    let score_of = |p: &Partial| {
        p.metrics(
            target_boundaries,
            target_syllables,
            total_len,
            true,
        )
        .combined
    };

    // Exact path duplicates (same words + same pronunciations) can be
    // generated through multiple edit alignments. Keep the better one.
    //
    // A duplicate path only has to be resolved against the incumbent's
    // combined score, so that score is scored once per *colliding* key and
    // then cached. Keys that never collide are never scored here at all:
    // every surviving candidate is scored exactly once below.
    let mut dedup: HashMap<String, Partial> = HashMap::new();
    let mut incumbent_score: HashMap<String, f64> = HashMap::new();
    for candidate in candidates {
        match dedup.get(&candidate.key) {
            Some(old) => {
                let old_score = *incumbent_score
                    .entry(candidate.key.clone())
                    .or_insert_with(|| score_of(old));
                let candidate_score = score_of(&candidate);
                if candidate_score > old_score {
                    incumbent_score.insert(candidate.key.clone(), candidate_score);
                    dedup.insert(candidate.key.clone(), candidate);
                }
            }
            None => {
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
            .entry((p.word_count().min(16), band.min(16), rhythm_band))
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

    // Order the retained indices against the cached metrics. Recomputing
    // `metrics` per comparison is the single largest cost in approximate
    // search: the results are already materialised above, so sorting the
    // indices keeps the same order for O(k log k) field reads.
    let mut picked: Vec<usize> = selected.into_iter().collect();
    picked.sort_by(|&a, &b| {
        cmp_desc(metrics[a].combined, metrics[b].combined).then(a.cmp(&b))
    });
    picked.into_iter().map(|i| items[i].clone()).collect()
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
// Bounds over the approximate lattice
// -----------------------------------------------------------------

/// Extremums of the score components contributed by a run of target
/// spans.  Each field is extremized in the direction that can only
/// *raise* the final score, so a run summarized this way dominates
/// every alignment it can actually produce.
///
/// Syllables are a range rather than an extremum, because the rhythm
/// axis reads a range: a clue whose chosen words sum to some syllable
/// count inside `min_syllables..=max_syllables` scores no worse than
/// one that lands on the closest end.
#[derive(Clone, Copy, Debug)]
struct SpanExtremes {
    min_cost: f64,
    max_familiarity: f64,
    max_shape: f64,
    min_closed: usize,
    min_reused: usize,
    min_syllables: usize,
    max_syllables: usize,
}

impl SpanExtremes {
    const ZERO: SpanExtremes = SpanExtremes {
        min_cost: 0.0,
        max_familiarity: 0.0,
        max_shape: 0.0,
        min_closed: 0,
        min_reused: 0,
        min_syllables: 0,
        max_syllables: 0,
    };

    /// Add a run in front of this one.  Used to relax a prefix into a
    /// whole path: each field is the best the concatenation can offer.
    fn upper_plus(self, other: SpanExtremes) -> SpanExtremes {
        SpanExtremes {
            min_cost: self.min_cost + other.min_cost,
            max_familiarity: self.max_familiarity + other.max_familiarity,
            max_shape: self.max_shape + other.max_shape,
            min_closed: self.min_closed + other.min_closed,
            min_reused: self.min_reused + other.min_reused,
            min_syllables: self.min_syllables + other.min_syllables,
            max_syllables: self.max_syllables + other.max_syllables,
        }
    }

    /// Join two candidate continuations by taking, per axis, the value
    /// that holds whichever one a completion picks.  Used to relax the
    /// rest of the target: any completion follows one path through the
    /// span DAG, so the bound has to hold for the best of the options,
    /// not the worst.  That means the `min_*` fields are minimized and
    /// the `max_*` fields maximized.
    fn best_of(self, other: SpanExtremes) -> SpanExtremes {
        SpanExtremes {
            min_cost: self.min_cost.min(other.min_cost),
            max_familiarity: self.max_familiarity.max(other.max_familiarity),
            max_shape: self.max_shape.max(other.max_shape),
            min_closed: self.min_closed.min(other.min_closed),
            min_reused: self.min_reused.min(other.min_reused),
            min_syllables: self.min_syllables.min(other.min_syllables),
            max_syllables: self.max_syllables.max(other.max_syllables),
        }
    }
}

/// Boundary novelty is the one axis that is not a sum over spans: it
/// compares the clue's inner cuts with the target's as sets.
///
/// The two counts cannot be chosen independently.  A clue inner cut that
/// lands on a target inner boundary is counted in *both* the shared set
/// and the clue's own set, so naming `added` the boundaries a completion
/// adds to the shared count and `clue_inner` the clue's final inner cut
/// count, the reachable triples satisfy
///
/// ```text
/// shared_final = shared + added
/// clue_inner  >= max(words - 1, shared, added)
/// ```
///
/// and the novelty is `1 - shared_final / (clue_inner + target_inner -
/// shared_final)`.  That ratio is *not* monotone in `added` on its own,
/// so rather than reason about which extreme wins, the maximum is taken
/// over the small set of reachable `added` values directly.  The result
/// is an upper bound, and because the search only ever uses it to discard
/// a span path, a loose one costs time and never quality.
fn novelty_upper_bound(
    words: usize,
    shared: usize,
    still_possible: usize,
    target_inner: usize,
) -> f64 {
    let reachable = still_possible.min(target_inner.saturating_sub(shared));
    let clue_floor = words.saturating_sub(1).max(shared);
    let mut best = 0.0_f64;
    for added in 0..=reachable {
        let clue_inner = clue_floor.max(added);
        let union = clue_inner + target_inner - shared - added;
        if union == 0 {
            continue;
        }
        let novelty = 1.0 - (shared + added) as f64 / union as f64;
        best = best.max(novelty);
    }
    best.clamp(0.0, 1.0)
}

/// Score of a span path that is still open, in the final scorer's own
/// weights.
///
/// Boundary novelty is deliberately absent.  It is a function of where
/// the path *ends*, so scoring an open path as if it had ended credits
/// it with a novelty it has not earned, and the shorter the path the
/// more inflated that credit is — which is precisely the bias that
/// makes a "rank" built this way prefer short paths.  Every other axis
/// is already fixed by the path taken so far.
fn partial_span_score(
    ext: SpanExtremes,
    words: usize,
    target_syllables: usize,
) -> f64 {
    let denom = words.max(1) as f64;
    axes::SIMILARITY_PER_WORD * (-ext.min_cost)
        + axes::FAMILIARITY * ext.max_familiarity / denom
        - axes::WORD_NOVELTY * ext.min_reused as f64 / denom
        + axes::RHYTHM
            * rhythm_match_in(
                ext.min_syllables,
                ext.max_syllables,
                target_syllables,
            )
        + axes::SHAPE * ext.max_shape / denom
        + axes::CLOSED_CLASS
            * closed_class_penalty(ext.min_closed as f64, denom)
}

/// Score of a *finished* span path, where boundary novelty is exact.
/// This is the same quantity the final scorer will compute for the best
/// alignment of this structure, so it is the right key for ordering
/// the structures the lexical enumeration expands.
fn complete_span_score(
    ext: SpanExtremes,
    words: usize,
    shared: usize,
    target_inner: usize,
    target_syllables: usize,
) -> f64 {
    let denom = words.max(1) as f64;
    let union = words.saturating_sub(1) + target_inner - shared;
    let novelty = if union == 0 {
        0.0
    } else {
        (1.0 - shared as f64 / union as f64).clamp(0.0, 1.0)
    };
    axes::SIMILARITY * (1.0 - ext.min_cost / 4.0).clamp(0.0, 1.0)
        + axes::NOVELTY * novelty
        + axes::WORD_NOVELTY * (1.0 - ext.min_reused as f64 / denom)
        + axes::FAMILIARITY * ext.max_familiarity / denom
        + axes::RHYTHM
            * rhythm_match_in(
                ext.min_syllables,
                ext.max_syllables,
                target_syllables,
            )
        + axes::SHAPE * ext.max_shape / denom
        + axes::CLOSED_CLASS
            * closed_class_penalty(ext.min_closed as f64, denom)
}

/// Admissible upper bound on the final score of *every* alignment
/// reachable through a span path.
///
/// `head` summarizes the spans already taken and `tail` summarizes every
/// way of finishing the target from where the path currently stands.
/// Relaxing both upward and taking the scorer's own weights gives, for
/// every axis, a value no reachable alignment can beat:
///
/// * `similarity` falls with cost, so the cheapest possible tail wins;
/// * `word_novelty` falls with reuse, so the least-reusing tail wins,
///   divided by the *smallest* word count the path can end at;
/// * `familiarity` and `shape` rise with the words chosen, so the
///   most favorable tail wins, again over the smallest word count;
/// * `rhythm` peaks when the syllable total lands on the target's, and
///   the reachable totals lie inside the relaxed range, whose closest
///   point to the target is what `rhythm_match_in` returns;
/// * `novelty` is bounded by [`novelty_upper_bound`];
/// * `closed_class` is *subtracted*, so an upper bound has to assume the
///   smallest penalty the completion could pay: the fewest forced
///   closed-class words over the *most* words it could still reach.
///   That is the one axis `denom` is wrong for — a small word count
///   inflates the closed-class share rather than deflating it.
///
/// Being sound rather than tight, this is the right key for discarding a
/// path outright and the wrong key for ranking one against another.
fn span_score_bound(
    head: SpanExtremes,
    tail: SpanExtremes,
    words: usize,
    shared: usize,
    still_possible: usize,
    target_inner: usize,
    target_syllables: usize,
) -> f64 {
    let ext = head.upper_plus(tail);
    let denom = words.max(1) as f64;
    // Every word consumes at least one IPA character, so a completion
    // cannot end at more than one word per remaining character.
    let widest = (words + still_possible).max(1) as f64;
    axes::SIMILARITY * (1.0 - ext.min_cost / 4.0).clamp(0.0, 1.0)
        + axes::NOVELTY
            * novelty_upper_bound(
                words,
                shared,
                still_possible,
                target_inner,
            )
        + axes::WORD_NOVELTY * (1.0 - ext.min_reused as f64 / denom)
        + axes::FAMILIARITY * ext.max_familiarity / denom
        + axes::RHYTHM
            * rhythm_match_in(
                ext.min_syllables,
                ext.max_syllables,
                target_syllables,
            )
        + axes::SHAPE * ext.max_shape / denom
        + axes::CLOSED_CLASS
            * closed_class_penalty(ext.min_closed as f64, widest)
}

// -----------------------------------------------------------------
// Final proposal diversity
// -----------------------------------------------------------------

/// Smallest number of structures a proposal may monopolise, and the
/// largest fraction of the visible list any one structure may hold.
///
/// These are *policy* parameters of [`select_diverse`], not weights
/// fitted to a target: a list of `n` slots is split between at least
/// `STRUCTURE_FLOOR` different resegmentations, and a single structure
/// is never given more than `1 / STRUCTURE_FLOOR` of the slots unless
/// the pool itself contains fewer structures than that.
const STRUCTURE_FLOOR: usize = 3;

/// The word-boundary structure of a clue: the phoneme offsets at which
/// one clue word ends and the next begins, as the search aligned them (see
/// [`Clue::cuts`]).  Two clues with the same structure are two
/// spellings of the same resegmentation, which is the redundancy that
/// matters for a Mad Gab list.
///
/// The last entry is the end of the target's stream, not an inner
/// boundary, so it is dropped.
///
/// These offsets used to be re-derived here by summing each clue word's
/// own IPA length.  That is equivalent only when every alignment is
/// phonetically exact: under an approximate alignment a word consumes a
/// run of the *target's* phonemes, so the accumulated offsets drift, and
/// every wording of one resegmentation was counted as its own structure.
/// Measured on real searches, that made the share cap in
/// [`select_diverse`] a no-op — for one target the 300 best-scoring
/// candidates were all wordings of a single resegmentation, and the
/// "represent the strong structures" step of the policy had nothing to
/// represent.  See
/// [../../docs/work/items/w-04f83f.md](../../docs/work/items/w-04f83f.md).
fn clue_structure(c: &Clue) -> Vec<usize> {
    c.cuts[..c.cuts.len().saturating_sub(1)].to_vec()
}

/// Pick `top_n` proposals under an ordered rule, highest score first.
///
/// The rule, in order, is:
///
/// 1. **Represent the strong structures.**  Within the visible cutoff
///    — the `top_n` best-scoring candidates the search found — take the
///    best-scoring candidate of every boundary structure, in descending
///    score order, until the list is full.  The list therefore always
///    spans the resegmentations the search actually found rather than
///    one per structure by accident of iteration order.  The cutoff is
///    what bounds this step: a resegmentation too weak to reach the
///    visible list does not get to spend a slot on it, however distinct
///    it is.
/// 2. **Then score, under a share cap.**  Walk the pool in descending
///    score order and admit any candidate whose boundary structure is
///    still under its [`STRUCTURE_FLOOR`]-th share of the list.  A
///    candidate is admitted on its own score with no diversity penalty
///    at all, so a near-equal alternative in an already-represented
///    structure is visible; the slots a saturated structure gives up go
///    to the best candidate of a structure that is under-filled, which
///    is the trade the policy exists to make.
/// 3. **Never return a short list.**  If the pool runs out with every
///    structure at its cap, the cap is dropped and the list is filled in
///    score order.
///
/// The difference from the previous policy is step 2.  A repeat of an
/// already-shown structure used to be charged `MMR_LAMBDA` times the
/// pool's score spread times a Jaccard overlap, i.e. up to `0.175 *
/// spread`.  For a real search the spread is tens of thousandths, so a
/// second member of a strong structure was priced far above any score
/// difference it could be compared against: a candidate that cleared
/// the visible cutoff was effectively unreachable, and the list was
/// neither the best wordings nor one-per-structure — it was a blend
/// that lost both.  Here a second member is admitted exactly when its
/// own score earns it a slot, so a near-equal alternative in a
/// represented structure is visible while a far-worse sibling is not,
/// and the share cap is what stops one structure from owning the list.
fn select_diverse(clues: Vec<Clue>, top_n: usize) -> Vec<Clue> {
    if top_n == 0 || clues.is_empty() {
        return Vec::new();
    }
    if clues.len() <= top_n {
        return clues;
    }

    // `finish` hands us candidates in descending score order; sort by
    // score (and phrase, for a deterministic tie-break) so the rule's
    // "descending score order" is a property of this function.
    let mut order: Vec<usize> = (0..clues.len()).collect();
    order.sort_by(|&a, &b| {
        cmp_desc(clues[a].score, clues[b].score)
            .then_with(|| clues[a].phrase.cmp(&clues[b].phrase))
    });

    let structures: Vec<Vec<usize>> = clues.iter().map(clue_structure).collect();
    let cutoff = &order[..top_n];

    let mut picked: Vec<usize> = Vec::with_capacity(top_n);
    let mut taken = vec![false; clues.len()];
    let mut counts: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut represented: HashSet<Vec<usize>> = HashSet::new();

    // 1. one representative per boundary structure, best first, within
    // the visible cutoff.
    for &i in cutoff {
        if picked.len() == top_n {
            break;
        }
        if represented.insert(structures[i].clone()) {
            admit(i, &structures, &mut picked, &mut taken, &mut counts);
        }
    }

    // 2. then score: walk the pool in descending score order and admit
    // any candidate whose structure still has room under the cap.  The
    // walk is over the whole pool, not just the cutoff, because a
    // structure-dominated cutoff cannot fill the list on its own: the
    // slots a structure gives up are taken by the best candidate of an
    // under-filled structure, which is exactly the trade the policy is
    // meant to make.
    //
    // The cap is a share of the *list*, and the number of structures it
    // is divided by is the number the search found in the whole pool,
    // not the number that happened to reach the cutoff.  A cutoff with a
    // single structure in it is a monoculture, not a reason to hand the
    // whole list to that structure.
    let available: HashSet<&[usize]> = structures.iter().map(Vec::as_slice).collect();
    let cap = share_cap(top_n, available.len());
    for &i in &order {
        if picked.len() == top_n {
            break;
        }
        if taken[i] {
            continue;
        }
        if counts.get(&structures[i]).copied().unwrap_or(0) < cap {
            admit(i, &structures, &mut picked, &mut taken, &mut counts);
        }
    }

    // 3. a short list is worse than an unbalanced one: if the pool is
    // exhausted with every structure at its cap, drop the cap.
    for &i in &order {
        if picked.len() == top_n {
            break;
        }
        if !taken[i] {
            admit(i, &structures, &mut picked, &mut taken, &mut counts);
        }
    }

    // The list is shown in score order; the rule above decided
    // membership, not presentation.
    picked.sort_by(|&a, &b| cmp_desc(clues[a].score, clues[b].score));

    let mut slots: Vec<Option<Clue>> = clues.into_iter().map(Some).collect();
    picked
        .into_iter()
        .map(|i| slots[i].take().expect("picked once"))
        .collect()
}

/// How many members of one boundary structure `top_n` slots may hold,
/// given how many structures the candidate pool offers.  With at least
/// `STRUCTURE_FLOOR` structures to choose from, no structure may take
/// more than a `1 / STRUCTURE_FLOOR` share; with fewer, the share is
/// widened to what the pool can support, which for `STRUCTURE_FLOOR`
/// structures is again `1 / STRUCTURE_FLOOR`.  At least two members are
/// always allowed, so a structure is never a one-shot.
fn share_cap(top_n: usize, available: usize) -> usize {
    let floor = STRUCTURE_FLOOR.min(available.max(1));
    top_n
        .div_ceil(floor)
        .max(2)
        .max(top_n.div_ceil(available.max(1)))
}

/// How many wordings of one boundary structure the lexical enumeration
/// always funds, whatever else the budget is doing.
///
/// This is [`share_cap`] evaluated against the *floor* rather than against
/// the number of structures a real pool turns out to hold, so it is the
/// tightest cap the display policy can impose and it is known before the
/// search runs.  It is the right breadth for the search because the two
/// numbers are two ends of the same contract: the enumeration is filling a
/// list of `top_n` slots, [`select_diverse`] is choosing what goes in them,
/// and a structure's share of that list is capped.  This many wordings of a
/// structure are therefore *selectable*; deeper than this is depth, which
/// exists to give rare resegmentations a chance at a strong wording rather
/// than to fill slots that are already filled.
///
/// Measured on five real targets at `--approximate --top 50`, the retained
/// pool holds 137-180 distinct structures of which 4-12 are visible, and
/// each visible structure is filled to its cap (17 = `share_cap(50, 180)`)
/// out of the 48-64 wordings of it that the old per-segmentation budget
/// produced.  The record is in `docs/work/items/w-6b2f04.md`.
fn structure_wording_allowance(top_n: usize) -> usize {
    share_cap(top_n, STRUCTURE_FLOOR)
}

/// Total wordings one boundary structure may be funded, out of the global
/// emission budget.
///
/// `budget` is the whole lexical phase's allowance and `structures` is how
/// many distinct boundary structures it has to cover, so this is an equal
/// share, rounded *down*: `structures * ceiling <= budget` holds exactly,
/// which is what makes the per-structure caps sum to at most the global
/// budget rather than exceeding it by a rounding artefact.
///
/// It is floored at `breadth`, so a pool with more structures than the
/// budget has room for still gets every structure a full turn — in that
/// case the budget, not this ceiling, is what binds.  The caller caps it
/// again at the per-segmentation ceiling, so no single alignment can absorb
/// a share meant for many.
fn structure_depth_ceiling(
    budget: usize,
    structures: usize,
    breadth: usize,
) -> usize {
    (budget / structures.max(1)).max(breadth)
}

fn admit(
    i: usize,
    structures: &[Vec<usize>],
    picked: &mut Vec<usize>,
    taken: &mut [bool],
    counts: &mut HashMap<Vec<usize>, usize>,
) {
    picked.push(i);
    taken[i] = true;
    *counts.entry(structures[i].clone()).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build the persistent clue-word list a `Partial` would have after
    /// extending an empty one with `words`, in order.
    fn word_chain(words: Vec<ClueWord>) -> Option<Rc<WordNode>> {
        let mut tail = None;
        let mut len = 0usize;
        for word in words {
            len += 1;
            tail = Some(Rc::new(WordNode {
                word,
                prev: tail,
                len,
            }));
        }
        tail
    }

    const TINY: &str = r#"{
        "cat":  { "rarity": 100, "ipa": { "cmu": "kæt" }, "alt_display": "CAT" },
        "kit":  { "rarity": 200, "ipa": { "cmu": "kɪt" } },
        "at":   { "rarity": 80,  "ipa": { "cmu": "æt" } },
        "ka":   { "rarity": 5000,"ipa": { "cmu": "kæ" } }
    }"#;

    /// The regression this change is about: a clue's resegmentation is
    /// the set of target-stream offsets the *search* cut at, and under an
    /// approximate alignment that is not the same thing as summing the
    /// clue words' own IPA lengths.
    ///
    /// The two words below differ from their target spans only in how
    /// many target phonemes they consumed, and that is only possible
    /// under a non-phonetic alignment: a clue word may carry material the
    /// target does not have, or leave some of the target's out.
    /// Re-deriving the offsets from the clue words therefore invents a
    /// structure the search never chose, and every consumer of the
    /// structure — the diversity policy's share cap, and the integration
    /// test that checks it — then groups wordings of one resegmentation
    /// as if they were unrelated.
    #[test]
    fn clue_structure_is_the_alignment_not_the_words_own_lengths() {
        let target_phrase = TargetPhrase::new("alpha beta");
        // Nine phonemes of target. The first word consumes five of them
        // while carrying three characters; the second consumes four while
        // carrying seven.
        let aligned = Partial::empty()
            .extend_parts(
                &target_phrase, "first", "abcde", None, false, 5, 0.0,
            )
            .extend_parts(
                &target_phrase, "second", "abcdefg", None, false, 4, 0.0,
            );
        assert_eq!(*aligned.cuts, vec![5, 9]);

        let clue = aligned.into_clue("abcdefghi", &[5, 9], 2);
        // The search cut at 5. The second clue word carries seven
        // characters, so summing the words' own lengths would have said 7.
        assert_eq!(clue_structure(&clue), vec![5]);

        let naive: Vec<usize> = {
            let mut at = 0usize;
            let mut cuts = Vec::new();
            for w in clue.words.iter().skip(1) {
                at += w.ipa.chars().count();
                cuts.push(at);
            }
            cuts
        };
        assert_eq!(
            naive,
            vec![7],
            "this fixture is only meaningful while the two notions differ"
        );
    }


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
        // Every word is three IPA characters, so the cumulative offsets
        // are just the running total of the word lengths.
        let cuts: Vec<usize> = words
            .iter()
            .scan(0usize, |at, w| {
                *at += w.ipa.chars().count();
                Some(*at)
            })
            .collect();
        Clue {
            phrase: phrase.to_string(),
            ipa: "bbb".to_string(),
            words,
            score,
            cuts,
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
        let map = structure_map(&pool);
        let picked = select_diverse(pool, 10);
        assert_eq!(picked.len(), 10);
        let distinct: HashSet<&Vec<usize>> =
            picked.iter().map(|c| boundaries_of(&map, c)).collect();
        assert!(
            distinct.len() >= 6,
            "expected proposals spanning several resegmentations, got {distinct:?}"
        );
    }

    /// The boundary structure of a synthetic pool.
    ///
    /// Structures are supplied by the fixture rather than derived, the
    /// way the search supplies them: a candidate's structure is the set
    /// of target-stream offsets it was aligned at, and these fixtures have
    /// no target stream.  Each word is three IPA characters, so
    /// `x y z` is [3, 6] and the neighbours are [3] and [3, 6, 9].
    fn structures_for(clues: &[Clue]) -> Vec<Vec<usize>> {
        clues
            .iter()
            .map(|c| c.cuts[..c.cuts.len().saturating_sub(1)].to_vec())
            .collect()
    }

    fn structure_map(clues: &[Clue]) -> HashMap<String, Vec<usize>> {
        clues
            .iter()
            .zip(structures_for(clues))
            .map(|(c, s)| (c.phrase.clone(), s))
            .collect()
    }

    fn boundaries_of<'a>(
        map: &'a HashMap<String, Vec<usize>>,
        c: &Clue,
    ) -> &'a Vec<usize> {
        map.get(&c.phrase)
            .expect("candidate came from this pool")
    }

    /// The defect this policy rule exists to fix: a second member of an
    /// already-represented structure is priced on its own score, not
    /// made unreachable.  A near-equal alternative is admitted; a much
    /// worse sibling of the same structure is not.
    #[test]
    fn near_equal_alternative_in_a_shown_structure_is_selected() {
        // Every word is three IPA chars, so "x y z" is the structure
        // [3, 6] and "x y" / "x y z w" are the neighbouring ones.
        let mut pool: Vec<Clue> = vec![
            // Structure [3, 6]: a leader, a near-equal alternative and a
            // far-worse sibling.
            clue("lead0 rr ss", 0.9300),
            clue("lead1 rr ss", 0.9296),
            clue("lead2 rr ss", 0.7000),
        ];
        // Six further structures, each with a good member, so the
        // near-equal alternative is competing for real slots.
        for s in 0..6usize {
            pool.push(clue(&format!("other{s} tt"), 0.9280 - s as f64 * 1e-3));
            pool.push(clue(&format!("more{s} tt uu"), 0.9100 - s as f64 * 1e-3));
        }

        let picked = select_diverse(pool, 6);
        let phrases: Vec<&str> = picked.iter().map(|c| c.phrase.as_str()).collect();
        assert!(
            phrases.contains(&"lead0 rr ss") && phrases.contains(&"lead1 rr ss"),
            "the best and the near-equal member of a shown structure must both be \
             visible; got {phrases:?}"
        );
        assert!(
            !phrases.contains(&"lead2 rr ss"),
            "a far-worse sibling of a shown structure must not be visible; got {phrases:?}"
        );
        assert_eq!(picked.len(), 6);
    }

    /// The share cap: no single structure may take more than its
    /// `STRUCTURE_FLOOR`-th share of the visible list, even when it owns
    /// the whole cutoff.
    #[test]
    fn one_structure_cannot_take_the_whole_list() {
        let top_n = 10;
        let mut pool: Vec<Clue> = (0..top_n)
            .map(|i| clue(&format!("own{i} rr ss"), 0.930 - i as f64 * 1e-4))
            .collect();
        // Two further structures, both weaker than the dominant one but
        // reachable inside the top-N window.
        for s in 0..6usize {
            pool.push(clue(&format!("more{s} tt uu vv"), 0.920 - s as f64 * 1e-4));
            pool.push(clue(&format!("other{s} tt"), 0.910 - s as f64 * 1e-4));
        }

        let map = structure_map(&pool);
        let picked = select_diverse(pool, top_n);
        assert_eq!(picked.len(), top_n);
        let owned = picked
            .iter()
            .filter(|c| *boundaries_of(&map, c) == vec![3, 6])
            .count();
        assert_eq!(
            owned,
            top_n.div_ceil(STRUCTURE_FLOOR),
            "the dominant structure must be held to its share"
        );
        let distinct: HashSet<&Vec<usize>> =
            picked.iter().map(|c| boundaries_of(&map, c)).collect();
        assert_eq!(distinct.len(), STRUCTURE_FLOOR, "got {distinct:?}");
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
        let target_boundaries = [2usize, 4, 6, 8];
        let mut p = Partial::empty();
        p.sub_cost_total = sub_cost_total;
        p.syllables = syllables;
        p.closed = closed;
        p.cuts = Rc::new(cuts.to_vec());
        p.words = word_chain(
            (0..words)
                .map(|i| ClueWord {
                    word: format!("w{i}"),
                    ipa: "abc".to_string(),
                    rarity: Some(1_000.0),
                    sub_cost: 0.0,
                })
                .collect(),
        );
        p.metrics(
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

    /// Build a synthetic candidate pool with distinct path keys so the
    /// dedup stage never has to score an incumbent twice.
    fn candidate_pool(target: &TargetPhrase, n: usize) -> Vec<Partial> {
        let mut pool = Vec::with_capacity(n);
        for i in 0..n {
            let word = approx::FuzzyWord {
                word: format!("word{i}"),
                ipa: "kæt".to_string(),
                ipa_len: 3,
                syllables: 1,
                rarity: Some(100.0 + i as f64),
                closed: false,
            };
            let consumed = 3;
            pool.push(
                Partial::empty()
                    .extend_fuzzy(target, &word, consumed, (i % 7) as f64 * 0.05),
            );
        }
        pool
    }

    /// P1 regression: beam retention must order the retained indices
    /// against the already-materialised metrics, not recompute
    /// `Partial::metrics` inside a comparator.
    ///
    /// With one `metrics` call per distinct candidate the count is exactly
    /// the pool size. A comparator that re-ran `metrics` would need
    /// O(n log n) calls, so this asserts the exact number rather than a
    /// wall clock.
    #[test]
    fn prune_partials_scores_each_candidate_exactly_once() {
        let target = TargetPhrase::new("wreck a nice beach");
        let boundaries = [3usize, 6, 9];
        for n in [64usize, 256, 1024, 4096] {
            // k grows with the pool, as it does in the search: the
            // completed-hypothesis pool prunes thousands of candidates at a
            // time, and a comparator that re-ran `metrics` would cost
            // 2 k log2 k extra calls there.
            let k = n / 4;
            let pool = candidate_pool(&target, n);
            counters::take(&counters::METRICS);
            let kept = prune_partials(pool, k, &boundaries, 4, 12);
            let calls = counters::take(&counters::METRICS);
            assert_eq!(
                calls,
                n as u64,
                "expected one metrics call per candidate for n={n}"
            );
            assert_eq!(kept.len(), k);
        }
    }

    /// The beam shares one persistent clue-word list between a candidate
    /// and all of its siblings, so the list has to come back in clue
    /// order and it has to *stop*: a candidate is walked more than once
    /// (scoring, then `into_clue`), and an iterator that restarts instead
    /// of ending allocates without bound.
    #[test]
    fn shared_word_list_yields_clue_order_and_ends() {
        let target = TargetPhrase::new("wreck a nice beach");
        let words = ["wreck", "a", "nice", "beach"];
        let mut p = Partial::empty();
        for w in words {
            let word = approx::FuzzyWord {
                word: w.to_string(),
                ipa: "abc".to_string(),
                ipa_len: 3,
                syllables: 1,
                rarity: Some(1_000.0),
                closed: false,
            };
            p = p.extend_fuzzy(&target, &word, 3, 0.0);
        }

        assert_eq!(p.word_count(), words.len());
        let got: Vec<&str> = p.words().map(|w| w.word.as_str()).collect();
        assert_eq!(got, words, "shared word list lost clue order");

        // The prefix a sibling shares is still intact and unchanged.
        let sibling = p
            .clone()
            .extend_fuzzy(
                &target,
                &approx::FuzzyWord {
                    word: "dune".to_string(),
                    ipa: "abc".to_string(),
                    ipa_len: 3,
                    syllables: 1,
                    rarity: Some(1_000.0),
                    closed: false,
                },
                3,
                0.0,
            );
        assert_eq!(p.word_count(), words.len(), "extension mutated its prefix");
        assert_eq!(sibling.word_count(), words.len() + 1);

        let mut iter = p.words();
        let seen = iter.by_ref().count();
        assert_eq!(seen, words.len());
        assert!(iter.next().is_none(), "word iterator restarted when drained");
    }

    /// The retained hypotheses come back ordered by combined score, and
    /// the best candidate in the pool leads. Retention is a portfolio
    /// rather than a plain top-k, so this checks the ordering the index
    /// sort has to preserve, not the membership rule.
    #[test]
    fn prune_partials_returns_best_scoring_candidates_first() {
        let target = TargetPhrase::new("wreck a nice beach");
        let boundaries = [3usize, 6, 9];
        let pool = candidate_pool(&target, 64);
        let best = pool
            .iter()
            .map(|p| p.metrics(&boundaries, 4, 12, true).combined)
            .fold(f64::NEG_INFINITY, f64::max);
        let kept = prune_partials(pool, 8, &boundaries, 4, 12);
        let got: Vec<f64> = kept
            .iter()
            .map(|p| p.metrics(&boundaries, 4, 12, true).combined)
            .collect();
        assert!(got.windows(2).all(|w| w[0] >= w[1]), "not ordered by combined score: {got:?}");
        assert_eq!(got[0], best, "best candidate must lead the retained set");
    }

    /// P2 regression: the incremental aggregates on `Partial` must stay
    /// bit-identical to folding the whole word vector, so scores are
    /// unchanged by the refactor.
    #[test]
    fn incremental_aggregates_match_a_full_refold() {
        let target = TargetPhrase::new("wreck a nice beach");
        let boundaries = [3usize, 6, 9];

        // The pre-incremental `metrics`: fold every per-word aggregate out
        // of the word vector.
        fn refold(
            p: &Partial,
            target: &TargetPhrase,
            syllables: usize,
            partial: bool,
        ) -> Metrics {
            let similarity = (1.0 - p.sub_cost_total / 4.0).clamp(0.0, 1.0);
            let novelty =
                boundary_novelty(&p.cuts, &[3usize, 6, 9], 12, partial);
            let reused = p
                .words()
                .filter(|w| target.reuse.reuses(&w.word))
                .count() as f64;
            let count = p.word_count();
            let word_novelty = 1.0 - reused / count.max(1) as f64;
            let familiarity = if count == 0 {
                0.0
            } else {
                p.words()
                    .map(|w| word_familiarity(w.rarity))
                    .sum::<f64>()
                    / count as f64
            };
            let rhythm = rhythm_match(p.syllables, syllables);
            let shape_quality = if count == 0 {
                0.0
            } else {
                p.words()
                    .map(|w| {
                        let f = word_familiarity(w.rarity);
                        lexical_shape_quality(&w.word, f)
                    })
                    .sum::<f64>()
                    / count as f64
            };
            let closed = p.closed as f64;
            let content = if count == 0 {
                0.0
            } else {
                1.0 - closed / count as f64
            };
            let closed_penalty = closed_class_penalty(closed, count as f64);
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

        for p in candidate_pool(&target, 24) {
            let mut multi = p.clone();
            for extra in 0..4 {
                let word = approx::FuzzyWord {
                    word: format!("extra{extra}"),
                    ipa: "niːs".to_string(),
                    ipa_len: 3,
                    syllables: 1,
                    rarity: if extra % 2 == 0 {
                        None
                    } else {
                        Some(90_000.0)
                    },
                    closed: extra % 2 == 1,
                };
                multi = multi.extend_fuzzy(&target, &word, 3, 0.1 * extra as f64);
            }
            for p in [p, multi] {
                for partial in [true, false] {
                    assert_eq!(
                        p.metrics(&boundaries, 4, 12, partial),
                        refold(&p, &target, 4, partial)
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Reachability: the bounds that decide what the approximate search
    // is allowed to drop.
    // -----------------------------------------------------------------

    fn approximate_generator(top_n: usize) -> Generator {
        Generator::from_json(
            open_english_pronouncing_dictionary::CORPUS_JSON,
            GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n,
                ..GeneratorConfig::default()
            },
        )
        .unwrap()
    }

    /// The global lexical budget is shared out per boundary structure,
    /// and the sharing rule is arithmetic rather than chosen.
    ///
    /// Two properties, and only two, because the mechanism is exactly two
    /// numbers: the per-structure ceiling is an equal share of the global
    /// budget, and it never falls below the breadth the display policy can
    /// admit.  So the per-structure ceilings either sum to at most the
    /// global budget, or every structure is still guaranteed its full
    /// breadth and the budget is the binding constraint at the loop
    /// instead.
    #[test]
    fn structure_depth_ceiling_shares_the_global_budget() {
        const BUDGET: usize = 16_384;
        for breadth in [2usize, 4, 7, 17] {
            let mut previous = usize::MAX;
            for structures in 1..=4096 {
                let ceiling = structure_depth_ceiling(BUDGET, structures, breadth);
                assert!(ceiling >= breadth, "{structures} structures");
                assert!(
                    ceiling <= previous,
                    "ceiling grew at {structures} structures"
                );
                previous = ceiling;
                if ceiling > breadth {
                    assert!(
                        structures * ceiling <= BUDGET,
                        "{structures} structures at {ceiling} each exceed {BUDGET}"
                    );
                }
            }
        }
        // A pool with more structures than the budget can fund still gets
        // every structure a full turn.
        assert_eq!(
            structure_depth_ceiling(BUDGET, 100_000, 17),
            17
        );
    }

    /// The width a slot is opened at is the *first* stage, not a ceiling:
    /// the schedule has to be able to reach every alternative of the widest
    /// slot, so that the traversal can never be the reason a candidate is
    /// missing.  It must also stay logarithmic in the width, because a
    /// stage the traversal does not need is work nobody asked for.
    #[test]
    fn branch_stage_schedule_reads_every_alternative_in_log_stages() {
        let start = LEXICAL_BRANCH_STAGE_0;
        for widest in 1..=SPAN_SHORTLIST {
            let mut cap = start.min(widest);
            let mut stages = 0;
            while let Some(wider) = next_branch_stage(cap, widest) {
                assert!(wider > cap, "stage {cap}->{wider} on widest {widest}");
                cap = wider;
                stages += 1;
                assert!(
                    stages <= 8,
                    "widest={widest} needed {stages} stages to reach {cap}"
                );
            }
            assert_eq!(
                cap,
                widest,
                "the schedule must end at the widest slot, not short of it"
            );
        }
        // Geometric, and three stages span a 160-wide shortlist from 10.
        assert_eq!(next_branch_stage(10, SPAN_SHORTLIST), Some(40));
        assert_eq!(next_branch_stage(40, SPAN_SHORTLIST), Some(160));
        assert_eq!(next_branch_stage(SPAN_SHORTLIST, SPAN_SHORTLIST), None);
    }

    /// The rule is asked for a wider stage only when the traversal has run
    /// out of nodes at the current one, so a stage the traversal never needs
    /// is never opened: a slot no wider than the first stage, and a
    /// saturated one, both answer `None` rather than a wider number.
    #[test]
    fn branch_stage_schedule_never_offers_depth_the_lists_do_not_have() {
        for cap in 0..=SPAN_SHORTLIST {
            assert_eq!(
                next_branch_stage(cap, cap),
                None,
                "cap {cap} offered depth beyond {cap}"
            );
            for widest in 0..=cap {
                assert_eq!(
                    next_branch_stage(cap, widest),
                    None,
                    "cap {cap} with widest {widest} should be exhausted"
                );
            }
            for widest in (cap + 1)..=SPAN_SHORTLIST {
                let wider = next_branch_stage(cap, widest).expect("there is depth");
                assert!(
                    wider <= widest,
                    "cap {cap} offered {wider} for a widest slot of {widest}"
                );
            }
        }
    }

    /// The reserve's floor is the traversal's *first* stage, and this asserts
    /// exactly that much: every index the reserve can spend is one the
    /// traversal's opening stage cannot generate, and the reserve can still
    /// reach the deepest alternative the widest list has.
    ///
    /// It is deliberately **not** a claim that no index below the floor is
    /// ever spent twice.  [`next_branch_stage`] can open 40 and 160 as well,
    /// so a traversal that *widened* would overlap the reserve's range; the
    /// widening fires zero times on the six real multi-clause targets
    /// (`docs/work/items/w-9d4e17.md`), which is a measurement and is
    /// recorded as one on [`sweep_index`] rather than asserted here.
    #[test]
    fn the_coverage_sweep_starts_where_the_traversal_stops() {
        // Every index the reserve can spend is one the traversal's first
        // stage cannot generate, and the reserve can still reach the deepest
        // alternative the widest list has.  So the reserve's spend is
        // coverage and not a re-run of the traversal.
        for width in 0..=SPAN_SHORTLIST {
            let mut seen: HashSet<usize> = HashSet::new();
            for phase in 0..SPAN_SHORTLIST {
                let Some(at) = sweep_index(width, 16, 0, phase) else {
                    assert!(
                        width <= LEXICAL_BRANCH_STAGE_0,
                        "width {width} has no index the traversal misses"
                    );
                    continue;
                };
                assert!(
                    at >= LEXICAL_BRANCH_STAGE_0,
                    "width {width}: reserve spent at {at}, which the \
                     traversal's first stage already generates"
                );
                assert!(at < width, "width {width}: reserve walked off the end");
                seen.insert(at);
            }
            assert_eq!(
                seen.len(),
                width.saturating_sub(LEXICAL_BRANCH_STAGE_0).min(SPAN_SHORTLIST),
                "width {width}"
            );
        }
    }

    /// The breadth the enumeration guarantees is the breadth the display
    /// policy can admit, and it is derived from the same contract rather
    /// than restated.
    #[test]
    fn structure_breadth_is_the_display_policys_own_share_cap() {
        for top_n in [1usize, 5, 10, 20, 50, 200, 4096] {
            let breadth = structure_wording_allowance(top_n);
            assert_eq!(breadth, share_cap(top_n, STRUCTURE_FLOOR));
            assert!(breadth >= 2, "a structure is never a one-shot");
        }
        let mut previous = 0;
        for top_n in [1usize, 10, 20, 50, 200, 4096] {
            let breadth = structure_wording_allowance(top_n);
            assert!(breadth >= previous, "breadth shrank at top_n={top_n}");
            previous = breadth;
        }
    }

    /// What the change buys, asserted externally: approximate mode keeps
    /// spanning several boundary structures on real targets.
    ///
    /// Deliberately a floor and not an equality.  A tighter claim — that
    /// the visible list spans *more* structures than before — would be a
    /// measurement of one run rather than a property, and a claim about
    /// the depth of any one structure in the returned proposals would be
    /// broader than this mechanism, because the ordinary beam also
    /// contributes clues to the same pool.
    #[test]
    fn approximate_proposals_span_several_boundary_structures() {
        for (target, _) in reachability_corpus() {
            let clues = approximate_generator(50).generate(target);
            assert!(clues.len() >= STRUCTURE_FLOOR, "{target:?}");
            let structures: HashSet<Vec<usize>> =
                clues.iter().map(clue_structure).collect();
            assert!(
                structures.len() >= STRUCTURE_FLOOR,
                "{target:?}: {} visible slots span only {} boundary                  structures",
                clues.len(),
                structures.len()
            );
        }
    }

    /// The coverage reserve is bounded by named arithmetic, and the
    /// arithmetic is the whole of the mechanism.
    ///
    /// Four claims.  The reserve is a strict fraction of the per-segmentation
    /// allowance, so ordinary quality keeps the majority of it.  Every index
    /// it spends is one the traversal's first stage cannot generate.  Its
    /// candidate list is bounded by the number of slot subsets, which is what
    /// keeps the rule from needing an unbounded pool to work.  And its sweep
    /// is *uniform over the slot's list*: a systematic sample that leaves the
    /// interval between two sampled indices unsampled is not a sample, and
    /// that is the defect this rule exists to remove.
    #[test]
    fn depth_profile_reserve_is_bounded_by_named_arithmetic() {
        assert!(
            EMIT_PROFILE_RESERVE > 0
                && EMIT_PROFILE_RESERVE < LEXICAL_COMBINATIONS_PER_SEGMENTATION
                    / 2,
            "the reserve is a part of the allowance, not all of it"
        );
        assert!(EMIT_PROFILE_MAX_DEEP >= 1 && EMIT_PROFILE_MAX_DEEP <= 4);

        // The reserve is carved out of `emit_allowance` and the traversal
        // is handed the remainder, so the per-segmentation total is
        // unchanged whatever the reserve manages to afford.
        for emit_allowance in 1..=LEXICAL_COMBINATIONS_PER_SEGMENTATION {
            let profile_allowance = EMIT_PROFILE_RESERVE.min(emit_allowance);
            for spent in 0..=profile_allowance {
                assert_eq!(spent + (emit_allowance - spent), emit_allowance);
            }
        }

        // The reserve never exceeds its own allowance, every tuple is a
        // profile of at most `EMIT_PROFILE_MAX_DEEP` deep slots, and every
        // index it spends is one the traversal's own first stage cannot
        // generate.
        for depth in 1..=24usize {
            let widths = vec![SPAN_SHORTLIST; depth];
            for phase in 0..64usize {
                let tuples = coverage_tuples(
                    &widths,
                    EMIT_PROFILE_RESERVE,
                    EMIT_PROFILE_MAX_DEEP,
                    phase,
                );
                assert!(tuples.len() <= EMIT_PROFILE_RESERVE, "depth {depth}");
                for tuple in &tuples {
                    let deep: Vec<usize> = tuple
                        .iter()
                        .copied()
                        .filter(|&i| i != 0)
                        .collect();
                    assert!(!deep.is_empty(), "{tuple:?} is not a profile");
                    assert!(
                        deep.len() <= EMIT_PROFILE_MAX_DEEP,
                        "{tuple:?} is deeper than the reserve's own cap"
                    );
                    // Every index spent is one the traversal's own first
                    // stage cannot generate, and no coordinate walks off
                    // the end of its own list.
                    assert!(
                        deep.iter().all(|&i| i >= LEXICAL_BRANCH_STAGE_0),
                        "{tuple:?} re-spent the traversal's own width"
                    );
                    for (slot, &i) in tuple.iter().enumerate() {
                        assert!(i < widths[slot], "slot {slot} of {tuple:?}");
                    }
                }
                // Every slot is still touched by a one-deep shape before any
                // deeper shape is paid for, so a reserve smaller than the
                // number of slots cannot leave a slot uncovered.
                let singles: HashSet<usize> = tuples
                    .iter()
                    .filter(|t| t.iter().filter(|&&i| i != 0).count() == 1)
                    .map(|t| t.iter().position(|&i| i != 0).unwrap())
                    .collect();
                if depth <= EMIT_PROFILE_RESERVE {
                    assert_eq!(
                        singles.len(),
                        depth,
                        "depth {depth} phase {phase}: {tuples:?}"
                    );
                }
            }
        }

        // A slot no wider than the traversal's opening width drops out
        // rather than inventing an index, so the rule cannot walk off the
        // end of a list.
        let widths = [3usize, LEXICAL_BRANCH_STAGE_0 * 2];
        for phase in 0..64usize {
            for tuple in coverage_tuples(
                &widths,
                EMIT_PROFILE_RESERVE,
                EMIT_PROFILE_MAX_DEEP,
                phase,
            ) {
                for (slot, &i) in tuple.iter().enumerate() {
                    assert!(i < widths[slot], "slot {slot} of {tuple:?}");
                }
            }
        }

        // The sweep is a *cover*, not a sample: with the stride rounded up
        // and the start rotated, one sweep of `per` indices reaches the end
        // of the range and wraps, so a run's sweeps tile the whole list
        // rather than sampling a few points of it.  Asserted for widths
        // where `per` does and does not divide the span, because the
        // rounding is exactly what removes the gap.
        let per = EMIT_PROFILE_RESERVE;
        for width in (LEXICAL_BRANCH_STAGE_0 + 1)..=SPAN_SHORTLIST {
            let span = width - LEXICAL_BRANCH_STAGE_0;
            // Two consecutive rotations are enough to cover any remainder.
            let mut seen: HashSet<usize> = HashSet::new();
            for phase in 0..(span + per) {
                if let Some(at) = sweep_index(width, per, 0, phase) {
                    seen.insert(at);
                }
            }
            assert_eq!(
                seen.len(),
                span,
                "width {width} (span {span}, per {per}): the sweep left a gap"
            );
        }
    }

    /// A **pairing** is a point of a shape class's index rectangle, not a
    /// diagonal of it, and the reserve has to be able to say so.
    ///
    /// The rule it replaces gave every slot of a class *one* index, the
    /// narrowest slot's, so two slots of a class could never hold two
    /// different alternatives.  Every wording it emitted was therefore
    /// "deep in some slots, all at the same rank", and a resegmentation —
    /// which is deep in every slot, at ranks unrelated to one another — was
    /// not late in that reserve's order but absent from it.
    ///
    /// The widths below are the shape of a real five-slot segmentation: four
    /// slots with a full `SPAN_SHORTLIST` of alternatives and one span with
    /// only a handful, which is why the class containing it drops out.  Two
    /// things are asserted, and both are about the *shape* of the emitted
    /// tuples, with no word, phrase or target anywhere in them:
    ///
    /// * a tuple deep in three or more slots whose coordinates are not all
    ///   equal — a wording off the traversal's best alternative in several
    ///   slots at several unrelated ranks, which is what a pairing is; and
    /// * a tuple at the reserve's own maximum depth, which is what says the
    ///   depth cap is what bounds depth and not the order the classes are
    ///   walked in.
    #[test]
    fn depth_profile_reserve_emits_pairings_not_only_diagonals() {
        let widths = [SPAN_SHORTLIST, 7, SPAN_SHORTLIST, SPAN_SHORTLIST, 93];
        for phase in 0..64usize {
            let tuples = coverage_tuples(
                &widths,
                EMIT_PROFILE_RESERVE,
                EMIT_PROFILE_MAX_DEEP,
                phase,
            );
            let deep_of = |t: &[usize]| -> Vec<usize> {
                t.iter().copied().filter(|&i| i != 0).collect()
            };
            assert!(
                tuples
                    .iter()
                    .any(|t| deep_of(t).len() >= 3 && {
                        let deep = deep_of(t);
                        deep.iter().any(|&i| i != deep[0])
                    }),
                "phase {phase}: every tuple is a diagonal, so a pairing of \\
                 deep alternatives in different slots is not expressible; \\
                 the reserve emitted {tuples:?}"
            );
            assert!(
                tuples
                    .iter()
                    .any(|t| deep_of(t).len() == EMIT_PROFILE_MAX_DEEP),
                "phase {phase}: the reserve never reached its own maximum \\
                 depth, so the order of the classes is what bounds depth; \\
                 the reserve emitted {tuples:?}"
            );
        }
    }


    /// What the reserve buys, asserted externally: the depth-profile
    /// emissions reach further into a slot's candidate list than the
    /// traversal's own emissions do, on real targets, at the same
    /// per-segmentation allowance.
    ///
    /// **This front's bar changed, and the change is measured.** It used to
    /// read `traversal < LEXICAL_BRANCH_KEEP`, which was true by
    /// construction: the traversal could not even push a node at index 10.
    /// Since the width became a property of the traversal's own budget that
    /// is no longer a ceiling, and on two of the three corpus targets the
    /// traversal now *does* cross it — reaching index 38 on one and 31 on
    /// another, and emitting 65 and 41 wordings at index 10 or deeper.  So
    /// the traversal partly buys what the reserve buys, and the two spends
    /// share one allowance.
    ///
    /// What survives is the front's actual purpose, and it is asserted
    /// rather than assumed: the traversal's own emissions stay inside the
    /// width its first stage opens at, while the reserve reaches indices
    /// the traversal cannot generate at all.  The reserve's spend is
    /// therefore still coverage the traversal does not have, and the two
    /// spends share one allowance: the reserve spends first and is bounded
    /// by `EMIT_PROFILE_RESERVE` whatever the traversal does with the
    /// remainder.  The measured composition is in
    /// `docs/work/items/w-9d4e17.md`.
    #[test]
    fn depth_profile_emissions_reach_deeper_than_the_traversal() {
        for (target, _) in reachability_corpus() {
            let _ = approximate_generator(50).generate(target);
            let traversal = counters::take_depth(&counters::DEEPEST_TRAVERSAL);
            let profile = counters::take_depth(&counters::DEEPEST_PROFILE);
            assert!(
                traversal < profile,
                "{target:?}: the traversal reached slot depth {traversal}, \
                 past the reserve's own {profile}, so the reserve is no \
                 longer buying coverage the traversal lacks"
            );
            assert!(
                profile >= 4 * LEXICAL_BRANCH_STAGE_0,
                "{target:?}: the reserve only reached slot depth {profile}"
            );
        }
    }

    /// Score of an explicit word sequence, by cheapest alignment over
    /// the target stream, with the real final scorer.
    fn score_alignment(
        g: &Generator,
        target: &str,
        words: &[&str],
    ) -> Option<f64> {
        let (ipa, boundaries, syllables) =
            transcribe_with_boundaries(g.corpus(), target, true).unwrap();
        let chars: Vec<char> = ipa.chars().collect();
        let total = chars.len();
        let target_phrase = TargetPhrase::new(target);

        let mut at = 0usize;
        let mut partial = Partial::empty();
        for &w in words {
            let wanted = w.to_lowercase();
            let mut best: Option<approx::FuzzyMatch> = None;
            for m in g.fuzzy_lexicon.matches_at(&chars, at, 0.5, 1) {
                let word = g.fuzzy_lexicon.word(m.word_idx);
                if word.word.to_lowercase() != wanted {
                    continue;
                }
                if best.is_some() && best.unwrap().cost <= m.cost {
                    continue;
                }
                best = Some(m);
            }
            let m = best?;
            partial = partial.extend_fuzzy(
                &target_phrase,
                g.fuzzy_lexicon.word(m.word_idx),
                m.consumed,
                m.cost,
            );
            at += m.consumed;
        }
        if at != total {
            return None;
        }
        Some(partial.metrics(&boundaries, syllables, total, false).combined)
    }

    /// Per-span extremums over the *whole* fuzzy lattice (not the
    /// search's shortlist), and the suffix relaxation over the resulting
    /// span DAG.  Taking the extremums over everything the lattice
    /// offers keeps this reference strictly more generous than the
    /// search's own summary, so a violation is the search's fault.
    fn lattice_spans_and_tail(
        g: &Generator,
        target: &str,
    ) -> (Vec<Vec<(usize, SpanExtremes)>>, Vec<Option<SpanExtremes>>) {
        let (ipa, _, _) =
            transcribe_with_boundaries(g.corpus(), target, true).unwrap();
        let chars: Vec<char> = ipa.chars().collect();
        let n = chars.len();
        let mut spans: Vec<Vec<(usize, SpanExtremes)>> =
            (0..n).map(|_| Vec::new()).collect();
        for p in 0..n {
            let lattice = g.fuzzy_lexicon.matches_at(&chars, p, 0.5, 1);
            let mut ends: Vec<usize> =
                lattice.iter().map(|m| p + m.consumed).collect();
            ends.sort_unstable();
            ends.dedup();
            for end in ends {
                let mut ext = SpanExtremes {
                    min_cost: f64::INFINITY,
                    max_familiarity: 0.0,
                    max_shape: 0.0,
                    min_closed: 1,
                    min_reused: usize::MAX,
                    min_syllables: usize::MAX,
                    max_syllables: 0,
                };
                for m in lattice.iter().filter(|m| p + m.consumed == end) {
                    let word = g.fuzzy_lexicon.word(m.word_idx);
                    let familiarity = word_familiarity(word.rarity);
                    ext.min_cost = ext.min_cost.min(m.cost);
                    ext.max_familiarity =
                        ext.max_familiarity.max(familiarity);
                    ext.max_shape = ext.max_shape.max(
                        lexical_shape_quality(&word.word, familiarity),
                    );
                    ext.min_closed =
                        ext.min_closed.min(usize::from(word.closed));
                    ext.min_syllables =
                        ext.min_syllables.min(word.syllables);
                    ext.max_syllables =
                        ext.max_syllables.max(word.syllables);
                }
                // Reuse is a property of the target's own words, and the
                // lattice can contain both reusing and fresh words for
                // one span, so the minimum here is zero.
                ext.min_reused = 0;
                spans[p].push((end, ext));
            }
        }
        let mut tail: Vec<Option<SpanExtremes>> = vec![None; n + 1];
        tail[n] = Some(SpanExtremes::ZERO);
        for p in (0..n).rev() {
            let mut acc: Option<SpanExtremes> = None;
            for (end, ext) in &spans[p] {
                let Some(after) = tail[*end] else { continue };
                let joined = ext.upper_plus(after);
                acc = Some(match acc {
                    None => joined,
                    Some(so_far) => so_far.best_of(joined),
                });
            }
            tail[p] = acc;
        }
        (spans, tail)
    }

    /// Concrete alignments of real targets, to check the bounds against.
    /// Deliberately not restricted to what the search returns.
    fn reachability_corpus() -> Vec<(&'static str, Vec<Vec<&'static str>>)> {
        vec![
            (
                "It's just a stupid game",
                vec![
                    vec!["hits", "justice", "dupe", "hid", "came"],
                    vec!["it", "justice", "two", "end", "game"],
                    vec!["ich", "just", "a", "stoop", "a", "gave"],
                ],
            ),
            (
                "recognize speech",
                vec![
                    vec!["wreck", "a", "nice", "beach"],
                    vec!["wreck", "a", "now", "spits"],
                    vec!["reckon", "i", "speaks"],
                ],
            ),
            (
                "I love you",
                vec![
                    vec!["eye", "love", "you"],
                    vec!["alive", "views"],
                ],
            ),
        ]
    }

    #[test]
    fn novelty_upper_bound_dominates_every_reachable_jaccard() {
        for words in 1..8usize {
            for shared in 0..=3usize {
                for still in 0..8usize {
                    let ub = novelty_upper_bound(words, shared, still, 4);
                    for clue_inner in 0..words {
                        for target_shared in 0..=4usize {
                            // Only reachable triples: a boundary the path
                            // already shares is also one of the clue's own
                            // inner cuts, and the counts only grow.
                            if target_shared < shared
                                || clue_inner < shared
                                || target_shared - shared > clue_inner
                            {
                                continue;
                            }
                            let union = clue_inner + 4 - target_shared;
                            let novelty = if union == 0 {
                                0.0
                            } else {
                                1.0 - target_shared as f64 / union as f64
                            };
                            assert!(
                                ub + 1e-12 >= novelty,
                                "words={words} shared={shared} still={still} \
                                 clue_inner={clue_inner} \
                                 target_shared={target_shared}: \
                                 bound {ub} below {novelty}"
                            );
                        }
                    }
                }
            }
        }
    }

    /// P3 regression: the reuse test stems each word once and then serves
    /// every repeat from the memo, so the hot path neither stems nor
    /// allocates per call.
    #[test]
    fn reuse_test_stems_each_word_once() {
        let target = TargetPhrase::new("wreck a nice beach");
        counters::take(&counters::NOVELTY_STEM);
        // "wrecking" stems to "wreck", which is a target word.
        assert!(target.reuse.reuses("wrecking"));
        let first = counters::take(&counters::NOVELTY_STEM);
        assert_eq!(first, 1, "one stem computation for a cold word");
        for _ in 0..1000 {
            assert!(target.reuse.reuses("wrecking"));
        }
        assert_eq!(
            counters::take(&counters::NOVELTY_STEM),
            0,
            "repeated reuse tests must hit the memo, not re-stem"
        );
    }

    /// P3 regression: the allocation-free shape measure must agree with
    /// the definition it replaced, including for punctuation, case and
    /// non-ASCII letters.
    #[test]
    fn lexical_shape_quality_matches_reference() {
        for word in [
            "",
            "-",
            "a",
            "A",
            "i",
            "I",
            "x",
            "İ",
            "ab",
            "A-B",
            "cat",
            "straße",
            "Å",
            "å",
            "one two",
            "'twas",
        ] {
            for familiarity in [0.0, 0.25, 1.0] {
                assert_eq!(
                    lexical_shape_quality(word, familiarity),
                    lexical_shape_quality_reference(word, familiarity),
                    "shape quality for {word:?} at familiarity {familiarity}"
                );
            }
        }
    }

    /// The load-bearing property: the value the search computes for a
    /// span structure must be at least what the real final scorer gives
    /// every alignment of it, and the admissible bound must dominate
    /// even before the structure has finished.  If this fails, the
    /// search is discarding alignments for a reason that has nothing to
    /// do with how good they are.
    #[test]
    fn structural_bounds_dominate_the_real_scorer() {
        let g = approximate_generator(10);
        for (target, candidates) in reachability_corpus() {
            let (ipa, boundaries, syllables) =
                transcribe_with_boundaries(g.corpus(), target, true)
                    .unwrap();
            let chars: Vec<char> = ipa.chars().collect();
            let total = chars.len();
            let target_inner = boundaries
                .iter()
                .copied()
                .filter(|&b| b < total)
                .count();
            let (spans, tail) = lattice_spans_and_tail(&g, target);
            let tail = tail[0].expect("every target has a span path");

            for words in candidates {
                let Some(score) =
                    score_alignment(&g, target, &words)
                else {
                    continue;
                };

                // Rebuild the structure this alignment occupies.
                let mut at = 0usize;
                let mut head = SpanExtremes::ZERO;
                let mut shared = 0usize;
                let mut followed = true;
                for &w in &words {
                    let wanted = w.to_lowercase();
                    let found = g
                        .fuzzy_lexicon
                        .matches_at(&chars, at, 0.5, 1)
                        .into_iter()
                        .filter(|m| {
                            g.fuzzy_lexicon
                                .word(m.word_idx)
                                .word
                                .to_lowercase()
                                == wanted
                        })
                        .min_by(|a, b| {
                            a.cost.partial_cmp(&b.cost).unwrap_or(
                                std::cmp::Ordering::Equal,
                            )
                        });
                    let Some(m) = found else {
                        followed = false;
                        break;
                    };
                    let end = at + m.consumed;
                    let Some((_, ext)) =
                        spans[at].iter().find(|(e, _)| *e == end)
                    else {
                        followed = false;
                        break;
                    };
                    head = head.upper_plus(*ext);
                    if end < total && boundaries.contains(&end) {
                        shared += 1;
                    }
                    at = end;
                }
                assert!(
                    followed && at == total,
                    "{target:?} {words:?} does not follow the lattice"
                );

                let keyed = complete_span_score(
                    head,
                    words.len(),
                    shared,
                    target_inner,
                    syllables,
                );
                assert!(
                    keyed + 1e-12 >= score,
                    "{target:?} {words:?}: structure key {keyed} is below \
                     the real score {score}"
                );

                let bound = span_score_bound(
                    SpanExtremes::ZERO,
                    tail,
                    words.len(),
                    0,
                    total,
                    target_inner,
                    syllables,
                );
                assert!(
                    bound + 1e-12 >= score,
                    "{target:?} {words:?}: admissible bound {bound} is \
                     below the real score {score}"
                );
            }
        }
    }

    /// The property the whole front exists for: an alignment that the
    /// fuzzy lattice offers and that scores well enough to belong in the
    /// output must actually be in the pool the search enumerates.
    ///
    /// The reference is an independent beam over the fuzzy lattice, keyed
    /// on the *real* final scorer, so it does not share the search's span
    /// DP, its per-span shortlist or its lexical enumeration.  Whatever
    /// it finds at or above the top-`n` cutoff has to be present.
    #[test]
    fn high_scoring_lattice_alignments_survive_into_the_enumerated_pool() {
        /// Partials kept per target offset by the reference beam.  Wide
        /// enough that a good resegmentation is not pruned purely for
        /// having an expensive word in it.
        const REFERENCE_KEEP: usize = 24;

        for (target, _) in reachability_corpus() {
            let pool = approximate_generator(4096).generate(target);
            assert!(!pool.is_empty(), "{target:?} produced no clues");
            let mut ranked = pool.clone();
            ranked.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let cutoff = ranked[9].score;
            let signatures: HashSet<String> = pool
                .iter()
                .map(|c| phrase_signature(&c.phrase))
                .collect();

            let g = approximate_generator(10);
            let (ipa, boundaries, syllables) =
                transcribe_with_boundaries(g.corpus(), target, true)
                    .unwrap();
            let chars: Vec<char> = ipa.chars().collect();
            let total = chars.len();
            let target_phrase = TargetPhrase::new(target);
            let mut beam: Vec<Partial> = vec![Partial::empty()];
            let mut reference_best = f64::NEG_INFINITY;

            for at in 0..total {
                let mut next: Vec<Partial> = Vec::new();
                for partial in &beam {
                    if partial.sub_cost_total > 1.5 + 1e-9 {
                        continue;
                    }
                    let remaining = 1.5 - partial.sub_cost_total;
                    for m in
                        g.fuzzy_lexicon.matches_at(&chars, at, 0.5, 1)
                    {
                        if m.cost > remaining + 1e-9 {
                            continue;
                        }
                        next.push(partial.extend_fuzzy(
                            &target_phrase,
                            g.fuzzy_lexicon.word(m.word_idx),
                            m.consumed,
                            m.cost,
                        ));
                    }
                }
                next.sort_by(|a, b| {
                    let sa = a
                        .metrics(&boundaries, syllables, total, true)
                        .combined;
                    let sb = b
                        .metrics(&boundaries, syllables, total, true)
                        .combined;
                    cmp_desc(sa, sb).then_with(|| a.key.cmp(&b.key))
                });
                // Distinct alignments only: the reference is about which
                // resegmentations are reachable, not about how many
                // spellings of one resegmentation exist.
                let mut seen: HashSet<String> = HashSet::new();
                next.retain(|p| seen.insert(p.key.clone()));
                next.truncate(REFERENCE_KEEP);
                for p in &next {
                    if p.cuts.last().copied() == Some(total) {
                        let score = p
                            .metrics(&boundaries, syllables, total, false)
                            .combined;
                        reference_best = reference_best.max(score);
                        if score + 1e-12 >= cutoff {
                            let phrase: Vec<&str> =
                                p.words().map(|w| w.word.as_str()).collect();
                            let signature = phrase_signature(&phrase.join(" "));
                            assert!(
                                signatures.contains(&signature),
                                "{target:?}: the reference beam found \
                                 {signature:?} scoring {score}, at or \
                                 above the {cutoff} top-10 cutoff, but the \
                                 search did not enumerate it"
                            );
                        }
                    }
                }
                beam = next;
            }
            assert!(
                reference_best + 1e-12 <= ranked[0].score,
                "{target:?}: the search's best clue scores {} but an \
                 independent lattice beam reached {reference_best}",
                ranked[0].score
            );
        }
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
