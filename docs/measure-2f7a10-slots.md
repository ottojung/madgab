# w-2f7a10 measurement front — per-slot feasibility and the gate

SCRATCH. Measurement and diagnosis only. Nothing here is a fix and nothing
here may be merged. Two branches, both off-line instruments:

* `scratch/2f7a10-slots` — forked from the accumulation head `65e5531`
  (parent arm, `EMIT_PROFILE_MAX_DEEP = 3`, diagonal reserve);
* `scratch/2f7a10-slots-after` — forked from `madgab-pairing-2f7a10` at
  `3b14482` (the sibling front's points-not-diagonals reserve, max-deep 4,
  per-slot rotation), carrying the **same** instrumentation so the two arms
  are compared by the same probe.

No `src/` behaviour is changed, no test assertion is touched, no
`axes::*` / `select_diverse` / `adjacency.rs` / share cap / `EMIT_*`
constant is moved, no budget is re-set, and nothing is merged into or out of
any other branch. `scratch/2f7a10-base` was not merged.

## What is instrumented, and where

One read-only hunk in `src/lib.rs`, in two places.

1. Immediately after the per-slot ranked lists are built for a segmentation
   and before the reserve is called, keyed by `ZZ_STRUCT` (or every
   structure, with `ZZ_ALL=1`):

   * `STRUCT cuts=… widths=… total_budget=…`
   * `SLOT s rank i word w cost c` — the whole ranked candidate list of
     every slot, in rank order;
   * `RANKSET slot s word w rank i cost c` / `… ABSENT` — whether a word is
     ever *offered* in that slot at all, and at what rank;
   * `COST …` — the additive arithmetic of the coordinate combination named
     by `ZZ_TUPLE`: each slot's chosen cost, each slot's best-word cost
     `m_j`, `sum_{j != k} m_j + cost_{k,i}`, and the total against
     `total_budget`.
2. Around the reserve call, capturing the tuples `coverage_tuples`
   *actually returned* (not re-derived):
   `RESERVE cuts=… widths=… reserve=… phase=… n=…` and one
   `RESERVE_TUPLE [..]` line per tuple.

Plus the same pool dump the baseline arm uses, `ZZ_POOL_OUT`, sitting after
the sort and after the `phrase_signature` dedup and before `select_diverse`:
the **deduplicated pool the release binary produces on the default path**.

Non-intrusiveness, checked rather than asserted: the 50 visible clues are
byte-identical with every probe set and unset, on both arms
(`cmp` of the `--approximate --top 50` stdout, both arms).

## Admissibility, per number

* **Pool membership** (pool sizes, member counts, per-word occurrences and
  slot positions, the `misfit` zeros): deduplicated default-path pool of a
  release build. Admissible.
* **Per-slot lists, per-slot ranks, the reserve's returned tuples, and the
  cost arithmetic**: line-level traces inside the default approximate path
  of a release build. Admissible as traces, and marked as such below.
* **Shortlist ranks from a re-budgeted or re-sampled harness**: none. There
  is no wider harness on either branch. The `sweep_index` sets in §3 are
  arithmetic recomputed from the traced `widths`, `reserve` and `phase` of
  the single call the traversal actually made; they are not a second
  enumeration and no tuple is emitted from them.

Build note: `CARGO_TARGET_DIR` under `/workspace` (`/tmp` is noexec here).

## 1. Per-slot feasibility table, canonical structure `[3, 10, 13, 15, 19]`

Release build, default path, `--approximate --top 50`, target
`It's just a stupid game`. Trace, not inferred: the structure is
materialised **once in the whole run** (one `STRUCT` line, one `RESERVE`
call, `phase = 151`, `reserve = 16`).

Per-slot widths, and the per-slot rank of the five requested words. `—` means
the word is **not in that slot's candidate list at all**; a number is its
rank in that slot's ranked list, with its substitution cost:

| slot | width | `hits` | `justice` | `dupe` | `hid` | `came` |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 160 | **7** (0.200) | — | — | — | — |
| 1 | 7 | — | **0** (0.000) | — | — | — |
| 2 | 160 | — | — | **13** (0.150) | — | — |
| 3 | 160 | — | — | — | **99** (0.369520) | — |
| 4 | 93 | — | — | — | — | **11** (0.400) |

Identical on both arms (parent `65e5531` and sibling `3b14482`): the widths
`[160, 7, 160, 160, 93]` and the ranks `7 / 0 / 13 / 99 / 11` reproduce, and
so does the fact that **each word appears in exactly one slot's list**.

Two structural facts about slot 1 that matter for everything below:

* its width is **7**, and `sweep_index` returns `None` for any
  `width <= LEXICAL_BRANCH_STAGE_0 = 10`, so **every shape class containing
  slot 1 is dropped outright**. That is why the parent emits 14 tuples from a
  reserve of 16 (two of the sixteen classes contain slot 1), and why slot 1's
  coordinate is 0 in every tuple on both arms. `justice` is reachable only as
  a slot's *best* word.
* the first ranks of each list: slot 0 `each, eats, itch, ich, it, which,
  rich, hits`; slot 1 `justice, justin, justices, justin's, adjusts,
  justify, just`; slot 2 `too, two, to, tube, coop, coupe, too-too, poop`;
  slot 3 `add, and, said, ad, ed, end, at, bad`; slot 4 `gave, games,
  games', gain, gay, aim, give, same`.

Pool membership, the same structure, deduplicated default-path pool
(parent 17,907 / after 17,647, 47 members on both arms):

| word | occurrences | distinct slot positions, pool-wide |
| --- | --- | --- |
| `hits` | 46 | {0} |
| `dupe` | 14 | {2, 3, 4} |
| `hid` | 5 | {0, 4} |
| `came` | 165 | {3, 4, 5, 6, 7} |
| `misfit` | 0 | absent |

So three of the four requested off-target coordinates are individually
realised somewhere in the pool (`hits` at slot 0, `dupe` at slot 2, `came`
at slot 4) and `hid` is realised only at slots 0 and 4 — **never at the
slot-3 position it is asked for** — and the four are never realised together.

## 2. Which gate rejects the requested coordinate combination

Requested tuple, in the traversal's own index space:
`(7, 0, 13, 99, 11)`.

**(a) LIST construction — REFUTED.** All four off-target words are in their
slot's ranked candidate list (§1: ranks 7, 13, 99, 11, all `< width`). The
per-slot lists are the traversal's own `slots[s]`, sorted by contribution
then alphabetically, and the requested words are in them at those ranks. No
width, rotation, phase or opening-width cut removes them. The lists *are*
the admission; the words are admitted.

**(c) COST bound — REFUTED, with the arithmetic.** `build` compares
`sum_s cost_s <= total_budget + 1e-9` additively. Traced values for this
structure (`total_budget = 1.5`):

| slot | index | word | cost | `m_j` (best-word cost) | `sum_{j != k} m_j + cost_{k,i}` | fits |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 7 | `hits` | 0.200000 | 0.083773 | 0.648352 | yes |
| 1 | 0 | `justice` | 0.000000 | 0.000000 | 0.532125 | yes |
| 2 | 13 | `dupe` | 0.150000 | 0.200000 | 0.482125 | yes |
| 3 | 99 | `hid` | 0.369520 | 0.101234 | 0.800412 | yes |
| 4 | 11 | `came` | 0.400000 | 0.147118 | 0.785007 | yes |

`m = [0.083773, 0.0, 0.2, 0.101234, 0.147118]`, `sum m = 0.532125`.
**Total 1.119520 ≤ 1.5**, slack **0.380480**. Every per-slot slack
`sum_{j != k} m_j + cost_k <= total_budget` holds with 0.65–1.02 to spare, so
the "affordable in the leading slot, unaffordable in a later slot" arithmetic
is **not** what rejects this tuple. Consistently, of the tuples the reserve
actually offered for this structure, **0 of 14 (parent) and 0 of 15
(after)** were cost-rejected: the offered totals run 0.63 → 1.47, i.e. the
bound is close but not binding here. (The deepest offered tuple on the
after arm, `bit justice taught mud i'm`, costs 1.4672 and was still
admitted.)

**(b) SWEEP — this is the gate.** `sweep_index` never returns the requested
coordinates, and the reason is structural rather than a tuning accident:

* `sweep_index(width, per, nth, phase) = 10 + (nth * stride + phase) % span`
  with `span = width - 10` and `stride = ceil(span / per)`. It is an
  **arithmetic progression modulo `span`, floored at 10**. On the after arm
  (`reserve = per = 16`, `phase = 151`, `COVERAGE_SLOT_ROTATION = 37`),
  recomputing it from the traced `widths`/`reserve`/`phase` of that one call:

  | slot | width | reachable as the class's shared head | reachable as a later member (own width, own rotation) | requested rank |
  | --- | --- | --- | --- | --- |
  | 0 | 160 | 11,13,19,21,25,31,37,41,43,49,51,55,61,67,71,73,78,79,81,84,85,90,91,101,111,121,131,141,151 | — | **7: in neither, and below the floor 10** |
  | 2 | 160 | same list as slot 0 | 15,25,35,…,155 | **13: head only** |
  | 3 | 160 | same list as slot 0 | 12,22,32,…,152 | **99: in neither** |
  | 4 | 93 | 13,19,25,31,37,43,49,55,61,67,73,78,79,84,85,90 | 13,19,25,31,37,43,49,55,60,61,66,67,72,78,84,90 | **11: in neither** |

  Three separate facts, each sufficient on its own for the coordinates that
  concern it:
  1. **Rank 7 is below the sweep floor.** The reserve only ever emits index
     `>= LEXICAL_BRANCH_STAGE_0 = 10`; ranks 0–9 are the traversal's own
     corner. `hits` at rank 7 is therefore *never* a reserve coordinate, in
     any phase, in any class — and the traversal reaches it only in the
     best-bound corner, which is why the pool has `hits` at slot 0 in 46
     candidates and never in combination with a deep word elsewhere.
  2. **Rank 99 is in neither progression.** A 160-wide slot is swept at
     stride 10, so its reachable set is one residue class mod 10; 99 is in
     none of the classes this phase produces. `hid` is at rank 99 of 160 —
     *offered, affordable, and unswept*.
  3. **Rank 11 is in neither progression for slot 4** (93-wide, stride 6,
     floored at 10: 13, 19, 25, …), and rank 13, which *is* reachable in
     slot 2, is reachable only as the class's **shared head**, i.e. only
     together with the same index 13 written into every other member of its
     class.

  And the class rule itself is the sharpest form: on the parent one index
  serves the whole subset, so a `k`-deep shape is necessarily a **diagonal**
  — the same rank in every deep slot; on the after arm the class's first
  slot keeps that shared head and the rest take their own index, but the head
  is still a single index shared with nothing and drawn from the class's
  *narrowest* width. Either way, the requested tuple needs four slots at
  three *mutually unrelated* ranks (7, 13, 99, 11), and no rule in the
  reserve can produce three unrelated ranks from one draw of a stride.

* The reserve gets **one** draw for this structure: the traversal
  materialises `[3, 10, 13, 15, 19]` exactly once (`phase = 151`, one
  `RESERVE` line, 14 tuples on the parent, 15 after). So the phase rotation,
  which is what is supposed to let successive segmentations tile the lists,
  has no second turn to give this structure.

**Verdict: gate (b), the sweep.** (a) and (c) are refuted by measurement.
The quantity that rejects the requested combination is the pair
*(index arithmetic of `sweep_index`, one-draw-per-structure)* — concretely,
`hid`'s rank 99 lies outside the residue class mod 10 that a 160-wide slot
is swept at on this phase, `came`'s rank 11 lies outside the stride-6
progression of a 93-wide slot, and `hits`'s rank 7 lies below the sweep
floor of 10 outright. On the parent the depth cap adds a second, independent
reason (the tuple is 4-deep, `EMIT_PROFILE_MAX_DEEP = 3`); on the after arm
that reason is gone — a 4-deep tuple *is* emitted, `bit justice taught mud
i'm` — and the wording is still absent, which is the cleanest available
evidence that **shape was never the binding constraint**.

## 3. The same confinement on other real targets

Pool-wide occurrences and the set of distinct slot positions a word ever
occupies, deduplicated default-path pool, release build, after arm
(`3b14482`). None of these targets contains either acceptance example.

| target | pool | witness | occurrences | distinct positions | reading |
| --- | --- | --- | --- | --- | --- |
| `It's just a stupid game` | 17,647 | `hits` | 46 | {0} | confined to one position |
| | | `hid` | 5 | {0, 4} | never at the slot it is asked for |
| `put it back on the shelf` | 15,285 | `taught` | 31 | {0} | **leading-slot only, in all 25 structures it appears in** |
| `when the rain finally stopped` | 15,167 | `aar` | 13 | {3} | confined to one mid position |
| `the cat sat on the mat` | 15,893 | `tickets` | 1 | {1} | one occurrence, one position |
| `a whole lot of trouble` | 15,749 | `delve` | 1 | {7} | one occurrence, one position |
| `he was a big fat man` | 19,013 | `honour` | 2 | {1, 6} | two positions, two occurrences |
| `what are you going to do` | 14,062 | `perdues` | **0** | absent | real zero |
| all seven | — | `misfit` | **0** | absent | real zero, on every target |

`taught` is the strongest non-canonical witness and it is not a
single-structure artefact: in every one of the 25 structures of
`put it back on the shelf` in which it occurs, it occurs at **slot 0 only**
(1–2 occurrences each). The baseline front's `perdues` figure does not
reproduce on the after arm — it is 0 there — which is recorded rather than
smoothed over; `misfit` is 0 on all seven targets on both arms, as the
baseline recorded on its six.

## 4. The general property, and the shape a regression test should take

**The property.** *A word that is offered in a slot's candidate list, and is
affordable under the additive cost bound, can still be unreachable in that
slot inside any emitted tuple, because the reserve's coverage is an
arithmetic sweep of each slot's list rather than the list itself: the index a
slot receives is a residue-class member of a stride that grows with the slot's
width and is drawn once per structure, and every index below the traversal's
opening width is excluded outright. A word is therefore confined to the slot
classes whose sweep happens to contain its rank — a leading-slot-only or
single-position word is the normal case, not an anomaly — and because a
shape class spends one draw, a set of deep coordinates at mutually unrelated
ranks is not expressible at any depth cap. Concretely, on the canonical
structure a word at rank 7 of 160 is never a reserve coordinate, a word at
rank 99 of 160 is offered and affordable and unswept, and the four deep
coordinates needed are 7 / 13 / 99 / 11 — three unrelated ranks — so the
pairing is not enumerated at `EMIT_PROFILE_MAX_DEEP` 3 or 4.*

**The test.** At the `coverage_tuples` / `sweep_index` boundary, as a shape
property over synthetic width vectors and many phases, naming no word,
target or phrase: *for a width vector with all slots well above the opening
width, the union over a run of phases of the index sets the reserve can place
in each slot covers each slot's whole list above the floor — i.e. a rank that
is inside a slot's width is reachable in that slot for some phase — and the
reserve can emit a tuple whose deep coordinates are pairwise different ranks
not all equal.* The present code fails both halves: the union over all phases
is a residue class, and the per-slot index sets are equal within a class. The
executable-boundary complement, at the pool level, is that a real target's
pool contains a candidate in which two off-target words occupy two different
non-leading slots *of the same structure*, neither of which is the modal word
of its slot — the property the existing
`approximate_pool_reaches_alternatives_past_the_opening_slot_width` states
for one word, restated for a pair. `taught` on `put it back on the shelf` is
the already-measured non-canonical witness for the confinement half.

## 5. Honest limits of this evidence

* The per-slot lists, the reserve's tuples and the cost arithmetic are
  **traces inside the default path**, not pool membership. They are
  admissible as traces; they are not membership claims, and no pool-membership
  claim in §1 or §3 rests on them.
* The reachable-index sets in §2 are arithmetic recomputed from the traced
  `widths`, `reserve` and `phase` of the one call the traversal made. They
  reproduce the emitted tuples exactly (every one of the 15 after-arm tuples
  is in those sets), which is the check that they are the right
  recomputation — but they were not themselves read off a second run.
* `hid`'s rank 99 is unswept *on this phase*. Because the structure is
  materialised once, the sweep gets one draw; had it been materialised at a
  different phase, 99 could have been reached, since 99 is a legal index in a
  160-wide list. So the claim is that the pairing is **not guaranteed to be
  enumerated**, and is measurably not enumerated on the default path, not
  that rank 99 is arithmetically unreachable in all circumstances.
* Slot 1's width of 7 is target-specific. The "class dropped when a member
  slot is not wider than the opening width" rule is general, but this
  structure is the one where it fires most.
* Both arms are single runs on a shared host. Wall clock was not measured
  here; criterion 7 belongs to the other front, and this front makes no
  timing claim.
* The after arm is `3b14482`, the sibling front's in-flight follow-up. If
  that branch moves, §2's after column must be re-taken; the parent column
  will not move.
* `cargo fmt`, `cargo clippy` and doctests do not exist on this host
  (`docs/environment-notes.md`); no claim is made about them. The
  instrumentation compiles clean under `cargo build --release` on both arms.

## Reproduction

```sh
cd /workspace/madgab-slots            # or checkout scratch/2f7a10-slots-after
CARGO_TARGET_DIR=$PWD/target-after cargo build --release

ZZ_ALL=1 ZZ_SLOT_OUT=/tmp/all.txt \
  ./target-after/release/madgab --approximate --top 50 "It's just a stupid game"

ZZ_POOL_OUT=/tmp/pool.txt \
ZZ_SLOT_OUT=/tmp/slot.txt \
ZZ_STRUCT="3,10,13,15,19" \
ZZ_WORDS="hits,justice,dupe,hid,came" \
ZZ_TUPLE="7,0,13,99,11" \
  ./target-after/release/madgab --approximate --top 50 "It's just a stupid game"
```
