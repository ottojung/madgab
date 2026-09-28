---
work_item: true
id: w-8f0b3d
state: working
priority: high
owner: front agent-8f0b3d1 (running, 1 prompt, deliberately NOT steered 2026-09-28T00:28Z by pass coord-5e1f - healthy, only 3m old, and already producing the executable-boundary numbers the milestone predicate needs) / opened and claimed 2026-09-28T00:26Z by coord-c4d2
updated: 2026-09-28T00:34:00Z
branch: madgab-cli-recheck-8f0b3d
worktree: /workspace/madgab-cli-recheck-8f0b3d
base: 97c9397 (post-milestone-acceptance, pushed)
umbrella: w-4b1e07
source_item: w-3c5b18
---

# Re-measure the milestone predicate at the executable boundary (no src change)

## Goal

Every load-bearing number in this repository's case-2 record comes from `#[cfg(test)]` harnesses,
`examples/zz-probe-*.rs` files and library-level captures. The itinerary's primary milestone is
stated about **the executable's approximate proposal set**, and its completion checks require the
behavior to be "covered by regression tests at an appropriate API or executable boundary". That
boundary has never been measured directly. This item closes that gap with an objective,
reproducible, non-instrumented fact about the shipped CLI.

It is deliberately **not** another case-2 reach front. `docs/work/OBSTRUCTION-MAP.md` section 3
prices every remaining search-side shape negative and states "case-2 reach is closed as a search-side
question"; `w-9b4a15`/`w-3a8c05` price a weight-free lower-bound rank of 1,127 for the canonical
tuple. What is *not* on the record is what the shipped binary actually does, unmodified, on the
two canonical inputs.

## Scope

Read-only measurement plus tests. **No production `src/` change is in scope for this item** and none
may be made: `agent-3c5b18` holds `src/lib.rs` and any coordinator-side edit here would contend
with it.

1. On integrated `post-milestone-acceptance`, build the shipped release binary and run it with
   `--approximate` on both canonical inputs, exactly as a user would:
   - `recognize speech` -> is `wreck a nice beach` in the printed proposals? At what display
     position, and what is the pool rank if the binary reports it?
   - `It's just a stupid game` -> is `Hits Justice Dupe Hid Came` in the printed proposals under
     *any* case/punctuation/word-order normalization, not just verbatim? Record verbatim first,
     then normalized.
2. Record the **sensitivity** of the predicate to the documented public knobs only
   (`--per-word-budget`, `--total-budget`, `--top-n`, any seed/determinism flag). This is the
   question a later general-search-quality front actually needs: what the current *reachable*
   envelope of the shipped CLI looks like, without touching the search.
3. Establish whether a **cheap, general, non-re-pinning regression test can be written at an
   executable or public-API boundary** for whatever the answer is — including for the currently
   failing canonical case, if and only if the test asserts the *documented current* behavior and
   names it as a known gap rather than asserting success. If no such test is honest, say so and
   write none.
4. Report the wall clock and the exact commands, so a later pass can reproduce without re-deriving.

## Constraints

- No phrase-specific hard-coding in any production code or in any test assertion. The
  `no_phrase_hard_coding` fence over `src/` must stay clean; tests that name the canonical strings
  live in the existing canonical-example suites and are already the convention, so follow it
  rather than inventing a new pattern.
- Do **not** edit `src/lib.rs`, `src/approx.rs` or `src/main.rs` on this front. If the measurement
  shows a defect in the CLI's argument handling, report it; do not fix it here.
- Do not re-run any priced negative from `OBSTRUCTION-MAP.md` section 2, and do not open a fifth
  reach front. This front measures; it does not chase.
- `corpus_integration` is expected at 12 passed / 1 failed, the known base red
  `approximate_finds_classic_madgab_resegmentation`. Run it with `-- --test-threads=2`; it is
  SIGKILLed at default parallelism on this host, including on pristine base.
- `cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run on this host**
  (no `rustup`); do not claim them (`docs/environment-notes.md`).
- Run **one** test suite at a time. Parallel suite runs have SIGKILLed this host for memory
  pressure, not for a source reason.
- `main` is untouched. This accumulates on `post-milestone-acceptance` only.

## Completion criteria

1. Both canonical inputs measured through the **shipped release binary** on integrated HEAD, with
   exact commands, exit status, printed output, wall clock, and the verbatim-vs-normalized answer
   for case 2 recorded either way.
2. The documented public-knob sensitivity of the predicate recorded as a small table (knob, value,
   case-1 verdict, case-2 verdict), with the cost (wall clock) of each setting.
3. Either a general regression test at an executable/public-API boundary is added with its
   **red/green evidence and fence results**, or a written statement of why no honest test exists
   at that boundary. No test is written that hard-codes a phrase into production behaviour.
4. `docs/work/REPORT-8f0b3d.md` pushed on the front branch, with an explicit statement of what the
   milestone predicate *actually* is today at the executable boundary, and an explicit INTEGRATE/HOLD
   recommendation for whatever the front changed (possibly nothing, if the measurement is
   docs-only).
5. `no_phrase_hard_coding` 9/9, `corpus_integration` 12/1 with the known red **not re-pinned**,
   `emit_coverage` green, both determinism suites green, `cargo test --release --lib` no worse than
   76/0/12, and `wreck a nice beach` still produced for `recognize speech` at or better than
   display rank 27. Anything worse is a regression and blocks the front.
6. This work item updated with objective state and the next action. No self-merge.

## Handoff

Front `agent-8f0b3d1` launched 2026-09-28T00:26Z by pass `coord-c4d2` in
`/workspace/madgab-cli-recheck-8f0b3d` on branch `madgab-cli-recheck-8f0b3d`, created from
`post-milestone-acceptance` at `97c9397`. Left RUNNING for a later fresh pass to inspect.

**Why this front is independent of the live one and why it was opened now rather than after.**
The live front `agent-3c5b18` (`w-3c5b18`, the `prune_partials` discard threshold) holds
`src/lib.rs`. This front holds no production file at all, runs against the *shipped* binary, and its
answer is a property of the release artifact rather than of any in-flight diff. Running them
concurrently is the parallelism the schedule asks for, and it does not duplicate: the live front
prices a threshold inside the search, this one prices the predicate the itinerary's completion
checks are written against. Two prior passes (00:06Z, 00:16Z) both declined to open any front and
recorded this item as the successor to open *on a priced negative* from the live front; opening it
now is strictly better, because its result is needed either way and it cannot come back negative in
a way that wastes the front.

**Next action for a fresh pass.** `antonina agent status --id 8f0b3d1`, then
`git -C /workspace/madgab-cli-recheck-8f0b3d log --oneline -3` and `status --short`. The
precondition for review is a **pushed** commit (`git ls-remote origin madgab-cli-recheck-8f0b3d`).
On terminal, review `docs/work/REPORT-8f0b3d.md`, verify line by line with
`git diff --stat <parent> <new> -- src tests examples` that no production line landed, then
integrate onto `post-milestone-acceptance` only, never `main`. Independently, keep `w-3c5b18`'s own
review path: generality first, then `no_phrase_hard_coding` 9/9, `corpus_integration` 12/1 with the
known red not re-pinned, `--lib` no worse than 76/0/12, and `wreck a nice beach` at or better than
display rank 27.

### Reconciliation pass coord-5e1f (2026-09-28T00:27Z-00:34Z): front healthy and already producing the number the milestone needs

Pass verdict: this front is the right one, it is running, and its in-flight measurements are
already the most useful durable information in the queue. Left RUNNING, not steered — it was
prompted 3m ago and needs no redirect. No integration candidate. `main` untouched.

**State at inspection.** `agent-8f0b3d1`: `running`, alive, 1 prompt, 3m, worktree
`/workspace/madgab-cli-recheck-8f0b3d` **clean** on branch `madgab-cli-recheck-8f0b3d`, which is
pushed at `7eee678` (the item's own commit — it has not yet added one). It is using the shipped
release binary as intended and has produced results no earlier front could, because every
load-bearing number on this branch so far came from a `#[cfg(test)]` harness, an
`examples/zz-probe-*.rs` file or a library capture.

**In-flight measurements, provisional until the report is pushed.** Against
`target/release/madgab` on `post-milestone-acceptance` at `97c9397`:

| input | knob | result |
| --- | --- | --- |
| `recognize speech` | default | `wreck a nice beach` at display rank 27 of 50, score 0.920 |
| `recognize speech` | `--top` 1, 10, 25, 50, 100, 200 | absent |
| `recognize speech` | `--top` 500, 1000 | present at display rank 27 |
| `It's just a stupid game` | `--top` 1 .. 1000 | `hits justice dupe hid came` **absent at every value** |

Search cost is 1.04s for case 1 and 1.20s for case 2 from a 513ms/538ms corpus load, so the whole
sweep runs in seconds and this front can afford the full documented-knob envelope.

**Why this changes the queue's reasoning, and why the front must still finish.** The `--top` sweep
retires knob-shaped explanations for case 2 that the library-side fronts could not: at the
executable boundary the clue is not hidden behind `top_n`, an emission ceiling or a ranking cut at
any width up to 1000. Combined with `w-2f7a10`'s and `w-3a8c05`'s lower-bound results, the case-2
blocker should now be treated as a **reaching** problem, not a ranking or budget problem — the
canonical alignment is not produced by this pipeline for this input under any of its public knobs.
That is a materially different statement than the one the last four passes could make, and it is the
input any later general search-quality front should be judged against. It is still provisional: an
in-flight log is not a report, and the default-knob case-1 rank of 27 needs reconciling against the
"display 26" figure that older notes carry.

**Next action for a fresh pass.** `antonina agent status --id 8f0b3d1`. The precondition for review
is unchanged and is a **pushed** commit — `git ls-remote origin madgab-cli-recheck-8f0b3d`, not a
local `origin/madgab-*` ref, because this host's fetch refspec only tracks
`post-milestone-acceptance`. On terminal, review `docs/work/REPORT-8f0b3d.md`, verify with
`git diff --stat <parent> <new> -- src tests examples` that **no production line landed** (this
front's own criteria forbid editing `src/lib.rs`, `src/approx.rs` and `src/main.rs`), confirm the
`--top` table is reproduced there and not only in the log, and check whether the front settled the
26-vs-27 question. Integrate docs-only onto `post-milestone-acceptance`, never `main`. Then hand
the reconciled case-2 statement back to [w-4b1e07](w-4b1e07.md) as the objective current form of
the milestone predicate, replacing the case-2 reach attempts this queue has now exhausted.
