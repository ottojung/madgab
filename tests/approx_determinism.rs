//! Approximate-mode determinism and the `top_n` relationship, tested at
//! the boundaries a user can observe.
//!
//! Two fronts can silently break both properties while they are in
//! flight: the scoring front [w-04f83f](../docs/work/items/w-04f83f.md)
//! and the corpus-allocation front
//! [w-d17a62](../docs/work/items/w-d17a62.md).  Approximate mode is the
//! branch that reaches the answer through `HashMap`/`HashSet`
//! retention, a `HashMap`-built substitution table and a span shortlist
//! whose `HashSet` dedup feeds the beam, so it needs its own guard
//! rather than inheriting `tests/exact_determinism.rs`.
//!
//! ## Why run-to-run determinism is a *process* property
//!
//! `build_lexicon` parses the corpus JSON into a `HashMap<String,
//! RawEntry>` and then sorts `words` before building the trie, so the
//! trie itself is order-stable.  The `substitution_costs` table and the
//! `found` map in `FuzzyLexicon::matches_at` are only read by key or
//! sorted before use, so a differing `HashMap` seed should not reach the
//! answer.  That is an argument, not a guard: a future change that
//! iterates one of those maps into a list would reintroduce exactly the
//! bug [w-5d03af](../docs/work/items/w-5d03af.md) fixed for exact mode,
//! and it would not be visible to any in-process test.  Each run below
//! is therefore a real subprocess with a fresh hash seed, and only the
//! proposal lines are compared (the header carries wall-clock timings).
//!
//! ## Why membership, not prefix, is the `top_n` relationship
//!
//! `--top-n` is not a truncation of a fixed ranking.  It feeds the
//! retention budget: the completed-hypothesis pool is pruned to
//! `top_n * 128` clamped to `[1024, 8192]` (`final_keep` and the
//! per-level `keep` in the approximate search), and it feeds
//! `select_diverse`, where the representative step only looks inside
//! the cutoff `order[..top_n]` and the share cap is
//! `share_cap(top_n, available)` = `ceil(top_n / STRUCTURE_FLOOR)`.
//! A larger request therefore searches a larger pool *and* selects
//! under a different rule, so the prefix of the smaller list is not
//! promised and measurably does not hold: for "I love you", `--top 5`
//! and `--top 10` share all five phrases but `--top 10` inserts
//! `why a view` and `high a view` between `eye a view` and
//! `isle come view`, and ranks the shared `isle come view` /
//! `i'll come view` lower than the smaller list shows them.
//!
//! What does hold, and is what a user can rely on, is *membership
//! monotonicity*: every proposal shown at `--top n` is still shown at
//! `--top m` for `m > n`.  Both knobs move monotonically — pruning
//! keeps the best-scoring `K`, so a bigger budget yields a superset of
//! candidates, the representative step's cutoff grows, and the share
//! cap grows with it — so a bigger request can add proposals and
//! reorder them, but is not expected to drop one the smaller request
//! already paid for.  This is asserted below over four targets and
//! three `(n, m)` pairs each (the full measurement behind the choice
//! is in the work item's handoff).

use std::collections::HashSet;
use std::process::Command;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// Canonical and spread targets.  "I love you" and "recognize speech"
/// are the itinerary's canonical inputs; "It's just a stupid game" is a
/// full multi-word sentence; "taco cat" is a short target, so the test
/// is not silently a test of one long-phrase regime.
const TARGETS: &[&str] = &[
    "I love you",
    "It's just a stupid game",
    "recognize speech",
    "taco cat",
];

/// Small on purpose: the retention budget floors at 1024 regardless, so
/// a larger `top_n` costs search time without testing anything new.
const TOP_NS: &[usize] = &[3, 5];

/// The proposal lines of one approximate-mode CLI run.
///
/// Returns the phrase of each numbered proposal in display order; the
/// score is dropped on purpose so a legitimate re-scoring by the
/// scoring front does not fail this test.  Header and timing lines are
/// skipped for the same reason.
fn approximate_run(target: &str, top_n: usize) -> Vec<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_madgab"))
        .args(["--approximate", "--top", &top_n.to_string(), target])
        .output()
        .expect("the madgab binary should be runnable");
    assert!(
        out.status.success(),
        "madgab exited with {:?} for {target:?} at --top {top_n}; stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let proposals: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix(|c: char| c.is_ascii_digit())?;
            rest.trim_start()
                .strip_prefix(". ")?
                .split_once(']')
                .map(|(_, phrase)| phrase.trim().to_lowercase())
        })
        .collect();
    assert!(
        !proposals.is_empty(),
        "approximate run produced no proposals for {target:?} at --top {top_n}"
    );
    proposals
}

/// Same target, same options, fresh process: identical proposal list in
/// identical order.
///
/// A different hash seed per process is the point.  Within one process
/// the assertion would hold trivially.
#[test]
fn approximate_mode_is_reproducible_across_processes() {
    for &target in TARGETS {
        for &top_n in TOP_NS {
            let first = approximate_run(target, top_n);
            let again = approximate_run(target, top_n);
            assert_eq!(
                first, again,
                "approximate output for {target:?} at --top {top_n} differed \
                 between two runs of the same binary"
            );
        }
    }
}

/// A raised `--top` may add and reorder proposals, but must not retract
/// one the smaller request already showed.
///
/// Prefixed with the in-process variant deliberately: the interesting
/// invariant here is the relation *between* two `top_n` values, and one
/// `Generator` serves both, so the corpus load and the fuzzy-lexicon
/// build (the expensive part) are paid once.
#[test]
fn raising_top_n_does_not_retract_shown_proposals() {
    let mut g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: 3,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse");

    // Small on purpose (see TOP_NS): three pairs per target, all cheap.
    let pairs = [(3usize, 5usize), (3, 8), (5, 8)];

    for &target in TARGETS {
        for &(small, big) in &pairs {
            g.set_config(GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n: small,
                ..GeneratorConfig::default()
            });
            let narrow: Vec<String> = g
                .generate(target)
                .iter()
                .map(|c| c.phrase.to_lowercase())
                .collect();
            assert!(
                !narrow.is_empty(),
                "no proposals for {target:?} at top_n {small}"
            );
            assert_eq!(
                narrow.len(),
                narrow.iter().collect::<HashSet<_>>().len(),
                "duplicate proposals at top_n {small} for {target:?}"
            );

            g.set_config(GeneratorConfig {
                mode: SearchMode::approximate(),
                top_n: big,
                ..GeneratorConfig::default()
            });
            let wide: HashSet<String> = g
                .generate(target)
                .iter()
                .map(|c| c.phrase.to_lowercase())
                .collect();

            let retracted: Vec<&String> = narrow.iter().filter(|p| !wide.contains(*p)).collect();
            assert!(
                retracted.is_empty(),
                "{target:?}: raising --top {small} to --top {big} retracted {} \
                 proposal(s) the smaller list had already shown: {retracted:?}",
                retracted.len()
            );
        }
    }
}
