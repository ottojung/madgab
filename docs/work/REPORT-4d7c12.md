# w-4d7c12 - the canonical case-2 multiset dies at the joint enumeration boundary, and every coverage rule is a priced negative

Front `agent-4d7c121`, branch `madgab-tupreach-4d7c12`, base `a1d48c0`
(`post-milestone-acceptance`). Release mode throughout,
`CARGO_TARGET_DIR=/workspace/target-4d7c12`.

**Verdict: HOLD.** The stage is localised and three general coverage rules are
priced. No production change is proposed for integration: every rule is a
priced negative, and none of them earns a production line.

Reproduce every number below with:

```
cargo test --release --lib front_4d7c12 -- --ignored --nocapture --test-threads=1
```

---

## 0. What was added, and where it lives

Two `#[cfg(test)]` items in `src/lib.rs`, and nothing else:

* `mod stage_probe` — a per-thread recorder. It is told a *word multiset* by
  the test that arms it; it reads no sentence, no clue and no target, and the
  words it watches are named only in the test module. Every hook site in
  `generate_approximate` and `coverage_tuples` is a `#[cfg(test)]` statement
  that is a no-op when the probe is unarmed.
* `mod front_4d7c12` — the measurement driver: five `#[ignore]`d tests.

Both are excluded from the phrase fence by the existing
`#[cfg(test)] mod` rule in `tests/no_phrase_hard_coding.rs`, and the fence is
**9/9**. This instrumentation is measurement apparatus and is **not** proposed
for integration as production source.

Four of the rule constants read as ordinary production functions so the
release build still compiles:

```rust
#[cfg(test)]     fn coverage_rule_full_vector() -> bool { stage_probe::armed(...) }
#[cfg(not(test))] fn coverage_rule_full_vector() -> bool { false }
```

A shipped build therefore reads `false` for every rule, and the
`coverage_tuples` rewrite is behaviour-identical to the base — which the
fence in §6 confirms rather than asserts.

---

## 1. The per-stage counts for the case-2 target

`Generator::generate_pool("It's just a stupid game")`,
`SearchMode::approximate()` (per-word 0.5, total 1.5), `top_n = 50`, 19 phones.

| # | stage | count |
|---|---|---|
| 1 | fuzzy per-span candidate generation | **108 spans, 14,180 candidates offered** |
| 2 | per-span shortlist (`SPAN_SHORTLIST = 160`) | **9,877 candidates retained** |
| 3 | structural segmentation DP | **1,854 complete paths, 256 after `SEGMENTATION_KEEP`** |
| 4 | slot/heap retention | **240 segmentations funded**, opening widths {5, 7, 10} |
| 5 | emissions | **2,929 reserve draws + 12,431 traversal + 1,024 adjacency = 16,384** |
| 6 | `Partial` assembly | **16,384 wordings** |
| 7 | dedup / pool | **18,949 clues** |

Wall clock **1.72 s**; pool **18,949**; per-word pool membership
`hits 19, justice 10,044, dupe 30, hid 2, came 138`. All three reproduce
[../REPORT-6b2e19.md](../REPORT-6b2e19.md) §1 exactly, so the base is
confirmed and every number below is a base number.

### The multiset, stage by stage

**After stage 2 it is fully representable.** All five words are retained on
every span they need, at these *shortlist* ranks:

| word | spans carrying it, with shortlist rank |
|---|---|
| `hits` | `[0,3)`#22 `[0,4)`#9 `[1,3)`#35 |
| `justice` | `[3,8)`#1 `[3,9)`#0 `[3,10)`#0 `[3,11)`#0 `[3,12)`#0 … 13 spans |
| `dupe` | `[9,13)`#54 `[10,13)`#64 `[10,14)`#21 `[11,13)`#80 `[11,14)`#45 |
| `hid` | `[0,2)`#111 `[2,4)`#84 `[13,15)`#85 |
| `came` | `[14,19)`#5 `[15,19)`#7 `[16,18)`#20 `[16,19)`#8 `[17,19)`#12 |

Six complete span chains over the shortlists can carry the multiset. **Two
survive the structural DP's retention and are funded**:

| schedule rank | span chain | slot widths | opening width | needed slot indices |
|---|---|---|---|---|
| **151** | `[0,3) [3,10) [10,13) [13,15) [15,19)` | `[160, 7, 160, 160, 93]` | **7** | **7 / 0 / 13 / 99 / 11** |
| **187** | `[0,3) [3,11) [11,13) [13,15) [15,19)` | `[160, 2, 160, 160, 93]` | **7** | **7 / 0 / 22 / 99 / 11** |

**The first stage at which the multiset is unrepresentable is stage 4, the
slot/heap retention.** The traversal's opening width is
`affordable_opening_width(5, LEXICAL_HEAP_POP_LIMIT) = affordable_opening_width(5, 4000) = 7`,
and the walk tests `index < cap`. The canonical needs indices
`7 / 0 / 13 / 99 / 11` in traversal-index units: four of the five are outside
the 7-wide opening, and the deepest one is at **99**, i.e. **14.1x** the
opening width. Nothing upstream of stage 4 loses it, and nothing downstream
recovers it: 0 of 2,929 reserve draws, 0 of 12,431 traversal emissions and 0
of 1,024 adjacency admissions carry the multiset, so it is 0 of 18,949 pool
members.

### The control, and why it is the control

The same measurement on the green case, `recognize speech` → `wreck a nice
beach`: 78 spans / 10,751 offered, 7,360 shortlisted, 369 DP paths / 256 kept,
241 funded, 3,051 reserve + 12,309 traversal + 1,024 adjacency = 16,384,
**18,289 pool members**, wall clock 1.67 s.

32 chains could carry it; the two that are emitted are
`Traversal sched 0 tuple [1,0,0,2]` and `Traversal sched 3 tuple [1,0,2,2]`,
and for both funds the report prints
`every index inside the opening width: **true**` — needed indices
`1/0/0/2` and `1/0/2/2` against a cap of 10.

**So the green case is emitted by the traversal inside its opening width, and
the red case is not emitted inside its opening width by anything.** That is
the whole difference, and it is a property of the joint configuration, not of
any word: `hid` is reachable alone (2 pool wordings) and `justice` is
reachable alone (10,044), and the pair of them at those ranks is not.

---

## 2. Why the reserve does not recover it — the second, independent half

The coverage reserve is the only mechanism in the search that reads a slot
index past the traversal's opening width, so if any stage could recover the
multiset it is this one. Measured, at the two funds that could carry it:

```
sched 151 widths [160,7,160,160,93] -> 14 draws:
  [0:11] [2:21] [3:31] [4:13] [0:51 2:58] [0:61 3:68] [0:31 4:17] [2:81 3:88]
  [2:43 4:29] [3:49 4:35] [0:117 2:10 3:39] [0:61 2:47 4:33] [0:67 3:53 4:39]
  [2:73 3:59 4:45]
sched 187 widths [160,2,160,160,93] -> 15 draws:
  ... [0:32 2:54 3:76 4:15]
```

Three measured facts about those 29 draws:

1. **Slot 1 is absent from every one of them.** `coverage_tuples` computes
   each subset's sweep span as the width of the subset's **narrowest** slot
   (`sweep_index(narrowest, …)`), and `sweep_index` returns `None` when
   `width <= LEXICAL_BRANCH_STAGE_0 = 10`. Fund 151 has a 7-wide slot 1 and
   fund 187 a 2-wide one, so **every subset containing slot 1 emits nothing at
   all**, and the reserve's coverage of those two funds is a cover of the
   product *with slot 1 deleted*. The canonical needs slot 1 at index 0, which
   is the one coordinate the sweep refuses to name.
2. **The largest funded subset is three deep slots** (14 and 15 draws; the
   16-draw reserve exhausts inside the size-3 level of a
   `C(5,1)+C(5,2)+C(5,3)` walk). The canonical needs **four** non-zero
   coordinates. This is the "essentially zero JOINT coverage" that
   [w-c3f81a](items/w-c3f81a.md) recorded, now with the count.
3. **No draw lands on a needed index.** Slot 0 is drawn at 11/31/51/61/67/117
   and never 7; slot 2 at 21/58/10/81/43/10/47/73 and never 13; slot 3 at
   31/68/88/49/39/53/59 and never 99; slot 4 at 13/17/29/35/33/39/45 and never
   11.

So the loss is **not** at the opening slot, **not** at a later slot, **not** at
the DP and **not** at dedup. It is at stage 4, and it stays lost at stage 5
because the only mechanism that could cross stage 4 is *marginal by
construction*.

---

## 3. Reconciliation with the closed fronts

Item obligation 2 asks for this explicitly. Three corrections, one
confirmation, one correction of this front's own earlier reading.

### 3.1 `hid` at rank 119 (front B, `w-5e2d41`) — **corrected to 99**

Front B measured, at `a8a8f70`, that the canonical's per-slot indices are
`7, 0, 13, 119, 11`. On this head they are **`7, 0, 13, 99, 11`** at schedule
rank 151 and `7, 0, 22, 99, 11` at rank 187. `affordable_opening_width(5,
4000) = 7` is **confirmed**, and 119 vs 99 is a 20% difference that changes no
conclusion: 99 and 119 are both far outside a 7-wide opening.

The cause is visible in front B's own table. Front B's rank 119 is on the
head `a8a8f70`; on the *base* head it recorded `hid` at rank **99** of slot 4
on span `[13,15)`. This head is base, not `a8a8f70`, so 99 is the base number
and 119 is the `a8a8f70` number. **The 119 does not belong on integrated
HEAD and is corrected to 99 here.**

### 3.2 `hid` is retained on `[0,2)` on this head — front B's premise is head-specific

Front B's §2.2(c) attributes base's 2 `hid` pool wordings to a reserve draw
`[136,0,0,0,0,0]` on the 6-slot structure `[(0,2),(2,3),(3,10),(10,13),(13,15),
(15,19)]`, with `hid` at rank 136 of 160 in the shortlist of `[0,2)`, and
reports that on `a8a8f70` `hid` is *not* retained on `[0,2)` at all.

Measured on this head: `hid` **is** retained on `[0,2)`, at shortlist rank
**111** (front B's 136 is a different ordering — `SlotAlt::contribution`, not
the shortlist's own order; both are consistent with the same membership). So
the shortlist-retention defect front B localises is **not present on
integrated HEAD**. Every one of the five canonical words is retained on every
span the six candidate chains need. This is the single most consequential
difference between my measurement and front B's, and it is why the successor
front must not be told to go and fix a shortlist rule.

### 3.3 The cheap-end floor is confirmed out of scope, by a different route

Front B priced the cheap-end retention floor a vacuous negative (`min_cost` of
`[0,2)` is 0.0, `hid` is at 0.35, so no finite multiple of the minimum reaches
it; the 1.5x band is satisfied on 108 of 108 spans). I did not re-run that
hypothesis. My measurement reaches the same exclusion from the other side:
there is no shortlist retention to fix, so no rule about *which* candidates a
span retains can be the answer on this head.

### 3.4 The budget and the depth cap are confirmed out of scope

`w-9e2b41` (per-slot enumeration cap) and `w-c3f81a` (depth cap) are not
re-litigated here. Both are consistent with what I measured and neither is the
first loss: the opening width is 7 at depth 5 and the reserve's depth bound
(`funded_slot_depth([160,7,160,160,93]) = 4`) is not what is short — the
reserve emits 14 draws, not 0, so it is not depth-starved.

### 3.5 One correction to this front's own first commit

`d7786cf`'s message says "0 of 15 at sched 187 touch slot 1". That is right,
but it understates the finding: at sched 187 the reserve *does* emit one
four-deep draw, `[0:32 2:54 3:76 4:15]`, and it is a full-vector draw **over
the wrong four slots** — `{0,2,3,4}`, not `{0,1,2,3,4}`. The four-deep family
is reachable; slot 1 is what is unreachable, and slot 1 is unreachable for the
narrowest-slot reason in §2, not for a budget reason. `cc024ca` and this
report carry the sharper form.

---

## 4. The three priced rules

All three are general, key only on measured per-slot quantities (slot widths,
the sweep floor, the subset shape), and none reads a word, a clue, a target or
a segmentation identity. Each is implemented as a one-line change inside
`coverage_tuples` and priced on this head, release, `top_n 50`.

| # | rule | shape |
|---|---|---|
| **A** | `RULE_PER_SLOT_SPAN` | each subset member draws its index from **its own** slot's span instead of the subset's **narrowest** — a per-span coverage rule |
| **B** | `RULE_FULL_VECTOR` | the **all-slots** subset is funded, so the reserve spends at least one draw on a **joint** configuration and not only on the product's marginal axes — a joint-coverage reserve over the cut vector |
| **C** | `RULE_JOINT_ONLY` × {2,4,8,16} | the whole per-segmentation reserve goes to joint draws, at a test-only multiple of the shipped slice — rule B *sized* |

### The pricing table

`reach` = pool members whose word multiset is exactly the canonical five.
`rank` = `wreck a nice beach`'s rank in `recognize speech`'s pool.
`reserve g/a` = reserve tuples **generated** / **admitted** past `build`'s
total-cost test.

| rule set | case-2 reach | pool 2 | wall 2 | reserve g/a | case-1 reach | case-1 rank | pool 1 | wall 1 |
|---|---|---|---|---|---|---|---|---|
| base | **0** | 18,949 | 1.04 s | 2,929 / 3,244 | 1 | **27** | 18,289 | 1.02 s |
| A | **0** | 18,934 | 1.24 s | 2,864 / 3,244 | 1 | **27** | 18,292 | 1.14 s |
| B | **0** | 18,936 | 1.24 s | 2,871 / 3,247 | 1 | **27** | 18,267 | 1.15 s |
| A+B | **0** | 18,923 | 1.25 s | 2,802 / 3,247 | 1 | **27** | 18,279 | 1.13 s |
| C ×2 | **0** | 18,330 | 1.24 s | **4 / 2,496** | 1 | **27** | 17,800 | 1.14 s |
| C ×4 | **0** | 18,334 | 1.26 s | **15 / 4,992** | 1 | **27** | 18,212 | 1.14 s |
| C ×8 | **0** | 18,334 | 1.25 s | **15 / 4,992** | 1 | **27** | 18,212 | 1.15 s |
| C ×16 | **0** | 18,334 | 1.25 s | **15 / 4,992** | 1 | **27** | 18,212 | 1.15 s |

**Every row is a priced negative on tuple reach.** None of the three reaches
the canonical; none re-pins or disturbs the green case, which stays at pool
rank 27 with its pool within 0.1% of base; and none costs wall clock anything
(the base figures this item sets are 9.5 s and 21.3 s, and every row is
1.02-1.26 s, i.e. **well inside both**). So the negatives are on reach, not on
cost — which makes them clean negatives, not marginal ones.

### The arithmetic that makes them negative, rather than merely unsuccessful

The joint reserve is **cost-limited, not coverage-limited**, and that is a
measurement, not an inference:

* at C ×4 the reserve **generates 4,992 joint tuples** and **4,977 of them are
  rejected** by `build`'s `total_cost > total_budget` test. **15 survive —
  0.30%**. The surviving ones are the cheap corner, because a tuple with a
  non-zero index in *every* slot pays five substitutions instead of one.
* scaling to ×8 and ×16 **changes nothing** (4,992 generated, 15 admitted at
  both), so the joint family is not short of draws: the binding limits are the
  per-segmentation `emit_allowance` and the 16,384 global emission ceiling,
  both of which are already saturated at 16,384/16,384 on every row.
* under the joint rules the reserve emits **0 draws at fund 151 and 0 at fund
  187**, and `dupe` and `hid` fall from 30 and 2 pool wordings to **0 and 0**.

So a joint reserve is not merely unable to reach the canonical here, it is
**strictly destructive**: the marginal draws it displaces are the only reason
`hid` is in the pool at all. That is a stronger negative than "it did not
work", and it is the reason rule C is reported as one.

The canonical's own budget is not the obstacle: its least-cost alignment costs
**1.1195** against a total budget of **1.5**, so it is comfortably affordable
and the *configuration* is what is unreachable, not the cost.

---

## 5. Why no rule in this family can work, stated so a successor does not re-bracket it

At the decisive fund the joint index space is

```
160 x 7 x 160 x 160 x 93 = 2,666,496,000 cells
```

and the canonical sits at `[7, 0, 13, 99, 11]`. A coverage rule that samples
that space uniformly needs on the order of `2.66e9` draws to be a cover; the
entire run has 16,384 emissions to spend, of which the reserve's share is
2,929. A *traversal* that reaches the cell by bound order needs the number of
better-bound cells in front of it, which is the multiplicative precedence
[w-b3e91a](items/w-b3e91a.md) and `w-5b1e93` already measured at 2,036,664
wordings for the same structure. Both routes are out of reach by three to five
orders of magnitude, and both were priced before this front.

What is left is not a coverage question at all. A cell inside the opening
width is reachable *today*, by the traversal, at zero extra cost — that is
exactly what the green case does (`[1,0,0,2]` and `[1,0,2,2]` at caps of 10).
The canonical would be reachable the same way if the traversal's slot ordering
put the right word of each of its five spans inside the first 7 positions.
That is an **ordering** property of `SlotAlt::contribution`, not a coverage
one, and it is a different front from this one. It is also the shape
[w-9e2b41](items/w-9e2b41) measured and recorded in its own goal statement —
"the per-slot floors across the six admission passes are `7/0/3/85/7`; no
re-specification of admission can beat them" — which is a statement about
*ordering*, arrived at from the width side.

---

## 6. Tree state and fence

`CARGO_TARGET_DIR=/workspace/target-4d7c12`, release, single runs unless
stated:

| target | result |
|---|---|
| `cargo test --release --lib` | **74 passed / 0 failed / 4 ignored** (24.8-25.6 s over three runs) |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed / 0 failed** |
| `cargo test --release --test corpus_integration -- --test-threads=2` | **12 passed / 1 failed** — `approximate_finds_classic_madgab_resegmentation`, the known pre-existing red at base, **not re-pinned and not re-run as a target** |
| `cargo test --release --test emit_coverage` | **4 passed / 0 failed** |
| `cargo test --release --test approx_determinism` | **4 passed / 0 failed** |
| `cargo test --release --test exact_determinism` | **1 passed / 0 failed** |

The 4 ignored are this front's own `#[ignore]`d measurement probes
(`localise_case2_stage_by_stage`, `localise_case1_stage_by_stage`,
`localise_case2_under_the_joint_rules`, `price_the_coverage_rules`). The 74
is the base count; the item's expectation of 74 passed / 0 failed holds, and
the 4 ignored are accounted for.

`git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"` over
`src/` returns 41 lines; the only ones this front added are
`src/lib.rs:7392-7397`, inside `mod front_4d7c12`, which is a `#[cfg(test)]`
module. No production source, no baseline, no `ZZ_*` probe, no environment
knob and no debug binary was added; `examples/` is untouched.

`cargo fmt --check` and `cargo clippy` **cannot run on this host** (see
[../environment-notes.md](../environment-notes.md)) and are **not claimed**.

### One defect this front introduced and fixed, recorded because it is a trap

The first version of the rule rewrite materialised the reserve's whole subset
list (`sum_{d=1..max_deep} C(depth, d)`) *before* the `out.len() >= reserve`
check. On a large synthetic depth that is astronomically large, and
`cargo test --release --lib` died with **SIGKILL** inside
`depth_profile_reserve_is_bounded_by_named_arithmetic` — the very test that
bounds the reserve. The final version generates each level lazily and checks
the reserve before materialising it, so the memory profile is the base's. This
is worth recording because the failure mode is an OOM kill, not a panic, and it
looks like a host fault rather than a code fault.

---

## 7. The coordinate a successor front should attack

Named precisely enough to start from rather than re-bracket:

* **Stage:** slot/heap retention — `src/lib.rs:2194`'s
  `let mut cap = affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT).min(widest);`
  — and, secondarily, the *ordering* of `slots[k]`, which is
  `SlotAlt::contribution(word_count)` at `src/lib.rs:1911`.
* **Measured numbers that decide it:** opening width **7** at depth 5;
  needed per-slot indices **`7 / 0 / 13 / 99 / 11`** at schedule rank **151**
  (widths `[160, 7, 160, 160, 93]`); the same word at rank 187 with
  `7 / 0 / 22 / 99 / 11` (widths `[160, 2, 160, 160, 93]`); joint space
  **2,663,040,000** cells against **16,384** emissions; the green case's
  emitted tuples `[1,0,0,2]` and `[1,0,2,2]` inside a cap of **10**.
* **What is already closed and must not be re-run:** the shortlist
  (`w-7b40d2`, `w-5e2d41` front B — and on this head there is nothing to fix,
  §3.2), the emission budget and pop budget (front B: 7,312 of 16,384
  unspent at the point of loss), the depth cap (`w-c3f81a`), the per-slot
  width allocation (`w-9e2b41`), emission order (`w-b3e91a`, `w-5b1e93`),
  and joint coverage of the reserve (this report, §4, three rules).
* **What is open:** whether the traversal's per-slot **order** can be made to
  place a span's non-cheapest-but-correct word inside the opening width
  without losing the descending-bound property that makes the emission order
  worth anything. That is a ranking question on `contribution`, it is
  orthogonal to scoring (no weight, no axis — `w-3a8c05`'s surface is
  untouched), and it is the only remaining mechanism that does not require
  more budget than the search already spends.
* **One caveat a successor should inherit:** the scorer still rates the
  canonical **0.0944** below the top-50 cutoff
  ([../REPORT-6b2e19.md](../REPORT-6b2e19.md) §1). Putting the tuple in the
  pool is necessary and **not sufficient** for it to be displayed. The two
  fronts are independent and both are on the path; this front's verdict does
  not touch the scoring side and does not duplicate its counterfactual table.

## 8. Verdict

**HOLD.** The stage is localised — **stage 4, the slot/heap retention, at
opening width 7 against needed indices 7/0/13/99/11** — the three general
coverage rules available at that stage are priced, and all three are negatives
with the arithmetic that makes them so. No production change is earned, so
nothing is proposed for integration, and `main` and
`post-milestone-acceptance` are untouched. The branch carries the
measurement and the pricing, both re-runnable from the tree.
