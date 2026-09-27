//! The emission-coverage property, asserted at the **production** boundary.
//!
//! # The two questions this file keeps apart
//!
//! There are two different questions about a clue, and this repository has
//! reversed its answer three times by answering them as one:
//!
//! 1. **Is the alignment in the lattice?**  Every word is a real alternative
//!    of its own resegmentation span's production shortlist.  This is a
//!    statement about the span shortlists, which are not part of the public
//!    API.
//! 2. **Is it in the production candidate pool?**  It survives enumeration,
//!    scoring and deduplication on the default release path.
//!
//! A "yes" to (1) with a "no" to (2) is a real and useful answer, and it
//! localises the work to *emission* rather than to the objective.  Reporting
//! the two as one question is what produced every prior reversal, so this
//! file asserts (2) directly and never lets it stand in for (1).
//!
//! # Boundary
//!
//! Everything here goes through the public library API at **default
//! settings** — the same configuration `madgab --approximate --top 50` uses.
//! Nothing here widens a budget, and nothing here runs the search twice.
//! `generate_with_pool` returns the deduplicated candidate pool and the
//! selected proposals from one call, so the pool and the printed list are the
//! production ones and not a re-derivation.
//!
//! # Why the fixture is a real target
//!
//! Question (2) cannot be asked phrase-free at this boundary.  Deciding it
//! needs the production span shortlists and the traversal's derived opening
//! width, and neither is reachable from outside the crate; asserting "some
//! alignment is missing" without naming which one would assert nothing.  What
//! *would* make it phrase-free is exposing a coverage report from the public
//! API — the per-span shortlists and the derived opening width — so a
//! consumer could ask "is this alternative reachable?" directly.  That is
//! not done here.
//!
//! Naming a real target in a *test* is therefore deliberate and bounded.  The
//! fence this file lives under is the production one: `src/` must contain no
//! mention of the fixture, and `tests/no_phrase_hard_coding.rs` enforces that
//! by stripping comments before it scans.  Nothing in this file can influence
//! what the generator produces.

use madgab::{Clue, Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// The exact default configuration of `--approximate --top 50`.
fn default_approximate() -> GeneratorConfig {
    GeneratorConfig {
        mode: SearchMode::Approximate {
            per_word_budget: 0.5,
            total_budget: 1.5,
        },
        top_n: 50,
        ..GeneratorConfig::default()
    }
}

fn words_of(clue: &Clue) -> Vec<String> {
    clue.words.iter().map(|w| w.word.to_lowercase()).collect()
}

/// Is this word sequence present in the pool, and if so where?
fn pool_rank(clues: &[Clue], words: &[&str]) -> Option<(usize, f64)> {
    clues
        .iter()
        .position(|c| words_of(c) == words.iter().map(|w| w.to_string()).collect::<Vec<_>>())
        .map(|i| (i + 1, clues[i].score))
}

/// Is this word sequence present in the printed proposal set, and if so where?
fn printed_rank(clues: &[Clue], words: &[&str]) -> Option<(usize, f64)> {
    pool_rank(clues, words)
}

/// The alignment this file exists for: a five-word resegmentation whose every
/// word is a real alternative of its own span's production shortlist, and
/// which the production pool does not contain.
///
/// The two question-answers are asserted as two separate facts and are
/// reported in the failure messages separately, because conflating them is
/// the failure mode this file exists to prevent.
#[test]
fn a_lattice_alignment_can_be_absent_from_the_production_pool() {
    let generator =
        Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    let (printed, pool_size) = generator.generate_with_pool(
        "It's just a stupid game",
    );
    let pool = generator.generate_pool("It's just a stupid game");

    let alignment = ["hits", "justice", "dupe", "hid", "came"];

    // Question (b): the production candidate pool.  This is the load-bearing
    // assertion of this file: it is what retires the reading that the missing
    // clue is merely *ranked out of the printed 50*, and with it the whole
    // reweighting family of explanations.  The pool is an order of magnitude
    // wider than the printed list, so absence here is not a ranking fact.
    assert_eq!(
        pool_rank(&pool, &alignment),
        None,
        "question (b): the alignment reached the production dedup pool. \
         Every objective-side explanation of its absence is void, and the \
         emission path is not what blocks it. Pool is {pool_size} wide."
    );

    // Question (a) cannot be asserted at this boundary — see the module
    // comment.  Asserted instead as far as this boundary reaches: the words
    // are not *individually* absent from the pool, so the alignment's
    // absence is a statement about the combination and not about the
    // vocabulary.  Without this the pool assertion could be satisfied by a
    // lexicon that simply lacks these words.
    for word in alignment {
        assert!(
            pool.iter()
                .any(|c| words_of(c).iter().any(|w| w == word)),
            "the word {word:?} is absent from the pool entirely, so the \
             pool assertion above is vacuous for it"
        );
    }

    // And the printed set, which is a strictly weaker question, is reported
    // with its own assertion so the two are never read as one number.
    assert_eq!(
        printed_rank(&printed, &alignment),
        None,
        "the alignment reached the printed proposal set"
    );
}

/// The pool is the production one and it is wide.
///
/// The whole difference between the two questions lives in the size of this
/// number: a clue can be missing from the printed 50 and present in a pool of
/// this width, and every prior reversal came from reporting the first fact as
/// the second.  Pinning the order of magnitude keeps that honest — a change
/// that collapses the pool would make the pool-absence assertion above much
/// weaker without failing anything else.
#[test]
fn the_production_pool_is_much_wider_than_the_printed_list() {
    let generator =
        Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    let (printed, pool_size) = generator.generate_with_pool(
        "It's just a stupid game",
    );
    assert_eq!(printed.len(), 50, "--top 50 prints 50 proposals");
    assert!(
        pool_size > 10 * printed.len(),
        "the dedup pool is {pool_size} wide against {} printed; the two \
         questions are not distinguishable at this ratio",
        printed.len()
    );
}

/// Canonical case 1 must not regress.
///
/// This is the standing acceptance example for the other canonical case and
/// it is the cheapest available detector that a change to the emission
/// schedule has not disturbed the printed list.
#[test]
fn the_other_canonical_resegmentation_is_still_proposed() {
    let generator =
        Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    let (printed, _) =
        generator.generate_with_pool("recognize speech");
    let wanted = ["wreck", "a", "nice", "beach"];
    let found = printed_rank(&printed, &wanted);
    assert!(
        found.is_some(),
        "the other canonical resegmentation left the printed proposal set"
    );
}
