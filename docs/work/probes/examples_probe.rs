use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = SearchMode::Approximate { per_word_budget: 0.5, total_budget: 1.5 };
    let config = GeneratorConfig { mode, top_n: 50, ..GeneratorConfig::default() };
    let g = Generator::from_json(CORPUS_JSON, config).unwrap();
    for t in &args {
        let start = std::time::Instant::now();
        let (sel, pool) = g.generate_with_pool(t);
        let e = start.elapsed();
        let cutoff = sel.last().map(|c| c.score).unwrap_or(0.0);
        println!("{t}\tpool={pool}\ttop={}\tcutoff={cutoff:.12}\t{:?}", sel.len(), e);
    }
    if std::env::var("MADGAB_PROBE_SLOTS").is_ok() { madgab::probe::dump_slots(); } else { madgab::probe::dump(); }
}
