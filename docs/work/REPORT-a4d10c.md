# REPORT a4d10c — default-visibility ordering for w-c31a07

Branch `madgab-order-a4d10c`, base `40f4b5d` (`origin/post-milestone-acceptance`).
`CARGO_TARGET_DIR=/workspace/target-a4d10c` throughout; no `src/` change was made.
`src/main.rs` untouched (disjoint front `agent-d4e2b0`).

**Verdict: HOLD.** Criterion 2 is a complete, priced negative across all five
surfaces. Recommend closing `w-c31a07` as a priced negative.

---

## Criterion 1 — which stage places the canonical at 27

**The ordering key, not the selection/diversity layer and not the final scorer.**

Measured at the library boundary on the release build via the public
`Generator::generate_pool` / `Generator::generate`:

| fact | value |
| --- | --- |
| canonical `wreck a nice beach` pool rank | **26** (= display 27) |
| canonical score | **0.919950** |
| canonical boundary structure | **`[3, 5, 10]`** |
| pool size (`recognize speech`) | **13,801** |
| gap to the 10th displayed | 0.919950 vs 0.921813 (`let egg nice pitch`) ≈ **0.0019** |

Stage-by-stage:

* **The scorer is not the stage.** The canonical's score is *correct*; it is
  simply 27th in a descending order of scores that are themselves within
  0.002 of each other. Nothing downstream re-scores it.
* **The ordering key is the stage.** The final order is the
  `picked.sort_by(|&a, &b| cmp_desc(clues[a].score, clues[b].score))` at
  **`src/lib.rs:4065`**, fed by the pool sort at **`src/lib.rs:2444`**. That
  descending-score key is what produces the display rank.
* **The selection/diversity layer is a pure pass-through at the shipped
  default.** At `top_n` = 10, 11 and 15, *every* displayed clue sits at exactly
  its own pool rank — verified over the whole displayed list, not a sample.
  `select_diverse` (`src/lib.rs:3967`), `share_cap` (`src/lib.rs:4114`) and
  `structure_reserve_slots` (`src/lib.rs:4142`) are all **slack** at the
  default, so the canonical is excluded by the **cutoff**, not by a diversity
  decision about it. Pinned by
  `the_selection_layer_is_a_pure_pass_through_at_the_shipped_default`.

  The cap does begin to bind as the list grows — first divergence from pool
  order at `top_n` = 12, and by `--top 50` the display order is materially
  rearranged (`let a guy speaks` shown at display 42, pool rank 64). That
  rearrangement is strictly downstream of the default and cannot rescue a
  near-tie that the default's own cutoff already excluded.

This is the precise answer to "selection layer, final scorer, or ordering key":
**the ordering key at `src/lib.rs:4065`**, acting on a correctly-scored
candidate, with the diversity layer uninvolved at the shipped default.

## Criterion 2 — priced negative, all five surfaces

Every surface was measured on this item's data. All five are now closed.

**1. Cluster merge — negative (inherited, re-confirmed structurally).** The
canonical is the only canonical-reading member that outranks nothing in its
own cluster; it loses *within* `[3,5,10]`, whose members above it are
`wreck a nice pitch` (0.921438), `wreck egg nice pitch/peach`, `let egg nice
beach`, `let ugh nice pitch`, `let a nice beach` and others. The three
structures ranked above it are `[3,4,10]`, `[2,6,10]`, `[3,6,10]`. Promoting by
`phrase_signature`/`clue_structure` or merging clusters cannot lift it because
the structure it needs to be promoted *within* is the structure it already
belongs to and still loses in. Pinned by
`canonical_loses_within_its_own_structure_not_to_a_cluster`: **156** pool
members share `[3,5,10]`, **12** of them outrank the canonical.

**2. Structure reserve (best-member-per-structure) — negative (inherited,
re-confirmed by criterion 1).** At the shipped default the reserve is slack
(`structure_reserve_slots(10)` slots go to structures that are already
visible), so it neither helps nor costs. Confirmed negatively: the canonical
is not the best member of its own structure, so no per-structure reserve can
promote it. Do not re-run.

**3. Content-word share — negative (inherited, re-measured).**
`wreck a nice beach` = **3/4** content words; `let egg nice pitch` = **4/4**
and outranks it at 0.921813. Share is *anti*-correlated with the canonical's
rank here, so promoting on it demotes it. Pinned by
`content_word_share_does_not_separate_the_canonical_from_its_outrankers`.

**4. Exact-IPA homophone — negative, and degenerate at phrase level.** Two
forms, both measured:

* *Phrase-level* (`Clue::ipa`): **degenerate**. All 13,801 pool members carry
  the same target IPA stream, so the whole pool collapses to **1** group and
  the entire list is deduplicated away. Useless as a selection rule.
* *Per-word* (`words[i].ipa` joined): **negative**. 13,801 → **10,133** groups
  (3,668 collapsed), canonical still at group rank **25** — outside the
  default ten. Adding the boundary structure to the key changes nothing
  material (10,217 groups, canonical still at 25): pitch/peach/beach are
  *not* IPA-identical, which is exactly why this surface was suspected
  insufficient. Pinned by
  `per_word_exact_ipa_grouping_leaves_the_canonical_outside_the_default`.

**5. Rhyme / ending-phoneme-run family — negative (new, this front).** This is
the surface the prior front identified and left open: the visible list is
several wordings differing only in the **final word** (`pitch` / `peach` /
`beach`), which are near-rhyme, not homophones. The general, bounded rule
tested is:

> group candidates by `(boundary structure, leading words, trailing run of the
> last word's IPA)`, and admit only the best-scoring member of each group.

This is keyed on no word list and no canonical sentence; the run length is the
only parameter. Priced at run lengths 1, 2, 3, 4 on both cases:

| trailing run | `recognize speech` groups | collapsed | canonical in visible top 10? |
| --- | --- | --- | --- |
| 1 phone | 9,645 | 4,156 | **no** |
| 2 phones | 10,396 | 3,405 | **no** |
| 3 phones | 11,796 | 2,005 | **no** |
| 4 phones | 12,694 | 1,107 | **no** |

**Why the rhyme family cannot lift it, which is the substantive result.** The
rule does work as redundancy removal — at run 2 it collapses 3,405 of 13,801
members and the visible head becomes genuinely distinct wordings
(`yeah 'cause i.'s pitch`, `let a guys pitch`, `let a nice pitch`,
`wreck a guys pitch`, `read 'cause i.'s pitch`, `let egg nice pitch`,
**`wreck a nice pitch`**, …). But it surfaces `wreck a nice pitch` (0.921438),
**not** `wreck a nice beach` (0.919950), because `wreck a nice pitch` is the
*best-scoring member of the canonical's own rhyme class* and therefore its
representative. A rhyme rule decides **which ending class** is shown; it
cannot change **which member of a class** is best. Since the canonical is not
the best member of its class by 0.0015, the rule substitutes a different
wording of the same alignment and never reaches the canonical. Pinned by
`rhyme_family_grouping_admits_a_different_member_not_the_canonical`.

This is a real, general, bounded improvement to the *display policy* on this
data (9 of the 10 visible clues are one of two rhymes of the same ending word,
pinned by `the_visible_head_is_one_ending_repeated_rather_than_distinct_wordings`).
It is **not** a fix for this item's stated goal, so it is not landed here —
landing it would change the visible list without delivering case 1, which is a
scope change no front should make unilaterally.

## Criterion 3 — regression test

`tests/display_ordering_attribution.rs`, 9 tests, all phrase literals in
`tests/`. Covers criterion 1's stage attribution, the pass-through property,
the priced negatives for all five surfaces with their exact numbers, and the
case-2 pool state.

```
cargo test --release --test display_ordering_attribution
test result: ok. 9 passed; 0 failed
```

## Criterion 4 — case 1 and case 2 re-measured on the shipped binary

`target/release/madgab --approximate --top 10 "recognize speech"` (verbatim):

```text
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

Case 1 unchanged: absent at the default, present at 27 of 50. Case 2, verbatim:

```text
 1. [0.921] it said thus test oop day
 …
10. [0.918] it justice too bad same
```

Neither is made worse — nothing changed. `approximate_finds_classic_madgab_resegmentation`
was **not** relaxed and **not** re-pinned.

## Criterion 5 — cost accounting

No source change, so no runtime cost is incurred. Baseline on the shipped
release binary, single run each:

| case | pool size | wall clock | expansions |
| --- | --- | --- | --- |
| `recognize speech` | 13,801 | 1619 ms (corpus 502 ms, search 946 ms) | unchanged — no `src/` diff |
| `It's just a stupid game` | 14,555 | 1401 ms (corpus 380 ms, search 858 ms) | unchanged — no `src/` diff |

**`cargo fmt` and `cargo clippy` did not run.** Neither subcommand is
installed in this environment (`error: no such command: fmt`,
`error: no such command: clippy`). I am not claiming them.

## Validation

```
cargo test --release --lib                 -> 83 passed; 0 failed; 12 ignored
cargo test --release --test no_phrase_hard_coding -> 9 passed; 0 failed   (9/0, src/ allowlist 0)
cargo test --release --test corpus_integration -- --test-threads=2
                                              -> 12 passed; 1 failed
```

The single failure is `approximate_finds_classic_madgab_resegmentation`, the
known pre-existing case-2 red, unmodified and not re-pinned.

## Recommendation

Close `w-c31a07` as a **priced negative**. Criterion 2 is explicitly accepted
as complete in this form, all five surfaces are covered with numbers, the
criterion 1 measurement is now stage-precise, and criterion 3 is pinned so none
of it is re-derived.

The one thing worth carrying forward as a *separate* item rather than a hold:
the rhyme-class grouping is a measured, general, bounded improvement to the
display policy that removes a real user-visible redundancy (6 of 10 visible
slots spent on two rhymes of one ending word) but does **not** deliver case 1
and must not be sold as if it did.
