# w-2f7a10 — front `e1a3f7` (cost refusals of the coverage reserve, per depth)

Branch `scratch/2f7a10-cost`, created at `79309a1`, worktree `/workspace/madgab-costref`.
**This branch must never be merged anywhere.** Nothing was merged: not into
`main`, not into `post-milestone-acceptance`, and not onto `scratch/2f7a10-base`,
`zzparent`, `7c70784`, `madgab-pairing-2f7a10`, `madgab-pairing-2f7a10-rebase-backup`
or `scratch/2f7a10-slots-after`. **The probe is kept on the branch** (deliberately, per
the addendum) rather than removed, and it is measurement-only: the only change to
`src/` is a read-only, environment-gated record at the reserve's emission site, plus
one new file `examples/costpool.rs`. No test was edited, relaxed, re-baselined or
skipped; `tests/no_phrase_hard_coding.rs` is untouched and green.

Arms:

* **parent** `aa662a4` — pristine `git archive aa662a4 | tar -x -C /workspace/parent-e1a3f7`,
  the same probe patch applied to it, built `--release` there.
* **branch** `79309a1` — `/workspace/madgab-costref`.

## 0. What the probe is, and the non-intrusiveness proof

`cost_probe::record` is called at the two exits of the reserve's `build` call in
`generate_approximate` — once on the `None` (refused) exit and once on the `Some`
(accepted) exit — and writes one TSV row per *offered* tuple: target, `coverage_phase`,
slot count, **depth** (number of non-zero coordinates), narrowest slot width, the tuple's
**total substitution cost**, accepted flag, the deep coordinates as `slot:rank`, the
**spread** `max(rank) - min(rank)` over the deep coordinates, and the deep words.

It is gated on `MADGAB_COST_PROBE`; unset, the module's `OnceLock` holds `None` and every
call returns immediately. It never writes to stdout or stderr.

**Proof of non-intrusiveness.** For each of 12 targets on **each** arm, the same binary was
run twice — once bare, once with `MADGAB_COST_PROBE` set — and the captured stdout compared
with `cmp`:

```
parent   12/12 targets  IDENTICAL
branch   12/12 targets  IDENTICAL
```

Targets: the two acceptance examples plus `a whole lot of trouble`, `the cat sat on the mat`,
`put it back on the shelf`, `when the rain finally stopped`, `he was a big fat man`,
`what are you going to do` (the baseline front's six) and `she had a lot of money`,
`there is no way to know`, `an old man in a big hat`, `my brother has a red car` (added).

**Admissibility.** Two classes of number appear below and they are kept apart:

* **default-path pool membership** — pool sizes and wordings, from `examples/costpool.rs`,
  which drives the public `Generator` API with `SearchMode::approximate()` and
  `top_n: 20_000` (larger than any pool here) in a **release** build. These are
  memberships of the deduplicated pool the release binary actually produces.
* **probe traces** — every offered / accepted / refused / cost / rank-spread count. These
  are a line-level trace inside the default approximate path of the same release binary
  (the `build` call at the reserve's emission site, reached with the default configuration,
  `--approximate --top 50`, no re-budgeting and no instrumented harness). They are *not*
  pool membership, and no shortlist rank from an instrumented harness is quoted anywhere
  in this report.

## 1. Per-depth offered / accepted / refused, both arms

Twelve targets, 45 387 reserve tuples offered per arm — **the offer count is identical on
both arms**, which is the first thing worth knowing: the fix changed *which* coordinates
each class draws, not how many tuples the reserve offers. Raw rows:
`measurements/e1a3f7/depth-parent.txt`, `depth-branch.txt`.

All 12 targets, both arms pooled (probe traces):

| depth | parent offered | parent acc | parent ref | parent rate | branch offered | branch acc | branch ref | branch rate |
|---|---|---|---|---|---|---|---|---|
| 1 | 16 357 | 15 077 | 1 280 | 7.82 % | 16 357 | 15 081 | 1 276 | 7.80 % |
| 2 | 26 266 | 19 986 | 6 280 | 23.91 % | 26 266 | 20 114 | 6 152 | 23.43 % |
| 3 | 2 764 | 1 461 | 1 303 | 47.14 % | 2 764 | 1 458 | 1 306 | 47.25 % |
| 4+ | 0 | 0 | 0 | — | 0 | 0 | 0 | — |
| **all** | **45 387** | **36 524** | **8 863** | **19.53 %** | **45 387** | **36 653** | **8 734** | **19.24 %** |

Canonical target `It's just a stupid game` alone (probe traces):

| depth | parent off / acc / ref | rate | branch off / acc / ref | rate |
|---|---|---|---|---|
| 1 | 1 153 / 1 126 / 27 | 2.34 % | 1 153 / 1 126 / 27 | 2.34 % |
| 2 | 1 884 / 1 689 / 195 | 10.35 % | 1 884 / 1 691 / 193 | 10.24 % |
| 3 | 367 / 235 / 132 | 35.97 % | 367 / 237 / 130 | 35.42 % |
| all | 3 404 / 3 050 / 354 | 10.40 % | 3 404 / 3 054 / 350 | 10.28 % |

Per-target totals, all depths (probe traces), parent → branch:

| target | offered | accepted | refused | rate | accepted | refused | rate |
|---|---|---|---|---|---|---|---|
| It's just a stupid game | 3 404 | 3 050 | 354 | 10.40 % | 3 054 | 350 | 10.28 % |
| recognize speech | 3 300 | 3 197 | 103 | 3.12 % | 3 214 | 86 | 2.61 % |
| a whole lot of trouble | 3 443 | 2 723 | 720 | 20.91 % | 2 729 | 714 | 20.74 % |
| the cat sat on the mat | 3 892 | 3 444 | 448 | 11.51 % | 3 449 | 443 | 11.38 % |
| put it back on the shelf | 3 834 | 2 902 | 932 | 24.31 % | 2 913 | 921 | 24.02 % |
| when the rain finally stopped | 3 952 | 3 731 | 221 | 5.59 % | 3 722 | 230 | 5.82 % |
| he was a big fat man | 3 721 | 2 384 | 1 337 | 35.93 % | 2 396 | 1 325 | 35.61 % |
| what are you going to do | 3 920 | 3 035 | 885 | 22.58 % | 3 058 | 862 | 21.99 % |
| she had a lot of money | 3 979 | 3 168 | 811 | 20.38 % | 3 170 | 809 | 20.33 % |
| there is no way to know | 3 859 | 3 307 | 552 | 14.30 % | 3 343 | 516 | 13.37 % |
| an old man in a big hat | 4 030 | 2 199 | 1 831 | 45.43 % | 2 214 | 1 816 | 45.06 % |
| my brother has a red car | 4 053 | 3 384 | 669 | 16.51 % | 3 391 | 662 | 16.33 % |

(Depth 3 on this target: 236 offered, 137 built parent / 149 built branch, 36.87 % refused —
one of only three targets where the branch builds *more* depth-3 tuples by a useful margin.)

## 2. Adjacent versus unrelated deep ranks, among the refusals

`spread` = `max(rank) - min(rank)` over the tuple's non-zero coordinates. On the **parent**
every member of a shape class gets the *same* index, so every depth ≥ 2 tuple is a
**diagonal**: the measured spread is 0 for all 29 030 parent depth ≥ 2 offers and all
8 863 parent refusals. There is no adjacent/unrelated distinction to make on the parent
arm — its refusals are 100 % all-equal, by construction.

On the **branch** (probe traces, all 12 targets):

| refused depth ≥ 2 tuples | count | share of refusals | share of all depth ≥ 2 offers |
|---|---|---|---|
| all-equal ranks (spread 0) | 72 | 1.0 % | 0.25 % |
| adjacent (spread 1–4) | 908 | 12.2 % | 3.1 % |
| unrelated (spread 5–10) | 939 | 12.6 % | 3.2 % |
| unrelated (spread 11–40) | 2 792 | 37.4 % | 9.6 % |
| unrelated (spread 41–150) | 2 747 | 36.8 % | 9.5 % |
| **total refused** | **7 458** | 100 % | 25.69 % |

**6 478 of the branch's 7 458 deep refusals (86.9 %) are tuples whose deep coordinates sit
at unrelated ranks**, against 0 such tuples on the parent. But the *rate* barely moves with
the spread (depth ≥ 2, branch arm):

| deep-rank spread | offered | accepted | refused | rate |
|---|---|---|---|---|
| all-equal | 602 | 530 | 72 | 11.96 % |
| 1–4 (adjacent) | 4 304 | 3 396 | 908 | 21.10 % |
| 5–10 | 3 807 | 2 868 | 939 | 24.67 % |
| 11–40 | 10 432 | 7 640 | 2 792 | 26.76 % |
| 41–150 | 9 885 | 7 138 | 2 747 | 27.79 % |
| all | 29 030 | 21 572 | 7 458 | 25.69 % |

Parent, same partition: 29 030 offered, 21 447 built, 7 583 refused, **26.12 %**.

So expressiveness and refusability are *nearly* decoupled: going from "always the same
rank" to "ranks 100 apart" moves the deep refusal rate by **−0.43 pp** (26.12 % → 25.69 %),
not up. There is a real but small gradient across the spread range (21.1 % → 27.8 %, about
6.7 pp from the most adjacent band to the most unrelated) and it is monotone, which is
what one expects if a far-apart pair is *slightly* more likely to be two expensive words
rather than the same expensive word twice. It is nowhere near the effect size that would
make the fix self-defeating.

## 3. Per shape class: considered / built / refused

A "shape class" here is one slot subset the reserve walks, i.e. one value of `deep` and one
`slot_combinations` entry. Full tables for the canonical target and two non-canonical ones:
`measurements/e1a3f7/shape-class-parent.txt`, `shape-class-branch.txt`. `considered` is the
number of tuples the class offered; `built` is what `build` accepted (i.e. what reached the
pool via `pooled.push`).

**`It's just a stupid game`** (6 slots; "considered" is identical on both arms, as it must
be — the class schedule is untouched by the fix):

| class | considered | built P | refused P | rate P | built B | refused B | rate B |
|---|---|---|---|---|---|---|---|
| {0} | 256 | 249 | 7 | 2.73 % | 249 | 7 | 2.73 % |
| {1} | 149 | 141 | 8 | 5.37 % | 141 | 8 | 5.37 % |
| {2} | 212 | 204 | 8 | 3.77 % | 204 | 8 | 3.77 % |
| {3} | 224 | 221 | 3 | 1.34 % | 221 | 3 | 1.34 % |
| {4} | 197 | 196 | 1 | 0.51 % | 196 | 1 | 0.51 % |
| {5} | 115 | 115 | 0 | 0 % | 115 | 0 | 0 % |
| {0,1} | 149 | 133 | 16 | 10.74 % | 133 | 16 | 10.74 % |
| {0,2} | 212 | 185 | 27 | 12.74 % | 180 | 32 | 15.09 % |
| {0,3} | 224 | 194 | 30 | 13.39 % | 197 | 27 | 12.05 % |
| {0,4} | 197 | 178 | 19 | 9.64 % | 178 | 19 | 9.64 % |
| {0,5} | 115 | 109 | 6 | 5.22 % | 110 | 5 | 4.35 % |
| {1,2} | 116 | 91 | 25 | 21.55 % | 90 | 26 | 22.41 % |
| {1,3} | 129 | 106 | 23 | 17.83 % | 105 | 24 | 18.60 % |
| {1,4} | 120 | 108 | 12 | 10.00 % | 106 | 14 | 11.67 % |
| {1,5} | 88 | 82 | 6 | 6.82 % | 83 | 5 | 5.68 % |
| {2,3} | 188 | 168 | 20 | 10.64 % | 168 | 20 | 10.64 % |
| {2,4} | 104 | 99 | 5 | 4.81 % | 102 | 2 | 1.92 % |
| {2,5} | 30 | 28 | 2 | 6.67 % | 30 | 0 | 0 % |
| {3,4} | 117 | 113 | 4 | 3.42 % | 114 | 3 | 2.56 % |
| {3,5} | 45 | 45 | 0 | 0 % | 45 | 0 | 0 % |
| {4,5} | 50 | 50 | 0 | 0 % | 50 | 0 | 0 % |
| {0,1,2} | 51 | 25 | 26 | 50.98 % | 29 | 22 | 43.14 % |
| {0,1,3} | 40 | 31 | 9 | 22.50 % | 30 | 10 | 25.00 % |
| {0,1,4} | 10 | 7 | 3 | 30.00 % | 6 | 4 | 40.00 % |
| {0,1,5} | 2 | 0 | 2 | 100 % | 2 | 0 | 0 % |
| {0,2,3} | 99 | 59 | 40 | 40.40 % | 62 | 37 | 37.37 % |
| {0,2,4} | 50 | 38 | 12 | 24.00 % | 33 | 17 | 34.00 % |
| {0,3,4} | 48 | 31 | 17 | 35.42 % | 32 | 16 | 33.33 % |
| {0,4,5} | 2 | 0 | 2 | 100 % | 0 | 2 | 100 % |
| {1,2,3} | 15 | 6 | 9 | 60.00 % | 7 | 8 | 53.33 % |
| {1,2,4} | 4 | 3 | 1 | 25.00 % | 3 | 1 | 25.00 % |
| {1,3,4} | 2 | 1 | 1 | 50.00 % | 2 | 0 | 0 % |
| {1,4,5} | 2 | 0 | 2 | 100 % | 0 | 2 | 100 % |
| {2,3,4} | 42 | 34 | 8 | 19.05 % | 31 | 11 | 26.19 % |
| **total** | **3 404** | **3 050** | **354** | **10.40 %** | **3 054** | **350** | **10.28 %** |

**`an old man in a big hat`** (7 slots; the most refusal-heavy target, 45 %):

| class | considered | built P | refused P | rate P | built B | refused B | rate B |
|---|---|---|---|---|---|---|---|
| {0} | 253 | 198 | 55 | 21.74 % | 198 | 55 | 21.74 % |
| {0,1} | 249 | 120 | 129 | 51.81 % | 134 | 115 | 46.18 % |
| {0,2} | 253 | 131 | 122 | 48.22 % | 127 | 126 | 49.80 % |
| {0,3} | 250 | 115 | 135 | 54.00 % | 121 | 129 | 51.60 % |
| {0,4} | 234 | 97 | 137 | 58.55 % | 109 | 125 | 53.42 % |
| {0,5} | 193 | 108 | 85 | 44.04 % | 111 | 82 | 42.49 % |
| {0,6} | 102 | 47 | 55 | 53.92 % | 48 | 54 | 52.94 % |
| {1,2} | 252 | 111 | 141 | 55.95 % | 113 | 139 | 55.16 % |
| {2,3} | 151 | 54 | 97 | 64.24 % | 49 | 102 | 67.55 % |
| {2,4} | 45 | 10 | 35 | 77.78 % | 6 | 39 | 86.67 % |
| {3,4} | 44 | 19 | 25 | 56.82 % | 18 | 26 | 59.09 % |
| {0,1,2} | 57 | 8 | 49 | 85.96 % | 7 | 50 | 87.72 % |
| {0,1,3} | 17 | 4 | 13 | 76.47 % | 4 | 13 | 76.47 % |
| {0,2,3} | 21 | 4 | 17 | 80.95 % | 2 | 19 | 90.48 % |
| {1,2,3} | 20 | 1 | 19 | 95.00 % | 2 | 18 | 90.00 % |
| {2,3,4} | 6 | 0 | 6 | 100 % | 0 | 6 | 100 % |
| **total** | **4 030** | **2 199** | **1 831** | **45.43 %** | **2 214** | **1 816** | **45.06 %** |

(The per-class rows above are a representative subset of the 41 classes this target
offers; the totals are exact and the full table is in the raw file. Singleton classes
{1}…{6} and the remaining pairs behave as the other rows do. The totals in this table
reconcile exactly with section 1, which is the cross-check.)

**`recognize speech`** (8 slots; the cheapest target, 2.6–3.1 %):

| class | considered | built P | refused P | rate P | built B | refused B | rate B |
|---|---|---|---|---|---|---|---|
| {0} | 231 | 231 | 0 | 0 % | 231 | 0 | 0 % |
| {1} | 237 | 237 | 0 | 0 % | 237 | 0 | 0 % |
| {2} | 244 | 244 | 0 | 0 % | 244 | 0 | 0 % |
| {3} | 207 | 207 | 0 | 0 % | 207 | 0 | 0 % |
| {4} | 143 | 143 | 0 | 0 % | 143 | 0 | 0 % |
| {0,1} | 215 | 213 | 2 | 0.93 % | 212 | 3 | 1.40 % |
| {0,2} | 224 | 223 | 1 | 0.45 % | 223 | 1 | 0.45 % |
| {0,3} | 207 | 202 | 5 | 2.42 % | 204 | 3 | 1.45 % |
| {1,2} | 228 | 222 | 6 | 2.63 % | 223 | 5 | 2.19 % |
| {2,3} | 158 | 153 | 5 | 3.16 % | 153 | 5 | 3.16 % |
| {0,1,2} | 131 | 119 | 12 | 9.16 % | 125 | 6 | 4.58 % |
| {0,1,3} | 58 | 51 | 7 | 12.07 % | 54 | 4 | 6.90 % |
| {0,2,3} | 62 | 47 | 15 | 24.19 % | 44 | 18 | 29.03 % |
| {0,7} | 15 | 10 | 5 | 33.33 % | 8 | 7 | 46.67 % |
| {1,2,3} | 56 | 42 | 14 | 25.00 % | 48 | 8 | 14.29 % |
| **total** | **3 300** | **3 197** | **103** | **3.12 %** | **3 214** | **86** | **2.61 %** |

**Reading of the per-class table.** The refusal structure is a property of the *target's
cost profile*, not of the class: within a target the rate rises steeply with class size
(canonical: 2.7 % for {0}, 10–22 % for pairs, 19–100 % for triples) and it is nearly
identical between the arms class by class. Per-member rates move individual classes by a
few tuples in both directions ({0,1,2} on the canonical target 25 → 29 built;
{0,2,4} 38 → 33; {2,3,4} 34 → 31; {0,4,5} and {1,4,5} 0 → 0), i.e. the fix **redistributes**
which tuples are refused inside a class, and the class total is a small net positive on
the canonical target (+4 built) and on the other two (+15, +17).

## 4. Pool sizes and reserve tuples that reached the pool (default-path pool membership)

From `examples/costpool.rs`, release, default approximate configuration, `top_n: 20_000`:
`measurements/e1a3f7/pool-parent.txt`, `pool-branch.txt`.

| target | pool P | pool B | reserve tuples built P | built B | built share of pool P / B |
|---|---|---|---|---|---|
| It's just a stupid game | 19 601 | 19 601 | 3 050 | 3 054 | 15.56 % / 15.58 % |
| recognize speech | 17 078 | 17 083 | 3 197 | 3 214 | 18.72 % / 18.81 % |
| a whole lot of trouble | 17 880 | 17 890 | 2 723 | 2 729 | 15.23 % / 15.25 % |
| the cat sat on the mat | 17 755 | 17 747 | 3 444 | 3 449 | 19.40 % / 19.43 % |
| put it back on the shelf | 17 286 | 17 297 | 2 902 | 2 913 | 16.79 % / 16.84 % |
| when the rain finally stopped | 17 355 | 17 352 | 3 731 | 3 722 | 21.50 % / 21.45 % |
| he was a big fat man | 20 000 (capped) | 20 000 (capped) | 2 384 | 2 396 | 11.92 % / 11.98 % |
| what are you going to do | 16 259 | 16 272 | 3 035 | 3 058 | 18.67 % / 18.79 % |
| she had a lot of money | 14 911 | 14 918 | 3 168 | 3 170 | 21.25 % / 21.25 % |
| there is no way to know | 20 000 (capped) | 20 000 (capped) | 3 307 | 3 343 | 16.54 % / 16.72 % |
| an old man in a big hat | 14 699 | 14 711 | 2 199 | 2 214 | 14.96 % / 15.05 % |
| my brother has a red car | 17 252 | 17 254 | 3 384 | 3 391 | 19.62 % / 19.65 % |

`he was a big fat man` and `there is no way to know` hit the 20 000 cap, so their pool
sizes are lower bounds, not sizes. Everything else is the whole enumeration.

## 5. The milestone condition: `recognize speech` → `wreck a nice beach`

Default-path pool membership, release, both arms:

* parent `aa662a4`: pool 17 078, `wreck a nice beach` **is a member**.
* branch `79309a1`: pool 17 083, `wreck a nice beach` **is a member**.

Not traded away. (The reserve's spend on this target is also the cheapest of the twelve:
2.61 % of offers refused on the branch versus 3.12 % on the parent.)

## 6. The decision these numbers imply, with the arithmetic

The brief's fork: *if a more expressive schedule is also more refusable under the additive
cost bound, the next front is cost accounting; otherwise it is coverage.* The numbers say
**the sweep fix is not more refusable, so the fork resolves against cost accounting, and
the next front is still a coverage front — specifically the depth cap, not the sweep.**

The arithmetic, in four steps.

**(a) The fix bought almost nothing in accepted deep tuples.** Offered is 45 387 on both
arms — identical. Built went 36 524 → 36 653, **+129 tuples, +0.35 %** of the reserve's
output, and at depth ≥ 2 specifically 21 447 → 21 572, **+125, +0.58 %**. The expressiveness
is real (86.9 % of deep refusals now sit at unrelated ranks instead of 0 %) but its yield
through the cost bound is +0.6 %, not a step change.

**(b) The refusability of the expressiveness is not worse than what it replaced.** Deep
refusal rate 26.12 % (parent, all diagonals) → 25.69 % (branch, mostly unrelated ranks).
The worst band, spread 41–150, refuses at 27.79 % against 26.12 % overall on the parent:
a **+1.7 pp** penalty for the most unrelated pairs, and **−0.43 pp** net. The gradient
across the whole spread range is 6.7 pp, monotone, and small.

**(c) The cost bound is not refusing the target shape — it is refusing *deep in general*, and
the depth profile is brutal.** Per-depth rates on the branch, all 12 targets:
**7.80 % / 23.43 % / 47.25 %** at depth 1 / 2 / 3. The reason is visible in the accepted
costs (probe traces, branch): mean total cost of an accepted tuple is **1.078** at depth 1,
**1.217** at depth 2, **1.297** at depth 3, against a `total_budget` of **1.5**. The
*baseline* tuple — every slot at its own best alternative — already costs ≈ 1.0, because the
non-deep slots' best alternatives are not free. The marginal cost of one more deep
coordinate is only **+0.139** then **+0.080**, but it is charged against a headroom that
starts at ≈ 0.4. That is why depth 3 is refused 47 % of the time and why the reserve's
depth-3 spend is small in absolute terms (2 764 of 45 387 offers, 6 %).

**(d) Therefore the binding constraint on the canonical target is a coverage cap, and it is
measured, not inferred.** The reserve never offers a tuple with **four** non-zero
coordinates on either arm: **0 depth-4 offers out of 45 387, on both arms** — the ceiling is
`EMIT_PROFILE_MAX_DEEP = 3`, reached before the traversal and before `build` is called. The
requested canonical resegmentation is a **four-deep** shape (`hits` `dupe` `hid` `came` at
slots 0, 3, 4, 5, ranks 6–8 / 10–32 / 99 / 5–50). And it is *affordable*: `4e8a52` measured
its total substitution cost at **0.919520 ≤ 1.5**, and this front's accepted-cost bands
(branch) show **4 692 accepted reserve tuples already priced at ≤ 0.9**, i.e. ≥ 0.6 of
headroom against a marginal deep-coordinate cost of +0.08 to +0.14. So a four-deep tuple of
the requested kind would be *accepted* by `build` if the reserve ever offered one, and the
reserve structurally cannot.

Conclusion: **cost-accounting front — refuted by this measurement.** Changing how a deep tuple
is priced against `total_budget` cannot produce the requested wording, because the requested
wording is not refused by the bound when it is offered in the shapes measured here, and
because the shape that would carry it is never constructed. The next front is a **coverage**
front, and the specific quantity is `EMIT_PROFILE_MAX_DEEP = 3` (with the emission allowance
as the second constraint: 16 tuples per segmentation against a rank rectangle of
160 × 160 × 160 × 160 × 93 for the canonical target, i.e. the reserve samples ~0.001 % of
the four-deep rectangle even if the cap is lifted).

**A refutation of "sweep was the only gate", stated plainly.** The sweep fix is *not* the
whole story either, in the direction the brief anticipated: the per-member rates bought
expressiveness at essentially zero cost in refusals (+0.58 % accepted at depth ≥ 2), so this
is **not** a case of pool breadth traded for refused shapes. But it is also not a fix for
the item's target: with the sweep fixed, the requested pairing is *expressible in principle*
only at depth ≤ 3, and the requested shape is depth 4. So the coordinator should record the
sweep as **necessary and not sufficient**, with the sweep's own yield now measured at
+0.35 % overall, and the next front as the depth cap.

**Blocker, reported as a blocker.** Nothing in this front is a blocker for the numbers
above. One thing *is* a blocker for the next front and is reported rather than fixed, per
the addendum: `EMIT_PROFILE_MAX_DEEP = 3` and `EMIT_PROFILE_RESERVE = 16` are the only two
quantities standing between the current reserve and the requested shape, and lifting the
first is not free — the depth-3 refusal rate is already 47 % across targets and 35.4 % on
the canonical target, so the *offer* rate at depth 4 would have to be paid for in refusals
unless the cost accounting changes at the same time. That tension is the next front's
problem to price, not this one's to solve.

## 7. Is the reserve-shape arm's 10.4 % → 25.1 % improved, unchanged or worsened by 79309a1?

**Unchanged.** Canonical target, all depths: **10.40 % (parent) → 10.28 % (branch)**;
per depth 2.34/10.35/35.97 % → 2.34/10.24/35.42 %. Across all twelve targets:
**19.53 % → 19.24 %**. `79309a1` neither improves nor worsens that figure — it moves it by
−0.12 pp on the canonical target and −0.29 pp across the twelve, which is inside the noise
of which tuples the reserve happens to draw. And it does not rescue it either: the per-depth
profile on this arm is still 7.8 % / 23.4 % / 47.3 %, so a *fourth* deep slot would be
appended to a class ladder whose third rung is already refused three times in five. If the
reserve-shape arm's fourth slot is what produced 25.1 % overall, `79309a1` leaves that
number where it was; the two arms are not comparable on that figure without re-running the
reserve-shape arm with this probe on it, which this front did not do and should not be read
as having done.

## 8. Runtime

`--approximate --top 50` on `It's just a stupid game`, the search milliseconds the binary
reports for itself, six runs per arm **interleaved in one session with the arm order
alternated** (branch, parent, parent, branch, …):

```
branch  1603 1578 1527 1991 1811 1366   median 1590   min 1366   max 1991
parent  1598 1597 1338 1405 1463 1430   median 1448   min 1338   max 1598
```

The ranges overlap heavily (parent's max exceeds branch's min by 12 %) and the medians differ
by 142 ms with a within-arm spread of 625 ms on the branch and 260 ms on the parent. The
probe is gated off for these runs, and the branch's extra work is a pure-function arithmetic
change inside the same loop. **This is host noise and no runtime claim is made from it.** The
1.77–2.51 s column in the item's notes is not quoted as a comparison, as instructed.

## 9. Suites

Both arms, release:

| suite | branch `79309a1` + probe | parent `aa662a4` + probe |
|---|---|---|
| `--lib` | 53 passed, 0 failed | 50 passed, 0 failed |
| `--test corpus_integration` | 10 passed, **1 failed**: `approximate_finds_classic_madgab_resegmentation` | 10 passed, **1 failed**: same test |
| `--test exact_determinism` | 1 passed | 1 passed |
| `--test approx_determinism` | 2 passed | 2 passed |
| `--test no_phrase_hard_coding` | 6 passed | 6 passed |

`approx_determinism` green on both arms, and the fence green on both arms. The one failure
is the same test on both arms and is the item's target, not a guard: it is red on the
parent too, exactly as `4e8a52` reported. No test was edited, re-baselined, skipped or
relaxed; `approximate_output_is_locked` is untouched and green (it is in the 10 passing).

`cargo fmt`, `cargo clippy` and doctests **do not exist on this host** and are not claimed
(`docs/environment-notes.md`).

## 10. Artifacts on this branch

```
REPORT-e1a3f7.md                             this report
src/lib.rs                                  the gated probe (measurement-only)
examples/costpool.rs                        default-path pool membership harness
measurements/e1a3f7/probe-parent.tsv        45 387 raw offered tuples, parent arm
measurements/e1a3f7/probe-branch.tsv        45 387 raw offered tuples, branch arm
measurements/e1a3f7/depth-{parent,branch}.txt
measurements/e1a3f7/shape-class-{parent,branch}.txt
measurements/e1a3f7/pool-{parent,branch}.txt
```

The probe TSV columns: `target, coverage_phase, slot_count, depth, narrowest_width,
total_cost, accepted, deep_ranks (slot:rank), spread, deep_words`.

## 11. The one most valuable next measurement

**Re-run this exact probe with `EMIT_PROFILE_MAX_DEEP = 4` (and nothing else changed) on
the default release path, and read the depth-4 row of the same table.** Everything in this
report points at that one cell: the requested shape is depth 4, depth 4 is currently
offered zero times, the requested shape's cost (0.919520) is inside a budget of 1.5, and the
depth-3 refusal rate (47 % across targets) says the depth-4 refusal rate is the open
question. A single number — the depth-4 offered/built/refused triple, and the pool delta on
the canonical target — decides whether the next front is "lift the cap and pay for it" or
"lift the cap *and* change how a deep tuple is priced", and it is measurable with the probe
that is now on this branch. Everything else in the reserve's cost story is second-order next
to it.
