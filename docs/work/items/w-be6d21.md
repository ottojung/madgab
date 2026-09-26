---
work_item: true
id: w-be6d21
state: done
priority: high
owner: agent-be6d21
updated: 2026-09-26T15:55:00Z
branch: madgab-budget-envelope
worktree: /workspace/madgab-budget-envelope
---

# Measure the budget lever: what an 8x global emission budget buys, and what it costs

## Goal

Turn the last untried milestone lever from an arithmetic claim into a
measured envelope, so the coordinator can decide it with numbers instead of
a ratio.

[w-8f3c61](w-8f3c61.md) §8 closes the enumeration-order family in closed
form: the canonical clue `hits justice dupe hid came` for `It's just a
stupid game` is **134,400 emissions of its own segmentation deep** in the
best conceivable stratified order and **> 404,081 emissions / > 1,000,000
pops deep** under the order actually used, against a whole-search
`LEXICAL_GLOBAL_EMISSION_BUDGET` of **16,384** and
`LEXICAL_GLOBAL_POP_BUDGET` of **1,024,000**. Its §8.3 then names exactly two
remaining levers: change what the enumeration optimises
([w-3b8e15](w-3b8e15.md), the cost model, running), or **grow the budget
~8x**.

The second lever has never been measured. §8 explicitly deferred it: "raising
the total to ~8x is a budget decision that belongs to the coordinator, not to a
front fenced out of the budget." This item is that measurement, so the
decision is informed rather than deferred again.

This is a **measurement** item. It does not change shipped behaviour.

## What to measure

From the release binary, on the real approximate path, with only the budget
constants varied:

- global emission budget at **1x, 2x, 4x, 8x, 16x** of 16,384 (and the pop
  budget scaled alongside, since emissions alone may be capped by pops);
- for each point: **search wall clock**, **candidate-pool size**, and whether
  `hits justice dupe hid came` is **enumerated** and **ranked in the printed
  top 50** for `It's just a stupid game`;
- the same on at least three further real targets, including
  `recognize speech` and `congratulations on your promotion`, so the cost is
  not read off one input;
- whether the milestone's *other* half, `wreck a nice beach` for
  `recognize speech`, survives each budget point, and whether
  `approximate_output_is_locked` still holds at 1x (it must — that is the
  baseline, not a variable).

Baseline for wall clock: ~2.0 s search + ~419 ms corpus load for
`--approximate --top 20` on `It's just a stupid game`.

## Completion criteria

- A table of budget multiple x {emission budget, pop budget, search wall
  clock, pool size, canonical clue enumerated?, canonical clue in top 50?} over
  the target set, measured from the release binary, reproducible from the
  commands shown in the report.
- An explicit statement of the **cheapest budget multiple at which the
  canonical clue is enumerated**, and of whether it is *ranked* into the
  visible top 50 at that point or merely enumerated-and-cut (those are very
  different outcomes, because the score cutoff of ~0.9 that it is measured
  against, 0.819901, is a separate question from reach).
- If the clue is enumerated but never ranked, say so plainly with the cutoff
  arithmetic, because that result would relocate the milestone from a budget
  problem to a scoring problem and [w-04f83f](w-04f83f.md) /
  [w-2e5b93](w-2e5b93.md) are the items that own scoring.
- A stated verdict: is the 8x lever **affordable**, **not affordable**, or
  **affordable but insufficient**, with the number that decides it. If it is
  affordable-but-insufficient, that is a real result and closes the budget
  family for good.
- **No phrase-specific hard-coding**: no special case for any word, substring
  or word sequence of the acceptance examples, in anything committed.
- Committed work is **documentation only** (`docs/work/budget-envelope.md`),
  on branch `madgab-budget-envelope`, pushed, tree clean.

## Fences

- **Do not commit any `src/` change.** Varying the budget constants locally to
  take the measurements is expected and necessary; committing it is not. The
  report is the deliverable, so that the coordinator can decide the budget
  question in a later pass without inheriting an unreviewed constant change on
  a fourth branch that also touches `src/lib.rs`.
- Not yours and not to be touched: the traversal order, the width schedule,
  the per-slot list contents, the cost model, and any scoring or `axes::*`
  change. Those belong to [w-3b8e15](w-3b8e15.md), [w-9d4e17](w-9d4e17.md)
  and [w-e07c42](w-e07c42.md), all running concurrently.
- No `zz_*` test, sweep script, `.bench/` tree, or `MADGAB_*` diagnostic
  left on the branch. Scratch work and sweep scripts belong in `/tmp`.
- `cargo fmt`, `cargo clippy` and doctests do not exist on this host; see
  [../../environment-notes.md](../../environment-notes.md). Do not report them
  as satisfied.
- A refutation with numbers is an acceptable and useful outcome. Do not
  manufacture a favourable number, and do not recommend a budget the
  measurements do not support.

## Handoff / notes

Filed by coordinator pass `coord-5e22` on 2026-09-26T13:20Z, from the gap
named in [w-8f3c61](w-8f3c61.md) §8 and §8.3, which is also recorded in the
umbrella [w-4b1e07](w-4b1e07.md). The rationale for running this *beside*
the cost-model front rather than after it: the two levers are independent, the
budget envelope is a coordinator decision that needs a number whichever way
the cost model turns out, and taking the measurement now means that if
[w-3b8e15](w-3b8e15.md) refutes the cost model, the next pass already knows
whether the budget lever is worth anything or whether the milestone is
genuinely unreachable without a scoring change.

Ownership: branch `madgab-budget-envelope`, worktree
`/workspace/madgab-budget-envelope`, created from `origin/post-milestone-acceptance`
at `aac97e1`. Commit early and often; a finished agent's work on a detached
HEAD is unrecoverable from a later pass's point of view.

## Close-out (coordinator coord-4d92, 2026-09-26T15:55Z)

Agent `be6d21` succeeded (exit 0). The documentation-only fence held: the
branch's whole diff against `aac97e1` is one added file,
`docs/work/budget-envelope.md` (431 lines), with `git diff HEAD -- src/` empty
at every commit, and the tree clean. Reviewed and merged into
`post-milestone-acceptance` as **`89c85ff`**; branch `madgab-budget-envelope`
is durable on `origin`.

All completion criteria in the item are met: the table is reported per budget
multiple over five real targets, reproduced from the commands in the document;
**enumerated** and **ranked** are reported separately per row; the
enumerated-but-cut case is stated plainly; the verdict is
**affordable-but-insufficient** with the deciding number; and no constant is
landed. The criterion "cheapest multiple at which the canonical clue is
enumerated" has a real answer — **there is none up to 256x**, because the
global budget is already at its maximum reachable value at 1x, which is a
stronger result than the 8x question asked for.

Unverifiable on this host and therefore **not** claimed: `cargo fmt --check`,
`cargo clippy`, doctests and the `wasm32` build — see
[../../environment-notes.md](../../environment-notes.md). No `src/` behaviour
changed, so no Rust test result is at stake for this item.

The verdict is carried into the umbrella [w-4b1e07](w-4b1e07.md): the budget
family is closed for good, the binding constraint is per-slot width
(`LEXICAL_BRANCH_KEEP = 10`, [w-9d4e17](w-9d4e17.md)), and the milestone is
reach-then-rank, in two different places. Handoff item 4 (a one-line comment
recording that the global budgets are non-binding) is left to
[w-3b8e15](w-3b8e15.md), which owns `src/lib.rs`.
