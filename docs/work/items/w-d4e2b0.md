---
work_item: true
id: w-d4e2b0
state: working
priority: normal
owner: front agent-d4e2b0 (claimed and launched 2026-09-28T01:53Z by pass coord-3f18)
updated: 2026-09-28T01:53:00Z
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

### Front report, integration and coordinator validation 2026-09-28T02:12Z (pass coord-8f3b)

The front committed `0cc070b` (`src/main.rs` + `tests/pool_rank_reporting.rs`, +460/-2) and pushed
`madgab-poolrank-d4e2b0`. Report written to
`docs/work/REPORT-d4e2b0.md` **but still uncommitted in the front's worktree at the time of this
pass** — the front is still `running`, mid-way through its own full serial suite, so the report has
not been folded into the commit. Recorded here as an outstanding durability gap; a later pass
should confirm the report is committed and pushed, and can then close this item.

**Integrated into `post-milestone-acceptance` as `9f01a49`**, ahead of the front's still-running
full suite, after review on the tree. Reviewed as reporting-only, exactly as claimed:

* the default path's stdout is byte-identical — `render_row`'s `None` arm reproduces the old
  `"{:2}. [{:.3}] {}"` format exactly, and the phrase still begins at the first `]`;
* no change to search, scoring, selection, ordering, the default `--top`, or what ranks first;
* the pool-size line goes to **stderr**, so it cannot perturb a stdout parser, and it is read out
  of the `generate_pool` result `generate` was already computing and discarding — zero extra search
  on the default path;
* no phrase literal in `src/`; the canonical strings live in `tests/`, which `no_phrase_hard_coding`
  does not scan.

**Coordinator validation, on the integrated tree, `cargo test --release -- --test-threads=1`:**

| suite | result |
| --- | --- |
| `tests/pool_rank_reporting.rs` | **5 passed / 0 failed** |
| `tests/no_phrase_hard_coding.rs` | **9 passed / 0 failed**, `src/` allowlist still 0 |
| `tests/cli_milestone_predicate.rs` | **3 passed / 0 failed / 1 ignored** |

**Measured consequence worth keeping.** The two coordinates diverge sharply, and for the canonical
case-1 input they coincide — which is why they were never separated before:

| target | pool size | display 50's pool rank |
| --- | --- | --- |
| `recognize speech` | 18 289 | 27 |
| `It's just a stupid game` | 18 949 | 115 |
| `Coors light` | 12 956 | 1 346 |

The front's own full serial suite is still running and its remaining results are unconfirmed here;
`tests/corpus_integration.rs` is expected to be 12/1 on the known base red
`approximate_finds_classic_madgab_resegmentation`, which the front reports it did not relax, re-pin
or skip. `cargo fmt` and `cargo clippy` do not exist on this host, so the integrated change is
neither format- nor lint-verified — carried forward as a standing caveat.

**Next action for a later pass:** confirm `docs/work/REPORT-d4e2b0.md` is committed and pushed, read
the front's final suite result, and if it matches the above, set this item `state: done`.
