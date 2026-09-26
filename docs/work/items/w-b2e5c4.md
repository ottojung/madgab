---
work_item: true
id: w-b2e5c4
state: working
priority: high
owner: agent-b2e5c4
updated: 2026-09-26T22:52:00Z
branch: madgab-representation-b2e5c4
worktree: /workspace/madgab-representation-b2e5c4
---

# Represent enumerated resegmentations in the emitted set: the gate is visibility, not enumeration

## Why this item exists

Three independent fronts finished on 2026-09-26 and all three converge on the
same finding, from different directions:

- [w-1c3e77](w-1c3e77.md) (front `4d3ab7`, `madgab-enum-depth4`, code
  `784deaa`, report `8a5dd3b`): force-emitting the canonical wording into the
  pool on the base gives it score `0.799901` at **rank 9997 of 17826**; the
  best candidate of its structure is rank 3376; the visible `--top 50` cutoff
  is `0.898`. `select_diverse` fills 50 slots from the head of the score
  order, so the milestone test is a **visibility assertion, not a
  pool-membership assertion**.
- [w-9c4d21](w-9c4d21.md) (front `9c4d21`, `madgab-aggform-9c4d21`): 24
  configurations of the reserve's three constants, 4,096 draws against
  2.67e9 points; the wording is rank ~50-100 **by score** and rank ~33.6M
  **by cost**, so ordering the reserve's sample by an admissible score bound
  rather than by cost is the named next action.
- [w-a7e2b3](w-a7e2b3.md) (review): front A's mechanism is kept but its env
  knob / `eprintln!` probe / example-naming doc comments are fence
  violations; front B's mechanism is arithmetically refuted.

So more enumeration, deeper reserves, wider slots and better segmentation
retention are all measured to be *insufficient levers* for the canonical
milestone: the candidate is reachable in the pool and still never surfaces.
The unfrozen mechanism is the **representation step**: how enumerated
candidates of a structurally under-represented *resegmentation* earn an
emission slot when their own score is below the head-of-order cutoff.

## Goal

Make the emission/selection step represent enumerated resegmentations on a
criterion other than "this candidate's own score is inside the visible
cutoff", in a way that is general, phrase-free, and does not special-case
any input.

The minimal form named by `w-1c3e77`'s report is a **per-structure reserve
slot**: `select_diverse` step 1 reserves a bounded number of slots for the
best candidate of each distinct (structure / resegmentation) class rather
than letting the top-50 by score monopolise all slots.

## Constraints and fences

- **No phrase-specific hard-coding.** No `recognize speech`, no
  `wreck a nice beach`, no `It's just a stupid game`, no
  `Hits Justice Dupe Hid Came`, no per-input exceptions anywhere in `src/`,
  including **doc comments and string literals** (`tests/no_phrase_hard_coding.rs`
  strips comments, so the human/agent reviewer must read the diff by eye as
  well as run the test).
- **No `env::var` production knobs** in `src/lib.rs` for this experiment and
  **no `eprintln!` census probe harness** in production code. Measure with
  tests, `--` CLI surface, or a scratch branch; do not land diagnostics.
- Do not weaken, relax or edit the assertion in
  `approximate_finds_classic_madgab_resegmentation` to make it pass.
- Do not change the milestone criterion: the requested clue must appear in
  the generated approximate proposal set for its input.
- General behaviour only: whatever representation rule lands must be
  justifiable for inputs other than the canonical two, and must have a
  stated cost (slots, runtime, pool size) measured before/after.

## Contention note

The live `src/lib.rs` fronts are:

- `/workspace/madgab-retain-6f2b18` (`w-6f2b18`, agent `6f2b18`, retention
  census, uncommitted `tests/retention_census.rs`),
- `/workspace/madgab-enum-depth4` (`w-1c3e77`, terminal, code at `784deaa`
  not yet landed).

This front targets `select_diverse` / emission slot assignment, which is
outside the `slot_is_affordable` and `affordable_opening_width` hunks in
`784deaa`. Work only in this front's own worktree. The coordinator sequences
merges; the second `src/lib.rs` branch is rebased onto the first.

## Completion criteria

- `cargo test --release --test corpus_integration
  approximate_finds_classic_madgab_resegmentation` passes on this branch
  with its assertion unmodified, **or** the item closes with a measured
  refutation of the representation hypotheses and the arithmetic on record.
- `cargo test --release --lib`, `--test corpus_integration`,
  `--test exact_determinism`, `--test approx_determinism`,
  `--test no_phrase_hard_coding` all pass.
- A new regression test is shown **red** on the base it claims to fix.
- Report **ENUMERATED** and **RANKED**/`SURFACED` separately, with pool
  size, emitted-slot count, wall clock and score distribution before/after,
  for **both** canonical targets and at least two non-canonical inputs
  (so a general improvement is distinguishable from a lucky constant).
- The branch is committed and pushed; verify with `git ls-remote --heads
  origin <branch>` rather than a local ref.

## Handoff / notes

Opened 2026-09-26T22:48Z by coordinator `coord-9f4b`, from the convergence
of `w-1c3e77`'s visibility measurement, `w-9c4d21`'s score-vs-cost ordering
result and `w-a7e2b3`'s review verdict. Agent id reserved: `b2e5c4`.
Worktree: `/workspace/madgab-representation-b2e5c4` (branch
`madgab-representation-b2e5c4`), from `post-milestone-acceptance`.

2026-09-26T22:52Z, coordinator `coord-9f4b`: claimed and launched. Agent
`b2e5c4` created at 22:49Z in `/workspace/madgab-representation-b2e5c4`,
prompted with the full brief, `alive: yes` at handoff. Left running and
unsteered (this repository has previously lost an agent to a mid-turn
steer). Nothing is landable from it yet.
