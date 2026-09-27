# Independent review of `784deaa` (w-1c3e77) on `madgab-enum-depth4`

Reviewer: independent pass, worktree `/workspace/madgab-review-1c3e77`,
branch `madgab-review-1c3e77`, forked from `8a5dd3b` (= `784deaa` + the
front's report). Scope: read `03fdf52..784deaa -- src tests` in full, review
the two mechanisms on their merits, re-derive the red/green evidence, re-run
the five named suites, and re-measure the milestone guards. Nothing was
pushed, no other worktree was touched, `main` is untouched.

## Verdict

**INTEGRATE WITH NAMED FIXES.** One fix, one hunk pair, now applied and
committed on this branch: the two production doc comments that name a
canonical acceptance example (`src/lib.rs:239` and `src/lib.rs:268` as landed
by `784deaa`) must not, and a new test in `tests/no_phrase_hard_coding.rs`
now makes that un-repeatable. With that applied, `784deaa` is a correct,
fence-clean, general wall-clock and pool-coverage improvement and should land
on its own justification, **not** as milestone progress — the milestone is
still blocked for the reason the front itself records, and I re-verified that
independently.

The mechanism is sound. The report's own bottom line — objective not met,
blocker is visibility, not enumeration — is accurate.

## 1. Fences, checked by eye and by the suite

Diff read in full: `src/lib.rs` +145/-87, `tests/corpus_integration.rs`
+51/-0.

| fence (w-1c3e77 body + w-a7e2b3) | verdict | evidence |
| --- | --- | --- |
| No phrase/word/substring of either canonical example in `src/` or `tests/`, **comments included** | **VIOLATED as landed; FIXED here** | `src/lib.rs:239` — ``/// `It's just a stupid game`, 87 of 13 640 on `recognize speech` — so this``; `src/lib.rs:268` — ``/// segmentations, 27 emit no wording at all on `It's just a stupid game`,``. Both are production doc comments above `slot_is_affordable` / `affordable_opening_width`, outside any `#[cfg(test)] mod`. These are exactly the two lines `w-a7e2b3` §2a and §3a(1) named (at its own line numbers 239 and 268 — unchanged) and the review's minimum edit set required removing. `tests/no_phrase_hard_coding.rs` strips comments, so the suite passed anyway: 6/6 on `784deaa`, exactly as the review predicted. |
| No `env::var` production knob | clean | `grep -rn "env::var" src/` returns nothing outside `main.rs`'s CLI args (none). The two `MADGAB_TRACE_*` blocks this commit *deletes* were the previous agent's only such reads; removing them is a net fence improvement. |
| No `eprintln!` in production `src/` | clean | none outside `src/main.rs`'s pre-existing CLI error/usage output, which `784deaa` does not touch. |
| No `ZZ_*` / `zz_*` / `MADGAB_*` / `EXP_*` scaffolding | clean | none in `src/` or `tests/`; `git status` was clean, the previous agent's `tests/zzprobe.rs` is not on the branch. |
| No change to `axes::*` weights or `boundary_novelty` | clean | neither name appears in the diff. |
| No change to `select_diverse` admission order, share cap, `STRUCTURE_FLOOR` | clean | `select_diverse` is touched only by the *deletion* of the `MADGAB_TRACE_PHRASES` block above its call site; the call itself is unchanged. |
| No change to `src/adjacency.rs` / `ADJACENCY_*` | clean | `src/adjacency.rs` not in the diff; no `ADJACENCY_*` line changed. |
| No change to `EMIT_PROFILE_MAX_DEEP`, `EMIT_PROFILE_RESERVE`, `LEXICAL_COMBINATIONS_PER_SEGMENTATION`, `sweep_index`, `sweep_rate`, `coverage_tuples`, `SEGMENTATION_KEEP` | clean | all seven are byte-identical between `03fdf52` and `784deaa`; the diff's only hunks are the two new free functions, the trace-block deletions, the two suffix-cost vectors, the `cap` initialiser and the two affordability guards. |
| No relaxation or re-baselining of an acceptance test | clean | `tests/corpus_integration.rs` is `+51/-0`: one new test appended, nothing edited. `approximate_output_is_locked` and every other corpus_integration test are literally the same bytes as the base. |

### The fix

On this branch, one commit, three hunks:

1. `src/lib.rs` — the `slot_is_affordable` doc comment: the measured
   discarded fraction is now stated as "4.2% of built wordings on one
   measured target, 0.6% on another" instead of naming the two targets, plus a
   sentence saying why.
2. `src/lib.rs` — the `affordable_opening_width` doc comment: the census is
   stated as "27 emit no wording at all on one measured four-word target,
   109 on a six-word one and 84 on another", with a pointer to the work item
   for the per-target numbers. The other two targets it named
   (`a whole lot of trouble`, `the cat sat on the mat`) are not canonical and
   could have stayed; removing them keeps the census uniform and costs
   nothing.
3. `tests/no_phrase_hard_coding.rs` — new test
   `no_canonical_example_in_a_production_doc_comment`. It scans every
   `src/*.rs` except `main.rs` (whose doc examples are the CLI's documented
   usage, the one place the file's own design permits the phrases), skips
   `#[cfg(test)]` regions using the existing `test_lines`, and fails on any
   comment line holding **three** consecutive words of a watched phrase. The
   threshold is three for the reason the module already documents: one or two
   words is ordinary prose and must not fire.

**Red/green of the new fence test, observed:** on `784deaa`'s `src/lib.rs`
it fails with exactly

```
  src/lib.rs:239  [phrase-in-production-comment]
      a production comment holds 4 words of the canonical target phrase (its just a stupid)
  src/lib.rs:268  [phrase-in-production-comment]
      a production comment holds 4 words of the canonical target phrase (its just a stupid)
```

and it passes after the fix, with the other six tests in the suite
unaffected. So the violation the previous review had to find by eye is now
found by the machine, at review time, in a suite the coordinator already runs.

## 2. The two mechanisms on their merits

### `slot_is_affordable(committed, later_minima, candidate, total_budget)`

*What it now bounds.* The **index space the traversal pushes**: a candidate
enters the heap only if a complete wording containing it is cost-admissible.

*Is the derivation sound against the constant it derives from?* Yes, and I
checked the arithmetic rather than the prose. `suf_min_cost` is built as
`suf_min_cost[k+1] + min(cost of slot k)`, so `suf_min_cost[k+1]` is exactly
`sum_{j>k} min cost of slot j`; `later_min_cost[k] = suf_min_cost[k+1] -
slot_min_cost[k]` is that sum, and the `slot_min_cost` vector exists only to
express it (a harmless indirection, not a bug). Any leaf the guard rejects
satisfies `committed + candidate + sum_{j>k} min_j > total_budget + 1e-9`,
while any real leaf costs `committed + candidate + sum_{j>k} actual_j` with
`actual_j >= min_j`, so `build`'s own test at `src/lib.rs:1528`
(`total_cost > total_budget + 1e-9`) would also have rejected it. **The guard
is implied by `build`'s test**, not an independent assumption, and it uses the
same epsilon. Sound.

*Function of input/search state, or a hand-picked constant?* State. It reads
`total_budget`, the prefix's committed sum and the per-slot minima — all
computed from this run's candidate lists. No literal target number anywhere.

*Could it help inputs other than the two canonical ones?* Yes, and it does:
it is a filter over *every* segmentation of *every* target, and the report's
own discarded-fraction measurement (4.2% and 0.6% of built wordings) is the
evidence that it *binds* rather than being a no-op. It is a pure filter, so
it can only remove candidates; the front is explicit that it is not a
milestone route, and that is correct.

*Constant retune in disguise / unbindable bound / unsupported claim?* No
retune: it adds no constant at all. The claim the code does **not** support is
mild and worth naming: the doc comment says it "binds when a caller tightens
`total_budget`", and that is true but untested here — at the shipped budget
`total_budget = 1.5` the filter's premise (over-budget candidates being
walked) is largely refuted by the front's own measurement that per-slot
budget-derived depth equals the list width in all 256 segmentations. The
comment states the shipped-budget figure honestly rather than claiming a win;
that is the right call, and I did not find a claim the code fails to support.

### `affordable_opening_width(slot_count, pop_limit)`

*What it now bounds.* The **opening width of the per-slot best-first walk**:
the largest `w` with `1 + w + ... + w^(d-1) <= pop_limit`, capped by
`LEXICAL_BRANCH_STAGE_0` and by the widest list present.

*Is the derivation sound against `LEXICAL_HEAP_POP_LIMIT`?* Yes, and the
quantity derived is the right one. A breadth-first walk of the product tree
cannot reach a leaf at depth `d` until it has popped every node at levels
`0..d-1`; with opening width `w` and a geometric ladder that is
`sum_{l<d} w^l` pops, so `sum > pop_limit` means the walk provably cannot
emit *anything* from that segmentation however it spends its pops. The
`pop_limit` is passed in and called as `LEXICAL_HEAP_POP_LIMIT` — the
constant is read, not copied, so the derivation is against the real bound.

*Is the arithmetic right?* I recomputed it. `d=4`: `1+10+100+1000 = 1111 <=
4000`, unchanged at 10. `d=5`: 11111 > 4000, 7381 > 4000, 4681 > 4000, 2801
<= 4000 → **7**. `d=6` → 5 (3906). `d=7` → 3 (1093). `d=8` → 3 (3280).
`d=9` → 2 (511). Monotone decreasing in `d`, as it must be. `fits(1) = d`,
always below a 4 000 limit, so the `while width > 1` loop always terminates
and cannot stall the traversal at zero width. `saturating_mul`/`saturating_add`
mean no overflow at pathological `d`. All correct.

*Function of state, or a hand-picked constant?* State: two live quantities
(`depth`, the pop limit). The `LEXICAL_BRANCH_STAGE_0` cap is a pre-existing
constant the function does not introduce.

*Could it help other inputs?* Yes, and structurally rather than by tuning.
This is the point that makes it a genuine fix and not a retune: at width 10
**no** segmentation with 5 or more slots can emit, ever, so narrowing it costs
no output anywhere. The change is therefore confined to segmentations that
emitted nothing on the base — which is exactly what the new regression test
observes from the outside, and the report states it plainly rather than
hiding it. `next_branch_stage` is untouched, so a traversal that drains at the
derived width with appetite left still opens the next stage (and by then it
has already emitted, so the later pops are not the wasted kind).

*Constant retune in disguise / unbindable bound?* It is **not** a retune, and
it **can** bind. Two honest limits, neither a defect:

* it can only ever *shrink* the width, so as a milestone lever it moves the
  index range the wrong way. `w-a7e2b3` §2b's arithmetic (a 5-slot cap of 7
  against a required slot-3 index of 99) still stands and I re-derived it
  above; this commit does not claim otherwise, and the front's report states
  the objective as NOT met for exactly this reason.
* the series assumes a *full* width `w` at every level and ignores the
  per-slot list widths, so where some slots are narrow the derived width can
  be lower than the walk strictly needs. That makes the bound conservative
  rather than wrong, and the `.min(widest)` at the call site covers the common
  case, but it is a real generality limitation: the derivation is a function
  of `depth` alone, not of `depth` and the widths together. Worth a sentence
  in the doc comment; I did not add one, since it is a comment-only concern
  and I did not want to expand the hunk beyond the fence fix.

The trade the report admits — fewer *distinct* deep resegmentations in the
pool (47 → 32 on one target), flat-to-slightly-down distinct-word count — is
the honest cost of spending the same pops on fewer, funded structures, and it
is stated in the report, the code comment and the test's own doc comment. I
did not independently re-measure 47 → 32; I did confirm the pool-size figures
it quotes for two targets (19 511 → 20 000 and 17 277 → 19 542; the first is
capped by the `top_n = 20_000` the helper uses, so the 20 000 figure is a
ceiling, not a count) and I re-measured the wall clock independently.

**Wall clock, interleaved against the base in one session, three runs each,
`--approximate --top 50`, this reviewer's own binaries:**

| target | base | tip |
| --- | --- | --- |
| `It's just a stupid game` | 1972 / 1952 / 2146 ms | 1322 / 1688 / 1377 ms |
| `a whole lot of trouble` | 1968 / 2472 / 2068 ms | 1330 / 1537 / 1398 ms |
| `the cat sat on the mat` | 2230 / 2320 / 2575 ms | 1416 / 1407 / 1446 ms |

Roughly a 30% reduction, no regression on any run. This corroborates the
report's criterion-7 column from an independent session.

## 3. Red/green for `approximate_pool_reaches_resegmentations_deeper_than_one_walk`

Red was run in a **separate throwaway worktree** at `/tmp/opencode/red-1c3e77`
(detached at `03fdf52`, removed afterwards; this worktree was never moved off
`8a5dd3b`). The test was transplanted verbatim from `784deaa` into the base
tree's `tests/corpus_integration.rs` and run with a target dir under
`/workspace`.

**RED on `03fdf52`** — observed, not quoted from the report:

```
thread 'approximate_pool_reaches_resegmentations_deeper_than_one_walk' panicked at tests/corpus_integration.rs:537:9:
she sells sea shells: "see if a law thus ish l.'s" missing, so a resegmentation deeper than one
uniform-width walk covers contributed no wording at all; pool had 16837 clues, e.g.
["see well see l.'s", "see air see l.'s", "see well see else", "see air see else",
 "see else if l.'s", "see well this l.'s", "see al see l.'s", "see air this l.'s"]
test result: FAILED. 0 passed; 1 failed
```

**GREEN on `784deaa`** — it passes as part of the 11/12 corpus_integration run
below, and passes in isolation.

The red run is a real red: the base's pool for that target is 16 837 clues and
the wording is simply not among them, which is the property the test claims
and not an incidental failure. The target and the three wordings are ordinary
English and name neither acceptance example, and `src/` does not know them
(confirmed by the fence suite). The test's own doc comment is candid that it
asserts *reach*, not quality, and that distinct deep resegmentations fall
slightly — I agree with that framing; it is the right claim for a change whose
mechanism is "stop spending pops on a walk that cannot emit".

## 4. The five suites, re-run on this branch after the fix

```
cargo test --release --lib                      53 passed, 0 failed
cargo test --release --test corpus_integration  11 passed, 1 failed
                                                 (approximate_finds_classic_madgab_resegmentation only)
cargo test --release --test exact_determinism    1 passed, 0 failed
cargo test --release --test approx_determinism   4 passed, 0 failed
cargo test --release --test no_phrase_hard_coding 7 passed, 0 failed  (6 on 784deaa, + the new one)
```

`cargo fmt`, `cargo clippy` and doctests do not exist on this host and are
not claimed. The wasm32 build was not attempted.

## 5. Milestone guards, re-measured here

* `approximate_finds_recognize_speech_resegmentation` — **still passes**.
* `madgab --approximate --top 50 "recognize speech"` — **still produces
  `wreck a nice beach`**, at visible rank 28 (`28. [0.918] wreck a nice
  beach`). Re-measured after the doc-comment fix as well.
* `approximate_finds_classic_madgab_resegmentation` — **red, confirmed**, and
  **red for the stated reason**. The failure is a plain `assert!` at
  `tests/corpus_integration.rs:136` with the message
  `canonical clue missing from top 50; got: ["it justice too bad aim", ...]`
  — a value assertion over a well-formed result list, **not** a panic inside
  the search, not an unwrap, not a malformed candidate. No test bug: the
  assertion is byte-identical to the base's.
* I also independently checked the report's stronger claim, that the wording
  is **never enumerated**, rather than merely unranked. With
  `top_n = 20_000` on the default release path:
  * base `03fdf52`: pool 19 511, contains = **false**, rank = none.
  * tip `784deaa`: pool 20 000, contains = **false**, rank = none.
  * `recognize speech` → `wreck a nice beach`: present on both, rank 27.

  So the objective is genuinely an enumeration/visibility question the front
  does not answer, exactly as its report says, and the milestone test cannot
  be turned green by anything inside this front's fence.

## 6. Verdict and the one hunk pair

**INTEGRATE WITH NAMED FIXES**, and the fixes are applied and committed here:

* the two production doc comments (`src/lib.rs:239`, `src/lib.rs:268` as
  landed) must not name `It's just a stupid game` or `recognize speech`; and
* `tests/no_phrase_hard_coding.rs` gains
  `no_canonical_example_in_a_production_doc_comment` so the next reviewer does
  not have to find it by eye again. Shown red on `784deaa` at exactly those
  two lines, green after.

Land it on the justification the report itself gives — a derived traversal
opening width that stops 27–109 of 256 retained segmentations from burning
4 000 pops each to emit nothing, plus a cost-admissibility filter that skips
pushed subtrees `build` would have dropped, together worth ~30% wall clock and
a larger pool, with both milestone guards intact. **Do not land it as
milestone progress.** `w-a7e2b3`'s refutation of it as a route (a 5-slot cap
of 7 against a required slot-3 index of 99) is correct and is not addressed
by this commit, and the report agrees. Whether the front is closed
`superseded` by w-6f2b18, or kept open against the unblocking conditions the
coordinator already recorded, is the coordinator's call and not mine.

One thing the integrator should know: the diff's line numbers collide with
front A's (`w-9c4d21`) exactly as `w-a7e2b3` §3a predicted — this commit's
+92-line block at `src/lib.rs:242` shifts every line A touches. Nothing here
touches A's hunks, so the conflict is mechanical, but sequence them.

Tree left clean; the throwaway worktree and both throwaway target directories
were removed. Nothing pushed, nothing merged, no other worktree read-write,
`/workspace/madgab-representation-b2e5c4` untouched.
