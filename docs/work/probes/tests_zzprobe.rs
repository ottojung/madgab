use std::collections::HashMap;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn pool(target: &str) -> Vec<madgab::Clue> {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 20_000,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    g.generate(target)
}

#[test]
fn zzprobe() {
    let mut out = String::new();
    for target in [
        "It's just a stupid game",
        "recognize speech",
        "taco cat",
        "the cat sat on the mat",
        "a whole lot of trouble",
        "when the rain finally stopped",
        "sign on",
        "big spender",
    ] {
        let clues = pool(target);
        let hit = clues
            .iter()
            .filter(|c| c.phrase.to_lowercase() == "hits justice dupe hid came")
            .count();
        // group by cuts
        let mut by_cuts: HashMap<Vec<usize>, Vec<Vec<String>>> = HashMap::new();
        for c in &clues {
            let words: Vec<String> = c
                .phrase
                .split(' ')
                .map(|w| w.to_lowercase())
                .collect();
            by_cuts
                .entry(c.cuts[..c.cuts.len() - 1].to_vec())
                .or_default()
                .push(words);
        }
        // per structure: modal word per slot
        let mut best_nonmodal = 0usize;
        let mut count_ge: HashMap<usize, usize> = HashMap::new();
        let mut best_example = String::new();
        for (cuts, members) in &by_cuts {
            if members.len() < 4 {
                continue;
            }
            let n = members[0].len();
            let mut modal: Vec<String> = vec![String::new(); n];
            for s in 0..n {
                let mut m: HashMap<&String, usize> = HashMap::new();
                for words in members {
                    *m.entry(&words[s]).or_default() += 1;
                }
                let mut best = (0usize, "");
                for (w, c) in &m {
                    if *c > best.0 {
                        best = (*c, w.as_str());
                    }
                }
                modal[s] = best.1.to_string();
            }
            for words in members {
                let d = (0..n)
                    .filter(|&s| words[s] != modal[s])
                    .count();
                *count_ge.entry(d).or_default() += 1;
                if d > best_nonmodal {
                    best_nonmodal = d;
                    best_example = words.join(" ");
                }
            }
        }
        let mut hist: Vec<String> = count_ge
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();
        hist.sort();
        out.push_str(&format!(
            "{target:?} pool={} membership={} structures={} max_nonmodal={} example={:?} hist=[{}]\n",
            clues.len(),
            hit,
            by_cuts.len(),
            best_nonmodal,
            best_example,
            hist.join(" ")
        ));
    }
    std::fs::write("/tmp/opencode/1c3e77/pool.txt", out).unwrap();
}
