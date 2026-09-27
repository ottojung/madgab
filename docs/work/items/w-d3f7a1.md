---
work_item: true
id: w-d3f7a1
state: done
priority: high
owner: d3f7a1 (adversarial review front, 2026-09-27T03:37Z)
updated: 2026-09-27T03:37:00Z
branch: madgab-review-axes-d3f7a1
worktree: /workspace/madgab-review-axes-d3f7a1
reviews: 5258e7b..f71b634 (front: refs/heads/wip/madgab-objective-axes-f71b634)
---

# Adversarial review front: the objective-axis front (w-9c6f2b)

## Premise correction, read this first

**This item file did not exist when the review was asked for.** There is no
`docs/work/items/w-d3f7a1.md` in `5258e7b` or in `f71b634`, and no work item
anywhere on either side of the diff that describes this review front, its
fences, or its completion criteria. The dispatch said "read it in full and
execute it exactly"; that was not possible. The review below was therefore
performed against the *stated* goal and constraints of the dispatch, not
against a written brief: review `5258e7b..f71b634`, check every fence the
subject item [w-9c6f2b](w-9c6f2b.md) declares, and report the verdict, the
evidence table and the next action. If a real brief existed and is missing
from the tree, **this review does not claim to satisfy it** — a coordinator
should check whether w-d3f7a1 was ever opened and, if so, re-dispatch it
against the real text.

Everything the dispatch named was honoured: the diff range is exactly
`5258e7b..f71b634`, the only work item written on this branch is this file
plus one new item, [w-4e2b19](w-4e2b19.md), and `w-7c1f64`, its worktree
`/workspace/madgab-emit-coverage` and agent `7c1f64` were not touched.

## VERDICT: the claim is supported. The shipped change is general, and the
## one hard fence that could have broken (canonical case 1) does not break.
## No mechanism defect. One documentation defect filed as w-4e2b19.

The subject item makes a strong, falsifiable claim and stakes the whole
front on it: *the objective is not what keeps canonical case 2 out of the
printed set; the clue is absent from the emission pool, so no axis can
help.* If that is true, the front correctly retires itself as a route to
the milestone instead of chasing its own `+0.0221` margin, which is the
expensive mistake to avoid. I re-measured the numbers that decide it, and
they hold. I did not independently reproduce the pool-membership
measurement itself; see the one gap in the evidence table.

## Evidence

`src/lib.rs` 2544-2573 is the entire behavioural change: `sub_cost_total /
4.0` becomes `sub_cost_total / words.max(1) / SIMILARITY_COST_PER_WORD`,
with `words = self.word_count()`. Everything else in `src/lib.rs` is a
comment, one new named constant (`SIMILARITY_COST_PER_WORD = 1.0`,
src/lib.rs:2742) and two tests. `tests/corpus_integration.rs` is a
re-lock. No other file changed.

### Reproduced on this host, release build, from my worktree

| claim in w-9c6f2b | stated | measured here | verdict |
|---|---|---|---|
| `cargo test --release --lib` | 58/58 | **58 passed, 0 failed** | confirmed |
| `--test corpus_integration` | 12 pass / 1 fail | **12 passed, 1 failed** (`approximate_finds_classic_madgab_resegmentation`) | confirmed |
| `--test exact_determinism` | 1/1 | **1 passed** | confirmed |
| `--test approx_determinism` | 4/4 | **4 passed** | confirmed |
| `--test no_phrase_hard_coding` | 7/7 | **7 passed** | confirmed |
| canonical case 1, printed rank 28 / 0.918, unchanged on both trees | rank 28, 0.918 | **rank 28, 0.918 on `f71b634`; the top 8 of case 1 are byte-identical between `5258e7b` and `f71b634`** | confirmed, and the strongest single piece of evidence here |
| case-2 cutoff rises (0.8978 -> 0.9171), the relative-cutoff caveat | 0.9171 | **printed rank 1 = 0.924, printed rank 50 = 0.921; the whole printed prefix sits above 0.92 on `f71b634` and above 0.905 on `5258e7b`** | confirmed in direction; the cutoff rose with the scores, exactly as the item warned |
| a four-word clue is bit-identical under the change | asserted as the reason case 1 is unmoved | **holds arithmetically: `c/4/1.0 == c/4`; and the 8-item case-1 prefix is identical across the two binaries** | confirmed |
| `hits justice dupe hid came` is ABSENT from the printed 50 | absent | **absent from the printed 50 on both binaries; `dupe` appears in no printed line on either** | confirmed for the printed set |
| ... and absent from the *pool* (0 of 18,917), not merely ranked out | the load-bearing claim | **NOT INDEPENDENTLY REPRODUCED** — see below | unverified |

The pool-membership claim is the one that decides the front's retirement,
and it is asserted on the strength of a harness that lives on
`scratch/9c6f2b-harness` and is deliberately not in this branch. I can
confirm the consequence (the clue is not in the printed 50) but not the
cause (it was never a candidate). That distinction is the whole claim, so
it is stated here as a gap rather than folded into the verdict. It is
consistent with everything else I measured — in particular the printed
prefix on `f71b634` is *more* concentrated than on `5258e7b` (see below),
which is what you would expect if emission coverage, not ranking, is the
active constraint — but consistency is not proof, and
[the same measurement is owned by w-7c1f64](w-7c1f64.md) and must be
confirmed there before the front's retirement is treated as settled.

### Fence check

| fence | result |
|---|---|
| no phrase-specific hard-coding | **pass.** `git grep -i -E "wreck\|beach\|recognize\|justice\|stupid\|dupe\|came\|hid" -- src/` returns hits only in `src/lexical.rs` test word lists and in `#[cfg(test)]` code in `src/lib.rs`; none are new in this diff. `no_phrase_hard_coding` is 7/7, including the doc-comment case comment-stripping would miss |
| no `zz*` file, `eprintln!`, env knob, phase timing | **pass.** the only match for those patterns in added lines is prose inside the item's own fence list |
| no test relaxed, re-baselined or `#[ignore]`d | **pass.** the only test-body change is the `approximate_output_is_locked` fixture, and it is a re-lock of a 3-word target whose similarity term the axis change necessarily moves. Both other pre-existing corpus tests are untouched |
| no `axes::*` weight moved | **pass.** the diff adds `SIMILARITY_COST_PER_WORD` and touches no existing weight or `SIMILARITY_PER_WORD` |
| no cost / pronunciation / dictionary / `GAP_COST` / retention front re-opened | **pass.** zero hunks outside `metrics` and the test module |
| canonical case 1 must not regress | **pass**, and by the margin, not by luck: rank and score identical, prefix identical |
| `cargo fmt` / `cargo clippy` | **not present on this host and not run**, not claimed anywhere by this front |

### The two judgement calls I checked hardest, because they are where this
### kind of change usually breaks

**1. Is the new axis genuinely per-word, or does it desynchronise from
`word_count()`?** `sub_cost_total` has exactly one accumulator in the whole
file, src/lib.rs:2519, inside the same `extend_parts` body that pushes the
word, and `word_count()` is `self.words.len` (src/lib.rs:2414). There is
therefore no state in which the cost and the count disagree, and
`words.max(1)` covers the `words == None` empty partial. The only place
they can be set apart is the `#[cfg(test)]` setter at src/lib.rs:4108,
which is test-only. **Clean.**

**2. Does the test actually pin what it claims, or is it rigged to pass?**
`similarity_is_scored_per_word_not_per_candidate` builds n-word candidates
over a fixed 64-phone target and asserts `combined` is flat in `n`. That
is only a valid test of the similarity axis if the other axes are
length-neutral across those chains, and they are, and not by accident: the
target boundary list is `&[]` so `boundary_novelty` is 1.0 for every `n`
(`shared == 0` in every case); all six words are 2 syllables and the test
passes `target_syllables = n`, so `RHYTHM` is 1.0 for every `n`; the words
cycle through six fixed rarities, classes and shapes. I verified the
length-neutrality claim rather than taking it, and the test is honest.
**Clean, and unusually well constructed.**

### Observations that are not defects

- **The printed case-2 prefix is slightly *less* diverse after the
  change.** Counting the first three words of each of the printed 50:
  `5258e7b` puts 19 of 50 in one `it justice too ...` family and spreads
  the rest over 5 more; `f71b634` puts 22 of 50 in one `it said thus ...
  too ...` family. Distinct three-word prefixes go 11 -> 10. This is a
  continuation of a degeneracy that already existed — the base tree is
  already 19/50 in a single family — not a regression introduced here, and
  the item already records the share cap as inert. Recorded so the
  selector-policy front sees the direction, not because it is a blocker.
- **The axis is now per-word while the feasibility budget is still
  absolute.** src/lib.rs:818 and src/lib.rs:5392 gate on a raw
  `sub_cost_total` against a total budget, and the one-word span proxies
  still order on `SIMILARITY_PER_WORD`. So the axis no longer prices the
  absolute gate that still applies. The item measured this exact trade,
  recorded the pair (case-2 proxy rank 6328 with the proxies left alone,
  1917 with them re-derived, at the cost of losing
  `approximate_pool_reaches_matches_deep_in_a_span`) and declined to take
  it because it is enumeration-side. That is the right call and it is
  disclosed, not hidden. It is the sharpest question the successor front
  inherits.

## Defect filed

One real defect, non-mechanical, filed as [w-4e2b19](w-4e2b19.md): the
re-lock comment in `tests/corpus_integration.rs` states that "the scores of
the retained ones are unchanged", and they are not — the five retained
phrases fall by 0.0023 to 0.0087, and `isle of new` alone moves
0.930994 -> 0.922661. The same comment's own arithmetic paragraph says
every score falls, so the two halves of the comment contradict each other.
It is in the file a future reviewer reads to decide whether a further score
move is acceptable, and it understates the churn it is describing. Cheap
to fix, worth fixing before anyone treats this lock as evidence of
stability.

## Next action

**For a coordinator:**

1. Land `madgab-objective-axes` (`refs/heads/wip/madgab-objective-axes-f71b634`)
   and this review branch `madgab-review-axes-d3f7a1`. `main` and
   `post-milestone-acceptance` were not pushed from this worktree and must
   not be.
2. **Do not open another objective-axes front.** The evidence agrees with
   the subject item: the objective is not the binding constraint on
   canonical case 2, and the enumeration-side, cost-side, distance-side
   and retention-side levers are already refuted by measurement in
   w-c4d7e8, w-1c3e77, w-6f2b18, w-7b41d2 and w-b2e5c4.
3. **The one thing still owed by this front's own logic is the pool
   measurement.** w-9c6f2b retires itself on a number produced by a
   scratch harness that is not in the tree. I could not reproduce it and
   did not. [w-7c1f64](w-7c1f64.md) already owns emission coverage and is
   in flight; the confirmation it needs is narrow — assert
   `hits justice dupe hid came` is absent from the pool, not merely from
   the printed 50 — and it should be attached to that front rather than
   reopening this one. Until it exists in a test, "the objective is not
   the blocker" rests on a measurement no one can re-run from a branch.
4. Fix w-4e2b19 whenever `tests/corpus_integration.rs` is next touched.

## Notes

- `cargo fmt` and `cargo clippy` do not exist on this host and were not
  run. `git`, `grep`, `sed`, `awk` and `date` need
  `export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"` first; a missing
  `git` here is a truncated PATH, not a host outage.
- For the side-by-side measurements I built `5258e7b` in a scratch
  worktree with `CARGO_TARGET_DIR` outside `/tmp` (which is `noexec` on
  this host, so cargo build scripts fail there with `Permission denied`
  and the error is misleading). Neither scratch worktree is on any branch
  and neither is part of this review's commits.
- `It is just a stupid game` is not a substitute for
  `It's just a stupid game`. All case-2 numbers here are the latter.
