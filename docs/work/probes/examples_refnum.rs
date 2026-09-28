use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
fn main() {
    let mode = SearchMode::Approximate { per_word_budget: 0.5, total_budget: 1.5 };
    let config = GeneratorConfig { mode, top_n: 50, ..GeneratorConfig::default() };
    let g = Generator::from_json(CORPUS_JSON, config).unwrap();
    for t in std::env::args().skip(1) {
        let start = std::time::Instant::now();
        let (sel, pool) = g.generate_with_pool(&t);
        let e = start.elapsed();
        let poolv = g.generate_pool(&t);
    println!("{}\tpool={}\tn={}\tpool50={:.12}\tsel50={:.12}\t{:?}", t, pool, sel.len(), poolv[49].score, sel.last().map(|c|c.score).unwrap_or(0.0), e);
    }
}
