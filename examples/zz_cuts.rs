use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let e = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig { mode: SearchMode::Exact, top_n: 50, ..Default::default() },
    ).unwrap();
    for t in ["It's just a stupid game", "recognize speech"] {
        println!("== {t:?}");
        for c in e.generate(t).iter().take(60) {
            println!("  {:?} cuts {:?} {:.6}", c.phrase, c.cuts, c.score);
        }
    }
}
