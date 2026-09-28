//! w-5e2d42 base-side probe: pool membership of five named words on the
//! base commit, with the in-crate report enabled via env vars.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGET: &str = "It's just a stupid game";
const WORDS: &[&str] = &["hits", "justice", "dupe", "hid", "came"];

fn main() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::Approximate {
                per_word_budget: 0.5,
                total_budget: 1.5,
            },
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let (_printed, pool_size) = g.generate_with_pool(TARGET);
    let pool = g.generate_pool(TARGET);
    println!("pool {} (reported {})", pool.len(), pool_size);
    for w in WORDS {
        let n = pool
            .iter()
            .filter(|c| c.words.iter().any(|t| t.word == *w))
            .count();
        println!("{w}: pool wordings containing it: {n}");
    }
}
