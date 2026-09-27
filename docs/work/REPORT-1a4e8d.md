---
work_item: w-1a4e8d
report: true
id: REPORT-1a4e8d
state: done
branch: madgab-reserve-1a4e8d
base: post-milestone-acceptance at fea96d4
verdict: HOLD
updated: 2026-09-28T00:40:00Z
---

# REPORT-1a4e8d — the partial case measured: the reserve has no admissible interior, and the withheld candidate is not withheld by the reserve at all

## 0. What this front was asked, and what it found

`REPORT-7c9d21.md` §3 priced the post-pool selection layer's available lift at **+0.007805**
printed top-50 `SIMILARITY` by removing the layer wholesale, and then wrote:

> There is no partial version either: the lift is monotone in the reserve size and the reserve
> size is pinned at exactly 12 by a fence that reads exactly `TOP_N / 4` […] That last step is an
> argument, not a measurement.

This front ran the sweep. The answer is that the argument was correct, and it is now a
measurement: **the printed structure count is a nondecreasing function of the reserve size, it
equals the reserve size for every reserve size from 7 to 12, and the fence needs 12. The only
reserve size that keeps `approximate_list_represents_enumerated_resegmentation` green is the
shipped 12.** The largest admissible reduction is **zero**, and its measured printed top-50 cost is
therefore **zero**. There is no interior point to price.

A second, more useful thing fell out of the sweep, and it is a *correction of emphasis* rather
than of fact: **the withheld pool-rank-48 candidate is not recoverable by the reserve axis at
any reserve size, including 0.** Removing the reserve moves the `top_n` cutoff from pool rank 42
out to 58 — so the cutoff is no longer what excludes it — and a *shallower member of its own
structure* takes the slot that would have gone to it. §3 is right that the reserve is the reason
the cutoff sits at 42, and right that the cutoff is the mechanism at the shipped reserve; what it
does not say is that the mechanism *changes* as the reserve is reduced, so that no amount of
reserve reduction recovers the candidate. Details in §3.

`src/lib.rs` is byte-identical to `fea96d4`. This is a measurement-only front and its verdict is
**HOLD**.

## 1. Is the reserve size shared? No — it is selection-local, and I varied it as such

This had to be settled before varying anything, because the item forbids widening a shared
constant's blast radius. From `src/`, at the base `fea96d4`:

| symbol | production call sites | other references |
|---|---|---|
| `STRUCTURE_RESERVE_DIVISOR` (`:4089`, value 4) | **one** — read only inside `structure_reserve_slots` (`:4078`) | doc comments at `:4066`, `:4069`, `:4082-4084` |
| `structure_reserve_slots` (`:4077-4079`) | **one** — `let reserve = structure_reserve_slots(top_n);` at `src/lib.rs:3991`, inside `select_diverse`'s step 1 | two in-`src` unit tests: `:4486`/`:4491` (the synthetic-pool reserve test) and `:4507-4515` (`structure_reserve_is_bounded_by_the_slot_count`, which pins `structure_reserve_slots(50) == 12`) |

So the divisor feeds exactly one production decision: how many slots `select_diverse` step 1
spends on the best member of each newly seen boundary structure. It is not read by the
enumeration budget (`structure_wording_allowance` at `:4126` is `share_cap`, a *different*
function), by `share_cap` (`:4098-4104`), by the scorer, or by the objective weights. Nothing
outside printed approximate selection observes it.

**Therefore I varied a selection-local parameter and did not widen a shared constant.** In
practice that took the form of threading an optional reserve through `select_diverse` on a scratch
branch (§5) rather than editing `STRUCTURE_RESERVE_DIVISOR` at all, and for a second reason worth
stating: the divisor can only ever produce `50 / D`, i.e. the reserve sizes **50, 25, 16, 12, 10,
8, 7, 6, 5, 4, 3, 2, 1** — it cannot produce 11 or 9, which are exactly the interior points the
question needs. Sweeping the reserve *size* rather than the divisor is both the local parameter
and the finer axis. Had the divisor been shared, the front would have had to stop and report that
instead.

**No collision.** The sweep needed no edit to the objective weight vector (`axes::*`) or to the
score function (`Metrics::combined`), so the surfaces of `w-e086cc` and `w-3f6a21` were not
touched. Neither `/workspace/madgab-e086cc` nor `/workspace/madgab-pairscore-3f6a21` was read,
entered or run.

## 2. Method, and why these numbers are the production numbers

Same shape as `REPORT-7c9d21` §2, re-run rather than inherited:

* For each of the three targets in the record's table — `the cat sat on the mat`,
  `she sells sea shells`, `recognize speech` — build the shipped `Generator`
  (`SearchMode::approximate()`, `top_n = 50`, `beam_width = 64`), take `generate_pool` (the
  post-dedup, pre-selection pool, public since `src/lib.rs:894`), and take `counters::drain_scored`
  for the per-candidate `SIMILARITY`.
* **Replay check, asserted in the probe, not assumed:** the deduped `Scored` rows equal the pool
  element-for-element — same length, same phrase order, same score at every index — and replaying
  the selection rule with the reserve pinned to `structure_reserve_slots(50)` reproduces
  `generate`'s printed 50 **exactly** on all three targets. So every row below is the production
  trace at that reserve size, not a re-derivation of it.
* `SIMILARITY` is `counters::Scored::similarity`, the same per-candidate axis the record priced,
  and the printed mean is the mean over the printed 50.

**Independent confirmation that the harness is the record's harness** — every anchor in
`REPORT-7c9d21` reproduces to the digit: pool `18289`; printed mean `SIMILARITY` `0.848330`;
printed mean score `0.919922801`; pool mean `SIMILARITY` `0.795259`; deepest admitted pool rank
`227`; green case at pool rank 27 and printed rank 27; the withheld member at pool rank 48 with
score `0.918840116882308` and `in_printed = false`; and full layer removal giving printed mean
`SIMILARITY` `0.856135` and the structure triple **6 / 2 / 4**. A sweep that cannot reproduce the
endpoints is not measuring the same thing, and this one does.

## 3. Criterion 1 — the table

`fence green` is `approximate_list_represents_enumerated_resegmentation`
(`tests/corpus_integration.rs:502`), which requires `shown.len() >= TOP_N / 4 = 12` on its **two**
targets (`the cat sat on the mat`, `she sells sea shells`); `recognize speech` is carried in the
triple because the record's table and this item's criterion 1 both use three, and it is the green
target. The withheld candidate is pool rank 48 on `recognize speech`; its "pool rank" is 48 at
every row because the pool does not change — what changes is whether it is printed.

Printed top-50 mean `SIMILARITY` is on `recognize speech`; `Δ` is against the shipped row.

| reserve `r` | structures (cat / shells / speech) | fence green | green case printed rank | withheld F7 (pool rank 48) printed? | printed mean SIM | Δ vs shipped |
|---|---|---|---|---|---|---|
| **12 (shipped)** | **12 / 12 / 12** | **yes** | **27** | **no** | **0.848330** | 0 |
| 11 | 11 / 11 / 11 | no | 27 | no | 0.849208 | +0.000878 |
| 10 | 10 / 10 / 10 | no | 27 | no | 0.850094 | +0.001764 |
| 9 | 9 / 9 / 9 | no | 27 | no | 0.850569 | +0.002239 |
| 8 | 8 / 8 / 8 | no | 27 | no | 0.852596 | +0.004266 |
| 7 | 7 / 7 / 7 | no | 27 | no | 0.851988 | +0.003658 |
| 6 | 6 / 6 / 6 | no | 27 | no | 0.853345 | +0.005015 |
| 5 | 6 / 6 / 5 | no | 27 | no | 0.853638 | +0.005308 |
| 4 | 6 / 6 / 4 | no | 27 | no | 0.854530 | +0.006200 |
| 3 | 6 / 6 / 4 | no | 27 | no | 0.854530 | +0.006200 |
| 2 | 6 / 6 / 4 | no | 27 | no | 0.854530 | +0.006200 |
| 1 | 6 / 6 / 4 | no | 27 | no | 0.854530 | +0.006200 |
| 0 (reserve removed, cap kept) | 6 / 6 / 4 | no | 27 | no | 0.854530 | +0.006200 |
| *full layer removal (no reserve, no cap)* | *6 / 2 / 4* | *no* | *27* | ***yes*** | *0.856135* | *+0.007805* |

The last row is the record's comparison point (b), shown for reconciliation and **not** a point on
the reserve-size axis: it also removes the share cap, which no reserve value does.

Three things to read off the table.

**(a) The structure count is `r` itself, for every `r` from 7 to 12, on all three targets.** Not
approximately — exactly, three targets agreeing. Below `r = 7` the count saturates (6 on cat and
shells, 4 on speech) because the reserve can only spend a slot on a *newly seen* structure and the
cap-limited score walk has already represented more structures than the reserve can reach. Either
way the sequence is **nondecreasing in `r` on all three targets**, which is the monotonicity
`REPORT-7c9d21` §3 asserted and did not measure. It is now measured.

**(b) The `6 / 2 / 4` triple in the record is not a reserve size.** It is the full-removal list, and
it is reached only by removing the cap as well. Along the reserve axis the triple is
`6 / 6 / 4` at `r = 0` — cat agrees with the record, shells does not, because at `r = 0` the share
cap is still doing the work that puts six structures in the shells list. A reader who assumed
`6 / 2 / 4` was the `r = 0` row would be wrong on one target in three, and would draw the wrong
picture of how much of the structure count the reserve is responsible for versus the cap. Worth
recording; it does not change the verdict, and both readings put every reserve value below 12 in
the red.

**(c) The green case is printed rank 27 at every reserve size, 0 through 12.** It is admitted by
the score walk, sits at pool rank 27, and is inside the cutoff at every point of the sweep. It is
structurally immune to this coordinate, exactly as `REPORT-7c9d21` §3 said, now measured rather
than reasoned.

### The withheld candidate, which is the more interesting row

The record's mechanism is: *reserve spends 12 of 50 slots, so the score walk's 50th slot is
consumed at pool rank 42, so the `top_n` cut at `src/lib.rs:4025` drops rank 48.* That is correct
at the shipped reserve, and the sweep confirms the cutoff arithmetic exactly — printed pool ranks
at `r = 12` are `[40, 41, 42]` and nothing deeper.

What the sweep adds is that the mechanism is **not stable under the sweep**. Measured members of
the withheld candidate's own structure (270 pool members, share cap 17) in the printed list:

| `r` | members of its structure printed | at pool ranks | cutoff reaches | rank 48 printed? |
|---|---|---|---|---|
| 12, 11, 10, 9 | **16** | `[5, 6, 10, 13, 15, 16, 18, 20, 21, 24, 26, 27, 29, 32, 34, 40]` | 42 – 43 | no |
| 8 … 0 | **17** | `[…, 40, **46**]` | 58 – 218 | no |

So the instant the reserve stops spending 12 slots, the cutoff moves out past rank 48 — and rank
48 *still* does not print, because the freed slots are absorbed by a **shallower member of its own
structure, pool rank 46**, which becomes its 17th and exhausts `share_cap(50, 18289) = 17`. The
candidate is the 18th member of a saturated structure. It takes the *full* removal of the cap as
well (last row of the table) for it to print.

**Consequence.** On the reserve axis the withheld candidate is not merely unrecovered at the
shipped setting — it is unrecovered at *every* setting, including zero. No reserve size recovers
F7. If F7 is to be recovered it is `share_cap` / `STRUCTURE_FLOOR` that has to move, which is a
different parameter on a different fence, and is not this front's surface.

## 4. Criterion 2 — the largest green reduction, and its cost

**The largest reserve reduction that keeps `approximate_list_represents_enumerated_resegmentation`
green is zero slots. The reserve must stay at `structure_reserve_slots(50) = 12`.**

The settling number is **`r = 11` → printed structures `11 / 11 / 11`, against a required 12.**
One slot is one structure, and the fence is `>= 12`, so `11 < 12` is red on both of the fence's
own targets, not marginally red on a third-party one. The next point up, `r = 12`, is green. The
boundary is a single step and it is exactly where the shipped value sits.

**The measured printed top-50 cost of exactly that reduction is therefore `+0.0000000000`**, since
the only green reduction is the identity. That is a real answer and it is the whole answer; there
is no non-zero cost to quote, because there is no non-zero reduction.

For the coordinator's benefit, and clearly labelled as *not* the answer, the cost of the nearest
red step is measured too: `r = 11` would buy **`+0.000878`** printed top-50 `SIMILARITY` on the
green target (mean `SIMILARITY` 0.848330 → 0.849208, mean score 0.919922801 → 0.920039783) and
would spend it on nothing, because the green case stays at printed rank 27 and the withheld
candidate stays withheld. So the whole ladder is priced and none of it is collectible:

* `r = 11`, the nearest green-adjacent step: `+0.000878` — turns the fence red.
* `r = 0`: `+0.006200` of the available `+0.007805` (79.5%) — still fence red, still no F7.
  The missing `+0.001605` to reach the record's figure is the cap, not the reserve.
* full removal: `+0.007805` — fence red at 6 / 2 / 4.

## 5. Criterion 3 — does the partial case change the `w-7c9d21` closure verdict?

**No. It does not change it, and the settling number is `11 < 12`.**

`w-7c9d21` closed **HOLD** on the grounds that the selection layer's `+0.007805` is entirely the
representation reserve and the representation reserve is a fence. The sweep confirms both halves
as measurements: the lift is available only at reserve sizes that make the fence red, and the
fence is red at every reserve size below 12. The one step the report left as an argument is now
closed in the report's favour, so its verdict stands as written rather than as a provisional.

The three specific things `REPORT-7c9d21` §4 said a successor should not re-open, restated against
the measurement:

1. **"Every value of `STRUCTURE_RESERVE_DIVISOR` that keeps the fence green yields the same printed
   list."** Confirmed, and slightly strengthened: every value of the reserve *size* that keeps the
   fence green is the single value 12, because the printed structure count equals the reserve size
   on this range. There is exactly one green point, and it is the shipped one.
2. **"A successor should not open a third selection-side front."** Still the right routing. The
   selection layer's reserve axis is now fully swept and has no admissible interior, so it is
   closed as a surface, not merely priced.
3. **"The head lift is an objective-side coordinate."** Unchanged, and this front adds one line to
   it: the share cap is the *other* half of the selection layer's cost, it is worth a measured
   `+0.001605` on the green target beyond what the reserve accounts for, and it is a distinct
   parameter. It is **not** a recommendation and **no successor front is proposed here** — the item
   asks for one only if the verdict changes, and it does not. Recording the cap's measured size is
   a fact for whoever owns that surface, not an opening of it.

No `OBSTRUCTION-MAP.md` row is re-opened, nothing is re-pinned, and
`approximate_output_is_locked` / the representation fence were re-run, not re-baselined.

## 6. Fence suites at the base, run honestly

`cargo test --release`, `--test-threads=1` except where noted, dedicated
`CARGO_TARGET_DIR=/workspace/target-1a4e8d`. Run on the clean tree with the probe **absent**.

| suite | result |
|---|---|
| `no_phrase_hard_coding` | **9 passed / 0 failed** |
| `emit_coverage` | **7 passed / 0 failed** |
| `approx_determinism` | **4 passed / 0 failed** |
| `exact_determinism` | **1 passed / 0 failed** |
| `cargo test --release --lib` | **75 passed / 0 failed / 12 ignored** |
| `corpus_integration` (`--test-threads=2`) | **12 passed / 1 failed** |

The single `corpus_integration` failure is
`approximate_finds_classic_madgab_resegmentation` (`tests/corpus_integration.rs:134`), the
**known pre-existing base red** from `OBSTRUCTION-MAP.md` §4, reproduced identically before and
after the probe was reverted. It was **not** touched and **not** re-pinned.
`approximate_list_represents_enumerated_resegmentation` **ok**;
`approximate_output_is_locked` **ok**;
`approximate_finds_recognize_speech_resegmentation` **ok**.

`corpus_integration` was run at `--test-threads=2` because `OBSTRUCTION-MAP.md` §4 records
SIGKILL at default parallelism on this host at base too; every other run was `--test-threads=1`.

**`cargo fmt`, `cargo fmt --check` and `cargo clippy` cannot run on this host** — there is no
`rustup` on `PATH` and cargo reports `no such command: fmt` / `no such command: clippy`. They are
**not claimed**. Doctests were not run. The report is the only file this front changes.

## 7. Constraints

* **Measurement only, regression-free by construction.** `git diff fea96d4` on this branch is
  `docs/work/REPORT-1a4e8d.md` and nothing else. `src/lib.rs`, `tests/`, `examples/`, `web/` and
  `Cargo.toml` are byte-identical to `fea96d4`; `git status` clean. No production, test, example
  or manifest line reaches this branch.
* **The probe lived on `scratch/1a4e8d-reserve`, a local, unpushed branch**, and consisted of a
  `#[cfg(test)] mod reserve_sweep_probe` plus a one-line threading of an `Option<usize>` reserve
  through `select_diverse`. It was reverted (`git checkout src/lib.rs`, branch deleted) **before**
  every suite in §6 was run, so the numbers in §6 are the numbers the tree ships. The replay
  assertion in the probe (§2) is what licenses the table in §3: it is checked in-probe and would
  have failed loudly rather than printing a plausible wrong table.
* **No phrase-specific hard-coding.** The three target strings and the green case's clue text are
  test *data* inside `#[cfg(test)]` on the reverted scratch branch, never in production code, and
  no sentence, clue, word, canonical example or exact rank from any canonical example is
  special-cased in anything that reaches this branch. The withheld candidate is **not** located by
  a hard-coded rank anywhere: the sweep reports the printed pool ranks of a contiguous window and
  the members of one structure, and the record's `pool rank 48 / score 0.918840116882308` is
  *confirmed* against this run (it reproduced exactly) rather than assumed by the code.
  `no_phrase_hard_coding` is 9/9 with the probe absent, which is the state that ships.
* **No fence weakened, deleted, re-pinned or `#[ignore]`d.** Every test in §6 is byte-identical to
  base.
* **Objective weight vector and score function untouched.** `axes::*` and `Metrics::combined` were
  not edited; the sweep needed no such edit, so there is no surface collision to report.
  `/workspace/madgab-e086cc` and `/workspace/madgab-pairscore-3f6a21` were not read, entered or run.
* **The representation fence was not re-baselined**, per `REPORT-4d1e93.md` §4; it was measured
  and found green at the shipped reserve and red at every smaller one.
* **Nothing self-merged.** Not into `post-milestone-acceptance`, not into `main`. This report goes
  to `madgab-reserve-1a4e8d`. `scratch/1a4e8d-reserve` was local-only and is deleted.
