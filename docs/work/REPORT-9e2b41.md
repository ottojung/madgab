# REPORT-9e2b41 — the per-slot width front: a priced negative

Work item **w-9e2b41**, branch `madgab-width-9e2b41`, based on
`post-milestone-acceptance` at `534a39c`. Nothing is integrated by this front.

**Verdict: the width mechanism is closed, and the deliverable is a priced
negative with arithmetic.** A general per-slot, depth-aware allocation rule was
derived, implemented, wired into the traversal, measured, and **reverted**: it is
admissible against the charged quantity and it still costs the fence. The branch
lands the derivation, the arithmetic, and the tests, and changes **zero
production lines**.

---

## 0. Headline numbers

| quantity | value |
|---|---|
| depth-1 word's traversal index vs cap | **7 vs 7**, test `index < cap` → miss by exactly one |
| same tuple's five traversal indices | **7 / 0 / 13 / 99 / 11** (not `7 / 0 / 3 / 85 / 7`) |
| minimal allocation admitting it | `c = (8, 1, 14, 100, 12)` |
| its frontier, `F(c) = 1 + Σ_k Π_{j<k} c_j` | **11 329** |
| per-segmentation pop limit | **4 000** |
| **shortfall** | **7 329 pops, a factor of 2.83** |
| jointly affordable allocations at depth 4, limit 4 000, pools `[*,*,*,160]` | **58 106** |

The first row confirms agent-0f3a172 and refutes its scope; the second refutes
the work item's own summary of it; the last row is the finding.

## 1. Step 1 — reproduction and localisation (before any change)

Built release at `534a39c` and ran the approximate search on the canonical
target. The proposal set does **not** contain the canonical clue; it returns 50
`it said thus …` / `each thus …` wordings.

Instrumenting the traversal (on a scratch branch; probes kept out of `examples/`
and out of the committed tree) to dump, for every segmentation, the per-slot
`slots[k]` list in traversal order together with the open `cap`:

| depth | uniform `cap` | per-slot traversal indices of the tuple words |
|---|---|---|
| 4 | 10 | 10 / 0 / 16 / 7 |
| 5 | 7 | **7 / 0 / 13 / 99 / 11** |
| 6 | 5 | 6 / 0 / 32 / — / — / 5 |

So, in traversal-index units, at the depth-5 segmentation
`[(0,3),(3,10),(10,13),(13,15),(15,19)]` the words sit at `hits` 7, `justice` 0,
`dupe` 13, `hid` 99, `came` 11, against a uniform `cap` of
`affordable_opening_width(5, 4 000) = 7` (`1+7+49+343+2401 = 2 801 ≤ 4 000`; at 8
the series is 4 681).

**Confirmed**, narrowly: the depth-1 word is at traversal index 7 against
`LEXICAL_BRANCH_STAGE_0`-derived cap 7, and the walk's test is
`for i in 0..slots[k].len().min(caps[k])`, i.e. `index < cap`, so it is excluded
by exactly one index.

**Refuted, and this is the load-bearing correction:** "one index outside" is a
statement about the *first slot only*, not about the tuple. Three of the five
slots are far outside cap 7 — one by 6 indices, one by 92. The item's own
`7/0/3/85/7` is in **shortlist** units; in the units the walk actually indexes,
the same tuple is `7/0/13/99/11`. This is the fourth independent confirmation of
the re-sorting fact (`SlotAlt::contribution` divides by the slot's word count),
and it settles the recorded `237-of-240` versus miss-by-one disagreement in the
direction both sides were half right about: the miss-by-one is real and it is
*not* the binding exclusion.

## 2. Which quantity is actually charged

Priority-1 steer item, answered from the source on this head, not from a model.
`src/lib.rs` charges **`popped`** — the number of nodes the walk actually
expands — against `LEXICAL_HEAP_POP_LIMIT` (and `spent_pops + popped` against
`LEXICAL_GLOBAL_POP_BUDGET`). There is no `pops_left` and no per-slot charge.

The only model of `popped` the loop itself supplies is the truncated lattice:
`seen` admits each `(k, prefix)` once, so a pass pops at most as many nodes as
the `caps`-truncated product has above its leaves, which is

```text
F(c) = 1 + c_0 + c_0·c_1 + … + c_0·…·c_{d-2}          (c_{d-1} absent)
```

`F` is the **product of the per-slot prefix widths**, it is charged **jointly,
once, for the whole vector**, and `F(c) ≤ pop_limit` is sufficient for a pass to
complete without hitting the pop wall. `affordable_opening_width`'s
`1 + w + … + w^(d-1)` is this read on a constant vector; the scalar is the
special case, not the general quantity.

This is the same quantity the coordinator's 0f3a173 finding computes, and the
three rows it quotes reproduce exactly under `F`: `d=5` `9/10/10/10/160` → 10 000;
`d=7` `10/11/11/11/11/14/160` → 2 210 791; `d=4` `36/36/38/160` → 50 581. So
that finding and this front's derivation agree on the charged quantity, and the
marginal table it warns about is inadmissible for exactly the reason stated: the
lattice is a product, so per-slot marginals do not stack. That is pinned by
`per_slot_marginals_do_not_stack_into_a_jointly_affordable_allocation`, which
also checks that every entry of each row is individually affordable — which is
what makes such a table look legal.

## 3. The rule

`per_slot_opening_widths(pool_sizes, pop_limit, uniform_floor)`. The per-slot
cost is **measured off the traversal**, not assumed: slot `k` is expanded once
per prefix that reaches it, and the number of such prefixes is
`M_k = c_0·…·c_{k-1}`, so the marginal cost of one more candidate in slot `k` is
exactly `M_k` pushes. Keyed only on pool sizes, the pop allowance and the scalar
opening width — no word, clue, target or segmentation is read, and no
environment knob is added.

* Pass 1 charges each slot its own `M_k`, in slot order, never below
  `uniform_floor` (so it cannot narrow what already worked).
* The last slot's width does not enter `F` — a leaf hangs off it — so it is
  allocated its whole list and charged nothing.
* Pass 2 hands any overrun back to the widest charged slot until `F(c) ≤
  pop_limit` **jointly**.

The rule is `#[cfg(test)]` and is **not** called by the traversal. The traversal
still opens every slot at the scalar `affordable_opening_width`.

## 4. The priced negative, with arithmetic

**Claim.** For an index tuple `t` at depth `d` with per-slot pools wide enough,
admission by *any* allocation requires `c_k ≥ t_k + 1` (the walk's test is
`index < cap`), and `F` is monotone non-decreasing in every component — so
`F(t + 1)` is a **lower bound over every allocation that admits `t`**, and
`F(t + 1) − pop_limit` is the exact shortfall in pops. No allocation rule, of any
shape, can beat it, because the bound is a property of the tuple and the limit
rather than of the rule.

**Applied to the canonical depth-5 tuple `t = (7, 0, 13, 99, 11)`:**

```text
minimum admitting caps   (8, 1, 14, 100, 12)
F = 1 + 8 + 8 + 112 + 1 120 · 100
  = 1 + 8 + 8 + 112 + 11 200 = 11 329
pop limit                4 000
shortfall                7 329 pops = 2.83 ×
```

Monotonicity is not assumed: `admission_is_impossible_for_every_allocation_once_the_frontier_bound_is`
brute-forces it over every vector with entries in `1..=3` up to four slots, and
then enumerates every allocation that could admit a witness tuple and checks none
of them fits.

**The deeper finding, and the reason this is not a tuning mistake.** `F` is a
product, so the pop allowance fixes `c_0·c_1·…·c_{d-2}` and **not the factors**.
At depth 4 with a 160-wide last slot there are **58 106** distinct jointly
affordable allocations, and they are not interchangeable: `[1,40,7,7,160]`,
`[2,28,7,7,160]`, `[3,22,7,7,160]` and `[8,8,7,7,160]` all satisfy `F ≤ 4 000`
and buy very different enumerations for the same pops. **The pop budget has no
opinion about the per-slot split**, so any rule presenting a per-slot width as
*derived* from the allowance is presenting a choice as an arithmetic result.

This was not a theoretical observation. It was confirmed by mutation: making the
rule *forget* the per-slot cost (`by_budget = room` instead of
`room / marginal`) is **behaviourally equivalent on every tested shape**, because
pass 2's joint repair washes the difference out. The suite catches that mutation
only through the joint-affordability and no-narrowing assertions, which is the
correct place for it, and the mutation is reported here rather than hidden: the
per-slot term in pass 1 is a tie-break, and the joint constraint is what binds.

Choosing among the affordable family needs a **second, non-pop criterion**, and
the only other ceiling in the search is not one: `LEXICAL_GLOBAL_EMISSION_BUDGET`
*saturates* (16 384/16 384) rather than trading off, and the per-segmentation
emission allowance is already about four fifths spent before enumeration begins.
That is the same wall `REPORT-4b1e07` identified, now reached from the other
side.

## 5. Why the rule was not shipped, measured

The rule was wired into the traversal — per-slot ceilings driving the expand
loop, with the widening ladder made per-slot so a draining stage still opens the
next one — and measured on the fence:

| configuration | result |
|---|---|
| baseline `534a39c` | 12 passed / 1 failed; 3 pool-reach guards green |
| rule wired, all slots per-slot | **2 of 3 pool-reach guards red** (`…_matches_deep_in_a_span`, `…_deeper_than_one_walk`) |
| rule wired, **only slot 0** allowed to exceed the floor | **SIGKILL** (OOM) |
| rule wired, per-slot ladder | SIGKILL (OOM) |

The OOM is the informative one and it is arithmetic, not a tuning miss. The
traversal's live cost is the **frontier** `F`, not the pop count: `F` is exactly
the number of `seen`/`heap` entries a pass allocates. At the canonical depth-5
shape the rule opens `[160, 7, 7, 7, 93]`, `F = 64 001`, against the uniform-7
`F = 2 801` — a **22.8×** increase in resident heap, × 256 retained
segmentations. Widening the *cheapest* slot is not free in the units that
matter, and the units that matter are memory and the emission allowance, not the
number of pushes.

So the rule is admissible, correct, and unusable: **a priced negative.**

## 6. Arithmetic corrections this front confirms

Requested of the successor front and independently reproduced here, so the
record's errors are not propagated:

* At depth 5 the per-slot affordable width at slot 1 is **10**, not 7 —
  `3 998 ≤ 4 000`. The printed `9 / 7` *decreases* and so contradicts the
  monotonicity the accompanying test claims to assert. The correct reading of
  that derivation is `9 / 10 / 10 / 10 / 160`, and the `d=5` row's cost is
  2 801.
* The **depth-6** last-slot cost is **3 906**, not 2 801 carried over:
  `1+5+25+125+625+3125 = 3 906`. Measured independently by this front's
  `first_leaf_frontier` on the uniform-5 vector at depth 6, which is exactly what
  `the_scalar_opening_and_its_frontier_are_what_the_docs_say` prints.
* `assert_eq!(previous, list)` with `list = SPAN_SHORTLIST = 160` refutes the
  `93` in that test's own doc comment. The `93` is the *measured* width of one
  segmentation's list, not a derived width, and does not belong in a table whose
  header says 160-wide lists.

The scalar law's own row, printed by the test log on every `--lib` run:

```text
depth 1: uniform 10, frontier 1
depth 2: uniform 10, frontier 11
depth 3: uniform 10, frontier 111
depth 4: uniform 10, frontier 1111
depth 5: uniform 7,  frontier 2801
depth 6: uniform 5,  frontier 3906
depth 7: uniform 3,  frontier 1093
depth 8: uniform 3,  frontier 3280
```

## 7. Tests, and why they are not self-referential

Ten tests, all in `mod tests`, none naming a phrase, word, clue or target. Every
expectation is re-derived by **generating the truncated lattice**
(`counted_prefixes`, `counted_frontier`) rather than by calling the production
recurrence, so mutating `first_leaf_frontier` or `per_slot_opening_widths` moves
the production side alone and the assertion still bites. The rule is asserted
against its *measured marginal cost* and against the depth law in
traversal-index units, and the depth law's ceiling sequence is recomputed in the
test from counted prefixes of a fixed reference shape.

Mutation-verified. Eleven mutations, ten caught:

| mutation | caught by |
|---|---|
| frontier counts the last slot | 4 tests |
| drop the uniform floor (narrows) | no-narrowing |
| ignore the marginal cost (uniform) | proportionality |
| one candidate short | proportionality |
| drop the overrun repair | proportionality |
| the free slot charged the floor | free-slot test |
| the repair spends the free slot | free-slot test |
| off-by-one in the marginal base | proportionality |
| frontier drops its leading 1 | 2 tests |
| scalar law widened to full pools | no-narrowing |
| **forget the per-slot cost** | **joint-affordability only** — see §4; reported, not hidden |

`the_free_slot_is_opened_to_its_whole_pool` earned its place before it was
finished: it caught a real defect in this front's own rule, where pass 2's
hand-back could spend the *free* last slot.

## 8. Gate

Release, `CARGO_TARGET_DIR` local, worktree `/workspace/madgab-width-9e2b41`.

| command | result |
|---|---|
| `cargo test --release --test corpus_integration` | **12 passed, 1 failed** (13) |
| `cargo test --release --lib` | **72 passed, 0 failed** |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed, 0 failed** |
| `git diff --numstat` | **`src/lib.rs` 667 insertions, 0 deletions; production lines changed: 0** |

The single failure is **`approximate_finds_classic_madgab_resegmentation`**, the
expected pre-existing one. It is **not** re-pinned and no literal in it or in
the three `approximate_pool_reaches_*` guards is touched. `tests/` is
byte-identical to `534a39c`; `approximate_output_is_locked` is not re-baselined;
`approximate_finds_recognize_speech_resegmentation` (canonical case 1) is green.
The baseline on pristine `534a39c` is identical, 12/1, which is expected given
the production diff is zero and is the reason the branch is behaviour-neutral by
construction rather than by measurement.

Note for the next run: `corpus_integration` is **SIGKILLed on this host at
default test parallelism** (13 tests, 26 s of work, host memory pressure). It is
not a property of any change here — pristine `534a39c` shows it too. Run it with
`-- --test-threads=2`. A reviewer who sees SIGKILL at default parallelism on this
host should reproduce on the base head before reading anything into it.

## 9. What the successor front inherits

1. The width mechanism is **closed as a pop-budget question**. The charged
   quantity is `F`, it is joint, and it is a product.
2. The canonical tuple is **2.83× outside the pop limit** and no allocation rule
   reaches it. The only remaining levers are raising
   `LEXICAL_HEAP_POP_LIMIT` (and paying 22.8× in frontier), or a mechanism that
   is not prefix enumeration.
3. A per-slot width is **not derived** from the pop allowance; 58 106 allocations
   satisfy it jointly at depth 4. Any width rule must be argued from a criterion
   the pop budget does not already fix.
4. The emission ceiling is **not** that criterion: it saturates, and it is
   already ~81% spent per segmentation. `EMIT_PROFILE_MAX_DEEP = 3` caps the
   reserve at three non-zero coordinates while the canonical tuple needs four,
   so the reserve cannot supply the two deep coordinates either.
5. The concrete successor is therefore the **shape of the coverage reserve**, not
   its width: a reserve that samples a *non-corner* profile would spend its
   allowance on depth rather than on the cost-best corner, which is the one
   place the emission budget can still buy something the traversal cannot.
