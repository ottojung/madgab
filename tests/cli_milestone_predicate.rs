//! The itinerary's primary milestone predicate, measured at the
//! **executable** boundary: what `madgab --approximate` actually prints
//! for the two canonical inputs, as a user runs it.
//!
//! ## Why this file exists
//!
//! Every load-bearing number in the case-2 record was taken from
//! `#[cfg(test)]` harnesses, `examples/zz-probe-*.rs` and library-level
//! captures. The milestone, however, is stated about the *shipped
//! binary's* proposal set, so the predicate had never been measured
//! there. This file pins the measured answer at the boundary the
//! milestone is actually written about, following the established
//! canonical-example convention of naming the canonical strings (as
//! `tests/corpus_integration.rs` and `tests/approx_determinism.rs`
//! already do) and the established executable-boundary pattern of
//! driving `CARGO_BIN_EXE_madgab` as a subprocess
//! (`tests/approx_determinism.rs`, `tests/exact_determinism.rs`).
//!
//! ## What the measurement found, and what these tests therefore assert
//!
//! * **`wreck a nice beach` for `recognize speech`**: present, at
//!   display rank **27 of 50**, score 0.920 (0.9199502875218423 full).
//!   The binary does not print a pool rank, only the display index, so
//!   the display index is the only user-visible coordinate. It is
//!   **not** present at the shipped default `--top 10`, and is not
//!   present at any `--top` below 27.
//! * **`Hits Justice Dupe Hid Came` for `It's just a stupid game`**:
//!   **absent** at every documented public knob measured (see
//!   `docs/work/REPORT-8f0b3d.md`). In the `--top 1000` default-budget run,
//!   the earliest row carrying two of its five words is
//!   `it justice too bad came` at display rank 39, and `hid` is never
//!   emitted in any form. This is the known base red
//!   `approximate_finds_classic_madgab_resegmentation`, and
//!   `OBSTRUCTION-MAP.md` §3 closes case-2 reach as a search-side
//!   question.
//!
//! So one test asserts the **green** case-1 fact at the executable
//! boundary, one test records the **default-`top_n` gap** that the
//! milestone statement has to be read with, and one test **names the
//! case-2 gap** rather than asserting success. None of them hard-codes
//! a phrase into production behaviour: `no_phrase_hard_coding` scans
//! `src/`, `web/` and `examples/`, and this file is a canonical-example
//! suite.
//!
//! ## These are honest-current-behavior assertions
//!
//! The case-2 test is `#[ignore]`d rather than green-on-purpose: it is
//! the same known gap `corpus_integration` carries as a red, and
//! `OBSTRUCTION-MAP.md` §4 says do not re-pin it and do not let a change
//! turn it green by accident. `#[ignore]`d here means the assertion is
//! the *desired* state, is documented, and cannot fire by accident; the
//! executable-boundary **fact** that it is not currently met is asserted
//! by the non-ignored test beside it, so the record is complete either
//! way.

use std::process::Command;

/// The itinerary's canonical case-1 target/clue pair.
const CASE1_TARGET: &str = "recognize speech";
const CASE1_CLUE: &str = "wreck a nice beach";

/// The itinerary's canonical case-2 target/clue pair. The known gap.
const CASE2_TARGET: &str = "It's just a stupid game";
const CASE2_CLUE: &str = "Hits Justice Dupe Hid Came";

/// The display rank `wreck a nice beach` holds for `recognize speech`
/// under `--approximate --top 50` on the shipped binary, and therefore
/// the rank any future change must not push it past (the standing green
/// fact in `OBSTRUCTION-MAP.md` §4).
const CASE1_RANK: usize = 27;

/// The shipped default `--top`, i.e. what a user gets with no `--top`
/// flag at all.
const DEFAULT_TOP: usize = 10;

/// One approximate-mode CLI run, as `(display_position, phrase)` in
/// display order, 1-based.
///
/// Scores are dropped on purpose: a legitimate re-scoring must not fail
/// a membership/rank test. The header and timing lines the binary writes
/// to stderr never reach stdout, so nothing else has to be filtered.
fn approximate_run(target: &str, top_n: Option<usize>) -> Vec<(usize, String)> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    cmd.args(["--approximate"]);
    if let Some(n) = top_n {
        cmd.args(["--top", &n.to_string()]);
    }
    let out = cmd
        .arg(target)
        .output()
        .expect("the madgab binary should be runnable");
    assert!(
        out.status.success(),
        "madgab exited with {:?} for {target:?} at --top {top_n:?}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut proposals = Vec::new();
    for line in stdout.lines() {
        let rest = line.trim_start();
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            continue;
        }
        let rest = rest[digits.len()..].trim_start();
        let rest = rest.strip_prefix(". ").expect("a numbered proposal");
        let (_, phrase) = rest.split_once(']').expect("a scored proposal");
        proposals.push((digits.parse::<usize>().expect("a display index"), phrase.trim().to_string()));
    }
    assert!(
        !proposals.is_empty(),
        "approximate run produced no proposals for {target:?} at --top {top_n:?}"
    );
    proposals
}

/// Normalize one printed clue the way the milestone statement's
/// "normalized" clause asks for: case-insensitive, punctuation
/// stripped, word order **as given**.
///
/// Word order is deliberately *not* permuted. The itinerary's
/// normalized predicate is "case and punctuation insensitive, word order
/// as given", so permuting would widen the assertion past the predicate.
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Display rank of `clue` in one run, matching verbatim then normalized.
///
/// Returns the 1-based display position, or `None` if absent. Verbatim
/// is tried first so the reported rank is the rank of an exact display
/// when one exists.
fn rank_of(proposals: &[(usize, String)], clue: &str) -> Option<usize> {
    let needle = normalize(clue);
    proposals
        .iter()
        .find(|(_, p)| p == clue)
        .or_else(|| proposals.iter().find(|(_, p)| normalize(p) == needle))
        .map(|(rank, _)| *rank)
}

/// The green fact, at the executable boundary: the canonical case-1
/// clue is printed for `recognize speech` at `--top 50`, at or better
/// than its standing display rank 27.
#[test]
fn canonical_case_one_is_displayed_at_or_better_than_its_standing_rank() {
    let proposals = approximate_run(CASE1_TARGET, Some(50));
    let rank = rank_of(&proposals, CASE1_CLUE).unwrap_or_else(|| {
        panic!(
            "canonical case-1 clue {CASE1_CLUE:?} is not displayed for {CASE1_TARGET:?} at --top 50; \
             the green fact in OBSTRUCTION-MAP.md §4 is a loss. Top 10 shown: {:?}",
            &proposals[..proposals.len().min(10)]
        )
    });
    assert!(
        rank <= CASE1_RANK,
        "canonical case-1 clue regressed: display rank {rank} > {CASE1_RANK} at --top 50"
    );
}

/// The shipped default is `--top 10`, and at that default the canonical
/// case-1 clue is **not** printed. This is recorded as current behavior
/// so the milestone statement cannot be read as claiming otherwise: the
/// predicate holds at `--top 50`, not at the out-of-the-box default.
///
/// The assertion is on the *absence* at the default plus the presence
/// once the width is raised, so it fails if the default is ever widened
/// to cover rank 27, which would be a *change* in user-visible behavior
/// a later pass should make deliberately.
#[test]
fn shipped_default_top_n_does_not_display_the_canonical_case_one() {
    let at_default = approximate_run(CASE1_TARGET, None);
    assert_eq!(
        at_default.len(),
        DEFAULT_TOP,
        "the shipped default --top should still be {DEFAULT_TOP}"
    );
    assert!(
        rank_of(&at_default, CASE1_CLUE).is_none(),
        "canonical case-1 clue now appears at the shipped default --top {DEFAULT_TOP}; \
         that is a real user-visible change and this test should be re-read before it is accepted"
    );
    assert!(
        rank_of(&approximate_run(CASE1_TARGET, Some(50)), CASE1_CLUE).is_some(),
        "canonical case-1 clue must be reachable once --top is raised to 50"
    );
}

/// The known gap, named rather than asserted as success. `#[ignore]`d
/// so it cannot fire by accident; run with `--ignored` to check it.
#[test]
#[ignore = "known base red: approximate_finds_classic_madgab_resegmentation; \
            case-2 reach is closed as a search-side question (OBSTRUCTION-MAP.md §3)"]
fn canonical_case_two_is_displayed() {
    for top in [10usize, 50, 200, 1000] {
        let proposals = approximate_run(CASE2_TARGET, Some(top));
        assert!(
            rank_of(&proposals, CASE2_CLUE).is_some(),
            "canonical case-2 clue {CASE2_CLUE:?} not displayed for {CASE2_TARGET:?} at --top {top} \
             (verbatim and normalized); nearest: {:?}",
            proposals
                .iter()
                .find(|(_, p)| normalize(p).contains("justice"))
                .map(|(_, p)| p.as_str())
                .unwrap_or("<none>")
        );
    }
}

/// The executable-boundary **fact** behind the gap above: across every
/// documented public knob combination measured, the canonical case-2
/// clue is not printed. This is a non-ignored assertion, so the record
/// of the gap is not `#[ignore]`d into invisibility.
#[test]
fn canonical_case_two_is_absent_across_the_documented_public_knobs() {
    // --approximate --top {10,50,200,1000} at the default budgets, and
    // the two budget knobs raised. Only documented CLI flags, per
    // src/main.rs USAGE; there is no seed or determinism flag.
    let runs: Vec<Vec<String>> = [
        vec!["--approximate", "--top", "10"],
        vec!["--approximate", "--top", "50"],
        vec!["--approximate", "--top", "200"],
        vec!["--approximate", "--top", "1000"],
        vec!["--approximate", "--top", "1000", "--per-word-budget", "1"],
        vec!["--approximate", "--top", "1000", "--per-word-budget", "3"],
        vec!["--approximate", "--top", "1000", "--total-budget", "3"],
    ]
    .iter()
    .map(|args| {
        let out = Command::new(env!("CARGO_BIN_EXE_madgab"))
            .args(args)
            .arg(CASE2_TARGET)
            .output()
            .expect("the madgab binary should be runnable");
        assert!(
            out.status.success(),
            "madgab exited with {:?} for {CASE2_TARGET:?} at {args:?}",
            out.status.code()
        );
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split_once(']').map(|(_, p)| p.trim().to_string()))
            .collect()
    })
    .collect();

    let needle = normalize(CASE2_CLUE);
    for (args, proposals) in [
        ("--approximate --top 10", &runs[0]),
        ("--approximate --top 50", &runs[1]),
        ("--approximate --top 200", &runs[2]),
        ("--approximate --top 1000", &runs[3]),
        ("--top 1000 --per-word-budget 1", &runs[4]),
        ("--top 1000 --per-word-budget 3", &runs[5]),
        ("--top 1000 --total-budget 3", &runs[6]),
    ] {
        assert!(
            !proposals.iter().any(|p| normalize(p) == needle),
            "canonical case-2 clue {CASE2_CLUE:?} is now displayed at {args}; \
             this known gap closed, and the corresponding ignored test should be enabled"
        );
    }
}
