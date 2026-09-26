# REPORT-9d3b17 — tip verification of `3b14482` for w-2f7a10 (front 9d3b17)

Worktree `/workspace/madgab-verify-9d3b17`, branch `scratch/2f7a10-tip-verify`, based at
`3b14482`. Measurement only. Nothing merged; `main`, `post-milestone-acceptance`,
`zzparent`, `scratch/2f7a10-base` and `madgab-pairing-2f7a10-rebase-backup` untouched.

**What is a probe here.** Two additions on my scratch branch only, both env-gated and
read-only:

* `MADGAB_POOL_DUMP=<path>` — in `Generator::finish`, immediately **after** the
  `phrase_signature` dedup and **before** `select_diverse`, writes the deduplicated pool
  the unmodified default search just built (rank, score, target, phrase, per-slot word
  list, resegmentation cuts) to that file.
* `zz_counters` — seven relaxed atomic tallies (`produced`, `attempted`,
  `cost_rejected`, `profile_built`, `allowance_stop`, `segmentations`,
  `spent_emissions`) printed only by the same env gate. They gate nothing.

Everything in §1–§5, §7 below is read out of that deduplicated default-path pool on a
**release** build, seven targets, `SearchMode::approximate()`, `top_n 50`,
`beam_width 64` — the same configuration `tests/corpus_integration.rs::approximate_proposals`
uses. No re-budgeted harness, no shortlist ranks, no widened enumeration, no
`select_diverse` reasoning. §6 wall clock is a separate timing harness with no probe.

Arms: parent `fe4ad78` and mid `a58a49b` were extracted with
`git archive <sha> | tar -x -C /workspace/arms/<sha>` and carry the **same** probe patch;
no merge was used to build any arm. The `3b14482` binary is this worktree.

**Run-to-run caveat, measured not assumed.** The pool at this size is not stable across
processes (the repo's own test comment says so). Four independent runs per arm: pool size
varies by ≤ 10 members per target; every named witness below is stable across runs except
`touched` on `a whole lot of trouble` (1–2 at `3b14482` vs 3 on the parent). The visible
top 50 *is* stable and byte-identical across runs and arms.

---

## 1. Membership, deduplicated default-path pool (probe), release

| target | arm | pool | distinct words | witnesses |
|---|---|---|---|---|
| It's just a stupid game | fe4ad78 | 17 906 | 2 013 | misfit 0, hits 46, dupe 14, hid 5, came 165, taught 11, justice 8 138 |
| | a58a49b | 17 636 | 1 888 | misfit 0, hits 52, dupe 9, hid 4, came 174, taught 4, justice 8 010 |
| | **3b14482** | **17 648** | **1 970** | misfit 0, hits 48, dupe 10, hid 3, came 167, taught 7, justice 8 029 |
| a whole lot of trouble | fe4ad78 | 16 151 | 1 794 | delve 1, touched 3 |
| | a58a49b | 15 708 | 1 663 | delve 1, touched 3 |
| | **3b14482** | **15 754** | **1 747** | delve 1, touched 1–2 |
| the cat sat on the mat | fe4ad78 | 16 262 | 1 801 | **tickets 3, capture 1, copper 0** |
| | a58a49b | 15 859 | 1 633 | **tickets 0, capture 0, copper 0** |
| | **3b14482** | **15 893** | **1 780** | **tickets 1, capture 0, copper 0** |
| put it back on the shelf | fe4ad78 | 15 633 | 1 793 | louis 8 |
| | a58a49b | 15 256 | 1 627 | louis 8 |
| | **3b14482** | **15 285** | **1 729** | louis 8 |
| when the rain finally stopped | fe4ad78 | 15 612 | 2 266 | aar 13, wince 5 |
| | a58a49b | 15 120 | 2 042 | aar 13, wince 12 |
| | **3b14482** | **15 167** | **2 243** | aar 13, wince 5 |
| he was a big fat man | fe4ad78 | 19 169 | 1 613 | honour 1 |
| | a58a49b | 18 991 | 1 489 | honour 1 |
| | **3b14482** | **19 015** | **1 549** | honour 2 |
| what are you going to do | fe4ad78 | 14 544 | 1 518 | perdues 0, perdue 2 |
| | a58a49b | 14 034 | 1 338 | perdues 0, perdue 2 |
| | **3b14482** | **14 060** | **1 438** | perdues 0, perdue 2 |

Seven-target totals: pool **115 277 → 112 604 (a58a49b) → 112 822 (3b14482)**;
distinct (target, word) **12 798 → 11 680 → 12 456**.

### 1b. Every witness that regressed to zero, not only `tickets`

Per target, distinct words present in the parent pool and **absent at `3b14482`**
(`regressions.txt` carries the full per-target lists with parent counts; total 1 724):

| target | lost at a58a49b | lost at 3b14482 | still lost at both | recovered by 3b14482 | gained at 3b14482 |
|---|---|---|---|---|---|
| It's just a stupid game | 429 | 297 | 183 | 114 | 254 |
| a whole lot of trouble | 342 | 218 | 131 | 87 | 171 |
| the cat sat on the mat | 371 | 229 | 142 | 87 | 208 |
| put it back on the shelf | 400 | 236 | 151 | 85 | 172 |
| when the rain finally stopped | 531 | 304 | 198 | 106 | 281 |
| he was a big fat man | 326 | 215 | 131 | 84 | 151 |
| what are you going to do | 384 | 225 | 135 | 90 | 145 |
| **total** | **2 783** | **1 724** | **1 071** | **653** | **1 382** |

Named regressions at the tip (parent count ≥ 4, absent at `3b14482`):

* `the cat sat on the mat`: `conn.` 8, `fan` 7, `nerve` 6, `cal` 5, `chess` 5, and
  **`capture` 1**.
* `put it back on the shelf`: `tons` 10, `did` 8, `tie` 6, `r.` 5, `sack` 5.
* `when the rain finally stopped`: `niece` 5, `asia` 4, `can` 4, `ey` 4, and
  **`copper` 1** (on *this* target; `copper` is 0 on the cat-sat pool on all three arms,
  so the sibling front's "`copper` also at zero" there is not a regression).
* `he was a big fat man`: `zip` 9, `e.g.` 8, `gossip` 6, `golf` 5, `sa` 5, `sabine` 5.
* `what are you going to do`: `sea` 7, `cory` 4, `mah` 4, `suing` 4, `you're` 4.
* `It's just a stupid game`: `west` 5, `p` 4, `plaid` 4, `sun` 4.
* `a whole lot of trouble`: `sit` 6, `one` 5, `fat` 4, `lob` 4, `och` 4, `trott` 4.

So: the breadth cost is **real at the tip and larger than "just `tickets`"** — 1 724
(target, word) pairs go to zero, of which 653 were already zero at `a58a49b` and are
restored by the continuity clause, and 1 071 are lost on both cuts. Two of the four
witnesses the sibling front named (`tickets` on cat-sat, `touched` on
`a whole lot of trouble`) are **not** zero at the tip.

## 2. Probe non-intrusiveness

Visible stdout (7 targets × (header + 50 clues) = 357 lines), byte comparison, three arms:

```
fe4ad78  IDENTICAL   a58a49b  IDENTICAL   3b14482  IDENTICAL
```

Also byte-identical to the same arm's stdout before the counter probe was added, and the
top-50 output is identical to the parent on **all seven** targets for both `a58a49b` and
`3b14482` (50 clues, same order, same scores) — which confirms `2f7a11`'s top-50 claim on
its own commits.

## 3. Pool breadth — the 11.5 % claim, attributed

Metric of the requested form, `occurrences / number of distinct slot positions`, summed
over every distinct word of the pool (pool-wide, seven targets):

| metric | fe4ad78 | a58a49b | 3b14482 | a58a49b vs parent | 3b14482 vs parent |
|---|---|---|---|---|---|
| Σ occ / distinct slot positions | 211 103 | 211 195 | 206 208 | **+0.04 %** | **−2.32 %** |
| same, non-leading occurrences only | 155 241 | 153 700 | 153 085 | −0.99 % | −1.39 % |
| distinct (word, non-leading slot) pairs | 24 286 | 21 738 | 23 192 | **−10.49 %** | **−4.50 %** |
| distinct words with ≥1 non-leading slot | 11 631 | 10 485 | 11 236 | **−9.84 %** | **−3.40 %** |
| distinct words (all slots) | 12 798 | 11 680 | 12 456 | −8.74 % | −2.67 % |
| pool members | 115 277 | 112 604 | 112 822 | −2.32 % | −2.13 % |

**The 11.5 % breadth loss is a property of `a58a49b`, not of the tip.** Every breadth
measure roughly halves its damage at `3b14482` (non-leading pairs −10.5 % → −4.5 %;
words with a non-leading slot −9.8 % → −3.4 %). I could not reproduce the exact
`9657 / 8546` pair from the sibling front on any of these definitions or targets, so that
specific pair remains unreplicated; the *direction and size* of the claim, and its
attribution, are confirmed.

Per-word slot-position histograms, canonical target (`It's just a stupid game`),
`occurrences / distinct slots`, plus the raw per-slot counts:

| word | fe4ad78 | a58a49b | 3b14482 | fe4ad78 slots | a58a49b slots | 3b14482 slots |
|---|---|---|---|---|---|---|
| hits | 46/1 | 52/2 | 48/1 | 0:46 | 0:50 1:2 | 0:48 |
| dupe | 14/3 | 9/2 | 10/2 | 2:6 3:7 4:1 | 2:4 3:5 | 2:6 3:4 |
| hid | 5/2 | 4/2 | 3/1 | 0:4 4:1 | 0:3 3:1 | 0:3 |
| came | 165/5 | 174/5 | 167/5 | 3:26 4:68 5:49 6:16 7:6 | 3:27 4:72 5:53 6:16 7:6 | 3:27 4:67 5:51 6:16 7:6 |
| taught | 11/3 | 4/2 | 7/2 | 2:6 3:4 4:1 | 2:3 3:1 | 2:5 3:2 |
| justice | 8 138/3 | 8 010/3 | 8 029/3 | 1:4276 2:2944 3:918 | 1:4196 2:2896 3:918 | 1:4212 2:2899 3:918 |

`hid` at the tip is **worse than the parent, not better**: 3 occurrences, leading slot
only (parent: 5, slots 0 and 4). The requested wording needs `hid` off the leading slot.
`hits` returns to leading-slot-only at the tip (parent behaviour). `came`, `taught`,
`dupe`, `justice` are all between the parent and `a58a49b`, closer to the parent than
`a58a49b` was. Other targets: `taught` on `put it back on the shelf` is 28 → 29 → **31**
members and **leading-slot only on all three arms**; `wince` on `when the rain finally
stopped` is 5 → 12 → **5**.

## 4. The pool guard test at the tip

```
$ cargo test --release --test corpus_integration \
    approximate_pool_reaches_alternatives_past_the_opening_slot_width -- --exact
running 1 test
test approximate_pool_reaches_alternatives_past_the_opening_slot_width ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 21.37s
```

Same test, same command, other arms:

```
fe4ad78  ... ok       (1 passed; 0 failed)   parent baseline green
a58a49b  ... FAILED   (0 passed; 1 failed)
  tests/corpus_integration.rs:357:9: the cat sat on the mat: "tickets" missing from a
  pool of 17355 clues, so a real alternative below the traversal's opening slot width
  was not reached
```

**`2f7a11`'s claim about the tip is correct and `3a9c21`'s refutation does not transfer.**
The guard is green at `3b14482` and red at `a58a49b`; the sibling front measured the
wrong commit. `3b14482` does not merely shrink the loss on this guard — it restores the
parent baseline for the four named witnesses (`tickets` 1, `touched` 1–2, `wince` 5,
`taught`/`delve`/`louis`/`aar`/`honour`/`perdue` unchanged) and clears the ≥1 000
distinct-word floor on all three targets (1 780 / 1 747 / 2 243 vs floors of 1 000).

## 5. The requested wording

`hits justice dupe hid came` on `It's just a stupid game`, release, default path:

| arm | pool member | visible top 50 |
|---|---|---|
| fe4ad78 | **NO** (0 members) | NO |
| a58a49b | **NO** | NO |
| 3b14482 | **NO** | NO |

All six pairs of its own words are 0 in the tip's pool as well (`hits+dupe`, `hid+dupe`,
`hits+hid`, `dupe+came`, `hid+came`, `hits+came`). `misfit` is 0 on every target and
every arm. This criterion is **not** met at the tip, exactly as `2f7a11` states.

`recognize speech` → `wreck a nice beach`: present —
`approximate_finds_recognize_speech_resegmentation` is green in the tip's
`corpus_integration` run (10 passed / 1 failed, the one failure being
`approximate_finds_classic_madgab_resegmentation`, which is also red on the parent).

## 6. Interleaved wall clock, parent `fe4ad78` vs `3b14482`

Alternating arm order per round, 7 timed runs per arm per target, **one discarded
warm-up search per binary**, one target per process, medians:

```
It's just a stupid game   fe4ad78  n=7  median 1.373s  min 1.315  max 1.424
It's just a stupid game   3b14482  n=7  median 1.347s  min 1.291  max 1.495   (-1.9 %)
the cat sat on the mat    fe4ad78  n=7  median 1.492s  min 1.435  max 1.935
the cat sat on the mat    3b14482  n=7  median 1.520s  min 1.438  max 2.399   (+1.9 %)
```

Both deltas are inside the run-to-run spread (tip max 2.399 s vs its median 1.520 s). No
wall-clock regression; the timings do not support a claim either way beyond that.

## 7. Full suite at `3b14482` (release, nothing re-baselined, no test edited)

| command | result |
|---|---|
| `cargo test --release --lib` | **51 passed, 0 failed** |
| `cargo test --release --test corpus_integration` | **10 passed, 1 failed** — `approximate_finds_classic_madgab_resegmentation` (line 136) |
| `cargo test --release --test exact_determinism` | 1 passed, 0 failed |
| `cargo test --release --test approx_determinism` | 2 passed, 0 failed |
| `cargo test --release --test no_phrase_hard_coding` | 6 passed, 0 failed |

`approximate_finds_classic_madgab_resegmentation` is the item's **target**, not a guard,
and is red on the parent `fe4ad78` too. `cargo fmt`, `clippy` and doctests were not run
and are not claimed.

## 8. Check against `2f7a11`'s committed report (`82894d0`)

Claims I could test, on my own arms and binaries:

| claim | my measurement | agrees? |
|---|---|---|
| `tickets` broke on the first cut, continuity clause restores it | 3 → 0 (a58a49b) → 1 (3b14482), stable over 4 runs; guard test red → green | **yes** |
| visible top 50 byte-identical to parent on all six targets at the tip | identical on **seven** targets, both cuts | **yes** |
| canonical pool 17 907 → 17 648 | 17 906 → 17 648 | yes (−1 from run noise) |
| distinct words 2 014 → 1 970 | 2 013 → 1 970 | yes (−1 run noise) |
| canonical `dupe` 14→10, `hid` 5→3, `taught` 11→7 | 14→10, 5→3, 11→7 (at the tip) | **yes, exactly** |
| canonical `hits` 48, `came` 167, `justice` 8 029 | 48, 167, 8 029 | **yes, exactly** |
| counter table, canonical: produced 3404→3469, built 3049→2598, cost-rejected 355→871, spent 15 204→14 977, segmentations 256→256 | 3404→3469, 3049→2598, 355→871, 15 204→14 977, 256→256 | **yes, every figure** |
| counter table, `put it back on the shelf`: produced 3834→3866, built 2900→2368, cost-rejected 934→1498, spent 12 452→12 171 | 3834→3866, 2902→2368, 932→1498, 12 454→12 171 | yes (built/cost-rejected ±2, spent ±2) |
| mechanism: same segmentations, +2 % tuples, far more tuples refused by the additive `total_budget` (10.4 % → 25.1 % canonical) | canonical refusal share 355/3404 = **10.4 %** → 871/3469 = **25.1 %**; segmentations 256 on both, all seven targets | **yes, exactly** |
| breadth not recoverable within the same constants | not independently derivable from my measurements; `spent_emissions` is 1.5 % *below* the parent and `allowance_stop` is **0 on all seven targets on all three arms**, i.e. the reserve is never truncated by the per-segmentation allowance, so nothing was left unspent to convert — consistent with the claim | consistent, not proven |
| "no other witness is known to me to have regressed" | **contradicted in detail**: 1 724 (target, word) pairs are absent at the tip that the parent has, including `capture` (cat-sat, 1→0), `copper` (when-the-rain, 1→0), `tons` 10→0, `zip` 9→0, `did` 8→0, `fan` 7→0, `sea` 7→0, `tie` 6→0, `sit` 6→0, `one` 5→0, `sack` 5→0, `chess` 5→0, `cal` 5→0, `golf` 5→0 | **no** |

My own counter table for the other five targets, for the record (produced / built /
cost-rejected / spent, parent → a58a49b → 3b14482; segmentations 256 on all three):

```
a whole lot of trouble        3443/2718/725/10676  3483/2174/1309/10249  3483/2236/1247/10292
the cat sat on the mat        3892/3445/447/11940  3929/2828/1101/11579  3929/2897/1032/11619
when the rain finally stopped 3952/3731/221/10218  3986/3120/866/9753   3986/3181/805/9788
he was a big fat man          3721/2389/1332/14620 3756/1845/1911/14486  3756/1970/1786/14503
what are you going to do      3920/3033/887/11450  3963/2319/1644/11061  3963/2388/1575/11072
```

## 9. Attribution summary of the contested measurement

* The −11.5 %-style breadth loss, the `tickets` 3 → 0 regression, and the red pool guard
  are all properties of **`a58a49b`**.
* At **`3b14482`**: pool guard green, `tickets` restored, roughly half the breadth loss
  restored, visible output byte-identical to the parent, wall clock flat, the only failing
  test is the item's own target, and the requested wording is still absent.
* What `3b14482` does **not** undo: a real if smaller breadth cost — 1 724 (target, word)
  pairs to zero, 1 071 of them lost on both cuts — and `hid`/`hits` are back to
  leading-slot-only, i.e. the property this item is about is not advanced on the
  canonical target by this commit.

## 10. Reproduce

```sh
# tip arm (this worktree), probe off then on
cargo build --release --example zz_pool_dump_9d3b17
./target/release/examples/zz_pool_dump_9d3b17 > a.txt
MADGAB_POOL_DUMP=/tmp/p.tsv ./target/release/examples/zz_pool_dump_9d3b17 > b.txt
cmp a.txt b.txt            # identical

# parent arm, never merged
mkdir -p /workspace/arms/fe4ad78
git archive fe4ad78 | tar -x -C /workspace/arms/fe4ad78
# apply the same env-gated probe patch to that tree, then
cd /workspace/arms/fe4ad78 && cargo build --release --example zz_pool_dump_9d3b17
```

Artifacts left in `/workspace/arms/dumps`: `*.pool2.tsv` (probe dumps),
`*.stdout2.txt` (visible output), `analysis.txt`, `regressions.txt`, `timing.tsv`.
`/workspace/arms/fe4ad78` and `/workspace/arms/a58a49b` are `git archive` extractions
carrying the probe; they are measurement arms, never merge sources.
