// SCRATCH-ONLY analysis: rank boundary structures by their best achievable
// score, using the whole deduplicated candidate pool (top_n huge bypasses
// select_diverse truncation).
use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
use std::collections::{BTreeMap, HashMap, HashSet};

fn main() {
    let target = std::env::args().nth(1).unwrap();
    let wanted = std::env::args().nth(2).unwrap_or_default().to_lowercase();

    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 1_000_000,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let clues = g.generate(&target);
    println!("pool size {}", clues.len());

    // A clue's structure signature is its sequence of clue-word IPA lengths.
    // Two clues with the same signature and same total cost have the same cuts.
    let mut best: HashMap<String, (f64, String)> = HashMap::new();
    for c in &clues {
        let sig = c
            .words
            .iter()
            .map(|w| w.ipa.chars().count().to_string())
            .collect::<Vec<_>>()
            .join("-");
        let e = best.entry(sig).or_insert((c.score, c.phrase.clone()));
        if c.score > e.0 {
            *e = (c.score, c.phrase.clone());
        }
    }
    let mut ranked: Vec<(String, f64, String)> = best
        .into_iter()
        .map(|(k, v)| (k, v.0, v.1))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("distinct structures {}", ranked.len());
    for (i, (sig, score, phrase)) in ranked.iter().enumerate().take(20) {
        println!("{:3}. [{:.4}] {:22} {}", i + 1, score, sig, phrase);
    }
    for i in [24, 49, 99, 199, 299, 399, 499] {
        if let Some(r) = ranked.get(i) {
            println!("structure rank {:3}: [{:.4}] {:22} {}", i + 1, r.1, r.0, r.2);
        }
    }

    // What is the position of the wanted phrase, and the best fill of its
    // own structure?
    if !wanted.is_empty() {
        let wc = clues
            .iter()
            .find(|c| c.phrase.to_lowercase() == wanted)
            .cloned();
        match wc {
            None => println!("wanted {wanted:?}: NOT GENERATED"),
            Some(c) => {
                let sig = c
                    .words
                    .iter()
                    .map(|w| w.ipa.chars().count().to_string())
                    .collect::<Vec<_>>()
                    .join("-");
                let rank = clues.iter().position(|x| x.phrase == c.phrase).unwrap();
                let srank = ranked.iter().position(|r| r.0 == sig);
                let sbest = ranked
                    .iter()
                    .find(|r| r.0 == sig)
                    .map(|r| (r.1, r.2.clone()));
                println!(
                    "wanted {wanted:?}: score={:.4} pool_rank={rank} sig={sig} structure_rank={srank:?} structure_best={sbest:?}",
                    c.score
                );
                // how many distinct structures lie above this phrase's score?
                let above: HashSet<&String> = ranked
                    .iter()
                    .filter(|r| r.1 > c.score)
                    .map(|r| &r.0)
                    .collect();
                println!("structures scoring above it: {}", above.len());
                let mut bands: BTreeMap<usize, usize> = BTreeMap::new();
                for r in &ranked {
                    *bands.entry((r.1 * 1000.0) as usize).or_default() += 1;
                }
                println!("score histogram (milli): {:?}", &bands.iter().collect::<Vec<_>>()[..8.min(bands.len())]);
            }
        }
    }
}
