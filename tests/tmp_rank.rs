//! TEMPORARY diagnostic: rank position of the canonical clues in a
//! wide output pool. Deleted before commit.
use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn gen() -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .unwrap()
}

#[test]
fn tmp_rank_classic() {
    let g = gen();
    let clues = g.generate("It's just a stupid game");
    for (i, c) in clues.iter().enumerate() {
        if c.phrase.to_lowercase() == "hits justice dupe hid came" {
            eprintln!("TMP classic rank={i} score={:.4}", c.score);
            return;
        }
    }
    eprintln!(
        "TMP classic ABSENT from top {}; top5={:?} pool_top_scores={:?}",
        clues.len(),
        clues.iter().take(5).map(|c| (&c.phrase, c.score)).collect::<Vec<_>>(),
        clues.iter().take(3).map(|c| c.score).collect::<Vec<_>>(),
    );
}

#[test]
fn tmp_rank_speech() {
    let g = gen();
    let clues = g.generate("recognize speech");
    for (i, c) in clues.iter().enumerate() {
        if c.phrase.to_lowercase() == "wreck a nice beach" {
            eprintln!("TMP speech rank={i} score={:.4}", c.score);
            return;
        }
    }
    eprintln!(
        "TMP speech ABSENT from top {}; top5={:?}",
        clues.len(),
        clues.iter().take(5).map(|c| (&c.phrase, c.score)).collect::<Vec<_>>(),
    );
}
