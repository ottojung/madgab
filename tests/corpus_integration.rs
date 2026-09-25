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
