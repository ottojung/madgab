# Recovered content of `scratch-3f8c62-landed` (commit `514ed91`)

Pass 184 of the paused-programme reconciliation (`w-paused-reconciliation`) found this commit
**at risk**: it was held only by the local branch `scratch-3f8c62-landed`, which is on no
`origin` head, and its tree is reachable from no object under any `audit/*` ref.

The commit could not be pushed as a branch: it also commits **329 build-output paths**
(`target-base/`, including a 129 MB `.rlib` and a 71 MB `.rmeta`), which GitHub rejects
(GH001, >100 MB file limit). That rejection is correct and was not worked around.

What is archived here is the commit's entire **non-build** content, which is exactly one
file — `src/lib.rs`, +390/-38 (not the "+428" of a mixed build+source stat line):

| File | Meaning |
|---|---|
| `src-lib-rs.patch` | `git diff 514ed91^ 514ed91 -- src/lib.rs` |
| `src-lib-rs.blob` | the resulting `src/lib.rs` verbatim (sha256 `95b58230…`) |
| `COMMIT` / `PARENT` | `514ed91` and its parent `8bfe7de` |

`8bfe7de` is already an ancestor of `origin/post-milestone-acceptance`, so the parent is
durable on `origin` and only this one commit's delta needed saving.

## Verification

`git apply --check` then `git apply` of the patch onto `8bfe7de:src/lib.rs` in a scratch
directory reproduces the archived blob **byte-for-byte** (sha256 match above), so the
recovery is a reproducible harness, not just an archived script (rule 8).

The build artifacts are deliberately **not** archived: they are Cargo output, are
regenerable, and are the only reason the branch push was rejected.
