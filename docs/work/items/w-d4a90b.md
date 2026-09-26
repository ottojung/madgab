---
work_item: true
id: w-d4a90b
state: working
priority: high
owner: agent-d4a90b
updated: 2026-09-26T21:40:00Z
branch: madgab-postpunch-measure
worktree: /workspace/madgab-d4a90b
---

# Re-measure the second canonical example on the post-PUNCH head

## Goal

Every recorded distance for the itinerary's **second** canonical example —
`It's just a stupid game` -> `Hits Justice Dupe Hid Came` — was taken on a
head that predates the PUNCH scoring axis (`6a93c2a`, integrated from
[w-6ad4c1](w-6ad4c1.md)). PUNCH is a seventh additive axis in the objective, so
the numbers in [w-4b1e07](w-4b1e07.md), [w-558697](w-558697.md) and
[w-7b41d2](w-7b41d2.md) are stale for exactly the target the milestone needs.
The first example *was* re-measured post-PUNCH; this one never was.

This is a **measurement-only** front. It opens no scoring hypothesis, changes no
constant, and proposes no fix. Its output is the current distance, quoted as
both a score delta and a rank, at the CLI default `top_n` and at `--top 50`, so
the next routing decision (including `9f1c05`'s aggregation-form verdict) is
taken against the current tree rather than a three-axis-old one.

## Fence

- **No edits to `src/` or `tests/`.** This front measures; it does not change the
  thing being measured. Any temptation to "just widen the budget so it fits" is
  exactly the phrase-adjacent hack the itinerary forbids.
- No change to any `axes::*` constant, retention constant, or the selection
  policy. Those belong to the closed families in [w-3f9c02](w-3f9c02.md) and
[w-8a1d47](w-8a1d47.md) and to the live front [w-9f1c05](w-9f1c05.md).
- Artifacts and the report go to `measurements/d4a90b/` and
  `REPORT-d4a90b.md` on `madgab-postpunch-measure`, pushed as they are produced.
  **Never merge that branch into `post-milestone-acceptance`**; the report is
  copied into this work item by the coordinator.
- If a `--release` build is needed, build in this worktree only.

## Questions to answer, in order

1. On `post-milestone-acceptance` at the current head, what is the release
   binary's approximate output for `It's just a stupid game` at the **CLI
   default `top_n`** and at **`--top 50`**? Give the pool count, the top-band
   cutoff, and whether `hits justice dupe hid came` is enumerated at all.
2. If it is enumerated: its score, the score of the worst visible proposal, the
   **score delta**, and its **rank** in each configuration. Quote the delta and
   the rank separately; do not collapse them into one "visible" claim.
3. Per-axis decomposition of that delta **on the current head** — the question
   [w-558697](w-558697.md) answered for the old objective, re-asked because
   PUNCH is a new axis and may have changed which term dominates. State
   explicitly whether any axis now *moves the target* (as opposed to moving the
   band around it), and whether the six weights still sum to exactly 1.00.
4. Regression status, quoted exactly: `cargo test --release --test
   corpus_integration` — is `approximate_finds_recognize_speech_resegmentation`
   green and `approximate_finds_classic_madgab_resegmentation` still red? No
   test may be edited to change this.
5. Wall clock for one default approximate run and one `--top 50` run, so the
   next pass can price any breadth-based proposal.

## Completion criteria

- `REPORT-d4a90b.md` exists and is pushed to `origin/madgab-postpunch-measure`,
  with a one-line `VERDICT:` on its first line summarising the delta and rank.
- All five questions answered with quoted numbers, each traceable to a command
  and a base commit.
- No file under `src/` or `tests/` modified on the branch (`git diff
  post-milestone-acceptance...madgab-postpunch-measure -- src tests` empty).
- The coordinator has copied the numbers into this item, after which
  `state: done`.

## Handoff / notes

Launched 2026-09-26T21:40Z by coordinator `coord-2d6f` as Antonina agent
`d4a90b` in `/workspace/madgab-d4a90b`, branched from `post-milestone-acceptance`
at `f5b9eaa`. It does not contend with `9f1c05` (different worktree, and
`9f1c05` owns the aggregation-form design while this front owns only the
current-tree distance). Left running; a later fresh pass inspects it.

The second example's *status* is unchanged by this item: the milestone is still
unmet, and closing it belongs to [w-2f7a10](w-2f7a10.md), [w-4b1e07](w-4b1e07.md)
and [w-a02d28](w-a02d28.md) together.
