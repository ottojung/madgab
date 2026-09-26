# w-2f7a10 — agent 2f7a11, front report (traversal tuple space)

Branch `madgab-pairing-2f7a10` at `3b14482`, pushed. Base `bbfb94a` (rebased from
`fe4ad78`; the upstream commits are docs-only, so the rebase was trivial). `src/lib.rs`
is the only file changed. The four `examples/zz_*.rs` probes and this report are
untracked and off the branch. Nothing merged; `scratch/2f7a10-base` and
`madgab-pairing-2f7a10-rebase-backup` untouched.

## 1. Diagnosis (criterion 1)

The requested resegmentation of `It's just a stupid game` sits, in the canonical
structure `[3,10,13,15,19]`, at per-slot candidate-list ranks **(7, 0, 13, 99, 11)**
over slot widths **[160, 7, 160, 160, 93]** (measured on the default release path).
Two quantities in the default path reject it, both in `src/lib.rs`:

1. **`coverage_tuples`, the `tuple[slot] = at` assignment (parent line 403, now
   `profile_tuple`).** A class of slot subsets was swept at the stride of its
   *narrowest* slot and **one** index was written into every member.
   *Rejecting quantity: one index per class* — no tuple the reserve emitted could
   hold two different non-zero coordinates. Every emitted tuple was a **diagonal**
   of its class's index rectangle; a pairing is a *point* of that rectangle.
2. **The class walk, `for deep in 1..=max_deep` (parent line 381), with
   `EMIT_PROFILE_MAX_DEEP = 3`.** For that segmentation the reserve offered
   **15 tuples, every one at most 3 deep and every one a diagonal**; the shape
   needed is **1 tuple with 4 distinct non-zero coordinates**. The 4-deep class
   `{0,2,3,4}` was the 16th class tried against `EMIT_PROFILE_RESERVE = 16`, so it
   was funded **zero** times.

Tuples offered for that structure: **15** (all ≤3-deep). Tuples needed: **1** at a
shape the reserve could not express. Reserve slots available: **16**.

**Refutation of "`EMIT_PROFILE_MAX_DEEP` alone is the gate", with numbers:** raising
it 3 → 5 and changing nothing else gives pool 17,906 → 17,918 and canonical members
**47 → 47, unchanged** — the reserve is exhausted inside depth ≤ 3, so the class
*order*, not the cap, was the gate.

## 2. The change, and why it is general

* `profile_tuple` keeps the class's **first** slot on the shared-index sweep the old
  rule gave it (a function of the class's widths alone) and gives every **other**
  slot its own width and its own `COVERAGE_SLOT_ROTATION`-rotated phase. The first
  coordinate of every class is byte-identical to the rule it replaces, so the reserve
  is a superset of its old coverage one coordinate at a time; the other coordinates
  stop being copies, which turns a diagonal into a point.
* The class walk spends all one-deep classes first — keeping "a reserve smaller than
  the slot count still touches every slot" — then interleaves the deeper shapes
  round by round, so a reserve too small for every class pays for a spread of
  *depths*. On the segmentation above the 4-deep shape is now the 15th tuple of 16.
* `EMIT_PROFILE_MAX_DEEP` 3 → 4. Its old reason ("beyond two the classes outnumber the
  reserve") was a statement about the old *order* and expired with it; the cap is now
  what bounds the depth of a funded shape, which the new unit test asserts.

`build`'s additive `total_budget` comparison, `EMIT_PROFILE_RESERVE`,
`LEXICAL_COMBINATIONS_PER_SEGMENTATION`, the per-slot candidate lists and the
per-structure retention are untouched. No axis weight, no `select_diverse`, no
`adjacency.rs`, no `STRUCTURE_FLOOR`, no share cap. No added line contains a word,
substring or word sequence of either acceptance example; the no-hard-coding fence is
green.

## 3. The breadth cost — accounting, not assurance

Per target, release build, default approximate path, counters read inside that path
(`offered` = tuples `coverage_tuples` produced, `built` = passed `build`,
`cost-rejected` = refused by the additive `total_budget` comparison):

| target | offered P→B | built P→B | cost-rejected P→B | spent emissions P→B | segmentations P→B |
|---|---|---|---|---|---|
| It's just a stupid game | 3404→3469 | 3049→2598 | 355→871 | 15204→14977 | 256→256 |
| recognize speech | 3300→3356 | 3197→2769 | 103→587 | 13431→13166 | 256→256 |
| put it back on the shelf | 3834→3866 | 2900→2368 | 934→1498 | 12452→12171 | 256→256 |
| I love you | 751→771 | 723→682 | 28→89 | 4191→4167 | 63→63 |
| big spender | 3329→3381 | 2893→2454 | 436→927 | 10165→9779 | 237→237 |
| play games with me now | 3099→3170 | 2643→2098 | 456→1072 | 16384→16384 | 246→246 |

**Correction to my own earlier report:** the "256 → 512 segmentations" figure was
measured on a build *without* the continuity clause and does **not** hold on the final
code. On `3b14482` the segmentation count is identical to the parent's on all six
targets, and the visible top 50 is byte-identical on all six (50 clues, same structure
counts, same distinct-word counts). So the earlier explanation of the pool shrink via
segmentation count is **withdrawn**.

The mechanism that is left is the one the numbers above show directly: the reserve
offers slightly *more* tuples (+2%) and the same number of segmentations, but a much
larger share of them is **refused by `build` for exceeding `total_budget`** — 10.4% →
25.1% on the canonical target, 3.1% → 17.5% on `recognize speech`. A tuple deep in
four slots pays four non-zero substitution costs against the same additive bound, so
the shapes this front buys are exactly the ones that bound prunes. `spent_emissions`
falls 1.5–3.8% because nothing re-spends the freed allowance: the binding constraint
is `emit_allowance` per segmentation, not the global budget, and the schedule ends at
the same place.

Pool, on the canonical target: **17,907 → 17,648**; distinct words the pool draws on
**2,014 → 1,970**; structures **333 → 333**.

**Can the breadth be bought back from the reserve's own bound? Measured answer: no.**
The freed allowance is not budget-limited — `spent_emissions` is *below* the parent's
and the global budget is not exhausted on four of the six targets — so there is nothing
to convert. The only way to restore the parent's emission count inside the same
constants is to spend fewer reserve tuples on shapes `build` will refuse, which is
exactly the spend this front exists to make. The constant that would have to move is
the additive cost bound itself, `total_budget` (CLI default 1.5): the refused tuples
are 3- and 4-slot deep, whose substitution costs sum to roughly 1.5–2.0, so
`total_budget` would have to rise by roughly 35–65% to admit them all. That is a
change to what the search considers a *good clue*, not a change to the traversal's
tuple space, and it is outside this front. **Not recoverable within the same bound
without giving back the pairing shapes.** I am not raising `EMIT_PROFILE_RESERVE` or
any other constant to hide it.

**Which baseline witnesses regressed on my arm:** I did not run the sibling
verification front's witness table, so I can only speak for what I measured. The one
witness I can name is `tickets` on `the cat sat on the mat`, which my *first* cut
broke — that cut rotated every coordinate of a class, including the first, so it took
with it every alternative only the old index reached. The continuity clause restores
it: `approximate_pool_reaches_alternatives_past_the_opening_slot_width` is green on
`3b14482` (it was red on that cut), and no other witness is known to me to have
regressed. `tickets` on my final arm is reachable, which is a superset claim I can
make because the first coordinate of every class is byte-identical to the parent's.

## 4. Criterion 4 — the test is a real guard

`depth_profile_reserve_emits_pairings_not_only_diagonals`, on synthetic widths
`[160, 7, 160, 160, 93]` over 64 phases, naming no word, phrase or target. Ported
verbatim onto the parent's `coverage_tuples` and run there, it is **red at phase 0**,
on the assertion

> `tuples.iter().any(|t| deep_of(t).len() >= 3 && { let deep = deep_of(t);
> deep.iter().any(|&i| i != deep[0]) })`
> — *"every tuple is a diagonal, so a pairing of deep alternatives in different slots
> is not expressible; the reserve emitted `[[10,0,0,0,0], [0,0,20,0,0], [0,0,0,30,0],
> [0,0,0,0,28], [50,0,50,0,0], [60,0,0,60,0], [46,0,0,0,46], [0,0,80,80,0],
> [0,0,58,0,58], [0,0,0,64,64], [110,0,110,110,0], [76,0,76,0,76], [82,0,0,82,82],
> [0,0,88,88,88]]`"*

All 14 of the parent's tuples have every deep coordinate equal. A pool-level test in
`tests/` cannot discriminate instead: the visible top 50 is byte-identical on the
parent, and the public API does not expose the pre-selection enumeration that moves.

`recognize speech` → `wreck a nice beach` re-measured on the final code: **present**
(`wreck` 11 occurrences, `beach` 10 across 2 slot positions, `nice` 17 — all identical
to the parent's arm), and `approximate_finds_recognize_speech_resegmentation` is green.

## 5. Criterion 3 — not met, and why

Every pair of the requested wording's own words is still **0** in the deduplicated
default-path pool (`hits+dupe`, `hid+dupe`, `hits+hid`, `dupe+came`, `hid+came`,
`hits+came`), and the wording is still not enumerated. This is arithmetic, not a
matter of ordering: a sweep is an arithmetic progression, a reserve is at most
`LEXICAL_COMBINATIONS_PER_SEGMENTATION` = 64 draws, so its stride is at least
`span / 64` = 3, and the requested coordinates differ pairwise by 6, 92 and 4 — gcd 2.
No progression of stride ≥ 3 passes through 7, 13, 99 and 11, so no order of the
reserve reaches it. Reaching it needs a *product* sample rather than a progression,
which needs a class budget of at least `span / 2` = 75 draws — more than a
segmentation's whole allowance. The reserve shape is therefore no longer the binding
gate, and this front stops here.

## 6. What the pairing property bought, measured

Canonical structure `[3,10,13,15,19]`, default-path pool: members **47 → 47**, and the
deepest member goes from **3 slots off the structure's modal word to 4**, with members
at three-or-more off-modal going **4 → 5**. That is the general property this front
owns, and it is now reachable and measurable. The deeper per-word numbers moved the
other way (`dupe` 14 → 10, `hid` 5 → 3, `taught` 11 → 7) — the same breadth cost,
seen word by word.

## 7. Validation

| suite | result |
|---|---|
| `cargo test --release --lib` | **51 passed, 0 failed** |
| `--test corpus_integration` | **10 passed, 1 failed** — `approximate_finds_classic_madgab_resegmentation`, which **also fails on `fe4ad78`** and is this item's target, not a guard |
| `--test exact_determinism` | 1 passed, 0 failed |
| `--test approx_determinism` | 2 passed, 0 failed |
| `--test no_phrase_hard_coding` | 6 passed, 0 failed |

Both named guards are green: `approximate_finds_recognize_speech_resegmentation` and
`approximate_pool_reaches_alternatives_past_the_opening_slot_width`.

**Wall clock**, interleaved with `fe4ad78` in this session, six targets, two rounds
each: parent **12,471 ms / 12,133 ms**, this branch **11,187 ms / 10,052 ms**. Not a
regression. The old 1.77–2.51 s column is host noise and is not quoted.

`cargo fmt`, `cargo clippy`, doctests and `wasm32` builds **do not exist on this host
(`docs/environment-notes.md`); I did not run them and make no claim about them.**

## 8. Scope discipline

I stopped expanding this front once the reserve shape stopped being the binding gate,
per the coordinator's steer. The per-slot candidate-list / sweep-index / additive-cost
question, including why `dupe` and `hid` sit in disjoint slot classes, is not mine and
I have not touched `src/` on behalf of the front that owns it.
