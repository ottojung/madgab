//! The executable-level pool-rank query: `--pool-rank` placed **after** the
//! target phrase, naming one clue.
//!
//! ## The distinction this file exists to hold
//!
//! Two facts about a missing clue are routinely conflated in this project's
//! records (`docs/work/OBSTRUCTION-MAP.md` §"Conflicts", item 7):
//!
//! * **absent from the pool** — the search never built the alignment, and no
//!   objective change can ever surface it; and
//! * **built but outranked** — it is in the pool at some rank, and a display
//!   or objective change could move it.
//!
//! They have opposite causes and opposite fixes, and every one of them is
//! about *emission*, not about display. The shipped binary could show neither,
//! because the pool's contents were unreachable from outside the crate until
//! `generate_pool` existed and the CLI grew a surface over it. So this file
//! pins the three outcomes as **distinct observable facts**:
//!
//! | outcome            | exit | stdout/stderr says |
//! |--------------------|------|--------------------|
//! | rank + displayed   | 0    | `pool rank N of M; displayed at D` |
//! | rank, not displayed| 0    | `pool rank N of M; NOT in the display (generated, then ranked out ...)` |
//! | absent from pool   | **3**| `ABSENT from the pool: not generated at all ...` |
//!
//! Exit 3 is the load-bearing part. A caller that reads a non-zero exit as
//! "the run failed" is wrong for absence, and this repository has repeatedly
//! read it that way; making absence a distinct code makes that reading a
//! crash rather than a reading. Failure (1) and usage error (2) keep their
//! meanings.
//!
//! Like `tests/corpus_integration.rs` and `tests/pool_rank_reporting.rs`, this
//! is a canonical-example suite and so names the canonical strings; it adds
//! nothing to `src/`'s allowlist. The flag has no default value, so no
//! canonical clue is a literal anywhere in `src/`.

use std::process::{Command, Output};

/// Itinerary canonical case-1 target/clue pair.
const CASE1_TARGET: &str = "recognize speech";
const CASE1_CLUE: &str = "wreck a nice beach";

/// Itinerary canonical case-2 target.
const CASE2_TARGET: &str = "It's just a stupid game";

/// A clue that is in no pool this search can build: a real English phrase
/// whose phonemes do not reach either canonical target. Chosen so the
/// absent branch is exercised by a *negative* fact about the search rather
/// than by a typo in the query.
const NOT_A_CLUE: &str = "wreck a nice bogus";

/// The pool-rank query line, whether it landed on stdout (a hit) or stderr
/// (an absence). Absence goes to stderr so that stdout stays the numbered
/// proposal list, which is what every other consumer parses.
fn verdict_line(out: &Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    stdout
        .lines()
        .chain(stderr.lines())
        .find(|l| l.starts_with("pool-rank:"))
        .unwrap_or_else(|| {
            panic!(
                "no pool-rank verdict line.\nstdout:\n{stdout}\nstderr:\n{stderr}"
            )
        })
        .to_string()
}

fn run(target: &str, clue: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    cmd.args(["--approximate", "--top", "50"]);
    if let Some(clue) = clue {
        // The documented position: after the target phrase, so the flag is a
        // query about one clue rather than an annotation of every row.
        cmd.arg(target).arg("--pool-rank").arg(clue);
    } else {
        cmd.arg(target);
    }
    cmd.output()
        .expect("the madgab binary should be runnable")
}

/// The pool size the run reported on its stderr pool line. Every rank and
/// every absence below is stated against it, so it is read once, here, rather
/// than re-parsed inline at each assertion.
fn pool_size_of(out: &Output) -> usize {
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .find_map(|l| l.trim_start().strip_prefix("(pool: "))
        .and_then(|rest: &str| rest.split(' ').next())
        .and_then(|n: &str| n.parse().ok())
        .expect("a pool size on the pool line")
}

/// The pool rank a verdict line names, if it names one.
fn rank_of(line: &str) -> Option<String> {
    line.split("is pool rank ")
        .nth(1)
        .and_then(|rest: &str| rest.split(' ').next())
        .map(str::to_string)
}

/// The canonical clue is in the pool, and the report names a rank inside it
/// and whether it also made the display. Both coordinates are named, so
/// neither can be read as the other.
#[test]
fn a_clue_in_the_pool_is_reported_with_its_rank() {
    let out = run(CASE1_TARGET, Some(CASE1_CLUE));
    assert!(
        out.status.success(),
        "a pool hit must exit 0; got {:?}. stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let line = verdict_line(&out);

    let pool_size = pool_size_of(&out);

    let rank: usize = rank_of(&line)
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("no pool rank in {line:?}"));
    assert!(rank >= 1, "pool rank must be 1-based, got {rank} in {line:?}");
    assert!(
        rank <= pool_size,
        "reported rank {rank} exceeds the pool size {pool_size}: {line:?}"
    );
    assert!(
        line.contains(&format!("of {pool_size}")),
        "the report must state the pool size it ranks within: {line:?}"
    );

    // The display position, when there is one, is stated as such; when there
    // is not, the line says so in words rather than staying silent.
    if line.contains("displayed at ") {
        assert!(
            !line.contains("NOT in the display"),
            "a row cannot be both displayed and not displayed: {line:?}"
        );
    } else {
        assert!(
            line.contains("NOT in the display"),
            "a pool hit that did not reach the display must say so explicitly: {line:?}"
        );
    }
}

/// The absent case. The assertions are about *distinctness*, not about the
/// pool: whatever the pool is, absence is a different observable fact from
/// failure, and the two must never share an exit code or a message shape.
#[test]
fn a_clue_outside_the_pool_is_reported_as_absent_and_not_as_a_failure() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        let out = run(target, Some(NOT_A_CLUE));

        assert_eq!(
            out.status.code(),
            Some(3),
            "absence must be its own exit code for {target:?}, distinct from \
             success (0), failure (1) and usage error (2). stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !out.status.success(),
            "absence is deliberately not a success exit, so a caller cannot \
             mistake it for a hit"
        );

        let line = verdict_line(&out);
        assert!(
            line.contains("ABSENT from the pool"),
            "the verdict must say ABSENT in those words for {target:?}: {line:?}"
        );
        assert!(
            line.contains("not generated"),
            "absence is an emission fact and must be worded as one for \
             {target:?}: {line:?}"
        );
        assert!(
            !line.contains("is pool rank"),
            "an absent clue must not be given a rank for {target:?}: {line:?}"
        );
        // The pool size is stated, so "absent" is a claim about a pool of a
        // known size rather than an unbounded shrug.
        let pool_size = pool_size_of(&out);
        assert!(
            line.contains(&format!("of {pool_size} scored candidates")),
            "the absent report must state the pool it is absent from: {line:?}"
        );
    }
}

/// The two absences above are absences from a pool that *was* built. A run
/// that could not build a pool is a failure, and must not be reported as an
/// absence. This is the distinction the item asks for, pinned from the other
/// side: exit 1, not exit 3.
#[test]
fn a_run_that_cannot_build_a_pool_is_a_failure_not_an_absence() {
    // A target with no transcription at all never reaches a pool, so the run
    // is a *failure* (exit 1) and must not borrow exit 3 or the word ABSENT.
    // The exit code is the whole point of the test, so it is pinned against
    // the two neighbouring codes as well.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    let out = cmd
        .args(["--approximate", "zzzqqq"])
        .arg("--pool-rank")
        .arg("some clue")
        .output()
        .expect("the madgab binary should be runnable");
    assert_eq!(
        out.status.code(),
        Some(1),
        "a target that cannot be searched is a failure, not an absence. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("ABSENT"),
        "a failure must not be worded as an absence finding: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // A usage error is a third thing again: the trailing form needs a clue.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    let usage = cmd
        .args(["--approximate", "recognize speech"])
        .arg("--pool-rank")
        .output()
        .expect("the madgab binary should be runnable");
    assert_eq!(
        usage.status.code(),
        Some(2),
        "a trailing --pool-rank with no clue is a usage error, not an absence"
    );
    assert!(
        !String::from_utf8_lossy(&usage.stderr).contains("ABSENT"),
        "a usage error must not be worded as an absence finding"
    );
}

/// **Criterion 2, as a test.** The default invocation's stdout is unchanged:
/// no added line, no added column, no changed spacing. The query form is
/// opt-in, and the opt-in-ness has to be checkable at the executable, because
/// "the default path is untouched" is otherwise a claim in a commit message.
#[test]
fn the_default_invocation_stdout_is_unchanged_by_the_query_surface() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        let plain = run(target, None);
        assert!(plain.status.success());

        let stdout = String::from_utf8_lossy(&plain.stdout);
        let rows: Vec<&str> = stdout.lines().collect();
        assert_eq!(rows.len(), 50, "default run for {target:?} row count");

        for row in &rows {
            let head = row.split(". ").nth(1).expect("a numbered row");
            let (labels, phrase) = head.split_once(']').expect("a scored row");
            assert_eq!(
                labels.trim_start_matches('[').len(),
                5,
                "the default bracket must still hold the score alone, not a \
                 labelled field: {row:?}"
            );
            assert!(
                !phrase.trim().is_empty(),
                "default row for {target:?} has an empty phrase: {row:?}"
            );
        }
        assert!(
            !stdout.contains("pool rank"),
            "the default path must not put a pool rank in stdout for {target:?}"
        );
        assert!(
            !stdout.contains("pool-rank:"),
            "the default path must not print a query verdict for {target:?}"
        );
        assert!(
            !String::from_utf8_lossy(&plain.stderr).contains("second search"),
            "the default path must not pay for the second search: {}",
            String::from_utf8_lossy(&plain.stderr)
        );
    }
}

/// A query changes what is *reported*, never what is *produced*: the rows it
/// prints are the rows the default path prints, in the same order, byte for
/// byte. A query that filtered, re-sorted or annotated the list would make
/// its own rank figures describe a list the user did not ask for.
#[test]
fn the_query_prints_exactly_the_default_rows() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        // The verdict line is the one thing the query adds, and on a hit it
        // goes to stdout; strip it so this compares the *proposal rows* and
        // not the report about them.
        let rows = |o: Output| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.starts_with("pool-rank:"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let plain = rows(run(target, None));
        let queried = rows(run(target, Some(CASE1_CLUE)));
        assert_eq!(
            plain, queried,
            "the query form changed the proposal list for {target:?}; it is a \
             reporting flag"
        );
    }
}

/// The flag is a no-op on the exact mode. `--approximate` is what makes a pool
/// wide enough for the question to be interesting, but the flag itself must
/// not require it and must not change what exact mode produces: the rows come
/// out identical, and the report is a statement about the exact pool.
#[test]
fn the_query_is_a_no_op_on_the_exact_mode() {
    let mut plain_cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    let plain = plain_cmd
        .arg(CASE1_TARGET)
        .output()
        .expect("the madgab binary should be runnable");
    assert!(plain.status.success());

    let mut query_cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    let queried = query_cmd
        .arg(CASE1_TARGET)
        .arg("--pool-rank")
        .arg(CASE1_CLUE)
        .output()
        .expect("the madgab binary should be runnable");

    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        String::from_utf8_lossy(&queried.stdout),
        "the query form changed exact-mode stdout; it must be a no-op there"
    );

    // And it is genuinely a query on the exact pool, not a silent no-op: a
    // verdict line is always produced, whichever way it came out.
    let line = verdict_line(&queried);
    assert!(
        line.contains("is pool rank") || line.contains("ABSENT from the pool"),
        "the exact-mode query must answer the question it was asked: {line:?}"
    );
}

/// Case 2's clue is the known-red `approximate_finds_classic_madgab_resegmentation`
/// target, which this front must not relax, re-pin or skip. The query surface
/// is the instrument that makes its status *readable*, and the one thing it
/// must not do is paper over it: whatever the search does with that clue, the
/// binary says which of the two facts it is, and it says it from the pool
/// rather than from the printed list.
#[test]
fn case_two_is_reported_as_present_or_absent_never_as_a_guess() {
    let out = run(CASE2_TARGET, Some(CASE1_CLUE));
    // A foreign clue is absent in the case-2 pool; this asserts the surface
    // reports that as a finding rather than exiting non-zero-as-failure.
    assert_eq!(out.status.code(), Some(3));
    assert!(verdict_line(&out).contains("ABSENT from the pool"));
}

/// The flag's price is the second search, and the binary reports it. A
/// surface whose cost is only knowable by timing it from outside cannot be
/// weighed against the alternative of not asking the question, and "opt-in"
/// is only a real claim if the cost of opting in is visible.
#[test]
fn the_query_reports_the_second_search_it_pays_for() {
    let out = run(CASE1_TARGET, Some(CASE1_CLUE));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let line = stderr
        .lines()
        .find(|l| l.contains("second search"))
        .unwrap_or_else(|| panic!("no second-search cost reported: {stderr}"));
    let ms: u64 = line
        .split_whitespace()
        .last()
        .and_then(|n| n.trim_end_matches(')').trim_end_matches("ms").parse().ok())
        .unwrap_or_else(|| panic!("no millisecond figure in {line:?}"));
    assert!(ms < 60_000, "implausible second-search cost {ms}ms in {line:?}");

    // And the default path says nothing about it, which is the observable
    // form of "off by default".
    let plain = String::from_utf8_lossy(&run(CASE1_TARGET, None).stderr).into_owned();
    assert!(
        !plain.contains("second search"),
        "the default path must not report a second search it does not run: {plain}"
    );
}

/// Whitespace and case in a typed clue are a shell artefact, not a fact about
/// the search: `Wreck  A Nice Beach` asks the same question as the canonical
/// spelling. A surface that reported absence here would be measuring the
/// user's typing, and every absent-from-pool conclusion drawn through it would
/// be one keystroke deep.
#[test]
fn a_typed_clue_is_matched_on_normalized_words() {
    let canonical = run(CASE1_TARGET, Some(CASE1_CLUE));
    let messy = run(CASE1_TARGET, Some("  Wreck   A Nice   Beach  "));
    assert_eq!(canonical.status.code(), messy.status.code());
    let (a, b) = (verdict_line(&canonical), verdict_line(&messy));
    assert_eq!(
        rank_of(&a),
        rank_of(&b),
        "the same clue, differently typed, must get the same answer:\n{a}\n{b}"
    );
}
