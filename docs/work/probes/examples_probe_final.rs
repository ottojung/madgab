use std::collections::BTreeMap;
use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
const TARGETS: &[&str] = &[
    "recognize speech", "It's just a stupid game", "I love you", "a whole lot of trouble",
    "he was a big fat man", "what are you going to do", "some kind of wonderful thing",
    "she had a lot of money", "there is no way to know", "the cat sat on the mat",
    "if you want to go now", "they are going to be late", "one of those other people",
    "when the rain finally stopped", "you can do it yourself",
];
fn main() {
    let g = Generator::from_json(CORPUS_JSON, GeneratorConfig { mode: SearchMode::approximate(), top_n: 50, beam_width: 64, ..GeneratorConfig::default() }).unwrap();
    println!("{:<30} {:>7} {:>14} {:>26} {:>9} {:>8}", "target", "pool", "rank50", "visible n", "meanscore", "n-opt");
    for t in TARGETS {
        let pool = g.generate_pool(t);
        let rank50 = pool[49.min(pool.len()-1)].score;
        let vis = g.generate(t);
        let mut d: BTreeMap<usize,usize> = BTreeMap::new();
        for c in &vis { *d.entry(c.words.len()).or_default() += 1; }
        let mean: f64 = vis.iter().map(|c| c.score).sum::<f64>() / vis.len() as f64;
        let strata = vis.iter().fold(BTreeMap::<usize,Vec<f64>>::new(), |mut m,c| { m.entry(c.words.len()).or_default().push(c.score); m });
        let mut best = String::new();
        let mut bestc = 0f64;
        for (k, v) in &strata { if (*v).len() as f64 > bestc { bestc = (*v).len() as f64; best = format!("n={} max {:.4} count {}", k, v.iter().cloned().fold(f64::MIN, f64::max), v.len()); } }
        println!("{:<30} {:>7} {:>14.12} {:>26} {:>9.6} {}", t, pool.len(), rank50, d.iter().map(|(k,v)| format!("{k}:{v}")).collect::<Vec<_>>().join(" "), mean, best);
    }
    // canonical case 2 pool membership at defaults and at the test's settings
    for (label, top) in [("top50", 50usize), ("top20000", 20_000)] {
        let g2 = Generator::from_json(CORPUS_JSON, GeneratorConfig { mode: SearchMode::approximate(), top_n: top, ..GeneratorConfig::default() }).unwrap();
        let pool = g2.generate_pool("It's just a stupid game");
        let want = "hits justice dupe hid came";
        let hit = pool.iter().position(|c| c.phrase.to_lowercase() == want);
        println!("case2 {label}: pool={} canonical={}", pool.len(), match hit { Some(i) => format!("rank {} score {:.6}", i+1, pool[i].score), None => "missing".to_string() });
    }
}
