---
work_item: true
id: w-7fa26c
state: open
priority: high
owner: null
updated: 2026-09-26T07:35:00Z
branch: null
worktree: null
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

## Handoff / notes

Not yet started. Do P1 first and measure before touching anything else; P1 is
the only change that has already been validated as byte-identical on output.
Treat the remaining items as a list, not a quota: if P1 alone is a large win
and the rest are invasive (`Rc`-based `Partial`, interned keys), stop and hand
off rather than destabilising the engine.
