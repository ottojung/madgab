---
work_item: true
id: w-paused-recon
state: working
priority: normal
owner: coord-7d3b
updated: 2026-09-28T05:58:00Z
branch: post-milestone-acceptance
worktree: /workspace/madgab
---

# Paused-programme reconciliation log

This is **not** a development queue entry. MadGab development is **paused** by a human
decision recorded in [../../accepted-state-2026-09-27.md](../../accepted-state-2026-09-27.md)
and `## Status: accepted and paused` in [../../skills/itinerary-madgab.md](../../skills/itinerary-madgab.md).
This document exists only so a recurring coordinator pass can find, in one place, what the
paused programme left behind and what must not be resumed without an explicit human
instruction.

## Standing rules for a scheduled pass while this document exists

1. **Create no new MadGab work items. Claim no superseded item. Launch no agent.**
2. **Resume no front**, including one that looks obviously unfinished or obviously valuable.
3. Accumulate durable state on `post-milestone-acceptance`. **Never push to `main`.**
4. The one genuinely useful recurring action is **at-risk state recovery**: finding work
   that exists only in a prunable worktree or only as an uncommitted diff, and making it
   durable. Passes `coord-a1c4`, `coord-b7f9` and `coord-c4d2` (below) each did exactly that
   and nothing else.
5. Prefer a dedicated `recovery/*` branch for archived scaffolding rather than adding
   scratch probes to the release-history branch.
6. **Recovery passes are not automatically complete.** An archive pass can be *partly*
   right: `coord-b7f9` verified its 29 files by walking dirty worktrees, but its enumeration
   rules silently skipped whole classes of file (see the `coord-c4d2` entry below). When
   checking a *new* recovery archive, verify by **basename and content hash against the
   live worktree**, not by "was this path archived at all" — two different programs in this
   repo share the basename `examples/zz_5e2d42_spans.rs`, and two different files share
   `src/probe.rs`.
7. **Hash-compare against blobs *and* against archived diffs.** A file archived as a
   `*.diff` patch has no blob of its own, so a pure content-hash sweep reports all eight
   instrumented `src/lib.rs` copies as "unarchived" even though
   `docs/work/probe-patches/` already carries them. Verify those by diffing the live
   worktree's `src/lib.rs` against its recorded patch before re-archiving, or a pass will
   spend its whole budget re-saving state that is already durable.

## Programme census at 2026-09-28T05:37Z (this pass)

* Work items: **87 `done`, 12 `superseded`, 0 `open`, 0 `blocked`, 0 `working`.** The only
  two `state: open` files in the tree are the two protocol *examples*
  (`docs/work/TEMPLATE.md` with placeholder id `w-000000`, and the example header inside
  `docs/skills/work-items.md` with placeholder id `w-a1b2c3`). Neither is a real task, and
  neither should ever be claimed. **The queue is genuinely empty; there is nothing to pick
  up, which is the expected state, not a defect to fix.**
* Antonina agents: **none alive.** Every agent in `antonina agent list` is terminal
  (`succeeded`, `failed`, or `stopped`). The two most recent, `3a8f01` and `3a8f02`, are
  `stopped` with `alive: no`; they were stopped by the pause, not by failure, and were
  deliberately left that way.
* Branches: ~120 local branches, most parked research history. Their work items are all
  closed. Do not treat branch count as a work queue.
* `main` vs `post-milestone-acceptance`: `origin/main` is `0267ade` (*Merge accepted
  MadGab approximate-search release state*), which is **not** an ancestor of
  `post-milestone-acceptance`; they diverged at `734e37e`. The single extra commit on
  `post-milestone-acceptance` is the `coord-a1c4` note `7be1922`. So the accepted release
  was merged to `main` by a human and the two refs have each moved one commit since. **Do
  not reconcile this by merging or pushing.** It is release history and is not this
  programme's business.

## Unintegrated, unvalidated work deliberately parked

These are the *only* substantive artifacts the paused programme left unintegrated. They are
candidates **only** if a human explicitly reopens development. None is known to be good;
none has been validated on this host.

| artifact | where | what it is |
|---|---|---|
| `9a1d189` + `7c97a97` | `madgab-diversity-3a8f01` (pushed) | `wording_reserve_slots` in `src/lib.rs` plus its 361-line P1-P4 window-reachability test. The test was untracked in no commit until `coord-a1c4` recovered it. |
| `a279cc8` | `madgab-poolrank-3a8f02` (pushed) | `w-3a8f02`'s `--pool-rank "<clue>"` CLI query form. **Default-output invariance was never proven**; a default-output change is a hard reject for that item regardless of feature quality. |
| `90d691e` | `scratch/c1d3a7-measure` (pushed) | The `ZZ_INJECT` tuple-injection measurement hook recovered by this pass. Scratch instrumentation, env-gated. |

## Archived measurement scaffolding (this pass)

`recovery/probe-scaffolding-2026-09-28` = `51ebdd1`, pushed, **not merged**:

* `docs/work/probes/` — 21 untracked probe sources (fronts `558697`, `1c3e77`, `5b1e93`,
  `0f3a17`, `5e2d42`, `8a1d47`, `c1d3a7`, `fillstrat`, …) with `SOURCES.tsv` mapping each
  archived copy back to its live worktree path.
* `docs/work/probe-patches/` — 8 unstaged `src/lib.rs` instrumentation diffs. Skipped where
  the identical blob already existed in history (`c3f81a` = `211a226`, `5e2d41` = `8db0eab`).

**Fence note, read this before ever promoting any of it.** Several archived probes *do*
contain canonical phrases (`recognize speech`, `wreck a nice beach`, the case-2 clue). They
sit under `docs/`, which `tests/no_phrase_hard_coding.rs` does not scan — that fence walks
`src/`, `web/` and `examples/` only — and `ALLOWLIST_CAPS` is unchanged. This is
documentation of past measurements, not production coupling. **If any of it is ever promoted,
its phrase literals must be removed as part of that promotion, not waived.**

## Second recovery pass (`coord-c4d2`)

`recovery/probe-scaffolding-2026-09-28` = **`3ce5262`**, pushed, not merged. Adds:

* `docs/work/probe-artifacts/` — four probe sources plus the two `prof/` markdown write-ups.
* `docs/work/probe-output/c1d3a7-m-head200.txt` — head of the 4.0 MB `ZZMETRICS` dump.
* `docs/work/probe-artifacts/README.md` — provenance per file, plus the standing fence note.

**The lesson from this pass is recorded as standing rule 6 above.** `51ebdd1` was not wrong
so much as *partly* right: it walked the dirty worktrees correctly but applied three
enumeration rules that each dropped files — skip a basename already archived, skip untracked
directories other than `examples/`, skip raw output rather than source. Four sources and two
write-ups survived all three rules. The two most interesting losses were **same-basename,
different-content**: `floor-5e2d42-probe`'s `zz_5e2d42_spans.rs` (`661d335e`) is a different
program from the archived base-side copy (`15f4a6c5`), and `probe-0f3a17`'s `src/probe.rs`
(`d78cc4c7`) is a different 71-line slot-recorder from the archived `madgab-axis-558697` copy
(`6549c937`). A future pass must hash-compare, not path-compare.

Deliberately **not** archived: `target-front-3a8f01/` (1.4 GB) and `target-front-3a8f02/`
(1.3 GB), which are Cargo `target/` directories from the two paused fronts, and the 58 MB
`prof/` binaries and `results-*.txt`/`sum-*.txt` (~2 MB), which the two kept markdown files
already summarise.

## Third recovery pass (`coord-7d3b`)

`recovery/probe-scaffolding-2026-09-28` = **`0a12e33`**, pushed, not merged. Swept all 21
worktrees and hashed every dirty and untracked file against every blob reachable in this
repository. 51 live files had no matching blob. Of those:

* 8 are the instrumented `src/lib.rs` copies **already durable as
  `docs/work/probe-patches/*.diff`** — a false positive, now standing rule 7;
* 46 are the `prof/` tree of `madgab-approx-runtime`, of which the two markdown write-ups
  were already archived and the rest was deliberately dropped. Two of them were a real gap:
  **`prof/run.sh` and `prof/summarize.py`**, the harness that produced every number in those
  write-ups, had never been archived, so the second pass's findings were not reproducible.
  Both are now at `0a12e33`, together with the 24-file `prof/baseline/` directory (92 KB)
  that the `scale.txt` baseline column rests on. Provenance in
  `docs/work/probe-artifacts/README.md`.
* `c1d3a7-instr/m.txt` (4.0 MB `ZZMETRICS` dump) had already been covered by the
  `probe-output/c1d3a7-m-head200.txt` head from `3ce5262` — verified, not re-archived.

**Remaining unarchived live state, all deliberate and all reproducible:** the two 30 MB
instrumented binaries `prof/madgab-baseline` and `prof/madgab-prof`, the ~2 MB
`results-*.txt`/`sum-*.txt` summaries (superseded by the archived markdown), and the
`target-front-3a8f01`/`3a8f02` Cargo directories (2.7 GB). Nothing at risk remains.

One loose end, unchanged and not actionable while paused: `prof/README.md` documents a real
`src/lib.rs` change in `prune_partials` (cache `metrics` instead of recomputing per
comparison) that exists in no branch and no commit. The harness that measured it is now
durable; the change itself still is not.

## The preserved limitation (do not re-litigate)

Approximate mode generates `wreck a nice beach` for `recognize speech`. It does **not**
generate `Hits Justice Dupe Hid Came` for `It's just a stupid game`; that regression is
`#[ignore]`d in `tests/corpus_integration.rs:134` and named in
`tests/cli_milestone_predicate.rs`. This was investigated deeply, priced negative on every
surface tried, and accepted. The one promising direction left is a qualitatively different
whole-path algorithm (compact pronunciation DAG with k-best / A*-style search, or a strong
backward suffix heuristic) — **not** another widening of the Cartesian-prefix traversal. A
pass that prices that direction must read `docs/accepted-state-2026-09-27.md` and the
`docs/work/REPORT-*.md` history first, and must not re-price any front that is already
recorded as a priced negative.

## Pass log

* **`coord-f81a`, 2026-09-28T04:26Z–04:35Z** — pause landed mid-flight. Made the two
  in-flight fronts durable, left both agents running at the time.
* **`coord-a1c4`, 2026-09-28T05:16Z–05:20Z** — reconciliation only. Recovered the untracked
  361-line test to `7c97a97`. Recorded in `w-3a8f01`.
* **`coord-b7f9`, 2026-09-28T05:26Z–05:38Z** — reconciliation only. Census as
  above; recovered the `ZZ_INJECT` hook to `90d691e` and the 29 at-risk scaffolding files to
  `51ebdd1`. No agent launched, no item claimed, nothing integrated, `main` untouched.
* **`coord-c4d2` (this pass), 2026-09-28T05:36Z–05:45Z** — reconciliation only. Re-ran the two
  prescribed checks: no live Antonina agent (`antonina agent list` is entirely terminal), and
  the dirty-worktree sweep **did** find unarchived state — the six files and one output head
  now at `3ce5262`. Census unchanged (87 done, 12 superseded, 0 open, 0 blocked, and the
  `w-paused-reconciliation` log itself as the only `working` entry). No agent launched, no
  front resumed, nothing integrated,   `main` untouched.
* **`coord-7d3b` (this pass), 2026-09-28T05:41Z–05:58Z** — reconciliation only. Both
  prescribed checks run again: `antonina agent list` is entirely terminal (the two paused
  fronts `3a8f01`/`3a8f02` are still `stopped`, deliberately left that way), and the
  hash-level worktree sweep **did** find a real gap — the `prof/` harness and baseline raw
  output, now at `0a12e33` on the recovery branch. Census unchanged (87 done, 12
  superseded, 0 open, 0 blocked; this log the only `working` entry). No agent launched, no
  front resumed, no item claimed, nothing integrated, `main` untouched.

## Next action for a fresh pass

Read `docs/accepted-state-2026-09-27.md`, then check only two things: `antonina agent list`
for anything alive, and every worktree's `git status --porcelain` for uncommitted `src/` or
untracked `examples/`/`tests/`/`src/` files **and untracked directories** not already
covered. Verify coverage by **content hash against the live file** — against blobs *and*
against the archived diffs (standing rules 6 and 7; two same-basename/different-content
pairs and eight diff-archived `src/lib.rs` copies have already tripped a naive check). If
both checks are clean, **there is no work to do** — confirm the pause, record nothing
further to avoid commit noise, and exit. Do not open a front.

