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

/// The candidate pool deep enough to show wordings that only a wide slot
/// search finds.
///
/// `top_n` here is deliberately far larger than the pool any of these
/// targets produces (the largest is under 20 000), so what comes back is
/// the whole enumeration rather than a selected prefix of it: the point is
/// the pool, not the visible list, and a word that only a deep match
/// supplies can sit thousands of ranks down.
fn approximate_pool(target: &str) -> Vec<String> {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 20_000,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    g.generate(target)
        .into_iter()
        .map(|c| c.phrase.to_lowercase())
        .collect()
}

/// A slot's candidate list is a property of the traversal, not a uniform
/// pre-filter: a word that sits deep in one span's match list must still be
/// reachable, because the enumeration opens a slot wider once the traversal
/// has run out of nodes at the narrow width and still wants wordings.
///
/// The observable form of that is a *wording the narrow search never
/// produces*.  Each entry below was measured absent from the pool of the
/// binary built from the parent commit — which truncates every slot to a
/// fixed width — and present in this one's; the deep word is named in the
/// position it actually occupies, because that is the only thing that
/// distinguishes them.  Note what this is *not* asserting: that a deep word
/// is missing from the narrow pool.  For these targets it is not — the
/// narrow search reaches the same rare final words, and what it cannot
/// reach is the particular combination.  So the guard is on the wording,
/// not on the vocabulary.
///
/// The wordings are re-measured against the current parent, which includes
/// the reserved depth-profile spend: that reserve changes every pool, and an
/// earlier trio chosen against a tree without it stopped discriminating
/// (two of the three are now produced by the narrow search too).  A wording
/// in this list is therefore a statement about the *merged* behaviour of the
/// two mechanisms, not about this front alone.
///
/// All four targets are ordinary English phrases, not the acceptance
/// examples, and nothing in `src/` knows them; see
/// [w-9d4e17](../../docs/work/items/w-9d4e17.md) for the measurement.
#[test]
fn approximate_pool_reaches_matches_deep_in_a_span() {
    for (target, wording) in [
        // First-slot matches from past the head of that span's list.
        ("I love you", "ask lovey"),
        ("big spender", "began edgar"),
        // A second-slot match, behind a first-slot match the narrow search
        // does produce ("plague" is in both pools).
        ("play games with me now", "bay games meow"),
        // A final word the narrow search does not propose for this target.
        ("taco cat", "taco net"),
    ] {
        let pool = approximate_pool(target);
        assert!(
            pool.iter().any(|p| p == wording),
            "{target}: {wording:?} missing, so a match deep in a span's match \
             list was not reached; pool had {} clues, e.g. {:?}",
            pool.len(),
            &pool[..pool.len().min(8)]
        );
    }
}

/// A clue's resegmentation, as the search aligned it.
///
/// This reads [\"madgab::Clue::cuts\"] rather than re-deriving offsets
/// by summing the clue words' own IPA lengths.  The two disagree under
/// an approximate alignment — a clue word consumes a run of the target's
/// phonemes, not its own transcription — so re-deriving reports one
/// resegmentation as many, and this test was measuring the diversity
/// policy's *belief* about the list rather than the list.
fn clue_boundaries(c: &madgab::Clue) -> Vec<usize> {
    c.cuts[..c.cuts.len().saturating_sub(1)].to_vec()
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
/// The values have been re-baselined twice on
/// `post-milestone-acceptance`, because scoring and selection fronts
/// (the closed-class clue-quality axis, the ordered selection rule, and
/// the resegmentation-offset fix in [w-04f83f]) landed after the perf
/// work was measured and intentionally change approximate output.  The
/// lock protects the tree as integrated from here on; it does not claim
/// any of those changes was behaviour-preserving, which their own work
/// items recorded.
///
/// The re-baseline in w-04f83f is a *selection* change only: no score
/// moved, and the same candidates are in the pool.  The visible list
/// differs because the diversity policy's share cap started working —
/// it can now tell that several of the previous entries are wordings of
/// one resegmentation — so the slots they gave up went to the best
/// candidates of other resegurations.
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
                "0.931877 how ill view",
                "0.931877 now ill view",
                "0.930994 isle of new",
                "0.930262 aisle of new",
                "0.930120 i.'s of new",
                "0.929948 yeah ill view",
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
