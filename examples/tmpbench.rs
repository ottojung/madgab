use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
fn main() {
    let t = std::env::args().nth(1).unwrap_or_else(|| "classic".to_string());
    let target = if t == "speech" {
        "recognize speech"
    } else {
        "It's just a stupid game"
    };
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let now = std::time::Instant::now();
    let clues = g.generate(target);
    let dt = now.elapsed();
    println!("=== {target:?} n={} time={dt:?}", clues.len());
    for c in clues.iter().take(5) {
        println!("  {:.4} {}", c.score, c.phrase);
    }
}
