use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
use std::collections::HashMap;

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
];

fn gen(top_n: usize) -> Generator {
    Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse")
}

fn structure(c: &madgab::Clue) -> Vec<usize> {
    let mut cuts = c.cuts.clone();
    cuts.pop();
    cuts
}

fn main() {
    let pool = gen(10).generate_pool("recognize speech");
    let disp = gen(10).generate("recognize speech");
    for (i, c) in disp.iter().enumerate() {
        let rank = pool.iter().position(|p| p.phrase == c.phrase).unwrap();
        println!("{:2}. {:.6} rank {:4} {:?} {}", i + 1, c.score, rank, structure(c), c.phrase);
    }
    let mut d: HashMap<Vec<usize>, usize> = HashMap::new();
    for c in &disp {
        *d.entry(structure(c)).or_insert(0) += 1;
    }
    println!(
        "recognize speech: distinct {} dom {} | canonical present: {}",
        d.len(),
        d.values().copied().max().unwrap(),
        disp.iter().any(|c| c.phrase == "wreck a nice beach")
    );

    let mut tot = 0f64;
    let mut shares = 0f64;
    for t in TARGETS {
        let disp = gen(10).generate(t);
        let mut d: HashMap<Vec<usize>, usize> = HashMap::new();
        for c in &disp {
            *d.entry(structure(c)).or_insert(0) += 1;
        }
        let dom = d.values().copied().max().unwrap();
        tot += d.len() as f64;
        shares += dom as f64 / 10.0;
        println!("{t:32} distinct {:2} dom {dom}/10", d.len());
    }
    println!(
        "MEAN distinct = {:.4}  MEAN dominant share = {:.4}",
        tot / TARGETS.len() as f64,
        shares / TARGETS.len() as f64
    );
}
