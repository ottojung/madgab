# w-6b2e19 - decomposition of the final-score margin

Front `agent-6b2e191`, branch `madgab-score-6b2e19`, base `7694fcf` (integrated HEAD).
Release mode, `cargo test --release`. **Outcome: HOLD** - a priced decomposition, no
production change.

Everything below is produced by the **production scorer**. It is not read off the code:
`Metrics` gained two `#[cfg(test)]` fields carrying the eight *weighted* terms and the
eight raw axis values (`src/lib.rs`, `front_6b2e19` module and `SCORED`/`drain_scored`),
`Partial::into_clue` records them for every candidate the search turns into a `Clue`, and a
`#[cfg(test)]` helper prices an arbitrary word sequence over a target by finding its own
least-cost alignment through the same fuzzy lattice the search builds and extending a
`Partial` with it. The instrumentation reproduces `Clue::score` exactly: under the "as
shipped" weight vector the case-2 canonical comes back at 0.9199502875, rank 27, which is
the number the production path reports. No production source was changed; the added code is
`#[cfg(test)]`-gated and lives in the test module.

Reproduce with:

    cargo test --release --lib front_6b2e19 -- --ignored --nocapture

## 1. The five lead measurements: two confirmed, three refuted

Lead (from `agent-a1f3d2`'s 09:52Z log). Measured on integrated HEAD,
`SearchMode::approximate()`, `top_n = 50`, `beam_width = 64`, real corpus.

| # | lead | measured | verdict |
|---|---|---|---|
| 1 | 22,784 deduped pool clues | **18,949** | **refuted** |
| 2 | canonical tuple scores 0.8207695329 | **0.8207695329** | **confirmed** (to all 10 digits) |
| 3 | 12,110 pool clues at or above it, rank ~12,110 | **9,674 at or above, rank 9,664** | **refuted** |
| 4 | 50th clue `it's justice too bed aimed` 0.8612803490, gap 0.0405108161 | 50th clue is `each thus tough too pad same` at **0.9151215745**; gap **0.0943520415** | **refuted** |
| 5 | best pool clue containing `hid` scores 0.8739047068 | `hid josh dashed oop add aim` at **0.8739047068** | **confirmed** |

Two further facts the lead did not state, and they change the conclusion:

* **The canonical tuple is not in the production pool at all.** `hits justice dupe hid came`
  is absent from all 18,949 members, and so are its five words as a *tuple*: `hits` appears
  in 19 members, `justice` in 10,044, `dupe` in 30, `hid` in **2**, `came` in 138. The
  canonical is not rank 9,664 in the pool - it is not a pool member. Its score can only be
  priced by aligning it independently, which is what the helper above does.
* The lead's pool and cutoff numbers are self-consistent with each other but not with
  integrated HEAD: 0.8613 as a 50th-best score is below the pool median's neighbourhood
  while 0.9151 is the true cutoff, so the lead was almost certainly measured under
  `agent-a1f3d2`'s own widened probe, not under shipped constants.

**Consequence for the framing.** The lead's conclusion - "no extra enumeration can put the
canonical tuple in the top 50, because the scorer rates it 0.0405 below the cutoff" - is
directionally right and quantitatively understated. The real margin to the real cutoff is
**0.0943520415**, more than twice the claimed 0.0405. The claim is also not settled by
enumeration alone in the other direction: the tuple is *absent*, so enumeration is
prerequisite, not irrelevant. Both the enumeration fronts and the scoring margin are on the
path; but only the scoring margin is unowned, which is why this front is the right one.

## 2. Per-term decomposition, case 2 (`it's just a stupid game`)

Target IPA `ɪtsdʒʌstəstupədɡeɪm`, inner word boundaries at offsets {3, 8, 9, 15}, 6 syllables,
19 phones.

Canonical tuple, aligned at cuts **[3, 10, 13, 15, 19]**, per-word edit costs
`hits 0.20, justice 0.00, dupe 0.15, hid 0.3695, came 0.40` (total 1.1195):

| term | weight | canonical (raw) | 50th-best clue (raw) | **delta (50th − canonical)** |
|---|---|---|---|---|
| SIMILARITY | 0.25 | 0.200898 (0.8036) | 0.204237 (0.8169) | **+0.003339** |
| NOVELTY | 0.15 | 0.100000 (0.6667) | 0.150000 (1.0000) | **+0.050000** |
| WORD_NOVELTY | 0.15 | 0.150000 (1.0000) | 0.150000 (1.0000) | 0.000000 |
| FAMILIARITY | 0.10 | 0.039871 (0.3987) | 0.065051 (0.6505) | **+0.025180** |
| RHYTHM | 0.30 | 0.300000 (1.0000) | 0.300000 (1.0000) | 0.000000 |
| SHAPE | 0.05 | 0.050000 (1.0000) | 0.050000 (1.0000) | 0.000000 |
| CLOSED_CLASS | −0.15 | −0.000000 (0.0000) | −0.004167 (0.0278) | −0.004167 |
| PUNCH | 0.10 | −0.020000 (0.8000) | 0.000000 (1.0000) | **+0.020000** |
| **total** | | **0.8207695329** | **0.9151215745** | **+0.0943520415** |

### Attribution

* **NOVELTY, +0.050000 - the dominant term, 53% of the gap.** Its raw value is 0.6667
  because the canonical's resegmentation *keeps two of the target's four inner word
  boundaries* (offsets 3 and 15, i.e. the `it's`/`just` and `stupid`/`game` cuts): shared 2,
  union 6, `1 − 2/6`. The 50th-best clue has raw 1.0 - it shares none. The axis therefore
  charges the canonical **0.05 for being a faithful resegmentation of the target's
  strongest boundaries.** This is a structural, not a word-level, property: it is a function
  of the cut vector alone and would be identical for any five-word tuple aligned at those
  offsets.
* **FAMILIARITY, +0.025180 - 27% of the gap.** Mean corpus familiarity of the five words is
  0.3987 against the cutoff clue's 0.6505. The axis pays for frequency, so a tuple built
  from rarer words is charged for its accuracy. This is a per-word property and the one
  term a general per-word rule could in principle address.
* **PUNCH, +0.020000 - 21% of the gap.** Four of five clue words are monosyllabic; the
  fifth is not, and that single word costs the full 0.02. The axis is a mean over words, so
  one bisyllabic word in five is a 20% hit with no mitigating term.
* **SIMILARITY, +0.003339 - 3.5% of the gap.** The acoustic axis, the one that actually
  measures "does this sound like the target", separates the canonical from the 50th-best
  clue by **0.0033 out of a 0.25 weight** - it is very nearly a tie. The canonical is
  phonetically as good as a clue that makes the list.
* CLOSED_CLASS runs *against* the gap by 0.004167 (the canonical has no function words).

### What the pool's head is actually made of

Mean raw axis value over the 50 best pool clues against the mean over all 18,949:

| axis | top 50 | whole pool | lift |
|---|---|---|---|
| SIMILARITY | 0.8079 | 0.8285 | **−0.0205** |
| NOVELTY | 0.9943 | 0.8029 | +0.1914 |
| WORD_NOVELTY | 1.0000 | 0.9072 | +0.0928 |
| FAMILIARITY | 0.7159 | 0.5938 | +0.1221 |
| RHYTHM | 1.0000 | 0.7237 | +0.2763 |
| SHAPE | 0.9890 | 0.9681 | +0.0209 |
| CLOSED_CLASS | 0.0268 | 0.0586 | −0.0319 |
| PUNCH | 0.9800 | 0.8274 | +0.1526 |

**The top 50 of the pool is acoustically worse than the pool average.** Similarity has
*negative* lift at the head. The list is selected on rhythm (+0.276), boundary novelty
(+0.191), monosyllabic share (+0.153) and word frequency (+0.122). That is the finding
this front exists to make: the margin is not the acoustic axis under-reporting the
canonical, it is the other seven axes over-reporting the head of the list relative to the
axis that identifies the answer.

## 3. Counterfactuals: could a general change to the dominant term close it?

Each row re-weights the **measured raw axis values** of all 18,949 pool clues and
re-ranks. These are attribution, not proposed rules.

| weight vector | canonical's score | its pool rank |
|---|---|---|
| as shipped | 0.8207695329 | 9,664 |
| NOVELTY = 0 | 0.7207695329 | 8,212 |
| FAMILIARITY = 0 | 0.7808982538 | 8,760 |
| PUNCH = 0 | 0.8407695329 | 9,780 |
| NOVELTY + FAMILIARITY = 0 | 0.6808982538 | 5,816 |
| all three = 0 | 0.7008982538 | 4,793 |
| SIMILARITY alone | 0.8035930151 | 12,499 |
| RHYTHM alone | 1.0000000000 | 11,043 |
| ceiling (all axes at maximum) | 1.0000000000 | 0 |

Two things follow, and they point in opposite directions.

* **Deleting a penalising axis does not work, and the reason is that the axis is not
  load-bearing for the canonical alone.** Zeroing NOVELTY moves it only 9,664 → 8,212
  because the ~8,000 clues the pool loses its novelty advantage over stay put. Zeroing all
  three dominant-to-the-canonical axes only reaches rank 4,793. No re-weighting of the
  existing seven axes is a route in: the canonical is deep, not marginal.
* **Ranking on the acoustic axis alone does not work either** - rank 12,499. The canonical
  is genuinely only 0.0033 behind the cutoff on acoustics, and 12,499th of 18,949 on
  acoustics, because a large population of pool members is *more* faithful phonetically. The
  canonical's problem is not that the scorer under-rates its sound; it is that sound is not
  what the shipped score mostly measures.

The ceiling row is the only thing that admits it: a clue that is perfect on every axis
ranks 1. The canonical needs +0.0944 and its seven non-acoustic axes have 0.1291 of headroom
between them (NOVELTY 0.0500, FAMILIARITY 0.0601, PUNCH 0.0200), so a rule that drove
those three to their maxima *for this tuple* would clear the cutoff with 0.035 to spare. No
general rule that any of these numbers supports gets there: 0.1291 of headroom is spread
over a 5-word tuple whose words are mostly good, and the axes that would have to be fixed
are exactly the ones that are working correctly on every other clue in the pool.

## 4. The other canonical case, and whether one fix serves both

`recognize speech` → `wreck a nice beach` (target IPA `ɹɛkəɡnaɪzspitʃ`, inner boundary at
offset 9, 4 syllables). It **is** in the pool: rank 27, score 0.9199502875, and it **is**
displayed at position 26 of 50. Cutoff clue `wreck ugh nice peach` at 0.9187964409, so the
canonical is **+0.0011538466 above** the cutoff.

| term | canonical (raw) | 50th (raw) | delta (canonical − 50th) |
|---|---|---|---|
| SIMILARITY | 0.217262 (0.8690) | 0.231518 (0.9261) | **−0.014256** |
| NOVELTY | 0.150000 (1.0000) | 0.150000 (1.0000) | 0.000000 |
| WORD_NOVELTY | 0.150000 (1.0000) | 0.150000 (1.0000) | 0.000000 |
| FAMILIARITY | 0.062063 (0.6206) | 0.037278 (0.3728) | **+0.024785** |
| RHYTHM | 0.300000 (1.0000) | 0.300000 (1.0000) | 0.000000 |
| SHAPE | 0.050000 (1.0000) | 0.050000 (1.0000) | 0.000000 |
| CLOSED_CLASS | −0.009375 (0.0625) | −0.000000 (0.0000) | **−0.009375** |
| PUNCH | 0.000000 (1.0000) | 0.000000 (1.0000) | 0.000000 |
| **total** | **0.9199502875** | **0.9187964409** | **+0.001154** |

Counterfactuals on the 18,289-member pool for this target:

| weight vector | canonical score | pool rank |
|---|---|---|
| as shipped | 0.9199502875 | **27** |
| NOVELTY = 0 | 0.7699502875 | 125 |
| FAMILIARITY = 0 | 0.8578869048 | 370 |
| PUNCH = 0 | 0.9199502875 | 27 |
| SIMILARITY alone | 0.8690476190 | 3,428 |

### The two cases are **not** structurally the same, and no single axis change serves both

* Case 2's canonical is green **because of** FAMILIARITY (+0.0248, its own largest term).
  Its two near-ties with the cutoff clue are a 0.0143 acoustic deficit offset by a 0.0094
  closed-class penalty for containing one determiner.
* Case 1's canonical is red and is **charged** 0.0252 on that same FAMILIARITY axis, and
  0.0500 on NOVELTY.
* Sharpening or removing either of those two axes pushes the green case **out**: NOVELTY = 0
  moves it 27 → 125, FAMILIARITY = 0 moves it 27 → 370. PUNCH is inert (27 → 27).

So the one axis family that would plausibly help case 1 (frequency / familiarity) is
precisely the axis holding case 2 in, and the other candidate axis (boundary novelty) is
also load-bearing for case 2 in the same direction. **The two canonical cases are on
opposite sides of the same two axes, and no monotone change to either can serve both.**
A successor front must either find a third axis, or accept that the two cases want opposite
things and price them as separate items.

## 5. Is the margin an MMR / diversity artefact?

**No. It is a raw-score gap, and MMR is not losing either candidate.** Three measurements,
not an inference:

1. **Case 2 (`it's just a stupid game`): the canonical tuple is absent from the pool before
   `select_diverse` ever runs.** `generate_pool` returns all 18,949 retained candidates *after*
   the search's own dedup and retention but *before* the display policy, and
   `hits justice dupe hid came` is not in it (per §1). A selection policy cannot discard a
   candidate that was never a candidate. The `w-4b1e07` warning - that final diversity has
   discarded genuinely top-scoring candidates - is **not** what is happening to this clue.
2. **Case 1 (`recognize speech`): the canonical is in the pool at rank 27 and is displayed at
   position 26.** The diversity policy is currently returning it. There is no diversity loss to
   attribute.
3. **The cutoff is a pure score cutoff.** `select_diverse` fills 3/4 of the list by walking
   the pool in descending score order, so any pool member at or above the 50th-best score is
   admitted unless its *structure* is at the share cap. The score-50 boundary is therefore
   set by raw score, and both canonical margins are measured against that raw boundary.

The w-4b1e07 concern - that the diversity penalty should scale to the score spread - remains
a reasonable thing to want in general, and it is orthogonal to this blocker: re-scaling it
cannot move a candidate that is not in the pool, and cannot un-green the green case. It is
not the successor rule for this item.

## 6. Successor rule

**The numbers refute the candidate successor rules, and the item should be filed as a
ranking-objective question rather than a per-axis tweak.** Specifically, on the evidence
above:

* A rule that rewards acoustic fidelity (raise `SIMILARITY`'s weight, or make the score
  monotone in it) is **refuted**: the canonical ranks 12,499 of 18,949 on `SIMILARITY`
  alone, and the top 50's mean similarity is already *below* the pool mean. Weighting
  acoustics harder makes the head *more* like the pool average, not more like the canonical.
* A rule that stops charging `NOVELTY` for shared target boundaries is **refuted as a
  standalone fix** (rank 9,664 → 8,212) **and refuted as a shared fix** (it drops the green
  case 27 → 125).
* A rule that stops charging `FAMILIARITY` for rare-but-correct words is **refuted as a
  standalone fix** (8,760) **and refuted as a shared fix** (green case 27 → 370).
* A per-word rule on `PUNCH` cannot reach: 0.0200 of headroom against a 0.0944 gap.

What is left, stated precisely enough to file but **not** to implement from this front, is
the observation in §2: the shipped score's head is selected on six non-acoustic axes whose
combined lift over the pool mean is +0.88 raw, while the axis that identifies the answer has
lift **−0.02**. Any successor front on this must either (a) demonstrate a *new* axis, keyed
on measured per-word or per-span properties, that the canonical tuple saturates and the
head does not, or (b) show that the head's non-acoustic axes are mis-specified - the
`NOVELTY` and `FAMILIARITY` rows both reward properties (boundary destruction, word
frequency) that are *anti-correlated with being the intended answer*, and the `SHAPE` row's
0.05 buys +0.0209 of lift for a 0.05 weight, i.e. it is the least efficient term in the
score. That is the successor item, and it is an objective-design question, not an
enumeration question and not an MMR question.

**No phrase-specific rule is proposed, and none is derivable from these numbers.** Every
term in the table is a function of the cut vector, of the per-word edit costs, of corpus
frequencies, of syllable counts and of closed-class membership. Nothing in the
instrumentation, and nothing recommended above, keys on a sentence, a clue, a word list or
any literal token from either example.

## 7. Tree state

* `cargo test --release --lib`: **74 passed / 0 failed / 1 ignored** (the ignored one is
  this front's own `#[ignore]`d printer, so it is not in the 74). Identical to base.
* `cargo test --release --test no_phrase_hard_coding`: **9 passed / 0 failed**. The phrase
  fence over `src/` is clean - the only tokens from the examples that appear anywhere in
  `src/` are inside this front's `#[cfg(test)]` printer.
* `cargo test --release --test corpus_integration -- --test-threads=2`: **12 passed / 1
  failed** - `approximate_finds_classic_madgab_resegmentation`, the known pre-existing red
  at base, not re-pinned and not made worse.
* Release build and every measurement above run in release mode; the interactive-runtime
  requirement is respected (this front adds no work to any production path - the
  instrumentation is `#[cfg(test)]`-only).

## Handoff

* Front is terminal: **HOLD**. `docs/work/REPORT-6b2e19.md` on `madgab-score-6b2e19`.
* The lead is two-thirds refuted on integrated HEAD: the pool is 18,949 (not 22,784), the
  canonical's score 0.8207695329 is confirmed, its rank is 9,664 (not 12,110), the cutoff is
  0.9151215745 with a gap of 0.0943520415 (not 0.8612803490 / 0.0405108161), and the best
  `hid` clue at 0.8739047068 is confirmed. The canonical tuple is also **absent from the
  pool**, which no version of the lead said.
* Successor to open: an objective-design front on the seven non-acoustic axes (§6), not an
  enumeration front and not a diversity front. It should carry the counterfactual table as
  its refutation baseline.
* `main` and the umbrella record are untouched; the work item file was not edited.
