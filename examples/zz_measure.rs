//! ZZ_ scratch measurement instrument (never committed to the impl branch).
use std::collections::HashSet;
use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn pool(target: &str, top: usize) -> Vec<(String, Vec<usize>, f64)> {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: top,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    g.generate(target)
        .into_iter()
        .map(|c| (c.phrase.to_lowercase(), c.cuts, c.score))
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let top: usize = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(50);
    let targets = [
        "It's just a stupid game",
        "I love you",
        "big spender",
        "play games with me now",
        "the cat sat on the mat",
        "a whole lot of trouble",
        "when the rain finally stopped",
    ];
    for t in targets {
        let t0 = Instant::now();
        let p = pool(t, top);
        let ms = t0.elapsed().as_millis();
        let distinct_words: HashSet<&str> =
            p.iter().flat_map(|(ph, _, _)| ph.split(' ')).collect();
        println!("target {t:?}: pool {} in {ms}ms, distinct words {}", p.len(), distinct_words.len());
        // canonical structure membership for the pairing target
        if t.starts_with("It's") {
            for st in [
                vec![3usize, 10, 13, 15, 19],
            ] {
                let members: Vec<&(String, Vec<usize>, f64)> =
                    p.iter().filter(|(_, c, _)| *c == st).collect();
                println!("  structure {st:?}: members {}", members.len());
                let mut ranked: Vec<&(String, Vec<usize>, f64)> = members.clone();
                ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
                for (i, m) in ranked.iter().enumerate().take(60) {
                    println!("   {:3} [{:.6}] {}", i, m.2, m.0);
                }
            }
        }
    }
}
