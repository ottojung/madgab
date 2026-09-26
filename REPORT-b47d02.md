# w-2f7a10 — front `b47d02` (does lifting `EMIT_PROFILE_MAX_DEEP` to 4 offer anything?)

Branch `madgab-depth4-b47d02`, created at `74e3d8b`, worktree `/workspace/madgab-depth4-b47d02`.
**This branch must never be merged anywhere** — not into `main`, not into
`post-milestone-acceptance`, not onto any other branch. The `cost_probe` instrument from
`e1a3f7` is kept, deliberately. No test was edited, relaxed, re-baselined, skipped or
deleted. No fix is implemented here; the findings below are findings, including the one
that contradicts the brief's expectation.

## 0. What was changed, per arm

* **main arm — cap 4.** This worktree, `74e3d8b` with exactly one line changed in
  `src/lib.rs`: `EMIT_PROFILE_MAX_DEEP: usize = 3` → `4`. Nothing else in `src/`; the
  probe from `e1a3f7` is unchanged. `examples/costpool.rs` gained a *generic* slot-exact
  witness check (see §4), which is harness code, not `src/`.
* **control arm — cap 3.** Pristine `git archive 74e3d8b | tar -x -C
  /workspace/madgab-b47d02-cap3`, built `--release` there; the same `costpool.rs` copied in
  so the two arms' harnesses are byte-identical.
* **auxiliary arm `aux96` — cap 4 + reserve 96.** Pristine `74e3d8b` in
  `/workspace/madgab-b47d02-aux64` with **two** lines changed: the cap to 4 *and*
  `EMIT_PROFILE_RESERVE: usize = 16` → `96`. This arm exists only to price the depth-4
  row, because at reserve 16 the schedule never reaches a four-slot subset (§3). It is
  **not** a proposal and must not be read as one.
* **`aux96` budget arms.** The same `aux96` binary driven with the CLI's existing
  `--total-budget 2.0` and `--total-budget 2.5` flags. No source change at all on these two.

Nothing else was touched: no scoring, ranking, segmentation, selection, adjacency, share
cap or budget constant on any arm.

## 1. The headline: the depth-4 row on the main arm is empty

`measurements/b47d02/replicate-summary.txt`. Twelve targets, `--approximate --top 50`,
release, default path, four independent replicates per arm:

| arm | replicate | offered | accepted | refused | rate | **depth-4 offers** |
|---|---|---|---|---|---|---|
| cap 3 | 1 | 45 387 | 36 674 | 8 713 | 19.20 % | **0** |
| cap 3 | 2 | 45 387 | 36 664 | 8 723 | 19.22 % | **0** |
| cap 3 | 3 | 45 387 | 36 662 | 8 725 | 19.22 % | **0** |
| cap 3 | 4 | 45 387 | 36 665 | 8 722 | 19.22 % | **0** |
| **cap 4 (main)** | 1 | 45 387 | 36 650 | 8 737 | 19.25 % | **0** |
| **cap 4 (main)** | 2 | 45 387 | 36 664 | 8 723 | 19.22 % | **0** |
| **cap 4 (main)** | 3 | 45 387 | 36 647 | 8 740 | 19.26 % | **0** |
| **cap 4 (main)** | 4 | 45 387 | 36 655 | 8 732 | 19.24 % | **0** |

**Lifting the cap to 4 offers a depth-4 tuple zero times, in 45 387 offers, on four
independent runs, across all twelve targets.** The per-depth split is unchanged as well
(`measurements/b47d02/depth-summary.txt`):

| depth | cap 3 offered / acc / ref / rate | cap 4 offered / acc / ref / rate |
|---|---|---|
| 1 | 16 357 / 15 073 / 1 284 / 7.85 % | 16 357 / 15 069 / 1 288 / 7.87 % |
| 2 | 26 266 / 20 137 / 6 129 / 23.33 % | 26 266 / 20 121 / 6 145 / 23.40 % |
| 3 | 2 764 / 1 464 / 1 300 / 47.03 % | 2 764 / 1 460 / 1 304 / 47.18 % |
| 4 | 0 / 0 / 0 / — | **0 / 0 / 0 / —** |

The cap-3 and cap-4 differences (≤ 24 accepted tuples, ≤ 0.06 pp) are inside the
**run-to-run noise of the same binary**, quantified in §1.1. The honest reading is:
**this change is a no-op on the default path**, and its only measured effect is noise.

### 1.1 The probe trace is not run-to-run deterministic (new finding, and it bounds every number here)

Two runs of the *same cap-3 binary* produce TSVs that differ (`cmp` fails at line 5 633 of
45 387). Aggregated over the twelve targets the totals move by ±11 accepted tuples
(36 662 – 36 674 on cap 3, 36 647 – 36 664 on cap 4) and by ≤ 0.06 pp in rate, per depth.
`tests/approx_determinism.rs` is green on both arms, so this is a narrow nondeterminism the
determinism test does not cover, not a broken build. It matters for interpretation: **any
cap-3-vs-cap-4 difference smaller than about ±25 accepted tuples or ±0.06 pp is not
measurable with this instrument**, and the parent report's single-run per-depth figures
should be read with the same band. Recorded, not chased — it is not this front's territory.

Per-target, all depths (`measurements/b47d02/depth-summary.txt`), cap 3 → cap 4, first
replicate: every target's rate moves by ≤ 0.30 pp, e.g. `an old man in a big hat`
44.81 % → 45.11 %, `It's just a stupid game` 10.28 % → 10.31 % — inside the noise band
above.

## 2. Why the cap was never the gate: the reserve budget is

`measurements/b47d02/depth-reach.txt`, and the code path itself
(`coverage_tuples`, `src/lib.rs:384`).

Per segmentation the reserve is `EMIT_PROFILE_RESERVE.min(emit_allowance)` = **16** tuples,
and `coverage_tuples` walks **breadth before depth**: all 1-subsets, then all 2-subsets,
then all 3-subsets, then 4-subsets, stopping the moment `out.len() >= reserve`. For the
canonical target's 6 slots the first four-slot class sits at position
`C(6,1)+C(6,2)+C(6,3)+1` = **42**; for `recognize speech`'s 8 slots at
`8+28+56+1` = **93**. With a budget of 16 the enumeration returns long before any of them.

Depth 3 is reached at all only by accident: `sweep_index` returns `None` for a slot whose
width is at most `LEXICAL_BRANCH_STAGE_0` (10), and such a class is skipped without
consuming budget, so on some phases a 3-subset does get emitted. Measured, per
(target, segmentation-phase) group:

| arm | groups | with a depth ≥ 3 offer | with a depth ≥ 4 offer | max depth histogram |
|---|---|---|---|---|
| cap 3 | 3 071 | 1 441 | **0** | d1 11, d2 1 619, d3 1 441 |
| cap 4 | 3 071 | 1 441 | **0** | d1 11, d2 1 619, d3 1 441 |
| aux96 (reserve 96) | 3 069 | 2 990 | **2 773** | d1 11, d2 68, d3 217, d4 2 773 |

So the parent's section 11 premise — "`EMIT_PROFILE_MAX_DEEP = 3` caps the depth before the
traversal" — is **correct about the constant and wrong about its role**: the cap is a
*ceiling*, and the binding constraint is the 16-tuple per-segmentation reserve combined with
the breadth-first ordering. Lifting a ceiling that is never reached cannot move the table.
The requested shape is blocked by `EMIT_PROFILE_RESERVE = 16` and by the ordering, not by
`EMIT_PROFILE_MAX_DEEP`.

Also worth recording: the internal invariant at `src/lib.rs:4179` is
`assert!(EMIT_PROFILE_MAX_DEEP >= 1 && EMIT_PROFILE_MAX_DEEP <= 4)`, so **4 is the maximum
depth this design can express at all**; the cap cannot be raised further without editing
that test. No test was edited here.

## 3. The depth-4 row, priced on the auxiliary arm (`aux96`, cap 4 + reserve 96)

`measurements/b47d02/probe-aux96.tsv`, 123 160 offered tuples.

| depth | offered | accepted | refused | rate | mean cost of accepted |
|---|---|---|---|---|---|
| 1 | 16 345 | 15 125 | 1 220 | 7.46 % | 1.073 |
| 2 | 37 814 | 28 376 | 9 438 | 24.96 % | 1.223 |
| 3 | 48 756 | 22 542 | 26 214 | 53.77 % | 1.317 |
| **4** | **20 245** | **4 241** | **16 004** | **79.05 %** | **1.360** |
| all | 123 160 | 70 284 | 52 876 | 42.93 % | |

Per target, depth 4 only (`measurements/b47d02/aux96-per-target-d4.txt`):

| target | d4 offered | d4 accepted | d4 refused | d4 rate | target's d3 rate |
|---|---|---|---|---|---|
| It's just a stupid game | 1 400 | 461 | 939 | 67.07 % | 36.80 % |
| recognize speech | 888 | 321 | 567 | 63.85 % | 39.08 % |
| when the rain finally stopped | 1 550 | 364 | 1 186 | 76.52 % | 37.52 % |
| the cat sat on the mat | 2 182 | 528 | 1 654 | 75.80 % | 46.92 % |
| there is no way to know | 2 151 | 647 | 1 504 | 69.92 % | 40.04 % |
| she had a lot of money | 1 493 | 308 | 1 185 | 79.37 % | 56.05 % |
| a whole lot of trouble | 1 755 | 213 | 1 542 | 87.86 % | 60.62 % |
| he was a big fat man | 2 015 | 351 | 1 664 | 82.58 % | 67.23 % |
| my brother has a red car | 1 703 | 342 | 1 361 | 79.92 % | 51.24 % |
| what are you going to do | 1 240 | 195 | 1 045 | 84.27 % | 55.97 % |
| put it back on the shelf | 2 202 | 326 | 1 876 | 85.20 % | 61.56 % |
| an old man in a big hat | 1 666 | 185 | 1 481 | 88.90 % | 78.90 % |

**This is the number the parent front asked for, and it is 79 %.** A four-deep tuple is
offered and accepted **once in five**; on the canonical target, **once in three**. Compare
depth 3 at 53.8 % refused and depth 2 at 25.0 %: the ladder continues, it does not fall off
a cliff, but it is steep. Mean accepted cost rises monotonically with depth
(1.073 → 1.223 → 1.317 → **1.360**) against `total_budget` 1.5, which is the mechanism.

### 3.1 The refusals are the additive budget, not a per-word gate

Same `aux96` binary, existing CLI flag only, no source change
(`measurements/b47d02/aux96-budget-sensitivity.txt`):

| `--total-budget` | d1 ref | d2 ref | d3 ref | **d4 ref** | d4 accepted |
|---|---|---|---|---|---|
| 1.5 (default) | 7.46 % | 24.96 % | 53.77 % | **79.05 %** | 4 241 |
| 2.0 | 0.02 % | 0.27 % | 2.79 % | **10.79 %** | 17 740 |
| 2.5 | 0.00 % | 0.00 % | 0.00 % | **0.03 %** | 19 764 |

Raising the additive bound from 1.5 to 2.0 takes depth-4 acceptance from 20.9 % to 89.2 %.
So a four-deep tuple is **not** intrinsically unbuildable; it is priced out by the additive
sum over the baseline tuple's own cost. The requested shape's own measured cost,
0.919520 (`4e8a52`), is far under 1.5, so on this evidence the requested wording is not the
kind of four-deep tuple that gets refused — it is a *cheap* one — and the 79 % refusal rate
is dominated by tuples whose non-deep slots are already expensive.

Caveat stated plainly: `aux96` spends 6× the reserve per segmentation, so it also
overlaps the traversal's emissions; the 2.0/2.5 arms change what the traversal is competing
with, and the offer counts move (123 160 → 120 744 → 120 014). These arms price the
*acceptance* of a four-deep tuple under a looser bound, not a proposed configuration.

## 4. Does a four-slot class get offered, and does the requested shape appear?

Stated plainly, because it is the item's target:

* **The class schedule does include four-slot subsets** — `slot_combinations(6, 4)` has 15
  members for the canonical target — but at reserve 16 the enumeration returns at 16 tuples
  and never reaches them (§2). Zero four-slot classes are ever constructed.
* **With the cap at 4 and nothing else changed: zero depth-4 tuples are offered per target,
  so there is no narrowest-slot-width distribution of accepted depth-4 tuples to report on
  the main arm.** The distribution exists only on `aux96`
  (`measurements/b47d02/depth4-detail.txt`): 4 241 accepted depth-4 tuples spread over
  narrowest widths 0–108, densest at 10–19 (1 110), 99–108 (555), 30–39 (684) — i.e. the
  accepted deep tuples are **not** concentrated at the cheap narrow slots, because a narrow
  slot means a small span and a coarse sweep stride.
* **On `aux96` the requested shape is still not reached.** 34 distinct four-slot sets are
  offered; the canonical target's **{0,3,4,5}** set — the exact slot pattern of the
  requested clue `hits justice dupe hid came` — is offered **1 085 times** and accepted
  **206 times** (canonical target alone: 110 offered, 43 accepted). Among **all** 20 245
  depth-4 offers, and again among the ~19 800 at `--total-budget 2.0` and `2.5`, the number
  of tuples whose four deep words are exactly `hits dupe hid came` is **0**.
* **Strict pool membership confirms it.** `examples/costpool.rs` now also asks the strict
  question — is there *one* pool phrase whose token at each of slots 0, 3, 4, 5 is
  respectively `hits`, `dupe`, `hid`, `came`? — on both main arms: **strict members 0**
  (`measurements/b47d02/pool-cap4-r*.txt`). The loose word-level test in the harness
  reports `hits=true dupe=true hid=true came=true` on every run, which only means each word
  occurs *somewhere* in some pool phrase; that check should not be read as the milestone.

**Conclusion for this item, plainly: lifting the cap to 4 does not put the requested wording
one step closer. The emission allowance of 16 tuples per segmentation samples roughly
0.001 % of the four-deep rectangle, exactly as the parent report anticipated, so even in the
arm that does construct four-slot classes 1 085 offers at the right slot set produce the
requested word 0 times.**

## 5. Pool delta, and the milestone witness

`measurements/b47d02/pool-cap{3,4}*.txt`, release, default approximate configuration,
`top_n: 20_000`, four runs per arm. Deltas are given as *cap-4 minus cap-3 within the
observed run-to-run band of each arm*, so that a delta inside the band is called noise.

| target | cap 3 (4 runs) | cap 4 (4 runs) | delta vs cap-3 band |
|---|---|---|---|
| It's just a stupid game | 19 601 ×4 | 19 601, 19 613 ×3 | 0 … +12 (band 0) |
| recognize speech | 17 081, 17 083 ×2 | 17 083, 17 102 ×3 | 0 … +21 (band +2) |
| a whole lot of trouble | 17 890 ×2, 17 893 ×2 | 17 890 ×2, 17 893 ×2 | 0 (identical) |
| the cat sat on the mat | 17 750 ×2, 17 752, 17 755 | 17 750 ×2, 17 752, 17 754 | 0 … +4 (band +5) |
| put it back on the shelf | 17 294 ×2, 17 297 | 17 294 ×2, 17 297, 17 300 | 0 … +6 (band +3) |
| when the rain finally stopped | 17 348 ×3, 17 352 | 17 348 ×2, 17 352 ×2 | 0 … +4 (band +4) |
| he was a big fat man | 20 000 (capped) | 20 000 (capped) | 0 |
| what are you going to do | 16 272, 16 273 ×2, 16 276 | 16 265, 16 273, 16 277, 16 282 | −8 … +10 (band +4) |
| she had a lot of money | 14 918 ×2, 14 919 ×2 | 14 919, 14 920 ×2, 14 921 | 0 … +3 (band +1) |
| there is no way to know | 20 000 (capped) | 20 000 (capped) | 0 |
| an old man in a big hat | 14 710, 14 711 ×2, 14 721 | 14 710, 14 721 ×2, 14 722, 14 723 | 0 … +13 (band +11) |
| my brother has a red car | 17 254, 17 255 ×2, 17 260 | 17 253, 17 259 ×2, 17 260 | −2 … +6 (band +6) |

Every delta is at or inside the arm's own run-to-run band. Consistent with §1: the cap
change moves nothing.

**Milestone condition: `recognize speech` → `wreck a nice beach`.**
**It survives on every run of both arms** — 8 of 8 runs, `wreck a nice beach=true`
(`pool-cap3*.txt`, `pool-cap4*.txt`). Not traded away. For scale, the `aux96` arm's pools
are 17 % – 25 % larger (e.g. `an old man in a big hat` 14 711 → 16 620, `recognize speech`
17 083 → 19 246) and it also keeps the witness — but it buys that size by spending 6× the
reserve, which is a different trade and not a proposal.

## 6. Runtime

Interleaved, probe **off**, `--approximate --top 50`, the search milliseconds the binary
reports for itself, 7 runs per arm, arm order alternated every run
(`measurements/b47d02/runtime-interleaved.txt`, `runtime-medians.txt`):

```
It's just a stupid game
  cap4  3058 2509 3301 2709 2429 2606 2516   median 2606  min 2429  max 3301  spread 872
  cap3  2996 2983 2753 3043 3616 2337 2460   median 2983  min 2337  max 3616  spread 1279

recognize speech
  cap4  2037 2112 1809 2564 1648 1959 2033   median 2033  min 1648  max 2564  spread 916
  cap3  1988 2138 1799 1853 1685 2351 2417   median 1988  min 1685  max 2417  spread 732
```

The ranges overlap almost completely on both targets (cap-3's max exceeds cap-4's median on
the canonical target; cap-4's max exceeds cap-3's median on `recognize speech`), and the
within-arm spread (872–1 279 ms) is larger than any between-arm median difference
(377 ms, 45 ms). **No runtime claim is made in either direction.** This is consistent with
§1: the cap-4 binary does the same work.

## 7. Suites

Release, on the main arm (cap 4):

| suite | cap 4 (this branch) | cap 3 (control) |
|---|---|---|
| `--lib` | 53 passed, 0 failed | 50 passed, 0 failed (per `REPORT-e1a3f7.md`) |
| `--test corpus_integration` | 10 passed, **1 failed**: `approximate_finds_classic_madgab_resegmentation` | 10 passed, **1 failed**: same test (re-run here) |
| `--test exact_determinism` | 1 passed | 1 passed |
| `--test approx_determinism` | 2 passed | 2 passed |
| `--test no_phrase_hard_coding` | 6 passed | 6 passed |

The only failure is the pre-existing red, red on the control arm too, and it is this item's
target rather than a regression. **No new red.** Nothing was made green by editing anything,
and a green `--lib` is not milestone progress. `cargo fmt`, `cargo clippy` and doctests do
not exist on this host and are not claimed (`docs/environment-notes.md`).

## 8. The decision this measurement implies

The brief's fork was: *"the depth-4 offered/built/refused triple decides whether the next
front is 'lift the cap and pay for it' or 'lift the cap **and** change how a deep tuple is
priced'."* The triple, measured:

1. **On the main arm the triple is 0 / 0 / 0** — the cap is not the gate, so "lift the cap"
   is not a front at all. It is a no-op. The gate is `EMIT_PROFILE_RESERVE = 16` against a
   breadth-before-depth ordering that puts the first four-slot class at position 42 (6
   slots) / 93 (8 slots).
2. **When depth 4 is genuinely offered (`aux96`), the triple is 20 245 / 4 241 / 16 004 —
   79.05 % refused** (67.07 % on the canonical target), and the refusals are the additive
   `total_budget`: at `--total-budget 2.0` the same tuples are accepted 89.2 % of the time.
3. **And even then the requested wording is offered 0 times** out of 1 085 offers at its
   exact slot set {0,3,4,5}, at three different budgets, and 0 strict pool members.

So the answer to the fork is: **neither, as stated.** Lifting the cap buys nothing; lifting
the cap *together with* the reserve would buy a lot of four-deep tuples at 21 % acceptance
and would still not produce the requested clue, because a 16-tuple (or even 96-tuple)
allowance per segmentation samples a vanishing fraction of the four-deep rectangle. The
next front is a **sampling / enumeration** front — how the per-segmentation allowance is
spent over deep shapes — not a cap front and not, on this evidence, a pricing front. The
pricing question does have a real answer available (the 79 % is a budget artefact, and the
requested tuple's own cost is 0.9195), but pricing is not what is standing in the way.

**Blockers, reported as blockers, not fixed.**

* **`EMIT_PROFILE_RESERVE = 16` and the breadth-before-depth order** are the binding
  constraints on four-deep coverage. Lifting them has a measured pool cost (`aux96`: +17 %
  to +25 % pool size) that has not been priced against the guard set, and doing so is
  explicitly a different front.
* **The emission allowance samples ~0.001 % of the four-deep rectangle.** Until a front
  changes what the allowance buys, no cap value and no budget value can reach the requested
  wording. This is the real blocker on the milestone and this front did not touch it.
* **The probe trace is not reproducible run to run** (§1.1, ±11 accepted tuples). Any future
  front comparing arms on differences below that band is measuring noise, and the existing
  `approx_determinism` test does not cover it. Reported, not chased.
* `EMIT_PROFILE_MAX_DEEP <= 4` is asserted internally, so 4 is the ceiling of the current
  design; going deeper requires changing that assertion, i.e. a test change, which is not
  this front's to make.

## 9. Artifacts on this branch

```
REPORT-b47d02.md                          this report
src/lib.rs                                one line: EMIT_PROFILE_MAX_DEEP 3 -> 4
examples/costpool.rs                      + generic slot-exact witness check
tools-b47d02/measure.sh                   probe runner (12 targets, --approximate --top 50)
tools-b47d02/timeit.sh                    interleaved wall-clock runner
tools-b47d02/an1..an5.js                  the aggregation scripts used below
measurements/b47d02/probe-cap3{,-r1..r3}.tsv   45 387 offered tuples, control arm, 4 runs
measurements/b47d02/probe-cap4{,-r1..r3}.tsv   45 387 offered tuples, main arm, 4 runs
measurements/b47d02/probe-aux96{,-tb2.0,-tb2.5}.tsv   auxiliary arm, 96-tuple reserve
measurements/b47d02/depth-summary.txt      per-depth and per-target offered/acc/ref
measurements/b47d02/replicate-summary.txt one line per replicate, both main arms
measurements/b47d02/depth-reach.txt       per-phase max-depth reach, both main arms + aux
measurements/b47d02/aux96-per-target-d4.txt       depth-4 row per target, auxiliary arm
measurements/b47d02/aux96-budget-sensitivity.txt  depth rows at total_budget 1.5/2.0/2.5
measurements/b47d02/depth4-detail.txt      slot-set and narrowest-width detail, depth 4
measurements/b47d02/pool-cap3*.txt        pool membership, control arm, 4 runs
measurements/b47d02/pool-cap4*.txt        pool membership, main arm, 4 runs
measurements/b47d02/pool-aux96.txt        pool membership, auxiliary arm
measurements/b47d02/runtime-*.txt         interleaved wall clock and medians
```

Probe TSV columns: `target, coverage_phase, slot_count, depth, narrowest_width, total_cost,
accepted, deep_ranks (slot:rank), spread, deep_words` — unchanged from `e1a3f7`.
