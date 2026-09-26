---
work_item: true
id: w-c4e8d7
state: working
priority: high
owner: coord-3f9a
updated: 2026-09-26T16:58:00Z
branch: madgab-enum-reach
worktree: /workspace/madgab-enum-reach
agent: c4e8d70
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

Claimed and launched: agent `c4e8d70`, worktree `/workspace/madgab-enum-reach`, branch
`madgab-enum-reach` from `6f1022a`, left running. The push of this file is
the claim event.

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

## Pass 2026-09-26T16:58Z (coordinator `coord-3f9a`): membership is already
non-zero, the blocker has moved to the pairing slot, and the front is left running

No source change on this pass and nothing integrated: the agent's only commit
on the implementation branch is `a9c141a` "wip: uniform coverage sweep", which
is mid-work by its own title and was not reviewed. `post-milestone-acceptance`
was clean and identical to `origin` at `95f7a6c`; `main` does not exist on this
host, local or remote; a sweep of every local branch for commits not in the
accumulation head found no finished source work (all ahead-commits are
docs-only, scratch probes, or the agent's live WIP). The single live worktree
is on the **named** branch `scratch/c4e8d7-measure`, so the `04f83f0`
detached-head hazard is absent; `c0ac549` holds its `ZZ_TRACE` / `ZZ_STAGE` /
`ZZ_POOL_OUT` probes and `a9c141a` is branch-referenced, so nothing is
unrecoverable.

**Objective re-measurement on `95f7a6c`** by this coordinator, with the release
binary, not copied forward:

```text
cargo test --release --test corpus_integration approximate_finds
  approximate_finds_recognize_speech_resegmentation  ok
  approximate_finds_classic_madgab_resegmentation    FAILED (still)
  1 passed; 1 failed; 8 filtered out; 2.43s
recognize speech:        raw phrase="wreck a nice beach" rank=27 score=0.918313383
                         raw_cutoff rank=49 score=0.917045726   (search 1403ms)
It's just a stupid game: raw phrase="hits justice dupe hid came" missing
                         candidates=18065  raw_cutoff 0.915691888  (search 1675ms)
```

`18065` is the number every pool dump from this front must be compared against.

**The front has already moved the quantity that matters, and this is the first
time that has been true of any front since the queue reached enumeration.**
From the agent's own `ZZ_POOL_OUT` dump on its instrumented tree, the requested
wording's middle word goes from **0 to 14 pool candidates**. Criterion 3's
membership half is therefore met in measurement, ahead of any commit. Two
things follow, and they are the reason this pass is a steer rather than a
review:

* **The contradiction w-4b1e07's 11:52Z pass flagged is resolved in favour of
  the shortlist being reachable.** The 3/2/10/99/11 per-slot rank table and the
  zero-emission count are no longer in conflict, because the traversal is no
  longer what was measured. The remaining question is the *pairing*, not
  membership: in the dump, `dupe` is emitted alongside `eeg`, `uhh`, `add`,
  `gave` — for example `it justice dupe eeg aim` at structure `[2,11,13,16,19]`
  and `each justice dupe eeg gave` at `[3,11,13,16,19]`. Slot 3 is solved and
  the `hid` slot at the canonical cut is what is not.
* **Per this item's own sequencing rule, membership non-zero with the wording
  still invisible is a successful end state**, and the correct deliverable from
  here is the canonical structure's best-member rank and fill rank on the new
  head, then stop — not a re-run of the selection question that
  [w-8a1d47](w-8a1d47.md) closed.

**Facts handed to the agent rather than left for it to derive** (it was steered
non-blockingly and left running, `prompts: 2`):

* `dupe` **is** in the shipped corpus, so corpus admission is not the defect:
  `open-english-pronouncing-dictionary` 0.1.0, `data/openepd.json`, rarity
  `40210`, `ipa.cmu` `dˈup`, against a default `max_rarity` of `50000` at
  `src/lib.rs:86`. For scale, 281502 entries of which 231501 are above the
  cutoff. This closes the "cheaper explanation" a later pass would otherwise
  have spent a front on.
* The span shortlist is at `src/lib.rs:696-858`, not in `src/approx.rs`, and its
  portfolio is `SPAN_AXIS_KEEP=16` by cost and by familiarity,
  `SPAN_RARITY_KEEP=4` by rarest, 4 per cost band, 16 by quality, then a fill to
  `SPAN_SHORTLIST=160` **in that same quality order** (850-857). Every one of
  those orders prefers cheap-and-familiar, and the fill inherits it, so a
  mid-frequency word has no pass that prefers it — which is the defect the
  comment at 752-758 claims to fix with four reserved slots. Which pass retains
  or drops the word is now the cheapest discriminating check left, and it
  separates "the traversal never offered it" from "the portfolio never kept it".
* The named caps it is working under (`SPAN_SHORTLIST` 123,
  `LEXICAL_COMBINATIONS_PER_SEGMENTATION` 125, `LEXICAL_BRANCH_STAGE_0` 10 and
  `_GROWTH` 4 at 141-142, `SPAN_*` 614-616, `LEXICAL_HEAP_POP_LIMIT` 619), and
  the closed-and-recorded list it must not re-sweep: `w-9d4e17`, `w-5f1c04`,
  `w-e07c42`, `w-be6d21`, `w-3f9c02`, `w-8a1d47`.
* Hygiene: probes stay on `scratch/c4e8d7-measure`; `madgab-enum-reach` must
  end on a clean non-`wip:` commit and be pushed, and the scratch branch pushed
  too. Never `main`.

No second front was opened. Every remaining candidate territory is either the
one this agent is in or a closed one, and the milestone's blocker count is
still one; manufacturing a third front on closed territory would contend rather
than compose.

Next action for a later fresh pass: read `c4e8d70`. If it has landed a final
commit on `madgab-enum-reach`, review it as a diff for generality — the
rejection must be gone, no phrase-specific case, no `axes::*` or `select_diverse`
edit, no `zz_`/`MADGAB_*` scaffolding, no relaxed acceptance test — then run
`cargo test --release --lib`, `--test corpus_integration`,
`--test exact_determinism`, `--test approx_determinism` and the six-target wall
clock, and integrate onto `post-milestone-acceptance`, never `main`. If it
reports only the best-member and fill ranks, that is this item's end state and
the selection question is re-opened as a new item from those numbers rather than
inside this one.
