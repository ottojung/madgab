//! **An objective change has two separable halves, and this file keeps them
//! apart.** It is the instrument a fence re-derivation needs, and the reason
//! one is possible at all.
//!
//! # What is asserted
//!
//! For a spread of ordinary targets at two list lengths, through the public
//! library API at default settings:
//!
//! * `generate_pool` is returned in **non-increasing score order**;
//! * every clue `generate` prints is a member of `generate_pool`, **with the
//!   same score to the bit** — nothing is recomputed, rescaled or adjusted on
//!   the way to the screen;
//! * the printed phrases are pairwise distinct;
//! * the printed list is exactly `min(top_n, pool)` long.
//!
//! Together those say: **the display is a pure function of the pool and of
//! the candidates' scores.** So when a weight vector changes what the
//! generator does, those changes are attributable — a fence about *which
//! candidates exist* is a reach fence, decided from the pool alone, and a
//! fence about *which of them are shown* is a ranking fence, decided from
//! the same pool. Conflating the two is what makes an objective change
//! unattributable, and it is the failure mode
//! `docs/work/OBSTRUCTION-MAP.md` §2 row 4 records twice (`hid` lost from a
//! parked fill, and from an objective vector, with the two very different).
//!
//! # Why this file has to exist, and what it deliberately does not assert
//!
//! Every objective weight in this search is read by the search as well as by
//! the final scorer — twice, in fact. The measurement behind that statement
//! is `docs/work/REPORT-e086cc.md` §3, and it is worth restating the shape of
//! it here, because the obvious form of the claim is false and only the
//! measured form is useful:
//!
//! * **A weight in the final scorer alone is *not* search-neutral.** The
//!   search's discard threshold is the *final-scorer* value of the worst
//!   clue the ordinary beam already committed to, and a `span_score_bound`
//!   that cannot beat it returns `NEG_INFINITY`, which drops the span path.
//!   So
//!   re-weighting the scorer moves the threshold, which moves what the
//!   search is allowed to keep. Measured over five scorer-only perturbations
//!   on seven ordinary targets: **0 of 5 left the pool invariant**, with the
//!   distinct-word count moving 5,185 -> 5,148 .. 5,190 and per-target pool
//!   sizes moving by up to 426 candidates.
//! * The second site, besides the walk's `bound(prefix)` heap key, is that
//!   threshold. It is the reason the objective is an **input to the search**
//!   and not a post-processing step over it, and it is why every weight in
//!   the objective has to be re-derived against the reach fences rather than
//!   assumed to be free.
//!
//! The *stability* half of that — that two searches at the same objective
//! return the same pool — is `tests/approx_determinism.rs`, and is not
//! restated here.
//!
//! What this file does **not** assert, because it is not a property of the
//! tree and asserting it would be asserting a wish: that the pool is
//! invariant under re-weighting. It is not, and no future objective change
//! may assume it is. `approximate_finds_classic_madgab_resegmentation` is red
//! at base and must not be turned green by accident; nothing here touches
//! either canonical case.
//!
//! No phrase, clue, word list or expected output appears below. The targets
//! are ordinary English and none of them is an expected answer; `src/` knows
//! none of them, and `tests/no_phrase_hard_coding.rs` enforces that.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
use std::collections::HashSet;

/// The shipped CLI's default search, at a given list length.
fn approximate(top_n: usize) -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n,
            ..GeneratorConfig::default()
        },
    )
    .expect("the corpus loads")
}

#[test]
fn the_display_is_a_pure_function_of_the_pool_and_its_scores() {
    for top_n in [10usize, 50] {
        for target in [
            "the cat sat on the mat",
            "a sturdy green cardigan",
            "we should have told her",
            "big spender",
        ] {
            let generator = approximate(top_n);
            let (printed, _) = generator.generate_with_pool(target);
            let pool = generator.generate_pool(target);

            // The pool is returned score-ordered, so "the head" is a prefix
            // of it and nothing downstream has to re-derive the order.
            assert!(
                pool.windows(2).all(|w| w[0].score >= w[1].score),
                "{target:?} at top_n {top_n}: the returned pool is not in \
                 non-increasing score order, so its prefix is not its head"
            );

            let scores_match = printed.iter().all(|c| {
                pool.iter()
                    .any(|p| p.phrase == c.phrase && p.score.to_bits() == c.score.to_bits())
            });
            assert!(
                scores_match,
                "{target:?} at top_n {top_n}: a printed clue is not in the \
                 pool at its own score, so the display is not a function of \
                 the pool and an objective change cannot be attributed to \
                 reach or to ranking"
            );

            let shown: HashSet<&str> = printed.iter().map(|c| c.phrase.as_str()).collect();
            assert_eq!(
                shown.len(),
                printed.len(),
                "{target:?} at top_n {top_n}: the printed list repeats a phrase"
            );
            let in_pool: HashSet<&str> = pool.iter().map(|c| c.phrase.as_str()).collect();
            assert!(
                shown.is_subset(&in_pool),
                "{target:?} at top_n {top_n}: the printed list contains a \
                 phrase the pool does not"
            );

            assert_eq!(
                printed.len(),
                top_n.min(pool.len()),
                "{target:?} at top_n {top_n}: the printed list is {} long \
                 against a pool of {}, so the display is not a prefix of the \
                 pool",
                printed.len(),
                pool.len()
            );
        }
    }
}
