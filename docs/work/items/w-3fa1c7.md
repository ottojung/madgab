---
work_item: true
id: w-3fa1c7
state: working
priority: high
owner: agent-3fa1c7 (claimed 2026-09-26T22:59Z by coord-4c02)
updated: 2026-09-26T22:59:00Z
branch: madgab-reserve-scoreorder-3fa1c7
worktree: /workspace/madgab-reserve-scoreorder-3fa1c7
---

# Spend the coverage reserve on tuples ordered by an admissible *score*
# bound instead of by cost

## Goal

Make the approximate traversal's coverage reserve draw its tuples in an
order that maximises expected final score, rather than in cost-strided
index order, as a general behaviour. Success is a landed, general change
with a measured acceptance census — not a demonstration that the canonical
example is unreachable.

## Why this item exists

This is the successor [w-9c4d21](w-9c4d21.md) named as its single next
action, and it is the last unfrozen lever on the enumeration side of the
milestone:

* that front closed the *spend* family with numbers — 24 configurations of
  the reserve's three constants, 4,096 draws against 2,666,496,000 points,
  and 33,635,985 tuples ahead of the target on the traversal's own
  **cost** bound — so "spend more of the reserve" and "derive the
  reserve's constants" are refuted, not merely priced out. Do not re-open
  them.
* the same measurement says the requested wording is rank ~50–100 of the
  pool **by final score** (`0.79990129077367222`, only `0.0978513` below
  the worst visible `--top 50` proposal) and rank ~33.6M **by admissible
  cost**. The emissions are chosen by the cost bound, and that is the only
  thing excluding it. So the lever is the **order** the reserve samples
  in, not the size of the sample.
* [w-1c3e77](w-1c3e77.md) and [w-6f2b18](w-6f2b18.md) independently
  derived, from the traversal and from the pool, that no score-monotone
  *per-segmentation* walk under a per-segmentation budget reaches a
  four-deep coordinate set. The reserve is exactly the budget that is
  **not** per-segmentation, so it is the one place where that argument
  does not apply.

## Scope and collision boundary

This front owns the traversal's **reserve sample and its order**:
`coverage_tuples`, `profile_tuple`, `sweep_index`, `sweep_rate`,
`EMIT_PROFILE_RESERVE`, `EMIT_PROFILE_MAX_DEEP`, and the emission-budget
accounting in `build`. It may read `axes::*`, `select_diverse`,
`structure_wording_allowance` and the per-slot candidate lists to
diagnose, but must not change them.

Forbidden: sweeping or re-tuning any `axes::*` weight or any
`boundary_novelty` form; `src/adjacency.rs` and `ADJACENCY_*`;
`select_diverse`'s admission rule, share cap or `STRUCTURE_FLOOR`; the
per-slot affordability and opening-width hunks of `784deaa` (they are
under review on [w-1c3e77](w-1c3e77.md) and this branch must be rebased
onto them when they land); relaxing or re-baselining either acceptance
test; any phrase-specific case in `src/` or `tests/`, **including doc
comments and string literals**; and any `env::var` production knob,
`eprintln!` probe, or `ZZ_*` / `zz_*` / `MADGAB_*` scaffolding on the
branch.

Contention note: agent `b2e5c4` ([w-b2e5c4](w-b2e5c4.md)) is running in
`/workspace/madgab-representation-b2e5c4` and edits `select_diverse` in
the same file. Do not touch its worktree. The regions do not overlap, and
the coordinator sequences merges: the earlier-merged branch is rebased
onto the other. Report which `src/lib.rs` regions you touched so the merge
order can be chosen.

## Completion criteria

1. **Census first, then design.** On the default release path, measure the
   reserve's current draw for at least six real targets (including both
   canonical targets) — how many tuples it emits, the rank of the
   requested coordinate set under the current cost order, and the same
   rank under the proposed score order. A mechanism that cannot be shown
   to move that rank is a refutation and is an acceptable result.
2. **An admissible bound, argued.** The bound must be admissible in the
   same sense `build`'s cost bound is: it must never exclude a tuple the
   current draw would have emitted, and its precomputation cost must be
   bounded and stated. The search already computes score-shaped suffix
   maxima for its heuristics; say whether they can be reused as-is.
3. **A landed, general change** if a fix is warranted, with the reserve's
   post-change purpose recorded in the item and in code comments. No
   special case for any word, substring or word sequence of either
   acceptance example, in `src/` or `tests/`.
4. **A regression test** expressing the general property — the reserve
   emits tuples that a cost-order draw would never have reached, naming
   neither acceptance phrase — and **shown red on the base it claims to
   fix**.
5. Before/after on **both** canonical targets and at least two
   non-canonical inputs: pool size, emitted-slot count, wall clock,
   visible `--top 50` composition (distinct structures, worst visible
   score), and whether `recognize speech` still surfaces
   `wreck a nice beach`. Report **ENUMERATED** and **RANKED** separately.
6. `cargo test --release --lib`, `--test corpus_integration`,
   `--test exact_determinism`, `--test approx_determinism`,
   `--test no_phrase_hard_coding` reported. `cargo fmt`, `cargo clippy`
   and doctests do not exist on this host
   ([../../environment-notes.md](../../environment-notes.md)); do not
   claim them.
7. Wall clock on `--approximate --top 50` not regressed, measured
   **interleaved against the base in the same session** — do not quote a
   figure from another session or another item.
8. The branch is committed and pushed; verify durability with
   `git ls-remote --heads origin <branch>`, not with a local
   remote-tracking ref. Commit with
   `git -c commit.gpgsign=false commit` (this host signs by default and
   `gpg` is absent). `main` must never be pushed to or merged into.

## Handoff / notes

Opened 2026-09-26T22:59Z by coordinator `coord-4c02` from
`post-milestone-acceptance` at `e7231a8`, on [w-9c4d21](w-9c4d21.md)'s
"next action 2" and the w-1c3e77 / w-6f2b18 convergence. Fork from the
**current accumulation head**; a front forked from an older head would
revert PUNCH (`6a93c2a`).

Milestone relationship: this front is the enumeration-side route. The
visibility-side route is [w-b2e5c4](w-b2e5c4.md), which is running. The
two are independent, and the milestone does not wait for either: if only
one lands, the milestone still moves.

State of the other live fronts at opening time, so a later pass does not
have to re-derive it:

* agent `b2e5c4` running in `/workspace/madgab-representation-b2e5c4`
  (branch `madgab-representation-b2e5c4`, from `f08f29b`); it has an
  uncommitted `select_diverse` "representation reserve" and is about to
  commit and push. Not landable at opening.
* agent `7e1a03` running in `/workspace/madgab-review-1c3e77`, reviewing
  `784deaa` from `w-1c3e77` independently; it has confirmed the new test
  red on `03fdf52` in a throwaway worktree. No verdict yet.
* `w-1c3e77` is `blocked` on that review and must not be re-opened a
  fourth time on the enumeration axis; its own report says the milestone
  test is a *visibility* assertion.

Nothing from this front is integrated. `main` is `c0ecd7c` and untouched.
