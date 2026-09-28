# w-3a8c05 - which axis property is mis-specified, and what it costs to fix

Front `agent-3a8c051`, branch `madgab-objaxis-3a8c05`, base `7e648fe` (integrated HEAD,
`post-milestone-acceptance`). Release mode, `cargo test --release`.
**Outcome: HOLD** - a measured diagnosis and a fully priced counterfactual table, no
production change.

Baseline: [REPORT-6b2e19.md](REPORT-6b2e19.md), integrated at `7e648fe`. Its counterfactual
table is carried here as the refutation baseline and is not re-run. Its instrumentation
(`#[cfg(test)]` per-term capture in `Metrics`/`into_clue` plus `SCORED`/`drain_scored` and a
least-cost lattice aligner) is re-used from branch `madgab-score-6b2e19` and extended on this
branch by a new `#[cfg(test)]` module, `front_3a8c05`, which adds per-candidate *structural*
features, pool-wide correlation statistics and a candidate-objective re-ranker. **No production
source is changed**: the only non-`cfg(test)` edit in the diff is
`let m = self.metrics(..); let score = m.combined;`, which is behaviour-identical, and the
release binary's output and timings are unchanged (§7).

Reproduce with:

    cargo test --release --lib front_3a8c05 -- --ignored --nocapture

## 0. Baseline reproduced, and one correction to it

| quantity | REPORT-6b2e19 | this front | verdict |
|---|---|---|---|
| pool, `it's just a stupid game` | 18,949 | **18,949** | confirmed |
| canonical score | 0.8207695329 | **0.8207695329** | confirmed |
| canonical pool rank | 9,664 | **9,674** | 6b2e19's own two numbers disagree by 10; the exact figure is 9,674 |
| 50th-best (cutoff) | 0.9151215745 | **0.9151215745** | confirmed |
| gap | 0.0943520415 | **0.0943520415** | confirmed |
| canonical is in the pool | no | **no** | confirmed |
| `recognize speech` pool | 18,289 | **18,289** | confirmed |
| canonical score / rank | 0.9199502875 / 27 | **0.9199502875 / 27** | confirmed |

The 9,664 vs 9,674 difference is a capture-selection artefact in 6b2e19's instrument: a phrase
can be completed at more than one point in the search, and that instrument kept the *first*
capture for a phrase rather than the one matching the retained pool member's own score. This
front keeps, per pool member, the capture whose term sum is closest to that member's
`Clue::score`, and self-checks it:

    capture self-check: 0 pool members with no capture, worst |recomputed - Clue::score| 0.000e0

so the re-ranking below starts from a pool that reproduces the shipped order *exactly*. That
also moves 6b2e19's §2 lift table slightly: the true top-50 mean `SIMILARITY` is **0.8091**
against a pool mean of **0.8271** (lift **−0.0180**), not 0.8079 / 0.8285 / −0.0205. The sign
and the conclusion are unchanged; the exact figures are above.

## 1. The two axes are not mis-weighted. They are mis-*aimed*, and the aim is measurable.

### 1.1 `NOVELTY` — the axis gets *word-boundary reuse* backwards, and it is already
double-counted by `WORD_NOVELTY`

`boundary_novelty` is the Jaccard distance between the target's inner word boundaries and the
clue's. Its doc comment states the intent: "counting only the boundaries that disappeared would
give zero novelty to a useful split that preserved an original boundary, so the measure is a
Jaccard distance and rewards both added and removed boundaries."

The measurement says the trade was the wrong way round, over the whole 18,949-candidate pool for
`it's just a stupid game`:

| shared target boundaries | n | mean `SIMILARITY` | mean shipped score | mean clue words | mean `FAMILIARITY` |
|---|---|---|---|---|---|
| 0 | 5,219 | 0.8082 | 0.8618 | 5.334 | 0.5285 |
| 1 | 7,471 | 0.8214 | 0.7313 | 5.637 | 0.6158 |
| 2 | 3,508 | 0.8478 | 0.7888 | 5.527 | 0.5370 |
| 3 | 1,729 | 0.8330 | 0.6765 | 6.492 | 0.7267 |
| 4 | 1,022 | **0.8845** | **0.5414** | 6.311 | 0.7369 |

Keeping the target's word boundaries is associated with **+0.076 of raw acoustic similarity**
(0.8082 → 0.8845) and costs **0.3204 of shipped score**. Restricted to the 14,109 candidates
that reuse no target word - so that "restating the sentence" is already excluded, and the
residual concern `NOVELTY` was invented for cannot apply:

* r(`NOVELTY`, `SIMILARITY` | reworded) = **−0.316**
* r(boundary fidelity, `SIMILARITY` | reworded) = **+0.341**
* r(`NOVELTY`, boundary fidelity) = **−0.973** over the whole pool (they are the same statistic
  read two ways, so this is a check, not a finding)

**The property `NOVELTY` gets backwards is *boundary reuse is the failure mode*.** It is not.
Reuse of a target word boundary is the signature of a clue word being aligned to a whole target
word, which is the cheapest alignment there is, and the pool says so. The failure mode the axis
was written for - the answer being the sentence restated - is measured *exactly*, and without
approximation, by `WORD_NOVELTY` (fraction of clue words that are a target word, `r = −0.693`
with boundary fidelity, i.e. a genuinely independent statistic). `NOVELTY` charges up to 0.15
for a property the objective already measures at 0.15 elsewhere, and the charge is
anti-correlated with the acoustic axis.

The same finding holds, weaker, on the second target: reworded candidates give
r(`NOVELTY`, `SIMILARITY`) = −0.100, r(fidelity, `SIMILARITY`) = +0.075, and the shared=1 group
has both the higher similarity (0.8219 vs 0.7817) and the lower shipped score (0.7005 vs 0.7201).

**One hypothesis of mine was refuted by the measurement, and is recorded so it is not re-tried:**
`NOVELTY` is *not* an over-segmentation length artefact. r(`NOVELTY`, clue word count) is
**−0.240** (and −0.089 among reworded candidates), i.e. *negative* - the Jaccard union term does
not reward shredding the target. The mechanism is purely the reuse charge, not the union term.

### 1.2 `FAMILIARITY` — the axis is not measuring readability, it is measuring word *shortness*
and it reads the closed-class share that `CLOSED_CLASS` already prices

`FAMILIARITY` is the mean of `word_familiarity(rarity)`, a log-frequency rank. The property it
presumably means to express is that a solver can *read* the answer. Pool-wide, over all 18,949
candidates:

| correlates with `FAMILIARITY` | r (case 2) | r (case 1) |
|---|---|---|
| mean clue word length (IPA chars) | **−0.412** | **−0.357** |
| closed-class word share | **+0.615** | **+0.465** |
| clue word count | **+0.470** | +0.318 |
| `PUNCH` (share of monosyllables) | +0.281 | +0.360 |
| mean per-word edit cost | −0.071 | +0.209 |
| `SIMILARITY` | −0.206 | **−0.550** |

**The property `FAMILIARITY` gets backwards is *commonness of individual words is a proxy for
readability of the phrase*.** It is not: the correlation with word *length* is −0.41 and with
closed-class share +0.62, so the axis is largely a detector of "short, common, function-adjacent,
numerous" words - the `it said thus tough too pad same` population. Its own doc comment already
concedes half of this ("frequency points the wrong way: `the`, `a`, `it` and `each` are among
the commonest words in English") and then adds `CLOSED_CLASS` at −0.15 to penalise exactly the
words the +0.10 axis pays for. The two terms are **one measurable quantity with opposite signs**,
r = +0.615, and their net is a noisy, length-driven residual rather than a readability judgement.
On the second target the same pairing is what makes the green case green: `wreck a nice beach`
is in the list **because of** `FAMILIARITY` (+0.0248) while being charged `CLOSED_CLASS`
(−0.0094) for the determiner it contains.

`FAMILIARITY` is also *anti-correlated with acoustics* (r = −0.206, and **−0.550** on the second
target), so the second 27% of the gap is not a readability judgement about the canonical's rare
words at all; it is the length/frequency confound above.

## 2. The three proposed axes, all keyed only on measurable per-clue structure

No candidate keys on a sentence, a clue, a word list, or any literal token. Each is a function of
the clue's word count, the target's word count, the clue's cut vector, the target's cut vector,
and per-word corpus rarity - all available in the library.

* **PARSIMONY** `1 − |w_clue − w_target| / max(w_clue, w_target)`. Nothing in the shipped
  objective reads the clue's **word count at all**, and a Mad Gab answer is read as a phrase
  standing in for the sentence: a five-word sentence answered by a six-word phrase is a different
  sentence, not a resegmentation of this one.
* **RESEG** `(shared / target_inner_boundaries) × WORD_NOVELTY`. "A resegmentation that is fully
  reworded yet structurally faithful to the target's own word boundaries." The `WORD_NOVELTY`
  gate is what stops this being a pure inversion: the one clue family that boundary-sharing alone
  would reward - a restatement, which shares every word *and* every boundary - scores 0.
* **CLEAN** `PARSIMONY × RESEG`. The conjunction: the target's own number of words *and* the
  target's own word boundaries, with every word replaced.

Case-2 values: canonical is 5 words for a 5-word target, shares 2 of 4 inner boundaries, reuses
no target word ⇒ `PARSIMONY` 1.0000, `RESEG` 0.5000, `CLEAN` 0.5000. The 50th-best clue
`each thus tough too pad same` is 6 words and shares none ⇒ 0.8333 / 0.0000 / 0.0000.

## 3. The decisive measurement: a monotone lower bound on the best rank achievable

This is the number that decides the item, and it does not depend on any weight choice. A pool
clue that is `>=` the canonical on **every** axis in a set stays `>=` it under **every**
non-negative weighting of that set. So the count of such clues is a hard floor on the canonical's
rank. Case 2, pool of 18,949:

| axis set | clues `>=` the canonical | ⇒ rank floor |
|---|---|---|
| `SIMILARITY` | 12,309 | 12,310 |
| `PARSIMONY` | 5,813 | 5,814 |
| `RESEG` | 4,052 | 4,053 |
| `CLEAN` | 4,366 | 4,367 |
| `SIMILARITY` + `PARSIMONY` | 4,323 | 4,324 |
| **`PARSIMONY` + `RESEG` + `SIMILARITY`** | **1,126** | **1,127** |

**No monotone objective over any measurable per-candidate property can put
`hits justice dupe hid came` in the top 50.** The best case over the three proposed axes plus the
acoustic axis is rank 1,127, and adding more axes only raises the floor. The canonical is not
marginally short; it is inside a cell of 1,126 competitors that dominate it on every property
that could be rewarded.

The only way past that floor is to reward the axes the canonical is *worse* on - `FAMILIARITY`
(0.3987, 16,986 clues above it) and `PUNCH` (0.8000, 10,265 above it) - i.e. to prefer **rare**
words or **polysyllabic** words. Those are the inversions the item asks to be priced as refuted;
§4 prices them (C6, C7, C8) and all three are worse than shipping.

**This closes the front's question.** The binding constraint on case 2 is not the objective's
weighting and not its *aim*; it is that the tuple is acoustically ordinary (12,310th of 18,949 on
`SIMILARITY`, 0.0033 behind the cutoff) and simultaneously structurally ordinary. A scoring
objective cannot distinguish it, because there is nothing left to distinguish it by. What it
needs is for the enumeration front (w-4d7c12) to surface a *better* tuple for this target - the
`0.9157359250` band measured in w-a1f3d2 is the same width and the same story.

## 4. The counterfactual table: thirteen priced candidates, both canonical cases

Every row re-ranks the captured pool of 18,949 (case 2) / 18,289 (case 1) under a candidate
objective. Weight vectors keep the shipped `[0.25, 0.15, 0.15, 0.10, 0.30, 0.05, −0.15, 0.10]`
unless the row says otherwise; PUNCH keeps the shipped `w·(v−1)` shift.

| candidate | case 2 score | case 2 rank | in top 50? | case 1 score | case 1 rank | in top 50? |
|---|---|---|---|---|---|---|
| **C0 as shipped** | 0.8207695329 | 9,674 | no (absent) | 0.9199502875 | **27** | **yes** |
| C1 `+PARSIMONY 0.10` (SHAPE→0, RHY 0.30→0.25) | 0.8207695329 | 8,748 | no | 0.8699502875 | **46** | **yes** |
| C1b `+PARSIMONY 0.15` (from `NOVELTY`) | 0.8707695329 | 7,393 | no | 0.8449502875 | 128 | no |
| C1c `+PARSIMONY 0.20` (from `NOVELTY`+`PUNCH`) | 0.9407695329 | 4,678 | no | 0.8699502875 | 898 | no |
| C2 `+RESEG 0.10` (SHAPE→0, RHY→0.25) | 0.7707695329 | 8,220 | no | 0.8199502875 | 1,840 | no |
| C2b `NOVELTY`→`RESEG 0.15` | 0.7957695329 | 2,755 | no | 0.7699502875 | 2,517 | no |
| C3 `+PARSIMONY 0.10 +RESEG 0.10` (RHY→0.20, SHAPE→0) | 0.8207695329 | 7,973 | no | 0.8199502875 | 1,868 | no |
| C3b `+PARSIMONY 0.10 +RESEG 0.10` (`NOVELTY`→0, RHY→0.20) | 0.7707695329 | 3,026 | no | 0.7199502875 | 2,160 | no |
| C5 `+PARSIMONY 0.05 +RESEG 0.05` (from `SHAPE`) | 0.8457695329 | 8,737 | no | 0.8949502875 | 166 | no |
| C6 `+PARSIMONY 0.10`, `FAMILIARITY`→0.05 | 0.9008338933 | 8,353 | no | 0.9389185961 | 95 | no |
| C7 `FAMILIARITY` = content-word mean | 0.8207695329 | 9,055 | no | 0.9073047484 | 319 | no |
| C8 `FAMILIARITY` = worst content word | 0.7844046259 | 9,684 | no | 0.8834643382 | 320 | no |
| C9 `NOVELTY`→0 **and** `+RESEG 0.15` (max > 1.0) | 0.7957695329 | **2,755** | no | 0.7699502875 | 2,517 | no |
| C10 `NOVELTY`→`CLEAN 0.15` | 0.7957695329 | 2,863 | no | 0.7699502875 | 2,265 | no |
| C11 `NOVELTY`→`CLEAN 0.15`, `+PARSIMONY 0.05` | 0.7957695329 | 2,770 | no | 0.7449502875 | 2,337 | no |
| C12 `NOVELTY`→`CLEAN 0.10`, `+RESEG 0.05` | 0.8291028663 | 3,016 | no | 0.8199502875 | 2,122 | no |

Reading the table:

* **C9 is the best possible boundary-fidelity play** - `NOVELTY` deleted with nothing given up in
  return, so the documented 0.0–1.0 score bound is deliberately broken to price the family. It
  moves case 2 from 9,674 to **2,755**, the best any boundary correction achieves, and it drops
  the green case from 27 to **2,517**. It is also, plainly: **this helps only by preferring
  shared boundaries.** Priced as refuted, as the item requires.
* **C10/C11/C12, the conjunction that is *not* an inversion**, do no better than the inversion
  (2,863 / 2,770 / 3,016). Gating boundary fidelity on `WORD_NOVELTY` and conjoining it with word
  count parsimony buys nothing, because the `WORD_NOVELTY` gate is 1.0 for 14,109 of 18,949
  candidates and is therefore almost inert. The conjunction is not a rescue; it is the same
  inversion with extra steps.
* **C1 is the only candidate that keeps both canonical cases in the list** (case 1 at 46), and it
  is the only one that costs nothing: it removes `SHAPE`, the least efficient term in the score
  (0.05 weight for +0.0209 of head lift), and gives 0.10 to a property nothing else reads. It
  moves case 2 9,674 → 8,748 and **does not** reach the top 50.
* **The two inversion candidates are refuted by their own numbers.** C7 (drop function words from
  the familiarity average) is 9,055 and C8 (worst-word rather than mean familiarity) is 9,684 -
  the latter *worse than shipping*, exactly as §3 predicts, because both reward rare words. On
  the green case C6/C7/C8 all push it out (95 / 319 / 320).
* Every candidate that improves case 2 worsens case 1, and the only one that does not is C1, which
  improves neither. **The two canonical cases still sit on opposite sides, and the front's own
  counterfactual set adds three more axes without changing that.**

### Per-axis advantage budget, case 2, canonical vs the 50th-best clue

| axis | canonical | 50th | Δ raw | weight | max gain | pool clues above |
|---|---|---|---|---|---|---|
| `SIMILARITY` | 0.8036 | 0.8169 | −0.0134 | 0.25 | −0.0033 | 12,309 |
| `NOVELTY` | 0.6667 | 1.0000 | −0.3333 | 0.15 | −0.0500 | 14,531 |
| `WORD_NOVELTY` | 1.0000 | 1.0000 | 0.0000 | 0.15 | 0.0000 | 0 |
| `FAMILIARITY` | 0.3987 | 0.6505 | −0.2518 | 0.10 | −0.0252 | 16,986 |
| `RHYTHM` | 1.0000 | 1.0000 | 0.0000 | 0.30 | 0.0000 | 0 |
| `SHAPE` | 1.0000 | 1.0000 | 0.0000 | 0.05 | 0.0000 | 0 |
| `CLOSED_CLASS` | 0.0000 | 0.0278 | −0.0278 | −0.15 | −0.0042 | 14,378 |
| `PUNCH` | 0.8000 | 1.0000 | −0.2000 | 0.10 | −0.0200 | 10,265 |
| `PARSIMONY` *(new)* | 1.0000 | 0.8333 | **+0.1667** | — | +0.1667 | 0 |
| `RESEG` *(new)* | 0.5000 | 0.0000 | **+0.5000** | — | +0.5000 | 0 |
| `CLEAN` *(new)* | 0.5000 | 0.0000 | **+0.5000** | — | +0.5000 | 0 |

The canonical's *only* advantages are word-count parsimony (0.167) and boundary fidelity (0.500),
against deficits of 0.0500 + 0.0252 + 0.0200 + 0.0033 + 0.0042 on the shipped axes. The three
axes it saturates (`WORD_NOVELTY`, `RHYTHM`, `SHAPE`, all 1.0 with zero pool spread above it)
cannot be traded: they are worth nothing to it.

## 5. Per-axis lift tables for the head, before and after

Case 2 (`it's just a stupid game`), pool mean over 18,949:

| axis | pool | C0 head (lift) | C1b head (lift) | C9 head (lift) | C10 head (lift) |
|---|---|---|---|---|---|
| `SIMILARITY` | 0.8271 | 0.8091 (**−0.0180**) | 0.8840 (**+0.0570**) | 0.8560 (+0.0289) | 0.8677 (+0.0406) |
| `NOVELTY` | 0.8064 | 1.0000 (+0.1936) | 0.8124 (+0.0060) | 0.5000 (−0.3064) | 0.5000 (−0.3064) |
| `WORD_NOVELTY` | 0.9072 | 1.0000 (+0.0928) | 1.0000 (+0.0928) | 1.0000 (+0.0928) | 0.9733 (+0.0661) |
| `FAMILIARITY` | 0.5938 | 0.7159 (+0.1221) | 0.7315 (+0.1377) | 0.6221 (+0.0283) | 0.6429 (+0.0491) |
| `RHYTHM` | 0.7237 | 1.0000 (+0.2763) | 1.0000 (+0.2763) | 1.0000 (+0.2763) | 1.0000 (+0.2763) |
| `SHAPE` | 0.9681 | 0.9890 (+0.0209) | 0.9914 (+0.0233) | 0.9857 (+0.0176) | 0.9828 (+0.0147) |
| `CLOSED_CLASS` | 0.0586 | 0.0268 (−0.0319) | 0.0312 (−0.0274) | 0.0250 (−0.0336) | 0.0250 (−0.0336) |
| `PUNCH` | 0.8274 | 0.9800 (+0.1526) | 0.8000 (−0.0274) | 1.0000 (+0.1726) | 1.0000 (+0.1726) |
| `PARSIMONY` *(new)* | 0.8657 | 0.8500 (−0.0157) | 1.0000 (+0.1343) | 0.8333 (−0.0323) | 0.8333 (−0.0323) |
| `RESEG` *(new)* | 0.2466 | 0.0000 (−0.2466) | 0.3050 (+0.0584) | 0.7500 (+0.5034) | 0.7300 (+0.4834) |
| `CLEAN` *(new)* | 0.2656 | 0.0000 (−0.2656) | 0.3050 (+0.0394) | 0.6250 (+0.3594) | 0.6250 (+0.3594) |

Case 1 (`recognize speech`), pool mean over 18,289:

| axis | pool | C0 head (lift) | C1b head (lift) | C9 head (lift) |
|---|---|---|---|---|
| `SIMILARITY` | 0.7953 | 0.8561 (+0.0609) | 0.8581 (+0.0628) | 0.8594 (+0.0641) |
| `NOVELTY` | 0.8841 | 1.0000 (+0.1159) | 0.6933 (−0.1907) | 0.6667 (−0.2174) |
| `WORD_NOVELTY` | 0.9484 | 1.0000 (+0.0516) | 1.0000 (+0.0516) | 1.0000 (+0.0516) |
| `FAMILIARITY` | 0.5302 | 0.6181 (+0.0879) | 0.6599 (+0.1297) | 0.6553 (+0.1251) |
| `RHYTHM` | 0.5115 | 1.0000 (+0.4885) | 1.0000 (+0.4885) | 1.0000 (+0.4885) |
| `SHAPE` | 0.9669 | 0.9865 (+0.0195) | 0.9743 (+0.0074) | 0.9722 (+0.0053) |
| `CLOSED_CLASS` | 0.0322 | 0.0312 (−0.0010) | 0.0312 (−0.0010) | 0.0300 (−0.0022) |
| `PUNCH` | 0.9026 | 1.0000 (+0.0974) | 1.0000 (+0.0974) | 1.0000 (+0.0974) |
| `PARSIMONY` *(new)* | 0.4680 | 0.5000 (+0.0320) | 0.5000 (+0.0320) | 0.5000 (+0.0320) |
| `RESEG` *(new)* | 0.3142 | 0.0000 (−0.3142) | 0.9200 (+0.6058) | 1.0000 (+0.6858) |
| `CLEAN` *(new)* | 0.1680 | 0.0000 (−0.1680) | 0.4600 (+0.2920) | 0.5000 (+0.3320) |

Three things are worth keeping from these tables even though nothing ships:

1. **C1b repairs the one objectively wrong thing in the head.** The shipped top 50 is
   acoustically *worse* than the pool mean (lift **−0.0180**); under C1b the same target's top 50
   is acoustically **better** than the pool mean (lift **+0.0570**), and its `PUNCH` lift goes
   from +0.1526 to −0.0274 and its `RHYTHM` head stops being a monotone artefact of "six short
   words". This is a *general* quality improvement to the objective and it is available for free
   by deleting `SHAPE` and reading the clue's word count. It does not reach the canonical, so it
   is **not** proposed here - a change that does not advance a stated goal should be its own item
   with its own evidence, not smuggled in on the back of this one.
2. **The green case is provably immune to word-count parsimony.** Its pool's top 200 is *100%
   four-word clues* (histogram `{4: 200}`), and its `PARSIMONY` head lift is +0.0320 both before
   and after. Whatever the merits of the axis, it cannot regress this case - which is why C1 is
   admissible at all where C1b/C1c are not.
3. **`RESEG`/`CLEAN` head lift is large and positive in both tables** (up to +0.6858), so these
   are not vacuous axes: the head genuinely does select for boundary destruction, and the head's
   `RESEG` mean is 0.0000 in every configuration while the pool mean is 0.2466/0.3142. The
   mis-specification is real, measured, and large. It is simply not the binding constraint.

## 6. Verdict

* **`NOVELTY`'s mis-specified property: *boundary reuse is the failure mode*.** Measured: among
  14,109 reworded candidates, r(`NOVELTY`, `SIMILARITY`) = −0.316 and r(fidelity, `SIMILARITY`) =
  +0.341; across the whole pool, mean `SIMILARITY` rises 0.8082 → 0.8845 and mean shipped score
  falls 0.8618 → 0.5414 as shared boundaries go 0 → 4; and the property is already measured
  independently and exactly by `WORD_NOVELTY`.
* **`FAMILIARITY`'s mis-specified property: *per-word frequency is a proxy for phrase
  readability*.** Measured: r with word length −0.412, with closed-class share +0.615 (the
  opposite-signed term's own input), with word count +0.470, with `SIMILARITY` −0.206 / −0.550.
* **Neither mis-specification is the blocker.** The strongest possible correction of the first
  (C9, `NOVELTY` deleted outright with nothing given up) reaches rank 2,755 of 18,949 and drops
  the green case to 2,517. The conjunction that is *not* a boundary inversion (C10/C11/C12) does
  no better. The two `FAMILIARITY` inversions are worse than shipping (9,055 / 9,684). And the
  weight-free bound in §3 puts a floor of **rank 1,127** under *every* monotone objective over
  the measured properties.
* **Therefore: no re-specification of the non-acoustic axes can put the canonical in the top 50
  for its own target, and no candidate that helps case 2 keeps `wreck a nice beach` in the list.**
  Reported explicitly as required by the item: *the best candidate cannot clear 0.0944 without
  regressing the green case, and cannot clear it at all.*

**The residual, correctly scoped, is not an objective question.** Case 2 needs a *better tuple* for
`it's just a stupid game` - one that is acoustically faithful *and* parsimonious *and*
boundary-faithful, i.e. a real top-50 answer - not a re-rating of an ordinary one. That is the
enumeration front's surface (w-4d7c12), and this front does not touch it.

## 7. Tree state and constraints

* `cargo test --release --lib`: **74 passed / 0 failed / 2 ignored** (both ignored are this
  front's and w-6b2e19's `#[ignore]`d printers). Identical to base.
* `cargo test --release --test no_phrase_hard_coding`: **9 passed / 0 failed**. No token from
  either example appears anywhere in `src/` outside `#[cfg(test)]` modules; the only occurrences
  are the canonical phrases passed as *data* to the `#[ignore]`d printer, inside the test module.
* `cargo test --release --test corpus_integration -- --test-threads=2`: **12 passed / 1 failed** -
  `approximate_finds_classic_madgab_resegmentation`, the known pre-existing red at base, not
  re-pinned and not made worse. Displayed list for case 2 is byte-identical to base.
* **Wall clock**, release binary, `madgab --approximate --top 50`, 3 runs each, whole process
  including corpus load: `recognize speech` 1282 / 1344 / 1324 ms, `It's just a stupid game`
  1414 / 1343 / 1357 ms. Unchanged from base by construction - this front adds no work to any
  production path, and the release binary is byte-behaviour-identical (the sole non-`cfg(test)`
  diff line is `let m = self.metrics(..); let score = m.combined;`).
* Release mode for every measurement. `cargo fmt --check` and `cargo clippy` **cannot run on this
  host** (see [../../environment-notes.md](../environment-notes.md)) and are not claimed.
* No production change, so no red/green regression test is added: this front's deliverable is the
  measurement, and the item's contract is that a production change requires such a test. A change
  that did not satisfy the item's own success criteria would not have been made merely to have
  one.
* All instrumentation is behind `#[cfg(test)]` in `src/lib.rs` on `madgab-objaxis-3a8c05`. It is
  never to be integrated as production source. No `ZZ_*` probes, no environment knobs, no
  baseline changes, no debug binaries. No search, shortlist, budget or retention code touched.

## Handoff

* Front is terminal: **HOLD**. `docs/work/REPORT-3a8c05.md` on `madgab-objaxis-3a8c05`.
* The scoring-objective question is **closed as refuted with a bound**, not merely with a table:
  rank ≥ 1,127 for any monotone objective over measurable per-candidate properties (§3). A later
  pass should not re-open "re-weight or re-specify the seven non-acoustic axes" for this target.
* Two findings are *positive* and worth their own items, both evidenced here and neither shipped:
  1. **`SHAPE` is the least efficient term in the score** (0.05 weight, +0.0209 head lift, and
     pool-wide r with everything ≤ 0.33) and deleting it for a word-count-parsimony term flips
     the top 50's `SIMILARITY` lift from −0.0180 to +0.0570 (§5). This is a general objective
     improvement; it does not serve either canonical case's margin.
  2. **`FAMILIARITY` and `CLOSED_CLASS` are one statistic with opposite signs** (r = +0.615) and
     should be re-specified as a pair - content-word familiarity plus a separate
     readability/lexicality term - rather than tuned independently. The green canonical case sits
     on that pair: +0.0248 from one term, −0.0094 from the other, for the same determiner.
* Case 2's blocker is now attributable to a *tuple quality* deficit, not a scoring deficit and not
  a reachability deficit. The correct next front is one that asks the enumeration side for a
  top-50-quality answer to `it's just a stupid game`, and the correct first question for it is
  whether the search's retention admits any 5-word, boundary-faithful, acoustically faithful
  tuple for that target at all.
* `main` and `post-milestone-acceptance` are untouched; this branch is pushed and stops here for
  a coordinator pass.
