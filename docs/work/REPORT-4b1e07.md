# REPORT-4b1e07 — independent adversarial review of the width front ebf5e7d

Work item **w-0f3a17**, branch `madgab-lattice-0f3a17`, commit
`adf1672..ebf5e7d` (`ebf5e7d78bc739f6ed947955858526db32e17629`), reviewed as
landed in worktree `/workspace/madgab-review-lattice` on
`madgab-review-lattice-4b1e07`. Nothing was fixed and no `src/lib.rs` edit was
made in the review worktree; the red-checks below were run in a throwaway
worktree on a detached `ebf5e7d`, which was removed afterwards.

`cargo fmt`, `cargo clippy` and doctests cannot run on this host and are not
claimed.

## 0. Verdict

**HOLD.**

Single strongest reason: **the priced negative's central table is arithmetically
wrong and self-contradictory, and the wrong number has been copied into
production `src`.** The record's §3 concludes that the per-slot width
derivation buys "9 / 7 / 10 / 10 / 93" at `d = 5` and therefore is "a constant
replacing a constant". Under the derivation the same document defines, slot 1
affords **10** (cost 3,998 ≤ 4,000), not 7; the derived row is
**9 / 10 / 10 / 10 / list**, which is monotone non-decreasing as the document
insists, whereas the printed `9 / 7` is *decreasing* and so contradicts the
very monotonicity the accompanying test claims to assert. The same wrong tuple
is duplicated into the doc comment of the new test at `src/lib.rs:4591`, where
it is contradicted six lines below by the test's own
`assert_eq!(previous, list)` with `list = SPAN_SHORTLIST = 160`.

A durable record whose load-bearing numbers are wrong — and whose wrongness is
copied into the crate — is exactly the artefact that mis-prices the successor
front. §5 below states the change that would make it integrable.

The second, independent defect (the test's maximality clause is unreachable
dead code, so the test is mutation-insensitive in the narrowing direction) is
real but smaller and separately fixable.

## 1. Is the `src` change behaviour-neutral?

**Yes.** Verified against the diff, not the message.

The `src/lib.rs` diff is 3 hunks: +144/−1 lines, of which 138 are the new
test. The non-test part is exactly two changes and nothing else:

* `src/lib.rs:982` (pre) `const LEXICAL_HEAP_POP_LIMIT: usize = 4_000;` is
  deleted from the body of `generate_approximate`;
* `src/lib.rs:261` (post) `const LEXICAL_HEAP_POP_LIMIT: usize = 4_000;` is
  added at module scope, with a doc comment;
* a three-line explanatory comment replaces the old declaration in place.

The value is **4,000 on both sides** — unchanged, digit for digit. Every use
site is unchanged in text and in meaning; `grep -n LEXICAL_HEAP_POP_LIMIT`
over `adf1672:src/lib.rs` and `ebf5e7d:src/lib.rs` returns the same five
sites (doc comment at 234/261, the `SEGMENTATION_KEEP *` global product, the
`affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT)` call at
1899→1910, and the two `popped >= LEXICAL_HEAP_POP_LIMIT` guards at
1962→1973 and 2034→2045), at each of which the name resolves to the same
value after the move. Moving an associated const on `impl Generator` to module
scope cannot change a value and introduces no shadowing (no other binding of
that name exists).

Corroborating: the behaviour-sensitive guard `approximate_output_is_locked`
passes (§3), so the pinned pool sizes did not move. `tests/` is **byte-identical**
across the commit (`git diff --numstat adf1672..ebf5e7d` lists only
`docs/work/items/w-0f3a17.md` and `src/lib.rs`), so no literal was re-pinned
and the record's claim to that effect is honest.

## 2. Is the new test non-vacuous?

Partly. Two loops; they are not equally strong, and one of them contains the
sibling-review defect class in a narrower form.

**Loop 1** (`for depth in 1..=9`, `for pop_limit in [1, 16, 111, 1_111,
4_000, 40_000]`) asserts, for `w = affordable_opening_width(depth,
pop_limit)`, that a locally re-implemented `internal_nodes` — a copy of the
`1 + w + ... + w^(d-1)` series inside the production `fits` closure — satisfies
`w == 1 || internal_nodes(uniform w) <= pop_limit`. This is, on its face, the
sibling defect: the returned width is compared against a quantity re-derived
from the same expression the production function used. It is **not** fully
vacuous, because the copy is independent and the series is a real quantity; I
confirmed by mutation that it does bite:

* **MUT-1** (delete the `while width > 1 && !fits(width)` narrowing loop, so
  the derivation degenerates to "always the widest, 10"): the test **fails** —
  `depth 2 at pop_limit 1: width 10 needs 11 internal nodes`. Good.
* **MUT-2** (change the production series from `1 + w + ... + w^(d-1)` to
  `w^(d-1)` alone, a genuine narrowing of the derivation): the test
  **fails** — `depth 5 at pop_limit 111: width 3 needs 121 internal nodes`.
  Good, and this is the guard the "self-referential" reading would have
  missed.

**But the maximality clause in loop 1 is dead code and can never execute.**
It is guarded by `if w > LEXICAL_BRANCH_STAGE_0`, and
`affordable_opening_width` starts at `width = LEXICAL_BRANCH_STAGE_0` and only
ever decrements, so `w` is bounded above by `LEXICAL_BRANCH_STAGE_0` by
construction. I confirmed this by enumerating all 54 cells with an independent
re-implementation: the clause is exercised in **0 of 54** cells. The assertion
the test appears to make — *"`{w}` is not the largest affordable width"*, i.e.
that the derivation is maximal — is therefore never evaluated, and its message
is unreachable text.

Consequence, demonstrated:

* **MUT-3** (`width -= 1` → `width -= 2`, an internally consistent narrowing
  that silently returns a width about 2× below the largest affordable at every
  depth ≥ 5): the **entire 62-test `--lib` suite passes green**. A defect that
  throws away a large fraction of the reachable width, in the one direction
  the test's maximality clause was written to catch, is invisible to the suite.

**Loop 2** (`for depth in 3..=8`, `pop_limit = LEXICAL_HEAP_POP_LIMIT`) is the
substantive half and is **non-vacuous**. It recomputes, per slot, the widest
`w ≤ SPAN_SHORTLIST` affordable with the other slots held at
`affordable_opening_width(depth, 4_000)`, and asserts (a) the affordable width
is non-decreasing in slot position, (b) never below the uniform baseline, (c)
the last slot affords its whole list, (d) when the baseline has been reduced
below `LEXICAL_BRANCH_STAGE_0`, the penultimate slot does **not** afford its
whole list. My independent enumeration reproduces all of it:

```
d=3 baseline=10 -> [160, 160, 160]
d=4 baseline=10 -> [ 36,  36,  38, 160]
d=5 baseline= 7 -> [  9,  10,  10,  10, 160]     penultimate-at-160: 55,280 > 4,000
d=6 baseline= 5 -> [  5,   5,   5,   5,   5, 160]  penultimate-at-160: 100,781 > 4,000
d=7 baseline= 3 -> [ 10,  11,  11,  11,  11,  14, 160]  penultimate-at-160: 39,244 > 4,000
d=8 baseline= 3 -> [  3,   3,   3,   3,   3,   3, 160]  penultimate-at-160: 117,733 > 4,000
```

The baseline column matches the record's §1 first-stage-cap column for every
depth (10/10/7/5 for `d = 3/4/5/6`, 3 at `d = 7, 8`). The `baseline <
LEXICAL_BRANCH_STAGE_0` branch is genuinely live — it fires for `d = 5, 6, 7, 8`
— and the penultimate-slot assertion is a real, non-trivial bound in each. The
monotonicity law and the "last slot is free" law are the two things the record
says the test "now asserts", and it does assert them, correctly.

So: the test is worth keeping and does pin the charging law. Its defect is the
unreachable maximality clause, not the self-reference — loop 1 is
self-referential in form but mutation-tested to catch real divergence, whereas
the clause that would have made it a two-sided guard is dead.

## 3. Integration gate

Run in `/workspace/madgab-review-lattice` at `ebf5e7d`, release, with
`CARGO_TARGET_DIR=/workspace/target-review-4b1e07` (set by this review; a fresh
directory, per the item's own constraint that a target dir must not be shared
with a probe worktree).

| command | result |
|---|---|
| `cargo test --release --test corpus_integration` | **12 passed, 1 failed** (13 total) |
| `cargo test --release --lib` | **62 passed, 0 failed** |
| `cargo test --release --test no_phrase_hard_coding` | **9 passed, 0 failed** |

The single failure is **`approximate_finds_classic_madgab_resegmentation`**,
and it is the expected and pre-existing one — the criterion-1 target this very
work item is trying and failing to reach:

```
panicked at tests/corpus_integration.rs:136:
canonical clue missing from top 50; got: ["it said thus test oop day",
"it said thus 'cause too day", ...]
```

The three `approximate_pool_reaches_*` guards are **all green**:

* `approximate_pool_reaches_resegmentations_deeper_than_one_walk` — ok
* `approximate_pool_reaches_matches_deep_in_a_span` — ok
* `approximate_pool_reaches_alternatives_past_the_opening_slot_width` — ok

This is the substantive integration result. `tests/` is byte-identical across
`707fb2a`, `b49f892` and `ebf5e7d`, so the two guards that were green at
`707fb2a` and red at `b49f892` are green again at `ebf5e7d`; the only thing
that differs is `src/lib.rs`, and `ebf5e7d`'s `src` change is the
value-neutral const move of §1. The record's §5 claim that
`approximate_output_is_locked` and the two pool-reach guards are untouched is
verified true. **`ebf5e7d` therefore does not regress the criterion-5 fence
relative to `707fb2a`**, and the src half of this diff is exonerated on the
gate.

`no_phrase_hard_coding` is fully green (9/9), including
`no_canonical_example_in_a_production_doc_comment` and
`the_allowlist_is_small_and_every_entry_justifies_itself`.

## 4. Phrase-specific hard-coding, magic literals, benchmark-shaped constants

**No phrase-specific hard-coding.** The diff adds no target string, no clue,
no word, no per-slot index, and no re-pinned acceptance literal. The new test
names only `affordable_opening_width`, `LEXICAL_HEAP_POP_LIMIT`,
`LEXICAL_BRANCH_STAGE_0` and `SPAN_SHORTLIST`. The
`no_phrase_hard_coding` fence agrees at 9/9. The diff contains no new
`ZZ_*`/`zz_*` probe instrumentation and no survey-dump test, as the item's
traps require.

Two lesser smells, neither a hard-coding finding:

* **The new test hard-codes `4_000` as a literal in loop 1** while
  `LEXICAL_HEAP_POP_LIMIT` — the very constant the whole `src` change exists to
  *hoist so that it can be named and asserted on* — is now in scope. Loop 2
  names it correctly (`let pop_limit = LEXICAL_HEAP_POP_LIMIT;`); loop 1 puts
  `4_000` in a literal list beside it. The stated justification for the diff is
  not followed through in half of the test that motivated it.
* Loop 1's `pop_limit` sweep `[1, 16, 111, 1_111, 4_000, 40_000]` is
  hand-chosen. `111 = 1+10+100` and `1_111 = 1+10+100+1000` are legitimately
  derived (they straddle the `d = 3` and `d = 4` crossovers), but `16` and
  `40_000` are arbitrary. This is a defensible test-design choice, not a
  benchmark-shaped constant: nothing in it is fitted to a pass/fail outcome.

## 5. What to change to make this integrable

Described, not made. Four edits, all small:

1. **Correct §3 of `docs/work/items/w-0f3a17.md`.** Under the derivation the
   document itself states, at `pop_limit = 4 000` with 160-wide lists and
   baseline 7, the per-slot widths are **9 / 10 / 10 / 10 / 160**, at internal
   node costs 3,601 / 3,998 / 3,977 / 3,830 / 2,801. Slot 1's cost at
   `w = 10` is `8 + 399w = 3,998 ≤ 4 000`, so 10 is affordable and 7 is not
   the maximum — the printed `7` is the *baseline*, not the derived width.
   The `d = 6` row's last-slot cost is also wrong: at baseline 5 it is
   `1+5+25+125+625+3125 = 3,906`, not the `2,801` carried over from the `d = 5`
   row. Keep the `93` only where it is earned — as the *measured* list width of
   the canonical segmentation, in §2 — and drop it from the §3 table, whose
   header already says "160-wide lists".
2. **Fix or delete the duplicated tuple in the new test's doc comment**
   (`src/lib.rs`, the sentence reading "at `d = 5`, `pop_limit = 4 000` it is
   9 / 7 / 10 / 10 / 93 against a uniform 7"). Either restate it as
   9 / 10 / 10 / 10 / 160 to match what the loop below computes, or drop the
   numbers and point at the work item. As landed it is refuted by the
   `assert_eq!(previous, list)` six lines below.
3. **Make the maximality clause live or delete it.** The `if w >
   LEXICAL_BRANCH_STAGE_0` guard is unreachable because the derivation is
   capped above by that constant. Either drop the guard and assert maximality
   unconditionally for every cell where `internal_nodes(&vec![w + 1; depth])`
   is the relevant comparison (handling `w == 1` by skipping the `w = 0`
   degenerate case), or — if the cap is genuinely part of the contract —
   replace it with the assertion that actually matters and is currently
   unstated: *when the series at `LEXICAL_BRANCH_STAGE_0` exceeds the pop limit,
   the returned width is the largest `w` whose series fits, computed
   independently in the test.* That is the assertion MUT-3 defeats today.
4. **Reconcile the three pop-utilisation figures.** The same measurement is
   reported as `0.9-43.6%` in §1 point 3 (consistent with the per-target
   table's last column), as `5-35%` in the same point, and as `5-44%` in §4
   and in the test's doc comment. Pick the per-target range and use it in all
   four places.

The `src` const move itself needs no change and the test's loop 2 needs no
change; both are sound and worth landing.

## Note on the emission-bound conclusion

Not a finding against the diff, recorded for the successor front: the negative's
load-bearing argument is that the traversal is emission-bound rather than
pop-bound, and the evidence supports it — 52 emissions per segmentation
against a 64 allowance (81%) while spending 0.9-43.6% of
`LEXICAL_GLOBAL_POP_BUDGET` and saturating
`LEXICAL_GLOBAL_EMISSION_BUDGET` at 16,384/16,384. The 81% figure is
conspicuously short of saturation per segmentation, so the conclusion is
"mostly emission-bound", not "emission-bound"; the record's own §4 correctly
redirects the successor front to the coverage reserve's shape rule (corner-only
shapes, at most 3 non-zero coordinates, which §2 shows cannot supply the
canonical tuple's two deep coordinates) rather than to the width. That
redirect is the durable part of this record and it is sound.
