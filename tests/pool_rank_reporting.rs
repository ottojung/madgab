//! The CLI's two rank coordinates, tested at the **executable** boundary.
//!
//! ## Why this file exists
//!
//! `src/main.rs` prints a numbered proposal list, and until this front the
//! number in front of a row was the only rank coordinate the shipped binary
//! showed. That number is the **display position**: the row's 1-based place
//! in the printed list, *after* the search's selection policy has narrowed
//! the deduplicated candidate pool to `--top`. The **pool rank** is the row's
//! place in that pool, *before* the narrowing.
//!
//! The two are not the same number, and the gap between them is not small.
//! Measured on this binary at `--approximate --top 50`:
//!
//! | target                             | pool size | display 50's pool rank |
//! |------------------------------------|-----------|------------------------|
//! | `recognize speech`                 | 18 289    | 27  (the display is a prefix here) |
//! | `It's just a stupid game`          | 18 949    | 115 |
//! | `Coors light`                      | 12 956    | 1 346 |
//!
//! So "the binary shows rank 27" was ambiguous, and this project's own
//! records show the cost: a "display rank 27" figure, an older "26", and a
//! pool-rank figure from a `#[cfg(test)]` harness, all describing the same
//! canonical row. Every pool-content or pool-ordering change therefore had to
//! be reviewed from library internals, because the shipped artifact could not
//! show the coordinate those changes move.
//!
//! ## What is asserted here
//!
//! * the default path is unchanged in shape, and costs no extra search while
//!   now reporting the pool's size and the expansion factor;
//! * `--pool-rank` labels **both** coordinates on the row, so neither can be
//!   silently read as the other;
//! * the reported pool ranks are well-formed: in range, non-decreasing along
//!   display order, and never above the pool size;
//! * the canonical case-1 fact survives, re-measured through the new
//!   annotation rather than copied forward.
//!
//! The assertions on the *values* are deliberately range-and-order shaped
//! rather than exact. Exact pool sizes and exact pool ranks are properties of
//! the search and the ordering/selection policy, which belong to other fronts;
//! pinning them here would turn a reporting test into a tripwire on work this
//! front is explicitly not doing.
//!
//! Like `tests/cli_milestone_predicate.rs` and `tests/corpus_integration.rs`,
//! this file names the canonical strings. It is a canonical-example suite, so
//! `no_phrase_hard_coding` does not scan it, and it adds nothing to `src/`'s
//! allowlist.

use std::process::Command;

/// Itinerary canonical case-1 target/clue pair.
const CASE1_TARGET: &str = "recognize speech";
const CASE1_CLUE: &str = "wreck a nice beach";

/// Itinerary canonical case-2 target. Its clue is the known base red
/// `approximate_finds_classic_madgab_resegmentation`; this front does not
/// assert anything about it beyond the rank coordinates of the rows that *are*
/// printed, which is a search-side question, not a reporting one.
const CASE2_TARGET: &str = "It's just a stupid game";

/// The standing display rank the canonical case-1 clue holds in the shipped
/// binary. A change that pushed it past this is a real loss and must be made
/// deliberately, not by accident.
const CASE1_DISPLAY_RANK: usize = 27;

const TOP: usize = 50;

/// One CLI run's parsed result.
struct Run {
    /// `(display position, phrase)` in display order, 1-based, parsed the way
    /// the pre-existing parsers parse the **default** shape: the phrase is
    /// everything after the first `]`.
    default_rows: Vec<(usize, String)>,
    /// `(display position, pool rank)` as reported by `--pool-rank`.
    pool_ranks: Vec<(usize, usize)>,
    /// The pool size and displayed count parsed from the stderr pool line.
    pool_size: usize,
    displayed: usize,
}

fn raw_run(target: &str, pool_rank: bool) -> (String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_madgab"));
    cmd.args(["--approximate", "--top", &TOP.to_string()]);
    if pool_rank {
        cmd.arg("--pool-rank");
    }
    let out = cmd
        .arg(target)
        .output()
        .expect("the madgab binary should be runnable");
    assert!(
        out.status.success(),
        "madgab exited with {:?} for {target:?} (--pool-rank {pool_rank}); stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Run the binary twice, once in each mode, and parse both.
fn run(target: &str) -> Run {
    let (default_stdout, _default_stderr) = raw_run(target, false);
    let (annotated_stdout, annotated_stderr) = raw_run(target, true);

    // The pool line is stderr-only, so it cannot perturb stdout parsers.
    let pool_line = annotated_stderr
        .lines()
        .find(|l| l.trim_start().starts_with("(pool:"))
        .unwrap_or_else(|| panic!("no pool line on stderr for {target:?}: {annotated_stderr}"));
    let mut pool_size = None;
    let mut displayed = None;
    for field in pool_line.trim_matches(['(', ')']).split(',') {
        let field = field.trim();
        if let Some(v) = field.strip_prefix("pool: ").and_then(|v| v.split(' ').next()) {
            pool_size = v.parse().ok();
        } else if let Some((v, _)) = field.split_once(" displayed;") {
            displayed = v.trim().parse().ok();
        }
    }

    let default_rows = default_stdout
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start();
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if digits.is_empty() {
                return None;
            }
            let (_, phrase) = rest[digits.len()..].trim_start().strip_prefix(". ")?.split_once(']')?;
            Some((digits.parse().ok()?, phrase.trim().to_string()))
        })        .collect();

    let pool_ranks = annotated_stdout
        .lines()
        .filter_map(|line| {
            let digits: String = line
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            let display: usize = digits.parse().ok()?;
            let head = line.trim_start().split_once(". ")?.1;
            let (labels, _) = head.split_once(']')?;
            let rank = labels
                .split_once("pool rank ")?
                .1
                .split(' ')
                .next()?
                .parse()
                .ok()?;
            Some((display, rank))
        })
        .collect();

    Run {
        default_rows,
        pool_ranks,
        pool_size: pool_size.expect("a pool size on the pool line"),
        displayed: displayed.expect("a displayed count on the pool line"),
    }
}

/// The default path keeps the shape every existing consumer parses: a bare
/// score in the bracket, the phrase straight after the first `]`, and no
/// coordinate it might mistake for a rank. The pool's size and the expansion
/// factor are reported, on stderr, without a second search.
#[test]
fn default_output_shape_is_unchanged_and_reports_the_pool_size() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        let (stdout, stderr) = raw_run(target, false);

        assert_eq!(
            stdout.lines().count(),
            TOP,
            "default run for {target:?} should still print {TOP} rows"
        );
        for line in stdout.lines() {
            // The index stays right-aligned in a two-column field, so
            // single-digit rows are space-padded and two-digit rows are not.
            let index_len = line
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .count();
            assert!(
                index_len == 1 && line.starts_with(' ') || index_len == 2,
                "default row for {target:?} lost its two-column index: {line:?}"
            );
            let head = &line[line.find(". ").expect("a numbered row") + 2..];
            let (labels, phrase) = head.split_once(']').expect("a scored row");
            assert_eq!(
                labels.trim_start_matches('[').len(),
                5,
                "the default bracket should still hold only the score, not a labelled \
                 field; got {labels:?} for {target:?}"
            );
            assert!(
                !phrase.trim().is_empty(),
                "default row for {target:?} has an empty phrase: {line:?}"
            );
        }
        assert!(
            !stdout.contains("pool rank"),
            "the default path must not put a pool rank in stdout; it is opt-in via --pool-rank"
        );
        assert!(
            !stderr.contains("second search"),
            "the default path must not pay for a second search: {stderr}"
        );
        assert!(
            stderr.contains("(pool: "),
            "the default path should now report the pool size on stderr: {stderr}"
        );
    }
}

/// `--pool-rank` names both coordinates on the row, so no reader — and no
/// downstream parser — can take one for the other. The phrase still begins
/// immediately after the first `]`, so a default-shaped parser and an
/// annotated one agree on what each row says.
#[test]
fn pool_rank_flag_labels_score_and_pool_rank_on_the_same_row() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        let (stdout, _) = raw_run(target, true);
        let rows: Vec<&str> = stdout
            .lines()
            .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()))
            .collect();
        assert_eq!(rows.len(), TOP, "annotated run for {target:?} row count");

        for line in rows {
            let (_, rest) = line.split_once(". ").expect("a numbered row");
            let (labels, phrase) = rest.split_once(']').expect("a labelled row");
            assert!(
                labels.contains("score "),
                "annotated row for {target:?} does not label its score: {line:?}"
            );
            assert!(
                labels.contains("pool rank "),
                "annotated row for {target:?} does not label its pool rank: {line:?}"
            );
            assert!(
                !phrase.trim().is_empty(),
                "annotated row for {target:?} has an empty phrase: {line:?}"
            );
        }
    }
}

/// `--pool-rank` is a reporting flag and nothing else: the rows it prints
/// must be the same rows, in the same order, that the default path prints.
/// The pool's own contents come from a second search, so this is also the
/// check that the second search and the first agree — which is what makes
/// the reported ranks describe the displayed rows at all.
#[test]
fn the_flag_annotates_the_same_rows_the_default_path_prints() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        let r = run(target);
        let (annotated_stdout, _) = raw_run(target, true);
        let annotated: Vec<String> = annotated_stdout
            .lines()
            .filter_map(|line| {
                let rest = line.trim_start().split_once(". ")?.1;
                rest.split_once(']').map(|(_, phrase)| phrase.trim().to_string())
            })
            .collect();
        let default: Vec<String> = r.default_rows.iter().map(|(_, p)| p.clone()).collect();

        assert_eq!(
            default, annotated,
            "--pool-rank changed which rows are printed for {target:?}; it is a \
             reporting flag and must not affect results"
        );
        assert_eq!(
            r.pool_ranks
                .iter()
                .map(|(d, _)| *d)
                .collect::<Vec<usize>>(),
            (1..=TOP as usize).collect::<Vec<usize>>(),
            "display positions for {target:?} should be 1..={TOP} in order"
        );
    }
}

/// The two coordinates are structurally constrained in a way only a real
/// pool can satisfy: every reported pool rank is inside the pool, ranks never
/// go backwards along the displayed order, and the display is a selection of
/// the pool rather than a wider view of it.
#[test]
fn reported_pool_ranks_are_in_range_and_ordered() {
    for target in [CASE1_TARGET, CASE2_TARGET] {
        let r = run(target);
        assert_eq!(
            r.pool_ranks.len(),
            TOP,
            "annotated run for {target:?} row count"
        );
        assert_eq!(
            r.displayed, r.pool_ranks.len(),
            "the pool line's displayed count should match the rows printed for {target:?}"
        );
        assert!(
            r.pool_size >= r.displayed,
            "pool size {} is below the {} displayed for {target:?}",
            r.pool_size,
            r.displayed
        );

        let mut previous = 0;
        for (display, rank) in &r.pool_ranks {
            assert!(
                *rank <= r.pool_size,
                "display {display} of {target:?} reports pool rank {rank} above the pool size {}",
                r.pool_size
            );
            assert!(
                *rank >= previous,
                "pool ranks must not go backwards along display order for {target:?}: \
                 {previous} then {rank}"
            );
            previous = *rank;
        }
        assert!(
            r.pool_ranks.last().expect("rows").1 > r.pool_ranks[0].1,
            "expected the selected display to be a subset of a wider pool for {target:?}"
        );
    }
}

/// The canonical case-1 fact, re-measured through the new annotation rather
/// than copied forward: the clue is still printed for its target, at a
/// display position no worse than the standing 27, and now carries a pool
/// rank too.
#[test]
fn canonical_case_one_survives_the_reporting_change() {
    let stdout = raw_run(CASE1_TARGET, true).0;
    let row = stdout
        .lines()
        .find(|l| l.contains(CASE1_CLUE))
        .unwrap_or_else(|| panic!("canonical clue {CASE1_CLUE:?} not printed for {CASE1_TARGET:?}"));

    let digits: String = row
        .trim_start()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let display: usize = digits
        .parse()
        .unwrap_or_else(|_| panic!("no display position in {row:?}"));
    assert!(
        display <= CASE1_DISPLAY_RANK,
        "canonical case-1 clue regressed: display position {display} > {CASE1_DISPLAY_RANK}"
    );

    let rank: usize = row
        .split_once("pool rank ")
        .expect("an annotated row")
        .1
        .split(' ')
        .next()
        .expect("a number")
        .parse()
        .expect("a numeric pool rank");
    assert!(rank >= 1, "pool rank must be 1-based, got {rank}");
}
