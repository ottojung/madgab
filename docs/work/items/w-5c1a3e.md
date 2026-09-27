---
work_item: true
id: w-5c1a3e
state: working
priority: high
owner: front agent-5c1a3e (running; STEERED 2026-09-27T23:45Z by pass coord-1f0d) / opened and claimed 23:19Z by coord-9c31; front launched 23:19Z in /workspace/madgab-retain-5c1a3e [madgab-retain-5c1a3e]
updated: 2026-09-27T23:47:00Z
branch: madgab-retain-5c1a3e
worktree: /workspace/madgab-retain-5c1a3e
---

# Price the per-span *retention* policy: the last layer that can put `hid` in the pool

## Goal

Measure — and, only if a rule survives its own fences, generalise — the **span shortlist retention
policy** in `src/approx.rs`, so that a word the canonical case-2 reading needs can survive its
target span's shortlist. Today the blocking fact is that `hid` is *pruned from the retained
shortlist for its target span*, so it never enters the pool and no downstream score, weight, or
selection change can recover the clue.

This is a **retention** front. It is the one layer of the pipeline that neither of the two running
fronts owns.

## Context

- Standing goal: [../skills/itinerary-madgab.md](../skills/itinerary-madgab.md). Canonical case 1
  (`wreck a nice beach` for `recognize speech`) is green on this branch at printed rank 27.
  Canonical case 2 (`hits justice dupe hid came` for `It's just a stupid game`) is **not in the
  pool**; `hid` is pruned at the span shortlist.
- The retention policy is `MATCHES_PER_SPAN = 256` (`src/approx.rs:14`) and, when a span has more
  matches than that, a three-stage keep at `src/approx.rs:208-265`: the cheapest `CHEAP_KEEP = 96`
  by cost, then `COST_BANDS = 4` cost bands each keeping `BAND_KEEP = 40` ranked by
  **rarity first, then cost**, then a cost-ordered fill. A candidate is dropped if it loses all
  three stages.
- `REPORT-1a4e8d.md` (integrated at `5e15cc8`) prices the *selection* layer exhaustively and
  closes it: the reserve has no admissible interior, and its withheld candidate is the 18th member
  of a structure already saturated at `share_cap = 17`. Selection is not where case 2 is lost.
- Reach on the lattice is closed by measurement (`w-2f1c03`, `OBSTRUCTION-MAP.md` §3), so a
  *search* front would re-derive a priced negative. Retention is different in kind: it is a
  question about which of the matches the fuzzy lexicon already produced are *kept*, not about
  whether the reading is reachable.

## Non-contention is mandatory

- [w-e086cc](w-e086cc.md) (`agent-e086cc`, `/workspace/madgab-e086cc`) varies the **objective
  weight vector** and is currently at the vector it calls A5.
- [w-3f6a21](w-3f6a21.md) (`agent-3f6a21`, `/workspace/madgab-pairscore-3f6a21`) works the
  **score function** (adjacent-slot substitution charge).

This front holds both fixed. It may only vary the *retention* constants and the ranking key used
inside the keep stages. If a candidate retention rule needs a weight-vector or scorer change to
pay off, stop and report that the surfaces collide.

## Completion criteria

1. A measurement of, for the canonical case-2 target span, the full ordered match list with, for
   each match, its cost, its cost band, its rarity key, and the stage at which it was dropped (or
   kept). State the exact stage and rank that drops `hid`, from the production path, not from a
   re-implementation of it.
2. A sweep over the retention policy — at minimum the `CHEAP_KEEP` / `COST_BANDS` split and the
   band ranking key (rarity-first vs cost-first) — reporting for each variant: whether `hid` is
   retained, the resulting pool size, the printed top-50 `SIMILARITY` on the green canonical case 1
   target, and whether the canonical case-2 clue reaches the pool. Small coherent variants only.
3. If a general rule survives, implement it as a *policy* change with the same character as the
   existing ones — a number of slots or an ordering key justified in a comment, not a fitted
   constant and never a word, span, phrase, or sentence from the canonical examples. If none
   survives, say so with the number that settles it and propose no code.
4. Regression tests at the API or executable boundary expressing the *general* property that
   survives (for example: a span whose matches exceed the retention budget keeps candidates from
   more than one cost band, or keeps a low-`CHEAP_KEEP`-rank candidate that a cost band rescues).
   No test may name `hid`, `wreck a nice beach`, `hits justice dupe hid came`, or an exact expected
   rank for a canonical input.
5. Fence suites green at the front's base and after any change: `no_phrase_hard_coding` 9/9,
   `emit_coverage`, `emit_spread`, `approximate_output_is_locked`, `approximate_list_represents_enumerated_resegmentation`,
   `approximate_determinism`, `exact_determinism`, both determinism suites, `--lib`. The one known
   pre-existing base red, `approximate_finds_classic_madgab_resegmentation` (canonical case 2), must
   stay untouched and **not** re-pinned. Report `fmt` and `clippy` honestly — this host has no
   `rustup`, so if they cannot run, do not claim them.
6. Work committed and pushed to `madgab-retain-5c1a3e`. Never self-merge into
   `post-milestone-acceptance`; never touch `main`. Use a dedicated `CARGO_TARGET_DIR` and
   `--test-threads=1`; do not enter `/workspace/madgab-e086cc` or
   `/workspace/madgab-pairscore-3f6a21`.

## Handoff / notes

Opened by the reconciliation pass `coord-9c31` at 23:19Z, immediately after `w-1a4e8d` closed the
reserve axis as a surface. The routing is from two durable facts, not from a guess: the reserve
axis is priced and closed, and `agent-e086cc`'s in-flight finding is that the alignment's absence
from the pool is **weight-independent** (never reached at any of the nine sweep vectors) — so a
third *objective-side* front would be contending with `w-e086cc` for a surface that cannot fix
case 2. Retention is the layer the failure actually sits in.

If criterion 2 shows no retention variant retains `hid`, that is a valid and useful priced
negative: it converts "the word is pruned" from an observation into a measured closure of the
retention axis, and the next front must be named elsewhere.

## Reconciliation pass coord-1f0d (2026-09-27T23:41Z-23:48Z)

Pass verdict: the front is healthy, its premise is falsified, and it has been converted into a bounded priced-negative delivery.
The front's own commit `a2e7898` records the criterion-1 measurement that `hid` is **not** pruned by the span-shortlist
retention policy - it is rescued at fill #104 and retained - so the first place it is actually lost is the discard
decision inside `prune_partials` at `src/lib.rs:3454`. That falsification is accepted as this item's result.

Actions taken: the front was steered (steer delivered 23:45Z, agent restarted) to (1) finish the non-vacuity mutation
check of its new fence test - it must fail when the fill stage is removed and when the band stage is removed - and keep
it only if genuinely non-vacuous, (2) bring the uncommitted `src/approx.rs` edit to a clean minimal form with no
probe/debug-module residue and no production-line change, committing it with a bounded `docs/work/REPORT-5c1a3e.md`
recording the priced negative, the `prune_partials` localisation, the per-stage counts and the falsified premise, and
(3) push `madgab-retain-5c1a3e` with an explicit INTEGRATE/HOLD. The front was told NOT to start the threshold work and
not to edit `prune_partials`, because the successor surface is now owned by an independent front - this keeps the two
fronts on disjoint files (`src/approx.rs` here, `src/lib.rs` there).

Successor opened this pass: `w-3c5b18` (committed and pushed on `post-milestone-acceptance` at `22b2088`), the discard
threshold inside `prune_partials`. Front `agent-3c5b18` created 23:44Z in `/workspace/madgab-thresh-3c5b18`
[`madgab-thresh-3c5b18`] off `22b2088`, prompted 23:45Z and left RUNNING. No source change on this branch, main untouched.
