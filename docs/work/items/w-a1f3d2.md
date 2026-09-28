---
work_item: true
id: w-a1f3d2
state: done
priority: high
owner: null
updated: 2026-09-27T10:05:00Z
branch: madgab-opening-a1f3d2
worktree: /workspace/madgab-opening-a1f3d2
opened_by: antonina front a1f3d2 (coordinator pass 09:44Z, on post-milestone-acceptance at ecebf2a)
source_item: w-5e2d41 (both fronts terminal priced negatives) / w-9e2b41 (closed width front)
base: ecebf2a
---

# The opening-budget front, priced: the canonical tuple is emittable and still ranks 12,110th, so the scoring axis is the binding constraint

## Verdict

**HOLD.** Two priced negatives, and the second one re-scopes the whole milestone.

Zero production lines land. `git diff ecebf2a -- src/ tests/` is **empty**; the
branch carries this record and nothing else. No cutoff, budget, threshold or
test was re-pinned; no baseline was re-taken.

## Goal (as commissioned)

Price the surface neither terminal front touched: **how the traversal's
pop/affordability budget is charged at the opening — jointly across the whole
candidate vector versus per branch, per slot or per depth** — and the general
rank/band retention rule that follows from charging it differently.

## What I measured first, on `ecebf2a`, before editing anything

`madgab --approximate --top 50`, release, defaults, corpus `CORPUS_JSON`.

| target | result |
| --- | --- |
| `recognize speech` | `wreck a nice beach` present, **rank 27 of 50**, score `0.9199502875` |
| `It's just a stupid game` | `Hits Justice Dupe Hid Came` **absent**; top-1 is `it said thus test oop day` at `0.9214042124` |

`cargo test --release --test corpus_integration -- --test-threads=2`:
**12 passed / 1 failed**, the one failure the pre-existing
`approximate_finds_classic_madgab_resegmentation`. That is the pristine
baseline; it is not re-pinned and is not claimed as fixed.

Measurement instrumentation lived only in a throwaway worktree
`/workspace/a1f3d2-probe` on the un-pushed `scratch/a1f3d2-probe` branch, and
that worktree and branch have been **deleted**. Nothing from it reaches this
branch: no `#[cfg(test)]` counter, no `ZZ_*` symbol, no environment knob, no
probe binary under `examples/`.

## Negative 1 — the canonical tuple is emittable, and the opening budget is *not* what hides it

This is the load-bearing result, and it is new: **every prior front on this
fence measured only whether a word could be *reached*. On this base, reaching
it is worth nothing.**

Target spans are indexed over the 19-character normalisation
`itsjustastupidgame`. Instrumented at the public search boundary, the
canonical clue's own structure is **retained** and its tuple is **legal and
payable**:

```text
structure  [[0,3],[3,10],[10,13],[13,15],[15,19]]   (schedule phase 151,
                                                       widths [160,7,160,160,93])
tuple      (7, 0, 13, 99, 11)          slots: hits / justice / dupe / hid / came
phrase     "hits justice dupe hid came"
score      0.8207695329
```

So the span chain exists, the structure survives `SEGMENTATION_KEEP`, the
tuple is inside every slot, its total substitution cost is inside
`total_budget`, and `build` accepts it. **It is one `popped += 1` away from
being in the pool.**

The traversal does not reach it, and the measurement of *why* is the same
arithmetic the closed fronts already priced, re-measured on this base:

* traversal `cap = affordable_opening_width(5, 4_000) = 7`, and the walk's
  test is `index < cap`, so slots 0/2/3/4 at indices 7/13/99/11 are all
  excluded. Measured over the whole run for this structure: 50 traversal
  emissions, **max index per slot 6, 0, 3, 5, 6**.
* the coverage reserve's 14 draws for this structure examine
  slot-3 indices `{31, 39, 49, 53, 68, 88}` (needed 99),
  slot-0 `{11, 31, 51, 61, 67, 117}` (needed 7),
  slot-2 `{10, 21, 43, 47, 58, 73, 81}` (needed 13),
  slot-4 `{13, 17, 29, 33, 35, 39, 45}` (needed 11). None lands on a needed
  index.

Both barriers are real. **Neither is the binding one.**

## Negative 2 (decisive) — the clue scores 0.0405 below the 50-slot cutoff, which is 12,060 places

The pool, after `finish`'s signature dedup, is **22,784 clues**. Score-sorted, the
best is `it said thus test oop day` at **`0.9214042124`** and the **50th-highest**
is `it said thus tas too gave` at **`0.9157359250`**.

```text
canonical tuple score                     0.8207695329
50th-highest score in the pool            0.9157359250
gap                                       -0.0949663921
width of the whole top-50 score band      0.9214042124 - 0.9157359250 = 0.0056682874
gap as a multiple of that band             16.75x
clues scoring >= the canonical tuple      12,110 of 22,784
=> descending rank of the canonical clue  12,110
```

The gap is **16.75 times the entire width of the top-50 score band** — the
visible list spans 0.0057 of score and the deficit is 0.0950. This is not a
ranking near-miss that better selection could absorb. The best canonical-family score found anywhere in the lattice is
`0.8421702452` (`sit justice dupe hid came`, a different slot-0 reading), so
the whole family is short.

**Consequence: no traversal, reserve, width, budget or retention change can
turn this guard green, because the clue that such a change would deliver lands
at rank 12,110.** The measured deficit is on the scoring side, not the
emission side. This retires the framing shared by `w-5e2d41` front A, front B,
`w-9e2b41` and `w-c3f81a` for this fence — all four are priced against
*reaching a word*, and reaching it is worth 12,060 places of nothing.

Why the score is low, measured rather than guessed: the `SIMILARITY` axis
charges **per-slot** substitution cost, and the canonical reading pays that
twice for one real match. `stu` → `dupe` and `pid` → `hid` are each expensive
alone, while the concatenation `dupe hid` is a near-exact match for `stupid`.
Substituting one slot changes the reading's score materially — the same
structure with `ack` in slot 3 scores `0.8414528536` against the canonical
tuple's `0.8207695329` — so the per-slot cost term is demonstrably the driver.

## The charging question, priced

`LEXICAL_HEAP_POP_LIMIT = 4_000` is charged **jointly, once, for the whole
candidate vector**: the traversal increments `popped` per popped node and
tests it against one scalar. The traversal's real cost is the per-slot
frontier `F(c) = 1 + c_0 + c_0*c_1 + ... + c_0*...*c_{d-2}`, which does **not**
include the last slot.

**Finding 1 — the joint cap is genuinely required, not routable around.** `F`
is exactly the number of `seen`/`heap` entries a pass allocates, and
`spent_pops` is a share of the global pop budget. A per-branch, per-slot or
per-depth re-booking of the same 4,000 changes *accounting*, not *reach*:
`F` is invariant to how the pops are booked, and `cap` is read off `F`. It
would multiply total work by up to `d` while leaving the live heap and the
reachable index set exactly where they are. This is the same category of
inadmissible traversal quantity already priced in `w-5b1e93` and `w-9e2b41`,
seen from the booking side. **A joint cap is required; that is the finding,
and it is recorded rather than engineered around.**

**Finding 2 — the per-slot *marginal* the closed width front used is the wrong
marginal, and the true one is much larger.** `w-9e2b41` charges one extra
candidate in slot `k` at `M_k = c_0*...*c_{k-1}`, the node-count product. That
understates the frontier's own marginal: raising `c_k` by one lengthens *every*
frontier term from `k` on, so the true delta is

```text
delta(k) = (c_0*...*c_{k-1}) * SUM_{j=k}^{d-2} (c_{k+1}*...*c_j)
```

For slot 0 at depth 5 with uniform 7 that is `1 + 7 + 49 + 343 = 400`, i.e.
**57× the `1` the node-count view charges**, and it is why the uniform-7
vector sits at `F = 2,801` and not at 8. This correction is arithmetic only
and lands nothing, but any successor that re-derives a per-slot width from
this budget must use the sum-of-products, not the single product, or it will
overrun the joint cap. A first implementation of exactly that rule, measured
here, **exceeded 4,000 at 11 of 240 real shapes with a maximum frontier of
4,551** before the correction was applied — the overrun is a live hazard, not
a theoretical one.

**Finding 3 — a per-slot reallocation is feasible but still buys nothing.**
Widening under the corrected delta, keeping the joint cap as the outer bound,
is affordable: at depth 5 the uniform-7 vector has `F = 2,801` and the spare
`1,199` goes to whichever slot has the smallest corrected marginal, giving
vectors such as `[7,7,7,10,*]` and `[7,2,27,9,*]`. Wired in and measured
end-to-end: **case 1 stays green at rank 27** (`wreck a nice beach`,
`0.9199502875`) and **case 2 stays red** (canonical count 0). The rule reaches
`hits` at rank 7 and nothing else, because the tuple's own minimum sub-lattice
costs `2.83x` — `w-9e2b41`'s figure, which this front confirms and does not
re-derive.

## Retention rule: what a rank/band floor would have to do here

A rank or band floor on the span shortlist is the right *shape* of rule (the
cost-ratio form is already refuted, vacuous at 1.5× on 0 of 885 spans), and on
this base the ranks it must hit are measured: slot 0 needs 7, slot 2 needs 13,
slot 3 needs 99, slot 4 needs 11, against a `cap` of 7 and a reserve that
examines `{31,39,49,53,68,88}` in slot 3. A floor that guaranteed coverage
across the traversal's *own* index order — not the cost order — would place
those ranks inside the reachable region.

**It would still not turn the guard green**, because of Negative 2. The
retention surface is real and worth a later front for *pool quality*; it is
not on the critical path for *this* fence.

## Successor rule, one line

> Stop pricing *reachability* of a clue and price its **score**: the
> substitution cost must be able to see a resegmentation that moves a word
> boundary as **one** event — score the concatenated clue IPA against the
> target IPA, or admit a cost for a pair of adjacent slots bounded by that
> pair's own phonetic distance — so that a reading which splits one target
> word across two clue words is charged once instead of twice, and a clue the
> lattice already emits stops ranking 12,000 places below the visible band.

## Completion criteria for this front

1. Reproduce the two canonical targets and the decisive numbers **before**
   editing: done, §"What I measured first".
2. Reproduce the red guard exactly as the predecessors recorded it, and state
   the measured baseline: done — 12/1 on `corpus_integration`, case 1 green at
   rank 27, case 2 red.
3. Price joint-versus-per-slot charging at the opening, and a per-branch or
   per-depth budget with a derived total keeping the joint cap as an outer
   bound: done, Findings 1-3. The joint cap is required; that is the finding.
4. State what a general rank/band retention floor would have to look like:
   done, with the four needed ranks measured.
5. Report exactly one of INTEGRATE or HOLD with the priced negative: **HOLD**.

## Verification actually run on this branch

`src/` and `tests/` are byte-identical to `ecebf2a`.

| command | result |
| --- | --- |
| `cargo build --release` | Finished, 24.02 s, no warnings shown |
| `madgab --approximate --top 50 "recognize speech"` | `wreck a nice beach` at rank 27 |
| `madgab --approximate --top 50 "It's just a stupid game"` | canonical absent; top-1 `it's justice too bad same` |
| `cargo test --release --test corpus_integration -- --test-threads=2` | **12 passed / 1 failed** (the pre-existing blocker) |

**A verification hazard this front hit, recorded because it will bite the next
one.** Re-using one `CARGO_TARGET_DIR` across two worktrees of the *same
package name* is not safe: the probe worktree (instrumented, different source
path) and this worktree share the artifact directory, and the deliverable
tree's test binary was then linked against the probe's library. It
manifested as a **phantom extra failure** — `corpus_integration` reported
`11 passed / 2 failed`, with `approximate_pool_reaches_matches_deep_in_a_span`
red and the run taking 139 s against a normal 28 s — on a tree whose `src/`
and `tests/` were byte-identical to `ecebf2a`. Re-run at pristine `ecebf2a`
in a clean worktree with its own target dir, that test passes in isolation
(6.76 s) and the suite is **12 passed / 1 failed**. Give each worktree its own
target directory, or a probe build will silently invalidate a fence.

`cargo fmt`, `cargo clippy` and doctests **cannot run on this host** (see
[../environment-notes.md](../environment-notes.md)) and are not claimed.

The per-slot allocation of Finding 3 was built and measured in the throwaway
probe worktree, never in this branch, and is reported as a measurement rather
than a candidate.

## Next action for a fresh pass

Do **not** open a fourth front on reachability for this fence. The measurement
above is the reason: the deliverable ranks 12,110th. Open the scoring front
instead, and treat `w-c3f81a` and the retention-rule question as pool-quality
work rather than as blockers for the canonical case-2 clue.
