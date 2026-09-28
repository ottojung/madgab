//! Probe: pristine pool size and rank-50 cutoff on the public API.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus");
    for (label, target) in [
        ("case1", "recognize speech"),
        ("case2", "It's just a stupid game"),
    ] {
        let started = std::time::Instant::now();
        let pool = g.generate_pool(target);
        let secs = started.elapsed().as_secs_f64();
        let mut scores: Vec<f64> = pool.iter().map(|c| c.score).collect();
        scores.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let cutoff = scores.get(49).copied().unwrap_or(f64::NAN);
        println!(
            "{label} pool={} rank50={cutoff:.12} secs={secs:.3}",
            pool.len()
        );
    }
}
