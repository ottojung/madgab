//! Measurement harness for the span-shortlist retention front (w-5c1a3e).
//!
//! NOT production and NOT a test. This reports, for whatever target and
//! whatever clue word sequence the caller names on the command line:
//!
//!   * the production dedup pool size (`top_n` as given, so a large value
//!     returns the whole enumeration rather than a selected prefix);
//!   * the pool rank and score of the named clue, or the length of its
//!     longest leading run if it is absent;
//!   * the printed top-N acoustic-similarity profile of a second target.
//!
//! Every input is a command-line argument. Nothing here names a phrase, a
//! clue or a word: `no_phrase_hard_coding` scans `examples/`, and a
//! measurement harness that hard-codes the very alignment it is measuring
//! would be the defect the front exists to price.
//!
//! Run with:
//!
//!     cargo run --release --example retention_probe -- <target> <clue words...> [top_n] [beam]

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// Cheap stand-in for the per-candidate acoustic axis, recomputed from
/// public clue data so the harness does not depend on library internals:
/// the share of the per-word edit budget that needed no edit at all.
fn similarity(c: &madgab::Clue) -> f64 {
    if c.words.is_empty() {
        return 0.0;
    }
    1.0 - c.words.iter().map(|w| w.sub_cost).sum::<f64>() / 4.0
}

fn words_of(c: &madgab::Clue) -> Vec<String> {
    c.words.iter().map(|w| w.word.to_lowercase()).collect()
}

fn generator(top_n: usize, beam: usize) -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n,
            beam_width: beam,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: retention_probe <target> <clue words...> [top_n] [beam]");
        std::process::exit(2);
    }
    let target = args[0].clone();
    // Everything after the target is the clue, so a caller can pass any
    // number of words without this file knowing them; a bare integer in
    // tail position is read as a numeric option instead.
    let (clue, nums): (Vec<String>, Vec<usize>) = args[1..]
        .iter()
        .fold((Vec::new(), Vec::new()), |(mut w, mut n), a| {
            match a.parse::<usize>() {
                Ok(v) => n.push(v),
                Err(_) => w.push(a.to_lowercase()),
            }
            (w, n)
        });
    let top_n = nums.first().copied().unwrap_or(200_000);
    let beam = nums.get(1).copied().unwrap_or(64);

    let g = generator(top_n, beam);
    let started = std::time::Instant::now();
    let pool = g.generate(&target);
    println!(
        "pool_size={} seconds={:.2} top_n={top_n} beam={beam}",
        pool.len(),
        started.elapsed().as_secs_f64()
    );

    let mut found = false;
    for (i, c) in pool.iter().enumerate() {
        if words_of(c) == clue {
            println!("POOL_RANK={} score={:.10}", i + 1, c.score);
            found = true;
            break;
        }
    }
    if !found {
        let mut best = 0usize;
        for c in pool.iter() {
            let w = words_of(c);
            let run = (0..clue.len())
                .find(|&k| w.get(k) != clue.get(k))
                .unwrap_or(clue.len());
            best = best.max(run);
        }
        println!("clue_absent best_leading_run={best} of {}", clue.len());
    }

    // Printed-list similarity profile for the same target: the aggregate the
    // selection-layer reports are priced against.
    let printed = generator(50, 64).generate(&target);
    let mean: f64 = printed.iter().map(similarity).sum::<f64>() / printed.len().max(1) as f64;
    let mean_score: f64 =
        printed.iter().map(|c| c.score).sum::<f64>() / printed.len().max(1) as f64;
    println!(
        "printed50_len={} printed50_mean_similarity={mean:.6} printed50_mean_score={mean_score:.10}",
        printed.len()
    );
}
