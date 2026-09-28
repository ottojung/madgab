---
work_item: w-2c9d41
state: done  # back-reference only; the work item is docs/work/items/w-2c9d41.md, which carries `work_item: true` and the canonical state. Not in the rule-34 discovery population. `produced` is not a state in skills/work-items.md; corrected by coord-6f4a.
docs_only: true
base: dd94988 (post-milestone-acceptance, pushed)
branch: madgab-obstruction-2c9d41
canonical_case_2: "It's just a stupid game" -> "Hits Justice Dupe Hid Came"
canonical_case_1_green: "recognize speech" -> "wreck a nice beach" at pool rank 27, displayed 26/50
updated: 2026-09-27T19:40:00Z
---

# OBSTRUCTION-MAP — which case-2 coordinates are already priced, and by what measurement

**This is a roll-up, not a verdict.** Every row records a decision some front has already made and
cites the report and section that priced it. Nothing here re-runs, re-derives or re-adjudicates a
coordinate. If a row is CLOSED and you think it should be open, the correct move is a *new* work
item that names a shape of fix not in §2, not a fresh run of the row's own coordinate — that is the
failure mode this document exists to stop (`w-1c7d40` "do not re-run it", `w-7b40d2` /
`w-5b1e93` superseded so they stop inviting duplicate fronts, `w-0f3a17` DO-NOT-RE-TAKE).

Scope: the stage-4 cut of the approximate search, and the surfaces around it. Base
`post-milestone-acceptance` at `dd94988`. Read [items/w-2c9d41.md](items/w-2c9d41.md) for the
obligations this satisfies.

---

## 1. The shared arithmetic every row rests on

One localisation, re-derived independently by four fronts on integrated HEAD, so it may be cited
without re-running:

| quantity | case 2 | source |
|---|---|---|
| opening width, `affordable_opening_width(5, 4000)` | **7** | [REPORT-4d7c12.md](REPORT-4d7c12.md) §1; [REPORT-1c7d40.md](REPORT-1c7d40.md) §1; [REPORT-5d9c04.md](REPORT-5d9c04.md) §1 |
| canonical per-slot traversal indices, schedule rank 151 | **`7 / 0 / 13 / 99 / 11`** | same three, all identical |
| canonical per-slot traversal indices, schedule rank 187 | **`7 / 0 / 22 / 99 / 11`** | same three |
| slot widths at the decisive fund | `[160, 7, 160, 160, 93]` | REPORT-4d7c12 §1, REPORT-5d9c04 §1 |
| joint index space at that fund | `160 x 7 x 160 x 160 x 93` = **2,666,496,000 cells** | REPORT-4d7c12 §5; REPORT-1c7d40 §7.2 |
| case-2 pool / case-1 pool | **18,949 / 18,289** | REPORT-1c7d40 §1, REPORT-5d9c04 §1 |
| case-2 reach | **0** — the canonical multiset is absent from the pool under every priced rule | REPORT-1c7d40 §8, REPORT-5d9c04 §8 |

**A conflict on record, both numbers kept.** [REPORT-9e2b41.md](REPORT-9e2b41.md) §0-§1 reports the
same tuple's indices as `7 / 0 / 13 / 99 / 11` but reports `hid` at **rank 119** in its own
shortlist-slot frame (and 136 on an older base), whereas REPORT-1c7d40 / REPORT-5d9c04 report
**99** in traversal-index units. Both are recorded rather than reconciled: 99 is the traversal index
inside the 160-wide `slots[k]` list, 119/136 are ranks at the `span_shortlists` API boundary on a
different head. The disagreement does not change any verdict, because every width and set rule is
refuted at *any* of these values. Do not treat either as wrong without measuring.

**The one standing blocker, and it is not to be re-pinned.**
`approximate_finds_classic_madgab_resegmentation` (case 2) is **red at base** and red at every head in
every report cited here. It is the pre-existing failure in `--test corpus_integration` (12 passed / 1
failed) and must never be re-pinned or made green by accident. A single word changing pool presence
is a *different* guard (`a_lattice_alignment_can_be_absent_from_the_production_pool`, `w-5e2d41`),
also currently red on the parked fill branch and green on integrated HEAD.

**The one standing green fact.** `wreck a nice beach` is produced for `recognize speech` at **pool
rank 27** (score **0.9199502875218423**), displayed 26/50, on integrated HEAD. It is green on every
rule in the tables below *unless* the rule regresses it, which three of the four set cuts do — so any
future candidate must report this rank, not just reach.

---

## 2. The priced-coordinate table

Status key: **CLOSED** = a front priced this coordinate and it failed (a priced negative, i.e. a
durable result, not a gap); **closed-by-constraint** = no rule of this shape can work at the shipped
budget; **OPEN** = named, not yet priced end to end.

| # | coordinate | priced by | decisive number(s) | status | the constraint that closed it |
|---|---|---|---|---|---|
| 1 | **Per-slot order** — can `slots[k]` be re-specified so the needed words land inside the opening width? | [REPORT-1c7d40.md](REPORT-1c7d40.md) §2, §3, §3.1, §7.3 (item [w-1c7d40](items/w-1c7d40.md), `done`) | 16 general keys, 4 full-pipeline orders, **0/20 reach**; best mean position **24.70** (key Q) against a cap of 7; needed words at cost percentile **0.43** (case 2) / **0.38** (case 1) vs the `cap`-th candidate's **0.50** / **0.43**; the opening width is a **0.0114**-wide band of the key, and the needed word sits at **34%** of the whole 160-wide list's key range | **CLOSED** | There is no per-candidate measurement that separates a needed word from the candidates ahead of it. The motivating hypothesis (a faithful resegmentation needs an *expensive* word and the cheap-first order buries it) is **measured to be backwards**: the needed words are *cheaper* than their slot's median. No sub-expression of `contribution` to drop — key P (cost term removed) has the highest `need contr` at 0.9316 and still reaches nothing at mean position 60.10. |
| 2 | **Per-slot admitted set** — can the *set* of 7 admitted per slot be chosen by something other than `index < cap`? | [REPORT-5d9c04.md](REPORT-5d9c04.md) §2, §2.1, §3, §6 (item [w-5d9c04](items/w-5d9c04.md)) | 4 general set cuts (median-straddle, mean-straddle, quantile, top-cost ∪ top-contribution) + the shipped prefix: **0/4 reach**, **0/2 funds fully admitted under every rule**; a `cap`-wide window on a 160-wide slot spans **0.044** of the slot's cost range and half of it reaches **0.0219** (= `cap/(2n)`, case 2) / **0.0313** (case 1), while the needed words sit a mean of **0.25** from the median and **0.28** from the mean of their own slot | **CLOSED at width 7, by arithmetic** | Any rule admitting exactly `cap` of `n` candidates on a single scalar per-candidate cost must place its whole set in a percentile window of width `cap/n`; for every placement of that window the needed words are outside, by a factor of **8-12**. Cost is the axis on which they sit 0.25 from the centre; the other fifteen keys (row 1) are the evidence that nothing else works either. Three of the four cuts also **lose the green case from the pool** (18,949 → 12,726 / 13,763 / 18,816); the union keeps it only by spending **2·cap** width and still reaches 0. |
| 3 | **Width / opening width** — can the per-slot width be re-allocated under the pop budget? | [REPORT-9e2b41.md](REPORT-9e2b41.md) §0, §2, §4, §5 (item [w-9e2b41](items/w-9e2b41.md), [w-9e2b41-measurement](items/w-9e2b41-measurement.md)) | Minimal allocation admitting the tuple is `c = (8, 1, 14, 100, 12)`; its frontier `F(c) = 1 + Σ Π_{j<k} c_j` = **11,329** against `LEXICAL_HEAP_POP_LIMIT` = 4,000 → shortfall **7,329 pops = 2.83×**; **58,106** jointly affordable allocations exist at depth 4 with pools `[*,*,*,160]`; a rule wired into the traversal opens `[160,7,7,7,93]`, `F = 64,001` vs uniform-7 `F = 2,801` (**22.8×** resident heap) × 256 retained segmentations, and **OOMs (SIGKILL)**; 2 of 3 pool-reach guards red even without the OOM | **CLOSED** | The charged quantity is `F`, the **product** of per-slot prefix widths, charged **jointly and once**; `F(t+1)` is a lower bound over *every* allocation admitting `t`, so no allocation rule of any shape can beat 2.83×. Two further constraints: the pop budget has **no opinion about the per-slot split** (58,106 affordable allocations), so a per-slot width is a *choice*, not a derivation; and the live cost is `F` — **memory and the emission allowance**, not pushes. Raising the limit is arithmetic (≈100-wide uniform ⇒ 1.01e8 pops, ≈25,000×; REPORT-1c7d40 §7.2). |
| 4 | **Fill strategy** — which 160 candidates a span's shortlist retains | item [w-7b40d2](items/w-7b40d2.md) (`superseded` by w-5e2d41), the defect priced in [w-0f3a17-shortlist-rule](items/w-0f3a17-shortlist-rule.md) and the successor [w-5e2d41](items/w-5e2d41.md) (`done`) | Defect: of **7,056** candidates within 1.5× of their span's cheapest over **939** span edges, **623 (8.8%)** are dropped at every width, worst per-span share **24 of 56 (43%)**, worst dropped quality rank 255. The stratified fill on the parked branch `a8a8f70` takes it **623 → 0**, worst share → 0, unrepresented non-empty cost bands **1 → 0** of 3,234, with both cutoffs bit-identical — but turns `a_lattice_alignment_can_be_absent_from_the_production_pool` red on the word `hid` (2 pool wordings → 0) | **CLOSED as a route to case 2** (the fill is *not* integrated; `madgab-fillstrat-7b40d2` stays parked deliberately so the red fence is not re-pinned) | Successor [w-5e2d41](items/w-5e2d41.md) closed it on both fronts: a 1.5× cheap-end floor is **vacuous** (satisfied on 108/108 spans, 0 of 885 spans unsaturated) and cannot reach the needed word anyway (`hid` at cost 0.3500000000 against span `[0,2)` `min_cost` 0.0000000000 — no finite multiple of the minimum reaches it, and the 1.5× band of that span is 1 of 256). Binding a floor anywhere costs band representation (`every_non_empty_cost_band_contributes…` goes red). The parameter-free limit of the family is the global `quality` fill, which re-opens the 623. Successor rule left standing: a **rank/band** floor, not a cheap-end cost-ratio one. |
| 5 | **Emission bound** — is the 16,384-emission ceiling the binding constraint? | item [w-5b1e93](items/w-5b1e93.md) (`superseded` by w-0f3a17), the decisive measurement in [w-5e2d41](items/w-5e2d41.md) front B §4 | At the structure that emits the needed word, **7,312 of 16,384 emissions (55.4%) unspent**; the ceiling is reached only at schedule position **240 of 256**; pop ceiling at **17.2%** (175,957 of 1,024,000); `STAGE-DRAIN` fires **0** times in the whole run, so no widening from 5/7 towards 160 ever begins | **CLOSED as a cause** (not binding) — but the *traversal* is **allowance-bound**: the depth-profile reserve takes 14-16 of `LEXICAL_COMBINATIONS_PER_SEGMENTATION` = 64 before enumeration, and the walk fills exactly the remaining 48-50 | The traversal stops on its **per-segmentation allowance**, not on any global budget; raising any global ceiling does not change how deep into a slot the walk looks, only how many tuples are drawn. `STAGE-DRAIN` = 0 also means the designed widening escape (7 → 28 → 112 → 160) is dead on this target. |
| 6 | **Coverage reserve and tuple placement** — the reserve's sweep as a joint-coordinate cover | item [w-c3f81a](items/w-c3f81a.md) (`done`; report [REPORT-c3f81b1.md](REPORT-c3f81b1.md), independent review) | `sweep_index` places one tuple per subset per phase at a **uniform stride**, i.e. uniform in each *marginal* index with essentially **zero joint** coverage; `EMIT_PROFILE_RESERVE` = 16 draws, `EMIT_PROFILE_MAX_DEEP` = 3. The 16 draws at the decisive structure are **bit-identical** across heads (`[136,0,0,0,0,0]`, `[0,146,0,0,0,0]`, `[0,0,0,10,0,0]`, …); a 16-draw sample of a `160·160·7·160·160·93 = 4.266e11` cell grid is a cover of each *slot*, never of the *tuple*. Depth cap measured: **all 34 four-deep tuples on the case-2 target come from five-slot** segmentations, and the canonical alignment needs five slots (`EMIT_PROFILE_MAX_DEEP = 3` capped the reserve at three non-zero coordinates) | **Mixed: the derived depth-cap bound CLOSED and integrated; the joint-coverage sweep itself is not the fix — and since `38361de` it is priced NEGATIVE as a reach coordinate (REPORT-2f1c03.md §3, reach-null by 5-6 orders of magnitude).** The depth cap was replaced by `funded_slot_depth(...)` at `c11e90e` — the bound was derived and the four-deep placement measured at **zero** emission cost, 16 targets, worst pool `+19` (+0.10%), pops <0.05%. But `w-c3f81a` is **explicitly not** to be cited as clearing the canonical case-2 target, and its residual finding (zero joint coverage in the sweep) is the same product-vs-linear-budget wall row 3 priced from the width side | The sweep is a **per-slot tiling** argument (`stride = span.div_ceil(per)`, rotated start) and says nothing about a 5- or 6-dimensional tuple; the canonical is a single point in a ~150^4 product. `w-c3f81a`'s surface was also shown *not* on the critical path for the `hid` guard: the decisive draw is bit-identical on both heads and only the span's shortlist contents changed underneath it (w-5e2d41 front B §4). **No successor front on the sweep-coverage surface was opened from w-c3f81a; the sweep-coverage question belongs to whichever fill surface is still open.** |
| 7 | **Floor** — a cheap-end retention floor in the shortlist fill | item [w-5e2d41](items/w-5e2d41.md) (`done`, priced negative on both fronts) and its review front [w-5e2d42](items/w-5e2d42.md), report appended to w-5e2d41 | **Vacuous at the item's own threshold**: 1.5× → **0 of 885** spans unsaturated, shortfall 0; 2.0× → 84 spans / 5,379; 3.375× → 165 / 12,785; 8.0× → 269 / 23,004. **Unreachable by arithmetic**: needed rank **119** (base rank 136) against `affordable_opening_width(d, 4000)` = 10 (d≤4), **7** (d=5), 5 (d=6) → **17×/19× past the opening width**. Ladders 1.5×4, 1.4×8, 1.3×10, 1.2×14 leave the fence red; 1.25×12 and 1.1×20 turn it green — **found by scanning against the fence and deliberately not landed** | **CLOSED** | Three independent constraints: the floor is already satisfied everywhere on the cheap end; no finite cheap-end multiple reaches a word at cost 0.35 on a span whose minimum is 0.0; and wherever it does bind it drains the cost range and empties a band (`SPAN_COST_BANDS` band 3 on span `(8,10)`: retained max cost 0.400000 vs offered 0.499816, 27 offered / 0 retained). The real constraint is the **opening width 7 vs a needed per-slot rank of 119** — owned by rows 2 and 3, not by the fill. |
| 8 | **Depth cap** — the reserve's non-zero-coordinate cap | [w-c3f81a](items/w-c3f81a.md) (integrated at `c11e90e`; derivation in the item's §6 / [REPORT-c3f81b1.md](REPORT-c3f81b1.md)) | `EMIT_PROFILE_MAX_DEEP = 3` replaced by `funded_slot_depth(slot_widths) = count of slots wider than LEXICAL_BRANCH_STAGE_0`. Pricing over **16** targets: **zero** emission cost (totals identical, including the one unsaturated target), pops <0.05%, worst per-target pool **+19** on `recognize speech` (+0.10%), three targets byte-identical. The naive `Σ C(depth, j)` model would have priced the change at 30-56 units — 88% of the per-segmentation allowance, i.e. *unaffordable* — so deriving the cap from the naive model would have manufactured a false negative | **CLOSED and INTEGRATED** (the only *landed* coordinate in this table) | The reservation accounting is `funded_slot_depth`, and the frontier floor it reads is the real bound; the constant's original justification was a comparison against the reserve that was never performed and is false as a bound. **This did not clear the milestone**: the canonical alignment is still absent, and the depth cap was never the reason — all 34 four-deep tuples on case 2 come from *five-slot* segmentations. |
| 9 | **Shortlist width** — `SPAN_SHORTLIST` = 160 | [w-7b40d2](items/w-7b40d2.md) criterion 3 and the width table; REPORT-4d7c12 §1; REPORT-9e2b41 §6 | `SPAN_SHORTLIST` = **160** confirmed by `assert_eq!(previous, list)` with `list = SPAN_SHORTLIST = 160`; the `93` in one test's doc comment is a **measured** segmentation width, not a derived width. Reachability: the 7-wide opening already covers **34%** of a 160-wide slot's whole key range; the canonical's deepest needed index is 99 of 160, at **62%** of that range. Reaching it by uniform width alone needs ≈**100**, i.e. **14.3×** the current 7, and `1 + 100 + 100² + 100³ + 100⁴` = 1.01e8 pops against a limit of 4,000 — a factor of **25,000** | **CLOSED (as a buy), at both ends** | The width is not a free parameter at either end: it is bounded below by the same joint frontier `F` row 3 prices (2.83× short) and above by the same product wall row 3 and REPORT-4d7c12 §5 prices (three to five orders of magnitude). `SPAN_SHORTLIST` is not read by the item's forbidden-by-cost list of levers; changing it changes which 160 are retained (row 4), not how deep the walk looks. |
| 10 | **Objective / ranking surface** — can scoring put the canonical in the printed top 50? | [REPORT-3a8c05.md](REPORT-3a8c05.md) §3, §5 (item [w-3a8c05](items/w-3a8c05.md), `done`); the axis change in [w-9c6f2b](items/w-9c6f2b.md) (`done`); [REPORT-9b4a15.md](REPORT-9b4a15.md) §4, §6 (item [w-9b4a15](items/w-9b4a15.md), `done`, integrated `515f8bd`, verdict HOLD; 14 vectors priced, C1d = 9/10, green case rank 46, case-2 lift −0.0180 → +0.0437) | **1,126** pool clues are `>=` the canonical on `PARSIMONY + RESEG + SIMILARITY` at once, so no monotone objective over any measurable per-candidate property can rank it better than **1,127**; its shipped score is **0.0943520415** below the top-50 cutoff. Shipped top-50 `SIMILARITY` lift vs pool mean is **−0.0180** (case 2) / **+0.0609** (case 1). `w-9c6f2b`'s D2 fix moves the case-2 cutoff from 0.8978 to 0.9171 while moving the answer less — the cutoff rises by more than the answer does; canonical alignment scores 0.7999 and a *zero-cost perfect pronunciation* of it scores 0.8699. `w-3a8c05`'s C1b (`SHAPE` 0.05 → 0, `NOVELTY` 0.15 → `PARSIMONY` 0.15) leaves case-2 at rank **7,393**; the axis itself is confirmed (C1b +0.0565, C1d +0.0437 with a sound bound) but the canonical tuple is **absent from the pool**, not mis-ranked in it | **CLOSED for the canonical tuple** (hard monotone lower bound of 1,127); **OPEN for general head quality; `w-9b4a15` priced it and is now `done`, so it is owned by [w-3f8c62](items/w-3f8c62.md) (`working`, front `agent-3f8c62`)** | §3's bound is weight-free: it is a floor under *every* monotone objective over measurable per-candidate properties, so no re-weighting reaches the top 50. `w-9b4a15` explicitly does **not** claim to reach the canonical tuple — its scope is the general quality fix (top-50 lift −0.0180 → +0.0570 under C1b, `PUNCH` lift +0.1526 → −0.0274, green case provably immune: `recognize speech`'s pool top 200 is 100% four-word, histogram `{4: 200}`). **A reach is not a display**: this row is why every search-side coordinate above is measured on *reach*, not on display. |
| 11 | **Adjacency / one-step substitution** (shape, for completeness) | [w-c1d3a7](items/w-c1d3a7.md) (`done`, `madgab-adjacency` @ `9767caf`) | **ENUMERATED: no. RANKED: no** — both unchanged, and the second is unchanged *by construction*. Force-injecting the clue's own wording on the default path (same segmentation, same slot indices) puts the clue's segmentation at structural rank **151 of 256** unchanged | **Mechanism landed and measured; canonical target REFUTED** | Enumeration was never the blocker — the tuple is lattice-reachable but not emission-reachable, and substituting one slot is a single-slot cost only if the *node* exists to substitute from. Do not cite this as a canonical route. |
| 12 | **Lattice coverage / emission order** (umbrella, DO-NOT-RE-TAKE) | [w-0f3a17](items/w-0f3a17.md) (state `working`, **DO NOT RE-TAKE**), superseding [w-5b1e93](items/w-5b1e93.md) | The predecessors' arithmetic: at **full** per-slot width 160 the traversal emits **2,036,664** wordings of the canonical segmentation's own structure, reaches index 159, and still never emits the canonical tuple — a factor of **124** in the cheapest reallocation and **1,309** in the segmentation's 2,666,496,000-wording product. By cost-best-first order the canonical resegmentation needs **≥404,081** emissions of its own segmentation; the cheapest depth-profile stratification is bounded below by **134,400** against a per-segmentation allowance of **64** | **CLOSED as a shape; the item is deliberately left `working`** so the milestone criterion is not falsely marked met | w-0f3a17's own successor fronts were rows 2, 4 and 6 above (`w-9e2b41` width, `w-7b40d2` fill, `w-c3f81a` placement); all three are terminal and their records are integrated. A pass that finds w-0f3a17 `working` with no live agent of its own must **read those three, not start a fourth front on the same code**. |
| 13 | **Worst-case / full-width traversal** (the order-vs-width frame) | [w-b3e91a](items/w-b3e91a.md), cited in REPORT-4d7c12 §5 and REPORT-1c7d40 §7.2 | **2,036,664** wordings at full width, factor 124 / 1,309 as row 12; and the key-side measurement: `contribution` width over a **whole** 160-wide list, mean **0.0338** (case 2) / **0.0322** (case 1), max 0.0613 / 0.0989; needed word's fraction of the whole-list key width, mean **0.3435** / **0.1381**, max 0.6615 / 0.6204 | **CLOSED** | It is an *order* limit, not a budget limit: the loss is a cut through a smooth function, and every key that moves the cut moves it in a direction that is worse for the pool (row 1) rather than better for the tuple. |

### Conflicts recorded, not resolved

Per the front's no-re-derivation rule, these are carried forward with both numbers intact.

1. **`hid` position: 99 vs 119 vs 136.** 99 (traversal index, REPORT-1c7d40 §1 / REPORT-5d9c04 §1, integrated HEAD) vs **119** (slot rank, REPORT-9e2b41 §2.3 / w-5e2d41 front B §2.3, head `a8a8f70`) vs **136** (slot 0 of span `[0,2)` on base `3e821c3`, w-5e2d41 front B §2.3). Different units and different heads; no verdict depends on the choice.
2. **The item-summary indices `7/0/3/85/7`.** REPORT-9e2b41 §1 records that this summary is in *shortlist* units and is refuted in *traversal* units, where the tuple is `7/0/13/99/11`. This also settles the recorded "237-of-240" vs "miss-by-one" disagreement: the miss-by-one is real for the depth-1 word and is **not** the binding exclusion; three of five slots are far outside cap 7 (one by 6 indices, one by 92).
3. **Depth-5 per-slot affordable width at slot 1: 9 or 10.** REPORT-9e2b41 §6 corrects a printed `9 / 7` to `9 / 10 / 10 / 10 / 160`, because the printed sequence *decreases* and so contradicts the monotonicity the accompanying test claims to assert.
4. **Depth-6 last-slot cost: 3,906, not 2,801** (`1+5+25+125+625+3125`), confirmed by REPORT-9e2b41's own `first_leaf_frontier` on the uniform-5 vector.
5. **The `93` in the shortlist-width test's doc comment** is a measured segmentation width, not a derived width, and does not belong in a table whose header says 160-wide.
6. **Case-2 pool size across heads:** 18,949 (integrated HEAD, REPORT-1c7d40 / REPORT-5d9c04) vs 18,936 (base `3e821c3`) and 18,933 (head `a8a8f70`, w-5e2d41 front B §1). All bit-reproducible on their own heads.
7. **"Not in the pool" vs "ranks out".** w-9c6f2b's Context records that of 1,545,903 enumerated five-word alignments 40,546 contain `dupe` and the best is rank 130 — *it ranks out, it is not absent* — which reverses the standing "not in the pool" conclusion for the *candidate lattice*. The integrated reports' "absent from the pool" is about the production approximate pool. Both are true about different objects; do not use either to re-open the other.

---

## 3. Remaining unpriced shapes

The stage-4 cut decomposes into exactly three axes at fixed budget — **order** (row 1), **set** (row 2),
**width** (row 3) — and all three are priced negative with arithmetic that holds before any candidate
is run. Rows 4-9 close the retention, reserve, floor, depth-cap and shortlist-width surfaces around
it; rows 10-13 close the scoring, adjacency and lattice-coverage shapes. The list below is therefore
short by design, and each entry is a shape that is **not** a re-specification of a closed row.

**Priced since this section was written (do not re-open without a new argument):**

* **Shape 1 is a priced NEGATIVE.** [w-3e91a4](items/w-3e91a4.md) (`done`) priced it in
  [REPORT-3e91a4.md](REPORT-3e91a4.md), integrated 2026-09-27 at `91c0e35`. The refutation is a
  **containment, not a percentile**: the walk's admission test is `index < cap`
  (`src/lib.rs:2238-2241` and again at `:2276`), so *every* admitted set under *any* key — scalar,
  set-level coverage, rank band, private oracle — is a subset of `{0, …, cap-1}`. At depth 5
  `cap = 7` and the canonical's per-slot indices are `7 / 0 / 13 / 99 / 11`, so 4 of its 5 words are
  outside the opening width for structural reasons no key can change; the cheapest single miss
  (index 7, slot 0) already needs `F = 11,329` vs `pop_limit = 4,000` = **2.83×**, and the reaching
  variant is `F = 64,001` = **22.8×** with a measured SIGKILL. Before truncation the shape
  re-specifies rows 1 and 2; after truncation it is a strict subset of the shipped rule and can only
  cost pool. The `hid` conflict (99 / 119 / 136) is therefore non-decisive; all three numbers are
  recorded in the report. **Successor rule:** a band property is a reach coordinate only if it
  changes `cap` or the shortlist *contents* — a **retention** band (which 160 are kept, at what
  rank; map row 4's standing successor rule), never an **admission** band.
* **Shape 2 is also a priced NEGATIVE.** [w-2f1c03](items/w-2f1c03.md) (`done`) priced it in
  [REPORT-2f1c03.md](REPORT-2f1c03.md), integrated 2026-09-27 at `38361de`. Joint-coordinate
  coverage in the reserve sweep is **reach-null by 5 to 6 orders of magnitude**: the shipped sweep
  indexes each member modulo the **narrowest member's span**, so at the decisive fund
  (`max(narrowest) = 93` over all 4-subsets of `[160,7,160,160,93]`) no deep coordinate above 92 is
  representable at all, and after the contained per-member-modulus fix the canonical's 4-deep cell
  is 1.2e-7 likely — one cell in ~8.2 million of a `150·150·150·83 = 279,562,500` frame fed by
  **34** draws against 2,895 reserve tuples and a saturated 16,384-emission ceiling (9.21e5× and
  1.63e5× respectively). Re-opening it as a *reach* coordinate needs a deep-slot product within
  `~D·ln D` of the draw count (`<= ~1.2e5` cells at `D = 16,384`) against `83^4 = 47,458,321` today.

**This section is now empty.** Shape 3 was priced by [REPORT-9e2b41.md](REPORT-9e2b41.md) and
[REPORT-5d9c04.md](REPORT-5d9c04.md), shape 1 above, and shape 2 above. **Case-2 reach is closed
as a search-side question**: no remaining search-side coordinate can enumerate the canonical cell.
Per §4, the live direction is general search quality, and it is now carried by
[w-3f8c62](items/w-3f8c62.md) (`working`, front `agent-3f8c62` in `/workspace/madgab-parsim-3f8c62`
on `madgab-parsim-3f8c62`), which lands the word-count parsimony axis priced by
[REPORT-9b4a15.md](REPORT-9b4a15.md) §6 and turns the red `head_not_worse_than_pool` fence green.
The one non-duplicative residue the sweep report found — the per-member modulus in
`coverage_tuples` — is a **later, non-blocking** front on the reserve side and must be decided
under the same head-lift criterion, never as a reach item.

**Unpriced, and genuinely available:**

1. ~~**A cut that is not a function of a single scalar per-candidate property at all**~~ —
   **PRICED NEGATIVE, see above (REPORT-3e91a4).** Original wording kept for provenance — a
   *structural*
   or *coverage*-keyed admission, e.g. admitting per slot the candidates that span the span's own
   candidate set in distinctness or in phonetic-feature coverage rather than in cost or in
   `contribution`. This is the successor rule both `w-5e2d41` fronts named independently (front A's
   A6, front B's §4.1) and it is a **rank/band** property, not a cheap-end cost-ratio one. The
   arithmetic that constrains it: a candidate at rank `r` is unreachable unless `r < cap`
   (10 / 7 / 5 at d ≤ 4 / 5 / 6), so the needed rank 119 is out at d=5 by 17×, and a 16-draw sample
   of a 4.266e11 grid covers each *slot*, never the *tuple* — so the property has to be about the
   **first `cap` ranks of each slot**, not about global draw coverage.
2. ~~**Joint-coordinate coverage in the reserve sweep**~~ — **PRICED NEGATIVE, see above
   (REPORT-2f1c03, `38361de`).** Original wording kept for provenance — deliberately *not* opened by
   `w-c3f81a` (whose successor is `w-5e2d41`, now closed) and so not priced anywhere at the time.
   The measurement to start from was `w-c3f81a`'s residual: the sweep is uniform in each
   *marginal* index with essentially **zero** joint coverage, and the canonical is a single cell of
   a ~150^4 product. Priced, it is reach-null by 5-6 orders of magnitude; the successor rule is that
   it must be decided as a **head-quality** item, never as a case-2 reach item.
3. ~~**A per-slot width vector argued from a criterion the pop budget does not already fix.**~~
   — **PRICED as a reach coordinate: closed by rows 3 and 9, and REPORT-2f1c03 above.** Original
   wording kept for provenance. REPORT-9e2b41 §4 is explicit: 58,106 jointly affordable allocations
   exist at depth 4, so any rule presenting a per-slot width as *derived* from the allowance is
   presenting a choice as an arithmetic result, and the only other ceiling is not one because
   `LEXICAL_GLOBAL_EMISSION_BUDGET` saturates. **This is not a small task**: any implementation that
   widens the cheapest slot pays 22.8× in resident frontier and OOMs. It stays closed.

**Not unpriced — do not spend a front here:**

* Re-running any row 1-13. `w-1c7d40` "do not re-run it"; `w-7b40d2` and `w-5b1e93` are `superseded`
  precisely so they stop inviting duplicate fronts; `w-0f3a17` is **DO-NOT-RE-TAKE**.
* Buying width, pops, `SPARSE_SHORTLIST`, the depth cap, the emission budget or `SPAN_SHORTLIST`.
  Each is priced above and each fails by three to five orders of magnitude or by a fence regression.
* Chasing the top-50 display for the canonical tuple. `REPORT-3a8c05.md` §3's weight-free bound of
  rank 1,127 makes it unattainable by *any* monotone objective. Reach is the only thing a search-side
  coordinate can move, which is why every row above is measured on reach.

**If shape 1-3 all fail, the honest next move is not another case-2 reach attempt.** They did all
fail, so this is now the operative paragraph. Case 2's canonical reach is a *search-quality*
question after the three axes are priced, and the case-1 green fact is what a general improvement
must not cost. The genuinely valuable work is then:

* let `w-9b4a15` land the one objectively wrong thing in the head (the `SHAPE` → `PARSIMONY` swap,
  row 10) on its own evidence, and
* treat further case-2 reach as an *open-ended* general search-quality item, decided by whether its
  successor improves the head lift (currently **−0.0180**) **without** moving the green case off
  pool rank **27** — not by whether it happens to enumerate one more cell of a 2.666e9 product.

**Where that now stands (2026-09-27, coord-c1d4a).** `w-9b4a15` is `done` and integrated at
`515f8bd` with verdict **HOLD**: it priced 14 weight vectors, confirmed the axis is the right repair,
and declined to ship it because the axis has to reach the structural keys and its own item fenced
it off from the search surface. Its red fence `head_not_worse_than_pool` is in the tree
`#[ignore]`d and red, and that gap is now owned by [w-3f8c62](items/w-3f8c62.md) (`working`, front
`agent-3f8c62` in `/workspace/madgab-parsim-3f8c62` on `madgab-parsim-3f8c62`), whose contract is
`REPORT-9b4a15.md` §6's C1d recipe plus the red/green fence. The head-lift criterion above is that
item's acceptance test, and the green case is its fence.

---

## 4. Standing notes, restated so no pass has to re-derive them

* **The blocker:** `approximate_finds_classic_madgab_resegmentation` (case 2) is **red at base**. It is
  red on every head in every report cited here. **Do not re-pin it and do not let a change turn it
  green by accident.** `corpus_integration` is expected at 12 passed / 1 failed. Run it with
  `-- --test-threads=2`: on this host it is SIGKILLed at default parallelism, on pristine base too,
  and that is not a property of any change (see
  [../environment-notes.md](../environment-notes.md)).
* **The green fact:** `wreck a nice beach` is produced for `recognize speech` at **pool rank 27**,
  score **0.9199502875218423**, displayed 26/50. Every future candidate reports this rank. Three of
  the four set cuts in row 2 lose this case from the pool entirely; three of the four orders' results
  in row 1 leave it at 27; row 4's parked branch keeps it and must stay parked.
* **Host limits:** `cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run here**
  (no `rustup`, no `rustfmt`/`clippy`/`rustdoc` components). Do not mark an item `done` while
  claiming they passed. `git commit` needs `-c commit.gpgsign=false`; the fetch refspec is narrowed,
  so push child branches with an explicit refspec.
* **No release test runs are required for this document.** This front produced no code; it ran no
  test and no probe, and it re-measured nothing.
