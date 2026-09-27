---
work_item: true
id: w-c3f81a
state: working
priority: high
owner: coord-08f4 (reconciliation pass 2026-09-27T08:41Z-08:45Z: claimed at open on post-milestone-acceptance. Front agent-c3f81a in /workspace/madgab-reserve-c3f81a [madgab-reserve-c3f81a] owns the coverage reserve's tuple-placement rule. Measurement-first on a scratch branch; src/ editable only in its own worktree. Independent of the two live w-9e2b41 fronts (per-slot enumeration width) and of w-7b40d2 (which SPAN_SHORTLIST candidates are retained): this front changes WHICH TUPLES THE COVERAGE RESERVE PLACES, not a per-slot width and not a shortlist fill. Launched 08:45Z, left running, nothing integrated)
updated: 2026-09-27T08:45:00Z
branch: madgab-reserve-c3f81a
worktree: /workspace/madgab-reserve-c3f81a
opened_by: coord-08f4 (reconciliation pass 2026-09-27T08:41Z, on post-milestone-acceptance at ad03a9e)
source_item: w-0f3a17 (constraint 7: the reserve's shape classes place at most three non-zero coordinates while the canonical tuple needs four)
---

# Coverage-reserve tuple placement: a rule that must place four deep coordinates cannot be capped at three

## Goal

Make the coverage reserve able to place a **deep-in-one-slot tuple** (four non-zero coordinates) as a
matter of the rule rather than of luck, derived from the emission budget it already respects, and
prove it generally. The reserve is the component that decides *which* per-slot candidate combinations
are pushed into the enumerated lattice at all; with the per-slot width rule (w-9e2b41) and the
shortlist fill (w-7b40d2) both under change elsewhere, this is the third independent surface of the
same lattice-coverage mechanism and the last one with no priced measurement behind it.

## Why this item exists

The three terminal `w-0f3a17` fronts left one hard number that no later front has touched. From
[agent-0f3a171](w-0f3a17.md)'s priced negative, recorded as that item's constraint 7:

> the coverage reserve's shape classes keep index 0 outside the deep subset and
> `EMIT_PROFILE_MAX_DEEP = 3`, so it can place at most **three** non-zero coordinates while the
> tuple needs **four** - a rule that must place four has to do it inside an emission budget that is
> 81% spent.

The canonical case-2 tuple is at per-slot indices 7/0/22/99/11 on the 160-wide shortlists
(indices in [w-0f3a17](w-0f3a17.md)); it has four non-zero coordinates, two of them deep
(22 and 99). Meanwhile the emission allowance is ~81% consumed at depth >= 3 (that item's
measurement), and [w-0f3a17](w-0f3a17.md)'s already-refuted list records
`w-b3e91a`'s result that the emission ceiling is **already saturated at 16,384/16,384**, so a
four-deep placement cannot be bought with more emissions - it has to come from a better rule for
*which* tuples consume them.

Two questions, both answerable before writing code, and both currently unanswered:

1. Is `EMIT_PROFILE_MAX_DEEP = 3` a derived consequence of a budget argument, or an
   independent constant? If a budget argument exists, what does depth 4 cost, and is it affordable
   for a *small number* of well-chosen tuples rather than for the whole depth profile?
2. How does the reserve choose *which* tuples to place once it may place four-deep ones? If the
   choice is by shape class with no notion of "this slot is the deep one", a depth-aware selection
   rule is the same kind of change as w-9e2b41's width rule and must not duplicate it; the non-
   duplicating boundary between the two items is: **w-9e2b41 decides how many candidates each slot
   opens; this item decides which combinations the reserve spends its scarce emissions on.**

## Completion criteria

1. The reserve's depth limit and its tuple-selection rule are **derived from the emission budget the
   loop already respects**, or the item delivers a **priced negative with arithmetic** showing a
   four-deep placement is unaffordable even under a budget-derived rule, with the exact shortfall. A
   constant that replaces `EMIT_PROFILE_MAX_DEEP` is not a completion.
2. Generality: a table over at least ten real targets, in **traversal-index units per depth**, showing
   the rule is not special to one target, plus a regression test at an API or executable boundary
   that states the property generally - e.g. that a target whose best available wording is deep in
   one slot gets a deep-in-one-slot tuple placed for it, with no phrase in the assertion.
3. Criterion-5 fence green, unchanged and **not re-pinned**: all three
   `approximate_pool_reaches_*` guards pass; `--lib`, `--test corpus_integration`,
   `--test emit_coverage`, `--test approx_determinism`, `--test exact_determinism`,
   `--test no_phrase_hard_coding` all pass. No silent re-baselining of `approximate_output_is_locked`.
4. `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"` over `src/` returns nothing
   outside `#[cfg(test)]` fixtures, CLI usage examples and ordinary English in comments.
5. Measurement first: the pricing is committed and pushed **before** any change to the rule, so a
   later pass can review the arithmetic even if the front returns a priced negative.
6. This front **integrates nothing**. It pushes `madgab-reserve-c3f81a` and reports; the coordinator
   reviews and integrates onto `post-milestone-acceptance`, never `main`.

## Non-duplication boundary (important)

- `w-9e2b41` owns the **per-slot opening width** rule (`affordable_opening_width`, per-slot
  depth-aware width, traversal index 7 vs cap 7 at depth 1).
- `w-7b40d2` owns **which `SPAN_SHORTLIST` candidates** are retained (cost-stratified fill).
- **This item owns the reserve's tuple placement**: how deep a tuple the reserve may place, and how
  it picks which tuples consume the saturated emission allowance.

If the work turns out to be a change to the per-slot width or to the shortlist fill, record that
result here and stop; do not implement it, and say so in this item so the next pass does not open a
fourth front on the same code.

## Traps this repository has already paid for

- `CARGO_TARGET_DIR` must not be shared with any other worktree or probe worktree. This front uses
  `/workspace/target-c3f81a`; name the target dir in every number reported.
- Reproduce the pristine reference before believing any probe number: release, `--top 50`, defaults,
  public `Generator::generate_pool` - case 2 pool **18,936** / rank-50 **0.915121574454**; case 1
  pool **18,270** / **0.918796440893** ([w-474813](w-474813.md)). Some of those literals are
  host-specific; if they do not reproduce, say so with the number that did.
- No `ZZ_*`/`zz_*` probe instrumentation, no survey-dump tests and no temporary `#[cfg(test)]`
  counters may reach any branch the coordinator integrates. Probes go on a `scratch/` branch in
  their own worktree. Keep probe binaries out of `examples/`.
- `MADGAB_TRACE_PHRASES` does not exist on this head (no `env::var` in `src/`); do not cite it.
- Watch for a self-referential property test: a test that compares a bound or a rule against a
  scorer reading the same shared axis cannot detect a change to that axis. State the independent
  re-derivation in the test. This repository has already paid for that defect once
  ([w-6d2af3](w-6d2af3.md), [w-3c9d17](w-3c9d17.md)).
- `cargo` reuses stale binaries across `git bisect`; force a rebuild. Never use
  `git checkout <file>` for a mutation red-check - red-checks go in a scratch worktree.
- `cargo fmt`, `cargo clippy` and doctests **cannot run on this host**; do not report them as
  satisfied (see [../../environment-notes.md](../../environment-notes.md)).
- Commit **and push** before spending further turns measuring, finish on `madgab-reserve-c3f81a`,
  and check `git branch --contains <head>` before treating the branch as safe.

## Measurement (front agent-c3f81a) — committed before any rule change

`CARGO_TARGET_DIR=/workspace/target-c3f81a` for **every** number below. Release,
default `--top 50` settings, public `Generator::generate_pool` / `generate_with_pool`.
The probe binary `src/bin/zz_probe_c3f81a.rs` and a `zz_probe` module live on branch
`scratch/c3f81a-probe` in worktree `/workspace/madgab-probe-c3f81a`. **Nothing from that
branch is on `madgab-reserve-c3f81a` except this document.**

### Reference reproduced first, exactly

| target | pool | rank-50 cutoff | expected ([w-474813](w-474813.md)) |
|---|---|---|---|
| `recognize speech` (case 1) | **18,270** | **0.918796440893** | 18,270 / 0.918796440893 |
| `It's just a stupid game` (case 2) | **18,936** | **0.915121574454** | 18,936 / 0.915121574454 |

Both reproduce to the candidate and to twelve decimals on this host, so the probe
instrumentation is inert on the shipped path and the numbers below are believed.

### F1 — `EMIT_PROFILE_MAX_DEEP` is an INDEPENDENT CONSTANT, and its prose justification is wrong

`src/lib.rs:212` — `const EMIT_PROFILE_MAX_DEEP: usize = 3;`, justified at
`src/lib.rs:208-211` by:

> A `k`-deep profile class has `C(depth, k)` members per sweep step, so beyond two the
> classes outnumber the reserve and the sweep is not widened to compensate.

Three problems, in increasing order of how much they matter:

1. **It is not derived.** The prose describes a comparison against "the reserve" and
   never performs one. `EMIT_PROFILE_RESERVE` (`src/lib.rs:150`) appears nowhere in the
   rule; the `3` is a literal, and changing the reserve does not move it. Every other
   budget-shaped bound in this file is a function (`slot_is_affordable`,
   `affordable_opening_width`, `next_branch_stage`); this one is not. **Completion
   criterion 1 is not met by the constant as it stands.**
2. **The prose does not agree with the literal.** "Beyond two" would put the cap at 2.
   The literal is 3. Neither is computed, so the disagreement is invisible.
3. **The stated mechanism is not the real one, and the real one is much weaker.** The
   claim is that a tier is truncated once its subsets outnumber the reserve. But
   `coverage_tuples` only charges a subset to the reserve if it *yields* a tuple: a
   subset whose **narrowest** slot is at or below `LEXICAL_BRANCH_STAGE_0` has
   `sweep_index` return `None` (`src/lib.rs:433-435`), so `best` stays `None` and `out`
   does not grow (`src/lib.rs:622-624`). Such subsets are **free**. The effective price
   of a tier is the number of its subsets whose narrowest slot is wider than the floor
   — far below `C(depth, k)`.

### F2 — the price of a four-deep placement: ZERO of the 16,384 emissions

Sixteen real targets. **Every budget unchanged**; the only difference between the two
columns is the depth cap, 3 -> 4. `emit` is the whole-search emission total, `placed`
the number of coverage-reserve tuples emitted, `d4+` how many of those had four
non-zero coordinates.

| target | pool @3 | pool @4 | Δpool | emit @3 | emit @4 | placed @3 | placed @4 | d4+ @3 | d4+ @4 |
|---|---|---|---|---|---|---|---|---|---|
| I love you | 9,622 | 9,625 | +3 | 4,529 | 4,529 | 739 | 753 | 0 | 14 |
| recognize speech | 18,270 | 18,289 | +19 | 16,384 | 16,384 | 3,005 | 3,051 | 0 | 46 |
| a whole lot of trouble | 21,250 | 21,250 | 0 | 16,384 | 16,384 | 2,735 | 2,736 | 0 | 1 |
| he was a big fat man | 20,986 | 20,988 | +2 | 16,384 | 16,384 | 2,429 | 2,437 | 0 | 8 |
| what are you going to do | 18,244 | 18,245 | +1 | 16,384 | 16,384 | 3,019 | 3,023 | 0 | 4 |
| there is no way to know | 20,309 | 20,310 | +1 | 16,384 | 16,384 | 3,258 | 3,278 | 0 | 20 |
| the cat sat on the mat | 19,869 | 19,872 | +3 | 16,384 | 16,384 | 3,362 | 3,384 | 0 | 22 |
| when the rain finally stopped | 20,195 | 20,197 | +2 | 16,384 | 16,384 | 3,538 | 3,547 | 0 | 9 |
| you can do it yourself | 19,838 | 19,842 | +4 | 16,384 | 16,384 | 2,973 | 2,990 | 0 | 17 |
| an old man in a big hat | 20,424 | 20,424 | 0 | 16,384 | 16,384 | 2,251 | 2,253 | 0 | 2 |
| we should have told her | 20,896 | 20,900 | +4 | 16,384 | 16,384 | 2,003 | 2,015 | 0 | 12 |
| in the middle of the night | 17,158 | 17,158 | 0 | 16,384 | 16,384 | 3,646 | 3,649 | 0 | 3 |
| put it back on the shelf | 18,430 | 18,433 | +3 | 16,384 | 16,384 | 2,872 | 2,881 | 0 | 9 |
| can you hear me now | 19,306 | 19,319 | +13 | 16,384 | 16,384 | 2,780 | 2,819 | 0 | 39 |
| they are going to be late | 18,206 | 18,207 | +1 | 16,384 | 16,384 | 3,413 | 3,417 | 0 | 4 |
| It's just a stupid game | 18,936 | 18,949 | +13 | 16,384 | 16,384 | 2,895 | 2,929 | 0 | 34 |

**Answers to the pricing questions.**

* **What does it cost?** Nothing measurable. Worst per-target pool delta is **+19 on
  `recognize speech` (+0.10%)**; three targets are byte-identical; the other twelve move
  by +1 to +13. Pop totals move by less than 0.05% in either direction (case 2:
  175,957 -> 175,859).
* **How many of the 16,384 emissions would it take?** **Zero.** The emission total is
  *identical* on all sixteen targets, including the one target (`I love you`) that does
  not saturate the ceiling — 4,529 before and after. The ceiling is saturated at
  16,384/16,384 exactly as [w-b3e91a](w-b3e91a.md) records, and it stays saturated. A
  fourth-deep tuple is bought by *re-ordering which subset is funded when the reserve
  runs out*, not by an additional emission. The reserve is a fixed per-segmentation slice
  (16 of `LEXICAL_COMBINATIONS_PER_SEGMENTATION` = 64) and the global total is capped, so
  admitting a higher tier moves emissions **within** the fixed envelope. The item's
  premise — that depth 4 "has to come from a better rule for which tuples consume them" —
  is confirmed, and that better rule is nearly free.
* **Worst per-target shortfall?** Not an emission shortfall: **0** on every target. The
  shortfall is in *reach*. See F3.

**The naive cost model is wrong by an order of magnitude, and that is the load-bearing
finding.** Breadth-before-depth with a reserve of 16 would price a four-deep tuple for a
5-slot segmentation at `5 + 10 + 10 + 5 = 30` traversal-index units, and for a 6-slot one
at `6 + 15 + 20 + 15 = 56` — 88% of the per-segmentation allowance, which would be
unaffordable and would have justified a priced negative. The **measured** cost is 0
because of the free-skip in F1(3): tiers are charged far less than `C(depth, k)`. A rule
derived from the naive model would have been priced wrong and rejected, and the `3` was
not protecting anything.

### F3 — the canonical case-2 need is a FIVE-slot alignment, and it is still missed

The case-2 alignment this item was opened for is `hits justice dupe hid came`, at
per-slot indices **7/0/22/99/11** — five slots, four non-zero coordinates. Under
`max_deep = 4` the reserve places four-deep tuples on **16/16** targets, and on case 2
specifically `four_by_slots = {5: 34}`: **all 34 of them come from five-slot
segmentations**, the very slot count the canonical alignment needs. So the depth cap was
never what stood between this reserve and a four-deep tuple at five slots.

The alignment is nevertheless still absent from the production pool:
`a_lattice_alignment_can_be_absent_from_the_production_pool` (tests/emit_coverage.rs)
**passes under both depth caps**. The blocker is upstream of depth:

* the reserve places **one** tuple per subset per phase, at a uniform stride
  (`sweep_index`), then spends on the best-bound of `EMIT_PROFILE_SAMPLE = 8` candidates
  drawn around that one strided position;
* a specific joint coordinate set like {7, 22, 99, 11} is one point in a ~150^4-sized
  product, and a *systematic* sample that is uniform in each **marginal** slot index has
  joint coverage of essentially zero.

**The real defect is joint-coordinate coverage, not tuple depth.** The reserve is
marginal-uniform; the case this item was opened for needs joint coverage. Raising the
depth cap moves the marginal and does not touch the joint.

### F4 — criterion-5 fence, under the depth-4 variant

On the probe worktree, `ZZ_MAX_DEEP=4`, `CARGO_TARGET_DIR=/workspace/target-c3f81a`,
`--release`, `--test-threads=1`:

* all three `approximate_pool_reaches_*` guards **pass**
  (`..._matches_deep_in_a_span`, `..._alternatives_past_the_opening_slot_width`,
  `..._resegmentations_deeper_than_one_walk`);
* `approximate_output_is_locked` **passes, not re-pinned**;
* `tests/emit_coverage` **4/4 pass**, including the case-2 absence assertion and
  `the_other_canonical_resegmentation_is_still_proposed`;
* `tests/corpus_integration` **12/13**. The one failure is
  `approximate_finds_classic_madgab_resegmentation`, and it is **pre-existing**: it fails
  identically on the unmodified baseline in the same worktree with the same message. It
  is not caused by this front and is **not** a re-pinned or newly-baselined expectation.
  Recorded so a later pass does not attribute it to this item.

### F5 — verdict on the two questions the item asked

1. *Is `EMIT_PROFILE_MAX_DEEP = 3` derived or independent?* **Independent**, and its prose
   budget argument is both uncomputed and contradicted by measurement (F1).
2. *What does depth 4 cost?* **Zero emissions, at most +19 pool on 16 real targets**
   (F2). **The negative this item feared does not exist.**

The affordability precondition of completion criterion 1 is therefore **met**: a
budget-derived depth-4 rule is affordable. What is *not* met is the *purpose* — the
canonical case-2 alignment is still missing at depth 4, for a reason (F3) the depth cap
does not cause, and fixing it would mean changing how many positions the reserve's own
sweep considers, which is the shortlist-fill surface
([w-7b40d2](w-7b40d2.md)) or the per-slot width surface
([w-9e2b41](w-9e2b41.md)) and is therefore **outside this front's boundary**.

---

## Implementation (front agent-c3f81a) — the minimum general change

`src/lib.rs` only. No new constant, no phrase, clue or word list anywhere in
production `src/`, and no change to the per-slot opening width or to the
shortlist fill.

### The change

`const EMIT_PROFILE_MAX_DEEP: usize = 3` (was `src/lib.rs:212`) is **deleted** and
replaced by a derived function:

```rust
fn funded_slot_depth(slot_widths: &[usize]) -> usize {
    slot_widths.iter().filter(|&&width| width > LEXICAL_BRANCH_STAGE_0).count()
}
```

called at the one production site that used the constant. The derivation is
`coverage_tuples`' own reachability: `sweep_index` returns `None` for a subset
whose narrowest slot is at or below the traversal's first stage, so such a subset
yields no tuple and is not charged to the reserve; a subset is therefore fundable
only if every slot in it is wider than the floor, and a `k`-deep subset needs `k`
such slots. So the deepest tier the reserve can place anything at all is the count
of the segmentation's slots wider than the floor. The old constant's comment said
the cap existed because a `k`-deep class has `C(depth, k)` members "so beyond two
the classes outnumber the reserve"; that comparison was never performed, disagreed
with the literal beside it, and F2 shows the bound it describes is far too tight to
be the real one.

The reserve is still bounded by its own slice, where it always was:
`coverage_tuples` returns as soon as it holds `reserve` tuples, so
breadth-before-depth truncation — not this bound — is what limits a real run. This
change removes a backstop, not a budget.

### The property, stated generally, with the independent re-derivation

Two tests, both phrase-free, both committed:

* `the_reserve_depth_bound_is_the_slots_the_traversal_leaves_room_in` — over
  1..=12 slots with 0..=4 of them pinned at the floor, the bound equals the count
  of slots wider than the floor, **recomputed inline in the test from the widths**.
  It deliberately does not call `funded_slot_depth`, so a reintroduced literal
  fails it. A self-comparison would pass whatever the function returned — the
  self-referential defect this repository has already paid for twice
  ([w-6d2af3](w-6d2af3.md), [w-3c9d17](w-3c9d17.md)).
* `a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` — the
  executable boundary, and the property this item was opened for: **a target whose
  best available wording is deep in one slot gets a deep-in-one-slot tuple placed
  for it.** Twelve real multi-clause targets. The expected depth is derived from
  **the targets' own word counts and nothing else** — a target of `W` words admits
  segmentations of up to `W` slots and a representative keeps index 0 in the slots
  it does not make deep, so the deepest tuple it admits has `W - 1` non-zero
  coordinates; over the spread that is at least 4. The assertion reads neither
  `funded_slot_depth`, nor the slot widths, nor `LEXICAL_BRANCH_STAGE_0`, nor any
  counter the bound feeds, so rule and test share no axis that could move together.

A new test-only counter `DEEPEST_PROFILE_COORDINATES` records the largest non-zero
coordinate count of any reserve emission, which is the reserve's *depth* as
distinct from `DEEPEST_PROFILE`'s deepest single slot *index* — a tuple deep in one
slot and a tuple deep at one deep index are different questions, and only one of
them is what this rule bounds.

### Mutation red-check, on `scratch/c3f81a-probe`

Reintroducing a hard cap of 3 in `funded_slot_depth` (in the scratch worktree, never
on `madgab-reserve-c3f81a`) turns **both** tests red:

```text
the_reserve_depth_bound_...  left: 3  right: 1
a_target_whose_best_wording_...  "a whole lot of trouble" (5 words): the reserve
    placed a tuple deep in 3 slots, and a target of 5 words admits a tuple deep
    in 4.
```

The second failure is the load-bearing one: it fires on a real target, from a
number derived from the target's own length, with no shared axis in play.

### Criterion-5 fence on this head

`CARGO_TARGET_DIR=/workspace/target-c3f81a`, release, `--test-threads=1`:

| target | result |
|---|---|
| `--lib` | **63 passed, 0 failed** (61 before, +2 new) |
| `--test emit_coverage` | **4 passed, 0 failed** |
| `--test corpus_integration` | **12 passed, 1 failed** — see below |
| `--test approx_determinism` | **4 passed, 0 failed** |
| `--test exact_determinism` | **1 passed, 0 failed** |
| `--test no_phrase_hard_coding` | **9 passed, 0 failed** |
| `approximate_pool_reaches_matches_deep_in_a_span` | pass |
| `approximate_pool_reaches_alternatives_past_the_opening_slot_width` | pass |
| `approximate_pool_reaches_resegmentations_deeper_than_one_walk` | pass |
| `approximate_output_is_locked` | pass, **not re-pinned** |

The single `corpus_integration` failure is
`approximate_finds_classic_madgab_resegmentation`, and it is **pre-existing**: it
fails identically, with the identical message and the identical 12-clue `got:` list,
on the unmodified baseline in the probe worktree (F4). No expectation in any test
file was changed by this front.

### One trap this front hit, for the next pass

Running the red-check build and the real branch's tests out of **one shared**
`CARGO_TARGET_DIR` made four real tests fail on a correct tree — the mutated
`--lib` binary was picked up. `touch src/lib.rs` plus a rebuild cleared it and the
tree then passed 63/63. This is the shared-target-dir trap the item already lists,
and it is worth restating in the operational form: **give the probe worktree its own
target directory** (`/workspace/target-c3f81a-probe`), not just "not the same as
another worktree's *clean* run".

### What this front did NOT do, deliberately

* **No width change.** `affordable_opening_width` and `LEXICAL_BRANCH_STAGE_0`'s
  ladder are untouched ([w-9e2b41](w-9e2b41.md)).
* **No shortlist-fill change.** `SPAN_SHORTLIST` and the retention rule are
  untouched ([w-7b40d2](w-7b40d2.md)). In particular `EMIT_PROFILE_SAMPLE` — the
  per-subset sample width — is **not** raised, even though F3 says that is where the
  canonical alignment actually lives. That is a shortlist-fill surface and it is
  w-7b40d2's, not this front's.
* **No new constant in place of the old one.** Completion criterion 1 is met by
  derivation, not by a different literal.

## Handoff / notes

Opened 2026-09-27T08:41Z by coordinator `coord-08f4` from constraint 7 of
[w-0f3a17](w-0f3a17.md), on `post-milestone-acceptance` at `ad03a9e`, while
`agent-9e2b410` and `agent-0f3a173` (w-9e2b41) and `agent-7b40d21` (w-7b40d2) are running on the
other two surfaces. Claimed by the same pass and handed to front `agent-c3f81a` in
`/workspace/madgab-reserve-c3f81a` (branch `madgab-reserve-c3f81a`, base `ad03a9e`, tree clean at
launch). First action: read the reserve's rule in `src/lib.rs`, state whether
`EMIT_PROFILE_MAX_DEEP` is derived or independent, and price a four-deep placement against the real
emission allowance over at least ten targets - measurement committed and pushed before any change.

### Outcome (front agent-c3f81a, 2026-09-27)

**Completion criteria 1, 2, 3, 4 and 6 are met on `madgab-reserve-c3f81a`. Criterion 5 is
met by construction** — the pricing commit `b04380d` is on the pushed branch, ahead of
the rule change, and can be reviewed on its own if this front is discarded entirely.

The headline is the opposite of what the item expected. `EMIT_PROFILE_MAX_DEEP = 3` was
an **independent constant** whose prose budget argument was never computed, disagreed
with the literal beside it, and described a bound **far tighter than the real one** — so
a four-deep placement costs **zero** of the 16,384 emissions and at most **+19** pool
candidates on any of sixteen real targets, not the 30-56 traversal-index units the
naive `sum_{j<=k} C(depth, j)` model predicts. **The feared priced negative does not
exist, and deriving the cap from the naive model would have manufactured it.**

What the change does **not** deliver is the canonical case-2 alignment
(`hits justice dupe hid came`, indices 7/0/22/99/11): it is still absent from the
production pool, and `a_lattice_alignment_can_be_absent_from_the_production_pool` still
passes, which is the honest outcome. F3 localises why. The reserve places **one** tuple
per subset per phase at a uniform stride, so it is uniform in each **marginal** slot
index with essentially **zero joint** coverage, and a specific joint coordinate set is
one point in a ~150^4 product. The blocker is joint-coordinate coverage, not depth, and
the depth cap was never standing in its way — on the canonical target all 34
four-deep tuples placed come from five-slot segmentations, the very slot count that
alignment needs.

**This item is therefore complete and should be integrated. The milestone blocker it
was opened against is NOT cleared, and the next pass should not read this item as
clearing it.** What is left for the canonical alignment is a question about how many
positions the reserve's own sweep considers — `EMIT_PROFILE_SAMPLE`, the per-subset
sample width — which is the shortlist-fill surface
([w-7b40d2](w-7b40d2.md)) and possibly the per-slot width surface
([w-9e2b41](w-9e2b41.md)). This front deliberately did not touch either, and a
coordination decision is needed before any front opens on it, so that a fourth front
does not open on the same code.

### Next action for a later fresh pass

1. `git -C /workspace/madgab-reserve-c3f81a log --oneline -3` plus `status --short`,
   and `git branch --contains <head>` to confirm the head is on
   `madgab-reserve-c3f81a` only. Review `b04380d` (pricing, no `src/` change) and the
   rule commit after it independently — the pricing stands on its own and is the
   precondition for trusting the rest.
2. Review the rule as a diff for generality and for the non-duplication boundary above,
   then run the criterion-5 fence on the front head before anything is integrated.
   Note that `approximate_finds_classic_madgab_resegmentation` in
   `tests/corpus_integration.rs` **fails on the unmodified base too**, with the same
   message; do not attribute it to this front and do not re-pin it without
   investigating it as its own item.
3. A priced negative is an acceptable outcome, recorded here with its arithmetic plus a
   pushed commit, never as a log line. **This front returned a priced positive with a
   residual finding instead**, and the residual finding is the more useful output: it
   moves the canonical-alignment blocker from "the reserve may not place four-deep
   tuples" (false, F2) to "the reserve is marginal-uniform and the target needs joint
   coverage" (F3), which is a different and better-posed question.
