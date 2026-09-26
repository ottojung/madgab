# `--approximate` profiling report

Repo: `/workspace/madgab-approx-runtime`, branch `madgab-approx-runtime`
(based on `post-milestone-acceptance`).
Measured on this machine with `target/release/madgab`, 3 runs per target unless
stated otherwise. Nothing committed, nothing pushed, no test edited.

---

## 0. Working-tree state (read this first)

The tree is **dirty and uncommitted**, as permitted. There is exactly **one**
non-instrumentation change, described in §5.1 — it is behaviour-identical and
verified as such, but you should decide explicitly whether to keep it.

Everything else is inert `TEMP-PROF` scaffolding (see `prof/README.md`), disabled
unless `MADGAB_PROF=1` is set. Nothing is committed; nothing is pushed.

---

## 1. Where the time goes

### 1.1 Coarse split (existing `load_ms`/`search_ms` on stderr)

`src/main.rs:142,158`. Note the load figure **includes two full `serde_json`
parses of the 15 MB `CORPUS_JSON`**: `Corpus::from_json` (lib.rs:206) and
`approx::build_lexicon` (lib.rs:221, which re-parses the same string purely to
recover the word list, `src/approx.rs:323`).

| target | n (IPA chars) | load | search | load as % of total |
|---|---|---|---|---|
| recognize speech | 14 | 0.40 s | 9.82 s | 3.9 % |
| It's just a stupid game | 19 | 0.44 s | 22.29 s | 1.9 % |
| I love you | 7 | 0.42 s | 3.42 s | 10.9 % |
| there is no place like home | 21 | 0.40 s | 33.13 s | 1.2 % |
| congratulations on your promotion | 30 | 0.42 s | 58.54 s | 0.7 % |
| my favorite color is blue | 19 | 0.48 s | 24.20 s | 1.9 % |
| insurance is important | 19 | 0.44 s | 26.84 s | 1.6 % |
| how much wood would a woodchuck chuck | — | ~0.40 s | ~0.3 s | ~57 % |

**Corpus load is 0.40–0.48 s and is never the problem** (≤ 0.7 % of the slow
cases; it is the *only* cost for the 0.6 s target). Split of the load, measured
separately: `Corpus::from_json` 0.30–0.32 s, `build_lexicon` 0.13–0.15 s.

**Conclusion: 95–99 % of the time is inside `Generator::generate_approximate`.**

### 1.2 Fine phase breakdown (baseline, mean of 3 runs)

Fractions are of `generate_approximate` time. `prune_partials` sub-rows overlap
each other, not the phases.

| phase | t1 recog. | t2 stupid | t3 love | t4 home | t5 congrats | t6 color | t7 insurance |
|---|---|---|---|---|---|---|---|
| **total search** | 9.82 s | 22.29 s | 3.42 s | 33.13 s | 58.54 s | 24.20 s | 26.84 s |
| lattice `matches_at` | 0.2 % | 0.1 % | 0.3 % | 0.1 % | 0.1 % | 0.1 % | 0.2 % |
| main forward beam | 99.8 % | 99.9 % | 99.7 % | 99.9 % | 99.9 % | 99.9 % | 99.8 % |
| — `extend_fuzzy` (clones) | 2.5 % | 2.4 % | 2.9 % | 2.2 % | 2.2 % | 2.4 % | 2.6 % |
| — `prune_partials` (all) | **93.7 %** | **94.3 %** | **92.8 %** | **95.0 %** | **95.2 %** | **94.4 %** | **94.5 %** |
| span shortlists | 2.8 % | 2.5 % | 3.2 % | 2.0 % | 1.9 % | 2.4 % | 2.1 % |
| structural DP | 1.9 % | 1.8 % | 2.1 % | 1.2 % | 1.3 % | 1.7 % | 1.4 % |
| lexical heap enum | 1.9 % | 1.7 % | 2.1 % | 1.2 % | 1.2 % | 1.7 % | 1.4 % |
| `finish` (score+sort+dedup+diverse) | 0.7 % | 0.4 % | 0.9 % | 0.3 % | 0.3 % | 0.4 % | 0.4 % |

Inside `prune_partials` (the whole game):

| sub-step | t1 | t2 | t3 | t4 | t5 | t6 | t7 |
|---|---|---|---|---|---|---|---|
| dedup loop (81) | 93.7 % | 94.3 % | 92.8 % | 95.0 % | 95.2 % | 94.4 % | 94.5 % |
| **final sort, metrics recomputed in comparator (85)** | **76.5 %** | **77.3 %** | **74.5 %** | **78.3 %** | **78.9 %** | **77.7 %** | **77.3 %** |
| `Partial::metrics` total (90) | 86.0 % | 88.6 % | 83.2 % | 90.1 % | 90.5 % | 88.9 % | 88.3 % |
| `metrics()` calls / run | 8.2 M | 11.5 M | 3.9 M | 14.1 M | **19.2 M** | 10.5 M | 12.8 M |

Note `80_prune_total` and `81_prune_dedup_loop` read the same because those
`prof::T` guards live in the function body and therefore run to end-of-function.
The meaningful decomposition is the 85/90 rows, which are block-scoped.

**So: `prune_partials` is 93–95 % of search time, and ~78 % of total search is a
single `sort_by` whose comparator re-runs `Partial::metrics` on both operands.**

Target 8 (`how much wood would a woodchuck chuck`) is omitted from the
per-phase tables because it returns **no clues at all** — the program exits at
`src/main.rs:160-164` before `finish` runs, so there is no search breakdown to
report. It is pure corpus-load dominated (~0.4 s load, ~0.3 s search).

---

## 2. Super-linear behaviour, with evidence

### 2.1 Measured scaling (post-fix-1 build, so the structural effects are visible)

`prof/scale.txt`, `n` = IPA length:

| n | 14 | 19 | 25 | 29 | 33 | 42 | 46 |
|---|---|---|---|---|---|---|---|
| search | 2.36 s | 4.54 s | 8.68 s | 13.51 s | 18.61 s | 30.42 s | 37.57 s |
| `prune_partials` calls | 6 905 | 10 679 | 14 971 | 17 220 | 20 080 | 25 139 | 28 274 |
| `metrics()` calls | 1.02 M | 1.47 M | 2.04 M | 2.31 M | 2.69 M | 3.32 M | 3.74 M |
| `extend_fuzzy` calls | 0.51 M | 0.74 M | 1.02 M | 1.16 M | 1.35 M | 1.67 M | 1.88 M |

* Time vs `n`: 46/14 = 3.29× longer target, 15.9× more time ⇒ ≈ **n^2.2**.
* `metrics()` calls grow only 3.7× (≈ linear in `n`), but *time per call* grows
  6.3× (1.15 µs → 7.2 µs). That is the quadratic: `Partial::metrics` is
  **O(W · T)** in clue words × target words (see §2.2), and W and T both grow
  with `n`.
* `prune_partials` calls grow 4.1× for 3.29× `n` — mildly super-linear, because
  the number of completed hypotheses reaching position `n` grows with `n`.

### 2.2 The specific quadratic mechanisms

1. **`Partial::metrics` is O(W·T) with allocations** (`src/lib.rs:1233`).
   `candidate_reuses_target` (lib.rs:943) is called once per clue word and does
   `targets.iter().any(|t| same_lexical_family(word, t))`; `same_lexical_family`
   (lib.rs:926) calls `novelty_stem` (lib.rs:897) on **both** operands, and
   `novelty_stem` allocates 2–3 `String`s per call. So one `metrics()` call
   performs ≈ `W·T·5` heap allocations. **This is 41–49 % of total search time**
   (measured in §3.2).
2. **`boundary_novelty` (lib.rs:1151) allocates two `HashSet<usize>` per call**,
   rebuilding `target_inner` from `target_boundaries` every time. 5–8 % of search.
3. **`Partial::extend_parts` (lib.rs:1190) builds `key` with
   `format!("{} {}", self.key, step_key)` (lib.rs:1217)** — O(key length) per
   extension, so **O(W²) per path**, on top of cloning the whole `words: Vec<ClueWord>`
   and `cuts: Vec<usize>`. `extend_fuzzy` totals 7–11 % of post-fix search and is
   itself quadratic in `n`.
4. **`final_keep` / completed-pool pruning.** `keep = top_n*128` = 2560 at
   `--top 20`, triggered every 5120 pushes (lib.rs:248–266). At `q == n` this
   fires ~18 400 times for `n = 30`, and each fire sorts 2560 items with a
   comparator that re-ran `metrics` — that is the 19.2 M `metrics()` calls.

### 2.3 Does output size grow super-linearly?

No. The *returned* list is exactly `top_n` items (dedup at lib.rs:827) — O(1) in
`n`. But the *internal* candidate pool is pinned at `SEGMENTATION_KEEP = 128`
segmentations × `LEXICAL_COMBINATIONS_PER_SEGMENTATION = 64` fills plus the
2560-deep completed beam — i.e. **the intermediate work is O(1) in `n` only in
the worst case, and O(n) in the number of `prune_partials` invocations**. The
super-linearity is in the *cost of each* invocation, not in the pool sizes.

### 2.4 Suspects that turned out **not** to matter

* **`matches_at` (src/approx.rs:76)** — 0.1–0.3 % of search. The weighted edit
  walk is cheap: 22 k–117 k stack pops per position, `HashMap` `best`/`found` and
  the per-position `Vec<Vec<FuzzyMatch>>` are not the bottleneck. Do not
  optimize this.
* **`select_diverse` (lib.rs:1362)** — 0.7–2.3 % of post-fix search
  (0.02–0.10 s). The `O(top_n · candidates)` loop is not the problem.
* **Corpus load / `build_lexicon`** — 0.4 s flat, ≤ 0.7 % of slow cases.
* **Structural DP** — 0.1 % of post-fix search. The
  `SEG_STATE_KEEP = 32` truncation keeps it tiny.
* **Lexical heap enumeration** — 4–6 % of post-fix search.

---

## 3. Ranked hot spots

### 3.1 #1 — `Partial::metrics` re-run inside the `prune_partials` sort comparator

`src/lib.rs:1508–1512` (original):

```rust
out.sort_by(|a, b| {
    let am = a.metrics(target_boundaries, target_words, total_len, true);
    let bm = b.metrics(target_boundaries, target_words, total_len, true);
    cmp_desc(am.combined, bm.combined)
});
```

**Cost: 46.2 s of 58.5 s (78.9 %) for `congratulations on your promotion`.**
Mechanism: `metrics` is a pure function of `(Partial, target_boundaries,
target_words, total_len, true)`; all four are loop-invariant, and the result is
*already materialised* in the local `metrics: Vec<Metrics>` at lib.rs:1443.
Sorting 2560 `usize` indices against the cached vector produces exactly the same
order for **2 × 2560 × log₂ 2560 ≈ 57 000 `metrics()` calls per prune**, ×
18 400 prunes.

### 3.2 #2 — `candidate_reuses_target` / `novelty_stem` inside `metrics`

Measured split of `Partial::metrics` (post-fix-1 run, so the percentages are of
the *remaining* time; four extra `Instant::now()` per call inflate the total by
~10 %):

| component | t1 | t2 | t3 | t4 | t5 | t6 | t7 |
|---|---|---|---|---|---|---|---|
| `candidate_reuses_target` (93) | 31.8 % | 42.8 % | 27.5 % | 47.2 % | 48.6 % | 45.1 % | 41.0 % |
| `boundary_novelty` (92) | 6.2 % | 6.7 % | 7.7 % | 6.7 % | 5.1 % | 6.2 % | 6.3 % |
| `lexical_shape_quality` agg (95) | 7.0 % | 5.4 % | 6.4 % | 5.0 % | 5.5 % | 5.1 % | 6.4 % |
| `word_familiarity` agg (94) | 3.0 % | 2.1 % | 3.2 % | 2.0 % | 1.9 % | 2.1 % | 2.5 % |

`lexical_shape_quality` (lib.rs:932) calls `normalized_word`, which allocates a
fresh `String` just to count chars and compare against `"a"`/`"i"`.

### 3.3 #3 — `Partial::extend_parts` clones and quadratic `key`

`src/lib.rs:1190`. 7.5–10.8 % of post-fix search, 0.5 M–1.9 M calls per run.
`words.clone()` (2 `String` allocs per word), `cuts.clone()`, and
`format!("{} {}", self.key, step_key)` which is O(W²) over a path's lifetime.

---

## 4. How the two fix classes divide

### 4.1 Behaviour-**preserving** (recommended, in order)

| # | Change | Est. saving | Risk |
|---|---|---|---|
| **P1** | `prune_partials`: sort `Vec<usize>` indices by the cached `metrics` vector instead of `Vec<Partial>` by a recomputing comparator (`src/lib.rs:1508`) | **3.8–4.9× end-to-end (measured)** | none |
| **P2** | `Partial::metrics`: hoist the per-word aggregates into incremental fields on `Partial`, updated in `extend_parts` — `reused_count: usize`, `familiarity_sum: f64`, `ipa_len_sum: usize`, `shape_sum: f64`. `metrics()` becomes O(1) + `boundary_novelty` | removes **41–49 %** of remaining time; est. **2.2–3×** on top of P1 | none, if the arithmetic order is preserved exactly |
| **P3** | `candidate_reuses_target` → precompute `HashSet<String>` of `novelty_stem(target_word)` once per `generate_approximate`; then it is one `novelty_stem(candidate)` + one hash lookup. Also `lexical_shape_quality` → count normalized alphanumerics without allocating | further **~10 %**; subsumed by P2 but worth doing anyway since `candidate_reuses_target` is also called from `span_lattice` (lib.rs:344, 678, 718) and the heap rank closure | none |
| **P4** | `boundary_novelty` (lib.rs:1151): replace both `HashSet<usize>` with a `u64` bitmask over positions `< total_len` (n ≤ ~64 for these targets; use `Vec<u64>` words for generality). `target_inner` for a given `covered` can be precomputed into an `n+1`-entry table, because `partial` only filters on `b <= covered` | **5–8 %** | none |
| **P5** | `Partial::extend_parts`: `words: Rc<[ClueWord]>` (or `Rc<Vec<..>>`) to make extension O(1) instead of O(W); `cuts: Rc<[usize]>`; replace `key: String` with a per-call **interned `u64` id** (dedup only needs key *equality*, so an exact-equality interner is equivalent) | **7–11 %** plus removal of the O(W²) `key` growth — the main remaining quadratic | low: `into_clue` needs `Rc::try_unwrap` or a clone of the slice |
| **P6** | `prune_partials` dedup loop (lib.rs:1430–1443): `old.metrics(...)` is recomputed for the incumbent on every duplicate. Cache the incumbent's `combined` alongside it | **4–7 %** | none |
| **P7** | Skip `build_lexicon`'s second `serde_json::from_str` (src/approx.rs:323) by having `Corpus` expose an iterator of preferred pronunciations | **0.13 s flat** (0.2–10 % of total depending on target) | low; may need a `phonetics-rs` change |
| **P8** | `finish`/`select_diverse` (lib.rs:807, 1362) | ≤ 2 % | not worth it |

P1+P2+P3+P4 together should put the 58.5 s case at roughly **1 s** of search
(~0.4 s load). P5 then removes the remaining `n²` term.

### 4.2 Behaviour-**changing** (only if you need more than ~50×)

Ranked by savings/risk. Each names the output property at risk.

1. **Lower the completed-hypothesis `keep` multiplier** (lib.rs:248–256 and
   271–276: `top_n * 128`, min 1024, max 8192). This is the single biggest
   lever: `keep` linearly controls how many times `prune_partials` runs and how
   big each sort is, and `keep = 2560` at `--top 20` is 128× larger than the
   20 clues actually returned. Dropping to `top_n * 8` would cut the prune
   count by ~16×.
   *Risks:* the entire point of the 128× pool (see the comment at lib.rs:240–247)
   is that completed hypotheses are never re-expanded, so a locally mediocre
   word can only be rescued by the *global* scorer. Narrowing it degrades
   **clue quality / final score**, and specifically the "globally-good parse
   that a tight beam misses" case. It will show up as lower top-1 score and
   fewer distinct resegmentations, most visibly on long targets.
2. **Narrow the per-span shortlist** (`SPAN_AXIS_KEEP = 16`, `SPAN_BAND_KEEP = 4`,
   lib.rs:311–313). These feed `SEGMENTATION_KEEP` and the heap, so narrowing
   risks losing the **globally-best resegmentation** and reducing phrase
   diversity — the near-homophone portfolio is explicitly there to avoid
   returning only the cheapest N words.
3. **Reduce `SEGMENTATION_KEEP = 128` / `LEXICAL_COMBINATIONS_PER_SEGMENTATION = 64`**
   (lib.rs:314–315). Note `segmentations` was observed to be **saturated at 128
   for every target with n ≥ 19** — the cap is binding, so lowering it directly
   discards the top-ranked structural alternatives. Risks losing the
   best-scoring segmentation entirely, and the `finish` pool is already
   dominated by these 128×64 = 8192 recovered fills.
4. **Reduce `beam_width` (default 64)** or tighten the `keep * 2` re-prune
   trigger (lib.rs:257). Risks losing mid-search resegmentations — the classic
   beam-search quality cliff — most damaging for targets with many
   word-boundary crossings.
5. **Drop `--approximate`'s `MATCHES_PER_SPAN = 256`** (src/approx.rs:14).
   Risks losing acoustic alternatives; interacts with 1 and 2.

Note that 1–5 all shrink the *candidate pool*; none of them touch the matcher,
and none of them are needed if P1–P5 land.

### 4.3 Not worth parallelising

`generate_approximate` is single-threaded and dominated by one sequential
dependency chain (beam → prune → prune). The only parallelisable regions are
`lattice` (0.2 %) and the corpus load (0.4 s). `matches_at` for different
positions `p` is independent and could be `rayon`-parallelised, but at 0.2 % of
runtime it is pointless. The real win from parallelism would be overlapping the
0.4 s corpus load with… nothing, since it gates everything.

---

## 5. Verification performed

### 5.1 P1 was implemented and validated (the one non-instrumentation change)

`prof/madgab-baseline` is a pristine `HEAD` binary kept as an output oracle.
Comparison covers **8 targets × 7 configurations = 35 combinations**
(including Exact mode, `--top 3 --beam 16`, altered budgets, `--max-rarity`,
`--min-word-len`):

* **33 / 35 byte-identical.**
* 1 differs only by a permutation of equal-scored clues (score sequence
  identical).
* 1 (`--top 50` + `congratulations on your promotion`) differs in the score
  sequence.

**Both discrepancies are pre-existing nondeterminism, not caused by the change.**
Evidence: `HashMap`/`HashSet` iteration order in Rust is randomised per process
(`RandomState`), and `prune_partials` returns `dedup.into_values()` and
`selected.into_iter()` in hash order; the stable sort then breaks score ties in
that order, which propagates through the beam. Running the **unmodified
baseline** 6 times on `--top 50 congratulations on your promotion` produces
**4 distinct score sequences**. At `--top 20`, 4 of 5 targets are deterministic
across 3 runs in *both* binaries; `congratulations on your promotion` is
nondeterministic in both.

End-to-end wall clock, best of 2, uninstrumented, `MADGAB_PROF` unset:

| target | baseline | after P1 | speed-up |
|---|---|---|---|
| recognize speech | 13.86 s | 3.66 s | 3.79× |
| It's just a stupid game | 28.82 s | 6.35 s | 4.53× |
| I love you | 4.66 s | 1.65 s | 2.82× |
| there is no place like home | 42.48 s | 10.43 s | 4.07× |
| congratulations on your promotion | 68.58 s | 14.11 s | 4.86× |
| my favorite color is blue | 28.86 s | 6.66 s | 4.33× |
| insurance is important | 28.52 s | 6.48 s | 4.40× |
| how much wood would a woodchuck chuck | 0.70 s | 0.65 s | 1.08× |

(This machine is noisy — the same baseline target measured anywhere from 11.0 s
to 13.9 s across sessions. Treat ratios, not absolutes, as the signal.)

### 5.2 Post-P1 phase breakdown (mean of 3, `prof/sum-exp6.txt`)

Fractions of post-P1 `generate_approximate` time:

| phase | t1 | t2 | t3 | t4 | t5 | t6 | t7 |
|---|---|---|---|---|---|---|---|
| total search | 2.38 s | 5.03 s | 0.97 s | 7.12 s | 10.94 s | 5.02 s | 5.13 s |
| lattice `matches_at` | 1.1 % | 0.6 % | 1.3 % | 0.6 % | 0.5 % | 0.6 % | 0.8 % |
| `prune_partials` (all) | 72.6 % | 76.0 % | 72.0 % | 78.6 % | 78.6 % | 76.4 % | 76.5 % |
| — dedup loop | 6.0 % | 4.4 % | 7.0 % | 3.9 % | 3.9 % | 4.0 % | 4.6 % |
| — metrics vector build | 49.5 % | 58.8 % | 45.0 % | 63.3 % | 63.6 % | 60.3 % | 57.2 % |
| — cells + 5 orderings | 5.3 % | 3.9 % | 6.7 % | 3.4 % | 3.0 % | 3.6 % | 4.1 % |
| — final sort (**was 77 %**) | **0.4 %** | **0.3 %** | **0.5 %** | **0.2 %** | **0.2 %** | **0.2 %** | **0.3 %** |
| `extend_fuzzy` | 9.8 % | 8.0 % | 10.8 % | 7.2 % | 7.7 % | 7.6 % | 8.5 % |
| span shortlists | 3.6 % | 3.6 % | 4.1 % | 4.1 % | 3.3 % | 3.6 % | 3.5 % |
| structural DP loop | 0.0 % | 0.1 % | 0.0 % | 0.1 % | 0.1 % | 0.1 % | 0.1 % |
| lexical heap enum | 5.6 % | 6.2 % | 4.3 % | 4.6 % | 4.9 % | 6.2 % | 5.0 % |
| `finish` | 3.0 % | 1.9 % | 3.4 % | 1.5 % | 1.3 % | 2.1 % | 1.9 % |
| — of which `select_diverse` | 1.9 % | 1.0 % | 2.2 % | 0.9 % | 0.8 % | 1.2 % | 1.1 % |

### 5.3 Tests

`cargo test --release` on my tree and on a stashed pristine tree give **the same
result**: 4 passed, 2 failed
(`approximate_finds_classic_madgab_resegmentation`,
`approximate_finds_recognize_speech_resegmentation`, in
`tests/corpus_integration.rs:133,143`). **These two failures are pre-existing on
`post-milestone-acceptance` / this branch's HEAD and are not caused by anything
I did.** No test was edited or weakened.

---

## 6. Recommended implementation order for the implementing agent

1. **P1** — `src/lib.rs:1508`. Replace the comparator sort with an index sort
   against the cached `metrics` vector. *Already validated in this tree; keep or
   re-derive.* Expect 3.8–4.9×.
2. **P2** — `src/lib.rs:1233` `Partial::metrics` + `src/lib.rs:1190`
   `Partial::extend_parts`. Add `reused_count: usize`, `familiarity_sum: f64`,
   `ipa_len_sum: usize`, `shape_sum: f64` to `struct Partial` (lib.rs:954) and
   update them in `extend_parts` in the **same order** as `metrics` currently
   iterates, so the `f64` sums are bit-identical. Keep
   `word_novelty = 1.0 - reused_count as f64 / words.len().max(1) as f64`
   exactly as written. Expect a further 2.2–3×.
3. **P3** — `src/lib.rs:943` `candidate_reuses_target`. Build
   `target_stems: HashSet<String>` once in `generate_approximate` (lib.rs:187,
   next to `target_words`); rewrite as
   `target_stems.contains(&novelty_stem(word))`. Pass it to the three other call
   sites (lib.rs:344, 678, 718). Also make `lexical_shape_quality` (lib.rs:932)
   allocation-free — it only needs the normalized *char count* and an
   `== "a" || == "i"` test.
   **This is a prerequisite for a clean P2**, so do 3a before/with 2 if you
   prefer; it is also independently correct for the non-`Partial` call sites.
4. **P4** — `src/lib.rs:1151` `boundary_novelty`. Bitmask over positions
   `< total_len`; precompute the `partial`-filtered `target_inner` mask per
   `covered` value into an `n+1` table built once in `generate_approximate`.
5. **P6** — `src/lib.rs:1430` dedup loop: cache the incumbent's `combined`.
6. **P5** — `src/lib.rs:954` `struct Partial`: `words: Rc<[ClueWord]>`,
   `cuts: Rc<[usize]>`, `key` → interned `u64` (scoped interner in
   `generate_approximate`; pass to `prune_partials`). This is the last remaining
   `n²` term. `into_clue` (lib.rs:1201) needs an `Rc::try_unwrap`/slice copy.
7. **P7** — `src/approx.rs:323`: eliminate the duplicate `serde_json::from_str`.
8. Only if still too slow, move to §4.2 and treat every change as a deliberate
   quality regression to be measured, not a free win.

Verification recipe for each step: rebuild, then re-run
`python3` diff of `target/release/madgab` against `prof/madgab-baseline` over
`prof/targets.txt × 7 configs`, comparing **score sequences** (not raw lines)
because the engine is already nondeterministic at `--top 50`.
