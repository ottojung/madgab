//! Integration tests against the real shipped corpus.
//!
//! These complement the unit tests in src/lib.rs by exercising the
//! full corpus and a few known Mad Gab / oronym regressions.

use std::collections::HashMap;

use madgab::lexical::is_closed_class;
use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn gen_unfiltered() -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            max_rarity: None,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse")
}

#[test]
fn canonical_words_have_expected_narrow_ipa() {
    let g = gen_unfiltered();
    let c = g.corpus();
    let checks: &[(&str, &str)] = &[
        ("stupid", "stup"),
        ("hid", "hɪd"),
        ("hits", "hɪts"),
        ("justice", "dʒʌs"),
        ("love", "lʌv"),
        ("just", "dʒ"),
        ("chair", "tʃ"),
        ("game", "eɪ"),
        ("came", "eɪ"),
        ("dupe", "dup"),
        ("a", "ə"),
        ("the", "ðə"),
    ];
    for (word, expected) in checks {
        let ipa = c
            .preferred_ipa(word)
            .unwrap_or_else(|| panic!("corpus must know {:?}", word));
        let stripped: String = ipa
            .chars()
            .filter(|&ch| ch != 'ˈ' && ch != 'ˌ')
            .collect();
        let stripped = stripped.replace('g', "ɡ");
        assert!(
            stripped.contains(expected),
            "expected {:?} in preferred IPA of {:?}, got {:?}",
            expected,
            word,
            ipa,
        );
    }
}

#[test]
fn known_madgab_pair_is_searchable() {
    let g = gen_unfiltered();
    let c = g.corpus();
    for w in [
        "it's", "just", "a", "stupid", "game", "hits", "justice", "dupe", "hid",
        "came",
    ] {
        assert!(
            c.preferred_ipa(w).is_some(),
            "corpus missing canonical Mad Gab word {:?}",
            w,
        );
    }
}

#[test]
fn generates_for_a_real_phrase() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            top_n: 5,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let clues = g.generate("I love you");
    assert!(!clues.is_empty(), "expected at least one clue for 'I love you'");
    for c in &clues {
        assert!(!c.phrase.is_empty());
        assert!(
            c.score.is_finite() && (0.0..=1.0).contains(&c.score),
            "score out of [0,1]: {}",
            c.score
        );
    }
}

#[test]
fn approximate_mode_runs_end_to_end() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 10,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let clues = g.generate("I love you");
    assert!(!clues.is_empty(), "approximate mode found no clues");
    for c in &clues {
        assert!(!c.words.is_empty());
    }
}

fn approximate_proposals(target: &str) -> Vec<String> {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    g.generate(target)
        .into_iter()
        .map(|c| c.phrase.to_lowercase())
        .collect()
}

#[test]
fn approximate_finds_classic_madgab_resegmentation() {
    let proposals = approximate_proposals("It's just a stupid game");
    assert!(
        proposals.iter().any(|p| p == "hits justice dupe hid came"),
        "canonical clue missing from top 50; got: {:?}",
        &proposals[..proposals.len().min(12)]
    );
}

#[test]
fn approximate_finds_recognize_speech_resegmentation() {
    let proposals = approximate_proposals("recognize speech");
    assert!(
        proposals.iter().any(|p| p == "wreck a nice beach"),
        "canonical clue missing from top 50; got: {:?}",
        &proposals[..proposals.len().min(12)]
    );
}

/// A Mad Gab answer has to be readable, and the aggregate property that
/// makes it readable is that it is built from *content* words.  This is
/// the corpus-level counterpart to `proposal_list_covers_distinct_
/// resegmentations`: the same two properties, measured over real
/// proposals from real targets rather than over a synthetic pool.
///
/// It is deliberately two targets, sharing one `Generator` (the corpus
/// load is ~0.5s and each approximate search is seconds), and it asserts
/// on proportions and on structural spread rather than on any
/// particular clue, so it keeps holding as the ranking changes.
#[test]
fn approximate_proposals_are_predominantly_content_words() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 20,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();

    for target in ["I love you", "a whole lot of trouble"] {
        let clues = g.generate(target);
        assert!(!clues.is_empty(), "no proposals for {target:?}");

        let content_share: f64 = clues
            .iter()
            .map(|c| {
                let closed = c
                    .words
                    .iter()
                    .filter(|w| is_closed_class(&w.word))
                    .count();
                1.0 - closed as f64 / c.words.len().max(1) as f64
            })
            .sum::<f64>()
            / clues.len() as f64;
        assert!(
            content_share > 0.75,
            "{target:?}: only {content_share:.3} of the words in the visible \
             proposals are content words; proposals were {:?}",
            clues
                .iter()
                .map(|c| c.phrase.as_str())
                .collect::<Vec<_>>()
        );

        // The axis must not have collapsed the list onto one
        // resegmentation: it is a tie-breaker between structures, not a
        // reason to prefer a single one.
        let mut counts: HashMap<Vec<usize>, usize> = HashMap::new();
        for c in &clues {
            *counts.entry(clue_boundaries(c)).or_default() += 1;
        }
        let dominant = counts.values().copied().max().unwrap_or(0);
        assert!(
            counts.len() >= 3,
            "{target:?}: visible proposals span only {} resegmentation(s): {:?}",
            counts.len(),
            clues
                .iter()
                .map(|c| c.phrase.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            dominant as f64 / clues.len() as f64 <= 0.5,
            "{target:?}: one resegmentation occupies {dominant} of {} \
             proposals; the content-word axis collapsed the diversity",
            clues.len()
        );
    }
}

fn clue_boundaries(c: &madgab::Clue) -> Vec<usize> {
    let mut cuts = Vec::new();
    let mut at = 0usize;
    for w in c.words.iter().skip(1) {
        at += w.ipa.chars().count();
        cuts.push(at);
    }
    cuts
}

/// Structural diversity on a real search: the visible list may not be
/// one resegmentation.  This is the half of the selection policy that
/// the synthetic unit tests in `src/lib.rs` cannot check, because they
/// need a real candidate pool.  (Those carry the other half: a
/// near-equal alternative in a shown structure is visible, while a much
/// worse sibling is not.)
#[test]
fn approximate_list_is_not_one_resegmentation() {
    // Deliberately not only the two acceptance phrases: this is a
    // general property of the policy, so it is checked on a spread.
    for target in [
        "recognize speech",
        "It's just a stupid game",
        "taco cat",
        "sign on",
        "big spender",
    ] {
        let g = Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n: 50,
                beam_width: 64,
                ..GeneratorConfig::default()
            },
        )
        .unwrap();
        let clues = g.generate(target);
        assert!(clues.len() >= 10, "{target}: only {} clues", clues.len());

        // The list is presented in descending score order, so it opens
        // with the best wordings the search found.
        for pair in clues.windows(2) {
            assert!(
                pair[0].score >= pair[1].score,
                "{target}: list is not in score order: {} then {}",
                pair[0].score,
                pair[1].score
            );
        }

        let mut shown: HashMap<Vec<usize>, usize> = HashMap::new();
        for c in &clues {
            *shown.entry(clue_boundaries(c)).or_default() += 1;
        }
        let largest = shown.values().copied().max().unwrap();
        assert!(
            largest * 2 <= clues.len(),
            "{target}: one structure holds {largest} of {} slots ({shown:?})",
            clues.len()
        );
    }
}

/// Behaviour lock for the optimized approximate search.
///
/// [w-7fa26c](../../docs/work/items/w-7fa26c.md) made the approximate
/// search ~1.3-2.1x faster and every optimization in it is required to be
/// behaviour-preserving.  These expected lists are the exact,
/// full-precision ranked proposals the pre-optimization binary produced;
/// a refactor that changes any score or the order of the proposals fails
/// here.  Scores are printed to six decimals, which is far tighter than
/// any scoring change is allowed to be.
///
/// The values were re-baselined on `post-milestone-acceptance` when this
/// lock was integrated, because two scoring/selection fronts (the
/// closed-class clue-quality axis and the ordered selection rule) landed
/// after the perf work was measured and intentionally change approximate
/// output.  The lock protects the tree as integrated from here on; it does
/// not claim the perf work was bit-identical to that older base, which
/// its own work item verified separately.
#[test]
fn approximate_output_is_locked() {
    const CASES: &[(&str, &[&str])] = &[
        (
            "I love you",
            &[
                "0.938335 isle a view",
                "0.937604 aisle a view",
                "0.937462 i.'s a view",
                "0.936762 eye a view",
                "0.932630 isle come view",
                "0.932621 i'll come view",
                "0.931899 aisle come view",
                "0.931877 how ill view",
                "0.931877 now ill view",
                "0.931756 i.'s come view",
            ],
        ),
    ];

    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 10,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();

    for (target, expected) in CASES {
        let got: Vec<String> = g
            .generate(target)
            .iter()
            .map(|c| format!("{:.6} {}", c.score, c.phrase))
            .collect();
        assert_eq!(&got, expected, "approximate output changed for {target:?}");
    }
}
