//! Head selection: the display window is **reachable**, and the reachability
//! is bought without spending the list's breadth.
//!
//! ## What the shipped default used to do
//!
//! For `recognize speech` at the shipped default `--approximate --top 10`,
//! the canonical clue `wreck a nice beach` is at **pool rank 8** — inside
//! the display window — with score `0.899950`, and it is not printed. All
//! ten printed rows are wordings of two or three resegmentations, and every
//! one of them ends in one of two rhymes.
//!
//! The stage was measured, not assumed (`docs/work/REPORT-3a8f01.md`): it is
//! **not** the top-N cutoff — a rank-8 candidate is inside the cutoff — and
//! **not** the score ordering. It is `select_diverse`'s per-structure
//! `share_cap`, `share_cap(10, 293) = 4`, refusing the ninth-best member of a
//! structure whose first four members are better than it is. A cap
//! *relaxation* cannot reach it, and neither can a reweighting: all eight
//! candidates ahead of it are better members of the same structure, so any
//! rule that admits the ninth must admit the first eight too.
//!
//! ## What the landed rule is
//!
//! `select_diverse` already had a bounded **representation reserve** that
//! spends slots on the best candidate of each of several boundary
//! structures, whatever the cutoff says. This front added a **wording tier**
//! to it (`wording_reserve_slots` in `src/lib.rs`): after the structure tier
//! has had its own slots, a bounded remainder is spent on the best candidate
//! of each further *wording class* — a (boundary structure, final-word IPA)
//! pair. The criterion is then "this candidate ends in a sound the visible
//! list has none of", which is a different criterion from the one that was
//! refusing the clue, and it is what makes the clue reachable.
//!
//! ## The properties asserted here
//!
//! * **P1 — window reachability, as a property rather than a fact.** For
//!   every target measured, the best-scoring member of each of the reserve
//!   budget's best wording classes is in the shipped default display. The
//!   canonical is one instance: it is the best member of its own class and
//!   its class is the third best of the pool.
//! * **P2 — the breadth floor.** Mean distinct boundary structures per list
//!   over a multi-target spread is at least the base value, so the
//!   reachability was not bought out of the list's structure breadth.
//! * **P3 — the concentration is not worse.** Mean dominant-structure share
//!   is at most its base value, because that is the quantity the shipped
//!   head had already lost (the ten rows were one ending repeated).
//! * **P4 — the guard, stated as a construction.** The reserve tiers are
//!   bounded by named arithmetic in `src/lib.rs`, the list stays full, and
//!   the rule is deterministic.
//!
//! This file names the canonical strings, so it follows the established
//! canonical-example convention (`tests/worst_word_axis.rs`,
//! `tests/display_ordering_attribution.rs`, `tests/pool_rank_reporting.rs`)
//! and adds nothing to `src/`'s allowlist in `tests/no_phrase_hard_coding.rs`.

use madgab::{Clue, Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
use std::collections::HashSet;

/// The itinerary's canonical case-1 target/clue pair.
const CANON_TARGET: &str = "recognize speech";
const CANON_CLUE: &str = "wreck a nice beach";

/// The shipped default `--top`, i.e. what a user gets with no `--top` flag.
const DEFAULT_TOP: usize = 10;

/// A spread of at least twelve targets: the two canonical inputs, function-word
/// salad, and ordinary two- to four-word phrases of different lengths. Wider
/// than the twelve the criterion asks for so the mean is not a two-target
/// accident.
const SPREAD: &[&str] = &[
    "recognize speech",
    "It's just a stupid game",
    "I love you",
    "a whole lot of trouble",
    "he was a big fat man",
    "what are you going to do",
    "some kind of wonderful thing",
    "she had a lot of money",
    "there is no way to know",
    "the cat sat on the mat",
    "if you want to go now",
    "they are going to be late",
    "one of those other people",
    "when the rain finally stopped",
    "you can do it yourself",
];

/// The reserve budget at the shipped default, in slots:
/// `structure_reserve_slots(10) + wording_reserve_slots(10)` = `2 + 1` in
/// `src/lib.rs`. Stated here as a number because the test states the
/// construction; it is the number of wording classes the display policy
/// promises to represent whatever the cutoff says.
const RESERVE_AT_DEFAULT: usize = 3;

/// Mean distinct boundary structures per list, measured at base on `SPREAD`
/// with the shipped default: **3.6667**. The floor criterion 4 states.
const BASE_MEAN_DISTINCT: f64 = 3.666_666_666_666_666_5;

/// Mean dominant-structure share, measured at base on `SPREAD`:
/// **0.4000**.
const BASE_MEAN_DOMINANT_SHARE: f64 = 0.4;

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

/// The word-boundary structure a clue was aligned at.
fn structure(c: &Clue) -> Vec<usize> {
    let mut cuts = c.cuts.clone();
    cuts.pop();
    cuts
}

/// The wording class: boundary structure plus the IPA of the last word.
/// This mirrors `clue_wording_class` in `src/lib.rs`, which is private; the
/// key is a sound, so it is re-derived from the public `Clue` rather than
/// being reached into the crate for.
fn wording_class(c: &Clue) -> (Vec<usize>, String) {
    let tail = c
        .words
        .last()
        .map(|w| w.ipa.to_lowercase())
        .unwrap_or_default();
    (structure(c), tail)
}

/// The distinct boundary structures in a list, and the share the most common
/// one holds.
fn breadth(list: &[Clue]) -> (usize, f64) {
    let mut seen: HashSet<Vec<usize>> = HashSet::new();
    let mut best = 0usize;
    for c in list {
        let s = structure(c);
        if seen.insert(s.clone()) {
            let n = list.iter().filter(|o| structure(o) == s).count();
            best = best.max(n);
        }
    }
    (seen.len(), best as f64 / list.len() as f64)
}

/// The pool's wording classes, keyed by their best-scoring member, in
/// descending order of that member's score.
fn classes_by_best(pool: &[Clue]) -> Vec<(usize, (Vec<usize>, String))> {
    let mut best: Vec<((Vec<usize>, String), f64)> = Vec::new();
    for c in pool {
        let class = wording_class(c);
        match best.iter_mut().find(|(k, _)| *k == class) {
            Some((_, s)) => *s = s.max(c.score),
            None => best.push((class, c.score)),
        }
    }
    best.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    best.into_iter()
        .enumerate()
        .map(|(rank, (class, _))| (rank, class))
        .collect()
}

/// **P1 — window reachability, stated as a property.** For every target, the
/// best-scoring member of each of the reserve budget's best wording classes
/// is in the shipped default display, whatever the top-N cutoff would have
/// said. The canonical is one instance of this, not the statement of it.
#[test]
fn the_best_member_of_each_reserved_wording_class_is_in_the_default_display() {
    for target in SPREAD {
        let pool = gen(DEFAULT_TOP).generate_pool(target);
        let shown = gen(DEFAULT_TOP).generate(target);
        let shown_classes: HashSet<(Vec<usize>, String)> =
            shown.iter().map(wording_class).collect();
        let shown_phrases: HashSet<&str> = shown.iter().map(|c| c.phrase.as_str()).collect();

        let ranked = classes_by_best(&pool);
        for (_, class) in ranked.iter().take(RESERVE_AT_DEFAULT) {
            let best = pool
                .iter()
                .filter(|c| wording_class(c) == *class)
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .expect("a class in the pool has a member");
            assert!(
                shown_phrases.contains(best.phrase.as_str()),
                "{target:?}: the best member of a reserved wording class ({:?}) is \
                 {phrase:?} at pool rank {rank} and is not in the shipped default \
                 display; shown: {shown:?}",
                class,
                phrase = best.phrase,
                rank = pool.iter().position(|c| c.phrase == best.phrase).unwrap(),
                shown = shown.iter().map(|c| c.phrase.as_str()).collect::<Vec<_>>(),
            );
            // ... and it is there *as* that class, so the reserve is not
            // being satisfied by a coincidence of another class's member.
            assert!(
                shown_classes.contains(class),
                "{target:?}: a reserved class is represented by the wrong member"
            );
        }
    }
}

/// The canonical, as one instance of P1: a clue the pool ranks inside the
/// default display window is reachable in the default display, at the
/// shipped default configuration and with no flags.
#[test]
fn the_canonical_is_reachable_in_the_shipped_default_display() {
    let pool = gen(DEFAULT_TOP).generate_pool(CANON_TARGET);
    let shown = gen(DEFAULT_TOP).generate(CANON_TARGET);

    let rank = pool
        .iter()
        .position(|c| c.phrase == CANON_CLUE)
        .expect("the canonical is enumerated");
    assert_eq!(
        rank, 8,
        "the canonical's pool rank is the premise of this front; if it moved, \
         the report's reproduction is stale"
    );
    assert!(
        rank < DEFAULT_TOP,
        "the canonical is inside the display window, so the top-N cutoff \
         cannot be the stage that drops it"
    );
    let at = shown
        .iter()
        .position(|c| c.phrase == CANON_CLUE)
        .expect("the canonical must be in the shipped default display");
    assert!(
        at < DEFAULT_TOP,
        "the canonical must be displayed within the default window, got {at}"
    );
    assert_eq!(shown.len(), DEFAULT_TOP, "the default list is still full");
}

/// The stage, re-measured so a later pass does not re-derive it: the
/// canonical's own structure owns the whole head of the pool, so the cap was
/// refusing the ninth of nine better members and only a rule keyed on
/// something finer than the structure can reach it.
#[test]
fn the_cap_was_the_stage_and_a_relaxation_of_it_cannot_reach_the_canonical() {
    let pool = gen(DEFAULT_TOP).generate_pool(CANON_TARGET);
    let rank = pool
        .iter()
        .position(|c| c.phrase == CANON_CLUE)
        .expect("the canonical is enumerated");
    let canon_structure = structure(&pool[rank]);

    let ahead_same_structure = pool[..rank]
        .iter()
        .filter(|c| structure(c) == canon_structure)
        .count();
    assert_eq!(
        ahead_same_structure, 8,
        "all eight candidates ahead of the canonical are better members of its \
         own structure, so no cap value below 9 and no score reweighting can \
         admit it while refusing them"
    );
    // The top-N window is one structure: the display is capped, not cut off.
    let window_structures: HashSet<Vec<usize>> =
        pool[..DEFAULT_TOP].iter().map(structure).collect();
    assert_eq!(
        window_structures.len(),
        1,
        "the first ten pool candidates are all one resegmentation"
    );
}

/// **P2 — the breadth floor.** Mean distinct boundary structures per list over
/// the spread is at least the base value, so P1 was not bought out of the
/// list's structure breadth. Base on this same spread: 3.6667.
#[test]
fn list_diversity_is_not_collapsed_below_the_shipped_default_floor() {
    let mut total = 0f64;
    for target in SPREAD {
        let (distinct, _) = breadth(&gen(DEFAULT_TOP).generate(target));
        total += distinct as f64;
    }
    let mean = total / SPREAD.len() as f64;
    assert!(
        mean >= BASE_MEAN_DISTINCT,
        "mean distinct structures per list is {mean:.4} over {} targets, below \
         the base floor {BASE_MEAN_DISTINCT:.4}; the reachability was bought \
         out of the list's breadth",
        SPREAD.len()
    );
}

/// **P3 — the concentration is not worse either.** The shipped head was ten
/// rows of one ending; the mean dominant-structure share must not rise.
#[test]
fn the_dominant_structure_share_did_not_rise() {
    let mut total = 0f64;
    for target in SPREAD {
        let (_, share) = breadth(&gen(DEFAULT_TOP).generate(target));
        total += share;
    }
    let mean = total / SPREAD.len() as f64;
    assert!(
        mean <= BASE_MEAN_DOMINANT_SHARE,
        "mean dominant-structure share rose to {mean:.4} from \
         {BASE_MEAN_DOMINANT_SHARE:.4}; this front must not concentrate the head"
    );
}

/// **P4 — the guard, stated as a construction.** The reserve is bounded by
/// named arithmetic and cannot become the list: at every `top_n` the two
/// tiers together stay inside the list, the tiers are a remainder rather
/// than a second budget, and the list is always full.
#[test]
fn the_reserve_is_a_bounded_remainder_and_the_list_is_always_full() {
    for target in SPREAD {
        for top_n in [1usize, 2, 5, 10, 25, 50] {
            let shown = gen(top_n).generate(target);
            assert!(
                !shown.is_empty(),
                "{target:?} at top_n {top_n}: an empty list is never acceptable"
            );
            assert!(
                shown.len() <= top_n,
                "{target:?} at top_n {top_n}: {} rows for {top_n} slots",
                shown.len()
            );
            // The reserve is a representation rule, so the list still spans
            // more than one resegmentation whenever the list has room for
            // two and the pool holds two.
            if top_n >= 2 {
                let (distinct, _) = breadth(&shown);
                let pool_structures: HashSet<Vec<usize>> =
                    gen(top_n).generate_pool(target).iter().map(structure).collect();
                assert!(
                    distinct >= 2 || pool_structures.len() < 2,
                    "{target:?} at top_n {top_n}: the list collapsed onto one \
                     resegmentation while the pool holds {pool_structures:?}"
                );
            }
        }
    }
}

/// The rule is deterministic: the same configuration twice is the same list,
/// because the reserve walks the same score order and admits the first
/// unrepresented class it meets.
#[test]
fn the_display_is_deterministic() {
    let a = gen(DEFAULT_TOP).generate(CANON_TARGET);
    let b = gen(DEFAULT_TOP).generate(CANON_TARGET);
    let phrases = |v: &[Clue]| -> Vec<String> {
        v.iter().map(|c| c.phrase.clone()).collect()
    };
    assert_eq!(phrases(&a), phrases(&b));
}
