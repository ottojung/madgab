//! SCRATCH PROBE for w-4d1e93 (probe branch only; never landed).
//!
//! Sweeps the word-count parsimony axis in the scorer *and* all three
//! structural keys, and at each point records what the two pool/print
//! fences read: the alignment's pool membership, the per-word membership
//! the fence's second assertion makes, the golden-lock target's head
//! score, the weight-free invariants, and whether the pool is closed under
//! word-level recombination of its own printed proposals.
//!
//! The word sequence named here is the same fixture the `emit_coverage`
//! fence names; this file exists to measure it under a weight, and `src/`
//! still contains no mention of it.

use madgab::objective_probe::set_parsimony;
use madgab::{Clue, Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

fn default_approximate() -> GeneratorConfig {
    GeneratorConfig {
        mode: SearchMode::Approximate {
            per_word_budget: 0.5,
            total_budget: 1.5,
        },
        top_n: 50,
        ..GeneratorConfig::default()
    }
}

fn words_of(clue: &Clue) -> Vec<String> {
    clue.words.iter().map(|w| w.word.to_lowercase()).collect()
}

fn rank_in(clues: &[Clue], words: &[String]) -> Option<usize> {
    clues.iter().position(|c| words_of(c) == words).map(|i| i + 1)
}

fn vocab(clues: &[Clue]) -> std::collections::HashSet<String> {
    clues
        .iter()
        .flat_map(words_of)
        .collect::<std::collections::HashSet<_>>()
}

fn has_vocab(clues: &[Clue], word: &str) -> bool {
    clues.iter().any(|c| words_of(c).iter().any(|w| w == word))
}

#[test]
fn probe_parsimony_sweep() {
    let target = "It's just a stupid game";
    let alignment: Vec<String> = ["hits", "justice", "dupe", "hid", "came"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let lock_target = "I love you";
    let golden_head = "0.933655 isle uhh view";
    let golden_tail = "0.906023 isle of ooh";

    let g = Generator::from_json(CORPUS_JSON, default_approximate()).unwrap();
    for lambda in [0.0f64, 0.001, 0.005, 0.02, 0.05, 0.10, 0.15, 0.25, 0.4, 1.0] {
        set_parsimony(lambda);
        let pool = g.generate_pool(target);
        let (printed, pool_size) = g.generate_with_pool(target);
        let lg = Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n: 10,
                ..GeneratorConfig::default()
            },
        )
        .unwrap();
        let lock = lg.generate(lock_target);
        let lock_str: Vec<String> = lock
            .iter()
            .map(|c| format!("{:.6} {}", c.score, c.phrase))
            .collect();

        let printed_in_pool = printed
            .iter()
            .filter(|c| rank_in(&pool, &words_of(c)).is_some())
            .count();
        let pv = vocab(&pool);
        let printed_words_present = printed
            .iter()
            .flat_map(words_of)
            .filter(|w| pv.contains(w))
            .count();
        let printed_words_total: usize = printed.iter().map(|c| c.words.len()).sum();

        // Is the pool closed under recombining the words of its own
        // printed proposals?  Every one- versus two-printed-clue splice
        // is a word sequence drawn entirely from pool vocabulary.
        let mut splices = 0usize;
        let mut splices_absent = 0usize;
        for a in printed.iter().take(6) {
            for b in printed.iter().take(6) {
                let aw = words_of(a);
                let bw = words_of(b);
                for k in 1..aw.len().min(bw.len()) {
                    let mut w = aw[..k].to_vec();
                    w.extend_from_slice(&bw[k..]);
                    splices += 1;
                    if rank_in(&pool, &w).is_none() {
                        splices_absent += 1;
                    }
                }
            }
        }

        let descending = printed.windows(2).all(|w| w[0].score >= w[1].score);
        let mut phrases: Vec<String> =
            printed.iter().map(|c| c.phrase.to_lowercase()).collect();
        let uniq = {
            let n = phrases.len();
            phrases.sort();
            phrases.dedup();
            phrases.len() == n
        };

        println!(
            "lambda={lambda:<6} pool={pool_size:<6} printed={} pin={printed_in_pool}/{} \
             words_in_pool={printed_words_present}/{printed_words_total} vocab={} \
             align_pool={} hid_in_pool={} lock_head={} lock_green={} desc={descending} uniq={uniq} \
             splices_absent={splices_absent}/{splices}",
            printed.len(),
            lock_str.first().cloned().unwrap_or_default(),
            pv.len(),
            rank_in(&pool, &alignment)
                .map(|r| format!("rank {r}"))
                .unwrap_or_else(|| "ABSENT".into()),
            has_vocab(&pool, "hid"),
            lock_str
                .first()
                .map(String::as_str)
                .unwrap_or("")
                .to_string(),
            lock_str.first().map(String::as_str) == Some(golden_head)
                && lock_str.last().map(String::as_str) == Some(golden_tail),
        );
    }
    set_parsimony(0.0);
}
