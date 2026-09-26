---
work_item: true
id: w-7fa26c
state: working
priority: high
owner: agent-a1b2c304
updated: 2026-09-26T10:05:00Z
branch: madgab-approx-runtime
worktree: /workspace/madgab-approx-runtime
---

# Make approximate search fast again (behaviour-preserving)

## Goal

`madgab --approximate` should be usable interactively. Behaviour must not
change: this is pure constant-factor and complexity work on
`Generator::generate_approximate`.

## Context

Supporting measurements: [approximate-runtime-profile.md](../approximate-runtime-profile.md),
by Antonina agent `a1b2c302` against commit 247404f.

Headline findings:

- corpus load is 0.40-0.48 s and is never the problem; 95-99 % of the time is
  in `generate_approximate`;
- `prune_partials` is 93-95 % of search time;
- ~78 % of total search was one `sort_by` in `prune_partials` whose comparator
  re-ran `Partial::metrics` on both operands, although the results were already
  materialised in a local `metrics: Vec<Metrics>`. Measured 3.8-4.9x on its
  own;
- `Partial::metrics` is O(W x T) in clue words x target words with heap
  allocations: `candidate_reuses_target` -> `novelty_stem` on both operands was
  41-49 % of the *remaining* time, `boundary_novelty`'s two `HashSet<usize>`
  5-8 %, `lexical_shape_quality` -> `normalized_word` allocation 5-7 %;
- `Partial::extend_parts` clones the whole word and cut vectors and grows a
  `key: String` with `format!`, i.e. O(W^2) per path;
- measured time vs target IPA length `n`: 3.29x longer target gave 15.9x more
  time, i.e. about n^2.2;
- `matches_at`, `select_diverse`, the structural DP, the lexical heap and the
  corpus load are all under 7 % and are not worth optimising;
- parallelising is pointless: the cost is one sequential beam -> prune chain,
  and the only parallelisable regions are 0.2 % and 0.4 s.

Some of this is already banked by commit 6250ba3 on
`madgab-approx-acceptance`, which memoised the target-word reuse test, counted
`boundary_novelty` by merge instead of building `HashSet`s, and made beam
retention deterministic. That work took the measured wall clock down 3-5x
(`congratulations on your promotion` 48.5 s -> 11.5 s). The sort-comparator fix
was *not* included, so the single largest win is still available.

## Completion criteria

- The `prune_partials` sort orders indices against the already-cached metrics,
  with no `metrics` call inside a comparator. Verify by instrumenting or by
  asserting call counts drop, not just that time drops.
- `Partial::metrics` no longer re-derives per-word aggregates from the word
  vector. If incremental fields are added to `Partial`, the arithmetic order
  must be preserved so results are bit-identical.
- `lexical_shape_quality` and the reuse test do not allocate on the hot path.
- Output is bit-identical to the pre-change binary across a spread of targets
  and configurations, with a short, reproducible comparison recipe in the
  handoff. (Note [w-5d03af](w-5d03af.md): exact mode is not currently
  deterministic, so exact-mode comparisons must be excluded or compared by
  score sequence only.)
- `cargo test --release` is green.
- Report the before/after wall clock for every target and configuration used in
  the comparison.

Not yet implemented. Do P1 first and measure before touching anything else.
Treat the remaining items as a list, not a quota: if P1 alone is a large win
and the rest are invasive (`Rc`-based `Partial`, interned keys), stop and
hand off rather than destabilising the engine.

## Handoff / notes

Claimed 2026-09-26T08:50Z for Antonina agent `a1b2c304`, worktree
`/workspace/madgab-approx-runtime`, branch `madgab-approx-runtime`, base
`d46d154` (the integrated `post-milestone-acceptance`). **Use d46d154 as the
comparison base, not the 247404f figures in this item**: the scorer, the
shortlist sizes and the DP all changed in 6250ba3, so the before-numbers
must be re-measured on d46d154 and the improvement reported relative to
that.

The dirty tree a1b2c302 left in this worktree was not thrown away. Its
`TEMP-PROF` scaffolding and the P1 fix are committed and pushed as
`archive/prof-scaffold-2026-09-26` (664d5c7), and the worktree was then
reset to d46d154. If P1 needs re-validating, read that branch first: its
final `prune_partials` sort already orders indices against the cached
`metrics` vector, and `prof/` (58 MB of untracked binaries) is still on
disk there for re-measurement. That branch is an archive, not an
integration source.

Expect a merge conflict in `src/lib.rs` with the clue-quality front
([w-9c2d51](w-9c2d51.md)); the two touch different functions but the same
file.

Toolchain caveat: `cargo fmt` and `cargo clippy` do not exist on this host,
so the "bit-identical output" criterion has to be demonstrated by the
comparison recipe in this item rather than by a diff. See
[../../environment-notes.md](../../environment-notes.md).

## Pass 2026-09-26T10:05Z (coordinator, comparison-base change)

Antonina agent `a1b2c304` is running in this worktree and is left running.
One thing has changed underneath it that it cannot see from inside the
worktree.

**The comparison base is no longer `d46d154`.** The content-word front
([w-9c2d51](w-9c2d51.md)) was integrated into `post-milestone-acceptance`
during this pass as **50bdda1** / **9633013** (merge of
`madgab-clue-quality` 4a0bedb), which adds a scoring axis to approximate
mode. The "output is bit-identical to the pre-change binary" criterion in
this item must therefore be demonstrated against a binary built at
**9633013**, not at `d46d154`. Comparing against `d46d154` will show
approximate-mode output differences that are the clue-quality axis, not a
behaviour change from the performance work, and reporting those as a
regression would be wrong.

Practical consequence: this branch's diff will need to apply on top of
9633013, and its merge may conflict in `src/lib.rs` with the merged
front. The clue-quality change touched `Partial::metrics`, `Partial`,
`prune_partials`, the span shortlist, the segmentation DP and the lexical
`bound` closure, so the conflict is likely to be in the same regions this
front is optimising. Keep the perf changes separable from scoring
semantics — if a hunk has to choose between "same arithmetic, fewer
allocations" and "the new axis's exact expression order", prefer the new
axis and note it in the handoff, since the front's own criterion is
arithmetic-order preservation and the new axis is what the final score
now means.

Also note the exact-determinism front ([w-5d03af](w-5d03af.md)) landed as
7fa8dc6, so exact-mode output is now stable across processes and
exact-mode comparisons in the recipe are no longer excluded for
nondeterminism.

