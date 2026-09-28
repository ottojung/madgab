# Archive of commits held by no ref (recovery pass coord-9c31, 2026-09-28)

## What this is

Two `git stash`-shaped commits from 2026-09-26 that were reachable from **no ref at all** — not a
branch, not `refs/stash`, and not any worktree reflog — and that no prior recovery pass had
archived. They were found only by `git fsck --unreachable`, because the standing rule-10 check
(`git rev-list --all --not --remotes=...`) can only see commits that hang off `--all`, i.e. off a
ref. These two did not.

`git gc` prunes unreachable objects after the default 90-day grace period, so this was a real
(at-risk) loss, not a hypothetical one.

## The commits

| commit | date | subject | base |
| --- | --- | --- | --- |
| `e9515446637e447c19b21ec2a20f5be71aabc06e` | 2026-09-26 10:57Z | `On madgab-clue-objective: wip2` | `293d723` |
| `921a3b62c5862c527f3c489adf80a515d10b1b1e` | 2026-09-26 11:03Z | `On madgab-clue-objective: timing-base` | `293d723` |

Both are two-parent stash commits: `<base>` and an `index on …` parent. They are **not** in an
ancestor relationship with each other (both branch from the same base `293d723`), so neither
supersedes the other and both are archived in full.

## Why they are not already safe

For each, the file contents were checked against every commit reachable from every ref
(`git rev-list --all` × `git rev-parse <commit>:<path>`):

* `e9515446` — `src/lib.rs` and `tests/corpus_integration.rs` are **both** reachable from no other
  commit. Unique.
* `921a3b62` — `src/lib.rs` is unique; `tests/corpus_integration.rs` happens to match `0f7f763`
  but `src/lib.rs` alone is enough to make the commit worth keeping.

## Contents

* `e951544-stash-worktree-vs-base.patch` — full binary diff base → worktree state.
* `921a3b6-stash-worktree-vs-base.patch` — likewise.
* `e9515446-src-lib.rs`, `e9515446-corpus_integration.rs`, `921a3b62-src-lib.rs` — the file
  contents verbatim, so the state is recoverable even if the patches are ever lost.

Both stash commits' *index* parents were checked and produce an empty diff against the base, so
no index-side state is missing.

## How to verify

```sh
git worktree add --detach /tmp/nr 293d723
git -C /tmp/nr apply --check <this-dir>/e951544-stash-worktree-vs-base.patch   # forward, clean
git -C /tmp/nr apply --check --reverse --3way <this-dir>/e951544-stash-worktree-vs-base.patch
```

Forward application was verified clean; plain `--reverse` needs `--3way` (the patch was generated
against the commit tree, not the worktree checkout, and `src/lib.rs` has 46 hunks). All three
copied files were `git hash-object`-checked against the source commits and match.

## What this is *not*

Not a development front, not a resume, not an integration. `main` is untouched. These patches are
archival evidence of state that a `git gc` would otherwise have destroyed; the pause gate is
untouched by this archive.
