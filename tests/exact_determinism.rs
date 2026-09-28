//! Exact-mode determinism, tested where the defect actually lives.
//!
//! `SearchMode::Exact` used to return a different clue set from one
//! process to the next. The cause was the corpus trie: `Corpus::from_json`
//! in phonetics-rs 0.3.1 inserts pronunciations while iterating a
//! `HashMap`, so the terminations sharing a trie node arrive in the
//! process's hash order, and `insert_top_k` keeps the first arrival on an
//! equal-`cheap_score` tie. Which hypothesis survives the beam therefore
//! depended on the process hash seed.
//!
//! That is a property of the *process*, not of any one call, so it cannot
//! be reproduced inside a single process — the honest boundary for this
//! regression is a real subprocess per run. Every run gets a fresh hash
//! seed, and the outputs must be byte-identical.
//!
//! Only stdout is compared: stderr carries the corpus-load and search
//! timings, which are wall-clock and necessarily vary.

use std::process::Command;

const RUNS: usize = 8;
const TARGET: &str = "I love you";

/// stdout of one exact-mode CLI run, as raw bytes.
fn exact_run() -> Vec<u8> {
    let out = Command::new(env!("CARGO_BIN_EXE_madgab"))
        .args(["--top", "20", TARGET])
        .output()
        .expect("the madgab binary should be runnable");
    assert!(
        out.status.success(),
        "madgab exited with {:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.stdout.is_empty(),
        "exact-mode run produced no clues for {TARGET:?}"
    );
    out.stdout
}

/// Repeated exact-mode invocations must be byte-identical.
///
/// Each run is a separate process, so each gets a fresh hash seed. Before
/// the fix this target produced two different outputs across eight runs
/// (e.g. a `[0.915]` clue present in some runs and absent in others).
#[test]
fn exact_mode_is_reproducible_across_processes() {
    let first = exact_run();
    for run in 2..=RUNS {
        let again = exact_run();
        assert_eq!(
            String::from_utf8_lossy(&again),
            String::from_utf8_lossy(&first),
            "exact-mode output for {TARGET:?} differed on run {run} of {RUNS}"
        );
    }
}
