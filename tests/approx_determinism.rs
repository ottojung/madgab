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
//! ## Why the *pool* has to be asserted, not just the proposals
//!
//! The visible list turned out to be deterministic while the candidate
//! pool behind it was not, and that asymmetry is invisible to a
//! proposal-level assertion: on the pre-fix binary the deduplicated pool
//! for one ordinary target alternated between two sizes run to run while
//! the visible `--top 20` stayed byte-identical, because the varying
//! candidates sat far below the display cutoff. Every reachability
//! claim in this project is denominated in pool *membership*
//! ([w-6b91d3](../docs/work/items/w-6b91d3.md)), so a pool that is not
//! reproducible makes a one-candidate `0 -> 1` undecidable. The test
//! below therefore reads the pool through
//! [`Generator::generate_with_pool`] and compares it across fresh
//! processes, and the mechanism is named in the source comment at the
//! structural DP's state map.
//!
//! ## Why run-to-run determinism is a *process* property
//!
//! `build_lexicon` parses the corpus JSON into a `HashMap<String,
//! RawEntry>` and then sorts `words` before building the trie, so the
//! trie itself is order-stable.  The `substitution_costs` table and the
//! `found` map in `FuzzyLexicon::matches_at` are only read by key or
//! sorted before use, so a differing `HashMap` seed should not reach the
//! answer *through them*.  That is an argument, not a guard, and it was
//! the argument that was wrong: the seed did reach the answer, through a
//! different map (the structural DP's state map).  A future change
//! that iterates any of these maps into a list would reintroduce exactly
//! the bug [w-5d03af](../docs/work/items/w-5d03af.md) fixed for exact
//! mode, and it would not be visible to any in-process test.  Each run
//! below is therefore a real subprocess with a fresh hash seed, and only
//! the proposal lines are compared (the header carries wall-clock
//! timings).
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

// -----------------------------------------------------------------
// Pool-level reproducibility
// -----------------------------------------------------------------

/// Targets for the pool guard.  Ordinary sentences of three different
/// shapes - a two-word collocation, a short clause, a longer clause and
/// a five-word phrase - and none of them is either of the two phrases
/// the project's acceptance criteria are written in terms of, so this
/// test cannot be satisfied by anything keyed to those.
const POOL_TARGETS: &[&str] = &[
    "recognize speech",
    "a whole lot of trouble",
    "it's just a stupid game",
    "the quick brown fox jumps",
];

/// Selects the helper test below and carries its target.  This lives in
/// the test, not in `src/`: nothing in the library needs to know that it
/// is being measured from a subprocess.
const POOL_HELPER_ENV: &str = "APPROX_POOL_HELPER_TARGET";

/// `top_n` for the pool guard.  The pool is a property of the *search*,
/// and `top_n` feeds the retention budget, so the count is only
/// comparable at a fixed `top_n`; 20 is an ordinary request.
const POOL_TOP_N: usize = 20;

/// The helper itself: a real search in a fresh process, reporting the
/// deduplicated pool size and the visible proposals.
///
/// It is a test that returns immediately unless the driver below names
/// it, so running the suite normally costs nothing.
#[test]
fn approximate_pool_helper() {
    let Ok(target) = std::env::var(POOL_HELPER_ENV) else {
        return;
    };
    let generator = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: POOL_TOP_N,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse");

    let (clues, pool) = generator.generate_with_pool(&target);
    assert!(
        !clues.is_empty(),
        "approximate search produced no proposals for {target:?}"
    );
    for clue in &clues {
        println!("POOLPROPOSAL {}", clue.phrase.to_lowercase());
    }
    println!("POOLSIZE {pool}");
}

/// One approximate search in a fresh process: its deduplicated pool
/// size, and its visible proposal phrases.
///
/// A subprocess is the whole point.  Within one process the structural
/// DP's state map has one hash seed for the lifetime of the process, so
/// an in-process comparison would hold trivially and would have passed
/// on the pre-fix binary.
fn approximate_pool_run(target: &str) -> (usize, Vec<String>) {
    let out = Command::new(std::env::current_exe().expect("test binary path"))
        .args([
            "--exact",
            "approximate_pool_helper",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(POOL_HELPER_ENV, target)
        .output()
        .expect("this test binary should be re-runnable");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "pool helper failed for {target:?}: {:?}\n{stdout}",
        out.status.code()
    );

    let mut pool = None;
    let proposals: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            if let Some(rest) = line.trim().strip_prefix("POOLSIZE ") {
                pool = rest.trim().parse::<usize>().ok();
                return None;
            }
            line.trim()
                .strip_prefix("POOLPROPOSAL ")
                .map(|p| p.to_string())
        })
        .collect();
    let pool = pool.unwrap_or_else(|| {
        panic!("pool helper printed no POOLSIZE line for {target:?}:\n{stdout}")
    });
    assert!(
        !proposals.is_empty(),
        "pool helper printed no proposals for {target:?}"
    );
    (pool, proposals)
}

/// The deduplicated candidate pool is byte-identical across fresh
/// processes, and so is the visible list drawn from it.
///
/// This is the assertion that was missing.  The visible `--top 20` was
/// already reproducible on the pre-fix binary while the pool behind it
/// alternated between two sizes on the same binary, because the varying
/// candidates ranked far below the display cutoff; a proposal-level
/// guard therefore could not see the defect.  `REPLICATES` is four
/// because the pre-fix failure rate was near one half per target, so a
/// smaller count would let the defect through often enough to matter.
#[test]
fn approximate_pool_is_reproducible_across_processes() {
    const REPLICATES: usize = 4;

    for &target in POOL_TARGETS {
        let (first_pool, first_proposals) = approximate_pool_run(target);
        for replicate in 1..REPLICATES {
            let (pool, proposals) = approximate_pool_run(target);
            assert_eq!(
                first_pool, pool,
                "approximate candidate pool for {target:?} at --top {POOL_TOP_N} \
                 was {first_pool} on the first fresh process and {pool} on \
                 replicate {replicate} of the same binary; a pool count is \
                 only a measurement if it is a property of the code and \
                 not of the run"
            );
            assert_eq!(
                first_proposals, proposals,
                "visible approximate proposals for {target:?} at --top \
                 {POOL_TOP_N} differed between the first fresh process and \
                 replicate {replicate} of the same binary"
            );
        }
    }
}
