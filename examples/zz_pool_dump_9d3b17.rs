//! Read-only tip-verify harness for w-2f7a10 (front 9d3b17).
//!
//! Runs the *unmodified default* approximate search (SearchMode::approximate(),
//! top_n 50, beam_width 64 -- the same configuration
//! `tests/corpus_integration.rs::approximate_proposals` uses) on the six
//! measured targets plus the canonical one, prints the visible top 50 to
//! stdout, and lets the env-gated probe in `src/lib.rs` dump the
//! deduplicated pool.  No search behaviour is changed here and no ranking,
//! budget or enumeration parameter is touched.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[&str] = &[
    "It's just a stupid game",
    "a whole lot of trouble",
    "the cat sat on the mat",
    "put it back on the shelf",
    "when the rain finally stopped",
    "he was a big fat man",
    "what are you going to do",
];

fn main() {
    let dump = std::env::var("MADGAB_POOL_DUMP").ok().filter(|s| !s.is_empty());
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

    for target in TARGETS {
        if dump.is_some() {
            std::env::set_var("MADGAB_POOL_TARGET", target);
        }
        let clues = g.generate(target);
        println!("TARGET\t{target}\tshown\t{}", clues.len());
        for (i, c) in clues.iter().enumerate() {
            println!("{}\t{:.9}\t{}", i, c.score, c.phrase);
        }
    }
}
