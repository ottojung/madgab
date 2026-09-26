//! Characterisation of the structural DP's **pool retention**, from outside
//! the crate.
//!
//! # Why this file exists
//!
//! [w-6f2b18](../docs/work/items/w-6f2b18.md) opened a front on the claim
//! that the number of segmentations the structural DP retains should be
//! derived from what the traversal does with them, rather than being the
//! hand-picked `SEGMENTATION_KEEP = 256`. Its measurement pass refuted that
//! claim on the real corpus, and this file is the part of the measurement
//! that survives as a guard: it pins the *externally observable* consequence
//! of the census so that a later retention change has to argue with it.
//!
//! # What the census measured, and what it means for a change here
//!
//! Measured on two real multi-clause targets at `--approximate --top 50` on
//! this head, with a per-segmentation emission tally added temporarily to
//! `Generator::build` and removed again:
//!
//! | | target A | target B |
//! | --- | --- | --- |
//! | segmentations the DP completed | 1854 | 369 |
//! | retained after `SEGMENTATION_KEEP` | 256 | 256 |
//! | distinct boundary structures among the retained | 256 | 256 |
//! | retained segmentations the traversal actually walked | 256 | 256 |
//! | retained segmentations that emitted **nothing** | **0** | **0** |
//! | retained segmentations that hit the per-segmentation pop limit | 57 | 86 |
//! | global emission budget spent | 15206 / 16384 | 13433 / 16384 |
//! | global pop budget spent | 505268 / 1024000 | 515111 / 1024000 |
//! | retained pool | 17827 | 15926 |
//!
//! Three things follow, and they are the reason no retention change landed:
//!
//! 1. **Retention is already saturated against the emission budget.** The
//!    global emission budget is `SEGMENTATION_KEEP *
//!    LEXICAL_COMBINATIONS_PER_SEGMENTATION` and every retained
//!    segmentation's allowance is clamped to at most
//!    `LEXICAL_COMBINATIONS_PER_SEGMENTATION`, so `SEGMENTATION_KEEP` is
//!    not a free parameter at all: it is
//!    `LEXICAL_GLOBAL_EMISSION_BUDGET / LEXICAL_COMBINATIONS_PER_SEGMENTATION`
//!    = `16384 / 64` = 256, the exact number of boundary structures the
//!    budget can fund one ceiling deep. Retaining more spends the same
//!    budget thinner; the measured per-segmentation ceiling falls
//!    64 -> 44 -> 28 -> 19 as the retained count goes 256 -> 384 -> 512 ->
//!    1024. Retaining fewer drops structures that were funding themselves:
//!    at 152 retained the pool falls 17827 -> 13606.
//! 2. **There is no empty-segmentation waste to reclaim at this head.** The
//!    claim that motivated the front - that a large share of retained
//!    segmentations emit nothing - measures **0 of 256** on both targets.
//!    The reproducible nearby quantity is the pop-censored count (57 and 86
//!    of 256): segmentations that spend the whole per-segmentation pop
//!    limit and still emit well under their allowance. Freeing their pops
//!    cannot help, because the pop budget is only half spent and a freed pop
//!    cannot lift any segmentation above the emission ceiling it already
//!    reaches.
//! 3. **Therefore the breadth a user asks for is not a retention
//!    quantity.** Asking for five times the visible list does not ask for
//!    five times the retained pool, because the retention count is pinned by
//!    the emission budget rather than by `top_n`. That is the property
//!    asserted below, and it is the one a "derive retention from measured
//!    productivity" change would break first.
//!
//! # Red on the hypothesis, green on the base
//!
//! `pool_breadth_is_not_linear_in_top_n` is green here and was run red
//! against a build that implements the front's own hypothesis 2 (retain the
//! DP's whole completed population, 1854 rather than 256). On this target the
//! base pool goes 6576 at `--top 10` to 11213 at `--top 50` - 1.71x, against
//! 5x for the visible list - and the hypothesis-2 build goes 16567 to 35201,
//! 2.12x, which trips the assertion below. The numbers are recorded in the
//! work item; the knob itself was scratch and is not on the branch.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// A multi-clause target with no relationship to either canonical example,
/// so nothing here can be satisfied by an example-shaped special case.
const TARGET: &str = "we sell sea shells by the sea shore";

fn generator(top_n: usize) -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse")
}

fn visible_and_pool(top_n: usize) -> (Vec<String>, usize) {
    let (clues, pool) = generator(top_n).generate_with_pool(TARGET);
    (
        clues.iter().map(|c| c.phrase.to_lowercase()).collect(),
        pool,
    )
}

fn boundary_structures(clues: &[String]) -> usize {
    let generator = generator(50);
    let seen: std::collections::HashSet<Vec<usize>> = generator
        .generate_with_pool(TARGET)
        .0
        .iter()
        .map(|c| c.cuts.clone())
        .collect();
    let _ = clues;
    seen.len()
}

/// The visible list must be a selection *from* the pool, and the pool must
/// be strictly larger than the list, or "the pool is what the search
/// reached" would not be a meaningful claim about reachability.
#[test]
fn pool_is_wider_than_the_visible_list() {
    let (visible, pool) = visible_and_pool(20);
    assert!(!visible.is_empty(), "no proposals for {TARGET:?}");
    assert!(
        pool > visible.len(),
        "pool {pool} is not wider than the {} visible proposals",
        visible.len()
    );
}

/// Retention is a function of the emission budget, not of `top_n`, so the
/// retained pool grows far more slowly than the visible list does. If a
/// change makes the pool track `top_n` - which is what "derive retention
/// from the measured population" degenerates into - this fails.
#[test]
fn pool_breadth_is_not_linear_in_top_n() {
    let (narrow_visible, narrow_pool) = visible_and_pool(10);
    let (wide_visible, wide_pool) = visible_and_pool(50);
    assert!(
        wide_visible.len() > narrow_visible.len(),
        "the wider request did not produce a wider list: {} vs {}",
        wide_visible.len(),
        narrow_visible.len()
    );
    assert!(
        wide_pool < narrow_pool * 2,
        "the retained pool {} tracked the visible list ({} -> {}) instead \
         of staying pinned by the emission budget ({} -> {})",
        wide_pool,
        narrow_visible.len(),
        wide_visible.len(),
        narrow_pool,
        wide_pool,
    );
}

/// The pool behind a request is a function of the target and the mode, not
/// of the order the two are observed in, and the visible list is drawn from
/// it. Pairs with `approx_determinism`, which checks the process-level
/// property; this pins the pool/visible relationship in one process.
#[test]
fn pool_and_visible_list_are_reproducible_in_process() {
    let first = visible_and_pool(15);
    let second = visible_and_pool(15);
    assert_eq!(
        first.1, second.1,
        "pool size moved between two identical requests: {} vs {}",
        first.1, second.1
    );
    assert_eq!(first.0, second.0, "visible list moved between two identical requests");
}

/// The census is only meaningful if the visible list actually spans several
/// boundary structures: a pool that is wide but whose visible half comes
/// from one structure would make "how many structures were retained"
/// unobservable from outside.
#[test]
fn visible_list_spans_several_boundary_structures() {
    let (visible, _) = visible_and_pool(50);
    let structures = boundary_structures(&visible);
    assert!(
        structures >= 3,
        "expected the visible list to span several boundary structures, \
         got {structures} for {} proposals",
        visible.len()
    );
}
