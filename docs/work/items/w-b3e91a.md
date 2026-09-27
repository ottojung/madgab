---
work_item: true
id: w-b3e91a
state: done
priority: high
owner: agent-b3e91a (claimed 2026-09-27T05:04Z by coord-05c2 from post-milestone-acceptance f2b2f1b; agent b3e91f finished 2026-09-27T05:56Z with a priced negative result; front closed and its branch integrated as 47ad98c/a49fed3 by coord-05d3 at 05:58Z; the remaining objective half is re-filed as w-7e1a04)
updated: 2026-09-27T06:07:00Z
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

### FRONT STEERED, AND ITS UNCOMMITTED WORK IS THE TOP RISK — 2026-09-27T05:46Z by coord-3f1a

`b3e91f` was steered (`--steer`, `prompts: 2`) and left running. It is still
`state: running`, `alive: yes`, and still the only owner of case 2. The steer
delivered: the base has moved to `29d9ad1` and now includes the D6c
integration `33a46f1` and `f794ad6`/`e6cfaf3`, so every emitted-size, depth,
adjacency and cutoff number taken on `f2b2f1b` is against the wrong
denominator; the current post-D6c denominator is case 2 pool **18,936** /
rank-50 cutoff **0.915122**; the w-5c11a2 "pool size unchanged" invariant and
the review's "49 raw fence hits" are both false and are to be struck; the
deliverable is a landed general change or a negative result with arithmetic,
not a third sweep; and it must commit and **push** its 41+ minutes of
uncommitted `src/lib.rs` work before spending further turns on measurement.

The concrete hazard for a later pass: at 05:46Z this worktree still had
`M src/lib.rs` uncommitted and its branch at `f2b2f1b`. If the agent finishes
or the host restarts before that push, the front's only implementation work is
unrecoverable, and this item would have to be reopened from scratch rather
than resumed. Check `git status` in `/workspace/madgab-emitbudget-b3e91a`
first on any later pass.

Note the steer reset the agent's `started` timestamp to
`2026-09-27T05:43:40Z` and its `prompts` count to 2. Do not read either as a
new agent or a restart.

## Objective state (agent b3e91a, base b0c7d66) — NEGATIVE, priced

**Verdict: the direction is retired, and it is retired by arithmetic rather
than by a sweep.** `Hits Justice Dupe Hid Came` is not unreachable because the
16,384 emissions are misallocated, and it is not unreachable because
`cap` is 7. It is unreachable because **2,036,664 better-bound wordings of its
own segmentation stand in front of it in the emission order** — 124× the entire
global emission ceiling — and no allocation of 16,384 emissions across 256
segmentations changes that number. Every reallocation is a *reordering* of at
most 16,384 emissions; the order distance is 2,036,664. The shortfall is a
factor of 124 in the cheapest possible reallocation and 1,309 in the
segmentation's own product space.

All numbers below are re-measured on `b0c7d66` (code identical to `29d9ad1` +
this front's `ca644e8`/`d751734`), release, defaults + `--top 50`, through the
public `Generator::generate_pool` boundary. Nothing below is a widened scratch
harness reported as production behaviour; the widened runs are labelled as the
counterfactual they are.

### 1. The emission ceiling is saturated, and the source said it was not

`ZZACCT` on the production path, canonical case 2, nothing widened:

```text
segmentations=256  spent_emissions=16384/16384  spent_pops=175957/1024000
adjacency_spend_left=0/1024
```

Case 1 is the same shape: `195370/1024000` pops, `16384/16384` emissions. The
search is emission-bound at 100 % and pop-bound at 17-19 %.

`src/lib.rs:1836` justified the adjacency operator *not* being carved out of
the traversal's allowance on the ground that the global ceiling "leaves slack
(14,239 of 16,384 spent)". **That ground is false**, and it was load-bearing:
it is the stated reason two mechanisms may share one ceiling without competing.
Corrected at `ca644e8`, with the measurement recorded beside it, and pinned by
a new phrase-free test `the_global_emission_ceiling_is_reached_not_merely_respected`
(`d751734`) which asserts the per-target *bound* (`emissions <= 16,384`,
including on a two-word target that spends 4,529) and, over the corpus, that
the ceiling is **reached** while the pop ceiling is not approached. The
distinction is deliberate: saturation is a fact about how big a particular
lattice is, and asserting it per target would be asserting a phrase's size; the
bound is a property of the search and is asserted per target.

This is the one landed change from this front. It is a comment correction plus
`#[cfg(test)]` counters: **no behaviour change, no `axes::*` weight moved.**

### 2. The traversal cannot reach the tuple at *any* cap, and the reason is order, not reach

Canonical segmentation `cuts = [3, 10, 13, 15, 19]`, slot widths
`[160, 7, 160, 160, 93]`, required index tuple `(7, 0, 13, 99, 11)`. Best-first
walk over the real `bound`, measuring the deepest per-slot index it actually
emits:

| cap | pops | emissions | deepest index emitted | tuple reached |
| --- | --- | --- | --- | --- |
| 7 (derived, production) | 19,465 (heap exhausted) | 16,664 | **6** | no |
| 160 (= full slot width, 23× the derived cap) | 4,000,001 (capped) | **2,036,664** | **159** | **no** |

The second row is the decisive one. At the *full* slot width the walk emits
2,036,664 wordings — **124.3× the entire global emission ceiling of 16,384**,
and 1,309× the tuple's own order distance is short of the product — and still
never emits the tuple, while reaching index 159. So this is **not a reach
limit**: the wide cap demonstrably reaches deep indices. It is an *order*
limit. 2,036,664 of the segmentation's `160·7·160·160·93 = 2,666,496,000`
wordings (0.0764 %) are emitted before the tuple, and the walk stops.

The pre-existing widening table in this item (`cap` 7/20/160 → pool
18,784/14,640/12,365) is therefore not merely a quality regression; at width
160 the traversal *cannot buy the alignment at any price it could afford*, and
buying it costs more than the whole budget by two and a half orders of
magnitude.

### 3. The additive operator cannot reach it either

The adjacency/neighbourhood operator moves by substituting one slot of a
complete wording, so its depth in one slot is additive rather than
multiplicative — its own module doc calls it the mechanism for "a slot index
far outside any width-capped prefix walk". Measured directly on the canonical
segmentation, 40 configurations, `per_slot ∈ {2, 4, 8, 16, 32, 64, 128, 160}`
× `pops ∈ {64, 512, 4096, 32768, 262144}`: **zero hits.**

The reason is the same order and the same key: the operator is best-first on
the *same* admissible `bound`, so its reachable set is the same corner, merely
reached additively. Additivity buys the *distance*, and the distance is not
what is missing.

### 4. The deep-index space is already served — the item's literal goal is met

Deep per-slot indices (≥ 99) actually placed in the production pool for case 2
alone, by mechanism, at production settings:

| mechanism | emissions with a per-slot index ≥ 99 |
| --- | --- |
| coverage reserve (`coverage_tuples`) | **2,682** |
| adjacency operator | 18 |
| lexical traversal | **0** |

So "a deep per-slot index is reachable on some retained segmentation" — this
item's stated goal — is **already true and already asserted**, by the existing
green test `depth_profile_emissions_reach_deeper_than_the_traversal`. The
canonical alignment is not the deep-index case; it is the *four-deep-slots-at-
unrelated-ranks* case, and a systematic sampler cannot hit four specific
coordinates at once without being aimed at them, which is the phrase-specific
special case this item forbids. `EMIT_PROFILE_MAX_DEEP = 3` also makes a
4-deep tuple structurally inexpressible to the reserve, and `sweep_index`'s
floor of 10 puts slot 0's index 7 in the unowned `(cap, 10)` band — but
unblocking both does not help, because the reserve draws 8 systematic samples
per subset and would have to hit `(7, 13, 99, 11)` simultaneously.

### 5. The three design directions, each retired with its own number

1. **Share the allowance toward under-represented boundary structures.**
   Retired by §2. A share is a reordering of ≤ 16,384 emissions; the order
   distance is 2,036,664. Measured as well: raising the operator's share
   uniformly (`ADJACENCY_RESERVE` 8→64, `ADJACENCY_GLOBAL_RESERVE` 1,024→8,192)
   moves case 2's distinct retained structures **308 → 214** and its pool
   18,936 → 19,141, because the operator runs per segmentation and starves the
   late-retained structures before the traversal reaches them. Uniform
   reallocation is a net loss; and there is no *non-uniform* reallocation that
   helps, for the factor-of-124 reason.
2. **Per-slot depth from final-score contribution rather than the
   pop-to-first-leaf proxy.** Retired by sign, not by size. The tuple's
   slot-3 word is the least familiar of the five (`FAMILIARITY` 0.398710
   aggregate) and its whole score is 0.820770 against a 0.924146 pool maximum.
   A depth rule derived from score contribution gives *that* slot *less*
   depth, not more. This direction moves the wrong way for the clue it was
   proposed for.
3. **Non-uniform slot width from the affordability arithmetic.** Retired by
   measurement: the cap sweep is already in this item and is a pool-width
   regression, and §2 adds that at full width the target is *still* not
   reached. There is no width at which the order distance shrinks.

### 6. Emission is no longer the binding blocker, and never was sufficient

Forced onto the unchanged production path on the current tree — same release
binary, same defaults, no budget widened, one tuple pushed into the pool:

```text
in_pool=(rank 9668 of 18937, score 0.820770)   in_printed=None
printed-50 cutoff = 0.912971   gap = +0.092201   rank deficit = 9,618 places
```

D6c (`33a46f1`) moved this alignment from 0.7999 to 0.820770, i.e. **+0.020870**,
and moved the cutoff with it; the residual gap is **+0.092201**, which is
**4.4×** what D6c delivered. The two-blocker framing in this item — "both
fronts are live, emission coverage is necessary" — is correct on its first half
and misleading on its second: emission coverage is *necessary* and it is now
measured to be nowhere near *sufficient*, and no emission-allocation change can
make it so, because **emissions do not move a score**. The binding blocker for
canonical case 2 is the objective, and it is worth +0.0922, not a share of
16,384.

### 7. Why there is no red-before/green-after test here, stated plainly

The task asked for a `tests/` assertion that the alignment is **present** in
`generate_pool`'s output at defaults — red before, green after. There is none,
because the property is not achievable by any change this front is allowed to
make, and a test asserting the alignment's presence would be a test that cannot
pass. Asserting its *absence* is what `tests/emit_coverage.rs` already does
(green, and it remains green), and that file's own comment is right that
asserting the bug is not the same as asserting the property.

The one test added, `the_global_emission_ceiling_is_reached_not_merely_respected`,
is a characterisation, **green before and green after**, and is reported as
such rather than dressed up as a red/green. It exists because the false slack
claim it pins is the load-bearing reason a future change might raise one
mechanism's share on the strength of budget that is not being spent.

`tests/corpus_integration.rs::approximate_finds_classic_madgab_resegmentation`
is **red**, and was red at the base `29d9ad1` before this front's change
(verified by running it on a clean checkout of the base) — it is not a
regression from `ca644e8`/`d751734`, both of which are comment-only plus
`#[cfg(test)]` counters. It stays red because §6 is the reason.

### 8. Validation, exactly as run

On `b0c7d66`, release, `CARGO_TARGET_DIR=/workspace/cargo-target-b3e91a`:

| command | result |
| --- | --- |
| `cargo test --release --lib` | **60 passed, 0 failed** |
| `cargo test --release --test corpus_integration` | **12 passed, 1 failed** — `approximate_finds_classic_madgab_resegmentation`, red at the base too (§7) |
| `cargo test --release --test emit_coverage` | **4 passed, 0 failed** |
| `cargo test --release --test approx_determinism` | **4 passed, 0 failed** |
| `cargo test --release --test exact_determinism` | **1 passed, 0 failed** |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed, 0 failed** |

No test was relaxed, no assertion was weakened, and **no output lock was
re-baselined** — this front changes no printed output, and the two canonical
printed sets are byte-identical to the base because the change is
comment-only. `cargo fmt`, `cargo clippy` and doctests **cannot run on this
host** and are not claimed.

Pool width and wall clock against the baselines, direct release-binary runs at
`--approximate --top 50`:

| target | pool | wall clock | baseline pool | baseline clock |
| --- | --- | --- | --- | --- |
| `It's just a stupid game` | 18,936 | **1.347 s** (load 381 ms, search 776 ms) | 18,824-18,936 | 1.46-1.6 s |
| `recognize speech` | 18,270 | **1.280 s** (load 388 ms, search 743 ms) | 18,242-18,270 | 1.37-1.58 s |

No regression: both pools sit at the top of their bands and both clocks are
*below* the quoted baselines. `MADGAB_TRACE_PHRASES` **does not exist on this
tree** (zero occurrences in `src/` and `tests/`); the accounting in §1 was
taken with the equivalent `ZZACCT` probe on `scratch/b3e91a-probe`, which was
never pushed and never merged.

### 9. Fence sweep, raw hits and per-hit classification

Raw case-insensitive sweep of `src/` for the canonical example and each of its
words. `git grep -i`, no filtering:

| hit | location | classification |
| --- | --- | --- |
| `It's just a stupid game` | `src/lib.rs:5323`, `src/lib.rs:5713` | **pre-existing** `#[cfg(test)]` acceptance-alignment fixture; unchanged by this front |
| `It's just a stupid game` | `src/main.rs:9`, `src/main.rs:11` | **pre-existing** CLI usage examples in module docs |
| `hits`/`justice`/`dupe`/`hid`/`came` | `src/lib.rs:5325`, `src/lib.rs:5714` | **pre-existing** `#[cfg(test)]` fixture |
| `hits`/`justice`/`dupe`/`hid` | `src/lexical.rs:302` | **pre-existing** `#[cfg(test)]` fixture |
| `came`/`stupid`/`game` | `src/lexical.rs:303` | **pre-existing** `#[cfg(test)]` fixture |
| `justice` | `src/lexical.rs:335` | **pre-existing** `#[cfg(test)]` fixture |
| `hits` | `src/approx.rs:473`, `src/approx.rs:511`, `src/approx.rs:512` | **pre-existing** `#[cfg(test)]` fixture / local variable named `hits` |
| `justice` | `src/lib.rs:5326` | **pre-existing** `#[cfg(test)]` fixture |
| `game`/`games` | `src/lexical.rs:371` | **pre-existing** `#[cfg(test)]` fixture |
| `a`, `just`, `its`, `came` | `src/adjacency.rs`, `src/lib.rs` prose | **explanatory comment** — ordinary English ("a wording", "justified", "its own", "came from") |
| `justified` (contains `just`) | `src/lib.rs` — 3 added lines of this front | **explanatory comment**, incidental substring, not the example |

**Zero production hits; zero added by this front.** The multi-word alignment
`Hits Justice Dupe Hid Came` appears in `src/` **0** times outside the
pre-existing `#[cfg(test)]` fixtures, and no `src/` line added by
`ca644e8`/`d751734` contains any word of the example other than the incidental
substring in `justified`.

`ZZ_PROBE_*` / `ZZACCT` / `ZZMAXIDX` / `ZZADJ` / `ZZRESERVE` / `ZZTRAV` /
`ZZADJMIT` / `ZZRES` and `tests/zz_probe_b3e91a.rs` exist **only** on
`scratch/b3e91a-probe` (local, never pushed, never merged). `b0c7d66` contains
none of them; `git grep ZZ_ b0c7d66 -- src tests` is empty.

### 10. What still blocks the milestone, and the next concrete action

**Blocker: the objective, by +0.092201.** Canonical case 2's alignment scores
0.820770 against a printed-50 cutoff of 0.912971 even when emitted. That is
4.4× what D6c delivered and it is not an emission quantity.

**Next action for a coordinator — one line, and it is not a sweep:**

1. **Close this front as a measured negative** and stop re-opening emission
   allocation for case 2. §2 is a factor of 124 and §5 retires all three
   directions with numbers; a fourth round of sweeping this lever has no
   remaining hypothesis to test.
2. **Re-file case 2 against the objective**, at +0.092201 on this alignment,
   in [w-9d4e10](w-9d4e10.md)'s and [w-474813](w-474813.md)'s territory. Note
   for whoever picks it up: `axes::*` weights are still untouched by this
   front, so the objective arithmetic there is still the single un-baselined
   surface for case 2.
3. **Do not wait on w-d1c8f** for this conclusion: the forced-emission
   measurement in §6 is taken on the tree that already contains `33a46f1`, so
   the verdict is post-D6c and does not move if the review's follow-up lands
   differently. If the follow-up *does* change `SIMILARITY` again, §6's
   +0.092201 is the number to re-measure, and §2's factor of 124 is
   unaffected because it is a property of the *order*, not of the scores.
4. **One cheap thing the measurement hands the next front for free:** the
   alignment's structure `[3, 10, 13, 15]` already holds 49 pool members, so
   it is *represented*; `select_diverse`'s representation reserve cannot rescue
   it either, and the share cap is 5. If a future objective change lifts the
   alignment above the cutoff, it will be printed by score order without any
   emission work — which is the strongest available argument that the emission
   half of this milestone was never the binding constraint.

---

## Closure 2026-09-27T06:07Z (coord-05d3)

The front is **done**, and the milestone is **not** reached. Those are two
different statements and this section exists to keep them apart.

`agent-b3e91f` finished at 05:56Z (`succeeded`, exit 0) with a negative,
priced result, committed as `47ad98c` on `madgab-emitbudget-b3e91a` and
pushed by the agent itself (sha-verified). The branch was reviewed here as a
diff and integrated into `post-milestone-acceptance` as `a49fed3`; `main`
untouched. The diff is **docs-only** (`docs/work/items/w-b3e91a.md`,
+269): the front's own source correction of the false "the ceiling leaves
slack" premise, and the test that pins it, were already integrated earlier at
`b0c7d66` as `d751734`, which the agent confirmed.

What is settled:

* the emission-allocation lever is **retired** for case 2, with arithmetic
  rather than opinion: 2,036,664 better-bound wordings of the canonical
  segmentation's own structure precede the canonical tuple, 124x the entire
  global emission ceiling, so every reallocation is a reordering of at most
  16,384 emissions and none of them reaches it;
* the emission ceiling is saturated and the pop ceiling is not, so the
  adjacency operator's stated funding rationale was false; that is corrected
  in the source and pinned by a test;
* the item's literal goal — emissions that reach deep per-slot indices — was
  already met and already asserted by the green
  `depth_profile_emissions_reach_deeper_than_the_traversal`;
* emission is **necessary and nowhere near sufficient**: forced onto the
  unchanged production path the alignment scores 0.820770 at pool rank
  9,668/18,937 against a 0.912971 printed-50 cutoff, a gap of **+0.092201**
  and a rank deficit of 9,618 places. Emissions do not move a score.

The second premise this item was opened on is therefore also retired: "a
candidate that is never emitted cannot be reweighted into the pool" is true
but no longer the operative constraint, because no permitted emission
allocation can emit it. **The binding blocker for case 2 is the objective**,
and it is re-filed as [w-7e1a04](w-7e1a04.md), opened and claimed on
`e542af5` at 06:02Z.

Do not re-open emission allocation for this clue. The next direction has to be
argued from the semantics of the score's form, not from another sweep.
