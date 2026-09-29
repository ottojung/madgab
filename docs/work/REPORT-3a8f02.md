# REPORT-3a8f02 — `--pool-rank` as an executable-level pool-rank query

**Item:** [w-3a8f02](items/w-3a8f02.md) · **Branch:** `madgab-poolrank-3a8f02` ·
**Base:** `f32cec6` · **Files touched:** `src/main.rs`, `tests/pool_rank_query.rs` (new).
`src/lib.rs` untouched — owned by [w-3a8f01](items/w-3a8f01.md).

**Verdict: landed.** All six completion criteria met. Default stdout is byte-identical to
base; the query surface is opt-in; absence exits 3 and is worded as a finding rather than a
failure; the cost is measured on 8 targets and self-reported by the tool.

---

## 0. Environment fact, stated once and never used as a result

This host has a 30 GiB cgroup limit. `/sys/fs/cgroup/memory.events` reported **4345 `oom` /
410 `oom_kill`** events before this front started, and a concurrent front on the same box was
SIGKILLed twice. During this front `rustc` was SIGKILLed (signal 9) while **linking** the
release `--lib` test binary, repeatedly, and the debug `--lib` binary was SIGKILLed while
**running**. Every build and test run here therefore used the prescribed workaround:

```text
CARGO_TARGET_DIR=/workspace/madgab-poolrank-3a8f02/target-front-3a8f02
CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_DEBUG=false
```

**No link or run OOM in this report is a negative result, and none is cited as one.** The
release `--lib` link was retried on a freshly restarted container and then completed:
**83 passed / 0 failed / 12 ignored**. The two SIGKILL bursts that preceded it were the
cgroup, not the code. Note also that `cargo test --release --lib` compiles `src/lib.rs` and
**never reads `src/main.rs`**, so no lib-suite result on this front could have been caused by
the change in the first place.

`cargo fmt` and `cargo clippy` are **not installed** on this host and were not run; no claim is
made about them. `CARGO_TARGET_DIR` was pinned to this front's own directory for every
invocation above, and build windows were kept short and serialized.

---

## 1. Criterion 2 first — default output is byte-identical (HARD REJECT, checked before feature work)

Base binary was rebuilt from `f32cec6:src/main.rs` into the same warm target dir and kept at
`/tmp/opencode/madgab-base`, so the comparison is base-source vs this front's source, same
toolchain, same profile.

`madgab --approximate "recognize speech"` — stdout, **identical on both builds**
(md5 `1dddc478dd680147ccd2fe86f3548745` on both):

```text
 1. [0.901] wreck a nice pitch
 2. [0.901] wreck a nice peach
 3. [0.901] let a nice pitch
 4. [0.901] let a nice peach
 5. [0.898] wreck eggs i.'s pitch
 6. [0.898] wreck eggs i.'s peach
 7. [0.898] let eggs i.'s pitch
 8. [0.898] let eggs i.'s peach
 9. [0.896] yeah keg nice pitch
10. [0.896] yeah keg nice peach
```

`madgab --approximate "It's just a stupid game"` — stdout, **identical on both builds**
(md5 `f526384f7f9083f4273ca8b37308c49c` on both):

```text
 1. [0.899] it said thus test oop dame
 2. [0.895] eat said thus test oop dame
 3. [0.891] it sad thus test oop dame
 4. [0.891] shit said thus test oop dame
 5. [0.888] eat siege us test oop dame
 6. [0.887] eat surge us test oop dame
 7. [0.887] each thus 'cause too pad aim
 8. [0.886] it justice too pah dame
 9. [0.886] each thus 'cause too pad same
10. [0.886] each thus death too pad aim
```

**No added line, no added column, no changed spacing.** `diff` on stdout is empty for both
targets.

**stderr** is byte-identical except the one timing line, which cannot be pinned because it
records wall-clock:

```text
- (corpus loaded in 628ms; search 1125ms)      # base
+ (corpus loaded in 498ms; search 1033ms)      # this front
- (corpus loaded in 505ms; search 1144ms)      # base, case 2
+ (corpus loaded in 448ms; search 1066ms)      # this front, case 2
```

**Why the default path is provably untouched, structurally.** The trailing query form is
recognized by splitting the post-flag argument vector on the literal token `--pool-rank`
*before* the target is joined:

```rust
if let Some(at) = args.iter().position(|a| a == "--pool-rank") { ... }
...
let target = args.join(" ");
```

With no trailing token the `position` lookup returns `None`, no branch is taken, and `target`
is computed by the original expression. The pre-existing annotate form (`--pool-rank` *before*
the target, which `tests/pool_rank_reporting.rs` exercises) is left exactly as it was. The
default path also still short-circuits before any second search: `needs_pool` is
`show_pool_rank || query_for.is_some()`, false for every default invocation, and the test
`the_default_invocation_stdout_is_unchanged_by_the_query_surface` asserts that no
`second search` appears on the default path's stderr.

---

## 2. Criterion 1 — the rank report, and absence as a distinct fact

`--pool-rank` already existed as an *annotate every row* flag. This front adds the **query**
form, disambiguated by position alone: `--pool-rank` **after** the target phrase names one
clue to look for.

```text
$ madgab --approximate --top 50 "recognize speech" --pool-rank "wreck a nice beach"
pool-rank: "wreck a nice beach" is pool rank 9 of 18301; displayed at 9        # exit 0

$ madgab --approximate "recognize speech" --pool-rank "wreck a nice beach"
pool-rank: "wreck a nice beach" is pool rank 9 of 13819; NOT in the display
           (generated, then ranked out of the top 10 shown)                   # exit 0

$ madgab --approximate "It's just a stupid game" --pool-rank "hits justice dupe hid came"
pool-rank: "hits justice dupe hid came" is ABSENT from the pool: not generated at
           all, out of a pool of 14549 scored candidates.                     # exit 3
  This is a finding, not a failure: the run succeeded and the clue was never built.
  Absence is not the same as ranking out of the display.
```

The second and third lines are the whole point of the front, and they are the two facts
`docs/work/OBSTRUCTION-MAP.md` §"Conflicts" item 7 records as routinely conflated. The tool now
distinguishes **all three** outcomes as separate observable facts:

| outcome | exit | wording | what it means |
|---|---|---|---|
| in pool, in display | 0 | `is pool rank N of M; displayed at D` | found, and shown |
| in pool, not in display | 0 | `is pool rank N of M; NOT in the display (generated, then ranked out…)` | **built, then outranked** — an objective question |
| absent from pool | **3** | `is ABSENT from the pool: not generated at all, out of a pool of M` | **never built** — an emission question |
| corpus load / no coverings | 1 | `madgab: no clue coverings found …` | the run failed |
| trailing form with no clue | 2 | `--pool-rank after the target needs a clue` | usage error |

### Why exit 3 for absence, justified

Absence is a **successful measurement of a negative fact**: the pool was built, and the clue
is not in it. The exit code is the load-bearing part, not decoration. A caller that reads any
non-zero exit as "the run failed" is wrong for absence, and this repository has repeatedly read
it that way — that misreading is what the OBSTRUCTION-MAP conflict entry records. Giving
absence its own code turns "the caller conflated a finding with a failure" from a reading into
a visible condition, and leaves 1 and 2 meaning what they already mean. It is also the only
choice that works for a script: the verdict line goes to **stdout** on a hit and to **stderr**
on an absence, and the three codes are all distinguishable without parsing either.

Absence is additionally never conflated with a failure *in the other direction* either: a
target that cannot be searched exits **1**, not 3, and never says `ABSENT`
(`a_run_that_cannot_build_a_pool_is_a_failure_not_an_absence`).

### Measured, and it moves a standing figure

The canonical case-1 clue is at **pool rank 9 of 18 301** (`--top 50`) / **rank 9 of 13 819**
(default `--top 10`), displayed at 9 with `--top 50` and **not** in the display at `--top 10`.
The standing green fact in the OBSTRUCTION-MAP records rank **27** for this clue; **9 is the
figure on this head and it is better, not a regression** — the WORST_WORD axis at 0.05 landed
after those numbers were taken. I record it rather than reconciling it: no verdict in any
report depends on it, and per the front's no-re-derivation rule both numbers stand.

Case 2's canonical clue now reads **ABSENT** (exit 3, pool 14 549). That is the standing
"absent from the production approximate pool" conclusion, now **visible from the binary**
instead of inferred from a library-side probe. Per the item and the OBSTRUCTION-MAP, this is
recorded as an observation, not re-used to re-open anything.

---

## 3. Criterion 3 — cost measured, on 8 targets

Wall clock, whole process including corpus load, median of 3 timed runs per point after a
warm-up run, same binary, same box:

| target | without flag | with flag | delta |
|---|---:|---:|---:|
| `recognize speech` | 1644 ms | 2534 ms | **+890 ms** |
| `It's just a stupid game` | 1550 ms | 2603 ms | **+1053 ms** |
| `Coors light` | 1262 ms | 1922 ms | **+660 ms** |
| `a fat cat` | 1064 ms | 1546 ms | **+482 ms** |
| `bright lights` | 1770 ms | 2763 ms | **+993 ms** |
| `open the door` | 1367 ms | 2163 ms | **+796 ms** |
| `wreck a nice beach` | 1627 ms | 2497 ms | **+870 ms** |
| `the quick brown fox` | 1653 ms | 2399 ms | **+746 ms** |
| **total (8 targets)** | **11.937 s** | **18.427 s** | **+6.490 s** |

**Per target: +482 ms to +1053 ms, mean +811 ms. Total over 8 targets: +6.490 s** on a base of
11.937 s, i.e. **+54 %**.

**The historical +315..+737 ms does not hold.** The new range is **+482..+1053 ms**, roughly
1.4–1.5× the recorded upper end, and the top of the range is well clear of the old ceiling. The
likely reason is the search itself: the cost is one extra `generate_pool` call, so it tracks
whatever the second search now costs, and the head has changed since the +315..+737 ms figure
was taken (WORST_WORD at 0.05 is integrated at `a7a406c`). I am not attributing the difference
to a specific change without measuring it, and the pre-WORST_WORD number is not re-derivable
on this head.

The tool now **self-reports** this cost, so it is measurable without an external stopwatch —
`(pool-rank run: second search 1048ms)` and similar, one per target, which tracked the
wall-clock deltas above to within noise. A surface whose price is only knowable by timing it
from outside cannot be weighed against the alternative of not asking the question; "opt-in" is
only a real claim if the cost of opting in is visible. Pinned by
`the_query_reports_the_second_search_it_pays_for`.

Cost is intrinsic to the design and not a defect: the pool's contents are only reachable via
`generate_pool`, which re-runs the search. The search is deterministic
(`tests/approx_determinism.rs`, `tests/exact_determinism.rs`), so the second run's pool is the
first run's pool and the reported rank describes the printed rows. That is precisely why the
flag is opt-in — the default path pays nothing for the pool *size* it already reports.

---

## 4. Criterion 4 — regression tests, new file `tests/pool_rank_query.rs`

9 tests, **9 passed / 0 failed**. Phrase literals live in `tests/`, which the fence does not
scan. The file names the two canonical strings, as `tests/corpus_integration.rs` and
`tests/pool_rank_reporting.rs` already do.

| test | pins |
|---|---|
| `a_clue_in_the_pool_is_reported_with_its_rank` | rank is 1-based, ≤ pool size, pool size stated, and the display position is named **or** its absence is named in words |
| `a_clue_outside_the_pool_is_reported_as_absent_and_not_as_a_failure` | exit **3** on both canonical targets; message says `ABSENT from the pool` and `not generated`; no rank is invented; the pool size is stated |
| `a_run_that_cannot_build_a_pool_is_a_failure_not_an_absence` | unsearchable target exits **1**, never says `ABSENT`; trailing form with no clue exits **2** and is not worded as an absence |
| `the_default_invocation_stdout_is_unchanged_by_the_query_surface` | **criterion 2 as a test**: 50 rows, bare score bracket, no `pool rank`, no `pool-rank:`, no `second search` |
| `the_query_prints_exactly_the_default_rows` | a query changes what is *reported*, never what is *produced* |
| `the_query_is_a_no_op_on_the_exact_mode` | exact-mode stdout byte-identical with and without the flag, and the flag still answers the question on the exact pool |
| `case_two_is_reported_as_present_or_absent_never_as_a_guess` | case 2's status is *readable* and never papered over |
| `the_query_reports_the_second_search_it_pays_for` | the cost is self-reported; the default path reports no second search |
| `a_typed_clue_is_matched_on_normalized_words` | `  Wreck   A Nice   Beach  ` gets the same answer as the canonical spelling |

The exact-mode test is worth one sentence: the flag must not *require* `--approximate`, and
must not change what exact mode produces. "No-op on exact mode" is pinned as **identical
stdout** plus **an answer**, so it cannot be satisfied by the flag simply doing nothing.

---

## 5. Criterion 5 — suite results

| suite | result |
|---|---|
| `cargo test --release --lib` | **83 passed / 0 failed / 12 ignored** |
| `--test no_phrase_hard_coding` | **9 passed / 0 failed** |
| `--test corpus_integration` | **12 passed / 1 failed** — the known red `approximate_finds_classic_madgab_resegmentation` (case 2) |
| `--test pool_rank_query` (new) | **9 passed / 0 failed** |
| `--test pool_rank_reporting` (pre-existing) | **5 passed / 0 failed** — unaffected |
| `cargo fmt` / `cargo clippy` | **not installed on this host; not run; no claim** |

**`approximate_finds_classic_madgab_resegmentation` was not relaxed, re-pinned or skipped.**
It is red at base and red here, with the same canonical clue missing from the top 50, and it
remains the single pre-existing failure in `--test corpus_integration` (12/1). The front makes
its status *readable* (§2, exit 3) and changes nothing about it.

**Fence: 9/0, zero `src/` allowlist entries.** `ALLOWLIST` was not edited at all;
`ALLOWLIST_CAPS` remains `("src", 0)`, so the `src/` cap is untouched and the new code adds no
allowlist entry. The flag has **no default value** — `--pool-rank` is a bare switch or a
trailing clue the user supplies — so no canonical clue is a literal anywhere in `src/`. The two
canonical strings appear in `src/main.rs` only inside the module's doc comment, which the fence
strips by design and which documents the CLI's usage.

---

## 6. Criterion 6 — this report

`docs/work/REPORT-3a8f02.md`.

---

## 7. Notes for the next pass

* The item's third pass-deferral reason — "`src/lib.rs` is owned by the live front" — is
  discharged: this front needed **no** `src/lib.rs` change, because `generate_pool` already
  existed. `w-3a8f01` can own `src/lib.rs` freely; the two fronts do not conflict.
* **Worth re-deriving when `src/lib.rs` is next touched:** every rank figure in this
  repository's older reports is ambiguous between display position and pool rank, and §2
  above shows a standing figure (27) that is now 9. The query surface is what makes that
  class of correction checkable, so the head fronts should prefer `--pool-rank` over reading
  the printed list.
* Exit code **3** is now part of the CLI's contract. Any wrapper or CI step that treats
  non-zero as "failed" will read an absent-from-pool finding as a failure; that is the
  intended behaviour of the code, not an oversight, and should be reflected in any
  automation added later.
