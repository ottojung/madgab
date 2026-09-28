---
work_item: w-3f8c62  # back-reference to the canonical item in docs/work/items/, not a second item (coord-9d1f)
id: w-3f8c62-report  # report id only; the task id is the back-reference above
state: done
priority: high
owner: front-3f8c62
updated: 2026-09-27T23:40:00Z
branch: madgab-parsim-3f8c62
worktree: /workspace/madgab-parsim-3f8c62
base: 8bfe7de
verdict: HOLD
scratch_branch: scratch-3f8c62-landed (local, deliberately unpushed)
---

# w-3f8c62 - the word-count axis lands and turns the head fence green, and lands with eight new reds

Front `front-3f8c62`, branch `madgab-parsim-3f8c62`, base `8bfe7de` (integrated HEAD).
Release mode throughout, `CARGO_TARGET_DIR=target`.
**Outcome: HOLD** - the full decomposition and the full price, and the axis
*implemented and measured end to end* so the numbers below are measurements of
production code rather than of a description of it. Nothing is proposed for
integration; see §5 for the eight fences that is not possible.

The landed variant is preserved verbatim on the local, **unpushed** branch
`scratch-3f8c62-landed` at `514ed91`, so a later pass can inspect or re-measure
it without this report being the only evidence. It is scratch by decision of
coordinator `coord-5d31` and must not be integrated as it stands.

Reproduce (all on the scratch branch, release, one suite at a time):

    cargo test --release --lib head_not_worse_than_pool -- --nocapture
    cargo test --release --lib price_the_budget -- --ignored --nocapture
    cargo test --release --lib price_the_property_over_a_spread -- --ignored --nocapture
    cargo test --release --lib a_target_whose_best_wording
    cargo run --release --example measure

`cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run on
this host** (see [../environment-notes.md](../environment-notes.md)) and are
**not claimed**.

---

## 0. The one-sentence finding

**No weight in this objective is search-neutral.** Every axis weight is read by
the search's own keys as well as by the final scorer, so the repair this item
asked for — put `PARSIMONY` at 0.15 in the structural keys as well as the
scorer, funded from `SHAPE`/`NOVELTY`/`RHYTHM` — is not a scoring change. It
fixes the red head fence for the right reason and it moves **eight** fences,
including the green canonical case. `REPORT-9b4a15.md` §3 recorded this as a
caveat it could only assert ("no candidate here is literally search-neutral");
§4 below measures it, decomposes it, and locates the exact two lines that
carry it.

## 1. Baseline, re-derived on integrated HEAD `8bfe7de`

Release mode, the deduplicated score-ordered pool from
`Generator::generate_pool` at the default configuration.

| | item's figure | measured | verdict |
|---|---|---|---|
| `It's just a stupid game` pool | 18,949 | **18,949** | confirmed |
| its head-50 `SIMILARITY` lift | **−0.0180** | **−0.0180** | confirmed |
| `recognize speech` pool | 18,289 | **18,289** | confirmed |
| its head-50 `SIMILARITY` lift | **+0.0609** | **+0.0609** | confirmed |
| `wreck a nice beach` pool rank | **27** | **27**, score `0.9199502875` | confirmed |
| case-2 head-200 word-count histogram | — | `{5: 27, 6: 173}` | matches REPORT-9b4a15 §1 |
| case-1 head-200 word-count histogram | — | `{4: 200}` | matches, the immunity argument |

**No disagreement with any figure the item quoted.** The canonical case-2 tuple
is *absent* from the pool, as in `REPORT-9b4a15.md` §1 and
`REPORT-3a8c05.md` §0; nothing here reaches it and nothing claims to
(`REPORT-3a8c05.md` §3's weight-free floor of rank 1,127 is unaffected by any
of this).

### 1.1 The fence, red before

    cargo test --release --lib head_not_worse_than_pool -- --ignored --nocapture

    walk the dog                   lift +0.1088
    open the window                lift +0.0929
    turn off the lights            lift +0.0415
    they ate the whole pie         lift -0.0080
    call the office tomorrow       lift +0.0519
    put the milk away              lift +0.0687
    the train leaves at noon       lift +0.0192
    my brother lost his wallet     lift -0.0184
    he runs to the station         lift +0.0421
    we should leave earlier        lift +0.1177
    => FAILED, 2 of 10 below the pool mean

**RED is the recorded colour on HEAD**, `#[ignore]`d, exactly as integrated at
`515f8bd`. Both failing numbers reproduce to the digit.

## 2. The axis, as landed (scratch branch, for the record)

`axes::PARSIMONY = 0.15`, `SIMILARITY 0.25` unchanged, `NOVELTY 0.15 → 0.05`,
`RHYTHM 0.30 → 0.25`, `SHAPE 0.05 → 0.0`, `WORD_NOVELTY`, `FAMILIARITY`,
`PUNCH`, `CLOSED_CLASS` unchanged. The positive weights sum to **0.95**, so the
objective's maximum is 0.95, inside the documented `Clue::score` range of
[0.0, 1.0] — the term is *funded*, not added. `SHAPE` is set to zero weight
rather than deleted from the code, so the objective's shape is unchanged and
the funding is checkable arithmetic.

The axis is `word_count_parsimony(clue_words, target_words) = 1 - |w_clue -
w_target| / max(w_clue, w_target)`: **two integer word counts and nothing
else**, no sentence, no clue, no word list, no target text, no per-input
branch, no literal token. `no_phrase_hard_coding` is **9/9** with the axis in
the tree.

It is in four places: `Partial::metrics` (the final scorer),
`partial_span_score`, `complete_span_score` and `span_score_bound`.

### 2.1 Why the bound is sound, and why that is not the same as "loose"

`span_score_bound` is the only key allowed to discard a path outright, and it
is compared against an `incumbent` that the same objective produces. The
parsimony axis is a function of the final word count alone, and the word
counts a path can still finish at are exactly `[words, words + still_possible]`
— at least what it has taken, and at most one per remaining IPA character.
Write `f(c) = word_count_parsimony(c, t)`: for `c <= t` it is `c/t`, rising in
`c`; for `c >= t` it is `t/c`, falling in `c`. So `f` is unimodal with a single
maximum at `c = t`, and the maximum over any interval is `f` at `t` clamped
into that interval. That is `word_count_parsimony_upper_bound`, and it is the
*exact* maximum over the reachable set — the axis's contribution to the
admissible key is exact wherever the target's own word count is still
reachable and is the correct endpoint relaxation where it is not.

`structural_bounds_dominate_the_real_scorer` is green with that in place, and
it is green **for that reason**, which is asserted rather than asserted-to by
`front_9b4a15::the_word_count_bound_is_tight_and_not_padded`: over every
`(words, target_words, still_possible)` in `0..=9 × 1..=8 × 0..=9` it compares
the bound against a brute-force `max` over the same interval (equal, to
1e-12) and asserts the bound is **strictly below 1.0** wherever the target's
word count is not reachable. A bound that had been made true by padding — say,
pinned at 1.0, which is what a lazy repair would do — would leave the fence
green and that test red.

One honest note on `partial_span_score`: the axis there is the *exact* value at
the path's current word count, not the upper bound, and that is safe because
the structural DP's representative buckets are keyed by
`(word count, shared boundaries)`, so every comparison the key takes part in is
between paths with the same word count and the term is a shared constant. It is
documented at the function.

### 2.2 The head fence, green after

    cargo test --release --lib head_not_worse_than_pool -- --nocapture

    walk the dog                   +0.1082
    open the window                +0.1043
    turn off the lights            +0.0439
    they ate the whole pie         +0.0511     (was -0.0080)
    call the office tomorrow       +0.0809
    put the milk away              +0.0907
    the train leaves at noon       +0.0440
    my brother lost his wallet     +0.0110     (was -0.0184)
    he runs to the station         +0.0379
    we should leave earlier        +0.1109
    => ok, 10 of 10

Un-`#[ignore]`d. The property is unchanged and still names no sentence, no clue
and no rank.

**This is a real, general result and it is the front's positive finding**: the
item's success criterion, expressed at the library boundary, is met, and it is
met by the axis rather than by a weight chosen to make two examples move.

### 2.3 The prices, on the post-change captures

`price_the_budget` and `price_the_property_over_a_spread`, re-run against the
pools the landed objective actually produces. `C0` in the harness is now written
out as the pre-`w-3f8c62` literals rather than read from `axes` — it used to be
`SHIPPED = axes::*`, which stops being a comparator the moment production
moves, and would have made every row relative to itself.

Canonical pair, landed:

| | case 2 | case 1 |
|---|---|---|
| pool | 18,949 → **17,449** (−7.9%) | 18,289 → **16,999** (−7.1%) |
| head-200 word-count histogram | `{5: 27, 6: 173}` → **`{5: 200}`** | `{4: 200}` → **`{4: 200}`** |
| head-50 `SIMILARITY` lift | −0.0180 → **+0.0329** | +0.0609 → **+0.0446** |
| green case pool rank | — | 27 → **48** |
| canonical tuple | still absent | — |

The case-1 immunity argument from `REPORT-3a8c05.md` §5 note 2 still holds
exactly: the top 200 is 100% four-word clues, so a four-word answer cannot be
hurt by the axis. It is hurt anyway — by rank, see §5.

Spread of ten ordinary targets, on the landed pools:

| candidate | ceiling | spread | green case rank | case-2 head lift |
|---|---|---|---|---|
| C0 as shipped before this item | 1.00 | 8/10 | 27 | −0.0169 |
| C1 `PARS 0.10`, `SHAPE→0`, `RHY 0.30→0.25` | 1.00 | 8/10 | 48 | +0.0237 |
| C1b `PARS 0.15`, `SHAPE→0`, `NOVELTY→0` | 0.95 | **10/10** | **185** | +0.0653 |
| **C1d (production)** | 0.95 | **10/10** | **48** | **+0.0329** |
| C1j / C1k (10/10 row, `SIM 0.25→0.30`) | 1.00 | **10/10** | **40** | +0.0442 |
| C2b / C2d (boundary fidelity instead) | 0.95 | 7/10 / 9/10 | 2,620 / 3,263 | +0.0350 / +0.0335 |

Three readings, and the second is the one the item asked for:

* **C1d against C1 and C1b, on the same captures.** C1 is 8/10 — it repairs
  neither failing lift once the pools have moved. C1b is 10/10 but pushes the
  green case to 185, out of the list. C1d is 10/10 *and* keeps the green case
  at 48, inside the top 50: it is the only vector in the table that holds both
  properties without touching `SIMILARITY`. `REPORT-9b4a15.md` §3's ranking of
  these three is reproduced and sharpened.
* **The 9/10 the item predicted, and why production is 10/10.** On the
  *pre-change* pool, re-ranking under C1d is **9/10** and loses exactly one
  target: `my brother lost his wallet`, at **−0.0016** (reproduced from my own
  run of the harness on HEAD, §1). On the *post-change* pool the same vector is
  10/10 and that target is **+0.0110**. The difference is not noise and not a
  better weight: the axis in the structural keys changes *which candidates
  exist*, not only how they are ordered, and 9/10 is a statement about
  re-ranking a fixed pool while the fence in §2.2 is a statement about
  production. Both numbers are kept; the fence is the criterion and it is 10/10.
* **The alternative use of the budget got worse, not better.** `C2b` falls from
  8/10 to **7/10** on the landed pools, and `C2d` from 10/10 to 9/10, while
  still costing the green case 2,620 and 3,263 ranks. `REPORT-3a8c05.md` §4's
  refusal of the boundary-fidelity family is re-confirmed on fresh pools.

### 2.4 The 10/10 row, priced

`C1j` and `C1k` remain the only vectors in the family that hold both
properties at 10/10 with the green case inside the top 50 — now at rank **40**
rather than 38, and with a larger case-2 head lift (+0.0442 against C1d's
+0.0329). **The row is in scope to report and out of scope to buy**, for a
reason this front can now state exactly rather than cite: they are bought by
raising `SIMILARITY` 0.25 → 0.30, and §4 below measures that *any* change to
any axis weight reaches the search's own keys. The 10/10 row therefore needs
the reserve-depth fence and the three pool-reach fences **re-derived** as a
search-behaviour claim, which is a front of its own on a surface this item
forbids. Scoped out, with the number: **+0.0113 of case-2 head lift and 8 pool
ranks of the green case, for a re-derived fence.**

### 2.5 Cost, and the 0.0–1.0 range

`examples/measure`, 24 targets, release, three runs each, whole process:

| | HEAD `8bfe7de` | landed C1d |
|---|---|---|
| search seconds | 25.39 / 23.47 / 23.12 | 27.45 / 24.64 / 24.71 |
| mean | 23.99 | 25.60 (**+6.7%**) |
| mean visible score | 0.9114 | 0.8568 |
| mean acoustic similarity (the harness's own public proxy) | 0.7587 | 0.8078 |
| all-content proposal share | 0.1042 | 0.2917 |
| mean distinct structures | 6.71 | 6.33 |

The wall-clock delta is **inside the run-to-run spread of either
configuration** (23.1–25.4 s before, 24.6–27.5 s after) and is not a
regression beyond noise; it is also not free, and the direction is explicable:
a term added to an *upper* bound that only the ideal path attains makes the
bound higher, so the search discards strictly less. Pool sizes fall 7–8%, which
is the same fact seen from the other side.

The visible score falls from 0.9114 to 0.8568 because the objective's maximum
fell from 1.00 to 0.95, which is the funding and not a regression: the highest
printed score in `measure` drops from 0.933655 to 0.889421, and the range
[0.0, 1.0] holds with room to spare. There is no separately named range test in
the suite; the guarantee here is the funding arithmetic, asserted by
`front_9b4a15::the_production_weights_are_the_priced_c1d` (positive weights
`+ PARSIMONY == 0.95`), plus the observed printed values.

Two side effects are *improvements* and are reported as such: the
all-content proposal share nearly triples (consistent with `REPORT-3a8c05.md`
§1.1 — the 0.10 taken off `NOVELTY` is the axis it measured as
anti-correlated with quality), and the harness's public acoustic proxy rises
+0.0491.

## 3. The green case: in the pool, out of the list

`wreck a nice beach` for `recognize speech`:

| | before | after |
|---|---|---|
| `generate_pool` rank | **27**, score `0.9199502875` | **48**, score `0.7949502875` |
| in the returned proposal set (`generate`) | yes | **no** |

C1d predicted rank 46; the measured rank on the changed pool is **48**. Inside
the top 50, by two places, at the default configuration *and* at the corpus
suite's `beam_width: 64` (measured both; the pool is 16,999 either way).

But `Generator::generate` does not return the pool's top `top_n` — it narrows
it through `select_diverse` — and at rank 48 the green case is no longer
selected. Two tests say so: `approximate_finds_recognize_speech_resegmentation`
and `emit_coverage::the_other_canonical_resegmentation_is_still_proposed`, both
green on HEAD.

Per the item's own instruction, a loss of the green case is a **reason to
report HOLD, not a weight to raise**, and no weight was raised on account of it.
The cause is legible and is not the axis's judgement of the clue: the case-1
pool's top 200 is 100% four-word clues before and after, so the axis is
indifferent to it; what moved is that **18,289 → 16,999 candidates** and the
display's diversity filter, i.e. the axis in the *keys*, not the axis in the
scorer. A scorer-only variant — §4, row 2 — leaves the green case alone and
still cannot be shipped, because it breaks the bound.

## 4. The isolation: which line carries the search change

This is the front's durable contribution. `REPORT-9b4a15.md` §4 found that
adding the axis to the scorer alone makes
`structural_bounds_dominate_the_real_scorer` false, and §3 asserted as a caveat
that no candidate is literally search-neutral. Both are now measured, and the
caveat is decomposed.

Every row below is one measurement of
`tests::a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` —
`w-1c7d40`'s reserve-depth fence, **green on integrated HEAD** — with the rest
of the tree held at HEAD. The axis weight is 0.15 throughout the first five
rows; the weights are at their HEAD values throughout the last three.

| # | variant | reserve-depth fence |
|---|---|---|
| 0 | HEAD as shipped | **green** |
| 1 | `PARSIMONY` in the **final scorer only**, weights unchanged | **green** |
| 2 | `PARSIMONY` in `span_score_bound` + scorer only, weights unchanged | **RED** |
| 3 | `PARSIMONY` in `complete_span_score` + `span_score_bound` + scorer | **RED** |
| 4 | `PARSIMONY` in `partial_span_score` too (all three keys) | **RED** |
| 5 | the complete C1d vector (axis in all four places, weights funded) | **RED** |
| 6 | weights only: `NOVELTY 0.15 → 0.05`, no axis anywhere | **green** |
| 7 | weights only: `RHYTHM 0.30 → 0.25`, no axis anywhere | **RED** |
| 8 | weights only: `SHAPE 0.05 → 0.00`, no axis anywhere | **RED** |

Read down the table:

* **Rows 1 and 2 are the pair that closes the item.** A term in the final
  scorer alone is a *scoring* change and the search does not move — and it
  cannot be shipped, because `span_score_bound` then understates the score it
  bounds and `structural_bounds_dominate_the_real_scorer` goes red. Adding it
  to the admissible bound is what makes the bound sound, and **the admissible
  bound is itself the search's pruning key** (the only
  `span_bound` call, and the `NEG_INFINITY` it returns when no completion
  survives is a discard). So the repair the item asks for *is* a search change,
  and the two obstructions `REPORT-9b4a15.md` §4 recorded are not two
  obstructions: they are the same one, seen from the two ends of one key.
* **Rows 6–8 say the weights are not a way out.** `RHYTHM` and `SHAPE` each
  redden the fence on their own. They are read by the walk's own heap key,
  `bound(prefix)` (`src/lib.rs:1920-1939`), which is the best-first ordering
  for the retained-wordings walk. `NOVELTY` alone is green, which is the only
  reason the *funding* mix looked like a lever; funding from `RHYTHM` and
  `SHAPE` is not.
* **Row 3 vs row 4.** `partial_span_score` contributes nothing: as §2.1 says,
  the DP's buckets are keyed by word count, so the term is a shared constant
  there. The two keys that matter are `complete_span_score`, which orders the
  finished structures the lexical enumeration expands, and
  `span_score_bound`, which discards.

### 4.1 The fenced property

The claim is now enforced by tests that are green on integrated HEAD, which is
the cheapest possible form of "fenced property" — it costs no new code:

> **No weight in the objective is search-neutral.** Every axis weight is read
> by the search's own keys — `SlotAlt::contribution` (`src/lib.rs:1124`),
> the walk's `bound(prefix)` (`src/lib.rs:1920`), `partial_span_score`,
> `complete_span_score` and `span_score_bound` — as well as by the final
> scorer, so changing a weight changes which candidates the search keeps, and
> any change to the objective must be measured against *all* of:
>
> * `tests::a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple`
>   (reserve depth, `w-1c7d40`),
> * `tests/approximate_pool_reaches_matches_deep_in_a_span`,
> * `tests/approximate_pool_reaches_resegmentations_deeper_than_one_walk`,
> * `tests/approximate_pool_reaches_alternatives_past_the_opening_slot_width`,
> * `tests::a_lattice_alignment_can_be_absent_from_the_production_pool`
>   (named in `OBSTRUCTION-MAP.md` §2's note on the parked fill branch),
> * and `tests::approximate_output_is_locked`, the golden-output lock that
>   exists precisely to make an objective change a conscious act.

All five are green on `8bfe7de`. All five are red under the axis
(`approximate_output_is_locked` red as a *deliberate* consequence: the printed
scores and the top-10 for one target are locked, and this front does not
unlock them). `REPORT-9b4a15.md` §3's footnote should be promoted from a caveat
to this statement.

## 5. Verdict, and what would unblock it

**HOLD.** The axis is right, it is the cheapest complete version of itself, it
is in the final scorer and in all three structural keys, the bound is exact
rather than padded, the red head fence turns green for the right reason, and
the canonical case-2 tuple remains out of reach exactly as
`REPORT-3a8c05.md` §3's weight-free bound says it must. It is also, measured,
not integrable: **eight** fences are red that were green on HEAD.

| fence | HEAD | landed C1d |
|---|---|---|
| `head_not_worse_than_pool` (the item's criterion) | red, `#[ignore]`d | **green, 10/10** |
| `structural_bounds_dominate_the_real_scorer` | green | green, and tight |
| `a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` | green | **RED** |
| `approximate_pool_reaches_matches_deep_in_a_span` | green | **RED** |
| `approximate_pool_reaches_resegmentations_deeper_than_one_walk` | green | **RED** |
| `approximate_pool_reaches_alternatives_past_the_opening_slot_width` | green | **RED** |
| `a_lattice_alignment_can_be_absent_from_the_production_pool` | green | **RED** (`hid` leaves the pool entirely) |
| `approximate_output_is_locked` | green | **RED** (deliberate) |
| `approximate_finds_recognize_speech_resegmentation` | green | **RED** (green case) |
| `the_other_canonical_resegmentation_is_still_proposed` | green | **RED** (green case) |
| `approximate_finds_classic_madgab_resegmentation` | **red at base** | red at base, not re-pinned |
| `cargo test --release --lib` | 75 / 0 / 12 | 78 / **1** / 11 |
| `no_phrase_hard_coding` | 9 / 9 | **9 / 9** |
| `emit_coverage` | 4 / 4 | **2 / 2 failed** |
| `approx_determinism` | 4 / 4 | **4 / 4** |
| `exact_determinism` | 1 / 1 | **1 / 1** |

`corpus_integration` is 7 passed / 6 failed against the expected 12 / 1, run
with `-- --test-threads=2`; five of the six are new and the sixth is the known
base red.

**The exact blocking condition.** The item forbids changing the search, and
the repair cannot be confined to the objective: `span_score_bound` is the
objective's *own* admissible bound and is simultaneously the search's only
pruning key, so any term that makes the bound sound changes which span paths
the search discards. Equivalently, and more usefully for whoever takes this
next: **an objective term is not free unless it is in the scorer alone, and a
scorer-only term is not sound unless the bound has it too.** There is no
placement of this axis that is both admissible and search-neutral, and §4 says
so with two measurements rather than one.

**What would unblock it**, in the order I would do it:

1. **A front that owns the search surface re-derives the reserve-depth and
   pool-reach fences** against the axis in the keys — not by weakening them,
   but by re-deriving the placement guarantee the reserve was funded under,
   because the reserve's depth cap was itself already re-derived once
   (`EMIT_PROFILE_MAX_DEEP` → `funded_slot_depth`, map row 8, and
   `OBSTRUCTION-MAP.md` calls that the only landed coordinate in the table).
   That front should start from `scratch-3f8c62-landed`, which is the variant
   with the red fences visible.
2. **Or**: split the axis out of the *ordering* keys and keep it in the
   *bound* only, accepting that the DP's structure ordering stays on the old
   weights. That is a smaller search claim and I did not price it, because
   §4 row 2 shows the bound alone is already a discard, so it would not have
   bought neutrality — it would only have bought a smaller delta.
3. **Not** a weight change: rows 6–8 of §4 price the weight lever, and it is
   closed.

### 5.1 The deferred residue, cross-referenced

`REPORT-2f1c03.md`'s P1 — a per-member modulus in `coverage_tuples`, so that
each member of a deep subset is indexed in *its own* slot's span rather than
the narrowest member's — is a **separate, later front** and is not implemented
here. The two surfaces are adjacent and compose badly in one direction only:
P1 changes the map `phase -> index` that the *reserve* walks, and it is priced
at zero cost in candidate visits, emissions and frontier (§3's table), i.e. it
changes the *support* of the reserve's draws and is reach-null. This change
changes the *objective and its keys*, so it changes the *set* of wordings that
survive the bound and reach the pool at all — measured at −7 to −8% of pool
size and five red reach fences, with the emission ceiling untouched
(`the_global_emission_ceiling_is_reached_not_merely_respected` stays green).
P1 therefore cannot unblock this item: adding coverage to a reserve whose
outputs are then pruned differently does not recover candidates the bound has
already discarded. The interaction that matters for the P1 front is the
opposite one, and it is a warning rather than a synergy: **a P1 front that
measures reach on a tree carrying this change would be measuring two
coordinate changes at once, and the reach-null result `REPORT-2f1c03.md` §3
records is only valid on a tree where the objective is unchanged.** If P1 is
taken while this axis is live, the two must be separated, exactly as §4
separates them here.

## 6. Constraints, and how each was honoured

* **No phrase-specific hard-coding.** The axis reads two integers and nothing
  else. No sentence, clue, word list, per-input branch or literal token enters
  `src/` outside a test module; `no_phrase_hard_coding` is **9/9** with the
  axis in the tree. The two canonical phrases and the ten-target spread appear
  only inside `#[cfg(test)]` modules, as data, as they already did.
* **Objective and its bounds only.** No change to search, shortlist, reserve,
  budget, retention, depth, width or emission order. `LEXICAL_HEAP_POP_LIMIT`,
  `SPAN_SHORTLIST`, `SPARSE_SHORTLIST`, `LEXICAL_COMBINATIONS_PER_SEGMENTATION`,
  `EMIT_PROFILE_RESERVE` and the emission ceiling are untouched — which is not
  the same as saying the search is unaffected, and §4 is the measurement of
  that.
* **Score range.** Preserved and now *narrower*: the objective's maximum is
  0.95 because every weight is funded, asserted by
  `front_9b4a15::the_production_weights_are_the_priced_c1d`.
* **Instrumentation.** Everything added is either production (the axis, the
  four call sites, the weights) or behind `#[cfg(test)]`. No `ZZ_*` probe, no
  environment knob, no baseline change, no debug binary. The
  `front_9b4a15` harness now prices the *production* axis rather than a copy of
  it, so a table cannot disagree with the shipped objective by a rounding of
  its own.
* **Release mode** for every number in this report.
* **`cargo fmt`, `cargo fmt --check`, `cargo clippy`, doctests**: cannot run on
  this host (no `rustup`, no `rustfmt`/`clippy`/`rustdoc`); **not claimed**.
* **No self-merge.** `main` and `post-milestone-acceptance` are untouched.
* **One suite at a time.** No suite died with signal 9 on this front, so
  nothing had to be rerun for host memory pressure.

## 7. Handoff

* Terminal: **HOLD**. `docs/work/REPORT-3f8c62.md` on `madgab-parsim-3f8c62`.
* The landed variant is on the local, unpushed `scratch-3f8c62-landed`
  (`514ed91`), so §4's rows can be re-measured without re-deriving the patch.
  It must not be integrated as it stands.
* The item's *question* is answered and its axis is confirmed: the word-count
  parsimony axis is the right repair, it is the cheapest complete version of
  itself, and the general property the item wanted — the returned top-N is not
  a worse acoustic match than the pool it was drawn from — is now **10/10 on
  the production surface** where it was 8/10.
* The item's *landing* is blocked, and the block is not a weight. The next
  front should be on the search surface, starting from
  `scratch-3f8c62-landed`, and its first obligation is §4.1's fenced property:
  every axis weight reaches the search's keys, so the reserve-depth fence and
  the three pool-reach fences must be **re-derived**, not preserved, before any
  objective change can be integrated.
* The case-2 canonical tuple is untouched by all of this and remains
  unreachable by any monotone objective (`REPORT-3a8c05.md` §3, rank ≥ 1,127).
  Do not let a later pass read this front as progress on it.

HOLD
