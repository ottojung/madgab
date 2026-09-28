---
work_item: w-2f1c03
state: done  # back-reference only; the work item is docs/work/items/w-2f1c03.md, which carries `work_item: true` and the canonical state. Not in the rule-34 discovery population. `produced` is not a state in skills/work-items.md; corrected by coord-6f4a.
docs_only: true
branch: madgab-joint-2f1c03
base: 27f986e (post-milestone-acceptance, pushed)
verdict: HOLD
updated: 2026-09-27T22:40:00Z
---

# REPORT-2f1c03 — shape 2 priced: the reserve's sweep is marginal-uniform, and the joint
# coverage it lacks cannot be bought with the shipped allowance

**Front:** pricing only. No `cargo` invocation of any kind, no test, no benchmark, no
enumeration, no probe, no measurement, no edit to `src/`, `tests/`, `examples/` or
`Cargo.toml`. Every coordinate the obstruction map already prices is **cited by report and
section, not re-derived**; the only new arithmetic is §1–§4 below, and it is arithmetic over
numbers already on record (widths, index positions, draw counts, emission ceilings).

**Answer, in one line:** joint-coordinate coverage is *representable* only after a zero-cost
change to the reserve's index function — because the shipped rule indexes **every** member of a
deep subset modulo the **narrowest** member's span, which caps every deep coordinate at that
slot's last index, and at the decisive fund that cap is 92 while the needed word sits at 99 /
119 / 136 (all three recorded, all three above the cap) — and even after that fix the reserve
places **34** four-deep tuples on the case-2 target into a frame of at least
`83^4 = 47,458,321` cells (`150·150·150·83 = 279,562,500` for the corrected subset), so a
specified cell is drawn with probability **≈1.2e-7 — one in 8.2 million**. On the map's own
decisive fund the shortfall is **2,895 draws against 2,666,496,000 cells = 9.21e5×**, and
**1.63e5×** even if the entire saturated 16,384-emission ceiling were spent on the reserve.
Shape 2 is a priced negative *as a reach coordinate*; the zero-cost residue is a general
coverage improvement to be judged on head lift, not on the canonical tuple.

**Housekeeping, recorded rather than assumed:** the coordinator's prompt directs this front to
read `docs/work/items/w-2f1c03.md`. **That file does not exist in this worktree** at base
`27f986e` (nor anywhere in `git log --all`). The obligations were therefore taken from the
coordinator's prompt itself plus the two documents that do exist, and the item file should be
written by the coordinator when the PR is opened. Nothing in this report depends on it.

---

## 0. Where shape 2 sits, and why it is not foreclosed by anything already priced

Shape 2 is the residual of [items/w-c3f81a.md](items/w-c3f81a.md) §F3 ("the real defect is
joint-coordinate coverage, not tuple depth"), carried into the map as
[OBSTRUCTION-MAP.md](OBSTRUCTION-MAP.md) §3 shape 2 and as the **open half of map row 6**.

Three reasons it is a genuinely open surface and not a re-run of a closed row:

1. **The reserve bypasses the walk's `cap` fence, so [REPORT-3e91a4.md](REPORT-3e91a4.md)
   §2.2's containment does not reach it.** That containment is about the *admission* sets
   `src/lib.rs:2238-2241` / `:2276`, whose union over all rules is `{0,…,cap-1}`. The reserve
   does not read `admit`: `coverage_tuples` indexes `slots[k]` directly, and every index it
   produces is `>= LEXICAL_BRANCH_STAGE_0 = 10` by construction (`sweep_index` returns
   `floor + …`, `src/lib.rs:597-610`). At the canonical depth-5 shape the walk's cap is **7**.
   So *every* tuple the reserve ever emits has all of its deep coordinates **outside** the
   walk's admitted prefix, by construction. This is not an argument; it is on record —
   `w-c3f81a` §F3 measured four-deep tuples at indices of 22 and 99 reaching the production
   path on the case-2 target. Shape 1 stays closed (REPORT-3e91a4 §7; the walk's admission is
   untouched by anything proposed here) and shape 2 is unaffected by it.
2. **The closed half of row 6 is not re-run here.** `funded_slot_depth` (integrated `c11e90e`,
   map row 8) is CLOSED and INTEGRATED with its sixteen-target measurement
   (`w-c3f81a` §F2: zero emission cost, worst pool `+19` on `recognize speech`, pops <0.05%).
   This report starts from that measurement and prices only the residual.
3. **Row 12 is not re-taken.** `w-0f3a17` is DO-NOT-RE-TAKE and its own successors were rows 2,
   4 and 6. This *is* row 6's unclosed half, which `w-c3f81a`'s handoff explicitly declined to
   open ("**No successor front is opened on the sweep from this item**") and which
   `w-5e2d41` did not take up. The map's 2,036,664 / factor-124 / factor-1,309 numbers are
   cited below, never recomputed.

---

## 1. (a) Is joint-coordinate coverage reachable under the shipped reserve allowance?

**Two different questions hide in "reachable", and the answer is *yes* to one and *no* to the
other.**

### 1.1 Representability — yes, but only after a change

At the decisive fund the slot widths are `[160, 7, 160, 160, 93]` (map §1;
REPORT-4d7c12 §1; REPORT-5d9c04 §1). The shipped rule indexes every member of a deep subset
**modulo the narrowest member's span**:

```rust
let narrowest = combo.iter().map(|&slot| slot_widths[slot]).min().unwrap_or(0);
...
sweep_index(narrowest, reserve, out.len(), member, rotation)   // src/lib.rs:751-786
```

and `sweep_index` returns `floor + (nth*stride + member + sweep_rate(member,span)*phase) % span`
with `span = narrowest - floor`. So a member's index is reduced **into the narrowest slot's
span**, and every deep coordinate of a `k`-deep tuple is therefore at most
`narrowest - 1`.

At the decisive fund a `k = 4` subset of five slots has widths drawn four-at-a-time from
`{160, 7, 160, 160, 93}`. Every 4-subset must include slot 1 (width 7) or slot 4 (width 93),
so

> **max over 4-subsets of `narrowest` = 93**, hence **every deep coordinate the reserve can
> place in a 4-deep tuple at this fund is `<= 92`.**

The needed deep coordinate in slot 3 is **99** in traversal units (REPORT-1c7d40 §1 /
REPORT-5d9c04 §1, integrated HEAD, four fronts identical), **119** at the `span_shortlists`
boundary (REPORT-9e2b41 §2.3, head `a8a8f70`), **136** on base `3e821c3` (w-5e2d41 front B
§2.3). **All three exceed 92.** The canonical case-2 4-deep coordinate set is therefore not
merely unlikely to be drawn at the decisive fund — under the shipped rule it is
**unrepresentable in every 4-deep subset at that fund**, on all three recorded positions, and
the verdict is therefore unit-independent. (This is a *different* statement from
REPORT-3e91a4 §2.2's containment: that one says no walk rule can push index 7 at `cap = 7`;
this one says the reserve cannot even *name* index 93 and above in a 4-deep tuple. Different
surface, different fence, same "no".)

### 1.2 Probability — no, and the allowance is what says so

Grant the representability fix of §3 and price the draw count against the frame.

| frame | cells | draws available | shortfall to be a cover |
|---|---|---|---|
| map §1 / REPORT-4d7c12 §5 decisive 5-slot fund `160·7·160·160·93` | 2,666,496,000 | **2,895** reserve tuples on case 2 (`w-c3f81a` §F2, `placed @3`; 2,929 at the integrated depth cap) | **9.21e5×** |
| same, entire 16,384-emission ceiling spent on the reserve | 2,666,496,000 | 16,384 | **1.63e5×** |
| tightest 4-deep frame, shipped shared modulus `83^4` | 47,458,321 | **34** four-deep tuples on case 2 (`w-c3f81a` §F3) | **1.40e6×** |
| 4-deep frame after the per-member-modulus fix, subset `{0,2,3,4}`: `(160-10)^3·(93-10)` | 279,562,500 | 34 | **8.2e6×** |
| six-coordinate grid (map row 6) `160·160·7·160·160·93` | 4.2659e11 | 2,895 | 1.47e8× |

Arithmetic: `2,666,496,000 / 2,895 = 9.21e5`. `2,666,496,000 / 16,384 = 1.63e5`.
`83^4 = 47,458,321`; `47,458,321 / 34 = 1.40e6`.
`150·150·150·83 = 279,562,500`; `279,562,500 / 34 = 8.2e6`.
Per-cell draw probability in the fixed frame, `34 / 279,562,500 = 1.22e-7`, i.e.
**one specified cell in ~8.2 million**.

Coupon-collector framing, because it is the general form: hitting **one** specified cell of an
`M`-cell frame with `D` independent draws has probability `≈ D/M`; *guaranteeing* coverage of
the frame needs `≈ M·ln M`. At the fixed frame `M = 2.80e8`, `M ln M ≈ 5.5e9` draws — **1.6e8×**
the shipped 34. Nothing about the *design* of 34 draws changes `D/M`; only a change in `D`
does.

**What the allowance is, exactly (cited, not re-derived):**

* `EMIT_PROFILE_RESERVE = 16` (`src/lib.rs:150`) — draws funded per segmentation; the map row 6
  records the 16 draws as **bit-identical across heads** (`[136,0,0,0,0,0]`,
  `[0,146,0,0,0,0]`, `[0,0,0,10,0,0]`, …).
* `EMIT_PROFILE_SAMPLE = 8` (`src/lib.rs:507`) — positions **looked at** per subset, of which
  **one** is emitted. Looking at 8 is not drawing 8, so this constant cannot be used to buy
  joint coverage; the doc comment says so ("sampling more positions per phase narrows the
  stride of the rotation, it does not bias it").
* `LEXICAL_COMBINATIONS_PER_SEGMENTATION = 64` — the walk's per-segmentation allowance, of which
  the depth-profile reserve takes **14-16** and the walk spends the remaining **48-50** (map
  row 5). So `RESERVE` is already ~25% of the segmentation's own allowance.
* `LEXICAL_GLOBAL_EMISSION_BUDGET = SEGMENTATION_KEEP · 64 = 256 · 64 = 16,384`, **saturated
  16,384/16,384** (w-b3e91a, restated in `w-c3f81a` §F2 and `w-0f3a17`). `STAGE-DRAIN` fires
  **0** times in the whole run, so the designed 7 → 28 → 112 → 160 widening ladder never begins
  (map row 5).
* `coverage_tuples` returns as soon as `out.len() >= reserve` (breadth-before-depth
  truncation). Raising `RESERVE` therefore does **not** raise the emission total: it takes the
  walk's share. Row 5 priced this surface and it is not binding *as a cause*, but it **is**
  binding as a *ceiling*.

**Answer to (a):** joint-coordinate coverage is **not reachable under the shipped reserve
allowance, and cannot be reached by any rule that preserves the draw count.** To be a cover of
the map's own decisive fund the reserve would need 2,666,496,000 draws against 2,895 shipped
(**9.21e5×**), or 16,384 if the entire saturated global ceiling were diverted to it
(**1.63e5×**). To be a cover of the tightest *fixed* 4-deep frame it needs 8.2e6×. In
emissions, a cover of the decisive fund is 2.67e9 emissions against 16,384 — the ceiling is
the wall, and it is already at its limit.

**What would make it reachable, priced:** `D/M <= ~1e-2` is the regime where a design property
could matter at all. At `D = 16,384` that means `M <= ~1.6e5` cells for the deep product, i.e.
roughly **36 per deep slot**. Four coordinates at 36 is `1.68e6` — still 100× out; three
coordinates at 36 is `46,656`, inside. So the *only* regime in which a joint-design property
could be a reach coordinate is a **narrowed** deep frame, and narrowing is map row 3 (per-slot
width) and row 9 (`SPAN_SHORTLIST`), both priced: 2.83× on pops for the cheapest single miss,
22.8× on resident frontier with a measured SIGKILL, and ≈100-wide uniform = 1.01e8 pops =
25,000×. **There is no reachable regime.** This is the general statement of the priced negative.

---

## 2. (b) The general, phrase-free property that would change marginal-uniform into joint

Stated as a property of the **index function of the draw**, of the span, its slots and its
candidate set only — no word, no clue, no target, no input, no literal token:

> **P1 — per-member span (representability).** Each member of a deep subset is indexed in
> **its own** slot's span above the traversal floor, not in the narrowest member's span.
> Formally: for a subset `S` and member `m ∈ S`, the draw is
> `floor + (…) mod (width_{S[m]} - floor)`, not `… mod (min_{s∈S} width_s - floor)`.
> Stated as a measurable invariant: **no member of a subset may be assigned an index at or
> above another member's own width**, and each member's index, taken over a run of phases,
> covers *that member's own* whole list above the floor (the property the shipped
> `sweep_rate`/`gcd` construction was written to preserve, and which P1 preserves).

> **P2 — rank-`k` joint support (decorrelation).** The deep coordinates of a `k`-deep subset
> must be drawn from a design of rank `>= k` in the product of the members' spans, not from a
> rank-1 coset. Stated as a measurable invariant: over the phases, the set of deep-coordinate
> vectors a subset produces must not lie in any single 1-parameter family
> `{(r, r+c_1, r+c_2, …) mod span_j}`; equivalently the offset applied to member `m` must be a
> deterministic function of `(subset identity, phase, m)` that is **not** affine in `m` with
> shared modulus.

Both are properties of a *function*, are testable without any phrase, and neither reads the
answer. P2 is the direct generalisation of what the shipped code already half-does: it gives
each member a distinct coprime rate (`sweep_rate(member, span)`, `src/lib.rs:613-628`) so that
"a set of deep coordinates at unrelated ranks is reachable instead of only a set at adjacent
ranks" — but the **modulus is still shared**, so the whole class still traces a rank-1 curve in
the deep-coordinate torus. Removing the shared modulus (P1) and the affine-in-`member` offset
(P2) is exactly the difference between a *marginal* design and a *joint* one.

**Why this is the right shape and not a re-specification of shape 1.** Shape 1 was an
**admission** cut — a property deciding *which candidates enter the opening width*, refuted by
containment because `admit ⊆ 0..min(cap, n)` (REPORT-3e91a4 §2.2, and its successor rule: a
band property is a reach coordinate only if it changes `cap` or the shortlist *contents*).
P1/P2 change **neither**: the walk's admitted set is bit-identical, the shortlist contents are
bit-identical, `cap` is untouched. They change only the map `phase -> index-tuple` that the
*reserve* walks, which is row 6's own surface and which — per §0.1 — is provably outside the
walk's cap fence already. This is also exactly the successor rule the map states for shape 1's
residue: "never an **admission** band" — this is neither an admission nor a retention band; it
is a **placement** rule.

**And the honest limit of P1+P2, stated before it is priced:** they change the *support* of the
reserve's draws, not their *number*. `D` is fixed by the allowance, so `D/M` is unchanged —
except that P1 *enlarges* `M` (from `83^4 = 4.75e7` to `150^3·83 = 2.80e8` for the canonical
subset), so on the canonical target P1+P2 move the cell from **unrepresentable** to
**1.2e-7 likely**. Necessary, and nowhere near sufficient.

---

## 3. (c) The cheapest version, priced in candidate visits and emissions

Cheapest version = **P1 alone** (P2 is free on the same accounting, so it is bundled, but P1
carries the canonical's representability and P2 carries no additional cost).

The change is confined to `coverage_tuples`'s per-member call: pass
`slot_widths[slot]` instead of `narrowest` to `sweep_index`, and take `sweep_rate`'s modulus
from the member's own span. Nothing else moves.

| quantity | shipped | after P1 (+P2) | delta |
|---|---|---|---|
| traversal pushes (`LEXICAL_HEAP_POP_LIMIT` consumers) | as shipped | as shipped | **0** — `coverage_tuples` does not push into the walk's heap; it builds tuples and hands them to `build` |
| `seen` / heap entries | as shipped | as shipped | **0** |
| resident frontier `F` | 2,801 (uniform-7, depth 5) | **2,801** | **0** — REPORT-9e2b41 §5's `F`, untouched because no per-slot width changes |
| reserve tuples held | `<= EMIT_PROFILE_RESERVE = 16` | **16** | **0** — the `out.len() >= reserve` truncation is unchanged |
| emissions | 16,384/16,384 (saturated) | **16,384/16,384** | **0** — one tuple emitted per funded subset, breadth-before-depth, unchanged |
| bound evaluations per segmentation | `<= EMIT_PROFILE_SAMPLE · EMIT_PROFILE_RESERVE` (256 as documented; 128 on the tight `8·16` reading) | same | **0** — same `<= 8 · 16` evaluations, each the arithmetic the traversal's own heap key already performs |
| per-member index arithmetic | one `div_ceil` + one `mul/add/mod` + a `gcd` walk in `sweep_rate` | same, with the member's own span | **0 additional** — the gcd walk is over a *different* modulus of the same magnitude |
| retained allocations between segmentations | none | none | **0** |

**In candidate visits: 0. In emissions: 0. In frontier: 0.** P1+P2 is the cheapest coordinate
this surface has produced — cheaper than anything in rows 1-4, which all cost either
`2.83×`-`25,000×` or pool.

**The SIGKILL / memory risk (REPORT-9e2b41 §5), priced on this surface.** The reserve-surface
rule above carries **no** new memory risk, because it creates no new live structure: `out` still
holds at most 16 `Vec<usize>`. The risk attaches to the two *alternatives* for buying joint
coverage, and both are the same wall REPORT-9e2b41 §5 measured from the traversal side:

* **By raising `EMIT_PROFILE_RESERVE`.** Being a cover needs `R/16 >= 9.21e5` placements, i.e.
  `R ≈ 1.47e7` tuples per subset. `out` is `Vec<Vec<usize>>` with one inner `Vec<usize>` of
  `depth` per placement, so at `depth = 5` that is `1.47e7 · (24 + 40) B ≈ 0.94 GB` resident
  **for a single segmentation**, and a full cover of the decisive fund is
  `2.67e9 · 64 B ≈ 171 GB`. These two figures are *arithmetic from the `Vec` element size*, not
  measurements, and they are recorded as such; they are stated here so that a successor does not
  discover them by OOM. It is the reserve-side image of REPORT-9e2b41 §5's `[160,7,7,7,93]`,
  `F = 64,001` vs `2,801` = **22.8×** resident, × 256 retained segmentations, **SIGKILL
  (measured)**. Note also that `R = 1.47e7` is `2.3e5×` the per-segmentation allowance of 64, so
  it is unaffordable in emissions before it is unaffordable in memory.
* **By routing joint coverage through the traversal instead of the reserve.** That is a
  per-slot-width change, i.e. row 3 / `w-9e2b41`, and it inherits the measured SIGKILL verbatim.
  `w-c3f81a` §F3 already records the traversal-side statement: at full per-slot width 160 the
  traversal emits **2,036,664** wordings of the canonical segmentation's own structure, reaches
  index 159, and still never emits the tuple — factor **124** in the cheapest reallocation and
  **1,309** in the 2,666,496,000-wording product (map row 12 / row 13). The reserve route is
  short by 9.21e5× and the traversal route by 1,309×; they are two faces of the same
  product-vs-linear-budget wall, and `w-c3f81a`'s residual finding is that wall reached from the
  placement side.

So the cheapest version of the property is free *and* the expensive versions are the two that
are already priced. That is the whole of the arithmetic in (c).

---

## 4. (d) Explicit re-specification check against the map's 13 rows

| # | row | this shape | why |
|---|---|---|---|
| 1 | per-slot **order** | **not a re-specification** | P1/P2 change the reserve's `phase -> index` map, not the sort of `slots[k]` (REPORT-1c7d40 §2/§3, 16 keys, 0/20 reach, best mean position 24.70 vs cap 7). No ordering is compared or chosen. |
| 2 | per-slot **admitted set** | **not a re-specification** | The walk's `admit` is bit-identical; `cap` is not read. Row 2's `cap/n = 0.0219` percentile arithmetic is untouched and unnecessary. |
| 3 | **per-slot width** | **adjacent, and it is the one that sets the ceiling** | The 4-deep frame `150^3·83` and the `max(narrowest) = 93` cap in §1.1 are read off the widths, so this surface is *widthed by* row 3. It does not change `F`, and it does not need to — it only aims at cells `F` cannot enumerate. Row 3's numbers (2.83× pops; 22.8× frontier, SIGKILL) are inherited unchanged and are what makes the reach unbuyable. |
| 4 | **fill strategy** | **adjacent, and this is where the residue is** | P1 depends on per-slot widths, which the fill determines. Under the shipped global-`quality` fill the needed words **are** retained (map row 4's 623-dropped defect is on the cheap end; the successors `w-5e2d41` priced the cheap-end floor negative on three counts). So P1 is the first shape since shape 1 that can even *address* a needed word at rank 99/119/136. It still cannot *draw* it. Map row 4's standing successor rule (a rank/band **retention** floor) is untouched and is not re-litigated here. |
| 5 | **emission bound** | **inherits it, and it is the decisive constraint** | 16,384/16,384 saturated; the traversal is allowance-bound (reserve 14-16 of 64, walk 48-50); `STAGE-DRAIN` = 0. §1.2's `D/M` arithmetic is row 5 expressed as a coverage ratio. P1/P2 cost 0 emissions, which is the only reason they are implementable at all — and 0 emissions is exactly why they are reach-null. |
| 6 | **reserve coverage / placement** | **this IS row 6, and only its unclosed half** | The depth-cap half is CLOSED and INTEGRATED (`c11e90e`, map row 8, `w-c3f81a` §F2's sixteen-target table). The joint-coverage half is what this report prices, for the first time, from `w-c3f81a`'s residual and from map §3 shape 2. The 16 bit-identical draws and the `4.266e11` grid are cited from row 6, not recomputed. |
| 7 | **cheap-end floor** | **untouched** | 1.5× vacuous on 108/108 spans, 0 of 885 unsaturated; no finite multiple reaches a word at cost 0.35 on a span whose `min_cost` is 0.0; binding a floor empties a band. Not re-run. |
| 8 | **depth cap** | **used, not re-opened** | `funded_slot_depth` is what makes 4-deep subsets fundable at all on a 5-slot segmentation (all 34 of them on case 2, `w-c3f81a` §F3) and its zero-emission cost is what lets §3's accounting be 0. Its sixteen-target measurement is cited. |
| 9 | **`SPAN_SHORTLIST = 160`** | **inherited** | The 150- and 83-wide spans above the floor are 160 minus 10. Buying width to change them is priced: ≈100-wide uniform = `1+100+…+100^4 = 1.01e8` pops vs 4,000 = **25,000×**; and the deeper slot needed is 92 → 100 by width alone, which is 14.3× the current 7. |
| 10 | **objective / ranking** | **adjacent, explicitly not this front's** | The reserve already selects *within* each subset by an **admissible bound** (`EMIT_PROFILE_SAMPLE = 8`, one-sided, no tuple is ever dropped in favour of a worse-bounded one). Extending that selection from "which of 8 positions" to "which of the `C(depth,k)` subsets" would be a placement-selection rule, and it is **still reach-null**: it chooses *which* 2,895 of 2.67e9 cells, so `D/M` is unchanged. `REPORT-3a8c05.md` §3's weight-free floor of rank 1,127 concerns the printed top 50, not pool presence, so it does not bear on reach. **The SHAPE→PARSIMONY question is `w-9b4a15` / `agent-9b4a153`'s to land on its own evidence and this front expresses no view on it.** |
| 11 | **adjacency / one-step substitution** | **untouched** | `w-c1d3a7` @ `9767caf`: enumerated no, ranked no; forcing the clue's own wording leaves its segmentation at structural rank 151 of 256. Not re-run. |
| 12 | **lattice coverage / emission order** | **adjacent; this is the front's mandate, not a re-take** | `w-0f3a17` is DO-NOT-RE-TAKE and names rows 2, 4, 6 as its own successors. This is row 6's unclosed half, which `w-c3f81a`'s handoff explicitly left unopened. The 2,036,664 / 124× / 1,309× figures are cited. |
| 13 | **worst-case / full-width traversal** | **inherited** | The `contribution`-key numbers (whole-list key width 0.0338 / 0.0322, the opening 7 spanning 0.0114 = 34%, the needed word at 0.3435 / 0.1381) are the *order* limit. P1/P2 change the index function, not the key, so that limit is untouched. |

**Conflicts on record, both/all numbers kept, none reconciled and none load-bearing here.**

1. **The needed slot-3 position: 99 vs 119 vs 136.** 99 (traversal index, integrated HEAD) /
   119 (`span_shortlists` slot rank, head `a8a8f70`) / 136 (slot 0 of span `[0,2)`, base
   `3e821c3`). **This report's §1.1 result is unit-independent and does not need the choice:**
   `max(narrowest) = 93` at the decisive fund, so *all three* exceed the deepest index any
   4-deep subset can name (92), and *all three* also exceed `cap = 7` (REPORT-3e91a4 §2.2), and
   under 99/119/136 the miss is 92/112/129 = 14.1×/17.0×/19.4× the opening width. The
   per-cell probability in §1.2 moves by 1.2× between the extremes and changes no verdict.
2. **Index triples: `7/0/13/99/11` (rank 151) vs `7/0/22/99/11` (rank 187) vs the item-summary
   `7/0/3/85/7`.** Both traversal-index readings and the shortlist-unit summary are carried.
   The 4-deep coordinate sets are `{7,13,99,11}` / `{7,22,99,11}` / `{7,3,85,7}`; the maximum
   member is 99 / 99 / 85, so the §1.1 exclusion bites the first two at the decisive fund and
   the third only in its 85 reading — which is precisely why 85 is recorded as *shortlist* units
   (REPORT-9e2b41 §1) and is not reconciled here.
3. **Depth-5 per-slot widths: `[160,7,160,160,93]` at the decisive fund (map §1) vs
   `9/10/10/10/160` at another (REPORT-9e2b41 §6, which corrects a printed `9/7` that
   decreases and so contradicts its own monotonicity assertion).** §1.1's `max(narrowest) = 93`
   is stated *for the decisive fund*; on the `10/10/10/160` reading the maximum narrowest
   4-subset is 10 and the exclusion is total. **The verdict is the same on both.**
4. **Depth-6 last-slot cost 3,906, not 2,801** (`1+5+25+125+625+3125`; REPORT-9e2b41 §6).
   `F = 2,801` is used above as the depth-5 uniform-7 frontier per map row 3 / REPORT-9e2b41 §5.
5. **Case-2 pool: 18,949 (integrated HEAD) / 18,936 (base `3e821c3`) / 18,933 (head
   `a8a8f70`).** All three bit-reproducible on their own heads.
6. **The `93` in the shortlist-width test's doc comment is a measured segmentation width**, not
   a derived width (map conflict 5). §1.1 uses the map §1 fund `[160,7,160,160,93]`, not the
   test comment.
7. **"Not in the pool" vs "ranks out"** (map conflict 7, `w-9c6f2b`): the candidate *lattice*
   ranks such a multiset out rather than being absent from it. Both are true of different
   objects. This report's frames are frames of the **traversal's and the reserve's index space**,
   which is upstream of both, so neither reading bears on §1's arithmetic.

---

## 5. Standing facts this front did not touch

* **The blocker.** `approximate_finds_classic_madgab_resegmentation` (case 2) is red at base
  and at every head cited here. Not chased, not re-pinned, and no expectation in any test file
  is proposed for change.
* **The green fact.** `wreck a nice beach` for `recognize speech` at **pool rank 27**, score
  **0.9199502875218423**, displayed 26/50. Nothing here was measured, so this front makes **no**
  claim about it; but P1/P2 change *which* tuples the reserve emits and therefore change pool
  contents, so any successor that implements them must report this rank and not merely that the
  case is still reached. This is the specific risk `w-c3f81a` §F2 recorded for its own
  zero-emission-cost change (worst pool `+19` on `recognize speech`, +0.10%) — the sign there
  was positive, but a *removal* of the shared-modulus confinement removes tuples, and row 2's
  three cuts show this pool can lose 6,223 candidates (18,949 → 12,726) to a set rule.
* **Host / hygiene.** No `cargo` command, test, benchmark, enumeration, probe or measurement
  was run. `/workspace/madgab-parsimony-9b4a15` (`agent-9b4a153`, live) was not touched; the
  two OOM-killed predecessors are the reason this front ran no build at all.
  `git diff --stat 27f986e..HEAD` shows `docs/` only. `cargo fmt`, `clippy` and doctests cannot
  run on this host and are not claimed.
* **Non-duplication.** Nothing proposed here reads a word, a clue, a target or a literal token;
  P1 and P2 are properties of a function of the span's per-slot widths, the phase and the
  member index. `src/`, `tests/`, `examples/` and `Cargo.toml` are untouched, and no baseline
  changed.

## 6. Verdict

**HOLD.**

Shape 2 is a **priced negative as a reach coordinate**, on arithmetic that holds before any
candidate is run:

* the shipped reserve indexes every member of a deep subset **modulo the narrowest member's
  span**, so at the decisive fund — where `max(narrowest) = 93` over all 4-subsets of
  `[160,7,160,160,93]` — **no deep coordinate above 92 is representable**, and the needed slot-3
  position is 99, 119 or 136 on the three recorded heads/units. All three are excluded; the
  verdict is unit-independent;
* after the per-member-modulus fix (P1, plus the rank-`k` decorrelation P2, both **phrase-free,
  0 extra pushes, 0 extra frontier `F`, 0 extra emissions**), the canonical's 4-deep cell moves
  from *unrepresentable* to **1.2e-7 likely — one specified cell in ~8.2 million** of a
  `150·150·150·83 = 279,562,500` frame fed by **34** draws;
* the draw count is the wall, and it is already settled (w-b3e91a, restated in `w-c3f81a`
  §F2): `2,895` reserve tuples
  against `2,666,496,000` cells = **9.21e5×**; the entire saturated 16,384-emission ceiling
  spent on the reserve = **1.63e5×**; a coupon-collector cover of the fixed frame = **1.6e8×**;
* the only regime in which a joint-design property could be a reach coordinate is a **narrowed**
  deep frame of `~<= 1.6e5` cells, and narrowing is map rows 3 and 9, priced at 2.83× (pops),
  22.8× (resident frontier, **measured SIGKILL**) and 25,000× (`1.01e8` pops);
* the two expensive ways to buy joint coverage — raising `EMIT_PROFILE_RESERVE` to
  `~1.47e7` (0.94 GB resident for one segmentation by `Vec`-element arithmetic, `~171 GB` for a
  cover, and 2.3e5× the per-segmentation allowance of 64) and routing coverage through the
  traversal (REPORT-9e2b41 §5's measured SIGKILL) — are the two faces of the wall already priced
  as rows 3 and 12.

**Successor rule, one line:** joint-coordinate coverage in the reserve sweep is a *free,
general, phrase-free* coverage improvement that is **reach-null by 5 to 6 orders of magnitude**,
so it must be decided as a **head-quality** item (does it move the top-50 lift off −0.0180
without moving the green case off pool rank 27) and never as a case-2 reach item; and a re-open of
this surface as a *reach* coordinate requires a deep-slot product within `~D·ln D` of the draw
count, i.e. `<= ~1.2e5` cells at `D = 16,384`, against `83^4 = 47,458,321` today.

**The one non-duplicative residue, for a future implementation front (not this one):** P1 is a
contained, zero-cost, strictly-improving change to `coverage_tuples`' per-member call in
`src/lib.rs`, with a general two-part test available in phrase-free form — (i) *no member of a
subset is assigned an index at or above that member's own slot width, for every subset at every
phase*, with the expectation recomputed inline from the widths rather than by calling the
function; and (ii) *the deep-coordinate set of a `k`-deep subset is not confined to any
1-parameter family modulo the members' spans* — that is, a single coordinate may no longer be a
function of the others by an affine map with shared modulus. It must report `recognize speech`'s
pool rank 27 and the three `approximate_pool_reaches_*` guards, and it must be landed (if at all)
under the head-lift criterion. It is **not** a case-2 reach fix, and this report should not be
read as one.
