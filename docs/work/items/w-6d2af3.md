---
work_item: true
id: w-6d2af3
state: done
priority: high
owner: "agent-6d2af301 (TERMINAL 2026-09-27T08:11Z: succeeded, exit 0, 1 prompt; verdict REJECT, recorded on madgab-review-bound-6d2af3 at da47326 and integrated verbatim into post-milestone-acceptance as b88051e) / coord-2f9c (integration of the review record only; no source change, nothing from the emission-bound front integrated)"
updated: 2026-09-27T08:12:00Z
branch: madgab-review-bound-6d2af3
worktree: /workspace/madgab-review-bound-6d2af3 (agent-6d2af301 terminal at 08:11Z with the report pushed; clean tree at da47326; reviewed range 707fb2a..b49f892 unchanged, 4f76b16 above it is docs-only)
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

### Pass 2026-09-27T08:12Z (coordinator coord-2f9c): closed - the review is delivered and integrated

Agent 6d2af301 is terminal (`succeeded`, exit 0, 1 prompt) and its verdict is
**REJECT** on the emission-order front at `b49f892`, recorded on
`madgab-review-bound-6d2af3` at `da47326`. That report is a docs-only commit
and its content is integrated verbatim into `post-milestone-acceptance` as
`b88051e`, so the accumulated record of this review is the reviewer's own
file.

Completion criteria: 1 met (fence reported per test, the two depth-reach
guards red at the unchanged pinned literals `ask lovey` / pool 11,235 and
`see if a law thus ish l.'s` / pool 17,230, against 11,236 and 17,236 at
`958771d`), 2 met (admissibility re-derived, the committed/suffix split shown
to partition `0..depth` exactly, both coverage gaps measured by mutation),
3 met (hard-coding fence clean), 4 met (no re-pinning,
`approximate_output_is_locked` untouched, `tests/` diff empty over
`707fb2a..b49f892`), 5 met (pool and cutoff tables for both cases, the host's
non-reproducibility of the reference figures stated, canonical case 2
explicitly 0 of 18,859), 6 met (REJECT plus named fixes, committed, pushed,
nothing merged). Item is `done`.

Canonical result still measured, not assumed: `wreck a nice beach` **is**
produced for `recognize speech` (rank 27 of 18,816 at `--top 50`); `Hits
Justice Dupe Hid Came` remains 0 of 18,859 for `It's just a stupid game`, so
criterion 1 of the milestone is still unmet.

Consequence for [w-5b1e93](w-5b1e93.md): the emission-order head is closed as
un-integrable on evidence, not merely unreviewed. Its named fixes are the
checklist any successor must carry, and its `blocked` state stands.

### Next action for a later fresh pass

Read this verdict before re-reading the front branch. The b49f892 fixes worth
carrying forward onto any successor are: make the two depth-reach guards
green with the pinned literals unchanged, or replace them with a property
about the *pool* (wordings outside the opening-width product that survive
into `generate_pool` above the rank-50 cutoff) rather than about
`coverage_tuples`' return value, which the current replacement test only
re-derives; give the sibling sites the tight-end identity so reverting the
normaliser turns the suite red; de-circularise the lexical property test by
pinning the scorer's similarity term independently as
`1 - (cost / phones) / axes::SIMILARITY_COST_PER_PHONE`; delete or substantiate
the "0.167 of the axis" novelty claim that mutation contradicts; delete the
dead `target_boundaries_of` and `parts` scaffolding.

### Review of `707fb2a..b49f892`, 2026-09-27 — **REJECT** (fix list below; a retry must carry fixes 1-5)

Reviewed `707fb2a..b49f892` (`958771d`, `28d5e4a`, `b49f892`; `src/lib.rs`
only, +644/-72) from this worktree at `b49f892`, release, dedicated
`CARGO_TARGET_DIR=/workspace/madgab-review-bound-6d2af3/target`. Baseline
trees for the red-checks and the pool probe were separate worktrees
(`/workspace/rb6d2af3-probe` at `b49f892`, `/workspace/rb6d2af3-base` at
`707fb2a`, branches `scratch/6d2af3-*`, their own
`CARGO_TARGET_DIR=/workspace/rb6d2af3-{probe,base}-target`), both removed
after use; the front worktree was not touched. `cargo fmt`, `cargo clippy`
and doctests cannot run on this host and are **not** claimed.

The front's later `4f76b16` touches only `docs/`, so these numbers still
describe the head of the reviewed range. This verdict is **REJECT as
landed**, which is the same shape as [w-3c9d17](w-3c9d17.md): the algebra
is sound and should be carried forward, the fence is red, and criterion 1
of [w-5b1e93](w-5b1e93.md) is unmet.

#### 1. The fence — criterion 5 of [w-5b1e93](w-5b1e93.md)

Release, this tree, `b49f892`:

| suite | result |
|---|---|
| `--lib` | **63 passed, 0 failed** (61 at `958771d`; +2 new tests) |
| `--test corpus_integration` | **10 passed, 3 failed** |
| `--test emit_coverage` | **4 passed, 0 failed** |
| `--test approx_determinism` | **4 passed, 0 failed** |
| `--test exact_determinism` | **1 passed, 0 failed** |
| `--test no_phrase_hard_coding` | **9 passed, 0 failed** |

The three `corpus_integration` failures:

* `approximate_pool_reaches_matches_deep_in_a_span` — **still red**, and
  **still red at the same literal**: `I love you`: `"ask lovey"` missing,
  pool 11,235.
* `approximate_pool_reaches_resegmentations_deeper_than_one_walk` — **still
  red**, same literal: `"see if a law thus ish l.'s"` missing, pool 17,230.
* `approximate_finds_classic_madgab_resegmentation` — canonical case 2,
  pre-existing, identical 12-item top-50 list on both trees.

The two named guards are the only thing this front had to deliver on and
neither moved. Criterion 5 is not met.

**Which site in the diff stops them seeing their wordings** (steer 1's
question). It is **`958771d`**, the single `1 - cost/4.0` ->
`similarity_axis` change in the lexical heap key — not `28d5e4a` and not
`b49f892`. [w-3c9d17](w-3c9d17.md) §4 already measured both guards passing
at `707fb2a` and failing at `958771d` at these same literals, with pools
11,236 and 17,236; my pools at `b49f892` are 11,235 and 17,230, i.e. the
two later commits moved the pool by -1 and -6 and recovered neither
wording. The rest of the range cannot be the cause: `heap_key` is a pure
extraction of the existing `quantized` closure (no behaviour change), and
the novelty re-derivation is provably a **no-op at this site** — for a
complete segmentation `|A| = depth - 1`, `SegPath::shared` is `|A ∩ B|`
counted once per distinct increasing end (`src/lib.rs:1413`), and
`boundary_novelty` filters the terminal cut with `c < total_len`, so
`1 - shared/(target_inner + (depth-1) - shared) == 1 - |A ∩ B| / |A ∪ B|`
exactly. I confirmed this by mutation: restoring the old novelty formula
into the traversal leaves every test green (§2).

**Is the tightening confined to inadmissible cost forms?** The only pruning
change in the range is the similarity normaliser, in three bound sites; it
is general, and it is a correctness fix, not a special case. The front's
attribution — that the repair removed a mis-ranking which was incidentally
surfacing wordings that score far below the cutoff — is consistent with
everything I measured (I confirmed both wordings are absent from the pool
the tests read) but I did **not** independently reproduce their 0.590-0.610
scores and do not adopt that number. Either way the verdict is the same:
the fence is red, the acceptance goal is unmet, and the guards' property
has not been stated in a form that survives.

#### 2. Admissibility, re-derived

Notation: `n = chars.len()` where `chars` is the target's IPA
(`src/lib.rs:823`) — the same integer the scorer divides by as
`total_len` (`src/lib.rs:2774`, `into_clue`), so `n` is the scorer's
*denominator*, and `P = n.max(1) >= 1`; `C*` is the total edit cost of any
completion of a prefix, `C_min` the bound's cost; `c = 0.30 =
axes::SIMILARITY_COST_PER_PHONE`; `wc = depth`.

**`lexical_score_bound` — admissible, still an upper bound, split exact.**

* *Committed/suffix split.* `committed` sums slots `0..k-1`; `suf[k]` sums
  slots `k..depth-1`. For `k = prefix.len()` the two index sets partition
  `0..depth`. **No term is double-counted and none is dropped.**
* *SIMILARITY.* `similarity_axis(C_min, n) = (1 - (C_min/n)/c).clamp(0,1)`
  against the scorer's `(1 - (C*/n)/c).clamp(0,1)`; `C_min <= C*` and
  `x -> (1-x).clamp(0,1)` is non-increasing, so `B >= S(C*)` for every
  `n >= 1`, at both clamp ends (both are 0 when `C* >= n c`). The old
  `1 - C/4.0` sat below the score for every `n > 4/0.30 = 13.3` phones.
  Repaired correctly.
* *NOVELTY.* Now `boundary_novelty(seg_cuts, &target_boundaries, n, false)`,
  the scorer's own expression on the segmentation's own cuts. Every
  completion of a fixed segmentation has the same cut set and the same
  `partial = false`, so this term is **exact, not relaxed** — an equality,
  hence an upper bound. Sound, and a no-op relative to the old formula.
* *WORD_NOVELTY.* `relaxed.reused = Σ_{k∈suffix} 1[all candidates in slot k
  have reused == 1] <= ` the reuse count of any completion, and
  `1 - reuse/wc` decreases in `reuse`. Upward-relaxed, admissible.
* *FAMILIARITY, SHAPE.* Per-slot `max` summed over the suffix is `<=` the
  sum any completion realises, over the same `wc` the scorer divides by.
  Admissible.
* *CLOSED_CLASS.* Per-slot minimum closed count `<=` the real count;
  `closed_class_penalty` is convex, so the minimum stays a lower bound on
  the penalty after the square. Admissible (pre-existing, unchanged).
* *RHYTHM.* Both ends of the suffix's syllable interval are carried and
  `rhythm_match_in(min, max, target_syllables)` is monotone upward in `min`
  and downward in `max`. Admissible, and tighter than a single extremum.
* *PUNCH.* Omitted. Its term is `PUNCH * (share - 1) <= 0` with maximum
  exactly 0, so the maximum over completions contributes nothing.
  **Omission is an upward relaxation, not a dropped term.**

So: 7 axes bounded upward, 1 exact, 1 omitted-upward, 0 dropped, 0
double-counted. `lexical_score_bound` is an upper bound on the score of
**every** alignment it claims to dominate.

**The three repaired sibling sites — correct, and two of the three are
unasserted.**

* `complete_span_score` (`src/lib.rs:3587`) reads
  `similarity_upper(ext.min_cost, target_phones)`, and its call site
  (`src/lib.rs:1336`) passes `n`, the target phone count. `ext` is a
  `SpanExtremes` for a complete span path, so `min_cost` is a sum of
  per-span minima along that path and is a lower bound on the structure's
  total cost. **Admissible.** It is a ranking key, not a discard key, so
  this is an improvement rather than a correctness fix. Its `NOVELTY` term
  still uses the other derivation, `1 - shared/(words-1+target_inner-shared)`;
  for a complete path that equals the scorer's expression, so it is
  admissible — but it was not brought along with the similarity repair, and
  the new doc comment's "this is the one expression every bound has to go
  through" does not extend to it.
* `span_score_bound` (`src/lib.rs:3642`) reads
  `similarity_upper(ext.min_cost, target_phones)` with `target_phones = n`
  from `src/lib.rs:1351`. `head.upper_plus(tail)` sums along one path and
  `tail[at]` is a per-axis `best_of` (min of costs, max of the rest) over
  the alternatives, so `ext.min_cost` is a lower bound on the total cost of
  **every** completion through that path — which is what the function's own
  doc comment claims. **Admissible**, now with the denominator the scorer
  uses. This is the one repair in the range that was a real correctness
  fix: this key is allowed to discard a path outright.
* `structural_bounds_dominate_the_real_scorer` (`src/lib.rs:5610`) now
  passes `total` (the phone count) to both, so it does *exercise* them. It
  is **not a guard**. Measured on the scratch tree at `b49f892`: with
  `similarity_upper` reverted to `(1.0 - min_cost/4.0).clamp(0.0, 1.0)`,
  `structural_bounds_dominate_the_real_scorer` **still passes**. The
  domination assertion is masked by slack in the other axes over
  `reachability_corpus`'s short targets. This is exactly the gap
  [w-3c9d17](w-3c9d17.md) §5.3 named, and it is still not catching the
  repair after the repair.

**The new property assertions — non-vacuous, but not where claimed.**

`the_lexical_bound_dominates_every_completion_it_covers` bites in the
relaxation directions and is blind in the direction its doc comments credit
it with. Measured on the scratch tree at `b49f892`, one mutation at a time:

| mutation | result |
|---|---|
| over-charge the bound's cost by `+1.0` (`similarity_axis(committed.cost + relaxed.cost + 1.0, …)`) | **FAILS** (caught) |
| drop the bound's novelty by `0.05` | **FAILS** (caught) |
| revert `similarity_axis` to the per-word `1 - cost/4.0` | **passes** |
| restore the old novelty formula | **passes** |

The two passing rows are the point. The test compares `lexical_score_bound`
against `Partial::metrics`, and **both call `similarity_axis`**, so a change
to the shared normaliser moves the reference and the measurement together
and is invisible; the novelty term is shared the same way. The assertion is
sound — the relaxation directions really are checked — but it is a
self-referential identity, and the doc comment's "The per-word form fails
that assertion on every target here" is a claim the test does not
establish. The tight-end identity (`bound - score ==` the `PUNCH` shortfall
at full length) is the strongest part of the test and does hold.

The `heap_key(bound) >= heap_key(score)` assertion is correct and answers
[w-3c9d17](w-3c9d17.md) §5.5 (steer 1's question): `(x * 1e9).round() as i64`
is monotone non-decreasing in `x`, so the key inherits the bound's
domination. It adds no evidence beyond the float assertion, but the check
was asked for and it is the right one.

`the_coverage_reserve_reaches_outside_the_opening_width_product`
(`src/lib.rs:6116`) is **not** a substitute for the two red guards. It calls
`coverage_tuples` directly with a constant bound closure, and
`sweep_index` draws its ranks past the floor by construction, so "some
coordinate is `>= cap`" restates the sweep's own design invariant. It never
calls `generate_approximate`, never builds a pool, never ranks, never
compares against the scorer's cutoff, and passes if only 3 of its 14 targets
contribute. What the two red guards assert is that a *deep* wording reaches
the **pool**. Nothing in this test says it does.

#### 3. Canonical cases and the pool, at `--top 50`

Public `Generator::generate_pool`, `max_rarity: None`, `top_n: 50`,
`beam_width: 64`, release, three runs each, on the head and on its parent
`707fb2a` in a scratch worktree. Pool and cutoff are deterministic across
runs.

| target | pool @707fb2a | pool @b49f892 | rank-50 cutoff @707fb2a | rank-50 cutoff @b49f892 |
|---|---|---|---|---|
| `recognize speech` | 18,834 | **18,816** (-18, -0.10%) | 0.918796440893 | **0.918796440893** (identical) |
| `it's just a stupid game` | 19,230 | **18,859** (-371, -1.93%) | 0.913032117383 | **0.913453604342** (+0.000421) |

**The reference values in this item's brief do not reproduce, loudly.**
Case 1's reference cutoff `0.918796440893` reproduces exactly, but its pool
is given as 18,270 where this tree gives 18,834 (and its own parent gives
18,834); case 2's reference 18,936 / `0.915121574454` does not reproduce at
all — the parent alone gives 19,230 / `0.913032117383`. Those references are
for a different tree and/or configuration, so I do not score the front
against them; every comparison above is parent-vs-head on one harness, one
build per tree.

Canonical outcomes:

* `wreck a nice beach` for `recognize speech`: **produced**, 1 of 18,816
  pool members, printed by `madgab --approximate --top 50` at **rank 27**.
  Case 1 is intact.
* `hits justice dupe hid came` for `it's just a stupid game`: **0 of
  18,859**. Still never enumerated. Criterion 1 of
  [w-5b1e93](w-5b1e93.md) is **not** met by this range.

`b49f892`'s own contribution on top of `28d5e4a`: case 2's pool moves from
18,815 to 18,859 (**+44 members, +0.23%**) and its rank-50 cutoff by
**0.000000**. The three-commit range as a whole is a priced negative on the
acceptance goal: -371 pool members and +0.000421 of cutoff on case 2, and
the clue is still 0 for 18,859.

Wall clock, `generate_pool` search phase, medians of three:

| case | @707fb2a | @b49f892 | delta |
|---|---|---|---|
| `recognize speech` | 3.077 s | 2.763 s | -10.2% |
| `it's just a stupid game` | 4.601 s | 4.729 s | +2.8% |

The 1.280 s / 1.347 s references are **not reproducible on this host on
either tree** — the parent alone measures 3.077 s and 4.601 s, 2.4x and
3.4x the references — so I claim neither that they are met nor that they
are missed; I claim only that the head does not regress against its parent
on one harness. Whole-process wall clock for `madgab --approximate --top 50`
is 6.409 s and 6.300 s (corpus load 2.487 s / 2.168 s, search 3.355 s /
3.496 s). The CLI's default `--max-rarity 50000` gives a different pool from
the reference configuration; its printed rank-50 cutoff is 0.913 for both
cases and is not comparable to the numbers above.

#### 4. Hard-coding fence

`git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"` over
`src/` at `b49f892`: 27 hits, every one accounted for:

| category | where | count |
|---|---|---|
| `#[cfg(test)]` fixtures | `src/lexical.rs:302,303,335` (that `mod tests` opens at `src/lexical.rs:260`), `src/lib.rs:4075,4363-4375,4417-4422,4465,4493-4494,4543,4564,5487-5498,5554-5561,6283-6286` | 22 |
| CLI usage examples | `src/main.rs:9,11` | 2 |
| ordinary English in a comment / a message | `src/lib.rs:3060` ("became a per-*phone* axis"), `3308` ("key came from cells"), `4168` ("candidate came from this pool") | 3 |

The **only** line added anywhere in `707fb2a..b49f892` matching the pattern
is `src/lib.rs:3060`, a doc comment. No phrase-specific case, no substring
test, no numeric constant tuned to a fixture. `--test no_phrase_hard_coding`
9/9. **Clean.**

#### 5. No re-baselining

`git diff 707fb2a..b49f892 -- tests/` is **empty** — 0 lines. Every
`tests/` file is byte-identical to the parent, so
`approximate_output_is_locked` is untouched and **passes**, and
`approximate_finds_classic_madgab_resegmentation` is untouched and fails
with the identical 12-item list. No pinned literal was swapped; the front
correctly reverted the swap steer 4 caught. `examples/` contains only
`measure.rs`: no `zzz_*`, no `ZZ_*`, no dump or survey example, no
temporary counter is tracked. **Clean.**

#### Named fixes a retry must carry

1. **The fence.** `approximate_pool_reaches_matches_deep_in_a_span` and
   `approximate_pool_reaches_resegmentations_deeper_than_one_walk` are
   still red at `b49f892` at the same literals. A retry must make them
   green **with the pinned literals unchanged**, or replace them with a
   property that is about the *pool* — wordings outside the opening-width
   product that survive into `generate_pool` above the rank-50 cutoff — not
   about `coverage_tuples`' return value, which the current replacement
   test only re-derives. A re-pin is not an acceptable route.
2. **Assert the sibling repair.** `structural_bounds_dominate_the_real_scorer`
   passes with `similarity_upper` reverted to the stale `/4.0` form
   (measured). Give those two sites the same tight-end identity the lexical
   test uses — at a completed structure, bound == score up to the `PUNCH`
   shortfall — so reverting the normaliser turns the suite red.
3. **De-circularise the lexical property test.** It compares the bound to
   `Partial::metrics` and both read `similarity_axis`, so the per-word
   regression its doc comments credit it with finding leaves it green
   (measured). Pin the scorer's similarity term in the test as an
   independent expression, `1 - (cost / phones) / axes::SIMILARITY_COST_PER_PHONE`,
   and assert the bound against that.
4. **Delete or substantiate the novelty claim.** The comment at
   `src/lib.rs:1711-1722` and the `lexical_score_bound` doc say boundary
   novelty "is a second, independent source of the same defect", differing
   "by 0.167 of the axis, which is 0.025 of a final score", credited to the
   new test. Restoring the old formula leaves the new test green
   (measured), and the two are provably equal at this site (see §1). The
   new form is the better one to keep; the measurement claim is not
   supported. Same class as [w-3c9d17](w-3c9d17.md) named fix 4.
5. **Remove the dead test scaffolding.** `fn target_boundaries_of`
   (`src/lib.rs:6089`) is never used and the `parts` closure
   (`src/lib.rs:5924`) is bound and never called; both emit build warnings
   (`dead_code`, `unused_variables`) on a clean `b49f892`. Delete or use
   them.
6. Not a defect, recorded so it is not re-litigated: `partial_span_score`
   still reads the old per-word normaliser, deliberately, because it is a DP
   representative's ranking weight and not a bound. The new doc comment
   says so and the reasoning is sound — but the *adjacent* `NOVELTY`
   derivations at the three sites are the ones a future pass should look
   at, and `complete_span_score`'s is the one still on the old derivation
   while its similarity term has moved.

#### Reproducing

```sh
export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"
export CARGO_TARGET_DIR=/workspace/madgab-review-bound-6d2af3/target
cd /workspace/madgab-review-bound-6d2af3        # b49f892
cargo test --release --lib
for t in corpus_integration emit_coverage approx_determinism \
         exact_determinism no_phrase_hard_coding; do
  cargo test --release --test $t
done
```

The red-checks in §2 and the pool probe in §3 were run in
`/workspace/rb6d2af3-probe` and `/workspace/rb6d2af3-base` on branches
`scratch/6d2af3-probe` / `scratch/6d2af3-base` with their own
`CARGO_TARGET_DIR`s, each mutation applied by a script and reverted with
`git checkout -- src/lib.rs` **inside the scratch tree**; both worktrees
were removed afterwards and neither carries a commit. No `zz*` file was
committed anywhere.

### Pass 2026-09-27T08:00Z (coordinator coord-3f77): your target is unchanged, but the front is now terminal

Not a steer, and the review was **not** interrupted. One fact for the record,
because it bears on the verdict you owe:

- The reviewed range is still `707fb2a..b49f892`. The front pushed
  `4f76b16` on top, and that commit touches **only**
  `docs/work/items/w-5b1e93.md` (+492/-1) — no `src/`, no `tests/`. Your
  bytes have not moved, so your fence numbers still describe the head.
- Agent `5b1e930` is **terminal** (`succeeded`, exit 0, 9 prompts) and its own
  recommendation is **HOLD**: it agrees the two `approximate_pool_reaches_*`
  guards are red at its head and that the pool-reach wordings it loses
  (`ask lovey`, `see if a law thus ish l.'s`, `see ish air law this ish l.'s`,
  scoring 0.590-0.610 against a ~0.90 cutoff) were surfaced by the old key
  *because it mis-ranked*. It declined to re-pin them and asserts the property
  in-crate instead.
- Its attribution claim, which you should check rather than inherit: the two
  guards are red because the repair removed a mis-ranking that was incidentally
  surfacing low-scoring wordings — i.e. the pruning is confined to
  inadmissible cost forms, not a general narrowing of the lattice. If that is
  right, the honest outcome is **FIX** with a named, general way to state the
  guards' property without a literal swap; if the guards are the only thing
  holding those wordings in the pool, the pruning is general and the verdict is
  **REJECT**.

[w-5b1e93](w-5b1e93.md) is now `blocked` on exactly this, and the successor
front [w-0f3a17](w-0f3a17.md) is open for the different mechanism your
measurement points at: the clue is outside the enumerated lattice
(`affordable_opening_width` returns 7 at depth 5, so the canonical tuple's
per-slot indices 13/99/11 are never pushed), not late in its order.

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
