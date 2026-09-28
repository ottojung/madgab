# Ignored-file recovery — 2026-09-28

Provenance: reconciliation pass `coord-3c8f` (12:42Z–12:50Z). Not a merge candidate.
Durability only, per standing rule 5.

## What this is

`Cargo.lock`, the resolved dependency graph of the accepted release, copied verbatim from
`/workspace/madgab/Cargo.lock`.

| property | value |
|---|---|
| git blob | `3b1a0a54310f63b1ffa7bc1be78af1673dfd41ef` |
| sha256 | `42a62f2e579f9d23bced334c7919e866827bfe40e57689204782891ae61bf8ee` |
| size | 5,614 bytes |
| `[[package]]` entries | 24 |
| identical live copies | **118**, one per registered worktree |

## Why it needed rescuing

`.gitignore` line 2 is `Cargo.lock`, and this repository has **no `Cargo.lock` in any commit
on any ref** — `git log --all -- Cargo.lock` is empty and `git rev-list --objects --all
--reflog` names the path zero times. So the pinned dependency versions of the accepted
release existed only as 118 untracked working-tree files, invisible to every sweep in this
programme's history:

* **rule 44's dirty-path test** enumerates `git status --porcelain`, which does not list
  ignored paths at all;
* rules 6–9's hash sweep enumerated the same status output;
* the five object keys (commit / tree / ref / reflog / loose blob) hold nothing, because the
  bytes were never written into the object store.

This is the sixth instance of the log's one recurring failure shape — **an enumeration that
can only see what git chose to tell it about, being read as a statement about what exists.**
Pass 56 closed the *uncommitted* gap; this closes the *ignored* one, which is the same gap
with `.gitignore` sitting in front of it.

The one command that sees it, per worktree:

```sh
git status --porcelain -uall --ignored | grep '^!!'
```

## Measurement and controls

| step | result |
|---|---|
| registered worktrees swept | 127 |
| ignored paths, `target*` excluded by path component (rule 9) | **118** |
| distinct content hashes among them | **1** (`3b1a0a54`) |
| of those, absent from the whole object store | **1** |
| positive control: `git check-ignore -v Cargo.lock` | fires (`.gitignore:2`) |
| negative control: `/web/pkg/` (also ignored, line 3) | **0** paths — correctly absent, the directory does not exist |

`/web/pkg/` is recorded because it is the other ignored path this repository can ever
produce, and a pass that reported it as swept without counting it would repeat the
`coord-2b7e` filter bug. It is a build artifact of the WASM packager and is regenerable; the
lockfile is not, because the exact pinned versions of the accepted release are what the
release notes and the timing numbers in
`docs/accepted-state-2026-09-27.md` were measured against, and a fresh `cargo build` today
would resolve differently.

## Verification

The archived copy's blob hash equals the live file's
(`3b1a0a54…`, via `git hash-object` without `-w`, so the hash was computed and not created),
and its sha256 matches the live file byte for byte. One distinct content across 118 paths is
itself the control: the check can report non-zero, and it did.

## Standing note

Nothing here is production code and nothing is proposed for integration. `Cargo.lock` remains
git-ignored and must stay so — the accepted release intentionally does not pin dependencies
in-tree. This archive exists so that the *measurement context* of the accepted state is
recoverable, not so that a future build is forced to these versions.
