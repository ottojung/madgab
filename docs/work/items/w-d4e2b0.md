---
work_item: true
id: w-d4e2b0
state: done
priority: normal
owner: front agent-d4e2b0 (claimed 2026-09-28T01:53Z by pass coord-3f18; work complete and pushed 2026-09-28T02:2xZ)
updated: 2026-09-28T02:29:00Z
branch: madgab-poolrank-d4e2b0
worktree: /workspace/madgab-poolrank-d4e2b0
---

# Expose a candidate's pool rank in the approximate CLI output

## Goal

Make the approximate CLI report each proposal's rank in the scored pool, so "display position"
and "pool rank" stop being conflated, and add a regression test at the executable boundary for
the milestone predicate.

## Why this is worth a front

[w-8f0b3d](w-8f0b3d.md) measured the shipped binary and reported **one unrepaired gap it names
itself**: the CLI prints no pool rank and exposes no flag to add one. The consequence is
concrete and has already cost this queue repeatedly — six passes of notes carry a "display
rank 27" figure, an older figure of "26", and a pool-rank figure from a `#[cfg(test)]` harness,
because the artifact never exposed the distinction. A front that changes pool contents or pool
ordering therefore cannot be reviewed from the shipped binary at all, only from library
internals, and its evidence is not comparable with the last front's.

This is a small, general, non-phrase-specific improvement to the tool's observability, and it
makes every later search-quality front cheaper to review.

## Completion criteria

1. The approximate CLI prints or otherwise reports each proposal's rank within the scored pool,
   or accepts a flag that makes it do so. Default output should stay backwards-compatible in
   shape unless that is impossible; say which you chose and why.
2. The distinction between **display position** and **pool rank** is defined in the code, so the
   two cannot be silently equated again.
3. A regression test at the executable boundary covering the reported rank. The phrase literal
   must live in `tests/` or a `#[cfg(test)]` module, and `no_phrase_hard_coding` must stay
   **9/0** with `src/` at zero allowlist entries.
4. Case 1 remains present at display 27-or-better in the shipped binary, and case 2 is not made
   worse. Re-measure; do not assume. `corpus_integration` must stay 12 passed / 1 failed with
   the known case-2 red **not re-pinned**.
5. `docs/work/REPORT-d4e2b0.md` stating what the tool printed before, what it prints after, and
   the wall-clock cost of the change.

## Constraints

- No phrase-specific hard-coding and no exception keyed on either canonical sentence or clue word.
- Touch only `src/main.rs` and the tests you add. Do not change search, scoring, selection,
  `src/lib.rs` or `src/approx.rs` — this is a reporting front.
- No change to the default `--top` value and no change to what the tool ranks first; ordering is
  [w-c31a07](w-c31a07.md)'s surface, not this one's.
- Work on a focused branch in its own worktree created from `post-milestone-acceptance`. Never
  merge into `main`; integrate into `post-milestone-acceptance`.

## Handoff and notes

Opened 2026-09-28T01:42Z by pass `coord-9a41`. Not claimed; a later pass claims it by pushing
the `state: working` metadata change.

Disjoint from [w-c31a07](w-c31a07.md) by construction — this one is confined to `src/main.rs`
reporting, that one changes ordering/selection. They can run in parallel in separate worktrees.

### Claimed 2026-09-28T01:53Z (pass coord-3f18)

Claimed by pushing this metadata change against `origin/post-milestone-acceptance` at `a953c61`,
then launched as front `agent-d4e2b0` in worktree `/workspace/madgab-poolrank-d4e2b0` on branch
`madgab-poolrank-d4e2b0` off `a953c61`. Launched deliberately in parallel with the still-running
`agent-c31a07`, which is confined to ordering/selection in `src/approx.rs`; this front is confined
to `src/main.rs` reporting, so the two do not contend and neither may merge.

Host note for the front: use its own `CARGO_TARGET_DIR` (the coordinator's `/workspace/madgab/target`
is the only warm tree and must stay warm for later fence runs), and run tests with
`-- --test-threads=1` — the 13-test `corpus_integration` run at default threads was SIGKILLed by
host memory pressure in the 01:36Z pass (49 of 62 GB used, 11 available), so the serial run is the
measurement, not an inconvenience.

### Completed 2026-09-28T02:29Z by front agent-d4e2b0

Pushed on `madgab-poolrank-d4e2b0`. Work commit `0cc070b` (`src/main.rs` +
`tests/pool_rank_reporting.rs`); report at
[../REPORT-d4e2b0.md](../REPORT-d4e2b0.md). Not merged into
`post-milestone-acceptance` — integration is the coordinator's call, and
[w-c31a07](w-c31a07.md) is still running in parallel.

**What changed.** Two separable things, both in `src/main.rs`.

1. Every run now prints the pool's size and the expansion factor to **stderr**:
   `(pool: 18289 scored candidates, 50 displayed; expansion 365.8x)`. This costs
   **zero** extra search — `generate` is defined as `generate_with_pool(..).0`,
   so the pool size was already being computed and discarded; the change reads it.
2. A new `--pool-rank` flag labels both coordinates on the row, inside the one
   existing bracket so the phrase still starts at the first `]` and every existing
   parser is unaffected:
   ` 1. [score 0.924, pool rank 1 of 18289] yeah 'cause i.'s pitch`

The flag costs a **second search** (+315 to +737 ms measured per process, the
self-reported second search being 805 ms for case 1), because
`generate_with_pool` reports the pool's size only and a rank needs the pool's
contents via `generate_pool`. There is no single-call public API returning both
the selected proposals and the pool — `search` is private — and `src/lib.rs` is
out of scope for this front. Hence the opt-in flag. The **default path is
unchanged in wall clock and byte-identical on stdout** (verified by md5 on three
targets, before vs after).

**Why the flag rather than changing the default output.** The item asked to state
the choice. The default shape is what `tests/approx_determinism.rs`,
`tests/exact_determinism.rs` and `tests/cli_milestone_predicate.rs` parse, and
those are not this front's to edit. A flag keeps every consumer working and keeps
the extra search off the default path.

**The two coordinates diverge, which is the point.** At `--approximate --top 50`:

| target | pool size | display 50's pool rank |
|---|---|---|
| `recognize speech` | 18 289 | 27 |
| `It's just a stupid game` | 18 949 | 115 |
| `Coors light` | 12 956 | 1 346 |

For the canonical case-1 input the selection happens to be a prefix of the pool,
so display position and pool rank coincide — which is exactly why the two were
never separated before. The milestone's own input is the one input where the
distinction is invisible.

**Re-measured canonical numbers** (shipped binary, re-measured, not assumed):

* **case 1** — `wreck a nice beach` for `recognize speech` is present at
  **display 27 of 50**, score 0.920, **pool rank 27 of 18 289**, unchanged from
  base. Still absent at the shipped default `--top 10`, as before.
* **case 2** — `It's just a stupid game` is **not made worse** and not made
  better: not printed at any documented public knob, exactly as at base. Its
  first row is still `it said thus test oop day`; display 50 is
  `it josh test oop add aim` at **pool rank 115 of 18 949**. The known base red
  `approximate_finds_classic_madgab_resegmentation` is still red and was **not**
  relaxed, re-pinned or skipped.

**Validation** (serial, `cargo test --release --no-fail-fast -- --test-threads=1`,
own `CARGO_TARGET_DIR=/workspace/target-d4e2b0`):

| suite | result |
|---|---|
| unittests `src/lib.rs` | 83 passed, 0 failed, 12 ignored |
| `tests/pool_rank_reporting.rs` (new) | **5 passed / 0 failed** |
| `tests/corpus_integration.rs` | **12 passed / 1 failed** — the 1 being `approximate_finds_classic_madgab_resegmentation` |
| `tests/no_phrase_hard_coding.rs` | **9 passed / 0 failed**, `src/` allowlist entries still **0** |
| `tests/cli_milestone_predicate.rs` | 3 passed, 0 failed, 1 ignored |
| `tests/approx_determinism.rs` | 4 passed / 0 failed |
| `tests/exact_determinism.rs` | 1 passed / 0 failed |
| `tests/emit_coverage.rs` | 7 passed / 0 failed |
| `tests/objective_is_a_search_input.rs` | 1 passed / 0 failed |
| unittests `src/lib.rs` | 83 passed, 0 failed, 12 ignored |

The whole serial suite is green apart from two targets, neither a new failure:

* `corpus_integration` — the known base red, as required.
* `--doc` — **not a code failure.** `rustdoc` is not installed on this host
  (`command -v rustdoc` → not found; only `cargo` and `rustc` are), so the
  doctest target cannot execute. Doc-tests come from `src/lib.rs`, which this
  front did not touch, so it would fail identically at base.

`cargo test` without `--no-fail-fast` halts at `corpus_integration` by design, so
the full picture above required that flag.

**Honest gaps.**

* **`cargo fmt` and `cargo clippy` were not run** — this host has no rustup, so
  neither is available. Formatting and lints are unverified; a `fmt` pass on a
  rust-capable host may adjust the new code.
* The pool ranks come from a second search, not the one that produced the
  displayed rows. The search is deterministic and
  `the_flag_annotates_the_same_rows_the_default_path_prints` checks the two runs
  agree rather than assuming it. The proper fix is a public API returning both
  from one search, which is a `src/lib.rs` change and therefore another front.

**Suggested follow-up item, not done here:** expose selected proposals *and* the
pool from a single public call in `src/lib.rs`. It would halve `--pool-rank`'s
cost, remove the two-search consistency assumption, and let library-level tests
assert a candidate's pool rank directly. That is outside this front's declared
surface, so it is recorded rather than attempted.
