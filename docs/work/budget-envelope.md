# The budget envelope: what an 8x global emission budget buys, and what it costs

Measurement for [w-be6d21](../items/w-be6d21.md), branch
`madgab-budget-envelope`, worktree `/workspace/madgab-budget-envelope`, from
`aac97e1`. **Documentation only — no `src/` change is committed.** The budget
constants were varied locally to take the measurements and restored; the tree
is clean and `git diff HEAD -- src/` is empty.

## Verdict, first

**The 8x lever is affordable and it buys nothing. It is not merely
insufficient; it is arithmetically incapable of mattering, and so is 16x, and
so is 256x.**

The deciding numbers, all from the release binary on
`It's just a stupid game` at `--approximate --top 50`:

| configuration | global emission | per-segmentation emission | search | pool | canonical clue |
|---|---|---|---|---|---|
| 1x (shipped) | 16,384 | 64 | 1.57 s | 15,710 | **missing** |
| 16x global only | 262,144 | 64 | 1.52 s | **15,710** | missing |
| 1x global, 16x per-seg | 16,384 | 1,024 | 1.50 s | **15,710** | missing |
| **16x both** | 262,144 | 1,024 | **2.37 s** | **100,576** | **missing** |
| 64x both | 1,048,576 | 4,096 | 2.75 s | 175,882 | missing |
| 256x both | 4,194,304 | 16,384 | 2.81 s | **177,029** | missing |

Two things decide it.

1. **Raising the global budget alone is provably inert**, and the search
   confirms it byte-for-byte: 1x, 2x, 4x, 8x    and 16x produce the **same
   15,710-candidate pool, the same rank-49 cutoff score 0.915691888, the same
   pool rank for every probe phrase, and the same enumerated-or-missing
   verdict**, on all five targets. The reason is arithmetic, not
   tuning: the whole search spends at most
   `SEGMENTATION_KEEP * LEXICAL_COMBINATIONS_PER_SEGMENTATION =
   256 * 64 = 16,384` emissions, because `emit_allowance` is clamped at
   `LEXICAL_COMBINATIONS_PER_SEGMENTATION` per segmentation
   (`src/lib.rs:1015-1017`). **`LEXICAL_GLOBAL_EMISSION_BUDGET` is already at
   its maximum reachable value at 1x.** Any multiple above 1 multiplies a
   number that cannot be reached. The row "1x global, 16x per-seg" is the
   control that isolates this: with the per-segmentation ceiling lifted 16x
   while the global budget stayed at 16,384, the pool did not move by one
   candidate.
2. **Even when the ceiling is lifted too, the clue never arrives, and the
   pool saturates.** At 16x on both ceilings the pool grows 6.4x to 100,576
   and the clue is still absent; by 64x it has essentially stopped growing
   (175,882) and at 256x it has stopped entirely (177,029). 177,029 is not a
   budget artefact — it is the size of the set the search can reach at all,
   because the per-slot fan-out is `LEXICAL_BRANCH_KEEP = 10`
   (`src/lib.rs:1210`). The canonical clue's `hid` sits at **walk rank 99**
   in its slot, so that node is never pushed. [w-6b2f04](../items/w-6b2f04.md)
   reached the same conclusion by a different route ("no amount of popping
   reaches a node that is never pushed"); this item confirms it *through the
   budget lever itself*, which is what the coordinator asked for.

So the milestone clue is **not enumerated at any budget multiple tested, up to
and including 256x.** The budget family is closed: not "expensive", not
"insufficient" — *saturated*.

## Is it a budget problem or a scoring problem?

**Neither. It is a width/reach problem, and the scoring question is still
open behind it.** The spec's distinction was "enumerated-and-cut" (scoring)
versus "genuinely ranked" (budget). The measured answer is a third case, and
it is worth stating precisely because it relocates the milestone:

- The clue is **not enumerated** at any multiple, so it is never cut by the
  score cutoff and never ranked. It is absent from the pool itself.
- The cutoff arithmetic is therefore *not* what is blocking it today, but it
  would be the next wall. The clue's score is **0.819901** against a rank-49
  cutoff of **0.915691888** on `It's just a stupid game` — a gap of 0.0958.
  Even a hypothetical enumeration fix would surface it at rank ~400, not in
  the visible 50. Recorded so the number is not re-derived, and so that
  [w-04f83f](../items/w-04f83f.md) / [w-2e5b93](../items/w-2e5b93.md) know
  that a reach fix alone does not close the milestone.

## Measurement 1: the required sweep, 1x / 2x / 4x / 8x / 16x

`LEXICAL_GLOBAL_EMISSION_BUDGET` at 1x, 2x, 4x, 8x, 16x of 16,384, with
`LEXICAL_GLOBAL_POP_BUDGET` scaled alongside (1,024,000 / 2,048,000 /
4,096,000 / 8,192,000 / 16,384,000). Everything else untouched. Release
binary, `--approximate --top 50`, one target per row, **search wall clock is
the median of 3 runs**; each run reports its own `search Nms`.

Pool, cutoff and every found-or-missing verdict are **identical across all
five multiples on every target**, so the table is given once with the
multiple range in the header.

| target | probe phrase | 1x…16x pool | enumerated? | raw rank / score | rank-49 cutoff | in visible top 50? | search 1x | search 16x |
|---|---|---|---|---|---|---|---|---|
| `It's just a stupid game` | `hits justice dupe hid came` **(canonical)** | 15,710 | **no** | — | 0.915691888 | no | 1627 ms | 1683 ms |
| `recognize speech` | `wreck a nice beach` **(milestone's other half)** | enumerated at pool rank 27 | **yes, at every multiple** | 27 / 0.918313383 | 0.917045726 | **yes, printed rank 28** | 1523 ms | 1399 ms |
| `congratulations on your promotion` | `congratulations new wrap a motion` | ranked 439 | yes, at every multiple | 439 / 0.866674019 | 0.877365489 | no | 2829 ms | 2754 ms |
| `a whole lot of trouble` | `hole la tongue true able` | 13,247 | **no** | — | 0.895695875 | no | 1563 ms | 1436 ms |
| `I love you` | `now ill of` | ranked 299 | yes, at every multiple | 299 / 0.913463121 | 0.929627967 | no | 480 ms | 464 ms |

Wall clock, per target, all five multiples (median of 3, ms):

| target | 1x | 2x | 4x | 8x | 16x |
|---|---|---|---|---|---|
| `It's just a stupid game` | 1627 | 1420 | 1520 | 1503 | 1683 |
| `recognize speech` | 1523 | 1331 | 1363 | 1586 | 1399 |
| `congratulations on your promotion` | 2829 | 2556 | 2705 | 2767 | 2754 |
| `a whole lot of trouble` | 1563 | 1445 | 1392 | 1453 | 1436 |
| `I love you` | 480 | 427 | 446 | 489 | 464 |

That is run-to-run noise on a loaded host (the raw triples for the canonical
target are 1624/1627/2025, 1446/1420/1378, 1520/1563/1519, 1503/2097/1482,
1717/1683/1674). There is **no cost curve**: 16x the global budget costs
nothing measurable, which is the same fact as the pool being unchanged, seen
from the other side.

Corpus load is 433-522 ms across all runs, against the ~419 ms baseline in
the spec and the 402-507 ms in [w-8f3c61](../items/w-8f3c61.md) §1. The 1x
pool of 15,710 reproduces the `--top 50` figure recorded in
[w-6b2f04](../items/w-6b2f04.md) and [w-8f3c61](../items/w-8f3c61.md) §1
exactly, which is the cross-check that the 1x baseline is the same tree they
measured.

### The milestone's other half

`wreck a nice beach` for `recognize speech` **survives every budget point**,
and it already holds at 1x: enumerated at pool rank 27 with score 0.918313383
against a cutoff of 0.917045726, printed at rank 28 of the visible 50. Its
score is *above* the rank-49 cutoff — it is a genuine ranking, not an
enumeration-and-cut. The budget lever does not change its rank by one at any
multiple.

### `approximate_output_is_locked`

Holds, at 1x, on the restored tree:

```
$ cargo test --release --test corpus_integration approximate_output_is_locked
test approximate_output_is_locked ... ok
$ cargo test --release --test approx_determinism
test approximate_mode_is_reproducible_across_processes ... ok
test result: ok. 2 passed; 0 failed
```

`approximate_finds_classic_madgab_resegmentation` still **fails** (8 passed,
1 failed), which is the milestone and the reason this item exists. Nothing was
re-baselined. `cargo fmt`, `cargo clippy` and doctests do not exist on this
host and are not claimed; see [../environment-notes.md](../environment-notes.md).

## Measurement 2: separating the two ceilings, and the saturation point

`LEXICAL_GLOBAL_EMISSION_BUDGET` is inert because
`LEXICAL_COMBINATIONS_PER_SEGMENTATION` (= 64) is the real per-segmentation
ceiling and there are at most `SEGMENTATION_KEEP` (= 256) of them, so total
spend is capped at 16,384 by construction. Varying the two independently:

| global emission | per-seg emission | global pop | pool (canonical target) | search | clue |
|---|---|---|---|---|---|
| 16,384 (1x) | 64 | 1,024,000 | 15,710 | 1573 ms | missing |
| 262,144 (16x) | 64 | 16,384,000 | **15,710** | 1523 ms | missing |
| 16,384 (1x) | **1,024 (16x)** | 16,384,000 | **15,710** | 1503 ms | missing |
| 262,144 (16x) | 1,024 (16x) | 16,384,000 | **100,576** | 2368 ms | missing |
| 1,048,576 (64x) | 4,096 (64x) | 65,536,000 | 175,882 | 2750 ms | missing |
| 4,194,304 (256x) | 16,384 (256x) | 262,144,000 | **177,029** | 2812 ms | missing |

Reading the table:

- Rows 1-2: the global budget is not the binding term. Nothing changes.
- Row 3: the per-segmentation ceiling is the *only* binding term among these
  three, and lifting it 16x while the global budget stays at 1x still changes
  nothing, because the global budget then binds instead. This is the proof
  that 1x is already sufficient *for the current per-segmentation ceiling* —
  there is no slack being wasted and no headroom being bought.
- Row 4: with both lifted, the search does 6.4x the work for **+0.8 s** — so
  the lever really is cheap — and the pool grows 6.4x. Still missing.
- Rows 5-6: **the pool saturates at ~177,000.** 64x buys 3.4% more candidates
  over 16x; 256x buys 0.6% more over 64x. A saturated pool is a statement
  about the *shape* of the reachable set, not about its size, and the shape is
  set by `LEXICAL_BRANCH_KEEP = 10`.

The same rows for `recognize speech` (probe `wreck a nice beach`): found at
pool rank 27 with score 0.918313383 and printed at rank 28 at **every** one of
the six configurations, including 256x. Rank 27 is also invariant. Neither
the milestone's blocker nor its other half is budget-sensitive in either
direction.

## Why the budget cannot be the answer, stated as arithmetic

Three quantities, all read from the shipped source, compose into a closed
bound on what any budget multiple can buy:

1. Total emissions the loop can ever spend is
   `SEGMENTATION_KEEP * LEXICAL_COMBINATIONS_PER_SEGMENTATION = 256 * 64 =
   16,384`, because `emit_allowance` is
   `depth_ceiling.saturating_sub(held).clamp(1, LEXICAL_COMBINATIONS_PER_SEGMENTATION)`
   (`src/lib.rs:1015-1017`) and the schedule visits at most `SEGMENTATION_KEEP`
   segmentations. **`LEXICAL_GLOBAL_EMISSION_BUDGET` = 16,384 is therefore
   already the exact maximum, and every multiple above 1x is unreachable
   spend, not unspent budget.** This is why the 1x/2x/4x/8x/16x table is
   byte-identical rather than merely similar.
2. The reachable leaf set is bounded by the per-slot fan-out
   `LEXICAL_BRANCH_KEEP = 10` (`src/lib.rs:1210`), which is what the 177,029
   saturation measures.
3. The canonical clue needs walk rank 99 in one of its five slots, so it is
   outside that set by construction. This is
   [w-8f3c61](../items/w-8f3c61.md) §8's `T = (7, 0, 13, 99, 11)` and
   §8.3's 134,400-emission down-set bound, re-confirmed here from the other
   direction: the budget is not 8x short of the clue, the *order and width*
   are.

Note the interaction that makes §8.3's "grow the budget ~8x" arithmetic
misleading as a budget question. 134,400 is 8.2x the global budget but it is a
**per-segmentation** figure for one lucky segmentation; the global budget
would have to be `134,400 * (number of segmentations that must be funded that
deep)`, and no budget-derived quantity distinguishes the canonical
segmentation from the other 179 single-alignment structures — it is rank 151 of
256 by `span_objective`. So even taken at face value, 8x is not the
multiplicative factor that would be needed; the number that matters is
`134,400 x (a selectivity that does not exist)`.

## Reproducing

Everything above is reproducible from the commands shown, in this order,
with no committed source change. Patch the two constants to the literal
values for the multiple in question (the definitions are at `src/lib.rs:412`
and `src/lib.rs:414`, currently written as
`SEGMENTATION_KEEP * LEXICAL_COMBINATIONS_PER_SEGMENTATION` and
`SEGMENTATION_KEEP * LEXICAL_HEAP_POP_LIMIT`):

```sh
cargo build --release
MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
  ./target/release/madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE_PHRASES="wreck a nice beach" \
  ./target/release/madgab --approximate --top 50 "recognize speech"
```

Verbatim 1x baseline, the run every table above reconciles to:

```text
$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
    ./target/release/madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=15710
MADGAB_TRACE raw_cutoff rank=49 score=0.915691888 phrase="it justice too day mm"
(corpus loaded in 522ms; search 1627ms)
```

`missing candidates=N` is the pool size and the not-enumerated verdict, read
from the `raw phrase=` line alone (never from a concatenation with the
`raw_cutoff` line, which [w-8f3c61](../items/w-8f3c61.md) §1 records as a
past reporting hazard). `rank=R score=S` is the enumerated verdict, and
`R` is the rank in the pre-`select_diverse` deduplicated pool. Whether the
clue is in the **visible** top 50 is read from the numbered stdout list, which
is the only place `select_diverse`'s output is observable.

Sweep scripts used to produce the tables live in `/tmp/sweep.mjs`,
`/tmp/sweep2.mjs`, `/tmp/sweep3.mjs` with raw output in `/tmp/results*.json`.
They are scratch, deliberately not committed, and they make no
phrase-specific decision: the phrases are only ever passed to the existing
`MADGAB_TRACE_PHRASES` diagnostic and matched against stdout.

## Constraint proofs on this branch

- `git diff HEAD -- src/` is **empty**. `git status --short` is empty. The
  only committed path is this file.
- No `zz_*` test, no sweep script, no `.bench/` tree, no `MADGAB_*`
  diagnostic added to the branch. The `MADGAB_TRACE_PHRASES` variable used
  above is pre-existing shipped code (`src/lib.rs:1264`), invoked as an
  environment variable only.
- No phrase-specific special-casing anywhere: the budget constants are
  integers, and the report's phrases are measurement probes passed to a
  pre-existing diagnostic. `git grep -i -E` over the branch's diff finds no
  occurrence of any acceptance word outside prose in this document.
- Nothing on the traversal order, the width schedule, the per-slot list
  contents, the cost model, or any `axes::*`/scoring function was touched.
  `LEXICAL_BRANCH_KEEP`, `SEGMENTATION_KEEP` and `SPAN_SHORTLIST` were
  **held at their shipped values for the whole of measurement 1**, which is
  the required sweep, and varied only in the diagnostic rows explicitly
  labelled as such in measurement 2.

## Handoff

1. **Do not open a budget front. This closes the family.** The global
   emission/pop budget is not the binding constraint at any multiple from 1x
   to 256x; it is not 8x short of the milestone, it is *unreachable*, and the
   pool saturates at ~177,000 candidates because of
   `LEXICAL_BRANCH_KEEP = 10`.
2. **The milestone's real remaining cost is width, not depth.** The cheapest
   configuration measured anywhere on this front that would push the
   canonical `hid` node is full per-slot width, already measured by
   [w-6b2f04](../items/w-6b2f04.md) at 17.2 s and 10x the wall clock, and by
   [w-9d4e17](../items/w-9d4e17.md) at 0.7 s *only* combined with a
   sum-cost order. That pair — full width **plus** a non-myopic order — is
   the same conclusion [w-3b8e15](../items/w-3b8e15.md) is already chasing
   from the cost-model side, and this item's number is the budget-side
   confirmation that the budget is not a third option.
3. **Keep the two findings separate in the next pass.** Reaching the clue
   (width/order) and ranking it (score, gap of 0.0958 against the 0.9157
   cutoff) are two problems owned by two different fronts. A fix to either
   one alone does not close the milestone.
4. `LEXICAL_GLOBAL_EMISSION_BUDGET` and `LEXICAL_GLOBAL_POP_BUDGET` are, as
   of this measurement, **provably non-binding in the shipped
   configuration**. That is arguably worth a comment in `src/lib.rs` in a
   later pass — it is not this item's to make, since it is a `src/` change.
