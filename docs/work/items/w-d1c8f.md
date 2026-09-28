---
work_item: true
id: w-d1c8f
state: done
priority: high
owner: agent-d1c8f1 (claimed 2026-09-27T04:53Z by coord-4e21 from post-milestone-acceptance 4b51dce; delivered verdict pass-with-follow-up and report f015b32; integrated by coord-4f7a as 33a46f1)
updated: 2026-09-27T05:20:00Z
opened_by: coord-4e21 (reconciliation pass 2026-09-27T04:52Z-04:55Z)
branch: madgab-review-d6c-d1c8f
worktree: /workspace/madgab-review-d6c
reviews: madgab-d6c-9d4e10 (36589f8..4f3442c) — the delivered w-9d4e10 D6c implementation
---

# Adversarial review of the delivered D6c per-phone `SIMILARITY` change before integration

## Why this item exists

Agent `9d4e11` finished and pushed the w-9d4e10 implementation on
2026-09-27T05:05Z-equivalent pass time: branch `madgab-d6c-9d4e10` at
`4f3442c`, two commits over the claim base `36589f8`
(`4679cf9` a coordinator mid-run durability snapshot, `4f3442c` the reviewed
commit). `madgab-d6c-9d4e10` is confirmed pushed and in sync with
`origin/madgab-d6c-9d4e10` at `4f3442c`, so the work is durable and safe to
review. It is **not yet integrated** into `post-milestone-acceptance`.

This is a read-only adversarial review front. Its job is to decide whether
`4f3442c` may be integrated, and to name precisely what is wrong if not.

## What the change claims

Per [w-5c11a2](w-5c11a2.md)'s *Implementation-ready specification*: normalise
the `SIMILARITY` axis per phone instead of per word, with
`axes::SIMILARITY_COST_PER_PHONE = 0.30` taken from the specification's swept
safe band rather than re-tuned. The stated general property being bought is
that a clue is no longer penalised for containing more words when the words
carry equal per-phone phonetic cost.

The agent reports: `metrics` reads
`cost_per_phone = self.sub_cost_total / total_len.max(1)`; `Metrics` gained a
`similarity: f64` field so the axis can be asserted alone;
`approximate_output_is_locked` re-locked; the superseded test
`similarity_is_scored_per_word_not_per_candidate` **removed and replaced** by
`similarity_is_charged_per_phone_not_per_word`; one new integration test
`a_short_multi_syllable_proposal_set_is_not_one_word_count_class` added.
Reported counts: `--lib` 58/0, `--test corpus_integration` 12 passed 1 failed
(only the expected-red `approximate_finds_classic_madgab_resegmentation`),
`--test exact_determinism` 1/0, `--test approx_determinism` 4/0,
`--test emit_coverage` 4/0, `--test no_phrase_hard_coding` 7/0. Canonical
case 1 measured present at printed rank 27 (was 28), score 0.920 (was 0.918).

## The questions this review must answer

1. **The deleted test.** `w-9d4e10`'s fences say *do not relax, re-baseline or
   `#[ignore]` a test to make something pass*. The agent deleted
   `similarity_is_scored_per_word_not_per_candidate` and replaced it, arguing
   the deleted assertion encoded exactly the per-word specification D6c
   replaces, and that the replacement is strictly stronger. Judge that claim on
   its merits. If the replacement genuinely does not cover any property the
   deleted test covered that still holds, say so; if the deletion was a
   re-baseline in disguise, say that instead. Do not accept the agent's own
   justification as the argument.
2. **Constant semantics.** Is `SIMILARITY_COST_PER_PHONE = 0.30` used as
   specified, with no re-sweep or re-tuning smuggled in via the new
   `total_len.max(1)` guard, the `Metrics.similarity` field, or any other
   change? Diff the actual arithmetic against the specification in
   [w-5c11a2](w-5c11a2.md).
3. **The three enumeration-side `SIMILARITY_PER_WORD` proxies** are claimed to
   be deliberately left on the old scale. Verify they really are untouched and
   that leaving them is defensible rather than an inconsistency the change
   introduces.
4. **Is anything else in the diff?** The claim is "nothing else": no weight
   change, no `PUNCH`/`RHYTHM` change, no budget widening, no cap or emission
   site touched (`src/lib.rs:1877`, `src/lib.rs:1955`), no `zz*` file, no
   `eprintln!` probe, no env-var knob, no phase timing, no `#[ignore]`. Verify
   from the diff, not from the report. `src/lib.rs` shows 241 changed lines,
   which is a lot for one formula; account for every hunk.
5. **The re-locked `approximate_output_is_locked`.** Confirm it was re-locked
   against the *new* behaviour with measured values rather than re-baselined
   to whatever the code happened to print, and that the w-4e2b19
   self-contradicting comment is genuinely gone.
6. **Fence sweep.** Re-run the sweep the item specifies —
   `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"` over
   `src`, `web` and `examples` — and classify every hit. The claim is the hit
   set is byte-identical to [w-d5a2c1](w-d5a2c1.md)'s at `42ced98`. Verify
   that, and check by eye as well as by
   `tests/no_phrase_hard_coding.rs`, which strips comments.
7. **Canonical case 1 must not regress.** Measure it yourself through the
   shipped release executable rather than trusting the reported rank 27. The
   itinerary's standing goal is the canonical approximate-search examples, so
   an unverified case-1 claim is the single most expensive thing to get wrong
   here.
8. **The unmeasured predictions.** The agent explicitly did not measure the
   case-2 cutoff `-0.002007`, the case-1 cutoff `+0.001771`, or the case-2
   alignment's exact residual, on the grounds that measuring them exactly
   would need an instrument the item forbids. Assess whether that is a
   legitimate deferral or an untested claim that should block integration.
   Per [w-d5c11a2](w-d5c11a2.md) the design is *not* expected to reach
   canonical case 2, so absence of that is not a defect.

## Fences for this front

- **Read-only review.** Produce a written verdict and a report. Do **not**
  change `src/` or `tests/`, do not fix anything, do not re-sweep the constant.
  If you believe a change is needed, say so in the verdict and let the next
  coordinator dispatch an implementation front.
- Scratch instrumentation goes on a `scratch/*` ref only, never on this branch.
- No phrase-specific hard-coding anywhere, including in the report's own test
  snippets. Quote the canonical strings only when naming them.
- Validate with `cargo test --release --lib` and
  `cargo test --release --test <name>`. `CARGO_TARGET_DIR` must be outside
  `/tmp` (`/tmp` is `noexec`). `cargo fmt` and `cargo clippy` do not exist on
  this host; state them as unavailable rather than as passing.
- `git`, `grep`, `sed`, `awk` and `date` need
  `export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"` first — see
  [../environment-notes.md](../environment-notes.md). A `command -v` miss here
  is a truncated `PATH`, not a host outage, and a pass has already been misled
  by that twice.
- Do not touch `main`. Do not merge anything into
  `post-milestone-acceptance`; integration is the next coordinator's job.
- Do not open a new measurement front, and do not re-derive the refutations
  already recorded in [w-d3f7a1](w-d3f7a1.md).

## Completion criteria

- [ ] Every one of the eight questions above is answered with evidence, and
      the evidence is a command output, a diff excerpt, or a measured run — not
      a restatement of the implementer's report.
- [ ] A verdict of exactly one of: **pass** (integrate `4f3442c`),
      **pass-with-follow-up** (integrate, and a named new work item is
      required), or **hold** (do not integrate; the named defect must be fixed
      on the front first).
- [ ] Canonical case 1 independently measured through the release executable,
      with rank and score, and the word-count-class distribution of the printed
      50 for both canonical targets recorded before and after.
- [ ] `git diff 36589f8..4f3442c` is fully accounted for hunk by hunk.
- [ ] Fence sweep reported as raw hits plus a per-hit classification.
- [ ] The report is committed on `madgab-review-d6c-d1c8f` and pushed;
      working tree clean, `HEAD` on the branch and not detached.
- [ ] This item updated with the verdict, evidence, blockers and the next
      action, and pushed to `post-milestone-acceptance`. Never `main`.

## Handoff / notes

- Opened by coordinator `coord-4e21` on 2026-09-27T04:53Z, in the pass that
  found agent `9d4e11` terminal (`succeeded`) on
  [w-9d4e10](w-9d4e10.md) with its work delivered and pushed but unreviewed
  and unintegrated.
- The review worktree is based at `4f3442c`, the front's tip, so the diff under
  review is exactly `36589f8..4f3442c`.
- `1f6c41` (the [w-1f6c40](w-1f6c40.md) baseline measurement front) is
  **running** in `/workspace/madgab-baseline-1f6c40` on branch
  `madgab-baseline-1f6c40` and was left running by the opening pass. Its branch
  is not yet pushed, which is expected for a front four minutes old. Do not
  contend with it; it measures the tree, this front reviews it.
- If the verdict is `pass` or `pass-with-follow-up`, the next coordinator can
  integrate `4f3442c` into `post-milestone-acceptance` with a merge or a
  cherry-pick of `4f3442c` alone, since `4679cf9` is a coordinator durability
  snapshot and not part of the reviewed change.

## Outcome: PASS-WITH-FOLLOW-UP, delivered by `d1c8f1`, integrated as `33a46f1`

`d1c8f1` reported on 2026-09-27T05:12Z. Report committed and pushed on
`madgab-review-d6c-d1c8f` at **`f015b32`**, working tree clean, `HEAD` attached
to the branch, `main` untouched. All eight questions answered; the answer index
is in the report.

Verdict: **pass-with-follow-up**.

- **Q1, the deleted test — legitimate replacement, not a re-baseline.** The
  deleted `similarity_is_scored_per_word_not_per_candidate` was restored into the
  new tree and is **red** with its original message; its claim (flat in word
  count at equal *per-word* cost) is the literal negation of the D6c
  specification. The replacement `similarity_is_charged_per_phone_not_per_word`
  is **red on the pre-change base `36589f8`**. Discriminating in both
  directions, which a re-baseline cannot be.
- **Q2, canonical case 1 — not regressed, measured independently** through the
  shipped release executable: `wreck a nice beach` at printed **rank 27**,
  score **0.920**, word-count classes `{4: 50}`. A from-scratch build of the base
  `36589f8` gave rank 28 / 0.918 / `{4: 50}`. Exact score via `generate_pool`
  0.918313383 -> 0.919950288, matching the w-5c11a2 prediction 0.919950 exactly.
- **Q3, the fence sweep — byte-identical, confirmed** by SHA-256 of the
  normalised hit set at `42ced98` and `4f3442c`: `f0f21e86`. 49 hits, none in a
  decision; only real-code hits are two English `came`s in `expect()` panics and
  the DOM `hidden` property. The implementer's reported counts were wrong
  (30/9/3 = 42; actually 37/9/3 = 49) — bookkeeping only.
- **Q8, the unmeasured predictions — a legitimate deferral, not a blocker.** The
  reviewer measured them anyway: both cutoffs confirm exactly (case 2
  `-0.002007`, case 1 `+0.001771`). The pool size does not confirm, which is F1.
- Case 2 classes moved from `{6: 50}` to `{5: 6, 6: 44}`: the monopoly breaks.
  All seven targets' before/after distributions reproduced exactly.

### The last paragraph of the handoff above is WRONG, and integrating by it would
### have been a re-baseline

It says a cherry-pick of `4f3442c` alone suffices because `4679cf9` is "a
coordinator durability snapshot and not part of the reviewed change". That is
false. `4f3442c` is **deletions-only** in `src/lib.rs`; the
`SIMILARITY_COST_PER_PHONE` formula and both new tests live in `4679cf9`. Proven
by cherry-picking `4f3442c` alone onto `36589f8` and finding
`SIMILARITY_COST_PER_PHONE` present **0 times** with `emit_coverage` still at 3
tests — and a red re-lock waiting to be "fixed" by re-baselining, which is
exactly what the w-9d4e10 fences forbid. **Merge the branch tip, or take both
commits.**

### What was actually integrated, and how it was validated

`coord-4f7a` merged the **branch tip** `4f3442c` (not the lone commit) into
`post-milestone-acceptance` as **`33a46f1`**. Only
`docs/work/items/w-9d4e10.md` conflicted; `src/lib.rs`,
`tests/corpus_integration.rs` and `tests/emit_coverage.rs` merged cleanly, as
the reviewer's test-merge had predicted.

On the integrated tree: `--lib` **58/0**; `corpus_integration` **12 passed /
1 failed** — the expected-red `approximate_finds_classic_madgab_resegmentation`,
assertion unmodified; `exact_determinism` **1/0**; `approx_determinism` **4/0**;
`emit_coverage` **4/0**; `no_phrase_hard_coding` **9/0**. All six suites were
run to completion after the merge and every count matches `d1c8f1`'s independent
measurement of the same commits exactly.

Unavailable on this host, confirmed rather than assumed: `cargo fmt`,
`cargo clippy`, `cargo test --doc`, wasm32 build. Stated as unavailable, not
passing.

### Follow-ups raised, not dropped

Filed as **[w-474813](w-474813.md)**: F1, the pool did not stay put
(18,242 -> 18,270; 18,917 -> 18,936) against the w-5c11a2 prediction, which
means the case-2 objective gap has a stale denominator; and F2, the axis's `0.0`
floor is no longer pinned by any test after the deletion Q1 accepted.

### Corrections the reviewer recorded against itself

- The item *did* exist, at `2d19836` on `post-milestone-acceptance`. The review
  branch is based at `4f3442c` and cannot see it; the reviewer initially
  mis-recorded this as a missing item and corrected it in `f015b32`.
- One pool measurement was first taken against a stale `CARGO_TARGET_DIR` and
  was re-measured per-tree.
- F6: the `approximate_output_is_locked` lock lives in `corpus_integration`, not
  `approx_determinism`; the attribution in w-5c11a2 and w-9d4e10 is wrong.

