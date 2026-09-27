---
work_item: true
id: w-5d9c04
state: done
priority: high
owner: front-5d9c04
updated: 2026-09-27T19:40:00Z
branch: madgab-slotset-5d9c04
worktree: /workspace/madgab-slotset-5d9c04
base: 8d4b193
verdict: HOLD
---

# w-5d9c04 - the per-slot *set* is not a lever either: five general set cuts, all zero on reach, and the arithmetic that kills the family

Front `front-5d9c04`, branch `madgab-slotset-5d9c04`, base `8d4b193` (integrated HEAD).
Release mode, `cargo test --release`, `CARGO_TARGET_DIR=/workspace/target-5d9c04`.
**Outcome: HOLD** - a decomposition of the stage-4 cut into its *set* coordinate, five
general set cuts priced end to end, and a measured refutation of the family. No production
change is proposed.

Reproduce with:

    cargo test --release --lib front_5d9c04 -- --ignored --nocapture

---

## 0. What this front changed, and where it lives

`src/lib.rs` only, **682 insertions and 3 deletions**, of which the 3 deletions are the
opening-width loop header. The stage-4 cut is now

```rust
admit.clear();
match admit_sets.as_deref() {
    Some(sets) => admit.extend_from_slice(&sets[k]),
    None => admit.extend(0..slots[k].len().min(cap)),
}
for &i in &admit {
```

where `admit_sets` is `None` in **every release build** and in every test that has not
armed a set rule. The `None` arm is the shipped `index < cap` verbatim, so the release
walk is behaviour-identical to base - confirmed by §1's reproduction of every base number
to the digit, and by `cargo build --release` with no warnings. The one cost the release
path takes is a `clear` plus an `extend` of at most `cap` indices per pop.

Everything else is `#[cfg(test)]`: the `SlotCut` enum, the `admitted` rule, the
`admitted_sets` arming hook, and the `front_5d9c04` module, which names the two canonical
examples as *data*. The per-slot *order* apparatus of `w-1c7d40` is re-used as it stands
(`slot_probe`, `Seg`, `Emit`, `carries`-style fund location); no parallel probe was built
and nothing is measured twice. The widening stage's own `opened..` loop is deliberately
left alone - it governs the *widening* pass, not the opening width, and widening does not
happen on these targets anyway (`w-9d4e17`).

No score weight, no axis definition, no `Metrics` field, no `SPARSE_SHORTLIST`, no depth
cap, no `LEXICAL_HEAP_POP_LIMIT`, no emission budget, no baseline, no `examples/`, no
`ZZ_*` probe and no environment knob is touched. `cargo fmt --check` and `cargo clippy`
cannot run on this host (see [../environment-notes.md](../environment-notes.md)) and are
**not claimed**.

---

## 1. Obligation 1: the stage, the opening width and the needed indices, re-derived

`Generator::generate_pool`, `SearchMode::approximate()`, `top_n = 50`, `beam_width = 64`,
release, shipped prefix cut:

```
affordable_opening_width(5, 4000) = 7
case 2: pool 18949 reach 0 reserve 3051 traversal 12309
case 1: pool 18289 rank Some((27, 0.9199502875218423))
  case 2 rank 151 widths [160, 7, 160, 160, 93] cap 7 -> indices [7! 0 13! 99! 11!]  partial
  case 2 rank 187 widths [160, 2, 160, 160, 93] cap 7 -> indices [7! 0 22! 99! 11!]  partial
  case 1: 14 funds carry the multiset, 4 of them fully inside the cap (42/56 observations)
```

Every one of these is a `REPORT-1c7d40` §1 and `REPORT-4d7c12` §1 number, reproduced
exactly: **7**, **`7 / 0 / 13 / 99 / 11`** at schedule rank 151, **`7 / 0 / 22 / 99 / 11`**
at 187, pool **18,949** / **18,289**, reserve 3,051, traversal 12,309, and
`wreck a nice beach` for `recognize speech` at pool rank **27**, score **0.9199502875**.
The base is confirmed, so every number below is a base number. The `!` marks a needed
index the rule does not admit.

---

## 2. Obligation 2: five set cuts, priced end to end

Four general set rules plus the shipped prefix, each re-run through the whole pipeline
with the per-slot *order* held at the shipped one so that the set is the only thing that
differs from the baseline. `rch2` is whether the 18,949-member pool holds a member whose
word multiset is the canonical one.

| cut | rule | case-2 reach | pool 2 | wall 2 | case-1 reach | rank 1 | score 1 | pool 1 | wall 1 | reserve / traversal |
|---|---|---|---|---|---|---|---|---|---|---|
| **prefix (shipped)** | `index < cap` | **0** | **18,949** | **987-1,067 ms** | 1 | **27** | 0.9199502875 | 18,289 | 992-1,082 ms | 3,051 / 12,309 |
| **median-straddle** | `cap` candidates in a contiguous cost window centred on the slot's median cost rank | **0** | **12,726** | 1,252-1,448 ms | **0** | **—** | — | 13,347 | 1,317-1,484 ms | 3,279 / **4,601** |
| **mean-straddle** | the same window centred on the slot's mean cost | **0** | 13,763 | 1,309-1,600 ms | **0** | — | — | 15,357 | 1,220-1,337 ms | 3,279 / 6,099 |
| **quantile** | the `cap` candidates at evenly spaced cost quantiles | **0** | 18,816 | 1,198-1,316 ms | **0** | — | — | 18,709 | 1,010-1,416 ms | 3,279 / 9,296 |
| **union top-cost + top-contrib** | `cap` cheapest-by-cost ∪ `cap` best-by-contribution | **0** | 18,985 | 1,383-1,591 ms | 1 | **27** | 0.9199502875 | 18,374 | 1,404-1,570 ms | 3,279 / 11,988 |

Three runs of the pricing probe, release, same target dir; pools, ranks and spends are
bit-identical across runs and only the wall clock moves. Against the item's base figures
of **1,025 ms** (case 2) and **1,271 ms**: the prefix row reproduces 1,025 ms, and no
variant is slower than about +60 %. This is not a wall-clock question either way.

**Reach of the canonical tuple is 0 in all four set rules**, and so it is in the sixteen
keys and four orders of `REPORT-1c7d40` §3 - **24 priced configurations, none of them
reaching**. Three of the four set rules additionally **lose the green case from the pool
outright**: `wreck a nice beach` for `recognize speech` is in the baseline pool at rank
27 and is **absent** from the median, mean and quantile pools, so those three regress the
one blocker that is *not* red. The item's constraint that the green case stay inside the
top 50 is satisfied by the prefix and the union alone, and the union is the row that buys
its set with a doubled opening width (§4).

### 2.1 What each set admits of the tuple's own words

Re-deriving the sets from the traversal's captured slot lists with the *same* `admitted`
the walk used, over every fund that carries the tuple's multiset:

| cut | case 2: funds fully admitted / observations | case 1: funds fully admitted / observations |
|---|---|---|
| prefix (shipped) | **0/2**, 2/10 | 4/14, 42/56 |
| median-straddle | **0/2**, 2/10 | **0/14**, 7/56 |
| mean-straddle | **0/2**, 2/10 | **0/14**, 8/56 |
| quantile | **0/2**, 2/10 | **0/14**, 20/56 |
| union (2·cap) | **0/2**, 3/10 | 7/14, 47/56 |

On case 2 the only observations any cost rule admits are the two words that sit on the
7-wide and 2-wide slots, where the prefix already had them. **Every rule admits none of
the needed words on any 160-wide slot** - the ones at indices 7, 13, 22, 99 and 11. The
union, at twice the width, gets exactly one more: `dupe` at index 13 of a 160-wide slot
at rank 151. The quantile rule, which deliberately spreads itself over the whole cost
range, admits 20 of 56 case-1 observations and still **0 of 14 funds**, because the words
are in different slots and a fund needs all of them.

---

## 3. Obligation 3: the property the prefix cut gets wrong, and its measurement

**There is no such property, and the set family is refuted by an arithmetic that holds
before any of it is run.** Two measurements decide it.

**First, the needed words are not near any centre a rule can name.** Their cost percentile
in their own slot's cost distribution, against the two centres a general rule can use,
and against the largest distance a `cap`-wide window can reach:

| quantity | case 2 | case 1 |
|---|---|---|
| needed word's distance from the **median** cost percentile, mean / max | **0.2716 / 0.5000** | 0.2545 / 0.5000 |
| needed word's distance from the **mean** cost percentile, mean / max | 0.2836 / 0.6596 | 0.3528 / 0.7876 |
| needed words on a slot of width ≥ 100 lying within `cap/(2n)` of the median | **0/6** | **0/41** |
| … of the mean cost percentile | **0/6** | 2/41 |

**Second, `cap/(2n)` is 0.0219 (case 2, 7 of 160) and 0.0313 (case 1, 10 of 160).** A
`cap`-wide window in a 160-wide slot spans **4.4 % of that slot's cost range**, and half
of it reaches 2.2 %. The needed words sit a mean of **0.25** away from the median. They
are **an order of magnitude outside** anything a cost window of the affordable width can
reach, on either centre, by a factor of 8 to 12.

That is why the required rule, the median-straddle, is a *miss by construction* and not a
miss of placement: `REPORT-1c7d40` §2.2's motivating measurement — needed words at cost
percentile 0.43 (case 2) and 0.38 (case 1) against the `cap`-th candidate's 0.50 and 0.43
— puts them **0.07 to 0.12** past a median cut, and the window is **0.044** wide. The
parent's own framing, that they are "just on the wrong side of a median cut", is true
about the *side* and off by a factor of two to three about the *margin*: a band 0.07-0.12
wide needs a width of 12-20 candidates at 160, against an affordable 7.

The refutation generalises past the four rules priced. **Any** rule that admits exactly
`cap` of `n` candidates on the basis of a single scalar per-candidate cost must place
its whole set inside a percentile window of width `cap/n`, and for every placement of
that window the needed words are outside. Cost is the axis on which they sit 0.25 from
the centre; `REPORT-1c7d40` §3 priced the other fifteen per-candidate keys that exist and
none landed a fund inside the cap either. So the set coordinate is not merely unpromising
at 7 of 160 - it is closed at any width the pop limit affords, and it closes in the same
place `REPORT-4d7c12` §5 and `REPORT-1c7d40` §7.2 closed coverage and width: three to
five orders of magnitude.

**What the prefix cut gets wrong is not its *set*, it is its *width*, and that is the
coordinate this front prices as closed.** The prefix set is the tightest descending set
available (§4) and it already contains the green case's words at 4 of 14 carrying funds.
Its defect is that it is 7 candidates wide against a 160-wide slot, and no selection of 7
out of 160 - by cost, by contribution, or by any mix of the two - can contain a word at
the 27th percentile of that slot. Widening is the only remaining direction and the item
forbids buying it; `w-9e2b41` owns the width allocation.

---

## 4. Obligation 4: the descending-bound loss, priced

The heap key is `bound(prefix)`, an upper bound on the score of a completion computed
from the prefix and the suffix's extremums, and it **does not read the slot order or the
admitted set**. So the descending-bound property of the *emission order* is preserved by
construction under every rule here, and it is not what these rules lose. What the set
decides, and what was measured over a full case-2 run:

| cut | traversal emissions inside the `cap` prefix / outside it | distinct slot-0 indices reached | `contribution` band across the admitted set, mean | traversal emissions |
|---|---|---|---|---|
| **prefix (shipped)** | **12,431 / 0** | **10** | **0.011416** | **12,309** |
| median-straddle | 338 / 3,124 | **80** | **0.007522** | 4,601 |
| mean-straddle | 205 / 4,435 | 56 | 0.012863 | 6,099 |
| quantile | 523 / 9,014 | 73 | 0.016662 | 9,296 |
| union (2·cap) | 11,320 / 1,047 | 23 | 0.017411 | 11,988 |

* **The property is preserved and it is not the cost.** The walk's key is untouched, so
  the emission order stays descending in final score. What changes is the *narrowness* of
  the set the key has to work inside, and the two rows say it precisely.
* **The median-straddle is a *tighter* set and a much worse walk.** Its band across the
  admitted set is 0.0075 against the prefix's 0.0114 — the best number in the table, and
  it buys nothing. Because the set is no longer sorted by the key, the heap's best-first
  order inside the opening width degrades: the walk reaches **80** distinct slot-0
  indices instead of **10**, and turns the same pop budget into **4,601** emissions
  instead of 12,309. The pool follows, 18,949 → **12,726** (−33 %), and the green case
  leaves the pool entirely. This is the same arithmetic `REPORT-1c7d40` §3.1's candidate D
  found for an inverted order and `REPORT-4d7c12` §4 found for a joint reserve: the
  marginal, cheap-end emissions are what populate the pool, and a set that is not
  key-descending displaces exactly those.
* **The quantile rule is the informative negative.** Its band is the *widest* in the
  table (0.0167 against 0.0114) and its pool is the closest to base (18,816 against
  18,949) — the set is nearly inert on the pool — and it still loses the green case and
  reaches nothing. Pool preservation and reach are not on the same axis.
* **The union is the only rule that keeps the green case, and it pays for it in width.**
  It is the shipped prefix set ∪ the `cap` cheapest-by-cost, so it admits up to **2·cap**
  (measured 8 to 18 candidates per slot against an affordable 7 to 10). **A variant that only works
  by spending more is a priced negative in its own right**, and here it does not even
  work by spending more: reach is still 0 with the width doubled, wall clock is +40 %,
  the reserve's spend is 7.5 % up, and 1,047 emissions now sit outside the affordable
  width. No width, budget, depth cap, pop limit or `SPARSE_SHORTLIST` value is raised by
  any row; the union's extra width is inside the *same* `LEXICAL_HEAP_POP_LIMIT` and the
  same 16,384-emission ceiling.

**In what respect the stated benefit is preserved, weakened or lost:** preserved, the
bound and therefore the descending emission order; weakened, the tightness of the set the
bound sorts within - 0.0075 to 0.0167 against 0.0114, i.e. up to 46 % wider for three of
the four rules, and 0.0075 for the one that is tighter but pays for it with an 8×-wider
walk; lost, the guarantee that **every** traversal emission lies inside the opening
width, which the shipped rule has exactly (12,431 / 0) and which no set rule keeps. That
guarantee is worth a third of the pool and the green case, so it is the thing a successor
should not give up.

---

## 5. Obligation 5: the scoring caveat, cited and not re-derived

[REPORT-3a8c05.md](REPORT-3a8c05.md) §3 is a hard monotone lower bound and it is
inherited here untouched: **1,126 pool clues are `>=` the canonical on
`PARSIMONY + RESEG + SIMILARITY` at once, so no monotone objective over any measurable
per-candidate property can rank it better than 1,127, and its shipped score is
0.0943520415 below the top-50 cutoff.** That table is not re-derived, not duplicated and
not re-measured. It is the reason this front's success criterion is **reach** and not a
display: even a perfect search would not put this tuple in the top 50, so the only thing a
search-side coordinate can move is whether the pool holds it at all.

Measured and reported as they are, not fixed: the canonical is **absent from the pool
under all five cuts**, so it has no pool rank and no score to report; the green case's
rank and score are **27 and 0.9199502875** wherever it is in the pool at all, and it is
**out of the pool** under the median, mean and quantile cuts. No score weight, no axis
definition and no `Metrics` field is touched by this branch; the scoring surface belongs
to `w-9b4a15`.

---

## 6. The counterfactual table

| what was priced | configurations | case-2 reach | funds fully admitted (case 2) | green rank | verdict |
|---|---|---|---|---|---|
| per-candidate keys (`REPORT-1c7d40` §3) | 16 | 0/16 | 0/2 funds under every key | 27 where re-run | refuted |
| full-pipeline orders (`REPORT-1c7d40` §3.1) | 4 | 0/4 | — | 27, 27, 27, — | refuted |
| **per-slot set cuts (this front)** | **4** | **0/4** | **0/2 funds under every rule** | 27, —, —, —, 27 | **refuted** |
| **total** | **24** | **0/24** | | | |

(Plus the shipped prefix row, which is the baseline rather than a variant, and three
`#[ignore]`d probes that price no rule.)

The mechanism is the same in all twenty-four: the needed words sit a mean of 0.25 in cost
percentile from their own slot's centre on lists 100-160 wide, an affordable `cap` of 7-10
reaches 0.022-0.031, and no selection of that size on any per-candidate key contains
them.

---

## 7. Tree state

`CARGO_TARGET_DIR=/workspace/target-5d9c04`, release throughout:

| target | result |
|---|---|
| `cargo build --release` | clean, **no warnings** |
| `cargo test --release --lib` | **75 passed / 0 failed / 9 ignored** (26.6 s) — 74 is the base count, +1 is this front's new unit test; the 9 ignored are the 6 of `w-1c7d40` and 3 of this front |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed / 0 failed** |
| `cargo test --release --test corpus_integration -- --test-threads=2` | **12 passed / 1 failed** — `approximate_finds_classic_madgab_resegmentation`, the known pre-existing red at base, **not re-pinned and not made worse** |
| `cargo test --release --test emit_coverage` | **4 passed / 0 failed** |
| `cargo test --release --test exact_determinism` | **1 passed / 0 failed** |
| `cargo test --release --test approx_determinism` | **4 passed / 0 failed** |

No example token appears in `src/` outside a `#[cfg(test)]` module: `hid`, `dupe`,
`came`, `hits`, `justice`, `wreck`, `beach` and `recognize` occur only in the constants of
`mod front_1c7d40` and `mod front_5d9c04`, which is what the 9/9 is there to enforce.

---

## 8. Verdict

**HOLD.** Five things are established and none of them is a production change.

1. **The stage re-derives exactly on integrated HEAD**: opening width 7 at depth 5,
   needed indices `7 / 0 / 13 / 99 / 11` at rank 151 and `7 / 0 / 22 / 99 / 11` at 187,
   pool 18,949 / 18,289, green rank 27 at 0.9199502875, reserve 3,051, traversal 12,309.
2. **The per-slot set is not a lever.** Four general set cuts plus the prefix, priced end
   to end, 24 configurations across both this front and its parent, **none reaching**.
   Three of the four lose the green case from the pool entirely; the one that keeps it
   (the top-cost ∪ top-contribution union) is a doubled opening width, so it is a priced
   negative on spend as well as on reach.
3. **The decisive statement is the arithmetic, not the table.** A `cap`-wide set cut on a
   160-wide slot spans 0.044 of that slot's cost range and reaches 0.022; the needed words
   sit 0.25 from the median and 0.28 from the mean of their own slot. **No** cost-keyed set
   cut of the affordable width can contain them, on any centre and at any placement, so
   the family is closed at width 7 rather than merely unpromising. What the prefix cut
   gets wrong is its **width**, not its **set** — and the width is the coordinate
   `w-9e2b41` owns and the item forbids buying.
4. **The descending bound is not the cost, but its narrowness is.** The bound and the
   descending emission order are untouched by construction; what the set rules lose is the
   tightness of the set the bound sorts within (up to 46 % wider) and the guarantee that
   every emission is inside the opening width (12,431/0 shipped, 338/3,124 under the
   median-straddle). The median-straddle is the tightest set measured (0.0075) and it
   costs a third of the pool, because a set that is not key-descending turns the same pop
   budget into 4,601 emissions instead of 12,309.
5. **The ranking caveat stands and was not chased**: 1,126 clues dominate the canonical on
   `PARSIMONY + RESEG + SIMILARITY` at once (`REPORT-3a8c05.md` §3), so reach is the only
   thing a search coordinate can move, and the canonical is still absent from the pool
   under every rule.

`docs/work/REPORT-5d9c04.md` and the measurement that produced it are on
`madgab-slotset-5d9c04`. `main` and `post-milestone-acceptance` are untouched; no
self-merge.

**The next coordinate, named precisely enough to start from.** Not the set and not the
order - the set is closed by the `cap/(2n)` arithmetic above and the order is closed by
`REPORT-1c7d40` §3. What survives is the third thing §2.2 of that report measured and
this front confirms from the other side: **the affordable width itself is 7 because
`affordable_opening_width` derives it under a uniform scalar law
`1 + w + ... + w^(d-1) <= LEXICAL_HEAP_POP_LIMIT`, while the walk's actual per-slot
marginal cost is the product of the widths above it.** The cut is not wrong about *which*
candidates it takes at 7; it is wrong that 7 is the right number for every slot. The
coordinate is a derived per-slot width vector, and it is `w-9e2b41`'s to take — this front
takes nothing, changes nothing and hands the measurement on. If a successor wants to
re-open the case-2 tuple from the search side after that, the only untried shape left is a
cut that is not a function of a single scalar cost at all — and the sixteen keys of
`REPORT-1c7d40` §3 are the evidence that there is nothing left there.
