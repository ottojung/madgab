---
work_item: true
id: w-9b4a15
state: done
priority: normal
owner: front-9b4a15
updated: 2026-09-27T20:40:00Z
branch: madgab-parsimony-9b4a15
worktree: /workspace/madgab-parsimony-9b4a15
base: a1d48c0
verdict: HOLD
---

# w-9b4a15 - a word-count parsimony axis is the right repair, and it cannot be shipped by this front: three fences, none of them the weights

Front `front-9b4a15` (recovered; predecessors `agent-9b4a151` and `agent-9b4a152` died on
this worktree), branch `madgab-parsimony-9b4a15`, base `a1d48c0` (integrated HEAD, with
`w-3a8c05`'s report in it). Release mode throughout, `CARGO_TARGET_DIR=target`.
**Outcome: HOLD** - decomposition and pricing only, which this item defines as a full
success. No production line changed; the branch is `#[cfg(test)]` instrumentation and this
report.

Reproduce:

    cargo test --release --lib price_the_budget -- --ignored --nocapture
    cargo test --release --lib price_the_property_over_a_spread -- --ignored --nocapture
    cargo test --release --lib head_not_worse_than_pool -- --ignored --nocapture

`cargo fmt --check` and `cargo clippy` cannot run on this host
(see [../environment-notes.md](../environment-notes.md)) and are **not claimed**.

---

## 0. What is in the branch

`src/lib.rs` only, and every line of it behind `#[cfg(test)]`: the `front_9b4a15` re-ranking
harness carried over from the two checkpoint commits (inherited, and corrected - see §2),
four further candidates priced by this front, and the `head_not_worse_than_pool` regression
fence. No weight, no axis, no `Metrics` field, no `SPARSE_SHORTLIST`, no
`LEXICAL_HEAP_POP_LIMIT`, no budget, no depth, no retention, no emission order, no
baseline, no `examples/`, no `ZZ_*` probe, no environment knob. Nothing outside a test
module in `src/` names an example token.

## 1. The baseline, re-derived

On integrated HEAD, release mode, the deduplicated score-ordered pool from
`Generator::generate_pool` at default configuration:

| | shipped |
|---|---|
| `It's just a stupid game` pool | 18,949 |
| its head-50 `SIMILARITY` lift over the pool mean | **−0.0180** |
| its head-200 clue word-count histogram | `{5: 27, 6: 173}` |
| `recognize speech` pool | 18,289 |
| its head-50 `SIMILARITY` lift over the pool mean | **+0.0609** |
| its head-200 clue word-count histogram | **`{4: 200}`** |
| the green case's pool rank | **27**, score 0.9199502875 |
| the canonical case-2 tuple | **absent from the pool** |

Every figure the item quoted reproduces to the digit, including the case-1 immunity
histogram `{4: 200}` that the immunity argument rests on, and including both head lifts.

The case-2 tuple is a finding, not an inconvenience. It is not ranked 18,949th; it is not
in the pool at all. The rows that mention either canonical word are of the shape `it
justice too bad …`, and no row is the canonical tuple. That is the same fact
`approximate_finds_classic_madgab_resegmentation` reports as its pre-existing red, and it
is `w-1c7d40`'s to own. So the item's "rank and score of the canonical case-2 tuple before
and after" is **unmeasurable on this surface by any scoring change**; what *is* measurable
is the pool's head lift, which is what §3 prices. [../REPORT-3a8c05.md](../REPORT-3a8c05.md)
§4's 7,393 for C1b is a rank in a wider enumeration than the approximate search's pool and
is cited, not re-derived here.

## 2. The fence, and a defect in the inherited version of it

The property the item asks to be proved is, for a fixed target, `mean(SIMILARITY, top-N) >=
mean(SIMILARITY, pool)` - the generator's own head must not be a worse phonetic match than
the set it was drawn from. It is expressed with ten ordinary targets, no clue, no expected
phrase and no rank, and it lives at the library boundary: the pool is
`Generator::generate_pool`'s own return value and the per-candidate `SIMILARITY` is the
scorer's own.

**The inherited version of this test measured the wrong head.** The candidate capture
arrives in *emission* order, and the test took the first 50 captured rows as the head. The
returned pool is in descending score order, so the first 50 captured rows are a prefix of
the search's output, not its top 50. The two orders name **two entirely different targets**
as the failing ones:

```text
capture order (wrong):  "he runs to the station"     -0.0526
                        "we should leave earlier"    -0.0825
score order (correct):  "they ate the whole pie"     -0.0080
                        "my brother lost his wallet" -0.0184
```

Both readings are red, so the test's *colour* was never wrong, but the diagnosis, and
every price in §3 computed from the same collapsed pool, were. The harness's own
`dedup` had the same defect in a second form: it deduplicated in capture order, so it kept
the first-seen rather than the highest-scoring member of each spelling-variant group,
unlike `generate_approximate`, which sorts first and then keeps the first. Both are fixed
here, and the fence now **asserts** that its score-ordered, deduplicated capture reproduces
the returned pool's phrase sequence exactly, so the two cannot drift apart again:

```text
cargo test --release --lib head_not_worse_than_pool -- --ignored --nocapture
  they ate the whole pie        head 50 mean 0.7390 against pool 18116 mean 0.7470, lift -0.0080
  my brother lost his wallet    head 50 mean 0.8036 against pool 19663 mean 0.8220, lift -0.0184
  ... 8 of 10 above the pool mean, up to +0.1498
```

**RED is the recorded colour on HEAD.** The test is `#[ignore]`d and stays in the tree: a
red test left visible is the fence a later front has to make green, and deleting it is how
it stops being anyone's problem.

## 3. The prices, on the same captures

Fourteen weight vectors, each a function of two word counts and the shipped axes only. The
PARSIMONY axis is `1 - |w_clue - w_target| / max(w_clue, w_target)`; it reads two integers
and no word, letter, clue or target text. `ceiling` is the objective's maximum, the sum of
its positive weights, and every candidate below is at or under the documented 1.0.

Spread of ten ordinary targets, "green" = the candidate's head-50 mean `SIMILARITY` at or
above the pool mean:

| candidate | ceiling | spread | green case rank | case-2 head lift |
|---|---|---|---|---|
| C0 as shipped | 1.00 | 8/10 | **27** | **−0.0180** |
| C1 `PARS 0.10`, `SHAPE→0`, `RHY 0.30→0.25` | 1.00 | 8/10 | 46 | +0.0345 |
| C1b `PARS 0.15`, `SHAPE→0`, `NOVELTY→0` | 0.95 | **10/10** | **166** | +0.0565 |
| C1f `PARS 0.05`, `SHAPE→0` | 1.00 | 8/10 | 46 | +0.0249 |
| C1c `PARS 0.10`, `SHAPE→0`, `RHY→0.20` | 0.95 | 8/10 | 46 | +0.0345 |
| C1g `PARS 0.10`, `SHAPE→0`, `NOVELTY→0.05` | 0.95 | 9/10 | 46 | +0.0437 |
| C1d `PARS 0.15`, `SHAPE→0`, `NOVELTY→0.05`, `RHY→0.25` | 0.95 | 9/10 | 46 | +0.0437 |
| C1h `PARS 0.15`, `SHAPE→0`, `NOVELTY→0`, `RHY→0.20` | 0.85 | **10/10** | **166** | +0.0565 |
| C1i `PARS 0.15`, `SHAPE→0`, `NOVELTY→0.05` | 1.00 | 9/10 | 46 | +0.0437 |
| C1j `PARS 0.10`, `SHAPE→0`, `NOVELTY→0.05`, `SIM 0.25→0.30` | 1.00 | **10/10** | **38** | +0.0537 |
| C1k `PARS 0.15`, `SHAPE→0`, `NOVELTY→0.05`, `SIM→0.30`, `RHY→0.25` | 1.00 | **10/10** | **38** | +0.0537 |
| C1l `PARS 0.10`, `SHAPE→0`, `NOVELTY→0`, `SIM→0.30` | 0.95 | 10/10 | 149 | +0.0699 |
| C2b `RESEG 0.15`, `SHAPE→0`, `NOVELTY→0` | 0.95 | 8/10 | 2,604 | +0.0299 |
| C2d `RESEG 0.15`, `SHAPE→0`, `NOVELTY→0.05`, `RHY→0.25` | 0.95 | **10/10** | **3,247** | +0.0299 |

Reading of the table:

* The axis works. It is the intended repair and it does what
  [../REPORT-3a8c05.md](../REPORT-3a8c05.md) said it would: the case-2 head goes from
  −0.0180 to +0.0345 at the item's own comparator C1, and `PARSIMONY`'s own head lift
  goes from −0.0157 to +0.1343. Its `PUNCH` lift collapses from +0.1526 to −0.0274, so
  the "monotone short-words" artefact really is an artefact.
* **Weight on `RHYTHM` is the wrong place to pay for it.** C1c (`RHY 0.30→0.20`) leaves
  both failing lifts bit-identical to C1. Both failures are repaired by weight off
  `NOVELTY`, which is the axis a mis-worded clue satisfies by shredding the target.
* **`PARSIMONY` alone is not enough at a fixed budget.** C1f (0.05) and C1 (0.10) leave the
  spread at 8/10; C1g and C1d reach 9/10. The last 0.05 has to come from somewhere else.
* **The alternative use of the same budget is priced and is worse.** Handing the freed
  0.15 to boundary fidelity instead (`C2b`, `C2d`) buys a large `RESEG` lift (+0.5034) and
  a smaller acoustic one (+0.0299 vs +0.0345 for the same spend on `PARSIMONY`), and costs
  the green case 2,604–3,247 ranks. C2d does reach 10/10 and is still inadmissible, for
  the same reason C1b is.

One caveat on the whole table, because it decides §4: every axis weight in it is read by
the search's internal proxies as well as by the final scorer, so *no* candidate here is
literally search-neutral, and "leaves the search alone" below means "was measured not to
move the reserve-depth fence", which is the only search-side fence that moved.

## 4. Why this is HOLD and not INTEGRATE

Two weight vectors reach 10/10 *and* keep the green case inside the top 50 - C1j and C1k,
at rank 38. They were implemented in full, as production code, and measured end to end.
**Both break the tree**, in different ways, and neither break is about the parsimony axis
being wrong:

1. `structural_bounds_dominate_the_real_scorer` goes red for **every** candidate that adds
   the axis to the final scorer. `span_score_bound` and `complete_span_score` shadow the
   final score; the word-count axis is not in them, so the structural key understates the
   score it is supposed to bound. Measured, C1: *"structure key 0.8423 is below the real
   score 0.8466"*. Repairing it means teaching the structural DP the axis - which is
   `w-1c7d40`'s search surface, the one this item forbids this front to touch.
2. `a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple` - `w-1c7d40`'s
   reserve-depth fence - goes red for C1k and for C1j, the two 10/10 vectors that also buy
   weight by raising `SIMILARITY`. `SIMILARITY`, `NOVELTY` and `RHYTHM` are read by the
   search's own proxies, so those vectors change which candidates the search keeps, and
   they move the reserve. Raising `SIMILARITY` is a *search* change wearing a scoring
   change's clothes, and the item's red line is that there are none.

(The third failure seen while pricing, `incremental_aggregates_match_a_full_refold`, is an
artefact of the incremental test twin's fixed four-word target once the axis enters
`combined`; it is a test-side follow-on, not an independent obstacle.)

So the obstruction is three-sided and none of the three sides is a weight:

* **reserve-depth-breaking** (C1j, C1k): the only 10/10 vectors that keep the green case
  inside the top 50, and they break `w-1c7d40`'s reserve-depth fence by raising
  `SIMILARITY`, which is a *search* change wearing a scoring change's clothes;
* **green-case-dropping** (C1b, C1h, C1l, C2d): the 10/10 vectors that do not break it, and
  they push the green case to 149–3,247;
* **bound-unsound** (all of them): the axis is not in the structural keys, so shipping it
  in the final scorer alone makes an existing soundness fence false.

The item's own successor scope says reaching better case-2 behaviour is `w-1c7d40`'s, and
§3 shows this axis does not change the case-2 picture anyway (the tuple is absent, not
mis-ranked). The honest contribution here is the decomposition, the corrected measurement,
the fourteen-row price, and the red fence - and the red fence is the point: it names the
defect precisely enough that the front which can touch the search surface can fix it.

## 5. Cost

No production line changed, so pool sizes, wall clock and the score's 0.0–1.0 range are
unchanged by construction. Measured for the record: the two canonical pools in 3.1 s
together; the ten-target fence, which builds and verifies a full pool per target, in 14.6 s;
`corpus_integration`'s 13 searches in 51.6 s single-threaded. Against the item's 9.5 s and
21.3 s base figures there is nothing to compare, because nothing moved. Every candidate in
§3 is inside the range: `ceiling` is the objective's maximum and the largest is 1.00, with
`CLOSED_CLASS` and `PUNCH` both non-positive by construction.

## 6. Findings for other fronts

* **For `w-1c7d40` (owns enumeration/order, and the search surface this front may not
  touch).** The cheapest complete version of this repair is: add
  `1 - |w_clue - w_target| / max(w_clue, w_target)` at weight 0.15, take 0.05 from `SHAPE`,
  0.10 from `NOVELTY`, and 0.05 from `RHYTHM` (C1d); add it to `complete_span_score`,
  `partial_span_score` and `span_score_bound` as well as the final scorer, so
  `structural_bounds_dominate_the_real_scorer` stays true; keep `SIMILARITY` at 0.25, since
  every candidate that raises it breaks your reserve-depth fence. That vector is 9/10 on the
  spread, keeps the green case at rank 46, and turns the case-2 head lift from −0.0180 to
  +0.0437 - a real improvement, at 9/10 rather than 10/10, and the residual is worth your
  own pricing rather than this front's.
* **For whoever writes the next front on the 10/10 row.** C1j and C1k are the only vectors
  that hold both properties, and they need the reserve-depth fence re-derived rather than
  preserved. That is a search-behaviour claim and needs its own item.
* **For the record, and not chased here.** The case-2 tuple is absent from the approximate
  pool, not mis-ranked in it. Any future report quoting a case-2 *rank* from this surface
  should say which pool it came from.
* **`REPORT-3a8c05` §5 note 1 is reproduced and confirmed**: the same target's top 50 goes
  from lift −0.0180 to +0.0565 under C1b, and the `PUNCH` lift from +0.1526 to −0.0274. Its
  "cannot ship" judgement was right, for a reason it did not have - §4.

## 7. Downstream fences

Release mode, one suite at a time, `CARGO_TARGET_DIR=target` (the corpus suite needs
`-- --test-threads=1` on this host or it is SIGKILLed by memory pressure, as `emit_coverage`
was on the inherited run):

| suite | result |
|---|---|
| `cargo test --release --lib` | **75 passed / 0 failed / 12 ignored** |
| `--test no_phrase_hard_coding` | **9 / 9** |
| `--test corpus_integration` | **12 passed / 1 failed** - `approximate_finds_classic_madgab_resegmentation`, the known pre-existing red, **not re-pinned** |
| `--test emit_coverage` | 4 / 4 |
| `--test approx_determinism` | 4 / 4 |
| `--test exact_determinism` | 1 / 1 |

The item's fence was written as "74 passed / 0 failed, 2 ignored". Measured on this branch
*and* on unmodified HEAD it is **75 / 0 / 12**: one more non-ignored and three more ignored
tests than the item's figure, all of them from work integrated after the item was opened.
Nothing regressed, and the discrepancy is reported rather than reconciled by editing the
fence.

The three ignored tests on this branch are this front's: the two pricing harnesses and the
red fence. `cargo test --release --doc` cannot run here (no `rustdoc`), so no doctest is
claimed either.

## 8. Verdict

**HOLD** - decomposition and pricing only, which this item defines as a full success. The
item's objective is unchanged: the shipped top 50 for a target is *not* guaranteed to be
acoustically at least the pool mean, and the red fence that says so is in the tree at
`head_not_worse_than_pool`. The word-count parsimony axis is confirmed as the right repair
and priced against one alternative use of the same budget, and §4 records why shipping it
is `w-1c7d40`'s ticket: the axis has to reach the structural keys, and the only vectors that
make the property universal also move the search.
