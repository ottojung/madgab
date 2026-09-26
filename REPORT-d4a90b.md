VERDICT: on `f5b9eaa` the second canonical example is still not produced — `hits justice dupe hid came` is absent from the entire candidate pool (missing from all 17827 pool candidates at `--top 50`, and from all 13498 at CLI default `top_n` 10), so it has **no rank at all**; its reconstructed score is `0.79990129077367222`, a **score delta of `0.09785131568084526`** below the worst visible proposal at `--top 50` (`0.89775260645451749`) and **`0.10026892900503859`** below the worst visible at CLI default `top_n` 10 (`0.90017021977871081`).

# w-d4a90b — re-measure the second canonical example on the post-PUNCH head

- Work item: `docs/work/items/w-d4a90b.md` (front-matter `state: working`, owner `agent-d4a90b`).
- Branch: `madgab-postpunch-measure`, HEAD and base `f5b9eaaea1d3992d1588f556c0da18eb237a38bd` (forked from `post-milestone-acceptance`).
- Target: `It's just a stupid game` -> `hits justice dupe hid came`.
- Target IPA: `/ˈɪtsdʒˈʌstəstˈupədɡˈeɪm/`, normalized `ɪtsdʒʌstəstupədɡeɪm`, `n=19` chars, target inner boundaries `[3, 8, 9, 15, 19]`, target syllables `6`.
- **Measurement only.** Nothing under `src/` or `tests/` was edited; no `axes::*` constant, retention constant or selection policy was touched. Fence check at the end of this report.
- This front does not claim the second canonical example is "reached" and proposes no fix. **The finding below is a negative result about where the loss happens, and it was not what the brief expected to find.**

## Headline, and the one thing that changed the shape of the answer

Every stale number in [w-4b1e07], [w-558697] and [w-7b41d2] treats the second example as a *scoring* problem: a proposal that exists somewhere and sits a measurable distance below the band. On the post-PUNCH head that framing is wrong for this target. The library's own shipped trace hook says the phrase is not merely out of the top 50 — it is not in the pool at all:

```
$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" target/release/madgab --approximate "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=13498
MADGAB_TRACE raw_cutoff rank=9 score=0.900786673 phrase="it justice too bed aim"

$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" target/release/madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17827
MADGAB_TRACE raw_cutoff rank=49 score=0.898008919 phrase="it justice too bed same"
```

`missing candidates=N` is the crate's own wording for "this phrase is not among the N deduplicated, score-ordered pool members". So **rank is undefined, not large.** There is no rank-51, no rank-5000: the candidate is not enumerated. The score delta in the VERDICT is therefore a *counterfactual* distance — what the phrase would have to gain to reach the band — and it is a useful number for pricing a proposal, but it is not a rank displacement.

Independently, the phrase **is** admissible under the shipped budgets, which localises the loss. Reconstructed through the crate's own `Partial::extend_fuzzy` path against the same fuzzy lattice `generate_approximate` builds (per-word budget `0.5`, total budget `1.5`):

```
path[0].word_costs  hits=0.20000000000000001/hɪts justice=0.00000000000000000/dʒʌstəs
                   dupe=0.14999999999999999/dup hid=0.36951981416578961/hɪd
                   came=0.40000000000000002/keɪm
path[0].agg        sub_cost_total=1.11951981416578961  (total budget 1.5)
                   every per-word cost <= 0.5            (per-word budget 0.5)
complete_alignments 1
```

Every word is inside the per-word budget and the total is inside the total budget, with exactly one complete alignment of this word sequence through the lattice. So the phrase is reachable by the scoring model and admissible by the budget, and is nevertheless dropped before the pool is formed. **The loss is in the search's retention/enumeration, not in the objective.** This front does not open a hypothesis about which retention step, and it explicitly did not widen any budget to make the target appear (see "Temptations declined").

## Q1 — release-binary approximate output, default `top_n` and `--top 50`

Both from the release binary on this branch; full listings in `measurements/d4a90b/q1-default-topn.txt` and `q1-top50.txt`.

| | CLI default (`top_n` 10) | `--top 50` |
|---|---|---|
| command | `target/release/madgab --approximate "It's just a stupid game"` | `target/release/madgab --approximate --top 50 "It's just a stupid game"` |
| pool count (candidates `select_diverse` chose from) | `13498` | `17827` |
| proposals printed | `10` | `50` |
| worst **visible** proposal (post-diversity) | rank 10, `0.900`, `it said thus test oop games` | rank 50, `0.898`, `it said thus tas too dame` |
| **top-band cutoff**, pre-diversity (`MADGAB_TRACE raw_cutoff`) | rank 9, `0.900786673`, `it justice too bed aim` | rank 49, `0.898008919`, `it justice too bed same` |
| is `hits justice dupe hid came` enumerated? | **no** | **no** |

Two cutoffs are quoted because they differ and the distinction is load-bearing: the raw pool's `top_n`-th member is *not* the worst thing the user sees, because `select_diverse` reorders and rebalances after the pool is scored. At `--top 50` the raw cutoff is `0.898008919` while the worst visible is `0.89775260645451749`; at `top_n` 10 the raw cutoff is `0.900786673` against a worst visible of `0.90017021977871081`.

The CLI's own banner gives the timings (Q5) and confirms the mode: note the CLI default mode is `Exact`, so `--approximate` is required; the per-word/total budgets default to `0.5`/`1.5`, which are the same values `SearchMode::approximate()` gives the tests, so the CLI and the test harness are directly comparable.

## Q2 — score, worst visible score, delta, rank

**The target is not enumerated in either configuration, so its rank is undefined in both.** Quoting delta and rank separately, as asked:

| quantity | CLI default (`top_n` 10) | `--top 50` |
|---|---|---|
| target score | `0.79990129077367222` | `0.79990129077367222` |
| **score delta** vs worst visible | `0.10026892900503859` | `0.09785131568084526` |
| **rank** | *undefined — absent from all `13498` pool candidates* | *undefined — absent from all `17827` pool candidates* |

Supporting deltas, all from `measurements/d4a90b/q2-q3-arithmetic.txt`:

```
canon                       0.79990129077367222
delta vs worst visible @10  0.10026892900503859
delta vs worst visible @50  0.09785131568084526
delta vs rank1 @50          0.10528552125503088
delta vs raw cutoff @10     0.10088538222632781
delta vs raw cutoff @50     0.09810762822632779
```

The target's score is a reconstruction, not a printed number, and the reconstruction is verified two ways: the probe drives the crate's own `Partial::extend_fuzzy` and then the crate's own `Partial::into_clue`, and the resulting `into_clue` score reproduces the printed score of a *known* enumerated proposal to the last bit —

```
it justice too bad aim    combined=0.90518681202870310  into_clue_score=0.90518681202870310   (printed: " 1. [0.905]")
it said thus tas too dame combined=0.89775260645451749  into_clue_score=0.89775260645451749   (printed: "50. [0.898]")
hits justice dupe hid came combined=0.79990129077367222  into_clue_score=0.79990129077367222   (never printed)
```

Closest enumerated relatives of the requested wording, for orientation (not substitutes, and not a fix):

- `it justice too bad came` — rank **18** of 50, `0.900`. The `-hit / -came / justice` skeleton is present and competitive.
- `it justice too bed aim` — the pre-diversity cutoff at `top_n` 10.
- No `hid`, `dupe` or `hits` proposal is enumerated at either breadth.

## Q3 — per-axis decomposition of the delta, on the current head

Weights as compiled on `f5b9eaa` (`src/lib.rs` `mod axes`, quoted by the probe):

```
SIMILARITY=0.25 NOVELTY=0.15 WORD_NOVELTY=0.15 FAMILIARITY=0.1
RHYTHM=0.3 SHAPE=0.05 CLOSED_CLASS=-0.15 PUNCH=0.1
```

**Do the six weights still sum to exactly 1.00?** Yes, exactly, in `f64`:

```
weights_sum_six            1.00000000000000000
weights_sum_six_plus_punch 1.10000000000000009
```

The six pre-PUNCH additive weights (`SIMILARITY + NOVELTY + WORD_NOVELTY + FAMILIARITY + RHYTHM + SHAPE`) sum to `1.00000000000000000`; adding `PUNCH` gives `1.10000000000000009` (the `...009` tail is binary representation of `1.1`, not a drift). `CLOSED_CLASS` is the only signed weight, `-0.15`, and is applied as `-0.15 * closed_penalty`; `PUNCH` is applied as `0.10 * (punch - 1.0)`, so both are non-positive and the objective's maximum is unchanged at `1.0`. This is consistent with the reasoning recorded in the PUNCH commit: the term is shifted by its own maximum to preserve the `Clue::score` bound.

Raw axis values, target vs the worst visible proposal at `--top 50`:

| axis (raw) | `hits justice dupe hid came` | `it said thus tas too dame` | difference |
|---|---|---|---|
| `similarity` | `0.720120046` | `0.761433348` | `+0.041313301` |
| `novelty` | `0.666666667` | `1.000000000` | `+0.333333333` |
| `word_novelty` | `1.000000000` | `1.000000000` | `0.000000000` |
| `familiarity` | `0.398712792` | `0.623942695` | `+0.225229904` |
| `rhythm` | `1.000000000` | `1.000000000` | `0.000000000` |
| `shape` | `1.000000000` | `0.983333333` | `-0.016666667` |
| `closed_penalty` | `0.000000000` | `0.027777778` | `+0.027777778` |
| `punch` | `0.800000000` | `1.000000000` | `+0.200000000` |

Weighted contributions to the score delta (target -> worst visible at `--top 50`):

| axis | weight | target term | worst-visible term | contribution to delta |
|---|---|---|---|---|
| `NOVELTY` | `0.15` | `0.100000000` | `0.150000000` | **`+0.050000000`** |
| `FAMILIARITY` | `0.10` | `0.039871279` | `0.062394270` | `+0.022522990` |
| `PUNCH` | `0.10` | `-0.020000000` | `0.000000000` | `+0.020000000` |
| `SIMILARITY` | `0.25` | `0.180030012` | `0.190358337` | `+0.010328325` |
| `CLOSED_CLASS` | `-0.15` | `-0.000000000` | `-0.004166667` | `-0.004166667` |
| `SHAPE` | `0.05` | `0.050000000` | `0.049166667` | `-0.000833333` |
| `WORD_NOVELTY` | `0.15` | `0.150000000` | `0.150000000` | `0.000000000` |
| `RHYTHM` | `0.30` | `0.300000000` | `0.300000000` | `0.000000000` |
| **total** | | `0.799901291` | `0.897752606` | **`+0.097851316`** |

The eight per-axis deltas sum to `0.097851316`, matching the measured end-to-end delta `0.09785131568084526` exactly. The decomposition is closed.

**Does any axis now move the target rather than the band?** Yes, and this is the answer most worth recording, because it is not the answer the pre-PUNCH decomposition gives:

- `RHYTHM` (`0.30`, the largest weight) and `WORD_NOVELTY` (`0.15`) are **saturated at `1.0` for both** the target and the worst visible proposal. They contribute `0.000000000` to the delta. Neither axis distinguishes them; the band's advantage is not rhythm.
- **`NOVELTY` is now the dominant term** at `+0.050000000` of `0.097851316`, i.e. **51% of the whole deficit**. This is the axis that moves the band *around* the target rather than moving the target: the target's own resegmentation `hits|justice|dupe|hid|came` lands on `cuts=[3, 10, 13, 15, 19]` against target boundaries `[3, 8, 9, 15, 19]`, reproducing only 2 of the 4 inner boundaries, so `novelty = 0.666666667`; the worst visible proposal's `cuts=[2, 4, 7, 10, 13, 19]` reproduces none, so it is rewarded the full `1.0`. The axis is penalising the canonical example for being *too faithful* to the target's boundaries, which is a structural property of the requested wording and not something breadth can buy.
- **`PUNCH` moves the target.** The new axis costs the canonical example a full `0.020000000` (raw `punch = 0.800000000`: `punch_count=4` of `5` words, because `dupe` is not monosyllabic, and `PUNCH` is applied as `0.10 * (0.8 - 1.0)`). The worst visible proposal is entirely monosyllabic (`6` of `6`, `punch = 1.0`) and therefore pays nothing. So on the pre-PUNCH objective the canonical example's deficit would have been `0.077851316`; **PUNCH alone widens the measured gap by `0.020000000`**, about 20% of it, and it widens it asymmetrically — the band is already at PUNCH's ceiling and cannot gain, only the target can lose. This is the concrete post-PUNCH change to the second example's distance, and it is a *scoring* effect; it is not, however, the reason the example is missing (see the headline).
- `FAMILIARITY` contributes `+0.022522990` (`hits`, `hid` and `dupe` are rarer than the band's words). Unlike the axes above, this one is not saturated and does move the target, but it is the ordinary kind of lexical cost and is not specific to this example.
- `SIMILARITY` is close (`+0.010328325`, raw `0.720120046` vs `0.761433348`): phonetically the two are close, as the `sub_cost_total` of `1.11951981416578961` against `1.5` also says.

## Q4 — regression status, quoted exactly

No test was edited. Command, on this branch, base `f5b9eaa`:

```
$ cargo test --release --test corpus_integration
```

From `measurements/d4a90b/q4-corpus-integration.log`, quoted:

```
running 11 tests
test approximate_finds_recognize_speech_resegmentation ... ok
test approximate_finds_classic_madgab_resegmentation ... FAILED

failures:

---- approximate_finds_classic_madgab_resegmentation stdout ----

thread 'approximate_finds_classic_madgab_resegmentation' (573393) panicked at tests/corpus_integration.rs:136:5:
canonical clue missing from top 50; got: ["it justice too bad aim", "it said thus test oop dame", "it justice too pad aim", "it said thus test oop day", "it justice too bad same", "it said thus test oop gave", "it justice too peg aim", "it justice too pad same", "it justice too bad name", "it justice too bed aim", "it justice too pig aim", "eat justice too bad aim"]

failures:
    approximate_finds_classic_madgab_resegmentation

test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.82s
```

- `approximate_finds_recognize_speech_resegmentation` — **green** (`ok`). The *first* canonical example is still found.
- `approximate_finds_classic_madgab_resegmentation` — **still red** (`FAILED`), 10 passed / 1 failed.

Note that the harness's failure message says "missing from top 50", but the `MADGAB_TRACE` evidence above is sharper than the test's own wording: the phrase is missing from the pool of `17827`, not from the visible 50. The test is a top-50 membership test; the underlying absence is deeper than the test claims.

## Q5 — wall clock

`measurements/d4a90b/q5-wallclock.txt`, three runs each on this machine, whole-process wall clock via `date +%s%N` around the release binary, with the binary's own `corpus loaded / search` split:

| config | run 1 | run 2 | run 3 |
|---|---|---|---|
| default (`top_n` 10) | `2664 ms` (load `484ms`, search `1959ms`) | `2848 ms` (load `572ms`, search `2072ms`) | `2514 ms` (load `513ms`, search `1808ms`) |
| `--top 50` | `2357 ms` (load `455ms`, search `1696ms`) | `2124 ms` (load `473ms`, search `1464ms`) | `2215 ms` (load `471ms`, search `1553ms`) |

Representative single-run figures for pricing a breadth proposal: **~2.5 s** end to end at CLI default, **~2.2 s** at `--top 50`, of which roughly `0.45-0.57 s` is corpus load and `1.5-2.1 s` is search. **`--top 50` is not measurably more expensive than the default here** (it is if anything slightly cheaper, and the run-to-run spread of ~0.3-0.7 s exceeds any breadth effect). Breadth alone is therefore not the lever: raising `top_n` from 10 to 50 changed the pool from `13498` to `17827` candidates and did not move the cost.

## Method, reproducibility, and fences

**Headline numbers (Q1, Q2 pool/cutoff, Q4, Q5) come only from the shipped release binary and the existing test harness**, so a reader with only this branch can reproduce them:

```
cargo build --release
target/release/madgab --approximate "It's just a stupid game"
target/release/madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE_PHRASES="hits justice dupe hid came" target/release/madgab --approximate "It's just a stupid game"
MADGAB_TRACE_PHRASES="hits justice dupe hid came" target/release/madgab --approximate --top 50 "It's just a stupid game"
cargo test --release --test corpus_integration
```

**Q3's per-axis decomposition is the only thing that needed a probe**, because `mod axes` and the `Partial` aggregates are private. The probe lives **outside the repository**, at `/tmp/opencode/d4a90b-probe`, and was built with `CARGO_TARGET_DIR=/workspace/madgab-d4a90b/target/d4a90b-probe` (inside the git-ignored `/target/`; `/tmp/opencode` is mounted `noexec`, hence the redirect). It works by textual `include!` of the branch's own `src/lib.rs` into a module, so the appended code sits in the same module as the internals and reads the real `axes` constants and the real `Partial` fields. Consequences, stated plainly:

- The probe compiled a **copy** of `src/lib.rs` at `/tmp/opencode/d4a90b-probe/src/lib-shim.rs`, because an inner `//!` doc comment is illegal mid-module. The copy differs from `src/lib.rs` **only** in the five leading `//!` header lines rewritten to `//`; no code line differs, and `adjacency.rs` / `approx.rs` / `lexical.rs` are symlinks to the branch's real, unmodified files. `diff` of the shim against the original is the 5-line header rewrite and nothing else.
- **No file in this worktree's `src/` or `tests/` was written to, by the probe or by anything else.** The probe ran against the branch's own code, not against an instrumented copy of the tree.
- The probe is a *measurement* device, not a proposed change. Its correctness rests on the exact `into_clue` score agreement quoted in Q2, which ties it to the numbers the shipped binary prints.

**Temptations declined, recorded as instructed.** Two places where a phrase-adjacent shortcut was available and was not taken:

1. The per-word and total budgets (`0.5` / `1.5`) are what make the canonical example admissible at all (§ headline). Raising them, or raising `beam_width`, or raising the completion-retention multiplier, would very likely have made it appear in the top 50 and turned this report green. That is precisely the phrase-specific widening the fence forbids, so no budget, retention or selection constant was touched, and the missing-from-pool result is reported as the finding.
2. The reconstruction could have been "helped" by hand-picking an alignment. It was not: the lattice admits exactly one complete alignment of this word sequence (`complete_alignments 1`), and the score is whatever that alignment scores.

No scoring hypothesis is opened, no constant is proposed, and the second example's *status* is unchanged: the milestone is still unmet. What is new is that the gap is now localised — the example is admissible and scoreable at `0.79990129077367222`, `0.09785131568084526` below the visible band, and it never enters the pool, so the binding constraint is enumeration, not the objective. Routing that observation belongs to [w-2f7a10], [w-4b1e07] and [w-a02d28]; the aggregation-form design remains [w-9f1c05]'s.

## Artifacts

All on `madgab-postpunch-measure`:

| file | contents |
|---|---|
| `measurements/d4a90b/build-release.log` | `cargo build --release` |
| `measurements/d4a90b/q1-default-topn.txt` | full CLI default `top_n` 10 output (Q1) |
| `measurements/d4a90b/q1-top50.txt` | full `--top 50` output (Q1) |
| `measurements/d4a90b/q1-trace-rawrank.txt` | `MADGAB_TRACE_PHRASES` pool-rank and raw-cutoff evidence (Q1, Q2) |
| `measurements/d4a90b/q2-q3-arithmetic.txt` | every delta and per-axis contribution, computed in `f64` (Q2, Q3) |
| `measurements/d4a90b/q3-probe-top50.txt` | probe: pool, canonical presence, rank 1 and worst-visible scores (Q2) |
| `measurements/d4a90b/q3-probe-top10.txt` | same at CLI default `top_n` 10 (Q2) |
| `measurements/d4a90b/q3-probe-canonical-and-worst.txt` | probe: per-axis raws, aggregates, terms, word costs, `into_clue` scores (Q3) |
| `measurements/d4a90b/q4-corpus-integration.log` | `cargo test --release --test corpus_integration`, unedited (Q4) |
| `measurements/d4a90b/q5-wallclock.txt` | three timed runs per configuration (Q5) |

## Fence check

```
$ git diff --stat post-milestone-acceptance...madgab-postpunch-measure -- src tests
$ git status --short
?? measurements/
```

`git diff --stat post-milestone-acceptance...madgab-postpunch-measure -- src tests` is **empty**: no file under `src/` or `tests/` differs between the milestone branch and this branch, and the only additions on this branch are `measurements/d4a90b/` and this report. No `axes::*` constant, retention constant or selection policy was changed. This branch is **not** merged into `post-milestone-acceptance`; `main` was not touched.
