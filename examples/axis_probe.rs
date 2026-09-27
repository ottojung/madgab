//! SCRATCH HARNESS — w-1f6c40. NOT FOR MERGE.
//!
//! Records the approximate-mode baseline for a target given on the
//! command line: the printed proposal set, its word-count histogram,
//! the pool size and score-ordered cutoff, and — for each probe phrase
//! given on the command line — its printed rank, its pool rank, its
//! final score and its per-axis decomposition.
//!
//!     cargo run --release --example axis_probe -- "target phrase" "probe phrase" ...
//!
//! Every phrase is an argument. Nothing here names an example: the only
//! literals are axis names taken from the library, so the numbers this
//! prints cannot depend on which phrase was asked about.

use std::collections::BTreeMap;
use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TOP_N: usize = 50;

fn main() {
    let mut args = std::env::args().skip(1);
    let target = args.next().expect("usage: axis_probe <target> [probe ...]");
    let probes: Vec<String> = args.collect();

    let started = Instant::now();
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: TOP_N,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse");
    println!("config: mode=approximate per_word=0.5 total=1.5 top_n={TOP_N} beam={} max_rarity={:?} min_word_ipa_chars={}",
        g.config().beam_width, g.config().max_rarity, g.config().min_word_ipa_chars);
    println!("corpus_load_s: {:.3}", started.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let printed = g.generate(&target);
    let printed_s = t0.elapsed().as_secs_f64();

    let t1 = Instant::now();
    let (printed2, pool_size) = g.generate_with_pool(&target);
    assert_eq!(printed.len(), printed2.len());
    let pool_s = t1.elapsed().as_secs_f64();

    let t2 = Instant::now();
    let pool = g.generate_pool(&target);
    let pool_only_s = t2.elapsed().as_secs_f64();
    assert_eq!(pool.len(), pool_size, "pool size disagrees with the vector");

    println!("target: {target}");
    println!("printed_search_s: {printed_s:.3}");
    println!("pool_search_s: {pool_only_s:.3}");
    println!("pool_size: {pool_size}");
    println!("printed_count: {}", printed.len());

    let hist: BTreeMap<usize, usize> = {
        let mut h: BTreeMap<usize, usize> = BTreeMap::new();
        for c in &printed {
            *h.entry(c.words.len()).or_default() += 1;
        }
        h
    };
    let hist_txt: Vec<String> =
        hist.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    println!("printed_word_count_histogram: {{{}}}", hist_txt.join(", "));

    let last = printed.last().expect("a non-empty printed set");
    println!("printed_last_score: {:.6}", last.score);
    if let Some(first) = pool.get(49) {
        println!("pool_rank50_score: {:.6}", first.score);
    }
    let best = &printed[0];
    println!("best_printed: [{:.6}] {}", best.score, best.phrase);
    for (k, v) in g.axis_breakdown(best, &target).expect("axes") {
        println!("best_printed_axis: {k} {v:.6}");
    }

    // Self-check: the decomposition must reproduce the production score.
    let mut worst = 0.0f64;
    for c in printed.iter().chain(pool.iter().take(2000)) {
        let terms = g.axis_breakdown(c, &target).expect("axes");
        let total = terms.iter().rev().find(|(k, _)| *k == "TOTAL").unwrap().1;
        worst = worst.max((total - c.score).abs());
    }
    println!("axis_recomposition_max_abs_err: {worst:.3e}");

    println!("--- printed set");
    for (i, c) in printed.iter().enumerate() {
        println!("{:2}. [{:.6}] {} (words {})", i + 1, c.score, c.phrase, c.words.len());
    }

    for probe in &probes {
        println!("--- probe: {probe}");
        let words: Vec<&str> = probe.split_whitespace().collect();
        let printed_rank = printed.iter().position(|c| c.phrase == *probe);
        let pool_rank = pool.iter().position(|c| c.phrase == *probe);
        match printed_rank {
            Some(i) => println!("printed_rank: {}", i + 1),
            None => println!("printed_rank: NOT PRINTED"),
        }
        match pool_rank {
            Some(i) => println!("pool_rank: {}", i + 1),
            None => println!("pool_rank: NOT IN POOL"),
        }
        if let Some(i) = pool_rank {
            println!("pool_score: {:.6}", pool[i].score);
            println!("--- probe axes, from the emitted clue");
            for (k, v) in g.axis_breakdown(&pool[i], &target).expect("axes") {
                println!("emitted_axis: {k} {v:.6}");
            }
        }
        if let Some(i) = printed_rank {
            println!("printed_score: {:.6}", printed[i].score);
        }
        match g.alignment_axes(&target, &words) {
            Some(terms) => {
                println!("--- probe axes, reconstructed on the search lattice");
                for (k, v) in terms {
                    println!("axis: {k} {v:.6}");
                }
            }
            None => println!("axis: NOT PLACABLE ON THE LATTICE"),
        }
    }
}
