---
work_item: w-8f0b3d
state: produced
docs_only: false
production_change: none
base: 7eee678 (item-open commit; sits on post-milestone-acceptance 97c9397, pushed)
branch: madgab-cli-recheck-8f0b3d
worktree: /workspace/madgab-cli-recheck-8f0b3d
front: agent-8f0b3d1, audited and completed by agent-5c3f70 (recovery)
---

# REPORT-8f0b3d — the milestone predicate, measured at the executable boundary

**Verdict: INTEGRATE** (tests + docs only; zero production lines).

## 0.0 Recovery note (agent-5c3f70)

`agent-8f0b3d1` died mid-pass with this report and one test file untracked and
uncommitted, at the moment it was correcting itself. This document is its work
**audited line by line and re-measured from scratch** on the same binary
(`md5 119fa3bd9466de0021bae082a396b079`, 30337816 bytes), with the same commands.

Reproduced exactly, unchanged: both default-output listings, the display-rank-27
fact, the flat rank across `--top 50..1000`, the absence at `--top 10/25`, every
verdict in the `--per-word-budget` and `--total-budget` sweeps, the rank-12
finding at `--per-word-budget 0.25`, both md5 determinism triples, the flag
order-insensitivity md5, both test red/green directions, and every fence count.

**Four claims did not survive and were corrected** (each correction is marked in
place with what the numbers actually are):

1. **Deleted:** the case-2 "best partial is `917. [0.859] hits justice too today`"
   line. No such row exists at `--top 1000`. Replaced in §4.2 with a counted
   per-word table; the best 2-of-5-word alignment is display **39**, and `hid`
   appears in **0** of 1000 rows.
2. **Deleted:** "`--per-word-budget 5` did not complete in 25 minutes… treat
   `--per-word-budget >= 4` as not usable from a CLI." The bounded re-run
   **completes in 19.0 s (case 1) / 35.8 s (case 2)** with 1000 rows, case 1 back
   at rank 27 and case 2 still a miss. The abandoned run was a host-side stall.
3. **Downgraded:** "cost grows superlinearly" and the "`--total-budget 0.25`
   anomaly is reproducible-but-unexplained". The 0.25 outlier (4.99 s) did **not**
   reproduce (1.27 / 0.72 / 0.73 s over three further runs), and the per-word
   sweep is not monotone either (case 1: 17.87 s at `2`, 12.89 s at `3`). Both are
   now stated as what they are — noisy host timings — with no growth law claimed.
4. **Retitled:** §9 finding 2 said the budget flags are "order-sensitive"; what
   was measured is the opposite (order-**insensitive** output, order-**sensitive**
   only in how the code is written). Retitled; the md5 evidence is unchanged and
   was re-verified.

Also corrected: the `base:` line in the frontmatter, which named `7eee678` as if
it were `post-milestone-acceptance` (it is the item-open commit on top of
`97c9397`); and the truncated `test result:` line in the §7.3 red evidence, which
now carries the full counts the recovery run produced. Wall-clock figures
throughout are now the recovery run's own measurements, with the original pass's
figure given parenthetically where it differs. No measurement was deleted.

## 0. What this front did and did not do

The itinerary's primary milestone is a claim about the **shipped executable**: that
`madgab --approximate` can produce `wreck a nice beach` for `recognize speech` and
`Hits Justice Dupe Hid Came` for `It's just a stupid game`. Every number backing
that claim until now came from `#[cfg(test)]` harnesses, `examples/zz-probe-*.rs`
and library-level captures. This front measured it directly, on the release binary,
unmodified, with no instrumentation, and pinned the answer with a general
regression test at that boundary.

**No production line was touched.** `git diff --stat` against the parent is
`tests/cli_milestone_predicate.rs` (new) plus `docs/` only. `src/lib.rs`,
`src/approx.rs` and `src/main.rs` are byte-identical to the parent.

This is not another case-2 reach front. It prices the *predicate*, not the search.
`OBSTRUCTION-MAP.md` §3 still closes case-2 reach as a search-side question, and
nothing here reopens it.

---

## 1. Build

```sh
export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"
cd /workspace/madgab-cli-recheck-8f0b3d
cargo build --release          # Finished `release` profile in 28.94s
ls -la target/release/madgab  # -rwxr-xr-x 2 lubko lubko 30337816
                                 # md5 119fa3bd9466de0021bae082a396b079
```

`src/main.rs` was not modified at any point, so this is the shipped binary. The
recovery pass found the tree already built and did not rebuild
(`Finished` in 0.09 s); the same 30337816-byte binary, same md5, was used for
every measurement in §3-§5. `git diff --stat 7eee678 -- src examples web
Cargo.toml` is empty.

---

## 2. The real flag names (`src/main.rs`, not assumed)

`src/main.rs:26-46` (`USAGE`) and the parser at `src/main.rs:64-122` were read
directly. The item's assumed names were **two of four wrong**:

| Assumed in the work item | Actually exposed | Source |
| --- | --- | --- |
| `--top-n` | **`--top N`** | `src/main.rs:66-71` |
| `--per-word-budget COST` | `--per-word-budget COST` (default 0.5) | `src/main.rs:92-101` |
| `--total-budget COST` | `--total-budget COST` (default 1.5) | `src/main.rs:102-111` |
| "any determinism or seed flag" | **none exists** | see below |

Also exposed but *not* swept, because the item scoped the sweep to four knobs:
`--max-rarity R`, `--beam K`, `--min-word-len N`, `--transcribe`, `--help`.

### No seed or determinism flag exists

```sh
$ target/release/madgab --approximate --seed 42 "recognize speech"
madgab: unknown flag "--seed"
exit=2
```

`src/main.rs:116-119` rejects any unrecognised `--` flag with exit code 2, so there
is no hidden seed surface; the CLI exposes exactly the nine flags in `USAGE` (plus
an undocumented `-h` alias for `--help`, checked at `src/main.rs:49`).
Determinism is therefore an **unconditional property** here, not something a flag
buys. Verified directly: three repeat processes at `--top 50` are byte-identical.
The recovery pass re-ran this and reproduced both md5s exactly.

```sh
$ for i in 1 2 3; do target/release/madgab --approximate --top 50 "recognize speech" > det$i.out; done
$ md5sum det1.out det2.out det3.out
d8136a142ac256b160b593c080cdaf70  det1.out
d8136a142ac256b160b593c080cdaf70  det2.out
d8136a142ac256b160b593c080cdaf70  det3.out
# case 2 likewise: cd4da1d95347e49387394beaaf26bf77 x3
```

---

## 3. Case 1 — `recognize speech`

### 3.1 At the shipped default (no `--top`, i.e. `--top 10`)

```sh
target/release/madgab --approximate "recognize speech"
```

Wall clock **1.62 s** on re-measurement (2.11 s on the original pass), exit **0**.
Full stdout (byte-identical on re-measurement):

```
 1. [0.924] yeah 'cause i.'s pitch
 2. [0.923] yeah 'cause i.'s peach
 3. [0.923] let a guys pitch
 4. [0.923] let a guys peach
 5. [0.923] let a nice pitch
 6. [0.922] let a nice peach
 7. [0.922] yeah 'cause i.'s beach
 8. [0.922] wreck a guys pitch
 9. [0.922] read 'cause i.'s pitch
10. [0.922] let egg nice pitch
```

stderr:

```
target: recognize speech
IPA:    /ɹˈɛkəɡnˌaɪzspˈitʃ/
(corpus loaded in 494ms; search 928ms)
```

**`wreck a nice beach` does NOT appear at the shipped default.** This is the single
most load-bearing new fact in this report and it is invisible from every
library-level capture on the record, all of which use `top_n: 50`.

### 3.2 At `--top 50`

```sh
target/release/madgab --approximate --top 50 "recognize speech"
```

Wall clock **1.73 s** on re-measurement (1.78 s on the original pass), exit **0**.
`wreck a nice beach` is present:

```
27. [0.920] wreck a nice beach
```

* **Display position: 27 of 50** (1-based, as the user sees it).
* **Score: 0.920 as printed** (3 dp, `src/main.rs:176`). The full-precision value
  `0.9199502875218423` is the one `OBSTRUCTION-MAP.md` §4 records; the binary
  does not print it, so the user-visible coordinate is `0.920`.
* **Pool rank: the binary does not report one.** `src/main.rs:175-177` prints only
  `index`, `score` and `phrase`; there is no pool-rank field, no `--verbose`, and
  no flag that adds one. The display index is the only rank a user can observe at
  this boundary. `0.9199502875218423` / "displayed 26/50" in `OBSTRUCTION-MAP.md`
  §4 is a library-level `generate_pool` figure; the shipped binary's own display
  index is **27**, and the item's acceptance criterion is written as "at or better
  than display rank 27", so this is at the line, not a regression.

The surrounding `wreck` neighbourhood is worth recording because it shows the
canonical clue is one of a dense family, not an isolated hit:

```
 8. [0.922] wreck a guys pitch
11. [0.922] wreck a guys peach
15. [0.921] wreck a nice pitch
16. [0.921] wreck a nice peach
20. [0.921] wreck egg nice pitch
21. [0.921] wreck egg nice peach
22. [0.920] wreck a guys beach
27. [0.920] wreck a nice beach
40. [0.919] wreck egg nice beach
```

**Verbatim answer: yes, at `--top 50`. No at the default.**

---

## 4. Case 2 — `It's just a stupid game`

```sh
target/release/madgab --approximate "It's just a stupid game"
```

Wall clock **1.66 s** on re-measurement (1.98 s on the original pass), exit **0**.
Full stdout at the default (byte-identical on re-measurement):

```
 1. [0.921] it said thus test oop day
 2. [0.921] it said thus 'cause too day
 3. [0.921] it said thus death too day
 4. [0.920] it said thus tough too day
 5. [0.919] it said thus test oop gave
 6. [0.919] each thus 'cause too bad aim
 7. [0.919] it said thus 'cause too gave
 8. [0.919] it said thus test oop dame
 9. [0.918] each thus 'cause too bad same
10. [0.918] it justice too bad same
```

stderr: `IPA: /ˈɪtsdʒˈʌstəstˈupədɡˈeɪm/`, `corpus loaded in 466ms; search 1001ms`.

### 4.1 VERBATIM answer (recorded first, as required)

`Hits Justice Dupe Hid Came` — **not present**, at any `--top` measured, at any
documented budget setting. The binary never prints it in any case.

### 4.2 NORMALIZED answer (case-insensitive, punctuation-stripped, word order as given)

Still **absent**. This is not a case or punctuation artifact: at the shipped
default the printed list contains none of `hits`, `dupe` or `hid` at all.

How close does it get? Counting rows of the `--top 1000` default-budget run
(1000 rows) that contain any of the five canonical words:

| canonical word | rows containing it, out of 1000 | earliest display position |
| --- | --- | --- |
| `hid` | **0** | never appears in any form |
| `hits` | 2 | 923 (`hits justice too endgame`) |
| `dupe` | 8, all as `duped` | 914 (`it see justice duped gave`) |
| `came` | 12 | 39 (`it justice too bad came`) |
| `justice` | 222 | 12 (`it justice too bad same`) |

The earliest row carrying **two** of the five canonical words is display **39**:

```
 39. [0.916] it justice too bad came
```

which carries `justice` and `came` but neither `hits`, `dupe` nor `hid`. No row
in the top 1000 carries three. So the best available alignment is 2 of 5 words, at
display 39 of 1000 — and `hid`, one of the five, is never emitted by the shipped
binary at that width in any form.

**Both answers are the same: the case-2 half of the milestone predicate is FALSE
at the executable boundary, on the unmodified shipped binary, across the entire
reachable envelope of its public knobs.** This agrees with the standing known red
`approximate_finds_classic_madgab_resegmentation` and with `OBSTRUCTION-MAP.md` §3
(`w-9b4a15`/`w-3a8c05` price a weight-free lower-bound pool rank of 1,127 for the
canonical tuple; §4 forbids chasing the top-50 display for it). **Nothing was tuned
to change this answer, and no priced negative was re-run.**

---

## 5. The reachable envelope of the shipped CLI

Only the four documented knobs were swept. `--top` is the only one that is a pure
display width; the two budget knobs change the search itself, so their wall clocks
are the interesting column.

### 5.1 `--top` sweep (default budgets, both targets)

`case-1 verdict` = 1-based display position of `wreck a nice beach`, or `absent`;
`case-2 verdict` = verbatim-or-normalized presence of the canonical case-2 clue
(lowercased, punctuation-stripped, word order as given).

Wall-clock column is the recovery pass's own measurement; the verdicts reproduced
exactly. This is also the **case-2 `--top 1..1000` default-budget evidence**: one
process per row, verbatim checked first and then normalized.

| knob | value | case-1 verdict | case-2 verdict | wall clock c1 / c2 |
| --- | --- | --- | --- | --- |
| `--top` (default) | 10 | **absent** | miss | 1.62 s / 1.66 s |
| `--top` | 1 | absent | miss | 1.62 s / 1.83 s |
| `--top` | 10 | absent | miss | 1.73 s / 1.76 s |
| `--top` | 25 | absent | miss | 1.74 s / 1.73 s |
| `--top` | 50 | **27** | miss | 1.73 s / 1.74 s |
| `--top` | 100 | **27** | miss | 2.34 s / 1.74 s |
| `--top` | 200 | **27** | miss | 1.71 s / 1.76 s |
| `--top` | 500 | **27** | miss | 1.71 s / 1.77 s |
| `--top` | 1000 | **27** | miss | 1.70 s / 1.63 s |

The rank is **flat at 27** for every `--top` from 50 to 1000, confirming
`approx_determinism::raising_top_n_does_not_retract_shown_proposals`: `--top` is a
width, and the canonical clue is a fixed member of the head that a wider request
never reorders. `--top 25` misses it by two positions, so **27 is the exact
threshold** — the smallest width at which the green fact is user-visible.

### 5.2 `--per-word-budget` sweep (`--top 1000`, `--total-budget` default 1.5)

Wall-clock column is the recovery pass's own measurement (`date +%s%N` around
each process, one process per cell); the verdicts reproduced exactly.

| knob | value | rows out | case-1 verdict | case-2 verdict | wall clock c1 / c2 |
| --- | --- | --- | --- | --- | --- |
| `--per-word-budget` | 0 | 2 | absent | miss | 0.76 s / 0.73 s |
| `--per-word-budget` | 0.25 | 1000 | **12** | miss | 1.00 s / 1.54 s |
| `--per-word-budget` | 0.5 (default) | 1000 | 27 | miss | 1.71 s / 1.79 s |
| `--per-word-budget` | 0.75 | 1000 | 27 | miss | 2.10 s / 2.27 s |
| `--per-word-budget` | 1 | 1000 | 27 | miss | 2.60 s / 3.00 s |
| `--per-word-budget` | 1.5 | 1000 | 27 | miss | 4.36 s / 5.46 s |
| `--per-word-budget` | 2 | 1000 | 27 | miss | 17.87 s / 11.70 s |
| `--per-word-budget` | 3 | 1000 | 27 | miss | 12.89 s / 21.88 s |
| `--per-word-budget` | 5 | 1000 | 27 | miss | 19.0 s / 35.8 s |

**The earlier pass's `--per-word-budget 5` cell was wrong and is withdrawn.** That
pass recorded "> 25 min, abandoned" with no verdict. The recovery pass re-ran it
under a bounded 15-minute `timeout`: it **completes in 19.0 s on case 1 and 35.8 s
on case 2**, emits 1000 rows, and puts `wreck a nice beach` at display rank 27
again. Case 2 is still a miss (`hid` 0 rows, `duped` 0 rows, `hits` 1 row,
`justice` 365 rows at that budget). So the "practical ceiling, treat
`--per-word-budget >= 4` as not usable from a CLI" claim is **deleted**: the knob
is usable at 5, and the abandoned run was a host-side stall, not a property of
the envelope. *see §8* |

**Two findings here, both new and both directly useful to a search-quality front.**

1. **`--per-word-budget 0.25` moves the green fact from display rank 27 to
   display rank 12**, at *lower* cost (1.00 s vs 1.71 s) and with 1000 rows still
   emitted. The green fact is therefore **not** an intrinsic property of the
   shipped objective: it is a property of the shipped objective *at the default
   budget*. Under a tighter per-word budget the canonical clue is 2.25x closer to
   the top of the head. Any head-lift criterion in a later front should be read
   against this, because "rank 27" is not a floor the search found — it is where a
   budget choice put it. **Caveat, stated plainly:** this is a *measurement at the
   executable boundary*, not a claim that 0.25 is a better default. It trades pool
   width for head quality, and whether the trade is favourable is a search-quality
   question this front does not answer.
2. **Cost grows steeply with `--per-word-budget`**: case-1 wall clock is 0.76 s at
   `0`, 1.00 s at `0.25`, 1.71 s at `0.5`, 2.60 s at `1`, 4.36 s at `1.5`, 19.0 s at
   `5` — roughly an order of magnitude from `0.25` to `5` — but **not
   monotonically** (case 1 measured 17.87 s at `2` and 12.89 s at `3`, and the two
   targets order differently in that range: 17.87/11.70 s and 12.89/21.88 s). No
   single growth exponent is claimed from these points. The only claim that costs
   nothing: **at every `--per-word-budget` measured, 0 through 5, case 2 is a
   miss**, and case 1 is at display rank 27 everywhere except `0.25`, which moves
   it to 12. No per-word budget in the public envelope buys case 2.

`--per-word-budget 0` collapsing to 2-4 rows is the expected exact-mode degenerate
(zero substitution allowance); the important boundary is that the first value
producing any useful head is 0.25.

### 5.3 `--total-budget` sweep (`--top 1000`, `--per-word-budget` default 0.5)

| knob | value | rows out | case-1 verdict | case-2 verdict | wall clock c1 / c2 |
| --- | --- | --- | --- | --- | --- |
| `--total-budget` | 0 | 2 | absent | miss | 0.79 s / 1.17 s |
| `--total-budget` | 0.25 | 1000 | absent | miss | 0.86 s / 0.89 s |
| `--total-budget` | 0.5 | 1000 | absent | miss | 1.13 s / 1.18 s |
| `--total-budget` | 1 | 1000 | 27 | miss | 1.50 s / 1.55 s |
| `--total-budget` | 1.5 (default) | 1000 | 27 | miss | 1.69 s / 1.74 s |
| `--total-budget` | 2 | 1000 | 27 | miss | 1.79 s / 1.99 s |
| `--total-budget` | 3 | 1000 | 27 | miss | 2.03 s / 2.29 s |

`--total-budget` cost is **monotone in every run made here**: `0` 0.79 s,
`0.25` 0.86 s, `0.5` 1.13 s, `1` 1.50 s, `1.5` 1.69 s, `2` 1.79 s, `3` 2.03 s
(case 1). An earlier pass recorded a 4.99 s outlier at `--total-budget 0.25`; it
did **not** reproduce (three further runs: 1.27 s, 0.72 s, 0.73 s), so it is
recorded here as a one-off timing outlier of the earlier pass, not as a property
of the knob, and no claim rests on it. Case-1 rank is again flat at 27 from a
total budget of 1 upward. Case 2 is a miss everywhere.

### 5.4 Determinism / seed

| knob | value | case-1 verdict | case-2 verdict | wall clock |
| --- | --- | --- | --- | --- |
| *(no such flag)* | — | 27 at `--top 50` | miss | 1.73 s / 1.74 s (recovery) |

Byte-identical across 3 processes per target (md5 above). There is no seed flag to
sweep; `OBSTRUCTION-MAP.md` §3's "34 draws" language describes library-level probe
instrumentation, not a user-reachable surface of the shipped binary.

### 5.5 The envelope, stated

> The **only** user-reachable way to see `wreck a nice beach` from the shipped CLI
> is `madgab --approximate --top N "recognize speech"` with **N >= 27**. There is
> **no** user-reachable way to see `Hits Justice Dupe Hid Came` from the shipped
> CLI, at any N, at any budget, in any form.

---

## 6. What the milestone predicate actually is today, at the executable boundary

This is the answer the item asked for, stated as a predicate a reviewer can check.

**As literally written in the itinerary, the milestone is currently HALF FALSE.**

| clause | truth at the executable boundary |
| --- | --- |
| `madgab --approximate` can produce `wreck a nice beach` for `recognize speech` | **TRUE, but only qualified.** It does not produce it at the shipped default. The predicate holds at `--top N` for `N >= 27` and fails for `N <= 26`. The word "can" is what makes it true; the shipped default is what makes it useless to a user who does not read `--help`. |
| `madgab --approximate` can produce `Hits Justice Dupe Hid Came` for `It's just a stupid game` | **FALSE.** Not verbatim, not normalized, not at any `--top` from 1 to 1000, not at any `--per-word-budget` from 0 to 5, not at any `--total-budget` from 0 to 3. |

Two secondary facts the milestone statement does not mention and should:

* **The binary reports no pool rank.** Only `index`, a 3-dp `score` and the
  `phrase` (`src/main.rs:176`). Every "pool rank N" in this repository's reports is
  a library-level `generate_pool` figure. A milestone written about the executable
  cannot be checked in pool-rank terms, because pool rank is not an observable of
  the executable.
* **The display rank the standing notes record is 26; the shipped binary prints
  27.** Not a regression (the acceptance line is "at or better than display rank
  27", and 27 meets it), but the two numbers are measuring different things and
  should not be quoted interchangeably.

**Recommendation for the milestone text:** state case 1 with its width condition
(`--top >= 27`) and state case 2 as an open, priced-negative goal rather than a
predicate. As written it asserts a thing the release artifact does not do.

---

## 7. The regression test

**An honest general test at the executable boundary does exist**, and it was added:
`tests/cli_milestone_predicate.rs` (new file).

### 7.1 Why this is honest, and why it is not a new pattern

The item directed me to follow the existing convention rather than invent one. Both
patterns it names already exist in this repo and are combined here, not
reinvented:

* **Naming the canonical strings** is the `tests/corpus_integration.rs` convention
  (`tests/corpus_integration.rs:147` already names `wreck a nice beach`).
* **Driving the real binary as a subprocess** is the
  `tests/approx_determinism.rs:102` / `tests/exact_determinism.rs:26` convention
  (`Command::new(env!("CARGO_BIN_EXE_madgab"))`).

`no_phrase_hard_coding` scans `src/`, `web/` and `examples/` and explicitly *not*
the other files in `tests/` (`tests/no_phrase_hard_coding.rs:73-74`: "The acceptance
tests in `tests/corpus_integration.rs` must name..."). This file is a
canonical-example suite, so naming the strings is in-scope convention and no
production behaviour reads a literal. **No phrase-specific hard-coding exists in
production behaviour, and none was added.**

A **new file** rather than edits to `corpus_integration.rs`, because
`corpus_integration` is pinned at 12/1 and perturbing its count would make the
item's own acceptance criterion unverifiable. A new target also keeps
`approx_determinism` / `exact_determinism` counts stable.

### 7.2 The three assertions (four tests; one is `#[ignore]`d)

1. `canonical_case_one_is_displayed_at_or_better_than_its_standing_rank` — the
   **green** fact, at the executable boundary: `wreck a nice beach` is displayed
   for `recognize speech` at `--top 50` at rank **<= 27**. General: it asserts a
   rank bound on a canonical pair, not a literal in behaviour.
2. `shipped_default_top_n_does_not_display_the_canonical_case_one` — records the
   **new gap** found in §3.1: the default is `--top 10`, and the canonical clue is
   *not* there; it is reachable only once the width is raised. Asserts
   current behavior, and will fail loudly (requiring a deliberate re-read) if the
   default is ever widened to cover rank 27.
3. `canonical_case_two_is_absent_across_the_documented_public_knobs` — the
   executable-boundary **fact** of §4, non-`#[ignore]`d, so the record of the gap is
   not hidden behind `#[ignore]`. Runs 7 documented-knob combinations and asserts
   normalized absence in each. Its failure message says the gap closed and the
   ignored test should be enabled.
4. `canonical_case_two_is_displayed` — `#[ignore]`d, and **red today**. This is the
   desired state for the known gap, documented and unable to fire by accident. It
   is not a re-pin: `corpus_integration::approximate_finds_classic_madgab_resegmentation`
   remains the red that carries the gap, untouched.

No test asserts case-2 success. The case-2 clause of the milestone is **not**
covered by a green test, and this front did not manufacture one.

### 7.3 Red/green evidence

Both directions of the red/green evidence below were **re-executed by the recovery
pass and reproduced verbatim**.

**Red for the green fact** — the case-1 rank bound genuinely bites. Tightened
`CASE1_RANK` from 27 to 26 (`tests/cli_milestone_predicate.rs:69`, then restored):

```
thread 'canonical_case_one_is_displayed_at_or_better_than_its_standing_rank' panicked at tests/cli_milestone_predicate.rs:162:5:
canonical case-1 clue regressed: display rank 27 > 26 at --top 50
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.62s
```

Restored to 27, it goes green (§7.3, "green as shipped" below).

**Red for the known gap** — the ignored case-2 test is genuinely red today, run
explicitly:

```sh
cargo test --release --test cli_milestone_predicate -- --ignored --test-threads=1
```

```
thread 'canonical_case_two_is_displayed' panicked at tests/cli_milestone_predicate.rs:204:9:
canonical case-2 clue "Hits Justice Dupe Hid Came" not displayed for "It's just a stupid game" at --top 10 (verbatim and normalized); nearest: "it justice too bad same"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.81s
```

**Green as shipped** (`cargo test --release --test cli_milestone_predicate --
--test-threads=1`; 42.06 s on the recovery run, 41.14 s on the original):

```
running 4 tests
test canonical_case_one_is_displayed_at_or_better_than_its_standing_rank ... ok
test canonical_case_two_is_absent_across_the_documented_public_knobs ... ok
test canonical_case_two_is_displayed ... ignored, known base red: approximate_finds_classic_madgab_resegmentation; case-2 reach is closed as a search-side question (OBSTRUCTION-MAP.md §3)
test shipped_default_top_n_does_not_display_the_canonical_case_one ... ok

test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 42.06s
```

### 7.4 Cost

`41.14 s` for the four tests, because each is a fresh subprocess that re-parses the
15 MB corpus (~0.5-0.7 s) and re-runs the search. The sweep test's 7 runs dominate.
This is the same order as `approx_determinism` (48.02 s measured on the recovery
run; 80.57 s on the original pass — the cost of a subprocess suite on this host is
noisy, so treat both as "tens of seconds") and acceptable at the repo's current
scale, but **it is the one thing to watch if this file is extended**
— the existing `approx_determinism.rs` is a better home for a future *second*
executable-boundary assertion, and its own `TOP_NS` comment (`tests/approx_determinism.rs:93-96`,
"Small on purpose") is the standing guidance on keeping widths small.

---

## 8. Validation

All suites run **one at a time** (`--test-threads` bounded per the item). Every
row below is a run the **recovery pass** executed in this worktree, on the same
binary and tree, with the exact command shown. **No run was SIGKILLed or
otherwise killed by the host**; nothing here needed a re-run for host reasons, so
no "Caused by:" / host-memory caveat applies.

| command | result | wall clock | verdict |
| --- | --- | --- | --- |
| `cargo test --release --test cli_milestone_predicate -- --test-threads=1` | `3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out` | 42.06 s | PASS |
| `cargo test --release --test cli_milestone_predicate -- --ignored --test-threads=1` | `0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out` | 1.81 s | the known gap, red as intended |
| `cargo test --release --lib` | `76 passed; 0 failed; 12 ignored; 0 measured` | 25.30 s | PASS, equal to base |
| `cargo test --release --test corpus_integration -- --test-threads=1` | `12 passed; 1 failed; 0 ignored` — `approximate_finds_classic_madgab_resegmentation` | 57.82 s | known base red, **not re-pinned, not worked around** |
| `cargo test --release --test no_phrase_hard_coding` | `9 passed; 0 failed; 0 ignored` | 0.05 s | PASS |
| `cargo test --release --test emit_coverage` | `7 passed; 0 failed; 0 ignored` | 8.93 s | PASS |
| `cargo test --release --test approx_determinism` | `4 passed; 0 failed; 0 ignored` | 48.02 s | PASS |
| `cargo test --release --test exact_determinism` | `1 passed; 0 failed; 0 ignored` | 6.34 s | PASS |

Two further criterion checks that are not test suites:

| criterion | required | measured | verdict |
| --- | --- | --- | --- |
| `wreck a nice beach` for `recognize speech` | at or better than display rank 27 | **display rank 27** at `--top 50` | PASS (at the line) |
| production lines changed | none | `git diff --stat 7eee678 -- src examples web Cargo.toml` empty | PASS |

Total validation wall clock: 191 s of test time, plus ~200 s of CLI measurement
(the `--top`, `--per-word-budget` and `--total-budget` sweeps of §5 and the
bounded `--per-word-budget 5` re-run).

**Nothing is worse than base. No regression.**

### Not run, and not claimed

Per `docs/environment-notes.md` and the item, this host has no `rustup` and no
`rustfmt`/`clippy`/`rustdoc` components, so these **cannot run here and were not
run**:

* `cargo fmt` / `cargo fmt --check`
* `cargo clippy --all-targets -- -D warnings`
* `cargo test --doc` / any doctest

A reviewer must run all three before accepting. The new test file is
hand-formatted to the surrounding style; that is a claim about appearance, not a
`rustfmt` pass.

---

## 9. Defects found, reported and NOT fixed

Per the item, defects are reported, not repaired. All are in `src/main.rs`, which
this front may not edit.

1. **No pool rank is reported, and there is no way to ask for one.** A user cannot
   distinguish a phrase that is 27th-best of 50 from one that is 27th-best of 2.
   Every reachability claim in this project is denominated in pool membership, and
   none of it is observable at the executable boundary. *Severity: design gap, not
   a bug.* It is the direct cause of the item's question "and the pool rank if the
   binary reports one" having the answer "it does not".
2. **The two budget flags are order-insensitive today, but only by accident of
   implementation, and the usage text does not say so.**
   `src/main.rs:92-111` deliberately re-applies the budgets when the mode is
   already `Approximate`, so a budget flag given *before* `--approximate` is
   still honoured — but only because the code re-reads the variables rather than
   the already-built `config.mode`. This is correct today and
   fragile: a reader could reasonably "simplify" it to mutate `config.mode` directly
   and silently break `madgab --per-word-budget 0.25 --approximate "..."`. *Severity:
   latent maintenance hazard, no observed misbehaviour.* The order-insensitivity
   was measured, not assumed — re-verified by the recovery pass, same md5s:

   ```sh
   $ madgab --approximate --per-word-budget 0.25 --top 50 "recognize speech"   # e1685bf45eaea2e6181a78f2c5173ce7
   $ madgab --per-word-budget 0.25 --approximate --top 50 "recognize speech"   # e1685bf45eaea2e6181a78f2c5173ce7
   ```

   Byte-identical, and both put the canonical clue at display rank 12, which also
   independently confirms the §5.2 rank-12 finding at a second width.
3. **The shipped default `--top 10` hides the project's flagship green fact.** A
   user who types the canonical invocation gets no `wreck a nice beach`. This is a
   product decision, not a bug, but it is the finding most likely to be mistaken for
   a regression, so it is stated in §3.1 and pinned by test 2.

No defect was found that makes the CLI wrong; it is correct and it is simply more
narrow than the milestone statement implies.

---

## 10. INTEGRATE / HOLD

**INTEGRATE**, on `post-milestone-acceptance` only, never `main`.

* Production change: **none**. `git diff --stat <parent> <head> -- src` is empty.
* Test change: one new test file, 3 green / 1 deliberately-`#[ignore]`d, with
  red/green evidence for both directions.
* Fences: all green; `corpus_integration` 12/1 with the known red untouched; `--lib`
  76/0/12, equal to base.
* Nothing measured here is a regression, and nothing was tuned to improve a verdict.

A coordinator should review the §3.1 default-`--top` finding and the §6 milestone
restatement first: those are the two things that change what the itinerary
*claims*, and they are docs decisions, not code decisions.

---

## 11. Next useful action

**Do not open a case-2 reach front.** §3 of the obstruction map is unchanged by
anything measured here, and §5.3 just re-confirmed at the executable boundary that
the whole reachable envelope misses it.

The next useful action is a **docs-only, no-code** decision on the milestone text,
because this front has established that the current text asserts something false:

> Restate the primary milestone as *"at `--top >= 27`, `madgab --approximate`
> produces `wreck a nice beach` for `recognize speech`; the shipped default
> `--top 10` does not, and `Hits Justice Dupe Hid Came` is not reachable at any
> documented knob."* That is a one-paragraph edit to the itinerary, and it is the
> only change that makes the completion checks in
> `docs/work/OBSTRUCTION-MAP.md` §4 checkable against the release artifact.

A second, genuinely open question this front surfaced and did **not** answer, worth
a front only if a coordinator wants it: **is `--per-word-budget 0.25` a better
default than 0.5?** It moves the green fact from display rank 27 to 12 at 0.6x the
wall clock, and 1000 rows still come back. That is a head-lift question measured
*within the existing search*, not a reach question, so it is squarely inside §3's
"general search quality" direction and squarely outside anything priced so far. It
would need the head-lift criterion of §4 (currently −0.0180) to be decided, and it
must not cost case 1 — which, on this measurement, it would not.

**A defect fix for `src/main.rs` is NOT recommended as a front.** Findings 1 and 3
in §9 are deliberate product decisions and finding 2 is a comment-level hazard. A
front that touches `src/main.rs` would contend with nothing today, but it would be
fixing something nobody is currently broken by, and this repository has a strong
record of that costing more than it returns.
