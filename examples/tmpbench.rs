use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;
fn main() {
    let which = std::env::args().nth(1).unwrap_or_else(|| "classic".to_string());
    let targets: Vec<&str> = match which.as_str() {
        "speech" => vec!["recognize speech"],
        "short" => vec!["I love you"],
        "all" => vec![
            "It's just a stupid game",
            "recognize speech",
            "I love you",
            "hello there how was your day",
        ],
        _ => vec!["It's just a stupid game"],
    };
    for t in targets {
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
        let clues = g.generate(t);
        let dt = now.elapsed();
        println!("=== {t:?} n={} time={dt:?}", clues.len());
        for c in clues.iter().take(10) {
            println!("  {:.4} {}", c.score, c.phrase);
        }
    }
}
