//! The span shortlist must keep the whole cost range, not just the cheap end.
//!
//! The approximate search bounds the per-span alternative count with a
//! shortlist: a span offers up to 256 candidates and retains 160. Six
//! named admission passes (cost, familiarity, rarity, and four cost
//! bands with their own quality / familiarity / rarity sub-passes) fill
//! the first 100 slots, and a *fill* spends the rest.
//!
//! The fill is the only part of the rule that decides contents, and it is
//! where a general property is easy to lose: the single ranking score
//! used for it degrades monotonically with phonetic distance, so a fill
//! that spends its slots globally leaves the expensive end of the cost
//! range represented by whatever happens to survive a global comparison.
//! A shortlist is then not guaranteed to hold one word from every part of
//! the span's own cost distribution, and a wording that is only
//! competitive in a dearer band is unreachable at any traversal width.
//!
//! The property asserted here is stated on measured quantities only —
//! band population, cost, and what the shortlist retained — and the band
//! edges are **re-derived from the data inside the test**, as the quartile
//! cuts of the span's own candidate costs. They are deliberately not read
//! from the selection rule's own cost axis, so a future change to that
//! axis cannot make this assertion vacuous or hide a regression behind a
//! constant it also moved.
//!
//! Run with:
//!
//! ```text
//! cargo test --release --test span_shortlist_bands
//! ```

use std::collections::{BTreeMap, HashSet};

use madgab::{Generator, GeneratorConfig, SearchMode, SpanCandidate};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// The per-word budget the production approximate default uses.
const PER_WORD_BUDGET: f64 = 0.5;

/// A spread of real targets of different lengths and phoneme mixes, none
/// of them a canonical example and none of them named for one: the
/// property is about spans, so it is stated over spans.
const TARGETS: &[&str] = &[
    "taco cat",
    "I love you",
    "recognize speech",
    "It's just a stupid game",
    "the cat sat on the mat",
    "can you hear me now",
    "a whole lot of trouble",
    "my brother has a red car",
    "we should have told her",
    "in the middle of the night",
    "what are you going to do",
    "some kind of wonderful thing",
];

fn generator() -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus parses")
}

/// Candidates of every span of `target`, keyed by span edge.
fn spans_of(g: &Generator, target: &str) -> BTreeMap<(usize, usize), Vec<SpanCandidate>> {
    let mut out: BTreeMap<(usize, usize), Vec<SpanCandidate>> = BTreeMap::new();
    for c in g.span_candidates(target, PER_WORD_BUDGET) {
        out.entry((c.start, c.end)).or_default().push(c);
    }
    out
}

/// The band's upper edges, re-derived from the span's own candidate costs:
/// the three interior cuts of four equal-width bands over the cost range
/// that span actually offers.  They are deliberately not read from the
/// selection rule's own cost axis (which is anchored on the per-word
/// budget), so a future change to that axis cannot make this assertion
/// vacuous or hide a regression behind a constant it also moved.
fn band_cuts(costs: &[f64]) -> Vec<f64> {
    let mut sorted = costs.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let (lo, hi) = (sorted[0], sorted[sorted.len() - 1]);
    (1..4).map(|q| lo + (hi - lo) * (q as f64) / 4.0).collect()
}

/// The band of `cost` under `cuts`.
fn band_of(cost: f64, cuts: &[f64]) -> usize {
    cuts.iter().filter(|&&c| cost >= c).count()
}

/// A shortlist is only a portfolio if it spans the span's own cost range:
/// every non-empty band of a span must contribute at least one retained
/// word.  A band can be empty in the shortlist when the fill spends all of
/// its slots in one score whose ordering is monotone in phonetic distance,
/// and then every wording in that band is unreachable at every width.
#[test]
fn every_non_empty_cost_band_contributes_to_the_span_shortlist() {
    let g = generator();
    let mut bands_checked = 0usize;
    let mut spans_checked = 0usize;
    for target in TARGETS {
        let kept: BTreeMap<(usize, usize), HashSet<String>> = g
            .span_shortlists(target, PER_WORD_BUDGET)
            .into_iter()
            .fold(BTreeMap::new(), |mut acc, c| {
                acc.entry((c.start, c.end))
                    .or_default()
                    .insert(c.word);
                acc
            });
        for (edge, candidates) in spans_of(&g, target) {
            if candidates.len() < 2 {
                continue;
            }
            let costs: Vec<f64> = candidates.iter().map(|c| c.cost).collect();
            let cuts = band_cuts(&costs);
            let retained = kept.get(&edge).expect("every offered span is kept");
            spans_checked += 1;
            for band in 0..4 {
                let population = candidates
                    .iter()
                    .filter(|c| band_of(c.cost, &cuts) == band)
                    .count();
                if population == 0 {
                    continue;
                }
                let retained_here = candidates
                    .iter()
                    .filter(|c| band_of(c.cost, &cuts) == band)
                    .filter(|c| retained.contains(&c.word))
                    .count();
                bands_checked += 1;
                assert!(
                    retained_here > 0,
                    "target {target:?} span {edge:?}: cost band {band} of 4 has \
                     {population} candidates and the shortlist retains none of \
                     them, so no wording in that band is reachable at any width"
                );
            }
        }
    }
    assert!(
        bands_checked > 3_000 && spans_checked > 800,
        "the survey is too small to mean anything: {spans_checked} spans, \
         {bands_checked} non-empty bands"
    );
}

/// The same property stated acoustically rather than by band: within a
/// factor of the span's own cheapest candidate — the threshold the
/// acceptance reachability test already uses — a retained word must
/// exist in every non-empty band, not only in the cheapest one.  This is
/// the statement the undifferentiated fill breaks worst, because the tail
/// it appends is the far end of the cost range.
#[test]
fn every_non_empty_cost_band_keeps_a_word_near_the_span_cheapest() {
    let g = generator();
    let mut checked = 0usize;
    for target in TARGETS {
        let kept: BTreeMap<(usize, usize), HashSet<String>> = g
            .span_shortlists(target, PER_WORD_BUDGET)
            .into_iter()
            .fold(BTreeMap::new(), |mut acc, c| {
                acc.entry((c.start, c.end))
                    .or_default()
                    .insert(c.word);
                acc
            });
        for (edge, candidates) in spans_of(&g, target) {
            if candidates.len() < 2 {
                continue;
            }
            let costs: Vec<f64> = candidates.iter().map(|c| c.cost).collect();
            let cuts = band_cuts(&costs);
            let cheapest = costs
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            let competitive = cheapest * 1.5;
            let retained = kept.get(&edge).expect("every offered span is kept");
            for band in 0..4 {
                let population = candidates
                    .iter()
                    .filter(|c| band_of(c.cost, &cuts) == band)
                    .filter(|c| c.cost <= competitive + 1e-9)
                    .count();
                if population == 0 {
                    continue;
                }
                let retained_here = candidates
                    .iter()
                    .filter(|c| band_of(c.cost, &cuts) == band)
                    .filter(|c| c.cost <= competitive + 1e-9)
                    .filter(|c| retained.contains(&c.word))
                    .count();
                checked += 1;
                assert!(
                    retained_here > 0,
                    "target {target:?} span {edge:?}: band {band} of 4 has \
                     {population} candidates within 1.5x of the span's cheapest \
                     ({cheapest:.3}) and the shortlist retains none of them"
                );
            }
        }
    }
    assert!(
        checked > 500,
        "the survey is too small to mean anything: {checked} populated \
         competitive bands"
    );
}

/// What the shortlist bounds must not move: a span retains at most the
/// same number of alternatives as before, so the per-span branching the
/// structural DP relaxes over is unchanged.  A stratified fill that grew
/// the list would be a different change with a different bound, so the
/// width is pinned here rather than argued.
#[test]
fn the_shortlist_width_is_unchanged() {
    let g = generator();
    for target in TARGETS {
        let mut widest = 0usize;
        let mut counts: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for c in g.span_shortlists(target, PER_WORD_BUDGET) {
            *counts.entry((c.start, c.end)).or_default() += 1;
        }
        for (edge, n) in &counts {
            widest = widest.max(*n);
            assert!(
                *n <= 160,
                "span {edge:?} of {target:?} retains {n} alternatives, over \
                 the per-span bound of 160"
            );
        }
        assert!(
            widest > 100,
            "no span of {target:?} fills the list, so the fill is untested: \
             widest is {widest}"
        );
        // Every retained word is a candidate the span actually offered.
        let shortlist = g.span_shortlists(target, PER_WORD_BUDGET);
        for (edge, candidates) in spans_of(&g, target) {
            let offered: HashSet<&str> =
                candidates.iter().map(|c| c.word.as_str()).collect();
            for c in shortlist.iter().filter(|c| (c.start, c.end) == edge) {
                assert!(
                    offered.contains(c.word.as_str()),
                    "span {edge:?} of {target:?} retained {:?}, which the span \
                     never offered",
                    c.word
                );
            }
        }
    }
}
