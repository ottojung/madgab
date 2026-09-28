# Front report 3f6a21 — the adjacent-slot (boundary-move-as-one-event) substitution charge

Work item: [`items/w-3f6a21.md`](items/w-3f6a21.md)
Branch: `madgab-pairscore-3f6a21` — base `9a572c4` (`post-milestone-acceptance`)
Probe worktree: `/workspace/3f6a21-probe`, branch `scratch/3f6a21-probe`, **never
merged, never pushed**; deleted at the end of this front.

## Verdict

**HOLD.**

Zero production lines land. `git diff 9a572c4 --stat -- src tests` is **empty**
on this branch. No cutoff, budget, threshold, lock or test was re-pinned.

Both general forms the item asked to be priced were priced, and both are
measured to be worth nothing for the fence they were opened for:

* **form (a)**, score the concatenated clue IPA of an adjacent slot pair
  against the concatenated target span, delivers a discount of exactly
  **0.000000** on the canonical pair and on the canonical clue as a whole.
* **form (b)**, a bounded cross-slot term derived from the pair's own phonetic
  distance, has no derivation that is not form (a) — and form (a)'s derived
  bound is 0.000000. Priced at the most generous bound the model can even
  name (a full one-gap allowance at every one of the four boundaries, 0.80 of
  cost, four times any reading the code supports) it still leaves the
  canonical clue **0.05926521** below the visible band, i.e. **9.43×** the
  whole width of the top-50 score band.

## The one number that decides it

The `SIMILARITY` axis is read per *phone* over the whole target, not per
slot, so the substitution charge is already a whole-clue quantity. For the
canonical tuple the entire headroom left on that axis is

```text
per-slot costs  [0.2, 0.0, 0.15, 0.3695198141657896, 0.4]   total 1.1195198141657896
target phones                                              19
cost per phone                                             0.0589220955
SIMILARITY = 1 - 0.0589220955 / 0.30 = 0.80359300
axis term = 0.25 * 0.80359300 = 0.20089825
headroom  = 0.25 * (1 - 0.80359300) = 0.04910175
```

and the measured deficit to the visible band is

```text
canonical tuple score                     0.8207695329
50th-highest score in the pool            0.9151215745
deficit                                   0.0943520416
width of the whole top-50 score band      0.9214042124 - 0.9151215745 = 0.0062826379
deficit as a multiple of that band        15.02x
headroom on the one axis a charge reads   0.04910175   (52.0% of the deficit)
deficit the similarity axis cannot reach  0.04525029   (47.9%)
```

So a substitution-charge change is bounded above by moving half the gap, and
the measured value of its move is zero.

## What I measured first, on `9a572c4`, before editing anything

Release build, this worktree's own `CARGO_TARGET_DIR`, defaults, shipped
`CORPUS_JSON`, wall clock around the executable.

| target | canonical clue | pool rank | score | top-1 | 50th | wall |
| --- | --- | --- | --- | --- | --- | --- |
| `recognize speech` | `wreck a nice beach` | **27 of 50**, in the pool | `0.9199502875` | `yeah 'cause i.'s pitch` `0.9236413310` | `wreck ugh nice peach` `0.9187964409` | **1681 ms** |
| `It's just a stupid game` | `hits justice dupe hid came` | **absent** | — | `it said thus test oop day` `0.9214042124` | `each thus tough too pad same` `0.9151215745` | **1704 ms** |

Visible top-50 redundancy, measured as the number of distinct first-four-word
signatures among the 50 printed rows:

| target | rows | distinct signatures | largest family |
| --- | --- | --- | --- |
| `recognize speech` | 50 | 22 | 5 (`yeah 'cause i.'s`) |
| `It's just a stupid game` | 50 | 29 | 5 (`it justice too bad`) |

Pool sizes: 19,635 and 20,616 clues under `generate_with_pool` at
`top_n = 100_000`; 18,289 and 18,949 under the CLI's own `top_n = 50` path.
The canonical case-2 clue is absent from both.

## Obligation 2 — the case-2 gap, decomposed: reach part **0.000000**, score part **0.0943520416**

Instrumented at the enumeration boundary (the slot shortlists the traversal
expands), for the structure the canonical tuple occupies,
`[(0,3),(3,10),(10,13),(13,15),(15,19)]`:

```text
slot 0  span (0,3)   hits     ipa hɪts     cost 0.200000    (cheapest in slot: it's / ɪts, 0.0)
slot 1  span (3,10)  justice  ipa dʒʌstəs  cost 0.000000    (cheapest in slot: justice, 0.0)
slot 2  span (10,13) dupe     ipa dup      cost 0.150000    (cheapest in slot: coop, 0.11424)
slot 3  span (13,15) hid      ipa hɪd      cost 0.369520    (cheapest in slot: add, 0.10123)
slot 4  span (15,19) came     ipa keɪm     cost 0.400000    (cheapest in slot: game, 0.0)
total 1.119520  <=  total_budget 1.5, and every slot cost <=  per_word_budget 0.5
```

**The item's context claim that "`hid` is pruned from the retained shortlist
for its target span" is false on this base.** Across the whole run `hid` is
retained in 100 slot shortlists, at a minimum cost of 0.350000, and in this
structure's slot 3 at 0.369520. The same holds for all five words (`hits`
143 shortlists, `justice` 160, `dupe` 149, `came` 176).

**Reach part = 0.000000.** Not one word of the canonical clue is pruned, the
tuple's total cost is inside `total_budget`, and every per-slot cost is
inside `per_word_budget`. Forced through the same
`Partial::extend_fuzzy` → `into_clue` path the traversal itself uses, the
tuple builds and scores **0.8207695329** with `cuts = [3,10,13,15,19]`. The
shipped run never *scores* it — the traversal's per-slot opening width and
the coverage reserve's tuples do not draw those positions — so what the
traversal costs this clue is exactly nothing; it is simply never scored.

**Score part = 0.0943520416**, the whole deficit, and it is all of it: the
tuple is payable and still 15.02 band-widths short. Which part can a pair
charge plausibly move? Only the score part, and only the 0.04910175 of it
that sits on `SIMILARITY`; the other 0.04525029 is on axes a substitution
charge does not read. Measured axis profile of the tuple's near-sibling at
structure `[(0,3),(3,11),(11,13),(13,15),(15,19)]` (score `0.8098046206`),
against the green case-1 clue `wreck a nice beach` (score `0.9199502875`):

| axis | canonical sibling | case-1 green |
| --- | --- | --- |
| similarity | 0.7597 | 0.8690 |
| novelty | 0.6667 | 1.0000 |
| word_novelty | 1.0000 | 1.0000 |
| familiarity | 0.3987 | 0.6206 |
| rhythm | 1.0000 | 1.0000 |
| shape | 1.0000 | 1.0000 |
| closed_class penalty | 0.0000 | 0.0625 |
| punch | 0.8000 | 1.0000 |

The deficit is concentrated in **novelty and familiarity**, both of which are
already at their maximum for the case-1 clue. That is where the 0.0453 lives.

## Obligation 3 — two general forms, priced

Measured with the fuzzy trie's own cost model (`GAP_COST = 0.20`, per-character
`phonetics::distance`, banded Levenshtein), re-implemented independently in the
probe so the arithmetic is visible.

### Form (a) — concatenated clue IPA against the concatenated target span

| pair | per-slot | concatenated | discount |
| --- | --- | --- | --- |
| slot 2 + 3, the pair that splits one target word (`tup`+`əd` vs `dup`+`hɪd`) | 0.150000 + 0.369520 = 0.519520 | 0.519520 | **0.000000** |
| slot 0 + 1 (`ɪts`+`dʒʌstəs` vs `hɪts`+`dʒʌstəs`) | 0.200000 + 0.000000 = 0.200000 | 0.200000 | **0.000000** |
| the whole five-slot clue against the 19-phone target | 1.119520 | 1.119520 | **0.000000** |

**Why it is zero, structurally.** The per-slot cost is *already* an
unconstrained Levenshtein alignment of the target span against the clue word,
with insertions and deletions allowed at the span's ends and no constraint at
either. Concatenating the two optimal per-slot alignments is therefore a
feasible alignment of the union, so the union's optimum is at most the sum of
the parts — the charge is never *worse* for splitting a word, which is
consistent with the objective's own doc comment that reading the axis per
phone "removes" the span-length artefact. But it is also never *better*:
nothing is charged at the boundary in the first place, so there is no
double-charge to discount. **The premise this front was opened on — that a
resegmentation is charged twice — does not hold in this cost model.** A third
clue not selected for this item (`tits justice tuba dog way mm`, 6 slots,
per-slot sum 0.75) also gives a whole-clue concatenation of exactly 0.75.

### Form (b) — bounded cross-slot term from the pair's own phonetic distance

The only bound derivable without a free constant is

```text
charge = min( sum of per-slot costs , d_ipa(concat clue pair, concat target pair) )
```

which is form (a) restricted to a pair, and therefore **0.000000** by the
table above. Getting a nonzero bound requires a premise the model does not
contain. The only magnitude in it that could serve is `GAP_COST = 0.20`, the
price the model already charges for one inserted or deleted phone. Priced at
the most generous version of that reading — a full one-gap allowance for
*every* boundary of the five-slot clue, `4 * 0.20 = 0.80` of cost, several
times anything the model supports:

```text
cost 1.1195198 - 0.80 = 0.3195198
cost per phone 0.01681789 -> similarity 0.94394037 -> term 0.23598509
gain 0.03508684 -> new score 0.85585637
deficit remaining to the 50th-place band  0.05926521  =  9.43x the band width
```

So the unprincipled upper bound on form (b) still misses by 9.43 band widths.
Form (b) is a strict improvement on form (a) only in the sense that (a) is
exactly the identity and (b) is a smaller identity.

## Verdict detail: which fences, and why none were re-run

No rule survived, so no production line was written and **no fence was
re-run against a change** — there is no change to re-pin. For the record, the
fences' state on this branch is exactly the base's, and the base's is:

| command | result |
| --- | --- |
| `cargo build --release` | Finished, 26.81 s, no warnings shown |
| `cargo test --release --lib` | **75 passed / 0 failed / 12 ignored**, 26.52 s |
| `cargo test --release --test corpus_integration -- --test-threads=2` | **12 passed / 1 failed**, 27.17 s — the pre-existing `approximate_finds_classic_madgab_resegmentation` |
| `cargo test --release --test emit_coverage -- --test-threads=2` | **4 passed**, 5.51 s |
| `cargo test --release --test approx_determinism -- --test-threads=2` | **4 passed**, 47.69 s |
| `cargo test --release --test exact_determinism -- --test-threads=2` | **1 passed**, 5.82 s |
| `cargo test --release --test no_phrase_hard_coding -- --test-threads=2` | **9 passed**, 0.01 s |

`git diff 9a572c4 --stat -- src tests` is empty, so
`approximate_output_is_locked`,
`approximate_pool_reaches_resegmentations_deeper_than_one_walk` and
`approximate_pool_reaches_matches_deep_in_a_span` are untouched and neither
is the pre-existing `corpus_integration` failure re-pinned.

Obligation 5, the hard-coding check: `git grep -i -E
"wreck|beach|recognize|justice|stupid|dupe|came|hid" -- src` returns 49 hits,
all of which predate this front and all of which are either inside
`#[cfg(test)]` fixtures (`src/lexical.rs:302`, `src/lexical.rs:335`,
`src/lib.rs`'s test module) or CLI usage examples and ordinary English in
`//!` comments (`src/main.rs:9`, `src/main.rs:11`, `src/lexical.rs:19`).
Nothing in production reads either sentence.

`cargo fmt`, `cargo clippy` and doctests **cannot run on this host** (see
[`../environment-notes.md`](../environment-notes.md)) and are **not claimed**.

## UNREACHED

Honest list of what this front did not get to, with the reason.

1. **The distribution of form (a)'s discount over the whole pool — UNREACHED.**
   Two sweep invocations over adjacent slot pairs hit the 400 s and 420 s
   shell timeouts, and a third burned 30 minutes, because
   `phonetics::distance` is recomputed per character pair inside the DP and
   the pool is ~20,000 clues per target. Memoisation of the confusion table
   was added to the probe and the sample cut to 200 clues, but the run was
   cut by the coordinator steer before it was re-executed. **Wall clock lost
   to these three runs: ~44 minutes of the front's ~75.**
   *Consequence for the verdict:* the structural argument above shows the
   discount is **≥ 0** in general (the union optimum can never exceed the sum
   of the parts) but does **not** by itself show it is 0 in general. The
   claim "form (a) is the identity" is therefore established for the
   canonical clue and for the four direct measurements, and *not* established
   pool-wide. That is enough to refuse the charge for this fence — on the
   canonical case it is exactly zero, so it cannot move the fence — and it is
   **not** enough to retire the idea for pool quality. A successor that wants
   the general claim must run the bounded sweep.
2. **A run of the locked fences against a candidate — NOT APPLICABLE.** No
   rule survived to be written down, so there was nothing to run them against.
   The suite table above is the base baseline, not a post-change result.
3. **A regression test — NOT WRITTEN, deliberately.** The item requires one
   only `if INTEGRATE`. A test asserting a mechanism that measured to be the
   identity would be a test asserting nothing, and one asserting the canonical
   pair is forbidden. Writing either would be worse than writing none.
4. **A weight vector — NOT RUN, on instruction.** agent-e086cc closed the
   weight lever and the placement lever this week; no weight vector was
   varied here.

## Successor rule, one line

> Stop pricing the *shape* of the substitution charge: the canonical tuple is
> already payable at cost 1.11952 against a budget of 1.5 and still 0.09435
> below the visible band, the entire headroom on the one axis a charge reads
> is 0.0491 with a measured value of 0.0000, and 0.0453 of the deficit sits in
> **novelty and familiarity** where the green case-1 clue is already at 1.0 —
> so price those two axes, and budget any probe of the pair charge with a
> memoised confusion table and a bounded sample so it finishes inside the
> shell timeout instead of costing 44 minutes.

## Verification actually run on this branch

`src/` and `tests/` are byte-identical to `9a572c4`. The only content of this
branch is this report, the work item, and the item's handoff.

| command | result |
| --- | --- |
| `cargo build --release` | Finished, 26.81 s, no warnings shown |
| `madgab --approximate --top 50 "recognize speech"` | `wreck a nice beach` at rank 27, wall 1681 ms |
| `madgab --approximate --top 50 "It's just a stupid game"` | canonical absent, top-1 `it said thus test oop day`, wall 1704 ms |
| `cargo test --release --lib` | 75 passed / 0 failed / 12 ignored |
| `cargo test --release --test corpus_integration` | 12 passed / 1 failed (pre-existing) |
| `cargo test --release --test emit_coverage` | 4 passed |
| `cargo test --release --test approx_determinism` | 4 passed |
| `cargo test --release --test exact_determinism` | 1 passed |
| `cargo test --release --test no_phrase_hard_coding` | 9 passed |
| `git diff 9a572c4 --stat -- src tests` | empty |
| `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid" -- src` | 49 pre-existing hits, all test fixtures / usage examples / comments |

Each worktree used its own `CARGO_TARGET_DIR`; nothing was shared with any
other worktree of this package, per `w-a1f3d2`'s recorded hazard.
