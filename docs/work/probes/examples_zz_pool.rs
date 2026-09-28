use std::collections::{HashMap, HashSet};
use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args.get(0).cloned().unwrap_or_else(|| "/tmp/opencode/pool.tsv".into());
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    std::env::set_var("ZZ_POOL_OUT", &out);
    let clues = g.generate("It's just a stupid game");
    let _ = clues;
    let text = std::fs::read_to_string(&out).unwrap();
    let mut rows: Vec<(String, Vec<usize>, f64)> = Vec::new();
    for l in text.lines() {
        let mut it = l.split('\t');
        let p = it.next().unwrap().to_string();
        let c: Vec<usize> = it.next().unwrap().trim_matches(|ch| ch == '[' || ch == ']')
            .split(", ").filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap()).collect();
        let s: f64 = it.next().unwrap().parse().unwrap();
        rows.push((p, c, s));
    }
    println!("pool {}", rows.len());
    let mut by: HashMap<Vec<usize>, Vec<&(String, Vec<usize>, f64)>> = HashMap::new();
    for r in &rows { by.entry(r.1.clone()).or_default().push(r); }
    println!("distinct structures {}", by.len());
    let canon: Vec<usize> = vec![3, 10, 13, 15, 19];
    if let Some(m) = by.get(&canon) {
        println!("canonical [3,10,13,15,19] members {}", m.len());
        let mut r: Vec<&&(String, Vec<usize>, f64)> = m.iter().collect();
        r.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        for (i, x) in r.iter().enumerate() { println!("  {:3} [{:.9}] {}", i, x.2, x.0); }
        println!("  cutoff: {:?}", rows.get(49).map(|x| (x.2, x.0.clone())));
    } else {
        println!("canonical structure absent");
    }
    let words: HashSet<&str> = rows.iter().flat_map(|(p,_,_)| p.split(' ')).collect();
    println!("distinct words {}", words.len());
}
