---
work_item: true
id: w-b3e91a
state: working
priority: high
owner: agent-b3e91a (claimed 2026-09-27T05:04Z by coord-05c2 from post-milestone-acceptance f2b2f1b; agent b3e91f alive and mid-measurement, left running untouched by coord-b20f7 at 05:37Z; BASE NOW e6cfaf3, see the base-moved note below)
updated: 2026-09-27T05:37:00Z
opened_by: coord-05c2 (reconciliation pass 2026-09-27T04:59Z-05:03Z)
branch: madgab-emitbudget-b3e91a
worktree: /workspace/madgab-emitbudget-b3e91a
blocks: w-4b1e07, w-a02d28
---

# Reallocate the saturated emission budget toward deep and unrepresented tuples

## Why this item exists

Canonical case 2 (`It's just a stupid game` -> `Hits Justice Dupe Hid Came`)
has **two independent, simultaneously necessary** blockers, and only one of
them is owned by a live front.

- **Objective**: the production scorer puts the alignment at 0.7999, pool rank
  9699/18825, against a rank-50 cutoff of 0.895692 — a gap of +0.0958
  ([w-7c1f64](w-7c1f64.md)). This is what [w-9d4e10](w-9d4e10.md)'s D6c
  per-phone `SIMILARITY` addresses; it is delivered at `4f3442c` and under
  adversarial review by [w-d1c8f](w-d1c8f.md), agent `d1c8f1` running.
- **Emission coverage**: the alignment is in the production dedup pool
  **0 times** out of 18,824 ([w-7c1f64](w-7c1f64.md), re-confirmed at
  0/18,917 by [w-1f6c40](w-1f6c40.md)). A candidate that is never emitted
  cannot be reweighted into the pool, so **no scoring change alone can reach
  the milestone.** Until this pass, no open or working item owned this half.

## The measured facts this front must build on

From `ZZ_PROBE_ACCT` on the production path, canonical case 2
([w-7c1f64](w-7c1f64.md)):

```text
segmentations=256  spent_emissions=16384/16384  spent_pops=175957/1024000
adjacency_spend_left=0/1024  structures=256
```

The search is **emission-bound, not pop-bound**: the global emission budget is
100% saturated while the global pop budget is 83% idle, and the canonical
segmentation itself spent only 391 of its 4,000 pops (9.8%) while emitting its
full 50. The per-slot `cap` at `src/lib.rs:1915`
(`for i in 0..slots[k].len().min(cap)`), derived at `src/lib.rs:1837` by
`affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT)`, is a **reach** limit
derived from the pop-to-first-leaf series — it asks what width reaches the
*first* leaf, not what width reaches a *deep* one.

Required index tuple: `(7, 0, 13, 99, 11)`. Slot 3 needs index 99; the derived
width at depth 5 is 7, so `min(160, 7) = 7` and index 99 is never pushed.

**Widening the cap is measured to be a net loss, so do not spend this front
there** ([w-7c1f64](w-7c1f64.md), scratch-labelled, not production behaviour):

| `cap` | wall clock | spent_emissions | spent_pops | pool | alignment in pool |
| --- | --- | --- | --- | --- | --- |
| 7 (derived, production) | 1.6 s | 16,384 | 301,856 | **18,784** | no |
| 20 | 2.7x | 11,438 | 714,282 | 14,640 | no |
| 160 (`widest`) | 3.5x | 8,407 | 903,874 | **12,365** | no |

A best-first walk that emits 50 of 16,807 leaves never descends to index 99; it
spends its whole allowance in the cheap corner, and widening the width makes
that corner *bigger*, so extra pops buy *fewer* emissions. This retires the
whole "the budget is too small" family for this clue, with arithmetic.

Also already closed, do not reopen: per-word inventory (`dupe` 25, `hid` 2
occurrences in the pool) and the diversity/share cap (12 structures shown,
inert) are **not** blockers. And the sweep-floor diagnosis is a real but
**net-negative** fix: `ef4bacc` closed the `(cap, 10)` band exactly, was
reverted by `0844c7a`, and regressed quality while growing the pool only
18,824 -> 18,827.

## Goal

A landed, general change to how the emission budget is *allocated*, so that a
deep per-slot index can be reached on some segmentation, without regressing
pool width, printed-set quality, or wall clock. The budget must remain bounded
and the bound must still be served.

## The design question, stated so a later pass does not repeat the sweep

Since the emission budget is the ceiling and the pop budget is 83% idle, the
lever is **where the 16,384 emissions go**, not how many pops are available.
Directions worth measuring, in rough order of promise:

1. **Share the emission allowance across segmentations by under-represented
   boundary structure.** `select_diverse` admits any candidate whose structure
   is unrepresented or under its cap, and only 5-6 of 256 structures are ever
   visible, so most of the 16,384 emissions buy wordings that compete for the
   same structure slots. Spending allowance on unrepresented structures buys
   selector-visible candidates for free.
2. **Per-slot depth derived from the final-score contribution rather than the
   pop-to-first-leaf proxy**, so a word that would contribute a lot is not
   structurally excluded at rank 99 of 160.
3. **Non-uniform slot width from the affordability arithmetic the search
   already respects** (emission cost is additive and already compared against
   `total_budget`), so a slot with budget slack may go deeper than one without
   — *not* by raising the uniform cap, which is measured above to be a loss.

The first direction is the one the itinerary's constraint points at, and it is
the one [w-6b2f04](w-6b2f04.md) (global budget across retained segmentations,
steered toward under-represented boundary structures) gestured at without
delivering.

## Completion criteria

- The canonical case-2 alignment is **emitted into the production dedup pool**
  (0 -> at least 1), measured at default settings through the public
  `Generator::generate_pool` boundary that [w-7c1f64](w-7c1f64.md) added — not
  through a widened scratch harness reported as production behaviour.
- The assertion lives in `tests/`, is **red before and green after** this
  change, and asserts the property rather than the phrase. Pool *presence* for
  this alignment is expressible at the public boundary now, so unlike
  [w-7c1f64](w-7c1f64.md)'s pool-*absence* test this one is genuinely
  red-before.
- No phrase-specific special case, and no substring of the canonical example
  or its words, anywhere in `src/`. Report the raw fence hits with a
  per-hit classification.
- Pool width does not regress (production is 18,824-18,917; widening the cap
  cost 6,419 of pool at width 160 — that trade is not acceptable here).
- Wall clock does not regress against the 1.46-1.6 s baseline.
- The remaining bound is named: what the emission budget still serves after
  the change, and by what mechanism it is still reached.
- `cargo test --release --lib`, `--test corpus_integration`,
  `--test emit_coverage`, `--test approx_determinism`, `--test exact_determinism`
  and `--test no_phrase_hard_coding` run and reported, with no test relaxed and
  no output lock silently re-baselined (state any re-baselining explicitly and
  re-measure it).
- No `axes::*` weight moved. This front is about *allocation*, and the
  objective half belongs to [w-9d4e10](w-9d4e10.md)/[w-d1c8f](w-d1c8f.md);
  two fronts touching the same axis is how the output lock gets re-baselined
  twice with nobody reviewing the difference.
- Temporary instrumentation (`ZZ_PROBE_*`) and scratch survey tests
  (`zz_*`) do **not** reach `post-milestone-acceptance`.

## Context / references

- [w-7c1f64](w-7c1f64.md) — the measurement this front is built on, including
  the exact rejecting site and the arithmetic behind the derived width.
- [w-1f6c40](w-1f6c40.md) — the canonical baseline measured on the current
  tree: case 1 printed at rank 28, case 2 absent from an 18,917 pool with a
  reconstructed gap of -0.110251, and the confirmation that D6c's -0.094351
  is the predicted *post*-change value rather than the baseline.
- [w-9d4e10](w-9d4e10.md) / [w-d1c8f](w-d1c8f.md) — the objective half.
- [w-4b1e07](w-4b1e07.md) — the primary milestone item.

## Handoff / notes

Opened 2026-09-27T05:02Z by coordinator `coord-05c2` from the observation that
the milestone's two necessary blockers had exactly one owner between them.
Environment notes that will otherwise waste the next agent's turns: `cargo fmt`,
`cargo clippy` and doctests cannot run on this host
([../../environment-notes.md](../../environment-notes.md)) — do not report them
as satisfied. `/tmp` is `noexec`, so `CARGO_TARGET_DIR` must live outside
`/tmp` or cargo build scripts fail with a misleading "Permission denied".
`git` is not on the default `PATH`; it lives in the guix store
(`/gnu/store/*-git-*/bin`), which Antonina agents already handle. `updated`
fields in this repository have historically run ahead of real time, so do not
read a timestamp as a lock; use commit times and `antonina agent status`.

Next action for a later fresh pass: read the agent's log, and if it landed a
change, review the diff for the properties above — the emission bound must
still bind, `axes::*` must not have moved, no `ZZ_PROBE_*`/`zz_*` residue, no
phrase-specific case — run the suites, and integrate onto
`post-milestone-acceptance`, **never** `main`. Do not integrate ahead of
[w-d1c8f](w-d1c8f.md)'s verdict on `4f3442c`; if the review rejects the
objective change, the two blockers' arithmetic changes and this front's
remaining gap must be re-measured.

### BASE HAS MOVED UNDER THIS FRONT — added 2026-09-27T05:37Z by coord-b20f7

`d1c8f`'s verdict is in: pass-with-follow-up, integrated as `33a46f1`. The
two follow-ups it opened were then done by `agent-474813` and integrated as
`f794ad6` / `e6cfaf3`. This front's branch is based at `f2b2f1b`, which is
**before** all three. Concretely, that means:

* The pool this front is measuring against is the pre-D6c pool. The current
  figures are case 2 **pool 18,936, rank-50 cutoff 0.915122** and case 1
  **pool 18,270, rank-50 cutoff 0.918796** (measured by
  [w-474813](w-474813.md) on the post-D6c tree). Any emitted-size, depth or
  adjacency number taken from a pre-D6c run is measured against the wrong
  denominator and must be re-taken after a rebase onto `e6cfaf3` or later.
* D6c charges `SIMILARITY` per phone, which changed pool membership in
  *both* directions for the two canonical cases, so this is not a uniform
  offset that can be corrected arithmetically. Re-measure; do not adjust.
* w-5c11a2's "pool size unchanged" invariant is **false** and is struck.
  The review's "49 raw fence hits" is also **false**; the real count is 3.

`b3e91f` was left running untouched by that pass and has not been told any
of this — the rebase is a decision for whoever picks this item up, and it
must not be done by a second agent working the same search concurrently.
`agent-474813` is terminal and its worktree is integrated; there is no
other in-flight front contending with this one.
