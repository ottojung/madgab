# Archive: four WIP `src/lib.rs` states held by no ref and no reflog

Branch `recovery/reflog-only-wip-lib-2026-09-28`, cut from `ff73e2f` (never merged).

## What this is

Pass 92 of the paused-programme reconciliation log
([`docs/work/items/w-paused-reconciliation.md`](../../items/w-paused-reconciliation.md))
established that the standing "0 at risk" verdict was an inference, not a
measurement, and classified the at-risk commits as 7 held by a local ref /
**81 reflog-only**. It named archiving those reflog-only commits as the next
action.

This is that archive, narrowed to the content that is actually unique.

## How the candidate set was derived

Non-circularly against the **ref-only** object set, not against
`--all --reflog` (which would be satisfied by the very reflog entry that would
be lost — rule 10):

1. `git rev-list --all --reflog | sort` (1083) minus `git rev-list --all | sort`
   → the **82** reflog-only commits. Cross-checked: all 82 appear in the
   at-risk set computed against the 197 `ls-remote`-confirmed remote refs.
2. Per **tree** (rule 28 — `git ls-tree -r`, never `git diff-tree`, which
   prints nothing for a merge and all four of these are stash merges), the
   distinct blobs are **673**.
3. `comm -23` those against `git rev-list --objects --all | cut -d' ' -f1`
   (field 1 per rule 17) → the blobs **no ref holds**.
4. Rule 9 filter on the path component: 75 → **4**.

The 71 excluded are all under `target-after/`, i.e. the build output rule 41
names — committed by `33c409e`, already-durable by definition, and 355 MB not
worth re-archiving.

## The four

| blob | bytes | lines | holder commit | what it is |
|---|---|---|---|---|
| `07b2932046b3` | 275,497 | 6,399 | `f6688de9` | WIP on `madgab-floor-5e2d41` (w-5e2d41 cheap-end retention floor) |
| `81a04204abf5` | 249,262 | 5,885 | `5c21572` | WIP on `scratch/emit-probe` — carries `ZZ_PROBE_{FORCE,SPANS,WORDS,ACCT,WIDECAP,EDGES,DPTRACE}` instrumentation |
| `a004d777c378` | 215,246 | 5,149 | `e34eb42` | WIP on `madgab-enum-1c3e77` (enumeration front claim) |
| `f7258d4ddff8` | 126,456 | 3,299 | `44e36a6` | WIP on `madgab-clue-objective` |

Each differs from its own first parent (`491cc8b`, `719efb0`, `8d797e1`,
`f35df8f` respectively), so none is a duplicate of a state a ref already holds.
None of the four is carried by any of the 19 existing `recovery/*` branches.

## Verification

Archived **verbatim**, so the blobs become ref-held permanently. Verified by
**blob identity** — the strong form: `git hash-object` of each archived file
equals the source sha. **4/4 MATCH**, and a negative control (a truncated copy
of the last blob) hashes to something else, so the check can fail.

Per rule 7 these were *not* treated as already-covered by
`docs/work/probe-patches/*.diff`; the identity test was run against the
recovery branches' full object sets, and all four returned absent.

## Exclusions and their reasons

* `target-after/**` (71 blobs) — Cargo build output, rule 41.
* `docs/work/items/w-paused-reconciliation.md` @ `3daf0618` — this is pass
  91's own text of the log, and the current version is on
  `post-milestone-acceptance`. A stale copy of the log is not at-risk state.
* Everything reachable from a ref: 6526 objects, the durable set.

## Not a merge candidate

None of this is code to integrate. It is the WIP state of four fronts that
were later **priced negatives or integrated by other means**, kept so a
successor can see what those fronts actually had in hand. `81a04204abf5`'s
`ZZ_PROBE_*` instrumentation is the only one with research value on its face.

**If any of it is ever promoted into a merge, the phrase literals must be
stripped first** — several of these files predate the current corpus and
contain probe phrase lists. Nothing in `docs/`, `src/`, `tests/`, `web/`,
`examples/` or `Cargo.toml` on `post-milestone-acceptance` was touched.
