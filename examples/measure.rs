//! Measurement harness for approximate-mode proposal quality.
//!
//! Not a test: this exists to compare scoring variants on aggregate
//! quality over a spread of targets, so the weight of a new axis is
//! chosen from numbers rather than from whether one canonical example
//! moved.  Run with
//!
//!     cargo run --release --example measure
//!
//! Reports, per variant:
//!   * the mean share of proposals whose words are *all* content words;
//!   * the mean number of distinct boundary structures per proposal list;
//!   * the mean share of proposals that reuse a target word (proxy for
//!     "is this just the target respelt");
//!   * the mean final score of the visible proposal list, and the mean
//!     acoustic similarity, so a content-word win cannot hide a loss of
//!     sayability;
//!   * search wall clock.

use std::collections::HashSet;
use std::time::Instant;

use madgab::lexical::is_closed_class;
use madgab::{Clue, Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// A spread of targets: the two canonical acceptance phrases, the
/// function-word salad cases the axis is meant to fix, and a mix of
/// ordinary two- to four-word phrases of different lengths.
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
    "an old man in a big hat",
    "every single one of them",
    "we should have told her",
    "with all of his little friends",
    "just a little bit more",
    "my brother has a red car",
    "in the middle of the night",
    "put it back on the shelf",
    "can you hear me now",
];

const TOP_N: usize = 20;

struct Stats {
    all_content: f64,
    mean_content: f64,
    distinct_structures: f64,
    dominant_structure_share: f64,
    reuse_share: f64,
    mean_score: f64,
    mean_similarity: f64,
    seconds: f64,
}

fn boundaries_of(c: &Clue) -> Vec<usize> {
    let mut cuts = Vec::new();
    let mut at = 0usize;
    for w in c.words.iter().skip(1) {
        at += w.ipa.chars().count();
        cuts.push(at);
    }
    cuts
}

fn all_content(c: &Clue) -> bool {
    c.words.iter().all(|w| !is_closed_class(&w.word))
}

fn content_share(c: &Clue) -> f64 {
    if c.words.is_empty() {
        return 0.0;
    }
    let closed = c.words.iter().filter(|w| is_closed_class(&w.word)).count();
    1.0 - closed as f64 / c.words.len() as f64
}

/// Cheap stand-in for the acoustic axis, recomputed from public clue
/// data so the harness does not depend on library internals: the share
/// of each clue word's phone budget that needed no edit at all.
fn similarity(c: &Clue) -> f64 {
    if c.words.is_empty() {
        return 0.0;
    }
    1.0 - c.words.iter().map(|w| w.sub_cost).sum::<f64>() / 4.0
}

fn measure(g: &Generator, verbose: bool) -> Stats {
    let mut acc = Stats {
        all_content: 0.0,
        mean_content: 0.0,
        distinct_structures: 0.0,
        dominant_structure_share: 0.0,
        reuse_share: 0.0,
        mean_score: 0.0,
        mean_similarity: 0.0,
        seconds: 0.0,
    };
    let n = TARGETS.len() as f64;

    for target in TARGETS {
        let started = Instant::now();
        let clues = g.generate(target);
        acc.seconds += started.elapsed().as_secs_f64();
        assert!(!clues.is_empty(), "no proposals for {target:?}");

        acc.all_content +=
            clues.iter().filter(|c| all_content(c)).count() as f64 / clues.len() as f64;
        acc.mean_content +=
            clues.iter().map(content_share).sum::<f64>() / clues.len() as f64;
        acc.mean_score +=
            clues.iter().map(|c| c.score).sum::<f64>() / clues.len() as f64;
        acc.mean_similarity +=
            clues.iter().map(similarity).sum::<f64>() / clues.len() as f64;

        let target_words: HashSet<String> = target
            .split_whitespace()
            .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .collect();
        let reused = clues
            .iter()
            .map(|c| {
                let hit = c
                    .words
                    .iter()
                    .filter(|w| target_words.contains(&w.word.to_lowercase()))
                    .count();
                hit as f64 / c.words.len() as f64
            })
            .sum::<f64>()
            / clues.len() as f64;
        acc.reuse_share += reused;

        let mut counts: std::collections::BTreeMap<Vec<usize>, usize> =
            std::collections::BTreeMap::new();
        for c in &clues {
            *counts.entry(boundaries_of(c)).or_default() += 1;
        }
        acc.distinct_structures += counts.len() as f64;
        let dominant = counts.values().copied().max().unwrap_or(0);
        acc.dominant_structure_share += dominant as f64 / clues.len() as f64;

        if verbose {
            eprintln!("--- {target}");
            for (i, c) in clues.iter().take(10).enumerate() {
                eprintln!(
                    "{:2}. [{:.4}] {}  (content {:.2})",
                    i + 1,
                    c.score,
                    c.phrase,
                    content_share(c)
                );
            }
        }
    }

    acc.all_content /= n;
    acc.mean_content /= n;
    acc.distinct_structures /= n;
    acc.dominant_structure_share /= n;
    acc.reuse_share /= n;
    acc.mean_score /= n;
    acc.mean_similarity /= n;
    acc
}

fn main() {
    let verbose = std::env::args().any(|a| a == "-v" || a == "--verbose");
    let started = Instant::now();
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: TOP_N,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse");
    eprintln!("corpus load {:.2}s", started.elapsed().as_secs_f64());

    let stats = measure(&g, verbose);

    println!("{:<34} {:>8}", "metric", "value");
    println!("{:<34} {:>8.4}", "all-content proposal share", stats.all_content);
    println!("{:<34} {:>8.4}", "mean content-word share", stats.mean_content);
    println!("{:<34} {:>8.2}", "mean distinct structures", stats.distinct_structures);
    println!("{:<34} {:>8.4}", "mean dominant-structure share", stats.dominant_structure_share);
    println!("{:<34} {:>8.4}", "mean target-word reuse share", stats.reuse_share);
    println!("{:<34} {:>8.4}", "mean visible score", stats.mean_score);
    println!("{:<34} {:>8.4}", "mean acoustic similarity", stats.mean_similarity);
    println!("{:<34} {:>8.2}", "search seconds (24 targets)", stats.seconds);
}
