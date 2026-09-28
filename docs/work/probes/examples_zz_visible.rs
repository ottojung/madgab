use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let targets = [
        "It's just a stupid game",
        "recognize speech",
        "put it back on the shelf",
        "I love you",
        "big spender",
        "play games with me now",
    ];
    for t in targets {
        let t0 = Instant::now();
        let g = Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n: 50,
                ..GeneratorConfig::default()
            },
        )
        .unwrap();
        let clues = g.generate(t);
        let ms = t0.elapsed().as_millis();
        let mut by: std::collections::HashMap<Vec<usize>, usize> = std::collections::HashMap::new();
        for c in &clues {
            *by.entry(c.cuts.clone()).or_default() += 1;
        }
        let words: std::collections::HashSet<&str> =
            clues.iter().flat_map(|c| c.phrase.split(' ')).collect();
        println!(
            "{t:?}\tvisible {}\tstructures {}\tdistinct visible words {}\t{}ms",
            clues.len(),
            by.len(),
            words.len(),
            ms
        );
    }
}
