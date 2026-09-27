---
work_item: true
id: w-e086cc
state: done
priority: high
owner: front-e086cc
updated: 2026-09-27T23:12:00Z
branch: madgab-e086cc
worktree: /workspace/madgab-e086cc
base: post-milestone-acceptance at 106afc5 (pushed)
verdict: HOLD
scratch_branch: scratch/e086cc-probe (local, deliberately unpushed)
---

# w-e086cc - the eight fences are incidental, and the objective is a *search input*, not a ranker

Front `front-e086cc`, branch `madgab-e086cc`, base `106afc5`. Release mode throughout,
`CARGO_TARGET_DIR=target`, one suite at a time. **Outcome: HOLD** - the classification is
complete, two rows of `REPORT-3f8c62.md` §4 are corrected by measurement, the load-bearing
general fact is confirmed and *strengthened*, and the one weight-free property that could be
expressed is expressed and is green on HEAD.

Nothing is proposed for integration, and no vector is landable: §5 gives the arithmetic.
Two artefacts are landed and are safe under any vector: a new weight-free fence
(`tests/objective_is_a_search_input.rs`) and a correction to the tree's own comment at the
search's discard threshold. `cargo test --release` is **75 passed / 0 failed / 12 ignored**
on the lib, unchanged from base.

Reproduce the classification (all on the local, unpushed `scratch/e086cc-probe`):

    W_SIM=.. W_NOV=.. W_WNOV=.. W_FAM=.. W_RHY=.. W_SHA=.. W_PARS=.. W_PUN=.. \
      cargo test --release --lib probe_e086cc_sweep -- --nocapture --test-threads=1
    # the two decisive cross-checks, against the real suites and not the probe:
    W_SIM=0.30 W_SIMW=0.075 W_NOV=0.05 W_WNOV=0.15 W_FAM=0.10 W_RHY=0.25 \
      W_SHA=0.00 W_PARS=0.15 W_PUN=0.10 cargo test --release --test emit_coverage
    W_PARS=0.00 W_NOV=0.05 W_SIM=0.25 W_WNOV=0.15 W_FAM=0.10 W_RHY=0.30 W_SHA=0.05 \
      W_PUN=0.10 W_SIMW=0.0625 cargo test --release --lib \
      a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple

`cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run on this host**
(no `rustup`, no `rustfmt`/`clippy`/`rustdoc`) and are **not claimed**.

---

## 0. The one-sentence finding

**Six of the eight fences are weight-coupled and two are weight-independent, and the split
does not follow the intuition the previous front was working from** - in particular the
alarming `hid` row is *incidental*, and the fence the previous front used as its instrument,
the reserve-depth one, is the fence the previous front mis-measured. Underneath all of it
sits a stronger version of the load-bearing fact than the item stated: **not only is no
objective weight search-neutral, there is no search-neutral *placement* of one either**,
because the search's discard threshold is itself a final-scorer value. The objective is an
input to the search.

## 1. Baseline, re-derived on integrated HEAD `106afc5`

`src/` and `tests/` are byte-identical between `8bfe7de` (the previous front's base) and this
one, so every disagreement recorded below is a measurement and not a base drift.

| | measured | verdict |
|---|---|---|
| `cargo test --release --lib` | **75 / 0 / 12** | unchanged by this item |
| `corpus_integration` (`--test-threads=2`) | **12 / 1**, the one being case 2 | base red, not re-pinned |
| `emit_coverage` / `approx_determinism` / `exact_determinism` | 4/0, 4/0, 1/0 | green |
| `no_phrase_hard_coding` | 9/9 | green |
| case-2 pool | **18,949** | matches `REPORT-3f8c62.md` §1 |
| case-1 pool, green-case rank and score | **18,289**, **27**, `0.9199502875218423` | matches |
| `head_not_worse_than_pool` | **2 of 10 below the pool mean** (`they ate the whole pie` −0.0094, `my brother lost his wallet` −0.0194) | matches; the two identities and the signs are the criterion |

The probe's control vector - the eight axis weights at their HEAD values with the parsimony
axis at weight 0 - reproduces all of the above to the digit, including the two identity of the
failing head targets, which is what makes the eleven-vector sweep below a measurement of the
production surface rather than of a description of it.

## 2. The measurement

One build, eleven vectors, 39 measurements per vector, 7-12 ordinary English targets per
measurement, every one of the eight fences plus the head criterion. The eight fences are
measured by their own observables (the reserve's deepest placed tuple; the named wording and
word witnesses; the alignment's presence and its words' presence; the printed top-10; the
green case's pool rank and printed presence), never by a proxy of them.

**Admissible** means the positive weights sum to ≤ 1.00, so the objective's maximum stays
inside the documented `Clue::score` range. **Head** is `head_not_worse_than_pool` counted as
targets with a negative lift, so `0/10` is the criterion green.

| vector | (sim, nov, wnov, fam, rhy, sha, **pars**, pun) | ceiling | f1 reserve-depth | f2 deep-in-span | f3 past-opening-width | f4 deeper-than-one-walk | f5a alignment absent | f5b every word present | f6 output lock | f7 green case printed | f8 green case in pool | head |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| control (HEAD weights, axis 0) | .25 .15 .15 .10 .30 .05 **.00** .10 | 1.00 | **G** | **G** | **G** | **G** | **G** | **G** | **G** | **G** | **G** | 2/10 |
| A1 = the landed C1d | .25 .05 .15 .10 .25 .00 **.15** .10 | 0.95 | R | R | R | R | **G** | **R** | R | **R** | G | **0/10** |
| A2 C1 shape, pars .10 | .25 .10 .15 .10 .30 .00 **.10** .10 | 1.00 | R | R | R | R | **G** | **G** | R | **R** | G | 2/10 |
| A3 pars .20 | .25 .00 .15 .10 .20 .00 **.20** .10 | 1.00 | R | R | R | R | **G** | **G** | R | **R** | G | **0/10** |
| A4 pars .30 | .25 .00 .10 .10 .15 .00 **.30** .10 | 1.00 | R | R | R | R | **G** | **G** | R | **R** | G | 2/10 |
| A5 sim .30, pars .15 | **.30** .05 .15 .10 .25 .00 **.15** .10 | 1.00 | R | R | R | R | **G** | **G** | R | **R** | G | **0/10** |
| A6 pars .05 | .25 .10 .15 .10 .20 .05 **.05** .10 | 1.00 | R | R | R | R | **G** | **R** | R | **G** | G | 2/10 |
| B1 axis .15, **unfunded** | .25 .15 .15 .10 .30 .05 **.15** .10 | 1.10 | R | R | R | R | **G** | **G** | R | **G** | G | 2/10 |
| B2 SHAPE → 0 | .25 .15 .15 .10 .30 **.00** .00 .10 | 1.00 | **G** | R | **G** | R | **G** | **G** | R | **R** | G | 2/10 |
| B3 RHYTHM → .25 | .25 .15 .15 .10 **.25** .05 .00 .10 | 1.00 | **G** | R | **G** | **G** | **G** | **G** | R | **G** | G | 2/10 |
| B4 NOVELTY → .05 | .25 **.05** .15 .10 .30 .05 .00 .10 | 1.00 | **R** | **G** | **R** | **R** | **G** | **G** | R | **G** | G | 1/10 |

Supporting numbers for the rows the table's `G`/`R` does not carry: the green case's pool rank
is 27 at control, B1, B3, B4 and A6, **48** at A1, A2 and B2, 200 at A3, 662 at A4 and **40**
at A5; `hid` leaves the pool entirely at exactly two vectors, **A1 and A6**; the case-2
canonical alignment is in the pool at **none** of the eleven.

### 2.1 Cross-checked against the real suites, not only the probe

The probe is a re-implementation of eight fences, so the two decisive rows were re-measured by
running the fences themselves.

* **A5, `emit_coverage`: 3 of 4 pass.** `a_lattice_alignment_can_be_absent_from_the_production_pool`
  is **green** at A5 and **red** at A1, and at A1 it is red for exactly the reason the probe
  reports (`the word "hid" is absent from the pool entirely`). So the alarming row is real and
  it is **incidental**: it is a property of that funding mix, not of the axis.
* **A5, `corpus_integration`: 10 of 12, 1 failed at base and 1 new** - the three pool-reach
  fences, the output lock and the green-case display fence, exactly as the table says.
* **The reserve-depth fence, run as the real test at five vectors**: control green, **B2
  green, B3 green, B4 red, A1 red**. Reproduces §3.

## 3. The load-bearing general fact, confirmed and strengthened

The item's premise, which I confirm: **no objective weight in this search is search-neutral.**
Every weight is read both by the final scorer and by the walk's best-first heap key
`bound(prefix)` (`src/lib.rs:1920-1939`).

The premise is also, on this evidence, *weaker than the truth*, and the difference is the most
useful thing this front found.

> **The search's discard threshold is a final-scorer value.** The bar a span path must beat to
> survive is the final-scorer value of the worst clue the ordinary beam has already committed
> to (`src/lib.rs:1190-1205`), and a `span_score_bound` that cannot beat it returns
> `NEG_INFINITY`, which drops the path. So **a term in the final scorer alone is not
> search-neutral either**: re-weighting the scorer moves the bar, which moves which span paths
> the bound may discard, which moves what is in the pool at all.

Measured, with a scorer-only weight channel read by `Partial::metrics` and by nothing else, so
that no search key could move: five perturbations, seven ordinary targets, a very large
`top_n`, the pool's phrase set and distinct-word set compared.

| scorer-only perturbation | distinct words over 7 targets | pool sizes that moved |
|---|---|---|
| none (base) | **5,185** | — |
| `NOVELTY` 0.15 → 0.30 | 5,177 | 6 of 7, up to **+156** candidates |
| parsimony axis at 0.15, scorer alone | 5,158 | 7 of 7, up to **−426** |
| `SIMILARITY` 0.25 → 0.05 | 5,190 | 7 of 7 |
| `SHAPE` 0.05 → 0.20 | 5,187 | 7 of 7 |
| `RHYTHM` 0.30 → 0, `PUNCH` 0.10 → 0 | 5,148 | 7 of 7 |

**0 of 5 left the pool invariant**, in both directions. Note the third row specifically: that is
`REPORT-3f8c62.md` §4 **row 1** - "the axis in the final scorer only" - and it moves the pool on
every one of seven targets.

Two corrections follow, and both matter for whoever takes this next.

* **`REPORT-3f8c62.md` §4 rows 1 and 2 are false as written.** "A term in the final scorer
  alone is a *scoring* change and the search does not move" is refuted above. The pairing of
  rows 1 and 2 - scorer-only is free, but a scorer-only term is unsound unless the bound has it
  too - is the right *shape* with the wrong first half: a scorer-only term is neither free nor
  the safe option.
* **§5's operative rule is refuted.** "An objective term is not free unless it is in the scorer
  alone, and a scorer-only term is not sound unless the bound has it too" - the first clause is
  false. There is no placement of a new axis that leaves the search alone, **including the
  scorer**, which is why §5's unblock is larger than that front assumed.

The tree's comment at the threshold already said the discard is "sound: nothing the bound covers
could have entered the output anyway". That is true, and it is true *for the objective that
built both sides of the comparison* - which the comment did not say. The comment now says it,
and `tests/objective_is_a_search_input.rs` carries the measurement.

## 4. The classification, fence by fence

Two of the eight are sound properties of the search. Six are incidental and must be re-derived
together with whatever vector lands. For each, the measurement that decided it, and the
weight-free form where one exists.

### 4.1 Weight-independent (2) - a sound property of the search, true at every vector measured

| fence | content | why it is weight-independent | weight-free form | test |
|---|---|---|---|---|
| `a_lattice_alignment_can_be_absent_from_the_production_pool`, **first** assertion | the alignment is absent from the production pool | **11 of 11 vectors**, including every axis-carrying one, and the `OBSTRUCTION-MAP.md` §1 count of 1,126 better pool clues | *no monotone objective over measurable per-candidate properties ranks a tuple absent from the pool* - a statement about the objective, not the search, and already fenced by the base-red `approximate_finds_classic_madgab_resegmentation`, which stays red | already the existing test; **nothing to add** |
| the green case's **pool** half (`f8`; the pool presence that `the_other_canonical_resegmentation_is_still_proposed` and `approximate_finds_recognize_speech_resegmentation` both presuppose) | the resegmentation is in the production pool | **11 of 11 vectors**, rank 27 … 662 but present everywhere | *the pool is not a display*: a target's best resegmentation is in the pool even when it is not printed | the pool half of the two existing green-case tests; **nothing to add** |

These two are what the map's §1 calls "the one standing green fact" and "the one standing
blocker", and they are the *only* two of the eight that survive every vector. Note what they
have in common: both are statements about **absence or presence of a combination in a set**,
which is set-theoretic and does not read a weight.

### 4.2 Weight-coupled (6) - incidental, and re-derived with the vector

| fence | green at | red at | classification and what it costs |
|---|---|---|---|
| **f1** `a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` | control, **B2** (SHAPE→0), **B3** (RHYTHM→.25) | **B4** (NOVELTY→.05) and all **7** axis-carrying vectors, incl. all three that are 10/10 on the head | **weight-coupled**, and coupled to the axis's *presence in the keys* rather than to any weight value: worst placed depth is 2 or 3 against the derivation's 4. Re-derive with the vector. |
| **f2** `approximate_pool_reaches_matches_deep_in_a_span` | control, **B4** | the other 9, incl. all 7 axis-carrying | **weight-coupled.** |
| **f3** `approximate_pool_reaches_alternatives_past_the_opening_slot_width` | control, **B2**, **B3** | the other 8, incl. all 7 axis-carrying | **weight-coupled.** |
| **f4** `approximate_pool_reaches_resegmentations_deeper_than_one_walk` | control, **B3** only | the other 9, incl. all 7 axis-carrying | **weight-coupled**, and the *least* incidental of the four: one weight change restores it. |
| **f5** `a_lattice_alignment_can_be_absent_from_the_production_pool`, **second** assertion (`hid` leaves the pool) | **9 of 11**, including **A5**, an admissible vector that is **10/10 on the head** | A1 and A6 only | **weight-coupled** - and this is the item's §3 question answered. The alarming row is a property of two funding mixes, not of the axis. Its weight-free form is *not expressible*, and §5 shows why: the pool's word set is not objective-invariant. |
| **f6** `approximate_output_is_locked` | the shipped vector only | **10 of 10** others | **weight-coupled by construction** - it is the lock, not a property. It exists to make an objective change a conscious act, and re-baselining it is the act. No weight-free form; re-derive with the vector, deliberately. |
| **f7** the green case's **display** half | **A6** (axis 0.05, admissible), B1, B3, B4, control | A1, A2, A3, A4, A5, B2 - including **the three vectors that are 10/10 on the head** | **weight-coupled, and in direct conflict with the head criterion**: see §5. |

**Weight-free forms that do not exist, and the reason.** For f1-f4 and f7, the content is a
*reach* claim - "this wording, or this resegmentation, or this display, exists" - and reach is
exactly what §3 shows to be a function of the objective. There is therefore no weight-free
version of them to assert; asserting one would be asserting a wish. What is weight-free, and is
already fenced, is the *admissibility* each of them rests on: a discard never removes a
candidate better than the search's own threshold, which is `structural_bounds_dominate_the_real_scorer`
and must be re-measured at every vector. So: for these five, the honest statement is **the fences
are vector-relative and their only weight-free content is the bound's admissibility, which is
already a red/green test at the library boundary.**

## 5. Why this is HOLD, and what would unblock it

**HOLD.** The classification is delivered, and the item's own success criterion for INTEGRATE -
*a fence re-derivation plus the smallest objective change that keeps every sound fence green* -
is not met, for a reason that is now measured rather than argued.

**No admissible vector keeps the head criterion and the green case's display simultaneously.**
Across the six admissible axis-carrying vectors: A1, A3 and A5 are **10/10 on the head** and
all three are **red on f7**; A6 is the only one green on f7 and it is **2/10 on the head**; A2
and A4 are red on both. The item's own instruction is that a loss of the green case is a reason
to report HOLD and not a weight to raise, and no weight was raised on account of it.

That is a *conflict*, not a re-derivation, and the six weight-coupled fences cannot be
re-derived out of it, because of §3: a vector changes the search's own discard threshold, so
"these fences hold at this vector" is a statement about a *different search*, and re-deriving
them means admitting that the search moved. Re-deriving f1-f4 and f7 as anything other than
vector-relative properties would be re-pinning them, which this item forbids and which
`OBSTRUCTION-MAP.md` §2 exists to prevent.

**The exact unblocking condition**, in the order I would do it:

1. **Make the discard threshold objective-independent, or accept that the objective is a search
   input.** The cheapest version: derive the threshold from the structural keys alone, so the
   objective returns to being a ranker over a fixed candidate set and §3's invariance becomes
   true and assertable. This is a search change, on a surface this item forbids, and it is
   where the work belongs. Its price is a weaker prune, so it has to be bought against the
   emission ceiling, not asserted.
2. **Or land on the strength of A5 and re-derive, knowingly.** A5 is the best vector measured:
   ceiling 1.00, **10/10 on the head**, and the only vector that is simultaneously **green on the
   lattice fence** (so the `hid` half needs no re-derivation), at 7.9% pool. It is red on f1-f4,
   f6 and f7, i.e. **six of the eight** - one more than C1d. That is a worse trade than C1d, and
   on this evidence it is not a trade at all.
3. **Not a weight, and not a placement of the axis.** §3 prices the placement lever and closes
   it: there is no search-neutral placement, so "put the axis somewhere cheaper" is not
   available. §4.2 prices the weight lever: of the three one-weight changes that are candidates
   for funding (B2, B3, B4), two restore fences the head criterion does not need and one
   (`NOVELTY → 0.05`) is itself 1/10 from the head while being red on f1, f3 and f4.

## 6. The weight-free general property, and its test

The weight-free property that survives, and is landed, is not about any weight. It is the
separation the re-derivation needs:

> **The display is a pure function of the pool and of the candidates' scores.** The pool is
> returned in non-increasing score order; every printed clue is a pool member with the same
> score to the bit; the printed phrases are pairwise distinct; and the printed list is exactly
> `min(top_n, pool)` long.

Green on HEAD, 12 target/length combinations, at the library boundary, through the public API,
in `tests/objective_is_a_search_input.rs`. It is what makes f7's row attributable at all: with
it, "the objective moved the pool" and "the objective moved the display" are separable
questions, and a pool-reach fence and a display fence cannot be conflated - which is the failure
mode `OBSTRUCTION-MAP.md` §2 row 4 records twice.

Its companion, also landed as a comment at `src/lib.rs:1184-1201`, is §3: the threshold is a
final-scorer value, so the objective is a search input, and the discard's soundness is a
statement about the pair.

What I could **not** express as a green weight-free test, and why, for each fence, is in §4.2.
The short form: the *reach* content of f1-f4 and f7 is not expressible, because §3 measures that
the pool is not objective-invariant - **0 of 5** scorer-only perturbations preserved it - so "no
objective weight removes a candidate" is false and cannot be fenced. The stability half (two
searches at one objective return one pool) is `tests/approx_determinism.rs` and is not restated.

## 7. Constraints, and how each was honoured

* **No phrase-specific hard-coding.** The landed test names four ordinary English targets and
  no expected output, clue or score; `src/` gains only a comment. `no_phrase_hard_coding` is
  **9/9**. Nothing anywhere is keyed on `hid`, on `wreck a nice beach` or on
  `Hits Justice Dupe Hid Came`; `hid` appears in this report, in the test's own failure
  messages, and in the *fence's* existing assertions, and in no production code.
* **No fence weakened, deleted or `#[ignore]`d.** `cargo test --release --lib` is
  **75 / 0 / 12**, byte-for-byte the base. `head_not_worse_than_pool` is still red and still
  `#[ignore]`d. No test in `src/lib.rs` or `tests/` was modified. The two landed artefacts are
  one new test file and one comment.
* **`coverage_tuples`, `sweep_index`, `EMIT_PROFILE_*` untouched.** Confirmed by diff.
* **No `OBSTRUCTION-MAP.md` row re-opened.** Rows 1-13 are cited, not re-run; the canonical
  case-2 tuple is measured absent from the pool at all eleven vectors and is **not** claimed as
  progress. `approximate_finds_classic_madgab_resegmentation` is red at base and red here.
* **Instrumentation.** The weight-vector channel and the probe modules live only on the local,
  unpushed `scratch/e086cc-probe`. Nothing in the environment, no `ZZ_*` probe, no knob, no
  debug binary reaches `madgab-e086cc`.
* **Release mode** for every number in this report.
* **No self-merge.** `main` and `post-milestone-acceptance` are untouched.

## 8. Handoff

* Terminal: **HOLD**. `docs/work/REPORT-e086cc.md` on `madgab-e086cc`.
* **The classification is the deliverable** (§4). Six of the eight fences are incidental and
  two are sound; the split is a set-theoretic one (statements about a set containing a
  combination survive; statements about a wording being produced do not).
* **The strengthened general fact, stated once so it can be cited without re-deriving.**
  *Every objective weight in this search is read by the search, and there is no placement of a
  term that is not.* Two sites, both measured: the walk's `bound(prefix)` heap key
  (`src/lib.rs:1920-1939`), and the **discard threshold, which is a final-scorer value**
  (`src/lib.rs:1190-1205`). A `span_score_bound` that cannot beat that bar returns
  `NEG_INFINITY` and drops the span path, so re-weighting the scorer alone moves the pool:
  **0 of 5** scorer-only perturbations left it invariant over seven ordinary targets, by up to
  **426** candidates. Consequence for planning: an objective change is a **reach** change
  first and a ranking change second, and `REPORT-3f8c62.md` §5's rule that a term "is not free
  unless it is in the scorer alone" is false.
* **The two corrected rows of `REPORT-3f8c62.md` §4**, carried forward as corrections, not as
  reconciliations (§9 holds both readings):
  * **rows 1-2** — "a term in the final scorer alone is a pure scoring change and the search
    does not move" is refuted; row 1's own variant (the axis in the scorer only) moved the pool
    on **7 of 7** targets.
  * **rows 6-8** — transposed. The one weight that reddens the reserve-depth fence on its own
    is **`NOVELTY` 0.15 → 0.05**; `RHYTHM 0.30 → 0.25` and `SHAPE 0.05 → 0.00` are both
    **green**. Verified by running the fence at all five vectors.
* **Why no vector is landable, in one line of arithmetic** (detail in §5). The criterion is
  `head_not_worse_than_pool` 10/10; the fence is the green case in the **printed** 50. Over the
  six admissible axis-carrying vectors:

  | | head 10/10 | green case printed |
  |---|---|---|
  | A1 = C1d, A3, A5 | **yes** | **no** (pool rank 48 / 200 / 40) |
  | A6 (axis 0.05) | no — 2/10 | **yes** (pool rank 27) |
  | A2, A4 | no — 2/10 | no |

  **The set is empty.** The only vector green on the display is the one that does not fix the
  head, and all three that fix the head lose the display. No weight was raised on account of the
  green case, per the item's own instruction.
* **Successor rule** (the shape of the next front, so this one is not re-run):
  **an objective change may only be landed together with a re-derivation of the six
  weight-coupled fences as that vector's properties, never as properties of the search** — and
  §3 shows why that is not a formality: a vector changes the search's own threshold, so
  re-deriving them is admitting the search moved. Consequently the live direction is the
  **threshold**, not the objective: derive it from the structural keys so the objective returns
  to being a ranker over a fixed candidate set, and buy the weaker prune against the emission
  ceiling rather than asserting it. **Do not open a front that re-runs a weight vector** (the
  weight lever is closed by §5 item 3 and the placement lever by §3), and **do not attack
  `RHYTHM` or `SHAPE`** — §9 shows they were never the problem.
* The next front owns the search surface, and its first job is the **threshold**, not the axis.
  See §5 item 1. `scratch/e086cc-probe` is local and unpushed and carries the eleven-vector
  harness to re-measure from; `scratch-3f8c62-landed` (`514ed91`) is still the unlanded C1d
  variant. Neither may be integrated as it stands.
* The canonical case-2 tuple remains unreachable by any monotone objective
  (`REPORT-3a8c05.md` §3, rank ≥ 1,127; re-measured absent from the pool at 11 of 11 vectors).
  Do not let a later pass read this front as progress on it.
* **Not claimed:** `cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests — no `rustup`
  and no `rustfmt`/`clippy`/`rustdoc` on this host. `src/lib.rs` carries **comment-only**
  changes, 16 added lines and **zero** production-logic lines, plus one new test file; no fence
  weakened, deleted or `#[ignore]`d, and the ignored head fence's own red is unchanged
  (−0.0080 and −0.0184, `REPORT-3f8c62.md` §1.1).

## 9. A correction to `REPORT-3f8c62.md` §4 rows 6-8, kept separate because it is a disagreement

`REPORT-3f8c62.md` §4 rows 6-8 measure three one-weight changes from HEAD with the axis absent,
and read: `NOVELTY 0.15→0.05` **green**, `RHYTHM 0.30→0.25` **red**, `SHAPE 0.05→0.00` **red**
- concluding that "the weights are not a way out" and that "`NOVELTY` alone is green, which is
the only reason the *funding* mix looked like a lever; funding from `RHYTHM` and `SHAPE` is not."

Measured here, on the same weights, with the same axis-absent tree, and cross-checked by running
the reserve-depth fence itself at all five vectors (§2.1):

| one-weight change from HEAD | `REPORT-3f8c62.md` §4 | measured here |
|---|---|---|
| `NOVELTY` 0.15 → 0.05 | row 6, **green** | **RED** (worst placed depth 3 against 4) |
| `RHYTHM` 0.30 → 0.25 | row 7, **red** | **GREEN** |
| `SHAPE` 0.05 → 0.00 | row 8, **red** | **GREEN** |

The three are transposed, and the summary sentence built on them is inverted with them. What
survives of §4's conclusion is only that **the funding mix is not a way out**; the mechanism it
gives for that is wrong. The correct mechanism is the one §4.2's table shows: of the three
candidate funding sources, the one that reddens the reach fences on its own is **`NOVELTY`**,
which is also the one C1d takes 0.10 from - so C1d's funding compounds the effect rather than
causing it, and `SHAPE → 0` (the other half of C1d's funding) is **green on three of the four
weight-coupled fences** and is not the problem.

Both numbers are recorded rather than reconciled, per this repository's standing practice. The
disagreement does not change any verdict in §4 or §5 - no vector is landable either way - but
it does change what a re-derivation front should attack, and the transposed reading would have
it attack `RHYTHM` and `SHAPE`, which are the two axes that were never the problem.

HOLD
