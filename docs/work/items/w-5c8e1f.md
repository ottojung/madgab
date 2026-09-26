---
work_item: true
id: w-5c8e1f
state: working
priority: high
owner: coord-3f10
updated: 2026-09-26T18:52:00Z
branch: madgab-score-reseg-9a41d3
worktree: /workspace/madgab-score-9a41d3
agents: 9a41d3 (opened by coord-3f10)
---

# The objective, not the enumeration, refuses a full resegmentation: make the
# similarity axis mean "how much worse than the best wording this segmentation
# admits", and land the phrase-free resegmentation guard

## Goal

Make `madgab --approximate` able to propose a **full resegmentation** — a visible
proposal sharing no word with its target — for ordinary multi-word targets, including
`It's just a stupid game`, as a result of a general change to how the similarity axis
is computed. Never a phrase-specific case.

## Why this item exists: front `c81e55` refuted the search-side framing

`c81e55`'s report is durable: `git show madgab-depthcap-c81e55:REPORT-c81e55.md`
(branch `madgab-depthcap-c81e55`, pushed at `b8391f6`). Its headline is a
*production* number, obtained by pushing the requested index tuples through the
ordinary `build` with the ordinary scorer and `select_diverse`:

```text
both admitting segmentations emitting the requested wording:
  raw phrase="hits justice dupe hid came" rank=9867 score=0.819901291
  raw_cutoff rank=49 score=0.915691888 phrase="it justice too day mm"
  17 908 candidates
```

So the entire search, emitting that wording from *every* segmentation that admits it,
would move it from absent to **9867th of 17 908** — a gap of **0.0958**, about **11x
the width of the visible top-50 band**. Its other measured results:

- the depth cap `EMIT_PROFILE_MAX_DEEP = 3` (`src/lib.rs:212`) is **inert** for the
  shapes that carry the wording: those are 5-slot segmentations, where a depth-4 class
  needs 30 draws of the 16 available, so raising the cap to 4 changes nothing there;
- `4e8a52`'s "pool membership" metric is nearly uncorrelated with the milestone, so
  pool-membership claims must not be used as the acceptance measure for this goal;
- exactly **2 of 256** segmentations admit the wording, both at cost 1.1195 and 1.3695,
  both inside `total_budget` 1.5, so segmentation and the cost bound are not the gate;
- the wording's slot 1 (`justice` at index 0) is already pinned into all 50 visible
  proposals, so the requested clue loses nothing by being forced there.

## The general defect, which is what this item actually fixes

`c81e55` §6, measured over 13 real targets, counting visible proposals that share **no**
word with the target (>=3-letter prefix rule so `it's`/`it` counts as shared):

| target | full resegments in visible 50 |
|---|---|
| **It's just a stupid game** | **0 / 50** |
| the cat sat on the mat | 29 / 50 |
| recognize speech | 44 / 50 |
| when the rain finally stopped | 31 / 50 |
| the other nine | 49-50 / 50 |

On 1 of 13 targets the visible proposal set contains no resegmentation at all. That is a
property of the code meeting an ordinary target, not a fact about two sentences.

## The change to implement

**M7a, the relative similarity axis.** The similarity axis is currently
`1 - total_cost / 4`, an absolute IPA-edit budget against a bare `4.0` constant, so a
wording pays for a cost scale that means nothing relative to the alternatives for the
same segmentation. Change it to

```text
1 - (total_cost - segmentation_floor) / 4
```

where `segmentation_floor` is the additive cost of the all-argmin tuple of that
segmentation — the floor `total_budget` is already measured against, and the same
suffix-minimum structure the traversal's admissible bound already computes, so it
should be free. Predicted by `c81e55` §5: the requested clue 0.819901 -> 0.833368
(rank 9867 -> ~1101 of 3 050), `wreck a nice beach` unchanged to within 0.001, and the
wording still **0.093 short** of the 50th-best tuple. **M7a alone is expected NOT to
green `approximate_finds_classic_madgab_resegmentation`, and must not be sold as doing
so.** It is worth landing because it makes the axis mean something, and because it is
the only member of the scoring family with a general argument.

**M7b is forbidden.** `c81e55` §5 shows the only reweighting that reaches the cutoff
deletes FAMILIARITY and cuts NOVELTY, moving 21.3 % of the scoring mass into RHYTHM,
which works purely because the requested clue maxes RHYTHM. That is an objective
re-pointed at one clue's free lunch, it is indistinguishable from tuning to the
acceptance clue, and it would break `approximate_output_is_locked`. Do not do it, do
not approximate it, and do not reach for a new axis whose only justification is that it
happens to lift the one clue this repository measures.

## Also required: the phrase-free regression test

Land `c81e55`'s proposed test in `tests/corpus_integration.rs` (API boundary, naming
no wording): for a table of ordinary multi-word targets, assert the visible proposals
contain at least one full resegmentation, using the >=3-letter shared-prefix rule so a
clipped target word counts as shared. **Land it red-first and say in the report which
targets are red**; do not weaken the rule to make it green, and do not delete or
`#[ignore]` it. The milestone's intent is "the approximate mode can propose a
resegmentation", which is a property of the generator, not of one dictionary lookup.

## Scope and collision boundary

This item owns the similarity axis' *definition* and the resegmentation guard. Live
fronts own other territory and must not be contended with:

- [w-2f7a10](w-2f7a10.md): `coverage_tuples`, `sweep_index`, `EMIT_PROFILE_MAX_DEEP`,
  `EMIT_PROFILE_RESERVE`, `LEXICAL_COMBINATIONS_PER_SEGMENTATION`, the emission budget
  accounting in `build`, per-structure retention. `b47d02` is still running there.
- [w-6b91d3](w-6b91d3.md): pool reproducibility — hashing, key derivation, collection
  order, and the tightening of `tests/approx_determinism.rs`. `d3b7c2` is still
  running there. Do not touch `coverage_tuples` while those fronts are live.
- Axis *weights* were closed by [w-3f9c02](w-3f9c02.md). This item changes one axis'
  formula, never a weight, and never any other axis.
- `select_diverse`'s admission order, the share cap and `STRUCTURE_FLOOR` are closed by
  [w-8a1d47](w-8a1d47.md).

Forbidden: editing, relaxing, re-baselining, skipping or deleting
`approximate_finds_classic_madgab_resegmentation`,
`approximate_finds_recognize_speech_resegmentation` or
`approximate_output_is_locked`; any phrase-specific, word-specific, substring-specific
or dictionary-lookup special case; any `MADGAB_*`/`ZZ_*`/`zz_*`/probe scaffolding left
on the branch; merging `post-milestone-acceptance` yourself; touching `main`.
`approximate_output_is_locked` may be re-baselined **only** with a recorded list of
which structures moved and why, and only as a direct consequence of this change.

## Completion criteria

1. The similarity axis' new definition is landed, general, and argued from the
   mechanism: no target, phrase, word, substring, clue or environment value reaches
   it, and `git grep` over the added `src/` lines finds none.
2. Both the change and the reasoning are measured with the **production** scorer
   (`MADGAB_TRACE_PHRASES`, or the same injection discipline `c81e55` used) on the
   canonical target, on `recognize speech`, and on at least three further ordinary
   targets: the wording's rank/score before and after, the visible top-50 before and
   after, and the number of visible full resegmentations per target before and after.
3. `tests/no_phrase_hard_coding` green, and the resegmentation guard landed red-first
   with the per-target red/green state recorded in the report.
4. `cargo test --release --lib`, `--test corpus_integration`, `--test exact_determinism`,
   `--test approx_determinism`, `--test no_phrase_hard_coding` reported per suite with
   pass/fail counts. `cargo fmt`, `cargo clippy` and doctests **do not exist on this
   host** — see [../../environment-notes.md](../../environment-notes.md) — and must not
   be claimed.
5. `recognize speech` -> `wreck a nice beach` still visible, and its raw-cutoff margin
   reported: `c81e55` measured it at **0.00127** (raw rank 27, 0.918313383 against a
   0.917045726 cutoff), so it is one hundredth of a point from falling and every
   pool-changing change is a coin flip on it. If it falls, say so plainly and do not
   re-baseline the test to hide it.
6. `approximate_finds_classic_madgab_resegmentation` reported by name. If it is still
   red, that is an accepted outcome of M7a and must be stated as such; if it goes green,
   state exactly which mechanism did it and confirm the diff contains no special case.
7. Wall clock measured on the canonical target and `recognize speech` against the
   pre-change binary, interleaved, with medians.
8. A pushed `REPORT-9a41d3.md` on the front's branch, reached with
   `git ls-remote origin madgab-score-reseg-9a41d3`, containing every number above,
   stating what was *not* measured, and ending with a single line
   `MERGE RECOMMENDATION: MERGE|HOLD`.

## Handoff / notes

Opened by coordinator `coord-3f10` at `27f9776` on `post-milestone-acceptance`, which is
now `madgab-int-79309a1` (`53ba542`, the per-member sweep fix `e5b1c2e` merged on
`a3f19c`'s durable `MERGE` recommendation in `REPORT-a3f19c.md` §0).

The base measurements every claim above rests on: the injection result and the axis
table in `REPORT-c81e55.md` §3/§5, the 13-target resegmentation counts in §6, and the
guarding margin in §1. Read them before implementing rather than re-deriving them.

**If M7a lands and the canonical test is still red, that is this item's expected
outcome, not a failure.** In that case the honest next question is a new work item about
the objective, not a second attempt at the search side: `c81e55` §4 has already
measured the search side to be provably zero-effect on this milestone (M1-M6), and §5
has measured that no reweighting of the six existing axes reaches the cutoff without
destroying one. The remaining general degrees of freedom are *new* axes with an
independent argument, and each must survive the question "would this axis help inputs
that are not the acceptance examples?".

### Opened and launched `coord-3f10`, 2026-09-26T18:55Z

Worktree `/workspace/madgab-score-9a41d3` on branch `madgab-score-reseg-9a41d3`, created
from `post-milestone-acceptance` at `27f9776` (which is the merge of `a3f19c`'s
`79309a1` sweep fix). Agent `9a41d3` opened and running. Integration is this
coordinator's job, not the front's: merge only on a pushed
`MERGE RECOMMENDATION: MERGE`.
