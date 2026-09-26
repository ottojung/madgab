# The aggregation form of the approximate objective — design space and verdict

Work item [w-9f1c05](items/w-9f1c05.md), front `9f1c05`, branch `madgab-aggform`.
Measurements, method and the full per-target tables are in
[`REPORT-9f1c05.md`](../../REPORT-9f1c05.md). This document is the design
half: what the aggregation form *is* in this codebase, the taxonomy of general
re-expressions, what each member costs, and why the family is closed.

**One-line answer, stated first so a reader can route without the rest:**

> **VERDICT: AGGREGATION IS NOT THE GATE.** Seven general aggregation forms
> were evaluated against a captured pool and three end-to-end. The best of them
> moves a 0.0958 deficit by 0.0125 and **zero ranks**; the form that moves it
> furthest moves it to rank 7380 of 17 922 at `--top 50`, still 0.0479 short of
> a 0.0047-wide band. The requested wording is not in the emitted pool at all, on
> any form, at any `top_n` — so no aggregation form can make it reachable,
> because reachability is decided before the objective is ever evaluated.

---

## 1. What the form actually is

`src/lib.rs:2407`, inside `Partial::metrics`:

```rust
let combined = axes::SIMILARITY * similarity
    + axes::NOVELTY * novelty
    + axes::WORD_NOVELTY * word_novelty
    + axes::FAMILIARITY * familiarity
    + axes::RHYTHM * rhythm
    + axes::SHAPE * shape_quality
    + axes::CLOSED_CLASS * closed_penalty;
```

Seven terms, six positive weights summing to exactly `1.00` and one signed
penalty at `-0.15`. `closed_penalty` is `(closed/words)²`. That is the whole of
the aggregation: an unweighted-in-any-other-sense linear form over per-candidate
axis values in `[0,1]`.

**The form is duplicated in four places, and this dominates the design space.**

| site | what it shadows | what breaks if you change only `combined` |
|---|---|---|
| `src/lib.rs:2407` `metrics` | the objective itself | — |
| `src/lib.rs:3014` `partial_span_score` | the structural DP's per-span representative key | the DP keeps the wrong representatives |
| `src/lib.rs:3038` `complete_span_score` | the key that orders structures for expansion | the wrong structures get expanded |
| `src/lib.rs:1583` `span_score_bound` | the **admissible upper bound** | the search is silently truncated |
| `src/lib.rs:785-800` + `:1194` | the `incumbent` threshold: a span path is dropped when its bound is under the `final_keep`-th best `combined` | the emitted **pool** changes |

The last row is the one that decides the question. `metrics().combined` is not
merely the number printed next to a clue: it is the search's own admission
threshold. So an aggregation form *can* change pool membership — but only
because it perturbs a cut whose soundness argument (`src/lib.rs:780-783`: "A
span path is discarded outright only when its admissible bound is under it,
which is sound") depends on `span_score_bound` still bounding the new
objective.

That is why **no form in the taxonomy below is a one-hunk change**, and it is
the structural reason the family is expensive rather than merely unhelpful.

## 2. The taxonomy, and what each member costs

Every form here is general: a function of the seven axis values and, where the
form needs a scale, of the pool's own per-axis statistics. None reads a phrase,
a word, a substring, a dictionary or the environment.

| id | form | mechanism it is meant to buy | measured result |
|---|---|---|---|
| OFF-A | linear sum (control) | — | the production objective; deficit −0.095790597 at `--top 50` |
| OFF-B | within-target z-normalisation | a zero-variance axis wastes its mass; give the mass to the axes that discriminate | deficit **worse**, −0.214515960; rank 9865 → 10234; 38 of 50 churned |
| OFF-C | within-target rank percentile | distribution-free version of OFF-B | deficit **worse**, −0.247432033; rank → 10037; 42 of 50 churned |
| OFF-D | weighted geometric mean (soft-min) | a sum lets one strong axis buy off a collapsed one; a product does not | deficit **worse**, −0.136211849; rank → 10017; **leaves `[0,1]`** (see §4) |
| OFF-E | tiered: primary varying axis gates, rest break ties | a strong content-word signal should dominate a saturated one | deficit **worse**, −0.104362241; rank → 12264; 39 of 50 churned |
| OFF-F | saturated-axis masking: drop dead mass, renormalise live axes | the minimal, isolated test of the saturated-axis hypothesis | deficit −0.083296172, i.e. **0.0125 better**; **rank unmoved at 9865**; 0 of 50 churned |
| OFF-G | hard gate on `WORD_NOVELTY` (full-resegmentation requirement) | the literal reading of "a saturated axis masking a real signal" | deficit worse in the raw band, but the only form that moves the rank: 9865 → **7380** of 17 922 |

OFF-F is the decisive row. It is the *only* form that isolates the item's
hypothesis with nothing else changed, and it is worth **0.0125 of a 0.0958
deficit and exactly zero ranks.**

## 3. Why the family is closed — the domination argument

The reason is visible in one table and does not depend on any of the seven
forms. On the canonical target's visible band, the requested wording stands as
follows (pool of 17 914 at `--top 50`, the wording injected through the
ordinary `build` so that the production scorer will discuss it):

| axis | weight | requested wording | band min | band max | band sd | below band min? |
|---|---|---|---|---|---|---|
| `SIMILARITY` | 0.25 | 0.720120 | 0.772178 | 0.874692 | 0.030449 | **YES** |
| `NOVELTY` | 0.15 | 0.666667 | 1.000000 | 1.000000 | **0.000000** | **YES** |
| `WORD_NOVELTY` | 0.15 | 1.000000 | 1.000000 | 1.000000 | **0.000000** | no (parity) |
| `FAMILIARITY` | 0.10 | 0.398713 | 0.526273 | 0.829959 | 0.078086 | **YES** |
| `RHYTHM` | 0.30 | 1.000000 | 1.000000 | 1.000000 | **0.000000** | no (parity) |
| `SHAPE` | 0.05 | 1.000000 | 0.897458 | 1.000000 | 0.015712 | no (above) |
| `CLOSED_CLASS` | −0.15 | 0.000000 | 0.000000 | 0.040000 | 0.014664 | no (at min) |

**The candidate is dominated by every member of the band on six of seven axes.**
It is strictly below the band's *minimum* on all three axes that carry any
variance in the band, and at exact parity on the three that are constant.

Two consequences, and they are the whole verdict:

1. **Every form that is non-decreasing in each axis value — which is every
   member of the taxonomy above, and includes any per-axis rescaling,
   renormalisation, z-scoring, rank substitution or mass redistribution —
   preserves domination.** Such a form cannot lift the candidate above any band
   member. That is why OFF-B, OFF-C, OFF-D and OFF-E all make it *worse*: they
   sharpen the axes on which it is already last. To beat the band, a form has
   to be **non-monotone** in some axis, i.e. to prefer *less* of something. The
   only two forms here that are non-monotone in effect are the two gates
   (OFF-E, OFF-G), and both are measured worse or, at best, rank-moving-without
   arriving.
2. **A form cannot create a pool member.** The emitted set is fixed before any
   score is computed (§1, last row is a cut, not a source). The requested
   wording is absent from the emitted pool at `--top 10`, `--top 20` **and**
   `--top 50`, and absent under all seven forms.

The exact decomposition of the −0.095790597 deficit against the rank-49 cutoff
member `it justice too day mm`:

```
NOVELTY       -0.050000000   (52.2 % of the gap)   <- the saturated axis
FAMILIARITY   -0.036947704   (38.6 %)
SIMILARITY    -0.019969988   (20.8 %)
CLOSED_CLASS  +0.006000000
SHAPE         +0.005127095
WORD_NOVELTY  +0.000000000   <- exactly zero: saturated at 1.0 on both sides
RHYTHM        +0.000000000   <- exactly zero: saturated at 1.0 on both sides
             ------------
              -0.095790597
```

So the item's hypothesis is *true and small*: `WORD_NOVELTY` and `RHYTHM` hold
**0.45 of the 1.00 weight mass and contribute exactly nothing** to this
comparison. But OFF-F, which deletes precisely that dead mass and renormalises,
converts the 0.45 into 0.0125 of movement and 0 ranks, because renormalising
lifts the band by almost exactly as much as it lifts the candidate — the band is,
by construction, the top of the same distribution on those axes too.

## 4. The `[0,1]` contract, and the one form that breaks it outright

`Clue::score` is documented in `[0,1]`. The objective's maximum is exactly `1.0`
(six weights summing to 1.00, penalty non-positive), so the linear form respects
it with no headroom to spare.

**OFF-D / `MADGAB_AGG_FORM=1` (the weighted geometric mean) violates it, and is
disqualified on admissibility.** Its arithmetic: the `CLOSED_CLASS` term carries
weight `-0.15` and contributes the *complement* `1 - closed_penalty`. When a
clue is nothing but function words, `closed_penalty = 1.0`, so the complement
is `0.0`, and the log is undefined; the harness clamps it to `1e-6`, giving

```
ln term = -0.15 * ln(1e-6) = -0.15 * (-13.815511) = +2.072327
score   = exp(2.072327)   = 7.9481
```

as an upper bound, with all six positive axes at 1.0. Observed rank-49 cutoff
on `recognize speech` under that form: **5.813541148**, and band widths of
1.79708 on `I love you` and 4.79984 on `recognize speech`. A negative weight
under a logarithm is the whole problem: it turns a penalty into a bonus and
then amplifies it. The form is not repairable by a constant without becoming a
different form, and it is disqualified rather than tuned. Per the 21:30Z
coordinator pass, no further budget was spent on it.

**Any re-baseline of `approximate_output_is_locked` would have to be argued
side by side, never silently.** Nothing in this front proposes one: the only
form that reaches the `[0,1]` question at all is disqualified on it, and the two
surviving forms are inert or catastrophic. A re-baseline is a coordinator
decision about a test this front may not touch, and no measurement here argues
for one.

## 5. What an implementation-ready form would have had to look like

Recorded so the next pass does not have to re-derive it, and so the bar is
explicit. A form that *could* move this candidate would need all four of:

1. **A non-monotone preference.** Domination (§3) says no monotone re-expression
   can help. The form must prefer *less* of some axis, which means justifying
   why less is better — a claim about puzzle quality, not about acoustic fit.
2. **Pool-level statistics available at `metrics()` time.** `metrics()` is
   called from inside the beam, the DP and the final scorer, before the pool
   exists. OFF-B, OFF-C and OFF-F all need per-target per-axis moments or
   percentiles. That is a **two-pass redesign** of `generate_approximate`, not a
   hunk — and it is a redesign of the *search*, not of the scorer.
3. **A co-changed bound.** `span_score_bound`, `partial_span_score` and
   `complete_span_score` must be re-derived for the new form, or the
   `span_bound`/`incumbent` cut at `src/lib.rs:1194` silently truncates the
   search. Front `9a41d3` is the precedent for what happens when this is
   skipped: a scoring change that left the bound behind, two green guards red.
4. **A green `wreck a nice beach` at a margin that is not being spent.** The
   guard sits **+0.001267657** inside the band on `recognize speech` at
   `--top 50`. Any form with in-band variance is a coin flip on it. OFF-D and
   OFF-F are exactly that coin flip, and both lose: raw rank 27 → 1069 and
   2895, visible position 28 → absent.

Item 4 is the cheapest and most decisive of the four, and it is the reason this
family should be considered closed rather than under-explored: the guard has
**0.0013 of margin and the band is 0.0046 wide**. Any re-expression that
reshuffles the band is, on this target, more likely to lose the guard than to
gain the milestone.

## 6. Scope of this document

* Everything here is measured at **`--top 50`**, release builds, on
  `madgab-aggform` at `8ad2d53`, whose `src/` is byte-identical to `3d520c0`.
* `src/` was **not edited on this branch**. All instrumentation lives in
  `git archive` extractions under `/workspace`, reproduced as patches on the
  measurement-only branch `scratch/9f1c05-aggform`, which must never be merged.
* Per the 21:30Z coordinator pass, any *implementation* front that ever revisits
  this must branch from **`1dab2d0` or later** (which carries `PUNCH` as
  `6a93c2a`), never from `8ad2d53`, whose base predates that merge.
* `cargo fmt`, `cargo clippy` and doctests do not exist on this host and are not
  claimed.
