# w-2f7a10 — front 4e8a52 (sweep / shape-class coupling)

Branch `madgab-sweep-4e8a52`, tip `79309a1`, pushed to `origin` and verified with
`git ls-remote origin madgab-sweep-4e8a52` → `79309a1044b5858b20c7dd2de8d0f8d0cb305b75`.
Base `aa662a4` (`post-milestone-acceptance`). One commit, `src/lib.rs` only,
189 added / 15 removed lines. Nothing merged into the branch; `main` untouched;
`scratch/2f7a10-base`, `zzparent` and `7c70784` never merged or rebased onto. This
report and all probes are untracked and off the branch.

## 1. The gate, re-derived on aa662a4

`sweep_index` on this base is byte-for-byte the rule in the brief:

```text
sweep_index(w, per, nth, phase) = 10 + (nth * ceil((w-10)/per) + phase) % (w-10)
```

Read-only env-gated trace (`MADGAB_SLOT_TRACE` / `MADGAB_SLOT_WORDS`, both removed
from the tree before committing) on the default release path, canonical target,
256 funded segmentations:

```text
offered coverage tuples                    3404
refused by build's additive total_budget     355  (10.4 %)
segmentations whose slot lists contain ALL FOUR of
  hits / dupe / hid / came                        13
of those 13, segmentations whose reserve reachable
  index set contains ANY of the four requested
  coordinates                                     0
```

Per-segmentation, the 13 segmentations that hold the requested wording:

| phase | widths | requested ranks (slot:rank) | total substitution cost | in reserve's reachable set |
|---|---|---|---|---|
| 71  | 98,160,160,160,160,93  | hits@0:8 dupe@3:18 hid@4:99 came@5:19 | 1.369520 | 0 of 4 |
| 135 | 160,160,7,160,160,93   | hits@1:78 dupe@3:10 hid@4:99 came@5:19 | 1.319520 | 0 of 4 |
| 136 | 160,7,160,13→160,160,160 | hits@0:6 dupe@2:10 hid@3:99 came@5:50 | 1.119520 | 0 of 4 |
| 151 | 160,7,160,160,160,93   | hits@0:7 dupe@2:13 hid@3:99 came@4:11 | 1.119520 | 0 of 4 |
| 168 | — | hits@0:6 dupe@2:10 hid@3:99 came@4:5  | 0.919520 | 0 of 4 |
| (13 total) | | | 0.919520 – 1.369520 | **0 of 52** |

So, on this base:

* **candidate lists are not the gate** — all four coordinates are present, at ranks
  6/7/8, 10/13/18/22/32, 99 and 5/11/19/50, all below their slot's width;
* **the additive cost bound is not the gate** — the requested tuple costs
  0.919520 – 1.369520 against a `total_budget` of 1.5 in all 13 segmentations, and
  the bound refuses 10.4 % of offered tuples, not any of these;
* **the sweep is the gate** — 0 of the 52 (segmentation, coordinate) pairs is
  reachable. Rank 99 in a width-160 slot and rank 13 in a width-160 slot are the
  two the brief names; both are OUT here too.

The three-gate discrimination reproduces. Ranks 7 / 13 / 99 / 11 on widths
`[160,7,160,160,93]` match the inherited diagnosis exactly; they differ from the
`7 / 13 / 99 / 11` reported off the reserve-shape arm only because that arm is not
integrated here.

## 2. The scheme

The measured defect has two parts, and they are separable.

**(a) The class coupling.** One index was written into every member of a shape
class, so every tuple the reserve emitted was a *diagonal* of its class's index
rectangle. A pairing is a *point* of that rectangle. This is not a cap problem:
raising `EMIT_PROFILE_MAX_DEEP` cannot help while every class spends one index.

**(b) The per-slot residue class.** `stride = ceil(span / per)` grows with the
width, so a wide slot's reachable set is one residue class of a growing step.

Part (b) is, on measurement, **not** what confines anything. The `phase` term has
coefficient 1, so for any stride the set `{nth*stride + phase mod span}` over
`span` consecutive phases is the whole list, for every width. Per-word sweep rates
measured on the default path, parent vs branch, two targets, 512 segmentations:

```text
              list-presences   swept by the reserve
              parent  branch   parent  branch
  hits            221      221    2.7 %   3.2 %
  dupe            154      154    9.1 %   7.8 %
  hid             110      110    5.5 %   5.5 %
  came            188      188    1.1 %   2.1 %
```

Marginal per-slot reach is, as expected, unchanged to within noise. So the change
fixes (a) and deliberately leaves the stride of the class draw alone, because the
stride is what makes a *single* phase spread over the list rather than spend one
window of it — and a structure is typically funded one or two times, so
within-phase spread is worth more than per-phase novelty.

**Chosen scheme.** `sweep_index` gains a `member` argument (the member's position
in the class) and each member rotates at its own rate:

```text
index(member) = 10 + (nth*ceil(span/per) + member + rate(member, span) * phase) % span
rate(0, span) = 1
rate(j, span) = the next integer > rate(j-1, span) coprime to span
```

with the span taken from the class's *narrowest* slot, so every member's index is
legal in every member.

* **Deterministic** — `rate` is a pure function of `(member, span)`, both integers
  the search already has; no clock, no hash seed, no allocation.
* **Coprimality** is what makes the coverage claim a property of *every* width
  rather than of the widths that divide evenly: a rate sharing a factor with the
  span visits only the residues reachable by that factor.
* **Differing rates** are what stop a class rotating as a rigid block. The offsets
  between a class's members' indices are themselves a function of the phase, so a
  set of deep coordinates at unrelated ranks is expressible, not only a set at
  adjacent ranks.
* `member 0` keeps the parent's index exactly, so the reserve is a superset of the
  old coverage one coordinate at a time — the same continuity argument 2f7a11 used
  for its arm.

**Alternatives evaluated and rejected.**

| scheme | per-slot coverage | class pairing | single-phase spread | verdict |
|---|---|---|---|---|
| keep AP stride, add `member` offset only | already full (phase coeff 1) | members adjacent ranks only | kept | rejected: cannot express 99 and 50 in one class |
| keep AP stride, per-member coprime rate (**chosen**) | full, every width | members at phase-dependent offsets | kept | chosen |
| replace stride with unit step | full, every width | members adjacent only | **lost** — 16 contiguous ranks out of 150 | rejected on the within-phase argument above |
| low-discrepancy / bit-reversal over the product | would need `span^deep` draws | full | n/a | rejected: needs ~75 draws per class against a 16-draw reserve |

**Cost.** Emissions: unchanged (`EMIT_PROFILE_RESERVE` still 16 per segmentation,
same class walk, same order, `build`'s additive `total_budget` comparison
untouched, same per-segmentation and global allowances). Per call: one extra
`usize` argument, one `%`, and `rate`'s short coprime walk bounded by
`EMIT_PROFILE_MAX_DEEP` iterations. Wall clock below.

## 3. The regression test, red on aa662a4

Three unit tests at the `coverage_tuples` / `sweep_index` boundary, naming no word,
phrase or target, over synthetic all-wide width vectors:

* `the_coverage_sweep_covers_each_slot_and_pairs_its_class_members` — depths 1–6,
  widths `vec![SPAN_SHORTLIST; depth]`, a run of `span` phases: the union over
  phases of the index sets placeable in each slot equals that slot's whole list
  above the floor, **and** at least one emitted tuple per depth ≥ 2 has its deep
  coordinates at pairwise different ranks;
* `one_phase_of_the_coverage_sweep_is_spread_over_the_list` — a single phase
  reaches at least a third of the list's indices, so the schedule cannot be
  quietly reduced to a window;
* `every_sweep_rate_is_coprime_to_the_span_it_is_used_on` — the arithmetic the
  coverage claim rests on, for every span 1..150 and member 0..3.

Shown RED on `aa662a4` (scratch tree from `git archive aa662a4`, tests transplanted
onto the parent's 4-argument `sweep_index`):

```text
test tests::depth_profile_reserve_is_bounded_by_named_arithmetic ... FAILED
  [30, 30] reused one rank                                    (the inverted
  shared-index assertion, red at src/lib.rs:4134 on the parent)
test tests::one_phase_of_the_coverage_sweep_is_spread_over_the_list ... FAILED
test tests::the_coverage_sweep_covers_each_slot_and_pairs_its_class_members ... FAILED
  depth 2: no emitted tuple put its deep coordinates at pairwise different ranks
test result: FAILED. 49 passed; 3 failed
```

GREEN on the branch: `cargo test --release --lib` → **53 passed, 0 failed**.

**Precisely which clause is red where.** The `pairwise different ranks` clause is
red on the parent and green on the branch. The `per-slot coverage` clause is
**green on the parent too** — as section 2 says, the old phase coefficient already
tiled the list over a long enough run. I kept the coverage clause because it is the
property the new per-member rates must not break, not because it discriminates.
`one_phase_of_the_coverage_sweep_is_spread_over_the_list` is red on the parent
because the parent has no member axis at all, so one phase draws 16 indices, not 64.

**Relation to 2f7a11's criterion-4 test.** `2f7a11` wrote
`depth_profile_reserve_emits_pairings_not_only_diagonals` on widths
`[160,7,160,160,93]` over 64 phases, asserting a tuple with ≥ 3 deep coordinates
not all equal. My change **subsumes the "not only diagonals" half** of that
assertion and generalises it to all-wide vectors, and that test's *stated* reason
for the parent's redness applies verbatim to my parent. It does **not** subsume the
other half: their test needs a **4-deep** class, which needs `EMIT_PROFILE_MAX_DEEP`
3 → 4 and a class order that funds `{0,2,3,4}`; I changed neither, so their test
would still be red on my branch. My test is not a rename of theirs — different
widths, different run length, a per-slot coverage clause and a coprimality clause
they do not have, and it is red on `aa662a4` on its own.

## 4. Effect on the default path

Release build, default approximate path. Pool dump is a read-only env-gated
(`MADGAB_POOL_DUMP`) write placed after the `phrase_signature` dedup and before
`select_diverse`, applied identically to both arms and **removed from the branch
tree before committing**. Non-intrusiveness: visible stdout **byte-identical with
and without the probe on all 7 targets, on both arms** (`cmp`).

```text
target                          pool parent -> branch   aggregate breadth metric*
It's just a stupid game          17906 -> 17906  ( 0)      1047.38 -> 1048.13
recognize speech                 15906 -> 15909  (+3)         5.25 ->    5.25
put it back on the shelf         15633 -> 15642  (+9)         7.00 ->    7.00
the cat sat on the mat           16262 -> 16259  (-3)         2.60 ->    2.40
when the rain finally stopped    15610 -> 15605  (-5)         7.00 ->    7.17
a whole lot of trouble           16149 -> 16159  (+10)        2.29 ->    2.33
he was a big fat man             19171 -> 19165  (-6)         1.00 ->    1.00
```

\* occurrences / number of distinct word positions the ten watch words ever occupy,
summed over the pool. "Slot position" is measured as the word's index inside the
emitted wording; the pool dump carries no span identity.

Per-word slot-position histograms (occurrences / distinct positions), parent →
branch:

```text
It's just a stupid game
  hits      46/1 [0:46]                    ->  49/2 [0:48 1:1]      +1 position
  dupe      14/3 [2:6 3:7 4:1]             ->  11/2 [2:6 3:5]        -1
  hid        5/2 [0:4 4:1]                 ->   5/2 [0:4 3:1]         =
  came     165/5 [3:26 4:68 5:49 6:16 7:6] -> 167/5 [3:26 4:68 5:51 6:16 7:6]  =
  justice 8138/3 [1:4276 2:2944 3:918]    -> 8145/3 [1:4280 2:2947 3:918]
put it back on the shelf
  taught    28/1 [0:28]                    ->  32/2 [0:31 1:1]      +1 position
  honour     3/2 [2:1 3:2]                 ->   1/1 [2:1]            -1
  delve     11/3 [4:3 5:5 6:3]             ->   9/3 [4:2 5:4 6:3]     =
the cat sat on the mat
  hits       2/1 [1:2]                     ->   3/2 [1:2 2:1]       +1 position
  tickets    3/1 [1:3]                     ->   2/1 [1:2]            -1
  aar        1/1 [3:1]                     ->   0/0 []          LOST (1 occurrence)
  taught     4/2 [5:1 7:3]                 ->   4/2 [5:1 7:3]         =
  honour     3/2 [2:1 3:2]                 ->   3/2 [2:1 3:2]         =
when the rain finally stopped
  aar       13/1 [3:13]                    ->  13/1 [3:13]            =
  taught    27/4 [4:4 5:4 6:8 7:11]        ->  27/4 [4:3 5:5 6:8 7:11]  =
  came       2/1 [2:2]                     ->   3/1 [2:3]             =
a whole lot of trouble
  aar       13/6 [2:2 5:4 6:4 7:1 8:1 9:1] ->  13/6, same               =
  taught     1/1 [3:1]                     ->   0/0 []          LOST (1 occurrence)
  honour     1/1 [3:1]                     ->   0/0 []          LOST (1 occurrence)
he was a big fat man
  honour     1/1 [6:1]                     ->   1/1 [6:1]             =
  aar        1/1 [4:1]                     ->   1/1 [4:1]             =
recognize speech
  hits      20/4 [1:2 2:4 3:9 4:5]         ->  20/4 [1:2 2:4 3:8 4:6]   =
```

**Requested wording present?**

```text
"wreck a nice beach"  (recognize speech)     pool: parent TRUE  branch TRUE   top 50: TRUE
"Hits Justice Dupe Hid Came" (stupid game)   pool: parent FALSE branch FALSE  top 50: FALSE
```

Three single-occurrence witnesses are lost (`aar` on the mat, `taught` and `honour`
on a whole lot of trouble) and three words gain a slot position (`hits` twice,
`taught` on the shelf). Net effect on the aggregate metric: +0.07, 0.00, 0.00,
−0.20, +0.17, +0.04, 0.00. **This is a wash, and I am not claiming it as an
improvement.**

## 5. Suites at the tip

| suite | base `aa662a4` | tip `79309a1` |
|---|---|---|
| `cargo test --release --lib` | 50 passed, 0 failed | **53 passed, 0 failed** |
| `--test corpus_integration` | 10 passed, **1 failed** | 10 passed, **1 failed** (same test) |
| `--test exact_determinism` | 1 passed, 0 failed | **1 passed, 0 failed** |
| `--test approx_determinism` | 2 passed, 0 failed | **2 passed, 0 failed** |
| `--test no_phrase_hard_coding` | 6 passed, 0 failed | **6 passed, 0 failed** |

Red at the base, by name:

* `approximate_finds_classic_madgab_resegmentation` — red at the base and red at my
  tip. **This is the item's target, not a guard.** Top 50 begins
  `it justice too bad aim`, `it justice too pad aim`, … — the requested wording is
  still not produced.
* `approximate_pool_reaches_alternatives_past_the_opening_slot_width` — **green at
  the base** on `aa662a4` (the brief anticipated it might be red at some points; on
  this base it is not) and **green at my tip**. Not re-baselined, not touched.

No test was re-baselined, relaxed, skipped or deleted. `approximate_output_is_locked`
is untouched and green. No acceptance test was edited at all. `cargo fmt`,
`cargo clippy` and doctests **do not exist on this host** and I ran none.

Fence scan of `aa662a4..79309a1`: one file (`src/lib.rs`), 189 added lines, **0**
containing `recognize`, `speech`, `wreck`, `nice`, `beach`, `stupid`, `justice`,
`dupe`, `hits`, `hid`, `came`, `mad gab`, `phrase_signature`, `zz_` or
`MADGAB_TRACE`. No permanent `MADGAB_TRACE`-style instrumentation added; the two
probes used here were removed from the tree before the commit and live only in
`/tmp/opencode/*.patch` and the untracked parent-arm tree.

## 6. Wall clock

Interleaved parent-versus-branch, alternating arm order each repetition, one
discarded warm-up per binary, medians of 7. Parent arm built by
`git archive aa662a4 | tar -x -C /workspace/zzparent4e8a52` — never by merging and
never by checking a worktree onto that commit.

```text
It's just a stupid game   parent 2330 ms   branch 2292 ms   -1.6 %
the cat sat on the mat    parent 2319 ms   branch 2330 ms   +0.5 %
```

Both within run-to-run noise (individual runs span 2097–3219 ms). No wall-clock
regression.

## 7. Verdict on the milestone criteria

* `wreck a nice beach` for `recognize speech` — **already met at the base**, still
  met: in the pool and in the visible top 50, `approximate_finds_recognize_speech_resegmentation` green.
* `Hits Justice Dupe Hid Came` for `It's just a stupid game` — **NOT met.** Not in
  the pool, not in the top 50, before or after.
* Regression test at an appropriate boundary — met for the *mechanism*
  (`coverage_tuples` / `sweep_index`), red on the base, not for the milestone
  behaviour.
* No phrase-specific hard-coding — met.
* Relevant tests pass — met except the one target test, which was already red.
* On the accumulation branch, not `main` — the coordinator's call; this branch is
  pushed and merges nothing.

**So the milestone is not reached, and this change does not move it.** I am not
softening that: the change is validated as correct, general, cheap and safe, and it
does not measurably improve the outcome.

## 8. The single most valuable next measurement

**How many of the reserve's emitted tuples are `build`-refused, split by depth,
on the default path, with a per-member-rate sweep in place — and, paired with it,
how many of them are refused because their coordinates are at *adjacent* ranks
rather than unrelated ones.**

The reason this is the next thing and not another sweep change: the sweep is no
longer the binding gate. With one shared index removed, a pairing is expressible,
and the measured outcome did not move, which means the binding constraint has moved
downstream to `build`'s additive `total_budget` comparison. A 3-deep tuple pays
three non-zero substitution costs against the same 1.5 bound; 2f7a11 measured that
refusal share rising 10.4 % → 25.1 % when they added a fourth deep slot, and this
branch's 10.4 % is measured on the same base. Until the depth-conditional refusal
rate is known per sweep scheme, no further change to the index schedule can be
evaluated: a more expressive schedule that is more refusable buys nothing, and that
is exactly the trade this front cannot yet price.

Concretely: instrument the default path to report, per offered tuple, its depth,
its total cost, and whether `build` accepted it — parent and branch, seven targets.
That number decides whether the next front is a cost-accounting change or another
coverage change, and it is the number this item still does not have.
