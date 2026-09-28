# REPORT 5e9c41 — calibration signal beyond total cost (w-5e9c41)

Branch `madgab-scorecal-5e9c41`, base `6366d3c`.
`CARGO_TARGET_DIR=/workspace/target-5e9c51` throughout. `src/adjacency.rs` and
`src/main.rs` untouched.

**Verdict: PARTIAL, and the axis is landed.** The lead hypothesis was correct and a
general term exists: `axes::WORST_WORD` reads the **maximum** per-word edit cost, which
the total-cost objective is structurally blind to. It moves the canonical from **pool
rank 27 to pool rank 9**. It does **not** put the canonical in the shipped `--top 10`
display, because it concentrates the pool head into the canonical's own structure and
the diversity layer's per-structure `share_cap` then excludes it. Both coordinates are
stated separately below. The milestone is **not** met by this front.

---

## Criterion 1 — what the objective cannot see, measured

**The `SIMILARITY` axis and every other axis in the objective are functions of the
*total* edit cost.** `src/lib.rs:3011` reads `sub_cost_total / total_len`; the
remainder divide by word count. The per-slot cost vector `sub_cost_i` is summed the
moment it reaches the accumulator and nothing downstream of the sum records *which
word paid it*.

Measured at the library boundary, per-slot costs of the visible head of
`recognize speech` at base:

| rank | score | per-slot costs | worst |
| --- | --- | --- | --- |
| 1 | 0.923641 | 0.136, 0.29, 0.20, 0.06 | 0.290 |
| 3 | 0.922971 | 0.214, 0.00, **0.40**, 0.06 | 0.400 |
| 8 | 0.921896 | 0.00, 0.00, **0.40**, 0.06 | 0.400 |
| 10 | 0.921813 | 0.214, 0.108, 0.20, 0.06 | 0.214 |
| **27** | **0.919950** | **0.00, 0.20, 0.20, 0.15** | **0.200** |

The canonical is the **best-behaved clue in the top ten by the property the objective
does not compute**: no word in it costs more than 0.20, while eight of the nine clues
above it contain a word costing 0.29 or 0.40. It loses by 0.0019 on the total and
wins outright on the maximum.

**This is the blind spot, stated exactly:** the total is a sufficient statistic for
every current axis, so no reweighting of the existing axes can express a preference
for allocation shape — a reweighting of a mean is still a function of the mean. This
is why the item is not a repeat of w-e086cc, and why the term below is an axis and
not a weight.

## Criterion 2 — the term, and the honest partial

### The term

```text
axes::WORST_WORD: f64 = 0.05
axes::WORST_WORD_COST: f64 = 0.5      // = SearchMode::approximate()'s per_word_budget

worst_word = clamp(1 - max_i(sub_cost_i) / WORST_WORD_COST, 0, 1)
term       = WORST_WORD * (worst_word - 1)        // non-positive, zero headroom
```

* Reads the **maximum** per-slot cost, not the total. It is the only term in the
  objective that reads the *shape* of the cost distribution.
* Justification that generalises: a Mad Gab clue is solved **one word at a time**. A
  listener who misses one word of a four-word clue has no answer, so the binding
  constraint is the *worst* word and the mean is the wrong summary. Minimising the
  maximum also never rewards a candidate for being excellent on average, only for not
  being bad anywhere.
* Anchored to `SearchMode::approximate()`'s own `per_word_budget` (0.5), so the axis's
  scale is the budget the search was given, not a number fitted to these targets.
* Applied as `w * (v - 1)` exactly like `PUNCH`, so it is non-positive and the
  objective's documented `[0, 1]` maximum is unchanged. Pinned by
  `the_axis_costs_no_headroom_because_it_is_shifted_by_its_own_maximum`.
* Runtime cost **zero**: `Partial` gains one `f64` running maximum, one `max` per
  extension. No `src/adjacency.rs` change, no re-run of the span-DP keys. (Those keys
  omit `PUNCH` too, for the same reason — they are bounds, and the axis's non-positivity
  is in the safe direction for them.)

**No phrase literal in `src/`.** `no_phrase_hard_coding` is **9 passed / 0 failed** with
`src/` at **zero** allowlist entries. `grep` for the canonical strings over `src/`
returns nothing.

### It is the only shape of eight that works

Eight allocation-shape statistics were measured by re-ranking the **captured pool**
offline, so the comparison is exact and costs no re-enumeration. Only the maximum
lifts the canonical; the other seven are neutral or actively harmful:

| shape | probe weight 0.05 | 0.10 | 0.20 | 0.40 | direction |
| --- | --- | --- | --- | --- | --- |
| **min max slot cost** | **10** | **9** | **5** | **5** | **lifts** |
| 2nd-highest slot cost | 51 | 67 | 84 | 106 | demotes |
| share of slots at/above 0.20 | 119 | 174 | 227 | 314 | demotes |
| share of exact slots | 82 | 191 | 244 | 313 | demotes |
| −stdev of slot cost | 28 | 46 | 78 | 109 | demotes |
| −Σcost² | 20 | 27 | 39 | 49 | demotes |
| −mean slot cost | 28 | 31 | 41 | 50 | demotes (redundant with `SIMILARITY`) |
| −(0.5·max + 0.5·mean) | 17 | 18 | 20 | 32 | barely moves |

(Ranks are the canonical's pool rank; base is 27.) A per-word *per-phone* minimax was
also tried and is a strong negative — it demotes the canonical to rank 89 at weight
0.05 — because a two-phone slot is charged the same rate as a five-phone one.

### The weight sweep, and the invariant that sets the ceiling

`WORST_WORD` swept on the real build, with the two quantities that matter:

| `WORST_WORD` | canonical pool rank | in `--top 10`? | display@25 / @50 | `a sturdy green cardigan` `@top 50` word-count classes | coverage invariant |
| --- | --- | --- | --- | --- | --- |
| **0.00 (base)** | 27 | no | absent / 27 | `[6, 7]` | **green** |
| 0.02 | 11 | no | absent / 11 | `[6, 7]` | green |
| **0.05 (shipped)** | **9** | **no** | **9 / 9** | **`[6, 7]`** | **green** |
| 0.10 | 7 | no | 7 / 7 | `[7]` | **RED** |

**`0.10` turns `a_short_multi_syllable_proposal_set_is_not_one_word_count_class` red**:
the printed 50-slot set for `a sturdy green cardigan` becomes the single word-count
class `[7]`, because `share_cap` admits a fixed number of slots per structure and one
structure owns the head. That is a real user-visible monoculture and it is the reason
the axis ships at `0.05`, which is the largest weight that keeps the invariant green.
This is disclosed, not tuned around: the trade is a diversity cost against a
readability gain, quantified below.

### The partial, stated plainly

| coordinate | at base | with the axis |
| --- | --- | --- |
| canonical **pool rank** (`recognize speech`) | 27 of 13 801 | **9 of 13 819** |
| canonical **score** | 0.919950 | 0.899950 |
| gap to the 10th displayed | −0.0019 (below cutoff) | — |
| **canonical in shipped `--top 10` display** | **no** | **no** |
| canonical display position at `--top 25` / `--top 50` | absent / 27 | **9 / 9** |

**The shipped default `--top 10` still does not show `wreck a nice beach`.** The axis
moved the ordering key's input by 18 pool positions and the milestone is still not met.

**The new stage is the share cap, not the cutoff.** The axis promotes the canonical's
*own* structure `[3,5,10]` into **20 of the top 20** pool slots (base: 8 of 20).
`select_diverse`'s per-structure `share_cap(10, 291)` then holds that structure to a
small number of slots, and the canonical is its 9th member, so it never gets one. It is
excluded by a **diversity decision about a structure**, not by its own score.

This is recorded as a measured, named interaction. Per the pass coordinator it is **not**
a new front: w-c31a07 and `REPORT-a4d10c.md` priced the ordering and selection surfaces,
and the five surfaces there were not re-run.

### What the axis cost, in aggregate

24-target spread, `examples/measure.rs`, single run each:

| metric | base | `WORST_WORD = 0.05` | direction |
| --- | --- | --- | --- |
| all-content proposal share | 0.1042 | 0.1854 | **+78%** |
| mean content-word share | 0.8435 | 0.8453 | + |
| mean acoustic similarity | 0.7587 | **0.8168** | **+0.058** |
| mean visible score | 0.9114 | 0.8660 | − (the axis is a penalty; the axes below it rise) |
| mean distinct structures per list | 6.71 | 5.79 | **−0.92** |
| mean dominant-structure share | 0.3292 | 0.4000 | **+0.07** |
| search seconds (24 targets) | 19.84 | 21.62 | noise — no algorithmic change |

Both directions are real. The axis buys acoustic readability and content-word quality
and pays in list diversity, because it systematically prefers one resegmentation whose
every word is individually good.

### Case 2 is untouched

`hits justice dupe hid came` is **not enumerated at all** — it is absent from the
14 549-member pool, not merely unranked. That blocker is upstream of scoring and was
already priced in w-1c3e77 and w-04f83f.
`approximate_finds_classic_madgab_resegmentation` is **red at base and red now**, for
that reason. It was **not** relaxed, re-pinned, skipped or ignored.

## Criterion 3 — regression tests

`tests/worst_word_axis.rs`, 7 tests, all phrase literals in `tests/`:

* `the_axis_moves_the_canonical_out_of_the_near_tie_band` — pool rank 8 (display 9).
* `the_canonical_has_no_bad_word_where_its_outrankers_do` — the separation as a fact
  about the pool, not about one clue: the canonical's worst word (0.20) is cheaper than
  every outranker's.
* `total_cost_cannot_see_the_allocation_and_the_maximum_can` — **the property, stated
  as a construction**: `[0.1, 0.1, 0.1, 0.1]` and `[0.0, 0.0, 0.0, 0.4]` have identical
  totals and are distinguished only by the maximum.
* `the_axis_is_anchored_to_the_per_word_budget_and_is_non_positive` — **the generality
  guard**: non-positive over the whole cost range, exactly zero at an exact match,
  exactly the full weight at the per-word budget, monotone non-increasing. Nothing in
  the formula reads a target, a clue or a word count.
* `the_axis_costs_no_headroom_because_it_is_shifted_by_its_own_maximum` — every pool
  score stays in `[0, 1]`.
* `the_canonical_is_in_the_pool_but_still_outside_the_default_display` — **the partial,
  pinned as a partial**, with the mechanism (20 of the top 20 in `[3,5,10]`).
* `case_two_canonical_is_absent_from_the_pool_which_is_an_enumeration_fact` — keeps
  case 2 from being read as a result of this front.

`tests/display_ordering_attribution.rs` and `tests/corpus_integration.rs` were
**re-based, not weakened**, each re-pin carrying its base value inline:

| pin | base | now |
| --- | --- | --- |
| canonical pool rank | 26 | 8 |
| canonical score | 0.919950 | 0.899950 |
| pool size, `recognize speech` | 13 801 | 13 819 |
| pool size, case 2 | 14 555 | 14 549 |
| `[3,5,10]` members | 156 | 160 |
| same-structure siblings ahead | 12 | 8 |
| per-word-IPA groups / canonical's group rank | 10 133 / 25 | 10 121 / 9 |
| visible head is one ending repeated | 9 of 10 | **10 of 10** |
| `approximate_output_is_locked`, `I love you` | 10 phrases | 10 phrases, 4 replaced |

Two pins are **substantively changed and labelled as such**:

* `the_selection_layer_is_a_pure_pass_through_at_the_shipped_default` — **this property
  no longer holds.** At base every displayed clue sat at exactly its own pool rank at
  `top_n` 10, 11 and 15. It now does not, because the cap binds. The test asserts the
  new fact and its comment records the base result, so the next front sees the stage
  change rather than inheriting a stale claim.
* `the_visible_head_is_one_ending_repeated_rather_than_distinct_wordings` — 9 of 10 at
  base, **10 of 10** now. The user-visible redundancy is *worse* after the axis. Stated
  in the test, not papered over.

The five priced negatives in `REPORT-a4d10c.md` were **not re-run**. Their assertions
are untouched; where one had an incidental score comparison attached
(`content_word_share_does_not_separate_...`), the *priced negative itself* — content
share is anti-correlated here — still holds and is still asserted. Only the incidental
score ordering moved, and the comment says so and names the axis as what replaced it.

## Criterion 4 — suite

```
cargo test --release --lib                -> 83 passed; 0 failed; 12 ignored
cargo test --release --test no_phrase_hard_coding        ->  9 passed; 0 failed  (9/0, src/ allowlist 0)
cargo test --release --test worst_word_axis              ->  7 passed; 0 failed
cargo test --release --test display_ordering_attribution ->  9 passed; 0 failed
cargo test --release --test emit_coverage                ->  7 passed; 0 failed
cargo test --release --test exact_determinism            ->  1 passed; 0 failed
cargo test --release --test objective_is_a_search_input  ->  1 passed; 0 failed
cargo test --release --test pool_rank_reporting          ->  5 passed; 0 failed
cargo test --release --test approx_determinism           ->  4 passed; 0 failed
cargo test --release --test cli_milestone_predicate      ->  3 passed; 0 failed; 1 ignored
cargo test --release --test corpus_integration           -> 12 passed; 1 failed
```

The single failure is `approximate_finds_classic_madgab_resegmentation`, the known
pre-existing case-2 red. **Test counts are at or above base and no new red was
introduced.** `emit_coverage` is green at the shipped weight; it is red at
`WORST_WORD = 0.10`, which is the priced reason for the weight.

**`cargo fmt` and `cargo clippy` did not run.** Neither subcommand is installed on this
host (no rustup): `error: no such command: fmt`, `error: no such command: clippy`. I am
not claiming them.

## Criterion 5 — output shape

`src/main.rs` is untouched. The printed output shape is unchanged; only the scores and
the phrases inside the list moved. Verbatim at the shipped default:

```
target: recognize speech
(pool: 13819 scored candidates, 10 displayed; expansion 1381.9x)

 1. [0.901] wreck a nice pitch
 2. [0.901] wreck a nice peach
 3. [0.901] let a nice pitch
 4. [0.901] let a nice peach
 5. [0.898] wreck eggs i.'s pitch
 6. [0.898] wreck eggs i.'s peach
 7. [0.898] let eggs i.'s pitch
 8. [0.898] let eggs i.'s peach
 9. [0.896] yeah keg nice pitch
10. [0.896] yeah keg nice peach
```

Case 2, verbatim: `it said thus test oop dame` … `it justice too pah dame`. Neither is
made worse; the canonical is still absent, as at base.

## Recommendation

Land the axis at `0.05` — it is the load-bearing finding of this front, it is a real
new measurement rather than a reweighting, and it is the only route anyone has found
that moves the canonical at all. But state the result as what it is:

* **achieved**: canonical pool rank 27 → 9; a general, zero-cost axis; +78% all-content
  proposals and +0.058 acoustic similarity over a 24-target spread;
* **not achieved**: the canonical is still not in the shipped `--top 10`, and case 2 is
  untouched;
* **priced**: list diversity falls 6.71 → 5.79 distinct structures per list, the visible
  head becomes 10 of 10 one ending, and `WORST_WORD = 0.10` is barred by
  `a_short_multi_syllable_proposal_set_is_not_one_word_count_class`.

The next front, if the coordinator opens one, is the interaction named in Criterion 2:
a scoring axis that improves every clue's local quality **concentrates the pool head in
one resegmentation**, and `share_cap` is then decided by a structure, not by a clue. That
is a selection-policy question and it is explicitly out of this front's scope.
