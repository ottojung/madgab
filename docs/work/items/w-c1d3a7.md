---
work_item: true
id: w-c1d3a7
state: working
priority: high
owner: agent-c1d3a7
updated: 2026-09-26T19:40:00Z
branch: madgab-adjacency
worktree: /workspace/madgab-adjacency
---

# Reach a locally-expensive slot by one-step substitution from candidates already found, not by traversal depth

## Goal

The approximate search only ever proposes wordings that its best-first
traversal reaches by *cumulative* cost. That is why a wording which is
*jointly* excellent but locally expensive in a single slot is unreachable at
any affordable budget: reaching it costs the sum of every better-bound node
in front of it, not the cost of the one step that differs.

Replace or augment the traversal with an **adjacency operator**: take wordings
already emitted into the pool and, for each slot, substitute that slot's next
alternative, re-score the result, and admit it to the pool. Substituting one
slot is a single-slot cost, so depth in one slot becomes *additive* in the
number of substituted slots rather than multiplicative in the traversal's
levels, and a node that is deep in exactly one slot is reachable in one step
from the node that is at index 0 in that slot.

This is a different algorithmic *shape* from every closed front (see the
fence table below): none of them changed what a single edge of the search
costs.

## Why this and not something else: the measurements already on record

- [w-3e5c7](w-3e5c7.md), integrated as `24f3cf7`: with the canonical
  alignment **forced to index 0 of all five slots** and that one
  segmentation's caps lifted, it sits at emission 23,962 / pop 28,930 of its
  own segmentation, at structural rank 151 of 256. The forced run *did* reach
  the clue. So the wording is a real, reachable, in-pool-quality candidate;
  only the *path* to it is expensive.
- [w-8f3c61](w-8f3c61.md) §8, from a confirmed build: by cost-best-first
  order the canonical resegmentation needs **>= 404,081 emissions** of its own
  segmentation, and the cheapest depth-profile stratification is bounded
  below by **134,400 emissions**, against a per-segmentation allowance of
  **64**. §8.3 named exactly two remaining levers: change what the enumeration
  optimises, or grow the budget ~8x. Both are now closed —
  [w-3b8e15](w-3b8e15.md) refuted the cost model by measurement (a rebuilt
  articulatory cost lowered the clue's price 3.07x and moved *nothing*, then
  cost the one canonical case that worked), and [w-be6d21](w-be6d21.md)
  measured the global budget inert across 1x..256x.
- `5f1c04`'s in-flight measurement on
  [w-5f1c04](w-5f1c04.md) (`ZZ_DEEPCAP` = 65,536 / 131,072 / 262,144 with the
  global emission ceiling genuinely raised): a single boundary structure
  funded to a depth of 65,536 emits **79,711** candidates in the whole search
  and 131,072 emits **80,367** — and **the canonical clue is still not
  reached at any of them**. The one structure's traversal itself saturates
  near 66,176 emissions while the per-slot product is on the order of 10^9,
  and the stage-0 width (`LEXICAL_BRANCH_STAGE_0 = 10`) is the reason. So
  "fund one structure deeply" is being refuted in flight, and the residual is
  the *shape* of the traversal, not its budget.
- [w-9d4e17](w-9d4e17.md), integrated as `4c2200a`: the staged width schedule
  widens only when the traversal drains, and its own record says the widening
  fires **0 times** on multi-clause targets. The canonical target is
  multi-clause. So the adjacency operator is needed precisely where the
  widening does not fire.

## Why this is general

An adjacency operator over an emitted candidate set is a standard
neighbourhood move. It has no phrase in it: it is a rule about slots and
alternatives, so it applies to every input. It is the natural complement to
the integrated depth-profile reserve, which buys *coverage of the index-tuple
space* from the same traversal: the reserve widens coverage of the cheap
corner, adjacency walks outwards from candidates that already exist. A
refutation with numbers is an acceptable and useful outcome — several
families here have closed that way, and this repository records them.

## Fences

- Closed families, not to be reopened: enumeration **order** (w-8f3c61),
  **budget size** (w-be6d21), **budget definition / per-segmentation
  funding** (w-5f1c04, w-6b2f04), **per-slot truncation point**
  (w-9d4e17), **emission selection** (w-e07c42), **matcher cost**
  (w-3b8e15), **per-candidate allocations** (w-d17a62), **scoring axes**
  (w-2e5b93, w-9c2d51, w-04f83f).
- `5f1c04` currently owns the traversal loop in `src/lib.rs`. Prefer landing
  the operator in `src/lexical.rs` (or a new module) with a small, clearly
  marked call site. If the operator genuinely cannot be expressed without
  editing the loop, say so in this item and coordinate through the work item
  rather than editing another front's region silently.
- No phrase-specific case. `wreck`, `beach`, `recognize`, `speech`, `hits`,
  `justice`, `stupid`, `dupe`, `hid`, `came` and their substrings must not
  appear in production `src/`; a measurement harness reading them from the
  environment belongs in `/tmp` or in this item.
- No `axes::*` move. `approximate_output_is_locked` must not be re-baselined
  without a justified before/after table, not a silent re-baseline.
- No `zz_*`, `.bench`, `MADGAB_*` diagnostic or build tree left on the branch.

## Completion criteria

1. Either the approximate mode produces `hits justice dupe hid came` for
   `It's just a stupid game` as a proposal, or the mechanism is refuted with
   numbers (pool size, whether the clue is **enumerated**, wall clock, on at
   least three non-canonical targets) and the item is closed as a refutation.
2. The first canonical condition is preserved: `wreck a nice beach` for
   `recognize speech` still appears, at a recorded rank.
3. Report **enumerated** and **ranked** separately, always. They have been
   conflated before and it cost a pass.
4. The change is bounded with named arithmetic and no wall-clock or pool
   regression on the non-canonical targets.
5. Regression tests express external behaviour (a pool contains a wording one
   substitution away from a found one), not implementation details.
6. `cargo test --release --lib`, `--test corpus_integration`,
   `--test exact_determinism` and `--test approx_determinism` pass, except for
   the known single blocker
   `approximate_finds_classic_madgab_resegmentation`, which must be reported
   with its exact output rather than assumed.
7. Branch pushed, worktree clean, this item updated with objective state.

## Handoff / notes

Claimed by coordinator `coord-a13c` and assigned to agent `c1d3a7` in
`/workspace/madgab-adjacency` (branch `madgab-adjacency`, from `769733d`).
The `5f1c04` saturation numbers in "Why this and not something else" were
read from that agent's in-flight instrumentation and should be re-measured
from a confirmed build before they are quoted as final anywhere.
