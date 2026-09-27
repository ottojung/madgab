---
work_item: true
id: w-b7d4c1
state: working
priority: normal
owner: coord-4e19 (claimed 2026-09-27T21:13Z on post-milestone-acceptance at c9bf807; front agent-b7d4c1 launched 21:13Z in /workspace/madgab-covmod-b7d4c1 [madgab-covmod-b7d4c1])
updated: 2026-09-27T21:13:00Z
branch: madgab-covmod-b7d4c1
worktree: /workspace/madgab-covmod-b7d4c1
opened_by: coord-4e19 (reconciliation pass 2026-09-27T21:11Z-21:15Z)
source_items: docs/work/REPORT-2f1c03.md §3 and its closing successor rule (the "one non-duplicative residue", P1: the per-member modulus in `coverage_tuples`), docs/work/OBSTRUCTION-MAP.md §3 ("The one non-duplicative residue the sweep report found - the per-member modulus in `coverage_tuples` - is a later, non-blocking front on the reserve side and must be decided under the same head-lift criterion, never as a reach item"), docs/work/items/w-3f8c62.md constraint 7
predecessors: w-2f1c03 (done, priced the reserve sweep reach-null by 5-6 orders of magnitude and left P1 as the residue), w-c3f81a (done, reserve placement), w-3f8c62 (working, concurrent and independent: it owns the objective/bound surface only and is explicitly fenced away from this one)
base: post-milestone-acceptance at c9bf807 (pushed)
---

# Land the per-member sweep modulus in `coverage_tuples`, decided as a head-quality item

## Goal

Ship **P1** from [REPORT-2f1c03.md](../REPORT-2f1c03.md) §3: in `coverage_tuples` (`src/lib.rs`),
index each subset member against **that member's own slot width** instead of against the
**narrowest** member's span, so the coverage reserve's draws stop being a function of the other
coordinates through a shared modulus. The report prices it at **zero** extra traversal pushes, zero
extra heap entries, zero extra emissions, zero extra frontier and zero extra bound evaluations;
per-member index arithmetic is unchanged in cost. It is a general, phrase-free coverage
improvement, not a case-2 reach fix, and it must be judged as one.

## Why this front now

`OBSTRUCTION-MAP.md` §3 is now empty: case-2 **reach** is closed as a search-side question
(shapes 1, 2 and 3 all priced negative). The map names exactly one non-duplicative residue left on
the reserve side, and this is it. The concurrent front [w-3f8c62](w-3f8c62.md) owns the
objective/bound surface and is fenced off from this one by its own item, so the two do not
contend: this front touches the reserve's tuple construction, that one touches the score and its
structural bounds.

## What this front must do

1. **Re-derive the baseline on integrated HEAD, release mode**: pool size, shipped top-50
   `SIMILARITY` lift against the pool mean for both canonical targets (**−0.0180** case 2,
   **+0.0609** case 1), and `wreck a nice beach`'s pool rank for `recognize speech` (**27**).
   Record any disagreement with those figures rather than proceeding on them.
2. **Implement P1 (+P2 if it is free on the same accounting, as the report says it is)** confined
   to `coverage_tuples`'s per-member call: pass `slot_widths[slot]` rather than `narrowest` to
   `sweep_index`, and take `sweep_rate`'s modulus from the member's own span.
3. **Add the two phrase-free property tests the report already specifies**:
   (i) *no member of a subset is assigned an index at or above that member's own slot width, for
   every subset at every phase*, with the expectation recomputed inline from the widths rather than
   by calling the function under test; and (ii) *the deep-coordinate set of a `k`-deep subset is
   not confined to any 1-parameter family modulo the members' spans* - a single coordinate may no
   longer be a function of the others by an affine map with a shared modulus.
4. **Decide under the head-lift criterion, and report the reach arithmetic too.** The success
   signal is head quality (top-50 lift off −0.0180 for case 2) with the green case still inside
   the top 50 (pool rank 27 reported before and after), plus the three
   `approximate_pool_reaches_*` guards. The report must also state, with numbers, whether the
   canonical 4-deep cell became more likely (the report predicts **unrepresentable → 1.2e-7**,
   still nowhere near reachable) so nobody later mistakes this for a reach fix.
5. **Fence runtime and memory**: the reserve must stay bounded by `out.len() >= reserve`, the
   emission ceiling must stay saturated-or-under with no new emission, and the resident frontier
   must not move. A wall-clock, pool-size or memory regression beyond measurement noise is a
   reason to report `HOLD`, not a reason to raise a budget.
6. **Say how it interacts with [w-3f8c62](w-3f8c62.md)** in one paragraph: that front changes score
   and bounds only, so the two should compose; if the measured head lifts interact, say how.

## Constraints

* **No phrase-specific hard-coding.** No sentence, clue, word list, per-input branch or special
  case for `hid`, `dupe`, `hits`, `came`, `justice`, `wreck`, `beach`, `recognize` or any other
  literal token. `no_phrase_hard_coding` must stay 9/9 and no new example token may appear in
  `src/` outside a test module. That test is a hard fence.
* **Reserve tuple construction only.** No change to scoring, to `LEXICAL_HEAP_POP_LIMIT`,
  `SPAN_SHORTLIST`, `SPARSE_SHORTLIST`, `LEXICAL_COMBINATIONS_PER_SEGMENTATION`,
  `EMIT_PROFILE_RESERVE`, the emission ceiling, the depth cap, any width table, or the walk's
  admission rule (`index < cap`). Those are map rows 1-13 and are priced.
* **Instrumentation** stays behind `#[cfg(test)]` or in `examples/`/`tests/`. No `ZZ_*` probes, no
  environment knobs, no baseline changes.
* Release mode only for measurement. `cargo fmt`, `cargo fmt --check`, `cargo clippy`, doctests
  and `wasm32` builds **cannot run on this host** — see
  [../environment-notes.md](../../environment-notes.md). Do not claim them. `git commit` needs
  `-c commit.gpgsign=false`; the fetch refspec is narrowed, so push child branches with an
  explicit refspec.
* End with exactly one of `INTEGRATE` (a real production change plus the two property tests plus a
  green tree) or `HOLD` (decomposition and pricing only, a full success, with the numbers).
* **No self-merge.** Push the focused branch and stop; a later coordinator pass reviews and
  integrates onto `post-milestone-acceptance`. **`main` is never touched.**

## Independence

* `w-0f3a17` and `w-4b1e07` carry **DO-NOT-RE-TAKE**.
* Closed or priced-negative items: `w-1c3e77`, `w-1c7d40`, `w-3a8c05`, `w-3e91a4`, `w-4d7c12`,
  `w-5d9c04`, `w-5e2d41`, `w-5e2d42`, `w-7b40d2`, `w-5b1e93` (`superseded`), `w-9e2b41`,
  `w-c3f81a`, `w-d4e8b1`, `w-2f1c03`, `w-9b4a15`, `w-2c9d41`. Read their records; do not re-run
  their hypotheses or re-pin their red.
* The red to preserve, not to re-pin: `approximate_finds_classic_madgab_resegmentation` (case 2) is
  **red at base**. `corpus_integration` is expected at 12 passed / 1 failed with
  `-- --test-threads=2`; it is SIGKILLed at default parallelism on this host even on pristine base.

## Completion criteria

1. The four baseline figures re-derived on integrated HEAD, or a disagreement recorded with both
   numbers.
2. P1 landed, or a priced argument for `HOLD`, with the report stating zero/positive cost in
   pushes, heap entries, emissions, frontier and bound evaluations.
3. The two phrase-free property tests present, red before the change and green after, with no
   example token in the assertion.
4. Head-lift numbers before/after for both canonical targets, `wreck a nice beach`'s pool rank
   before/after, and the three `approximate_pool_reaches_*` guards reported.
5. `cargo test --release --lib` green on the front's branch head (or the failures named against
   base), and the report pushed on `madgab-covmod-b7d4c1`.
6. The branch is pushed and un-integrated; `main` untouched.

## Notes / handoff

Opened by coordinator pass 21:11Z-21:15Z. No prior work on this surface exists: the residue was
first named in `REPORT-2f1c03` and explicitly deferred. If the front reports `HOLD`, the successor
rule is the report's own closing line, and the next front must be a different surface - §3 of the
obstruction map has nothing else left, so a `HOLD` here likely means the milestone needs a human
decision about scope rather than another search-side front.
