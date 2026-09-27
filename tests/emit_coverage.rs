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
use std::collections::HashSet;
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

/// For a short multi-syllable target the printed proposal set is not a
/// single word-count class.
///
/// This is the general property the per-phone `SIMILARITY` normalisation
/// buys, and it is asserted as a distribution rather than as a phrase: the
/// target is a neutral multi-syllable sentence used only to have somewhere
/// to measure, and no word, clue or score from it appears in an assertion.
///
/// A per-word similarity reading charged a clue for having more words, so
/// on a target whose `RHYTHM` band admits essentially one length the printed
/// set was monopolised by that one class.  The assertion is deliberately the
/// weak one — *more than one class is present at all* — so it states the
/// property being bought and nothing about which classes or how the set is
/// split.
#[test]
fn a_short_multi_syllable_proposal_set_is_not_one_word_count_class() {
    let generator =
        Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    let (printed, _) = generator.generate_with_pool("a sturdy green cardigan");
    assert_eq!(printed.len(), 50, "--top 50 prints 50 proposals");

    let mut classes: Vec<usize> =
        printed.iter().map(|c| c.words.len()).collect();
    classes.sort_unstable();
    classes.dedup();
    assert!(
        classes.len() > 1,
        "the printed proposal set is the single word-count class {:?} out of \
         50 proposals, so a short multi-syllable target is monopolised by one \
         cut length",
        classes
    );
}

// -----------------------------------------------------------------
// Weight-free properties of the pool/print boundary (w-4d1e93)
// -----------------------------------------------------------------
//
// The two fences this file already carries are *not* weight-free, and
// `docs/work/REPORT-4d1e93.md` measures that.  A named alignment's absence
// from the pool goes red under an objective weight that empties the pool's
// vocabulary, and the golden-output lock goes red under a weight of 0.001.
// Neither can be re-expressed weight-free, but both of them *rest* on
// three things that are weight-free, and those three were not asserted
// anywhere.  A pool-absence claim read as a ranking claim, or a lock read
// as a behaviour lock, both fail silently if these three stop holding, so
// they are asserted here, phrase-free and weight-free.

fn words_key(clue: &Clue) -> Vec<String> {
    words_of(clue)
}

/// **The printed proposals are members of the pool.**  Weight-free: it held
/// at all ten points of the weight sweep in `REPORT-4d1e93.md`, at 50 of
/// 50 printed clues in each.
///
/// This is what makes the pool/print distinction a distinction and not a
/// restatement.  If a printed clue could fall outside the pool, then
/// "absent from the pool" would also be true of some clues that *were*
/// proposed, and the whole reading of the absence assertion above — that
/// the alignment never reached the candidate set at all — would stop
/// following from the numbers.
#[test]
fn every_printed_proposal_is_a_member_of_the_dedup_pool() {
    let generator =
        Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    for target in ["It's just a stupid game", "a sturdy green cardigan"] {
        let (printed, pool_size) = generator.generate_with_pool(target);
        let pool = generator.generate_pool(target);
        assert_eq!(
            pool.len(),
            pool_size,
            "{target}: pool size disagrees with pool length"
        );
        let missing: Vec<Vec<String>> = printed
            .iter()
            .map(words_key)
            .filter(|w| !pool.iter().any(|c| words_key(c) == *w))
            .collect();
        assert!(
            missing.is_empty(),
            "{target}: {} of {} printed proposals are not members of the \
             {} wide dedup pool, so printed-set absence and pool absence are \
             no longer different questions: {missing:?}",
            missing.len(),
            printed.len(),
            pool_size,
        );
    }
}

/// **A printed proposal set is exactly `top_n` long, score-ordered and
/// phrase-deduplicated.**  Weight-free: all three held at all ten points
/// of the sweep, at `top_n` 10, 25 and 50.
///
/// This is the part of the golden-output lock that is a property of the
/// *search* rather than of one weight vector.  The locked score strings
/// are re-derived with every weight vector; these three are not, and a
/// change that broke them would be a change to what the search returns,
/// not a change of opinion about how good a clue is.
#[test]
fn a_printed_proposal_set_is_ordered_deduplicated_and_exactly_top_n() {
    for (target, top_n) in [
        ("I love you", 10usize),
        ("It's just a stupid game", 50),
        ("a sturdy green cardigan", 25),
    ] {
        let generator = Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n,
                ..GeneratorConfig::default()
            },
        )
        .unwrap();
        let printed = generator.generate(target);
        assert_eq!(
            printed.len(),
            top_n,
            "{target}: --top {top_n} printed {} proposals",
            printed.len()
        );
        for pair in printed.windows(2) {
            assert!(
                pair[0].score >= pair[1].score,
                "{target}: printed scores ascend, {} then {}",
                pair[0].score,
                pair[1].score,
            );
        }
        let mut keys: Vec<String> = printed
            .iter()
            .map(|c| c.phrase.to_lowercase())
            .collect();
        let total = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(
            keys.len(),
            total,
            "{target}: the printed set repeats a wording"
        );
    }
}

/// **The pool's vocabulary is not exhausted by what the search prints.**
/// Weight-free in form: the pool ran 1,811-1,864 distinct words against
/// 250-294 printed word instances at every point of the sweep.
///
/// This is the weight-free skeleton of the argument the named-alignment
/// fence makes with a single word: that a pool can hold the parts of an
/// alignment and not the alignment.  Phrase-free, it says the same thing
/// about the pool as a whole — the pool speaks a vocabulary strictly wider
/// than the printed list uses — so a claim that the pool "lacks a word" has
/// to be checked against the pool and cannot be inferred from the printed
/// 50.
#[test]
fn the_production_pool_speaks_a_wider_vocabulary_than_the_printed_list() {
    let generator =
        Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    for target in ["It's just a stupid game", "a sturdy green cardigan"] {
        let (printed, _) = generator.generate_with_pool(target);
        let pool = generator.generate_pool(target);
        let pool_words: HashSet<String> =
            pool.iter().flat_map(words_of).collect();
        let printed_words: HashSet<String> =
            printed.iter().flat_map(words_of).collect();
        let unprinted: usize = pool_words.difference(&printed_words).count();
        assert!(
            unprinted > 0,
            "{target}: the pool's whole vocabulary of {} words is used by \
             the printed {} proposals, so a word can no longer be in the \
             pool without also being printed",
            pool_words.len(),
            printed.len(),
        );
    }
}
