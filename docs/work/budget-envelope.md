# The budget envelope: what an 8x global emission budget buys, and what it costs

Measurement for [w-be6d21](../items/w-be6d21.md), branch
`madgab-budget-envelope`, worktree `/workspace/madgab-budget-envelope`, from
`aac97e1`. **Documentation only — no `src/` change is committed.** The budget
constants were varied locally to take the measurements and restored; the tree
is clean and `git diff HEAD -- src/` is empty.

## Verdict, first

**One word: affordable-but-insufficient.**

Not affordable is refuted — 16x on both ceilings costs **+0.8 s** (1.57 s to
2.37 s) and grows the pool **6.4x** to 100,576. Sufficient is refuted much
harder than the 8x question required: the canonical clue is **not enumerated
at 1x, 2x, 4x, 8x, 16x, 64x or 256x**, and the candidate pool *saturates* at
177,029 by 64x, which is the size of the set the search can reach at all.

**Cheapest multiple at which the canonical clue is enumerated: there is none,
up to 256x.** And the reason is not a shortfall to be bought — it is that
`LEXICAL_GLOBAL_EMISSION_BUDGET` is already at its maximum reachable value at
**1x**, so every multiple above 1x multiplies a number that cannot be spent.

**Is a knob genuinely needed? No, and nothing should be landed.** The
measurement's conclusion is that the shipped constants are correct and
non-binding; the defect is `LEXICAL_BRANCH_KEEP = 10` (per-slot width), which
is [w-9d4e17](../items/w-9d4e17.md)'s and [w-3b8e15](../items/w-3b8e15.md)'s
to change, not this branch's. So this branch lands **documentation only**,
`docs/work/budget-envelope.md`, with no constant change to review or inherit.
The `src/lib.rs` in this worktree is modified only while a sweep is running
and is reverted before every commit; `git diff HEAD -- src/` is empty at every
commit on this branch and the branch's entire diff against its base is one
added file.

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

**For the milestone clue, neither: it is a width/reach problem, and the
scoring question is a second, independent wall behind it.** The spec's
distinction was "enumerated-and-cut" (scoring) versus "genuinely ranked"
(budget). The canonical clue turns out to be a third case, which is worth
stating precisely because it relocates the milestone:

- The clue is **not enumerated** at any multiple from 1x to 256x, so it is
  never cut by the score cutoff and never ranked. It is absent from the pool
  itself.
- The cutoff arithmetic is therefore *not* what is blocking it today, but it
  would be the next wall. The clue's score is **0.819901** against a rank-49
  cutoff of **0.915691888** on `It's just a stupid game` — a gap of 0.0958.
  Even a hypothetical enumeration fix would surface it at rank ~400, not in
  the visible 50.

**But the sweep does return the enumerated-but-cut case, on other real
probes, and that is the part that relocates work rather than closing it.**
`congratulations new wrap a motion` (pool rank 439, score 0.866674019) and
`now ill of` for `I love you` (pool rank 299, score 0.913463121) are both
genuine readable resegments that the search **produces** and the scorer
**discards**, at every budget multiple including 1x. A scorer admitting them
would have to lower the cutoff by 0.0107 and 0.0162. So
[w-04f83f](../items/w-04f83f.md) / [w-2e5b93](../items/w-2e5b93.md) are not
working on a hypothetical: there is a measured, enumerated, cut population
for them to recover, and the budget lever cannot recover any of it. Full
table and provenance below.

## Measurement 1: the required sweep, 1x / 2x / 4x / 8x / 16x

`LEXICAL_GLOBAL_EMISSION_BUDGET` at 1x, 2x, 4x, 8x, 16x of 16,384, with
`LEXICAL_GLOBAL_POP_BUDGET` scaled alongside (1,024,000 / 2,048,000 /
4,096,000 / 8,192,000 / 16,384,000). Everything else untouched. Release
binary, `--approximate --top 50`, one target per row, **search wall clock is
the median of 3 runs**; each run reports its own `search Nms`.

Pool size, ENUMERATED and RANKED are **identical across all five multiples
on every target**; only the wall clock moves, and only by run-to-run noise.
The 25 rows are given in full rather than collapsed, because the answer to
"does 8x buy anything" has to be readable one row at a time.

| multiple | emission budget | pop budget | target | probe | search wall clock | pool | ENUMERATED | pool rank / score | RANKED (visible top 50) |
|---|---|---|---|---|---|---|---|---|---|
| 1x | 16,384 | 1,024,000 | `It's just a stupid game` | `hits justice dupe hid came` **(canonical)** | 1627 ms | 15,710 | **NO** | — | **no** |
| 2x | 32,768 | 2,048,000 | `It's just a stupid game` | `hits justice dupe hid came` **(canonical)** | 1420 ms | 15,710 | **NO** | — | **no** |
| 4x | 65,536 | 4,096,000 | `It's just a stupid game` | `hits justice dupe hid came` **(canonical)** | 1520 ms | 15,710 | **NO** | — | **no** |
| 8x | 131,072 | 8,192,000 | `It's just a stupid game` | `hits justice dupe hid came` **(canonical)** | 1503 ms | 15,710 | **NO** | — | **no** |
| 16x | 262,144 | 16,384,000 | `It's just a stupid game` | `hits justice dupe hid came` **(canonical)** | 1683 ms | 15,710 | **NO** | — | **no** |
| 1x | 16,384 | 1,024,000 | `recognize speech` | `wreck a nice beach` **(other half)** | 1523 ms | 13,251 | **yes** | 27 / 0.918313383 | **yes** (printed rank 28) |
| 2x | 32,768 | 2,048,000 | `recognize speech` | `wreck a nice beach` **(other half)** | 1331 ms | 13,251 | **yes** | 27 / 0.918313383 | **yes** (printed rank 28) |
| 4x | 65,536 | 4,096,000 | `recognize speech` | `wreck a nice beach` **(other half)** | 1363 ms | 13,251 | **yes** | 27 / 0.918313383 | **yes** (printed rank 28) |
| 8x | 131,072 | 8,192,000 | `recognize speech` | `wreck a nice beach` **(other half)** | 1586 ms | 13,251 | **yes** | 27 / 0.918313383 | **yes** (printed rank 28) |
| 16x | 262,144 | 16,384,000 | `recognize speech` | `wreck a nice beach` **(other half)** | 1399 ms | 13,251 | **yes** | 27 / 0.918313383 | **yes** (printed rank 28) |
| 1x | 16,384 | 1,024,000 | `congratulations on your promotion` | `congratulations new wrap a motion` | 2829 ms | 7,725 | **yes** | 439 / 0.866674019 | **no** |
| 2x | 32,768 | 2,048,000 | `congratulations on your promotion` | `congratulations new wrap a motion` | 2556 ms | 7,725 | **yes** | 439 / 0.866674019 | **no** |
| 4x | 65,536 | 4,096,000 | `congratulations on your promotion` | `congratulations new wrap a motion` | 2705 ms | 7,725 | **yes** | 439 / 0.866674019 | **no** |
| 8x | 131,072 | 8,192,000 | `congratulations on your promotion` | `congratulations new wrap a motion` | 2767 ms | 7,725 | **yes** | 439 / 0.866674019 | **no** |
| 16x | 262,144 | 16,384,000 | `congratulations on your promotion` | `congratulations new wrap a motion` | 2754 ms | 7,725 | **yes** | 439 / 0.866674019 | **no** |
| 1x | 16,384 | 1,024,000 | `a whole lot of trouble` | `hole la tongue true able` | 1563 ms | 13,247 | **NO** | — | **no** |
| 2x | 32,768 | 2,048,000 | `a whole lot of trouble` | `hole la tongue true able` | 1445 ms | 13,247 | **NO** | — | **no** |
| 4x | 65,536 | 4,096,000 | `a whole lot of trouble` | `hole la tongue true able` | 1392 ms | 13,247 | **NO** | — | **no** |
| 8x | 131,072 | 8,192,000 | `a whole lot of trouble` | `hole la tongue true able` | 1453 ms | 13,247 | **NO** | — | **no** |
| 16x | 262,144 | 16,384,000 | `a whole lot of trouble` | `hole la tongue true able` | 1436 ms | 13,247 | **NO** | — | **no** |
| 1x | 16,384 | 1,024,000 | `I love you` | `now ill of` | 480 ms | 8,670 | **yes** | 299 / 0.913463121 | **no** |
| 2x | 32,768 | 2,048,000 | `I love you` | `now ill of` | 427 ms | 8,670 | **yes** | 299 / 0.913463121 | **no** |
| 4x | 65,536 | 4,096,000 | `I love you` | `now ill of` | 446 ms | 8,670 | **yes** | 299 / 0.913463121 | **no** |
| 8x | 131,072 | 8,192,000 | `I love you` | `now ill of` | 489 ms | 8,670 | **yes** | 299 / 0.913463121 | **no** |
| 16x | 262,144 | 16,384,000 | `I love you` | `now ill of` | 464 ms | 8,670 | **yes** | 299 / 0.913463121 | **no** |

Every column except wall clock is a constant function of the multiple. Wall
clock is noise on a loaded host: the canonical target raw triples are
1624/1627/2025, 1446/1420/1378, 1520/1563/1519, 1503/2097/1482,
1717/1683/1674. There is **no cost curve**: 16x the global emission budget
and 16x the global pop budget cost nothing measurable, which is the same
fact as the pool being unchanged, seen from the other side.

Corpus load is 433-522 ms across all runs, against the ~419 ms baseline in
the spec and the 402-507 ms in [w-8f3c61](../items/w-8f3c61.md) §1. The 1x
pools reproduce the `--top 50` figures already recorded in
[w-6b2f04](../items/w-6b2f04.md) and [w-8f3c61](../items/w-8f3c61.md) §1
exactly (`It's just a stupid game` 15,710; `congratulations on your
promotion` 7,725), which is the cross-check that the 1x baseline is the same
tree they measured.

### The enumerated-but-cut case, which this sweep does exhibit

The coordinator flagged this as the most valuable outcome available, so it is
worth being exact about which of the two it is, per probe:

| probe | target | ENUMERATED | RANKED | pool rank | score | rank-49 cutoff | gap |
|---|---|---|---|---|---|---|---|
| `hits justice dupe hid came` | `It's just a stupid game` | **no** | no | — | (0.819901) | 0.915691888 | (0.0958) |
| `wreck a nice beach` | `recognize speech` | yes | **yes** | 27 | 0.918313383 | 0.917045726 | **-0.0013** |
| `congratulations new wrap a motion` | `congratulations on your promotion` | **yes** | **no** | 439 | 0.866674019 | 0.877365489 | +0.0107 |
| `hole la tongue true able` | `a whole lot of trouble` | **no** | no | — | — | 0.895695875 | — |
| `now ill of` | `I love you` | **yes** | **no** | 299 | 0.913463121 | 0.929627967 | +0.0162 |

Provenance of the probes, stated so the table is not over-read: the two
canonical probes (`hits justice dupe hid came`, `wreck a nice beach`) are the
milestone's own acceptance clues. The other three are real, readable
resegmentations chosen as *depth* probes — the deepest genuinely-resegmented
proposal the search produced for that target on a deeper `--top 1000`/`1500`
run — so that "how deep does the pool reach" has a fixed landmark. That is
why `congratulations new wrap a motion` (rank 439 of a `--top 1000` run) and
`now ill of` (rank 299 of `--top 600`) are enumerated at `--top 50` while
`hole la tongue true able` (rank 1250 of `--top 1500`) is not: the pool itself
is a function of `top_n` through `structure_wording_allowance`, so a probe
deeper than the `--top 50` pool cannot be in it. The `(0.819901)` in the
first row is **quoted from [w-8f3c61](../items/w-8f3c61.md) §8.3**, not
measured here, because the clue is not enumerated at any multiple and so has
no score in the pool to read.

So all three regimes are present, and they are stable across the whole 1x-16x
range:

- **enumerated-and-ranked**: `wreck a nice beach`. Score *above* the cutoff.
  Nothing to fix; it is a working result at 1x.
- **enumerated-and-cut**: `congratulations new wrap a motion` (rank 439) and
  `now ill of` (rank 299). These are genuine, readable resegmentations that
  the search *does* produce and the score cutoff discards. **This is the
  scoring problem, and it is real and independent of the budget**: no budget
  multiple moves either one, because both are already in the pool at 1x. For
  a scorer to admit them, the cutoff would have to fall by 0.0107 and 0.0162
  respectively.
- **not enumerated**: the milestone clue. This is the reach problem, and the
  budget is not what limits it.

The important consequence for the next pass: **the milestone has two
independent walls, in series, and the budget lever touches neither.** A
scoring fix alone leaves the canonical clue absent; a reach fix alone would
surface it at rank ~400, behind the same 0.0958 gap that stops the two
probes above. [w-04f83f](../items/w-04f83f.md) /
[w-2e5b93](../items/w-2e5b93.md) own the scoring wall and
[w-3b8e15](../items/w-3b8e15.md) owns the reach wall; neither front can close
the milestone alone.

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

- **`git diff aac97e1 HEAD -- src/` is 0 bytes, and the branch's whole diff
  against its base is one added file:**

  ```text
  $ git diff --name-status aac97e1 HEAD
  A	docs/work/budget-envelope.md
  $ git diff aac97e1 HEAD -- src/ | wc -c
  0
  ```

  `git status --short` is empty and `git status --porcelain --ignored` shows
  only `Cargo.lock` and `target/`, both gitignored.
- **On the transient `M src/lib.rs`.** The multiplier experiment edits
  `src/lib.rs` in the working tree, because that is the only way to vary a
  compile-time constant. The edit exists **only while a sweep is running**: the
  sweep restores the file from an in-memory copy of the original as its last
  step, and it is restored again before every commit. A coordinator or another
  agent reading `git status` mid-sweep will see `M src/lib.rs`; that is the
  experiment in flight, not a pending change, and it is never staged. If a
  future pass wants the experiment off this branch entirely, park it on a
  scratch branch — but note that even a reverted, unstaged, uncommitted
  experiment cannot reach any commit on this branch, which is what the fence
  actually protects.
- No `zz_*` test, no sweep script, no `.bench/` tree, no `MADGAB_*`
  diagnostic added to the branch (`git ls-files` matches none). The
  `MADGAB_TRACE_PHRASES` variable used above is pre-existing shipped code
  (`src/lib.rs:1264`), invoked as an environment variable only.
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
3. **Keep the three regimes separate in the next pass.** Measured, stable
   across 1x-256x, on real probes: *enumerated-and-ranked* (`wreck a nice
   beach`, rank 28, already correct at 1x); *enumerated-and-cut*
   (`congratulations new wrap a motion` at rank 439, `now ill of` at rank 299
   — a measured, recoverable population that a scoring change can address and
   no budget change can); and *not-enumerated* (the canonical clue, blocked
   by `LEXICAL_BRANCH_KEEP = 10`). Reaching the clue (width/order,
   [w-3b8e15](../items/w-3b8e15.md) / [w-9d4e17](../items/w-9d4e17.md)) and
   ranking it (score, gap 0.0958 against the 0.9157 cutoff,
   [w-04f83f](../items/w-04f83f.md) / [w-2e5b93](../items/w-2e5b93.md)) are
   two problems in series, owned by two different fronts. A fix to either one
   alone does not close the milestone.
4. **No constant should be landed from this item.** The shipped
   `LEXICAL_GLOBAL_EMISSION_BUDGET` and `LEXICAL_GLOBAL_POP_BUDGET` are
   correct and provably non-binding; raising them is a no-op, so there is no
   knob here worth reviewing. The one-line comment in `src/lib.rs` recording
   *why* they are non-binding (the `256 * 64 = 16,384` identity) would be a
   genuine improvement, but it is a `src/` change and belongs to whichever
   front next owns `src/lib.rs` — suggest folding it into
   [w-3b8e15](../items/w-3b8e15.md)'s diff rather than opening a branch for
   it.
