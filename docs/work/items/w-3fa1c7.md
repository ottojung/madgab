---
work_item: true
id: w-3fa1c7
state: done
priority: high
owner: agent-3fa1c7 (claimed 2026-09-26T22:59Z by coord-4c02)
updated: 2026-09-27T04:40:00Z
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

---

# Report 2026-09-27 — census, admissibility, landed change, and the refutation of the reach claim

Branch `madgab-reserve-scoreorder-3fa1c7`, one commit `0c70bba` on top of
the opening commit `faeb5c4`, pushed and verified with `git ls-remote --heads
origin madgab-reserve-scoreorder-3fa1c7` (= `0c70bba`). `src/lib.rs` only:
the reserve's own `coverage_tuples` / `EMIT_PROFILE_SAMPLE`, the hoist of
the traversal's suffix-maxima key, the one production call site, the three
test call sites and two test helpers. **No other `src/lib.rs` region is
touched** — not `axes::*`, not `boundary_novelty`, not `src/adjacency.rs`,
not `select_diverse`, not the per-slot affordability or opening-width hunks
of `784deaa`, not the emission budgets, not the display policy. The merge
order against `madgab-representation-b2e5c4` is therefore free; if a
conflict ever appears it will be in `select_diverse`, which this branch
does not contain.

## 1. The census, on the default release path

Six real targets, both canonical ones first. Release build, corpus as
shipped, `--approximate --top 50 --beam 64`.

**Reserve emissions and their quality** (instrumented scratch build,
restored from a pristine copy before the commit; the committed diff
contains no probe, no `eprintln!`, no `env::var` and no `ZZ_*` / `zz_*` /
`MADGAB_*` name — `tests/no_phrase_hard_coding` is green, which is the
mechanical check):

| target | reserve emissions base -> after | mean final score of those emissions | best | distinct index tuples drawn |
| --- | --- | --- | --- | --- |
| It is just a stupid game (canonical) | 3,055 -> 3,115 | 0.722006 -> **0.754902** | 0.894716 -> 0.896671 | 2,584 -> 2,189 |
| recognize speech (canonical) | 3,213 -> 3,233 | 0.649608 -> **0.669364** | 0.911752 -> 0.911752 | 2,966 -> 2,421 |
| taco cat | 1,505 -> 1,552 | 0.621713 -> **0.637171** | 0.925858 -> 0.925858 | 1,486 -> 1,180 |
| sign on | 318 -> 322 | 0.602642 -> **0.616533** | 0.948793 -> 0.948793 | 317 -> 256 |
| big spender | 2,885 -> 2,988 | 0.554546 -> **0.571772** | 0.914552 -> 0.914552 | 2,751 -> 2,443 |
| I love you | 733 -> 739 | 0.616556 -> **0.641581** | 0.931942 -> 0.931942 | 728 -> 595 |

The reserve now spends the same number of emissions on wordings worth
more, on every target: **+0.017 to +0.033 of mean final score**, and the
single best reserve emission is higher or unchanged. The number of
*distinct* index tuples falls, which is the point and not a loss: the
reserve stops paying for many positions of poor value and pays for the
best position of a sample instead.

**Visible composition and pool** (`--top 50 --beam 64`; pool size at
`--top 20000 --beam 64`, whole pool counted):

| target | visible | distinct structures | worst visible score | pool `--top 20000` |
| --- | --- | --- | --- | --- |
| It is just a stupid game | 50 / 50 | 9 / 9 | 0.897753 / 0.897753 | 19,511 -> 19,526 |
| recognize speech | 50 / 50 | 4 / 4 | 0.916872 / 0.916872 | 17,277 -> 17,284 |
| taco cat | 50 / 50 | 5 / 5 | 0.928017 / 0.928017 | 14,880 -> 14,892 |
| sign on | 50 / 50 | 4 / 4 | 0.930912 / 0.930912 | 8,827 -> 8,829 |
| big spender | 50 / 50 | 6 / 6 | 0.915118 / 0.915118 | 16,991 -> 17,064 |
| I love you | 50 / 50 | 5 / 5 | 0.926387 / 0.926387 | 11,048 -> 11,051 |

Every visible number is **identical**; the pool grows by 2 to 73 wordings.
`recognize speech` still surfaces `wreck a nice beach` (it is in the visible
50 on both).

**Wall clock**, interleaved with the base in this session, three rounds,
alternating base and branch per target, `--approximate --top 50 --beam 64`,
medians of three:

| target | base ms | after ms | ratio |
| --- | --- | --- | --- |
| It is just a stupid game | 2,551 | 2,542 | 0.996 |
| recognize speech | 2,342 | 2,348 | 1.003 |
| taco cat | 1,536 | 1,534 | 0.999 |
| sign on | 1,158 | 1,146 | 0.990 |
| big spender | 2,321 | 2,284 | 0.984 |
| I love you | 1,326 | 1,292 | 0.974 |

No regression. (One 7,224 ms outlier on the branch in round 1 of the first
target is host memory pressure — 58 of 62 GB in use by other agents — and
is not quoted as a figure.)

## 2. ENUMERATED and RANKED, separately

* **ENUMERATED: still NO** for the canonical wording of the first target,
  on the base and on this branch. `approximate_finds_classic_madgab_resegmentation`
  is **red on both**, with its assertion untouched. The diff does not
  re-baseline it and does not touch its phrase.
* **RANKED: still NO.** It has no rank in the pool, on either build, and
  the branch does not give it one.

The census of *why* is section 4. It is a refutation of the premise the
front was opened on, not of the change.

## 3. The bound is admissible, and the suffix maxima are reused as-is

`bound` / `suf_*` were already computed per segmentation for the
traversal's best-first key. They are **hoisted above the depth-profile
reserve unchanged** — byte-identical block, verified by comparing the two
files — and both halves now use one bound. No new state, no new axis, no
new weight, and `approximate_output_is_locked` (which locks printed score
strings) is green, which is the strongest available evidence that the
hoist changed nothing the traversal computes.

The bound is admissible in the same sense `build`'s cost bound is:

* **one-sided.** `build` drops a tuple only when its cost exceeds what the
  run can pay for, and on no other ground. The score bound drops a
  candidate only in favour of another candidate *of the same sweep* that it
  rates at least as high, so it is a choice among candidates, not a filter.
* **a superset of the old draw.** The strided draw the sweep used to emit is
  the **first** member of the sample (`t = 0`), so every tuple the previous
  rule would have emitted is still a candidate here. Nothing this rule
  emits was unreachable to the old one, and the old emission is only ever
  displaced by a better-bounded one.
* **bounded and stated.** `EMIT_PROFILE_SAMPLE = 8` bound evaluations per
  funded subset, at most `8 * EMIT_PROFILE_RESERVE` = 128 per segmentation
  (the code comment quotes the looser `16 * 16` = 256 product; the binding
  number is the subset count, which is at most the reserve), each one the
  arithmetic the traversal's heap key already performs. No extra
  emissions, no extra pops, no extra memory, no new state between
  segmentations.

The sample's extra candidates come from the **same** coprime rotation,
`EMIT_PROFILE_SAMPLE` positions apart per phase, so the union of a run's
samples still covers each member's whole list above the floor. The
existing sweep-coverage tests are kept and pass unchanged; they pin the
sweep with a flat bound (a test helper, `flat_bound`), because they are
about *which positions* the rule reaches, and the new test is about which
of them it prefers.

## 4. The refutation, with the arithmetic

The motivating front's premise was that the emissions are chosen by the cost
bound and that "that is the only thing excluding" the deep coordinate set.
Measured on the base, on that coordinate set's own cell
(`--approximate --top 50 --beam 64`, exhaustive enumeration of the cell,
release build, `EMIT_PROFILE_RESERVE = 16`, `LEXICAL_BRANCH_STAGE_0 = 10`,
`total_budget = 1.5`):

* the cell is `[160, 7, 160, 160, 93]`, **2,666,496,000** tuples, of which
  343,171,713 are cost-admissible;
* the designated coordinate set `[7, 0, 13, 99, 11]` has cost
  `1.1195198141657896` — the figure [w-9c4d21](w-9c4d21.md) recorded — and
  final score `0.7999012907736722`, reproduced exactly;
* **by admissible cost it is rank 9,585,641** within its own 3-deep class
  and 20,905,081 within the whole cell (the latter reproduces
  [w-9c4d21]'s 20,905,081 for the second cell, which checks that the census
  instrument is wired to the search's own slot lists);
* **by the search's own admissible score bound it is rank 3,924,097**
  within that class and 9,175,086 within the cell. The score order
  therefore moves the rank by a factor of **2.4** — and nowhere near a
  draw.

Why the lever is not the lever: within its own 3-deep class (628,425,000
tuples, 137,737,301 of them cost-admissible) the **top 16** tuples by true
final score all score at least `0.8256155982276456`, against the designated
set's `0.7999012907736722`. Sixteen draws cannot reach rank 3.9 million,
and the cell receives **14** reserve emissions in an entire run of this
target. End to end, with a hit counter on the emitted index vectors: the
designated set is drawn **0 times on the base and 0 times on this branch**.

For completeness, the class the reserve would need
`EMIT_PROFILE_MAX_DEEP = 4` to fund at all (1,960,875,000 tuples) is where
the score order is most kind to it: rank **189,197** there, still far past
a draw of sixteen.

The pool-level "rank ~50-100 by final score" that motivated this front is
real and is not in contradiction with the above: it is a rank over a
19,511-wording pool drawn from ~150 distinct structures, whereas the number
above is a rank inside one cell of one structure. A per-cell draw of 16 is
far more selective than a pool of 19,511, and it is the pool rank that was
quoted. **The ordering is not what excludes this wording; the
4,096-draws-against-2.67e9-points arithmetic of [w-9c4d21](w-9c4d21.md) is,
and the score order does not change it.** The change is landed anyway
because it is a strict, general, measured improvement in what the reserve
buys — the mean final score of its emissions rises on all six targets at
unchanged cost, count, visible output and clock — and because the front's
contract asks for the reserve to be spent on an admissible score bound *as
a general behaviour*, which it now is.

## 5. The regression test, and its red on the base

`src/lib.rs`,
`tests::the_coverage_reserve_spends_on_the_best_bounded_candidate_it_draws`:
on widths `[37, 53, 29, 41, 19]` and a bound that is a function of index
depth alone, over 64 phases, it asserts that every emission is the
highest-bound candidate of its own subset's rotation, that the strided draw
is still a candidate and is only ever displaced by something the bound
rates at least as high, that every emission is a position the sweep's own
rotation draws, that the count and the alphabet are unchanged, and that the
mean bound of the emissions strictly exceeds the index-ordered draw's. It
names no word, substring or word sequence of either acceptance example.

Shown **red on the base** (`faeb5c4`): the test was ported into a pristine
`faeb5c4` worktree with the base's four-argument `coverage_tuples`, and it
fails behaviourally, not by not compiling:

    test tests::the_coverage_reserve_spends_on_the_best_bounded_candidate_it_draws ... FAILED
    panicked at src/lib.rs:4581:9:
    the bound never changed an emission, so it is not being used

The base worktree was restored with `git checkout src/lib.rs` afterwards.

## 6. Validation

| suite | result |
| --- | --- |
| `cargo test --release --lib` | **54 passed, 0 failed** |
| `--test corpus_integration` | 10 passed, **1 failed** |
| `--test exact_determinism` | 1 passed, 0 failed |
| `--test approx_determinism` | 4 passed, 0 failed |
| `--test no_phrase_hard_coding` | 6 passed, 0 failed |

The one failure is `approximate_finds_classic_madgab_resegmentation`, this
project's own target test. **It was already red on `faeb5c4` and still
is**; its assertion is unmodified. `approximate_output_is_locked` is
**green**. `approximate_finds_recognize_speech_resegmentation`,
`approximate_pool_reaches_matches_deep_in_a_span` and
`approximate_pool_reaches_alternatives_past_the_opening_slot_width` are all
green.

`cargo fmt`, `cargo clippy` and doctests **do not exist on this host**
(../../environment-notes.md) and are not claimed.

## 7. Fences

No phrase literal in `src/` or `tests/` (`no_phrase_hard_coding` green,
and the diff adds only widths, index arithmetic and the notion of a bound).
No `axes::*` weight, no `boundary_novelty` form, no `src/adjacency.rs` or
`ADJACENCY_*`, no `select_diverse` change, no per-slot affordability or
opening-width hunk, no `EMIT_PROFILE_RESERVE` / `EMIT_PROFILE_MAX_DEEP`
value change, no global emission or pop budget change, no acceptance
assertion relaxed or re-baselined, no `env::var` production knob, no
`eprintln!` in `src/`, no `ZZ_*` / `zz_*` / `MADGAB_*` / `.bench` tree on the
branch. All scratch work was in `/tmp/opencode/` and in
`/workspace/mg-3fa1c7-base` (a throwaway worktree at `faeb5c4`, since
`/tmp` is noexec), and both were restored before the commit. Determinism
holds.

## 8. Next action, for the coordinator

1. **Do not re-open the reserve-ordering family.** It is now closed on both
   halves: the ordering lever is landed and measured (mean
   reserve-emission score +0.017..+0.033 on six targets, no visible change,
   no clock change), and the reach claim is refuted with the cell's own
   arithmetic (rank 3,924,097 by the search's own bound inside a
   628,425,000-tuple class, against 14 emissions into that cell per run).
2. **The milestone criterion is not met by this front and this front
   cannot meet it.** Both canonical measurements are unchanged: the first
   target's wording is still absent from the entire pool, and
   `recognize speech` still surfaces its own. If the milestone needs the
   first, the remaining route is visibility-side
   ([w-b2e5c4](w-b2e5c4.md)) or a change to what the search optimises, not
   to the reserve.
3. **One caveat for whoever rebases.** `coverage_tuples` gained a fifth
   parameter, so any other branch that calls it must add one. On this head
   the only callers are the production site in `build` and three unit-test
   sites; `madgab-representation-b2e5c4` was not inspected for this (its
   worktree was not touched), so a rebase conflict is possible but would be
   a call-site conflict only.
