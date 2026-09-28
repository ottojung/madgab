---
work_item: true
id: w-d4a90b
state: done
priority: high
owner: agent-d4a90b
updated: 2026-09-26T22:12:00Z
branch: madgab-postpunch-measure
worktree: /workspace/madgab-d4a90b
---

# Re-measure the second canonical example on the post-PUNCH head

## Goal

Every recorded distance for the itinerary's **second** canonical example —
`It's just a stupid game` -> `Hits Justice Dupe Hid Came` — was taken on a
head that predates the PUNCH scoring axis (`6a93c2a`, integrated from
[w-6ad4c1](w-6ad4c1.md)). PUNCH is a seventh additive axis in the objective, so
the numbers in [w-4b1e07](w-4b1e07.md), [w-558697](w-558697.md) and
[w-7b41d2](w-7b41d2.md) are stale for exactly the target the milestone needs.
The first example *was* re-measured post-PUNCH; this one never was.

This is a **measurement-only** front. It opens no scoring hypothesis, changes no
constant, and proposes no fix. Its output is the current distance, quoted as
both a score delta and a rank, at the CLI default `top_n` and at `--top 50`, so
the next routing decision (including `9f1c05`'s aggregation-form verdict) is
taken against the current tree rather than a three-axis-old one.

## Fence

- **No edits to `src/` or `tests/`.** This front measures; it does not change the
  thing being measured. Any temptation to "just widen the budget so it fits" is
  exactly the phrase-adjacent hack the itinerary forbids.
- No change to any `axes::*` constant, retention constant, or the selection
  policy. Those belong to the closed families in [w-3f9c02](w-3f9c02.md) and
[w-8a1d47](w-8a1d47.md) and to the live front [w-9f1c05](w-9f1c05.md).
- Artifacts and the report go to `measurements/d4a90b/` and
  `REPORT-d4a90b.md` on `madgab-postpunch-measure`, pushed as they are produced.
  **Never merge that branch into `post-milestone-acceptance`**; the report is
  copied into this work item by the coordinator.
- If a `--release` build is needed, build in this worktree only.

## Questions to answer, in order

1. On `post-milestone-acceptance` at the current head, what is the release
   binary's approximate output for `It's just a stupid game` at the **CLI
   default `top_n`** and at **`--top 50`**? Give the pool count, the top-band
   cutoff, and whether `hits justice dupe hid came` is enumerated at all.
2. If it is enumerated: its score, the score of the worst visible proposal, the
   **score delta**, and its **rank** in each configuration. Quote the delta and
   the rank separately; do not collapse them into one "visible" claim.
3. Per-axis decomposition of that delta **on the current head** — the question
   [w-558697](w-558697.md) answered for the old objective, re-asked because
   PUNCH is a new axis and may have changed which term dominates. State
   explicitly whether any axis now *moves the target* (as opposed to moving the
   band around it), and whether the six weights still sum to exactly 1.00.
4. Regression status, quoted exactly: `cargo test --release --test
   corpus_integration` — is `approximate_finds_recognize_speech_resegmentation`
   green and `approximate_finds_classic_madgab_resegmentation` still red? No
   test may be edited to change this.
5. Wall clock for one default approximate run and one `--top 50` run, so the
   next pass can price any breadth-based proposal.

## Completion criteria

- `REPORT-d4a90b.md` exists and is pushed to `origin/madgab-postpunch-measure`,
  with a one-line `VERDICT:` on its first line summarising the delta and rank.
- All five questions answered with quoted numbers, each traceable to a command
  and a base commit.
- No file under `src/` or `tests/` modified on the branch (`git diff
  post-milestone-acceptance...madgab-postpunch-measure -- src tests` empty).
- The coordinator has copied the numbers into this item, after which
  `state: done`.

## Handoff / notes

Launched 2026-09-26T21:40Z by coordinator `coord-2d6f` as Antonina agent
`d4a90b` in `/workspace/madgab-d4a90b`, branched from `post-milestone-acceptance`
at `f5b9eaa`. It does not contend with `9f1c05` (different worktree, and
`9f1c05` owns the aggregation-form design while this front owns only the
current-tree distance). Left running; a later fresh pass inspects it.

The second example's *status* is unchanged by this item: the milestone is still
unmet, and closing it belongs to [w-2f7a10](w-2f7a10.md), [w-4b1e07](w-4b1e07.md)
and [w-a02d28](w-a02d28.md) together.

## Coordinator pass 2026-09-26T22:12Z (coord-7a4e): reviewed, numbers copied in, `done`

Completion criteria are met, so this item is closed. Reviewed from the pushed
branch, not the worktree: `madgab-postpunch-measure` is `2860b57` on `origin`,
and `git diff post-milestone-acceptance...2860b57 -- src tests` is **empty**, so
the fence held and no constant, test or selection policy was touched. The
branch was **not** merged and must not be; the measurements are on it at
`measurements/d4a90b/`. Read the report with
`git show 2860b57:REPORT-d4a90b.md`.

### The five answers, copied in

**1 — default `top_n` and `--top 50` on the release binary.**

| | CLI default (`top_n` 10) | `--top 50` |
| --- | --- | --- |
| pool | `13498` | `17827` |
| proposals printed | 10 | 50 |
| worst **visible** (post-diversity) | rank 10, `0.900`, `it said thus test oop games` | rank 50, `0.898`, `it said thus tas too dame` |
| raw cutoff (`MADGAB_TRACE raw_cutoff`) | rank 9, `0.900786673`, `it justice too bed aim` | rank 49, `0.898008919`, `it justice too bed same` |
| requested wording enumerated? | **no** | **no** |

The raw cutoff and the worst visible proposal are quoted separately because they
differ: `select_diverse` reorders and rebalances after the pool is scored, so
the pool's `top_n`-th member is not the last thing the user sees.

**2 — score, delta and rank, quoted separately.** The target's **rank is
undefined in both configurations**, which is the headline:

| quantity | `top_n` 10 | `--top 50` |
| --- | --- | --- |
| target score | `0.79990129077367222` | `0.79990129077367222` |
| delta vs worst visible | `0.10026892900503859` | `0.09785131568084526` |
| rank | undefined — absent from all `13498` | undefined — absent from all `17827` |

Delta vs rank 1 at `--top 50` is `0.10528552125503088`. The score is a
reconstruction through the crate's own `Partial::extend_fuzzy` and
`Partial::into_clue`, verified by reproducing the printed scores of two known
enumerated proposals to the last bit. Also measured, and the reason the framing
is wrong: the phrase is **budget-admissible** — per-word costs `0.2 / 0.0 /
0.15 / 0.3695 / 0.4`, `sub_cost_total=1.11951981416578961` against
`total_budget 1.5`, every per-word cost <= `0.5`, and exactly one complete
lattice alignment. **No `hid`, `dupe` or `hits` proposal is enumerated at either
breadth**; the closest relative is `it justice too bad came` at rank 18 of 50.

**3 — per-axis decomposition on the current head.** Weights as compiled on
`f5b9eaa`: `SIMILARITY 0.25`, `NOVELTY 0.15`, `WORD_NOVELTY 0.15`,
`FAMILIARITY 0.1`, `RHYTHM 0.3`, `SHAPE 0.05`, `CLOSED_CLASS -0.15`,
`PUNCH 0.1`. The six pre-PUNCH additive weights sum to `1.00000000000000000`
exactly; with `PUNCH` the total is `1.10000000000000009`, and since `PUNCH` is
applied as `0.10 * (punch - 1.0)` and `CLOSED_CLASS` as `-0.15 *
closed_penalty`, both are non-positive and the objective's maximum is still
`1.0`.

Contributions to the delta, target -> worst visible at `--top 50`, summing to
the measured `0.097851316`:

| axis | contribution |
| --- | --- |
| `NOVELTY` | **`+0.050000000`** |
| `FAMILIARITY` | `+0.022522990` |
| `PUNCH` | `+0.020000000` |
| `SIMILARITY` | `+0.010328325` |
| `CLOSED_CLASS` | `-0.004166667` |
| `SHAPE` | `-0.000833333` |
| `WORD_NOVELTY`, `RHYTHM` | `0.000000000` |

Which axis *moves the target*: `RHYTHM` (weight `0.30`, the largest) and
`WORD_NOVELTY` are saturated at `1.0` for **both** sides and distinguish
nothing — the band's advantage is not rhythm. `NOVELTY` is now dominant at 51%
of the deficit, and it is the axis that moves the *band* around the target: the
canonical resegmentation `hits|justice|dupe|hid|came` lands on
`cuts=[3,10,13,15,19]` against target boundaries `[3,8,9,15,19]` and so
reproduces 2 of 4 inner boundaries (`0.666666667`), while the worst visible
proposal reproduces none and is rewarded the full `1.0`. `PUNCH` is the one axis
that moves the target: `punch=0.800000000` because `dupe` is not monosyllabic
(`punch_count=4` of `5`), costing a flat `0.020000000`, while the worst visible
proposal is 6 of 6 monosyllabic and pays nothing. On the pre-PUNCH objective
the deficit would have been `0.077851316`, so **PUNCH alone widens the measured
gap by `0.020000000`**, asymmetrically, because the band already sits at
PUNCH's ceiling and only the target can lose.

**4 — regression status, quoted.** `cargo test --release --test
corpus_integration` on this branch: `10 passed; 1 failed`.
`approximate_finds_recognize_speech_resegmentation ... ok`;
`approximate_finds_classic_madgab_resegmentation ... FAILED`, panicking at
`tests/corpus_integration.rs:136` with the same twelve `it justice ...` /
`it said thus ...` wordings. No test was edited. The harness says "missing from
top 50"; the trace evidence is sharper — the phrase is missing from the pool of
`17827`.

**5 — wall clock.** Three runs per configuration, whole process: `top_n` 10 at
`2664 / 2848 / 2514 ms` (search `1959 / 2072 / 1808 ms`), `--top 50` at
`2357 / 2124 / 2215 ms` (search `1696 / 1464 / 1553 ms`). Roughly `0.45-0.57 s`
is corpus load. **`--top 50` is not measurably more expensive than the default**:
raising `top_n` from 10 to 50 moved the pool from 13 498 to 17 827 candidates
and did not move the cost, so breadth alone is not the lever.

### What this closed, and what it opened

This item's brief expected a scoring distance and got a **localisation of the
loss to enumeration instead**: the wording is admissible, scoreable, and never
enumerated, so no score, admission, share-cap or cutoff change can surface it.
Two shortcuts were available and declined, and are recorded in the report: no
budget, retention or selection constant was widened to make the target appear,
and the alignment was not hand-picked (the lattice admits exactly one).

The follow-up is [w-1c3e77](w-1c3e77.md), opened on the same evidence, with an
independent measurement arm at [w-5d2a91](w-5d2a91.md). This item is closed and
is not reopened by them; the second example's status is unchanged, and it,
[w-2f7a10](w-2f7a10.md), [w-4b1e07](w-4b1e07.md) and
[w-a02d28](w-a02d28.md) still fold-close together on
`approximate_finds_classic_madgab_resegmentation` passing with its assertion
unmodified.
