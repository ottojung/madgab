---
work_item: w-4d1e93
report: true
id: REPORT-4d1e93
state: done
branch: madgab-4d1e93
base: 3ea86d2 (w-4d1e93, post-milestone-acceptance at 542b9a9)
verdict: HOLD
updated: 2026-09-27T22:40:00Z
---

# REPORT-4d1e93 — the pool/reach fences F5 and F6 are both weight-coupled

Scope: this front owns **F5** (`a_lattice_alignment_can_be_absent_from_the_production_pool`) and
**F6** (`approximate_output_is_locked`) only. F0–F4 belong to [w-e086cc](items/w-e086cc.md) and
were neither read, run nor duplicated here. Nothing in `/workspace/madgab-e086cc` was touched.

## 1. Classification, and the number that decided each

| fence | classification | the deciding number |
|---|---|---|
| **F5** `a_lattice_alignment_can_be_absent_from_the_production_pool` | **weight-coupled** (as a fence); its load-bearing conjunct is **weight-independent** over everything measured | the word `hid` leaves the production pool's vocabulary at **λ = 1.0** of the parsimony axis, and at the funded C1d vector `514ed91` — the fence panics at `tests/emit_coverage.rs:116`, the *per-word* assertion, while the pool-absence assertion at line 108 still passes. Alignment absent from the pool at **11 of 11** weight vectors measured. |
| **F6** `approximate_output_is_locked` | **weight-coupled**, and the most tightly coupled of the eight: it is a lock on *values*, not on behaviour | the locked string `0.933655 isle uhh view` becomes `0.934655 …` at **λ = 0.001**. Red at all nine non-zero points of the sweep, including a perturbation two orders of magnitude below the smallest shipped axis weight (`PUNCH = 0.10` is 10⁵× larger; the smallest *usable* non-zero step here is 0.001). |

Both fences are therefore properties of *one weight vector*, not of the search, and both must be
re-derived together with any objective change. Neither is expressible weight-free as written.

## 2. What was measured, and how

### 2.1 Reproduction at shipped weights, before anything changed

`emit_coverage` **4 passed / 0 failed** and `corpus_integration::approximate_output_is_locked`
**ok** at `3ea86d2`, `cargo test --release`, `--test-threads=1`, own `CARGO_TARGET_DIR`
(`target/probe-tgt`). So both fences were green before a byte was touched, and everything below is
a delta from that.

### 2.2 The probe

`src/`'s objective weights are `const`s, so on the unpushed scratch branch
`scratch/4d1e93-f5f6` (`cf44be7`) the axis was made a `fn` locally to the probe: a thread-local
`axes::probe::PARSIMONY`, shaped exactly as C1d's axis is (`1 - |w_clue - w_target| / max`, the
function `scratch-3f8c62-landed` adds), read by `Metrics::combined` **and** by all three
structural keys — `partial_span_score`, `complete_span_score`, `span_score_bound` — so a single
λ sweeps the whole axis family rather than one site. It is 0.0 at every shipped path, and no
shipped call site was edited to vary it. No shipped string, word or phrase is keyed on anywhere:
`no_phrase_hard_coding` is **9/9** with the probe in the tree.

Independently, the **achievable** C1d vector was re-measured from the local-only branch
`scratch-3f8c62-landed` (`514ed91`) in a throwaway worktree under `target/`, with its own
`CARGO_TARGET_DIR`. That branch is never integrated and never pushed; the remote declines it.

### 2.3 The sweep — `It's just a stupid game`, `--approximate --top 50`

| λ | pool size | alignment in pool | `hid` in pool vocab | printed ⊆ pool | lock head | lock green |
|---|---|---|---|---|---|---|
| 0 (shipped) | 18,949 | absent | yes | 50/50 | 0.933655 | **yes** |
| 0.001 | 18,947 | absent | yes | 50/50 | 0.934655 | no |
| 0.005 | 18,942 | absent | yes | 50/50 | 0.938655 | no |
| 0.02 | 18,917 | absent | yes | 50/50 | 0.953655 | no |
| 0.05 | 18,856 | absent | yes | 50/50 | 0.983655 | no |
| 0.10 | 18,777 | absent | yes | 50/50 | 1.033655 | no |
| 0.15 | 18,999 | absent | yes | 50/50 | 1.083655 | no |
| 0.25 | 18,852 | absent | yes | 50/50 | 1.183655 | no |
| 0.40 | 18,842 | absent | yes | 50/50 | 1.333655 | no |
| 1.00 | 19,013 | absent | **no** | 50/50 | 1.933655 | no |

Pool vocabulary ran **1,811–1,864** distinct words against **250–294** printed word instances,
every printed word in the pool vocabulary at every λ. Pool *size* moves by −0.9%…+0.3% across
the sweep, so the width the ratio fence reads is weight-sensitive even though the ratio is not.

### 2.4 F5, read properly

The F5 panic on the C1d vector is the finding. The isolation table in `REPORT-3f8c62.md` §4.1
records this fence as red under the axis with the gloss "`hid` leaves the pool entirely". That is
literally true and it is *not* the load-bearing claim: at `514ed91`

```
thread 'a_lattice_alignment_can_be_absent_from_the_production_pool' panicked at
tests/emit_coverage.rs:116: the word "hid" is absent from the pool entirely,
so the pool assertion above is vacuous for it
```

line 116 is the **second** assertion. Line 108 — the alignment is absent from the production pool
— **passed**. The alignment did not arrive; the word it is made of stopped being enumerated.
`OBSTRUCTION-MAP.md` row 4 records the same shape for the parked fill branch `a8a8f70`
("2 pool wordings → 0"), so this is now three independent surfaces — the fill, the parsimony
axis, and this sweep — on which F5's reds have all arrived through the *vocabulary* conjunct and
never through the alignment conjunct. **11 of 11** weight vectors measured leave the alignment
absent from the pool.

So F5 is a conjunction of two claims with opposite characters:

* *"this alignment is absent from the production pool"* — weight-independent across everything
  measured, and load-bearing: it is what retires the reweighting family of explanations.
* *"each of its words is individually in the pool"* — weight-coupled. Whether the enumeration
  retains the word at all is a search-output that moves with the weights, and the fence goes red
  the moment it does not.

A fence is only as weight-free as its weakest conjunct, so **F5 is weight-coupled and must be
re-derived with the vector.** What must *not* be lost in that re-derivation is the first claim:
it is the one that says the obstacle is reach and emission, not scoring, and it survived every
vector measured.

### 2.5 F5's weight-free form: looked for, and refuted by measurement

The general property F5 is a special case of is *"a combination can be absent from the pool while
every one of its parts is present"* — i.e. **the pool is not closed under word-level
recombination of its own printed proposals**. It is expressible phrase-free from the public API
(splice two printed clues' word sequences at each interior boundary; the splices are drawn
entirely from pool vocabulary), so it was measured before being proposed:

| λ | splices absent from the pool |
|---|---|
| 0 | **61 / 180** |
| 0.001 | 42 / 169 |
| 0.02 | 58 / 153 |
| 0.05 – 1.00 | **0 / 144** |

Recombination closure is itself weight-dependent — total closure at λ ≥ 0.05 — so the only
generalisation of F5 available at this boundary is **refuted by the same sweep that refutes
nothing else**. It is not landed as a test: a test asserting non-closure would be a *fourth*
incidental fence, which is the thing this front exists to stop making. This is the proof that no
weight-free form of F5's alignment-absence claim is expressible here: exposing the per-span
shortlists from the public API (question (a), which `OBSTRUCTION-MAP.md` records as the
distinguishing measurement) is the only route, and it is not a test change.

### 2.6 F6's weight-free form, which does exist

F6's golden table cannot be made weight-free — it is a function of the vector by construction, and
§2.3 reddens it at λ = 0.001. But the *invariants* the table sits on are properties of the search
and held at **10 of 10** sweep points, at `top_n` 10, 25 and 50: the printed set is exactly
`top_n` long, its scores are non-increasing, and no wording repeats. Those are asserted now
(§3). They are what lets a future front re-derive the table deliberately rather than discover
afterwards that the shape of the answer moved too.

## 3. What landed on `madgab-4d1e93`

Three new tests in `tests/emit_coverage.rs`, all phrase-free, all green at shipped weights, each
demonstrated **red** by a temporary `src/` mutation that was reverted:

| test | weight-free property it states | red proof |
|---|---|---|
| `every_printed_proposal_is_a_member_of_the_dedup_pool` | the printed set is drawn from the dedup pool (50/50 at every λ) | pool assembled with the selected clues removed → `6 passed / 1 failed`, "1 of 50 printed proposals are not members of the 18,899 wide dedup pool" |
| `a_printed_proposal_set_is_ordered_deduplicated_and_exactly_top_n` | exactly `top_n`, score-ordered, phrase-deduplicated | selection reversed before return → "printed scores ascend, 0.9060233075705043 then 0.9061647450002773" |
| `the_production_pool_speaks_a_wider_vocabulary_than_the_printed_list` | the pool's vocabulary is strictly wider than the printed list's, so "the pool lacks a word" cannot be inferred from the printed 50 | pool narrowed to the selected set → "the pool's whole vocabulary of 32 words is used by the printed 50 proposals" |

The first is a premise of F5's own argument: if a printed clue could fall outside the pool, "absent
from the pool" would also be true of clues that *were* proposed, and the reading of line 108
would stop following. The third is the weight-free skeleton of F5's per-word conjunct.

Verification on the landed tree, `cargo test --release`, `--test-threads=1`:

| suite | result |
|---|---|
| `emit_coverage` | **7 passed / 0 failed** (4 pre-existing + 3 new) |
| `cargo test --release --lib` | **75 passed / 0 failed / 12 ignored** (unchanged) |
| `no_phrase_hard_coding` | **9 / 9** |
| `corpus_integration` | 12 passed / 1 failed — `approximate_finds_classic_madgab_resegmentation`, the **known base red**, not re-pinned and not touched; F6 green within it |

`cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run on this host** (no
`rustup` components) and are **not claimed**. The new code is hand-formatted to the surrounding
style.

## 4. Verdict

**HOLD.**

The classification half is complete and the answer is that neither fence is integrable *as a
fence*: INTEGRATE on this front would require "a fence-rederivation plus the smallest objective
change that keeps both fences green", and F6 pins the only quantity that makes "keeps F6 green"
decidable — the score strings themselves. Measured, the set of weight vectors that keep F6 green
is the singleton {shipped}: nine of nine non-zero points of the sweep are red, the first of them
at 0.001. The smallest objective change that keeps both fences green is therefore *no change*,
which is not an objective change. Re-baselining the lock is not available to this front: the
twice-touched precedent for it (`w-04f83f`, `w-9d4e10`) is a deliberate, separately-argued
re-baseline of an acceptance test, and this item forbids weakening a fence to make a vector
landable.

### The exact unblocking condition

An objective change may be landed on this fence set when **one** of these is true:

1. **A named weight vector arrives with its own re-derived F6 lock and its own re-derived F5
   expectation, both argued in the same commit** — the re-baseline of the lock is the price of
   the change and is argued, not assumed; or
2. **F6 is replaced by the weight-free invariants alone** (the middle test in §3, which is now
   landed and green), and the behavioural lock is retired as a fence by a decision recorded
   somewhere other than this report. That is a loss of coverage, so it is a call for the
   coordinator, not for this front.

In case 1, F5's re-derivation must keep the *alignment-absence* conjunct on the same assertion it
occupies today and may re-derive the *vocabulary* conjunct against the new vector's enumeration
— because §2.4 measured that those two conjuncts have different characters, and folding them into
one expectation again is what produced every prior reversal in this area.

### What I would not claim

* I measured one axis family (word-count parsimony, 10 points, scorer + all three keys) and one
  funded vector (C1d). I did not sweep the redistribution of the existing axes — that is the
  sibling front's surface — so "the alignment's absence is weight-independent" is a measurement
  over 11 vectors, not a proof over the whole space. The *fence-level* classification does not
  depend on it: F5 is red at two of the eleven for a reason no further sweep can repair.
* The classification of F0–F4 is not mine and nothing here should be read as touching it.
* The base red in `corpus_integration` was left red.

## 5. Constraints

* **No phrase, clue, word or substring special case.** Nothing in `src/` or `tests/` on the landed
  branch is keyed on `hid`, `wreck a nice beach` or any other phrase; the three new tests name no
  clue and assert no score. `no_phrase_hard_coding` 9/9. The only fixture-naming in this front is
  the scratch probe, which names the same alignment the existing fence already names, in a test.
* **No fence weakened, deleted or `#[ignore]`d.** F5, F6 and every other test are byte-identical
  to `3ea86d2`; the landed diff is `tests/emit_coverage.rs` `+147/-0`, appended only. The three
  `src/` mutations used for the red proofs were reverted and the tree verified clean.
* **`coverage_tuples`, `sweep_index`, `EMIT_PROFILE_*` untouched.**
* **No `OBSTRUCTION-MAP.md` row re-opened**, and this is not treated as a case-2 reach item.
* **Probes on a separate unpushed branch**: `scratch/4d1e93-f5f6` (`cf44be7`) and the throwaway
  worktree on the local-only `scratch-3f8c62-landed` (`514ed91`, never pushed, never integrated).
  Neither is reachable from `origin`.
* **Own `CARGO_TARGET_DIR`** (`target/probe-tgt`, `target/c1d-tgt`), `--test-threads=1` on every
  run, one probe at a time. The sibling worktree was never read, entered or run.
* Not self-merged; nothing landed on `main`; this report and the item close-out go to
  `madgab-4d1e93`.
