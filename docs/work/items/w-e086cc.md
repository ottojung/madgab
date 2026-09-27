---
work_item: true
id: w-e086cc
state: working
priority: high
owner: coord-5f31 (reconciliation pass 2026-09-27T21:46Z-21:50Z: claimed and launched immediately as the successor that agent-3f8c62's terminal HOLD names verbatim. Front agent-e086cc created 21:50Z in /workspace/madgab-e086cc [madgab-e086cc] off post-milestone-acceptance at 106afc5, with docs/work/REPORT-3f8c62.md reachable read-only from /workspace/madgab-parsim-3f8c62 and the landed C1d variant re-measurable from the agent-local branch scratch-3f8c62-landed (514ed91), which the remote pre-receive hook declines to accept so it stays local. Left RUNNING. The concurrent front agent-3f8c62 is still live but in terminal report assembly at 0 commits past base, so the two do not contend on a file)
updated: 2026-09-27T21:50:00Z
branch: madgab-e086cc
worktree: /workspace/madgab-e086cc
opened_by: coord-5f31 (reconciliation pass 2026-09-27T21:46Z-21:50Z)
source_items: docs/work/REPORT-3f8c62.md section 4 (the isolation table) and section 7 handoff bullet 4
predecessors: w-3f8c62 (HOLD, priced, integrated), w-9b4a15 (done, priced the axis), w-1c7d40 (done, owns the reserve-depth fence this item must re-derive)
base: post-milestone-acceptance at 106afc5 (pushed)
---

# Re-derive the search-coupled fences, because every objective weight reaches the search's keys

## Goal

Establish which of the pool-reach and reserve-depth fences are **sound properties** of the
search — true for any objective, so they must survive every weight vector — and which are
**incidental to one weight vector** and must instead be re-derived as part of it. Then express
the sound ones as weight-free properties, with red/green tests at the library boundary.

This is the front `REPORT-3f8c62.md` §7 hands off verbatim, and it is the precondition for
landing any objective change at all, including the word-count parsimony axis.

## Why this front exists now

`REPORT-3f8c62.md` §4 is the durable finding. Its isolation table, measured one variant at a
time with the rest of the tree at HEAD, shows that

- a term in the final scorer alone is a pure scoring change and the search does not move
  (row 1, fence green), and it cannot ship anyway because `span_score_bound` then understates
  the score it bounds and `structural_bounds_dominate_the_real_scorer` goes red (row 2, red);
- `span_score_bound` is itself the search's pruning key — the only `span_bound` call, and the
  `NEG_INFINITY` it returns when no completion survives is a discard — so **the repair the
  previous item asked for is itself a search change**;
- therefore `RHYTHM` and `SHAPE` redden the reserve-depth fence on their own with no axis
  present at all (rows 7 and 8), because they are read by the walk's own best-first heap key
  `bound(prefix)` at `src/lib.rs:1920-1939`.

That last row is the load-bearing one and it has never been stated as a general property: in
this search **no objective weight is search-neutral**. Every weight is read twice — once by
the final scorer and once by the walk's heap key. `REPORT-9b4a15.md` §3 asserted this as an
unexamined caveat; §4 of the new report decomposes it.

Eight fences were green on HEAD and red under the landed C1d vector. Until each is classified,
no weight vector can be integrated, and the previous four fronts each ended in HOLD for exactly
this reason. Classifying them is the general, phrase-free work that unblocks the rest.

## What to do

1. Read `docs/work/REPORT-3f8c62.md` (readable at `/workspace/madgab-parsim-3f8c62/docs/work/REPORT-3f8c62.md`; it is the untracked terminal report of w-3f8c62). The landed C1d variant is on the local branch `scratch-3f8c62-landed` (`514ed91`) for re-measuring §4's rows without re-deriving the patch; the remote declines that branch, so treat it as local-only and never integrate it.
2. For each of the eight fences, decide by measurement whether it is **weight-independent** (red for structural reasons that no weight vector fixes — then it is a sound property and the axis is wrong) or **weight-coupled** (green at some achievable weight vector — then it is incidental and must be re-derived together with the vector). The reserve-depth fence, the three `approximate_pool_reaches_*` fences, `a_lattice_alignment_can_be_absent_from_the_production_pool`, and `approximate_output_is_locked` are the load-bearing set.
3. State the general property as a **weight-free** test where one exists. `a_lattice_alignment_can_be_absent_from_the_production_pool` going red because `hid` leaves the pool is the most alarming row: if a lattice alignment can be *pruned* by an objective weight, that is a reach property, not a scoring one, and the weight-free form of it is worth a test on its own.
4. Do NOT weaken, delete or `#[ignore]` a fence to make a vector landable, and do not touch `coverage_tuples`, `sweep_index`, or the `EMIT_PROFILE_*` constants — those surfaces are priced by w-2f1c03 and w-b7d4c1. Do not re-open any row of `docs/work/OBSTRUCTION-MAP.md`.
5. Rules that must hold: no phrase, clue, word or substring special case anywhere, in particular nothing keyed on `hid`, `wreck a nice beach` or `Hits Justice Dupe Hid Came`; the fences live in `src/lib.rs` and `tests/`; every property gets a red/green test expressed as external behaviour.

## Completion criteria

- Each of the eight fences is classified weight-independent or weight-coupled, with the measurement that decided it, in `docs/work/REPORT-e086cc.md`.
- The weight-free general property (or the proof that none is expressible for a given fence) is written down, with a red/green test for each one that can be expressed.
- Terminal verdict is exactly one of INTEGRATE or HOLD. INTEGRATE means a **fence-rederivation plus the smallest objective change that keeps every sound fence green**, `cargo test --release` no worse than 75 passed / 0 failed / 12 ignored on the item's own fence set, and no phrase-specific code. HOLD means the classification table plus the exact unblocking condition.
- Push `docs/work/REPORT-e086cc.md` and this item's close-out on `madgab-e086cc`, whether the verdict is INTEGRATE or HOLD. Probes go on a separate unpushed scratch branch. Never self-merge, and never land anything on `main`.

## Non-goal

Canonical case-2 reach. `REPORT-3a8c05.md` §3's weight-free bound (rank ≥ 1,127) says no monotone
objective reaches it, and `REPORT-3f8c62.md` §7 says so again: do not read this front as
progress on `approximate_finds_classic_madgab_resegmentation`. That test is red at base and
stays red here.

## Handoff

Opened and launched 21:50Z by coord-5f31 off integrated HEAD `106afc5`. Single prompt, no
steering. The concurrent front agent-3f8c62 is in terminal report assembly at 0 commits past
base and will be integrated by a later pass as a docs-only priced HOLD.
