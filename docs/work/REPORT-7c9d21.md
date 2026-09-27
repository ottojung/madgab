---
work_item: w-7c9d21
report: true
id: REPORT-7c9d21
state: done
branch: madgab-printgate-7c9d21
base: post-milestone-acceptance at 79ab6c9
verdict: HOLD
updated: 2026-09-27T23:40:00Z
---

# REPORT-7c9d21 — the post-pool stage that drops a pool-rank-48 candidate is the `top_n` cut, and the selection layer's whole available lift is the price of a fence

## 0. Two things the coordinator should know before reading the numbers

**There is no `docs/work/items/w-7c9d21.md`.** The work item this front was launched against does
not exist in the tree on any branch I can see; `docs/work/items/` has 78 items and none is
`w-7c9d21`. The brief arrived as prose in the launch prompt and I worked from that. I did **not**
write the item file, because a front authoring its own contract is how a scope silently grows; the
coordinator should land the item and check that what I priced is what it asked for. Everything
below is read off the prose brief and is self-contained.

**F7 and F8 are not in this tree.** `tests/emit_coverage.rs` here carries seven tests and none is
named F7 or F8; the F-numbers are agent-e086cc's local fence set, and `/workspace/madgab-e086cc`
was not read or entered. So I could not re-run e086cc's fences, and I did not try. What I did do
is take its two load-bearing *facts* as given and test them at the shipped objective, which is the
question this front was actually asked. Both reproduce: the green case is in the pool at rank 27
and in the printed set, and there is a pool member at rank 48 with `in_printed = false`.

## 1. The cited pipeline, pool to printed proposals

All of it is in `Generator::finish` and `select_diverse`, both in `src/lib.rs`. There are exactly
five stages and no others.

| # | stage | file:line | what it does to a candidate |
|---|---|---|---|
| 1 | score | `src/lib.rs:2420-2425` | `Partial::into_clue` runs the scorer; the objective vector is read **here and nowhere else in this path** |
| 2 | **ordering** | `src/lib.rs:2428-2433` | `sort_by` descending `score`, ties broken by ascending `phrase` |
| 3 | **dedup** | `src/lib.rs:2439-2441` | `retain` on `phrase_signature(&c.phrase)` — first spelling wins, so the **highest-scoring** member of each variant group survives |
| 4 | **selection** | `src/lib.rs:2444` → `select_diverse`, `src/lib.rs:3951-4056` | three sub-stages, see below |
| 5 | presentation | `src/lib.rs:4049-4055` | `picked.sort_by(cmp_desc(score))`; membership was already decided, this only orders it |

Stage 3 is the boundary that matters for this question. `finish` returns
`(select_diverse(clues.clone(), top_n), clues.len(), clues)` at `src/lib.rs:2443-2447`, and the
third element is what `generate_pool` hands back (`src/lib.rs:894-896`). **The "pool" in every
pool-rank number in this project, including rank 48, is *post*-dedup and *pre*-selection.** So a
pool-rank-48 member has already survived dedup by construction, and dedup cannot be the stage that
dropped it. That is worth stating because it is the kind of thing that gets re-derived wrong: the
number 48 is a post-dedup rank, not an emission rank.

`select_diverse` has three sub-stages:

| sub-stage | file:line | admits |
|---|---|---|
| representation reserve | `src/lib.rs:3991-4007` | the best-scoring member of each **newly seen** boundary structure, from the **whole** pool, bounded by `structure_reserve_slots(50) = 12` (`src/lib.rs:4077-4079`, divisor 4 at `:4087`) |
| score walk under a share cap | `src/lib.rs:4022-4034` | descending score, if `counts[structure] < cap` where `cap = share_cap(50, 312) = 17` (`src/lib.rs:4098-4104`) |
| cap-dropped fill | `src/lib.rs:4038-4045` | score order, cap ignored; only if the list is still short |

The output lock, F6 `approximate_output_is_locked` (`tests/corpus_integration.rs:556`), is a
**test**, not a stage. It observes the printed set and cannot drop anything. It is listed in the
brief's candidate set and is exonerated by being downstream of observation rather than of flow.

## 2. The deciding measurement

I replayed `select_diverse` stage by stage against the production pool, in an in-`src` unit test on
an unpushed scratch branch, using the shipped `clue_structure`, `structure_reserve_slots`,
`share_cap` and the shipped tie-break — no re-derivation of the rule. **The replay reproduces the
printed set exactly** (`replay matches printed set: true`, 50/50), so the trace below is the
production trace.

`recognize speech`, `SearchMode::approximate()`, `top_n = 50`, shipped objective:

```
pool = 18289   printed = 50   structures = 312
reserve = 12   cap = 17
mix: reserve = 12   score/fill = 38   deepest admitted pool rank = 227
```

The candidate the brief names is pool rank **48**, score `0.918840116882308`, and the trace says
`why = unpicked` — it was admitted by no sub-stage. Now the elimination:

* **Not dedup.** It is *in the pool*, and the pool is post-dedup (§1).
* **Not ordering.** Rank 48 is its correct rank under the shipped `sort_by`; the score order is
  the same order the walk walks.
* **Not the share cap.** This is the decisive one, because it is the intuitive answer and it is
  wrong. The candidate's boundary structure has **270** members in the pool, of which **16** are
  admitted, at pool ranks `[5,6,10,13,15,16,18,20,21,24,26,27,29,32,34,40]`. The cap is **17**. The
  candidate is the 19th member of its own structure in score order, but only 16 slots of that
  structure are used, so the structure is **one slot below its cap** and the cap was never
  consulted on its behalf. The cap is not binding.
* **It is the `top_n` cut.** `src/lib.rs:4025`, `if picked.len() == top_n { break; }`. The
  50th slot was consumed at **pool rank 42**; the walk breaks before it ever reaches 43. Ranks
  43, 44, 45, 46, 47, 48 are all `unpicked` for the same reason.

So the mechanism is the top-N cut. But naming only that would be misleading, because the top-N cut
is *why* 48 is dropped while the **reserve is why the cut is at 42 instead of 50**. Twelve of the
fifty slots are spent before the score walk begins, on the best member of twelve structures, drawn
from as deep as **pool rank 227**. The score walk therefore only gets 38 slots, and 38 score-ranked
candidates fill it by rank 42. **The reserve is the cause; the `top_n` cut is the mechanism.** The
twelve displaced candidates are exactly the twelve score-worthy members of pool ranks 43-54, and
rank 48 is one of them.

## 3. The priced lift, and why none of it is collectible

At the shipped objective, comparing the shipped printed 50 against the pool's own top 50 by score
(the selection layer removed), on the green target:

```
pool mean SIMILARITY                        0.795259
(a) shipped   printed 50 mean SIMILARITY    0.848330   lift +0.053071   mean score 0.919922801
(b) pure-score printed 50 mean SIMILARITY   0.856135   lift +0.060876   mean score 0.920479094

selection-layer available SIMILARITY lift (a -> b)   +0.007805
selection-layer available score     lift (a -> b)   +0.000556293
```

**Green case printed rank: 27 before, 27 after.** Unchanged under (b). The green case is admitted
by the **score** walk, not the reserve (`why = score`), and it sits at pool rank 27, inside the
cutoff either way — so it is structurally immune to anything this front could do to the selection
layer. That is a genuinely useful result for the coordinator: the green case is not a constraint on
this coordinate at all.

**+0.007805 SIMILARITY is the entire available lift, and it is not collectible.** It is not a
small number to be argued away — it is a *fenced* number. Taking it means deleting the
representation reserve, and the reserve is what
`approximate_list_represents_enumerated_resegmentations`
(`tests/corpus_integration.rs:502-528`) exists to require: the printed 50 must span at least
`TOP_N / 4 = 12` distinct boundary structures. Measured, the pure-score list spans:

| target | shipped structures | pure-score structures | fence needs |
|---|---|---|---|
| `the cat sat on the mat` | 12 | **6** | 12 |
| `she sells sea shells` | 12 | **2** | 12 |
| `recognize speech` | 12 | **4** | 12 |

So collecting +0.007805 takes a green fence to red by a factor of 2-6. F6
`approximate_output_is_locked` would go red at the same time, since the printed list changes.
Both are fences the brief forbids weakening, and per the brief and `REPORT-4d1e93.md` §4 a
re-baseline of F6 is a coordinator call, not a front's. **Therefore the +0.007805 is a priced
negative, not a missed opportunity.** There is no partial version either: the lift is monotone in
the reserve size and the reserve size is pinned at exactly 12 by a fence that reads exactly
`TOP_N / 4`, so every value of `STRUCTURE_RESERVE_DIVISOR` that keeps the fence green yields the
same printed list.

I want to be exact about what is and is not priced here. I priced the selection layer's slack by
removing it wholesale, which is the upper bound. I did not search for a rule that recovers part of
the +0.007805 *while keeping* 12 represented structures, and I do not believe one exists, because
the 12 structures the fence names are exactly the 12 whose best members sit below the score
cutoff — the reserve's entire content. But that last step is an argument, not a measurement, and
it is the one claim in this report that a successor front should re-measure rather than inherit.

## 4. Verdict

**HOLD.** No code change. The pipeline is cited, the deciding stage is measured, and the available
lift is priced at **+0.007805** printed-top-50 `SIMILARITY` — of which **zero** is collectible
without turning `approximate_list_represents_enumerated_resegmentations` red.

A priced negative needs a successor coordinate, and it is not ambiguous:

> **The head lift is an objective-side coordinate, not a selection-side one.** The selection layer
> has no slack left that is not a fence. The live owner is
> `docs/work/items/w-3f8c62.md` (front `agent-3f8c62`, branch `madgab-parsim-3f8c62`), landing the
> word-count parsimony axis priced at `REPORT-9b4a15.md` §6 as C1d. `OBSTRUCTION-MAP.md` §3
> already routes head lift there and says case-2 reach must be judged on head lift; this report
> adds that the *selection* layer is now also priced on that criterion and contributes nothing.
> A successor should not open a third selection-side front.

The other two live surfaces named in the brief stay where they are: the objective weight vector is
`agent-e086cc`'s and the score function is `agent-3f6a21`'s. This front touched neither, and it
confirms they are the only remaining places the head lift can come from.

## 5. Verification

Baseline on the clean tree, `cargo test --release`, `--test-threads=1`, dedicated
`CARGO_TARGET_DIR=/workspace/target-7c9d21`:

| suite | result |
|---|---|
| `no_phrase_hard_coding` | **9 passed / 0 failed** |
| `emit_coverage` | **7 passed / 0 failed** (4 pre-existing + 3 from `w-4d1e93`) |
| `approx_determinism` | **1 passed / 0 failed** |
| `exact_determinism` | **4 passed / 0 failed** |
| `cargo test --release --lib` | **75 passed / 0 failed / 12 ignored** (unchanged) |
| `corpus_integration` (`--test-threads=2`) | 12 passed / 1 failed — `approximate_finds_classic_madgab_resegmentation`, the **known base red** from `OBSTRUCTION-MAP.md` §4, not re-pinned and not touched. `approximate_output_is_locked` **ok**; `approximate_finds_recognize_speech_resegmentation` **ok**; `approximate_list_represents_enumerated_resegmentations` **ok** |

These are the numbers the brief required to stay green, and they are unchanged from base, because
this front landed no code. `corpus_integration` was run with `--test-threads=2` because
`OBSTRUCTION-MAP.md` §4 records SIGKILL at default parallelism on this host at base too.

`cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run on this host** (no
`rustup` components) and are **not claimed**. The report is the only file changed.

## 6. Constraints

* **No phrase-specific hard-coding.** No change landed. The two measurement modules lived on the
  unpushed branch `scratch/7c9d21-probe`, named `wreck`/`a`/`nice`/`beach` as *test data* in
  `#[cfg(test)]` code only, and were reverted before any suite was run. `src/` is byte-identical
  to `79ab6c9`; `git status` is clean. `no_phrase_hard_coding` is 9/9 with the probe absent, which
  is the state that ships.
* **No fence weakened, deleted or `#[ignore]`d.** Every test in §5 is byte-identical to base.
* **Objective weight vector and score function untouched.** `axes::*` and `Metrics::combined` were
  not edited. `/workspace/madgab-e086cc` and `/workspace/madgab-pairscore-3f6a21` were not read,
  entered or run.
* **Own `CARGO_TARGET_DIR`** (`/workspace/target-7c9d21`), `--test-threads=1` on every run except
  the documented `corpus_integration` exception.
* **No `OBSTRUCTION-MAP.md` row re-opened**, and this is not treated as a case-2 reach item.
* **Probe branch `scratch/7c9d21-probe` is unpushed** and holds only `#[cfg(test)]` code. This
  report goes to `madgab-printgate-7c9d21`. Nothing was merged to `main`, and nothing was
  self-merged into `post-milestone-acceptance`.
