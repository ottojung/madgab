use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
fn main() {
    let g = Generator::from_json(CORPUS_JSON, GeneratorConfig { mode: SearchMode::approximate(), top_n: 20_000, ..GeneratorConfig::default() }).unwrap();
    for t in ["I love you", "she sells sea shells", "big spender", "taco cat", "play games with me now"] {
        let pool = g.generate(t);
        println!("== {} pool={}", t, pool.len());
        for c in &pool { println!("{}\t{:.9}", c.phrase.to_lowercase(), c.score); }
    }
}
