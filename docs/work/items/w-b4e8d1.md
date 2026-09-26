---
work_item: true
id: w-b4e8d1
state: working
priority: normal
owner: agent-b4e8d1
updated: 2026-09-26T20:50:00Z
branch: madgab-review-b4e8d1
worktree: /workspace/madgab-review
---

# Review front: audit the accumulation branch for phrase-specific hard-coding, and review the adjacency WIP against the fences

## Goal

Two read-only obligations that the itinerary imposes on the milestone and that no
other front owns, kept off the region `c1d3a7` is editing so the two do not
contend:

1. **The no-hard-coding audit.** The milestone in
   [../skills/itinerary-madgab.md](../skills/itinerary-madgab.md) requires that
   the implementation "does not contain phrase-specific hard-coding for these
   examples". That has been asserted repeatedly in work-item prose and never
   *checked* against the tree on the accumulation branch. Check it, and record
   the method and the result so a later pass can re-run it in one command.
2. **A fence review of the in-flight adjacency WIP.** `c1d3a7`'s commit `c71e699`
   on `madgab-adjacency` carries an adjacency operator and its call site. Review
   it as a diff for the properties the fences in
   [w-c1d3a7](w-c1d3a7.md) require, and report defects **early**, while the agent
   can still fix them, rather than at integration time.

## Why this is useful and not a manufactured backlog

Integration currently has no reviewer: the single live front
([w-c1d3a7](w-c1d3a7.md)) is also the only thing that could review itself, and a
finished agent's own claim to generality is not review. This front is cheap,
read-only, and materially shortens the integration step that the itinerary
requires before anything lands on `post-milestone-acceptance`.

## Scope

- Read-only with respect to `src/` and `tests/`. Do **not** edit production code,
  and do not "fix" anything you find; report it.
- Your only writes are this work item and, if useful, one new work item for a
  real defect you find (see below). Do not touch `w-c1d3a7.md`; that agent owns
  it and is editing it right now, so a write from you would collide at rebase.
- Report ENUMERATED and RANKED separately wherever you report a pool result.
  Conflating them has cost a pass on this repository before.

## Part 1: the hard-coding audit

On `post-milestone-acceptance` (`fdc2769` at claim time), search all of `src/`
and `tests/` for the canonical example words and their distinctive substrings:
`wreck`, `wrecked`, `beach`, `recognize`, `recognise`, `speech`, `hits`,
`hit `, `justice`, `stupid`, `dupe`, `hid`, `came`, plus the full target strings
`just a stupid` and `it's just`. Also look for the *shape* of phrase-specific
special-casing, which is what actually matters and which a word list alone can
miss:

- any comparison of a whole input target, or of a whole target sentence, against
  a literal;
- any lookup keyed on a specific word identity rather than on a phonetic
  property, a cost, a score or a structural feature;
- any test-only constant that a production branch reads (a `#[cfg(test)]` value
  or an environment variable consulted in `src/` and used to steer the search);
  the standing exception is the `MADGAB_TRACE_*` diagnostics, which are
  observation-only and must not change a result — **verify** that they are
  observation-only by reading them, not by assuming it, and say so explicitly;
- any `zz_`/`ZZ_` identifier, `.bench` tree, or `MADGAB_*` diagnostic other than
  the standing trace ones.

Deliver the exact command or tool you used, so the next pass can re-run it
without re-deriving the method.

## Part 2: the fence review of `c71e699`

Read `c71e699` (`git -C /workspace/madgab-adjacency show c71e699`) and review
against these, which come from the fences in [w-c1d3a7](w-c1d3a7.md) and the
itinerary:

- **No phrase-specific case.** Part 1's standard applies to this diff too, at
  the level of the new code.
- **Bounded, with named arithmetic.** Any new constant, cap, pass count or
  iteration bound must have a stated derivation, and the bound must be derivable
  from an observable property of the input rather than from a chosen target.
- **No `axes::*` move** and no re-baselining of `approximate_output_is_locked`.
  Pool membership is the requirement; the score need not change.
- **No diagnostic left behind**: no `zz_`/`ZZ_`, no `MADGAB_*` outside the
  standing trace family, no `.bench`, no build tree, no scratch path.
- **No wall-clock or pool regression** that the change cannot pay for. Report
  `recognize speech` and `It's just a stupid game` search times and pool sizes
  on the merged base and, if you can build the WIP branch cheaply, on `c71e699`
  too; if the build is too expensive, say so and skip rather than guess.
- **Determinism**: the change must not introduce a HashMap/HashSet iteration
  order into anything that affects output. `--test exact_determinism` and
  `--test approx_determinism` are the gate for that, and the seeds are fixed.

`c71e699` is a WIP commit and the agent is still editing, so a WIP defect is
expected. Report what is wrong with the *current* text and say explicitly that
it is a WIP reading.

## Part 3: the blocker, confirmed from the merged head

The single known blocker is
`approximate_finds_classic_madgab_resegmentation` in
`--test corpus_integration`. It was re-measured on the merged head `fdc2769` by
the coordinator and failed with 9 passed / 1 failed. Re-run it once yourself and
record the exact failure text, so the record does not rest on a single
observation:

```sh
cd /workspace/madgab-review && cargo test --release --test corpus_integration
```

Report the failure message verbatim, including the visible wordings it got.

## Constraints

- Never merge or push to `main`. Your branch is `madgab-review-b4e8d1`; push it
  and record the remote sha.
- No `zz_*` test, sweep script, `.bench` tree or `MADGAB_*` diagnostic on your
  branch. Scratch work belongs in `/tmp`.
- `cargo fmt`, `cargo clippy` and doctests do not exist on this host — see
  [../environment-notes.md](../environment-notes.md). Do not report them.
- `cargo test --release --lib`, `--test exact_determinism` and
  `--test approx_determinism` must be reported as measured before you close.
- Do not run long measurement sweeps. This is a review front, not a measurement
  front. If something would take minutes, record it as unmeasured.

## Completion criteria

1. The hard-coding audit is done, with the method recorded as a re-runnable
   command, and a verdict that is either "clean, with the exceptions listed" or
   "defects at <file:line>", and every standing exception individually
   justified.
2. The `c71e699` fence review is written up, defect-first, each defect with a
   `file:line` and what the fence it violates is.
3. Any *real* defect in the adjacency mechanism that the agent cannot reasonably
   be expected to find is captured as a **new** work item created from
   [../work/TEMPLATE.md](../work/TEMPLATE.md) — a defect about mechanism
   behaviour is not a work item about this review, and must not die with this
   one. Link it from this item's handoff.
4. The blocker is re-measured on the merged head and its exact failure text
   recorded.
5. `--lib`, `--test corpus_integration`, `--test exact_determinism` and
   `--test approx_determinism` reported as measured, with the one known failure
   named.
6. This item updated with objective state, branch pushed, worktree clean.

## Handoff / notes

Claimed by coordinator `coord-2f6a` on 2026-09-26T20:50Z from
`post-milestone-acceptance` at `fdc2769`, worktree `/workspace/madgab-review`,
branch `madgab-review-b4e8d1`, agent `b4e8d1`. Launched in parallel with
`c1d3a7`, which is still running on `madgab-adjacency`; this front is fenced away
from that worktree and from `src/`.

Board resources: `antonina board resource list` returned nothing on this host, so
no board dependency is registered for `/workspace/madgab-review`. The pushed
branch and this item are the durability mechanism, per
[../skills/resources.md](../skills/resources.md).
