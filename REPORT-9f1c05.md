# VERDICT: AGGREGATION IS NOT THE GATE — and it is not a milestone route at all; the wording is never enumerated, so no aggregation form can surface it.

# REPORT-9f1c05 — is the objective's *aggregation form* what makes a 0.0958 deficit unreachable?

Front `9f1c05`, branch `madgab-aggform`, work item
[docs/work/items/w-9f1c05.md](docs/work/items/w-9f1c05.md).
**Base for every number in this report: `8ad2d53`**, whose `src/` tree is
**byte-identical to `3d520c0`** (`git diff 3d520c0..8ad2d53 -- src/` is empty).
Release builds, default approximate path, **`--top 50` unless stated**, and
`top_n` is stated with every count and rank. Nothing under `src/` or `tests/`
was edited on this branch. No test was edited, relaxed, re-baselined, skipped or
`#[ignore]`d. Nothing was merged; `post-milestone-acceptance` and `main` were not
touched. `src/` on this branch differs from `3d520c0` in nothing at all.

**This branch's committed delta is `docs/work/aggregation-form.md` and this
report, and nothing else.** The measurement corpus and the harness patches are
on the separate measurement-only branch `scratch/9f1c05-aggform`, which must
never be merged into `post-milestone-acceptance`.

---

## 0. The finding that decides the question, and the one that decides its scope

**The requested wording is not in the candidate pool. Not at rank 9865 — it has
no rank.** Measured on this base, release binary, default approximate path, with
the crate's own shipped trace hook and nothing added:

```
$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
    target/release/madgab --approximate --top 10 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=13471
$ ... --top 20  ->  missing candidates=14561
$ ... --top 50  ->  missing candidates=17913
```

`missing candidates=N` is the crate's own wording for "not among the N
deduplicated, score-ordered pool members". I also searched the captured
`--top 50` pool for the phrase and for its `phrase_signature`: **0 members at
all three `top_n`.** The contrast is the guard, which *is* ranked:

```
$ MADGAB_TRACE_PHRASES="wreck a nice beach" ... --approximate --top 50 "recognize speech"
MADGAB_TRACE raw phrase="wreck a nice beach" rank=27 score=0.918313383
```

**This corrects a premise the work item inherited.** The item, and every
document that descends from `w-558697`, states the wording "**is in the pool**:
9865 of 17 913 at `--top 50` on `3d520c0`". It is not. `558697`'s own report
says how that number was produced: a second gate, `MADGAB_558697_INJECT`, that
"forces one *environment-named* index tuple for one *environment-named*
segmentation into the pool so the **production** scorer and the **production**
`select_diverse` can be asked what they make of a wording **the search never
emitted**". The 9865 is the rank of an **injected** candidate. `missing
candidates=17913` on the same binary, in the same run configuration, is the
uninjected truth. I reproduced 9865 exactly (below), so I am not disputing the
measurement — I am identifying what was measured.

**Independently corroborated after `PUNCH` landed.** `git show
2860b57:REPORT-d4a90b.md` measures the same wording on the post-`PUNCH` head
`f5b9eaa` and finds it absent from all 17 827 pool candidates at `--top 50` and
all 13 498 at CLI-default `top_n` 10, i.e. "rank is undefined, not large", while
also showing it is *budget-admissible* (per-word 0.2/0.0/0.15/0.3695/0.4, total
1.11951981416578961 ≤ 1.5, one complete lattice alignment) and scoreable at
0.79990129077367222. So the loss is in enumeration, on both sides of the
`PUNCH` merge, and my pre-`PUNCH` and their post-`PUNCH` measurements agree.

**Consequence, stated as scope rather than worked around.** **No aggregation
form, no axis weight and no re-ranking of any kind can surface a wording the
enumeration never offers to the pool.** The objective is a *ranking* function: it
orders candidates that already exist and has no power to bring one into being.
For this target nothing is enumerated, so the question of how to combine the axes
never arises, and any report presenting a form as a milestone route would be
wrong. The aggregation-form question can only speak to what happens *after* a
wording is enumerated. The
objective is a *ranking* function; it has no power to bring a candidate into
existence. For this target nothing is enumerated, so no aggregation form can
surface it, and any report that presented one as a milestone route would be
wrong. My forms are therefore evaluated as a **general approximate-quality**
question, and as the thing that would matter *once enumeration is fixed* — which
is exactly the framing below.

**No form here is proposed as a milestone route.** The family is closed, on the
numbers in §3.

---

## 1. Method: forms evaluated OFFLINE against a captured pool

Per the brief, no form was evaluated by re-running the search. Two harnesses,
both `git archive` extractions of `8ad2d53` under `/workspace`, never a
checkout, never committed to any branch, reproduced as patches on
`scratch/9f1c05-aggform`
(`measurements/9f1c05/harness-dump.patch`, `harness-apply.patch`):

**Dump arm** — one presence-tested hunk in `Partial::into_clue`, emitting one
TSV line per retained clue: phrase, the seven axis values, the production
score, the syllable total and the phrase signature. `select_diverse`,
`coverage_tuples`, `sweep_index`, `EMIT_PROFILE_MAX_DEEP`,
`EMIT_PROFILE_RESERVE`, the share cap and `STRUCTURE_FLOOR` are **not touched**.
The variable's presence is tested; no code branches on its value except to read
a list this harness itself produced.

**Apply arm** — re-expresses only *how the seven axis values are combined*, at
the one point the objective is evaluated, selectable by a small integer over a
fixed enumerated list of forms. No word, phrase, target, substring, dictionary
entry or corpus lookup is named in it. **Control validation: with the form
switch unset the arm's visible stdout is byte-identical to the pristine
production binary** on both canonical targets (`cmp`, verified), and its pool
sizes and guard ranks reproduce `558697`/`9a41d3` to the digit.

**How the pool was captured, exactly:**

* release binary, `cargo build --release`, under `/workspace` (`/tmp` is
  `noexec`);
* **default approximate path** — `--approximate`, no other mode flag, no
  instrumented harness in the measured arm, no re-budgeting;
* **`top_n` = 50** (`--top 50`) for every pool capture, every pool count, every
  rank and every margin in this report. The pool is a strong function of
  `top_n` (13 471 / 14 561 / 17 913 at `--top 10` / `--top 20` / `--top 50` on
  this base for the canonical target), so a rank here is not a rank there and no
  number below mixes `top_n` values;
* 16 targets: the two canonical ones plus 14 ordinary sentences, none of which
  is an acceptance phrase except where stated.

**Two independent validations that the capture is the production pool:**

1. The seven pool counts `3e7b04` published at `--top 50` on `3d520c0` are
   reproduced **exactly, 7 of 7** (17913, 15908, 16169, 17091, 18474, 15609,
   19168).
2. The `axes::`-weighted sum of each dumped axis vector reproduces the
   production score to **7.5e-13** on all 16 targets. So the offline forms are
   evaluated against the production objective, not an approximation of it.

**Noise floor.** The visible-list floor on this head is **exactly 0** (front
`3e7b04`: 3 replicates × 13 targets byte-identical; 250 runs, pool span 0). I
re-took it for this front: 3 replicates × 3 forms × 2 targets of the apply arm,
**all 18 byte-identical** (§6). Every replicate range in this report therefore
reads `X-X`, which is the honest way to say "no run-to-run variation at all
here". The parent's ±25 tuples / ±0.06 pp is **not** inherited.

**Reading a number that looks like a rank.** The offline forms are scored on a
pool into which the requested wording was **injected** through the ordinary
`build`, so that the production scorer will discuss a wording the search never
emitted. The control arm of that injection reproduces `558697` **to the digit**:
rank 9865, score 0.819901291, cutoff 0.915691888. Those are ranks *of an
injected candidate*, at `--top 50`, and are labelled as such everywhere.

---

## 2. The seven forms, offline, on the canonical target

Pool of **17 914** at `--top 50` (17 913 emitted + 1 injected). Probe ranks and
margins are **of the injected candidate**. `churn` = how many of the raw top 50
leave the band. `reseg` = full resegmentations in the band under `c81e55`'s
verbatim rule. `salad` = band members whose closed-class share exceeds 1/3.

| id | form | rank | score | cutoff | margin | band width | churn | reseg | salad |
|---|---|---|---|---|---|---|---|---|---|
| OFF-A | linear sum (**control**) | 9865 | 0.819901291 | 0.915691888 | **−0.095790597** | 0.009495 | 0 | 0 | 0 |
| OFF-B | within-target z-normalisation | 10234 | 0.756911384 | 0.971427344 | −0.214515960 | 0.028573 | 38 | 0 | 0 |
| OFF-C | within-target rank percentile | 10037 | 0.281613912 | 0.529045944 | −0.247432033 | 0.052730 | 42 | 1 | 0 |
| OFF-D | weighted geometric mean | 10017 | 0.783632720 | 0.919844569 | −0.136211849 | 0.022086 | 13 | 0 | 7 |
| OFF-E | tiered: primary varying axis gates | 12264 | 0.753799003 | 0.858161244 | −0.104362241 | 0.004644 | 39 | 0 | 0 |
| OFF-F | saturated-axis masking, renormalise live | **9865** | 0.712957644 | 0.796253816 | **−0.083296172** | 0.008256 | **0** | 0 | 0 |
| OFF-G | hard gate on `WORD_NOVELTY` | **7380** | 0.420994589 | 0.957845944 | −0.536851355 | 0.004747 | 0 | 0 | 0 |

**Every form except OFF-F makes the deficit worse. OFF-F — the one form that
isolates the item's hypothesis with nothing else changed — is worth 0.0125 of a
0.0958 deficit and exactly zero ranks.**

**OFF-F is the decisive row and it is the whole hypothesis.** It deletes the
weight mass of every axis that is constant on the target and renormalises the
survivors — nothing else. Result: −0.095790597 → −0.083296172, rank 9865 →
**9865**, churn **0 of 50**. The reason the 0.45 of dead mass buys so little is
that renormalising lifts the band by nearly as much as it lifts the candidate:
the band is the top of the same distribution on those axes.

OFF-G's raw-band margin (−0.5369) is worse than OFF-A's while its **rank**
improves to 7380 of 17 922, because the gate re-scales the band more than the
candidate; the rank is the honest number and it is still 2 480 short of the band
end. §3's end-to-end table prices OFF-G properly, through the real
`select_diverse`.

---

## 3. End-to-end pricing, through the production `select_diverse`

Three forms are implementable inside `metrics()` without pool-level statistics,
so three are priced end to end. `F1`=OFF-D, `F2`=OFF-G, `F3`= the extreme
(lexicographic) member of the OFF-E family. **Form 1 is excluded from the sweep
per the 21:30Z coordinator pass**, on the admissibility disqualification in §5.

Visible churn is out of the real 50 returned by `select_diverse`, as a
spelling-signature identity. Pool delta is against the control arm's pool at the
same `top_n`.

| target | pool @ 50 | form | churn/50 | kept | full reseg/50 | salad/50 | pool Δ |
|---|---|---|---|---|---|---|---|
| **It's just a stupid game** (canonical) | 17913 | F0 control | 0 | 50 | **0** | 0 | 0 |
| | | F2 gate on WNOV | **0** | 50 | **0** | 0 | +9 |
| | | F3 lexicographic | **50** | 0 | 0 | 16 | −112 |
| **recognize speech** (canonical) | 15908 | F0 control | 0 | 50 | 44 | 0 | 0 |
| | | F2 gate on WNOV | **0** | 50 | **44** | 0 | −272 |
| | | F3 lexicographic | **50** | 0 | 25 | 0 | −81 |
| I love you (the output-lock target) | 9338 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | +1 |
| | | F3 | 47 | 3 | 30 | 10 | −785 |
| the cat sat on the mat | 16255 | F0 control | 0 | 50 | 29 | 0 | 0 |
| | | F2 | **0** | 50 | 29 | 0 | +134 |
| | | F3 | 50 | 0 | 0 | **50** | +430 |
| when the rain finally stopped | 15609 | F0 control | 0 | 50 | 31 | 0 | 0 |
| | | F2 | **0** | 50 | 31 | 0 | −711 |
| | | F3 | 50 | 0 | 0 | 26 | −59 |
| an old man in a big hat | 13041 | F0 control | 0 | 50 | 35 | 0 | 0 |
| | | F2 | **0** | 50 | 35 | 0 | +68 |
| | | F3 | 50 | 0 | 0 | 14 | −419 |
| what are you going to do | 14548 | F0 control | 0 | 50 | 49 | 0 | 0 |
| | | F2 | **0** | 50 | 49 | 0 | −269 |
| | | F3 | 50 | 0 | 0 | 40 | +392 |
| there is no way to know | 20119 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | +18 |
| | | F3 | 50 | 0 | 0 | 30 | −19 |
| she had a lot of money | 13341 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | +37 |
| | | F3 | 50 | 0 | 0 | **50** | +14 |
| a whole lot of trouble | 16169 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | −8 |
| | | F3 | 50 | 0 | 0 | 13 | −14 |
| he was a big fat man | 19168 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | +21 |
| | | F3 | 50 | 0 | 0 | 19 | +59 |
| my brother has a red car | 15649 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | +83 |
| | | F3 | 50 | 0 | 0 | 26 | −615 |
| put it back on the shelf | 15642 | F0 control | 0 | 50 | 50 | 0 | 0 |
| | | F2 | **0** | 50 | 50 | 0 | +1 |
| | | F3 | 50 | 0 | 0 | 43 | −255 |
| the other seven | 20023 | F0 control | 0 | 50 | 15 | 0 | 0 |
| | | F2 | **0** | 50 | **15** | 0 | −22 |
| | | F3 | 50 | 0 | 0 | 16 | −100 |
| the quick brown fox jumps | 17091 | F0 control | 0 | 50 | 36 | 0 | 0 |
| | | F2 | **0** | 50 | 36 | 0 | −31 |
| | | F3 | 50 | 0 | 0 | 0 | −189 |
| the mad gab for kids | 18474 | F0 control | 0 | 50 | 0 | 0 | 0 |
| | | F2 | **0** | 50 | 0 | 0 | −234 |
| | | F3 | 50 | 0 | 0 | 13 | +405 |

Full table with all 48 rows: `git show
scratch/9f1c05-aggform:measurements/9f1c05/price-summary.tsv`.

**Three readings.**

* **The control arm reproduces the published census, 12 of 13 rows, exactly**
  (canonical 0, recognize speech 44, cat-sat 29, rain 31, old man 35, what are
  you 49, and 50 on the six saturated ordinary targets), including
  `the other seven` = **15**, the row `558697` identified as a defect in
  `c81e55`'s artifacts. That is an independent third reproduction of the
  disputed number and it confirms `558697`.
* **F2 is inert on the visible list: churn 0 of 50 on 16 of 16 targets, and the
  full-resegmentation count identical to control on 16 of 16.** It perturbs the
  pool (−711 to +134 candidates) and the rank order without moving one visible
  phrase. It is safe and it is worth nothing.
* **F3 is disqualifying on quality, not on taste:** 50 of 50 visible churn on 15
  of 16 targets, full resegmentations driven to **0** on 11 of 16 (including
  both canonical targets' 44 → 25 and 0 → 0), and function-word salads up to
  **50 of 50**. A form that turns every visible list into a determiner salad on
  half the targets is not a general quality improvement under any reading.

## 4. `wreck a nice beach` — preserved, and it is the cheapest constraint here

Green at both boundaries on this base, and **not traded away**, because no form
is proposed for landing. Measured, `--top 50`, release binary:

| arm | raw rank | score | rank-49 cutoff | margin | visible position |
|---|---|---|---|---|---|
| production / F0 control | **27** | 0.918313383 | 0.917045726 | **+0.001267657** | **28** |
| F2 (the only guard-safe form) | **27** | 0.959156691 | 0.958522863 | **+0.000633828** | **28** |
| F1 (geometric; `[0,1]`-disqualified) | 1069 | 0.927742877 | 5.813541148 | −4.885798271 | **absent** |
| F3 (lexicographic) | 2895 | 0.863121635 | 0.974929744 | −0.111808109 | **absent** |

`wreck a nice beach` is produced for `recognize speech` at raw rank 27 and
visible position 28 on the control arm, matching `3e7b04`'s independent
measurement exactly, and `approximate_finds_recognize_speech_resegmentation` is
**green** (§7). F2 keeps it at rank 27 and visible position 28 — the same
position — but **halves its margin**, 0.001267657 → 0.000633828. F1 and F3 lose
it from the visible list outright.

Two facts here matter more than the form table. First, **the guard has 0.0013 of
margin and the band is 0.0046 wide**: any form that reshuffles the band is on
this target more likely to lose the guard than to gain anything. Second, at the
CLI's own default `top_n` 10 the guard is **not in the visible 10 on any arm,
including production** — it is visible position 28 at `--top 50`. I state that
as a fact about the base, not a regression; it is also the correction
`w-2f7a10`'s 20:45Z pass made to the "CLI defaults, visible rank 28" phrasing,
which is a `--top 50` observation.

## 5. The `Clue::score ∈ [0,1]` contract, and the one form that breaks it

**Disqualified: OFF-D / apply-arm `F1`, the weighted geometric mean.**

The objective's maximum is exactly 1.0 (six weights summing to 1.00,
`CLOSED_CLASS` signed at −0.15, `closed_penalty = (closed/words)² ≥ 0`), so the
linear form respects `[0,1]` with no headroom. The geometric form does not,
because `CLOSED_CLASS` carries a **negative** weight and must contribute the
complement `1 − closed_penalty`; when a clue is nothing but function words that
complement is `0.0`, the logarithm is undefined, and the harness's `1e-6` clamp
gives

```
ln term = (-0.15) * ln(1e-6) = (-0.15) * (-13.815511) = +2.072327
score   = exp(+2.072327) = 7.9481        <- upper bound, six positive axes at 1.0
```

**Observed: rank-49 cutoff 5.813541148** on `recognize speech`, and band widths
of **1.79708** on `I love you` and **4.79984** on `recognize speech**.** A
negative weight under a logarithm converts a penalty into an unbounded bonus.
The form is not repairable by a constant without becoming a different form, so
it is **disqualified on admissibility rather than tuned**, and per the 21:30Z
coordinator pass no further budget was spent on it. The two forms that survive
`[0,1]` (OFF-B at max score 1.000000 by construction of its affine map; OFF-F)
are both §2's failures.

**No other form breaks `[0,1]`:** max scores over the canonical pool are
OFF-A 0.925187, OFF-C 0.581776, OFF-E 0.862806, OFF-F 0.804510, OFF-G 0.962593,
all inside the contract. OFF-B's map is affine onto the pool's own `[min,max]`,
so it is in `[0,1]` by construction.

### `approximate_output_is_locked` — side by side, never silently

The lock is an exact list of `"%.6f phrase"` strings for `I love you` at
`top_n` 10. Side by side with F2, the only guard-safe form, at the lock's own
configuration:

| # | locked (and F0 control, byte-identical) | F2 |
|---|---|---|
| 1 | `0.938335 isle a view` | `0.969… isle a view` |
| 2 | `0.937604 aisle a view` | `0.969… aisle a view` |
| 3 | `0.937462 i.'s a view` | `0.969… i.'s a view` |
| 4 | `0.936762 eye a view` | `0.968… eye a view` |
| 5 | `0.931877 how ill view` | `0.966… how ill view` |
| 6 | `0.931877 now ill view` | `0.966… now ill view` |
| 7 | `0.930994 isle of new` | `0.965… isle of new` |
| 8 | `0.930262 aisle of new` | `0.965… aisle of new` |
| 9 | `0.930120 i.'s of new` | `0.965… i.'s of new` |
| 10 | `0.929948 yeah ill view` | `0.965… yeah ill view` |

**The phrases and their order are identical; every printed score changes.**
Because the lock asserts score *strings*, F2 would redden
`approximate_output_is_locked` while leaving the user-visible output identical in
membership and order. That is the sharpest possible statement of the cost of any
rescaling, and it is why no re-baseline is proposed here: **this front does not
argue for one, and no measurement in this report supports one.** A re-baseline
of that test is a coordinator decision about a test this front may not touch.

## 6. Wall clock and determinism

Whole-process wall clock including corpus load, **7 replicates per arm, arms
interleaved within each replicate**, release, `--approximate --top 50`:

| target | arm | min | median | max | n | vs control |
|---|---|---|---|---|---|---|
| It's just a stupid game | F0 control | 2.28 | 2.33 | 2.50 | 7 | — |
| | F1 (disqualified) | 2.17 | 2.23 | 2.66 | 7 | −4.1 % |
| | F2 | 2.07 | 2.28 | 2.41 | 7 | −2.0 % |
| | F3 | 2.31 | 2.50 | 2.73 | 7 | +7.4 % |
| recognize speech | F0 control | 1.93 | 2.01 | 2.42 | 14 | — |
| | F1 (disqualified) | 1.89 | 1.97 | 2.42 | 14 | −2.0 % |
| | F2 | 1.93 | 2.03 | 2.37 | 14 | +1.4 % |
| | F3 | 1.90 | 2.05 | 2.56 | 14 | +2.3 % |

Every delta is inside the within-arm spread (0.30–0.50 s) on both targets, and
the signs are not consistent across targets. **No form has a demonstrable wall
clock cost.** The `recognize speech` arm has n=14 because an earlier pass
truncated its own log and was re-run; both passes' samples are in the file.

**Determinism.** 3 replicates × 3 forms × 2 targets of the apply arm's visible
list, `--top 50`, **all 18 byte-identical**; and the control arm's visible
output is byte-identical to the pristine production binary on both canonical
targets. So the form changes are deterministic on this head, and the noise floor
for every comparison in this report is 0.

## 7. Suites, with a `git archive 3d520c0` base arm

Both arms are clean `git archive` extractions under `/workspace`, no probe.
Branch arm = `madgab-aggform` at `8ad2d53`; base arm = `3d520c0`.

| suite | branch `8ad2d53` | base `3d520c0` | verdict |
|---|---|---|---|
| `--lib` | **ok — 53 passed, 0 failed, 0 ignored** | ok — 53 passed, 0 failed, 0 ignored | no change |
| `--test corpus_integration` | **FAILED — 10 passed, 1 failed** | FAILED — 10 passed, 1 failed | **pre-existing red** |
| `--test no_phrase_hard_coding` | **ok — 6 passed, 0 failed** | ok — 6 passed, 0 failed | no change |
| `--test approx_determinism` | **ok — 4 passed, 0 failed** | ok — 4 passed, 0 failed | no change |
| `--test exact_determinism` | **ok — 1 passed, 0 failed** | ok — 1 passed, 0 failed | no change |

**The one red is pre-existing and byte-identical on both arms.**
`approximate_finds_classic_madgab_resegmentation`, at
`tests/corpus_integration.rs:136`, same message, same twelve wordings on branch
and base:

```
canonical clue missing from top 50; got: ["it justice too bad aim",
 "it justice too pad aim", "it justice too bad same", "it justice too peg aim",
 "it justice too pad same", "it justice too bad name", "it justice too bed aim",
 "it justice too pig aim", "eat justice too bad aim", "it justice too pad name",
 "it justice too pug aim", "it justice too bad came"]
```

`approximate_finds_recognize_speech_resegmentation` is **green** on both arms.
`approximate_output_is_locked` is **green** on both arms, unmodified. No test
was edited, relaxed, re-baselined, skipped or `#[ignore]`d; `grep -rn "#\[ignore"
tests/ src/` returns nothing. `cargo fmt`, `cargo clippy` and doctests do not
exist on this host and are not claimed.

## 8. Why the family is closed — the domination argument

The reason no form works does not depend on which of the seven you pick. On the
canonical target's visible band, the requested wording stands as follows (pool
of 17 914 at `--top 50`, wording injected so the production scorer will discuss
it):

| axis | weight | requested wording | band min | band max | band sd | below band min? |
|---|---|---|---|---|---|---|
| `SIMILARITY` | 0.25 | 0.720120 | 0.772178 | 0.874692 | 0.030449 | **YES** |
| `NOVELTY` | 0.15 | 0.666667 | 1.000000 | 1.000000 | **0.000000** | **YES** |
| `WORD_NOVELTY` | 0.15 | 1.000000 | 1.000000 | 1.000000 | **0.000000** | parity |
| `FAMILIARITY` | 0.10 | 0.398713 | 0.526273 | 0.829959 | 0.078086 | **YES** |
| `RHYTHM` | 0.30 | 1.000000 | 1.000000 | 1.000000 | **0.000000** | parity |
| `SHAPE` | 0.05 | 1.000000 | 0.897458 | 1.000000 | 0.015712 | above |
| `CLOSED_CLASS` | −0.15 | 0.000000 | 0.000000 | 0.040000 | 0.014664 | at min |

**The candidate is dominated by every member of the band on six of seven axes** —
strictly below the band's *minimum* on all three axes that carry any variance in
the band, and at exact parity on the three that are constant.

1. **Any form non-decreasing in each axis value preserves domination**, and that
   class includes every per-axis rescaling, renormalisation, z-scoring, rank
   substitution and mass redistribution. Such a form cannot lift the candidate
   above any band member. This is why OFF-B, OFF-C, OFF-D and OFF-E all make it
   *worse*: they sharpen the axes on which it is already last. To beat the band
   a form must be **non-monotone** — prefer *less* of something — and the only
   two here that are, OFF-E and OFF-G, are measured worse or rank-moving without
   arriving.
2. **A form cannot create a pool member.** The emitted set is fixed before any
   score is computed. `metrics().combined` is also the search's `incumbent`
   threshold (`src/lib.rs:785-800`), consumed by the span-path cut at `:1194` —
   so a form *can* move pool membership, but only by perturbing a cut whose
   soundness argument depends on `span_score_bound` still bounding the new
   objective. That is why no form here is a one-hunk change: `metrics` (`:2407`),
   `partial_span_score` (`:3014`), `complete_span_score` (`:3038`) and
   `span_score_bound` (`:1583`) all shadow the objective. Front `9a41d3` is the
   precedent for skipping the bound: two green guards went red.

The exact decomposition of the −0.095790597 deficit against the rank-49 cutoff
member `it justice too day mm` (`--top 50`):

```
NOVELTY       -0.050000000   52.2 % of the gap   <- the saturated axis
FAMILIARITY   -0.036947704   38.6 %
SIMILARITY    -0.019969988   20.8 %
CLOSED_CLASS  +0.006000000
SHAPE         +0.005127095
WORD_NOVELTY  +0.000000000   <- exactly zero: saturated at 1.0 on both sides
RHYTHM        +0.000000000   <- exactly zero: saturated at 1.0 on both sides
             -----------
              -0.095790597
```

So the item's hypothesis is **true and small**: `WORD_NOVELTY` and `RHYTHM` hold
**0.45 of the 1.00 weight mass and contribute exactly nothing** to this
comparison. OFF-F converts that 0.45 into 0.0125 of movement and 0 ranks.
`d4a90b` measures the same structure post-`PUNCH` and finds `NOVELTY` dominant
at 0.050000000 of 0.097851316 (51%), with `PUNCH` costing a further 0.020000000
asymmetrically — so the saturation finding is durable on both sides of the merge,
and the `PUNCH` axis is a genuinely asymmetric term the form family does not
address either.

## 9. What I did **not** measure

* **Anything on the post-`PUNCH` head.** Every number here is base `8ad2d53`
  (`src/` identical to `3d520c0`). `PUNCH` (`6a93c2a`) is **not** in this
  measurement; re-deriving from this base silently reverts it. Per the 22:15Z
  coordinator pass, any future front on this area must branch from `1dab2d0` or
  later.
* **The interaction of any form with `PUNCH`.** PUNCH is a seventh additive
  axis, so a form that rescales or renormalises the six pre-existing axes
  interacts with it, and that interaction is unmeasured. I flag it rather than
  guess: OFF-F's whole mechanism (drop dead mass) would treat `PUNCH`'s mass
  differently on a target where `PUNCH` is saturated, which `d4a90b` says it is
  on the canonical target.
* **Forms requiring pool-level statistics, end to end.** OFF-B, OFF-C and OFF-F
  need per-target per-axis moments or percentiles, which `metrics()` cannot see:
  it is called from inside the beam and the DP, before the pool exists. They are
  priced **offline only**, against the captured pool, which is exactly the
  method the brief asked for. Their end-to-end pool and churn consequences are
  **unmeasured**, and measuring them needs a two-pass redesign of
  `generate_approximate`.
* **Any form co-changed with `span_score_bound`.** Every end-to-end number here
  changes `combined` alone, so the admissibility of the `:1194` cut is broken by
  construction for the non-affine forms. The per-form pool deltas in §3 are the
  measurable signature of that; a properly co-changed implementation is a
  different and larger piece of work that I did not attempt.
* **A repaired or clamped geometric form.** Disqualified on the arithmetic in §5
  rather than tuned, which is the right disposition for a form whose repair is
  indistinguishable from inventing a new one.
* **`select_diverse`'s own policy.** Not touched, not re-implemented, not tuned.
  Its response to a reordered band is observed only through the visible lists in
  §3, which is where the guards are decided.
* **Targets beyond the 16 above**; `top_n` other than 10/20/50 (used only for the
  three-point pool-size series in §0, all on the production binary);
  `wasm`/`web` build; `examples/`; benchmarks; the `--release --doc` target.
* **The 4,79984-style band widths of OFF-D on ordinary targets** beyond the two
  quoted — I stopped measuring OFF-D on the coordinator's instruction.

## 10. Verdict, restated, and the next action

**VERDICT: AGGREGATION IS NOT THE GATE.**

1. **It is not a milestone route, and no form in this report is proposed as
   one. No aggregation form, weight change or re-ranking can surface a wording
   the enumeration never offers to the pool.** `w-d4a90b` measured the second
   canonical example absent from **all 17 827 candidates at `--top 50`** on the
   post-`PUNCH` head (`git show 2860b57:REPORT-d4a90b.md`, branch
   `origin/madgab-postpunch-measure`), and this front measures it absent from all
   13 471 / 14 561 / 17 913 on the pre-`PUNCH` base (§0). The objective ranks
   candidates that already exist; it has no power to create one. `d4a90b` shows the loss is in
   the search's retention/enumeration, and its implementation front
   (`1c3e77`/`madgab-enum-1c3e77`) and measurement arm (`5d2a91`) are the live
   work there. I did not touch their branches or worktrees.
2. **The family is closed on numbers, not on taste.** Seven general forms
   offline, three end to end. Best case OFF-F: 0.0125 of a 0.0958 deficit, **0
   ranks**, 0 of 50 churn. OFF-G: rank 7380 of 17 922 at `--top 50`, still
   0.0479 short. The structural reason is domination (§8): the candidate is
   below the band minimum on every axis with in-band variance, so no monotone
   re-expression — which is every normalising, rescaling, rank-substituting or
   mass-redistributing form — can help.
3. **Disqualified, with arithmetic:** **OFF-D / apply-arm F1**, the weighted
   geometric mean, on the `Clue::score ∈ [0,1]` contract — `exp((−0.15)·ln 1e-6)
   = 7.9481` as an upper bound, observed rank-49 cutoff **5.813541148**, band
   widths 1.79708 and 4.79984. **OFF-E / apply-arm F3**, the tiered and
   lexicographic members, on quality — 50 of 50 visible churn on 15 of 16
   targets, full resegmentations driven to 0 on 11 of 16, salads to 50 of 50,
   and the green guard lost from the visible list (raw rank 27 → 2895).
   **OFF-B and OFF-C**, z-normalisation and rank percentile, on the measurement
   itself — they *widen* the deficit to −0.214515960 and −0.247432033 and churn
   38 and 42 of 50.
4. **The guard was not traded away.** `wreck a nice beach` is produced for
   `recognize speech` at raw rank 27 / visible position 28 with margin
   **+0.001267657** on this base, green in the suite, and I propose no change
   that could move it.
5. **The general quality use of this measurement is a post-enumeration ranking
   question, not a milestone route.** Once enumeration is fixed, the form family
   is worth re-measuring for its own sake: OFF-B and OFF-C are the principled
   ways to stop a saturated axis from absorbing 0.45 of the weight mass, and
   their cost is now quantified rather than guessed (38 and 42 of 50 visible
   churn, deficit widened to −0.214515960 and −0.247432033). That is a general
   approximate-quality question for a post-enumeration front. It must be
   branched from `1dab2d0` or later, re-measured against `PUNCH`, and re-priced
   against the guard's 0.0013 of margin. It is **not** a route to
   `approximate_finds_classic_madgab_resegmentation`.
6. **No re-baseline is proposed.** `approximate_output_is_locked` is green and
   unmodified on both arms; §5 shows side by side what a rescaling would cost
   it, and nothing here argues for one.

**Recommended next action: record the refutation and stop manufacturing fronts
in this area.** The aggregation-form question is answered, and answered
negatively, with a structural reason that does not depend on the form chosen.
The live question is enumeration, and it is already owned
(`madgab-enum-1c3e77` from head `33cb6cb`, with `5d2a91` measuring
independently). Two durable corrections pass to whoever routes next:

* the "9865 of 17 913" figure is the rank of an **injected** candidate, and the
  wording is **absent from the pool at every `top_n` measured** — pre-`PUNCH`
  here (13 471 / 14 561 / 17 913) and post-`PUNCH` in `d4a90b` (13 498 / 17 827).
  Any future document that calls it "in the pool" should be corrected;
* `WORD_NOVELTY` and `RHYTHM` hold 0.45 of the weight mass and contribute
  **exactly zero** to this comparison. That is a real, durable, general
  observation about the objective — and OFF-F is the measurement of what acting
  on it would cost: nothing moves.

Any implementation front that ever revisits this must branch from **`1dab2d0` or
later**, never from `8ad2d53`, whose base predates the `PUNCH` merge.

MERGE RECOMMENDATION: MERGE (documentation only: this report and
`docs/work/aggregation-form.md`; `src/` and `tests/` untouched)
