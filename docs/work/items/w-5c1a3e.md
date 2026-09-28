---
work_item: true
id: w-5c1a3e
state: done
priority: high
owner: agent-5c1a3e (front worked the item to a priced negative; opened and claimed by coord-9c31 at 2026-09-27T23:19Z on post-milestone-acceptance at f21f34e, launched 23:19Z in /workspace/madgab-retain-5c1a3e [madgab-retain-5c1a3e]; steering from coord-1f0d at 23:44Z-23:45Z)
updated: 2026-09-27T23:58:00Z
branch: madgab-retain-5c1a3e
worktree: /workspace/madgab-retain-5c1a3e
report: ../REPORT-5c1a3e.md
---

# Price the per-span *retention* policy: the last layer that can put `hid` in the pool

## Goal

Measure — and, only if a rule survives its own fences, generalise — the **span shortlist retention
policy** in `src/approx.rs`, so that a word the canonical case-2 reading needs can survive its
target span's shortlist. Today the blocking fact is that `hid` is *pruned from the retained
shortlist for its target span*, so it never enters the pool and no downstream score, weight, or
selection change can recover the clue.

This is a **retention** front. It is the one layer of the pipeline that neither of the two running
fronts owns.

## Context

- Standing goal: [../skills/itinerary-madgab.md](../skills/itinerary-madgab.md). Canonical case 1
  (`wreck a nice beach` for `recognize speech`) is green on this branch at printed rank 27.
  Canonical case 2 (`hits justice dupe hid came` for `It's just a stupid game`) is **not in the
  pool**; `hid` is pruned at the span shortlist.
- The retention policy is `MATCHES_PER_SPAN = 256` (`src/approx.rs:14`) and, when a span has more
  matches than that, a three-stage keep at `src/approx.rs:208-265`: the cheapest `CHEAP_KEEP = 96`
  by cost, then `COST_BANDS = 4` cost bands each keeping `BAND_KEEP = 40` ranked by
  **rarity first, then cost**, then a cost-ordered fill. A candidate is dropped if it loses all
  three stages.
- `REPORT-1a4e8d.md` (integrated at `5e15cc8`) prices the *selection* layer exhaustively and
  closes it: the reserve has no admissible interior, and its withheld candidate is the 18th member
  of a structure already saturated at `share_cap = 17`. Selection is not where case 2 is lost.
- Reach on the lattice is closed by measurement (`w-2f1c03`, `OBSTRUCTION-MAP.md` §3), so a
  *search* front would re-derive a priced negative. Retention is different in kind: it is a
  question about which of the matches the fuzzy lexicon already produced are *kept*, not about
  whether the reading is reachable.

## Non-contention is mandatory

- [w-e086cc](w-e086cc.md) (`agent-e086cc`, `/workspace/madgab-e086cc`) varies the **objective
  weight vector** and is currently at the vector it calls A5.
- [w-3f6a21](w-3f6a21.md) (`agent-3f6a21`, `/workspace/madgab-pairscore-3f6a21`) works the
  **score function** (adjacent-slot substitution charge).

This front holds both fixed. It may only vary the *retention* constants and the ranking key used
inside the keep stages. If a candidate retention rule needs a weight-vector or scorer change to
pay off, stop and report that the surfaces collide.

## Completion criteria

1. A measurement of, for the canonical case-2 target span, the full ordered match list with, for
   each match, its cost, its cost band, its rarity key, and the stage at which it was dropped (or
   kept). State the exact stage and rank that drops `hid`, from the production path, not from a
   re-implementation of it.
2. A sweep over the retention policy — at minimum the `CHEAP_KEEP` / `COST_BANDS` split and the
   band ranking key (rarity-first vs cost-first) — reporting for each variant: whether `hid` is
   retained, the resulting pool size, the printed top-50 `SIMILARITY` on the green canonical case 1
   target, and whether the canonical case-2 clue reaches the pool. Small coherent variants only.
3. If a general rule survives, implement it as a *policy* change with the same character as the
   existing ones — a number of slots or an ordering key justified in a comment, not a fitted
   constant and never a word, span, phrase, or sentence from the canonical examples. If none
   survives, say so with the number that settles it and propose no code.
4. Regression tests at the API or executable boundary expressing the *general* property that
   survives (for example: a span whose matches exceed the retention budget keeps candidates from
   more than one cost band, or keeps a low-`CHEAP_KEEP`-rank candidate that a cost band rescues).
   No test may name `hid`, `wreck a nice beach`, `hits justice dupe hid came`, or an exact expected
   rank for a canonical input.
5. Fence suites green at the front's base and after any change: `no_phrase_hard_coding` 9/9,
   `emit_coverage`, `emit_spread`, `approximate_output_is_locked`, `approximate_list_represents_enumerated_resegmentation`,
   `approximate_determinism`, `exact_determinism`, both determinism suites, `--lib`. The one known
   pre-existing base red, `approximate_finds_classic_madgab_resegmentation` (canonical case 2), must
   stay untouched and **not** re-pinned. Report `fmt` and `clippy` honestly — this host has no
   `rustup`, so if they cannot run, do not claim them.
6. Work committed and pushed to `madgab-retain-5c1a3e`. Never self-merge into
   `post-milestone-acceptance`; never touch `main`. Use a dedicated `CARGO_TARGET_DIR` and
   `--test-threads=1`; do not enter `/workspace/madgab-e086cc` or
   `/workspace/madgab-pairscore-3f6a21`.

## Handoff / notes

Opened by the reconciliation pass `coord-9c31` at 23:19Z, immediately after `w-1a4e8d` closed the
reserve axis as a surface. The routing is from two durable facts, not from a guess: the reserve
axis is priced and closed, and `agent-e086cc`'s in-flight finding is that the alignment's absence
from the pool is **weight-independent** (never reached at any of the nine sweep vectors) — so a
third *objective-side* front would be contending with `w-e086cc` for a surface that cannot fix
case 2. Retention is the layer the failure actually sits in.

If criterion 2 shows no retention variant retains `hid`, that is a valid and useful priced
negative: it converts "the word is pruned" from an observation into a measured closure of the
retention axis, and the next front must be named elsewhere.

---

# Outcome — DONE, priced negative, verdict HOLD

Full record: [../REPORT-5c1a3e.md](../REPORT-5c1a3e.md).

**Verdict: HOLD.** No production line changes. One regression test added. The production
region of `src/approx.rs` is byte-identical to the base `4fee16f`.

## The premise is refuted

The item was opened on the fact that `hid` "is pruned from the retained shortlist for its
target span". **It is retained**, measured from the production path. For the canonical case-2
target (`/ɪtsdʒʌstəstʊpədɡeɪm/`, n=19) the reading's span for that word is `start=13
consumed=2`, i.e. `/əd/` — the ə and d of *stupid* — whose ordered match list has **374**
entries against `MATCHES_PER_SPAN = 256`:

| quantity | value |
|---|---|
| cost | **0.3695** |
| cost band | **2** |
| band sizes | **3 / 7 / 107 / 257** |
| rank in cost-ordered list | **ord = 104** |
| in-band rarity rank | **46** (against `BAND_KEEP = 40`) |
| rarity | 12601 |
| outcome | misses the cheap head (`CHEAP_KEEP = 96`), misses its band, **kept by the cost-ordered fill at `fill#104`** |

**No stage drops it.** It is rescued by the third stage, which has 152 slots of headroom when
it reaches ordered index 104. All five words of the reading are in their production shortlists
(`hits` `cheap#8`, `justice` ord 0, `dupe` ord 3, `hid` `fill#104`, `came` ord 34) at a
total cost of **1.1837** against `total_budget = 1.5`.

## Where it is actually lost: `prune_partials`

The alignment is never enumerated, at any beam width. Pool 20616 at `beam_width` 64 (the whole
enumeration, `top_n = 200000`), 20749 at 512, 20575 at 4096 — absent in all three. At 4096 the
prefix reaches 2 words at `p=10` and the 3-word prefix is not in the top 4096 at `p=13` (best
depth-1 candidate at combined-score rank 2681 of 4878). `hid` *is* in the pool twice, in other
alignments (ranks 4488, 13795), so this is about the combination, not the vocabulary.

**Localisation: `prune_partials` (`src/lib.rs:3454`), ranked by `Metrics::combined`.** That
surface belongs to **w-3c5b18 / agent-3c5b18** (branch `madgab-thresh-3c5b18`) per steering
from coord-1f0d at 23:44Z. It was **not** edited and **not** started here.

## The retention sweep is null (criterion 2)

14 variants over `MATCHES_PER_SPAN` / `CHEAP_KEEP` / `COST_BANDS` / `BAND_KEEP` / band key
(rarity-first vs cost-first). Full table in the report §3. Three readings:

* **`hid` retained in 14 of 14 variants**, in each case by a different stage depending only on
  where the constants put the boundary.
* **The case-2 clue reaches the pool in 0 of 14 variants.** No retention setting creates a beam
  slot for an alignment the objective ranks thousands of positions below the cut.
* **The case-1 printed top-50 is byte-identical in all 14 variants** (md5 `d8136a142ac2`), so
  printed top-50 mean `SIMILARITY` is unchanged at `0.848330` and the printed mean score
  reproduces at `0.9199228013`. Pool size moves within 20259–20671 (~±1%) and none of it
  reaches the visible list.

A sweep in which nothing moves is a sweep showing the axis is not on the path.

## Criteria 3 and 4

**Criterion 3 — no rule survives; no code proposed.** The settling number is 0 of 14 variants
reaching the pool, against a word retained in 14 of 14.

**Criterion 4 — one regression test, mutation-verified.** `approx::tests::
a_span_over_budget_keeps_candidates_neither_the_head_nor_its_band_keeps`, asserting over a
synthetic 450-match span against a 256 budget that the keep retains candidates *neither* the
cheap head *nor* its own band would keep, and that the two rescues are *different stages*: a
cheap-but-rare candidate needs the cost-ordered fill, an expensive-but-common one needs the
band's rarity-first keep. Non-vacuity by mutation on the delivered file: **fill removed →
FAILS**, **band stage removed → FAILS**. Honest limit, recorded in the report §4: **cheap head
removed → still passes**, because with a budget above the head the fill reproduces the
head's choices, so the head is not independently pinned.

The test names no canonical word, phrase, sentence or rank; its fixture is invented non-words
over the invented segment string `abc`, and its assertions are counts over cost groups, not
ranks.

## Validation actually run

Dedicated `CARGO_TARGET_DIR`, `--test-threads=1`, this branch only.

| suite | base `4fee16f` | delivered |
|---|---|---|
| `--lib` | 75 passed / 12 ignored | **76 passed / 12 ignored** |
| `no_phrase_hard_coding` | 9/9 | **9/9** |
| `emit_coverage` | 7/7 | **7/7** |
| `approx_determinism` | 4/4 | **4/4** |
| `exact_determinism` | 1/1 | **1/1** |
| `corpus_integration` | 12 / 1 failed | **12 / 1 failed** |

The one failure is the pre-existing base red
`approximate_finds_classic_madgab_resegmentation`, untouched and not re-pinned.

**`cargo fmt` and `cargo clippy` cannot run on this host** (no rustup/rustfmt/clippy) and are
**not claimed**; neither is `--doc`. `emit_spread` **does not exist as a test name on this
tree** — the emission-spread counters are asserted from `--lib`, which is green; recorded
rather than assumed.

## Scope held

Objective weight vector and score function untouched; `prune_partials` untouched; no fence
re-pinned; `/workspace/madgab-e086cc` and `/workspace/madgab-pairscore-3f6a21` never read,
entered or run. The 14-variant sweep machinery and the tracing were **measurement scaffolding
and are not in the delivered diff**; they survive in the labelled interim commit `f15a878` for
recovery, which is explicitly not to be integrated. The probe example was removed for the same
reason — the first version of it named the alignment under measurement and
`no_phrase_specific_hard_coding_in_src_web_or_examples` failed on it; the allowlist was not
edited and the finding not re-pinned.

## Blockers

None for this item; it is closed. The blocker it *would* have hit is recorded for the
coordinator: the remaining surface is the enumeration discard threshold, owned by `w-3c5b18`.

## Next action

None here. For the coordinator: integrate `REPORT-5c1a3e.md` (docs) and the single regression
test, treat the retention axis as closed for canonical case 2, and route the case-2 objective
to `w-3c5b18`. Note for that front: the reading is **enumeration-limited, not
retention-limited**, and at `beam_width` 4096 it is still ~2700 combined-score positions below
the cut at `p=13` — a discard-threshold change has to be worth about that gap, not about the
104-rank fill headroom this front measured.
