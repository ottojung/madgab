---
work_item: true
id: w-d5a2c1
state: working
priority: normal
owner: agent-d5a2c2 (claimed by coord-0a11 on 2026-09-27T04:12Z)
updated: 2026-09-27T04:12:00Z
opened_by: coord-0a11 (reconciliation pass 2026-09-27T04:12Z, on post-milestone-acceptance at c28b29f)
branch: madgab-audit-d5a2c1
worktree: /workspace/madgab-audit-d5a2c1
agents: d5a2c2 — see Handoff
---

# Fence and test-truth audit of the current `post-milestone-acceptance` tree

## Why this item exists

The recurrence front is [w-5c11a2](w-5c11a2.md), owned and running on agent
`c0ff01` in `/workspace/madgab-gap-recheck`. That front is a *search/ranking*
front. This item is deliberately a different front: an **independent audit of
the accumulated tree itself**, so that a later integration pass can accept or
reject `madgab-gap-recheck` against a pre-measured, current baseline instead of
trusting whatever a single agent reported.

It runs in its own worktree and its own branch, touches no file that
`madgab-gap-recheck` touches, and must not open a second search front.

## Goal

Establish, by measurement, the current objective state of
`post-milestone-acceptance` at `c28b29f` across five axes, and write the
result down where the next coordinator will read it.

## What to measure

1. **Test truth.** Run and record exact pass/fail counts for
   `cargo test --release --lib`, `--test corpus_integration`,
   `--test exact_determinism`, `--test approx_determinism`,
   `--test emit_coverage`, `--test no_phrase_hard_coding`. The expected
   baseline is: everything green except
   `approximate_finds_classic_madgab_resegmentation` in
   `tests/corpus_integration.rs`. **Any other failure is a finding**, and any
   *newly passing* milestone test is a finding too — report the actual numbers,
   not the expected ones.
2. **Canonical case 1.** `wreck a nice beach` present in the *printed* proposal
   set for `recognize speech`, via the executable. Record its rank and score.
   It must not regress; if it has, that outranks everything else in this item.
3. **Canonical case 2.** `Hits Justice Dupe Hid Came` for `It's just a stupid
   game`: record whether it appears in the printed set, and record the printed
   head so a later pass can diff against it.
4. **Fence audit of the tree.** Report the *actual* hits of
   `git grep -i -n -E "wreck|beach|recognize|justice|stupid|dupe|came|hid" --
   src` and classify each hit as exemplar list, `#[cfg(test)]`, doc comment, or
   real code. Extend the same sweep to `web/` and `examples/`. Also report any
   committed `zz*` file, any `eprintln!`/debug probe, any env-var knob, and
   any `#[ignore]` in the tree.
5. **Suite-honesty check.** Confirm no acceptance test has a relaxed assertion
   and no test is `#[ignore]`d, and that `tests/no_phrase_hard_coding.rs` is
   actually wired into the default test run.

## Fences

- **Read-only with respect to behaviour.** This item produces a report. Do not
  change scoring, search, ranking, or any assertion. Do not make a test pass.
  A test that is red is a *measurement* here, not a defect to fix.
- `CARGO_TARGET_DIR` must be outside `/tmp` (it is `noexec` and cargo build
  scripts fail with a misleading `Permission denied`).
- `git`, `grep`, `sed`, `date` need the `PATH` export from
  `docs/environment-notes.md`; a missing `git` is a truncated `PATH`, not a
  host outage. `cargo fmt` and `cargo clippy` **do not exist on this host** —
  do not report them as run.
- No `zz*` file, no `eprintln!` probe, no env-var knob, no phase timing.
  If you need a temporary probe to get a number, put it on a
  `scratch/*` branch in a *separate* worktree, and say so. Prefer
  `#[cfg(test)]` in `tests/`.
- Do not touch `post-milestone-acceptance` and do not touch `main`.
- Do not open an objective-axes, cost/pronunciation/dictionary/`GAP_COST`/
  retention, or emission front; all are refuted by measurement in
  [w-5c11a2](w-5c11a2.md)'s Context list.

## Completion criteria

- [ ] All five measurements above are recorded in this file's Handoff, with
      the exact commands run and the exact counts returned.
- [ ] The five-axis table is written as *measured*, with deviations from the
      expected baseline called out explicitly.
- [ ] The fence sweep is reported as raw hits plus a classification per hit.
- [ ] Either nothing actionable was found — in which case say so plainly and
      name the residual risk the next integration pass must still cover — or a
      finding is recorded with its exact site, and a *separate* follow-up work
      item is proposed (do not fix it here).
- [ ] `madgab-audit-d5a2c1` is pushed; the working tree is clean; `HEAD` is on
      that branch, not detached.
- [ ] This item is updated with objective state, validation, blockers and the
      next action, and pushed to `post-milestone-acceptance`. Never `main`.

## Handoff / notes

- Opened and claimed by coordinator `coord-0a11` at 2026-09-27T04:12Z on
  `post-milestone-acceptance` at `c28b29f`, in the pass that verified
  `c0ff01` alive and left it running. This front is **independent of**
  [w-5c11a2](w-5c11a2.md) by construction: different worktree, different
  branch, no shared file edits.
- Board protection for this worktree could **not** be registered this pass:
  `lubko-board create` returns `A Borys write capability is required`, and
  `antonina board resource add` requires an existing issue id. So unlike
  `/workspace/madgab-gap-recheck` (protected by open board issue 59), this
  worktree is **unprotected** against the host garbage collector. If it is
  gone, recreate it with
  `git worktree add -b madgab-audit-d5a2c1 /workspace/madgab-audit-d5a2c1
  post-milestone-acceptance`. The agent must push its branch before the end of
  its run for that to be safe.
- Baseline expectation copied from [w-5c11a2](w-5c11a2.md) as measured by
  `coord-5c11` at `f1943fd`: lib 58/0, corpus_integration 12/1, exact_determinism
  1/0, approx_determinism 4/0, emit_coverage 3/0, no_phrase_hard_coding 7/0.
  Treat those as a hypothesis to check, not as truth.
