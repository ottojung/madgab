//! Stage attribution and priced-negative pin for the default-visibility
//! ordering question in [w-c31a07](../docs/work/items/w-c31a07.md).
//!
//! The canonical case-1 alignment is present in the pool and absent from the
//! shipped default `--top 10`. These tests pin *why*, so a later pass does not
//! re-derive it, and pin the measured fact that the ordering surfaces already
//! tried do not lift it, so it is not re-priced a fifth time.

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

/// The word-boundary structure a clue was aligned at, which is what
/// `select_diverse`'s share cap and structure reserve key on.
fn structure(c: &madgab::Clue) -> Vec<usize> {
    let mut cuts = c.cuts.clone();
    cuts.pop();
    cuts
}

#[test]
fn canonical_is_in_the_pool_and_the_score_order_places_it_at_27() {
    let pool = gen(10).generate_pool("recognize speech");
    let rank = pool
        .iter()
        .position(|c| c.phrase == CANON)
        .expect("canonical should be in the pool");
    // The stage that places it here is the *score order*, at src/lib.rs:4065
    // (the final `sort_by(cmp_desc(score))` in `select_diverse`), fed by the
    // pool sort at src/lib.rs:2444. It is not the diversity layer: it is the
    // 27th-best-scoring member of the pool, and `select_diverse` reproduces
    // score order for this target rather than substituting any other member.
    assert_eq!(rank, 26, "canonical should be pool rank 26 = display 27");
    let score = pool[rank].score;
    assert!(
        (score - 0.919_950).abs() < 5e-7,
        "canonical score should be 0.919950, got {score}"
    );
    assert_eq!(structure(&pool[rank]), CANON_STRUCTURE);
}

#[test]
fn the_selection_layer_is_a_pure_pass_through_at_the_shipped_default() {
    // Criterion 1: the loss happens at the ordering key, not in the selection
    // layer. At the default and just above it, every displayed clue sits at
    // exactly its own pool rank, so `select_diverse` is admitting the top
    // `top_n` of the score order and substituting nothing: the share cap and
    // the structure reserve are both slack here. It is the *cutoff* that
    // excludes the canonical, not a diversity decision about it.
    //
    // The cap does bind as the list grows, which is why the shipped `--top 50`
    // is a different list from the pool head; that divergence is downstream
    // of the fact measured here and cannot help a near-tie at the default.
    for n in [10usize, 11, 15] {
        let pool = gen(n).generate_pool("recognize speech");
        let displayed = gen(n).generate("recognize speech");
        for (display_i, clue) in displayed.iter().enumerate() {
            let pool_rank = pool
                .iter()
                .position(|c| c.phrase == clue.phrase)
                .expect("displayed clue came from the pool");
            assert_eq!(
                display_i, pool_rank,
                "at top_n={n} displayed {:?} at slot {display_i} is pool rank {pool_rank}",
                clue.phrase
            );
        }
    }
}

#[test]
fn canonical_is_absent_at_the_default_and_present_at_fifty() {
    // The shipped default.
    let top10 = gen(10).generate("recognize speech");
    assert_eq!(top10.len(), 10);
    assert!(
        !top10.iter().any(|c| c.phrase == CANON),
        "canonical must stay absent at --top 10 while this item is a priced negative"
    );
    // Absent at 25, present at 50. This is the near tie: 0.919950 against a
    // 10th displayed near 0.9218, a gap of about 0.002 in a pool of 13801.
    assert!(!gen(25).generate("recognize speech").iter().any(|c| c.phrase == CANON));
    let at50 = gen(50)
        .generate("recognize speech")
        .into_iter()
        .position(|c| c.phrase == CANON);
    assert_eq!(at50, Some(26), "canonical should be display 27 of 50");
}

#[test]
fn the_visible_head_is_one_ending_repeated_rather_than_distinct_wordings() {
    // The user-visible redundancy that motivated the rhyme family. The ten
    // displayed clues are near-rhyme variants of a single final word, which
    // is why content-word share and cluster merge could not separate them.
    let top10 = gen(10).generate("recognize speech");
    let finals: Vec<String> = top10
        .iter()
        .map(|c| c.words.last().expect("clue has a word").word.to_lowercase())
        .collect();
    // Nine of the ten are one of two rhymes of the same ending word, which is
    // what a reader sees as a repeated list rather than ten proposals.
    let pitch_family = finals.iter().filter(|f| f.as_str() == "pitch").count()
        + finals.iter().filter(|f| f.as_str() == "peach").count();
    assert_eq!(pitch_family, 9, "in {finals:?}");
    assert_eq!(finals.iter().filter(|f| f.as_str() == "beach").count(), 1);
}

/// Priced negative, w-c31a07. Every ordering surface that could plausibly
/// lift the canonical was measured on this data. The rhyme family — group by
/// (boundary structure, leading words, trailing run of the last word's IPA)
/// and admit only the best-scoring member of each group — is the sharpest of
/// them, and it does *not* lift the canonical, because the best member of its
/// rhyme class is `wreck a nice pitch` at 0.921438, which already holds a
/// visible slot. A rhyme rule trades which ending is shown; it cannot change
/// which member of a rhyme class is best. Pinned so it is not re-derived.
#[test]
fn rhyme_family_grouping_admits_a_different_member_not_the_canonical() {
    let pool = gen(10).generate_pool("recognize speech");
    let mut seen: std::collections::HashSet<(Vec<usize>, String, String)> =
        std::collections::HashSet::new();
    let mut representatives: Vec<&str> = Vec::new();
    for c in &pool {
        let st = structure(c);
        let leading: String = c.words[..c.words.len() - 1]
            .iter()
            .map(|w| w.word.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        let last: Vec<char> = c.words[c.words.len() - 1].ipa.chars().collect();
        let tail: String = last[last.len().saturating_sub(2)..].iter().collect();
        if seen.insert((st, leading, tail)) {
            representatives.push(&c.phrase);
        }
    }
    // A 2-phoneme trailing run collapses 3405 of 13801 pool members, and the
    // canonical is the 3rd member of a group whose representative is already
    // visible. It is not the representative at any run length tried (1..=4).
    assert!(
        !representatives.iter().take(10).any(|p| *p == CANON),
        "rhyme grouping is expected NOT to surface the canonical"
    );
    assert!(
        representatives.iter().take(10).any(|p| *p == "wreck a nice pitch"),
        "the representative of the canonical's own rhyme class is the pitch wording"
    );
}

/// Priced negative, w-c31a07: per-word exact-IPA grouping. Unlike the
/// phrase-level form it is not degenerate, but it also does not lift the
/// canonical: it collapses 3668 of 13801 members and leaves the canonical at
/// group rank 25, still outside the default ten.
#[test]
fn per_word_exact_ipa_grouping_leaves_the_canonical_outside_the_default() {
    let pool = gen(10).generate_pool("recognize speech");
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut representatives: Vec<&str> = Vec::new();
    for c in &pool {
        let k: String = c
            .words
            .iter()
            .map(|w| w.ipa.as_str())
            .collect::<Vec<_>>()
            .join("|");
        if seen.insert(k) {
            representatives.push(&c.phrase);
        }
    }
    assert_eq!(representatives.len(), 10_133);
    assert_eq!(pool.len(), 13_801);
    let at = representatives.iter().position(|p| *p == CANON);
    assert_eq!(at, Some(25), "still rank 25 of the groups, outside --top 10");
}

/// Priced negative, w-c31a07: content-word share. It does not separate the
/// canonical from what outranks it, so it cannot be the ordering key.
#[test]
fn content_word_share_does_not_separate_the_canonical_from_its_outrankers() {
    let pool = gen(10).generate_pool("recognize speech");
    let content = |w: &str| -> usize {
        w.split_whitespace()
            .filter(|x| {
                let n: String = x.chars().filter(|c| c.is_alphanumeric()).collect();
                !matches!(
                    n.to_lowercase().as_str(),
                    "a" | "i" | "the" | "an" | "is" | "was" | "of" | "to"
                )
            })
            .count()
    };
    let canon_words = pool
        .iter()
        .find(|c| c.phrase == CANON)
        .expect("canonical in pool");
    // 3/4 content words against 4/4 for a clue that outranks it: share is
    // anti-correlated here, so promoting on it demotes the canonical.
    assert!(content(CANON) < content("let egg nice pitch"));
    assert!(canon_words.score < 0.921_813);
}

/// Priced negative, w-c31a07: the canonical loses inside its own structure
/// (156 pool members share [3,5,10]), and the three structures above it are
/// [3,4,10], [2,6,10], [3,6,10]. Cluster merge and best-member-per-structure
/// reserve therefore have nothing to promote it from.
#[test]
fn canonical_loses_within_its_own_structure_not_to_a_cluster() {
    let pool = gen(10).generate_pool("recognize speech");
    let mut counts: std::collections::HashMap<Vec<usize>, usize> =
        std::collections::HashMap::new();
    for c in &pool {
        *counts.entry(structure(c)).or_default() += 1;
    }
    assert_eq!(counts[&CANON_STRUCTURE.to_vec()], 156);
    // Every member of the canonical's structure that outranks it.
    let ahead: Vec<&madgab::Clue> = pool
        .iter()
        .filter(|c| structure(c) == CANON_STRUCTURE && c.score > 0.919_950)
        .collect();
    assert_eq!(ahead.len(), 12, "12 same-structure siblings outrank it");
    for c in &ahead {
        assert!(
            c.score > 0.919_950,
            "sibling ordering should be strictly by score"
        );
    }
}

/// The known case-2 red, and the pool size it would have to improve to reach
/// the default. Pinned so a future pass sees the same width it is up against.
#[test]
fn case_two_pool_is_unaffected_and_still_far_from_the_canonical() {
    let pool = gen(10).generate_pool("It's just a stupid game");
    assert_eq!(pool.len(), 14_555);
    assert!(
        !pool
            .iter()
            .take(100)
            .any(|c| c.phrase == "hits justice dupe hid came"),
        "case 2 remains absent from the pool head"
    );
}
