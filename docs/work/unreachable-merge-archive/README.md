# Unreachable merge-commit content — 2026-09-28

**39 commits, held by no ref and no reflog, carrying 39 unique source blobs, recovered here.**
Branch `recovery/unreachable-merge-content-2026-09-28`, based on `6f3e563`.

## Why this class was missed for thirty-one passes

The previous pass (`coord-9c31`) ran the recommended
`git fsck --unreachable` check, classified all 180 unreachable commits, and reported
**two** carrying unique unarchived content. That number is wrong, and the reason is
a spelling error of exactly the kind standing rules 9, 14, 17 and 22 exist to prevent:

> `git diff-tree -r <merge>` prints **nothing**. Git does not diff a merge commit against
> its first parent unless you ask (`-m`, `--cc`, or `--first-parent`). The command
> succeeds, exits 0, and reports an empty diff.

**85 of the 180 unreachable commits are merges** (measured: `git rev-list --parents -n1`
and a word count > 2). Every one of them looked empty to a `diff-tree`-based check. A
merge commit is the *normal* shape of a `git stash` entry, so the class most likely to
carry uncommitted human work is the one the check is blindest to by construction.

The 31 prior passes are not to blame for using `diff-tree`; the log never prescribed it.
The reusable lesson is the sixth instance of this repository's dominant failure mode:
**a check that cannot fail returns a clean, confident, wrong number.** `diff-tree` on a
merge is not a stricter test of anything — it is a no-op.

## What is in here

* `patches/<short>.diff` — 39 patches, each `git diff --binary <c>^1 <c>`.
* `files/<short>__<path>` — the 39 unique files verbatim, one per commit.
* `MANIFEST.tsv` — `<short> <full-commit> <parent1> <parent2> <path>`, 39 rows.

Content shape: 32 instrumented `src/lib.rs` copies, 6 `src/approx.rs` copies, and one
`docs/work/items/w-3c5b18.md` edit. Dates 2026-09-26 to 2026-09-28. They are WIP snapshots
of paused fronts (`w-1c3e77`, `w-2b6a19`, `w-2f7a10`, `w-4b1e07`, `w-7b40d2`,
`w-c1d3a7`, `w-9e2b41`, `w-e086cc`, and others), held by neither `refs/stash` nor any
branch — they are reachable **only** as loose objects awaiting `git gc`.

## Verification, in three independent layers

Not "does the path exist in the archive" (standing rule 6), but does the archive
reconstruct the thing:

1. **File identity.** Each archived file's `git hash-object` equals
   `git rev-parse <commit>:<path>`. **39/39 MATCH.**
2. **Forward application.** Each patch `git apply --check --cached` cleanly against a
   temporary index read from its own parent. **39/39 apply.**
3. **Patch to blob identity** — the strong form, which the first two do not give on their
   own. Each patch is actually *applied* to the parent's index and the resulting index
   entry for the path is compared with `git rev-parse <commit>:<path>`. **39/39 MATCH.**

The harness was shown able to fail before its clean result was believed (standing rules
14 and 18): a positive control (a known-covered commit's own diff reverse-applies —
detected) and a negative control (a truncated patch — rejected). A check whose sensitivity
is demonstrated can return a negative result; one that has not can only return a number
of unknown meaning.

Per standing rule 12, the patches are `git diff --binary <c>^1 <c>`, **not**
`git format-patch`: these are stash-shaped merges, and `format-patch` on a merge emits the
wrong side. Layer 3 is what confirms the choice — had `format-patch` been used, layer 2
would have failed at a plausible hunk.

## Two commits deliberately excluded, with reasons

* **`202aef9f`** (`untracked files on madgab-approx-runtime`, a root commit, 18 unique
  blobs) — 16 of its 18 are the `prof/{results,sum}*.txt` harness **outputs** already
  classified as regenerable by standing rule 8, and the other two are the 30 MB instrumented
  binaries `prof/madgab-baseline` and `prof/madgab-prof`, which four prior passes
  deliberately declined to archive as build output. The 24-file `prof/baseline/` set in the
  same tree hashes identically to the copy already at
  `docs/work/probe-output/approx-runtime-prof-baseline/` on `recovery/probe-scaffolding-2026-09-28`
  (spot-checked, `1.out` = `a1ce1ad3` on both sides). Excluded, and the exclusion is a
  judgement worth a second passer's eye rather than a silent one.
* **`0088d27c`** (`w-b3e91a`) and **`727eb36b`** (`madgab-axis-558697`) — both **already
  archived** by `coord-2b74` and `coord-11b9` respectively; their diffs are byte-identical
  to `docs/work/unreachable-objects/…-0088d27c-….diff` and
  `docs/work/probe-patches/madgab-axis-558697-src.diff`. Re-archiving them would be
  duplication, so the 40th candidate is excluded on evidence, not on assumption.

## Coverage check: what the other 138 look like

Of the 180 unreachable commits, 138 have **zero** unique blobs under `git ls-tree -r`
(compared field-1 against `git rev-list --objects --all --reflog | cut -d' ' -f1`,
per standing rule 17) — 85 merges and 53 non-merges. That is a real measurement rather
than the `diff-tree` no-op: for each commit every blob in its tree is tested, so a merge
is covered. **All 180 are now classified, and 39 carried recoverable content.**

## Fence note — repeated, and it is load-bearing here

These are `src/lib.rs` and `src/approx.rs` copies containing canonical phrases
(`recognize speech`, `wreck a nice beach`, `hits justice dupe hid came`) as probe
literals. They now live under `docs/`, which `tests/no_phrase_hard_coding.rs` does not
scan — it walks `src/`, `web/` and `examples/` only — and `ALLOWLIST_CAPS` is unchanged.
**If any of this is ever promoted, the phrase literals must be stripped as part of that
promotion, not waived.** Archiving is not promoting; the accepted head is unaffected and
remains green on its own fence.

## Reproducing the discovery

```sh
git fsck --unreachable --no-progress | awk '/unreachable commit/{print $3}' > u.txt
git rev-list --objects --all --reflog | cut -d' ' -f1 | sort -u > reach.txt
while read c; do
  git ls-tree -r "$c" | awk '{print $3}' | while read b; do
    grep -qx "$b" reach.txt || { echo "$c $b"; break; }
  done
done < u.txt | sort -u
```

`git ls-tree -r`, not `git diff-tree -r`. That is the whole finding.
