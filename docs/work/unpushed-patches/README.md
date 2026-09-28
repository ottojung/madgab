# Unpushed local commits archived as patches

Provenance for the `*.patch` files in this directory, added by the reconciliation pass
`coord-11b9` on 2026-09-28.

## Why these exist

Sixteen prior reconciliation passes swept **worktree files** for at-risk state and found none.
That sweep has a blind spot: it hashes files against `git rev-list --objects --all`, which
includes local branches. Content that exists *only* in a commit on a local-only branch is
therefore always reported as archived — the check is satisfied by the very ref that would be
lost. No commit on an unpushed branch can ever be reported unarchived by that method.

So this pass asked the question the file sweep cannot: **which commits are reachable from no
remote head?** Answer: five, on four local branches, holding real instrumented source.

## Method

`git rev-list --all --not refs/remotes/audit/*` over a full fetch of all 179 remote heads
(`git fetch origin '+refs/heads/*:refs/remotes/audit/*'`).

The full fetch is required and is a trap. The repository's configured refspec is
`+refs/heads/post-milestone-acceptance:refs/remotes/origin/post-milestone-acceptance`, so
`refs/remotes/` held only **19** stale entries. A containment or `git branch -a` check against
it reports false negatives. Fetch remote heads explicitly into a scratch namespace, or verify
with `git ls-remote`, as earlier passes correctly found.

19 local branches have no remote counterpart; 15 of them are contained by some remote head.
These 4 are not:

| branch | tip | at-risk commits | what it is |
|---|---|---|---|
| `phon-probe-d4e8b1` | `fc3a930` | 1 | `w-d4e8b1` phonetic-cost pricing probe instrumentation: +806/−1 `src/lib.rs`, +28 `src/approx.rs` |
| `scratch-3f8c62-landed` | `514ed91` | 1 | the landed C1d parsimony axis, +390/−38 `src/lib.rs`. **The commit is named "never to be integrated" in `w-3f8c62`; that decision is unchanged. It is archived so the content is not lost, not promoted.** |
| `scratch/0f3a17-shortlist-probe` | `b4a3009` | 2 | `c06953a` per-slot shortlist dump + two measurement probes, then `b4a3009` the traversal's per-slot index and each pass's rank. +909 `src/lib.rs` across the pair |
| `scratch/4d1e93-f5f6` | `cf44be7` | 1 | `w-4d1e93` thread-local word-count parsimony axis probe: +52/−2 `src/lib.rs`, +143 `tests/probe_f5f6.rs` |

Each patch is `git format-patch -1` of the single commit, restricted to
`src/ tests/ examples/ web/ Cargo.toml`. `scratch-3f8c62-landed` additionally carries 352 MB of
`target-base/` Cargo build output; that is excluded, and it is why its patch is a diff rather
than a tree copy.

## Verification

Every patch was verified by **forward application**, not by path or basename. For each, a
throwaway worktree was checked out at the commit's parent, the patch applied, and the resulting
blobs compared by `git rev-parse` / `git hash-object` against the at-risk commit:

| commit | applies | `src/lib.rs` | `src/approx.rs` | `tests/probe_f5f6.rs` |
|---|---|---|---|---|
| `fc3a930` | yes | `85c3562380` MATCH | `46f8b8da8c` MATCH | — |
| `514ed91` | yes | `f86907c9af` MATCH | — | — |
| `c06953a` | yes | `8cb5f4494b` MATCH | — | — |
| `b4a3009` | yes | `1747cf50cb` MATCH | — | — |
| `cf44be7` | yes | `7010a0a352` MATCH | — | `5178bf61d6` MATCH |

All five reconstruct their at-risk content exactly. This is a stronger check than standing rule
7's `git apply --check --reverse` (which only proves a worktree is *consistent with* an
archived diff) because it proves the archived diff *produces* the lost commit's bytes.

## Fence note — read before promoting any of this

Several of these patches **contain canonical phrases** (`recognize speech`,
`wreck a nice beach`, the case-2 clue), because they are instrumentation written to measure
those cases. They sit under `docs/`, which `tests/no_phrase_hard_coding.rs` does not scan —
that fence walks `src/`, `web/` and `examples/` only — and `ALLOWLIST_CAPS` is unchanged. This
is documentation of past measurements, not production coupling. **If any of it is ever
promoted, its phrase literals must be removed as part of that promotion, not waived.**

Every one of these is scratch measurement instrumentation, from fronts that are `done` or
`superseded`. None is production work and none is a candidate for integration on its own merit.
The value is that the measurements remain repeatable, which is the same reason the earlier
passes archived probe sources and the `prof/` harness.
