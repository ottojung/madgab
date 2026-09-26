//! Read-only wall-clock harness for w-2f7a10 (front 9d3b17).
//!
//! One discarded warm-up search per binary, then N timed runs of the
//! unmodified default approximate search (SearchMode::approximate(),
//! top_n 50, beam_width 64) per target.  One target per process so the
//! two arms can be interleaved by the driver.  Prints
//! `run<TAB>target<TAB>seconds` to stdout and nothing else.

use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let reps: usize = std::env::var("ZZ_REPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);
    let target = std::env::var("ZZ_TARGET").unwrap_or_else(|_| "It's just a stupid game".into());
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

    // Discarded warm-up: first search pays corpus/first-touch costs.
    let _ = g.generate(&target);
    for i in 0..reps {
        let started = Instant::now();
        let clues = g.generate(&target);
        println!(
            "run\t{}\t{:.4}\t{}",
            i,
            started.elapsed().as_secs_f64(),
            clues.len()
        );
    }
}
