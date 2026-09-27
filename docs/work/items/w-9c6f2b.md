---
work_item: true
id: w-9c6f2b
state: done
priority: high
owner: 9c6f2b (opened by coord-9a4e 2026-09-27T02:18Z; closed by agent 9c6f2b)
updated: 2026-09-27T03:05:00Z
branch: madgab-objective-axes
worktree: /workspace/madgab-objective-axes
---

# Objective-axis front: the two general scoring defects that jointly carry case 2

## Goal

Remove the two *general* defects in the candidate objective that
[w-c4d7e8](w-c4d7e8.md) priced, and decide by measurement whether they
jointly bring the canonical case-2 answer `Hits Justice Dupe Hid Came` into
the approximate proposal set for `It's just a stupid game` — without any
phrase-specific special case, and without regressing canonical case 1
(`wreck a nice beach` for `recognize speech`).

This is the last front with enough measured headroom. Every
enumeration-side, retention-side, budget-side, cost-side, pronunciation-side
and distance-side lever is refuted with arithmetic in
[w-c4d7e8](w-c4d7e8.md), [w-1c3e77](w-1c3e77.md), [w-6f2b18](w-6f2b18.md),
[w-7b41d2](w-7b41d2.md) and [w-b2e5c4](w-b2e5c4.md). Do not open a fifth
derivation of those refutations.

## Context

Measured on `648b4e0` by agent `c4d7e8` and recorded in
[w-c4d7e8](w-c4d7e8.md):

- The canonical answer **is in the candidate pool**: of 1,545,903
  enumerated five-word alignments, 40,546 contain `dupe`; the best is
  `it justice dupe add aim` at 0.8834, rank 130. It *ranks out*, it is not
  absent. This reverses the standing "not in the pool" conclusion.
- Its best admissible alignment costs 1.1195 of the 1.5 budget, cuts
  `[3,10,13,15,19]`, and decomposes as:

  ```text
  SIMILARITY   0.25 * (1 - 1.1195/4) = 0.1800
  NOVELTY      0.15 * 0.6667          = 0.1000
  WORD_NOVELTY 0.15 * 1.0             = 0.1500
  FAMILIARITY  0.10 * 0.3987          = 0.0399
  RHYTHM       0.30 * 1.0             = 0.3000
  SHAPE        0.05 * 1.0             = 0.0500
  CLOSED_CLASS 0.0                     = -0.0000
  PUNCH        0.10 * (0.8 - 1)       = -0.0200
                                   total = 0.7999
  ```

  against a rank-50 cutoff of **0.8978**, i.e. a gap of **+0.0979**.
- A zero-cost, perfect pronunciation still scores 0.8699, so no cost-model,
  dictionary, `GAP_COST` or substitution-weighting change can close the gap.
  That upper bound is independent of how the pool is enumerated.

### The two defects, and why neither works alone but both may work together

1. **Constant similarity normaliser.** `SIMILARITY` is
   `(1.0 - sub_cost_total / 4.0).clamp(0,1)` at `src/lib.rs:2377`. The
   divisor is the constant `4`, not the clue's word count or the target's
   phone count, so a *longer* clue pays more absolute similarity cost for
   the same per-word quality. This is a real, general, order-dependent
   per-word bias. Its maximum possible value here is **+0.0700** (the
   1.1195 -> 0.0 row), which alone leaves the answer 0.0279 short.
2. **Boundary novelty punishes the property a Mad Gab resegmentation is
   supposed to have.** `boundary_novelty` (`src/lib.rs:2564`) scores pure
   Jaccard distance, i.e. `Jaccard(preserved) - 1`. The canonical answer
   keeps target boundary offset 15 out of four, scoring 0.6667 and losing
   **-0.0500**. Rewarding boundary *addition* without punishing boundary
   *preservation* — so novelty cannot fall below `1 - added/total` — is
   worth at most **+0.0500** here, and alone lands 0.8499, still short.

**Joint headroom is +0.1200 against a +0.0979 gap**, a margin of +0.0221.
That is the entire reason this front exists, and it is arithmetic, not
optimism: the two levers are the only two the refutations leave open.

### The caveat that decides how this front must be worked

**The rank-50 cutoff is relative, not absolute.** Lifting similarity for
every long clue lifts the whole score distribution, so the cutoff rises
too, and the +0.0221 absolute margin may vanish. Therefore:

- The success criterion is the **rank of the requested clue in the printed
  proposal set**, never its absolute score. An absolute score above 0.8978
  measured against a stale cutoff is not evidence.
- Every cutoff figure quoted in this item is a *baseline on `648b4e0`* and
  must be re-measured on the tree being handed over.
- If both fixes land and the answer still ranks out, that is a **real and
  valuable result**: it moves the blocker from "the objective is
  miscalibrated" to "the relative ranking among equally well-scored long
  clues is decided elsewhere", which is a much sharper next question. It
  must be recorded as such, not re-attempted with a third weight move.

## Fences

- No phrase-specific hard-coding. No special case for these two sentences,
  their clues, their words, or any substring of them. `git grep -i -E
  "wreck|beach|recognize|justice|stupid|dupe|came|hid" -- src/` must return
  hits only in unit tests, explanatory comments and the pre-existing
  `src/main.rs` doc examples. `tests/no_phrase_hard_coding.rs` strips
  comments, so it does **not** catch example-naming doc comments; check by
  eye and with `git status` as well.
- No `zz*` probe file, no `eprintln!` probe, no env-var knob and no phase
  timing may reach the accumulation branch. Scratch probes belong on a
  `scratch/*` branch.
- Do not relax, re-baseline or `#[ignore]` any test to make something pass.
  `approximate_output_is_locked` in `tests/approx_determinism.rs` will
  legitimately change; re-measure it on the merged tree and record the new
  lock rather than copying the old one forward.
- Do not re-open a cost, pronunciation, dictionary, `GAP_COST`,
  `LEXICAL_BRANCH_KEEP`, `SEGMENTATION_KEEP` or retention front. They are
  closed by measurement.
- Canonical case 1 must not regress. `wreck a nice beach` must remain in
  the printed proposal set for `recognize speech`.

## Completion criteria

- [x] Both defects are either fixed by a general change or refuted with a
      measurement. Defect 1 is fixed generally (per-word normaliser) and
      defect 2 is refuted with a two-line proof, with the per-design
      arithmetic for all four candidate designs above.
- [x] A unit test states the corrected behaviour of each axis in general
      terms, with no example phrase in the assertion:
      `similarity_is_scored_per_word_not_per_candidate` (a longer clue of
      equally good words is not penalised; the axis still ranks on per-word
      quality) and `boundary_novelty_is_never_below_its_one_sided_readings`
      (the general form of "preserving a target boundary is never a
      docking reason", plus the two endpoints).
- [x] On the handed-over tree, release binary, `--approximate --top 50`,
      cutoff re-measured on that same tree: canonical case 1 present at
      printed rank 28 / 0.918; canonical case 2 **FAIL** — recorded as
      ABSENT from the pool (0 of 18,917), not as a rank.
- [x] `cargo test --release --lib` (58/58), `--test corpus_integration`
      (12 passed / 1 failed), `--test exact_determinism` (1/1),
      `--test approx_determinism` (4/4) and `--test no_phrase_hard_coding`
      (7/7) all run, exact results above.
      `approximate_finds_classic_madgab_resegmentation` remains red because
      the rank criterion is not met. The other four determinism tests, and
      in particular `raising_top_n_does_not_retract_shown_proposals` and
      `approximate_pool_is_reproducible_across_processes`, are green: the
      per-word axis changes scores but keeps the search reproducible and
      keeps the visible prefix monotone in `top_n`.
- [x] Branch `madgab-objective-axes` is **pushed** — as
      `refs/heads/wip/madgab-objective-axes-09a2956`, because the rebase
      recreated the commits and the branch cannot fast-forward over its own
      pre-rebase remote tip. The tree is clean and HEAD is on the named
      branch, not detached.
- [ ] This item is updated with objective state, validation, blockers and
      the next action — done above, but it is **committed on
      `madgab-objective-axes` and not pushed to
      `post-milestone-acceptance`**, because this agent was explicitly
      forbidden from pushing that branch. A coordinator must land it.
      `main` was never touched (`origin/main` is `c0ecd7c` before and
      after).

## RESULT: one defect fixed by a general change, one refuted with a proof, and the front retired as a route to the milestone

Agent `9c6f2b`, on `madgab-objective-axes` rebased onto
`post-milestone-acceptance` at `39f4e41`. The decisive measurement is the
one nobody had taken on the production path, and it retires this front:

> **The canonical case-2 answer is ABSENT from the production approximate
> pool. 0 of 18,824 candidates on the baseline tree and 0 of 18,917 with
> the fix. It is not ranked out; the search never proposes it.**

No value of any objective axis can put a clue in the printed proposal set
if the search does not emit it, so the `+0.0979` gap and the `+0.0221`
joint margin priced in the Context section were never the binding
constraint. Both defects were investigated anyway, and one of them is real.

### The three numbers, measured on the same tree, same run

Release binary, `--approximate --top 50`, `It's just a stupid game`:

| quantity | baseline `39f4e41` | with the per-word similarity axis |
|---|---|---|
| deduplicated pool | 18,824 | 18,917 |
| **`hits justice dupe hid came` raw rank** | **ABSENT** | **ABSENT** |
| rank-50 score cutoff | 0.897757527 | 0.917129199 |
| distinct structures in the printed 50 | 12 | 12 |
| max per-structure share (cap 16) | 17 | 17 |
| best clue containing `dupe` | rank 2575, 0.871144 | rank 2017, 0.894602 |
| best clue in an *unrepresented* structure | rank 188, 0.894068 | rank 122, 0.914598 |
| min score in the printed 50 | 0.895691888 | 0.914905228 |

Canonical case 1, `recognize speech`, same build: `wreck a nice beach` is
in the printed set at **rank 28, score 0.918** on both trees; the case-1
cutoff moves 0.917026 -> 0.911 and the phrase is untouched.

**Which of the three numbers excludes the clue, and by what: neither.**
It is excluded *before* both, at enumeration. It is not in the pool, so it
has no raw rank and no score to compare against the 0.9171 cutoff; and the
structure share cap is not binding either, because `max_share = 17` against
a cap of 16 means `select_diverse`'s step 3 ("never return a short list")
had already suspended the cap and filled the list in score order. The
printed 50 is therefore effectively the score-ranked prefix, and its 12
distinct structures are a *consequence* of that ranking rather than an
independent diversity filter. The best candidate whose structure is absent
from the printed list sits at raw rank 122 (0.9146), below the printed
minimum (0.9149): exclusion near the top is by score rank, full stop.

### Defect 1, the constant `4` divisor — FIXED, and it is a real bias

`SIMILARITY` was `(1.0 - sub_cost_total / 4.0).clamp(0,1)`: a *candidate's
total* cost divided by a constant, for an axis that the rest of the file
documents as a per-word measure. Two clues whose words were individually
equally good were ordered by how many words each happened to have, and a
long clue paid once for a property that belongs to each of its words.

It is now the **mean cost of a word on a one-unit-per-word scale**:

```rust
let cost_per_word = self.sub_cost_total / words.max(1) as f64;
let similarity =
    (1.0 - cost_per_word / axes::SIMILARITY_COST_PER_WORD).clamp(0.0, 1.0);
```

`SIMILARITY_COST_PER_WORD` is a new named constant in `axes`. One unit of
mean cost per word is the same scale the constant `4.0` expressed at the
four-word clue it was written for, so **a four-word clue scores bit-
identically and only the length term moves** — which is why canonical case
1, a four-word clue, is unmoved at rank 28.

Per-design score arithmetic, `SIMILARITY` weight 0.25:

| design | divisor | axis value at total cost `c`, `n` words | case-1 fence |
|---|---|---|---|
| **D1 (shipped)** mean cost, unit scale | `n` | `0.25 * (1 - c/n)` | **pass**, rank 28 |
| D2 documented per-word normaliser | `4n` | `0.25 * (1 - c/4n)` | **FAIL** |
| D3 per-word *budget* scale | `0.5n` | `0.25 * (1 - c/0.5n)` | not built, deflates every candidate |
| D0 status quo | `4` | `0.25 * (1 - c/4)` | pass, the bias |

D2 is the reading in which the constant `4.0` is the *per-word* normaliser
that `SIMILARITY_PER_WORD` already assumes, and it was built and measured
first. It lifts every candidate by `0.25 * 3c / 16`, i.e. it rewards
*total* cost, and it dropped canonical case 1 from printed rank 28 to raw
rank 810 of 19,833 — out of the printed 50. Refuted by the case-1 fence.
D3 was not built: the search's `per_word_budget` of 0.5 is a feasibility
budget, not a quality scale, and every candidate's similarity term would
fall (0.1800 -> 0.1380 on the canonical alignment), which is the opposite
of the fix.

**The one-word span proxies were deliberately left alone.** The exact
marginal weight of the new axis is `-SIMILARITY / (words + 1)`, so
`SIMILARITY_PER_WORD` is now a slightly wrong local key rather than an
exactly right one. Re-deriving all three proxy sites on the correct
marginal was built and measured: it improves the case-2 proxy materially
(best `dupe` clue raw rank 6328 -> 1917) but it **loses
`approximate_pool_reaches_matches_deep_in_a_span`**, the reach guard for
w-9d4e17. That is an enumeration-side effect and this front does not own
it, so the shipped change touches the axis and nothing else. The measured
pair (rank 6328 with the proxies alone, 1917 with them re-derived, reach
guard green and red respectively) is left here so the emission-coverage
front does not have to rebuild it.

### Defect 2, `boundary_novelty` — REFUTED, with a proof, not a measurement

The item asks for boundary novelty that "cannot fall below
`1 - added/total`". Both candidate re-definitions are dead, for different
reasons, and neither is a matter of taste:

1. **The item's literal floor is semantically inverted.** It awards
   `1 - added/total`, so a clue that reproduces the target's segmentation
   *exactly* — `added = 0` — scores the maximum 1.0. The axis would give
   its top score to the non-resegmentation.
2. **The preservation-agnostic readings are provably dominated.** With `a`
   added, `r` removed and `s` shared boundaries, the Jaccard distance the
   code already computes is `(a+r)/(a+r+s)`, and the two one-sided
   readings are `a/(a+s)` and `r/(r+s)`. Then

   ```text
   (a+r)/(a+r+s) - a/(a+s) = r*s / ((a+r+s)(a+s))  >= 0
   (a+r)/(a+r+s) - r/(r+s) = a*s / ((a+r+s)(r+s))  >= 0
   ```

   so the Jaccard distance is **never below either one-sided reading**, for
   every possible pair of boundary sets. A `max` over the three readings is
   therefore a mathematical no-op, not a small improvement. It was built
   and measured over the whole case-2 pool: **0 of 21,606 candidates
   changed novelty value.**

So the axis does not "punish the property a resegmentation is supposed to
have"; the residual 0.05 the canonical alignment loses is the *legitimate*
price of its two coincidences at target offsets 3 and 15, and any change
that recovered it would have to hand 1.0 to clues that are not
resegmentations at all. The change was reverted; the proof is pinned by
`boundary_novelty_is_never_below_its_one_sided_readings`, which also
asserts the two endpoints (identical segmentation scores 0, a full re-cut
scores 1).

### What the item's `+0.0221` margin actually was

Worth recording so no later pass recomputes it. On the item's own numbers
the canonical alignment scores 0.7999 and a **zero-cost perfect
pronunciation** of it scores 0.8699. The shipped change lifts the
canonical's similarity term from 0.1800 to 0.1940 and the case-2 cutoff
from 0.8978 to 0.9171 — the cutoff rises by more than the answer does,
which is exactly the relative-cutoff caveat the item warned about, and it
is why the rank of the best `dupe` clue is the only number worth reading
(2575 -> 2017, an improvement, and still 40x outside the top 50). The
`+0.0500` the item priced for defect 2 was never available: see the proof.

### Successor: emission coverage, not scoring

The alignment is **lattice-reachable** — w-c4d7e8's exhaustive scratch
enumeration found 40,546 `dupe`-bearing five-word candidates over the real
lattice — but **not emission-reachable**: the production approximate search
does not put it in the pool. So the open question is how much of the
lattice the emission budgets actually cover, and it is already open as
[w-7c1f64](w-7c1f64.md) (agent `7c1f64`, `/workspace/madgab-emit-coverage`).
It is not a vocabulary question (`dupe` is present at cost 0.150 and
survives the rarity filter), not a cost question, and not an axes question.
This front does not touch it.

A second, smaller observation for that front or a selector front, not
acted on here: at `top_n 50` the per-structure share cap is **already
inert** (17 members of one structure against a cap of 16), so
`select_diverse`'s diversity mechanism contributes nothing at this `top_n`
and the visible list is the score-ranked prefix. The best unrepresented-
structure candidate is 0.0003 below the printed minimum, so a cap that
were enforced rather than suspended would visibly widen the list. That is a
selector-policy question and is not owned here.

## Validation, on the tree being handed over

`madgab-objective-axes` = `post-milestone-acceptance` (`39f4e41`) + two
commits: the per-word similarity axis with its two general tests, and the
`approximate_output_is_locked` re-lock. Release build throughout.

```text
cargo test --release --lib
  test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured

cargo test --release --test corpus_integration
  test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured
  approximate_finds_recognize_speech_resegmentation ... ok      <- case 1
  approximate_finds_classic_madgab_resegmentation ... FAILED   <- case 2, expected red
  approximate_output_is_locked ... ok                           <- re-locked, see below
  approximate_pool_reaches_matches_deep_in_a_span ... ok
  approximate_pool_reaches_alternatives_past_the_opening_slot_width ... ok
  approximate_pool_reaches_resegmentations_deeper_than_one_walk ... ok
  approximate_list_is_not_one_resegmentation ... ok
  approximate_list_represents_enumerated_resegmentations ... ok
  approximate_proposals_are_predominantly_content_words ... ok

cargo test --release --test exact_determinism
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured

cargo test --release --test approx_determinism
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured
  approximate_pool_helper ... ok
  approximate_mode_is_reproducible_across_processes ... ok
  approximate_pool_is_reproducible_across_processes ... ok
  raising_top_n_does_not_retract_shown_proposals ... ok

cargo test --release --test no_phrase_hard_coding
  test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured
```

`approximate_finds_classic_madgab_resegmentation` remains red **because the
rank criterion is not met**, and the reason is now known to be upstream of
the ranking: the clue is not in the pool. No test was relaxed,
re-baselined or `#[ignore]`d.

### `approximate_output_is_locked`: the re-lock, with its before/after

This lock legitimately moves: the axis it locks is the axis this front
changed. The locked target is three words, so its similarity term becomes
`0.25 * (1 - cost/3)` from `0.25 * (1 - cost/4)` and every score falls by
`0.25 * cost / 12`, i.e. 0.0023-0.0024 across the list.

| rank | before | after |
|---|---|---|
| 1 | 0.938335 isle a view | 0.936034 isle a view |
| 2 | 0.937604 aisle a view | 0.935302 aisle a view |
| 3 | 0.937462 i.'s a view | 0.933655 isle uhh view |
| 4 | 0.936762 eye a view | 0.933646 i'll uhh view |
| 5 | 0.931877 how ill view | 0.922661 isle of new |
| 6 | 0.931877 now ill view | 0.922223 a ill view |
| 7 | 0.930994 isle of new | 0.921929 aisle of new |
| 8 | 0.930262 aisle of new | 0.921298 eye ill view |
| 9 | 0.930120 i.'s of new | 0.920450 isle of too |
| 10 | 0.929948 yeah ill view | 0.920124 yeah ill view |

**Five of the ten phrases are different**; the five that remain are
unchanged in wording and in order relative to each other. The churn is a
consequence of the three-word clue length reordering candidates that were
within 0.003 of one another, and it is neither a regression nor an
improvement claim. A four-word clue's similarity term is bit-identical
under this change, which is why canonical case 1 does not move at all.

### Fences

- `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid" --
  src/` returns hits only in unit tests and explanatory comments; the
  canonical phrases appear in `src/lib.rs` only inside `#[cfg(test)]` code.
  `tests/no_phrase_hard_coding.rs` is 7/7, and it also covers the
  example-naming doc comment case that comment-stripping would miss.
- No `zz*` file, `eprintln!` probe, env-var knob or phase timing reached
  this branch. The measurement harness (`ZZ_PHRASES`, `ZZ_STRUCT`,
  `ZZ_SUBSTR`, `ZZ_OFFSTRUCT`) lives on `scratch/9c6f2b-harness` and its
  baseline twin `scratch/9c6f2b-baseline`, both built on
  `post-milestone-acceptance`; neither is part of this branch.
- No `axes::*` weight was moved. `CLOSED_CLASS_WEIGHT`,
  `SIMILARITY`, `NOVELTY`, `WORD_NOVELTY`, `FAMILIARITY`, `RHYTHM`, `SHAPE`
  and `PUNCH` are all untouched; the change is one normaliser inside the
  `SIMILARITY` axis and one new named constant.
- No cost, pronunciation, dictionary, `GAP_COST`, `LEXICAL_BRANCH_KEEP`,
  `SEGMENTATION_KEEP` or retention front was re-opened. The one
  enumeration-adjacent effect measured (re-deriving the span proxies) was
  **not** shipped, precisely because it belongs to another front.
- Canonical case 1 did not regress: printed rank 28, score 0.918, unchanged.
- `cargo fmt` and `cargo clippy` do not exist on this host and were not
  run.

## Handoff / notes

- Opened by coordinator `coord-9a4e` on 2026-09-27T02:18Z, from
  [w-c4d7e8](w-c4d7e8.md)'s published upper bound. Agent `9c6f2b`, worktree
  `/workspace/madgab-objective-axes`, branch `madgab-objective-axes` off
  `post-milestone-acceptance`.
- **Next action for a coordinator: land `madgab-objective-axes`, and do not
  open another objective-axes front.** The successor is
  [w-7c1f64](w-7c1f64.md) (emission coverage), owned by agent `7c1f64`.
  This item is closed with the front retired as a route to the milestone:
  the objective is not what keeps canonical case 2 out of the printed set.
- `madgab-objective-axes` was rebased twice while the accumulation head
  moved, so its commits cannot fast-forward over the earlier remote tip.
  The current work is durable at
  `refs/heads/wip/madgab-objective-axes-09a2956`; the branch ref
  `madgab-objective-axes` on the remote is the pre-rebase tip `9001822`
  and should be replaced by a force-push or a fresh merge at the
  coordinator's discretion.
- `cargo fmt` and `cargo clippy` are absent on this host. Do not report
  them as run; see `docs/environment-notes.md`.
- `It is just a stupid game` is **not** a substitute for
  `It's just a stupid game`: 20 phones, boundaries `[2,4,9,10,16,20]`,
  7 syllables. It is a different target.
- The `+0.0979` gap and the `0.8978` cutoff in the Context section are a
  baseline on `648b4e0` and are stale. The current measurements are the
  table at the top of this update.
