# Reflog-only scratch state recovered from `w-9c6f2b` — 2026-09-28

Archived by a paused-programme reconciliation pass. Nothing here is live work and nothing
here may be resumed; see [../../accepted-state-2026-09-27.md](../../accepted-state-2026-09-27.md).

## Provenance

| | |
|---|---|
| blob | `9343e1d77f0db2a84f9b93bbba078213d45d44c7` (`src/lib.rs`, 211402 bytes) |
| only holder | reflog entry for `refs/heads/scratch/9c6f2b-probe`; held by **no ref** |
| sole commit carrying it | `4625220bc43fcaeecc52767f609922297945e166` |
| commit subject | `scratch: 9c6f2b axis probe (ZZ_AXES per-candidate metric dump in finish)` |
| commit author | `AssemblyP1 Agent <agent@assemblyp1.local>`, 2026-09-27T02:20:03Z |
| parent | `eed0d5cd3630a6d70367889a760770317094b240` (`w-9c6f2b: open the objective-axis front`) — ref-held |
| delta vs parent | `src/lib.rs` only, +56 / −3 |
| front | `w-9c6f2b`, `state: done`, integrated to `post-milestone-acceptance` |

## Why it was at risk

`4625220` is **not** an ancestor of the `scratch/9c6f2b-probe` tip (`e9a2797`). The branch
reflog shows a `reset: moving to madgab-objective-axes` immediately after `8e198fc`, which
moved the branch onto the rebased `9001822` line. `4625220` was therefore stranded on the
discarded side of that reset: reachable from the reflog, from no ref, and prunable by `git gc`.

## Why it is superseded rather than unfinished

The discarded ZZ_AXES probe was replaced, in the same reflog, by the ref-held
`9001822` → `64ced66` ("measurement harness on the rebased tree", `ZZ_PHRASES` / `ZZ_STRUCT`)
→ `e9a2797` ("fix harness tuple order"), which **is** on the branch tip. The front itself
closed `done` and was integrated and reviewed (`d3f7a1`, verdict pass). So the durable half of
this front was never missing; only this superseded intermediate probe was, and it is archived
here for completeness rather than for resumption.

## Verification

Blob identity, not path-presence: `git hash-object` of the archived file equals the source sha
exactly (`9343e1d…`). A truncated-copy negative control hashes differently, so the check can
fail. Re-measured after push: the blob is in the ref-reachable set.
