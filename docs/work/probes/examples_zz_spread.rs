use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn pool(target: &str) -> Vec<(String, Vec<usize>, f64)> {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
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
    let targets = [
        "It's just a stupid game",
        "recognize speech",
        "put it back on the shelf",
        "I love you",
        "big spender",
        "play games with me now",
        "the cat sat on the mat",
        "a whole lot of trouble",
        "when the rain finally stopped",
    ];
    for t in targets {
        let t0 = Instant::now();
        let p = pool(t);
        let ms = t0.elapsed().as_millis();
        let by: HashMap<Vec<usize>, Vec<&(String, Vec<usize>, f64)>> = {
            let mut m: HashMap<Vec<usize>, Vec<&(String, Vec<usize>, f64)>> = HashMap::new();
            for r in &p {
                m.entry(r.1.clone()).or_default().push(r);
            }
            m
        };
        // per-word membership, and per (word, slot-position) membership
        let mut word_slots: BTreeMap<String, BTreeMap<usize, usize>> = BTreeMap::new();
        for (phrase, _, _) in &p {
            for (i, w) in phrase.split(' ').enumerate() {
                *word_slots
                    .entry(w.to_string())
                    .or_default()
                    .entry(i)
                    .or_default() += 1;
            }
        }
        // how many distinct slot positions each well-populated word occupies
        let spread: Vec<(String, usize, usize)> = word_slots
            .iter()
            .filter(|(_, m)| m.values().sum::<usize>() >= 8)
            .map(|(w, m)| (w.clone(), m.values().sum::<usize>(), m.len()))
            .collect();
        println!("== {t:?} pool {} in {ms}ms structures {}", p.len(), by.len());
        let mut spread = spread;
        spread.sort_by_key(|x| std::cmp::Reverse(x.1));
        println!("   words(>=8): {}", spread.iter().map(|(w,n,s)| format!("{w}:{n}/{s}")).collect::<Vec<_>>().join(" "));
        if t.starts_with("It's") {
            for st in [vec![3usize, 10, 13, 15, 19]] {
                let m = by.get(&st).map(|v| v.len()).unwrap_or(0);
                println!("   canonical {st:?} members {m}");
            }
        }
        if t.starts_with("recognize") {
            println!("   wreck a nice beach present: {}", p.iter().any(|(ph,_,_)| ph == "wreck a nice beach"));
        }
        if t.starts_with("put it back") {
            for (w, n, s) in &spread {
                if w == "taught" {
                    println!("   taught: {n} occurrences across {s} slot positions");
                }
            }
        }
    }
}
