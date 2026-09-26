VERDICT: **4** — the requested wording `hits justice dupe hid came` is **4 coordinates deep simultaneously** (per-slot ranks `7 / 0 / 13 / 99 / 11` in the traversal's own contribution-sorted candidate lists) on `post-milestone-acceptance` at `33cb6cb`, while `EMIT_PROFILE_MAX_DEEP = 3` (`src/lib.rs:212`) bounds the reserve at 3; the four-deep figure in `docs/work/items/w-2f7a10.md` **still holds** on the PUNCH-gained tree, and `coverage_tuples` offers exactly **14** tuples for that structure across a full run, none deeper than 3.

Independent measurement arm for [w-5d2a91](docs/work/items/w-5d2a91.md). No code was moved:
`git diff 33cb6cb...scratch/5d2a91-measure -- src tests` is **empty**. This branch is
an arm, not a line of work, and must never be merged.

## Base commit and build

| | |
| --- | --- |
| base | `post-milestone-acceptance` @ `33cb6cb9ad49b8d59f6baf455cef51b3612c197a` |
| branch | `scratch/5d2a91-measure` (forked from `33cb6cb`, no commits to `src/` or `tests/`) |
| build | `cargo build --release` in `/workspace/madgab-5d2a91` (host has no `rustup`; `/tmp` is `noexec`, so the pristine binary was copied to `/workspace/5d2a91-probe/`) |
| path | default approximate path, `madgab --approximate --top 50 <target>`, all six targets and the canonical one |
| artefacts | `measurements/5d2a91/` (pool dumps, analysis scripts, the probe patch) |

## Provenance of every number — which is pool, which is probe

Two classes, stated separately because this queue has retracted a reachability
claim for conflating them.

**A. From the deduplicated default-path pool (admissible).** Everything in the
six-target table and the canonical-structure figures: pool size, distinct
structures, raw cutoff, fill rank, member counts, best rank, slot confinement.
These are membership of the deduplicated pool the release binary actually
produced, read by a hook placed immediately after the dedup `retain` and
immediately before `select_diverse` (`src/lib.rs:1904`). The hook does not
reorder, filter, re-score or select; `select_diverse(clues, top_n)` is still
returned on the untouched `clues`.

**B. From a probe (line-level trace inside the default path).** The per-slot
candidate-list widths, the per-slot contribution-sorted ranks, and the tuple
count `coverage_tuples` offers. These cannot be read off the pool — the pool
does not record which index produced a member — so they come from a read-only,
env-gated `eprintln!` block inside the segmentation loop, printing values it
recomputes from data already in scope (`slots`, `widths`, `build`) plus a second
call to the pure function `coverage_tuples` with the same arguments. It is a
line-level trace inside the default approximate path, which is the other form
the admissibility rule admits.

**There is a third, fully pre-existing instrument.** `MADGAB_TRACE_SPANS` /
`MADGAB_TRACE_WORDS` / `MADGAB_TRACE_PHRASES` are in the base tree at `33cb6cb`
and were used unmodified, with no probe compiled in at all, to cross-check the
pool-side numbers. Every figure below is reproduced by both routes where both
apply.

### Non-intrusiveness of the probe, proved

`madgab-pristine` (built from unmodified `src/lib.rs` at `33cb6cb`) vs the
probe build, with all `ZZ_*` variables unset, stdout **and** stderr, four
targets, `It's just a stupid game` / `wreck a nice beach` /
`put it back on the shelf` / `he was a big fat man`:

```
BYTE-IDENTICAL  It's just a stupid game
BYTE-IDENTICAL  wreck a nice beach
BYTE-IDENTICAL  put it back on the shelf
BYTE-IDENTICAL  he was a big fat man
```

The only normalisation applied is the wall-clock line
`(corpus loaded in NNNms; search NNNms)`, which is not output of the program in
any reproducible sense and varies between two runs of the *same* binary. Every
other byte matches, including the search's own `MADGAB_TRACE` lines. And with the
probe **enabled** the visible proposal list is still byte-identical to the
pristine binary's, so the probe is not merely dormant, it is inert:

```
stdout with probe ENABLED vs pristine: BYTE-IDENTICAL
```

## Number 1 — depth, width, and simultaneous deep count

Structure `cuts = [3, 10, 13, 15, 19]`, i.e. spans
`[(0,3),(3,10),(10,13),(13,15),(15,19)]`, for
`hits justice dupe hid came` against `It's just a stupid game`.
The structure is retained at segmentation rank **151** of 256.

Per-slot, in the traversal's own index space — the `slots` list **after** the
`contribution` sort at `src/lib.rs:1465`, whose index 0 is the traversal's best
index for that slot:

| slot | span | requested word | **candidate-list width** | **rank in the traversal's list** | deep? |
| --- | --- | --- | --- | --- | --- |
| 0 | `0-3` | `hits` | **160** | **7** | **yes** |
| 1 | `3-10` | `justice` | **7** | **0** | no — *is* the best index |
| 2 | `10-13` | `dupe` | **160** | **13** | **yes** |
| 3 | `13-15` | `hid` | **160** | **99** | **yes** |
| 4 | `15-19` | `came` | **93** | **11** | **yes** |

**Simultaneously deep coordinates: 4.** The answer is **4, not 3.** Only slot 1
sits on the traversal's best index, and it does so because `justice` is the best
alternative of the narrowest list (width 7) as well as the correct one.

For completeness, the same five words in the **lattice walk order**
(`SpanEdge::matches`, before the `contribution` sort) — this is the
`MADGAB_TRACE` view, and it is the origin of the queue's "hid rank 85 vs 99"
confusion recorded in [w-2f7a10](docs/work/items/w-2f7a10.md): **not new
evidence, a different list.**

| slot | 0 | 1 | 2 | 3 | 4 |
| --- | --- | --- | --- | --- | --- |
| walk-order rank | 22 | 0 | 64 | **85** | 7 |

### Does the four-deep figure still hold on a tree that has since gained a scoring axis?

**Yes, unchanged, and there is a structural reason as well as the measurement.**

Measured: the widths `[160, 7, 160, 160, 93]` and the ranks `7 / 0 / 13 / 99 / 11`
are **bit-for-bit the figures the 18:20Z pass of [w-2f7a10](docs/work/items/w-2f7a10.md)
recorded from the implementer's own `zz_pool` dump**. The four-deep figure
survives PUNCH.

The reason is visible in the source rather than inferred: the per-slot ordering
is by `SlotAlt::contribution` (`src/lib.rs:719`), and **`contribution` contains no
`PUNCH` term** — its terms are `SIMILARITY_PER_WORD`, `WORD_NOVELTY`,
`FAMILIARITY`, `CLOSED_CLASS`, `SHAPE`. PUNCH is applied only in the `axes::`
combination in `bound` and in the final scorer, so the axis moved the *scores*
of wordings without moving the *shape of the index space* the traversal walks.
An implementer should not expect re-running a depth measurement after an
additive axis to move it, and should expect the pool-side numbers to move a lot:
they did.

### `coverage_tuples` for that structure, and what the reserve can express

```text
ZZ_DEPTH seg_rank=Some(151) widths=[160, 7, 160, 160, 93]
        sorted_ranks=[Some(7), Some(0), Some(13), Some(99), Some(11)]
        walk_ranks=[Some(22), Some(0), Some(64), Some(85), Some(7)]
        phase=151 reserve=16 max_deep_const=3 offered=14 built=14 deepest_tuple=3
```

- The structure is realised by **exactly one** segmentation in the whole run
  (the probe fired once, at `phase = 151`), so "across a full run" is one
  `coverage_tuples` call.
- Tuples **offered**: **14** of the `EMIT_PROFILE_RESERVE = 16` allowance.
- Tuples that survive `build`'s additive `total_budget` comparison: **14** — no
  cost rejection at all for this structure on this head.
- **Deepest tuple offered: 3** deep coordinates. Never 4.
- So the reserve is not budget-starved (14 of 16 spent, 0 cost-rejected) and not
  order-starved; it is **shape-expressiveness-starved**. With
  `EMIT_PROFILE_MAX_DEEP = 3` and the breadth-before-depth walk at
  `src/lib.rs:396`, every one of the 14 tuples keeps at least two slots at index
  0, and the requested alignment needs four slots off index 0. The 4-deep shape
  is outside the class of shapes the reserve can express at all.

This is a *shape* statement, not a claim that raising the constant would work:
`w-2f7a10`'s own 17:20Z pass already measured that a 4-deep profile reserve on
this tree bought zero of the six pairs, and `w-1c3e77` should be free to refute
this if the arithmetic says otherwise.

The requested tuple is separately **cost-admissible** (`w-d4a90b`: per-word
`0.2 / 0.0 / 0.15 / 0.3695 / 0.4`, `sub_cost_total = 1.11951981416578961` against
`total_budget 1.5`, every per-word cost <= `0.5`), so budget is not the gate
either.

### Canonical structure, from the deduplicated default-path pool

`madgab --approximate --top 50 "It's just a stupid game"`, release, `33cb6cb`:

| quantity | value |
| --- | --- |
| pool (deduplicated) | **17,827** |
| distinct structures | **336** |
| raw cutoff (pool rank 49) | `0.898008919`, `it justice too bed same` |
| **fill rank** (pool rank of the 50th visible proposal) | **61** |
| worst visible (post-`select_diverse`) | `it said thus tas too dame` |
| **members of `[3,10,13,15,19]`** | **48** |
| best member | `each justice too add gain`, `0.858128467`, global rank 3376 |
| gap to raw cutoff | `-0.039880` |
| **requested wording** | **absent from all 17,827** |

Reproduced by the pre-existing `MADGAB_TRACE_PHRASES` with no probe compiled in:
`raw phrase="hits justice dupe hid came" missing candidates=17827` and
`raw_cutoff rank=49 score=0.898008919 phrase="it justice too bed same"`.
The pool size `17827` and the cutoff line are also identical to
[w-d4a90b](docs/work/items/w-d4a90b.md)'s independent measurement on this head, so
the probe build and the pristine build produce the same pool.

**Per-word membership inside that structure, and where it can occur** — this is
*sharper* than the parent's numbers, and it is the part an implementer should
read twice:

| word | members in `[3,10,13,15,19]` | slot distribution inside the structure |
| --- | --- | --- |
| `hits` | **1** | slot 0 only |
| `justice` | **48** | slot 1 only (i.e. all of them) |
| `dupe` | **0** | — |
| `hid` | **0** | — |
| `came` | **0** | — |

All ten pairs of the requested wording's own five words are **0** members in that
structure except `hits+justice` (1). So on this head the loss is not "one of the
pairs is missing": three of the five words are **not enumerated in this structure
at all**, even though each is individually affordable and (for `hid`, rank 99 of
160) inside a list the traversal can reach. `w-2f7a10`'s parent figures
(`dupe` 14, `hid` 5, `came` 157) were measured at `fe4ad78` and are stale; the
PUNCH axis and the pool changes since have moved all three to zero. Any claim
that "the reserve already emits `hid` alone in this structure" is **false on this
head** and must be re-measured, not inherited.

## Number 2 — six-target table

None of these six is an acceptance phrase. These are exactly the six
[w-2f7a10](docs/work/items/w-2f7a10.md)'s baseline arm used, chosen so the numbers
are directly comparable to that arm. Release build, `33cb6cb`,
`madgab --approximate --top 50`, deduplicated default-path pool. **Every cell in
this table is class A (pool), not a probe number.**

| target | pool | distinct structures | `--top 50` cutoff (score @ pool rank 49) | fill rank | witness | members | best rank | best score | slot confinement |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `a whole lot of trouble` | 16,173 | 335 | `0.881655059` | 49 | `aar` | 13 | 10,451 | `0.667453837` | 3/3 at slot 6 of its top structure `[2,4,5,9,11,14,16]`; pool-wide slots {2,5,6,7,8,9} |
| `the cat sat on the mat` | 16,639 | 331 | `0.889990259` | 49 | `tickets` | 2 | 2,090 | `0.860918885` | 2/2 at slot 1 pool-wide — **confeined to one slot class** |
| `put it back on the shelf` | 15,826 | 315 | `0.891867581` | 53 | `taught` | 32 | 1,774 | `0.866365144` | **31/32 at leading slot 0 pool-wide**; 2/2 at slot 0 in its top structure `[3,5,11,13,16]` — **confeined to the leading slot** |
| `when the rain finally stopped` | 14,781 | 350 | `0.911554031` | 49 | `taught` | 19 | 4,798 | `0.847357317` | slots {5,6,7} = {5,6,8}; 4/4 at slot 7 in its top structure |
| `he was a big fat man` | 19,229 | 306 | `0.892581157` | 104 | `honour` | 2 | 14,407 | `0.672670317` | slots {1,6}; 1/1 at slot 1 in its top structure `[3,4,7,9,12,15]` |
| `what are you going to do` | 13,929 | 311 | `0.895518806` | 66 | **`aar`** | **3,075** | **4,079** | `0.848791164` | **736/736 at slot 1** of its top structure `[3,5,7,10,12,14,16]`; pool-wide slots {1: 3039, 2: 36} — **the required well-populated, single-slot-class witness** |

"fill rank" is the pool rank of the 50th proposal the user actually sees, i.e.
the pool position of the last slot `select_diverse` filled. It differs from the
raw cutoff rank (49) because `select_diverse` reorders and rebalances after the
pool is scored; the two are quoted separately, as in [w-d4a90b](docs/work/items/w-d4a90b.md).

### The row the brief specifically asked for

`what are you going to do`, witness **`aar`**: **3,075** members pool-wide — by
some margin the most populated witness in the table — and inside its most
populated structure it occupies **736 of 736** occurrences at **slot 1**. It is
present, affordable, enumerable and heavily represented, and it is confined to a
single slot class. That is the general property any fix must serve, stated in
the pool's own terms and naming no acceptance phrase:

> a word that is well-populated in a boundary structure can still be confined to
> a single slot class within that structure.

`taught` on `put it back on the shelf` is the second, independent instance and
the one that reproduces the earlier arm: **32** members (the parent arm measured
28) with **31 of 32** at the **leading** slot — the same confinement as the
canonical `hid`. Neither is an acceptance example.

### Zeros, reported rather than substituted

Present in the dumps, **0 members pool-wide**, so no witness is offered:

| word | targets where it is 0 of 6 |
| --- | --- |
| `misfit` | **all six** (confirms the parent arm's zero on every target) |
| `perdu` | **all six** (the parent arm's witness for target 6: it is **gone** on this head) |
| `capture` | **all six** |
| `copper` | **all six** |
| `delve` | 4 of 6 — absent on `the cat sat on the mat`, `when the rain finally stopped`, `he was a big fat man`, `what are you going to do`; 1 member on `a whole lot of trouble`, 10 on `put it back on the shelf` |
| `tickets` | 4 of 6 — absent on `a whole lot of trouble`, `put it back on the shelf`, `when the rain finally stopped`, `he was a big fat man` |
| `louis` | 5 of 6 — absent on the first five, 9 members on `put it back on the shelf` |
| `honour` | 3 of 6 — absent on `a whole lot of trouble`, `when the rain finally stopped`, and (trivially) the target it witnesses |

Three of the parent arm's six witnesses have **regressed to zero on the targets
they were measured on**: `perdu` (2 -> 0) on `what are you going to do`,
`delve` (1 -> 1, but 3 -> 0 elsewhere) and `louis` (8 -> 0) on
`a whole lot of trouble`. `tickets` went 3 -> 2. The `w-2f7a10` table is
therefore **not** a current baseline and must not be used as one; the table above
is.

## Comparability and deltas against the recorded numbers

| quantity | `w-2f7a10` arm (`fe4ad78`, parent) | `w-d4a90b` (`f5b9eaa`) | **this arm (`33cb6cb`)** |
| --- | --- | --- | --- |
| canonical pool, `--top 50` | 17,906 | 17,827 | **17,827** |
| canonical structure members | 47 | — | **48** |
| best member score | `0.878128467` | — | **`0.858128467`** |
| best member global rank | 3,108 | — | **3,376** |
| `hid` members in that structure | 5 (4 leading) | — | **0** |
| `dupe` members in that structure | 14 (3 non-leading) | — | **0** |
| `came` members in that structure | 157 (5 slots) | — | **0** |
| `taught` on `put it back on the shelf` | 28, leading only | — | **32, 31/32 leading** |
| per-slot widths `[3,10,13,15,19]` | `[160,7,160,160,93]` | — | **`[160,7,160,160,93]`** |
| per-slot ranks `hits justice dupe hid came` | `7/0/13/99/11` | — | **`7/0/13/99/11`** |

The best member's score is exactly `0.020000000` lower than the parent's, and
[w-d4a90b](docs/work/items/w-d4a90b.md) measures `PUNCH` at weight `0.10` applied
as `0.10 * (punch - 1.0)`, costing a flat `0.020000000` to a wording that is not
6 of 6 monosyllabic. That is the PUNCH axis, exactly, and it is why the *scores*
moved while the *index-space shape* did not. It is a nice independent
confirmation that the axis is live on this head and that the two kinds of number
in this report must not be conflated.

## Commands, so every number above is re-derivable

Base `33cb6cb`, worktree `/workspace/madgab-5d2a91`.

```sh
# 0. pristine binary, from unmodified src/
cargo build --release
mkdir -p /workspace/5d2a91-probe && cp target/release/madgab /workspace/5d2a91-probe/madgab-pristine

# 1. class-B numbers (probe) -- see measurements/5d2a91/zz-probe.patch
git apply measurements/5d2a91/zz-probe.patch && cargo build --release
ZZ_DEPTH_SPANS='0-3,3-10,10-13,13-15,15-19' ZZ_DEPTH_WORDS='hits,justice,dupe,hid,came' \
  ./target/release/madgab --approximate --top 50 "It's just a stupid game" 2>&1 >/dev/null
git checkout src/lib.rs        # the probe is NEVER committed

# 2. class-A numbers, with NO probe compiled in, pre-existing trace only
MADGAB_TRACE_SPANS='0-3,3-10,10-13,13-15,15-19' MADGAB_TRACE_WORDS='hits,justice,dupe,hid,came' \
MADGAB_TRACE_PHRASES='hits justice dupe hid came' \
  ./target/release/madgab --approximate --top 50 "It's just a stupid game"

# 3. class-A pool dumps (needs the probe's ZZ_POOL_OUT block)
for t in "It's just a stupid game" "a whole lot of trouble" "the cat sat on the mat" \
         "put it back on the shelf" "when the rain finally stopped" \
         "he was a big fat man" "what are you going to do"; do
  ZZ_POOL_OUT=/tmp/pool.tsv ./target/release/madgab --approximate --top 50 "$t" >/dev/null 2>&1
done

# 4. analysis
node measurements/5d2a91/analyse.js <pool.tsv> hits 3,10,13,15,19 "hits justice dupe hid came"
node measurements/5d2a91/wit.js <pool.tsv> 'delve|tickets|louis|aar|honour|perdu|taught|misfit' "<target>"
```

Raw dumps as measured: `measurements/5d2a91/pool-stupidgame.tsv` and
`pool-0.tsv` … `pool-5.tsv`, in that target order.

## Fence

- `git diff 33cb6cb...scratch/5d2a91-measure -- src tests` -> **empty**.
- `git diff 33cb6cb...scratch/5d2a91-measure --name-only` -> `REPORT-5d2a91.md` and
  `measurements/5d2a91/` only.
- No constant, threshold, budget, test, `axes::*` weight, `select_diverse` rule or
  `adjacency.rs` line was touched. The probe is a patch file under
  `measurements/5d2a91/`, applied only to a scratch build and reverted.
- No `test` suite is reported here and none was run: this item asked for two
  numbers, and quoting a suite would imply a change. The base tree's regression
  status is [w-d4a90b](docs/work/items/w-d4a90b.md)'s to report, and it is
  unchanged by this arm.
- **`cargo fmt`, `cargo clippy` and doctests do not exist on this host** (see
  [docs/environment-notes.md](docs/environment-notes.md)). They are not claimed.
- Nothing was merged, into anything, and this branch must never be merged: it is
  an arm. `main` and `post-milestone-acceptance` are untouched and are the
  coordinator's to integrate.

## Handoff

The two numbers, for [w-1c3e77](docs/work/items/w-1c3e77.md) to check itself
against — **not** to quote as agreement, since this arm shares no code with it:

1. **4** deep coordinates, widths `[160, 7, 160, 160, 93]`, sorted ranks
   `7 / 0 / 13 / 99 / 11`; `coverage_tuples` offers **14** tuples for the
   canonical structure over a full run, all 14 cost-admissible, deepest **3**;
   canonical structure has **48** members; requested wording **0 of 17,827**.
2. The six-target table above, with `aar` on `what are you going to do`
   (**3,075** members, **736/736** at one slot) and `taught` on
   `put it back on the shelf` (**32** members, **31/32** leading) as the two
   single-slot-class confinement witnesses, and `misfit` / `perdu` / `capture` /
   `copper` as honest zeros on all six.

The sharpest new fact for the implementation front: on this head, three of the
five words of the requested alignment are **not enumerated in the canonical
structure at all**, so "each word is reachable alone there" is no longer true and
must not be assumed. The sharpest caution: the four-deep *shape* limit is real
and measured, but `w-2f7a10` has already recorded that raising
`EMIT_PROFILE_MAX_DEEP` on this tree bought **zero** of the six pairs, so this
report does **not** predict that the constant is the fix. A refutation of
hypothesis 1 by `w-1c3e77` is a good outcome, and this arm's numbers are the ones
to refute it against.
