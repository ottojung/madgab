# Stash-reflog recovery — 2026-09-28

Five `refs/stash` entries recovered because they are held by **no ref at all**. This is a new
object class, distinct from the ones standing rules 10, 11 and 13 cover:

- **Rule 10** counts `git rev-list --all`, which includes `refs/stash` — but `refs/stash` is a
  *single ref pointing at `stash@{0}` only*. `stash@{1}`..`stash@{5}` are reachable only through
  the reflog, so rule 10 never saw them.
- **Rule 13** counts `git fsck --unreachable`, which treats reflog entries as roots. These five
  are therefore *not* reported unreachable, so that check never saw them either.
- A `git stash drop`, or a `git stash clear`, or any `git reflog expire` on `refs/stash` destroys
  all five immediately and irrecoverably.

Verification that they really are invisible to both checks, run on this repository:

```
$ for c in f6688de 5c21572 e34eb42 5cd0d2a 44e36a6; do
    printf '%s --all=%s fsck-unreachable=%s --all --reflog=%s\n' "$c" \
      "$(git rev-list --all | grep -c ^$c)" \
      "$(git fsck --unreachable --no-progress 2>/dev/null | grep -c $c)" \
      "$(git rev-list --all --reflog | grep -c ^$c)"; done
```

Each is `--all=0`, `fsck=0`, `--reflog=1`. Only the third form finds them.

## Provenance, per entry

| stash | commit | base | content | at risk? |
|---|---|---|---|---|
| `stash@{1}` | `f6688de` | `f2fb62e` (w-5e2d41) | `src/lib.rs` +60/−34, cheap-end retention floor instrumentation | **yes** |
| `stash@{2}` | `5c21572` | `10e069f` (`scratch/emit-probe`, w-7c1f64) | `src/lib.rs` +36, `ZZ_PROBE_*` emission instrumentation | **yes** |
| `stash@{3}` | `e34eb42` | `f2908d1` (w-1c3e77) | `src/lib.rs` +162/−48, enumeration-front claim WIP | **yes** |
| `stash@{5}` | `44e36a6` | `4a0bedb`-era `madgab-clue-objective` | `src/lib.rs` +821/−183, the largest stash on the host | **yes** |
| `stash@{4}` | `5cd0d2a` | `880d7bc` (w-3b8e15) | `src/approx.rs` +211/−24 | no — content already durable |

`stash@{0}` (`496826b`, w-c3f81a) is not repeated here: it is held by `refs/stash` itself, so
rule 10 does see it, and `coord-2b74` already archived it at
`recovery/at-risk-refs-2026-09-28` as `496826b-w-c3f81a-*.patch`.

**`5cd0d2a` is the redundant one and is archived only for completeness.** Its resulting
`src/approx.rs` blob is `662eab99`, byte-identical to `b7b22b7:src/approx.rs`, which *is*
reachable — it is the same w-3b8e15 work that `coord-2b74` archived as
`880d7bc`/`b7b22b7`. Archiving it is not harmful, but nothing is lost without it. The other
four are not recoverable from anywhere else: their resulting `src/lib.rs` blobs
(`07b29320`, `81a04204`, `a004d777`, `f7258d4d`) are **not** in the 5,534-object reachable set,
whereas `5cd0d2a`'s `662eab99` is.

## Verification

By **forward application**, per standing rule 10's amended method, not rule 7's reverse check.
A throwaway worktree was created at each commit's own parent, the patch applied, and the
resulting file compared with `git hash-object` against `git rev-parse <commit>:<path>`:

```
f6688de  APPLIES  src/lib.rs=MATCH
5c21572  APPLIES  src/lib.rs=MATCH
e34eb42  APPLIES  src/lib.rs=MATCH
44e36a6  APPLIES  src/lib.rs=MATCH
5cd0d2a  APPLIES  src/approx.rs=MATCH
```

This matters here more than usual. The explicit two-dot form `git diff --binary <c>^ <c>` is
used rather than `git format-patch`, because **standing rule 12** was learned on `stash@{0}`
itself: `format-patch` on a stash commit silently emits the *index parent's* diff and the result
does not apply. Five more stash entries is five more chances to hit that, so the check is
re-run on every one rather than assumed.

## Nothing here is a merge candidate

These are stashes of in-flight work on closed and superseded fronts. Archiving is not promoting.

**Fence note, repeated from `coord-11b9` and `coord-2b74`:** these patches contain canonical
phrases as probe literals (`hits justice dupe hid came`, `recognize speech`,
`wreck a nice beach`) in instrumentation. `docs/` is **not** scanned by
`tests/no_phrase_hard_coding.rs`, `ALLOWLIST_CAPS` is unchanged, and any future promotion must
**strip the literals rather than waive them**. The accepted state is verified green on its own
fence at `post-milestone-acceptance` by `coord-4d31`'s execution, and nothing here disturbs it.
