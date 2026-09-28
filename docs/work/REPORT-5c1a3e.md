# REPORT-5c1a3e — the per-span retention policy has no effect on the canonical case-2 pool absence

**Item:** [w-5c1a3e](items/w-5c1a3e.md) · **Branch:** `madgab-retain-5c1a3e` ·
**Base:** `4fee16f` (post-milestone-acceptance) · **Front:** `agent-5c1a3e`,
`/workspace/madgab-retain-5c1a3e`

## Verdict

**HOLD** on the policy. No production line changes. One regression test is added; the
production region of `src/approx.rs` is byte-identical to the base.

The item's own premise is **refuted by measurement**: the canonical case-2 clue is *not*
lost in the per-span shortlist keep. Every word of the reading survives its target span's
retention, and the reading is still absent from the pool. The surface that loses it is
`prune_partials` (`src/lib.rs:3454`), which is a **different front's** — `w-3c5b18` /
`agent-3c5b18`, branch `madgab-thresh-3c5b18` — and was not touched here.

## 1. What the item assumed, and what is true

The item was opened on a durable-sounding fact: the word `hid` "is *pruned from the retained
shortlist for its target span*, so it never enters the pool and no downstream score, weight,
or selection change can recover the clue."

**It is retained.** Measured from the production path — not from a re-implementation — by
tracing `src/approx.rs:208-265` on the canonical case-2 target, whose normalised IPA is
`ɪtsdʒʌstəstʊpədɡeɪm` (n = 19).

The reading's span for that word is `start=13`, `consumed=2`, i.e. `/əd/` — the ə and d of
*stupid*. That span's ordered match list has **374** entries against `MATCHES_PER_SPAN = 256`,
so the three-stage keep is what decides it:

| quantity | value |
|---|---|
| cost | **0.3695** |
| cost band | **2** (`floor(0.3695 / 0.5 × 4)`) |
| band sizes | band0 **3**, band1 **7**, band2 **107**, band3 **257** |
| rank in the cost-ordered list | **ord = 104** |
| in-band rarity rank | **46** |
| rarity | 12601 |
| `CHEAP_KEEP` | 96 |
| `BAND_KEEP` | 40 |
| outcome | misses the cheap head, misses its band, **kept by the cost-ordered fill at `fill#104`** |

So the honest answer to "which stage and rank drops it" is: **no stage drops it.** It misses
the cheap head (104 > 96) and misses its cost band (rarity rank 46 > `BAND_KEEP` 40), and is
then **rescued by the third stage** — the cost-ordered fill, which runs because the first two
have not spent the budget (the stages overlap, so 96 + 4×40 unique words is under 256) and
therefore still has 152 slots of headroom when it reaches ordered index 104.

All five words of the reading are present in their production shortlists:

| word | span | cost | retained at |
|---|---|---|---|
| `hits` | `start=0 consumed=3` `/ɪts/` | 0.2000 | `cheap#8` |
| `justice` | `start=3 consumed=7` `/dʒʌstəs/` | 0.0000 | ord_in_span 0 |
| `dupe` | `start=10 consumed=3` `/tup/` | 0.1500 | ord_in_span 3 |
| `hid` | `start=13 consumed=2` `/əd/` | 0.3695 | `fill#104` |
| `came` | `start=14 consumed=5` `/dɡeɪm/` | 0.4642 | ord_in_span 34 |

Total substitution cost **1.1837** against `total_budget = 1.5`, so the reading is
budget-admissible as well as retention-admissible.

## 2. Where the reading is actually lost

Instrumenting the enumeration, the alignment is **never enumerated**. It does not reach the
pool at any beam width:

| `beam_width` | pool size | clue in pool? | deepest prefix reached |
|---|---|---|---|
| 64 (shipped) | 20616 | no | 0 words — even the one-word prefix is not in the surviving 64 at `p=3` |
| 512 | 20749 | no | — |
| 4096 | 20575 | no | 2 words, at `p=10`; the 3-word prefix is not in the top 4096 at `p=13` (the best depth-1 candidate there sits at combined-score rank 2681 of 4878) |

At the shipped `top_n = 200000` — which returns the whole enumeration, not a selected prefix —
the pool is **20616** clues and the reading is absent, its best leading run being 2 of 5. The
word `hid` *is* in the pool, twice, in other alignments (pool ranks 4488 and 13795), so this is
a statement about the combination and not about the vocabulary.

**Localisation: `prune_partials` (`src/lib.rs:3454`).** It ranks candidates by
`Metrics::combined` — the objective — and the reading's prefixes sit thousands of ranks below
the cut at every position. That is an objective-side surface, not a retention surface.

## 3. The retention sweep (item criterion 2)

Fourteen variants of the policy, each measured on the same two targets. `hid` retention is
read from the production keep; pool size and pool membership from the whole enumeration;
the case-1 figures from the printed top-50 at `--approximate --top 50`.

| variant (`MPS/CHEAP/BANDS/BAND/key`) | `hid` | pool size | case-2 clue in pool? | case-1 printed 50 |
|---|---|---|---|---|
| **256/96/4/40 rarity (shipped)** | `fill#104` | 20616 | no | identical |
| 256/64/4/48 rarity | `band2#46` | 20584 | no | identical |
| 256/128/4/32 rarity | `cheap#104` | 20362 | no | identical |
| 256/32/4/56 rarity | `band2#46` | 20574 | no | identical |
| 256/192/4/16 rarity | `cheap#104` | 20671 | no | identical |
| 256/0/4/64 rarity (no cheap stage) | `band2#46` | 20562 | no | identical |
| 256/96/8/20 rarity | `band5#11` | 20618 | no | identical |
| 256/96/2/80 rarity | `fill#104` | 20563 | no | identical |
| 256/96/4/40 **cost-first** | `fill#104` | 20613 | no | identical |
| 256/96/8/20 **cost-first** | `fill#104` | 20639 | no | identical |
| 384/144/4/60 rarity | retained † | 20584 | no | identical |
| 512/192/4/80 rarity | retained † | 20565 | no | identical |
| 128/48/4/20 rarity (tighter) | `fill#104` | 20259 | no | identical |
| 1024/384/4/160 rarity (wide) | retained † | 20602 | no | identical |

† at `MATCHES_PER_SPAN` ≥ 384 the `/əd/` span's 374 matches no longer overrun the budget, so
the keep does not run for that span at all and the word is trivially retained at
`ord_in_span = 104`. The prune-stage trace is silent there because it only fires on a span that
is actually pruned; retention was confirmed separately through the placement trace.

Three readings, all of them the reason this front closes as a negative:

1. **`hid` is retained in 14 of 14 variants**, and in every variant it is retained *by a
   different stage* depending only on where the constants put the boundary. It is not a
   fragile survivor of one lucky overlap; the fill's headroom is 152 slots and it needs 104.
2. **The case-2 clue reaches the pool in 0 of 14 variants.** No retention setting can create a
   beam slot for an alignment the objective ranks thousands of positions below the cut.
3. **The case-1 printed top-50 is byte-identical in all 14 variants** (md5 of the printed list
   `d8136a142ac2` throughout). Pool size moves within 20259–20671, about ±1%, and none of that
   movement reaches the visible list. The printed top-50 mean `SIMILARITY` is therefore
   unchanged at the value `REPORT-1a4e8d` records for this target, `0.848330`, and the printed
   mean score reproduces at `0.9199228013`.

A sweep in which nothing moves is not a sweep that found a safe point. It is a sweep that
shows the axis is not on the path.

## 4. What is delivered, and its non-vacuity

`HOLD` means no policy change. But the measurement established a general property that is
currently unguarded, and that property is the thing the front got wrong, so it is pinned by one
test: `approx::tests::a_span_over_budget_keeps_candidates_neither_the_head_nor_its_band_keeps`.

It states, over a synthetic span of 450 matches against a 256 budget, that a span over budget
keeps candidates **neither** the cheap head **nor** its own cost band's ranking would keep, and
that the two rescues are *different stages*:

* a **cheap but rare** candidate is past the cheap head and past its band's rarity-first keep,
  and survives only because the **cost-ordered fill** completes the budget;
* an **expensive but common** candidate is at the very end of the span's cost order, so no
  cost-ordered fill could reach it, and survives only because its **cost band's** rarity-first
  keep admits it.

**Non-vacuity, by mutation, on the delivered file:**

| mutation | result |
|---|---|
| cost-ordered fill removed | **FAILS** — "only 96 of 150 no-cost candidates survived … the cost-ordered fill is what completes it" |
| cost-band stage removed | **FAILS** — "no expensive-but-common candidate survived, so the cost-band stage contributes nothing the cheap head and the fill do not already cover" |
| cheap head removed | *passes* — see the honest limit below |
| none (as delivered) | passes |

**Stated limit:** the test does **not** detect removal of the cheap head, and cannot as
written, because with a budget larger than the head the fill alone reproduces the head's
choices. The cheap head is therefore not independently pinned by this test. That is recorded
rather than papered over; making the head load-bearing would need a fixture where the head's
*rarity* ordering disagrees with the fill's *cost* ordering within one band, which is a
different fixture and was not built for a HOLD delivery.

The test names no canonical word, phrase, sentence or rank. Its fixture is invented
non-words (`zqS…`, `zqM…`, `zqL…`) over the invented segment string `abc`, and the assertions
are counts over cost groups, not ranks. The first draft of the measurement harness named the
alignment under measurement and `no_phrase_specific_hard_coding_in_src_web_or_examples` failed
on it; **the allowlist was not edited and the finding was not re-pinned**, the harness was
rewritten instead.

## 5. Scope, and non-contention

* **The objective weight vector was not touched** (`axes::*` unchanged) and **the score
  function was not touched** (`Metrics::combined` unchanged). The sweep varied only
  `MATCHES_PER_SPAN`, `CHEAP_KEEP`, `COST_BANDS`, `BAND_KEEP` and the band ranking key, and
  every one of those variations was measurement scaffolding that is **not** in the delivered
  diff.
* **`prune_partials` was not edited**, and the discard-threshold front now on it
  (`w-3c5b18`) is not started here.
* `/workspace/madgab-e086cc` and `/workspace/madgab-pairscore-3f6a21` were not read, entered or
  run.
* **No fence was re-pinned.** The pre-existing base red
  `approximate_finds_classic_madgab_resegmentation` is untouched and still red, as it must be.

## 6. Validation actually run

On this branch, with a dedicated `CARGO_TARGET_DIR` and `--test-threads=1`.

| suite | base `4fee16f` | delivered |
|---|---|---|
| `--lib` | 75 passed, 12 ignored | 76 passed, 12 ignored (the one added test) |
| `no_phrase_hard_coding` | 9/9 | 9/9 |
| `emit_coverage` | 7/7 | 7/7 |
| `approx_determinism` | 4/4 | 4/4 |
| `exact_determinism` | 1/1 | 1/1 |
| `corpus_integration` | 12 passed / **1 failed** | 12 passed / **1 failed** |

The single failure is the known pre-existing base red,
`approximate_finds_classic_madgab_resegmentation` — the canonical case-2 test this front was
opened to fix. It is red at the base and red in the delivery, unchanged, and its literals were
not touched.

**Not run, and not claimed:** `cargo fmt` and `cargo clippy` **cannot run on this host** — no
`rustup`, no `rustfmt`/`clippy` component (`docs/environment-notes.md`). They are unverifiable
here and are not reported as satisfied. `cargo test --doc` is likewise unavailable (no
`rustdoc`). `emit_spread` **does not exist as a test name on this tree**; the emission-spread
counters are asserted from the `--lib` suite, which is green, and that is what is reported
rather than a suite name that would have been convenient.

## 7. Successor rule, in one line

Per-span retention is closed as a surface for the canonical case 2 — **the reading is retained
and is lost to `prune_partials`' objective-ranked cut** — so the next front must be named on
the enumeration's discard threshold, and by the coordinator's account that surface is
`w-3c5b18`'s; there is nothing left on the retention side to open.
