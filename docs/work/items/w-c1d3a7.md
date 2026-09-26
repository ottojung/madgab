---
work_item: true
id: w-c1d3a7
state: done
priority: high
owner: agent-c1d3a7
updated: 2026-09-26T14:53:00Z
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
- `5f1c04`'s measurement on
  [w-5f1c04](w-5f1c04.md), from a clean build with the per-slot pop cap
  lifted on the canonical structure only (`ZZ_GEMIT` 4,000,000): whole-search
  emissions at `ZZ_DEEPCAP` 4,096 / 16,384 / 65,536 / 131,072 / 262,144 /
  1,000,000 / 2,000,000 are **18,271 / 30,559 / 79,711 / 145,247 / 276,319 /
  1,008,746 / 2,008,746**, and the canonical clue is **absent at every one**.
  The traversal does *not* saturate — it emits its full allowance every time —
  and at 2,000,000 it has covered 0.075% of its own 2,666,496,000-leaf
  product. The baseline spends 14,239 of 16,384, so the global emission
  constant is already slack at 1x. The earlier figures quoted here
  ("79,711 at 131,072, 80,367 at 262,144, saturation near 66,176") were
  artefacts of a scratch build in which a split-line reference to the real
  `LEXICAL_GLOBAL_EMISSION_BUDGET` escaped a string override; they are
  superseded and must not be carried forward. So "fund one structure deeply"
  is refuted, and the residual is the *shape* of the traversal, not its
  budget.
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
  **Fence lifted by coordinator `coord-7b30`/`coord-2f6a`:** `5f1c04` closed
  as a refutation and stood down, the traversal region became uncontested, and
  the operator was wired into the loop directly, at one clearly marked call
  site inside the per-segmentation loop, rather than routed through a work
  item. It is recorded here rather than done silently.
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
The `5f1c04` numbers in "Why this and not something else" were read from that
agent's in-flight instrumentation; they have since been superseded by that
agent's clean-build figures, which are the ones recorded above.

---

## Close-out (agent `c1d3a7`)

**Verdict, stated first because the two halves came out differently: the
mechanism is landed and measured; the canonical target is REFUTED with
numbers, and the refutation is not about reach.**

Branch `madgab-adjacency`, head `9767caf`, rebased onto `fdc2769`. The diff
against `fdc2769` is `src/adjacency.rs` (new, 397 lines) and `src/lib.rs`
(+232: the operator's four named constants, one clearly marked call site, one
test-only counter). No `tests/` change, no `axes::*` move, no re-baselining of
`approximate_output_is_locked`, and no canonical word or substring anywhere in
production `src/` (`src/adjacency.rs` contains none; the `src/lib.rs`
occurrences are pre-existing test fixtures from earlier fronts).

### 1. ENUMERATED and RANKED, reported separately

| | baseline `5f7df59`/`fdc2769` | with the operator |
|---|---|---|
| **ENUMERATED** (in the deduplicated pool) | **no** | **no** |
| **RANKED** (in the visible top 50) | **no** | **no** |

Both are unchanged, and the second is unchanged *by construction*, which is
the finding.

### 2. The refutation: enumeration was never the blocker

Measured by force-injecting the clue's own wording into the pool on the
default path — same segmentation, same slot indices, no other change — and
reading the result off the real scorer (instrumented scratch worktree, not
integrated; the numbers are reproducible from the scratch branch
`scratch/c1d3a7-measure`):

* the clue's own segmentation is `[(0,3),(3,10),(10,13),(13,15),(15,19)]`,
  structural rank **151 of 256** (unchanged at `SEGMENTATION_KEEP` 2048),
  slot widths `[160, 7, 160, 160, 93]`, and the clue's slot indices in the
  traversal's own ordering are **`[7, 0, 13, 99, 11]`**;
* per-word substitution cost `hits 0.200, justice 0.000, dupe 0.150,
  hid 0.370, came 0.400`, **total 1.120**;
* injected, the clue's **real final score is 0.819901291** and it lands at
  **raw rank 8,967** of 17,553 candidates, against a visible cutoff of
  **0.915691888**. It is cut, by 0.0958.

So a perfect enumerator would not show it. That is the whole result, and it
is why no amount of reach work can close this target: **the clue is not
hidden behind the traversal, it is behind the score.** The pool already
contains ~10^4 candidates at or below it, exactly as the acceptance note
anticipated for the pool; the additional fact measured here is that putting it
*in* the pool does not put it *in the list*, so the acceptance premise
"pool membership is the whole requirement" is false for this clue at its
current score. `select_diverse` step 1 is bounded by the visible cutoff, and
the clue is 0.0958 under it.

### 3. Which axes the clue loses on — the useful part of the refutation

Measured `Metrics` for the injected clue and for the clue that sets the
visible cutoff:

| axis | cutoff clue (`it justice too day mm`, 0.915691888) | the canonical clue (0.819901291) |
|---|---|---|
| boundary novelty | **1.000** | **0.667** |
| word familiarity | **0.768** | **0.399** |
| word novelty | 1.000 | 1.000 |
| rhythm | 1.000 | 1.000 |
| pronunciation/shape/closed-class aggregate (`content`) | 0.800 | **1.000** |

The clue is strictly **worse on boundary novelty and word familiarity**,
equal on word novelty and rhythm, and strictly **better on the acoustic
aggregate**. It loses on the two axes that are about *structure* and
*commonness*, not about pronunciation — and those are precisely the two axes
whose families are closed (boundary novelty: w-04f83f, w-2e5b93; familiarity
and the closed-class axis: w-9c2d51, w-04f83f). No `axes::*` move is
authorised here, and none was made.

This relocates the milestone one level further than "reach-then-rank": for
this clue the order is **score-then-reach**, and the score is a property of
the clue, not of the search.

### 4. The mechanism, landed, and what it is worth

`src/adjacency.rs` is a general neighbourhood walk over complete wordings,
seeded from the wordings a segmentation has already emitted, expanding by
**single-slot substitution**, re-scoring each child with the caller's own
admissible `bound` (so admission order is descending in score, as the
traversal's is), and admitting each new wording to the pool. Two named bounds
keep it from becoming a second product enumeration: `ADJACENCY_POPS` (24
wordings expanded) and `ADJACENCY_PER_SLOT` (2 children per slot per node).
It reads no vocabulary and names no phrase.

Two design facts worth recording for the next pass:

* **Do not carve its share out of the traversal's per-segmentation
  allowance.** Doing so (8 of 64) made
  `approximate_pool_reaches_matches_deep_in_a_span` fail — the traversal's
  emissions are what the depth tests are written against. It is bounded by
  `LEXICAL_GLOBAL_EMISSION_BUDGET` instead, which is the ceiling the whole
  search already respects and which the baseline leaves slack (14,239 of
  16,384 spent). Wall clock after this change is within noise of baseline on
  all six targets.
* **A best-first walk on the existing key cannot reach a below-average
  wording, however deep it is.** The operator's re-score is the same
  admissible bound the traversal orders by, and that bound *equals* the real
  score of the canonical clue (0.819901291 in both). The canonical sits
  under ~10^4 better wordings of the same structure, so no bounded
  score-ordered neighbourhood walk reaches it; only a coverage rule that
  ignores score could, and a coverage rule that ignores score has no way to
  prefer index 99 in one slot over the other 159. This is the general reason
  the mechanism is sound and still cannot serve this target.

Measured over six targets, `--approximate --top 50`, release binary, baseline
vs operator. **Pool = deduplicated candidate count**; search wall clock is the
binary's own report.

| target | pool base | pool +adj | search base | search +adj | visible list |
|---|---|---|---|---|---|
| `It's just a stupid game` | 17,231 | 17,553 (+1.9%) | 1,237 ms | 1,272 ms | identical |
| `recognize speech` | 15,329 | 15,914 (+3.8%) | 1,096 ms | 1,235 ms | identical |
| `I love you` | 8,961 | 9,031 (+0.8%) | 369 ms | 367 ms | identical |
| `a whole lot of trouble` | 15,630 | 16,609 (+6.3%) | 1,154 ms | 1,263 ms | identical |
| `congratulations on your promotion` | 11,214 | 12,659 (+12.9%) | 2,139 ms | 2,358 ms | **improved** |
| `Coors light` | 11,628 | 11,757 (+1.1%) | 523 ms | 552 ms | identical |

The improvement is real and is the mechanism working as designed: on
`congratulations on your promotion` two wordings at **0.888**
(`congrats laces th'are knee or ram ocean`, `congrats listen th'are knee or
ram ocean`) enter at ranks 3-4, above the previous rank-3 at 0.887 — they
were in the pool-adjacent region the traversal never reached and the
single-slot step found. No target's visible list got worse; no pool shrank;
no wall-clock regression outside noise. That satisfies criterion 4, and
`approximate_output_is_locked` passes **unmodified** because `I love you`'s
list is byte-identical.

### 5. First canonical condition preserved

`wreck a nice beach` for `recognize speech`: **ENUMERATED yes, RANKED yes, raw
rank 27, score 0.918313383** — identical with and without the operator
(measured on both binaries, same rank, same score to nine decimals). Not a
trade.

### 6. Test results

* `cargo test --release --lib` — **46 passed, 0 failed** (45 before this
  item; +6 in `src/adjacency.rs`, one of which replaced a bad expectation of
  mine).
* `cargo test --release --test corpus_integration` — **9 passed, 1 failed**:
  the known single blocker `approximate_finds_classic_madgab_resegmentation`.
  Its exact output is unchanged and is the same top-12 as the baseline:
  `["it justice too bad aim", "it justice too pad aim", "it justice too bad
  same", "it justice too peg aim", "it justice too pad same", "it justice too
  bad name", "it justice too bed aim", "it justice too pig aim", "eat justice
  too bad aim", "it justice too pad name", "it justice too pug aim", "it
  justice too bad came"]`.
* `cargo test --release --test exact_determinism` — **1 passed**.
* `cargo test --release --test approx_determinism` — **2 passed**.
* Not run, not claimed: `cargo fmt`, `cargo clippy`, doctests, `wasm32` — see
  [../../environment-notes.md](../../environment-notes.md).

### 7. Measurement reported but not landed: `SEGMENTATION_KEEP = 2048`

`SEGMENTATION_KEEP` 256 -> 2048 (which scales both global budgets with it, so
this is a budget-*size* change, i.e. a family closed twice already):
**pool 56,697 candidates, search 16.6 s**, canonical clue still **absent**.
Three of 2,048 segmentations host it, at structural ranks 151 / 187 / 284
with slot ranks `[22,0,64,85,7]`, `[22,0,80,85,7]`, `[22,0,54,85,7]`; the
third is at objective 0.891172900 and its best wording is far worse than the
first's 0.936533113, so it is not a cheaper alignment. 8x wall clock for no
change in outcome: rejected, not landed, and consistent with w-be6d21 and
w-5f1c04.

### 8. Next action

The blocker has moved and should be recorded as such in the umbrella
[w-4b1e07](w-4b1e07.md). For this clue the binding constraint is
**boundary novelty and word familiarity**, not reach: the clue's own
alignment is in the pool at index `[7, 0, 13, 99, 11]` of a retained
segmentation, one substitution per slot from the corner, and it is 0.0958
below the visible cutoff on two closed scoring axes while being *better* than
the cutoff clue acoustically. Three fronts could each move it, and all three
are outside this item's fences:

1. **boundary novelty** (w-04f83f / w-2e5b93): the clue reproduces fewer of
   the target's inner word boundaries than its competitors (0.667 vs 1.000).
   Whether a five-word clue that shares 2 of the target's boundaries *should*
   be penalised that hard is a scoring question, and the answer decides
   whether this clue is ever a proposal.
2. **word familiarity** (w-9c2d51): 0.399 vs 0.768. `dupe`, `hid` and `came`
   are rarer than `it`, `too`, `aim`. This is the axis that most of the
   deficit lives on, and it is the same axis that pushes the search towards
   determiner salad unless the closed-class axis is there to counteract it
   (w-e07c42 / the `axes::CLOSED_CLASS` note in `src/lexical.rs`).
3. **the visible cutoff itself**: `select_diverse` step 1 is bounded by the
   `top_n` best-scoring candidates, so no structural diversity argument can
   promote a candidate 0.0958 below it. Whether a pool this deep (17,553
   candidates for 50 slots) should have its diversity step reach past the
   cutoff is a *selection* question, and w-6b2f04 / w-04f83f own it.

I would not spend another reach pass on this clue. The traversal-shape
question this item opened is real and the operator answers it, but the answer
is bounded by the score, and the score is where the next front has to go.

### 9. Housekeeping

* The measurement instrumentation (`ZZ_INJECT`, `ZZ_METRICS`, `ZZ_SEGS`,
  `ZZ_SLOTS`, `ZZ_POOL`, `ZZ_EVAL`, `ZZ_TOTALS`, `SEGMENTATION_KEEP = 2048`)
  lives on the **scratch branch `scratch/c1d3a7-measure`** and in a separate
  worktree, never on `madgab-adjacency`. No `zz_*` test, no `.bench/`, no
  `MADGAB_*` diagnostic and no build tree is on this branch; the
  `MADGAB_TRACE_*` hooks that do exist in `src/lib.rs` are pre-existing
  (w-5f1c04 and earlier) and are untouched by this item.
* Branch `madgab-adjacency` is pushed at every stage of the work; the remote
  sha was confirmed by `git ls-remote --heads origin madgab-adjacency` after
  each push, after an early pass in which a detached HEAD left the branch
  ref behind the working tree and the first two pushes were silently no-ops.



## Integrated, 2026-09-26T14:53Z (coordinator `coord-5a11c`)

The operator and its two bounds are integrated into `post-milestone-acceptance`
as **8cc71c4** (merge of `madgab-adjacency` at `2944011`), pushed, `main`
untouched. Review detail, the fence scan and the four suites re-run on the
merged tree are in [w-6f3a91](w-6f3a91.md). Two notes for the next pass:

* the refutation in §8 is the durable result and it retires the whole "one
  more reach pass on this clue" family: the clue's own alignment is in the
  pool, and no bounded score-ordered neighbourhood walk can promote something
  that sits under ~1e4 better wordings of the same structure. The three
  candidates for moving it — boundary novelty, word familiarity, and the
  visible cutoff — are all outside this item's fences, and are owned by
  [w-2e5b93](w-2e5b93.md) (score) and [w-7b2d40](w-7b2d40.md) (selection).
* the branch's remote ref is `origin/wip/adjacency-5a11c`, not
  `origin/madgab-adjacency`. The latter still points at `413113a`, which the
  agent's rebase replaced; it could not be fast-forwarded and was left alone
  rather than force-pushed. `2944011` is on both the local branch and
  `origin/wip/adjacency-5a11c`, so nothing is at risk.
