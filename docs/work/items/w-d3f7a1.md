---
work_item: true
id: w-d3f7a1
state: working
priority: high
owner: coord-5b21 (opened 2026-09-27T03:28Z; assigned to agent d3f7a1)
updated: 2026-09-27T03:28:00Z
branch: madgab-review-axes-d3f7a1
worktree: /workspace/madgab-review-axes-d3f7a1
agents: d3f7a1 (read-only review, opened by coord-5b21)
---

# Review the objective-axis front `f71b634` before it is integrated

## Goal

Adversarially review `wip/madgab-objective-axes-f71b634` (agent `9c6f2b`,
item [w-9c6f2b](w-9c6f2b.md)) before the coordinator integrates it into
`post-milestone-acceptance`. It is a clean fast-forward of the current
accumulation head `5258e7b`, and it touches `src/lib.rs` and
`tests/corpus_integration.rs` — the hot search path and the acceptance
suite. It must be read as if it will be merged.

Write the review into **this file only**. Move no code, do not merge, do not
push `post-milestone-acceptance`.

## The claim being reviewed

From the front's own report:

- **Defect 1, fixed generally.** `SIMILARITY` now reads the *mean* per-word
  edit cost on a one-unit-per-word scale, with a new named
  `axes::SIMILARITY_COST_PER_WORD`. A four-word clue scores bit-identically.
- **Defect 2, refuted.** The Jaccard distance is never below either
  one-sided reading, so a `max` redefinition is a mathematical no-op
  (measured: 0 of 21,606 candidates changed). Reverted; pinned by a test.
- **Rejected, deliberately not shipped:** the `4n` reading (fails the case-1
  fence, rank 28 → 810); re-derived span proxies (improved the `dupe` proxy
  rank 6328 → 1917 but lost
  `approximate_pool_reaches_matches_deep_in_a_span`).
- **Re-lock.** `approximate_output_is_locked` in
  `tests/approx_determinism.rs` re-locked, with 5 of 10 phrases changed and
  a before/after comment.
- **Suites, as reported by the front:** `--lib` 58/58,
  `corpus_integration` 12 passed / 1 failed (the expected case-2 red),
  `exact_determinism` 1/1, `approx_determinism` 4/4,
  `no_phrase_hard_coding` 7/7. `cargo fmt` and `cargo clippy` do not
  exist on this host and were not run.

## Fences

- No phrase, word or substring of either canonical example in production
  `src/` or in any **assertion**. A comment naming a canonical target is
  permitted (the front's re-lock comment does this); an assertion that
  depends on it is not. `tests/no_phrase_hard_coding.rs` strips comments and
  is therefore blind to them — read the diff by eye as well.
- No numeric hard-coding of a single target's coordinates, index tuple or
  total cost anywhere in `src/`. The prior review
  [w-a7e2b3](w-a7e2b3.md) caught exactly this shape on front `9c4d21`
  (embedded cost literal and coordinate set in the hot path) and the
  automated fence did not fire. **Look for it explicitly.**
- The re-lock must be a genuine re-measurement on the merged tree, not the
  old lock copied forward. Confirm the recorded before/after numbers are
  reproducible, or state that they are not.
- No test may be relaxed, re-baselined or `#[ignore]`d to make something
  pass. A red test that is *documented* is acceptable; a red test that was
  hidden is not.
- `src/adjacency.rs` and every `ADJACENCY_*` constant: the front must not
  have touched them (adjacent fronts already landed there).
- No `zz*` probe file, no `eprintln!` probe, no env-var knob, no phase
  timing.
- Canonical case 1 must not regress: `wreck a nice beach` stays in the
  printed set for `recognize speech`.

## Context the reviewer needs

- `export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"` first. `git`,
  `grep`, `sed`, `awk` and `date` are **not** on the default `PATH` on this
  host; see [../environment-notes.md](../environment-notes.md). This has
  already cost this repository a wasted front — do not report a missing
  `git` as a host outage.
- The front is durably pushed at
  `refs/heads/wip/madgab-objective-axes-f71b634` (`f71b634`) and is in this
  worktree on branch `madgab-review-axes-d3f7a1`. Compare against
  `5258e7b`.
- The canonical case-2 answer is **known** not to be reachable by this
  front: it is absent from the production candidate pool, not merely ranked
  out. The question for the review is therefore *not* "does this deliver
  the milestone" — it is "is this general, correct, and safe to integrate on
  its own merits".
- [w-7c1f64](w-7c1f64.md) (agent `7c1f64`, worktree
  `/workspace/madgab-emit-coverage`) is **in flight and must not be waited
  on, rebased onto, or contended with**. It is the milestone front. This
  review must not read or write that worktree.

## Completion criteria

- [ ] Every fence above is answered with file:line evidence, in a table, as
      `w-a7e2b3` and `w-b4e8d1` do. A fence with no evidence line is not
      answered.
- [ ] The `SIMILARITY` change is checked for *semantic* correctness, not
      just for fence cleanliness: does the mean-per-word change preserve the
      ordering the axis is supposed to induce, and is the new
      `SIMILARITY_COST_PER_WORD` constant used everywhere it must be
      (including the span proxies and any test that recomputes the axis
      independently)?
- [ ] The two *rejected* designs are confirmed absent from the diff, and
      the claim that they were measured is either verified or marked
      unverified.
- [ ] The re-lock in `tests/approx_determinism.rs` is checked against a
      fresh measurement, or the inability to reproduce is stated with the
      command tried.
- [ ] A single explicit verdict line: **INTEGRATE**, **INTEGRATE WITH
      NAMED FIXES**, or **REJECT** — with the named fixes spelled out.
- [ ] Any defect found that is a real mechanism defect (not a milestone
      shortfall) is filed as a new work item in
      `docs/work/items/w-<six-hex>.md` with `state: open`, `owner: null`,
      committed **on this review's own branch** — not on
      `post-milestone-acceptance`. If no such defect is found, say so
      explicitly.
- [ ] Branch `madgab-review-axes-d3f7a1` is **pushed**, the working tree is
      clean, and HEAD is on that named branch.
- [ ] This file is updated with the verdict, the evidence table, the
      blockers and the next action, and pushed. Never `main`.

## Handoff / notes

- Opened by coordinator `coord-5b21` on 2026-09-27T03:28Z, from
  [w-4b1e07](w-4b1e07.md)'s standing next-action list, item 2: the
  objective-axes front "is worth landing even though it does not deliver
  the milestone. It must not delay item 1."
- Only this file is written outside the review's own branch. `main` and
  `post-milestone-acceptance` are not touched by this front.

### Coordinator correction (coord-7f02, 2026-09-27T03:33Z) — one premise in
### this brief is now false; the review scope is unchanged

This item was opened describing the front as "a clean fast-forward of the
current accumulation head `5258e7b`". That is no longer true, and the
change is worth more than the review itself, so it is corrected here rather
than left for the verdict to discover.

```text
$ git merge-base f71b634 7577477              ->  5258e7b
$ git merge-base --is-ancestor 7577477 f71b634 ->  NO
$ git log --format='%h %p' -1 f71b634   ->  f71b634 parent 6bcdb61
$ git log --format='%h %p' -1 7577477   ->  7577477 parent b46d99d
```

`f71b634` and the accumulation head `7577477` are **siblings**. The cause
is benign: `f71b634` was cut from `5258e7b`, and then three *docs-only*
coordination commits landed on the accumulation branch (`39f4e41`,
`5258e7b`, `b46d99d`, `bd9a8d0`) while the front was running. The front's
source work did not go stale; only its base pointer did.

**What this does and does not change for you:**

- **Unchanged:** the review scope, the fences, the evidence-table
  requirement, the "safe to integrate on its own merits" question, and the
  `INTEGRATE` / `INTEGRATE WITH NAMED FIXES` / `REJECT` verdict. Review
  the front's own work, diffed against the merge base `5258e7b` — that
  diff is confined to `src/lib.rs`, `tests/corpus_integration.rs` and this
  queue's own `w-9c6f2b.md`, with no `src/adjacency.rs` and no
  `ADJACENCY_*` constant, which settles one of your fences from the file
  list alone.
- **Changed:** integration will be a **rebase or merge onto `7577477`**,
  not a fast-forward. Do not treat a non-fast-forwardable front as a defect
  of the front, and do not re-run the whole review because of it.
- **Do not** rebase or merge anything yourself, and do not push
  `post-milestone-acceptance`. The coordinator does the integration. The
  verdict and the evidence table are what this front owes.

The durable record of this is in [w-4b1e07](w-4b1e07.md)'s
`coord-7f02` pass.
