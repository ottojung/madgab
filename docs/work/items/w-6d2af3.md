---
work_item: true
id: w-6d2af3
state: working
priority: high
owner: agent-6d2af301 (claimed 2026-09-27T07:45Z by coord-c4e19 from post-milestone-acceptance e0b3a90)
updated: 2026-09-27T07:53:00Z
branch: madgab-review-bound-6d2af3
worktree: /workspace/madgab-review-bound-6d2af3 (agent-6d2af301 running)
reviews: madgab-emitbound-5b1e93 @ b49f892 (28d5e4a beneath it)
---

# Second adversarial review of the emission-bound front at `b49f892`

## Goal

Decide, independently and from a clean build, whether the emission-order
front's pushed head `b49f892` is fit to integrate into
`post-milestone-acceptance`: **INTEGRATE** or **REJECT**, with the fence
results and the admissibility derivation stated in numbers.

The first review ([w-3c9d17](w-3c9d17.md)) returned **REJECT as landed** for
`958771d`; its admissibility derivation was confirmed and its named fixes are
the checklist for this pass. The front has since pushed `28d5e4a` (joint
suffix bound, `SuffixRelaxation`, executable dominance property) and now
`b49f892` ("read every cost bound through the scorer's own similarity axis"),
which additionally repairs the three sibling inadmissible bound sites that
[w-3c9d17](w-3c9d17.md) §5 named as the obvious independent follow-on.

## What to review

Range `707fb2a..b49f892` in `/workspace/madgab-emitbound-5b1e93`, i.e. three
commits: `958771d`, `28d5e4a`, `b49f892` (+218/-19 in `src/lib.rs` in the
last one; ~434/97 in `28d5e4a`). Work in this item's own worktree and
`CARGO_TARGET_DIR`; the front worktree is live and must not be touched.

## Completion criteria

1. The criterion-5 suite set from [w-5b1e93](w-5b1e93.md) is run on
   `b49f892` and reported pass/fail per test: `--lib`, `--test
   corpus_integration`, `--test emit_coverage`, `--test approx_determinism`,
   `--test exact_determinism`, `--test no_phrase_hard_coding`. The two
   `corpus_integration` guards that were red at `28d5e4a`
   (`approximate_pool_reaches_matches_deep_in_a_span`,
   `approximate_pool_reaches_resegmentations_deeper_than_one_walk`) must be
   reported explicitly.
2. Admissibility re-derived, not assumed: for `lexical_score_bound` and for
   each of the three repaired sibling sites (`complete_span_score`,
   `span_score_bound`, and whatever `structural_bounds_dominate_the_real_scorer`
   covers), state whether the value is still an upper bound on the score of
   every alignment it claims to dominate, and whether the committed/suffix
   split double-counts or drops a term. Report whether the fix is exercised
   non-vacuously.
3. Hard-coding fence: `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"`
   over `src/` returns nothing outside `#[cfg(test)]` fixtures, CLI usage
   examples and ordinary English in comments.
4. No re-baselining: confirm no pinned literal in `tests/corpus_integration.rs`
   was swapped to match new output, that `approximate_output_is_locked` is
   untouched, and that `approximate_finds_classic_madgab_resegmentation` was
   not relaxed.
5. Wall clock for the two canonical cases at `--top 50` against 1.280 s and
   1.347 s, and the reference pool/rank-50 values (case 2 pool 18,936 /
   cutoff 0.915121574454; case 1 pool 18,270 / 0.918796440893). Report
   whether canonical case 2 is in the pool or not.
6. A one-line verdict, **INTEGRATE** or **REJECT**, plus any named fixes a
   retry must carry. Record the whole review in this item on this item's
   branch, commit and push it. Do not integrate anything yourself; do not
   merge to `main`.

`cargo fmt`, `cargo clippy` and doctests cannot run on this host; do not
report them as satisfied. See [../../environment-notes.md](../../environment-notes.md).

## Handoff / notes

### Pass 2026-09-27T07:53Z (coordinator coord-2b91): the two red guards are a regression, not a baseline

The front and the previous coordinator both recorded the two
`approximate_pool_reaches_*` guards as "deliberately red". The coordinator ran
the fence on both heads this pass and that is false: at base `707fb2a` (the
source of `post-milestone-acceptance` 12338b7) all three guards pass in
54.71 s, and at `b49f892` two of them fail. Full numbers are in
[w-5b1e93](w-5b1e93.md) § "Pass 2026-09-27T07:53Z". `707fb2a..b49f892` is
`src/lib.rs` only, so the test file is identical in both runs.

`b49f892` is therefore not integrable whatever the algebra looks like, and
criterion 1's expectation that those two guards be "reported explicitly" is
now a sharper question than the item first framed: the reviewer must say which
site in the diff stops them seeing their wordings, and must not accept a
re-baseline. The same measurement was sent to agent-6d2af301 as steer 1, with
the four specific questions it must answer — whether the tightening is
confined to inadmissible cost forms, whether `similarity_axis` is now the
scorer's own function at all four sites and whether `partial_span_score`'s
deliberate per-word exception is still honest, whether the new `heap_key`
quantisation assertion is non-vacuous, and the usual hard-coding and
no-instrumentation checks. It was asked for INTEGRATE / FIX / HOLD with the
failing evidence attached, and to push `madgab-review-bound-6d2af3` so the
report is durable. Left running.

### Next action for a later fresh pass

1. Read the verdict here. On INTEGRATE, rebase the reviewed commits onto the
   then-current `post-milestone-acceptance`, apply named fixes, integrate —
   never `main`. On REJECT, record the named fixes in
   [w-5b1e93](w-5b1e93.md) and steer the front once, then leave it.
2. Criterion 1 of [w-5b1e93](w-5b1e93.md) is still the milestone blocker; it
   is not closed by a green fence alone. A green fence on the whole suite,
   including the two `approximate_pool_reaches_*` guards, is now a
   precondition of any INTEGRATE verdict.
