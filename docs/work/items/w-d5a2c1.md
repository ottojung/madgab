---
work_item: true
id: w-d5a2c1
state: done
priority: normal
owner: agent-d0f11f (measured, pushed 2026-09-27T04:21Z)
updated: 2026-09-27T04:21:00Z (all five axes MEASURED; findings filed as w-3a7f0d)
opened_by: coord-0a11 (reconciliation pass 2026-09-27T04:12Z, on post-milestone-acceptance at c28b29f)
branch: madgab-audit-d5a2c1
worktree: /workspace/madgab-audit-d5a2c1
agents: d0f11f (claim, live); d5a2c2, d5a2c3, d5a2c4 retired at exit 127
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

- [x] All five measurements above are recorded in this file's Handoff, with
      the exact commands run and the exact counts returned.
- [x] The five-axis table is written as *measured*, with deviations from the
      expected baseline called out explicitly.
- [x] The fence sweep is reported as raw hits plus a classification per hit.
- [x] Either nothing actionable was found — in which case say so plainly and
      name the residual risk the next integration pass must still cover — or a
      finding is recorded with its exact site, and a *separate* follow-up work
      item is proposed (do not fix it here).
- [x] `madgab-audit-d5a2c1` is pushed; the working tree is clean; `HEAD` is on
      that branch, not detached.
- [x] This item is updated with objective state, validation, blockers and the
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

## LAUNCHED 2026-09-27T04:21Z (coord-3a11): the exit-127 wall is GONE, and the
## "launch only when nothing is running" rule is retired

Two recorded operational rules are now falsified or superseded. Both matter,
because following either one would have idled this front indefinitely.

1. **The 127 wall is gone.** A throwaway probe on this worktree succeeded:

   ```text
   antonina agent new    --id d0f11e --cwd /workspace/madgab-audit-d5a2c1 -> idle
   antonina agent prompt --id d0f11e 'Reply with exactly: RUNTIME_OK'        -> RUNTIME_OK, exit 0
   antonina agent status --id d0f11e -> succeeded, 4.8s
   ```

   So the `d5a2c2` / `d5a2c3` / `d5a2c4` failures at 04:12Z were a transient
   spawn-time host fault, not a standing condition, and not caused by `c0ff01`
   holding the runtime — the "one concurrent agent" story is already falsified
   elsewhere in this file and should not be repeated.
2. **Parallel fronts work.** `c0ff01` (w-5c11a2) was `running` + `alive: yes`
   the whole time, and `d0f11f` was created and prompted on *this* worktree
   in the same minute. Both are now running concurrently on disjoint worktrees,
   branches and files. `antonina agent prompt` returns promptly once the agent
   process is spawned, so launching a front does not have to block the
   coordinator.

Accordingly the "launch only when no `madgab-*` agent is `running`" next action
recorded above is **superseded**. The correct rule is: probe cheaply, and on
success launch — do not serialise fronts behind a live one. Note the probe
discipline is still sound for the *purpose it was recorded for*: a 127 does not
justify re-claiming or abandoning a `working` item whose named agent is
`running` + `alive: yes`. That half of the rule is unchanged and remains
correct.

Agent `d0f11f` was briefed to `git reset --hard origin/post-milestone-acceptance`
first (the worktree forked at `c28b29f` and would otherwise have measured a
stale tree), to claim this item and push the claim, to run the six cargo suites
and record exact counts, to measure both canonical cases through the executable,
to do the full fence sweep with a classification per hit, and to write a
five-axis **measured** table into this Handoff. It is told to push
`madgab-audit-d5a2c1`, to keep `HEAD` on that branch, to propose a separate
follow-up item rather than fixing anything it finds, and not to wait on
`c0ff01`. **Left running for a later fresh pass to inspect.**

### Measurement base vs. integration moves after it

The tree measured here is `post-milestone-acceptance` at **`42ced98`**, which is
what this worktree was reset to at the start of the run and what the six
release suites, both canonical cases and every fence sweep were executed
against. The accumulation tip has since moved to `1f72b59` ("integrate
w-5c11a2: price the canonical case-2 objective gap"), which landed **after**
these numbers were taken. So every figure in the table above describes
`42ced98` only, and none of it carries forward across that integration. A pass
integrating `1f72b59` must re-run axis 1 and axis 2 before accepting it; the
fence sweeps in axis 4 and the suite-honesty checks in axis 5 are structural
and are much less likely to move, but they are cheap so they should be re-run
too.

## MEASURED 2026-09-27T04:21Z (agent-d0f11f): all five axes, on the accumulation tip

**Setup actually performed** (this item's own launch instructions, run first):

```sh
export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"
git fetch origin && git checkout madgab-audit-d5a2c1
git reset --hard origin/post-milestone-acceptance   # -> 42ced98
git status -sb                                      # ## madgab-audit-d5a2c1, clean
export CARGO_TARGET_DIR=/workspace/cargo-target-d5a2c1   # NOT /tmp: /tmp is noexec
```

Claim commit `0ff4ed3` pushed to `madgab-audit-d5a2c1` and to
`post-milestone-acceptance` (fast-forward `42ced98..0ff4ed3`). `main` was
never touched; no `main` ref exists on this host.

**Build note:** the first `cargo test` ran `Updating crates.io index` /
`Locking 23 packages`, but `git status` after the run was clean — the
re-resolution did not modify the committed `Cargo.lock`.

### The five-axis table, written as MEASURED

| # | Axis | Command run | EXPECTED baseline (a hypothesis) | MEASURED | Deviation |
|---|---|---|---|---|---|
| 1a | lib tests | `cargo test --release --lib` | 58 / 0 | **58 passed; 0 failed; 0 ignored** (exit 0) | none |
| 1b | corpus integration | `cargo test --release --test corpus_integration` | 12 / 1 | **12 passed; 1 failed; 0 ignored** (exit 101) | none |
| 1c | exact determinism | `cargo test --release --test exact_determinism` | 1 / 0 | **1 passed; 0 failed; 0 ignored** (exit 0) | none |
| 1d | approx determinism | `cargo test --release --test approx_determinism` | 4 / 0 | **4 passed; 0 failed; 0 ignored** (exit 0) | none |
| 1e | emit coverage | `cargo test --release --test emit_coverage` | 3 / 0 | **3 passed; 0 failed; 0 ignored** (exit 0) | none |
| 1f | no phrase hard coding | `cargo test --release --test no_phrase_hard_coding` | 7 / 0 | **7 passed; 0 failed; 0 ignored** (exit 0) | none |
| 2 | canonical case 1 | `madgab --approximate --top 50 "recognize speech"` | present | **PRESENT, rank 28, printed score 0.918** — line `28. [0.918] wreck a nice beach` | **none — no regression** |
| 3 | canonical case 2 | `madgab --approximate --top 50 "It's just a stupid game"` | record head | **`hits justice dupe hid came` ABSENT from all 50** | consistent with 1b; head recorded below |
| 4 | fence sweep | `git grep -i -n -E "wreck\|beach\|recognize\|justice\|stupid\|dupe\|came\|hid" -- src web examples` | src clean; web/index.html + examples/measure.rs benign | **confirmed, with two extra benign files classified** | see classification below |
| 5 | suite honesty | grep sweeps + `Cargo.toml` | no `#[ignore]`, no relaxed assertion, test wired in | **confirmed; no `[[test]]` sections at all, so all 5 test files are auto-discovered** | none |

**Explicit deviation call-out, axis 1: NONE.** Every one of the six counts
matched the expected baseline exactly, and no milestone test that the baseline
expects to be red has silently started passing. The single failure is the
expected one, by name:

```
test approximate_finds_classic_madgab_resegmentation ... FAILED
  panicked at tests/corpus_integration.rs:136:5:
  canonical clue missing from top 50; got: ["it said thus 'cause too day",
  "it said thus test oop day", "it said thus death too day", ...]
```

and its sibling `approximate_finds_recognize_speech_resegmentation ... ok`.

**Explicit deviation call-out, axis 3: the CLI quirk recorded in this item does
NOT reproduce.** The item states that `"It's ..."` cannot be passed as a single
argv word and that the exit-1 spellings are not search failures. Measured, with
each spelling quoted so the shell passes exactly one word:

```sh
./madgab --approximate --top 50 "It's just a stupid game"   # exit 0
./madgab --approximate --top 50 "Its just a stupid game"    # exit 0
./madgab --approximate --top 50 "It is just a stupid game"  # exit 0
```

The apostrophe form exits **0**, and its 50 printed lines are `diff`-identical
to the `Its` form. The `It is` form is a genuinely different target (one extra
word) and a different head, as expected. So the reproducible fact is the
narrower one: **`It's` and `Its` are interchangeable here and print the same
set.** Filed for correction in [w-3a7f0d](w-3a7f0d.md). This is a stale
documentation claim, not a code defect, and it did not affect any number above.

Note also that the CLI prints scores at **3 decimals** while
`approximate_output_is_locked` in `tests/corpus_integration.rs` locks them at
**6**. The two are not comparable at face value; use the test for exact
figures and the CLI for printed-set membership and rank.

### Axis 3 — the recorded printed head (diffable by a later pass)

`madgab --approximate --top 50 "It's just a stupid game"`, top 12 of 50
(`hits justice dupe hid came` does not appear at any of the 50 ranks):

```text
 1. [0.924] it said thus 'cause too day
 2. [0.924] it said thus test oop day
 3. [0.923] it said thus death too day
 4. [0.923] it said thus tough too day
 5. [0.922] it said thus 'cause too gave
 6. [0.921] each thus 'cause too bad aim
 7. [0.921] it said thus 'cause too dame
 8. [0.921] it said thus test oop gave
 9. [0.921] it said thus death too gave
10. [0.921] each thus 'cause too bad same
11. [0.921] it said thus tess too day
12. [0.921] it said thus 'cause too games
```

This matches the first 12 strings of the `approximate_finds_classic_madgab_
resegmentation` failure message, as it must — same search, same `top_n: 50`.

### Axis 4 — fence sweep, every hit classified

`src/` — 30 hits, all accounted for, no real code in the search path:

| site | classification |
|---|---|
| `src/lexical.rs:19` | **doc comment** (module doc contrasting `beach` against a determiner) |
| `src/lexical.rs:302,303,335` | **`#[cfg(test)]` exemplar list** — all inside `mod tests` opened at `src/lexical.rs:260`, in `fn rejects_ordinary_content_words`; these are *negative* assertions (`assert!(!is_closed_class(word))`) |
| `src/lib.rs:3072` | **real code** — `expect("key came from cells")`; ordinary English `came` inside a panic message, not a phrase |
| `src/lib.rs:3837` | **comment** — quotes `"recognize speech"` while explaining a split |
| `src/lib.rs:3930` | **real code** — `expect("candidate came from this pool")`; same, `came` in prose |
| `src/lib.rs:4125,4129,4130,4133,4137` | **`#[cfg(test)]`** (inside `mod tests` at `src/lib.rs:3720`) — corpus-rarity contrast between `a` and `beach` |
| `src/lib.rs:4179,4180,4183,4184` | **`#[cfg(test)]`** — `phrase_signature` spelling-normalisation cases |
| `src/lib.rs:4227,4255,4256,4305,4326` | **`#[cfg(test)]`** — `TargetPhrase` reuse cases |
| `src/lib.rs:5017` | **doc comment** — matches on `came` inside `became` |
| `src/lib.rs:5171,5173,5174,5179,5181,5182` | **`#[cfg(test)]`** — `reachability_corpus()`, a deliberately exhaustive alignment corpus, not a search table |
| `src/lib.rs:5238,5240,5241,5245` | **`#[cfg(test)]`** — stem/reuse cases |
| `src/lib.rs:5559,5561,5562` | **`#[cfg(test)]`** — reachability assertions |
| `src/main.rs:9,11` | **doc comment** — the CLI's own `--help`/usage examples |

So `src` is clean in the stronger sense too: no phrase literal in `src`
participates in a *decision*. Every hit is a test fixture, a doc comment, or
prose inside a panic message.

`web/` — 9 hits, none in the search path:

| site | classification | in the search path? |
|---|---|---|
| `web/app.js:21,26,64,68,95` | **real code** — the DOM property `hidden`; substring match on `hid`, not a phrase hit | yes, `app.js` *is* a search entry point, but these five lines are show/hide of the results and error elements |
| `web/index.html:20` | **UI default input value** `value="It's just a stupid game"` | no — a form default, not a hint read by the search |
| `web/index.html:21` | **UI placeholder** | no |
| `web/index.html:42,47` | **real code** — the `hidden` attribute on `<section id="results">` and `<p id="error">` | no |

`examples/` — 3 hits, none in the search path:

| site | classification | in the search path? |
|---|---|---|
| `examples/measure.rs:16` | **doc comment** — explains that a content-word win must not hide a loss of acoustic similarity | no |
| `examples/measure.rs:31,32` | **benchmark harness** — the two canonical targets in a measurement example's input list | no; `measure.rs` is a separate binary, not a library input |

Other fence sweeps, all clean:

```sh
git ls-files | grep -E '(^|/)zz'      # -> no output: no committed zz* file anywhere
git grep -n '#\[ignore' -- src tests  # -> no output
git grep -n -E 'eprintln!|dbg!' -- src
  # src/main.rs:117,125,138,151,161,162,170,171,172,173 — all the CLI's own
  # user-facing error and progress output. No dbg! anywhere. No probe.
git grep -n -E 'env::var' -- src     # -> no output: no env-var knob in production code
git grep -n -E 'option_env!|std::env' -- src
  # only src/main.rs:49, `std::env::args()` for argv
```

**One thing that differs from the earlier git-only baseline, in `tests/` and
harmless:** `git grep -n 'env::var' -- tests` gives
`tests/approx_determinism.rs:261`, reading `POOL_HELPER_ENV` =
`"APPROX_POOL_HELPER_TARGET"` (declared at line 247, set by a child driver at
line 301). `fn approximate_pool_helper` returns immediately unless that
variable is set, so the default run costs nothing and is unaffected — and
1d above is 4/0 with it unset. It is an env-var *knob*, but it lives in
`tests/`, not `src/`, and it is the mechanism a later pass uses to take a
fresh-process pool dump. Recorded in [w-3a7f0d](w-3a7f0d.md) explicitly so a
later sweep does not re-open it as new.

### Axis 5 — suite honesty

- **No `#[ignore]`**: `git grep -n '#\[ignore' -- src tests` returns nothing,
  and every one of the six `test result:` lines above reports `0 ignored`.
- **No relaxed assertions**: `git grep` finds no `assert!(true`, no
  `assert!(1 == 1`, no `should_panic` in `src` or `tests`; no `todo!` or
  `unimplemented!`; no `#![allow(...)]` or `#[allow(...)]` anywhere; no
  `#[cfg(feature = ...)]` or `#[cfg(not(...))]`; `Cargo.toml` has **no
  `[features]` table at all**, so there is no flag that could gate a test out.
- The two `return;` early-exits inside tests are both benign on inspection:
  `tests/no_phrase_hard_coding.rs:843` is a stylistic "nothing to report, skip
  the report-building" before a `panic!`, and
  `tests/approx_determinism.rs:262` is the helper's documented no-op unless its
  env var is set. Neither can turn a failing assertion into a pass.
- **`tests/no_phrase_hard_coding.rs` is wired into the default run.** There are
  **no `[[test]]` sections in `Cargo.toml`**, so all five files in `tests/` are
  picked up by cargo's default `tests/*.rs` auto-discovery, and the explicit
  run in 1f is the same binary the default `cargo test` uses. Confirmed by 1f
  reporting 7 passed.
- The lock test `approximate_output_is_locked`
  (`tests/corpus_integration.rs:555-614`) asserts an exact 10-row,
  6-decimal table with `assert_eq!` and carries a long comment explaining why
  the current values are the correct ones. It is not relaxed, and it passes
  (see 1b's ok list).
- **Scope limit found, and it is the one actionable thing here**:
  `tests/no_phrase_hard_coding.rs:806-816` (`fn src_files`) walks
  `src_dir()` only. A phrase-specific hard-code under `web/` or `examples/`
  would be invisible to it. Today those hits are all benign (table in axis 4),
  so the *tree* is clean; it is the *fence* that has a narrower view than its
  test name suggests. Filed as [w-3a7f0d](w-3a7f0d.md); not fixed here.

### What a later integration pass must still cover

This item is a **report**. Nothing in it was changed to make anything pass.
Named residual risks, in priority order:

1. **Canonical case 2 is genuinely unfixed.** `hits justice dupe hid came` is
   absent from all 50 printed proposals for `It's just a stupid game`, and
   `approximate_finds_classic_madgab_resegmentation` is red on that account. The
   approximate search is currently returning a dense cluster of
   `it said thus 'cause too ...` near-homophones at the top instead. This is
   the one open functional regression, it is measured here rather than fixed
   here, and any integration pass accepting a search/ranking change must show
   this number moving.
2. **Canonical case 1 is intact but not comfortable**: `wreck a nice beach`
   sits at rank 28 of 50, score 0.918, behind ~27 same-family resegmentations
   (`wreck a nice peach`, `wreck a nice pitch`, `wreck egg nice peach`, ...).
   It is in the printed set, so this is **not** a regression, but the margin is
   thin enough that a small scoring change could drop it without turning any
   test red except `approximate_finds_recognize_speech_resegmentation`. Treat
   rank as a tracked number, not a boolean.
3. **`no_phrase_hard_coding` does not see `web/` or `examples/`** — see axis 5
   and [w-3a7f0d](w-3a7f0d.md).
4. **`cargo fmt` and `cargo clippy` do not exist on this host** (no `rustfmt`
   or `clippy` component, no `rustup`), and neither is reported as run. A
   work item whose completion criteria require `cargo fmt --check` or
   `cargo clippy --all-targets -- -D warnings` cannot be marked `done` here; the
   correct substitute is the six release test runs in axis 1, all of which are
   recorded above with exact counts.
5. This audit ran on `post-milestone-acceptance` at `42ced98` only. Any commit
   landed after that is **unmeasured** by this item, including anything the
   concurrently running `c0ff01` produces. Re-run axis 1 and axis 2 after any
   integration; do not carry these numbers forward across a rebase.
