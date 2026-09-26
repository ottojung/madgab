---
work_item: true
id: w-6ad4c1
state: working
priority: normal
owner: coord-b3d9
updated: 2026-09-26T20:35:00Z
branch: madgab-punch-6ad4c1
worktree: /workspace/madgab-punch-6ad4c1
agents: 6ad4c1 (landing front, opened by coord-b3d9)
---

# Land `PUNCH`: reward clue-word monosyllables as an additive scorer axis

## Goal

Land the one surviving candidate axis from [w-558697](w-558697.md) - `PUNCH`,
the share of a clue's words with one syllable or fewer - as a **general,
additive** scoring axis, with its representation problem solved rather than
hidden and the output-lock consequences recorded rather than absorbed silently.

This item deliberately does **not** claim to close the primary milestone.
`git show madgab-axis-558697:REPORT-558697.md` (pushed `578c85f`) measures
`PUNCH` as **saturated on the canonical target** - constant at 0.8000, sd
0.0000 across the whole visible top-50 band - so it cannot reorder that band,
and it moves the requested wording 134 ranks the *wrong* way (9865 -> 9999 of
17 913). Landing it is justified by general output quality, not by progress on
`approximate_finds_classic_madgab_resegmentation`.

## Why it is worth landing on the report's own numbers

- full resegmentations in the visible list, canonical target: **0 -> 30**;
- two ordinary targets: 31 -> 50 and 15 -> 22;
- one ordinary target: **-9**, a real cost that must stay in the report;
- `wreck a nice beach` still visible at raw rank 27, margin +0.00129;
- wall clock: no measurable cost.

A 0-of-50 resegmentation rate on a five-word target is a user-visible defect in
its own right, and this is the one measured fix for it that survives review.

## The representation problem

The six existing weights sum to **exactly 1.00** with `CLOSED_CLASS` at -0.15,
so the objective's maximum is exactly 1.0: there is **zero headroom**. A naive
`+ w*v` axis therefore breaks the documented `Clue::score in [0,1]` contract at
any weight above print precision, changes every printed score, and reddens
`approximate_output_is_locked` and `generates_for_a_real_phrase`. The report
measures that ordering is unaffected by writing the axis as `+ w*(v - 1)`, a
non-positive contribution that keeps the contract without a renormalisation
hack. Use that bounded form unless the front can show why another is required,
and show the alternative it rejected.

The output-lock re-baseline is a **decision, not a chore**: any scorer change
moves printed scores. Earlier passes established that a re-baselined lock is
expected and is not by itself a reason to reject, but the front must show the
old and new locks **side by side, per target**, so a reviewer can see that what
moved is scores and not which proposals are shown. A lock whose proposal
*membership* changes without a matching argument is a rejection.

## Completion criteria

1. Branch `madgab-punch-6ad4c1` off `post-milestone-acceptance`, pushed, with a
   `REPORT-6ad4c1.md` carrying the diff, the representation choice, the
   side-by-side lock comparison, per-target resegmentation rates on **at least
   six ordinary targets**, the wall clock, what was not measured, and a
   `MERGE RECOMMENDATION:` line.
2. `PUNCH` computed from what the scorer already holds - no new traversal pass -
   with its cost stated.
3. **Additive**: no existing weight moved, no existing axis rescaled, and the
   similarity axis' *definition* untouched. Front `9a41d3`'s M7a
   (`MERGE RECOMMENDATION: HOLD`) is the precedent for why.
4. No phrase-specific, word-specific, substring-specific, dictionary-lookup or
   environment-value case anywhere, in the report or in `src/`.
   `tests/no_phrase_hard_coding` green.
5. Both guards verified **on the branch** and at the **CLI's default
   configuration** with the release binary, not only at the test's
   `top_n: 50, beam_width: 64`: `wreck a nice beach` visible for
   `recognize speech`, and `approximate_finds_recognize_speech_resegmentation`
   passing.
6. `cargo test --release --lib`, `--test corpus_integration`,
   `--test no_phrase_hard_coding`, `--test approx_determinism`,
   `--test exact_determinism` run on the branch, with a
   `git archive <base>` arm to separate pre-existing red from new red.
   `approximate_finds_classic_madgab_resegmentation` is expected to stay red;
   that is pre-existing and must be shown pre-existing, not excused.

## Forbidden

No editing, relaxing, skipping or `#[ignore]`ing
`approximate_finds_classic_madgab_resegmentation`,
`approximate_finds_recognize_speech_resegmentation` or the *assertions* of
`approximate_output_is_locked` - its expected values may be re-baselined, with
the side-by-side evidence criterion 1 requires, and nothing else in it. No
touching `coverage_tuples`, `sweep_index`, `EMIT_PROFILE_MAX_DEEP`,
`EMIT_PROFILE_RESERVE`, `select_diverse`, the share cap or `STRUCTURE_FLOOR`.
Do not merge anything yourself, do not touch the accumulation branch, and never
`main`.

## Handoff / notes

Opened and claimed by coordinator `coord-b3d9` at 2026-09-26T20:35Z from
`post-milestone-acceptance` at `6b55e6a`. Front `6ad4c1` in
`/workspace/madgab-punch-6ad4c1` on `madgab-punch-6ad4c1`, pushed before the
agent starts.

Read these rather than re-deriving any of the design or the numbers:

```sh
git show madgab-axis-558697:REPORT-558697.md   # the axis, the four rejected
                                              # candidates, the headroom proof,
                                              # the saturation measurement
git show madgab-review-pool-3e7b04:REPORT-3e7b04.md  # top_n dependence, the zero
                                              # noise floor, the lock's real
                                              # membership churn
git show madgab-score-reseg-9a41d3:REPORT-9a41d3.md  # why a rescaling was HOLD
```

Host notes: `git -c commit.gpgsign=false commit`; `/tmp` is `noexec`, so build
under `/workspace` or `/tmp/opencode`; a second arm is
`git archive <sha> | tar -x -C <dir>`; verify durability with
`git ls-remote origin`, not a local remote-tracking ref; `grep`, `sed` and
`python3` are not on `PATH` (the usable binaries are under
`/gnu/store/2daxpqdg74pan6bp78cp6jzknb435w9d-coreutils-9.1/bin` and
`/gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin`); `node` works for
scanning. Do not amend or force-push a published branch; add a commit and say so
in the report.

One transient host hazard seen at 20:56Z, recorded so a later pass does not
misread it: the `antonina` CLI is shared with a *different* queue whose agents
were reinstalling it during this pass, and for about a minute
`antonina agent status` failed with a `SyntaxError` from a half-written launcher
wrapper. It recovered on its own with no action here. If `antonina` fails oddly,
re-run it before concluding that an agent died - and note that this front was
confirmed `running`/`alive` at 20:45Z and again after the CLI recovered.

Next action for a later fresh pass: `antonina agent status --id 6ad4c1`, then its
log, then `git ls-remote origin madgab-punch-6ad4c1` and
`git show madgab-punch-6ad4c1:REPORT-6ad4c1.md`. On a pushed
`MERGE RECOMMENDATION: MERGE`, review the diff for criterion 3 (additive) and
criterion 5 (both guards, CLI defaults), then integrate onto
`post-milestone-acceptance` - never `main`.

## Pass 2026-09-26T21:30Z (coordinator coord-2d6f): review verdict - the code is approved, the *branch* is held for pruning durable history

Agent `6ad4c1` finished **succeeded** with `MERGE RECOMMENDATION: MERGE`, the
branch pushed at `0a3097d`, and its own suites: `--lib` 53/53,
`--test corpus_integration` 10 passed / 1 failed (only the pre-existing
`approximate_finds_classic_madgab_resegmentation`), `--test
no_phrase_hard_coding` 6/6, `--test approx_determinism` 4/4, `--test
exact_determinism` 1/1. The coordinator reviewed the diff rather than the
report.

**`src/lib.rs` (53 changed lines) is approved and is general.** Criterion 3
(additive) holds: no existing weight moved, no axis rescaled, no
existing axis' definition or inputs changed. `PUNCH` is computed from the
syllable count the extension path already computes, so no new traversal pass.
The bounded `+ w*(v - 1)` form is used and the rejected alternative is
documented in the diff itself. Fence scan over the added `src/` lines returns
**0** hits for any canonical phrase token, word or substring, and **0** for
`ZZ_`, `MADGAB_TRACE` or `env::var`. Its CLI-default guard check shows
`wreck a nice beach` still produced for `recognize speech` (raw rank 27). This
is a general output-quality change and, as the report says first, it does not
close the primary milestone.

**The integration is nevertheless held, for one reason only.** The branch
deletes **265 lines of durable coordination history from four other work
items** that this front has no mandate to edit:

```text
docs/work/items/w-2f7a10.md   -115
docs/work/items/w-4b1e07.md    -49
docs/work/items/w-558697.md    -46
docs/work/items/w-3e7b04.md    -55
```

Those are other coordinators' pass records, and pruning them would delete
exactly the evidence a later fresh pass is supposed to reconcile from. A merge
of the branch as pushed would trade a general quality improvement for a
silent loss of durable state, so it is not merged as pushed.

Unblock condition, objective: a branch whose diff against
`post-milestone-acceptance` contains **no deletions from any work item other
than `w-6ad4c1.md`'s own state lines** - i.e. the four files above restored to
their content on `1486de1` while `src/lib.rs` and `REPORT-6ad4c1.md` are kept.
The simplest form of that is a fresh commit on `madgab-punch-6ad4c1` restoring
those four files from the accumulation branch; the agent is terminal, so this
is coordinator work and needs no new front. Do **not** force-push the
published branch.

Next action for a later fresh pass, in priority order:

1. On `madgab-punch-6ad4c1`, restore `w-2f7a10.md`, `w-4b1e07.md`,
   `w-558697.md` and `w-3e7b04.md` from `post-milestone-acceptance` in a new
   commit, push, and verify the diff against the accumulation head shows
   insertions only. Then merge onto `post-milestone-acceptance` - never `main` -
   and re-run `--lib`, `--test corpus_integration`, `--test
   no_phrase_hard_coding`, `--test approx_determinism` and `--test
   exact_determinism` on the merged tree, checking the side-by-side
   `approximate_output_is_locked` comparison the report carries.
2. `9f1c05` ([w-9f1c05](w-9f1c05.md)) is live and independent of this; it must
   not be delayed by this front's re-commit, and it is fenced to
   documentation, so the two compose.
