# w-d4e8b1 - the phonetic-cost term is priced, and it is not what holds the canonical case-2 clue out

Front `agent-d4e8b1`, branch `madgab-phon-d4e8b1` (report) and the **unpushed** scratch branch
`phon-probe-d4e8b1` (all instrumentation), base `44b427f`. Release mode throughout.
**Outcome: HOLD** - a priced negative, no production change.

Question the item asks: is the canonical case-2 alignment (`hits justice dupe hid came` for
`It's just a stupid game`, 0.8199 against a 0.9295 cutoff) held out by a general
pronunciation / phonetic-distance defect, or is its deficit explained by the two coordinates
already priced? **Answer, measured: the phonetic-cost term accounts for 0.0033385585 of the
0.0943520415 deficit - 3.54% - and a *perfect* pronunciation of the same wording at the same
cuts still scores 0.8698712792, 0.0452502953 below the cutoff. The remaining 96.46% sits in
`NOVELTY` (+0.0500000000), `FAMILIARITY` (+0.0251801497), `PUNCH` (+0.0200000000) and
`CLOSED_CLASS` (-0.0041666667) - the objective axes, which `REPORT-3a8c05.md` priced and which
are owned by a live front. That is a non-contending surface, so per the item's own rule this is a
`HOLD`, and the residual is stated here rather than chased.**

Every number below was produced by the production scorer. No axis was re-derived: the probe
reproduces `Clue::score` **bit-exactly on all 18,949 + 18,289 pool clues of both targets**
(assertion inside `check_terms`), so the term tables are decompositions of the shipped number.

---

## 0. Baseline: reproduced, with one stale count recorded

`cargo test --release --lib d4e8b1_baseline_is_reproduced -- --ignored --nocapture`

| quantity | item's number | measured on `44b427f` | verdict |
|---|---|---|---|
| case-2 pool | 18,949 | **18,949** | confirmed |
| case-1 pool | (18,289 per OBSTRUCTION-MAP) | **18,289** | confirmed |
| canonical score | 0.8207695329 | **0.8207695329** | confirmed to all 10 digits |
| canonical per-word costs | (0.20 / 0.00 / 0.15 / 0.3695 / 0.40 per REPORT-6b2e19) | **0.2000000000 / 0.0000000000 / 0.1500000000 / 0.3695198142 / 0.4000000000** | confirmed |
| case-2 cutoff | 0.9151215745 | **0.9151215745** (`each thus tough too pad same`) | confirmed |
| gap | 0.0943520415 | **0.0943520415** | confirmed |
| green case-1 control | 0.9199502875 at pool rank 27 | **0.9199502875 at rank 27**, displayed 26/50 | confirmed |
| canonical in the pool | absent | **absent** (rank -1) | confirmed |

**One disagreement with the item's fence, recorded rather than reconciled:** the item's validation
fence says `cargo test --release --lib` is `74/0`. On `44b427f` it is **75 passed / 0 failed /
9 ignored**, and on the probe branch **75 passed / 0 failed / 12 ignored** (the three extra
ignored tests are this front's own printers). Zero failures either way, so the fence's substance
holds; the pass count in the item text is stale by one.

## 1. What the phonetic term is, in the shipped code

`src/lib.rs:2964-2966`, inside `Partial::metrics`:

```
cost_per_phone = sub_cost_total / target_phones
similarity     = (1 - cost_per_phone / 0.30).clamp(0, 1)      // axes::SIMILARITY_COST_PER_PHONE
```

so the axis is `0.25 * (1 - total_edit_cost / (target_phones * 0.30))`. Its two structural
properties, both already documented in the code and both re-measured here:

* **It is a function of the total cost and the target's phone count only.** Phone distribution
  across words is already normalised away (the per-span and per-word readings are gone; the
  comment at `src/lib.rs:2928-2957` is the history).
* **Its floor is the search's own cost budget.** `SearchMode::approximate()` caps
  `sub_cost_total` at 1.5, so on this 19-phone target no candidate can score below
  `1 - 1.5/19/0.3 = 0.7368421053`. The pool's observed minimum raw `SIMILARITY` is **0.7368** at
  every word count, which is that cap and not a phonetic fact.

## 2. Term-by-term, both cases

`cargo test --release --lib d4e8b1_term_by_term_both_cases -- --ignored --nocapture`.
Deltas are *cutoff clue minus candidate*, so a positive delta is the deficit.

### Case 2, `It's just a stupid game` -> `hits justice dupe hid came`

Target IPA `ɪtsdʒʌstəstupədɡeɪm`, 19 phones, 6 syllables, inner boundaries at `{3, 8, 9, 15}`.
Canonical cuts `[3, 10, 13, 15, 19]`, total cost **1.1195198142** = 0.0589220955 per phone.

| term | weight | canonical raw | canonical weighted | cutoff raw | cutoff weighted | **deficit** | share of gap |
|---|---|---|---|---|---|---|---|
| **SIMILARITY (the phonetic term)** | 0.25 | 0.8035930151 | 0.2008982538 | 0.8169472487 | 0.2042368122 | **+0.0033385585** | **3.54%** |
| NOVELTY | 0.15 | 0.6666666667 | 0.1000000000 | 1.0000000000 | 0.1500000000 | +0.0500000000 | 52.99% |
| WORD_NOVELTY | 0.15 | 1.0000000000 | 0.1500000000 | 1.0000000000 | 0.1500000000 | 0.0000000000 | 0.00% |
| FAMILIARITY | 0.10 | 0.3987127920 | 0.0398712792 | 0.6505142857 | 0.0650514289 | +0.0251801497 | 26.69% |
| RHYTHM | 0.30 | 1.0000000000 | 0.3000000000 | 1.0000000000 | 0.3000000000 | 0.0000000000 | 0.00% |
| SHAPE | 0.05 | 1.0000000000 | 0.0500000000 | 1.0000000000 | 0.0500000000 | 0.0000000000 | 0.00% |
| CLOSED_CLASS | -0.15 | 0.0000000000 | -0.0000000000 | 0.0277777778 | -0.0041666667 | **-0.0041666667** | -4.42% |
| PUNCH | 0.10 | 0.8000000000 | -0.0200000000 | 1.0000000000 | 0.0000000000 | +0.0200000000 | 21.20% |
| **total** | | | **0.8207695329** | | **0.9151215745** | **+0.0943520415** | 100% |

The seven non-phonetic terms hold **+0.0910134830**, i.e. **96.46%** of the deficit, and the
`REPORT-6b2e19` §2 attribution is confirmed digit for digit (`NOVELTY` dominant at 53%,
`FAMILIARITY` 27%, `PUNCH` 21%, `CLOSED_CLASS` running the other way). The phonetic term is a
**3.54%** term, and `REPORT-6b2e19` had already recorded it as "a near tie" (it said 3.5%); this
front measures it at the same number and then bounds it.

### Case 1 control, `recognize speech` -> `wreck a nice beach` (green)

Target IPA `ɹɛkəɡnaɪzspitʃ`, 14 phones, 4 syllables. Cuts `[3, 4, 10, 14]`, total cost
**0.5500000000**. Cutoff clue `wreck ugh nice peach` at 0.9187964409.

| term | green raw | green weighted | cutoff raw | cutoff weighted | deficit |
|---|---|---|---|---|---|
| **SIMILARITY** | 0.8690476190 | 0.2172619048 | 0.9260731707 | 0.2315183169 | +0.0142564122 |
| NOVELTY | 1.0000000000 | 0.1500000000 | 1.0000000000 | 0.1500000000 | 0.0000000000 |
| WORD_NOVELTY | 1.0000000000 | 0.1500000000 | 1.0000000000 | 0.1500000000 | 0.0000000000 |
| FAMILIARITY | 0.6206338280 | 0.0620633828 | 0.3727812400 | 0.0372781240 | **-0.0247852588** |
| RHYTHM | 1.0000000000 | 0.3000000000 | 1.0000000000 | 0.3000000000 | 0.0000000000 |
| SHAPE | 1.0000000000 | 0.0500000000 | 1.0000000000 | 0.0500000000 | 0.0000000000 |
| CLOSED_CLASS | 0.0625000000 | -0.0093750000 | 0.0000000000 | -0.0000000000 | -0.0093750000 |
| PUNCH | 1.0000000000 | 0.0000000000 | 1.0000000000 | 0.0000000000 | 0.0000000000 |
| **total** | | **0.9199502875** | | **0.9187964409** | **-0.0011538466** (green by 0.0012) |

The control's own margin is **+0.0011538466** and it is *made* of `FAMILIARITY` (+0.0248) net of
`SIMILARITY` (-0.0143). Its perfect-pronunciation ceiling is **0.9526883828** (+0.0327380952),
i.e. the green case is green *with* room to spare on this axis, and its margin of 0.0012 is an
order of magnitude smaller than any phonetic move discussed in §4 - which is itself an argument
against touching this term: there is no room on this side either.

## 3. What the canonical's 1.1195198142 is made of

Per-word, each word against the target phones it covers. `recomputed` is the shipped cost model
re-derived from the two phone strings by the probe's own DP, which agrees with the lattice on all
five words to the last digit - so the component split below is a split of the number the scorer
actually charged, not of a re-implementation.

| word | clue IPA | target span | clue phones | span phones | shipped cost | length-mismatch part | substitution part |
|---|---|---|---|---|---|---|---|
| `hits` | hɪts | ɪts | 4 | 3 | 0.2000000000 | 0.2000000000 | 0.0000000000 |
| `justice` | dʒʌstəs | dʒʌstəs | 7 | 7 | 0.0000000000 | 0.0000000000 | 0.0000000000 |
| `dupe` | dup | tup | 3 | 3 | 0.1500000000 | 0.0000000000 | 0.1500000000 |
| `hid` | hɪd | əd | 3 | 2 | 0.3695198142 | 0.3695198142 | 0.0000000000 |
| `came` | keɪm | ɡeɪm | 4 | 4 | 0.4000000000 | 0.0000000000 | 0.4000000000 |
| **total** | | | 21 | 19 | **1.1195198142** | **0.7695198142** | **0.5500000000** |

Per-word share of the axis, `0.25 * cost / (19 * 0.30)`, and the score each word would reach if
*it alone* were pronounced perfectly:

| word | cost | share of the axis | score with that word free | gain |
|---|---|---|---|---|
| `hits` | 0.2000000000 | 0.0087719298 | 0.8295414627 | +0.0087719298 |
| `justice` | 0.0000000000 | 0.0000000000 | 0.8207695329 | +0.0000000000 |
| `dupe` | 0.1500000000 | 0.0065789474 | 0.8273484803 | +0.0065789474 |
| `hid` | 0.3695198142 | 0.0162070094 | 0.8369765423 | +0.0162070094 |
| `came` | 0.4000000000 | 0.0175438596 | 0.8383133926 | +0.0175438596 |
| **all five** | 1.1195198142 | 0.0491017462 | **0.8698712792** | **+0.0491017462** |

**The ceiling.** 0.8698712792 is the *perfect-pronunciation* score of this wording at this
alignment - a zero-cost phonetic axis, the best any phonetic-cost rule can do to this tuple.
It is **0.0452502953 short of the 0.9151215745 cutoff**. This reproduces `REPORT-3a8c05.md`'s
"a zero-cost perfect pronunciation of it scores 0.8699" from the other direction, and it is the
single number that decides the item: **no rule on the phonetic term, however general, can put
this clue in the top 50.** Its residual 0.0452502953 is 0.0500000000 (`NOVELTY`) +
0.0251801497 (`FAMILIARITY`) + 0.0200000000 (`PUNCH`) - 0.0041666667 (`CLOSED_CLASS`) -
0.0457544212 (the axis itself now scoring full marks) - i.e. it is the priced objective-axis gap
and nothing else.

## 4. Is the term's behaviour general and correct? The counterfactual sweep

The item asks for a *general, phrase-free* rule if one exists. Five candidate rules were stated
as functions of the span, the slot, the candidate set and the two phone strings only, and priced
over both canonical cases and ten ordinary targets. Each is a **cost-model** change applied at
scoring time with the alignment, the wording and the other seven axes held fixed, so each row is
the phonetic term's own doing and pool *membership* never moves.

The models, in the vocabulary of `src/approx.rs`'s walk (insert a clue phone, delete a target
phone, both at `GAP_COST = 0.20`; substitute at the shipped table):

| # | rule (all phrase-free) | case-2 canonical | delta | residual gap to cutoff | green case rank (27 = shipped) | median delta over ordinary pools |
|---|---|---|---|---|---|---|
| 0 | shipped | 0.8207695329 | - | 0.0943520415 | **27** | 0 |
| - | *perfect pronunciation* (upper bound, not a rule) | 0.8698712792 | +0.0491017462 | **0.0452502953** | - | - |
| 1 | insertions and deletions free (charge segment substitutions only) | 0.9156705126 | +0.0949009797 | **-0.0005489381** | **1006** | +0.027 .. +0.068 |
| 2 | charge deletions only (clue-added material is free) | 0.8470853224 | +0.0263157895 | 0.0680362521 | 308 | +0.001 .. +0.028 |
| 3 | charge insertions only (target phones the word fails to say are free) | 0.8295414627 | +0.0087719298 | 0.0855801118 | 140 | +0.007 .. +0.028 |
| 4 | half the shipped gap cost | 0.8383133926 | +0.0175438596 | 0.0768081819 | 119 | +0.008 .. +0.028 |
| 5 | normalise by the mean of the target's and the clue's phone counts (symmetric normalised edit distance) | 0.8232246202 | +0.0024550873 | 0.0918969543 | **43** | -0.001 .. +0.000 |

Ordinary-target detail (cutoff move and top-50 turnover, ten targets, pools 15,942-21,165):

| target | r1 substitutions-only | r2 deletions-only | r3 insertions-only | r4 half-gap | r5 symmetric norm |
|---|---|---|---|---|---|
| a quick brown fox | cutoff +0.0617, head 50/50 | +0.0148, 50/50 | +0.0280, 50/50 | +0.0186, 45/50 | -0.0025, 45/50 |
| the old man and the sea | +0.0735, 50/50 | +0.0319, 49/50 | +0.0308, 43/50 | +0.0303, 44/50 | +0.0011, 49/50 |
| she sells sea shells | +0.0649, 50/50 | +0.0138, 48/50 | +0.0334, 48/50 | +0.0210, 50/50 | -0.0031, 47/50 |
| my grandmother started to sing | +0.0439, 45/50 | +0.0115, 47/50 | +0.0256, 50/50 | +0.0153, 48/50 | -0.0014, 47/50 |
| the rain in spain falls mainly on the plain | +0.0333, 50/50 | +0.0041, 38/50 | +0.0167, 42/50 | +0.0093, 38/50 | -0.0009, 37/50 |
| pack my box with five dozen liquor jugs | +0.0382, 46/50 | +0.0035, 47/50 | +0.0257, 49/50 | +0.0136, 49/50 | -0.0026, 48/50 |
| there is no place like home | +0.0475, 49/50 | +0.0147, 50/50 | +0.0269, 50/50 | +0.0169, 50/50 | -0.0010, 47/50 |
| open the door and come inside | +0.0332, 49/50 | +0.0064, 41/50 | +0.0107, 50/50 | +0.0073, 47/50 | -0.0003, 37/50 |
| he bought a new pair of shoes | +0.0497, 50/50 | +0.0127, 50/50 | +0.0152, 50/50 | +0.0115, 50/50 | -0.0006, 49/50 |
| the train leaves at noon tomorrow | +0.0381, 50/50 | +0.0083, 49/50 | +0.0175, 50/50 | +0.0109, 35/50 | -0.0006, 43/50 |

Reading the sweep:

* **Rule 1 is the only one that reaches the cutoff, and it reaches it by making the term measure
  nothing.** With insertions and deletions free, *every* mispronunciation can be routed as
  delete-then-insert at zero cost, so the substitution component is **0.000000 across all 18,949
  case-2 pool clues** (measured: length-mismatch 18,676.014827, segment substitution 0.000000,
  100.0000% mismatch). A `SIMILARITY` axis that reads 1.0 for every candidate is not a phonetic
  term. It buys the canonical +0.0949 by lifting the median pool clue by +0.027..+0.068,
  rewriting **45-50 of every ordinary target's top 50**, and dropping the green case-1 control
  from **rank 27 to rank 1006**. Refuted as a fix, as a shared fix, and as a term.
* **Rules 2, 3 and 4 are monotone relaxations of the same quantity and all fail on both
  sides.** They buy the canonical +0.0088 to +0.0263 (residual 0.068 to 0.086 - nowhere near
  0.0944) and cost the green case 27 -> 308 / 140 / 119. There is no setting of the gap cost
  between 0.20 and 0 that both keeps the green case at 27 and lifts the canonical 0.0453, because
  every unit of gap relaxation lifts the whole pool by more than it lifts this tuple.
* **Rule 5 is the standard, defensible normalisation and it is worth +0.0025.** It is also not
  free: the green case moves 27 -> 43 and the top-50 churns on 8 of 10 ordinary targets. A
  general improvement that costs the standing green fact 16 ranks for 0.0025 of a 0.0944 gap is
  a bad trade, and it is not landed.
* **The cost of every row above is zero candidate visits and zero wall clock**, because each is
  a rescoring of the retained pool at `finish` time; the price is paid entirely in ranking
  quality. The alternative place to move this term - inside
  `approx::FuzzyLexicon::matches_at` (`src/approx.rs:109`, `GAP_COST` at `src/approx.rs:9`) -
  is **not this front's surface**: that cost is read by the span shortlist fill
  (`src/lib.rs:1004`), by `SlotAlt::contribution` and the slot extremes (`src/lib.rs:1125`,
  `:1236`), by the structural DP's `partial_span_score` (`src/lib.rs:3678`), by the partial
  scorer (`src/lib.rs:4700`) and by the per-slot ordering comparator (`src/lib.rs:7915`), i.e.
  by the shortlist, the width rules and the slot order that `w-2f1c03` and the priced reports
  own. Per the item's non-contending rule, this front stops there and reports it.

## 5. The defect hypotheses, one by one

The item lists five candidate systematic defects. Priced:

| hypothesis | verdict | number |
|---|---|---|
| per-phone cost asymmetry | **not a defect on this surface**: the axis normalises by the target's phone count and nothing else; changing the normaliser (rule 5) is worth +0.0025 | 0.0024550873 |
| unnormalised segment counts | **already fixed in the shipped code** and re-measured: the axis is a function of total cost / target phones, and the two canonical cases confirm the published per-word costs | see §3 |
| a cost floor/ceiling interacting with alignment length | **exists, and it is a floor, not a charge against the canonical**: the axis cannot read below 0.7368421053 because `total_budget = 1.5` caps `sub_cost_total`. It flattens the bottom of the scale for every candidate, canonical included; it does not single out this shape | 0.7368421053 |
| corpus / transcription normalisation gap | **not found**: the probe's own DP over the two normalised phone strings reproduces the lattice cost exactly on all five canonical words and on the pool | 0.0 difference |
| an ordering effect on this *shape* of alignment | **refuted, and it points the other way**: mean raw `SIMILARITY` *rises* with clue word count on the case-2 target (4 words 0.799293, 5 words 0.831271, 6 words 0.841119), so a five-word resegmentation is not systematically charged more than a four-word one. The canonical's raw 0.803593 sits at the **25.65th percentile of the 5,813 five-word pool clues** and beats **12,309 of 18,949** pool clues overall | see below |

```
raw SIMILARITY by clue word count, case 2 pool
  4 words: n  1718  mean 0.799293  min 0.736919  max 0.921771
  5 words: n  5813  mean 0.831271  min 0.736864  max 1.000000
  6 words: n  9070  mean 0.841119  min 0.736842  max 1.000000
  7 words: n  2158  mean 0.785575  min 0.736842  max 0.964912
  8 words: n   160  mean 0.752020  min 0.736898  max 0.799196
  9 words: n    30  mean 0.753314  min 0.739472  max 0.789474
canonical raw 0.803593: 4,322 of 5,813 five-word clues higher (percentile 0.2565)
                     12,309 of 18,949 pool clues higher
```

One real structural fact does come out of the decomposition, and it is worth recording as a
successor observation rather than a fix: **the shipped cost is 68.7% length mismatch
(0.7695198142 of 1.1195198142) and 49.1% segment substitution (0.5500000000) - and
`SIMILARITY` therefore charges a candidate partly for *where its word boundaries fall*, which is
what `NOVELTY` (0.15) and `RHYTHM` (0.30) already charge for structurally.** Two axes reading
the cut vector is a design observation about the objective, i.e. `w-9b4a15`'s and
`w-3a8c05`'s surface, not a phonetic-distance defect, and it is exactly why rule 1 (free
gaps) looks so attractive and is so bad.

## 6. Why this is a HOLD and not a rule

1. The phonetic term's share of the deficit is **0.0033385585 / 0.0943520415 = 3.54%**.
2. Its **entire achievable range** on this alignment is +0.0491017462, and using all of it
   still leaves **0.0452502953** of deficit, in `NOVELTY`, `FAMILIARITY`, `PUNCH` and
   `CLOSED_CLASS` - the five objective axes `REPORT-3a8c05.md` priced and `agent-9b4a153` is
   editing.
3. Every general, phrase-free rule on this surface that moves the number at all (five of them
   priced) **regresses the standing green case-1 control** from pool rank 27 to 43, 119, 140, 308
   or 1006, and churns 35-50 of every ordinary target's top 50.
4. The only surface where the term could still be changed materially is
   `approx::FuzzyLexicon::matches_at`, which feeds the shortlist fill, `SlotAlt::contribution`,
   the slot widths and the per-slot order - **owned by `w-2f1c03` and the priced reports**, and
   explicitly out of scope for this item.

So the deficit is **not** attributable to the phonetic term, and the residual lives on a
non-contending surface. Nothing is landed; there is no `INTEGRATE` production commit to identify.

## 7. Tree state, fences, and lessons

Fence on the probe branch (`phon-probe-d4e8b1`, unpushed), run **one suite at a time** - the host
OOM-killed two other agents today and serial execution is a survival requirement, not a
performance note:

| suite | result |
|---|---|
| `cargo test --release --lib` | **75 passed / 0 failed / 12 ignored** (base `44b427f`: 75 / 0 / 9; the item's fence text says 74/0 - see §0) |
| `cargo test --release --test emit_coverage` | **4 / 0** |
| `cargo test --release --test approx_determinism` | **4 / 0** |
| `cargo test --release --test exact_determinism` | **1 / 0** |
| `cargo test --release --test no_phrase_hard_coding` | **9 / 0** |
| `cargo test --release --test corpus_integration -- --test-threads=2` | **12 passed / 1 failed** - the known pre-existing red `approximate_finds_classic_madgab_resegmentation`, not re-pinned, not made green |

* **Phrase fence observed.** All instrumentation is `#[cfg(test)]`: a `front_d4e8b1` module at
  the end of `src/lib.rs` and two accessors in `src/approx.rs`
  (`substitution_cost`, `GAP_COST_FOR_TEST`), both in `#[cfg(test)]` regions that the fence's
  scanner excludes. No production source line is changed on either branch, no literal token,
  clue or sentence is special-cased anywhere, and the five counterfactual rules are functions of
  the two phone strings and the shipped segment costs only - they read no word, no clue and no
  target.
* `cargo fmt` and `cargo clippy` **cannot run on this host** (no `rustup`, no `rustfmt`/
  `clippy` components) and are not claimed. The probe is on an unpushed scratch branch for
  exactly that reason: it is measurement apparatus, not a landing.
* Not self-merged. `madgab-phon-d4e8b1` carries this report and nothing else; integration is a
  review front's call.

### Reproduce

    cargo test --release --lib d4e8b1_baseline_is_reproduced -- --ignored --nocapture
    cargo test --release --lib d4e8b1_term_by_term_both_cases  -- --ignored --nocapture
    cargo test --release --lib d4e8b1_is_the_phonetic_term_general -- --ignored --nocapture

(probe branch `phon-probe-d4e8b1`; the third takes ~110 s in release.)

## 8. Handoff

* Front terminal: **HOLD**. This report is the deliverable; there is no production change and no
  `INTEGRATE` commit to integrate.
* The item's premise is settled in the negative and should not be re-opened: the phonetic-cost
  term is 3.54% of the case-2 deficit, its whole achievable range leaves 0.0452502953 of deficit,
  and that residual is `NOVELTY` + `FAMILIARITY` + `PUNCH` + `CLOSED_CLASS`. The next move on
  this target is the objective-axis front, not a pronunciation front.
* Two observations handed on, neither owned here: (a) `SIMILARITY` has a **floor at 0.7368421053
  on a 19-phone target** imposed by `total_budget = 1.5`, so the bottom of the acoustic scale is
  flat for every candidate; (b) the shipped cost is 100% length mismatch across the entire
  case-2 pool once free gaps are allowed, i.e. the axis is largely reading the cut vector a
  second time. Both are objective-design questions for the front that owns the axes.
* The counterfactual harness (five cost models x two normalisers, ten ordinary targets, both
  canonical cases, green-case rank reported every time) is on `phon-probe-d4e8b1` and is
  reusable by any successor that wants to price another axis on the same footing.

## Verdict

**HOLD**
