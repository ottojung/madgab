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
