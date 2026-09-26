//! Measurement harness (front `e1a3f7`, work item `w-2f7a10`): pool sizes
//! and pool membership on the **default** approximate path, release build.
//!
//!     cargo run --release --example costpool
//!
//! For each target: the size of the deduplicated candidate pool the release
//! binary actually produces, plus whether given wordings are members of that
//! pool.  Read-only: it drives the public `Generator` API with the default
//! approximate configuration, and it changes nothing in `src/`.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[(&str, &[&str])] = &[
    ("It's just a stupid game", &["hits", "dupe", "hid", "came"]),
    ("recognize speech", &["wreck a nice beach"]),
    ("a whole lot of trouble", &["a", "lot"]),
    ("the cat sat on the mat", &["cat", "sat"]),
    ("put it back on the shelf", &["shelf", "back"]),
    ("when the rain finally stopped", &["rain", "stopped"]),
    ("he was a big fat man", &["fat", "man"]),
    ("what are you going to do", &["going", "do"]),
    ("she had a lot of money", &["money", "she"]),
    ("there is no way to know", &["know", "way"]),
    ("an old man in a big hat", &["hat", "man"]),
    ("my brother has a red car", &["car", "brother"]),
];

fn main() {
    for (target, wordings) in TARGETS {
        let g = Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n: 20_000,
                ..GeneratorConfig::default()
            },
        )
        .unwrap();
        let pool: Vec<String> = g
            .generate(target)
            .into_iter()
            .map(|c| c.phrase.to_lowercase())
            .collect();
        let members: Vec<String> = wordings
            .iter()
            .map(|w| {
                let hit = pool.iter().any(|p| {
                    p.split(' ').any(|t| t == *w) || p.split(' ').eq(w.split(' '))
                });
                format!("{w}={hit}")
            })
            .collect();
        println!(
            "{}\tpool {}\t{}",
            target.replace(' ', "_"),
            pool.len(),
            members.join(" ")
        );
    }
}
