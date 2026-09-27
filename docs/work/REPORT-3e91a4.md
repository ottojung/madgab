---
work_item: w-3e91a4
state: produced
docs_only: true
branch: madgab-struct-3e91a4
base: 6e1dc24 (post-milestone-acceptance, pushed)
verdict: HOLD
updated: 2026-09-27T21:05:00Z
---

# REPORT-3e91a4 — shape 1 priced: a structural / coverage / rank-band admission cut is
# refuted by the cap fence itself, not by any property of the key

**Front:** pricing only. No `cargo` invocation of any kind, no test, no benchmark, no
enumeration, no probe, no edit to `src/`, `tests/`, `examples/` or `Cargo.toml`. Every
coordinate below is **cited, not re-derived**; the only new arithmetic is the decision-theoretic
refutation in §2, which is a statement about the walk's own admission test and needs no run.

**Answer, in one line:** no such property can admit the needed class at `cap = 7` at depth 5,
because *every* admission rule of any shape whatsoever admits only indices `< cap` — the
canonical's per-slot indices are `7 / 0 / 13 / 99 / 11`, so **4 of its 5 words are outside the
opening width under every possible rule**, and the least of the five misses is index `7` in
slot 0, which alone needs `F = 11,329` against a pop limit of `4,000` (**2.83×**,
[REPORT-9e2b41.md](REPORT-9e2b41.md) §4). The coordinate is a priced negative.

---

## 1. The candidate property, stated in one sentence

> For each slot, admit at the traversal's opening width the `cap`
> candidates whose joint structural signature — phonetic-feature coverage and
> within-slot distinctness over the span's own candidate set, read as a *set* property and not
> as any single scalar — best spans the span's shortlist, so that no retained word is
> unreachable-by-omission rather than unreachable-by-cost.

This is a function of the span, its slots and its candidate set only: no `hid`, `dupe`, `came`,
`hits`, `justice`, `wreck`, `beach`, `recognize`, no literal input, no target, no clue. It is
the property [w-5e2d41.md](items/w-5e2d41.md) front A named in A6 and front B named in §4.1,
and the map's §3 shape 1.

## 2. The arithmetic: the refutation is a containment, not a percentile

### 2.1 Where a band would have to be applied

The stage-4 admission test is one `match` on one line, inside the pop loop:

* `src/lib.rs:2238-2241` — `match admit_sets.as_deref() { Some(sets) => admit.extend_from_slice(&sets[k]), None => admit.extend(0..slots[k].len().min(cap)) }`.
* `src/lib.rs:2242-2257` — the admitted indices are pushed, subject only to
  `slot_is_affordable(committed, later_min_cost[k], cost, total_budget)`.
* `src/lib.rs:2276` — the same prefix rule on the replay/widening path
  (`for i in opened..slots[k].len().min(cap)`).

Two distinct placements are possible, and they are the two readings of the map's wording:

* **after truncation** — a filter over the *first `cap` ranks*, which is the literal reading of
  §3 shape 1 and A6's first clause;
* **before truncation** — a rule that chooses *which* `cap` of the `n`-wide
  `slots[k]` (built and sorted by `SlotAlt::contribution` at `src/lib.rs:1789-1822`, drawn
  from the shortlist built at `src/lib.rs:1370-1392`) enter the opening width, i.e. a
  `slot_probe::admitted_sets`-shaped rule, which is the shape `REPORT-5d9c04` priced.

### 2.2 The `cap` fence is a property of the walk, not of the rule

Admission by *any* set requires `c_k >= t_k + 1`, because the walk's test is `index < cap`
([REPORT-9e2b41.md](REPORT-9e2b41.md) §4, from the code above), and

```
cap = affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT = 4_000)
    = 10 at d <= 4,  7 at d = 5,  5 at d = 6          (w-5e2d41 §4.1; map §3 shape 1)
```

The canonical case-2 alignment is a **5-slot** alignment (row 12 / `w-0f3a17`; and per
`REPORT-c3f81b1` the depth cap was not the reason: all 34 four-deep tuples on case 2 come from
five-slot segmentations). So the governing cap is **7**, and the canonical's per-slot traversal
indices are:

| slot | needed index | `cap` at d=5 | shortfall |
|---|---|---|---|
| 0 | **7** | 7 | 1 index (needs `cap >= 8`) |
| 1 | 0 | 7 | inside |
| 2 | **13** | 7 | 6 |
| 3 | **99** | 7 | 92 |
| 4 | **11** | 7 | 4 |

*(traversal-index units, schedule ranks 151 and 187: `7 / 0 / 13 / 99 / 11` and
`7 / 0 / 22 / 99 / 11` — [REPORT-1c7d40.md](REPORT-1c7d40.md) §1, [REPORT-5d9c04.md](REPORT-5d9c04.md) §1,
re-derived identically by four fronts; map §1.)*

**The argument.** Let `A` be the admitted set of slot `k` under *any* rule `R` — cost,
`contribution`, median-straddle, coverage, distinctness, rank-band, or a private oracle.
`A ⊆ {0, …, cap-1}` always, because the only loop that can push a child index is
`src/lib.rs:2242-2257` and its source is `admit`, and `admit ⊆ 0..min(cap, n)` in both match
arms. The union over *all* rules `R` of their admitted sets is therefore exactly
`{0, …, cap-1}` — the shipped prefix. The canonical requires indices `7, 13, 99, 11`, none of
which is in `{0..6}`. Hence

> **reach(canonical) = 0 for every admission rule of every shape, at `cap = 7`, d = 5.**

This is a decision-theoretic containment: it does not need a percentile, a distribution, a key
width, or a run. It is also why the map's *other* recorded numbers do not have to be
re-litigated here — `REPORT-5d9c04` §3's 0.25-from-the-median and `REPORT-1c7d40` §7.2's 0.0114
key band are about *where inside a window* a needed word lands; this argument is about the
window's **outer edge**, and it holds for a window chosen with perfect foresight of the answer.

### 2.3 The cheapest miss, priced

The most favourable case for any rule is that it gets slot 0 right and misses the rest: that
still needs `c_0 >= 8`, i.e. caps `(8, 1, 14, 100, 12)`,
`F = 1 + 8 + 8 + 112 + 11,200 = 11,329` against `4,000` — **shortfall 7,329 pops = 2.83×**
([REPORT-9e2b41.md](REPORT-9e2b41.md) §4), and the live cost is the frontier, not the pushes:
widening a slot to reach the tuple is `[160,7,7,7,93]`, `F = 64,001` vs `2,801` = **22.8×**
resident heap, which **OOMs (SIGKILL)** in the measured wiring (same report §5). No key, and no
set of keys, moves a number here.

### 2.4 The after-truncation reading is strictly worse than the shipped rule

If the band is applied *after* truncation (a filter on the first `cap` ranks), then
`A ⊆ shipped prefix` and `|A| <= cap` with equality only for the identity filter. So the
post-truncation reading of shape 1:

* **cannot** reach the tuple (by §2.2, since it admits a subset of what the prefix already
  admits, and the prefix is already short of it); and
* can only **remove** candidates from the 7 the walk would otherwise have expanded, which
  converts a truncated-tight set into a wider one in the walk's own key — exactly the measured
  loss in [REPORT-5d9c04.md](REPORT-5d9c04.md) §4, where the median-straddle (the tightest set
  measured, key width 0.0075) still spends the same pop budget as **4,601 emissions instead of
  12,309**, and three of the four set cuts drop the green case from the pool entirely
  (18,949 → 12,726 / 13,763 / 18,816).

So the after-truncation reading is a strict-improvement-free, pool-losing rule: the cheapest
possible version of shape 1 is the shipped rule, and the shipped rule is row 2's baseline.

### 2.5 The one thing a band property *is* good for

A6's second disjunct — "or **provably not needed** there" — is real and useful, but it is a
**diagnostic, not a reach coordinate**: a coverage/band survey over each span's first `cap`
ranks is a cheap, general, phrase-free *witness* that the needed class is unreachable-by-omission
at this budget, i.e. an audit of the fill (row 4) and the order (row 1) rather than a new cut.
It cannot move reach by the containment above, and it must not be re-filed as one. Note also that
a *retention* band floor of that kind is already the standing successor rule of the fill front
(map row 4's closing cell: "a **rank/band** floor, not a cheap-end cost-ratio one"), and that
`w-5e2d41` front B §4.1 already records the width half as the binding one.

## 3. Conflicts, both numbers kept, none decisive here

Per the item, nothing is reconciled. These are the numbers the argument *could* have used and
does not need, because §2.2 is a containment that survives all of them:

1. **`hid` position 99 vs 119 vs 136.** 99 (traversal index, `REPORT-1c7d40` §1 /
   `REPORT-5d9c04` §1, integrated HEAD) vs 119 (slot rank at the `span_shortlists` boundary,
   `REPORT-9e2b41` §2.3 and `w-5e2d41` front B §2.3, head `a8a8f70`) vs 136 (slot 0 of span
   `[0,2)` on base `3e821c3`). Under 99 the miss is 92 indices = **14.1×** `cap`; under 119 it is
   112 = **17.0×**; under 136, 129 = **19.4×**. All three are the same verdict.
2. **Item-summary indices `7/0/3/85/7`** (shortlist units, `REPORT-9e2b41` §1) versus the
   traversal-unit `7/0/13/99/11`. Under the summary, three of five are still outside `cap = 7`
   (7 by one, 85 by 78, and 3 and 7 are inside/at the edge), so the reach verdict is again the
   same — but note the summary reading is the *most* favourable one and it still needs `c_0 >= 8`.
3. **Depth-5 per-slot affordable width at slot 1: 10, not 9 or 7** (`REPORT-9e2b41` §6; the
   printed `9/7` sequence decreases and contradicts its own monotonicity test). The correct
   reading is `9 / 10 / 10 / 10 / 160`, whose `d=5` cost is 2,801.
4. **Depth-6 last-slot cost 3,906, not 2,801** (`1+5+25+125+625+3125`; `REPORT-9e2b41` §6).
5. **Case-2 pool size 18,949 (integrated HEAD) vs 18,936 (base `3e821c3`) vs 18,933 (head
   `a8a8f70`)**. Bit-reproducible on their own heads.
6. **"Not in the pool" vs "ranks out."** `w-9c6f2b` records that of 1,545,903 enumerated
   five-word alignments, 40,546 contain `dupe` and the best is rank 130 — the candidate
   *lattice* is not the production *pool*. Both are true about different objects. Shape 1 does
   not depend on either: the production pool is a downstream surface, and §2.2 bounds the
   traversal, which is upstream of both.

## 4. Re-specification check against the map's 13 rows

| # | row | verdict for a band/coverage/rank-band cut | why |
|---|---|---|---|
| 1 | per-slot **order** | **duplicates it** if applied before truncation | Choosing which 7 of 160 enter the opening width by a set property is a *selection*, and selection of the same 7 is row 1's `contribution` sort. Row 1 priced 16 general keys and 4 full-pipeline orders, 0/20 reach, best mean position 24.70 against a cap of 7 — and §2.2 shows the *reachable set* is `{0..6}` regardless. |
| 2 | per-slot **set** | **duplicates it** if applied before truncation; strictly worse if after | Same shape as row 2's four cuts, only with a set-level key instead of a scalar. Row 2's refutation is the `cap/n = 0.0219` percentile-window arithmetic plus the 0/4 reach; §2.2 supersedes it with the stronger containment, which needs no window argument at all. |
| 3 | **width** | **not duplicated, but foreclosed** | The only rule that reaches the tuple widens the slot; that is row 3 and `w-9e2b41`, priced at 2.83× (pops) and 22.8× (frontier, OOM). A band property does not change `F`. |
| 4 | **fill strategy** | adjacent, and this is where the honest residue lives | A band/coverage property over *retained* candidates is the fill's successor rule (map row 4 closing cell). It bounds *what the 160 contains* (row 4) but cannot change *how deep the walk reads* (row 3), so it cannot reach. |
| 5 | **emission bound** | **duplicates it / inherits it** | `STAGE-DRAIN` fires **0** times in the whole run, so the `7 → 40 → 160` widening ladder (`next_branch_stage`, `src/lib.rs:2268`) never begins, and the traversal is allowance-bound (reserve 14-16 of 64, walk spends 48-50). A band property that made the first stage *drain* would be a stage-widening-trigger change: row 5, priced. |
| 6 | **reserve coverage / placement** | adjacent, not this surface | 16 draws of a `4.266e11` grid cover each *slot*, never the *tuple* (`w-c3f81a` residual, `REPORT-4d7c12` §5). That is why the map insists the property be about the first `cap` ranks — and §2.2 shows that restriction is exactly what kills it. Map §3 shape 2 remains open and is a *different* item. |
| 7 | **floor** | **duplicates it** | A rank/band floor on retention is the named successor of the cheap-end floor; `w-5e2d41` priced the cost-ratio family negative on three counts. A *retention* band is row 4/7; an *admission* band is §2.2. |
| 8 | **depth cap** | untouched | `funded_slot_depth` (integrated `c11e90e`) prices the reserve's non-zero-coordinate cap; it does not bound the traversal's per-slot depth. |
| 9 | **shortlist width 160** | untouched | Reaching index 99/119 by width alone needs ≈100 uniform = **14.3×** the current 7, and `1+100+…+100⁴ = 1.01e8` pops vs 4,000 = **25,000×**. |
| 10 | **objective / ranking** | untouched, and not mine | 1,126 clues `>=` the canonical on `PARSIMONY + RESEG + SIMILARITY`; monotone-objective floor 1,127 (`REPORT-3a8c05` §3). `w-9b4a15` / `agent-9b4a153` owns scoring; this front does not opine. |
| 11 | adjacency / one-step substitution | untouched | `w-c1d3a7` @ `9767caf`: enumerated no, ranked no; forcing the clue's own wording leaves its segmentation at rank **151 of 256**. |
| 12 | lattice coverage / emission order | **this is where the shape would otherwise be re-filed** | At full width 160 the traversal emits **2,036,664** wordings of the canonical's own structure, reaches index 159, and still never emits the tuple (factor 1,309 in the 2,666,496,000-wording product). `w-0f3a17` is DO-NOT-RE-TAKE. |
| 13 | worst-case / full-width traversal | **the same fact from the key side** | `contribution` width over a whole 160-wide list: mean 0.0338 / 0.0322; the opening 7 already spans 0.0114, i.e. **34%** of the slot's whole key range; the needed word sits at 0.3435 / 0.1381 of that range. An order limit, not a budget limit. |

**Summary of the check:** in its before-truncation form, shape 1 is a re-specification of rows 1
and 2 with a set-level key; in its after-truncation form it is a strict subset of the shipped
rule and can only lose the pool; the only non-duplicative reading (§2.5) is an audit, not a cut.
No row is decided here.

## 5. Cost estimate in candidate visits

| placement | per-candidate work | visits per slot per segmentation | extra pushes | extra frontier |
|---|---|---|---|---|
| after truncation, filter on the first `cap` | one predicate eval per admitted candidate | `<= 7` (vs 7 today) | **0** | **0** |
| before truncation, coverage/distinctness key over the whole shortlist | one feature eval per candidate, plus one set-level pass | `160` (vs 7) = **23×** the admission work | **0** — `F` is unchanged, so pops and `seen` are unchanged | **0** |
| selecting *more* than `cap`, i.e. actually reaching index 99/119 | — | — | `F(8,1,14,100,12) = 11,329` = **2.83×** over 4,000 | `[160,7,7,7,93]`: `F = 64,001` = **22.8×**, **SIGKILL** measured |

The decisive reading of this table: the only *implementable* variants are the two zero-cost ones,
and §2.2 shows both are reach-null. The property is **cheap and useless** — which is precisely
why it needed pricing rather than a front that runs it.

## 6. Host / hygiene

* No `cargo` command was issued, no test or benchmark run, no enumeration or probe, no
  measurement. The memory fence (two OOM-killed predecessors today, `agent-9b4a153` live in
  `/workspace/madgab-parsimony-9b4a15`) was respected absolutely; that worktree was not touched.
* `git diff --stat 6e1dc24..HEAD` touches `docs/` only. `src/`, `tests/`, `examples/` and
  `Cargo.toml` are untouched.
* The standing red (`approximate_finds_classic_madgab_resegmentation`, red at base) was **not
  chased and not re-pinned**. The standing green fact — `wreck a nice beach` for
  `recognize speech` at **pool rank 27**, score 0.9199502875218423, displayed 26/50 — is the
  thing any future candidate must not cost; §2.4 notes that the after-truncation family
  threatens it (three of row 2's four cuts dropped the pool to 12,726 / 13,763 / 18,816).
* No phrase-specific or token-specific special casing is proposed; the property in §1 is a
  function of the span, its slots and its candidate set only.

## 7. Verdict

**HOLD.** Shape 1 is a **priced negative**, and its refutation is stronger than the reports
behind rows 1 and 2: it is a containment, not a percentile. Because the walk's admission test
is `index < cap` (`src/lib.rs:2238-2241`, and again at `:2276`), *every* admitted set — under any
key, any scalar, any set-level coverage property, any rank band — is a subset of
`{0, …, cap-1}`. At depth 5 `cap = 7`, and the canonical's indices are `7 / 0 / 13 / 99 / 11`:
**4 of 5 words are outside the opening width for structural reasons no key can change**, the
cheapest single miss (index 7 in slot 0) already costs **2.83×** the pop limit, and the version
that would reach it costs **22.8×** in resident frontier and was measured **OOM (SIGKILL)**.
Before truncation the shape duplicates rows 1 and 2; after truncation it is a strict subset of
the shipped rule that can only cost pool (18,949 → 12,726 in the tightest measured case). The
non-duplicative residue — A6's "provably not needed" disjunct — is an audit of the fill and the
order, not a reach coordinate.

**Successor rule, one line:** an admission cut on a structural / coverage / rank-band property
can only ever choose *within* the traversal's opening width, so it is a reach coordinate only if
it changes `cap` or the shortlist *contents* — if a future front wants to reopen this surface it
must price a **retention** band (which 160 are kept, at what rank), not an **admission** band
(which 7 of the kept are walked), and the width it must beat is the 2.83× frontier bound of
`w-9e2b41`, not a percentile.
