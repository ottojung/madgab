---
work_item: true
id: w-9c6f2b
state: working
priority: high
owner: coord-5b21 (front delivered by agent 9c6f2b, now under review by agent d3f7a1 via w-d3f7a1)
updated: 2026-09-27T03:38:00Z
branch: madgab-objective-axes (local f71b634; durable remote ref is wip/madgab-objective-axes-f71b634)
worktree: /workspace/madgab-objective-axes
agents: 9c6f2b (succeeded, delivered; not integrated) / d3f7a1 (read-only review, w-d3f7a1)
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

- [ ] Both defects are either fixed by a general change or refuted with a
      measurement. Measurement-only is not an acceptable end state; if
      neither can be fixed, deliver the implementation-ready specification
      (exact lines, per-design score arithmetic) so a later agent does not
      repeat the work.
- [ ] A unit test states the corrected behaviour of each axis in general
      terms (e.g. a clue preserving one target boundary cannot be docked
      below `1 - added/total`; a longer clue is not penalised purely for
      length), with no example phrase in the assertion.
- [ ] On the handed-over tree, measured with the release binary at
      `--approximate --top 50`, with a cutoff re-measured on that same tree:
      canonical case 1 still present, and canonical case 2's rank recorded
      explicitly, pass or fail.
- [ ] `cargo test --release --lib`, `--test corpus_integration`,
      `--test exact_determinism` and `--test approx_determinism` all run and
      their exact results recorded. The single known failure
      `approximate_finds_classic_madgab_resegmentation` is expected to remain
      red unless the rank criterion is genuinely met; say which.
- [ ] Branch `madgab-objective-axes` is **pushed**, the working tree is
      clean, and the HEAD is on that named branch and not a detached HEAD.
- [ ] This item is updated with objective state, validation, blockers and the
      next action, and pushed to `post-milestone-acceptance`. Never `main`.

## Handoff / notes

- Opened by coordinator `coord-9a4e` on 2026-09-27T02:18Z, from
  [w-c4d7e8](w-c4d7e8.md)'s published upper bound. Agent `9c6f2b`, worktree
  `/workspace/madgab-objective-axes`, branch `madgab-objective-axes` off
  `post-milestone-acceptance`.
- `cargo fmt` and `cargo clippy` are absent on this host. Do not report
  them as run; see `docs/environment-notes.md`.
- Independent, in-flight and **not** to be waited on:
  [w-4b1e07](w-4b1e07.md)'s integration queue (agent `5b1a02`) is landing
  three already-finished fronts onto `post-milestone-acceptance`. It alone
  may push that branch. This front must rebase onto whatever
  `post-milestone-acceptance` is when it validates, and must not push that
  branch itself.
- `It is just a stupid game` is **not** a substitute for
  `It's just a stupid game`: 20 phones, boundaries `[2,4,9,10,16,20]`,
  7 syllables. It is a different target.
