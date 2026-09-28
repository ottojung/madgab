//! Measurement-only harness (front `a3f19c`, w-2f7a10): default-path pool size
//! and pool membership of named wordings, release build, via the public API.
//! Not committed; changes nothing in `src/` or `tests/`.
//!
//! A witness containing a space is tested as an exact full phrase; a single-word
//! witness is tested as membership of that word in any pool wording.
//! `REPEATS` runs of the same target are printed so run-to-run pool variation
//! on identical code is visible rather than assumed away.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[(&str, &[&str])] = &[
    ("It's just a stupid game", &["hits justice dupe hid came", "hid", "hits"]),
    ("recognize speech", &["wreck a nice beach"]),
    ("a whole lot of trouble", &["delve", "trouble"]),
    ("the cat sat on the mat", &["tickets", "mat"]),
    ("put it back on the shelf", &["taught", "louis"]),
    ("when the rain finally stopped", &["aar", "rain"]),
    ("he was a big fat man", &["honour", "man"]),
    ("what are you going to do", &["perdue", "do"]),
    ("she had a lot of money", &["money"]),
    ("there is no way to know", &["way"]),
    ("an old man in a big hat", &["hat"]),
    ("my brother has a red car", &["car"]),
];

const REPEATS: usize = 3;

fn main() {
    for (target, witnesses) in TARGETS {
        for rep in 0..REPEATS {
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
            let members: Vec<String> = witnesses
                .iter()
                .map(|w| {
                    let hit = if w.contains(' ') {
                        pool.iter().any(|p| p == w)
                    } else {
                        pool.iter().any(|p| p.split(' ').any(|q| q == *w))
                    };
                    format!("{w}={hit}")
                })
                .collect();
            let cap = if pool.len() >= 20_000 { " CAPPED" } else { "" };
            println!(
                "{}\trep{}\tpool {}{}\t{}",
                target.replace(' ', "_"),
                rep,
                pool.len(),
                cap,
                members.join(" ")
            );
        }
    }
}
