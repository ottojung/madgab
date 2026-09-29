# Recovered: four reflog-only revisions of the reconciliation log

Branch: `recovery/reflog-log-revisions-2026-09-29` (created from `9f53202`).

## What these are

Four revisions of `docs/work/items/w-paused-reconciliation.md` that existed **only in the local
reflog**. Each was written by an earlier coordination pass, was never pushed under a ref, and its
blob is held by **no** commit, ref, or other object in any `audit/*`-reachable stream.

| file | source commit | pass | original blob | bytes |
|---|---|---|---|---|
| `w-paused-reconciliation-pass-92.md` | `e860ad67` | 92 | `3daf0618` | 743,043 |
| `w-paused-reconciliation-pass-163.md` | `35819c93` | 163 | `f1fff1ee` | 1,159,315 |
| `w-paused-reconciliation-pass-165.md` | `9f6347af` | 165 | `af791066` | 1,171,444 |
| `w-paused-reconciliation-pass-177.md` | `069074f3` | 177 | `edd7f4ff` | 1,252,685 |

Each file here is **byte-identical** to its original blob, verified by `git hash-object` on the
extracted file equalling the blob id in `rev-list --objects --all --reflog` — not by "the path was
archived", which is rule 6's failure mode. All four hashes reproduce:

```
edd7f4ff8264b09194c4b4f6f3db0b7233141033  pass-177
f1fff1eee79a5a82faa25a29701c5d10a8f680fc  pass-163
af79106613026d85cf4bbdd81d4ae21417ddabff  pass-165
3daf06186341adecaa0580b871cebd7b4c447079  pass-92
```

## How the gap was found

Pass 213. The reflog-only set is **87** commits. Of those, **63** have a tree that is not reachable
from any `audit/*` ref. Those 63 were expanded to non-build path entries (rule 9 filter:
`*/target/*`, `*/target-*/*`, `*/prof/*` excluded, by path component) and each entry's blob was
checked against the full `rev-list --objects --all` id set. Exactly **4** entries came back missing,
and all four are revisions of this log.

So the *code* content of the 63 is durable (its blobs are held by other commits); the only genuinely
unrecoverable bytes on this repository are these four log revisions.

## Why pass 92's revision matters

`e860ad67` is the entry that **first refuted** this log's standing "0 at-risk" verdict — *"the
standing 0-at-risk verdict is refuted: 81 of 88 are reflog-only, 317 objects held by no ref"*. It is
the origin of the at-risk machinery that every later pass runs. Losing it would remove the record of
why the machinery exists.

## Why this was not archived before

Earlier passes measured the at-risk set at the **commit** level (is a commit reachable from a ref?)
and, after pass 184, treated the residual `514ed91` commit object as the only open item. That is
correct for commit reachability and blind to this class: these four blobs *are* reachable, by the very
reflog that a `git gc` or reflog expiry deletes. The general form is rule 10's, one level down — the
instrument must ask whether the **content** is durable somewhere other than the thing that might be
pruned, not whether the commit is currently reachable.

Controls: a known-present id (`eaf7487`, the `recovery/at-risk-2026-09-29` tip) returns 1 hit in the
id set; a fabricated id returns 0. The 0 is a measurement, not a broken instrument.
