# w-d4e2b0 — Pool rank in the approximate CLI

Front `agent-d4e2b0`, branch `madgab-poolrank-d4e2b0`, worktree
`/workspace/madgab-poolrank-d4e2b0`, off `post-milestone-acceptance` at `4a796d4`.
Reporting front: `src/main.rs` and one new test file only.

## The gap this closes

[w-8f0b3d](items/w-8f0b3d.md) named a gap in itself: the CLI prints no pool rank
and exposes no flag to add one. The number the binary printed in front of a row
was the **display position** — the row's 1-based place in the list, *after* the
search's selection policy narrowed the deduplicated candidate pool to `--top`.
The **pool rank** is the row's place in that pool, *before* the narrowing. The
two are not the same number and the gap is not small.

The consequence has already cost this queue repeatedly: six passes of notes
carry a "display rank 27" figure, an older "26", and a pool-rank figure from a
`#[cfg(test)]` harness, all describing the same canonical row. A front that
changes pool contents or pool ordering therefore could not be reviewed from the
shipped binary at all, only from library internals.

## Before

`madgab --approximate --top 50 "recognize speech"`, base `4a796d4`:

```
target: recognize speech
IPA:    /ɹˈɛkəɡnˌaɪzspˈitʃ/
(corpus loaded in 382ms; search 759ms)

 1. [0.924] yeah 'cause i.'s pitch
 2. [0.923] yeah 'cause i.'s peach
25. [0.920] rec a guys pitch
26. [0.920] let ugh nice pitch
27. [0.920] wreck a nice beach
28. [0.920] rec a guys peach
```

stdout carries a bare score in the bracket and nothing else. stderr carries
target, IPA and two timings. Neither says how wide the pool the search actually
built was, and neither distinguishes "the search never proposed this" from "the
search proposed it and the display policy left it out" — the two are different
facts with different causes, and from outside the crate only the second was
observable.

## After

Two separable changes.

**1. Every run now reports the pool's size and the expansion factor**, on
stderr, using `generate_with_pool` — which the old `generate` call already
computed and threw away:

```
target: recognize speech
IPA:    /ɹˈɛkəɡnˌaɪzspˈitʃ/
(corpus loaded in 385ms; search 736ms)
(pool: 18289 scored candidates, 50 displayed; expansion 365.8x)

 1. [0.924] yeah 'cause i.'s pitch
 2. [0.923] yeah 'cause i.'s peach
25. [0.920] rec a guys pitch
26. [0.920] let ugh nice pitch
27. [0.920] wreck a nice beach
28. [0.920] rec a guys peach
```

This costs **zero** extra search: `generate` is defined as
`generate_with_pool(..).0`, so the second element was already being computed and
discarded. The change is reading it.

**2. A new `--pool-rank` flag labels both coordinates on the row:**

```
$ madgab --approximate --top 50 --pool-rank "recognize speech"
(pool: 18289 scored candidates, 50 displayed; expansion 365.8x)
(pool-rank run: second search 805ms)

 1. [score 0.924, pool rank 1 of 18289] yeah 'cause i.'s pitch
 2. [score 0.923, pool rank 2 of 18289] yeah 'cause i.'s peach
25. [score 0.920, pool rank 25 of 18289] rec a guys pitch
27. [score 0.920, pool rank 27 of 18289] wreck a nice beach
28. [score 0.920, pool rank 28 of 18289] rec a guys peach
```

The two numbers go **inside the one bracket, each labelled**. That is the point:
a reader cannot take one for the other, and a consumer that splits on the first
`]` still gets the phrase in the same position as before, so the annotation is
additive rather than a re-layout.

The flag costs a second search, because `generate_with_pool` reports the pool's
*size* only and a rank needs the pool's *contents*, which is
`generate_pool`. There is no single-call public API returning both the selected
proposals and the pool — `search` is private — and `src/lib.rs` is out of scope
for this front. Hence the flag, and hence the opt-in cost.

## The two coordinates really do diverge

Measured on the shipped binary at `--approximate --top 50`:

| target | pool size | display 1's pool rank | display 50's pool rank |
|---|---|---|---|
| `recognize speech` | 18 289 | 1 | 27 |
| `It's just a stupid game` | 18 949 | 1 | 115 |
| `Coors light` | 12 956 | 1 | 1 346 |

For the canonical case-1 input the selection happens to be a prefix of the pool,
so display position and pool rank coincide — which is precisely why the two were
never separated: the one input the milestone is written about is the one input
where the distinction is invisible. On `Coors light` display 50 sits 1 296 ranks
deep in a pool of 12 956. Any future report of "rank N" for a non-canonical
target is now checkable against the other coordinate.

## Wall-clock cost

Whole-process wall clock, two runs each, `--approximate --top 50`:

| target | default | `--pool-rank` | delta |
|---|---|---|---|
| `recognize speech` | 1410 / 1439 ms | 2052 / 2093 ms | +653 ms |
| `Coors light` | 1179 / 1186 ms | 1494 / 1501 ms | +315 ms |
| `It's just a stupid game` | 1427 / 1445 ms | 2186 / 2148 ms | +737 ms |

The delta is the second search (self-reported 805 ms for case 1) minus the small
variance between runs. The **default path is unchanged**: the pool size now
printed costs no measurable time, because the search had already run. Corpus
load (~380 ms) dominates both and is untouched.

## What was deliberately not changed

Search, scoring, selection, ordering, `src/lib.rs`, `src/approx.rs`, the default
`--top` of 10, and what the tool ranks first. The default `--top` value is
untouched, and `tests/cli_milestone_predicate.rs::shipped_default_top_n_does_not_display_the_canonical_case_one`
still passes, which is the test that would fire if the default widened.

Ordering is [w-c31a07](items/w-c31a07.md)'s surface and was not touched. This
front is disjoint from it by construction: it changes what the CLI *says* about
a row, not which rows exist or in what order.

`approximate_finds_classic_madgab_resegmentation` was not relaxed, re-pinned or
skipped. It is red at base and is red here.

## Validation

Serial runs (`cargo test --release -- --test-threads=1`), own
`CARGO_TARGET_DIR=/workspace/target-d4e2b0`:

* `tests/pool_rank_reporting.rs` (new) — **5 passed / 0 failed**.
* `tests/corpus_integration.rs` — **12 passed / 1 failed**, the one failure being
  the known base red `approximate_finds_classic_madgab_resegmentation`, not
  re-pinned.
* `tests/no_phrase_hard_coding.rs` — **9 passed / 0 failed**, `src/` allowlist
  entries still 0. This front adds no phrase literal to `src/`; the canonical
  strings live in `tests/`, which that test does not scan.
* the rest of the suite green: `src/lib.rs` unittests 83 passed / 0 failed /
  12 ignored, `approx_determinism` 4/0, `exact_determinism` 1/0,
  `objective_is_a_search_input` 1/0, `emit_coverage` 7/0, `main.rs` unittests
  0/0.

The full serial suite is green apart from two targets, both accounted for:

* `corpus_integration` — the known base red, as required.
* `--doc` — **not a code failure.** `rustdoc` is not installed on this host
  (`command -v rustdoc` → not found; only `cargo` and `rustc` are present), so
  the doctest target cannot execute at all. Doc-tests are extracted from
  `src/lib.rs`, which this front did not touch, so this is an environment
  limitation and not a regression. It would fail identically at base.

`cargo test` without `--no-fail-fast` halts at `corpus_integration` by design, so
the full picture above required that flag.

New tests, at the executable boundary, driving `CARGO_BIN_EXE_madgab`:

* `default_output_shape_is_unchanged_and_reports_the_pool_size` — the default
  bracket still holds only the score, the phrase still starts at the first `]`,
  the index is still right-aligned in two columns, `pool rank` never appears in
  stdout, and the pool line is on stderr so it cannot perturb a parser.
* `pool_rank_flag_labels_score_and_pool_rank_on_the_same_row` — both coordinates
  are labelled, on every row.
* `reported_pool_ranks_are_in_range_and_ordered` — every rank is within the
  pool, ranks never go backwards along display order, the pool is at least as
  wide as the display.
* `the_flag_annotates_the_same_rows_the_default_path_prints` — the flag changes
  reporting only: same rows, same order, display positions 1..=N. This also
  checks that the second search and the first agree, which is what makes the
  reported ranks describe the displayed rows.
* `canonical_case_one_survives_the_reporting_change` — case 1 re-measured
  through the new annotation rather than copied forward.

The value assertions are deliberately range-and-order shaped rather than exact.
Exact pool sizes and exact pool ranks are properties of the search and of the
selection policy, which belong to other fronts; pinning them here would turn a
reporting test into a tripwire on work this front is explicitly not doing.

## Honest notes

* **`cargo fmt` and `cargo clippy` did not run.** This host has no rustup, so
  neither is available. I have not verified formatting or lints. The new code
  follows the file's existing style by hand; a `fmt` pass on a rust-capable host
  may still adjust it.
* Exact-mode `--pool-rank` also works, and costs a second search there too. It
  is not exercised by the new tests beyond the shared shape assertions.
* The pool ranks come from a *second* search rather than the same one that
  produced the displayed rows. The search is deterministic
  (`tests/approx_determinism.rs`, `tests/exact_determinism.rs` both assert
  byte-identical stdout across processes), so the two runs' pools are the same
  pool, and `the_flag_annotates_the_same_rows_the_default_path_prints` checks
  that the two agree rather than assuming it. The real fix is a public API
  returning both from one search, which is a `src/lib.rs` change and therefore
  another front's.
* Expansion is reported as `pool_size / displayed`, which is a ratio of two
  different policies' outputs rather than a property of the search alone. It is
  named "expansion" and defined by that line; treat it as a display diagnostic,
  not a search invariant.
