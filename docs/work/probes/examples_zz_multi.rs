use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let out = std::env::var("ZZ_POOL_DIR").unwrap_or_else(|_| "/tmp/opencode".into());
    let targets = [
        "It's just a stupid game",
        "recognize speech",
        "put it back on the shelf",
        "I love you",
        "big spender",
        "play games with me now",
    ];
    for (i, t) in targets.iter().enumerate() {
        let g = Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig { mode: SearchMode::approximate(), top_n: 50, ..GeneratorConfig::default() },
        ).unwrap();
        let clues = g.generate(t);
        let mut words = std::collections::HashSet::new();
        for c in &clues { for w in c.phrase.to_lowercase().split(' ') { words.insert(w.to_string()); } }
        println!("{i}\t{}\tpool {}\tdistinct words {}\tvisible {}", t, clues.len(), words.len(), clues.len());
    }
    let _ = out;
}
