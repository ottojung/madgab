---
work_item: true
id: w-5c1a3e
state: working
priority: high
owner: agent-5c1a3e (front working the item; opened and claimed by coord-9c31 at 2026-09-27T23:19Z on post-milestone-acceptance at f21f34e, launched 23:19Z in /workspace/madgab-retain-5c1a3e [madgab-retain-5c1a3e])
updated: 2026-09-27T23:52:00Z
branch: madgab-retain-5c1a3e
worktree: /workspace/madgab-retain-5c1a3e
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

# Progress — agent-5c1a3e

## Durability

Branch `madgab-retain-5c1a3e`, pushed. `f15a878` is an explicitly labelled **INTERIM** commit
carrying env-gated measurement instrumentation and a phrase-free probe harness. **It is not
production and must not be integrated**; it exists so this front is recoverable from repository
state alone. It changes no retention constant and no band ranking key, and touches neither the
objective weight vector nor the score function.

## Criterion 1 — MEASURED, and it falsifies this item's own premise

**`hid` is not pruned from its target span's shortlist.** Read off the production path
(`MADGAB_RETENTION_TRACE=13`, `src/approx.rs:208-265`), target `It's just a stupid game`,
normalised IPA `ɪtsdʒʌstəstʊpədɡeɪm` (n = 19):

| quantity | value |
|---|---|
| target span | `start=13`, `consumed=2`, i.e. `/əd/` — the ə and d of *stupid* |
| `hid` cost | **0.3695** |
| cost band | **2** (`floor(0.3695 / 0.5 * 4) = 2`) |
| band sizes | band0 **3**, band1 **7**, band2 **107**, band3 **257** (ordered = 374) |
| rank in cost-ordered list | **ord = 104** |
| in-band rarity rank | **46** (`BAND_KEEP = 40`) |
| rarity | 12601 |
| verdict | loses the **band** stage, **rescued by the cost-ordered fill at `fill#104`** |

So the exact answer to "which stage and rank drops it": **no stage drops it.** It misses the cheap
head (`CHEAP_KEEP = 96`, it is at 104) and misses its cost band (rarity rank 46 against
`BAND_KEEP = 40`), and is then picked up by the third-stage fill, which still has room: cheap plus
band yield fewer than `MATCHES_PER_SPAN = 256` unique words because the stages overlap, so the fill
runs and reaches ordered index 104. It is retained at rank 104 of 256 kept.

**All five words of the canonical case-2 clue are present in their production shortlists** — none
is pruned anywhere on this alignment:

| word | span | cost | retained at |
|---|---|---|---|
| `hits` | `start=0 consumed=3` `/ɪts/` | 0.2000 | `cheap#8` |
| `justice` | `start=3 consumed=7` `/dʒʌstəs/` | 0.0000 | ord_in_span 0 |
| `dupe` | `start=10 consumed=3` `/tup/` | 0.1500 | ord_in_span 3 |
| `hid` | `start=13 consumed=2` `/əd/` | 0.3695 | `fill#104` |
| `came` | `start=14 consumed=5` `/dɡeɪm/` | 0.4642 | ord_in_span 34 |

Total substitution cost 1.1837 against `total_budget = 1.5`, so the reading is budget-admissible.

## Where the alignment is actually lost

Instrumenting the enumeration (`MADGAB_PATH`) shows the alignment is **never enumerated**. It does
not reach the pool at any `beam_width`:

* `beam_width = 64`: the one-word prefix `hits` is not in the surviving 64 at `p=3` at all
  (prefix match depth 0 at every position).
* `beam_width = 512`: pool 20749, still absent.
* `beam_width = 4096`: pool 20575, still absent. The prefix reaches **depth 2** at `p=10`
  (`hits justice`) but the three-word prefix is not among the top 4096 at `p=13`; the best
  depth-1 candidate there sits at combined-score rank 2681 of 4878.
* At the shipped `top_n = 200000` (the whole enumeration) the pool is **20616** clues and the
  clue is absent; its best leading run is 2 of 5. `hid` does appear in the pool, twice, in other
  alignments (`hid josh dashed oop add aim` at pool rank 4488, `hid see justice too add gave` at
  13795) — so the word is reachable, just not in this combination.

**Therefore the drop is the intermediate beam prune `prune_partials`
(`src/lib.rs:3454`), which ranks by `Metrics::combined` — the objective side — and not the
span-shortlist retention policy.** The front's premise ("the word is pruned, so retention is the
last layer that can fix it") is false as written, and that is the finding.

## Validation actually run at this point

Run with a dedicated `CARGO_TARGET_DIR` and `--test-threads=1`, on this branch only.

* Base, before any edit: `--lib` **75 passed / 12 ignored**; `no_phrase_hard_coding` **9/9**;
  `emit_coverage` **7/7**; `approx_determinism` **4/4**; `exact_determinism` **1/1**;
  `corpus_integration` **12 passed / 1 failed** — the single failure being the known pre-existing
  base red `approximate_finds_classic_madgab_resegmentation`, untouched and not re-pinned.
* At `f15a878`: `no_phrase_hard_coding` **9/9**; `corpus_integration` **12 passed / 1 failed**,
  the same single base red, which is also the evidence that the instrumentation is inert when its
  variables are unset.
* `emit_spread` does not exist as a test name on this tree; the emission-spread counters are
  asserted from the lib suite, which is green. Recorded rather than silently treated as run.
* **`cargo fmt` and `cargo clippy` cannot run on this host** (no `rustup`, no `rustfmt`/`clippy`
  component — `docs/environment-notes.md`). They are **not** claimed, here or in any later commit
  on this branch.
* The probe's first version named the alignment under measurement and
  `no_phrase_specific_hard_coding_in_src_web_or_examples` failed on it. The **allowlist was not
  edited and the finding was not re-pinned**; the harness was rewritten to take every input from
  `argv` instead.

## Collision warning (may become a blocker)

The measured drop point is `prune_partials`' ranking by `Metrics::combined`. That is the objective
surface. The coordinator pass at 23:28Z reports the weight lever **closed by `REPORT-e086cc`** —
that report is **not present on this branch**, so this is recorded as coordinator-reported and not
independently verified here. If that is right, the successor front for canonical case 2 is *not*
openable from this side, and the honest end state for this item is a priced negative on retention
plus a collision report, not an implementation.

## Next action

1. Finish **criterion 2** properly rather than stopping at criterion 1: make the retention
   constants and the band ranking key env-overridable **in the temporary instrumentation only**,
   and sweep the `CHEAP_KEEP` / `COST_BANDS` / `BAND_KEEP` split and rarity-first vs cost-first
   band ranking, reporting for each variant: whether the word is retained, pool size, printed
   top-50 `SIMILARITY` on the green canonical case-1 target, and whether the canonical case-2 clue
   reaches the pool.
2. Expect that sweep to be **null on pool membership**, since no variant can create a beam slot for
   an alignment the objective ranks below the beam cut. If it is null, the retention axis closes as
   a measured negative with the numbers above as the settling measurement, criterion 3 answers
   "no rule survives — propose no code", and criterion 4 is answered by the *diagnostic* rather
   than by a policy regression test. No production line changes and no fence is re-pinned.
3. Commit and push at least every 20 minutes, and keep this section current, so a fresh
   coordinator pass can recover the front from repository state alone.

The two running fronts' worktrees (`/workspace/madgab-e086cc`,
`/workspace/madgab-pairscore-3f6a21`) have not been read, entered or run.

