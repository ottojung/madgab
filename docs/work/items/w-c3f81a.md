---
work_item: true
id: w-c3f81a
state: working
priority: high
owner: coord-b7e4 (reconciliation pass 2026-09-27T09:12Z-09:32Z: front agent-c3f81a SUCCEEDED with two pushed commits - b04380d (pricing over sixteen targets, standalone) and 1973f05 (the rule: EMIT_PROFILE_MAX_DEEP=3 deleted, replaced by funded_slot_depth derived from the traversal floor). Both are real deliverables, not integrations. The front recommends INTEGRATE and reports a priced POSITIVE: zero emission cost on all sixteen targets, worst pool delta +19 (+0.10%). Its residual finding is the durable one - the depth cap was never why the canonical case-2 alignment is missing; all 34 four-deep tuples placed on that target come from five-slot segmentations, and the real blocker is that the reserve sweeps each marginal index at a uniform stride with essentially zero JOINT coverage. An independent read-only review agent-c3f81b1 was launched 09:19Z in /workspace/madgab-review-reserve over b04380d..1973f05 and left running; 1973f05 changes production src/lib.rs so nothing from that branch is integrated until its INTEGRATE/HOLD verdict lands. Decided here: joint-coordinate coverage in the sweep is the shortlist-fill surface that w-5e2d41 already owns with two live fronts, so NO new front is opened on it. main untouched)
updated: 2026-09-27T09:26:00Z  # corrected 09:29Z pass: the prior 09:31:00Z stamp was future-dated against the host clock
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

## Pass 2026-09-27T09:30Z (coordinator) — front terminal with an INTEGRATE recommendation; independent review launched

`agent-c3f81a` **succeeded** and pushed two commits on `madgab-reserve-c3f81a`:

- `b04380d` — the pricing, over sixteen real targets, **ahead of** the rule so it stands alone on review.
- `1973f05` — the rule: `const EMIT_PROFILE_MAX_DEEP: usize = 3` deleted and replaced by
  `fn funded_slot_depth(slot_widths) = slot_widths.iter().filter(|&&w| w > LEXICAL_BRANCH_STAGE_0).count()`,
  with the constant's own justification shown to be a comparison against the reserve that was never
  performed and that is false as a bound.

The pricing: **zero** emission cost on all sixteen targets (totals identical, including the one unsaturated
target), pops move <0.05%, worst per-target pool `+19` on `recognize speech` (+0.10%), three targets
byte-identical. The naive `Σ C(depth, j)` model would have priced this at 30-56 units — 88% of the
per-segmentation allowance — i.e. **unaffordable**, so deriving the cap from the naive model would have
manufactured a false negative. Reference gate reproduced exactly first: case 1 18,270 / 0.918796440893,
case 2 18,936 / 0.915121574454.

**The milestone blocker is NOT cleared and this item must not be read as clearing it.** The front's own
residual finding, which is the durable result here: the canonical case-2 alignment is still absent, and the
depth cap was **never** the reason. All 34 four-deep tuples placed on that target come from **five-slot**
segmentations — the slot count `hits justice dupe hid came` (7/0/22/99/11) needs. The real blocker is that
the reserve places one tuple per subset per phase at a **uniform stride**, so it is uniform in each
*marginal* index with essentially **zero joint** coverage. That is the same product-vs-linear-budget wall
[w-9e2b41](w-9e2b41.md) just priced, reached from the placement side, and it confirms that the next spend
belongs on joint-coordinate coverage in the reserve's sweep rather than on any width rule.

**Coordination decision, taken here so no later pass opens a duplicate front:** joint-coordinate coverage
in the sweep is the **shortlist-fill surface** (`EMIT_PROFILE_SAMPLE` / span-shortlist fill), which
[w-5e2d41](w-5e2d41.md) already owns with two fronts running. **No new front is opened on it.** When those
fronts report, the sweep-coverage question belongs in whichever of them is still open, or in a successor
opened from their result.

**Review launched, left running:** `agent-c3f81b1` in `/workspace/madgab-review-reserve`
[`madgab-review-reserve`] at 09:19Z, over `b04380d..1973f05` with `170fbb3` as the base, read-only, with its
own `CARGO_TARGET_DIR`. It must confirm the derivation really holds at the call site (that `sweep_index`
returns `None` for any subset whose narrowest slot is at or below the floor and those subsets are not
charged to `EMIT_PROFILE_RESERVE`), check the two new tests are discriminating and not self-referential,
run the criterion-5 fence on the head **and** on `170fbb3` side by side, and return one explicit
**INTEGRATE** or **HOLD**. **Nothing from `madgab-reserve-c3f81a` is integrated yet** — `1973f05` changes
production `src/lib.rs` and the repository's standing rule is that source is integrated only behind an
independent verdict.

**Next action for a fresh pass:** `antonina agent status/log --id c3f81b1`, then
`git -C /workspace/madgab-review-reserve log --oneline -3` and `git branch -a --contains <head>`. On
**INTEGRATE**, merge `madgab-reserve-c3f81a` into `post-milestone-acceptance` (never `main`) and move this
item to `done` with criterion 1 read as "the bound is derived and the four-deep placement is measured at
zero emission cost", explicitly **not** as milestone closure. On **HOLD**, apply its named edits. Either
way, do not re-pin `approximate_finds_classic_madgab_resegeneration` and do not open a sweep-coverage front
while `w-5e2d41` is live.
