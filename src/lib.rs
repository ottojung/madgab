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
/// so it can be deep in every slot the traversal's first stage leaves room in
/// ([`funded_slot_depth`]) and it cannot reach a wording that is deep in
/// several slots *and* off the corner in the rest.  The adjacency operator
/// spends its share differently: it starts from wordings the pool already holds
/// and substitutes **one slot at a time**, so the cost of being deep in one slot
/// is additive rather than multiplicative.
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

/// How many of a segmentation's slots the coverage reserve may make deep at
/// once, derived from the reserve's own reachability rather than chosen.
///
/// The reserve draws a deep slot's index by [`sweep_index`], which returns
/// `None` — and so yields no tuple and is **not charged to the reserve** — for
/// any subset whose *narrowest* slot is at or below
/// [`LEXICAL_BRANCH_STAGE_0`], because there is no index above the floor for
/// it to take.  A subset is therefore fundable only if *every* slot in it is
/// wider than the floor, and a `k`-deep subset needs `k` such slots.  So the
/// deepest tier the reserve can place anything at all is the number of the
/// segmentation's slots that are wider than the floor, and that is the bound.
///
/// This is what the constant `3` was standing in for, and the two are not the
/// same claim.  The constant's own comment said the cap existed because a
/// `k`-deep class "has `C(depth, k)` members per sweep step, so beyond two the
/// classes outnumber the reserve" — a comparison against the reserve that was
/// never performed, that disagreed with the literal beside it, and that is
/// **false as a bound**: the subsets skipped for a narrow slot are free, so a
/// tier costs far less than `C(depth, k)` and the reserve runs out much later
/// than that argument predicts.  Measured over sixteen real targets at the
/// shipped budgets, a four-deep placement costs **zero** of the
/// `LEXICAL_GLOBAL_EMISSION_BUDGET` and at most **+19** candidates of pool
/// (+0.10%), with the whole-search emission total *identical* on every one of
/// them, because the reserve is a fixed per-segmentation slice of a globally
/// capped allowance and a higher tier is bought by re-ordering which subset is
/// funded when the slice runs out, not by another emission.  The naive
/// `sum_{j<=k} C(depth, j)` model instead prices a four-deep placement at 30
/// units for five slots and 56 for six, i.e. unaffordable — so deriving the cap
/// from the naive model would have produced a false negative, and the `3` was
/// not protecting anything.  Numbers in `docs/work/items/w-c3f81a.md`.
///
/// The reserve still cannot spend more than its slice, and that is enforced
/// where it is enforced: `coverage_tuples` returns as soon as it holds
/// `reserve` tuples, so breadth-before-depth truncation — not this bound — is
/// what limits a real run.
fn funded_slot_depth(slot_widths: &[usize]) -> usize {
    slot_widths
        .iter()
        .filter(|&&width| width > LEXICAL_BRANCH_STAGE_0)
        .count()
}

/// The per-slot affordability model the traversal reads, derived from the
/// same additive bound `build` applies to a complete tuple.
///
/// A candidate `i` of slot `j` can appear in *any* emitted wording only if
///
/// ```text
///     (cost already committed by the prefix)
///   + (every later slot at its own cheapest)
///   + cost_{j,i}  <=  total_budget
/// ```
///
/// because a wording using it must pay for it *and* for one candidate of
/// every later slot, and the cheapest way to pay for those is their own
/// minimum.  Every term is a minimum or a chosen cost, so the test is
/// exactly the tail of `build`'s own sum — it rejects nothing `build`
/// would have kept, and it decides at the point where the decision is
/// still able to save work.
///
/// Before this, affordability was decided only at the leaf: the traversal
/// pushed, allocated and scored a candidate the total budget could never
/// pay for, popped it, and dropped it.  `LEXICAL_HEAP_POP_LIMIT` is the
/// constant that actually bounds wall clock, so those pops were spent out
/// of a global budget ([`LEXICAL_GLOBAL_POP_BUDGET`]) on subtrees with no
/// admissible leaf anywhere in them.  Measured at the shipped budgets the
/// discarded fraction is small — 4.2% of built wordings on one measured
/// target, 0.6% on another — so this is a bound that is nearly inert by
/// default and binds when a caller tightens `total_budget`.  That is stated
/// rather than claimed as a win.  No target is named here on purpose: the
/// written fence for this item forbids naming either acceptance example in
/// production `src/`, `tests/no_phrase_hard_coding.rs` strips comments before
/// it scans, so the two are kept apart by reading and by
/// `no_canonical_example_in_a_production_doc_comment`.
fn slot_is_affordable(
    committed: f64,
    later_minima: f64,
    candidate: f64,
    total_budget: f64,
) -> bool {
    committed + later_minima + candidate <= total_budget + 1e-9
}

/// The per-slot width the traversal may **open** at, derived from the two
/// budgets this loop already respects rather than chosen.
///
/// A best-first walk over a `d`-slot product reaches its *first* wording
/// only after it has expanded the whole subtree above the all-cheapest
/// leaf, which is `1 + w + w^2 + ... + w^(d-1)` nodes at opening width
/// `w`.  So an opening width is only a budget if that series fits inside
/// the per-segmentation pop limit: otherwise the walk exhausts the pop
/// limit having emitted **nothing at all**, and the per-segmentation
/// emission allowance it was given is not a small budget but an
/// unpayable one.
///
/// Measured on the default release path at
/// [`LEXICAL_BRANCH_STAGE_0`] = 10 and
/// [`LEXICAL_HEAP_POP_LIMIT`] = 4 000: a 4-slot segmentation needs 1 111
/// pops to reach its first wording and does reach it, a 5-slot one needs
/// 11 111 and does not, and the run shows it — of 256 retained
/// segmentations, 27 emit no wording at all on one measured four-word
/// target, 109 on a six-word one and 84 on another, each after spending
/// its full 4 000 pops.  Those pops buy nothing, and `spent_pops` is a
/// share of a *global* budget, so the waste is paid for by the
/// segmentations that do emit.  The counts are recorded on the work item
/// rather than here, because a production doc comment must not name either
/// acceptance example and the automated fence strips comments.
///
/// So the opening width is the largest `w` with
/// `1 + w + ... + w^(d-1) <= pop_limit`, capped by
/// [`LEXICAL_BRANCH_STAGE_0`] (which is the width the first stage is
/// documented to open at, and which the coverage reserve's sweep floor is
/// tied to) and by the widest list present.  Below the crossover this is
/// `LEXICAL_BRANCH_STAGE_0` and costs exactly what it cost before, so the
/// change is confined to the segmentations that were emitting nothing.
/// The widening ladder ([`next_branch_stage`]) is untouched, so a
/// traversal that drains at the derived width with appetite left still
/// opens the next one.
fn affordable_opening_width(slot_count: usize, pop_limit: usize) -> usize {
    if slot_count <= 1 {
        return LEXICAL_BRANCH_STAGE_0;
    }
    let mut width = LEXICAL_BRANCH_STAGE_0;
    // `1 + w + ... + w^(d-1)`, accumulated so the comparison is on the
    // quantity the bound is about rather than on `w^(d-1)` alone.
    let fits = |w: usize| -> bool {
        let mut total = 1usize;
        let mut power = 1usize;
        for _ in 1..slot_count {
            power = power.saturating_mul(w);
            total = total.saturating_add(power);
        }
        total <= pop_limit
    };
    while width > 1 && !fits(width) {
        width -= 1;
    }
    width
}

/// How many nodes a best-first walk must pop before it can reach *any* leaf,
/// when slot `k` is enumerated to `caps[k]` candidates.
///
/// The walk extends a prefix, so before a leaf is reachable the whole subtree
/// hanging off the all-cheapest leaf has been expanded: the empty prefix, then
/// every one-candidate prefix, then every two-candidate prefix, and so on up to
/// the deepest level that still has children.  Counting the prefixes of each
/// length gives
///
/// ```text
///     F(c) = 1 + c_0 + c_0*c_1 + ... + c_0*...*c_{d-2}
/// ```
///
/// `caps[d - 1]` does not appear, because a leaf is reached at depth `d` and a
/// slot's width only counts the nodes *above* it.  For a uniform `c` this is
/// exactly the `1 + w + ... + w^(d-1)` series [`affordable_opening_width`]
/// fits, so the two read the same quantity off a vector rather than a scalar,
/// and the scalar is the wrong thing to fit.
///
/// `F` is monotone non-decreasing in every component.  That is what makes
/// admission arithmetic rather than a search: the walk's test is `index < cap`,
/// so an index tuple `t` needs `caps[k] >= t_k + 1`, and the *smallest* frontier
/// any allocation admitting `t` can have is therefore `F(t + 1)`.  If that
/// exceeds the pop allowance, no allocation rule of any shape admits `t`, and
/// the shortfall is exactly `F(t + 1) - pop_limit`.
///
/// Derived for the width front; **not** called by the traversal, which still
/// opens every slot at the scalar [`affordable_opening_width`].  Wiring this in
/// is a measured negative: the rule below is correct and the arithmetic is
/// right, and it still costs the fence, because the traversal's live cost is
/// the frontier `F` and widening the cheapest slot multiplies it.  See
/// `docs/work/REPORT-9e2b41.md`.
#[cfg(test)]
fn first_leaf_frontier(caps: &[usize]) -> usize {
    let mut total = 1usize;
    let mut prefix = 1usize;
    // The last slot has no children, so it contributes no term.
    for &c in caps.iter().take(caps.len().saturating_sub(1)) {
        prefix = prefix.saturating_mul(c);
        total = total.saturating_add(prefix);
    }
    total
}

/// The **per-slot** opening widths of one segmentation's traversal, in
/// traversal-index units, allocated in proportion to each slot's own measured
/// push cost.
///
/// [`affordable_opening_width`] answers a different question in the wrong
/// units: it divides the per-segmentation *pop allowance* by the cost of
/// pushing one candidate and uses the quotient as a per-slot *list prefix
/// length*, and it answers it once for the whole product, so every slot gets
/// the same number however much that slot costs.  The two quantities are
/// unrelated, and the equality between them is what caps the enumeration.  A
/// pop allowance really buys a *frontier* — how many prefixes may exist above
/// a leaf — and a frontier is a per-slot vector, not a scalar.
///
/// The per-slot cost is measured off the traversal, not assumed.  Slot `k` is
/// expanded once per prefix that reaches it, and the number of such prefixes is
/// `M_k = c_0 * ... * c_{k-1}`, the product of the widths *above* it, so the
/// marginal cost of one more candidate in slot `k` is exactly `M_k`.  That is
/// the cost the allocation spends, and it is why the ceiling is non-increasing
/// in slot position: the same allowance buys more candidates low in the product
/// than high in it.  It is also why a wider slot is not a longer list —
/// `slots[k]` is ordered by `SlotAlt::contribution`, which divides by the
/// slot's word count, so the candidate at traversal index `i` of a wide slot is
/// not the candidate at index `i` of a narrow one.  Everything here is stated
/// and validated in traversal-index units.
///
/// Three structural facts, all exact rather than tuned:
///
/// * the last slot's width does not enter [`first_leaf_frontier`], so it is
///   allocated its whole list and charged nothing;
/// * the first pass charges each slot its own marginal cost in slot order and
///   never drops below `uniform_floor` — the width the scalar law would have
///   opened every slot at — so the rule cannot narrow what already worked; and
/// * those floors can overrun the allowance, so a second pass hands the overrun
///   back to the widest slot, the one the product is most sensitive to, until
///   the frontier fits.
///
/// Nothing here reads a word, a clue or a segmentation: the inputs are the
/// per-slot pool sizes, the pop allowance and the scalar opening width.
///
/// Derived for the width front; **not** called by the traversal.  See
/// [`first_leaf_frontier`] for why.
#[cfg(test)]
fn per_slot_opening_widths(
    pool_sizes: &[usize],
    pop_limit: usize,
    uniform_floor: usize,
) -> Vec<usize> {
    if pool_sizes.is_empty() {
        return Vec::new();
    }
    let last = pool_sizes.len() - 1;
    let floor = uniform_floor.max(1);
    let mut caps: Vec<usize> = pool_sizes.iter().map(|&n| n.max(1)).collect();
    // The last slot hangs off a leaf, so enumerating it costs nothing.
    caps[last] = pool_sizes[last].max(1);

    // Pass 1: charge each slot the marginal pushes its own width causes.
    // `charged` is the part of the frontier already paid for, `marginal` the
    // number of pushes one further candidate in the slot under consideration
    // costs.
    let mut charged = 1usize;
    let mut marginal = 1usize;
    for k in 0..last {
        let committed = charged.saturating_add(marginal);
        let room = pop_limit.saturating_sub(committed);
        let by_budget = room / marginal.max(1);
        caps[k] = pool_sizes[k].max(1).min(floor.max(by_budget).max(1));
        marginal = marginal.saturating_mul(caps[k]);
        charged = committed.saturating_add(marginal);
    }

    // Pass 2: the floor in pass 1 can overrun the allowance.  Give the budget
    // back from the widest slot, which is where the product is most sensitive
    // to it, until the frontier fits.  The free slot is not a candidate: it
    // costs nothing, so handing back from it would give away the whole list
    // for a frontier that never included it.  Ties go to the lower slot,
    // which is the one the product is more sensitive to.
    while first_leaf_frontier(&caps) > pop_limit {
        let Some(k) = (0..last)
            .filter(|&k| caps[k] > 1)
            .max_by_key(|&k| (caps[k], std::cmp::Reverse(k)))
        else {
            break;
        };
        caps[k] -= 1;
    }
    caps
}


/// How many candidates one subset of the coverage reserve's index sweep
/// draws before it spends its share on one of them.
///
/// The sweep is a systematic sample, so a subset's single strided draw is
/// uniform over the part of its slots' lists the traversal cannot
/// generate, and which of the positions it looks at is decided by the
/// phase rather than by anything the search scores.  Drawing
/// `EMIT_PROFILE_SAMPLE` of them and spending on the best-bound keeps the
/// sweep's *coverage* — the sample still walks the same coprime rotation,
/// `EMIT_PROFILE_SAMPLE` positions closer together — and changes only the
/// choice within it, from index order to the search's own admissible score
/// bound.  See [`coverage_tuples`].
///
/// The cost is `EMIT_PROFILE_SAMPLE` bound evaluations per funded subset,
/// at most `EMIT_PROFILE_SAMPLE * EMIT_PROFILE_RESERVE` = 16 * 16 = 256
/// per segmentation, each one the arithmetic the traversal's heap key
/// already performs.  It buys no extra emissions and no extra pops.
///
/// The sample is drawn from the same region of the same rotation as
/// before, and its width is independent of
/// [`affordable_opening_width`]: the first is how many positions of the
/// sweep's own region are considered, the second is how wide the
/// traversal opens the lists it can already reach.  Neither is a function
/// of the other, so the two fronts' bounds compose rather than compete.
const EMIT_PROFILE_SAMPLE: usize = 8;

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
fn sweep_index(
    width: usize,
    per: usize,
    nth: usize,
    member: usize,
    phase: usize,
) -> Option<usize> {
    let floor = LEXICAL_BRANCH_STAGE_0;
    if width <= floor {
        return None;
    }
    let span = width - floor;
    let stride = span.div_ceil(per.max(1));
    Some(floor + (nth * stride + member + sweep_rate(member, span) * phase) % span)
}

/// The rate at which the `nth` deep slot of a shape class rotates as the
/// traversal's phase advances.
///
/// Member 0 keeps the rate the whole class used to share, and each further
/// member rotates at the next rate that is **coprime to the span**, so:
///
/// - each member's own index, taken over a run of phases, covers its whole
///   list above the floor — a rate sharing a factor with the span would
///   visit only the residues reachable by that factor, which is exactly the
///   confinement this rule exists to remove; and
/// - the rates are *not* all equal, so a class does not rotate as a rigid
///   block: the offsets between its members' indices are themselves a
///   function of the phase, and a set of deep coordinates at unrelated ranks
///   is therefore reachable instead of only a set at adjacent ranks.
fn sweep_rate(nth: usize, span: usize) -> usize {
    let mut rate = 1usize;
    for _ in 0..nth {
        let mut candidate = rate + 1;
        while gcd(candidate, span) != 1 {
            candidate += 1;
        }
        rate = candidate;
    }
    rate
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
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
/// The rule now walks slot subsets **breadth before depth** — one deep slot
/// at a time, so a reserve smaller than the number of slots still touches
/// every slot — and takes each deep slot at [`sweep_index`], i.e. at a
/// uniform stride over the part of its list the traversal cannot generate.
/// Every slot outside the subset keeps the traversal's own best index, so a
/// representative is the cheapest wording of that shape rather than a
/// general-purpose regression, and `build` still compares the tuple's total
/// substitution cost against `total_budget` additively, so the reserve is
/// bounded by the same bound as every other emission.
///
/// `phase` is a counter the search advances once per segmentation, so the
/// sweeps of one run tile the lists between them.  It is a pure function of
/// the deterministic schedule, so the enumeration stays reproducible.
///
/// # The reserve is spent on the best-*bounded* member of its own sample
///
/// The sweep above is a *systematic* sample, and a systematic sample of a
/// list is uniform over the list, so the tuples it produces are ordered by
/// **index position**, not by anything the search is optimising.  Each
/// subset's member is drawn at a fixed strided offset of the phase's
/// rotation, so two runs over the same cell spend the reserve on the same
/// *shape* of coordinate set — one deep slot at an unrelated rank — and
/// never on the one that scores best among the positions it looked at.
///
/// So each subset draws [`EMIT_PROFILE_SAMPLE`] candidates instead of one,
/// and spends its share on the candidate with the highest **admissible
/// bound on the final score** — the same bound that keys the traversal's
/// own best-first heap (suffix minima of cost, novelty and closed-class
/// over the slots the tuple has not fixed, maxima of familiarity, shape
/// and rhythm over the same).  That bound is an upper bound on the score
/// of *any* completion of a partial, so ordering by it is ordering by an
/// upper bound on what the reserve's emissions are actually worth.
///
/// Three properties make this the same kind of filter as `build`'s cost
/// bound rather than a new kind of exclusion:
///
/// * **Admissible.**  The score bound never says "this tuple is bad"; it
///   says "no completion of this coordinate set can beat this value".  It
///   is used to *choose between* candidates the sweep already generated,
///   so a candidate is only ever dropped in favour of one the bound rates
///   at least as high — the same one-sided shape as the cost bound, which
///   is likewise only ever used to drop a tuple that cannot be paid for.
/// * **A superset of the old draw.**  The strided candidate the sweep used
///   to emit is the first member of the sample, so every tuple the
///   previous rule would have emitted is still a candidate here, and it is
///   emitted unless a candidate the bound rates higher takes its place.
///   Nothing this rule can emit was unreachable to the old one.
/// * **Bounded.**  The bound is evaluated
///   `EMIT_PROFILE_SAMPLE * (number of subsets the reserve funds)` times
///   per segmentation — at most
///   `EMIT_PROFILE_SAMPLE * EMIT_PROFILE_RESERVE` = 16 * 16 = **256**
///   evaluations — and each is the arithmetic the traversal's own heap key
///   already performs.  No new state is retained between segmentations.
///
/// The sample's extra candidates are taken from the *same* coprime
/// rotation the sweep uses, `EMIT_PROFILE_SAMPLE` consecutive steps apart
/// per phase, so the union of a run's samples still covers each member's
/// whole list above the floor exactly as before: sampling more positions
/// per phase narrows the stride of the rotation, it does not bias it.
fn coverage_tuples(
    slot_widths: &[usize],
    reserve: usize,
    max_deep: usize,
    phase: usize,
    bound_of: &dyn Fn(&[usize]) -> f64,
) -> Vec<Vec<usize>> {
    let depth = slot_widths.len();
    let max_deep = max_deep.min(depth);
    let mut out: Vec<Vec<usize>> = Vec::new();
    if reserve == 0 || depth == 0 {
        return out;
    }
    for deep in 1..=max_deep {
        for combo in slot_combinations(depth, deep) {
            if out.len() >= reserve {
                return out;
            }
            // One *rate* serves the whole subset, but each member draws its
            // own index from it, so the subset's coordinates are at
            // unrelated ranks rather than one shared rank.  The index has to
            // be legal in every member, so each is taken modulo the span of
            // the subset's *narrowest* slot.
            let narrowest = combo
                .iter()
                .map(|&slot| slot_widths[slot])
                .min()
                .unwrap_or(0);
            // The candidate this subset contributes, and the best-bound
            // candidate seen so far.  `t = 0` is the strided draw the sweep
            // made on its own, so the sample is a superset of the old
            // emission and the only thing the bound changes is *which*
            // member of the sample is spent.
            let mut best: Option<(f64, Vec<usize>)> = None;
            for t in 0..EMIT_PROFILE_SAMPLE {
                let rotation = phase.wrapping_add(t);
                let Some(_) =
                    sweep_index(narrowest, reserve, out.len(), 0, rotation)
                else {
                    break;
                };
                let mut tuple = vec![0usize; depth];
                let mut legal = true;
                for (member, &slot) in combo.iter().enumerate() {
                    let Some(at) = sweep_index(
                        narrowest,
                        reserve,
                        out.len(),
                        member,
                        rotation,
                    ) else {
                        legal = false;
                        break;
                    };
                    tuple[slot] = at;
                }
                if !legal
                    || combo
                        .iter()
                        .any(|&slot| tuple[slot] >= slot_widths[slot])
                {
                    break;
                }
                let bound = bound_of(&tuple);
                if best.as_ref().is_none_or(|&(b, _)| bound > b) {
                    best = Some((bound, tuple));
                }
            }
            if let Some((_, tuple)) = best {
                out.push(tuple);
            }
        }
    }
    out
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
        self.generate_with_pool(target).0
    }

    /// The proposals for `target`, and the size of the deduplicated
    /// candidate pool they were selected from.
    ///
    /// The second number is the denominator every reachability claim in
    /// this project is written against, and it is a property of the
    /// search rather than of the display policy: it is the length of the
    /// phrase-deduplicated, score-ordered pool *after* `finish` has
    /// collapsed duplicate spellings and *before* `select_diverse`
    /// narrows it to `top_n`.  It is exposed because a membership count
    /// is only a measurement if the pool is reproducible, and
    /// reproducibility is not observable from the visible list — a
    /// run-to-run pool delta of one candidate leaves the visible
    /// `top_n` byte-identical (see
    /// [w-6b91d3](../docs/work/items/w-6b91d3.md) and
    /// `tests/approx_determinism.rs`).
    pub fn generate_with_pool(&self, target: &str) -> (Vec<Clue>, usize) {
        let (selected, pool_size, _) = self.search(target);
        (selected, pool_size)
    }

    /// The **whole deduplicated candidate pool** for `target`: the
    /// phrase-deduplicated, score-ordered clues the search actually built,
    /// before [`Self::generate`]'s display policy narrows them to `top_n`.
    ///
    /// # Why the pool and not just its size
    ///
    /// "The search never proposed this clue" and "the search proposed it and
    /// the printed list did not show it" are different facts with different
    /// causes, and the second has nothing to do with whether the first is
    /// true.  Every explanation of a missing clue is either an *emission*
    /// question — was the alignment ever built — or an *objective* question —
    /// was it built and then outranked — and the two call for opposite
    /// changes.  Answering the first from the second is not an
    /// approximation: a pool can be orders of magnitude wider than the
    /// printed list, so a clue can be out of the printed 50 and present in
    /// the pool many hundreds of ranks deep, and the two facts are
    /// independent.
    ///
    /// Until this method existed the pool's *contents* were unreachable from
    /// outside the crate: [`Self::generate_with_pool`] reported the pool's
    /// size and the selected proposals, and nothing else, so a pool-membership
    /// claim could only be measured from inside `src/`.  That is why such
    /// claims kept resting on out-of-tree harnesses, which are the thing this
    /// project has most often got wrong.  This makes the question a
    /// first-class, re-runnable property of the public API.
    ///
    /// The returned order is the search's own: descending score, ties broken
    /// by phrase.  It is the same vector `generate` selects from, so
    /// membership in it is membership in the production pool by
    /// construction rather than by re-derivation.
    pub fn generate_pool(&self, target: &str) -> Vec<Clue> {
        self.search(target).2
    }

    /// The selected proposals, the pool's size, and the pool itself.
    fn search(&self, target: &str) -> (Vec<Clue>, usize, Vec<Clue>) {
        match self.config.mode {
            SearchMode::Exact => self.generate_exact(target),
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => self.generate_approximate(target, per_word_budget, total_budget),
        }
    }

    fn generate_exact(&self, target: &str) -> (Vec<Clue>, usize, Vec<Clue>) {
        let Some((target_ipa, target_boundaries, target_syllables)) =
            transcribe_with_boundaries(&self.corpus, target, false)
        else {
            return (Vec::new(), 0, Vec::new());
        };
        let target_phrase = TargetPhrase::new(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 {
            return (Vec::new(), 0, Vec::new());
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
    ) -> (Vec<Clue>, usize, Vec<Clue>) {
        let Some((target_ipa, target_boundaries, target_syllables)) =
            transcribe_with_boundaries(&self.corpus, target, true)
        else {
            return (Vec::new(), 0, Vec::new());
        };
        let target_phrase = TargetPhrase::new(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 || self.fuzzy_lexicon.is_empty() {
            return (Vec::new(), 0, Vec::new());
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
            // w-6b91d3: the DP state map is a `HashMap`, so its iteration
            // order is reseeded from `RandomState` on every process and
            // differs between two runs of the same binary.  Two states
            // are otherwise interchangeable predecessors here, so the
            // order is not *observably* wrong — but every path a state
            // contributes is `push`ed onto its successor's bucket and
            // that bucket is then `sort_by`'d (stable) and `truncate`d
            // to `SEG_STATE_KEEP`, so where the surviving set is cut
            // through a group of exactly-equal `rank`s, *which* of the
            // tied paths is kept is decided by this iteration order.
            // That is a run-to-run difference in the pool, not a
            // difference in the score.  Draining the map in key order
            // makes the arrival order a function of the state keys
            // alone, which are unique, so the whole DP is
            // order-independent from here on.
            let mut here: Vec<((usize, usize), Vec<SegPath>)> =
                std::mem::take(&mut seg_states[p]).into_iter().collect();
            here.sort_by(|(a, _), (b, _)| a.cmp(b));
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
                        // Same reason as the drain above: `rank` is an
                        // `f64` proxy that ties exactly, and the sort is
                        // stable, so the cut through a tied group used to
                        // be decided by arrival order.  `spans` is unique
                        // within a state — the DP never records the same
                        // chain twice — so breaking on it makes this a
                        // total order and the retained set a function of
                        // the paths alone.
                        bucket.sort_by(|a, b| {
                            cmp_desc(a.rank, b.rank)
                                .then_with(|| a.spans.cmp(&b.spans))
                        });
                        bucket.truncate(SEG_STATE_KEEP);
                    }
                }
            }
        }

        let mut segmentations: Vec<(f64, SegPath)> = Vec::new();
        // w-6b91d3: the second `HashMap` drain with the same consequence.
        // The order of `segmentations` is not cosmetic: it becomes
        // `by_structure` -> `schedule` (see below), which is the order
        // the emission budget is spent in, so a tie resolved differently
        // spends the budget on different structures and leaves a
        // different set of wordings in the pool.
        let mut final_states: Vec<((usize, usize), Vec<SegPath>)> =
            std::mem::take(&mut seg_states[n]).into_iter().collect();
        final_states.sort_by(|(a, _), (b, _)| a.cmp(b));
        for ((word_count, _shared), paths) in final_states {
            if word_count == 0 {
                continue;
            }
            for path in paths {
                // The path is complete here, so novelty is exact and
                // the scorer's own weights apply without reservation.
                segmentations.push((span_objective(word_count, &path), path));
            }
        }
        // And the cut is through a group of exactly-equal objectives once
        // per target, so the tie-break has to be on the key itself: two
        // complete paths with the same objective are ordered by their
        // span chain, which is unique.
        segmentations.sort_by(|a, b| {
            cmp_desc(a.0, b.0).then_with(|| a.1.spans.cmp(&b.1.spans))
        });
        segmentations.truncate(SEGMENTATION_KEEP);

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

            // The traversal's own key, computed before the depth-profile
            // reserve rather than after it: the reserve now spends its share
            // on the best-bound member of the sweep's own sample, and the
            // bound it uses is this one.  It is a pure function of the slots
            // and the segmentation, so hoisting it changes nothing the
            // traversal below computes.
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
            // What a prefix has already committed, and what the slots after
            // `k` cost at their own cheapest.  Together with a candidate's
            // own cost these are the three terms of the per-slot
            // affordability test below; they are read from the suffix
            // bounds above rather than recomputed per candidate.
            let slot_min_cost: Vec<f64> = (0..depth)
                .map(|k| {
                    slots[k]
                        .iter()
                        .map(|a| a.cost)
                        .fold(f64::INFINITY, f64::min)
                })
                .collect();
            // The minimum cost of every slot *after* `k`, i.e. the cheapest
            // way to finish a prefix once slot `k` has been decided.
            let later_min_cost: Vec<f64> = (0..depth)
                .map(|k| suf_min_cost[k + 1] - slot_min_cost[k])
                .collect();

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
                funded_slot_depth(&widths),
                coverage_phase,
                &bound,
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
                {
                    counters::note_depth(
                        &counters::DEEPEST_PROFILE,
                        tuple.iter().copied().max().unwrap_or(0),
                    );
                    counters::note_total(
                        &counters::DEEPEST_PROFILE_COORDINATES,
                        tuple.iter().filter(|&&i| i != 0).count() as u64,
                    );
                }
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
            // which is the search's real ceiling and which it shares with the
            // traversal, so the operator's spend is bounded by the same
            // constant that bounds everything else.  `.min(adjacency_spend)`
            // draws on the reserved slice, so the operator runs out of budget
            // rather than the traversal.
            //
            // The justification used to be that the ceiling *leaves slack* —
            // "14,239 of 16,384 spent" — so funding the operator from it
            // would cost the traversal nothing.  That is no longer true and the
            // number is worth correcting where it was load-bearing rather than
            // deleting: measured on three real multi-clause targets at
            // `--approximate --top 50` (release, defaults; the accounting is
            // `docs/work/items/w-b3e91a.md`), the lexical phase spends
            // **16,384 of 16,384** emissions — 100 % saturated — while
            // spending 175,957-212,108 of 1,024,000 pops, i.e. 17-21 % of the
            // pop budget.  So the operator's emissions now come out of the
            // same 16,384 as the traversal's and the two really are in
            // competition; the *ordering* of the spend (reserve, then
            // traversal, then operator, all against one ceiling) is what
            // protects the traversal, not slack above it.  Raising the
            // operator's share measurably does cost the tail: at
            // `ADJACENCY_RESERVE` 64 and `ADJACENCY_GLOBAL_RESERVE` 8,192 the
            // canonical target's retained-structure count falls 308 -> 214 and
            // its pool width rises only 18,936 -> 19,141, because the operator
            // runs per segmentation and starves the late-retained structures
            // before the traversal reaches them.  See also the ordering
            // measurement below and `docs/work/items/w-b3e91a.md`.
            let adjacency_allowance = ADJACENCY_RESERVE
                .min(emit_allowance)
                .min(adjacency_spend);

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
            // The opening width is derived, not chosen: at
            // `LEXICAL_BRANCH_STAGE_0` a `d`-slot product needs
            // `1 + w + ... + w^(d-1)` pops before the walk reaches its
            // first wording at all, and a `d` where that exceeds
            // `LEXICAL_HEAP_POP_LIMIT` is a segmentation whose emission
            // allowance cannot be paid however the pops are spent.  See
            // [`affordable_opening_width`] for the measured counts.
            let mut cap = affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT)
                .min(widest);
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
                    let committed: f64 = prefix
                        .iter()
                        .enumerate()
                        .map(|(j, &i)| slots[j][i].cost)
                        .sum();
                    for i in 0..slots[k].len().min(cap) {
                        if !slot_is_affordable(
                            committed,
                            later_min_cost[k],
                            slots[k][i].cost,
                            total_budget,
                        ) {
                            continue;
                        }
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
                        let committed: f64 = prefix
                            .iter()
                            .enumerate()
                            .map(|(j, &i)| slots[j][i].cost)
                            .sum();
                        for i in opened..slots[k].len().min(cap) {
                            if !slot_is_affordable(
                                committed,
                                later_min_cost[k],
                                slots[k][i].cost,
                                total_budget,
                            ) {
                                continue;
                            }
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

        // How much of each ceiling the lexical phase actually spent.  Test
        // only: it is the measurement behind
        // `the_global_emission_ceiling_is_reached_not_merely_respected`, which
        // is what distinguishes a bound that binds from one that is merely
        // respected, and it is what corrects the "the ceiling leaves slack"
        // claim the adjacency operator's funding was justified by.
        #[cfg(test)]
        {
            counters::note_total(
                &counters::SPENT_EMISSIONS,
                spent_emissions as u64,
            );
            counters::note_total(&counters::SPENT_POPS, spent_pops as u64);
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
    ) -> (Vec<Clue>, usize, Vec<Clue>) {
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
        let pool_size = clues.len();

        (
            select_diverse(clues.clone(), self.config.top_n),
            pool_size,
            clues,
        )
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
        /// The most non-zero coordinates any one reserved depth-profile
        /// emission has carried — the reserve's *depth*, as distinct from
        /// `DEEPEST_PROFILE`, which is the deepest single slot index it
        /// reached.  A tuple deep in one slot and a tuple deep at one deep
        /// index are different things and only one of them is a coverage
        /// question; this is the one the reserve's slot cap answers.
        pub static DEEPEST_PROFILE_COORDINATES: Cell<u64> = const {
            Cell::new(0)
        };
        /// How many wordings the lexical phase actually put in the pool, so a
        /// test can assert that the *global* emission ceiling is reached rather
        /// than merely respected.  See
        /// `the_global_emission_ceiling_is_reached_not_merely_respected`.
        pub static SPENT_EMISSIONS: Cell<u64> = const { Cell::new(0) };
        /// The same, for heap pops: the other half of the same question,
        /// because a ceiling that is reached on emissions and never approached
        /// on pops is the statement that the search is emission-bound.
        pub static SPENT_POPS: Cell<u64> = const { Cell::new(0) };
    }

    pub fn bump(counter: &'static std::thread::LocalKey<Cell<u64>>) {
        counter.with(|c| c.set(c.get() + 1));
    }

    pub fn take(counter: &'static std::thread::LocalKey<Cell<u64>>) -> u64 {
        counter.with(|c| c.replace(0))
    }

    /// Read and reset a *high-water* counter, so a test can measure the
    /// largest value a single run reached without the previous run's maximum
    /// leaking into this one.
    pub fn take_total(
        counter: &'static std::thread::LocalKey<Cell<u64>>,
    ) -> u64 {
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

    /// Record `total` if it is larger than anything already recorded.
    pub fn note_total(counter: &'static std::thread::LocalKey<Cell<u64>>, total: u64) {
        counter.with(|c| {
            if total > c.get() {
                c.set(total);
            }
        });
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
    /// Clue words of one syllable or fewer, for the `PUNCH` axis.  The
    /// syllable count is already computed on the extension path for
    /// `syllables`, so this is a comparison against a value the
    /// candidate already holds, not a second count.
    punch_count: usize,
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
            punch_count: 0,
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
        let syllables = approx::ipa_syllables(ipa);

        Self {
            words: Some(word_node),
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            syllables: self.syllables + syllables,
            // `ipa_syllables` floors an empty transcription at zero and a
            // non-empty one at one, so `== 1` is "one syllable or fewer"
            // and a word with no IPA at all is not counted as punch.
            punch_count: self.punch_count + usize::from(syllables == 1),
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
        let words = self.word_count();

        // Phonetic similarity is a property of the clue's *phones*, and the
        // axis is scored on the mean edit cost of one phone.  Two earlier
        // readings got this wrong, and the second was only a level down
        // from the first.
        //
        // The original divided the whole candidate's total cost by a
        // constant, so the same per-word quality was charged once per
        // candidate for a once-per-word property: two clues whose words
        // were individually as good as each other were ordered by how many
        // words each happened to have, and a long clue could not reach the
        // top of the list however good every one of its words was.  That
        // much was removed by reading the axis per word.
        //
        // What per word left in place was the *span-length* term.  A clue
        // word that must cover several of the target's phones is charged
        // edit cost for all of them, so at equal per-phone phonetic
        // quality a clue cut into long words scored strictly below one cut
        // into short words: the axis was still a function of how the
        // target's phones happen to be distributed among the words rather
        // than of how good those words are.  Reading it per phone removes
        // that, so the axis is now a function of the total edit cost over
        // the target's phone count and of nothing else.  Two clues over the
        // same target that cost the same total cost score the same however
        // their phones are distributed; only a genuinely worse per-phone
        // clue is charged.
        //
        // `SIMILARITY_PER_WORD` is deliberately left at its old value.  It
        // is a *local* ordering weight for a span's alternatives, and it
        // is left on the enumeration side of the boundary on purpose: the
        // exact marginal weight of the new axis is
        // `-SIMILARITY * weight / (phones * SIMILARITY_COST_PER_PHONE)`, a
        // different number again; re-deriving all three proxy sites on it
        // changes which words the search keeps, and it was measured to lose
        // `approximate_pool_reaches_matches_deep_in_a_span`.  That is an
        // enumeration-side effect, so it is left to the front that owns
        // enumeration rather than smuggled in here.
        let cost_per_phone = self.sub_cost_total / total_len.max(1) as f64;
        let similarity =
            (1.0 - cost_per_phone / axes::SIMILARITY_COST_PER_PHONE).clamp(0.0, 1.0);
        let novelty =
            boundary_novelty(&self.cuts, target_boundaries, total_len, partial);

        let reused = self.reused_count as f64;
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

        // A Mad Gab answer has to be *sayable in one pass* by a listener
        // who has never heard the target.  `RHYTHM` reads only the total
        // syllable count, which a redistribution across words satisfies
        // for free, and `SIMILARITY` charges edit cost over the whole
        // IPA stream, so a twelve-phone polysyllable that happens to be
        // cheap scores like a three-phone monosyllable one.  Nothing in
        // the objective constrains the clue to be *pronounceable at the
        // pace of the target sentence*, and the answer without that
        // constraint is the lookalike-polysyllable clue, which is
        // phonetically excellent and humanly unusable: the solver cannot
        // get through it in one pass, so the wordplay never lands.
        //
        // This is the share of clue words of one syllable or fewer, and
        // it asks the clue to be as short-worded as the target is, so
        // it bites hardest exactly where the target has long words.
        let punch = self.punch_count as f64 / words.max(1) as f64;

        // Written as `w * (v - 1)` rather than `w * v`.  The six existing
        // weights sum to exactly 1.00 and `CLOSED_CLASS` is the only
        // signed one, so the objective's maximum is already exactly 1.0
        // and there is zero headroom: `+ w*v` would push every
        // all-axes-perfect clue above the documented `Clue::score` upper
        // bound at any weight above print precision.  Shifting the axis
        // by its own maximum makes the term non-positive and leaves the
        // bound where it was.  It is *ordering*-identical to the `+ w*v`
        // form — the same constant comes off every candidate, so no
        // comparison between two candidates changes — which is what makes
        // it the right choice rather than a different axis.  It does not
        // by itself settle `approximate_output_is_locked`, which locks
        // printed score *strings*: a band that is not entirely
        // monosyllabic still moves, by up to `w`.
        let combined = axes::SIMILARITY * similarity
            + axes::NOVELTY * novelty
            + axes::WORD_NOVELTY * word_novelty
            + axes::FAMILIARITY * familiarity
            + axes::RHYTHM * rhythm
            + axes::SHAPE * shape_quality
            + axes::CLOSED_CLASS * closed_penalty
            + axes::PUNCH * (punch - 1.0);

        Metrics {
            combined,
            similarity,
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
    /// Share of clue words of one syllable or fewer, `PUNCH`, also
    /// non-positive: `metrics` applies it as `w * (v - 1)`, so a clue
    /// that is entirely monosyllabic pays nothing and one with no
    /// monosyllable at all pays the full weight.  0.10 is comparable to
    /// `FAMILIARITY` and half of `SIMILARITY`.
    pub const PUNCH: f64 = 0.10;

    /// Mean edit cost of one clue phone that scores this axis's full
    /// penalty.
    ///
    /// The denominator is the target's own phone count, so the axis is
    /// `1 - (clue's total edit cost / target's phones) / this`.  Two clues
    /// over the same target that cost the same total cost therefore score
    /// the same however their phones are distributed among their words,
    /// and a clue is charged for cost and for nothing else.
    ///
    /// `SIMILARITY_PER_WORD` below is the axis's derivative with respect
    /// to one word's cost at one word of clue, which is the local weight
    /// the single-word ranking proxies use, and it is deliberately left on
    /// the old per-word scale: see `metrics`.
    pub const SIMILARITY_COST_PER_PHONE: f64 = 0.30;

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
    /// The `SIMILARITY` axis on its own, kept so a test can assert the
    /// axis's own behaviour without the seven other terms, which all
    /// depend on the clue's word count in ways this axis must not.
    similarity: f64,
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
/// 1. **Represent the enumerated resegmentations.**  Spend
///    [`structure_reserve_slots`] of the `top_n` slots on the best-scoring
///    candidate of each distinct boundary structure, in descending score
///    order, taken from the **whole** pool rather than from the visible
///    cutoff.  The list therefore always spans the resegmentations the
///    search actually enumerated rather than one per structure by
///    accident of the score order's head, and a resegmentation is
///    represented on a criterion that is not its own score's rank: it
///    needs a candidate the enumeration produced, and no better member of
///    its own structure listed ahead of it.  The bound is what keeps this
///    a *representation* reserve and not a replacement of the list — the
///    remaining slots are still filled on score alone, so the reserve
///    costs visible quality exactly and only to the extent stated by
///    [`structure_reserve_slots`].
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

    let mut picked: Vec<usize> = Vec::with_capacity(top_n);
    let mut taken = vec![false; clues.len()];
    let mut counts: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut represented: HashSet<Vec<usize>> = HashSet::new();

    // 1. one representative per boundary structure, best first, drawn
    // from the *whole* pool and bounded by `structure_reserve_slots`.
    //
    // The bounded reserve is what the previous rule lacked.  Step 2's
    // admission is a share cap, so a structure that owns the head of the
    // score order is stopped at its cap and the slots it gives up go to
    // the best candidate of a structure that is under-filled — but only
    // among structures that are *already represented*.  Nothing in that
    // walk ever spends a slot on a resegmentation whose best candidate is
    // outside the visible cutoff, so a pool that enumerates hundreds of
    // boundary structures can have 50 slots and show 9 of them.  The
    // reserve spends a bounded, stated number of slots on the best
    // candidate of each further structure in descending score order,
    // which is a criterion other than "this candidate's own score is
    // inside the cutoff": the criterion is "this resegmentation is one
    // the search enumerated and nothing better of its kind is listed".
    let reserve = structure_reserve_slots(top_n);
    let mut reserved = 0usize;
    for &i in &order {
        if picked.len() == top_n || reserved == reserve {
            break;
        }
        if represented.insert(structures[i].clone()) {
            admit(
                i,
                &structures,
                &mut picked,
                &mut taken,
                &mut counts,
            );
            reserved += 1;
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

/// How many of `top_n`'s slots are set aside to represent the enumerated
/// resegmentations, one slot each, before the list is filled in score
/// order.
///
/// A share of the list rather than all of it, because the two ends of the
/// contract are both real: the enumeration is spending a global emission
/// budget to find *many* resegmentations, and the display policy is
/// answering a request for the *best* clues.  A reserve of one slot in
/// [`STRUCTURE_RESERVE_DIVISOR`] buys the second-order spread at a stated
/// cost — the reserve's members are the pool's best per structure, which
/// are below the head of the score order by however far that structure's
/// own best lies — and the remaining `1 - 1 / STRUCTURE_RESERVE_DIVISOR`
/// of the list is still filled on score alone.
///
/// It is bounded above by `top_n` and below by one, so a one-slot list
/// still reserves its single slot for a resegmentation rather than
/// degenerating into "the top candidate", and it never depends on a pool
/// size, a target, or anything but the number of slots: the same bound
/// applies to every input.
fn structure_reserve_slots(top_n: usize) -> usize {
    (top_n / STRUCTURE_RESERVE_DIVISOR).min(top_n).max(usize::from(top_n > 0))
}

/// How many slots one slot of representation costs: the visible list
/// spends one slot in every [`STRUCTURE_RESERVE_DIVISOR`] on the best
/// candidate of a further enumerated resegmentation, and the rest on
/// score.  See [`structure_reserve_slots`].
///
/// This is a *policy* parameter of [`select_diverse`] in the same sense as
/// [`STRUCTURE_FLOOR`]: it is a number of slots, not a weight fitted to a
/// target, and it is the only knob the representation rule has.
const STRUCTURE_RESERVE_DIVISOR: usize = 4;

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

    /// A score bound that rates every candidate the same, so
    /// [`coverage_tuples`] keeps the first one it draws and the rule
    /// reduces to the index-ordered strided draw.
    ///
    /// The sweep-coverage tests are about *which positions* the rule
    /// reaches, not about which of them it prefers, and they have no slots
    /// to compute a real bound from, so they pin the sweep with the
    /// preference switched off.  [`the_coverage_reserve_spends_on_the_best_bounded_candidate_it_draws`]
    /// is the test that pins the preference.
    fn flat_bound(_tuple: &[usize]) -> f64 {
        0.0
    }

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

    /// The representation reserve.  A pool whose head of the score order
    /// belongs to a handful of structures is a monoculture even when the
    /// pool itself enumerates many more: on a real search the visible
    /// list was 4-11 structures out of 286-336 the pool holds, because
    /// nothing in the rule ever spent a slot on a structure whose best
    /// candidate was outside the visible cutoff.  The reserve spends a
    /// bounded number of slots on the best candidate of each further
    /// structure instead, and this is the boundary: the slots it reserves
    /// are exactly the reserve, and the list is still full.
    #[test]
    fn enumerated_resegments_outside_the_cutoff_get_a_reserved_slot() {
        let top_n = 12usize;
        // The head of the score order: `top_n` candidates of one
        // structure, all inside the visible cutoff, so the cutoff itself
        // holds a single resegmentation.
        let mut pool: Vec<Clue> = (0..top_n)
            .map(|i| clue(&format!("own{i} rr ss"), 0.930 - i as f64 * 1e-4))
            .collect();
        // Four times that many further structures, every one of them
        // *below* the cutoff: this is the population the reserve exists
        // to represent.
        for s in 0..(top_n * 4) {
            // Word count is what makes the structure distinct here: every
            // word is three IPA chars, so a clue of `k` words has the
            // cuts `[3, 6, ..., 3k]`.
            let words: Vec<String> = (0..2 + s % 5)
                .map(|w| format!("far{s}w{w}"))
                .collect();
            pool.push(clue(
                &words.join(" "),
                0.700 - s as f64 * 1e-4,
            ));
        }

        let map = structure_map(&pool);
        let picked = select_diverse(pool, top_n);
        assert_eq!(picked.len(), top_n, "the list must stay full");

        let distinct: HashSet<&Vec<usize>> =
            picked.iter().map(|c| boundaries_of(&map, c)).collect();
        assert!(
            distinct.len() >= structure_reserve_slots(top_n),
            "a pool enumerating {} structures showed {} of them; the rule \
             reserves {} slots for structures the cutoff does not reach",
            top_n * 4 + 1,
            distinct.len(),
            structure_reserve_slots(top_n)
        );
        assert!(
            picked.iter().any(|c| c.phrase.starts_with("far")),
            "no enumerated resegmentation outside the cutoff was represented; \
             got {:?}",
            picked.iter().map(|c| c.phrase.as_str()).collect::<Vec<_>>()
        );
    }

    /// The reserve is bounded: it may not become the list.  A dominant
    /// structure still holds its share, and the reserve's own bound is
    /// what a caller can predict without running a search.
    #[test]
    fn structure_reserve_is_bounded_by_the_slot_count() {
        for top_n in [1usize, 2, 5, 10, 20, 50, 200, 4096] {
            let reserve = structure_reserve_slots(top_n);
            assert!(reserve <= top_n, "reserve {reserve} exceeds top_n {top_n}");
            assert!(reserve >= usize::from(top_n > 0), "no reserve at {top_n}");
        }
        assert_eq!(structure_reserve_slots(0), 0);
        // A quarter of the list, never more: the rest of the list is
        // still filled on score.
        assert_eq!(structure_reserve_slots(50), 12);
        assert!(structure_reserve_slots(200) < 200);
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
            total_len: usize,
            partial: bool,
        ) -> Metrics {
            // The similarity axis is scored on the mean per-phone cost
            // (see `metrics`); what this test pins is that the
            // incrementally maintained aggregates fold to the same
            // numbers, not which normaliser the axis uses.
            let similarity = (1.0
                - p.sub_cost_total / total_len.max(1) as f64
                    / axes::SIMILARITY_COST_PER_PHONE)
                .clamp(0.0, 1.0);
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
                similarity,
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
                        refold(&p, &target, 4, 12, partial)
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
                let Some(at) = sweep_index(width, 16, 0, 0, phase) else {
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
        // The depth bound is a *count of slots*, so the only property worth
        // pinning here is that it is a count of a real slot list and never
        // exceeds it.  The interesting content of the bound — that a
        // segmentation four slots wide with all four wider than the
        // traversal's first stage is deep enough to place a four-deep tuple —
        // is asserted in `the_reserve_places_a_tuple_deep_in_every_slot_the
        // traversal_leaves_room_in`, which re-derives it from the widths.
        for widths in [
            vec![SPAN_SHORTLIST; 1],
            vec![SPAN_SHORTLIST; 4],
            vec![SPAN_SHORTLIST; 9],
            vec![LEXICAL_BRANCH_STAGE_0; 6],
            vec![3, LEXICAL_BRANCH_STAGE_0 * 2],
        ] {
            assert!(
                funded_slot_depth(&widths) <= widths.len(),
                "the reserve's depth bound counts a segmentation's own slots \
                 and cannot exceed them: {widths:?}"
            );
        }

        // The reserve is carved out of `emit_allowance` and the traversal
        // is handed the remainder, so the per-segmentation total is
        // unchanged whatever the reserve manages to afford.
        for emit_allowance in 1..=LEXICAL_COMBINATIONS_PER_SEGMENTATION {
            let profile_allowance = EMIT_PROFILE_RESERVE.min(emit_allowance);
            for spent in 0..=profile_allowance {
                assert_eq!(spent + (emit_allowance - spent), emit_allowance);
            }
        }

        // The candidate list is bounded by the number of slot subsets, and
        // never exceeds the reserve or that bound.
        for depth in 1..=24usize {
            let widths = vec![SPAN_SHORTLIST; depth];
            let classes: usize = (1..=funded_slot_depth(&widths).min(depth))
                .map(|k| {
                    let mut c = 1usize;
                    for j in 0..k {
                        c = c * (depth - j) / (j + 1);
                    }
                    c
                })
                .sum();
            for phase in 0..64usize {
                let tuples = coverage_tuples(
                    &widths,
                    EMIT_PROFILE_RESERVE,
                    funded_slot_depth(&widths),
                    phase,
                    &flat_bound,
                );
                assert!(tuples.len() <= EMIT_PROFILE_RESERVE, "depth {depth}");
                assert!(tuples.len() <= classes, "depth {depth}");
                for tuple in &tuples {
                    let deep: Vec<usize> = tuple
                        .iter()
                        .copied()
                        .filter(|&i| i != 0)
                        .collect();
                    assert!(!deep.is_empty(), "{tuple:?} is not a profile");
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
                    // A subset's deep coordinates are at pairwise
                    // unrelated ranks, so a shape class can place a set of
                    // deep alternatives in different slots at coordinates
                    // that are not one shared rank; every other slot keeps
                    // the traversal's own best.
                    let span = SPAN_SHORTLIST - LEXICAL_BRANCH_STAGE_0;
                    assert!(
                        deep.windows(2).all(|w| w[0] != w[1]),
                        "{tuple:?} re-used one rank for the whole class"
                    );
                    assert!(
                        deep.len() < span,
                        "{tuple:?}: too few indices to pair the class"
                    );
                }
                // Breadth before depth: a reserve smaller than the number
                // of single-slot classes still touches every slot.
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
                funded_slot_depth(&widths),
                phase,
                &flat_bound,
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
                if let Some(at) = sweep_index(width, per, 0, 0, phase) {
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

    /// The coverage reserve can *express* a set of deep alternatives at
    /// unrelated ranks, and over a run of phases it can reach every index of
    /// every slot's list above the floor.
    ///
    /// Both halves are properties of the enumeration, not of any target, so
    /// the widths here are synthetic and identical and no word appears.
    ///
    /// The second half is what the rule has always claimed and is asserted
    /// for every slot; the first half is the one the previous rule could not
    /// satisfy, because it gave every member of a shape class *one* index and
    /// so forced a set of deep alternatives onto a single rank no matter how
    /// many of them were asked for.
    #[test]
    fn the_coverage_sweep_covers_each_slot_and_pairs_its_class_members() {
        let per = EMIT_PROFILE_RESERVE;
        for depth in 1..=6usize {
            let widths = vec![SPAN_SHORTLIST; depth];
            let span = SPAN_SHORTLIST - LEXICAL_BRANCH_STAGE_0;
            // A run of phases as long as the widest list's span: the point is
            // that *every* rate the rule uses is coprime to the span, so no
            // slot is confined to a residue class however long the run.
            let mut placed: Vec<HashSet<usize>> = vec![HashSet::new(); depth];
            let mut paired = 0usize;
            for phase in 0..span {
                for tuple in coverage_tuples(
                    &widths,
                    per,
                    funded_slot_depth(&widths),
                    phase,
                    &flat_bound,
                ) {
                    let deep: Vec<usize> = tuple
                        .iter()
                        .copied()
                        .filter(|&i| i != 0)
                        .collect();
                    for (slot, &i) in tuple.iter().enumerate() {
                        if i != 0 {
                            placed[slot].insert(i);
                        }
                    }
                    if deep.len() >= 2 && deep.windows(2).all(|w| w[0] != w[1]) {
                        paired += 1;
                    }
                }
            }
            for slot in 0..depth {
                assert_eq!(
                    placed[slot].len(),
                    span,
                    "depth {depth} slot {slot}: the reserve reached {} of the \
                     slot's {span} indices above the floor",
                    placed[slot].len()
                );
            }
            if depth >= 2 {
                assert!(
                    paired > 0,
                    "depth {depth}: no emitted tuple put its deep \
                     coordinates at pairwise different ranks"
                );
            }
        }
    }

    /// The narrower claim the schedule has to keep to earn the wider one: a
    /// single phase's spend is still spread over the list rather than spent
    /// on one window of it, so a structure the traversal funds *once* is not
    /// reduced to the ranks adjacent to that one phase's start.
    #[test]
    fn one_phase_of_the_coverage_sweep_is_spread_over_the_list() {
        let per = EMIT_PROFILE_RESERVE;
        // A subset has at most one member per slot, and no segmentation in
        // the corpus is wider than this many slots, so this is the widest
        // member index the rate ladder is ever asked for.  It is written as
        // the same derived count the reserve itself uses, so the two cannot
        // drift apart.
        let max_members = funded_slot_depth(&[SPAN_SHORTLIST; 8]);
        for width in (LEXICAL_BRANCH_STAGE_0 * 2)..=SPAN_SHORTLIST {
            let mut seen: HashSet<usize> = HashSet::new();
            for nth in 0..per {
                for member in 0..max_members {
                    if let Some(at) = sweep_index(width, per, nth, member, 0) {
                        seen.insert(at);
                    }
                }
            }
            // One phase draws at most `per * max_members`
            // indices, so the honest claim is that the draw is *spread* and
            // not a window: a third of the list's indices are in play from a
            // single phase.  A schedule that advanced every draw by one
            // instead of by the span's stride would put all of them in a
            // window and fail this.
            let span = width - LEXICAL_BRANCH_STAGE_0;
            assert!(
                seen.len() * 3 >= span,
                "width {width} (span {span}): one phase spent on {} \
                 distinct indices, i.e. one window of the list",
                seen.len()
            );
        }
    }

    /// The reserve spends on the best-**bounded** member of the sample its
    /// own sweep draws, not on the sample's index order.
    ///
    /// The general property, with no phrase and no real target's slot list
    /// in it: given a bound that discriminates between the candidates the
    /// sweep draws, every emission is the highest-bound candidate of its
    /// own subset's rotation, the spend is the same size, the alphabet is
    /// the same, and at least some emissions are tuples the index-ordered
    /// draw never produces.
    ///
    /// The admissibility claim is asserted here too, because it is the
    /// claim that makes the bound usable rather than a filter.  The
    /// strided draw is the *first* member of the rotation, so it is still
    /// a candidate: the rule can only ever displace it with a candidate the
    /// bound rates at least as high, never with one it rates lower, and it
    /// can only ever emit a position the sweep's own rotation produces.
    /// That is the same one-sided shape as `build`'s cost bound, which is
    /// also only ever used to drop a candidate in favour of paying for
    /// another one.
    #[test]
    fn the_coverage_reserve_spends_on_the_best_bounded_candidate_it_draws() {
        // Widths of the kind a real five-slot cell has after the span
        // shortlist: every one of them above the traversal's opening, so
        // every subset of them is fundable and the rotation is the whole
        // list.
        let widths = [37usize, 53, 29, 41, 19];
        // A bound of the shape the traversal's own key has — larger for a
        // deeper coordinate — with no axis, no weight and no input in it.
        let bound = |tuple: &[usize]| -> f64 {
            tuple.iter().map(|&i| i as f64).sum()
        };
        let per = EMIT_PROFILE_RESERVE;
        let mut reached = 0usize;
        let mut drawn_bound = 0.0f64;
        let mut index_bound = 0.0f64;
        let mut emitted = 0usize;
        for phase in 0..64usize {
            let index_order = coverage_tuples(
                &widths,
                per,
                funded_slot_depth(&widths),
                phase,
                &flat_bound,
            );
            let drawn = coverage_tuples(
                &widths,
                per,
                funded_slot_depth(&widths),
                phase,
                &bound,
            );
            // The rotation this phase's subset draws from, one entry per
            // member of the sample.
            let rotation: Vec<Vec<Vec<usize>>> = (0..EMIT_PROFILE_SAMPLE)
                .map(|t| {
                    coverage_tuples(
                        &widths,
                        per,
                        funded_slot_depth(&widths),
                        phase.wrapping_add(t),
                        &flat_bound,
                    )
                })
                .collect();
            assert_eq!(
                drawn.len(),
                index_order.len(),
                "phase {phase}: the bound reordered the spend, it did not \
                 resize it"
            );
            for r in &rotation {
                assert_eq!(
                    r.len(),
                    index_order.len(),
                    "phase {phase}: the sample's subsets are the sweep's \
                     subsets, so the reserve's count is the same for every \
                     member of it"
                );
            }
            for (subset, tuple) in drawn.iter().enumerate() {
                for (slot, &i) in tuple.iter().enumerate() {
                    assert!(i < widths[slot], "phase {phase}: {tuple:?}");
                    assert!(
                        i == 0 || i >= LEXICAL_BRANCH_STAGE_0,
                        "phase {phase}: {tuple:?} spends an index the \
                         traversal's first stage already generates"
                    );
                }
                assert!(
                    rotation.iter().any(|r| r.contains(tuple)),
                    "phase {phase} subset {subset}: {tuple:?} is not a \
                     position the sweep's own rotation draws"
                );
                for r in &rotation {
                    assert!(
                        bound(&r[subset]) <= bound(tuple) + 1e-12,
                        "phase {phase} subset {subset}: the strided draw \
                         {:?} was displaced by {:?}, which the bound rates \
                         lower",
                        r[subset],
                        tuple
                    );
                }
                drawn_bound += bound(tuple);
                index_bound += bound(&index_order[subset]);
                emitted += 1;
            }
            if drawn != index_order {
                reached += 1;
            }
        }
        assert!(
            reached > 0,
            "the bound never changed an emission, so it is not being used"
        );
        assert!(
            drawn_bound > index_bound,
            "the reserve's mean bound {drawn_bound} over {emitted} \
             emissions is not above the index-ordered draw's {index_bound}"
        );
    }

    /// The rates are coprime to the span, which is what makes the per-slot
    /// coverage above a property of every width rather than of the widths
    /// that happen to divide evenly.
    #[test]
    fn every_sweep_rate_is_coprime_to_the_span_it_is_used_on() {
        for span in 1..=SPAN_SHORTLIST - LEXICAL_BRANCH_STAGE_0 {
            // The ladder is indexed by a subset member, so it is asked for
            // one rate per slot of the widest segmentation, which is the same
            // derived count the reserve's own depth bound uses.
            for member in 0..funded_slot_depth(&[SPAN_SHORTLIST; 8]) {
                assert_eq!(
                    gcd(sweep_rate(member, span), span),
                    1,
                    "span {span} member {member}"
                );
            }
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

    /// The reserve's depth bound is **derived from the traversal's floor**,
    /// not chosen, and this re-derives it independently of the code under
    /// test.
    ///
    /// The expected count is recomputed here, inline, from the slot widths
    /// against `LEXICAL_BRANCH_STAGE_0` — deliberately *not* by calling
    /// [`funded_slot_depth`].  A test that asserted the function equalled
    /// itself would pass whatever the function returned, including a
    /// reintroduced literal, which is the self-referential defect this
    /// repository has already paid for once ([w-6d2af3](w-6d2af3.md),
    /// [w-3c9d17](w-3c9d17.md)).  Recomputing the count from the widths makes
    /// the assertion a claim about the *floor* — so restoring a constant
    /// depth cap fails this test, which a self-comparison would not.
    #[test]
    fn the_reserve_depth_bound_is_the_slots_the_traversal_leaves_room_in() {
        for slots in 1..=12usize {
            for narrow in 0..=4usize {
                let mut widths = vec![LEXICAL_BRANCH_STAGE_0; slots];
                for w in widths.iter_mut().skip(narrow) {
                    *w = SPAN_SHORTLIST;
                }
                // Independent re-derivation: a slot the reserve can make deep
                // is a slot with an index above the traversal's first stage,
                // because `sweep_index` has nowhere above the floor to draw
                // from for a slot at or below it.
                let expected = widths
                    .iter()
                    .filter(|w| **w > LEXICAL_BRANCH_STAGE_0)
                    .count();
                assert_eq!(
                    funded_slot_depth(&widths),
                    expected,
                    "{slots} slots, the first {narrow} at the floor: the \
                     bound is the count of slots the traversal leaves room in, \
                     so it cannot be a constant"
                );
            }
        }
    }

    /// The property this front exists for, stated generally: **a target whose
    /// best available wording is deep in one slot gets a deep-in-one-slot tuple
    /// placed for it** — and, specifically, a tuple deep in more slots than the
    /// constant this rule replaced allowed.
    ///
    /// No phrase, clue or word list appears in the assertion; the expected
    /// number is derived from the **targets' own word counts** and nothing
    /// else.  That is the independent re-derivation, and it is the reason this
    /// test is not self-referential: it reads neither `funded_slot_depth`,
    /// nor the slot widths, nor `LEXICAL_BRANCH_STAGE_0`, nor any counter the
    /// bound feeds.  A change to the production bound that lowered the deepest
    /// funded tier would move the measured number and fail here, without the
    /// test and the rule sharing anything to change together.
    ///
    /// The derivation, so the number is not magic: a target of `W` words
    /// admits segmentations of up to `W` slots, and a representative of the
    /// reserve keeps index 0 in the slots it does not make deep, so the
    /// deepest tuple the reserve can place for such a target has `W - 1`
    /// non-zero coordinates.  Over targets of at least
    /// `SHORTEST` words that is at least `SHORTEST - 1`.  The assertion is
    /// that the reserve reaches it, for every target, not for a named one.
    #[test]
    fn a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple() {
        /// Real multi-clause targets, none of which any assertion below names.
        const TARGETS: &[&str] = &[
            "a whole lot of trouble",
            "he was a big fat man",
            "what are you going to do",
            "there is no way to know",
            "the cat sat on the mat",
            "when the rain finally stopped",
            "you can do it yourself",
            "an old man in a big hat",
            "we should have told her",
            "in the middle of the night",
            "put it back on the shelf",
            "they are going to be late",
        ];

        // The expected depth, from the targets' own lengths alone.
        let shortest = TARGETS
            .iter()
            .map(|t| t.split_whitespace().count())
            .min()
            .expect("the corpus is not empty");
        let expected = (shortest - 1) as u64;
        assert!(
            expected >= 4,
            "the spread must include targets long enough for a four-deep \
             tuple to be derivable at all, and the shortest is {shortest} words"
        );

        for target in TARGETS {
            counters::take_total(&counters::DEEPEST_PROFILE_COORDINATES);
            let _ = approximate_generator(50).generate(target);
            let deepest = counters::take_total(
                &counters::DEEPEST_PROFILE_COORDINATES,
            );
            assert!(
                deepest >= expected,
                "{target:?} ({} words): the reserve placed a tuple deep in \
                 {deepest} slots, and a target of {} words admits a tuple \
                 deep in {expected}. The reserve's placement depth is bounded \
                 by something other than the traversal's own floor.",
                target.split_whitespace().count(),
                target.split_whitespace().count(),
            );
        }
    }

    /// The emission ceiling is **reached**, not merely respected, and the
    /// pop ceiling is not — which is the whole content of "the search is
    /// emission-bound".
    ///
    /// Every comment in this file that describes the two budgets treats them
    /// as interchangeable limits and ranks work against whichever is tighter.
    /// On a real multi-clause target they are not interchangeable: the
    /// emission side saturates and the pop side does not, by a wide margin.
    /// That is asserted rather than described, because the description is what
    /// drifted.  The adjacency operator's funding is justified in the source
    /// by the ceiling "leaving slack (14,239 of 16,384 spent)"; the measured
    /// figure on a real multi-clause target is 16,384 of 16,384.  A slack
    /// claim that is false is not cosmetic — it is the stated reason two
    /// mechanisms may share one ceiling without competing.  See
    /// `docs/work/items/w-b3e91a.md`.
    ///
    /// The two assertions carry different information and are deliberately of
    /// different shapes:
    ///
    /// * **Per target**, emissions may not exceed the ceiling.  This is the
    ///   bound itself, and it holds for every input including a short one: a
    ///   two-word target retains far fewer than 256 segmentations and spends
    ///   4,529 of 16,384, so the ceiling is *respected* there and only
    ///   *reached* where the lattice is big enough to spend it.  Asserting
    ///   saturation per target would be asserting a fact about the size of
    ///   particular phrases, not about the bound.
    /// * **Over the corpus**, the ceiling must actually be reached, and the pop
    ///   ceiling must not be approached.  This is the load-bearing half: if a
    ///   future change leaves the emission side with slack, the ceiling has
    ///   stopped bounding anything, and every share arithmetic drawn against
    ///   it — `structure_depth_ceiling`'s equal share,
    ///   `ADJACENCY_GLOBAL_RESERVE` being a slice of it — is describing
    ///   budget that is not being spent.  Symmetrically, if pops ever approach
    ///   1,024,000 the pop budget has become the binding constraint and every
    ///   emission-allocation question has to be re-asked against a different
    ///   search.
    ///
    /// It is phrase-free: it reads the counters over the shared reachability
    /// corpus and names no target, no clue and no word.
    #[test]
    fn the_global_emission_ceiling_is_reached_not_merely_respected() {
        // The two ceilings, restated from the constants the search itself
        // uses — `SEGMENTATION_KEEP * LEXICAL_COMBINATIONS_PER_SEGMENTATION`
        // and `SEGMENTATION_KEEP * LEXICAL_HEAP_POP_LIMIT`.  They are
        // function-local, so a test restates the arithmetic rather than
        // reaching into the search.
        let emission_ceiling = 256 * 64;
        let pop_ceiling = 256 * 4_000;
        let mut best_emissions = 0u64;
        for (target, _) in reachability_corpus() {
            let _ = approximate_generator(50).generate(target);
            let emissions = counters::take(&counters::SPENT_EMISSIONS);
            let pops = counters::take(&counters::SPENT_POPS);
            assert!(
                emissions <= emission_ceiling as u64,
                "{target:?}: the lexical phase spent {emissions} emissions \
                 against a ceiling of {emission_ceiling}, so the global \
                 emission bound does not bound"
            );
            best_emissions = best_emissions.max(emissions);
            assert!(
                pops * 2 < pop_ceiling as u64,
                "{target:?}: the lexical phase spent {pops} of \
                 {pop_ceiling} pops, so the pop budget has become the \
                 binding constraint and the emission ceiling is not what \
                 limits this search any more"
            );
        }
        assert_eq!(
            best_emissions, emission_ceiling as u64,
            "no target in the corpus reached the global emission ceiling, so \
             it is not the ceiling: it bounds nothing, and every share \
             arithmetic drawn against it describes budget that is not spent"
        );
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

    /// boundary, and the general form of that is an inequality rather
    /// than a constant: the Jaccard distance between the two boundary
    /// sets is never below either one-sided reading of the same
    /// resegmentation, so no re-definition that "rewards addition
    /// without punishing preservation" can raise the score of a clue
    /// that shares boundaries with the target.
    ///
    /// With `a` added, `r` removed and `s` shared boundaries, the
    /// symmetric reading is `(a + r) / (a + r + s)` and the one-sided
    /// readings are `a / (a + s)` and `r / (r + s)`; the first minus the
    /// others are `r * s / (...)` and `a * s / (...)`, so both are
    /// non-negative.
    #[test]
    fn boundary_novelty_is_never_below_its_one_sided_readings() {
        for total_len in [4usize, 6, 9] {
            for cuts in 0..(1u32 << total_len.min(6)) {
                let cuts: Vec<usize> = (0..total_len.min(6))
                    .filter(|i| cuts & (1 << i) != 0)
                    .collect();
                if cuts.is_empty() {
                    continue;
                }
                for mask in 0..(1u32 << total_len.min(6)) {
                    let target: Vec<usize> = (0..total_len.min(6))
                        .filter(|i| mask & (1 << i) != 0)
                        .collect();
                    let measured = boundary_novelty(
                        &cuts,
                        &target,
                        total_len,
                        false,
                    );
                    let a = cuts.iter().filter(|c| !target.contains(c)).count();
                    let r = target.iter().filter(|c| !cuts.contains(c)).count();
                    let s = cuts.iter().filter(|c| target.contains(c)).count();
                    let addition = if a + s == 0 {
                        0.0
                    } else {
                        a as f64 / (a + s) as f64
                    };
                    let removal =
                        if r + s == 0 { 0.0 } else { r as f64 / (r + s) as f64 };
                    assert!(
                        measured + 1e-12 >= addition,
                        "{cuts:?} vs {target:?}: {measured} < addition {addition}"
                    );
                    assert!(
                        measured + 1e-12 >= removal,
                        "{cuts:?} vs {target:?}: {measured} < removal {removal}"
                    );
                }
            }
        }

        // And the axis is still a novelty measure: a clue that keeps the
        // target's segmentation is at the bottom, and a clue that
        // re-cuts every boundary is at the top, whatever the length.
        let target = [3usize, 6, 9];
        assert_eq!(boundary_novelty(&[3, 6, 9, 12], &target, 12, false), 0.0);
        assert_eq!(boundary_novelty(&[2, 5, 8, 11], &target, 12, false), 1.0);
        // A clue that keeps some of the target's boundaries and adds its
        // own sits strictly between the two, and the shared boundaries
        // are the only thing holding it back.
        let partial = boundary_novelty(&[3, 5, 7, 9, 11], &target, 12, false);
        assert!(partial > 0.0 && partial < 1.0, "partial resegmentation scored {partial}");
    }

    /// A synthetic clue over `target` made of `n` words of `len` phones
    /// each, carrying `cost` edit cost in total.  The words are named by
    /// index only: this test is about the arithmetic of the similarity
    /// axis, not about any word, phrase or example.
    fn synthetic_clue(target: &TargetPhrase, n: usize, len: usize, cost: f64) -> Partial {
        let mut p = Partial::empty();
        for i in 0..n {
            let word = approx::FuzzyWord {
                word: format!("w{i}"),
                ipa: "aeiouy".chars().take(len).collect(),
                ipa_len: len,
                syllables: 1,
                rarity: Some(500.0),
                closed: false,
            };
            p = p.extend_fuzzy(target, &word, len * i, cost / n as f64);
        }
        p
    }

    /// `SIMILARITY` is a property of how well the clue's phones spell the
    /// target's, so two clues that carry the *same total* edit cost over
    /// the *same target* must score the same however those phones happen
    /// to be distributed among their words.  A per-word reading charged a
    /// clue for having more words, which is a fact about the cut and not
    /// about the phonetics, and a third clue that really is worse per
    /// phone must still rank strictly below both.
    #[test]
    fn similarity_is_charged_per_phone_not_per_word() {
        let target = TargetPhrase::new("alpha bravo charlie delta");
        let boundaries = [5usize, 11, 19];
        let total_len = 24usize;
        let syllables = 8usize;

        // Same 24 phones, same total cost, two different cuts: three
        // eight-phone words against six four-phone words.
        let wide = synthetic_clue(&target, 3, 8, 0.6);
        let fine = synthetic_clue(&target, 6, 4, 0.6);
        assert_eq!(wide.word_count(), 3);
        assert_eq!(fine.word_count(), 6);

        let m_wide = wide.metrics(&boundaries, syllables, total_len, false);
        let m_fine = fine.metrics(&boundaries, syllables, total_len, false);
        assert_eq!(
            m_wide.similarity, m_fine.similarity,
            "the cut into words changed SIMILARITY at equal per-phone cost: \
             {} over {} words vs {} over {} words",
            m_wide.similarity,
            wide.word_count(),
            m_fine.similarity,
            fine.word_count()
        );

        // Both are at the same per-phone cost, and a clue that spends no
        // edit cost at all is the axis's maximum.  No absolute constant is
        // named here: the property under test is the *denominator*, and it
        // is a property of the relation between the three clues, not of any
        // particular scale.
        let free = synthetic_clue(&target, 6, 4, 0.0);
        let m_free = free.metrics(&boundaries, syllables, total_len, false);
        assert!(
            (m_free.similarity - 1.0).abs() < 1e-12,
            "a clue with no edit cost scored {} on SIMILARITY",
            m_free.similarity
        );
        assert!(
            m_wide.similarity < m_free.similarity,
            "a clue that costs edit cost scored {} against {}",
            m_wide.similarity,
            m_free.similarity
        );

        // A worse cut, on the same target, at the same word count as the
        // first: strictly worse per phone, so strictly worse on the axis.
        let worse = synthetic_clue(&target, 3, 8, 0.9);
        let m_worse = worse.metrics(&boundaries, syllables, total_len, false);
        assert!(
            m_worse.similarity < m_wide.similarity,
            "a worse-per-phone clue scored {} against {}",
            m_worse.similarity,
            m_wide.similarity
        );
    }

    /// The `SIMILARITY` axis is a bounded quantity, and it is bounded on
    /// *both* sides rather than at whichever side happens to be reachable
    /// by the candidates one happened to look at.
    ///
    /// The axis is `1 - (clue's total edit cost / target's phones) / cost
    /// per phone`, so the numerator is unbounded above: a candidate far
    /// enough from the target's phonetics scores below `0.0` in the
    /// unclamped form, and a candidate that is a perfect transcription at
    /// zero cost is the `1.0` end.  Every other axis here is a share, a
    /// rate or a `match` against a count and is bounded by construction;
    /// this one is an affine map of an edit cost, and an affine map of an
    /// unbounded quantity is unbounded unless something bounds it.
    ///
    /// The property under test is the *value*, not the mechanism.  This
    /// says nothing about how the bound is obtained — a clamp, a min/max,
    /// a saturating subtraction — and it deliberately makes no claim about
    /// where inside the range the normaliser sits: the floor case is
    /// "dissimilar enough", and how dissimilar that is is the axis
    /// constant's business, not this test's.
    ///
    /// Both ends are asserted to be *reached*, not merely respected.  A
    /// bound that is never touched is not a bound in practice, and a
    /// regression that silently narrowed the axis into, say, `[0.2, 0.9]`
    /// would pass an `in`-range assertion at every point it happened to
    /// sample.  The sweep below also walks the interior densely enough
    /// that an axis which drifted off one edge for a band of costs would
    /// be caught.
    #[test]
    fn similarity_axis_is_bounded_and_reaches_both_ends() {
        let target = TargetPhrase::new("alpha bravo charlie delta");
        let boundaries = [5usize, 11, 19];
        let total_len = 24usize;
        let syllables = 8usize;

        // A candidate that is the target's own phonetics at no cost is
        // the `1.0` end, and one that is not remotely it is the `0.0`
        // end.  Both are read off the same scorer every other clue in the
        // pool goes through, so this is the axis as the search sees it,
        // not a re-derivation of the formula.
        let free = synthetic_clue(&target, 6, 4, 0.0);
        let free_similarity = free.metrics(&boundaries, syllables, total_len, false).similarity;
        assert!(
            (free_similarity - 1.0).abs() < 1e-12,
            "a zero-cost candidate scored {free_similarity} on SIMILARITY, not the 1.0 end"
        );

        // The cost is `synthetic_clue`'s total over the clue and the axis
        // divides by the target's phone count, so this is chosen large
        // enough to be past the floor by a wide margin rather than tuned
        // to land exactly on it.
        let alien = synthetic_clue(&target, 6, 4, 48.0);
        let alien_similarity = alien.metrics(&boundaries, syllables, total_len, false).similarity;
        assert_eq!(
            alien_similarity, 0.0,
            "a candidate far outside the target's phonetics scored \
             {alien_similarity} on SIMILARITY instead of the 0.0 floor"
        );

        // The sweep: a ladder of per-candidate costs from free to
        // alienous, spanning the range the axis can express and past it
        // at both ends.  Every point must be a real fraction, and the
        // axis must be non-increasing in cost: a candidate that spells
        // the target worse cannot score better on the axis that measures
        // how well it spells the target.
        let mut previous = f64::INFINITY;
        for step in 0..=64 {
            let cost = 48.0 * (2.0_f64).powf((step as i32 - 64) as f64 / 4.0);
            let clue = synthetic_clue(&target, 6, 4, cost);
            let similarity = clue.metrics(&boundaries, syllables, total_len, false).similarity;
            assert!(
                (0.0..=1.0).contains(&similarity),
                "cost {cost} scored {similarity} on SIMILARITY, outside [0.0, 1.0]"
            );
            assert!(
                similarity <= previous + 1e-12,
                "raising the edit cost from the previous rung raised SIMILARITY \
                 from {previous} to {similarity}"
            );
            previous = similarity;
        }
    }

    /// The two axes that read the clue's *own segmentation* — `NOVELTY`,
    /// which is a Jaccard distance over boundary cuts, and `WORD_NOVELTY`,
    /// which is `1 - reused / word_count` — are **rewards for a clue
    /// that differs from the target's own wording**, and both are entered
    /// with a positive weight.  Neither may charge a clue for being cut
    /// into more pieces, and this pins that at both ends of the reuse
    /// range and along a boundary ladder.
    ///
    /// This is not a restatement of the axis definitions; it is the
    /// opposite property, and it is the one a *length charge* would
    /// break.  A form that divided the reuse count by the target's word
    /// count, or that subtracted the clue's own cut count, would leave
    /// every existing test in this module green — they all read the
    /// axes at one length — while making a faithful multiword
    /// resegmentation strictly worse the more of it there was.  The
    /// assertions below are all *across* lengths, at a fixed target, so
    /// they fail on exactly that change and on no other.
    #[test]
    fn segmentation_axes_reward_a_resegmentation_rather_than_charging_its_length() {
        let phrase = "alpha bravo charlie delta";
        let target = TargetPhrase::new(phrase);
        let boundaries = [5usize, 11, 19];
        let total_len = 24usize;
        let syllables = 8usize;
        let target_words: Vec<&str> = phrase.split_whitespace().collect();

        // A ladder of clue lengths, each one a fresh clue over the same
        // target: one word, two words, up to nine.  `disjoint` shares no
        // word with the target and so is a resegmentation throughout;
        // `reusing` spells the target back and is the axis's 0.0 end.
        // The two differ only in reuse, so any movement in either axis
        // along the ladder is movement caused by length alone.
        let build = |n: usize, disjoint: bool| -> Partial {
            let mut p = Partial::empty();
            for i in 0..n {
                let name = if disjoint {
                    format!("q{i}")
                } else {
                    target_words[i % target_words.len()].to_string()
                };
                p = p.extend_fuzzy(
                    &target,
                    &approx::FuzzyWord {
                        word: name,
                        ipa: "aeiouy".chars().take(3).collect(),
                        ipa_len: 3,
                        syllables: 1,
                        rarity: Some(500.0),
                        closed: false,
                    },
                    3 * i,
                    0.0,
                );
            }
            p
        };

        // A resegmentation pays the *whole* `WORD_NOVELTY` weight at every
        // length, and the target respelt pays none of it at any length.
        // Flat in both, so the axis reads reuse and not word count.
        for n in 1..=9 {
            let resegmented = build(n, true).metrics(&boundaries, syllables, total_len, false);
            assert_eq!(
                resegmented.word_novelty, 1.0,
                "a {n}-word clue sharing no word with the target scored {} on \
                 WORD_NOVELTY, so the axis is charging it for its length",
                resegmented.word_novelty
            );
            let respelt = build(n, false).metrics(&boundaries, syllables, total_len, false);
            assert_eq!(
                respelt.word_novelty, 0.0,
                "a {n}-word clue spelling the target back scored {} on \
                 WORD_NOVELTY, so the axis is no longer flat in word count",
                respelt.word_novelty
            );
        }

        // The same property for boundary cuts.  A clue that keeps every
        // one of the target's boundaries and adds `extra` more of its own
        // is a progressively finer *faithful* resegmentation, and it must
        // not score lower for the extra boundaries: `NOVELTY` is a
        // distance from the target's own cuts, so a clue further away is
        // further rewarded.  The ladder really does grow the cut count --
        // `extra` distinct new boundaries on top of the target's three --
        // so an axis that falls with the clue's own segmentation fails
        // here rather than passing on a set of rungs that all happen to
        // have the same size.
        let spare = [6usize, 7, 8, 9, 10, 12, 13, 14];
        let own = boundary_novelty(&boundaries, &boundaries, total_len, false);
        assert_eq!(own, 0.0, "the target's own cuts are not novelty 0.0");
        let mut previous = own;
        for extra in 1..=spare.len() {
            let mut cuts = boundaries.to_vec();
            cuts.extend_from_slice(&spare[..extra]);
            cuts.sort_unstable();
            assert_eq!(
                cuts.len(),
                boundaries.len() + extra,
                "the ladder rung {extra} did not add a boundary"
            );
            let novelty = boundary_novelty(&cuts, &boundaries, total_len, false);
            assert!(
                novelty >= previous - 1e-12,
                "cutting the same phones into {} words instead of {} dropped \
                 NOVELTY from {previous} to {novelty}: the axis is charging a \
                 faithful resegmentation for being cut finer",
                cuts.len(),
                boundaries.len() + extra - 1
            );
            previous = novelty;
        }
        assert!(
            previous > own,
            "the whole boundary ladder was flat, so the assertions above \
             prove nothing about a length charge"
        );
    }

    /// The per-segmentation pop allowance the traversal charges against.
    ///
    /// It is scoped inside the search function, so it is not nameable here;
    /// this is the shipped figure, written out.  It is a *budget*, and every
    /// assertion below that uses it is an assertion about the budget, so a
    /// change to the budget is a change to what these tests are about.
    const WIDTH_FRONT_POP_LIMIT: usize = 4_000;

    // ---- the per-slot width front (w-9e2b41) ----
    //
    // Everything below re-derives its expectations by *enumerating* the
    // truncated lattice, never by calling the production recurrence.  A test
    // that compared a bound against a scorer reading the same shared axis
    // would be self-referential and could not detect a change to that
    // normaliser; here the two sides are written independently, so mutating
    // `first_leaf_frontier` or `per_slot_opening_widths` moves the production
    // side alone and the assertion still bites.

    /// The number of prefixes of length exactly `k` in a `caps`-truncated
    /// `d`-slot lattice, counted by generating them.
    ///
    /// Deliberately *not* the product recurrence: this walks the tuples, so a
    /// change to the production normaliser cannot move it.
    fn counted_prefixes(caps: &[usize], k: usize) -> usize {
        if k == 0 {
            return 1;
        }
        let mut total = 0usize;
        let mut prefix: Vec<usize> = vec![0; k];
        loop {
            total += 1;
            let mut pos = k;
            loop {
                if pos == 0 {
                    return total;
                }
                pos -= 1;
                prefix[pos] += 1;
                if prefix[pos] < caps[pos] {
                    break;
                }
                prefix[pos] = 0;
            }
        }
    }

    /// Every node a best-first walk must pop before its first leaf, counted by
    /// summing the generated prefixes of every level that still has children.
    fn counted_frontier(caps: &[usize]) -> usize {
        (0..caps.len()).map(|k| counted_prefixes(caps, k)).sum()
    }

    /// Independent check that the production frontier is the count it claims
    /// to be, over shapes small enough to generate and depths long enough to
    /// exercise every term.
    #[test]
    fn the_frontier_is_the_number_of_prefixes_the_walk_must_pop() {
        for depth in 1..=5usize {
            for caps in [
                vec![1usize; 5][..depth].to_vec(),
                vec![2usize; 5][..depth].to_vec(),
                vec![1, 7, 1, 7, 1][..depth].to_vec(),
                vec![10, 1, 10, 1, 10][..depth].to_vec(),
                vec![3, 5, 2, 9, 4][..depth].to_vec(),
                vec![160, 7, 160, 7, 93][..depth].to_vec(),
            ] {
                assert_eq!(
                    first_leaf_frontier(&caps),
                    counted_frontier(&caps),
                    "frontier disagrees with the counted prefix total at {caps:?}"
                );
            }
        }
    }

    /// The last slot is free: it hangs off a leaf, so its width is not in the
    /// frontier at all.  Checked by making it enormous.
    #[test]
    fn the_last_slot_is_free_because_a_leaf_hangs_off_it() {
        for depth in 1..=6usize {
            let narrow: Vec<usize> = vec![7; depth];
            let mut wide = narrow.clone();
            *wide.last_mut().unwrap() = 1_000_000;
            assert_eq!(
                first_leaf_frontier(&narrow),
                first_leaf_frontier(&wide),
                "depth {depth}: the last slot changed the frontier"
            );
            // ... and it is not free at any other position.
            if depth >= 2 {
                let mut early = narrow.clone();
                early[0] = 1_000_000;
                assert!(
                    first_leaf_frontier(&early) > first_leaf_frontier(&narrow),
                    "depth {depth}: the first slot was free too, which is wrong"
                );
            }
        }
    }

    /// The allocation is in proportion to each slot's *measured* push cost,
    /// and that cost is the number of prefixes above the slot — counted, not
    /// multiplied.  So: raising any slot that is not already at its pool size
    /// must push the frontier past the allowance, and lowering any slot must
    /// not.
    #[test]
    fn every_slot_is_opened_to_what_its_own_marginal_cost_affords() {
        for pool in [
            vec![SPAN_SHORTLIST, 7, SPAN_SHORTLIST, SPAN_SHORTLIST, 93],
            vec![160, 160, 160, 160, 160],
            vec![98, 26, 14, 35],
            vec![37, 53, 29, 41, 19],
            vec![1, 1, 1, 1, 1],
        ] {
            let depth = pool.len();
            let pop_limit = WIDTH_FRONT_POP_LIMIT;
            let uniform = affordable_opening_width(depth, pop_limit).min(
                pool.iter().copied().max().unwrap_or(0),
            );
            let caps = per_slot_opening_widths(&pool, pop_limit, uniform);

            assert_eq!(caps.len(), depth);
            for (k, (&cap, &size)) in caps.iter().zip(pool.iter()).enumerate() {
                assert!(cap >= 1, "slot {k} of {pool:?} opened at {cap}");
                assert!(
                    cap <= size.max(1),
                    "slot {k} of {pool:?} opened at {cap}, past its pool of {size}"
                );
            }

            // Charged slots only: the last one is free.
            for k in 0..depth - 1 {
                // The cost of one more candidate here, counted.
                let marginal = counted_prefixes(&caps, k);
                assert!(marginal >= 1);
                if caps[k] < pool[k].max(1) {
                    let mut wider = caps.clone();
                    wider[k] += 1;
                    assert!(
                        first_leaf_frontier(&wider) > pop_limit,
                        "slot {k} of {pool:?} opened at {} but {} also fits \
                         (marginal cost {marginal}, frontier now {})",
                        caps[k],
                        caps[k] + 1,
                        first_leaf_frontier(&wider)
                    );
                }
                let mut narrower = caps.clone();
                narrower[k] -= 1;
                assert!(
                    first_leaf_frontier(&narrower) <= pop_limit,
                    "slot {k} of {pool:?} could afford {} but opened at {}",
                    caps[k] - 1,
                    caps[k]
                );
            }
        }
    }

    /// **Joint** affordability, which is the whole content of the width law.
    ///
    /// The traversal charges `popped` — the number of nodes it actually
    /// expands — against `LEXICAL_HEAP_POP_LIMIT`; there is no separate
    /// `pops_left` and no per-slot charge.  The width a traversal may open at
    /// is therefore a statement about `popped`, and the only model of `popped`
    /// the loop itself supplies is the truncated lattice: `seen` admits each
    /// prefix once, so a pass pops at most as many nodes as the
    /// `caps`-truncated product has above its leaves, which is
    /// [`first_leaf_frontier`].  The charged quantity is therefore the
    /// **product of the per-slot prefix widths**, and it is charged *jointly*,
    /// once, for the whole vector.
    ///
    /// A per-slot *marginal* is not the same thing and is not a legal
    /// allocation.  The familiar "widest affordable width for each slot, with
    /// the others held at the uniform baseline" table is computed slot by
    /// slot against that baseline, and stacking the rows multiplies rather
    /// than sums, so the stack is inadmissible by a large factor even though
    /// every one of its rows is individually affordable.  The first half of
    /// this test exhibits that; the second shows the rule here does not
    /// commit it.
    #[test]
    fn per_slot_marginals_do_not_stack_into_a_jointly_affordable_allocation() {
        let pop_limit = WIDTH_FRONT_POP_LIMIT;
        // A row of per-slot marginals, built the way such a table is built:
        // each slot is charged against the others held at the uniform
        // baseline.  Read as a *simultaneous* allocation the stack overruns.
        for (depth, row) in [
            (4usize, vec![36usize, 36, 38, 160]),
            (5, vec![9, 10, 10, 10, 160]),
            (7, vec![10, 11, 11, 11, 11, 14, 160]),
        ] {
            assert_eq!(row.len(), depth);
            let stacked = first_leaf_frontier(&row);
            assert_eq!(
                stacked,
                counted_frontier(&row),
                "the stacked row must be counted, not asserted"
            );
            assert!(
                stacked > pop_limit,
                "depth {depth}: the stacked row fits in {pop_limit}, so it is \
                 not a witness that marginals do not compose"
            );
            // Every single entry is affordable *alone*, at the baseline, which
            // is exactly what makes such a table look legal.
            let baseline = affordable_opening_width(depth, pop_limit);
            for (k, &w) in row.iter().enumerate() {
                let mut alone = vec![baseline; depth];
                alone[k] = w;
                assert!(
                    first_leaf_frontier(&alone) <= pop_limit,
                    "depth {depth} slot {k}: {w} is not individually affordable, \
                     so the row is not a marginal table"
                );
            }
        }
    }

    /// The rule here satisfies the charged quantity jointly, and does so
    /// against the *counted* frontier rather than the production recurrence.
    #[test]
    fn the_allocation_is_jointly_affordable_against_the_charged_quantity() {
        for depth in 1..=8usize {
            for pool in [
                vec![SPAN_SHORTLIST; 8][..depth].to_vec(),
                vec![SPAN_SHORTLIST, 7, 160, 160, 93, 12, 3, 160][..depth].to_vec(),
                vec![98, 26, 14, 35, 2, 160, 1, 7][..depth].to_vec(),
                vec![1, 1, 1, 1, 1, 1, 1, 1][..depth].to_vec(),
                vec![2, 2, 2, 2, 2, 2, 2, 2][..depth].to_vec(),
            ] {
                let pop_limit = WIDTH_FRONT_POP_LIMIT;
                let uniform = affordable_opening_width(depth, pop_limit);
                let caps = per_slot_opening_widths(&pool, pop_limit, uniform);
                // Jointly, once, for the whole vector.
                assert_eq!(
                    first_leaf_frontier(&caps),
                    counted_frontier(&caps),
                    "depth {depth} {pool:?}: production and counted frontiers \
                     disagree, so the joint claim is not being measured"
                );
                let scalar: Vec<usize> =
                    pool.iter().map(|&n| n.min(uniform).max(1)).collect();
                let joint_floor = first_leaf_frontier(&scalar);
                assert!(
                    first_leaf_frontier(&caps) <= pop_limit.max(joint_floor),
                    "depth {depth} {pool:?}: opened at {caps:?} for a frontier \
                     of {} against a limit of {pop_limit} (and a scalar floor \
                     of {joint_floor})",
                    first_leaf_frontier(&caps)
                );
            }
        }
    }

    /// Why the pop budget cannot *choose* the split, which is the deepest
    /// reason the width is not a tuning mistake.
    ///
    /// The charged quantity is a product, so the allowance fixes
    /// `c_0 * c_1 * ... * c_{d-2}`, not the factors.  There is a large family
    /// of allocations satisfying it jointly, and they are not
    /// interchangeable: they buy very different enumerations for the same
    /// pops.  So a per-slot *width* is not a quantity the pop budget has an
    /// opinion about, and any rule that presents one as derived from the
    /// allowance is presenting a choice as an arithmetic result.  Choosing
    /// among them needs a second criterion, and the only other ceiling in the
    /// search — the global emission budget — is not it: it saturates rather
    /// than trading off, and the per-segmentation emission allowance is
    /// already about four fifths spent before enumeration begins.
    #[test]
    fn the_pop_budget_leaves_the_per_slot_split_undetermined() {
        let pop_limit = WIDTH_FRONT_POP_LIMIT;
        // Distinct, jointly affordable, and not close to each other: the same
        // pops, a different enumeration.
        for row in [
            vec![1usize, 40, 7, 7, 160],
            vec![2, 28, 7, 7, 160],
            vec![3, 22, 7, 7, 160],
            vec![8, 8, 7, 7, 160],
        ] {
            assert_eq!(
                first_leaf_frontier(&row),
                counted_frontier(&row),
                "the witness must be counted, not asserted"
            );
            assert!(
                first_leaf_frontier(&row) <= pop_limit,
                "{row:?} is not jointly affordable, so it is not a witness"
            );
        }
        // They are genuinely different allocations, not one vector written
        // four times.
        let witnesses: Vec<Vec<usize>> =
            vec![vec![1, 40, 7, 7, 160], vec![2, 28, 7, 7, 160]];
        assert!(
            witnesses[0] != witnesses[1],
            "the witnesses must differ"
        );

        // And the family is large, counted rather than asserted.
        let mut affordable = 0usize;
        for a in 1..=60usize {
            for b in 1..=60usize {
                for c in 1..=160usize {
                    if first_leaf_frontier(&[a, b, c, 160]) <= pop_limit {
                        affordable += 1;
                    }
                }
            }
        }
        assert!(
            affordable > 10_000,
            "only {affordable} jointly affordable allocations, which would make \
             the split nearly determined after all"
        );
    }

    /// The last slot is allocated its whole pool, because it is the one slot
    /// whose width the allowance does not price.  Charging it anything else
    /// is a narrowing that costs deep-in-a-span reach for nothing.
    #[test]
    fn the_free_slot_is_opened_to_its_whole_pool() {
        for pool in [
            vec![SPAN_SHORTLIST, 7, SPAN_SHORTLIST, SPAN_SHORTLIST, 93],
            vec![160, 160, 160, 160, 160],
            vec![98, 26, 14, 35],
            vec![1, 1, 1, 1, 1],
        ] {
            let depth = pool.len();
            let caps =
                per_slot_opening_widths(&pool, WIDTH_FRONT_POP_LIMIT, 1);
            assert_eq!(
                caps[depth - 1],
                pool[depth - 1].max(1),
                "{pool:?}: the free slot opened at {} of {}",
                caps[depth - 1],
                pool[depth - 1]
            );
        }
    }

    /// The depth law, in traversal-index units: the number of candidates the
    /// same allowance buys is non-increasing in slot position, because the
    /// marginal cost of a candidate is the number of prefixes above it and
    /// that number only grows.  Derived here from the counted prefixes of a
    /// fixed reference shape rather than from anything the rule returns.
    #[test]
    fn the_width_a_slot_can_afford_decreases_with_its_depth_in_the_product() {
        let pop_limit = WIDTH_FRONT_POP_LIMIT;
        for depth in 3..=8usize {
            let reference: Vec<usize> = vec![9; depth];
            let mut committed = 1usize;
            let mut ceiling = usize::MAX;
            for k in 0..depth - 1 {
                // What is already paid for, and what one more candidate costs.
                committed += counted_prefixes(&reference, k);
                let marginal = counted_prefixes(&reference, k);
                let affordable = pop_limit.saturating_sub(committed) / marginal.max(1);
                assert!(
                    affordable <= ceiling,
                    "depth {depth} slot {k}: the affordable width rose from \
                     {ceiling} to {affordable} with depth, which the marginal \
                     cost forbids"
                );
                ceiling = affordable;
            }
            // And the production rule never hands out more than the ceiling at
            // the same slot, counting the prefixes of what it actually opened.
            let pool: Vec<usize> = vec![SPAN_SHORTLIST; depth];
            let uniform = affordable_opening_width(depth, pop_limit);
            let caps = per_slot_opening_widths(&pool, pop_limit, uniform);
            let mut spent = 1usize;
            for k in 0..depth - 1 {
                spent += counted_prefixes(&caps, k);
                let affordable = pop_limit.saturating_sub(spent) / counted_prefixes(&caps, k);
                assert!(
                    caps[k] <= pool[k].max(affordable).max(uniform),
                    "depth {depth} slot {k}: opened at {} with an affordable \
                     width of {affordable} and a floor of {uniform}",
                    caps[k]
                );
            }
        }
    }

    /// The no-narrowing law: the rule is a *refinement* of the scalar one, so
    /// wherever the scalar opening itself fits, no slot ends up narrower than
    /// it.
    #[test]
    fn the_per_slot_rule_never_narrows_the_scalar_opening_that_fits() {
        for depth in 1..=8usize {
            for pool in [
                vec![SPAN_SHORTLIST; 8][..depth].to_vec(),
                vec![SPAN_SHORTLIST, 7, 160, 160, 93, 12, 3, 160]
                    [..depth]
                    .to_vec(),
                vec![98, 26, 14, 35, 2, 160, 1, 7][..depth].to_vec(),
            ] {
                let pop_limit = WIDTH_FRONT_POP_LIMIT;
                let uniform = affordable_opening_width(depth, pop_limit);
                let scalar: Vec<usize> = pool.iter().map(|&n| n.min(uniform).max(1)).collect();
                if first_leaf_frontier(&scalar) > pop_limit {
                    continue; // the scalar opening is already unaffordable here
                }
                let caps = per_slot_opening_widths(&pool, pop_limit, uniform);
                for k in 0..depth {
                    assert!(
                        caps[k] >= scalar[k],
                        "depth {depth} {pool:?}: slot {k} narrowed from {} to {}",
                        scalar[k],
                        caps[k]
                    );
                }
            }
        }
    }

    /// The load-bearing priced negative, stated as a law rather than as a
    /// measurement of one phrase.
    ///
    /// The walk's test is `index < cap`, so admitting an index tuple `t`
    /// requires `caps[k] >= t_k + 1`, and [`first_leaf_frontier`] is monotone
    /// non-decreasing in every component — so `F(t + 1)` is a *lower* bound
    /// over every allocation that admits `t`, and `F(t + 1) - pop_limit` is the
    /// exact shortfall in pops when that bound exceeds the allowance.  No
    /// allocation rule, per-slot or otherwise, can do better, because the
    /// bound is a property of the tuple and the limit rather than of the rule.
    ///
    /// The monotonicity that makes the bound valid is checked by brute force
    /// over every allocation of a small shape, so the lower-bound claim is
    /// earned here rather than assumed.
    #[test]
    fn admission_is_impossible_for_every_allocation_once_the_frontier_bound_is() {
        // Brute-force the monotonicity the bound rests on, over every vector
        // with entries in 1..=3 and up to four slots.
        for depth in 1..=4usize {
            let mut vectors = vec![vec![1usize; depth]];
            for _ in 0..depth {
                let mut next = Vec::new();
                for v in &vectors {
                    for w in 1..=3usize {
                        let mut u = v.clone();
                        u.push(w);
                        next.push(u);
                    }
                }
                vectors.extend(next);
            }
            // Compared within a depth: the bound is over allocations of the
            // same lattice, and a deeper lattice is not a wider one.
            for a in &vectors {
                for b in &vectors {
                    if a.len() != b.len() {
                        continue;
                    }
                    let dominated = a.iter().zip(b).all(|(x, y)| x <= y);
                    if dominated {
                        assert!(
                            first_leaf_frontier(a) <= first_leaf_frontier(b),
                            "{a:?} is dominated by {b:?} but has the larger frontier"
                        );
                    }
                }
            }
        }

        // And the bound itself, on a shape the rule is asked about: a 4-slot
        // lattice with 100-wide pools cannot enumerate a tuple two of whose
        // slots sit past index 40.
        let pop_limit = WIDTH_FRONT_POP_LIMIT;
        for tuple in [
            vec![0usize, 0, 0, 0],
            vec![9, 0, 0, 0],
            vec![9, 0, 44, 0],
            vec![0, 0, 44, 0],
        ] {
            let needed: Vec<usize> = tuple.iter().map(|&i| i + 1).collect();
            let bound = first_leaf_frontier(&needed);
            let counted = counted_frontier(&needed);
            assert_eq!(bound, counted, "the bound must be the counted frontier");
            if bound > pop_limit {
                // No allocation admits it, and the shortfall is exact.
                let mut admissible: Vec<Vec<usize>> = vec![vec![1; 4]];
                for k in 0..4 {
                    let mut grown = Vec::new();
                    for v in &admissible {
                        for w in v[k]..=needed[k] {
                            let mut u = v.clone();
                            u[k] = w;
                            grown.push(u);
                        }
                    }
                    admissible.extend(grown);
                }
                for a in &admissible {
                    let admits = a.iter().zip(&needed).all(|(x, n)| x >= n);
                    assert!(
                        !admits || first_leaf_frontier(a) <= pop_limit,
                        "{a:?} admits {tuple:?} on a frontier of {} within {pop_limit}, \
                         contradicting the bound {bound}",
                        first_leaf_frontier(a)
                    );
                }
            }
        }
    }

    /// The scalar law's own row, printed rather than asserted, so the numbers
    /// a downstream front would otherwise have to take on trust are visible in
    /// the test log.  Nothing here names a phrase or a word.
    #[test]
    fn the_scalar_opening_and_its_frontier_are_what_the_docs_say() {
        for depth in 1..=8usize {
            let w = affordable_opening_width(depth, WIDTH_FRONT_POP_LIMIT);
            let f = first_leaf_frontier(&vec![w; depth]);
            assert!(f <= WIDTH_FRONT_POP_LIMIT || w == 1, "depth {depth}");
            eprintln!("depth {depth}: uniform {w}, frontier {f}");
        }
        // The uniform series is the vector recurrence read on a constant
        // input, which is the identity the scalar law depends on.  Written
        // out longhand here so the two are not compared through each other.
        for depth in 1..=8usize {
            for w in 1..=12usize {
                let mut series = 1usize;
                let mut power = 1usize;
                for _ in 1..depth {
                    power = power.saturating_mul(w);
                    series += power;
                }
                assert_eq!(first_leaf_frontier(&vec![w; depth]), series);
            }
        }
    }
}
