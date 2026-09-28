# At-risk uncommitted worktree content, archived 2026-09-28 (coord-7a3e)

Seventh and eighth `recovery/*` archives. This one covers the class the earlier
archives missed: **live uncommitted worktree file content whose blob is in no
commit, no ref and no reflog**, so a single `git worktree prune` destroys it.

Each path is `files/<worktree-basename>/<path-relative-to-worktree-root>`.
The archived blob is byte-identical to the live file; verify with
`git hash-object <live file>` against the sha below.

| worktree | path | blob sha (live == archived) | size |
|---|---|---|---|
| `/workspace/floor-5e2d42-baseprobe` | `src/lib.rs` | `1689c3f306e9a558d738c45147c46c3d584616f5` | 273689 |
| `/workspace/c1d3a7-instr` | `m.txt` | `41601446842f81bb6fdc20429c4d87cdd1577152` | 3997607 |
| `/workspace/madgab-8a1d47-measure` | `src/lib.rs` | `6fc73603840aad467e5458670e2d5759590b051b` | 192538 |
| `/workspace/madgab-rdp-0f3a17` | `src/lib.rs` | `773d6830ad4f0ac014b5168685b9f84414b2b050` | 289531 |
| `/workspace/madgab-probe-5b1e93` | `src/lib.rs` | `afdfa8dd8f72c3dcf1e9763dc90a9b4e087ed83c` | 267838 |
| `/workspace/probe-0f3a17` | `src/lib.rs` | `c4e1c156f9129486f89dc8c82e98d3019231b212` | 266446 |
| `/workspace/floor-5e2d42-probe` | `src/lib.rs` | `e29128b19364c1c2fd9f193275f5dfe9603e32a0` | 280289 |

All six `src/lib.rs` files are **scratch instrumentation snapshots** (300/68/738/135/94
inserted lines respectively) from fronts already closed or superseded; none is
production code and none is proposed for integration. `m.txt` is a 21,020-line
`ZZMETRICS` dump from the `c1d3a7` measurement arm.

**Never to be integrated.** This branch is durability only. MadGab development is
paused; see `docs/accepted-state-2026-09-27.md` and
`docs/skills/itinerary-madgab.md`.

## How the population was found

```sh
for d in $(git worktree list --porcelain | grep '^worktree ' | cut -d' ' -f2); do
  git -C "$d" status --porcelain | while IFS= read -r l; do
    f=${l:3}; p="$d/$f"; [ -f "$p" ] || continue
    echo "$(git hash-object "$p")|$d/$f|$(stat -c%s "$p")"
  done
done | sort -u
# then keep only the shas for which `git cat-file -e <sha>` FAILS
```

That is 34 dirty paths across 21 worktrees, of which **7** are at risk; the other 27
hash to blobs a commit already holds and are therefore already durable.
