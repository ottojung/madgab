# w-2f7a10 — front `c81e55` (design and investigation: what mechanism makes a deep resegmentation reachable *and ranked*?)

Branch `madgab-depthcap-c81e55`, created at `2bacbcb` (`post-milestone-acceptance`).
Base for every number below: `2bacbcb`, **release**, default approximate path
(`--approximate --top 50`, no re-budgeting, no instrumented harness). Nothing was
merged into `main` or into `post-milestone-acceptance`. No test was edited, relaxed,
re-baselined, skipped or deleted. No fix was implemented.

**This branch's `src/` delta is a clearly-marked measurement patch and is throwaway.**
It is one new file `src/probe.rs` plus five call sites in `src/lib.rs`, all gated on
`MADGAB_C81E55` (unset ⇒ the module opens no file and every call returns), plus a
second gate `MADGAB_C81E55_INJECT`. Its only purposes are (a) to dump the per-slot
score components of every candidate of every segmentation, and (b) to push one
*environment-named* index tuple for one segmentation through the ordinary `build`
so that the **production** scorer and the **production** `select_diverse` can be
asked what they make of a wording the search never emitted. No word, phrase or
target is named in `src/`. `tests/no_phrase_hard_coding` is 6/6 green with the patch
applied. Nothing here is proposed for integration.

---

## 0. Headline

**The requested clue is not unreachable. It is unwanted.** The production scorer,
asked directly, ranks `hits justice dupe hid came` at **9867 of 17907 candidates**
with score **0.819901291**, against a rank-49 cutoff of **0.915691888** — a gap of
**0.0958**, which is **11× the width of the entire visible top-50 band**
(0.9250 → 0.9110 post-`select_diverse`; 0.9157 → 0.9170 at the raw cutoff).

Consequently:

1. **No search-side mechanism can green `approximate_finds_classic_madgab_resegmentation`.**
   Not the depth cap, not the emission allowance, not the class walk, not the sweep,
   not the cost bound, not the segmentation layer. The wording's *shape* is excluded
   three times over (measured, §2), and its *score* is excluded a fourth time
   (measured, §3), and the fourth is the one that binds.
2. **The depth cap is not the constraint the item names.** For the target's own
   segmentations the class walk runs out of allowance at depth 1 (6 slots) or depth 2
   (5 slots). Raising `EMIT_PROFILE_MAX_DEEP` changes nothing for them, and the
   shape that carries the requested wording would need **30 draws of the 16 available**
   at *any* cap. This is arithmetic, and it is the prediction the sibling cap-at-4
   front should find.
3. **The green guard is one hundredth of a point from falling.** `wreck a nice beach`
   clears the raw cutoff by **0.00127** (raw rank 27, score 0.918313383, cutoff
   0.917045726). Any pool-changing fix is a coin-flip on the one milestone condition
   that currently passes.
4. The honest general statement of the defect, which *is* actionable and is stated in
   §6: **on 1 of 13 real targets the visible proposals contain no full resegmentation
   at all** — 0 of 50 on `It's just a stupid game`, against 29–50 of 50 on the other
   twelve. The objective, not the enumeration, is what refuses a resegmentation.

---

## 1. Reproduction on `2bacbcb`

```
$ cargo build --release
$ ./target/release/madgab --approximate --top 50 "It's just a stupid game"
  1. [0.925] it justice too bad aim
  2. [0.924] it justice too pad aim
  ...                                              # 50 lines, "hits justice dupe hid came" absent
$ ./target/release/madgab --approximate --top 50 "recognize speech"
  ...
  28. [0.918] wreck a nice beach                    # present
```

Read out with the base's own `MADGAB_TRACE_PHRASES` gate (already in `src/lib.rs` on
`2bacbcb`, no patch needed), which reports the rank and score in the **raw sorted,
deduplicated** candidate list and the rank-49 cutoff:

```
$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17907
MADGAB_TRACE raw_cutoff rank=49 score=0.915691888 phrase="it justice too day mm"

$ MADGAB_TRACE_PHRASES="wreck a nice beach" madgab --approximate --top 50 "recognize speech"
MADGAB_TRACE raw phrase="wreck a nice beach" rank=27 score=0.918313383
MADGAB_TRACE raw_cutoff rank=49 score=0.917045726 phrase="red 'cause i.'s peach"
```

So: 17 907 candidates, the 50th best scores 0.915692, the requested clue is not
among them, and the guard is 27th of 17 9xx with a **0.00127** margin.

Suites on this branch (probe applied, release): `--lib` **50 passed / 0 failed**;
`--test corpus_integration` **10 passed / 1 failed** —
`approximate_finds_classic_madgab_resegmentation`, the item's target and red on the
base too; `approx_determinism` 2/0; `exact_determinism` 1/0;
`no_phrase_hard_coding` 6/0. `cargo fmt`, `cargo clippy` and doctests **do not exist
on this host** and are not claimed (`docs/environment-notes.md`).

---

## 2. What the reserve actually offers, and why the requested shape is structurally excluded

Canonical target, 256 segmentations. `measurements/c81e55/It_s_just_a_stupid_game_.tsv`
holds one `SEG` row per segmentation, one `ALT` row per candidate per slot, and one
`TUPLE` row per index tuple offered to the pool by the reserve or the traversal.
`tools`: `analyze.js` (A, B, C), `landscape.js`, `axes.js`, `fullreseg.js`.

### 2.1 the slot-count distribution and the class walk's arithmetic

| slots | segmentations | C(d,1) | C(d,2) | C(d,3) | C(d,4) | deepest stratum 16 draws can fund |
|---|---|---|---|---|---|---|
| 4 | 41 | 4 | 6 | 4 | 1 | **depth 4** (needs 15 draws) |
| 5 | 96 | 5 | 10 | 10 | 5 | **depth 2** (depth 3 needs 25) |
| 6 | 119 | 6 | 15 | 20 | 15 | **depth 1** (depth 2 needs 21) |

`coverage_tuples` walks **breadth before depth** and stops at
`EMIT_PROFILE_RESERVE = 16`, and `sweep_index` returns `None` — skipping the class
*without consuming a draw* — whenever the class's narrowest member is not wider than
`LEXICAL_BRANCH_STAGE_0 = 10`. Measured against the model, on the canonical target:

| slots | segs | reserve offered | accepted | d1 | d2 | d3 | d4 |
|---|---|---|---|---|---|---|---|
| 4 | 41 | 232 | 181 | 107 | 94 | 31 | **0** |
| 5 | 96 | 1 272 | 1 039 | 388 | 608 | 276 | **0** |
| 6 | 119 | 1 900 | 1 830 | 658 | 1 182 | 60 | **0** |
| all | 256 | **3 689** | **3 050** | 1 153 | 1 884 | 367 | **0** |

Accepted totals by depth: 1 126 / 1 689 / 235 at d1 / d2 / d3 — within 5 tuples of
`e1a3f7`'s 1 126 / 1 689 / 237, so my reconstruction of the reserve's offer stream is
theirs (my raw offered count 3 689 against their 3 404 is the sweep fix: `79309a1`'s
per-member legality test refuses diagonals the parent emitted, which is exactly the
tuple-level non-continuity `7d1c04` flagged).

**Zero depth-4 offers, and zero depth-3 offers at the model-predicted rate for
6-slot segmentations beyond the ~0.5 that the narrow-slot skips buy.** Two
independent reasons, and the cap is the weaker of them:

* **The allowance, not the cap.** The requested wording lives in a **5-slot**
  segmentation (§2.2). Funding a depth-4 class in a 5-slot segmentation costs
  `5 + 10 + 10 + 5 = 30` draws of the 16 available. Measured: 0 depth-4 offers in
  5-slot segmentations, and 0 everywhere.
* **The cap.** `EMIT_PROFILE_MAX_DEEP = 3` forbids depth 4 outright — but the walk
  would not reach it anyway, so the cap is currently *inert* for the target's own
  shapes. **Prediction for the sibling cap-at-4 front:** raising the cap to 4 yields
  depth-4 offers only in the 41 four-slot segmentations, where the depth-4 stratum is
  the 15th of 16 draws, and zero in the 96 five-slot and 119 six-slot segmentations
  that carry the milestone. On this base's numbers that is ≤ 41 tuples out of 3 689,
  all of them the wrong shape for the milestone.

### 2.2 where the requested wording actually lives

Exactly **two** of the 256 segmentations admit `hits justice dupe hid came` as one
word per slot:

| seg | slots | spans | widths | indices | cost | bound score |
|---|---|---|---|---|---|---|
| 151 | 5 | 0-3 3-10 10-13 13-15 15-19 | 160,7,160,160,93 | 7,0,13,99,11 | **1.1195** | 0.819901 |
| 187 | 5 | 0-3 3-11 11-13 13-15 15-19 | 160,2,160,160,93 | 7,0,22,99,11 | **1.3695** | 0.804276 |

The brief's framing is *four*-deep and it is right about the depth — indices
`7,0,13,99,11` have four non-zero coordinates — but the carrier is a **5-slot**
segmentation of a 5-word target, not a 6-slot one, and `4e8a52`'s 13 "segmentations
whose slot lists contain all four words" is a looser criterion than "this exact
five-word wording is buildable here". Two consequences:

* cost **1.1195 ≤ 1.5**, so `build` would accept it — `e1a3f7`'s "the cost bound is
  not the gate" reproduces, and `4e8a52`'s 0.919520 is a *different* (looser)
  coordinate choice in a different segmentation;
* slot 1 is the `just a` span, only **7 candidates** wide, and `justice` is its
  **index 0** with cost 0.0 (the target's own `just` is index 6, `reused = 1`). So
  every one of the 50 visible proposals is *structurally pinned* to keep `justice` —
  which the requested clue also does. **The shape is not what excludes it.**

### 2.3 the score-directed alternative is measured to point the wrong way

If the reserve chose each class member by the search's own objective instead of by a
uniform rank (`at`), it would move **away** from the requested clue. Per-slot argmax
with the other four slots pinned at the requested indices, seg 151:

| slot | requested | argmax | gain | candidates that beat the requested wording |
|---|---|---|---|---|
| 0 | 7 `hits` | 0 `each` | +0.0114 | 8 of 160 |
| 1 | 0 `justice` | 0 `justice` | 0 | 0 of 7 |
| 2 | 13 `dupe` | 0 `too` | +0.0148 | 13 of 160 |
| 3 | 99 `hid` | 0 `add` | +0.0241 | **91 of 160** |
| 4 | 11 `came` | 0 `gave` | +0.0136 | 10 of 93 |

**122 single-slot substitutions** of the requested wording beat it (133 in seg 187),
and a greedy single-slot ascent — which is precisely what the existing adjacency
operator does — reaches only **0.8839** after 4 steps and then stalls, still
**0.032 below the cutoff**. The requested wording sits in a basin entirely beneath
the visible band.

---

## 3. The decisive measurement: what the production scorer does with it

The measurement patch can push one named index tuple through the ordinary `build`, so
the wording enters the real pool and the real `finish` scores it with the real
`Metrics` and the real `select_diverse`:

```
$ MADGAB_C81E55_INJECT="151,7,0,13,99,11;187,7,0,22,99,11" \
  MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
  madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" rank=9867 score=0.819901291
MADGAB_TRACE raw_cutoff rank=49 score=0.915691888 phrase="it justice too day mm"
```

* both admitting segmentations emitting it: rank **9867 of 17 908**, score **0.819901291**
* one of them: rank 9868, same score
* no injection (today's behaviour): absent from 17 907

So the **entire search, if it emitted the requested wording from every segmentation
that admits it, would move it from "absent" to "9867th of 17 908"**. The pool would
grow by one entry in 17 908 (0.006 %). `4e8a52`'s pool-membership metric
(`"pool: FALSE"`) is therefore not a weak proxy for the milestone — it is nearly
uncorrelated with it, because membership is not what the test measures.

**Fidelity of the offline analysis.** My reconstruction of the objective from the
dumped per-slot components reproduces the production score for the requested clue
**exactly** (`0.819901` vs `0.819901291`). For arbitrary other tuples it is the
traversal's admissible *bound* rather than the final score and differs by up to
0.04 (§7). Every number in §3 and §5 that concerns the requested clue or the guard
is a **production** number from the injection or the base's own trace gate; the
reconstruction is used only for the axis decomposition, the landscape, and the
sweeps, and is labelled where it appears.

---

## 4. Mechanism shortlist, with the measurement behind each

Ordered by expected effect on the acceptance test, best first. "Effect" is the
measured change in the requested clue's production rank / score.

| # | mechanism | measured effect | verdict |
|---|---|---|---|
| M1 | **score-directed coordinate selection** (reserve picks each class member by the objective instead of a uniform rank) | per-slot argmax is index 0 in 4 of 5 slots; `hid`@99 is beaten by **91 of 160** candidates in its slot; 122 single-slot moves beat the wording | **wrong direction**; rejected |
| M2 | **depth cap as a function of slots and the rank rectangle** instead of a constant 3 | 0 depth-4 offers at cap 3; the class walk needs 30 of 16 draws in the 5-slot segmentations that carry the wording; ≤ 41 of 3 689 tuples at cap 4 and all in the wrong shape | **necessary, not sufficient, and not the binding constraint**; the sibling's arm should confirm 0 |
| M3 | **allowance reallocated across depths / shape classes** (stratified or round-robin instead of breadth-first-flat) | would fund depth-3/4 classes; costs 30 of the 64 per-segmentation emissions = **47 % of the 16 384 global budget**, against a baseline that already spends 14 239 and whose *visible top 50 is produced by the traversal*, not the reserve | **budget-infeasible and score-irrelevant** |
| M4 | **deep tuples priced against the marginal, not baseline, cost** | the wording's cost is 1.1195 ≤ 1.5 and `build` accepts it; `e1a3f7` measured 4 692 accepted tuples already ≤ 0.9 | **already not the gate**; zero effect |
| M5 | **segmentation proposes more than one segmentation per target** | 2 of 256 segmentations admit the wording and both already admit it at an affordable cost; the segmentation layer is not what withholds it | **zero effect** |
| M6 | **search emits the wording, unconditionally** | production rank 9867 of 17 908, score 0.819901, cutoff 0.915692, gap **0.0958** = 11× the visible band; greedy single-slot ascent stalls at 0.8839 | **cannot pass** |
| M7 | **scoring / ranking change** | see §5 | the only family with any effect, and the cheapest member of it is overfitting |

Two of these deserve a sentence each, because they are the ones the item is currently
funding:

**M2/M3 are defeated by the *global* budget, not just the per-segmentation one.**
`LEXICAL_GLOBAL_EMISSION_BUDGET = SEGMENTATION_KEEP × LEXICAL_COMBINATIONS_PER_SEGMENTATION
= 256 × 64 = 16 384`, and the reserve's 16 is carved out of the same 64 the traversal
uses. Funding depth-4 classes in the 5-slot segmentations costs 30/64 per
segmentation ⇒ **7 680 of 16 384 = 47 % of the whole search's emission budget** on
coverage sampling, against a baseline that already spends 14 239 and whose visible
top 50 is traversal output (reserve-built tuples score mean 0.72–0.75, p90 0.86–0.88,
**max 0.9167**; the visible band is 0.911–0.925). The reserve is, measurably, not
where the visible surface comes from.

**The reserve's own output is measurably off the visible surface.** Accepted reserve
tuples, canonical target, by source and depth:

| | n | mean | p10 | p50 | p90 | max |
|---|---|---|---|---|---|---|
| reserve d1 | 1 126 | 0.7448 | 0.6193 | 0.6954 | 0.8766 | **0.9167** |
| reserve d2 | 1 689 | 0.7214 | 0.5933 | 0.6762 | 0.8587 | **0.9042** |
| reserve d3 | 235 | 0.7320 | 0.5935 | 0.7791 | 0.8557 | **0.8828** |

Not one of the 3 050 accepted reserve tuples scores inside the visible band. Whatever
the reserve buys, it does not buy proposals.

---

## 5. The scoring family, costed

The requested clue's axis values (production-verified total 0.819901):

| axis | weight | value | contribution |
|---|---|---|---|
| similarity | 0.25 | 0.7201 | 0.1800 |
| boundary novelty | 0.15 | 0.6667 | 0.1000 |
| word novelty | 0.15 | 1.0 | 0.1500 |
| familiarity | 0.10 | 0.3987 | 0.0399 |
| rhythm | 0.30 | 1.0 | 0.3000 |
| shape | 0.05 | 1.0 | 0.0500 |
| closed class | −0.10 | 0.0 | 0.0000 |
| **total** | | | **0.819901** |

It is **maximal on 3 axes and mediocre on 3**, and the three it maxes carry only
**0.50 of the 1.00 mass**. Two measured consequences:

* **The most defensible general variant does not reach it.** Making the similarity
  axis *relative* — `1 − (cost − segmentation_floor)/4` instead of `1 − cost/4`, so
  the axis asks "how much worse than the best wording this segmentation admits"
  rather than an absolute IPA-edit budget that is a bare `4.0` constant — lifts the
  wording from 0.819901 to **0.833368**, rank 1101 of 3 050. The 50th-best accepted
  tuple under the same variant scores 0.926705, so the wording is still **0.093**
  short. This variant is worth doing on its own merits and is *not* a milestone fix.
* **No reweighting of the existing six axes reaches the cutoff, short of destroying
  one.** A single donor→taker move of *any* size, up to 100 % of the mass, does not
  reach 0.915692. The cheapest two-donor reweighting that does moves **21.3 % of the
  total scoring mass**: it deletes FAMILIARITY outright (0.10 → 0) and cuts NOVELTY
  from 0.15 to 0.0375, both into RHYTHM (0.30 → 0.5125). The requested clue scores
  **1.0 on rhythm**, so this works by paying it for a syllable-count coincidence it
  gets for free. It is not a mechanism for making resegmentations reachable; it is an
  objective re-pointed at one clue's free lunch. It does happen to leave the green
  guard *better* off (`wreck a nice beach` +0.0379 under the same weights, since it
  also maxes rhythm), and it would break `approximate_output_is_locked`, whose top-10
  for `I love you` is byte-locked at the current weights.

So the scoring family splits cleanly:

* **general, defensible, not a milestone fix** — the relative similarity axis (M7a).
* **milestone-passing, indefensible** — the 21.3 % mass move (M7b). It is
  indistinguishable from tuning to the acceptance clue, which the fence test exists
  to prevent in `src/` and which would be the same offence in `axes::*`.

---

## 6. The general, phrase-free statement of the defect — and the regression test

This is the one thing here that is both actionable and true of the code rather than
of the two sentences in the brief. Over the 13 real targets measured, counting visible
proposals that are **full resegmentations** (no target word retained, matched with a
≥3-letter prefix rule so `it's`/`it` counts as shared):

| target | full resegmentations in the visible 50 | target words retained |
|---|---|---|
| **It's just a stupid game** | **0 / 50** | `justice` ×50, `games` ×1 |
| the cat sat on the mat | 29 / 50 | matt ×5, catch ×8, match ×6, then ×3 |
| when the rain finally stopped | 31 / 50 | final ×19, whence ×1 |
| an old man in a big hat | 35 / 50 | many ×15 |
| recognize speech | 44 / 50 | rec ×6 |
| what are you going to do | 49 / 50 | wha ×1 |
| the other seven | 50 / 50 | — |

**On the canonical target the visible proposal set contains no full resegmentation at
all, and the canonical Mad Gab answer is a full resegmentation.** On the other twelve
targets full resegmentations dominate. That is a property of the objective meeting a
target whose `just a` span is only 7 candidates wide (so `justice` at index 0 is
forced into every proposal) and whose four remaining spans all have a near-homophone
at index ≤ 3 that scores 0.03–0.09 higher than anything deeper.

**Proposed regression test** (`tests/corpus_integration.rs`, API boundary, phrase-free,
no wording named): for each of a table of ordinary multi-word targets, assert that the
visible proposals contain at least one **full resegmentation** — a proposal with no
word in common with the target, compared with a ≥3-letter prefix rule so a clipping of
a target word (`it's` → `it`) counts as shared. Red today on `It's just a stupid game`
(0/50) and green on the other twelve. It expresses the milestone's actual intent
("the approximate mode can propose a resegmentation") as a property of the generator
rather than as a property of one dictionary lookup, and it stays meaningful when the
ranking moves. I recommend it be landed on the scoring front, red-first.

---

## 7. What I did **not** establish

* **I did not measure the cap-at-4 numbers.** That is the sibling front's arm. My
  claim about it is a prediction from the class-walk arithmetic of §2.1 plus the
  measured 0 depth-4 offers at cap 3, not a measurement of cap 4.
* **I did not measure the sweep fix's effect on any of this.** `2bacbcb` has no
  per-member sweep. Whether `79309a1` changes the requested clue's rank, its
  segmentations' admissibility, or the depth-4 offer count is not established. My
  3 689 vs `e1a3f7`'s 3 404 offered-tuple difference is the only place the two arms
  touch, and I attribute it to the fix's per-member legality refusal on the reasoning
  in `7d1c04` §2 rather than having measured it.
* **I did not establish that the 0.819901 is intrinsic to the wording.** I showed
  that no reweighting of the six existing axes and no segmentation of the requester's
  preferred shape reaches the cutoff. I did not search the space of *new* axes, and I
  make no claim that none exists.
* **My offline objective reconstruction is the traversal's admissible `bound`, not
  the final `Metrics` score.** It agrees with production to 6 decimal places for the
  requested clue and the guard, and differs by up to 0.04 for arbitrary tuples (e.g.
  `richard thus test oop uhh dame`: bound 0.6894 vs production 0.7291). Every §5
  sweep, the §2.3 landscape and the §6 counts that do not involve a production-traced
  number are bound-scale and are labelled as such. In particular the "reserve tuple
  score distribution" table is bound-scale; its *shape* is the claim, not its decimals.
* **I did not measure runtime.** The probe writes ~150 k–370 k rows per target and
  dominates the wall clock when enabled; with it disabled the binary is the base
  binary plus five gated call sites, and I make no runtime claim either way. I did not
  run the base/branch interleaved timing protocol.
* **I did not evaluate `select_diverse` as a lever.** Rank 9867 of 17 908 is far
  enough below the raw cutoff that no diversity policy reaches it, so I did not
  characterise the policy; if the scoring front lands, `select_diverse` becomes worth
  a front of its own.
* **I did not open, and am not opening, an implementation front.** Nothing on this
  branch is a fix.

---

## 8. Recommendation, and the next front

**Recommendation: do not open a search-side implementation front. The requested clue's
production score is 0.0958 below the rank-49 cutoff, and no cap, allowance, sweep,
cost, or segmentation change moves that number at all.** Opening a cap-at-4 or an
allowance-reallocation front against this milestone spends the item's remaining budget
on a change whose measured effect on the acceptance test is provably zero.

**Recommended first implementation: a scoring front, and only a scoring front.** Its
first commit should be **M7a, the relative similarity axis** — measure
`1 − (total_cost − segmentation_floor) / 4` instead of `1 − total_cost / 4`, where
`segmentation_floor` is the additive cost of the all-argmin tuple of that
segmentation (the floor `total_budget` is already measured against, and the same
`suf_min_cost` array the traversal's admissible bound already computes, so it is free).
Predicted numbers, from §5: the requested clue 0.819901 → 0.833368, rank 9867 → ~1101
of 3 050; `wreck a nice beach` unchanged to within 0.001; **this does not green the
milestone and should not be sold as doing so.** Its value is that it makes the
similarity axis mean something, and it is the only member of the scoring family with a
general argument. Guards: `recognize speech` → `wreck a nice beach` must stay in the
visible 50 (margin 0.00127 — re-measure it, do not assume it);
`approximate_output_is_locked` will need a deliberate, recorded decision because a
weight change moves every printed score; `approximate_proposals_are_predominantly_content_words`
must stay green; `approx_determinism` 2/0.

**The front to open second, and the one that can actually close the milestone, is the
one the coordinator has to decide exists:** a **resegmentation-objective** front whose
acceptance criterion is the phrase-free §6 property — *for every target, the visible
proposals include at least one full resegmentation* — and whose budget is explicitly
"raise the objective's valuation of a resegmentation", to be spent on what an axis
*measures* rather than on its weight, because §5 shows the weight route is a 21.3 %
mass move that deletes FAMILIARITY and buys rhythm. If the coordinator judges that
raising the objective's valuation of resegmentations is not a legitimate goal, then the
honest outcome is that `approximate_finds_classic_madgab_resegmentation` is **not
satisfiable as written** and the item's premise — that this is a coverage problem —
should be recorded as refuted, with §0 and §3 as the evidence.

**Acceptance criteria for that front, so a later pass can open it without re-deriving
this:**

1. `approximate_proposals_include_a_full_resegmentation` (§6) is red on
   `post-milestone-acceptance` and green at the tip, over ≥ 6 targets, naming no
   wording.
2. `recognize speech` → `wreck a nice beach` is still in the visible 50, **and its
   margin over the rank-49 cutoff is reported, not assumed** (today 0.00127).
3. `approximate_finds_classic_madgab_resegmentation` is reported as red or green
   honestly; if green, the report must show the clue's production rank and score and
   must show that no axis weight was moved by more than a stated fraction of the
   mass, with the §5 sweep reproduced.
4. `tests/no_phrase_hard_coding` 6/6; `--lib` green; `approx_determinism` 2/0;
   `exact_determinism` 1/0; no test edited, relaxed, re-baselined or skipped.
5. Any change to `approximate_output_is_locked` is an explicit coordinator decision
   recorded in the report, with both arms' numbers.

---

## 9. Artifacts on this branch

```
REPORT-c81e55.md                             this report
src/probe.rs                                the gated measurement probe (throwaway)
src/lib.rs                                  five gated probe call sites (throwaway)
measurements/c81e55/*.top50                  the 13 visible proposal lists, probe on
measurements/c81e55/tgt_*.txt                the 13 target strings
measurements/c81e55/*.tsv.gz                 the raw probe rows for the two targets every
                                             number in this report derives from (the other
                                             eleven targets were probed identically and
                                             their .tsv discarded; their .top50 is kept,
                                             which is all §6 uses)
measurements/c81e55/analyze.js               A. class-walk model  B. reserve score distribution
measurements/c81e55/landscape.js             §2.3 per-slot objective landscape
measurements/c81e55/axes.js                  §5 axis decomposition, S1 variant, reweighting sweep
measurements/c81e55/fullreseg.js             §6 full-resegmentation census
```

Unpack with `gunzip -k measurements/c81e55/It_s_just_a_stupid_game_.tsv.gz` before
running the three `node` scripts; they read the plain `.tsv`.

`SEG` columns: `id phase nslots word_count target_syllables shared target_inner_count
novelty spans widths`.  `ALT`: `seg slot i cost reused familiarity closed shape
syllables word`.  `TUPLE`: `seg source accepted indices` with source ∈ {`reserve`,
`traversal`}.

Reproduction:

```
cargo build --release
for t in "It's just a stupid game" "recognize speech" ... ; do
  f=$(echo "$t" | tr -c 'a-zA-Z' '_')
  echo "$t" > measurements/c81e55/tgt_$f.txt
  MADGAB_C81E55=measurements/c81e55/$f.tsv \
    ./target/release/madgab --approximate --top 50 "$t" 2>/dev/null \
    > measurements/c81e55/$f.top50
done
node measurements/c81e55/analyze.js   measurements/c81e55/It_s_just_a_stupid_game_.tsv \
                                      measurements/c81e55/It_s_just_a_stupid_game_.top50
node measurements/c81e55/landscape.js measurements/c81e55/It_s_just_a_stupid_game_.tsv \
                                      "hits justice dupe hid came"
node measurements/c81e55/fullreseg.js
RAW_CUT=0.915691888 node measurements/c81e55/axes.js \
     measurements/c81e55/It_s_just_a_stupid_game_.tsv \
     measurements/c81e55/It_s_just_a_stupid_game_.top50 "hits justice dupe hid came"

# the decisive one — ask the production scorer
MADGAB_C81E55_INJECT="151,7,0,13,99,11;187,7,0,22,99,11" \
MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
  ./target/release/madgab --approximate --top 50 "It's just a stupid game"
# -> rank=9867 score=0.819901291   raw_cutoff rank=49 score=0.915691888
```

Non-intrusiveness of the probe: visible stdout byte-identical with the probe on and
off, `cmp`, on the canonical target and on `recognize speech`. `no_phrase_hard_coding`
6/6 with it applied. It reads no target text and names no word; the only inputs are
integers from the environment.
