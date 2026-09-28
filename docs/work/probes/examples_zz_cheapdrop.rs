use std::collections::{BTreeMap, HashSet};

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGETS: &[&str] = &[
    "It's just a stupid game",
    "recognize speech",
    "I love you",
    "taco cat",
    "a whole lot of trouble",
    "what are you going to do",
    "some kind of wonderful thing",
    "the cat sat on the mat",
    "in the middle of the night",
    "my brother has a red car",
    "can you hear me now",
    "we should have told her",
];

fn main() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let b = 0.5f64;
    let (mut te, mut tc, mut td) = (0usize, 0usize, 0usize);
    let mut worst = (0usize, 0usize);
    for target in TARGETS {
        let mut spans: BTreeMap<(usize, usize), Vec<madgab::SpanCandidate>> =
            BTreeMap::new();
        for c in g.span_candidates(target, b) {
            spans.entry((c.start, c.end)).or_default().push(c);
        }
        let mut kept: BTreeMap<(usize, usize), HashSet<String>> = BTreeMap::new();
        for c in g.span_shortlists(target, b) {
            kept.entry((c.start, c.end)).or_default().insert(c.word);
        }
        let (mut e, mut c_, mut d) = (0usize, 0usize, 0usize);
        for (edge, cand) in &spans {
            let cheapest = cand
                .iter()
                .map(|x| x.cost)
                .fold(f64::INFINITY, f64::min);
            let ret = kept.get(edge).unwrap();
            let mut cheap = 0usize;
            let mut dropped = 0usize;
            for x in cand {
                if x.cost <= cheapest * 1.5 + 1e-9 {
                    cheap += 1;
                    if !ret.contains(&x.word) {
                        dropped += 1;
                    }
                }
            }
            e += 1;
            c_ += cheap;
            d += dropped;
            if worst.1 == 0 || dropped * worst.1 > worst.0 * cheap {
                worst = (dropped, cheap);
            }
        }
        println!("PROBE {target:?} edges={e} cheap={c_} dropped={d}");
        te += e;
        tc += c_;
        td += d;
    }
    println!("PROBE TOTAL edges={te} cheap={tc} dropped={td} worst={}/{}", worst.0, worst.1);
}
