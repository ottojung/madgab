---
work_item: w-b7d4c1
state: produced
docs_only: true
branch: madgab-covmod-b7d4c1
base: 1923bbf (post-milestone-acceptance)
verdict: HOLD
updated: 2026-09-27T21:40:00Z
---

# REPORT-b7d4c1 — P1's per-member modulus is free and reach-null exactly as priced, and it
# regresses the reserve's *depth* fence, which no permitted surface here can buy back

**Front:** agent-b7d4c1, branch `madgab-covmod-b7d4c1`, worktree
`/workspace/madgab-covmod-b7d4c1`, base `1923bbf`. Release mode for every measurement. The
production change **was implemented and measured**, and is preserved unlanded as
[w-b7d4c1-p1-per-member-modulus.patch](w-b7d4c1-p1-per-member-modulus.patch) (the whole `src/lib.rs`
diff — P1 plus the two property tests; applies clean to `1923bbf`). This branch carries `docs/`
only, so the tree is green at every commit boundary. `cargo fmt`, `cargo fmt --check`,
`cargo clippy`, doctests and `wasm32` builds cannot run on this host and are **not** claimed.

**Answer, in one line:** P1 costs exactly what [REPORT-2f1c03.md](REPORT-2f1c03.md) §3 priced —
**0** extra pushes, **0** extra heap entries, **0** extra emissions (16,384/16,384 saturated
before and after), **0** extra frontier, **0** extra bound evaluations, no new state, wall clock
unchanged — and it moves head lift by **+0.0004** (case 2) and **+0.0006** (case 1) while holding
`wreck a nice beach` at pool rank **27** with a bit-identical score; but per-member spans reach each
member's own last index (159) where the shared narrowest span capped every deep coordinate at 92,
so a four-deep tuple's additive substitution cost rises past `total_budget` often enough that
`build` drops it, and the green fence
`a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` goes **red on 3 of its 12
targets** — a repair that is scoring-side, fenced out of this item and owned concurrently by
`w-3f8c62`. Verdict: **HOLD**, with the numbers.

---

## 0. What is on this branch, and what is not

* `docs/work/REPORT-b7d4c1.md` (this file) and
  `docs/work/w-b7d4c1-p1-per-member-modulus.patch`.
* **Not** the `src/lib.rs` change, and **not** the two property tests, for the reason in §6: the
  property tests are red on base by construction, so landing them without the change would put a
  red fence on the branch, and landing them *with* the change puts a different red fence on the
  branch. The patch carries the two together, which is the only shape in which they mean anything.
  The `#[ignore]`d measurement probe used in §2/§5 is deliberately **not** in the patch and **not**
  on the branch; it was a scratch probe inside a test module, removed before the patch was cut.

## 1. Baseline re-derived on integrated HEAD (`1923bbf`), release mode

`cargo test --release --lib price_the_budget -- --ignored --nocapture` — the harness already in the
tree; it names the two canonical pairs as *data* inside a test module, and no production path reads
either string.

| figure | item's figure | measured on `1923bbf` | agrees |
|---|---|---|---|
| case-2 pool (`It's just a stupid game`) | 18,949 (REPORT-2f1c03 §4 conflict 5) | **18,949** | yes |
| case-1 pool (`recognize speech`) | 18,289 (map §1) | **18,289** | yes |
| case-2 head-50 `SIMILARITY` lift vs pool mean | **−0.0180** | **−0.0180** (0.8091 vs 0.8271) | yes |
| case-1 head-50 `SIMILARITY` lift vs pool mean | **+0.0609** | **+0.0609** (0.8561 vs 0.7953) | yes |
| `wreck a nice beach` pool rank for `recognize speech` | **27** | **27**, score 0.9199502875 | yes |
| `no_phrase_hard_coding` | 9/9 | **9 passed / 0 failed** | yes |
| `approximate_pool_reaches_*` (three guards) | green | **3 green** | yes |
| `corpus_integration` (`-- --test-threads=2`) | 12 passed / 1 failed | **12 passed / 1 failed**, red = `approximate_finds_classic_madgab_resegmentation` | yes, red preserved |
| `cargo test --release --lib` | green | **75 passed / 0 failed / 12 ignored** | yes |

**No baseline figure disagrees with the item**, so both are not carried. The other three test
binaries on base: `tests/emit_coverage` 4/4, `tests/approx_determinism` 4/4,
`tests/exact_determinism` 1/1.

## 2. The change, and the cost accounting

The whole of P1, confined to `coverage_tuples`'s per-member call (`src/lib.rs:740-786` at base):
the subset's `narrowest` width is no longer computed and passed to `sweep_index`; each member is
passed **`slot_widths[slot]`**, so `sweep_index` takes `sweep_rate`'s modulus from that member's own
span above the floor. The now-redundant `tuple[slot] >= slot_widths[slot]` re-check goes with it,
because per member the index is inside its own slot by construction; the `legal` flag stays and now
means *this member's* slot has no room above the floor, which is exactly what the old `narrowest`
test expressed. Consequences checked rather than assumed:

* `funded_slot_depth` is **unchanged** — a subset is fundable iff **every** member's slot is wider
  than the floor, which is the same statement as "the narrowest is wider than the floor", so the
  derived count and its red-free test are untouched;
* nothing else moves: no scoring line, no `LEXICAL_HEAP_POP_LIMIT`, no `SPAN_SHORTLIST`, no
  `SPARSE_SHORTLIST`, no `LEXICAL_COMBINATIONS_PER_SEGMENTATION`, no `EMIT_PROFILE_RESERVE`, no
  `EMIT_PROFILE_SAMPLE`, no emission ceiling, no depth cap, no width table, and no admission rule
  (`index < cap`).
* **P2 is not shipped as a separate edit.** With the per-member modulus in place the offset is
  already not affine in `member` with a shared modulus — that *is* property (ii) below — so the
  item's "+P2 if it is free on the same accounting" clause has no second edit to make.

| quantity | REPORT-2f1c03 §3 prediction | measured on the P1 tree | delta |
|---|---|---|---|
| traversal pushes (`LEXICAL_HEAP_POP_LIMIT` consumers) | 0 | pops per target (base → P1): 446,340→446,654; 220,967→221,538; 169,119→169,494; 188,273→188,698; 279,934→280,317; 209,654→209,752; 169,961→170,257; 229,648→230,188; 190,976→191,495; 195,376→195,570; 272,808→273,135; 224,586→224,966 | **0 in kind**: every target **+98 to +571 pops, worst +0.27%** |
| `seen` / heap entries | 0 | no new live structure; `out` still holds at most `reserve` `Vec<usize>` of `depth` | **0** |
| resident frontier `F` | 2,801 unchanged | unchanged — no per-slot width moved | **0** |
| reserve tuples held | ≤ 16 | 16 | **0** |
| emissions | 16,384/16,384 saturated | **16,384 / 16,384 on all twelve targets**, base identical | **0** |
| bound evaluations per segmentation | ≤ `EMIT_PROFILE_SAMPLE · EMIT_PROFILE_RESERVE` | same `t` loop, same `bound_of` calls, ≤ `8 · 16` | **0** |
| per-member index arithmetic | same, different modulus of the same magnitude | one `div_ceil`, one `mul/add/mod`, the same `gcd` walk — over that member's span | **0 additional** |
| retained allocations between segmentations | none | none | **0** |
| wall clock (12 targets, one process) | — | base 1.11–1.85 s/target, P1 1.05–1.61 s/target | within noise (if anything marginally faster: the dropped re-check) |
| case-2 pool size | — | 18,949 → **18,934** (−15, **−0.08%**) | within noise; same sign and order as `w-c3f81a` §F2's +19 (+0.10%) for its own zero-emission change |
| case-1 pool size | — | 18,289 → **18,292** (+3, **+0.016%**) | within noise |

**Accounting verdict: the report's zero-cost claim is confirmed on every line it priced.** What the
report priced was the *quantity* of the reserve's spend; it did not price the *content* of it, and
§5 is what that omission costs.

## 3. Head lift before/after, the green case, and the three guards

| measurement | before (`1923bbf`) | after (P1) | delta |
|---|---|---|---|
| case-2 head-50 `SIMILARITY` lift (pool mean subtracted) | **−0.0180** (0.8091 vs 0.8271) | **−0.0176** (0.8091 vs 0.8268) | **+0.0004**, toward zero |
| case-1 head-50 `SIMILARITY` lift | **+0.0609** (0.8561 vs 0.7953) | **+0.0615** (0.8561 vs 0.7947) | **+0.0006** |
| `wreck a nice beach` pool rank for `recognize speech` | **27**, score 0.9199502875 | **27**, score **0.9199502875** (bit-identical) | **0** |
| `approximate_pool_reaches_matches_deep_in_a_span` | green | **green** | — |
| `approximate_pool_reaches_alternatives_past_the_opening_slot_width` | green | **green** | — |
| `approximate_pool_reaches_resegmentations_deeper_than_one_walk` | green | **green** | — |
| `approximate_finds_classic_madgab_resegmentation` (case 2, **red at base**) | red; canonical absent from the pool | **red; canonical still absent from the pool** (`rank absent`, `score NaN`), same twelve top-50 phrases | preserved, not chased, not re-pinned |
| `head_not_worse_than_pool` (red at base, `w-3f8c62`'s fence) | red on **10/10** | red on **10/10**, the two named negatives at `−0.0077` and `−0.0184`, essentially unchanged | unchanged in kind |
| `no_phrase_hard_coding` | 9/9 | **9/9** | hard fence held |

Under the item's own criterion — head quality, with the green case still inside the top 50 — P1
**passes, and barely**: both canonical lifts move ~5e-4 in the right direction and the green case
does not move at all. That is the honest size of the head-lift signal available from a change the
report calls the cheapest coordinate this surface has produced. It is not nothing; it is not a
headline either, and it is three orders of magnitude below the objective lifts the concurrent front
is chasing (§7).

## 4. The two property tests: red before, green after, phrase-free

Both live in `src/lib.rs`'s test module, both phrase-free (no sentence, clue, word list or example
token in either assertion), and both recompute their expectation **inline from the slot widths and
`LEXICAL_BRANCH_STAGE_0`** rather than by calling the function under test — the same independence
discipline as `the_reserve_depth_bound_is_the_slots_the_traversal_leaves_room_in`, and for the
reason that test gives: a test that asserted the function equalled itself would pass whatever the
function returned, including a reintroduced literal.

1. **`each_sweep_member_is_indexed_in_its_own_slot_width`** — for every subset of every funded size
   and every phase, each member's drawn index is inside **its own** slot's list above the traversal's
   floor, and some member of the subset is drawn at or above the subset's **narrowest** width, so no
   member's draw is confined to another member's span. It also asserts that every recomputed subset
   was actually placed, so "every subset at every phase" is measured and not assumed: the first
   version of this test caught the reserve's own breadth-before-depth truncation hiding the
   four-deep subsets while the per-subset allowance was left at `EMIT_PROFILE_RESERVE`, and the
   allowance is set to the number of subsets **in the test** — nothing in production moved.
   * **Red on the base rule, final text:** `every deep coordinate stayed below the subset's
     narrowest width, so the wider members' own lists are unreachable in every phase (widths
     [160, 93, 40, 160])`.
2. **`a_deep_subsets_coordinates_are_not_a_one_parameter_family`** — for every two-member subset of
   differently wide slots, over **one full period of the subset's own draw** (the `lcm` of the two
   spans, recomputed from the widths), the wider member's coordinate is **not** a function of the
   narrower one's. Under one shared modulus both coordinates are affine in the phase with the same
   modulus, so the narrower member's value determines the wider's exactly and the pair traces a
   rank-1 curve of the index-tuple space; under per-member spans it does not.
   * **Red on the base rule, final text:** `the wider member's coordinate of [0, 1] in [160, 93] is
     a function of the narrower one's, so over one full period of the subset's own draw the pair
     traces a one-parameter family instead of a two-parameter support`.
   * One narrowing, recorded because it was measured rather than assumed: the claim is keyed by the
     **narrower** member, not asserted in both directions. Keyed by the wider member the map is
     injective whenever that member's own period is the joint period — 150 for `[160, 93]` — so the
     reverse direction would be a statement about the test's arithmetic rather than about the rule.
     The test says so in a comment instead of quietly asserting the vacuous half.

Both were run **red on the base rule with their final text** (the production hunk reverted by hand,
the tests untouched) and **green on the P1 rule**. No example token appears in either assertion, and
`no_phrase_hard_coding` stayed 9/9 with both tests in the tree.

## 5. What P1 actually costs: the reserve's depth fence goes red

This is the finding that decides the item, and it is not a budget question.

The reserve proposes at most `reserve` tuples per segmentation and stops at
`out.len() >= reserve`; `build` then keeps a tuple only if its **additive substitution cost** fits
`total_budget`. P1 leaves the first rule exactly where it was and changes only *which* indices the
sweep proposes — and a per-member span reaches that member's own last index, **159**, where the
shared narrowest-member span capped every deep coordinate at **92** at the decisive fund. A
four-deep tuple therefore arrives with four coordinates drawn from 10…159 instead of 10…92, its
substitution cost rises with them, and enough of those tuples no longer fit `total_budget` to be
emitted. The global ceiling stays saturated at 16,384 either way: the *ceiling* was never the wall,
the per-tuple cost bound is.

Measured with a `#[cfg(test)]`, `#[ignore]`d probe over the twelve targets of the fence itself
(`DEEPEST_PROFILE_COORDINATES`, `DEEPEST_PROFILE`, `SPENT_EMISSIONS`, `SPENT_POPS`, wall clock),
same harness on both trees:

| target | words | base depth | **P1 depth** | base emissions | P1 emissions |
|---|---|---|---|---|---|
| a whole lot of trouble | 5 | 4 | **3** | 16,384 | 16,384 |
| he was a big fat man | 6 | 4 | 4 | 16,384 | 16,384 |
| what are you going to do | 6 | 4 | **3** | 16,384 | 16,384 |
| there is no way to know | 6 | 4 | 4 | 16,384 | 16,384 |
| the cat sat on the mat | 6 | 4 | 4 | 16,384 | 16,384 |
| when the rain finally stopped | 5 | 4 | 4 | 16,384 | 16,384 |
| you can do it yourself | 5 | 4 | 4 | 16,384 | 16,384 |
| an old man in a big hat | 7 | 4 | **3** | 16,384 | 16,384 |
| we should have told her | 5 | 4 | 4 | 16,384 | 16,384 |
| in the middle of the night | 6 | 4 | 4 | 16,384 | 16,384 |
| put it back on the shelf | 6 | 4 | 4 | 16,384 | 16,384 |
| they are going to be late | 6 | 4 | 4 | 16,384 | 16,384 |

**Nine of twelve unchanged; three of twelve regress from a four-deep placement to a three-deep
one.** On base all twelve are 4. The fence
`a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` derives its expectation from the
targets' own word counts (the shortest is 5 words, so 4 is derivable) and is **green at base, red
on the P1 tree**:

```
"a whole lot of trouble" (5 words): the reserve placed a tuple deep in 3 slots, and a target
of 5 words admits a tuple deep in 4. The reserve's placement depth is bounded by something
other than the traversal's own floor.
```

Full suite on the P1 tree: `cargo test --release --lib` = **76 passed, 1 failed** (that fence);
`tests/emit_coverage` 4/4, `tests/approx_determinism` 4/4, `tests/exact_determinism` 1/1,
`tests/no_phrase_hard_coding` 9/9, `tests/corpus_integration` 12 passed / 1 failed (the preserved
red). **The tree is not green on the P1 rule, and the item requires it to be.**

The repair is fenced out three times over:

* the per-tuple cost bound and `total_budget` are **scoring** — outside "reserve tuple construction
  only", and `w-3f8c62` owns that surface concurrently;
* raising the budget to rescue the dropped tuples is forbidden twice over (the map's priced rows,
  and the item's own "no budget increases to rescue a regression") — and it would be buying a
  ~5e-4 head lift with a budget increase, which is a bad trade on its face;
* the only in-scope alternative, drawing a *shallower* index from the member's own list so the cost
  bound still fits, is not P1: it re-imposes a shared cap across the members' spans — the very
  confinement P1 exists to remove — and it makes property (i) red again by construction.

So the choice is not "P1 or nothing". It is **P1 plus a decision about the cost bound**, and that
decision belongs to `w-3f8c62`'s front, not to this one. §3 of the obstruction map is empty, and
this is the point where that stops being a scheduling fact and becomes a scope question for a
human.

## 6. The reach arithmetic, reported so nobody mistakes this for a reach fix

**P1 is reach-null, and the report's number stands: the canonical case-2 four-deep cell goes from
unrepresentable to ≈1.2e-7 likely — one specified cell in ~8.2 million — which is nowhere near
reachable.** On the map's decisive fund `[160, 7, 160, 160, 93]`:

* **Before:** every 4-subset contains slot 1 (width 7) or slot 4 (width 93), so
  `max(narrowest) = 93` and **no deep coordinate above 92 is representable in any 4-deep subset**,
  against a needed slot-3 position of 99 / 119 / 136 on the three recorded heads and units. All
  three are excluded; the verdict is unit-independent.
* **After:** the subset `{0,2,3,4}` can name `150·150·150·83 = 279,562,500` cells instead of
  `83^4 = 47,458,321`, so the same cell goes from *impossible* to
  `34 / 279,562,500 = 1.22e-7`. The draw count is fixed by the allowance and does not move.
* **The wall is still the draw count:** 2,895 reserve tuples against
  `160·7·160·160·93 = 2,666,496,000` cells = **9.21e5×** short; the entire saturated 16,384-emission
  ceiling spent on the reserve = **1.63e5×**; a coupon-collector cover of the fixed frame
  ≈ **1.6e8×**.
* **Measured on the P1 tree, and this is the point:** the canonical case-2 clue is still **absent
  from the pool** (`probe rank absent`, `score NaN`, exactly as at base), the canonical per-slot
  indices are unchanged, and the red `approximate_finds_classic_madgab_resegmentation` fails with
  the same twelve top-50 phrases. Representability moved by six orders of magnitude; reach moved by
  **zero**.

**Nothing in this front is a case-2 reach fix, and this report must not be read as one.** The
item's successor rule, restated: joint-coordinate coverage in the reserve sweep is a *free, general,
phrase-free* coverage improvement that is **reach-null by 5 to 6 orders of magnitude**; re-opening it
as a *reach* coordinate requires a deep-slot product within `~D·ln D` of the draw count, i.e.
`≤ ~1.2e5` cells at `D = 16,384`, against `83^4 = 47,458,321` today. P1 enlarges the frame; it does
not shrink the shortfall. And the reason this branch lands nothing is *not* the reach arithmetic —
it is the depth fence in §5.

## 7. Interaction with `w-3f8c62` (one paragraph, as required)

`w-3f8c62` owns the objective and its structural bounds — the scoring axis, the parsimony weight,
and the red fence `head_not_worse_than_pool` — while this front touched only the reserve's tuple
*construction*, so on paper the two are disjoint and should compose: P1 changes which index tuples
the reserve proposes, `w-3f8c62` changes how candidates are ranked, and neither reads the other's
inputs. The measurements agree with that and add one warning. P1 does not move
`head_not_worse_than_pool` at all (still red on 10/10, the two named negatives at `−0.0077` and
`−0.0184`, essentially unchanged), so it neither helps nor hinders that front's headline fix, and
P1's own signal (`+0.0004` / `+0.0006` head lift, pool `−0.08%` / `+0.016%`, green case pinned at
rank 27) is two to three orders of magnitude smaller than the `+0.03` to `+0.08` lifts that front
is chasing on its candidates (REPORT-9b4a15 §4), so the two do not compete on evidence and a
coordinator may take them in either order. The one genuine coupling runs the other way: **P1's depth
regression is a cost-bound problem, and the cost bound is that front's surface.** If `w-3f8c62`
lands a scoring/bound change under which deep-coordinate tuples are affordable within
`total_budget`, P1's fence regression disappears with no further work here and the patch in this
report becomes landable as-is; if that front leaves the cost bound alone, P1 stays a **HOLD** on
this surface however often it is re-measured. That sequencing fact should be in front of the
coordinator before either front is integrated, and it is why this report recommends handing P1 to
`w-3f8c62`'s successor as a joint item rather than landing P1 alone.

## 8. Standing facts this front did not touch

* The blocker `approximate_finds_classic_madgab_resegmentation` is red at base and on the P1 tree:
  not chased, not re-pinned, and no expectation in any test file is proposed for change.
* No phrase-specific hard-coding was added: no sentence, clue, word list, per-input branch or
  special case. `no_phrase_hard_coding` is 9/9 on the P1 tree. The only literals this front wrote
  are the width vectors in the two property tests (`[160, 93]`, `[160, 40, 93]`, …) and the twelve
  multi-clause target strings of the scratch probe, all inside a test module, and the probe is in
  neither the patch nor the branch.
* No budget was raised anywhere, on any surface, to rescue anything.
* `cargo fmt`, `cargo fmt --check`, `cargo clippy`, doctests and `wasm32` builds cannot run on this
  host (see [../environment-notes.md](../environment-notes.md)) and are **not** claimed. Commits
  used `-c commit.gpgsign=false`, and the child branch is pushed with an explicit refspec.
* `main` was not touched and nothing was self-merged. The concurrent front in
  `/workspace/madgab-parsim-3f8c62` and `/workspace/madgab-parsimony-9b4a15` were not read from
  or written to.
* Host note recorded rather than hidden: `cargo test --release --lib` at **default** parallelism is
  SIGKILLed on this host (the same failure mode the item records for `corpus_integration`), so the
  suites were run with `-- --test-threads=2`: 75 passed / 0 failed / 12 ignored on this branch's
  `docs/`-only tree, 76 passed / 1 failed on the P1 rule.

## 9. Verdict

**HOLD.**

* The residue is real, general, phrase-free and **free exactly as priced**: 0 extra pushes, 0 extra
  heap entries, 0 extra emissions (16,384/16,384 saturated on all twelve targets, base identical),
  0 extra frontier, 0 extra bound evaluations, no new state, wall clock unchanged, pops within
  +0.27% on every target.
* Its head-lift effect is **+0.0004** on case 2 (off −0.0180) and **+0.0006** on case 1 (off
  +0.0609); `wreck a nice beach` holds at pool rank **27** with a bit-identical score; all three
  `approximate_pool_reaches_*` guards stay green; `no_phrase_hard_coding` stays 9/9.
* Its reach effect is **zero**, exactly as predicted and exactly as required to be reported: the
  canonical 4-deep cell goes unrepresentable → **≈1.2e-7** likely (one cell in ~8.2 million of a
  279,562,500 frame fed by 34 draws), the canonical clue is still absent from the pool, the case-2
  red is untouched. **This is not a reach fix.**
* And it **regresses the green fence
  `a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` on 3 of its 12 targets**
  (four-deep placement → three-deep), because per-member spans reach index 159 where the shared
  span capped at 92 and `build`'s additive cost bound then drops the tuple. The repair is
  scoring-side: fenced out of this item, owned concurrently by `w-3f8c62`, and not buyable with a
  budget.

**Implementation and evidence are preserved, not discarded.**
[w-b7d4c1-p1-per-member-modulus.patch](w-b7d4c1-p1-per-member-modulus.patch) is the complete
`src/lib.rs` diff — P1 plus the two property tests — and applies clean to `1923bbf`. A successor
that is allowed to touch the cost bound, or that inherits `w-3f8c62`'s landed scoring change, can
apply it, re-run `cargo test --release --lib`, and expect the two property tests green, the three
pool guards green, rank 27 held, and the depth fence to turn with whatever makes deep tuples
affordable. **Successor rule:** if the cost bound has not moved by the time this is picked up, do
not re-measure this front — the numbers above do not change with the head, and the answer is still
`HOLD`; go instead to a human scope decision, because §3 of the obstruction map is empty and this
is the residue that says so.
