---
work_item: true
id: w-paused-recon
state: working
priority: normal
owner: coord-9a3c
updated: 2026-09-28T06:17:00Z
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
   spend its whole budget re-saving state that is already durable. The reliable test is
   `git apply --check --reverse <archived.diff>` run *inside* the live worktree, reading
   the diff out with `git show <recovery-branch>:<path>` — the recovery branch is not
   checked out in `/workspace/madgab`, so a bare path fails and looks like a real gap.
8. **Archived a *harness* is not the same as archived a *reproducible* harness.** A pass
   that archives scripts must also archive the inputs, data files and phrase lists they
   read, and must check that it did. `coord-7d3b` archived `run.sh` and `summarize.py`
   without noticing that `run.sh` ends in `done < prof/targets.txt`; the harness was
   therefore inert. Archiving the script is not evidence that the measurement can be
   repeated — grep the archived script for every path it opens and check each one.

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

## Fourth recovery pass (`coord-5e19`)

`recovery/probe-scaffolding-2026-09-28` = **`2408c25`**, pushed, not merged. The prescribed
sweep was re-run from scratch: all 21 worktrees, every dirty and untracked file hashed
against all 1504 blob objects reachable in this repository (Cargo `target/` directories and
the two 30 MB instrumented binaries excluded as build output). **28** live files had no
matching blob. They account for as:

* **19** files in `madgab-approx-runtime/prof/`. Sixteen of them are the
  `results-*.txt` / `sum-*.txt` summaries — the deliberate, already-documented drop, since
  the archived markdown write-ups summarise them. The other **three were a real gap and
  are this pass's recovery**: `targets.txt`, `scale.txt`, `scale-after.txt`, now durable
  under `docs/work/probe-inputs/`.
* **8** instrumented `src/lib.rs` copies — **independently re-verified this pass** with
  `git apply --check --reverse` inside each live worktree. All eight match their archived
  patch. Standing rule 7 confirmed, not taken on trust.
* **1** `c1d3a7-instr/m.txt` (4.0 MB `ZZMETRICS` dump), covered by
  `probe-output/c1d3a7-m-head200.txt` at `3ce5262`.

The third pass archived the harness (`run.sh`, `summarize.py`) and the baseline output and
concluded the `prof/` findings were reproducible. They were not: `run.sh`'s final line is
`done < prof/targets.txt`, so the archived harness had no input list to iterate, and the
scale series had no phrase list on either side of the change. Three small text files, none
of them source, raw output, or an unarchived `src/` file — which is exactly why rules 6 and
7, both of which reason about source and diffs, missed them. That gap is now standing
rule 8, and it is a better lesson than the two before it: *the previous passes kept
auditing for the wrong kind of file.*

**Nothing else is at risk.** With these three durable, the entire remaining unarchived live
state is binaries, Cargo `target/` directories, and summaries the archived markdown already
supersedes.

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

* **`coord-5e19` (this pass), 2026-09-28T05:46Z–06:00Z** — reconciliation only. Both
  prescribed checks run again. `antonina agent list` is entirely terminal (`3a8f01`/`3a8f02`
  still `stopped` by the pause, deliberately left that way). The hash sweep found a real
  gap that the previous three passes had missed: the `prof/` harness *inputs*,
  `targets.txt`/`scale.txt`/`scale-after.txt`, now at `2408c25` on the recovery branch —
  the third pass had archived the harness without the file `run.sh` reads, so the
  measurement it claimed to have preserved still could not be re-run. Rule 7's eight
  diff-archived `src/lib.rs` copies re-verified individually rather than assumed. Census
  unchanged (87 done, 12 superseded, 0 open, 0 blocked; this log the only `working` entry).
  No agent launched, no front resumed, no item claimed, nothing integrated, `main`
  untouched.

* **`coord-9a3c` (this pass), 2026-09-28T06:11Z–06:17Z** — reconciliation only, **no recovery
  needed**. Both prescribed checks run again, and this is the first pass whose sweep came back
  clean. `antonina agent list` is still entirely terminal — no `running`, `idle`-but-live or
  queued agent anywhere; the only `idle` entry is `a11d` in a 20724-day-old `/tmp` workdir,
  unrelated to MadGab. The full `-uall` sweep walked all 21 worktrees, hashed every dirty and
  untracked file (< 2 MB, Cargo `target/` excluded) against all **1509** reachable blob objects,
  and returned **24** unmatched files — which classify into exactly two already-known buckets
  and nothing else:

  * **8** instrumented `src/lib.rs` copies, **re-verified one at a time** with
    `git apply --check --reverse` of the matching `docs/work/probe-patches/*.diff` read out of
    the recovery branch. All eight reported `OK`. Standing rule 7 is now confirmed by direct
    test for the second consecutive pass rather than inherited.
  * **16** `madgab-approx-runtime/prof/results*.txt` and `sum*.txt` — all of them harness
    **outputs**, not inputs. Per standing rule 8 the archived harness was re-read end to end:
    `run.sh` reads only `prof/targets.txt` (archived at `2408c25`) and `$BIN`; `summarize.py`
    reads only the `results.txt` path `run.sh` writes. Every input is durable and every one of
    these 16 is regenerable, so the deliberate drop still stands and is now justified by reading
    the harness rather than by assertion.

  So unlike the three passes before this one, **there was no new gap of a new kind to find**,
  and the honest result is that the programme is fully durable. Per the standing rule above,
  a clean pass records no scaffolding and opens no front. Nothing was launched, resumed, claimed
  or integrated; `main` untouched; this log is the only change, and it adds no new commit beyond
  itself. A future pass should not repeat the whole sweep uncritically — but it should still
  repeat it, because the last three passes each found something and the sample of "nothing left"
  is still only one pass deep.

## Next action for a fresh pass

Read `docs/accepted-state-2026-09-27.md`, then check only two things: `antonina agent list`
for anything alive, and every worktree's `git status --porcelain` for uncommitted `src/` or
untracked `examples/`/`tests/`/`src/` files **and untracked directories** not already
covered. Verify coverage by **content hash against the live file** — against blobs *and*
against the archived diffs (standing rules 6 and 7; two same-basename/different-content
pairs and eight diff-archived `src/lib.rs` copies have already tripped a naive check).

The three passes before this one each missed something, and it was never the same kind of
thing twice, so do not narrow the sweep to whatever the last pass went looking for. A file
is unarchived if no reachable blob hashes to its content and no archived diff reconstructs
it — full stop, regardless of what it is. Note that `git rev-list --objects --all` feeds
`cat-file --batch-check` a *path* on most lines, so `$2` is the path, not the type; extract
the shas with `cut -d' ' -f1` first or the blob set comes out empty and every file looks
unarchived.

If both checks are clean, **there is no work to do** — confirm the pause, record nothing
further to avoid commit noise, and exit. Do not open a front.

