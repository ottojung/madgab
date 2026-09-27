---
work_item: true
measurement_for: w-9e2b41
id: w-9e2b41-measurement
state: done
owner: agent-0f3a173 (recovered front; read-only, no src/lib.rs edit in the integration worktree)
updated: 2026-09-27T09:58:00Z
branch: madgab-rederive-0f3a17
worktree: /workspace/madgab-rederive-0f3a17
base: de0fb30
probe_worktree: /workspace/madgab-rdp-0f3a17
probe_branch: scratch/rederive-0f3a17-probe
probe_target_dir: /workspace/target-rdp-0f3a17
---

# w-9e2b41 measurement: the joint affordability arithmetic, the 237-of-240 conflict settled, and the per-slot traversal ranks

Recovery of the front `agent-0f3a173`, which died exit 1 mid-measurement with no
commits. This file is the whole of its output. **No `src/lib.rs` change is proposed
here and none is made in `/workspace/madgab-rederive-0f3a17`**; the production width
rule is `agent-9e2b410`'s. The result is a **priced negative with arithmetic**: the
per-slot width mechanism as specified cannot reach the canonical clue, and the
reason is the product, not the constant.

`cargo fmt`, `cargo clippy` and doctests cannot run on this host and are not
claimed.

---

## 0. Where the numbers come from

* Integration worktree `/workspace/madgab-rederive-0f3a17` on
  `madgab-rederive-0f3a17`, base `de0fb30`. **Docs-only commit.**
* Probe worktree `/workspace/madgab-rdp-0f3a17` on
  `scratch/rederive-0f3a17-probe`, same base, its own
  **`CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17`** (never shared with the
  integration worktree or with any other agent's worktree). It carries
  `#[cfg(test)]` instrumentation in `src/lib.rs` and **is not pushed**; no probe
  binary was added to `examples/`.
* Probe inputs are read from outside the repository, so no target, clue or word
  string exists anywhere in the crate: `/workspace/rdp-0f3a17-targets.txt`
  (15 real targets), `/workspace/rdp-0f3a17-cases.tsv` (8 real
  (target, wording) pairs), `/workspace/rdp-0f3a17-ref.txt` (the two
  reference cases).

Exact commands, all from `/workspace/madgab-rdp-0f3a17`, release, defaults,
`--top 50`, public `Generator::generate_pool`:

```
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_pristine_reference   -- --nocapture
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_joint_affordability_table -- --nocapture
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_joint_affordability_real_targets -- --nocapture
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_canonical_slot_indices -- --nocapture
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_why_237_of_240 -- --nocapture
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_price_of_the_tuple -- --nocapture
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --test corpus_integration -- --test-threads=2
CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --test no_phrase_hard_coding
```

### Pristine reference reproduced first

| target | pool | rank-50 cutoff |
|---|---|---|
| `It's just a stupid game` (case 2) | **18,936** | **0.915121574454** |
| `recognize speech` (case 1) | **18,270** | **0.918796440893** |

Both exact, digit for digit. The per-target pop totals also agree with the
independent accounting already on record for three of the fifteen
(`w-b3e91a`): **175,957 / 195,370 / 446,341**. The probe base is clean.

### Fence status on the probe worktree (same base, so this is also the fence status of `de0fb30`)

| command | result |
|---|---|
| `cargo test --release --lib` | **66 passed, 0 failed** (62 pristine + 4 probe tests) |
| `cargo test --release --test corpus_integration` | **12 passed, 1 failed** — the one failure is `approximate_finds_classic_madgab_resegmentation`, pre-existing red at this base |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed, 0 failed** |

The three `approximate_pool_reaches_*` guards and `approximate_output_is_locked`
are **green** (verified individually). The instrumentation is inert: it observes
and never changes a value.

---

## 1. What the loop actually charges

Read from `src/lib.rs` at `de0fb30`.

* **The per-segmentation pop limit is `LEXICAL_HEAP_POP_LIMIT` = 4,000**, a
  `const` local to `generate_approximate` (line 982). It is tested in exactly
  two places, `popped >= LEXICAL_HEAP_POP_LIMIT` (lines 1965 and 2037).
* **`popped` counts every heap pop, leaves included.** It is incremented at the
  top of the walk loop, *before* the `k == depth` leaf branch (lines 1922-1925).
  So the charge is the **node count of the explored lattice**, not its internal
  nodes and not its leaf count.
* **The second charge is the global pop budget**, `LEXICAL_GLOBAL_POP_BUDGET` =
  `SEGMENTATION_KEEP * LEXICAL_HEAP_POP_LIMIT` = 256 x 4,000 = **1,024,000**,
  tested as `spent_pops + popped >= LEXICAL_GLOBAL_POP_BUDGET` and **only at the
  widening decision** (line 2038), not inside the walk. It is therefore a
  per-*target* constraint on widening, never a per-segmentation one.
* **The third and fourth charges are emissions**: `emitted >= emit_allowance`
  where `emit_allowance` is clamped to `LEXICAL_COMBINATIONS_PER_SEGMENTATION`
  = **64**, and `spent_emissions >= LEXICAL_GLOBAL_EMISSION_BUDGET` =
  `SEGMENTATION_KEEP * 64` = **16,384**, both tested per leaf (lines 1951-1954).
* **There is no `first_leaf_frontier` and no `pops_left` on this head.** The only
  frontier object is `expanded`, the replay list written by the widening pass
  (lines 1916-2051). Any derivation that names either of those is naming
  something this tree does not have.

**So what is actually being charged is the product of prefix widths**, in the
form of a node count. For a per-slot width vector `w = (w_0, ..., w_{d-1})`:

```
A(w) = 1 + w_0 + w_0*w_1 + ... + w_0*...*w_{d-2}      internal nodes; the walk's
                                                       charge before its FIRST leaf
T(w) = A(w) + w_0*...*w_{d-1}                         every node, leaves included;
                                                       `popped` converges here on a drain
```

`A` is exactly `affordable_opening_width`'s `fits` closure (lines 297-305)
generalised from a uniform `w` to a vector. `affordable_opening_width` returns
the largest **uniform** `w` with `A([w; d]) <= 4,000`, capped at
`LEXICAL_BRANCH_STAGE_0` = 10.

**The correct joint constraint is therefore `A(w) <= 4,000` to emit anything and
`T(w) <= 4,000` to enumerate the lattice.** Neither is a per-slot quantity, and
the pop limit is not the only one: 64 emissions per segmentation and 16,384
globally are charged alongside it, and on real targets they bind first (Section 5).

### `cap` is a scalar today, not a vector

`cap` is computed once per segmentation as
`affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT).min(widest)` (line 1902)
and used as `for i in 0..slots[k].len().min(cap)` (line 1977) — **the same
number for every slot**. There is no per-slot width in the tree to be corrected.
What the marginal table below describes is a table of what a *vector-valued*
`cap` could be; that is a structural change, and it has to be priced as one.

### A legal per-slot allocation

The fix to the marginal method is one word: **commit**. Solve slot 0 against the
baseline, keep the answer, solve slot 1 against the *committed* prefix, and so
on. The marginal method re-solves each slot in isolation and discards the other
slots' answers, so its row is a set of separate optima, not one feasible point.

```
joint_greedy(d, L, list):  b = affordable(d, L);  w = [b; d]
                           for k in 0..d:  w[k] = max{ v : A(w with w[k]=v) <= L }
```

`A(joint_greedy) <= 4,000` holds at every depth, by construction, and is printed
in the next section as the check.

---

## 2. The lost finding, re-derived and extended

`CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_joint_affordability_table`

The per-segmentation pop limit is **4,000** (read from inside the search, not
hard-coded in the probe). Lists are `SPAN_SHORTLIST` = 160 wide.

```
d  baseline  marginal_row            A(marginal) / T(marginal)   joint_greedy         A(joint) / T(joint)   A(marginal)/L   A(joint) ok
2  b=10      [160, 160]              161 / 25,921                [160, 160]           161 / 25,921           0.04x          true
3  b=10      [160, 160, 160]         25,761 / 4,147,521          [160, 23, 160]       3,841 / 596,481        6.44x          true
4  b=10      [36, 36, 38, 160]       50,581 / 7,980,841          [36, 10, 10, 160]     3,997 / 583,993       12.65x          true
5  b=7       [9, 10, 10, 10, 160]    10,000 / 1,459,999          [9, 7, 7, 7, 160]     3,601 / 501,121        2.50x          true
6  b=5       [5, 5, 5, 5, 5, 160]    3,906 / 507,811             [5,5,5,5,5,160]       3,906 / 507,811        0.98x          true
7  b=3       [10,11,11,11,11,14,160] 2,210,791 / 332,379,981     [10,3,3,3,3,3,160]    3,641 / 396,081      552.70x          true
8  b=3       [3,3,3,3,3,3,3,160]     3,280 / 356,479             [3,3,3,3,3,3,3,160]    3,280 / 356,479        0.82x          true
```

**The lost finding is reproduced exactly.** The depth-5 row `9 / 10 / 10 / 10 /
160` costs `1 + 9 + 90 + 900 + 9000 = 10,000` internal nodes against the
4,000-pop limit, a **2.50x overrun**. The depth-7 row `10 / 11 / 11 / 11 / 11 /
14 / 160` costs `1 + 10 + 110 + 1,210 + 13,310 + 146,410 + 2,049,740 =
2,210,791`. Both numbers are the ones the front died holding.

**Extension 1: the table overruns at four of seven depths, and the two depths
where it does not are the two where the marginal happens to equal the baseline.**
At `d = 3` (6.44x), `d = 4` (12.65x), `d = 5` (2.50x) and `d = 7` (**552.70x**)
the row is infeasible as a simultaneous allocation. `d = 2`, `d = 6` and `d = 8`
are feasible only because every marginal in those rows happens to equal the
baseline or the free last slot. So the method is not "sometimes too generous":
it is infeasible precisely where it is doing something.

**Extension 2 — the load-bearing one: "the last slot's depth is charged against
nothing" is an artefact of counting only the first leaf.** `A` does not contain
`w_{d-1}` at all, so a first-leaf budget hands the last slot its entire 160-wide
list for nothing. Under the quantity `popped` actually accumulates — `T` — the
*jointly legal* greedy row still costs **89x to 149x the pop limit at every
depth from 3 on** (149.1x at `d = 3`, 146.0x at `d = 4`, 125.3x at `d = 5`,
127.0x at `d = 6`, 99.0x at `d = 7`, 89.1x at `d = 8`), and the marginal row
costs 356,479 to 332,379,981. So:

> A per-slot affordability test derived from `A` bounds only the walk's *first*
> leaf. It says nothing whatever about whether the slot can be *enumerated*, and
> the traversal's pop charge is a node count, not a first-leaf count. Any width
> rule justified by `A` alone is justified for a property the loop does not care
> about.

This is the correction to the width front's own framing, and it is the same
category error one level up as the one `REPORT-4b1e07` charged against
`LEXICAL_HEAP_POP_LIMIT` being a budget figure used as an enumeration cap: the
derivation is a **first-leaf** figure used as an **enumeration** width.

### The same arithmetic over 15 real targets

`CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_joint_affordability_real_targets`

Every one of the fifteen reports a **widest shortlist of 160** and a
**160-wide list at every depth where the run is present**, so the table above is
the table for all fifteen; there is no per-target variation to hide behind. What
does vary is what the traversal actually spends:

| target | pool | segmentations | pops | % of 1,024,000 | emitted |
|---|---|---|---|---|---|
| It's just a stupid game | 18,936 | 240 | 175,957 | 17.2% | 12,465 |
| recognize speech | 18,270 | 241 | 195,370 | 19.1% | 12,355 |
| she sells sea shells | 15,781 | 241 | 131,333 | 12.8% | 12,292 |
| the cat sat on the mat | 19,869 | 240 | 279,999 | 27.3% | 11,998 |
| taco cat | 13,875 | 126 | 67,510 | 6.6% | 6,512 |
| sign on | 7,212 | 32 | 9,192 | 0.9% | 1,697 |
| big spender | 20,314 | 237 | 345,717 | 33.8% | 12,180 |
| I love you | 9,622 | 63 | 38,854 | 3.8% | 3,293 |
| a whole lot of trouble | 21,250 | 240 | 446,341 | 43.6% | 12,625 |
| Coors light | 12,951 | 124 | 68,150 | 6.7% | 6,483 |
| hit the ground running | 19,349 | 240 | 254,149 | 24.8% | 12,489 |
| he who laughs last | 21,236 | 241 | 215,317 | 21.0% | 12,440 |
| one man wolf | 20,126 | 240 | 176,819 | 17.3% | 12,751 |
| the early bird | 13,792 | 120 | 58,412 | 5.7% | 6,212 |
| let it be | 9,156 | 63 | 50,748 | 5.0% | 3,309 |

2,877 segmentations, 15 targets. The traversal spends **0.9% to 43.6%** of the
global pop budget and never approaches the per-segmentation 4,000: the carriers
in Section 4 spend **300 to 1,853 pops of 4,000** (7.5% to 46.3%). Meanwhile
every carrier emits **48 to 57 of its 64** (75% to 89%). **The binding charge is
the emission allowance, not the pop limit**, and this is measured per target, not
inferred.

---

## 3. The 237-of-240 conflict, settled

`CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_canonical_slot_indices`
and `... --lib rdp_why_237_of_240`

### The answer

**There is no conflict. The two fronts reported the same three rows.**

Canonical case 2, `It's just a stupid game` (240 chars of IPA), wording
`hits justice dupe hid came`. Across the target's **240 retained segmentations**,
exactly **3** carry the whole five-word wording as five *consecutive* slots, and
their per-slot **traversal** indices are:

| segmentation | depth | cap | traversal indices |
|---|---|---|---|
| spans `(0,3)(3,10)(10,13)(13,15)(15,18)(18,19)`, clue in slots 1-5 | 6 | 5 | **78 / 0 / 10 / 99 / 19** |
| spans at depth 5, clue in slots 0-4 | 5 | 7 | **7 / 0 / 13 / 99 / 11** |
| spans at depth 5, clue in slots 0-4 | 5 | 7 | **7 / 0 / 22 / 99 / 11** |

`agent-0f3a171` reported "present in 3, at `7/0/22/99/11`" — that is the third
row, and it named the right count. `agent-0f3a172` reported
`7/0/13/99/11` — that is the second row. **The two are the same measurement of
the same three segmentations; `22` and `13` are the *third coordinate on two
different depth-5 carriers of the same target*, not two answers to one cell.**
The record's `13/99/11` fragment is the tail of the second row and the
`7/0/22/99/11` is the third.

### The depth-1 miss, from my own run

At depth 5 the cap is **7** and the depth-1 word sits at traversal index **7**.
The walk's test is `for i in 0..slots[k].len().min(cap)`, i.e. `i < cap`, so
`7 < 7` is **false**. **Confirmed: canonical case 2 is 237-of-240 covered by the
lattice, and on the three that carry it the depth-1 coordinate misses by exactly
one traversal index.** Width 8 admits it; width 100 is the first width that
enumerates the whole wording (coordinate 3, index 99).

### Why 237 — and what that number is not

The 237 is a **boundary-structure** count, not a candidate-availability count.
Decomposing the 237 non-carriers of the canonical case:

| class | count | meaning |
|---|---|---|
| carry all five words on some slot, but not on five consecutive slots | **232** | the segmentation cuts the target elsewhere; every wording word is a real candidate |
| no wording word on any slot | **1** | nothing is available at all |
| carry the run | **3** | the lattice does contain the wording |

So for **236 of the 237**, `agent-0f3a171`'s inference — "for those 237 no width
and no emission allowance can reach it" — is **false as stated**: the words are
candidates, and what is missing is that the segmentation's boundary structure is
not the wording's. Only **1 of 240** fails for the reason the inference implies,
and it is not a carrier. Per-word coverage makes the same point from the other
side: over the 240 segmentations, `came` is on some slot of **176**,
`dupe` **149**, `hits` **143**, `justice` **160**, `hid` **91**. No wording word
is absent from the lattice of a typical segmentation.

**Verdict.** `agent-0f3a171`'s *count* stands and is confirmed. Its *conclusion* —
"so this is not a width defect at all", "the clue is outside the reserve's
lattice too, at any width" — is **refuted for 236 of its 237** and is not
load-bearing for the one case where it holds. The clue **is** in the lattice, in
exactly the one segmentation per depth whose boundaries are its own, and inside
it the depth-1 coordinate misses by one index.

The item's `3 / 2 / 10 / 99 / 11` matches none of the four per-slot orders and
matches none of my three rows; it should be struck, as `w-0f3a17`'s own 09:05Z
pass already concluded.

---

## 4. Per-slot traversal-index ranks per depth, and the binding slot

`CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_canonical_slot_indices`

Traversal-index units throughout, per depth, against the **current head** opening
width. Canonical case 2, the canonical target's 240 segmentations:

| depth | segmentations | head cap | widest list | carry the run | min width that enumerates the wording |
|---|---|---|---|---|---|
| 4 | 38 | 10 | 160 | **0 of 38** | — |
| 5 | 96 | **7** | 160 | 2 | **100** |
| 6 | 106 | **5** | 160 | 1 | **100** |
| 2, 3, 7, 8 | 0 | — | — | — | — |

**The binding slot is slot 3, not the depth-1 slot.** At depth 5 the coordinates
are `7 / 0 / 13 / 99 / 11` against cap 7: **four of the five are outside the cap**
(`7 < 7` false, `13`, `99`, `11`), and the binding one is the largest by an
order of magnitude — index **99**, min width **100**, ~14x the derived 7. The
depth-1 miss-by-one is real and is the *cheapest* of the four, which is why a
front that stopped at depth 1 would have under-priced the mechanism by 14x.
At depth 6 the same wording sits at `78 / 0 / 10 / 99 / 19` against cap 5: again
four of five outside, binding slot 3 at 99.

Two further real wordings on the same target, for scale (`it justice two end
game`, depth 5: twelve carriers, cheapest `[0,0,1,5,20]`, min width **21**;
`ich just a stoop a gave`, depth 6: thirteen carriers, min width **160**), and
case 1 (`wreck a nice beach`, depth 4, cap 10: fourteen carriers, cheapest
`[1,0,0,2]`, min width **36**; at depth 4 four of the fourteen are already
enumerated at cap 10, which is why case 1 needs nothing).

### The price of the tuple itself — the priced negative

`CARGO_TARGET_DIR=/workspace/target-rdp-0f3a17 cargo test --release --lib rdp_price_of_the_tuple`

The cheapest per-slot width vector that can *contain* a given wording is exactly
`(i_k + 1)` — you cannot be narrower than an index you must reach. So the joint
cost of a specific tuple is not a free parameter; it is `A(i+1)`. Priced against
4,000:

| target / wording | depth | indices | min widths `i+1` | `A` | x limit | `T` | x limit | affordable |
|---|---|---|---|---|---|---|---|---|
| canonical case 2 | 6 | 78/0/10/99/19 | 79/1/11/100/20 | 87,928 | 21.98x | 1,913,855 | 478.5x | **no** |
| **canonical case 2** | **5** | **7/0/13/99/11** | **8/1/14/100/12** | **11,329** | **2.83x** | 157,057 | 39.3x | **no** |
| canonical case 2 | 5 | 7/0/22/99/11 | 8/1/23/100/12 | 18,601 | 4.65x | 258,001 | 64.5x | **no** |

Over **95** carrier segmentations measured across the eight real
(target, wording) pairs: **74 affordable, 21 not**. The canonical case 2 is
**uniquely unaffordable** — its cheapest carrier already costs 2.83x, and all
three of its carriers are on the wrong side of the limit.

Why: `A(w) = 1 + w_0 + w_0 w_1 + ... + w_0...w_{d-2}`. The last slot's width
does not appear in it at all, and every earlier slot's is multiplied by the
prefix above it. For the canonical depth-5 carrier the prefix products are
**8, 8, 112, 11,200** — the binding coordinate is slot 3, whose minimum width
**100** is already paid for by the 8 x 1 x 14 = 112 the three slots above it
force. Measured spending on that very segmentation: **391 pops of 4,000 (9.8%)**
and **50 emissions of 64 (78%)**. It stops on emissions, with 3,609 pops unspent.

### Conclusion for the width front

1. **The per-slot marginal table is inadmissible as a simultaneous allocation.**
   Independently re-derived: 2.50x the pop limit at depth 5, 552.70x at depth 7,
   and infeasible at depths 3 and 4 as well. Any implementation that widens every
   slot to its marginal overruns badly.
2. **The joint constraint is `A(w) <= 4,000` to emit and `T(w) <= 4,000` to
   enumerate, and it is not the binding one.** Per-target measurement: 0.9-43.6%
   of the pop budget, 75-89% of the emission allowance. A width rule derived from
   the pop limit is derived from the slack.
3. **Even a perfectly legal per-slot width rule cannot reach the canonical clue
   at depth 5.** Its own minimum sub-lattice costs `A = 11,329`, 2.83x the pop
   limit, before any competing tuple is considered. `A` also ignores that the
   walk is best-first: the canonical tuple's bound is below 50 others the walk
   emits first. So the shortfall is not 14x on a constant — it is a structural
   product (`8 x 1 x 14 x 100` over four slots) against a linear budget.
4. **What the width mechanism can legitimately be given credit for**: the
   depth-1 miss-by-one is real (`7 < 7` false) and a per-slot vector is the only
   way to reach index 7 at depth 1 while leaving the deep slots narrow. It is
   worth one index out of the 100 the wording needs. That is a correct
   improvement and not a completion.

**Recommendation: the width front should land a legal, jointly-affordable
per-slot vector as an improvement in its own right, and record the canonical
depth-5 shortfall as a priced negative with this arithmetic.** Criterion 2 of
`w-9e2b41` cannot be met by the width rule at depth 5; the remaining mechanism
is a *placement* mechanism that spends emissions and pops on a named deep tuple
rather than on widening a product — which is what `w-c3f81a` (the coverage
reserve's tuple placement) and `w-7b40d2` (which shortlist candidates are
retained) are already open for, and which this measurement now prices.

---

## 5. Fences observed

* **Read-only on `src/lib.rs` in `/workspace/madgab-rederive-0f3a17`.** The only
  commit on `madgab-rederive-0f3a17` is this file. All `src/` instrumentation
  lives in the scratch worktree `/workspace/madgab-rdp-0f3a17` on
  `scratch/rederive-0f3a17-probe`, which is not pushed.
* **No probe binary in `examples/`.** None was added anywhere.
* **No phrase-specific case, no hard-coded target or clue string in any code
  proposed.** Every phrase lives in a TSV outside the repository; the probe holds
  only the paths. `no_phrase_hard_coding` is 9/9.
* **No new env knob.** `MADGAB_TRACE_PHRASES` is not introduced and does not
  exist on this head; no `env::var` was added to `src/`.
* **No re-baselining.** `approximate_output_is_locked` and all three
  `approximate_pool_reaches_*` guards are green and untouched; `tests/` is
  unmodified.
* **Own `CARGO_TARGET_DIR`,** named in every number above and not shared with any
  other worktree.
* **No edit to another agent's worktree,** and no attempt at the production width
  change, which is `agent-9e2b410`'s.
* `cargo fmt`, `cargo clippy` and doctests cannot run on this host and are **not**
  claimed.

## 6. Numbers that must not be contradicted downstream

| claim | value | where measured |
|---|---|---|
| per-segmentation pop limit | 4,000 | `src/lib.rs:982`, read from inside the search |
| global pop budget | 1,024,000 | `SEGMENTATION_KEEP * LEXICAL_HEAP_POP_LIMIT` |
| emission allowance / global emissions | 64 / 16,384 | `src/lib.rs:125, 997` |
| `affordable_opening_width(d, 4000)`, d=2..8 | 10/10/10/7/5/3/3 | probe, Section 2 |
| marginal row d=5 `A` / `T` | 10,000 / 1,459,999 (**2.50x**) | probe, Section 2 |
| marginal row d=7 `A` / `T` | 2,210,791 / 332,379,981 (**552.70x**) | probe, Section 2 |
| joint-greedy `T` / pop limit, d=3..8 | 149.1x / 146.0x / 125.3x / 127.0x / 99.0x / 89.1x | probe, Section 2 |
| canonical case 2, carriers of the run | **3 of 240** | probe, Section 3 |
| canonical case 2, traversal indices | 7/0/13/99/11, 7/0/22/99/11, 78/0/10/99/19 | probe, Section 3 |
| canonical case 2, 237 split | 232 boundary mismatch, 1 no candidate at all | probe, Section 3 |
| canonical depth-5 min widths and `A` | 8/1/14/100/12, `A` = 11,329 = **2.83x** | probe, Section 4 |
| binding slot | **slot 3**, index 99, min width **100**, prefix product 11,200 | probe, Section 4 |
| carriers spending | 300-1,853 pops of 4,000; 48-57 emissions of 64 | probe, Sections 2, 4 |
| carriers affordable under `A` | 74 of 95; **all 3 canonical carriers are not** | probe, Section 4 |
