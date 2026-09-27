// scratch probe for w-3c5b18; all inputs from argv/env, no phrase literals
use std::time::Instant;
use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let target = &args[1];
    let beam: usize = std::env::var("ZZ_BEAM").ok().and_then(|s| s.parse().ok()).unwrap_or(64);
    let top_n: usize = std::env::var("ZZ_TOPN").ok().and_then(|s| s.parse().ok()).unwrap_or(50);
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            beam_width: beam,
            top_n,
            max_rarity: None,
            mode: SearchMode::approximate(),
            min_word_ipa_chars: 1,
        },
    )
    .expect("corpus");
    let t = Instant::now();
    let pool = g.generate_pool(target);
    let el = t.elapsed();
    let needle: Vec<String> = std::env::var("ZZ_CLUE")
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    let mut rank = None;
    let mut deep = 0usize;
    for (i, c) in pool.iter().enumerate() {
        let words: Vec<String> = c.phrase.split_whitespace().map(|s| s.to_lowercase()).collect();
        let mut d = 0;
        let mut j = 0;
        for w in &words {
            if j < needle.len() && needle[j] == *w { d += 1; j += 1; }
        }
        if d > deep { deep = d; }
        if d == needle.len() && needle.len() > 0 && rank.is_none() { rank = Some(i + 1); }
    }
    println!(
        "beam={} top_n={} pool={} elapsed={:.2}s needle_pool_rank={:?} best_leading_run={} first10={:?}",
        beam, top_n, pool.len(), el.as_secs_f64(), rank, deep,
        pool.iter().take(10).map(|c| c.phrase.as_str()).collect::<Vec<_>>()
    );
}
