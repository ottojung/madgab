# w-1c7d40 - the per-slot order is not a lever: 16 general keys and 4 priced orders, all negative on reach

Front `agent-1c7d401`, branch `madgab-slotorder-1c7d40`, base `afe756a`
(`post-milestone-acceptance`, integrated REPORT-4d7c12 and REPORT-3a8c05).
Release mode throughout, `CARGO_TARGET_DIR=/workspace/target-1c7d40`.

**Verdict: HOLD.** The stage is re-derived on integrated HEAD, the
candidate order is shown *not* to be the lever the parent front left open,
and four general orderings are priced with full measurements. No production
change is proposed: the release build of this branch is byte-identical to
the base, because every line of the diff is `#[cfg(test)]`-gated.

Reproduce every number below with:

```
cargo test --release --lib front_1c7d40 -- --ignored --nocapture --test-threads=1
```

---

## 0. What was added, and where it lives

936 lines in `src/lib.rs`, **936 insertions and 0 deletions**: three
`#[cfg(test)]` items and four `#[cfg(test)]` hook statements. No production
line is modified, removed or reordered, so a release build of this branch
compiles to the base's behaviour by construction rather than by assertion.

* `mod slot_probe` — a per-thread recorder plus the candidate `SlotOrder`
  enum and its `permutation` function. Armed only from a test.
* `mod front_1c7d40` — the measurement driver: six `#[ignore]`d tests and
  a 16-entry key table.
* hook sites: after a slot's `sort_by` (re-order), after `cap` is computed
  (record the segmentation), at the reserve's admission, and at the
  traversal's leaf.

Both modules are excluded from the phrase fence by the existing
`#[cfg(test)] mod` rule in `tests/no_phrase_hard_coding.rs`, and the fence is
**9/9**. `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"`
over `src/` returns 42 lines; the ones this front added are inside
`mod front_1c7d40`, which is `#[cfg(test)]` and names the two examples as
*data* for the measurement only. Nothing in `src/` outside a `#[cfg(test)]`
module reads a word, a clue, a target or a segmentation — that is what the
sixteen keys in §3 are, one by one.

---

## 1. Obligation 1: the per-slot order re-derived on integrated HEAD

`Generator::generate_pool`, `SearchMode::approximate()`, `top_n = 50`,
`beam_width = 64`, release.

```
affordable_opening_width(5, 4000) = 7
case 2: pool 18949  reach None  reserve 3051  traversal 12309
case 1: pool 18289  rank Some((27, 0.9199502875218423))
case 2: 2/240 funds carry the multiset
  rank 151 widths [160, 7, 160, 160, 93] cap 7 -> indices [7, 0, 13, 99, 11]  all inside cap: false
  rank 187 widths [160, 2, 160, 160, 93] cap 7 -> indices [7, 0, 22, 99, 11]  all inside cap: false
```

Opening width **7** at depth 5; needed per-slot indices **`7 / 0 / 13 / 99 /
11`** at schedule rank 151 and **`7 / 0 / 22 / 99 / 11`** at rank 187; the
canonical is absent from the 18,949-member pool. All of
[REPORT-4d7c12.md](REPORT-4d7c12.md) §1 and §7 reproduces exactly, so the
base is confirmed and every number below is a base number.

The green control, spelled out of the traversal's own emissions rather than
asserted:

```
case-1 traversal sched 0 tuple [1, 0, 0, 2] cap 10 all inside true
case-1 traversal sched 3 tuple [1, 0, 2, 2] cap 10 all inside true
case-1 traversal emissions spelling the canonical multiset: 2
```

**So the parent's framing is confirmed on integrated HEAD**: the green case
is emitted by the traversal at tuples whose every index is inside the
opening width, and the canonical is not, and the difference is the order of
`slots[k]`. The question this front had to answer is whether that order can
be re-specified. §2 and §3 answer it: no.

---

## 2. Obligation 2: the current order as a defect, measured over the whole pool

`SlotAlt::contribution` is, verbatim, a five-term additive proxy on
`axes::{SIMILARITY_PER_WORD, WORD_NOVELTY, FAMILIARITY, CLOSED_CLASS, SHAPE}`
— the *acoustic* half of the objective, with `RHYTHM` absent because
`RHYTHM` is not additive over a slot. It is a good proxy for the score of a
slot in isolation. The question is whether it is a good proxy for the
*order* in which the opening width should be spent.

### 2.1 The opening width is spent on a very narrow band

Over every captured slot list of a full case-2 run — **1,268 slot lists**,
which is every slot of every one of the 240 funded segmentations — the
quantity that matters is how much of the shipped key separates the best
candidate in a slot from the last one the opening width admits:

| quantity | case 2 | case 1 |
|---|---|---|
| slot lists captured | 1,268 | 1,138 |
| contribution lost from the best to the `cap`-th candidate, mean | **0.011272** | **0.009459** |
| the same, maximum over all lists | 0.051000 | 0.073367 |
| cost percentile of the `cap`-th candidate, mean | **0.5032** | 0.4337 |
| cost percentile of the list's own midpoint, mean | 0.7502 | 0.7431 |

**This is the defect, and it is not the one the item's framing suggests.**
The first plausible story — "the opening width is wasted on near-ties
because `contribution` is clustered" — is **refuted by the third row**: the
candidate at the edge of the opening width already sits at cost percentile
**0.50**, i.e. at the *median* of its slot's cost distribution, not in a
cheap corner. The width is not being spent on a cluster.

What the first two rows say is sharper. A 7-wide opening on a 160-wide list
is spending **0.011 of contribution** — the seventh candidate is 1.1% of a
score behind the first. The width is not cheap, it is *fine-grained*: it is
a 1% band of the key, and a candidate sitting 3.5 positions outside it is
outside by a margin the key cannot resolve at 3 significant figures. There
is no tie to break and no cluster to escape; the opening width is simply a
hard cut through a smooth function, and the canonical's `hid` at index 99
of 160 is 14 positions past a cut that falls where it falls.

### 2.2 What separates the needed word from the seven ahead of it: nothing

The needed words' own positions and features, over the carrying funds:

| quantity | case 2 (2 funds, 10 observations) | case 1 (14 funds, 56 observations) |
|---|---|---|
| mean position of the needed word in its slot | **26.90** | **5.46** |
| mean cost percentile of the needed word | **0.4302** | **0.3775** |
| mean normalised contribution of the needed word (1.0 = slot's best) | 0.6565 | 0.8619 |

The needed words are **cheaper than the median of their own slot** (0.43 and
0.38 against the `cap`-th candidate's 0.50 and 0.43). So the intuition that
drives a cost-descending rule — "a faithful near-homophonic resegmentation
needs an expensive word, and the cheap-first order buries it" — is
**backwards on this measurement**: the needed words are on the cheap side of
their slots' medians, and the shipped cost-dominated order is, if anything,
already helping them. The words are excluded for being at position 27 and 5
on a key that separates them from the seven ahead by 0.011, not for being
expensive.

This is obligation 2's answer stated plainly: **there is no measurement that
separates a needed word from the candidates ahead of it.** Not cost (it is
below the median), not contribution (the band is 0.011 wide and the needed
word is inside the band whenever it is inside the cap at all), and §3 prices
the remaining features one by one rather than asserting it.

---

## 3. Obligation 3: sixteen general keys, priced against the opening width

Rather than argue from §2, the question was put to every general per-candidate
key that could plausibly be written. Each key is a comparator over two
`SlotAlt`s reading only measurements already carried on the candidate
(`cost`, `contribution`, `syllables`, `familiarity`, `shape`, `closed`,
`reused`) or a sub-expression of `contribution`. Each carrying fund's slots
were re-ranked under each key and the tuple's own five (resp. four) words
located. **A key that lands all of them inside the cap is an ordering that
reaches the tuple.** `need contr` is the needed word's normalised
contribution, reported only for the observations that land inside the cap —
so a key with a high `need contr` is a key that brings the *right* word in,
and a key with a low one is bringing the wrong word in.

Case 2 — 2 carrying funds, 10 slot observations, cap 7:

| key | funds fully inside | mean position | need contr |
|---|---|---|---|
| **A contribution desc (shipped)** | **0/2** | 26.90 | 1.0000 |
| D contribution asc (the inversion) | 0/2 | 87.60 | 0.0000 |
| E edit cost asc | 0/2 | 25.30 | 0.8809 |
| F edit cost desc | 0/2 | 84.10 | 0.0000 |
| G syllables asc | 0/2 | 35.20 | 0.6534 |
| H syllables desc | 0/2 | 43.90 | 0.6494 |
| I familiarity desc | 0/2 | 66.00 | 0.7927 |
| J familiarity asc | 0/2 | 48.50 | 0.3185 |
| K shape desc | 0/2 | 32.30 | 0.6534 |
| L closed-class asc | 0/2 | 36.40 | 0.6534 |
| M reuse asc | 0/2 | 38.50 | 0.7927 |
| N contribution, cost term only | 0/2 | 25.30 | 0.8809 |
| O contribution, no familiarity | 0/2 | 39.20 | 0.4700 |
| P contribution, no cost | 0/2 | 60.10 | 0.9316 |
| **Q cost asc, tie contribution desc** | **0/2** | **24.70** | 0.9244 |
| R contribution desc, tie cost asc | 0/2 | 26.90 | 1.0000 |

Case 1 — 14 carrying funds, 56 slot observations, cap 10:

| key | funds fully inside | mean position | need contr |
|---|---|---|---|
| **A contribution desc (shipped)** | **4/14** | **5.46** | 0.9166 |
| D contribution asc (the inversion) | 0/14 | 122.95 | 0.0814 |
| E edit cost asc | 3/14 | 12.16 | 0.8016 |
| F edit cost desc | 0/14 | 111.27 | 0.4429 |
| G syllables asc | 0/14 | 42.57 | 0.8242 |
| H syllables desc | 0/14 | 56.79 | 0.5761 |
| I familiarity desc | 0/14 | 29.02 | 0.9301 |
| J familiarity asc | 0/14 | 98.43 | 0.0814 |
| K shape desc | 0/14 | 44.32 | 0.8343 |
| L closed-class asc | 0/14 | 73.30 | 0.7976 |
| M reuse asc | 0/14 | 48.41 | 0.8242 |
| N contribution, cost term only | 3/14 | 12.16 | 0.8016 |
| O contribution, no familiarity | 0/14 | 78.36 | 0.0814 |
| P contribution, no cost | 0/14 | 26.62 | 0.7860 |
| **Q cost asc, tie contribution desc** | **4/14** | 11.09 | 0.8051 |
| R contribution desc, tie cost asc | 4/14 | 5.46 | 0.9166 |

**Not one of the sixteen keys puts the canonical tuple inside a 7-wide
opening.** The best is Q at mean position 24.70 against a cap of 7 — a 3.5x
gap, and Q's `need contr` of 0.9244 says the words it does bring inside the
cap are the ones that are nearly the slot's best under it, which is not the
tuple. The shipped order A is the *best or joint-best* key on case 1 and is
beaten on case 2 only by Q, E and N, all of which are cost-dominated
re-derivations of A and all of which also fail.

Two rows are worth a successor's attention because they are the cheapest
available confirmation of the mechanism. **P (contribution with the cost
term removed) has the highest `need contr` on case 2, 0.9316, and still
reaches nothing** at mean position 60.10: the words are not excluded for
being acoustically poor *relative to the slot*, they are excluded for being
at position 27 in a list of 160. **O (contribution with familiarity removed)
reaches mean position 39.20 with `need contr` 0.4700**: the lexical half of
the proxy is what pushes the needed word down, and removing it is not enough
either, because the two halves are correlated on this data. There is no
sub-expression of `contribution` to drop.

### 3.1 The four full-pipeline prices

Four general orderings, each re-run through the whole pipeline, each reported
with tuple reach, pool size, wall clock, the green case's pool rank and
score, and the emission split. `reach` = the pool holds a member whose word
multiset is exactly the canonical one.

| order | case-2 reach | pool 2 | wall 2 | case-1 reach | case-1 rank | score 1 | pool 1 | wall 1 | reserve / traversal |
|---|---|---|---|---|---|---|---|---|---|
| **A base `contribution` desc** | **0** | 18,949 | **1,025 ms** | 1 | **27** | 0.9199502875 | 18,289 | 1,031 ms | 3,051 / 12,309 |
| **B cost-stratified interleave** | **0** | 18,949 | 1,271 ms | 1 | **27** | 0.9199502875 | 18,289 | 1,163 ms | 3,051 / 12,309 |
| **C half-split interleave** | **0** | 19,251 | 1,176 ms | 1 | **27** | 0.9199502875 | 18,120 | 1,005 ms | 3,066 / 12,294 |
| **D contribution ascending** | **0** | **8,872** | 2,308 ms | **0** | — | — | **11,025** | 2,677 ms | **466 / 4,470** |

Against the item's base figures of **9.5 s** and **21.3 s**: every row is
1.0-2.7 s, i.e. **inside both by a factor of three to nine**. Nothing here is
a wall-clock question.

The three shapes are distinct and all three are negative, in three distinct
ways:

* **B, cost-stratified interleave** — round-robin over ten equal-count
  strata of each slot's own cost distribution, shipped order preserved
  inside a stratum. Motivated by §2.1's clustering hypothesis, which §2.1
  itself refutes. It is **behaviourally inert on the pool** (18,949 and
  18,289, both exactly base) and costs **+24% wall clock** on case 2, because
  the interleaved lists are no longer sorted by the key and the heap's
  equal-bound tie-breaks get worse. Reach 0.
* **C, half-split interleave** — alternating between the cheap half and the
  dear half. **The only order that changes the pool materially** (+302 on
  case 2, −169 on case 1) and it changes it in *both* directions, which is
  the signature of a re-shuffle rather than a re-targeting. It preserves the
  green case's rank and score exactly. Reach 0.
* **D, the pure inversion** — dearest contribution first. **Destructive, and
  priced as such**: the pool collapses 18,949 → **8,872** (−53%), the green
  case leaves the pool entirely (reach 0, no rank), the reserve's spend falls
  from 3,051 to **466** and the traversal's from 12,309 to 4,470, and wall
  clock rises **+125%**. This is the same arithmetic
  [REPORT-4d7c12.md](REPORT-4d7c12.md) §4 found for a joint reserve: the
  marginal, cheap-end emissions are what populate the pool, and any rule that
  spends the width on dear candidates displaces exactly those. D is the
  control that shows the descending property is load-bearing, not decorative.

**B and C are clean negatives; D is a strong one.** None reaches.

---

## 4. Obligation 4: the scoring caveat, stated and not re-derived

[REPORT-3a8c05.md](REPORT-3a8c05.md) §3 is a hard monotone lower bound and it
is inherited here untouched: **1,126 pool clues dominate the canonical on
`SIMILARITY + PARSIMONY + RESEG` at once, so no monotone objective over any
measurable per-candidate property can rank it better than 1,127, and its
shipped score is 0.0943520415 below the top-50 cutoff.** That table is not
re-derived and not duplicated above.

Measured and reported, not fixed:

* the canonical is **absent from the pool** on every one of the four orders,
  so it has no pool rank and no score to report; the item's `reach` column
  is the whole of what this front can move.
* the green case's rank and score are **identical across all four orders**
  (27, 0.9199502875) and identical to base.

No score weight, no axis definition and no `Metrics` field is touched by this
branch; `SPARSE_SHORTLIST`, the depth cap, `LEXICAL_HEAP_POP_LIMIT` and the
emission budget are untouched; `examples/` and the baselines are untouched.
**A reach is not a display**, and this front does not attempt to make it one
— the scoring surface belongs to the sibling front `w-9b4a15`.

---

## 5. Obligation 5: the descending-bound property, and what it is worth

The heap key is `bound(prefix)`: an *upper* bound on the real score of a
prefix, computed from the prefix plus the suffix's extremums, and it does not
read the order of `slots[k]`. So the descending-bound property is a property
of the *walk*, and a candidate order cannot take it away. What the order does
decide, and what was measured:

| order | traversal emits inside cap / outside | distinct slot-0 indices reached | mean contribution given up across the opening width |
|---|---|---|---|
| A base | 12,431 / 0 | 10 | **+0.011416** |
| B cost-stratified | 12,431 / 0 | 10 | +0.011416 |
| C half-split | 12,432 / 0 | 10 | +0.010028 |
| D inverted | **4,262** / 0 | **18** | **−0.010434** |

* **A preserves the property exactly, and buys its narrowness with a real
  cost.** Every one of the 12,431 traversal emissions is inside the opening
  width, and the walk reaches exactly 10 distinct slot-0 indices — the
  opening width, and no more. The price is the last column: the width is
  spent on a **0.0114**-wide band of the key. That is the property working
  as designed and it is also the reason the canonical is unreachable.
* **B preserves it trivially and degenerately**: the numbers are A's to four
  figures, because B's first `cap` entries are one per cost stratum and the
  heap's own bound re-sorts them anyway. It is a no-op that costs 24% wall
  clock.
* **C preserves it and very slightly improves the band** (0.0100 against
  0.0114), by putting the dearest half of the cheap half at the far edge of
  the width. That is a real but small improvement to the descending
  property, and it buys 0 reach.
* **D destroys it.** The mean contribution given up across the width goes
  **negative** — the candidates at the far edge of the opening width are now
  *better* than the ones at its near edge — and the walk's emissions fall
  from 12,431 to 4,262 while reaching 18 distinct slot-0 indices instead of
  10. D is a wider, worse, and much slower walk.

**The property is not the thing that is broken, and it is not the thing that
costs the tuple.** A is the tightest descending order available and it is the
one that reaches the green case; the canonical is lost at a *cut* through a
smooth function, and every key that moves the cut moves it in a direction
that is worse for the pool, not better for the tuple. A rule that worked only
by spending more of the 16,384-emission budget would be a priced negative in
its own right; §3.1 shows that the rules which do spend more of it (D) lose
the pool.

---

## 6. Tree state and constraints

`CARGO_TARGET_DIR=/workspace/target-1c7d40`, release throughout:

| target | result |
|---|---|
| `cargo test --release --lib` | **74 passed / 0 failed / 6 ignored** (24.4-25.3 s over runs) |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed / 0 failed** |
| `cargo test --release --test corpus_integration -- --test-threads=2` | **12 passed / 1 failed** — `approximate_finds_classic_madgab_resegmentation`, the known pre-existing red at base, **not re-pinned and not made worse** |
| `cargo build --release` | clean, **no warnings** |

The 6 ignored are this front's own `#[ignore]`d probes
(`..._reproduce_the_opening_width_and_the_needed_indices`,
`..._what_separates_the_needed_word`, `..._price_the_candidate_orders`,
`..._descending_bound_under_each_order`,
`..._rank_the_needed_word_under_every_general_key`,
`..._how_wide_the_key_is_over_a_whole_list`). The 74 is the base
count; all 6 are accounted for.

`git diff --stat` is **936 insertions, 0 deletions, one file**. Every
insertion is either inside a `#[cfg(test)] mod` or is a `#[cfg(test)]`-gated
statement at an existing hook site. No score weight, axis definition,
`Metrics` field, `SPARSE_SHORTLIST`, depth cap, pop limit, emission budget or
baseline is touched. No `ZZ_*` probe, no environment knob, no debug binary;
`examples/` is untouched. All instrumentation is `#[cfg(test)]` in
`src/lib.rs` on `madgab-slotorder-1c7d40` and is **not** proposed for
integration as production source.

`cargo fmt --check` and `cargo clippy` **cannot run on this host** (see
[../environment-notes.md](../environment-notes.md)) and are **not claimed**.

---

## 7. The counterfactual table, and the exact next coordinate

### 7.1 The full counterfactual table

Reach of the canonical tuple is **0** in all twenty priced configurations —
sixteen keys (§3) and four full-pipeline orders (§3.1) — and the mechanism is
the same in all twenty: the needed words sit at mean position 26.90 of a
160-wide list against an opening width of 7, and no per-candidate key moves
them past 24.70. The consolidated table is §7.3.

The green case's rank, score and pool are **unchanged at 27 /
0.9199502875 / 18,289** under every order that does not destroy the pool, and
the canonical green clue remains in the displayed 50 — so nothing here
disturbs the one blocker that is *not* red.

### 7.2 The exact next coordinate

The parent front's §5 named this coordinate and it is **closed, with the
arithmetic**:

> Stage 4, `src/lib.rs:2194`'s
> `let mut cap = affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT).min(widest);`
> and the ordering of `slots[k]`.

The ordering half is now closed too. What survives is the **width**, and it
is not a budget question either — the item forbids buying width, and §3.1
shows every rule that buys spend loses the pool. The surviving fact is
arithmetic and it is the same one REPORT-4d7c12 §5 measured for coverage:
at the decisive fund the joint index space is

```
160 x 7 x 160 x 160 x 93 = 2,666,496,000 cells
```

and the canonical sits at `[7, 0, 13, 99, 11]`. This front adds the
matching measurement on the *key's* side, over 1,252 case-2 slot lists and
1,137 case-1 ones:

| quantity | case 2 | case 1 |
|---|---|---|
| `contribution` width over a **whole** 160-wide slot list, mean | **0.0338** | **0.0322** |
| the same, maximum | 0.0613 | 0.0989 |
| `contribution` width across the **opening width**, mean | 0.0114 | 0.0095 |
| the same, maximum | 0.0510 | 0.0734 |
| the needed word's fraction of the whole-list key width, mean | **0.3435** | 0.1381 |
| the same, maximum | 0.6615 | 0.6204 |

So the opening width of 7 already covers **34% of the entire key range of a
160-wide slot**, and the needed words sit at 34% (case 2) and 14% (case 1) of
that range. The cut is not a hair's breadth that a better key could find a
place inside — it is already a third of the range — and the canonical's
deepest needed index, **99 of 160**, is at 62% of the range. Reaching it by
widening alone needs a uniform width of about **100, i.e. 14.3× the current
7**, and `1 + 100 + 100^2 + 100^3 + 100^4` is 1.01e8 pops against a
`LEXICAL_HEAP_POP_LIMIT` of 4,000 — a factor of 25,000. That is the
arithmetic that makes "just widen it" a priced negative before it is even
run, and it is the same 3-to-5-orders-of-magnitude wall REPORT-4d7c12 §5
measured for coverage.

**The next coordinate, named precisely enough to start from:** the opening
width is derived from `LEXICAL_HEAP_POP_LIMIT` by
`affordable_opening_width` under the *scalar* law
`1 + w + ... + w^(d-1) <= pop_limit`, and the derivation assumes a **uniform**
width across all `d` slots. At `d = 5` and `pop_limit = 4000` that gives 7.
The traversal's *actual* per-slot marginal cost is the product of the widths
above it (`M_k = c_0 ... c_{k-1}`, already computed in
`first_leaf_frontier`'s neighbourhood and already stated in the
`w-9e2b41` doc comment at `src/lib.rs:400-430`), so the uniform law
**over-charges slot 0 by the whole product above it and under-charges the
last slot by a factor of the same product**. A *non-uniform* derived width —
uniform 7 today, and a per-slot vector whose slot 0 is much wider because
slot 0 is the only slot whose expansion is paid for once per segmentation —
is the one lever at this stage that is not a coverage rule, not a budget
increase, and not an ordering. It is out of this front's scope
(`LEXICAL_HEAP_POP_LIMIT` is frozen, and `w-9e2b41` owns the per-slot width
allocation), and it is the coordinate a successor should be handed.

**A second, cheaper and more specific coordinate**, which this front's own
instrument makes visible and which nobody has claimed: the needed words are
at mean position **26.90** and the deepest at **99**, while the `cap`-th
candidate is already at cost percentile **0.50** and the needed words at
**0.43** (case 2) and **0.38** (case 1). There is therefore a *third*
mechanism between "order" and "width" that this front did not price because
it is not an order: **which candidates the cut is taken against, per slot.**
Today the cut is `index < cap` on one flat list, and the walk is re-sorted by
`bound` afterwards, so the cut discards 153 of 160 candidates and lets the
heap choose 7 of the survivors. A per-slot cut keyed on the slot's own cost
distribution — admit the `cap` candidates *straddling the slot's median
cost*, i.e. the cheapest half and the dearest half in equal measure, rather
than the `cap` best — is not the same rule as B (B reorders the list and lets
the heap re-sort, which is why B is inert) and it is not the same rule as
`w-9e2b41`'s per-slot *width* allocation. It selects a **set** rather than
an order, it needs no extra pops because it spends the same 7, and §2.1's
percentile-0.50 measurement says the needed words at percentile 0.43 are
*just* on the wrong side of a median cut — the tightest margin of anything
measured in this report and the cheapest thing left to try.

---

## 7.3 The 21 configurations

| what was priced | configurations | reach | best mean position vs cap 7 | green rank | verdict |
|---|---|---|---|---|---|
| per-candidate keys (§3) | 16 | 0/16 | 24.70 (Q) | not re-run | refuted |
| full-pipeline orders (§3.1) | 4 | 0/4 | — | 27, 27, 27, — | refuted |
| **total** | **20** | **0/20** | | | |

(The key-width measurement of §7.2 is a fifth `#[ignore]`d probe and prices
no order; it is reported because it sizes the two remaining coordinates.)

---

## 8. Verdict

**HOLD.** Three things are established and none of them is a production
change.

1. **The stage re-derives exactly on integrated HEAD**: opening width 7 at
   depth 5, needed indices `7 / 0 / 13 / 99 / 11` at rank 151 and
   `7 / 0 / 22 / 99 / 11` at rank 187, the green control's `[1,0,0,2]` and
   `[1,0,2,2]` inside a cap of 10, pool 18,949 / 18,289, green rank 27.
2. **The per-slot order is not the lever.** Sixteen general per-candidate
   keys and four full-pipeline orderings, twenty priced configurations, all
   zero on reach. The decisive measurements are that the needed words are
   *cheaper* than the median of their slots (percentile 0.43 and 0.38 against
   the `cap`-th candidate's 0.50 and 0.43) and that the opening width is a
   **0.0114-wide band** of the key — so the loss is a cut through a smooth
   function, not a cluster to escape, a tie to break, or an expensive word to
   prioritise. The parent's motivating hypothesis — that a faithful
   resegmentation needs a locally expensive word and the cheap-first order
   buries it — is **measured to be backwards on this data**.
3. **The descending-bound property is intact and is not the cost.** The
   walk's key does not read the slot order; the shipped order is the
   tightest descending order available and it is the one that reaches the
   green case. Its price is the 0.0114 band. The one rule that widens the
   band, D, halves the pool and drops the green case out of it entirely.

`docs/work/REPORT-1c7d40.md` is on `madgab-slotorder-1c7d40` with the
measurement that produced it. `main` and `post-milestone-acceptance` are
untouched; no self-merge. The next action is a coordinator pass over §7.2.
