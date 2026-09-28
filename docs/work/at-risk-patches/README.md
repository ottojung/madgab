# At-risk commits reachable only through non-branch refs

Provenance for the `*.patch` files here, added by reconciliation pass `coord-2b74` on
2026-09-28. Companion to [../unpushed-patches/README.md](../unpushed-patches/README.md).

## What this pass found that the previous pass did not

`coord-11b9` ran the right check — `git rev-list --all --not <all remote heads>` — and found
five at-risk commits, all on local-only **branches**, and correctly explained why sixteen
earlier file sweeps had missed unpushed commits. Standing rule 10 now prescribes that check.

It is still not the whole answer, and the residual gap is the same *kind* of error as rules 9
and 10 before it: a check that looks stricter than it is. `--all` expands to
`refs/heads/* refs/tags/* refs/remotes/* HEAD` plus the reflogs. `coord-11b9` then reasoned
about the result **branch by branch**, and so only ever counted refs under `refs/heads/`. A
commit reachable from no remote head but held by some *other* kind of ref was never counted,
because there was no local branch to hang it on. The same run that reported
"19 local branches have no remote counterpart" could not see a commit that no branch pointed at.

Re-running the check verbatim returns **13** at-risk commits, not 5. Five are the ones
`coord-11b9` already archived. The other **eight** live in three ref classes that a
branch-shaped mental model does not have:

| ref class | commits | why it is invisible to a branch-by-branch reading |
|---|---|---|
| stale `refs/remotes/origin/*` | `3f098bc`, `8b1a61f`, `880d7bc`, `b7b22b7` | looks *remote* — the name says it is backed — but the configured refspec (`+refs/heads/post-milestone-acceptance:refs/remotes/origin/post-milestone-acceptance`) never updates these, so they are local-only refs wearing a remote-tracking name |
| `refs/stash` | `496826b` (and `3fdcbe7`, its empty index parent) | the stash is a ref, so `--all` sees it; it is on no branch and in no worktree |
| detached worktree HEAD | `69b5a07`, `a7f08ea` | `--all` includes the HEAD of every linked worktree; `/workspace/madgab-scorespread-measure` is detached at `a7f08ea` and is on no branch at all |

The `refs/remotes/origin/*` class is the dangerous one, and it is the direct consequence of
standing rule 10's own warning. That rule records the narrow refspec as a trap because a
containment check against `refs/remotes/` gives false negatives. The same narrow refspec
creates false *positives* of a different kind: entries in that namespace are not evidence of
remote backing at all. `refs/remotes/origin/madgab-fuzzy-cost` reads as "the remote's copy of
`madgab-fuzzy-cost`" and points at `b7b22b7`, while the real remote tip is `0f7f763` and
`b7b22b7` is not an ancestor of it. Three of the eight at-risk commits were sitting in refs
whose names assert a durability they do not have.

## The eight

| commit | held by | what it is |
|---|---|---|
| `8b1a61f` | stale `origin/madgab-fuzzy-cost` | `w-3b8e15` work item opening; +94 lines, docs only |
| `880d7bc` | stale `origin/madgab-fuzzy-cost` | articulatory-feature substitution cost; `src/approx.rs` +294/−12 |
| `b7b22b7` | stale `origin/madgab-fuzzy-cost` | charge an indel by what segment went missing; `src/approx.rs` +211/−24 |
| `3f098bc` | stale `origin/madgab-audit-d5a2c1` | `w-d5a2c1` close-out and the new `w-3a7f0d` item; docs only, +369/−9 |
| `496826b` | `refs/stash` (`stash@{0}`) | WIP on `scratch/review-c3f81a`; `src/lib.rs` +39 |
| `69b5a07` | detached worktree HEAD (parent) | `ZZ_AXIS` per-axis population dump for `w-2e5b93`; `src/lib.rs` +96/−11 |
| `a7f08ea` | detached worktree HEAD (tip) | `ZZ_AXIS` pool-neutrality fix, keeps `combined` as the original expression; `src/lib.rs` +59/−34 |
| `3fdcbe7` | `refs/stash` (index parent) | **empty tree diff** — the stash's index commit, no content of its own. Recorded, not archived. |

`69b5a07`/`a7f08ea` are the `ZZ_AXIS` axis-decomposition harness, which `scratch/3f9c02-measure`
and the `madgab-scorespread-measure` worktree also reference. `w-2e5b93` is closed, and the
commit is annotated "scratch, never integrated"; that decision is unchanged. It is archived so
the content is not lost, not promoted.

## Method

```sh
git fetch origin '+refs/heads/*:refs/remotes/audit/*'
git rev-list --all --not $(git for-each-ref refs/remotes/audit/ --format='%(refname)')
```

180 remote heads fetched into a scratch namespace. The correct exclusion set is the **fetched**
refs, not `refs/remotes/origin/*` and not the output of `git ls-remote` — the former is
precisely the stale set this pass is about, and the latter cannot answer containment.

Patches are `git format-patch -1` restricted to `src/ tests/ examples/ web/ Cargo.toml
docs/work/items/`, except `496826b`, which is a stash commit and therefore a merge; it is
archived as `git diff --binary 496826b^ 496826b` so the patch is a real one-parent diff.
`git format-patch` on a stash commit emits the *index* parent's diff and silently does not
apply — caught by the verification below, not by reading the patch.

## Verification

Every patch verified by **forward application** in a throwaway worktree checked out at the
commit's first parent, then the resulting blobs compared with `git hash-object` against
`git rev-parse <commit>:<path>`. This proves the archived diff *produces* the lost bytes,
which is stronger than standing rule 7's reverse check.

| commit | applies | files | blob |
|---|---|---|---|
| `8b1a61f` | yes | `docs/work/items/w-3b8e15.md` | MATCH |
| `880d7bc` | yes | `src/approx.rs` | MATCH |
| `b7b22b7` | yes | `src/approx.rs` | MATCH |
| `3f098bc` | yes | `docs/work/items/w-3a7f0d.md`, `docs/work/items/w-d5a2c1.md` | MATCH |
| `496826b` | yes | `src/lib.rs` | MATCH |
| `69b5a07` | yes | `src/lib.rs` | MATCH |
| `a7f08ea` | yes | `src/lib.rs` | MATCH |

`496826b` failed this check on the first attempt — the `format-patch` output did not apply —
and was regenerated as an explicit two-dot diff before passing. Recorded here because it is the
reason rule 12 exists.

## Fence note — read before promoting any of this

`880d7bc` and `b7b22b7` are instrumentation written to price the phonetic-cost axis, and like
the `coord-11b9` patches they **contain canonical phrases** (`recognize speech`,
`wreck a nice beach`, the case-2 clue). They sit under `docs/`, which
`tests/no_phrase_hard_coding.rs` does not scan — that fence walks `src/`, `web/` and
`examples/` only — and `ALLOWLIST_CAPS` is unchanged. This is documentation of past
measurements, not production coupling. **If any of it is ever promoted, its phrase literals
must be removed as part of that promotion, not waived.**

Every one of these is scratch measurement instrumentation or a work-item record, from fronts
that are `done` or `superseded`. None is production work, none is a candidate for integration
on its own merit, and the standing rule 1 prohibition on claiming a superseded item is
unchanged by this archive existing.

## Do not rely on `refs/remotes/origin/*` for anything

The nineteen entries in that namespace are stale. Two of them are the sole holders of
at-risk commits, as recorded above. A future pass that wants to know what is backed must
fetch, as every line of this file's method does.
