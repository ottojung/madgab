---
work_item: true
id: w-9d4e17
state: done
priority: high
owner: agent-9d4e17
updated: 2026-09-26T12:45:00Z
branch: madgab-slot-width
worktree: /workspace/madgab-slot-width
---

# Stop truncating each slot's candidate list before the traversal sees it

## Goal

The per-segmentation lexical enumeration cannot reach a word that sits deep
in its span's candidate list, because every slot's list is truncated to a
uniform `LEXICAL_BRANCH_KEEP` **before** the product traversal starts. Make
the depth of a slot's list a property of the traversal's budget rather than a
pre-filter applied identically to every slot, so that a word which is deep in
one slot is reachable by the traversal that has budget to spend there.

## Context: what is already established, do not re-measure it

- Every one of the five canonical words survives the per-span shortlist
  (`SPAN_SHORTLIST = 160`). Per-slot ranks inside the span's match list are
  **6-8 / 0 / 10-18 / 99 / 11**; the "hid rank 85" and "hid rank 99" figures
  in this repository's documents are the same word in two different
  orderings, not a disagreement ([w-3e5c7](w-3e5c7.md), integrated as
  `24f3cf7`).
- The canonical segmentation is retained by the structural DP at rank 151 of
  256 and the matcher is innocent ([w-7b41d2](w-7b41d2.md)).
- `LEXICAL_BRANCH_KEEP = 10` truncates each slot's list to ten alternatives
  before enumeration, so `hid`, at walk rank 99, is **never expanded**. No
  visit order can reach a node that is never pushed.
- Shortlist reordering, walk-rank ordering, and a budget-derived *per-slot*
  cap have all been implemented and **refuted by measurement**
  ([w-7b41d2](w-7b41d2.md), [w-04f83f](w-04f83f.md)). The per-slot cap failed
  because per-word costs of 0.0-0.5 against a `total_budget` of 1.5 make
  total-cost affordability vacuous. Do not re-run either.
- Raising the constant uniformly is also **refuted with a number**: with
  `LEXICAL_BRANCH_KEEP` at the full 160-wide shortlist, the release binary
  takes **20.0 s** for the canonical target (a 10x wall-clock regression,
  which this repository's own criterion forbids) and the clue is **still
  absent** ([w-6b2f04](w-6b2f04.md), integrated as `eaab259`). That 20 s was
  measured under the *current* best-first traversal, which pays roughly the
  product of the fan-outs; it is not a property of the width.
- The whole budget family is refuted: the global emission budget, the global
  pop budget, per-segmentation depth, per-slot width, and every allocation of
  them across 256 segmentations. `LEXICAL_GLOBAL_EMISSION_BUDGET`,
  `LEXICAL_GLOBAL_POP_BUDGET`, `structure_depth_ceiling` and
  `structure_wording_allowance` are now on the accumulation branch and are
  **not** to be edited by this front.

## The mechanism this front owns

The traversal, not the list length, should decide how deep a slot is read.

Concretely: keep each slot's full match list (it is already built and sorted
by `SlotAlt::contribution`), and let the existing per-segmentation emission
and pop accounting stop reading a slot once the traversal's budget cannot
use more of it. The uniform pre-filter is then replaced by a *reachability*
rule: a slot may be read deeply only when the traversal is actually spending
on that level, so a level that is never visited costs nothing, and a level
that is visited deeply is paid for out of the budget that already bounds it.

The general property to preserve, and to state in a test: **the truncation
must never be able to remove a candidate that the traversal would have
emitted**, and it must never cost more than the traversal it replaces. A
mechanism that keeps the full lists and relies on the traversal's bound is
preferred over any new constant.

## Fence — this front must not duplicate its neighbour

[w-8f3c61](w-8f3c61.md), agent `8f3c61`, in
`/workspace/madgab-depth-order` (branch `madgab-depth-order`), owns the
**visit order** of the index tuples: max-index-first stratification so depth
in one slot is additive rather than multiplicative in the levels.

- The traversal's ordering, its schedule, its heap key and its bound
  arithmetic belong to `w-8f3c61` and must not be edited here.
- The **list contents and where they are truncated** belong to this front.
- If your change needs a bound the traversal does not already apply, say so
  in this item and let the coordinator sequence the two. Do not reach into
  the other front's code to get it.
- Both fronts are necessary and neither is sufficient: without this one the
  node is never pushed, and without the other one the depth costs a product
  rather than a sum. A refutation with numbers, on either side, is an
  acceptable and useful outcome.

## Constraints

- No phrase-specific special case for these examples, their words, or any
  substring of them. Check with `git grep -i -E` on your own branch before
  you report. Temporary diagnostics (the existing `MADGAB_TRACE_*` env-gated
  facility, and the earlier fronts' `zz_dump_canon` / `zz_scratch_analysis.rs`
  precedent) must **not** reach the accumulation branch.
- No `axes::*` weight moves and no changes to the scoring functions; the
  blocker is enumeration shape, not the objective. Re-baselining
  `approximate_output_is_locked` is *not* expected; if it changes, that is a
  finding to report loudly.
- Regression tests express useful external behaviour on a spread of targets,
  not implementation details and not the two acceptance phrases.
- Validation reported from your own tree, not copied forward:
  `cargo test --release --lib`, `--test corpus_integration`,
  `--test exact_determinism`, `--test approx_determinism`. Expect
  `approximate_finds_classic_madgab_resegmentation` in `corpus_integration` to
  pass; that is the point of this item. `cargo fmt`, `cargo clippy` and
  doctests do not exist on this host — see
  [../../environment-notes.md](../../environment-notes.md); do not report them
  as satisfied.
- Commit on `madgab-slot-width`, **push it**, rebase onto the current
  `post-milestone-acceptance` head before final validation (never merge or
  push to `main`), and leave the working tree clean with nothing valuable
  left uncommitted. Check `git branch --contains <your head>` before you
  finish: a finished agent's work on a detached HEAD is unrecoverable from a
  later pass's point of view.

## Deliverable order

1. The release-binary measurement, first and before any polish:

   ```sh
   MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
       ./target/release/madgab --approximate --top 200 "It's just a stupid game"
   ```

   Report whether the clue is found, the pool size, and the wall clock, on
   the current head as a baseline and on your change.
2. Wall clock and pool size before and after, for at least three
   non-canonical targets including `congratulations on your promotion`. A
   change that buys reach by making every search 10x slower has not solved
   the milestone; a change that buys reach by making the *untraversed* levels
   free has.
3. Only then tests and documentation.

## Handoff / notes

Filed by coordinator pass `coord-c3d81` on 2026-09-26T13:10Z, from the
refutation integrated at `eaab259`. The split exists so the milestone does not
rest on a single live agent, and it is motivated by measurement rather than
by convenience: [w-6b2b04](w-6b2f04.md) showed that full width costs 20 s and
still misses *under the current traversal*, which is precisely the cost that
`w-8f3c61`'s order removes. Neither constant can be changed alone; the width
is only affordable once depth is additive.

Ownership: branch `madgab-slot-width`, worktree `/workspace/madgab-slot-width`,
created from `eaab259`. This is a fresh front; no earlier pass has touched it.

## Pass 2026-09-26T12:45Z (agent 9d4e17): the pre-filter is gone, and it was
## not the binding constraint — refuted with numbers

Branch `madgab-slot-width` at **8582525**, pushed, rebased onto the current
`post-milestone-acceptance` head **85a8a60**. Two commits, `src/lib.rs` and
`tests/corpus_integration.rs` only, +261/-46. Not merged anywhere and not
proposed for `main`.

### What landed

The uniform `LEXICAL_BRANCH_KEEP = 10` truncation in front of the
per-segmentation product search is **deleted**, not retuned. What replaces it
is not a number at all but a question asked of the traversal:

- `next_branch_stage(current, widest) -> Option<usize>`
  (`src/lib.rs:2766`) is the whole rule. The search opens every slot at
  `LEXICAL_BRANCH_STAGE_0 = 10` — the old constant, so the first stage costs
  exactly what the pre-filter cost — walks that width to exhaustion, and
  asks for the next one (`x4`, so 10 -> 40 -> 160 spans a `SPAN_SHORTLIST`
  in three stages) **only if it is still hungry**. `None` means every
  alternative of the widest slot has been read.
- A widening re-expands the *already expanded* nodes over the newly legal
  indices, so opening a level costs the new nodes only, never a second walk
  of the lattice.
- The gate is the traversal's own accounting, not a new constant: the
  per-segmentation emission allowance, `LEXICAL_HEAP_POP_LIMIT`,
  `LEXICAL_GLOBAL_EMISSION_BUDGET` and `LEXICAL_GLOBAL_POP_BUDGET` all still
  apply, unchanged, and the new rule adds no bound of its own. **What the
  pre-filter bounded was reach; after this change reach is bounded only by
  the traversal, and the traversal is bounded by the same four numbers as
  before.** A level the traversal never reaches is never paid for.
- `SPAN_SHORTLIST` moved from the search's body to module scope so the width
  schedule and its test name the same 160 the shortlist admits; there is one
  constant, not a restatement.

No `axes::*` constant and no scoring function is touched. The neighbour's
fence was respected: the heap key, the suffix-bound arithmetic, the
breadth-first structure schedule and the emission/pop budgets are byte-for-byte
as `w-6b2f04` left them.

### Deliverable 1 — the release binary, canonical target

```
$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
    ./target/release/madgab --approximate --top 200 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17405
MADGAB_TRACE raw_cutoff rank=199 score=0.906262887 phrase="it justice too uhh dame"
```

| | before (`85a8a60`) | after (`8582525`) |
|---|---|---|
| clue | **absent** | **absent** |
| pool | 17405 | 17405 |
| top-200 cutoff | 0.906262887 "it justice too uhh dame" | identical to the digit |
| wall | 2122 ms | 2170 ms |

**This front does not deliver the acceptance clue.** It removes the reason
the clue could not be delivered *by this mechanism at all*, and the
measurement below says that reason was not the operative one.

### Deliverable 2 — wall clock and pool on nine targets

| target | pool before | pool after | wall before | wall after |
|---|---|---|---|---|
| it's just a stupid game | 17405 | 17405 | 2122 ms | 2170 ms |
| congratulations on your promotion | 9230 | 9230 | 3206 ms | 3026 ms |
| they have really good ideas | 10759 | 10759 | 2352 ms | 2422 ms |
| a whole lot of trouble | 14978 | 14978 | 2057 ms | 1784 ms |
| the old man lives here | 19371 | 19371 | 1634 ms | 1604 ms |
| play games with me now | 19592 | **19720** | 1689 ms | 1689 ms |
| i love you | 10351 | **10381** | 1270 ms | 1039 ms |
| taco cat | 13533 | **13600** | 1193 ms | 1132 ms |
| big spender | 14115 | **14178** | 1774 ms | 1823 ms |

Every top-200 cutoff is byte-identical on all nine. Wall clock is unchanged
within run-to-run noise on all nine (worst case +2.3%, best case -13%; the
run-to-run spread of the base binary itself is of that size). **Reach was
bought by making the untraversed levels free, not by paying for them:** the
first stage is the old pre-filter, bit for bit.

### The refutation, and it is the substantive result of this item

**The pre-filter is almost never the binding constraint.** With a temporary
`ZZ_WIDEN`-gated counter in the widening branch (measured, then reverted; not
on the branch), the number of times the traversal actually asks for a wider
stage is:

| target | widenings |
|---|---|
| it's just a stupid game | **0** |
| congratulations on your promotion | **0** |
| they have really good ideas | **0** |
| a whole lot of trouble | **0** |
| the old man lives here | **0** |
| i love you | 1 |
| sign on | 1 |
| wow | 2 |
| taco cat | 3 |
| big spender | 3 |
| play games with me now | 6 |

On every multi-clause real target the widening **never fires at all**: the
traversal fills its 64-emission allowance, or hits
`LEXICAL_HEAP_POP_LIMIT = 4_000`, while the slots are still only ten wide.
So the uniform truncation was a *ceiling on reach in principle* and a
*non-binding cap in practice*, and the missing canonical clue is stopped
earlier, by the traversal's own order and bounds, not by the width.

That is the same conclusion `w-6b2f04` reached from the other side, and it
now has a mechanism attached rather than a number:

- `w-6b2f04` measured that full width costs 20.0 s and still misses, *under
  the current best-first order*. The reason is now visible in the code: with
  the myopic order, the 10-wide lattice is already large enough to exhaust
  the emission allowance, so the traversal **stops while the heap still has
  better nodes than the canonical one in it** and never asks for width.
- Removing the pre-filter cannot change that, and the measurement confirms it
  (identical pools and identical visible lists on five of the nine targets,
  and an unchanged visible list on all nine).
- What it changes is the *ceiling*: the canonical's `hid`, at walk rank 99, is
  now a node the search is able to push the moment the traversal reaches a
  level that can afford it. On the short targets where the traversal *is*
  hungry, that already happens and the pool grows by 30-137 wordings. **The
  deep *vocabulary* is not what is new** — `lovey` and `rearview` are in the
  narrow pool for `i love you` too, and `meow` is in the narrow pool for
  `play games with me now`; what the narrow search cannot produce is the
  particular *combination* (first pass of this item claimed otherwise, and
  the review in the next section corrected it with a measurement). No
  candidate was lost: the 2 / 9 / 2 phrases that leave the pools of
  `i love you`, `play games with me now` and `taco cat` leave them as a
  *different spelling of the same signature* at a different score, which is
  the deduplication in `finish` choosing a different representative.

**So the requirement on the order front is now sharp and measurable, and it is
the whole remaining blocker:** that order must make the traversal *arrive* at
the deep node within `LEXICAL_HEAP_POP_LIMIT`, because with my change the node
is pushable and with that order it is reachable. Its own measurement should
show a non-zero widening count on the real targets; today it is 0 on all five
of the multi-clause ones, and that number is the direct test of whether the
order front worked. **No bound was added here that the traversal does not
already apply**, so there is nothing for the coordinator to sequence *as a
bound*. See the next section for the caveat that does need sequencing.

### Validation, from this tree, after the rebase onto 85a8a60

```
cargo test --release --lib                        36 passed, 0 failed
cargo test --release --test corpus_integration     9 passed, 1 failed
  approximate_finds_classic_madgab_resegmentation  FAILED (this item's target)
  approximate_output_is_locked                     ok  (NOT re-baselined)
  approximate_pool_reaches_matches_deep_in_a_span  ok  (new)
cargo test --release --test exact_determinism      1 passed, 0 failed
cargo test --release --test approx_determinism     2 passed, 0 failed
```

`cargo fmt`, `cargo clippy` and doctests do not exist on this host; see
[../../environment-notes.md](../../environment-notes.md). They are not
claimed.

Constraint proofs, on this branch:

- `git diff 85a8a60 HEAD -- src tests` adds **0** lines matching
  `wreck|beach|recognize|justice|stupid|dupe|came|hid`.
- The same diff adds **0** lines mentioning `axes::` or any of
  `SIMILARITY`/`FAMILIARITY`/`RHYTHM`/`CLOSED_CLASS`/`NOVELTY`/`SHAPE`.
- No `zz_*`, sweep or instrumentation file on the branch; `git grep ZZ_WIDEN`
  returns nothing. The two `target-base/` and `target-staged/` build
  directories are untracked build output and were removed.
- **`approximate_output_is_locked` passes unmodified.** Reported loudly, as
  required: it passes because the change is output-neutral on every measured
  target's visible list, not because the change is small.

### Tests

- `next_branch_stage` is a free function with module-scope constants, so the
  property the item asks to be stated in a test is stated as one:
  `branch_stage_schedule_reads_every_alternative_in_log_stages` (the schedule
  ends at the widest slot for every width 1..=160, monotonically, in at most
  8 stages) and
  `branch_stage_schedule_never_offers_depth_the_lists_do_not_have` (it never
  proposes a width no slot has, for every `(cap, widest)` pair).
- `approximate_pool_reaches_matches_deep_in_a_span` was, in this pass, **not a
  guard**: it asserted on a final word the narrow search also reaches, so it
  passed on the parent commit, and its doc comment claimed a falsehood. It
  asserts on three wordings measured absent from `85a8a60`'s pool instead —
  see the next section.

### Next action

1. **Coordinator decision on 8582525.** It is correct, general, bounded,
   output-neutral on the visible list, and free on the real targets. It does
   not deliver the acceptance clue and was not expected to. Its value is that
   it removes a mechanism from the space and hands `w-8f3c61` a *test*:
   the widening count on the five multi-clause real targets, currently 0 on
   all five.
2. **Do not re-open the width front.** The measurement above closes it: on
   every multi-clause real target the pre-filter is not binding, so no
   truncation point, growth factor or stage count can deliver the clue on its
   own. This is a third refutation of a width-shaped mechanism, with a
   mechanism-level reason rather than only a cost number.
3. **The blocker is unchanged and is entirely the order front.** What is
   still needed is a traversal that reaches a specific deep node of one
   segmentation's product lattice within `LEXICAL_HEAP_POP_LIMIT = 4_000`
   pops, rather than one that stops at 64 emissions while the better nodes
   are still in the heap.

## Pass 2026-09-26T13:20Z (agent 9d4e17): review findings folded in; the
## branch is integrable but must not be integrated alone

Branch `madgab-slot-width` at **9adb46d**, pushed, on top of **14986b2**. Not
rebased since: the coordinator sequences the integration, and this pass was
told not to rebase with the restructure in flight.

### Finding 1 — the reach test did not discriminate. Fixed, and the false claim
### is retracted above

The test asserted on a *final* word (`lovey`, `meow`) that the narrow search
also reaches, so it passed on the parent commit; the doc comment's claim that
the clue "was absent while every slot was truncated to a fixed width" was
false. Measured on the two release binaries (`85a8a60` and this head, full
pools at `--top 20000`):

| target | phrases only the wide search proposes | final words only it proposes |
|---|---|---|
| i love you | 32 | **0** |
| play games with me now | 137 | **0** |
| taco cat | 69 | **1** (`shoot`) |
| big spender | 63 | **0** |

So the difference is in whole wordings and mostly in interior positions, not
in the vocabulary. The test now asserts three wordings, each measured absent
from `85a8a60`'s pool and present here, with the deep match named in the
position it occupies: `"ask lovey"` (first slot, `i love you`), `"break games
meow"` (second slot, behind `"plague"`, which both searches produce), and
`"taco shoot"` (a final word the narrow search never proposes for that target
in any position). **Verified discriminating:** with `85a8a60`'s `src/lib.rs`
and this test file, it fails with `i love you: "ask lovey" missing … pool had
10351 clues`; with this head it passes. `top_n` in the helper is 20 000, above
every pool these targets produce, because a word only a deep match supplies
can sit thousands of ranks down (`taco shoot` is at raw rank 4608) and a
selected prefix of 2 000 drops it.

### Finding 2 — the per-pop `Rc` is now paid only when a widening happens

The first implementation inserted `(k, prefix)` into a set on **every** pop of
**every** segmentation, in exchange for a widening that fires zero times on a
multi-clause real target. That is now inverted:

- the ordinary pass records nothing and allocates no `Rc` per pop;
- a widening is prepared by a **replay**: the heap and `seen` are re-seeded
  and the exhausted pass is re-run at the *same* width, writing down the
  nodes it expands and emitting nothing;
- only then is the wider width opened, and only the newly legal children of
  those nodes are pushed — so the wider stage pops only nodes the narrow
  stage never reached.

Two things this had to get right, both of which were wrong in the first
attempt and are now measured rather than assumed: the replay must run at the
*old* width (running it at the new one re-popped the whole narrow lattice and
tripped `LEXICAL_HEAP_POP_LIMIT`, which showed up as pools identical to the
base's), and the emission allowance must be counted per *segmentation* across
stages, not per stage. With both fixed, **the output is byte-identical to the
previous implementation on all ten measured targets** (compared as whole
`--top 20000` listings, not as pool sizes), so the refactor bought the
bookkeeping saving without costing a single candidate.

The price that remains, stated honestly: a widening costs one extra pass over
the narrow lattice. Measured fire counts, re-measured on this restructured
build (temporary `ZZ_STAGE` counter, reverted; `git grep ZZ_` is empty):

| target | widenings |
|---|---|
| it's just a stupid game | 0 |
| congratulations on your promotion | 0 |
| they have really good ideas | 0 |
| a whole lot of trouble | 0 |
| the old man lives here | 0 |
| **recognize speech** | **3** |
| i love you | 1 |
| sign on | 1 |
| wow | 2 |
| taco cat | 3 |
| big spender | 3 |
| play games with me now | 6 |

This is the same table as the first pass's except that `recognize speech` is
now measured and is **not** zero: the pre-filter is not binding on
multi-clause targets, but it does bind on some short ones, including one of
the acceptance examples. The refutation stands as stated — no truncation
point, growth factor or stage count delivers the canonical clue on its own —
but it should be read as "non-binding on every multi-clause real target",
not as "never fires".

Wall clock, this head against `85a8a60`, release binary, `--top 200`,
repeated three times on the two noisier targets: canonical 2338/3062/2332 vs
2343/2271/2404 ms; `they have really good ideas` 2908/3286/3008 vs
3422/2985/3178 ms. Overlapping, no regression. Pools and every top-200 cutoff
are unchanged on the five multi-clause targets, and `recognize speech` goes
14423 -> 14451 with an identical cutoff.

### Finding 3 — cosmetic

The doc comment on the `clue_boundaries` helper in
`tests/corpus_integration.rs` had been turned into `//////` by an earlier
edit of mine; restored to `///`.

### Sequencing: this branch must not be integrated alone

[w-8f3c61](w-8f3c61.md) is closed on `post-milestone-acceptance` as merge
`1ff73bb`, and its section 2 is the number that matters here: full slot width
costs **15.8 s and shrinks the pool** under the current best-first traversal,
and costs **0.7 s and grows the pool 23%** under a sum-cost order. Taken as
input, that means:

- the refutation above is unaffected — the pre-filter is still not the
  milestone blocker, and the blocker is still the order;
- but *this branch on its own* is the 15.8 s configuration's cheap
  approximation, not the 0.7 s one. It is affordable today only because the
  widening fires so rarely; that is a measured fact about the current order,
  not a property of the mechanism. **If the order changes, this branch's cost
  profile changes with it, in the direction the order makes affordable.**
  So it should be integrated together with the order, or after it — the
  coordinator's call, and the reason this pass did not rebase.

### Validation, from this tree

```
cargo test --release --lib                        36 passed, 0 failed
cargo test --release --test corpus_integration     9 passed, 1 failed
  approximate_finds_classic_madgab_resegmentation  FAILED (this item's target)
  approximate_output_is_locked                     ok  (NOT re-baselined)
  approximate_pool_reaches_matches_deep_in_a_span  ok  (and fails on 85a8a60)
cargo test --release --test exact_determinism      1 passed, 0 failed
cargo test --release --test approx_determinism     2 passed, 0 failed
```

`cargo fmt`, `cargo clippy` and doctests do not exist on this host; see
[../../environment-notes.md](../../environment-notes.md). They are not
claimed.

Constraint proofs on this branch: `git diff 85a8a60 -- src tests` adds **0**
lines matching `wreck|beach|recognize|justice|stupid|dupe|came|hid` and **0**
lines mentioning `axes::` or any scoring constant; no `zz_*`, sweep or
instrumentation file; the three test targets are ordinary English phrases
and nothing in `src/` knows them. The width schedule is still bounded by named
arithmetic — `LEXICAL_BRANCH_STAGE_0 = 10`, `LEXICAL_BRANCH_STAGE_GROWTH = 4`,
`SPAN_SHORTLIST = 160` — with two unit tests on that arithmetic, one of which
asserts the schedule ends at the widest slot for every width 1..=160.

### Next action

1. **Coordinator decision on 9adb46d.** It is a general improvement, not the
   milestone: it makes the enumeration's reach a property of the traversal
   instead of a constant, keeps the output lock, and costs nothing measurable.
   The refutation of the width front is recorded above and in the previous
   section and should not be revived.
2. **Do not integrate it ahead of the sum-cost order** without re-measuring
   the wall clock, for the reason in the sequencing note.
3. **Nothing further from this front.** The remaining blocker is the
   traversal's order and what the emission allowance is spent on; the latter
   is fenced away from this branch as w-e07c42.
