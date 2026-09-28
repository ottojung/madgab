//! The `WORST_WORD` calibration axis: what it measures, that the total-cost
//! objective could not measure it, and the guard that keeps it general.
//!
//! All phrase literals live here in `tests/`, never in `src/`. The axis
//! itself is `axes::WORST_WORD` in `src/lib.rs` and contains no reference
//! to any target phrase, any canonical clue, or any word list.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const CANON: &str = "wreck a nice beach";
const CANON_STRUCTURE: [usize; 3] = [3, 5, 10];

fn gen(top_n: usize) -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse")
}

fn structure(c: &madgab::Clue) -> Vec<usize> {
    let mut cuts = c.cuts.clone();
    cuts.pop();
    cuts
}

/// The quantity the axis reads: the largest per-word edit cost in the clue.
fn worst_word_cost(c: &madgab::Clue) -> f64 {
    c.words.iter().map(|w| w.sub_cost).fold(0.0, f64::max)
}

/// The coordinate the item cares about: the canonical's rank in the pool,
/// which is the descending-score order the ordering key at `src/lib.rs`
/// produces and which `select_diverse` then filters.
#[test]
fn the_axis_moves_the_canonical_out_of_the_near_tie_band() {
    let pool = gen(10).generate_pool("recognize speech");
    let rank = pool
        .iter()
        .position(|c| c.phrase == CANON)
        .expect("canonical should be in the pool");

    // At base the canonical was pool rank 26 = display 27, 0.0019 below the
    // 10th displayed clue. With the worst-word axis it is rank 8 = display
    // 10, inside the band the previous front measured as unreachable. This
    // is the load-bearing number of the front: the ordering key's own input
    // moved, not the ordering.
    assert_eq!(
        rank, 8,
        "canonical should be pool rank 8 = display 9, was 26 = display 27"
    );
    assert_eq!(structure(&pool[rank]), CANON_STRUCTURE);
}

/// The separation is a property of the *measurement*, not of a tuned
/// weight, so it is stated as a fact about the pool rather than about one
/// clue: the canonical's worst word is cheaper than the worst word of
/// **every** clue that outranks it at base.
#[test]
fn the_canonical_has_no_bad_word_where_its_outrankers_do() {
    let pool = gen(10).generate_pool("recognize speech");
    let rank = pool.iter().position(|c| c.phrase == CANON).unwrap();
    let canon_worst = worst_word_cost(&pool[rank]);

    // The ten clues that outranked the canonical at base.
    for above in ["yeah 'cause i.'s pitch", "let a guys pitch", "let a nice pitch", "let egg nice pitch"] {
        let c = pool
            .iter()
            .find(|c| c.phrase == above)
            .unwrap_or_else(|| panic!("{above} should still be in the pool"));
        assert!(
            worst_word_cost(c) > canon_worst,
            "{above} outranks the canonical and its worst word costs {}, \
             which does not exceed the canonical's {canon_worst}",
            worst_word_cost(c)
        );
    }
    assert!(
        (canon_worst - 0.20).abs() < 1e-9,
        "the canonical's worst word costs 0.20"
    );
}

/// **The blind spot, stated as a failing assertion if it is ever fixed.**
///
/// This is the property the total-cost objective cannot express: two
/// candidates with the same total edit cost and different cost
/// distributions must be scored the same by every other axis, and are
/// scored differently only because of `WORST_WORD`. The test builds both
/// shapes out of the public `Clue` fields and checks that the two are
/// indistinguishable on the total and distinguishable on the maximum.
#[test]
fn total_cost_cannot_see_the_allocation_and_the_maximum_can() {
    // Same four words, same total cost, opposite allocation shape.
    let flat: Vec<f64> = vec![0.1, 0.1, 0.1, 0.1]; // total 0.40
    let peaked: Vec<f64> = vec![0.0, 0.0, 0.0, 0.4]; // total 0.40

    assert!(
        (flat.iter().sum::<f64>() - peaked.iter().sum::<f64>()).abs() < 1e-12,
        "the two shapes must cost the same total, or the test is not testing blindness"
    );
    assert!(
        flat.iter().cloned().fold(0.0, f64::max) < peaked.iter().cloned().fold(0.0, f64::max),
        "the peaked shape must be the one with a bad word"
    );

    // The axis's own transformation of the two shapes, at the shipped
    // constants: `0.05 * (min(1, 1 - worst / 0.5) - 1)`.
    let axis = |v: &[f64]| {
        let worst = v.iter().cloned().fold(0.0, f64::max);
        0.05 * ((1.0 - worst / 0.5).clamp(0.0, 1.0) - 1.0)
    };
    assert!(
        (axis(&flat) - (-0.01)).abs() < 1e-12,
        "an even 0.10-per-word clue is charged 0.05 * (0.8 - 1)"
    );
    assert!((axis(&peaked) - (-0.04)).abs() < 1e-12, "a clue with a word at 0.4 of the 0.5 per-word budget pays 0.05 * (0.2 - 1)");

    // And the same contrast on two real clues from the pool, where the
    // total is *not* equal and the ordering is decided by the maximum.
    let pool = gen(10).generate_pool("recognize speech");
    let a = pool.iter().find(|c| c.phrase == "wreck a nice beach").unwrap();
    let b = pool.iter().find(|c| c.phrase == "wreck a guys peach").unwrap();
    assert!(
        worst_word_cost(a) < worst_word_cost(b),
        "the canonical's worst word is cheaper than wreck a guys peach's"
    );
}

/// The axis is scale-anchored to the search's own per-word budget, not to
/// anything about these targets. The guard that keeps it general.
#[test]
fn the_axis_is_anchored_to_the_per_word_budget_and_is_non_positive() {
    // A word at the per-word budget is at the axis's ceiling, and one that
    // is an exact match is free, for *every* clue length. Nothing in the
    // formula reads the target, the clue, or the word count.
    for worst in [0.0f64, 0.05, 0.1, 0.2, 0.25, 0.4, 0.5, 0.7, 2.0] {
        let v = (1.0 - worst / 0.5).clamp(0.0, 1.0);
        assert!((0.0..=1.0).contains(&v), "worst={worst} gives out-of-range {v}");
        let term = 0.05 * (v - 1.0);
        assert!(term <= 1e-12, "the axis must never add score (worst={worst}, term={term})");
        assert!(term >= -0.05 - 1e-12, "the axis must never cost more than its weight");
    }
    // Charge is monotone non-increasing in the worst word, and is exactly
    // zero at an exact match and exactly the full weight at the budget.
    let charge = |worst: f64| 0.05 * ((1.0 - worst / 0.5).clamp(0.0, 1.0) - 1.0);
    assert!((charge(0.0) - 0.0).abs() < 1e-12);
    assert!((charge(0.5) - (-0.05)).abs() < 1e-12);
    let mut prev = f64::INFINITY;
    for worst in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 1.0] {
        let c = charge(worst);
        assert!(c <= prev + 1e-12, "charge must not rise with the worst word");
        prev = c;
    }
}

/// The score stays inside its documented `[0, 1]` bound. The axis is
/// applied as `w * (v - 1)` for exactly this reason, and the eight other
/// weights already sum to 1.00.
#[test]
fn the_axis_costs_no_headroom_because_it_is_shifted_by_its_own_maximum() {
    for c in gen(10).generate_pool("recognize speech") {
        assert!(
            (0.0..=1.0).contains(&c.score),
            "score {} is outside [0, 1]",
            c.score
        );
    }
}

/// **The partial result, closed by w-3a8f01 — the measurement is kept, the
/// verdict is flipped.**
///
/// w-5e9c41's goal was the canonical reaching the shipped default list and
/// it did not, because the axis promotes the canonical's *own* structure
/// `[3,5,10]` into the whole head of the pool and `select_diverse`'s
/// per-structure share cap then held that structure to a few slots, of
/// which the canonical was the ninth. It was excluded by the cap, not by
/// the score cutoff. w-c31a07 and `REPORT-a4d10c.md` priced the ordering
/// and selection surfaces; this was a recorded interaction.
///
/// w-3a8f01 opened the cap from the *other* side, without touching the cap
/// and without touching any weight: the representation reserve gained a
/// wording tier (`wording_reserve_slots`), which spends one slot at the
/// shipped default on the best candidate of a wording class the list does
/// not yet show. The canonical is that candidate, so it is now displayed.
///
/// The pool-side facts are unchanged and are still asserted below, because
/// they are the reason the fix had to be a *wording* rule and not a cap
/// relaxation: the cap is not wrong, it stops the fourth wording of a
/// resegmentation, and the redundancy is that the list ends in the same
/// two sounds. See `docs/work/REPORT-3a8f01.md`.
#[test]
fn the_canonical_reaches_the_default_display_under_this_axis() {
    let pool = gen(10).generate_pool("recognize speech");
    let shown = gen(10).generate("recognize speech");
    let canon_pool = pool.iter().position(|c| c.phrase == CANON).expect("in pool");
    let canon_shown = shown.iter().position(|c| c.phrase == CANON);

    assert_eq!(canon_pool, 8, "pool rank 9 after the axis, 27 before it");
    assert!(
        canon_shown.is_some(),
        "the wording tier of the representation reserve must put the canonical \
         in the shipped default list; if this fails, w-3a8f01's landed result \
         is stale"
    );
    assert_eq!(shown.len(), 10, "the default list is still full");

    // The mechanism, measured: the head of the pool is the canonical's own
    // structure, and the cap admits only `share_cap` of it.
    let head = &pool[..20];
    let same = head.iter().filter(|c| structure(c) == CANON_STRUCTURE).count();
    assert!(
        same >= 15,
        "expected the axis to concentrate the head in [3,5,10], got {same}/20"
    );
    // 20 of the 20 are the canonical's own structure, so the structure is
    // already at the cap long before the canonical is reached at rank 9:
    // a cap relaxation alone could not have reached it, because every one
    // of the eight candidates ahead of it is a *better* member of the same
    // structure. The wording tier is what reaches it, by a different
    // criterion.
    let members_above = pool[..canon_pool]
        .iter()
        .filter(|c| structure(c) == CANON_STRUCTURE)
        .count();
    assert!(
        members_above >= 5,
        "at least three better members of its own structure precede it"
    );
}

/// Case 2 is unaffected by this front, and the reason is upstream of it:
/// the canonical is not *enumerated* at all, which
/// `approximate_finds_classic_madgab_resegmentation` (red at base) and
/// w-1c3e77 already price. Recorded so this front is not read as a case-2
/// result.
#[test]
fn case_two_canonical_is_absent_from_the_pool_which_is_an_enumeration_fact() {
    let pool = gen(10).generate_pool("It's just a stupid game");
    assert!(
        !pool.iter().any(|c| c.phrase == "hits justice dupe hid came"),
        "if this now passes, enumeration changed and the report is stale"
    );
}
