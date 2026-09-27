---
work_item: true
id: w-d5a2c1
state: open
priority: normal
owner: null (claim WITHDRAWN by coord-0a11 at 2026-09-27T04:18Z — see Handoff)
updated: 2026-09-27T04:18:00Z (re-verified unlaunchable by coord-1f2b at 04:16Z — deliberately still unowned)
opened_by: coord-0a11 (reconciliation pass 2026-09-27T04:12Z, on post-milestone-acceptance at c28b29f)
branch: madgab-audit-d5a2c1
worktree: /workspace/madgab-audit-d5a2c1
agents: d5a2c2, d5a2c3, d5a2c4 — all exit 127 at spawn, see Handoff
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

## LAUNCH FAILED 2026-09-27T04:12–04:18Z (coord-0a11): back to `open`, unowned

The worktree and branch exist and are clean. The agent never ran a turn. Three
attempts, all dead at spawn:

| agent | cwd | result |
|---|---|---|
| `d5a2c2` | `/workspace/madgab-audit-d5a2c1` | exit **127** in 0.08s, `"OpenCode process had no pid"`, empty `output.log` |
| `d5a2c3` | `/workspace/madgab-audit-d5a2c1` | exit **127** in 0.08s (bare `RUNTIME_OK` probe) |
| `d5a2c4` | `/workspace/madgab` | exit **127** in 0.22s (bare `RUNTIME_OK` probe) |

`antonina agent new` succeeded every time; `agent prompt` is what dies, before
the prompt text is even read. This is byte-for-byte the same failure shape as
the retired `5c11a2` / `5c11a3` pair, and it is **not** worktree-specific
(`d5a2c4` used the main worktree and failed identically).

### IMPORTANT for the next pass: the throwaway probe is UNRELIABLE while another agent is running

At the same moment, `c0ff01` (holding [w-5c11a2](w-5c11a2.md)) was
`state: running`, `alive: yes`, and `ps` showed `openclaw-gateway` pid 78 up
since 03:58Z. The gateway was healthy and *one* agent was running fine, while
three *new* spawns in two different worktrees all died at 127.

The cheapest explanation consistent with every observation of this pass is a
**single-concurrent-agent limit on this host's runtime**: an existing
`running` agent holds the runtime, and a fresh `agent prompt` cannot get a pid.
That is a **hypothesis, not a proven fact** — this pass could not test it,
because testing it requires an idle runtime.

This matters because [w-5c11a2](w-5c11a2.md) tells the next pass that the cheap,
correct liveness check is "`ps` the gateway, then run a throwaway
`antonina agent prompt`". **Half of that advice is unsafe as written.** A
throwaway probe returning exit 127 while a front is live is *not* evidence that
the runtime is down, and *not* evidence that a `working` item is abandoned.
Taking it at face value here would have abandoned a perfectly healthy front and
re-queued work that was running fine.

**Corrected liveness check, for the next pass:**

1. `antonina agent list`, then the named agent's own `state` / `alive`.
   `running` + `alive: yes` is decisive on its own. Trust that alone.
2. Run a throwaway probe **only** if the named agent is `failed` or gone — and
   even then only after confirming no other agent is `running`.
3. `ps` for `openclaw-gateway` remains a useful necessary condition, but it is
   not sufficient, and it is not what failed here.

**Next action, concrete:** this item is `open`, owned by nobody, branch and
worktree ready. Do not open a new work item for it and do not redo its work.
When `antonina agent list` shows **no** `running` madgab agent, fetch
`post-milestone-acceptance`, re-verify the branch base, then
`antonina agent new --id <fresh-hex> --cwd /workspace/madgab-audit-d5a2c1`
followed by the full prompt from this item. The five measurements have **not**
been done.

Board protection for this worktree could **not** be registered:
`lubko-board create` answers `A Borys write capability is required`, and
`antonina board resource add` wants `ISSUE HOST PATH` with an existing issue
id. So unlike `/workspace/madgab-gap-recheck` (protected by open board issue
59), this worktree is **unprotected** against the host garbage collector. If it
has been removed, recreate with `git worktree add -b madgab-audit-d5a2c1
/workspace/madgab-audit-d5a2c1 post-milestone-acceptance`.

## Fence baseline measured directly by coord-0a11 (git-only, no cargo)

Cheap deterministic sweeps run in `/workspace/madgab` at `c250899`, so the
agent does not repeat them. All clean:

- `git ls-files | grep -E '(^|/)zz'` — **no committed `zz*` file anywhere**.
- `git grep -n '#\[ignore' -- src tests` — **no hits**.
- `git grep -E 'eprintln!|dbg!' -- src` — hits only at `src/main.rs:117-173`,
  which are the CLI's own user-facing error and progress output. Not probes.
- `git grep -E 'env::var' -- src` — **no hits**; no env-var knob.
- phrase sweep over `web/` and `examples/` — hits only in `examples/measure.rs`
  lines 31-32 (the benchmark harness, which is what a measurement harness is
  for) and `web/index.html` lines 20-21 (the UI's default target input value
  and placeholder). Neither is in the search path.
- phrase sweep over `src/` — hits confined to `src/lexical.rs` exemplar lists
  and `src/lib.rs` `#[cfg(test)]` modules, plus incidental non-example uses of
  the ordinary English words `came` and `beach` in code and comments. Classified
  by hand; the audit agent should re-confirm rather than re-derive.

**Not measured, and that is the whole point of this item:** the six test-suite
counts, canonical case 1's printed rank, and canonical case 2's printed head.
All need a release cargo run and none was attempted this pass.

## Re-verification 2026-09-27T04:12Z–04:16Z (coord-1f2b): still correctly
## unowned, and the "one concurrent agent" explanation is now falsified

`madgab-audit-d5a2c1` is still at `c28b29f` with a clean tree and no commits of
its own, and this front is still `open` and unowned. That is the right state,
not an oversight:

- `c0ff01`, the live milestone front on [w-5c11a2](w-5c11a2.md), is
  `state: running` / `alive: yes` (pid 2061). The standing rule recorded in
  [w-5c11a2](w-5c11a2.md) is not to run a throwaway spawn probe while a front
  of this repository is alive, because a 127 from such a probe does not mean
  the runtime is down. **No probe was run this pass and none should be.**
- `d5a2c2`, `d5a2c3`, `d5a2c4` are still `failed` / `exit 127` /
  `"OpenCode process had no pid"` with 0-byte `output.log`. They died at
  spawn and are retired, not pending. Do not restart them and do not wait on
  them.
- **New fact, and it corrects the prior note:** the "this host allows only one
  concurrent agent" hypothesis is **falsified**. `antonina agent list` shows
  `49c001` and `47b001` running at the same time in unrelated repositories.
  The host is not single-slot. So the 127s are their own spawn-time host bug
  and are not caused by `c0ff01` occupying a slot. The operational rule is
  unchanged; the causal story is wrong and should not be repeated.

This front is also the natural place to file that host bug: "a fresh
`antonina agent new` + `antonina agent prompt` returns
`OpenCode process had no pid` / exit 127 when *no* agent of this repository is
running" is a runtime defect in the Antonina agent host, and it is separately
worth a work item once a spawn has been observed to fail with nothing else
running.

**Partially satisfied without an agent, this pass** (the git-only half of this
item, measured on `post-milestone-acceptance` at `6528282`):

```text
git grep -i -n -E "wreck|beach|recognize|justice|stupid|dupe|came|hid" -- src
-> 30 hits, all in src/lexical.rs:302-335 exemplar lists or inside
   #[cfg(test)] modules in src/lib.rs (lines 4129+)
```

So the no-phrase-specific-hard-coding fence is **clean on the accumulation
tip**. What remains outstanding for this item is the cargo-dependent half:
executing the acceptance test set and checking that no test asserts a
phrase-specific exception in a way `tests/no_phrase_hard_coding.rs` cannot see.

**Next action:** launch on this worktree when no `madgab-*` agent is
`running`. Until then this item stays `open` and unowned, and the live
milestone front on [w-5c11a2](w-5c11a2.md) keeps the single repository slot.
