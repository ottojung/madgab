use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

#[test]
fn zz_probe() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 50,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let out = g.generate("It's just a stupid game");
    eprintln!("ZZOUT {:?}", out.iter().map(|c| c.phrase.to_lowercase()).collect::<Vec<_>>());
}
