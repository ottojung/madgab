//! PROBE7E1A04 measurement harness. Scratch branch only; never merges.
//!
//! For each target: dump the approximate pool's per-axis values (via the
//! PROBE7E1A04_AXES hook in `into_clue`) and the exact-mode ladder, and
//! print the pool size and rank-50 cutoff read from the public API so the
//! harness can be validated against the pristine figures.

use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[&str] = &[
    "It's just a stupid game",
    "recognize speech",
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
    let t0 = Instant::now();
    let approx = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let exact = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::Exact,
            top_n: 50,
            beam_width: 512,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    println!("load_s {:.3}", t0.elapsed().as_secs_f64());

    for (i, target) in TARGETS.iter().enumerate() {
        let tag = i;
        let path = format!("/workspace/probe-out/pool-{tag}.tsv");
        unsafe { std::env::set_var("PROBE7E1A04_AXES", &path) };
        let t = Instant::now();
        let pool = approx.generate_pool(target);
        let dt = t.elapsed().as_secs_f64();
        let cutoff = pool.get(49).map(|c| c.score).unwrap_or(f64::NAN);
        let r = pool.iter().position(|c| {
            c.phrase.to_lowercase()
                == std::env::var("PROBE7E1A04_MATCH").unwrap_or_default()
        });
        println!(
            "POOL\t{tag}\t{}\t{:.12}\t{:.3}\t{:?}",
            pool.len(),
            cutoff,
            dt,
            r.map(|v| v + 1)
        );

        let t2 = Instant::now();
        let ex = exact.generate_pool(target);
        println!("EXACT\t{tag}\t{}\t{:.3}", ex.len(), t2.elapsed().as_secs_f64());
        unsafe { std::env::remove_var("PROBE7E1A04_AXES") };
    }
    println!("total {:.3}", t0.elapsed().as_secs_f64());
}
