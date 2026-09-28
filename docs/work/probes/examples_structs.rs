use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let target = std::env::args().nth(1).unwrap();
    let top: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: top,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let clues = g.generate(&target);
    let ipa = g.corpus().transcribe(&target).unwrap();
    let target_ipa: String = ipa.chars().filter(|&c| c != 'ˈ' && c != 'ˌ').collect();
    let chars: Vec<char> = target_ipa.chars().collect();
    let mut tb: Vec<usize> = Vec::new();
    let mut acc = 0;
    for w in target.split_whitespace() {
        let key: String = w
            .to_lowercase()
            .trim_end_matches(['.', ',', '!', '?', ';', ':'])
            .to_string();
        acc += g
            .corpus()
            .preferred_ipa(&key)
            .unwrap_or_default()
            .chars()
            .filter(|&c| c != 'ˈ' && c != 'ˌ')
            .count();
        tb.push(acc);
    }
    println!("target {target:?} /{target_ipa}/ len {} boundaries {tb:?}", chars.len());

    // Recover each clue's cut structure from the emitted word IPA lengths: the
    // matcher consumed target spans, so cuts are the running sum of clue word
    // ipa lengths only when there are no indels. Instead ask the generator for
    // the full unscored list by raising top_n and grouping identical prefixes.
    let mut structures: std::collections::HashMap<String, usize> = Default::default();
    for (i, c) in clues.iter().enumerate() {
        let key = c.words.iter().map(|w| w.ipa.chars().count().to_string()).collect::<Vec<_>>().join("-");
        *structures.entry(key).or_default() += 1;
        if i < 25 {
            println!("{:2}. [{:.4}] {}", i + 1, c.score, c.phrase);
        }
    }
    println!("distinct ipa-length signatures in top {}: {}", clues.len(), structures.len());
    for (k, v) in structures.iter().take(20) {
        println!("   {k:30} x{v}");
    }
}
