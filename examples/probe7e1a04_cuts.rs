//! PROBE7E1A04: target transcriptions, and the Jaccard boundary novelty of
//! an explicitly supplied cut vector.  Scratch branch only.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn novelty(cuts: &[usize], target_boundaries: &[usize], total_len: usize) -> f64 {
    let a: Vec<usize> = cuts.iter().copied().filter(|&c| c < total_len).collect();
    let b: Vec<usize> = target_boundaries
        .iter()
        .copied()
        .filter(|&x| x < total_len)
        .collect();
    let mut shared = 0usize;
    let mut union = 0usize;
    let (mut i, mut j) = (0usize, 0usize);
    while i < a.len() && j < b.len() {
        union += 1;
        if a[i] == b[j] {
            shared += 1;
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1;
        } else {
            j += 1;
        }
    }
    union += a.len() - i + b.len() - j;
    if union == 0 {
        return 0.0;
    }
    1.0 - shared as f64 / union as f64
}

fn main() {
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
    for t in [
        "It's just a stupid game",
        "recognize speech",
        "I love you",
        "a whole lot of trouble",
    ] {
        let (ipa, b, syl) = g.probe_transcribe(t).unwrap();
        println!("T\t{t}\t{ipa}\t{}\t{b:?}\t{syl}", ipa.chars().count());
    }
    let (ipa, b, _) = g.probe_transcribe("It's just a stupid game").unwrap();
    let n = ipa.chars().count();
    for cuts in [
        vec![3usize, 10, 13, 15, 19],
        vec![4usize, 10, 13, 15, 19],
    ] {
        println!("C\t{cuts:?}\t{:.12}", novelty(&cuts, &b, n));
    }
}
