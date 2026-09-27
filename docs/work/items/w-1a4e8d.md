---
work_item: true
id: w-1a4e8d
state: working
priority: high
owner: coord-2c8d (opened and claimed 2026-09-27T22:47Z on post-milestone-acceptance at f9944df; front agent-1a4e8d launched 22:47Z in /workspace/madgab-reserve-1a4e8d [madgab-reserve-1a4e8d] and left running)
updated: 2026-09-27T22:47:00Z
branch: madgab-reserve-1a4e8d
worktree: /workspace/madgab-reserve-1a4e8d
---

# Measure the *partial* reserve case REPORT-7c9d21 explicitly declined to claim

## Goal

Turn one **argument** in the integrated record into a **measurement**, so that a later pass knows
whether the printed-set gate can be partially relieved rather than only fully removed.

`REPORT-7c9d21.md` §3 prices the post-pool selection layer's available lift at **+0.007805**
printed top-50 SIMILARITY and concludes none of it is collectible, because the pure-score list
spans **6 / 2 / 4** structures against `approximate_list_represents_enumerated_resegmentations`'s
required **12** (`tests/corpus_integration.rs:502`). The report flags its own weak joint itself, and
the flag is the reason this item exists:

> I priced the slack by removing the layer wholesale, which is an upper bound. The step from there
> to "no partial recovery exists" is an argument, not a measurement.

So the honest question is not "does the whole reserve come out" (answered: no, fence red) but
**how much of it has to stay for the representation fence to hold, and what is the printed top-50
cost of exactly that much.** Concretely, the reserve size is `structure_reserve_slots(50) = 12`
(`src/lib.rs:4077-4079`, divisor 4 at `src/lib.rs:4087`) and the reserve draws from as deep as pool
rank **227**, which is what pushes the `top_n` cut at `src/lib.rs:4025` down to pool rank **42**
instead of 50. Measure the *sweep* of intermediate reserve sizes — not just the two endpoints 12
and 0 — and find the largest reduction that keeps the representation fence green.

## Context

This is a **measurement-only** front. It changes no production line and is not a candidate for
`INTEGRATE`; its deliverable is a table and a successor coordinate. It exists because
`w-7c9d21` closed with the caveat above unresolved, and leaving it unresolved invites a later pass
to re-derive the whole selection layer or, worse, to re-baseline the representation fence on the
strength of an argument.

**Non-contention is mandatory.** This front must not touch either live front's surface:

- [w-e086cc](w-e086cc.md) (`agent-e086cc`, `/workspace/madgab-e086cc`) varies the **objective
  weight vector** and classifies fences F0–F4 against the walk's heap key.
- [w-3f6a21](w-3f6a21.md) (`agent-3f6a21`, `/workspace/madgab-pairscore-3f6a21`) works the **score
  function** (adjacent-slot substitution charge).

This front holds the **shipped objective and the shipped score function fixed** and varies only the
representation reserve size, which is a selection-layer parameter. If at any point the sweep seems
to require editing the weight vector or the scorer, stop and report that the surfaces collide.

Note also that `structure_reserve_slots` is shared machinery. Before varying it, establish from
`src/` whether the divisor it takes is used by any path other than approximate printed selection;
if it is, the front must vary a selection-local parameter instead and say why, rather than widening
a shared constant's blast radius.

## Completion criteria

1. A table of printed top-50 outcomes over the reserve size, covering at minimum the endpoints 12
   and 0 plus every intermediate value the sweep runs, with for each row: reserve size, the
   structure-count triple the representation fence measures, whether that fence is green, the green
   canonical case's printed rank, and the pool rank of the pool-resident withheld candidate
   `agent-e086cc` calls F7 (rank 48 at the shipped objective).
2. A decisive statement of the **largest** reserve reduction that keeps
   `approximate_list_represents_enumerated_resegmentation` green, and the printed top-50 cost of
   exactly that reduction measured, not estimated.
3. A statement of whether the partial case changes the closure verdict of `w-7c9d21`, with the
   number that settles it. If it does not change the verdict, say so plainly and do not open a
   successor front on this surface.
4. Regression-free by construction: no production, test, example, or `Cargo.toml` line changes
   reach `post-milestone-acceptance` from this front. Any probe lives on a `scratch/` branch or in
   `#[cfg(test)]` and is reverted before the fence suites are run.
5. All fence suites green at the front's base, with the known pre-existing base red in
   `corpus_integration` left untouched and **not** re-pinned: `no_phrase_hard_coding` 9/9,
   `emit_coverage`, `approximate_output_is_locked`, both determinism suites. Report `fmt` and
   `clippy` honestly if they cannot run on this host — do not claim what you did not run.
6. The report is pushed to `madgab-reserve-1a4e8d`. Never self-merge into
   `post-milestone-acceptance`; never touch `main`; no sentence, clue, word, or exact rank from the
   canonical examples special-cased in code.

## Handoff / notes

Opened by the reconciliation pass `coord-2c8d` at 22:47Z, immediately after `w-7c9d21` closed at
`f9944df` as a priced negative. It is deliberately the *smallest* useful front available: it asks
one numerical question that an integrated report already flagged as unmeasured, on the one
parameter that report identified, and it is orthogonal to both running fronts.

Host at open: load 7.7 of 32, 10 GB memory available, so this front is not contending for
compilation resources. Use a dedicated `CARGO_TARGET_DIR` and `--test-threads=1`; do not enter
`/workspace/madgab-e086cc` or `/workspace/madgab-pairscore-3f6a21`.

`REPORT-7c9d21.md` is integrated on this branch at `2f8e5c6` and is the required reading.
