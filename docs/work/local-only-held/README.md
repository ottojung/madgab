# Local-only held objects — recovery archive 2026-09-28

Branch: `recovery/local-only-held-2026-09-28`. **Not merged.** Nothing in this archive is
proposed for landing; this is evidence preservation, not a development front.

## What these are

Nine blobs that no ref on the remote holds, and that no earlier `recovery/*` archive
contains. They were found by `coord-3e88` on the forty-first pass.

## Derivation

1. `git fetch origin '+refs/heads/*:refs/remotes/audit/*'` — 187 remote refs, so the
   comparison set is every object the remote can reach (5 454 object shas).
2. `set -- $(git for-each-ref refs/remotes/audit/ --format='%(refname)')` then
   `git rev-list --all --not "$@"` — **11 commits**. Both sanctioned spellings were run
   and agree to the commit (standing rules 14, 30, 31); no `^$(…)`, no `xargs`.
3. Each of the 11 was classified by the ref that holds it
   (`git for-each-ref --contains <c>`, standing rule 11):

   | holder class | commits | disposition |
   |---|---|---|
   | local-only branch (`scratch/4d1e93-f5f6`, `scratch-3f8c62-landed`, `phon-probe-d4e8b1`, `scratch/0f3a17-shortlist-probe`) | `cf44be7f`, `514ed917`, `fc3a9306`, `b4a3009c`, `c06953a9` | unique content archived here |
   | `refs/stash` (working-tree and index parents of one entry) | `496826b5`, `3fdcbe7c` | one unique blob archived here; per standing rule 15 `refs/stash` is a single ref, not one per entry |
   | stale `refs/remotes/origin/*` names that `git ls-remote` contradicts (`madgab-audit-d5a2c1` local `3f098bcf` vs remote `36589f8`; `madgab-fuzzy-cost` local `b7b22b7`/`880d7bc`/`8b1a61f1` vs remote `0f7f763`) | 4 | one unique blob archived here; the three `fuzzy-cost` commits carry no blob the remote lacks |

4. Per commit, `git ls-tree -r <c>` (the **tree**, never the diff — standing rule 28),
   field 1 only (rule 17), each blob tested by `grep -qx` against the remote object set,
   with Cargo output excluded by path **component** `target*` (rule 9). That last filter
   removed **229 of 230** blobs from `scratch-3f8c62-landed`, every one under
   `target-base/`; without it this archive would carry a committed build directory.

   Result: 9 unique blobs over 9 (commit, path) rows.

## Verification

* Layer 1, identity: `git hash-object` of all 9 archived files reproduces the 9 candidate
  blob shas exactly.
* Negative control: `07cf372` (this branch's parent) and `6e1d0ce` (a pushed recovery
  branch tip) each report **0** blobs absent from the remote set, so the filter is known
  to be able to return zero rather than always returning a positive.
* Cross-check: the 9 shas were re-tested against the remote object set after a second
  `git fetch` immediately before archiving. Still 9.

## Fence note, read before promoting any of it

The six `src/lib.rs` and one `src/approx.rs` copy here are ZZ-instrumented measurement
sources from paused fronts. Several contain canonical phrases as probe literals. They sit
under `docs/`, which `tests/no_phrase_hard_coding.rs` does not scan, and
`ALLOWLIST_CAPS` is unchanged. **Any future promotion must strip the phrase literals, not
waive them.** The `tests/probe_f5f6.rs` and `docs/work/items/w-d5a2c1.md` copies are
research scaffolding for fronts `4d1e93` and `d5a2c1`, both closed.

## Reproduction

```sh
git fetch origin '+refs/heads/*:refs/remotes/audit/*'
git rev-list --objects $(git for-each-ref refs/remotes/audit/ --format='%(refname)') \
  | cut -d' ' -f1 | sort -u > /tmp/remote_objs.txt
set -- $(git for-each-ref refs/remotes/audit/ --format='^(%(refname))')
git rev-list --all --not $(git for-each-ref refs/remotes/audit/ --format='%(refname)')
# then per commit, ls-tree -r, filter target* by path component, grep -qx field 1
```
