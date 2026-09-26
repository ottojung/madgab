//! Integration tests against the real shipped corpus.
//!
//! These complement the unit tests in src/lib.rs by exercising the
//! full corpus and a few known Mad Gab / oronym regressions.

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

/// Behaviour lock for the approximate search.
///
/// The search is heavily optimized (see the work item for the perf work),
/// and every optimization is required to be behaviour-preserving. These
/// expected lists are the exact, full-precision ranked proposals the
/// pre-optimization binary produced; a refactor that changes any score or
/// the order of the proposals fails here. Scores are printed to six
/// decimals, which is far tighter than any scoring change is allowed to be.
///
/// To refresh after an intentional behaviour change, regenerate with
/// `cargo test --release --test corpus_integration dump_golden -- --nocapture`.
#[test]
fn approximate_output_is_bit_identical_to_the_baseline() {
    const CASES: &[(&str, &[&str])] = &[
        (
            "I love you",
            &[
                "0.971659 i'll a view",
                "0.954234 while of few",
                "0.950271 i'll but few",
                "0.964318 i'll of new",
                "0.953154 while a new",
                "0.964398 i'll up view",
                "0.950468 a will of",
                "0.942217 i'll are a",
                "0.961495 while a few",
                "0.940367 i'll on views",
            ],
        ),
        (
            "recognize speech",
            &[
                "0.934433 yet a guys each",
                "0.930286 yeah can i.'s it's",
                "0.919149 reckon i speaks",
                "0.926101 yet an i.'s pitch",
                "0.926282 read can i speaks",
                "0.925303 let a guy speaks",
                "0.920163 let an i seats",
                "0.929444 wreck a nice pitch",
                "0.923119 let egg nice each",
                "0.915371 yeah tag nice pitch",
            ],
        ),
        (
            "my favorite color is blue",
            &[
                "0.898527 mice a. ever it a liz blew",
                "0.887657 mm life avery skull in you",
                "0.891998 nice a. every colour new",
                "0.892411 me i've a.'s er it colour blew",
                "0.890920 knife avery took are in you",
                "0.891809 man if avery colour new",
                "0.891190 mm i've a. marie colour blew",
                "0.893486 man if a.'s er it uhh lib lou",
                "0.892875 mice avery took a lived lou",
                "0.893648 mm life a.'s er it a lived lou",
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
