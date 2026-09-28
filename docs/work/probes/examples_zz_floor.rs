use std::collections::BTreeMap;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let b = 0.5f64;
    let mut lens: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for c in g.span_shortlists("taco cat", b) {
        *lens.entry((c.start, c.end)).or_default() += 1;
    }
    let mut clens: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for c in g.span_candidates("taco cat", b) {
        *clens.entry((c.start, c.end)).or_default() += 1;
    }
    let mut full = 0;
    let mut total = 0;
    for (e, n) in &lens {
        if *n < 160 {
            println!("span {e:?} retained {n} of {} offered", clens[e]);
        } else {
            full += 1;
        }
        total += 1;
    }
    println!("{full} of {total} spans reach 160");
}
