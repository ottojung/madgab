# Reflog-held state recovery — 2026-09-28

Thirty blobs that were reachable from **no ref** and were held only by reflog entries, so
`gc`/`reflog expire` would have destroyed them. Found by `coord-b4e1`; archived on
`recovery/reflog-held-2026-09-28`, pushed, **not merged**.

## What class of object this is

Standing rule 10 counts `git rev-list --all`, which enumerates *ref names*. Rule 15 established
that a reflog is not a set of refs. This is the remaining half of that: a commit whose only
holder is a **reflog entry of an ordinary branch**. It is visible to
`git rev-list --all --reflog`, invisible to `git rev-list --all`, and — because `git fsck`
treats reflog entries as roots (rule 13) — it is not reported unreachable either. Both
established commit-level checks are individually correct and jointly blind, which is rule 15's
exact failure shape.

## How the candidate set was derived (non-circular)

The comparison set is the **ref-only** object set, `git rev-list --objects --all` (no `--reflog`,
5 763 objects), so the class under test is not part of what it is compared against. Per rule 28
the probe asks about each commit's **tree** (`git ls-tree -r`), never its diff, because
`git diff-tree -r <merge>` prints nothing. Field 1 only (rule 17). Cargo output excluded by path
**component** `target*` (rule 9) — that filter alone removed 72 of 117 blobs, every one under
`target-after/release/`.

Result: the 81 reflog-held-only commits (`coord-5b93`'s figure, reproduced) carry **117** blobs
no ref holds, of which **34** survive the component filter:

    23 src/lib.rs   5 tests/corpus_integration.rs   13 docs/work/items/*.md   1 .gitignore.tmp

## The strong coverage test, and the figure it produces

`coord-5b93` declined to archive on a patch-id comparison and said the right test is
**patch-applied → blob identity**. That test was run here, as the layer-3 form of rules 12 and 28:
each of the **88 patches already on the eight `recovery/*` branches** was applied with
`git apply --cached --binary` to a temporary index seeded from its own base tree, and every blob
in the resulting index was compared by identity against the 34 candidates.

    covered by existing archives:  4 / 34
    genuinely uncovered:         30 / 34   <- archived here

So the previous pass's **113-vs-4** split was indeed a lower bound, as it said, and the true
coverage figure is **4 of 34 (12%)** — the archives on the eight branches cover almost none of
this class. Three of the four came from the whole-stash archives on
`recovery/stash-reflog-2026-09-28`; the fourth from
`recovery/unreachable-merge-content-2026-09-28`.

Harness sensitivity was demonstrated before its result was believed, per rules 14 and 18:
a **positive control** (a known-covered blob is found in the covered set) and a **negative
control** (a ref-held `src/lib.rs` blob is correctly absent from the candidate set) both pass.
This matters because the previous pass's first attempt at this classification returned **363**
"unique blobs" from a comparison set that excluded reflog-reachable blobs by construction — a
probe that restated the definition of the class under test. The figure above does not have that
defect, and that is the whole difference between them.

## Verification of this archive

1. Every archived file's `git hash-object` equals the candidate blob sha — **30/30 MATCH**.
2. `MANIFEST.tsv` records `(commit, blob, path)` for each, so the archive is traceable back to
   the exact reflog-held commit that carried it.
3. **The strongest form available here, and the one that closes the class:** because these blobs
   are now committed *verbatim* on a pushed branch, `git rev-list --objects --all` contains all
   30. Re-running this pass's own candidate probe must therefore return **0**. That is the
   falsifiable figure a future pass should record, not "nothing found".

## Provenance and the fence note

The 19 `src/lib.rs` copies and 5 `tests/corpus_integration.rs` copies are ZZ-instrumented
measurement sources and work-item drafts from paused fronts (`w-2f7a10`, `w-7b2d40`, `w-9d4e17`,
`w-9c6f2b`, `w-d5a2c1`, `w-5d03af`, `w-a1f3d2`, `w-c1d3a7`, `w-8f0b3d`, `w-3a8f01`/`w-3a8f02`,
`w-4b1e07`, `w-9b4a15`). **Nothing here is a merge candidate.** They are archived as *evidence
of past measurement*, exactly as the other eight `recovery/*` branches are.

They contain canonical phrases as probe literals. They sit under `docs/`, which
`tests/no_phrase_hard_coding.rs` does not scan, and `ALLOWLIST_CAPS` is unchanged. **Any future
promotion must strip the phrase literals rather than waive them.**

`.gitignore.tmp` is a stray editor artefact and is kept in the count rather than quietly dropped.
