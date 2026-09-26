# REPORT-d3b7c2 — w-6b91d3: the default approximate path's candidate pool was not run-to-run deterministic

Front `d3b7c2`, branch `madgab-pooldet-d3b7c2`, based on `post-milestone-acceptance`
at `8d4cab1`. No merge, no rebase, no push to `main` or to
`post-milestone-acceptance`.

## 1. Localisation by line

**Hypothesis 1 is the cause, and it reaches the pool through hypothesis 2's
mechanism.** There is no parallelism anywhere in `src/` (`git grep` for
`rayon|thread::|spawn|par_iter` returns only `thread_local!` instrumentation
counters), so hypothesis 3 is refuted outright.

Two sites in the structural DP of `generate_approximate`, both
`RandomState`-seeded `HashMap` drains whose order was consumed by a
*stable* sort followed by a `truncate`:

- **`src/lib.rs:1078` at base `8d4cab1`** — `let here = std::mem::take(&mut seg_states[p]);`
  then `for ((word_count, shared), paths) in here`. `seg_states` is
  `Vec<HashMap<(usize, usize), Vec<SegPath>>>` (`src/lib.rs:1083-1085` (base `8d4cab1`: `1064-1066`),
  `HashMap::new()`, i.e. `RandomState`). Every path a state contributes is
  `push`ed onto its successor's bucket at `src/lib.rs:1137` (base: `1118`), the bucket is
  `sort_by(|a, b| cmp_desc(a.rank, b.rank))` (stable) and then
  `bucket.truncate(SEG_STATE_KEEP)` with `SEG_STATE_KEEP = 32`
  (`src/lib.rs:695` (base: `676`)). Where that cut falls inside a group of exactly-equal
  `rank`s, **which** of the tied paths survives is decided by the map's
  iteration order.
- **`src/lib.rs:1131` at base `8d4cab1`** — `for ((word_count, _shared), paths) in
  std::mem::take(&mut seg_states[n])`, feeding `segmentations`, which is
  `sort_by(cmp_desc(a.0, b.0))` (stable) and `truncate(SEGMENTATION_KEEP)`
  with `SEGMENTATION_KEEP = 256` (`src/lib.rs:696` (base: `677`)). This one is the more
  consequential of the two: `segmentations`' order is not cosmetic, it
  becomes `by_structure` → `schedule` (`src/lib.rs:1277-1309` (base: `1258-1290`)), which is the
  order `LEXICAL_GLOBAL_EMISSION_BUDGET` is spent in, so a tie resolved
  differently spends the budget on different boundary structures and leaves
  a different set of wordings in the pool.

### The arithmetic

`rank` and the `SEGMENTATION_KEEP` objective are `f64` proxies that tie
exactly, and the tie is not rare. I instrumented the base temporarily
(counters compiled out again; no trace of it remains on the branch) and got,
on `recognize speech`, **the same numbers on all eight runs**:

```text
DIAG bucket_cuts=3097 bucket_ties=182 seg_cuts=1 seg_ties=1 fp=<varies>
```

- `bucket_cuts=3097` truncations of a DP bucket to `SEG_STATE_KEEP`, of which
  `bucket_ties=182` had `rank[31] == rank[32]` — the cut fell exactly on a tie.
- `seg_cuts=1`, and that single cut to `SEGMENTATION_KEEP=256` was a tie too.
  So the pool delta reduces to **one** boundary structure entering or not
  entering the funded set, worth two wordings here.

`fp` is an FNV fingerprint of the `(word_count, shared)` key sequence drained
at `src/lib.rs:1078`, and it **differed on every one of the eight runs** while
the tie counts stayed constant. That is the direct measurement: the tie
structure is a property of the code, and the *choice within it* is a property
of the process's hash seed. The pool count tracked it, alternating 12606 /
12604 with a byte-identical visible top-20, exactly as the coordinator
reported.

### Sites checked and cleared

`build_lexicon`'s `HashMap<String, RawEntry>` (`src/approx.rs:383`) is
re-sorted before the trie is built (`src/approx.rs:415`); `found` in
`matches_at` (`src/approx.rs:99`, drained at `:184`) is sorted by a key that
is total within a span (`src/approx.rs:194`, cost → rarity → word, unique per
`word_idx`); the span shortlist's axes (`src/lib.rs:801-928`) sort over
inputs that are already ordered `Vec`s; `funded` and `where_`
(`src/lib.rs:1312`, `:1321`) are only ever read by key. The DP state map was
the one that decided retention. The `seen` set of `7faa74c`
(`src/lib.rs:1604`, `BuildHasherDefault<FxHasher>`) was already fixed and was
not involved.

## 2. The fix

`src/lib.rs` only, three edits, all inside the structural DP, all general:

1. `src/lib.rs:1108-1128` — the state map is drained through
   `into_iter().collect()` into a `Vec` and `sort_by`ed on the
   `(word_count, shared)` key before use. The key is unique, so this is a
   total order and the arrival order is a function of the state keys alone.
2. `src/lib.rs:1161-1171` — `bucket.sort_by` is now
   `cmp_desc(a.rank, b.rank).then_with(|| a.spans.cmp(&b.spans))`.
   `spans` is unique within a state (the DP never records the same chain
   twice), so the sort is total and the retained set no longer depends on
   arrival order at all.
3. `src/lib.rs:1181-1204` — the same drain-and-sort on `seg_states[n]`, and
   `segmentations.sort_by` tie-breaks on `spans` the same way.

Both halves are deliberate: sorting the iteration removes the seed
dependence, and the total order on the key means a future hash order could
not reintroduce it even if the drain were reverted. No target-specific,
phrase-specific or input-specific case; no `MADGAB_*` / `ZZ_*` / `zz_*`
scaffolding anywhere on the branch. **No retention constant was moved** —
`SEG_STATE_KEEP`, `SEGMENTATION_KEEP` and the emission budget accounting are
untouched (verified by scanning the diff for changed `const` declarations:
none). Nothing in `EMIT_PROFILE_MAX_DEEP`, `coverage_tuples`, `sweep_index`,
`axes::*`, `select_diverse`, the share cap or `STRUCTURE_FLOOR` was edited;
the two diff lines that mention `select_diverse` are the pre-existing call
site wrapped in the new return tuple and one doc reference. The diff does not
touch `coverage_tuples` or `sweep_index`, so it does not contend with the
live w-2f7a10 fronts in that territory; `src/lib.rs` is shared but the hunks
are at lines 424-459, 1108-1204 and 1853-1882, well clear of the
`coverage_tuples` function.

One public API addition, because pool size was otherwise unobservable: a one-
line `generate` delegating to a new `generate_with_pool` (`src/lib.rs:424`, `:443-459`), with `finish` returning `(Vec<Clue>, usize)` where the `usize`
is the deduplicated pool length after phrase-dedup and before
`select_diverse`. This is the denominator the project's admissibility rule is
written against, and it is what lets the test assert on it.

## 3. Reproducibility

Release binary on this head, `--approximate --top 20`, pool count read from
the pre-existing `MADGAB_TRACE_PHRASES` probe line (probe set to a
non-word string, so the trace reports the pool rather than a rank — no new
env surface was added). **20 consecutive runs per target:**

| target | head, 20 runs | base `8d4cab1`, 20 runs |
|---|---|---|
| `recognize speech` | 12605 ×20 | 12604 ×10, 12606 ×10 |
| `a whole lot of trouble` | 12484 ×20 | 12480 ×5, 12482 ×5, 12486 ×4, 12488 ×6 |
| `it's just a stupid game` | 14551 ×20 | 14554 ×12, 14555 ×8 |
| `the quick brown fox jumps` | 13535 ×20 | 13531 ×10, 13532 ×10 |

The base is non-identical on **4 of 4** targets; the head is identical on
4 of 4. `a whole lot of trouble` shows the base spanning four distinct pool
sizes, so the noise band on the pre-fix binary was wider than the ±1 the
coordinator saw.

Note honestly: the head's counts are not equal to either base count. Making
an arbitrary choice well-defined necessarily picks one of the two members of
each tie, so the pool moves by 1-2 candidates on three of the four targets
(`12605` vs `12604`/`12606`, `12484` vs `12480`-`12488`, `14551` vs
`14554`/`14555`) and is unchanged on the fourth. Per this item's own rule
that movement was previously undecidable noise; it is now a single
reproducible number. **Any other front that recorded a pool count on the base
should re-measure on this head rather than compare across the tie fix.**

## 4. `tests/approx_determinism.rs`

`approximate_pool_is_reproducible_across_processes` compares, across four
fresh processes per target, both the deduplicated pool size (via
`generate_with_pool`) and the visible proposal list. Four targets, all
ordinary sentences, none of them either acceptance phrase: `recognize
speech`, `a whole lot of trouble`, `it's just a stupid game`, `the quick
brown fox jumps`. The module doc now records that the visible list was
already reproducible while the pool was not, and that the pre-existing
argument for why the seed could not reach the answer was the argument that
was wrong.

A subprocess is the whole point: within one process the state map has one
seed for the process lifetime, so an in-process comparison would hold
trivially. The helper is a `#[test]` that returns immediately unless the
driver names it, selected by a test-local env var
(`APPROX_POOL_HELPER_TARGET`, deliberately **not** `MADGAB_*`-prefixed), so
running the suite normally costs nothing and `src/` gains no knob.

**Positive control.** I extracted `8d4cab1` to `/workspace/madgab-base`,
ported the same guard with the pool count read through the CLI trace (the
base has no `generate_with_pool`), and it fails:

```text
test approximate_pool_is_reproducible_across_processes ... FAILED
assertion `left == right` failed: pool for "recognize speech" was 12606 then 12604
```

So the test is a guard, not a restatement of the change.

No existing assertion was edited or re-baselined. The only edits to the
pre-existing tests are two paragraphs of module documentation.

## 5. Suites

| suite | result |
|---|---|
| `cargo test --release --lib` | **ok — 50 passed, 0 failed** |
| `cargo test --release --test corpus_integration` | 10 passed, **1 failed** (`approximate_finds_classic_madgab_resegmentation`, red on the base too — see §6) |
| `cargo test --release --test exact_determinism` | **ok — 1 passed** |
| `cargo test --release --test approx_determinism` | **ok — 4 passed** |

`cargo fmt`, `cargo clippy` and doctests do not exist on this host
(`docs/environment-notes.md`); I did not run them and make no claim about
them. `cargo test --release --lib` was re-run after the final edit to the
test file only, so the library result stands for the committed source.

## 6. Guards

- **`recognize speech` → `wreck a nice beach`: still generated and visible.**
  Rank **28** at `--approximate --top 50`, score `[0.918]` — identical rank and
  score on the base. `corpus_integration`'s
  `approximate_finds_recognize_speech_resegmentation` is green.
- **`approximate_finds_classic_madgab_resegmentation`: still red, for the
  same reason.** It fails at `tests/corpus_integration.rs:136` with
  `canonical clue missing from top 50`, and the 12 proposals it prints are
  **byte-identical on base and head**:
  `["it justice too bad aim", "it justice too pad aim", "it justice too bad same", "it justice too peg aim"…]` — I ran it on both trees and diffed. It is not green by accident, and it is not newly failing for a different reason: the canonical clue is 0 pool members on both, so the tie fix did not move it either way.

## 7. What I did NOT settle

- I did not re-run the whole 20×4 measurement after the very last commit; the
  only change after the table in §3 was renaming a test-local env var, which
  is in `tests/` and not in the search. The library source measured in §3 is
  the committed source.
- I did not audit `src/adjacency.rs` for the same class of defect. Its `seen`
  and `held` are `HashSet`s, but I traced their consumption and the operator
  is seeded from the deterministic `pooled` vec and bounded by an explicit
  pop budget; I did not prove it by measurement, and I did not instrument it.
  If w-6b91d3 is to be closed rather than merged, that is the loose end.
- `f64` equality is still the underlying reason ties exist. I gave the two
  cuts a total order; I did not change the ranking to make ties rarer, which
  would have moved the pool and is out of scope.
- The pool moved by 1-2 candidates on three of four targets relative to the
  base. I have not audited whether any previously published reachability
  table in `docs/work/` depends on a specific pre-fix count; that is a
  coordinator-level sweep, and this item's own noise-band rule says such
  deltas were never decidable anyway.

## 8. Next action

Merge into `post-milestone-acceptance`, then re-measure any pool-membership
number quoted on the base — the tie fix changed the pool by 1-2 candidates on
three of the four targets I checked, and every reachability claim in the
project is denominated in exactly that quantity. Criterion 5 of the item is
met with the one standing red test that w-2f7a10 already records as red.

MERGE RECOMMENDATION: MERGE
