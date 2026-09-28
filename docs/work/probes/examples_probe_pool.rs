//! PROBE (scratch/5b1e93-probe only). Harness validation + pool/visible
//! statistics over a spread of real targets, through the public
//! `Generator::generate_pool` boundary.

use std::collections::BTreeMap;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[&str] = &[
    "recognize speech",
    "It's just a stupid game",
    "I love you",
    "a whole lot of trouble",
    "he was a big fat man",
    "what are you going to do",
    "some kind of wonderful thing",
    "she had a lot of money",
    "there is no way to know",
    "the cat sat on the mat",
    "if you want to go now",
    "they are going to be late",
    "one of those other people",
    "when the rain finally stopped",
    "you can do it yourself",
];

fn main() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse");

    println!("{:<30} {:>7} {:>14} {:>7} {:>28} {:>8}", "target", "pool", "rank50", "n50", "visible n distribution", "meanscore");
    for target in TARGETS {
        let pool = g.generate_pool(target);
        let rank50 = pool[49.min(pool.len() - 1)].score;
        let vis = g.generate(target);
        let mut dist: BTreeMap<usize, usize> = BTreeMap::new();
        for c in &vis {
            *dist.entry(c.words.len()).or_default() += 1;
        }
        let dist_s: Vec<String> = dist
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect();
        let mean: f64 = vis.iter().map(|c| c.score).sum::<f64>() / vis.len() as f64;
        println!(
            "{:<30} {:>7} {:>14.12} {:>7} {:>28} {:>8.6}",
            target,
            pool.len(),
            rank50,
            vis.len(),
            dist_s.join(" "),
            mean
        );
    }
}
