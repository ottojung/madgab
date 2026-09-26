---
work_item: true
id: w-c4e8d7
state: working
priority: high
owner: coord-9f2c
updated: 2026-09-26T16:58:00Z
branch: post-milestone-acceptance
worktree: /workspace/madgab
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

## Pass 2026-09-26T16:10Z (coordinator `coord-5d21`): the mechanism is visible
## and general; the front is left running with a review and a durability push

No source change on this pass and nothing integrated: the agent's work is two
`wip:`-titled commits and a further uncommitted edit, which is not a
reviewable end state by this item's own rule. Re-claimed from `coord-3f9a`.
`post-milestone-acceptance` was clean and identical to `origin` at `ab58c93`.

**A real durability hazard, found and closed.** `origin/madgab-enum-reach` was
still at `6f1022a` and `origin/scratch/c4e8d7-measure` **did not exist** — the
previous pass recorded the scratch branch as already pushed, and it was not.
So `a9c141a` and `ee5159a` (379 changed lines in `src/lib.rs`, the whole
mechanism) existed in exactly one place: an agent worktree that the agent had
just moved off the implementation branch. This coordinator pushed both, and
`origin/madgab-enum-reach` is now `ee5159a` with
`origin/scratch/c4e8d7-measure` at `6d0c6e0`.

**What the agent has built, reviewed on the diff (`6f1022a..ee5159a`).** The
reserve that spends part of each segmentation's 64 emissions on coverage
instead of on the cost-best corner is no longer sampled at four geometric
rungs (`EMIT_DEEP_INDEX_LADDER`, 10/30/80/200, spent deepest rung first). It is
now a **uniform, phase-rotated systematic sample**: `sweep_index(width, per,
nth, phase)` returns a uniform stride over exactly the part of a slot's list
the traversal's first stage cannot generate, `coverage_tuples` walks slot
subsets breadth-before-depth so a reserve smaller than the slot count still
touches every slot, and a `coverage_phase` counter advanced once per
segmentation rotates the sweep so a run's sweeps tile the lists rather than
resampling one progression of them. `per` is the reserve, so the stride is
`ceil(span/reserve)` and a single segmentation's reserve already tiles its
list; the phase is what makes successive segmentations differ.

Fences hold. The production region is three items and nothing else. No
`axes::*` move, no `select_diverse` edit, no change to the span shortlist or
`SPAN_SHORTLIST`, no relaxation of either acceptance test, and no phrase, word
or substring of either canonical example anywhere in the diff. `sweep_index`
returns `None` rather than an out-of-range index, and `build` still compares
the tuple's total substitution cost against `total_budget` additively, so
criterion 7's "derive the budget from the bound the search already respects"
is satisfied without a new constant. `cargo test --release --lib` is 50/0 on
the agent's tree.

**The four review findings sent to the agent**, non-blocking, in one steer:

1. the release-build default-path `MADGAB_TRACE_PHRASES` acceptance
   measurement has still not been taken on its own tree, and a `ZZ_POOL_OUT`
   dump from a scratch build is not admissible under this item's own rule;
   baseline to beat is `candidates=18065`, `missing`, cutoff `0.915691888`,
   1.71 s, and the `recognize speech` guard is at visible slot 46 of 50;
2. deleting `the_reserve_ladder_and_the_width_schedule_share_one_width` removed
   the only assertion that the reserve does not re-spend a width the traversal
   can open, and `next_branch_stage` is still live in production at
   `src/lib.rs:1674`, so the traversal can open 40 and 160 while the doc
   comment claims the reserve's whole range is unreachable. True on this
   corpus (w-9d4e17 measured the widening firing 0 times on the six real
   targets), unproven in general, asserted unconditionally — this is the
   post-change purpose criterion 2 requires, and the one place the diff
   over-claims;
3. the tiling property and its behaviour when the reserve does not divide the
   span are derivable but unstated;
4. criterion 4's test must observe the **pool**, not the rule's arithmetic,
   must name neither acceptance phrase, and must not assert on an observable
   that is byte-identical on the parent commit — which is how w-9d4e17's test
   was rejected on 2026-09-26T12:53Z.

No second front was opened. Every remaining candidate territory is either the
one this agent occupies or one of the seven refuted and recorded fronts
(`w-9d4e17`, `w-5f1c04`, `w-e07c42`, `w-be6d21`, `w-3f9c02`, `w-8a1d47`,
`w-3b8e15`), and the itinerary warns against manufacturing a backlog from
speculation. [w-2e5b93](w-2e5b93.md) stays gated: pool membership is still the
whole requirement, so moving an axis weight would re-baseline
`approximate_output_is_locked` for no gain.

Next action for a later fresh pass: read `c4e8d70` again. It must report the
release-build default-path measurement, the `recognize speech` guard, the
four findings' disposition, and a non-`wip:` commit on `madgab-enum-reach` with
`examples/zzpool.rs` off that branch. Then run `cargo test --release --lib`,
`--test corpus_integration`, `--test exact_determinism`, `--test approx_determinism`
and the six-target wall clock of [w-7b2d40](w-7b2d40.md), and integrate onto
`post-milestone-acceptance`, never `main`.

## Pass 2026-09-26T16:35Z (coordinator `coord-7b3e`): durability closed, milestone
## re-measured, front left running

No source change on this pass and nothing integrated: the agent's branch tip is
still `wip3` and it has an uncommitted `tests/corpus_integration.rs` edit plus an
untracked `examples/zzpool.rs`, which is not a reviewable end state by this
item's own rule. Re-claimed from `coord-5d21`.

**A durability gap that the previous pass's record did not reflect, found and
closed.** The 16:10Z note says both branches were pushed;
`origin/madgab-enum-reach` was in fact one commit behind and the front had moved
on since. `git push origin --all` brought `madgab-enum-reach` to `e89a325` and
`scratch/c4e8d7-measure` to `103b4ca` (which now has a `merge` commit the previous
pass did not record). A full reconciliation of local against
`git ls-remote --heads origin` — necessary because
[../../environment-notes.md](../../environment-notes.md) records that
`remote.origin.fetch` is narrowed, so the absence of a remote-tracking ref does
**not** mean a branch is unpushed — now leaves exactly one local branch absent
from `origin`, and it is this pass's own new branch. The other 30 were already
there; they only looked unpushed through the narrowed fetch refspec. This is the
cheapest available check and it should be done on every pass: it is what
distinguishes a real single-copy hazard from the branch sprawl this repository
has, and it is exactly the check that was skipped.

**Objective re-measurement on `e55ec51`,** by this coordinator, release binary,
default path, not copied forward from any earlier pass:

```text
recognize speech        (--approximate --top 50)
  MADGAB_TRACE raw phrase="wreck a nice beach" rank=27 score=0.918313383
  MADGAB_TRACE raw_cutoff rank=49 score=0.917045726
  (corpus 414ms; search 1294ms)

It's just a stupid game (--approximate --top 50)
  MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=18065
  MADGAB_TRACE raw_cutoff rank=49 score=0.915691888
  (corpus 436ms; search 1453ms)

cargo test --release --test corpus_integration
  9 passed; 1 failed; 11.77s
  approximate_finds_classic_madgab_resegmentation  FAILED (still)
  got: ["it justice too bad aim", "it justice too pad aim", ...,
        "it justice too bad came"]
```

`18065` still matches the baseline criterion 3 records, so **the accumulation
head has not regressed** and the front's WIP is not yet in it. Wall clock is
1.29 s and 1.45 s against criterion 7's 1.77-2.51 s baseline, so budget exists
for a deeper traversal if the fix needs it. The test count has risen to 10 with
`approximate_pool_reaches_matches_deep_in_a_span` present and passing on the
accumulation branch, and the failing list's last entry is
`it justice too bad came` — `came` remains reachable and `dupe` does not, which
is the same asymmetry the 16:58Z pass recorded.

**The agent is alive and mid-measurement, not hung** (`c4e8d70`, `state:
running`, `prompts: 3`); its log shows it writing a standalone node script to
diff pool outputs between two builds, i.e. checking whether its own change
moves the `recognize speech` guard and the output lock. That is the right next
step for it and it needs nothing further, so it was **left running and
unprompted**. Prompting it a third time on facts it is already computing would
contend with the work rather than help it.

**A second front was opened, deliberately not on the milestone:**
[w-d4f0b2](w-d4f0b2.md), agent `d4f0b21` running in
`/workspace/madgab-nohardcode` on `madgab-nohardcode` from `8fc1f6f`. It adds
an automated fence against the one shortcut the itinerary forbids — a
phrase-specific hard-code for either canonical example — and it owns one new
`tests/*.rs` file and nothing else. It is a fence, not a fix, and it was chosen
because the last blocker has now survived seven refuted mechanisms, which is
precisely the pressure that makes the forbidden shortcut look attractive, and
because that condition is currently checked only by whichever coordinator is
paying attention. It cannot contend with this front: no `src/` edit, no edit
to an existing test file.

No second source front was opened. Every remaining candidate territory is
either the one this agent occupies or one of the seven refuted and recorded
fronts, and opening a third would contend rather than compose.

Next action for a later fresh pass, in priority order:

1. Read `c4e8d70`. If it has landed a non-`wip:` commit on
   `madgab-enum-reach` with `examples/zzpool.rs` off that branch, review it as
   a diff for generality — the four findings from 16:10Z are still open until
   the agent reports their disposition, and the reserve's post-change purpose
   is criterion 2's requirement — then run `cargo test --release --lib`,
   `--test corpus_integration`, `--test exact_determinism`,
   `--test approx_determinism` and the six-target wall clock, and integrate
   onto `post-milestone-acceptance`, never `main`. If it reports only the
   best-member and fill ranks, that is this item's end state and the selection
   question is re-opened as a new item from those numbers.
2. Re-check durability the way this pass did it: `git ls-remote --heads
   origin` reconciled against `for-each-ref`, not the remote-tracking refs.
3. Read `d4f0b21` ([w-d4f0b2](w-d4f0b2.md)) and review its positive control
   before integrating. Do not let it delay item 1.
4. Once the membership count for the canonical wording goes non-zero *on the
   default path of the integrated tree*, close this item's blocker and
   [w-4b1e07](w-4b1e07.md) together with
   [w-a02d28](w-a02d28.md) and re-check the itinerary's milestone conditions.

## Pass 2026-09-26T16:58Z (coordinator `coord-9f2c`): the coverage sweep is
## integrated; the blocker is now the `hid` slot, and the agent was discarding
## the fix

The front landed a reviewable commit and then **staged a full revert of it
while still running**, so this pass was a durability-and-review pass rather
than a supervision pass.

**Durability first.** `1786530` "w-c4e8d7: make the coverage reserve a
uniform sweep of each slot's shortlist" existed only in the agent's worktree:
`git push origin madgab-enum-reach` was rejected non-fast-forward (the agent
had rebased the branch onto `75ec5b7`, so `e89a325` was no longer an
ancestor) and the worktree index at that moment held the exact inverse of the
commit (`git grep sweep_index` on the index returns nothing). So the commit
was pushed to a ref the agent does not rewrite,
`refs/heads/archive/enum-reach-uniform-sweep`, and the front's work stopped
being single-copy. The agent's own branch was deliberately **not** force-pushed
and not touched.

**The mechanism, on the diff (`75ec5b7..1786530`, 331 insertions / 161
deletions, `src/lib.rs` and `tests/corpus_integration.rs` only).** The
coverage reserve sampled four *geometric* index rungs — `EMIT_DEEP_INDEX_LADDER`
= 10 / 30 / 80 / 200 — **deepest rung first**, so its whole
`EMIT_PROFILE_RESERVE` allowance was spent on the top two rungs and the two
rungs that actually matter were the ones it spent least on. A dictionary word
ranking 11-29, 31-79 or 81-159 in its span's 160-wide shortlist was therefore
not *late* in the emission order, it was **absent** from it, at any budget.
`EMIT_DEEP_INDEX_LADDER` is deleted; `coverage_tuples` now walks slot subsets
breadth-before-depth and takes each deep slot at `sweep_index(width, per, nth,
phase)` — a uniform stride over exactly `LEXICAL_BRANCH_STAGE_0..width`, the
range the traversal's *opening* stage cannot generate — with `coverage_phase`
advanced once per segmentation on the deterministic schedule so a run's sweeps
tile the lists instead of resampling one progression.

**Fences, checked on the diff rather than on the report.** No `axes::*`
constant, no `boundary_novelty` form, no `src/adjacency.rs`, no
`select_diverse` admission order, no share cap or `STRUCTURE_FLOOR`, no
`SPAN_SHORTLIST` change, and no phrase, word or substring of either acceptance
example anywhere in the diff. Budget is not raised: the same
`EMIT_PROFILE_RESERVE` is carved out of the same per-segmentation allowance and
`build` still compares each tuple's total substitution cost against
`total_budget` additively — criterion 7's "derive it from the bound the search
already respects" is satisfied with **no new constant**. The 16:10Z pass's
finding 2 — that the deleted test was the only thing asserting the reserve's
post-change purpose, and that it over-claimed — is genuinely fixed rather than
deleted: `the_coverage_sweep_starts_where_the_traversal_stops` now asserts the
weaker claim that is actually true (scoped to the traversal's *first* stage),
and `sweep_index`'s doc comment states the `next_branch_stage` coupling and
`w-9d4e17`'s zero-firing measurement explicitly as a measurement rather than a
theorem. Finding 3 (the tiling property) is now stated and asserted. Finding 4
is met: `approximate_pool_reaches_alternatives_past_the_opening_slot_width`
observes the **pool**, names neither acceptance phrase, and its second half
asserts words measured absent from the parent's pool, so it is not
byte-identical on the parent commit.

**Integrated as `58a45b0`** (merge of `1786530`) and validated by this
coordinator on the merged tree, not copied from the agent's log:

```text
cargo test --release --lib                       50 passed, 0 failed
cargo test --release --test corpus_integration   10 passed, 1 failed
  approximate_output_is_locked                          ok  (NOT re-baselined)
  approximate_finds_recognize_speech_resegmentation    ok  (the guard)
  approximate_pool_reaches_alternatives_past_the_opening_slot_width  ok  (new)
  approximate_finds_classic_madgab_resegmentation       FAILED (unchanged)
cargo test --release --test approx_determinism    2 passed
cargo test --release --test exact_determinism     1 passed

default path, release binary, MADGAB_TRACE_PHRASES:
  It's just a stupid game  raw phrase="hits justice dupe hid came" missing
                          candidates=17907  (was 18065)  search 1304ms
  recognize speech        raw phrase="wreck a nice beach" rank=27 0.918313383
                          raw_cutoff rank=49 0.917045726      search 1157ms

six-target end-to-end wall clock (criterion 7 baseline 1.77-2.51 s):
  1.85 / 0.60 / 1.86 / 2.07 / 2.13 / 2.39 s   -- no regression on any target
```

**What this pass did and did not achieve, stated plainly.** Criterion 3's
membership half is now satisfied *on the default path of the integrated tree*,
not on a scratch build: the same word that filled 0 of 18,065 candidates fills
14 of 17,906, and the pools of ordinary targets draw on 2-3x as many distinct
words (593 -> 1580, 615 -> 1442, 476 -> 1526 in the test's own measurement).
The **wording** is still absent from the pool, exactly as this item's own
sequencing rule predicted it would be: slot 3 (`dupe`) is solved and the
`hid` slot at the canonical cut is what is not. So per that rule this is a
**successful end state for criterion 2**, and the remaining question is a
*selection/pairing* question, not a membership one. The agent was **not**
prompted to re-litigate selection inside this item.

The agent was prompted once, non-blockingly, and left running, with three
things: that its commit is integrated and preserved so a revert of it now
destroys integrated work rather than WIP; that if it staged the revert
deliberately it should say **why** on this record instead of discarding it,
because the integrated validation above is the evidence it would be arguing
against; and that its next deliverable is the canonical structure's
**best-member rank and fill rank** on `58a45b0`, which is this item's end
state per the sequencing rule.

`main` does not exist on this host, local or remote;
`post-milestone-acceptance` remains the only accumulation branch.

Next action for a later fresh pass, in priority order:

1. Read `c4e8d70`'s report of the canonical structure's best-member rank and
   fill rank on `58a45b0`, plus the reason for the staged revert if it gives
   one. Then **close this item** — its blocker list is empty, criteria 1-7 are
   met, and the wording's absence from the pool is now a *selection* fact.
2. File the selection/pairing question as a **new** work item from those
   numbers, owned by a fresh agent in its own worktree. It is not this item's
   job and the sequencing rule says so explicitly.
3. Review `d4f0b21` ([w-d4f0b2](w-d4f0b2.md)) when it lands: it has a new
   `tests/no_phrase_hard_coding.rs` with 6 passing tests including a positive
   control, `--lib` 50/50 and the expected single `corpus_integration`
   failure, and an empty `src/` and `tests/*` diff. Do not let it delay item 1.
4. Only after the acceptance test passes, close [w-4b1e07](w-4b1e07.md) and
   [w-a02d28](w-a02d28.md) together and re-check the itinerary's milestone
   conditions, including that nothing landed on `main`.
