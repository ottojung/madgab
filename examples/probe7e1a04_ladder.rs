//! PROBE7E1A04: the exact-mode ladder. Every candidate is a faithful
//! resegmentation of the target (an exact cover of its phone stream by
//! lexicon words) with zero edit cost, so the only thing that varies across
//! the ladder is the segmentation itself.  Scratch branch only.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[&str] = &[
    "It's just a stupid game",
    "I love you",
    "a whole lot of trouble",
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
    "he was a big fat man",
];

fn main() {
    let t0 = std::time::Instant::now();
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::Exact,
            top_n: 50,
            beam_width: 4096,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    for (i, target) in TARGETS.iter().enumerate() {
        unsafe { std::env::set_var("PROBE7E1A04_AXES", format!("/workspace/probe-out/ladder-{i}.tsv")) };
        let pool = g.generate_pool(target);
        println!("LADDER\t{i}\t{target}\t{}", pool.len());
        for c in pool.iter().take(4) {
            println!("  \t{:.6}\t{}\t{}", c.score, c.words.len(), c.phrase);
        }
        unsafe { std::env::remove_var("PROBE7E1A04_AXES") };
    }
    println!("total {:.3}", t0.elapsed().as_secs_f64());
}
