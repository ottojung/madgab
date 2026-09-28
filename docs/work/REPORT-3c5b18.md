---
work_item: w-3c5b18
report: true
id: REPORT-3c5b18
state: done
priority: high
owner: front-3c5b18 (agent-3c5b18)
updated: 2026-09-28T00:45:00Z
branch: madgab-thresh-3c5b18
worktree: /workspace/madgab-thresh-3c5b18
base: post-milestone-acceptance at 10c4a29
verdict: HOLD
---

# REPORT-3c5b18 — the beam's admission gate is a rank horizon, not a value threshold, and the horizon cannot be opened inside the budget

**Item:** [w-3c5b18](items/w-3c5b18.md) · **Branch:** `madgab-thresh-3c5b18` ·
**Base:** `10c4a29` (post-milestone-acceptance) · **Front:** `agent-3c5b18`,
`/workspace/madgab-thresh-3c5b18`

## Verdict

**HOLD.** No production line changes. `src/lib.rs` gains **one test and zero
production lines**. The canonical case-2 reading is still absent from the pool,
and the reason is now a measurement rather than a localisation: the discard that
loses it is **not a value threshold at all**, so "price the value threshold" is
not a well-posed question on this surface, and the quantity that *is* the
threshold — the portfolio's per-order **rank horizon** — costs about `7r` beam
slots to move by `r` ranks, which the budget cannot pay for the whole chain.

**Case 1 is untouched and is the guard.** `wreck a nice beach` for
`recognize speech` is at **pool rank 27** on the delivered tree, re-measured
after the revert (§5). This front did not move it, by construction and by
measurement.

---

## 1. The localisation (completion criterion 1)

**File and function:** `src/lib.rs`, `prune_partials` (`fn` at `:3443`; the item
quoted `:3454`, which is the same function on this base). Called from three
places: the per-position expansion prune (`:1020`, `k = beam_width`), the
incremental prune that fires when a position's vector exceeds `2 * keep` (`:1064`,
`k = keep`), and the completed-hypothesis prune (`:1083`,
`k = top_n * 128` clamped to `[1024, 8192]`).

**The quantity compared — this is the finding.** `prune_partials` has **no
scalar value threshold in its keep path.** The dedup stage compares two
candidates' combined scores against *each other*; the retention stage then admits
by two structural rules and no score at all:

1. **cell protection** — up to two representatives per
   `(word_count, acoustic cost band, rhythm band)` cell, capped at `k / 2`, ordered
   by combined;
2. a **seven-way round-robin** over the axis rankings (combined, novelty,
   familiarity, acoustic, lexical novelty, rhythm, content), taking the
   `rank`-th element of every order in turn until the budget `k` is spent.

So the effective gate is a **rank horizon of about `k / 7` in each of the seven
orders** — a candidate is admitted if it is near the top of *at least one* order,
and discarded otherwise *however good its combined score is*. **There is no value
floor to price, and no value floor could reproduce the keep rule** (§4 pins this
as a test).

**The value at the loss point, and the keep-side distribution.** Measured from
the production path with an env-gated trace inside `prune_partials` (inert when
unset, reverted before delivery; harness in §7). For the canonical case-2 target
`It's just a stupid game`, IPA `ɪtsdʒʌstəstʊpədɡeɪm`, `n = 19`:

| beam | position | partial | `k` | deduped | combined | cost | rank_comb / nov / fam / **aco** / lex / rhy / con | best order rank | horizon `k/7` | keep-side combined floor → ceiling | keep-side cost ceiling | admitted |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **64** (shipped) | `p=3` | `hits` (1 word) | 64 | 129 | 0.484409 | 0.200000 | 4 / 40 / 11 / **22** / 40 / 40 / 37 | **22** | ≈ 9.1 | 0.250000 → 0.499099 | 0.260056 | **no** |
| **1024** | `p=10` | `hits justice` (2 words) | 1024 | 2049 | 0.552318 | 0.200000 | 1837 / 779 / 547 / **142** / 776 / 1969 / 671 | **142** | ≈ 146.3 | 0.380558 → 0.933283 | 1.500000 | **no** |
| **2048** | `p=13` | `hits justice dupe` (3 words) | 2048 | 4096 | 0.554878 | 0.350000 | 3234 / 2405 / 2853 / **421** / 1150 / 3299 / 868 | **421** | ≈ 292.6 | 0.401661 → 0.935526 | 1.500000 | **no** |

Three readings, and they are the whole report.

* **The value at the loss point is far ABOVE the keep-side value floor, in all
  three rows**: 0.484 against 0.250, 0.552 against 0.381, 0.555 against 0.402. A
  value floor lowered to admit the reading would admit essentially the whole
  population at that position. This is the direct, measured reason the item's
  framing — "price the threshold needed to keep the `hid`-bearing partial" — does
  not have a value answer on this surface.
* **The gate is the horizon, and the reading misses it on its best order every
  time**: acoustic rank 22 against a horizon of 9; 142 against 146 (a miss of
  **four ranks**); 421 against 293.
* **The chain is discovered one hop at a time, and each hop is a new prune
  population.** The three-word prefix is in **no** prune population at all at
  beams 64, 512 and 1024 (0 occurrences in the whole run); it appears for the
  first time at beam **2048**, in exactly **2** prune calls, and is rejected in
  both. This is why a beam sweep alone does not localise anything: at every
  shipped-reachable budget the `hid`-bearing partial **is not constructed at
  all**.

**Correction to `REPORT-5c1a3e`'s criterion-1 table, recorded rather than
reconciled.** That report reads, at `beam_width 64`, "even the one-word prefix is
not in the surviving 64 at `p=3`". Measured here from the production path at
`beam_width 64`, the one-word prefix **is** in the surviving set at `p=3` in the
**first two** prune calls at that position (`selected=true`, combined rank 4) and
is discarded in the **third** (`selected=false`, combined rank 31, best order
rank 22 acoustic). It is a *later* prune call at the same position that loses it,
which is why a single-call reading of `p=3` can see either answer. The
localisation is unchanged — the discard is in `prune_partials` — and the depth
figures (`beam_width 4096`: 3-word prefix not in the top 4096 at `p=13`) are
consistent with and reproduced in kind by the 2048 row above.

---

## 2. The threshold price (completion criterion 2)

**Required value.** In rank terms, and this is the only currency the surface has:
admitting a candidate at rank `r` in its best order costs about `7r` beam slots,
because the horizon is `k / |orders|` with `|orders| = 7`.

| hop | position | best order rank (measured) | `7r` — the `k` that would be needed | is that `k` enough? |
|---|---|---|---|---|
| 1 — `hits` | `p=3` | 22 | ≈ **154** | — |
| 2 — `hits justice` | `p=10` | 142 | ≈ **994** | **no, measured:** 1024 misses by 4 ranks |
| 3 — `hits justice dupe` | `p=13` | 421 | ≈ **2,947** | **no, measured:** 2048 misses by 128 ranks |
| 4 — `+ hid` | `p=15` | unmeasured | ≥ 421, and the population grows | chain incomplete |
| 5 — `+ came` | `p=19` | unmeasured | ≥ 421 | chain incomplete |

The per-hop rank is not shrinking and the population at each hop roughly doubles
with `k`, so the requirement **compounds multiplicatively**, not additively. The
chain does not close inside the last budget measured.

**Admitted-partial count and the rest of the price.** The `k` axis, measured on
the canonical case-2 target at `top_n = 50`, release mode, single run each:

| `k` (beam) | pool | wall clock | candidate expansions into `prune_partials` | 3-word prefix present? | case-2 clue in pool | best leading run |
|---|---|---|---|---|---|---|
| **64 (shipped)** | 19,231 | **1.20 s** | 1,088,828 | no | **no** | 2 of 5 |
| 128 | 19,500 | 2.09 s | — | no | **no** | 2 |
| 192 | 19,584 | 2.93 s | — | no | **no** | 2 |
| 256 | 19,493 | 3.83 s | — | no | **no** | 2 |
| 384 | 19,431 | 5.07 s | — | no | **no** | 2 |
| 512 | 19,466 | 6.88 s | 7,881,813 | no | **no** | 2 |
| 1024 | 19,441 | 12.80 s | 15,093,786 | no | **no** | 2 |
| 2048 | — | 25.94 s | 27,544,801 | yes (2 calls), rejected | **no** | 2 |

* **Wall clock is linear in `k`** on this target (≈ `0.0126 s` per beam slot plus
  a 0.4 s constant), and it is the right currency: `REPORT`'s own runtime profile
  (`docs/work/approximate-runtime-profile.md` §1.2) measures `prune_partials` at
  **92.8–95.2 %** of approximate search time. A 16× budget is a 10.7× search.
* **Node expansions are linear in `k` too**, 1.09 M → 27.5 M from `k` 64 → 2048
  (25.3×), so a `k` that admits hop 3 would cost of the order of `7 × 421` slots
  per position, compounding.
* **Emissions: unchanged by construction, and this is worth stating plainly.**
  `prune_partials` does not read any emission budget — the emission ceilings
  (`LEXICAL_GLOBAL_EMISSION_BUDGET`, the per-segmentation clamp) are downstream of
  the forward beam, in the recovery walk. A threshold on the beam cannot spend an
  emission; it spends *expansions* and *wall clock*, which is what is measured
  above. The emission envelope is therefore not the binding constraint here, and
  `docs/work/budget-envelope.md`'s saturation result (the pool stops growing by
  64× the emission budget, at 177,029) is about a different stage and is not
  re-run here.
* **DP paths and funded slots**: unchanged. The structural DP and the
  `structure_reserve_slots(top_n)` = 12 representation reserve are untouched by
  anything this front measured or changed; `REPORT-1a4e8d` closes the reserve
  axis and `REPORT-7c9d21` closes the `top_n` cut, and neither is re-opened.
* **Pool size barely moves**: 19,231 → 19,441 across a **32× budget increase**,
  i.e. **+1.1 %**. The beam budget is not buying coverage; it is buying depth in
  a region of the space the pool already saturates. That is the clearest single
  statement of why this axis is a bad trade.

**Explicitly confirmed, not a silent absence.** The canonical clue's pool rank is
`None` — *not measured as missing, measured as not present in the pool* — at
**every one of** `k` ∈ {64, 128, 192, 256, 384, 512, 1024, 2048} and at the
shipped `top_n = 50`; and at `top_n = 200000` (the whole enumeration, pool 20,812)
it is still absent, with best leading run 2 of 5. The earlier absence of a
`prune-stats` line in this front's log was a **harness slip on my side** — the
environment variable was not exported on that invocation, so no line was
emitted; it was not a silent filter, and every expansions figure in this report
comes from a run where the line was present and read.

**Green control, `recognize speech` → `wreck a nice beach`, before and after.**
The baseline this front is protecting is **display/pool rank 27**, which agrees
with the CLI front `agent-8f0b3d1`'s release-binary reading of 27 and is one
above the 26 in the standing notes; 27 is the number to reconcile, and this
front does not attempt that reconciliation.

| configuration | pool | printed rank | pool rank | printed mean score | wall clock | expansions |
|---|---|---|---|---|---|---|
| **base, `k`=64 (the guard)** | 18,858 | **27** | **27** | 0.919653909 | **1.59 s** | 985,463 |
| widest priced variant, `k`=2048 + cost fill `k/2` | 18,622 | **27** | **27** | 0.919709397 | **35.65 s** | 33,770,067 |
| **delivered tree, after the revert** | **18,858** | — | **27** | — | **1.01 s** | — |

`+0.000055488` printed mean score for **22.4×** the wall clock and **34.3×** the
expansions, and **zero** movement in case 1's rank. The delivered tree reproduces
the base pool exactly (18,858) and the base rank (27).

---

## 3. The general rule, priced and rejected (completion criterion 3)

The one candidate with the right character — the same shape as the shipped
per-span retention policy's third stage, a number of slots justified by the
budget and not fitted to any phrase — is a **budget-derived cost-ordered fill**:
reserve `fill = k / share` of the beam and spend it on the acoustic order, so the
cheap-but-mediocre corner gets a route in that the round-robin structurally cannot
give it. This is precisely the mechanism that rescued `hid` inside its span shortlist
(`REPORT-5c1a3e` §1), so it is the obvious thing to try here. Priced on the
canonical case-2 target at `k` = 2048, where the three-word prefix first exists:

| `fill` share | fill slots | 3-word prefix at `p=13` | its best order rank | its combined | pool | wall clock | case-2 clue |
|---|---|---|---|---|---|---|---|
| none (base) | 0 | present, **rejected** | **421** (acoustic) | 0.554878 | — | 25.94 s | **absent** |
| `k/8` | 256 | present, **rejected** | 421 | — | — | 31.07 s | **absent** |
| `k/4` | 512 | present, **rejected** | **1236** | 0.554878 | 19,578 | 39.30 s | **absent** |
| `k/2` | 1024 | present, **rejected** | **1236** | 0.554878 | 19,642 | 44.65 s | **absent** |

**The decisive reading: widening the fill does not move the reading into the beam,
it moves the reading's own rank from 421 to 1236.** The extra admitted partials make
the position *more* crowded for the very prefix the rule was opened for — the
fill buys coverage (pool +0.3 %) and cost (+69 % wall clock at `k/4`), and buys
nothing on the reading. This is the containment argument of `REPORT-3e91a4`
(`OBSTRUCTION-MAP` §3) applied to this surface: a band property is a reach
coordinate only if it changes the admitted **set's contents** in the reading's
direction, and here the contents change the other way.

**Therefore: no general rule survives, and the front records a priced negative.**
The exact reason the threshold cannot be opened without breaking the budget, in
one sentence:

> The gate is a rank horizon of `k/7`, so admitting the canonical chain costs about
> `7 × (22, 142, 421, …)` beam slots **per position**, the per-position requirement
> *grows* rather than shrinks as the population doubles with `k`, and the third hop
> alone needs `k ≈ 2,950` — while a **value** floor cannot substitute at any price,
> because the value at the loss point (0.552) already sits far above the keep-side
> value floor (0.381), so a floor that admits it admits the whole position.

Supporting sweep, the `k` axis itself, which is the honest form of the same
question: 8 budgets from 64 to 2048, pool `+1.1 %`, wall clock `21.6×`, expansions
`25.3×`, case-2 clue absent at **8 of 8**, best leading run pinned at 2 of 5 at
**8 of 8**.

---

## 4. What is landed, and its non-vacuity, stated honestly

One test, `tests`-adjacent, in the `src/lib.rs` test module:
**`beam_retention_is_a_portfolio_and_not_a_value_floor`**. It pins the general,
weight-free, phrase-free property this front established:

> the retained set is **not** the top `k` by combined score — a discarded candidate
> can carry a combined value above the minimum of the retained set — and at a budget
> equal to the population nothing is dropped at all.

Over the existing synthetic 64-candidate pool, at `k = 8` and at `k = 64`. It names
no canonical word, phrase, sentence or rank.

**Non-vacuity, by mutation, with both results reported:**

| mutation | result |
|---|---|
| cell protection disabled (`take(k/2)` → `take(0)`) **and** fill reduced to the `combined` order | **FAILS** — "no discarded candidate beat the retained minimum 0.545170981215589" |
| fill reduced to the `combined` order alone | **still green** |

So the test **does** distinguish the shipped keep path from a combined-value top-`k`
rule, and it **does not** by itself distinguish the seven-way round-robin from a
single-order fill — because the cell protection on its own already admits
candidates below the top `k`. The test's doc comment says exactly this and its name
claims only "not a value floor"; the coordinator's earlier steer asked that the
"round-robin is the load-bearing reason" claim not survive an experiment that
contradicts it, and it has been removed rather than softened. The rank-horizon
mechanism itself is reported in §1 as a measurement, not fenced by this test; a
fixture that separates the round-robin from the cell protection would need
candidates spread across cells *and* axes, and was not built for a negative
delivery.

**Everything else this front produced is reverted.** The env-gated trace, the
`ZZ_COST_FILL` experiment, the `trace_pos` parameter and the probe example are all
**not in the delivered tree**: `git diff` against the base is `src/lib.rs`,
**+55 lines, 0 deletions**, all of it the one test and its comment. Per the
itinerary's review checklist no `ZZ_`-gated or env-gated probe instrumentation
lands; the two interim commits that carried it (`0779260`, `b870d9a`) are labelled
NOT production in their own messages and are superseded by the revert, and a
reviewer integrating this branch should take the final tree, not the interim
history. The harness source is reproduced in §7 so the numbers are reproducible
without it.

---

## 5. Fence results on the delivered tree

Run on this branch, release mode, dedicated `CARGO_TARGET_DIR`, `--test-threads=1`
(`corpus_integration` at `--test-threads=2`, because `OBSTRUCTION-MAP.md` §4 records
SIGKILL at default parallelism on this host at base too).

| suite | base `10c4a29` | delivered |
|---|---|---|
| `cargo test --release --lib` | 75 passed / 0 failed / 12 ignored | **76 passed / 0 failed / 12 ignored** (the one added test) |
| `no_phrase_hard_coding` | 9/9 | **9/9** |
| `emit_coverage` | 7/7 | **7/7** |
| `approx_determinism` | 4/4 | **4/4** |
| `exact_determinism` | 1/1 | **1/1** |
| `corpus_integration` | 12 passed / **1 failed** | **12 passed / 1 failed** |
| pool-reach guards (3) | 3/3 | **3/3** |
| `approximate_output_is_locked` | ok | **ok** |
| `approximate_list_represents_enumerated_resegmentations` | ok | **ok** |
| `approximate_finds_recognize_speech_resegmentation` (case 1) | ok | **ok** |

The single `corpus_integration` failure is the known pre-existing base red
`approximate_finds_classic_madgab_resegmentation` — the canonical case-2 test this
line of work exists to fix. It is red at the base and red in the delivery,
untouched and **not re-pinned**; its literals were not modified.

**Green control re-measured on the delivered tree, after the revert:**
`recognize speech` → `wreck a nice beach` at **pool rank 27**, pool 18,858,
1.01 s — identical to the base pool size and rank. **Case 1 is not regressed and
this front did not move it.**

**Not claimed:** `cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests
**cannot run on this host** — no `rustup`, no `rustfmt`/`clippy`/`rustdoc`
component (`docs/environment-notes.md`). `emit_spread` does not exist as a test
name on this tree; the emission counters are asserted from the `--lib` suite, which
is green, and that is what is reported rather than a convenient suite name.

---

## 6. Constraints

* **No phrase-specific hard-coding.** The delivered `src/lib.rs` diff is one test
  over the pre-existing synthetic pool. No input sentence, clue, `hid`, `dupe`,
  `came` or canonical target is a literal anywhere in the delivered tree; the
  target strings and the clue were passed to the harness at runtime through
  `argv` and environment variables. `no_phrase_hard_coding` is **9/9** with the
  probe absent, which is the state that ships. The allowlist was never edited and
  no finding was re-pinned.
* **The objective weight vector, the score function and the selection layer are
  untouched.** `axes::*` and `Metrics::combined` are byte-identical to the base,
  confirmed by diff: 0 deleted lines in `src/lib.rs`. The two surfaces the item
  forbids editing were not edited.
* **`src/approx.rs` was not touched.** The retention front owns it; per
  `REPORT-5c1a3e` that surface is closed as a surface (14/14 variants retain the
  word, 0/14 reach the pool).
* **No fence weakened, deleted, re-pinned or `#[ignore]`d**, and no
  `OBSTRUCTION-MAP.md` row re-opened.
* **Nothing self-merged.** `main` and `post-milestone-acceptance` are untouched;
  this is `madgab-thresh-3c5b18` for a coordinator to integrate.
* The two other fronts' worktrees were not read, entered or run. `REPORT-5c1a3e`
  was read from `post-milestone-acceptance` at `cfe5578` and is cited, not re-run.

---

## 7. The measurement harness, for reproducibility

Not in the delivered tree (env-gated probe instrumentation does not land). It took
every input from `argv`/environment, so it is inert with respect to any phrase:

```rust
// examples/…: target from argv, beam from ZZ_BEAM, clue from ZZ_CLUE
let g = Generator::from_json(CORPUS_JSON, GeneratorConfig {
    beam_width: std::env::var("ZZ_BEAM").ok().and_then(|s| s.parse().ok()).unwrap_or(64),
    top_n: 50, max_rarity: None,
    mode: SearchMode::approximate(), min_word_ipa_chars: 1,
})?;
// pool rank of an env-supplied clue, and the longest leading run of that clue
// present anywhere in the pool
```

The localisation of §1 came from an `eprintln!` inside `prune_partials`, gated on
`ZZ_PRUNE_NEEDLE` (a word sequence) and `ZZ_PRUNE_STATS`, reporting per prune
call: position, `k`, deduped population, whether the needle is present, its rank in
each of the seven orders, its seven axis values, its combined value and cost, its
cell-protection status, and the keep-side minimum, median and maximum of combined
and of cost. It matched the partial by its **word sequence**, not by substring,
after a first attempt that followed an arbitrary substring and produced a
misleading answer.

---

## 8. Handoff, in one line

**The case-2 beam prune is closed as a surface too:** `prune_partials` admits by a
seven-way **rank horizon** of `k/7` and not by any value threshold, the canonical
chain needs `7 × (22, 142, 421, …)` slots per position with the requirement
growing rather than shrinking, 8 of 8 budgets and 4 of 4 cost-fill shares leave
the clue absent, and the only widening that was tried makes the reading's own rank
worse (421 → 1236) at +69 % wall clock — so **the next case-2 front must be named
on a surface this one did not touch, and must not be another beam-budget or
band-ordering front.**

Two facts a successor can rely on without re-deriving them:

1. **`prune_partials` has no value threshold.** Any proposal to fix case 2 by
   moving a score, a weight, or a floor *inside this function* is addressing a
   quantity that does not exist. The function's only inputs are the budget `k` and
   the seven orderings.
2. **The beam budget is not a coverage lever.** 32× the budget buys +1.1 % pool and
   21.6× the wall clock, and does not move the reading's depth at all (best leading
   run 2 of 5 at 8 of 8 budgets). Whatever buys case-2 coverage is not spending
   beam slots.

HOLD
