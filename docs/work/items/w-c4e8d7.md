---
work_item: true
id: w-c4e8d7
state: working
priority: high
owner: coord-7a10
updated: 2026-09-26T16:35:00Z
branch: madgab-enum-reach
worktree: /workspace/madgab-enum-reach
---

# A word that phonetically fits its slot is emitted in zero candidates:
# make the enumeration reach it

## Goal

Find the general reason a dictionary word that phonetically fits a target span
appears in **zero** of the ~18,000 candidates the default approximate path
emits, and fix it so that such words are enumerated. The quantity that must
move is **pool membership**, not score, not rank and not selection order.

## Why this is the milestone's last blocker, by measurement on the current head

`[w-8a1d47](w-8a1d47.md)`'s close-out measured the 2x2 that
`[w-3f9c02](w-3f9c02.md)` and `w-8a1d47` between them were pointed at, on
release builds of `fc5296f`, `--approximate --top 50`, pools dumped from the
selector's own input:

```text
It's just a stupid game, clean head:  pool 18065  structures 333
  hits 56   justice 7099   dupe 0   hid 18   came 157
It's just a stupid game, axis+F2:    pool 17973  structures 313
  dupe 0
cells (coverage-first off / on)     pools byte-identical, 18065 / 17973
  canonical structure [3,10,13,15] best-member rank: 94 clean, 63 under F2
```

So the requested wording is **not enumerated in any cell**. The pool does not
change at all when the admission order changes, which proves selection is not
what hides it, and `dupe` — a real dictionary word that fills the three-phoneme
slot in the middle of the alignment — is simply never produced. No admission
order, no share cap, no axis weight and no score cutoff can surface a candidate
the traversal never emits.

Both other fronts are now `done` and refuted, and both say the same thing from
opposite directions:

* `[w-3f9c02](w-3f9c02.md)` — the score side; a threshold acoustic charge moves
  the family's best-member rank from 3136 to 41, but breaks the first-canonical
  guard.
* `[w-8a1d47](w-8a1d47.md)` — the selection side; coverage-first admission
  reaches a structure by best-member rank where today's order cannot, and still
  cannot show a wording that is not in the pool, while it trades the guard away.

## The contradiction this front must resolve first, not assume away

`[w-4b1e07](w-4b1e07.md)`'s 11:52Z pass records per-slot candidate ranks of
**3 / 2 / 10 / 99 / 11** over the 160-wide per-span shortlist for this same
alignment — which puts `dupe` *inside* the shortlist at rank 10. That is
irreconcilable with 0 emissions, and the same class of error has already been
retracted once in this queue: `[w-7b41d2](w-7b41d2.md)`'s reachability claim
was withdrawn at 14:10Z because it had been measured with a harness whose
budget was wider than the default path.

**Admissibility rule for this item.** Reachability evidence counts only if it
is (a) taken on the **default** approximate path, (b) with a release build, and
(c) expressed as membership of the deduplicated pool the release binary
actually produces, or as a line-level trace inside that path. Shortlist ranks
from an instrumented or re-budgeted harness are not admissible. Say which of
your numbers are which.

`[w-5f1c04](w-5f1c04.md)` and `[w-6b2f04](w-6b2f04.md)` closed *enumeration
shape* on cost grounds on an older head; that closure is **not** evidence that
`dupe` is unreachable, and this item is entitled to re-open it. Their
conclusion, that a per-slot-capped best-first product search can only find
wordings that are near-optimal at every slot simultaneously, is a statement
about how the traversal is shaped — and it is the first hypothesis to test.

## Scope and collision boundary

This front owns lexical enumeration and nothing else: `FuzzyLexicon::matches_at`
and the per-span shortlist, the per-slot candidate lists, the best-first
product enumeration, `LEXICAL_BRANCH_KEEP`, `SPAN_SHORTLIST`,
`SEGMENTATION_KEEP`, and the emission budget accounting. It is uncontested:
both `axes::*` owners are closed and the selector front is closed.

Forbidden: sweeping or re-tuning `axes::*` weights or any `boundary_novelty`
form (closed by `w-3f9c02`), `src/adjacency.rs` and `ADJACENCY_*`, changing
`select_diverse`'s admission order or the share cap / `STRUCTURE_FLOOR`
(closed by `w-8a1d47`), landing the parked coverage-first patch, relaxing or
re-baselining **either** acceptance test, and any phrase-specific case.
`approximate_output_is_locked` may be re-baselined only with a record of which
structures moved, and only as a consequence of this item's own change.

## Completion criteria

1. **Diagnosis by line, before any fix.** The exact place in the default path
   where the word for that span is dropped: the function, the line, the list it
   was in or should have been in, and the value of the quantity that rejects
   it. If the answer is "it was never a candidate of the lexicon for this
   span", say so and show the span's IPA and the word's IPA.
2. **A landed, general fix** for the class, argued from the mechanism and never
   from either phrase: no special case for any word, substring or word sequence
   of either acceptance example, in `src/` or `tests/`.
3. **Measured membership, on the default path, release build:** for
   `It's just a stupid game`, the count of pool candidates containing the
   requested wording goes from **0** to non-zero, and the same count is
   reported for six real targets so the change is shown not to be specific to
   one input. Report the word-frequency distribution over the lexicon too, if
   the defect turns out to be a frequency or index cutoff.
4. **A regression test** that expresses the general property at an appropriate
   boundary — that a dictionary word fitting an unaligned span of the target is
   reachable in the approximate pool — and does not name either acceptance
   phrase.
5. `approximate_finds_recognize_speech_resegmentation` stays green. It is the
   guard, not a target.
6. `cargo test --release --lib`, `--test corpus_integration`,
   `--test exact_determinism`, `--test approx_determinism` reported, with
   `approx_determinism` green (raising `top_n` must never retract a shown
   proposal). `cargo fmt`, `cargo clippy` and doctests do not exist on this
   host; see [../../environment-notes.md](../../environment-notes.md). Do not
   claim them.
7. Wall clock on `--approximate --top 50` across the six targets of
   `[w-7b2d40](w-7b2d40.md)` not regressed. Baseline on `fc5296f`, end-to-end
   process time: 1.77 / 1.88 / 1.80 / 2.00 / 2.19 / 2.51 s. If deeper
   enumeration needs budget, **derive** it from the bound the search already
   respects (`total_budget`, compared additively at emission time) rather than
   raising a constant, and say what the constant was bounding.

## Handoff / notes

Filed by coordinator `coord-7a10` immediately after `w-8a1d47` closed with the
2x2. It is the successor to two refuted fronts and inherits their evidence
rather than repeating their sweeps: do **not** re-run the 2x2, do not re-sweep
axis weightings, and do not re-derive the coverage-first analysis. All of it is
recorded in `w-3f9c02` and `w-8a1d47`.

Measurement material already on disk and pushed, usable as-is:
`scratch/8a1d47-measure` (`53bbd17`) carries the `ZZ_POOL_OUT` pool-dump hook,
the F1/F2 axis ports, the coverage-first patch and a `MEASUREMENT.md` that
says how to rebuild each cell. Reproduce from there; keep new instrumentation
on a scratch branch of this item and off `madgab-enum-reach`.

**Sequencing, so the fronts compose instead of contend:** if membership goes
non-zero and the wording is still invisible, that is a *successful* end state
for criterion 2 and it re-opens the selection question with a much smaller
gap — report the structure's best-member rank and the fill rank on the new
head and stop there. Do not also re-litigate selection in this item.

Next action for a later fresh pass: review the diagnosis line and the diff for
generality, run the suites of criterion 6, and integrate onto
`post-milestone-acceptance`, never `main`.
