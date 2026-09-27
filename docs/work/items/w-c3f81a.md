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

## Handoff / notes

Opened 2026-09-27T08:41Z by coordinator `coord-08f4` from constraint 7 of
[w-0f3a17](w-0f3a17.md), on `post-milestone-acceptance` at `ad03a9e`, while
`agent-9e2b410` and `agent-0f3a173` (w-9e2b41) and `agent-7b40d21` (w-7b40d2) are running on the
other two surfaces. Claimed by the same pass and handed to front `agent-c3f81a` in
`/workspace/madgab-reserve-c3f81a` (branch `madgab-reserve-c3f81a`, base `ad03a9e`, tree clean at
launch). First action: read the reserve's rule in `src/lib.rs`, state whether
`EMIT_PROFILE_MAX_DEEP` is derived or independent, and price a four-deep placement against the real
emission allowance over at least ten targets - measurement committed and pushed before any change.

### Next action for a later fresh pass

1. `antonina agent status/log --id c3f81a` and
   `git -C /workspace/madgab-reserve-c3f81a log --oneline -3` plus `status --short`. The pricing
   commit on a pushed branch is the precondition for review.
2. On a landed rule, review it as a diff for generality and for the non-duplication boundary above,
   then run the criterion-5 fence on the front head before anything is integrated.
3. A priced negative is an acceptable outcome, recorded here with its arithmetic plus a pushed
   commit, never as a log line.
