---
work_item: true
id: w-c4d7e8
state: done
priority: normal
owner: c4d7e8
updated: 2026-09-27T03:10:00Z
branch: madgab-costaudit-c4d7e8
worktree: /workspace/madgab-costaudit-c4d7e8
---

# Cost/pronunciation/distance audit: case 2 is an objective-axis gap, not a cost gap

## Goal

Audit the four general fronts nobody had measured — pronunciation-data coverage of the
`dupe` sound, the per-word cost model under a five-word alignment, the phonetic distance
function, and the clue-normalisation input path — and decide with arithmetic which, if
any, is worth building on for the canonical case-2 answer
`It's just a stupid game` -> `Hits Justice Dupe Hid Came`.

## Result: all four are refuted, with numbers

Baseline reproduced on `648b4e0`, release build, `--approximate --top 50`:

- `recognize speech`: `wreck a nice beach` at rank 28, score 0.918. Case 1 passes.
- `Its just a stupid game`: rank 1 is `it justice too bad aim` 0.9052; rank 50 is 0.8978;
  no `dupe` in the list.

### 0. The premise "the target is not in the pool" is FALSE

Instrumented `FuzzyLexicon::matches_at` over the real lattice for
`/ɪtsdʒʌstəstʊpədɡeɪm/` (19 phones, `targets` inner boundaries `[8, 9, 15]`, 6 syllables):

- `dupe` `/dup/` is present at `start=10, span=3, cost=0.150` and `start=11, span=2,
  cost=0.372`. It is the **cheapest** 3-phone realisation of the `/tʊp/` region.
- `dupe` also survives the rarity filter: `rarity=40210.0` (< 50000).
- Exhaustive enumeration of 5-word alignments using the real lattice (top 5 words per
  slot, 3060 cut vectors) yields **1,545,903** candidates, of which **40,546 contain
  `dupe`**. The best of them is `it justice dupe add aim` at 0.8834, rank 130.

So the canonical answer is *in the pool* and *ranks out*. That is the opposite of the
prior conclusion and it changes the fix.

### 1. Exact alignment of the canonical wording, and the score decomposition

Exhaustive search over **all** contiguous `(start, span)` placements of the exact word
sequence `hits justice dupe hid came` under `per_word_budget = 0.5`, `total_budget = 1.5`
finds only **3** admissible alignments. Best is cuts `[3, 10, 13, 15, 19]`:

| word | span | IPA | sub_cost |
|---|---|---|---|
| hits | 3 (`ɪts`) | `/hɪts/` | 0.200 (insert `h`) |
| justice | 7 (`dʒʌstəst`) | `/dʒʌstəs/` | 0.000 |
| dupe | 3 (`tʊp`) | `/dup/` | 0.150 |
| hid | 2 (`əd`) | `/hɪd/` | 0.370 |
| came | 4 (`ɡeɪm`) | `/keɪm/` | 0.400 |

Total cost **1.1195** of the 1.5 global budget. Axis terms:

```
SIMILARITY   0.25 * (1 - 1.1195/4) = 0.1800
NOVELTY      0.15 * 0.6667          = 0.1000
WORD_NOVELTY 0.15 * 1.0             = 0.1500
FAMILIARITY  0.10 * 0.3987          = 0.0399
RHYTHM       0.30 * 1.0             = 0.3000
SHAPE        0.05 * 1.0             = 0.0500
CLOSED_CLASS 0.0                     = -0.0000
PUNCH        0.10 * (0.8 - 1)       = -0.0200
                                 total = 0.7999   (matches the prior front's 0.799901)
```

Gap to rank 50 (0.8978) is **+0.0979**. It decomposes as:

- **NOVELTY -0.0500** — the biggest single term. Clue inner cuts `{3,10,13,15}` share
  only `{15}` with the target's inner boundaries `{8,9,15}`, so Jaccard novelty is
  0.6667 instead of 1.0. The canonical answer is *penalised for preserving target word
  boundaries*.
- **FAMILIARITY -0.0330** — `dupe` `rarity=40210` -> `word_familiarity = 0.0351`
  (`src/lib.rs:2628`), which alone drags the 5-word mean from 0.599 down to 0.3987.
- **SIMILARITY -0.0293** — the only term any cost-model change can move.

### 2. THE REFUTATION: cost, pronunciation data and distance cannot close the gap

Recomputing the same alignment with the phonetic cost varied and nothing else changed:

| alignment cost | SIMILARITY term | total | vs rank-50 (0.8978) |
|---|---|---|---|
| 1.1195 (actual) | 0.1800 | 0.7999 | -0.0979 |
| 0.75 (all substitutions at a capped 0.10) | 0.2031 | 0.8230 | -0.0748 |
| 0.40 (every substitution free, indels kept) | 0.2250 | 0.8449 | -0.0529 |
| **0.00 (perfect pronunciation, free)** | **0.2500** | **0.8699** | **-0.0279** |

**A zero-cost, perfect pronunciation of the canonical wording still scores 0.8699 and
still misses the top 50 by 0.0279.** No change to the pronunciation dictionary, to
`GAP_COST`, to the per-word cost model, or to the substitution weighting can put this
answer in the top 50. The remaining headroom (0.8699 -> 0.9199) is reachable only by
raising `NOVELTY`, i.e. by changing the *objective's preference for boundary-shifted
clues* — the visibility/selection front, already refuted four times, not a cost front.

This is the same conclusion the earlier pass reached from the pool side; this pass
reaches it from the cost side and, unlike the pool-side argument, it is an upper bound
that does not depend on how the pool is enumerated.

### 3. Area-by-area

**(a) Pronunciation-data coverage — REFUTED as a cause.** `dupe` `/dup/`, `hits`
`/hɪts/`, `hid` `/hɪd/`, `came` `/keɪm/` are all in the corpus with rarity under
50000. The `/d/` + `/up/` sound is reached at cost 0.150, the minimum available for
that span. Derivation is `approx::build_lexicon` (`src/approx.rs:355`) calling
`Corpus::preferred_ipa` then `approx::normalize_ipa` (`src/approx.rs:296`), which
strips only `ˈ`/`ˌ` and nothing else. No drop, no mis-map.

**(b) Per-word cost model — REFUTED, and the canonical wording is *favoured* on three
of the six word-level axes.** `closed_class_penalty` (`src/lib.rs:2620`) squares the
closed-class share, so it is 0 for this clue against -0.006 for the winner `it justice
too bad aim` (+0.006 in the canonical's favour). `lexical_shape_quality`
(`src/lib.rs:2058`) only penalises 1-2 character words; all five canonical words score
1.00 versus 0.98 for the winner (+0.001 in its favour). `PUNCH` is -0.0200 for both.
`FAMILIARITY` is the only word-level term that hurts, and it hurts because `dupe` is
genuinely a rare word (40210), not because of any model defect. No term is scaled by
anything order-dependent: closed-class, familiarity, shape and punch are all
order-independent functions of the word multiset, and rhythm reads only the total
syllable count.

The one genuinely order-dependent, additive-per-word term is `SIMILARITY`:
`(1.0 - sub_cost_total / 4.0).clamp(0,1)` at `src/lib.rs:2377` divides the *total*
cost by the constant 4 rather than by word count or target length, so a longer clue
pays more absolute similarity. That is the real per-word bias, and the arithmetic above
prices its maximum possible value at **+0.0700** (the 1.1195 -> 0.0 row), which is
insufficient by 0.0279 even when fully collected.

**(c) Phonetic distance / metric compliance — a REAL but insufficient defect.**
`matches_at` (`src/approx.rs:70`) is a correct minimum-cost DP over `(trie node,
consumed)` states, so the distances it reports are true edit distances for the given
weighting. But the weighting is badly scaled, and measurably so: over the 55-symbol
IPA inventory used here there are 3066 distinct-symbol pairs, and

- **302 pairs cost strictly less than one `GAP_COST` (0.20)**, two of them **zero**:
  `ɕ`/`ç` are charged 0.0, i.e. a substitution is indistinguishable from an exact match.
  `r`/`ɾ` costs 0.0225.
- **2222 pairs (72.4%) cost more than `2 * GAP_COST` (0.40)** — strictly more than a
  delete plus an insert. For those pairs the search never performs a substitution at
  all; it silently takes the delete-then-insert path and reports the same 0.40.

`came` `/keɪm/` against `/ɡeɪm/` at cost exactly **0.400** is a live instance: the
`ɡ`->`k` substitution was priced out and rendered as an indel pair. The untested class
this opens, and the only one in this area not yet run, is a **feature-based
(place x manner x voicing x height) substitution cost bounded above by `GAP_COST`**,
which would collapse 72% of the symbol table into the substitution band instead of the
indel band. Priced: capping substitution at 0.10 and making it free both land at 0.8230
and 0.8449 respectively, i.e. **still 0.053-0.075 short of rank 50**. Recorded as a
real quality defect, refuted as a fix for this milestone.

**(d) Clue normalisation and the apostrophe — the brief's premise is REFUTED; the CLI
is fine.** `clean_input_word` (`src/lib.rs:1963`) lowercases and trims only trailing
`.,!?;:`. It does not strip apostrophes, and it does not need to: the corpus carries
`it's` -> `ˈɪts`, so `It's just a stupid game` transcribes to
`ɪtsdʒʌstəstʊpədɡeɪm` with boundaries `[3, 8, 9, 15, 19]`, **byte-identical to
`Its just a stupid game`**, and the executable runs it and prints the same top 3 at
0.905/0.905/0.904. Verified:

```
$ ./target/release/madgab --approximate --top 3 "It's just a stupid game"
target: It's just a stupid game
IPA:    /ˈɪtsdʒˈʌstəstˈupədɡˈeɪm/
 1. [0.905] it justice too bad aim
 2. [0.905] it said thus test oop dame
 3. [0.904] it justice too pad aim
```

`It-s` and `it.s` fail only because `it-s`/`it.s` are not dictionary keys at all, and
`transcribe_with_boundaries` returns `None` (`src/lib.rs:1951`), which the CLI reports
as `no clue coverings found` and exits 1. No silent normalisation loss.

The one real hazard here is the reverse of what was suspected: **`It is just a stupid
game` is NOT a safe substitute.** It transcribes to `ɪtɪzdʒʌstəstʊpədɡeɪm` — 20
phones, boundaries `[2, 4, 9, 10, 16, 20]`, 7 syllables — a different target with a
different length and a different boundary set. Anyone reproducing this case with `It is`
is measuring a different problem.

## Refuted, explicitly

- **The canonical case-2 answer is absent from the pool.** Refuted: 40,546 of 1,545,903
  enumerated 5-word candidates contain `dupe`; the best scores 0.8834 at rank 130.
- **The `dupe` sound is unreachable from the pronunciation data.** Refuted: cost 0.150
  at span 3, the cheapest available realisation.
- **The per-word cost model starves a five-word alignment.** Refuted: the canonical
  wording is *ahead* of the winner on closed-class (+0.006) and shape (+0.001), level
  on punch and rhythm, and its similarity cost of 1.1195 is not near the 1.5 budget.
- **Any change to the cost model or distance function can bring it into the top 50.**
  Refuted by the upper bound: perfect zero-cost pronunciation scores 0.8699 < 0.8978.
- **The apostrophe form is mangled on the way into the search.** Refuted: the CLI
  handles `It's` correctly and identically to `Its`.

## Completion criteria

- [x] Reproduce both canonical cases from a release build with `--top 50`.
- [x] Determine whether the canonical case-2 answer is in the pool or ranks out.
- [x] Measure the exact best alignment, its per-word costs and its score decomposition.
- [x] Audit pronunciation-data coverage, per-word cost model, distance function and the
      normalisation input path, each with a number.
- [x] Name the refutations and the remaining headroom.
- [x] No production code changed on `madgab-costaudit-c4d7e8`.

## Handoff / notes

- Measurement branch `scratch/c4d7e8-audit` (commit 360ed5d) holds the four probe tests
  (`zz_probe_case2`, `zz_scan5`, `zz_canon2`, `zz_ceiling`, `zz_distance`,
  `zz_apostrophe`). It is scratch-only and must never be merged; this branch carries no
  instrumentation.
- `cargo fmt` and `cargo clippy` are absent on this host and were not attempted.
- The binding constraint for case 2 is **`axes::NOVELTY` (0.15 weight) plus
  `axes::FAMILIARITY` (0.10 weight)**, together worth 0.083 of the 0.0979 gap. The
  canonical answer keeps the target's word boundary at offset 15 and, being five rare-ish
  content words, loses familiarity on `dupe`.
- **Recommended single next lever, if any:** stop opening cost/pronunciation/distance
  fronts for this milestone — the upper bound above closes them. The only lever with
  enough headroom is the *objective's boundary treatment*: the current
  `boundary_novelty` (`src/lib.rs:2564`) is a pure Jaccard distance that scores
  `Jaccard(preserved) - 1`, so a clue that keeps one target boundary in five is docked
  0.05. Whether the milestone's intended property (the answer is *sayable as a resegmentation
  of the target*) is better served by rewarding boundary *addition* without punishing
  boundary *preservation* is a scoring-policy question, not a search question, and it
  should be settled as such before another enumeration front is opened.
- Smallest-first implementation step, **only if** that policy question is answered
  yes: change `boundary_novelty` so shared boundaries cannot reduce the score below
  `1 - added/total` (one-line, `src/lib.rs:2564-2596`), add a unit test asserting a
  clue that preserves one target boundary scores at least 0.10 on the novelty term, and
  check the executable on both canonical cases. Predicted effect: canonical case 2
  0.7999 -> 0.8499 (`novelty` 0.6667 -> 1.0), still short of rank 50 — which is
  itself the honest answer to the milestone under the current `FAMILIARITY` axis, and
  means the policy change alone does not close it.
