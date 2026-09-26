---
work_item: true
id: w-7fa26c
state: working
priority: high
owner: agent-a1b2c304
updated: 2026-09-26T11:05:00Z
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

---

# Handoff — agent a1b2c304, branch `madgab-approx-runtime`

Base for every before/after number: **d46d154** (the current content of
`post-milestone-acceptance`). Nothing was merged to or pushed at `main`.

## What changed

Commit `Speed up approximate search without changing its output`
(`src/lib.rs`, `tests/corpus_integration.rs`). Three behaviour-preserving
changes, all arithmetic-order preserving:

1. **P1 — `prune_partials` orders indices against the cached metrics.**
   The final `out.sort_by(|a, b| cmp_desc(score_of(a), score_of(b)))` sorted
   `Vec<Partial>` with a comparator that called `Partial::metrics` on both
   operands, even though the results were already materialised in the local
   `metrics: Vec<Metrics>`. It now sorts the retained `Vec<usize>` against
   `metrics[i].combined` (with an index tie-break, see "Determinism" below).
   The six intermediate objective rankings, the structural-cell protection
   and the membership rule are untouched.
2. **Dedup scores each surviving candidate once.** The dedup loop scored the
   incumbent again for every later duplicate. It now scores a key only when
   that key actually collides, caching the incumbent's `combined` next to it.
   Candidates that never collide are scored exactly once, in the `metrics`
   vector.
3. **P2/P3 — `Partial` carries its per-word aggregates.** `Partial::metrics`
   re-derived `reused`, `familiarity` and `shape_quality` by folding the whole
   `words` vector on every call. `extend_parts` now accumulates
   `reused_count: usize`, `familiarity_sum: f64` and `shape_sum: f64` as the
   path grows, so `metrics` is O(1) plus `boundary_novelty`. A running total
   appended in clue-word order is bit-identical to re-folding the vector
   (`sum()` is also a left fold), which
   `incremental_aggregates_match_a_full_refold` asserts against the old
   definition. `extend_*` now takes the `&TargetPhrase` it needs for the reuse
   test; `metrics`/`finish`/`into_clue`/`prune_partials` no longer take it
   because they no longer need it. `lexical_shape_quality` counts the
   normalized characters in place instead of materializing the normalized
   string; the reuse test was already memoized and is now covered by a call
   count test.

**Stopped short of P5** (`Rc`-based `Partial`, interned path keys) and P4
(`boundary_novelty` bitmask), as the brief instructs — see "Why I stopped".

## Commands run

Validation commands (this host has no `cargo fmt`, `cargo clippy` and no
doctest runner; those were **not** run and are not claimed):

```
cargo build --release
cargo test --release --lib                 # 16 passed, 0 failed
cargo test --release --test corpus_integration
```

`cargo test --release --test corpus_integration` reports **5 passed, 2 failed**.
Both failures (`approximate_finds_classic_madgab_resegmentation`,
`approximate_finds_recognize_speech_resegmentation`) are **pre-existing at
d46d154** and reproduce byte-identically on an unmodified checkout of the base
commit, with the same assertion text and the same first 12 proposals:

```
git worktree add /workspace/baseline-check d46d154
cd /workspace/baseline-check && cargo test --release --test corpus_integration
# -> 4 passed, 2 failed (same two tests, same output)
```

So the "`cargo test --release` is green" criterion is **not** met, and cannot be
met by a behaviour-preserving change: those tests assert that specific
resegmentations are in the top 50, which d46d154's scoring/diversity work
changed. Filed as [w-a02d28](w-a02d28.md). Everything else in the suite is
green, including the new tests.

## Before/after wall clock

Recipe (host is a shared 32-core box; A and B are run alternately in the same
loop and the best of 3 is reported, so drift hits both equally):

```
cargo build --release
cp target/release/madgab target/madgab-baseline     # binary built from d46d154
# then, for each row below, A = target/madgab-baseline, B = target/release/madgab:
#   time A <args> ; time B <args>       (3 reps, report the minimum)
```

All rows are `--approximate`; the default corpus load is ~0.63 s of every
number (0.4-0.5 s `Corpus::from_json` + 0.13-0.15 s `build_lexicon`), so the
search-only speedup is larger than the totals below.

| configuration | target | before (ms) | after (ms) | speedup |
|---|---|---|---|---|
| `--approximate --top 20` | recognize speech | 4395 | 2983 | 1.47x |
| `--approximate --top 20` | It's just a stupid game | 6124 | 3383 | 1.81x |
| `--approximate --top 20` | I love you | 2180 | 1393 | 1.56x |
| `--approximate --top 20` | there is no place like home | 6870 | 3949 | 1.74x |
| `--approximate --top 20` | congratulations on your promotion | 11238 | 5613 | 2.00x |
| `--approximate --top 20` | my favorite color is blue | 7126 | 4915 | 1.45x |
| `--approximate --top 20` | insurance is important | 6839 | 3900 | 1.75x |
| `--approximate --top 20` | the quick brown fox jumps over | 7661 | 4879 | 1.57x |
| `--approximate --top 20 --beam 32` | recognize speech | 3338 | 2666 | 1.25x |
| `--approximate --top 5 --per-word-budget 0.8 --total-budget 2.5` | It's just a stupid game | 10938 | 5213 | 2.10x |
| `--approximate --top 20 --max-rarity 200000` | I love you | 2917 | 2133 | 1.37x |

The 11-row A/B table above was also run as a 17-row sweep (8 targets at
`--top 20` plus `--beam 32`, `--top 5 --per-word-budget 0.8 --total-budget 2.5`
and `--max-rarity 200000` on three targets) with a standalone harness; the
per-run best-of-2 wall clocks there were 4992-13785 ms before and 1401-10252 ms
after, with no row slower. Exact mode is excluded on purpose
([w-5d03af](w-5d03af.md)): it is not deterministic, so it cannot be compared
byte-for-byte.

P1 alone (before P2/P3) measured 1.21-1.70x on the same 11 rows; P2/P3 took
the best cases to ~2x.

## Bit-identical comparison recipe

17 cases (8 targets x `--approximate --top 20`, plus
`--beam 32`, `--top 5 --per-word-budget 0.8 --total-budget 2.5` and
`--max-rarity 200000` on three of the targets), 2 reps each, stdout captured
and compared with `cmp`:

```
# baseline binary (built from d46d154) -> out-base
# new binary                            -> out-new
for f in out-base/*.txt; do cmp -s "out-base/$f" "out-new/$(basename "$f")" || echo "DIFF $f"; done
```

Result: **0 differences across all 17 cases**, re-checked against a final clean
rebuild. The baseline binary is itself reproducible across processes (three
consecutive runs of the same case are `cmp`-identical), and the new tree adds
the `.then(a.cmp(&b))` index tie-break to the final prune sort, so the retained
set is ordered deterministically rather than by `HashSet` iteration order; the
two agree because the pre-change order was already fully determined by
`combined`.

In-tree equivalent that runs on this host without two checkouts:
`tests/corpus_integration.rs::approximate_output_is_bit_identical_to_the_baseline`
pins the exact full-precision (6 dp) ranked proposals for three targets at
`--approximate --top 10`; those values were taken from the d46d154 binary.

## Instrumented call-count evidence (not just wall clock)

`src/lib.rs` has a `#[cfg(test)] mod counters` with thread-local `METRICS` and
`NOVELTY_STEM` counters, bumped inside `Partial::metrics` and `novelty_stem`.
They are compiled out of release builds. The counters are what the new tests
assert on:

- `prune_partials_scores_each_candidate_exactly_once`: for pools of
  64/256/1024/4096 distinct candidates pruned to `k = n/4`, `metrics` is called
  **exactly `n` times** — one per candidate, independent of `k`. The old
  comparator form needs `2 k log2 k` extra calls.
  Verified that this test *fails* if the comparator is restored: at `n = 64,
  k = 16` it reports 244 calls instead of 64 (and the gap grows with `k`).
- `prune_partials_returns_best_scoring_candidates_first`: the retained list is
  non-increasing in `combined` and leads with the pool's best candidate, i.e.
  the index sort preserves the order the comparator produced.
- `incremental_aggregates_match_a_full_refold`: for 48 hypotheses of one and
  five words, the incremental `metrics` equals the pre-incremental definition
  field-for-field (exact `f64` equality, both `partial = true` and `false`).
- `reuse_test_stems_each_word_once`: 1000 repeat `reuses()` calls on a warm
  word perform **0** further `novelty_stem` calls (1 for the cold word), so the
  reuse test does not stem or allocate per call.
- `lexical_shape_quality_matches_reference`: the allocation-free version
  equals the previous definition for 16 words (empty, punctuation-only, case,
  `İ` which lowercases to two chars, `straße`, `Å`/`å`, embedded space) at three
  familiarity values.

## Why I stopped here

Post-change phase profile (temporary, uncommitted instrumentation;
`--approximate --top 20`, seconds, `congratulations on your promotion`,
`generate_approximate` total 5.22):

| phase | s | share |
|---|---|---|
| `prune_partials` (all) | 1.62 | 31 % |
| — dedup (String hashing of path keys) | 0.35 | |
| — sort by path key (`String` compare) | 0.18 | |
| — build the `metrics` vector | 0.04 | |
| — cell protection + 6 objective rankings | 0.32 | |
| — final index sort **+ clone the `k` retained `Partial`s** | 0.45 | |
| recovery search (span shortlists, structural DP, lexical heap) | 2.43 | 47 % |
| — of which `prune_partials` | 1.62 | |
| `extend_fuzzy` (942 280 calls) | 0.86 | 16 % |
| `finish` + `select_diverse` | 0.05 | 1 % |
| lattice `matches_at` | 0.03 | 0.6 % |

What is left is the invasive half of the item, and it is now the dominant cost:
the final prune step is dominated by *cloning* `k` `Partial`s (each clones a
`Vec<ClueWord>` of `String`s plus the path `key`), the dedup and key-sort
stages are `String`-hash/`String`-compare bound, and `extend_fuzzy` clones the
whole word and cut vectors and grows `key` with `format!` (O(W^2) per path,
942 k calls for this target). Fixing those means `Rc`-based `Partial` and
interned path keys — exactly the restructuring the brief says to hand off
rather than destabilise the engine. P4's `boundary_novelty` bitmask is no longer
worth doing on its own: it is a merge over `W + T` `usize`s with no allocation
and is now inside the 0.04 s `metrics` vector build.

`matches_at` (0.6 %), `finish`/`select_diverse` (1 %) and the corpus load are
confirmed irrelevant, as the earlier profile said.

## Not verified here

- `cargo fmt`, `cargo clippy` and doctests do not exist on this host. Nothing
  is claimed about them. Line widths in the new code were kept in the
  surrounding style by hand.
- The wall-clock numbers come from a shared machine and move by 10-20 % between
  runs; the ratios are from an interleaved A/B loop, not from two separate
  sweeps.
- `cargo test --release` in **debug-adjacent** configurations was not run;
  every timing and test run here is `--release`.
- Exact mode was not compared at all (nondeterministic at the base, see
  [w-5d03af](w-5d03af.md)). `Partial` is shared with exact mode and the
  incremental aggregates feed it, so exact-mode scores should be unchanged, but
  that is reasoned, not measured.
- No WASM/browser check; `src/wasm.rs` was not touched.

## Next action

Either file P5 (`Rc`-based `Partial` + interned path keys) as its own item with
a fresh golden-output baseline, or land the perf work as it stands once
[w-a02d28](w-a02d28.md) has decided what the two red acceptance tests should
assert. The state stays `working` because the green-suite criterion is unmet.
