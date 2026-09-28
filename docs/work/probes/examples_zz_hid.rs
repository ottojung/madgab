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
    .unwrap();
    let t = "It's just a stupid game";
    for w in ["hits", "justice", "dupe", "hid", "came"] {
        let in_shortlist = g
            .span_shortlists(t, 0.5)
            .into_iter()
            .any(|c| c.word == w);
        let in_pool = g
            .generate_pool(t)
            .iter()
            .any(|c| c.words.iter().any(|x| x.word == w));
        println!("{w}: shortlist={in_shortlist} pool={in_pool}");
    }
}
