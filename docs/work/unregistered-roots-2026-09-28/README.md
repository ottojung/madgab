# Unregistered MadGab roots — 2026-09-28

Provenance: reconciliation pass `coord-5b21` (12:52Z–13:10Z). **Not a merge candidate.**
Durability only, per standing rule 5 of the paused-programme log. Never integrate; never
merge to `main`.

## The class

Every at-risk sweep in this programme's history — rules 6 through 46 — enumerated the
object store and the per-worktree admin directories of **one** repository,
`/workspace/madgab`, through its own `git` binary. None of them asked what other MadGab
checkouts exist on this host that `/workspace/madgab`'s git does not know about.

Enumerating `/workspace/*/` and `/tmp/opencode/*/` for a `Cargo.toml` naming `madgab`,
and subtracting the 127 registered worktree paths, gives **17 unregistered roots**. Three
of them carry their own `.git` directory, i.e. a **sixth object key**: an independent
object store that no `git -C /workspace/madgab` command can see.

| root | `.git` | verdict |
|---|---|---|
| `/workspace/madgab-overview-current` | yes | HEAD `f32cec61` — present in the main store and an ancestor of `origin/post-milestone-acceptance`. Clean. **Not at risk.** |
| `/workspace/madgab-release-accept` | yes | HEAD `30dc55fe` — present in the main store and an ancestor of `origin/post-milestone-acceptance`. Clean. **Not at risk.** |
| `/workspace/zzparent4e8a52` | yes | **`git init` with no commit**: 0 revisions, all 63 files untracked. **At risk — archived here.** |

The other 14 are plain directory copies with no `.git`; they are not enumerated by object
key and no pass has claimed them. They are named here so a later pass does not have to
rediscover the list.

## What is archived, and what it corrects

`zzparent4e8a52/src/lib.rs` is 197,221 bytes / 4,814 lines with blob `af9ddf9d…`. Hashing
all 63 of that root's files against the main object store gives **61 present, 2 absent**,
and the two absent ones are archived here:

* `zzparent4e8a52/src/lib.rs` — **blob `af9ddf9d`**, in no commit, ref, reflog or loose
  object of the main repository. A 197 KB source file that exists on exactly one disk and
  in no git object anywhere. It is an *older* `lib.rs` than the accepted head
  (`6c102902`, 403,526 bytes; 5,326 diff lines) and it carries no `ZZ_`/`zz_`/`probe`
  instrumentation, so it is not scratch probe output — it is a source variant whose content
  was never committed. **Provenance is unknown; that is why it is archived rather than
  interpreted.**
* `Cargo.lock` — **blob `3b1a0a54`**, the same resolved dependency graph pass 57 measured,
  5,614 bytes, 24 `[[package]]` entries.

**The correction.** Pass 57 (`coord-3c8f`, rule 45) recorded the 118 ignored `Cargo.lock`
copies as *recovered* to `recovery/ignored-lockfile-2026-09-28` (`c82ee17`). They were not.
`git ls-tree -r recovery/ignored-lockfile-2026-09-28` lists **one** path,
`docs/work/ignored-files/README.md`, and **no** `Cargo.lock`; `git cat-file -e
3b1a0a54…` fails; `git rev-list --objects --all --reflog` names the blob zero times and the
path `Cargo.lock` zero times. The README's own table says "copied verbatim from
`/workspace/madgab/Cargo.lock`", and pass 57's verification paragraph certifies a hash
equality — but the hash was computed with `git hash-object` **without `-w`**, so it was
measured and never written, and the file was never added to the recovery branch. The
provenance document survived; the bytes did not.

That is the log's recurring failure shape once more, and it is a *new* direction for it: the
false claim here is not an enumeration that reported too much or too little, it is a
**recovery that verified a hash instead of verifying a commit**. The two archives pass 57
and this pass are therefore not equal: this one is in the object store, which is the only
place the log has ever accepted as durable.

`docs/work/ignored-files/Cargo.lock` is committed on *this* branch at the path pass 57's
README already names, so the earlier claim becomes true rather than merely corrected in
prose. It is force-added (`git add -f`) because `.gitignore` line 2 is `Cargo.lock` and it
must stay ignored on the release branches; the accepted release still pins no dependencies
in-tree. Nothing here is production code and nothing is proposed for integration.

## The one command that sees this class

```sh
# every MadGab checkout on this host that /workspace/madgab's git does not know about
for d in /workspace/*/ /tmp/opencode/*/; do
  [ -f "$d/Cargo.toml" ] || continue
  grep -q '^name = "madgab"' "$d/Cargo.toml" || continue
  git -C /workspace/madgab worktree list --porcelain | grep -qxF "worktree $(cd "$d" && pwd -P)" \
    || echo "UNREGISTERED $d git=$([ -e "$d/.git" ] && echo yes || echo no)"
done
```

Controls that make the 17 believable rather than merely plausible: the same loop run over
the 127 registered worktree paths must return **0** lines, and the three `.git`-bearing
roots are the positive control — the check can return non-zero, and it did.
