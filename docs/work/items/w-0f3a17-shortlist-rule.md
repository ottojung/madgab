---
work_item: false
id: w-0f3a17-shortlist-rule
state: done
priority: normal
owner: front agent-0f3a172 (measurement report; closed 2026-09-27, integrated at 534a39c)
updated: 2026-09-28T17:00:00Z
branch: scratch/0f3a17-shortlist
worktree: /workspace/madgab-sl-0f3a17-probe (probe branch scratch/0f3a17-shortlist-probe at b4a3009)
parent_item: w-0f3a17 (superseded)
---

# w-0f3a17 — the per-slot shortlist selection rule, as it actually behaves

> This is a **measurement report, not a queue entry** — it is why
> `work_item: false` and a terminal `state: done` are recorded here. It sat in
> `items/` with no YAML header at all, so every metadata-based census in the
> paused-programme reconciliation log counted 95 items in 96 files without
> noticing the difference, and the file was neither claimable nor reportable as
> missing. Header added by pass 93 purely to make the census self-consistent;
> nothing in the body changed, and nothing here is resumable.

Front: shortlist **contents** (the question the item declares separate from
the width question owned by the sibling front). Measurement only; no
`src/lib.rs` change is proposed or made here.

Branch: `scratch/0f3a17-shortlist` (this document).
Probe branch: `scratch/0f3a17-shortlist-probe` at `b4a3009`, worktree
`/workspace/madgab-sl-0f3a17-probe`, `CARGO_TARGET_DIR=/workspace/target-0f3a17-slprobe`.
The probe branch is **committed locally and deliberately not pushed**: it
carries `#[cfg(test)]` dump instrumentation, which must not reach any pushed
branch.

## 0. Pristine reference, reproduced first

`Generator::generate_pool`, release, `--top 50`, defaults, corpus
`CORPUS_JSON`, `CARGO_TARGET_DIR=/workspace/target-0f3a17-shortlist`:

| target | pool | rank-50 cutoff | expected |
|---|---|---|---|
| `It's just a stupid game` (case 2) | **18,936** | **0.915121574454** | 18,936 / 0.915121574454 |
| `recognize speech` (case 1) | **18,270** | **0.918796440893** | 18,270 / 0.918796440893 |

Both match [w-474813](w-474813.md) to the candidate, so the numbers below
are on a pristine base.

## 1. The question, and why "at any width" collapses to one measurement

The shortlist is built in one place, from the span's candidate set and the
target, and **nothing about the traversal's width enters it**. `selected` is
a function of `(matches, target_phrase)`; `cap`, `widest`, `next_branch_stage`,
`affordable_opening_width`, `LEXICAL_HEAP_POP_LIMIT` and the emission budgets
are not inputs to it — they only decide how deep into a list the enumeration
later reads. So

> a needed per-slot tuple is reachable at *some* width **iff** it is in
> `selected`

and "at what rank" is that tuple's index in the sorted shortlist. There is no
width to sweep. This is the single most important fact on this front, and it
is what makes the question answerable at all.

## 2. The rule, as specified

Per span edge `[p, end)` of the target IPA, with `matches` = every fuzzy
match at `p` whose `end` is `p + m.consumed`:

1. **Candidate set.** `matches`, in whatever order the matcher produced.
   Measured offer: 256 for most spans (the matcher's own cap), 7–256
   elsewhere; 2 for the `justice` span of case 2.
2. **`quality(m)`** — the single ranking function, descending, with
   `cmp_desc`:
   `-SIMILARITY_PER_WORD * cost + FAMILIARITY * familiarity
    - WORD_NOVELTY * reused + SHAPE * lexical_shape_quality
    + CLOSED_CLASS * closed_class_penalty(closed, 1.0)`.
3. **Six admission passes, in this order, each deduplicated by `word_idx`
   against every earlier pass** (`seen_words`, a word is inserted at most
   once, so a later pass never displaces an earlier one):
   * `by_cost`, cheapest **16**;
   * `by_familiarity`, most familiar **16**;
   * `by_rarity`, largest `rarity_rank` **4**;
   * four **cost bands**, `band = floor((cost / per_word_budget) * 4)`
     clamped to `0..=3`, and within *each* band three sub-passes of **4**
     each — by band quality, by band familiarity, by band rarity — so
     **4 x 4 x 3 = 48**;
   * `by_quality`, best **16**;
   * **fill**: walk the *full* `by_quality` order and take unseen words
     until `selected.len() >= SPAN_SHORTLIST = 160`.
4. **Truncation.** The only truncation is the `160` in the fill loop. It is
   a hard cut on `quality` rank, and it is the *last* word in the pipeline.
5. **Final order.** `selected.sort_by(cmp_desc(quality))`. So the index a
   tuple has inside the shortlist is its **quality rank among the retained
   160**, which is not its admission index: `cost`-pass words are admitted
   first and can still sort deep (`came` is admitted at 22 and sits at 7;
   `dupe` is admitted at 3 and sits at 64).
6. **Ceiling arithmetic.** The six passes can admit at most
   `16 + 16 + 4 + 48 + 16 = 100`, so on any span offering more than 100
   candidates **at least 60 of the 160 slots are filled by the
   undifferentiated quality fill** — i.e. at least 37.5% of every full
   shortlist is a plain top-up of one score, with no portfolio reasoning
   behind it.

Nothing in the rule is width-, depth-, cost- or target-aware except through
`per_word_budget` in the band edges.

## 3. The canonical alignment, slot by slot

`It's just a stupid game` (n=19), `hits justice dupe hid came`, cheapest
matcher alignment, stable across three runs. `IN@k` is the index in the
sorted 160-wide shortlist, `adm@j` the admission index, `q` the rank in the
span's *full* quality order (which is what the shortlist truncated), `pass`
which of the six passes kept it.

| slot | span | word | shortlist index | admission index | full-list quality rank | offered | pass |
|---|---|---|---|---|---|---|---|
| 0 | `[0,1)` | hits | **IN@22** | 8 | 22 | 256 | cost |
| 1 | `[1,2)` | justice | **IN@0** | 0 | 0 | 7 | cost |
| 2 | `[2,4)` | dupe | **IN@64** | 3 | 64 | 163 | cost |
| 3 | `[3,4)` | hid | **IN@85** | 109 | 85 | 256 | **fill** |
| 4 | `[4,5)` | came | **IN@7** | 22 | 7 | 93 | familiarity |

**All five canonical tuples are inside the shortlist at full 160 width.** None
is outside at any width. The other seven alignments in `reachability_corpus`
are likewise 33 of 33 words inside:

| target | alignment | per-slot shortlist indices |
|---|---|---|
| `It's just a stupid game` | `it justice two end game` | 143 / 0 / 157 / 3 / 89 |
| `It's just a stupid game` | `ich just a stoop a gave` | 81 / 61 / 158 / 12 / 159 / 4 |
| `recognize speech` | `wreck a nice beach` | 32 / 97 / 0 / 1 |
| `recognize speech` | `wreck a now spits` | 32 / 97 / 0 / 8 |
| `recognize speech` | `reckon i speaks` | 1 / 143 / 0 |
| `I love you` | `eye love you` | 13 / 149 / 159 |
| `I love you` | `alive views` | not matchable at `0.5`/1 over the whole target; irrelevant to the shortlist |

**Reconciliation with the item's `3 / 2 / 10 / 99 / 11`.** Those figures are
**not reproducible on this head** under either ordering I can construct:
sorted-quality indices are `22 / 0 / 64 / 85 / 7` and admission indices are
`8 / 0 / 3 / 109 / 22`. Neither vector is the item's, and no re-alignment of
the canonical wording changes the fact that every slot is *inside* the
shortlist. The item's *qualitative* claim survives in weakened form and its
stated numbers should be struck:

* `affordable_opening_width(5, 4000) = 7` is confirmed exactly
  (`1+7+49+343+2401 = 2801 <= 4000`; `w=8` gives `4681 > 4000`).
* Against width 7, **three of the five** canonical slots — 22, 64, 85 — are
  outside the production opening width, and slot 4 sits exactly on it. The
  item says three of five (10, 99, 11) are never pushed; that is right in
  count and wrong in the values, and 10 and 11 are not "never pushed" claims
  that survive: on this head the deep slots are 22, 64, 85.
* The item's inference "`LEXICAL_BRANCH_STAGE_0` is not the production
  opening width, therefore the deep indices are *outside the enumerated
  lattice*" **does not follow**. Outside the opening width means *late in the
  traversal*, which is the sibling front's question and is exactly what the
  widening ladder addresses. Outside the shortlist means *unreachable at any
  width*, which is this front's question and is not the case for any
  canonical tuple.

## 4. Per-target table: what the shortlist does drop

"Needed" is measured two ways, both mechanical and neither phrase-specific.

**(a) Real clues.** For each of twelve real targets, the *exact* search's own
top-200 pool is the source of needed tuples: every clue is re-aligned to the
target with the production matcher, and each clue word is looked up in the
shortlist of the span it lands on. Nothing is invented.

| target | real clues aligned | needed tuples checked | inside the shortlist | **outside** |
|---|---|---|---|---|
| `It's just a stupid game` | 3 | 16 | 16 | **0** |
| `recognize speech` | 2 | 4 | 4 | **0** |
| `I love you` | 12 | 36 | 36 | **0** |
| `taco cat` | 18 | 52 | 52 | **0** |
| `a whole lot of trouble` | 12 | 60 | 60 | **0** |
| `what are you going to do` | 12 | 72 | 72 | **0** |
| `some kind of wonderful thing` | 3 | 16 | 16 | **0** |
| `the cat sat on the mat` | 19 | 118 | 118 | **0** |
| `in the middle of the night` | 4 | 28 | 28 | **0** |
| `my brother has a red car` | 6 | 40 | 40 | **0** |
| `can you hear me now` | 14 | 70 | 70 | **0** |
| `we should have told her` | 16 | 80 | 80 | **0** |
| **total** | **121** | **730** | **730** | **0** |

**(b) Acoustic competitiveness.** A weaker, target-independent demand: every
candidate for a span whose cost is within **1.5x** of that span's cheapest
candidate — the same per-wording accumulator threshold the acceptance
reachability test uses — is a tuple the matcher considers genuinely good, so
if the shortlist drops one, a real wording of that quality is unavailable at
every width.

| target | span edges | cheap (<=1.5x min) | **cheap and dropped** | share | worst dropped quality rank |
|---|---|---|---|---|---|
| `It's just a stupid game` | 108 | 698 | 59 | 8.4% | 254 |
| `recognize speech` | 78 | 634 | 38 | 6.0% | 215 |
| `I love you` | 27 | 173 | 24 | 13.9% | 222 |
| `taco cat` | 34 | 221 | 38 | 17.2% | 214 |
| `a whole lot of trouble` | 77 | 628 | 95 | 15.1% | 227 |
| `what are you going to do` | 83 | 623 | 29 | 4.7% | 222 |
| `some kind of wonderful thing` | 130 | 903 | 72 | 8.0% | 245 |
| `the cat sat on the mat` | 80 | 728 | 71 | 9.8% | 247 |
| `in the middle of the night` | 91 | 707 | 83 | 11.7% | 245 |
| `my brother has a red car` | 95 | 556 | 33 | 5.9% | 227 |
| `can you hear me now` | 60 | 512 | 26 | 5.1% | 225 |
| `we should have told her` | 76 | 673 | 55 | 8.2% | 226 |
| **total** | **939** | **7,056** | **623** | **8.8%** | **255** |

The dropped words are real dictionary words, not junk — the recurring
families are the `a`- and `ɛ`-initial one-phoneme spans (`aar`, `aer`,
`ahl`, `arr`, `ar`, `ack`, `caw`, `cou`, `cur`, `eke`), which are exactly the
spans where the candidate set overflows 256 and the fill truncates. Worst
case measured: 24 of 56 cheap candidates dropped on the ɛ-initial spans
(`[8,9)` of `in the middle of the night`, `[4,5)` of `a whole lot of
trouble`, `[11,12)` of `we should have told her`) — 43% of the
acoustically-competitive words for those slots, gone at every width.

## 5. The answer to the question this front was opened to ask

**No needed per-slot tuple is missing from the shortlist, on any of the
twelve real targets, by either definition.** 730 of 730 real-clue tuples and
33 of 33 corpus-alignment tuples are present. The shortlist selection rule is
therefore **not** the defect that keeps `Hits Justice Dupe Hid Came`
unreachable, and the sibling front is **solving the right problem**: the
canonical slots at indices 22 / 64 / 85 are inside the enumerated lattice and
are blocked by the opening width, which is a budget-derivation question.

## 6. The latent defect, stated precisely, and the general fix (not implemented)

There is a real and provable weakness, which this measurement prices but does
not trigger.

**Defect.** The fill loop truncates on a single score. `quality` is
dominated by `FAMILIARITY` and `-SIMILARITY_PER_WORD * cost`, and both
degrade monotonically with phonetic distance, so the last 96 slots of a
full 160-wide shortlist are systematically the *worst-sounding* retained
words: the dropped set is not a random tail but a cost-monotone one. The
band structure that exists precisely to stop this — four cost bands, each
with its own quality/familiarity/rarity sub-passes — is applied only to the
first 100 slots, and the 60 slots the fill adds afterwards carry **no band
guarantee at all**: a cost band can be entirely absent from a full shortlist
if its words lose the global quality comparison. Consequently a span's
shortlist is not guaranteed to contain a single word within 1.5x of the
span's cheapest candidate in three of the four bands, and measured across
939 span edges it fails to do so for 623 competitive candidates (8.8%), up to
43% on the worst spans.

**General fix, not implemented.** Replace the single undifferentiated fill
with a *cost-stratified* fill: partition the fill's 60 slots across the four
cost bands in proportion to each band's population (at minimum a floor of a
few slots per non-empty band), and within a band fill by `quality`. This
keeps `SPAN_SHORTLIST = 160` — the constant that replaces a constant is not a
completion, so this changes *which* 160, not how many — and changes nothing
the rest of the search relies on:

* what it still bounds: the per-span alternative count is 160 exactly as
  before, so the structural DP's branching, the suffix relaxation and every
  `SpanExtremes` extremum are computed over the same number of candidates and
  stay admissible; the wall clock is untouched because no additional
  candidate is scored, only the order of an existing 96-word tail changes;
* the DP's `min_cost` / `max_familiarity` / `min_closed` / `min_syllables`
  can only get *tighter or equal* in the useful direction, because a
  band-stratified tail admits costlier-but-in-family words, so `min_cost`
  may rise — the span bound is a max over the shortlist, so a higher
  `min_cost` is a *stronger* (still valid) statement about what the DP may
  assume;
* it is checkable at an API boundary: "every non-empty cost band of a span
  contributes at least one word to the shortlist, and at least one retained
  word lies within 1.5x of the span's cheapest candidate in each band", over
  the twelve real targets, with no phrase in the assertion.

**Why this front does not implement it.** It is not what blocks the canonical
clue (5.0), it changes a production selection rule that two pool-reach guards
were re-pinned against, and the item's own handoff says not to contend on
`src/lib.rs` until the reviewer on w-6d2af3 has reported. It is filed here as
the specification the fix should be written against.

## 7. The number a later pass cannot cheaply recompute: which index, and which pass

Added at the request of coordination pass coord-0c1d. Probe
`scratch/0f3a17-shortlist-probe` at `b4a3009`, same worktree and target dir.

### 7.1 There are four per-slot orders, not one

This is the reason the item's `3 / 2 / 10 / 99 / 11` is unreproducible, and
it is the first thing a later pass must pin. Four distinct orderings of the
same span exist, and they disagree by up to 143 positions:

1. **admission order** — the order the six passes appended words, recorded
   as `insert_index` in §2.3. Never used by anything.
2. **shortlist order** — `selected.sort_by(cmp_desc(quality))`, the
   `edge.matches` order. This is the order the *truncation* and the
   structural DP see (§2.4–2.5).
3. **traversal order** — `slots[k]`, which is `edge.matches` **re-sorted by
   `SlotAlt::contribution(word_count)`**: familiarity, shape and closed-class
   terms divided by the slot count, `WORD_NOVELTY / word_count`, ties broken
   by the word string. **This is the order the lexical walk indexes into**:
   `for i in 0..slots[k].len().min(cap)`.
4. **per-pass orders** — the `by_cost` / `by_familiarity` / `by_rarity` /
   per-band orders, each of which is only ever read for its first 16, 4 or
   4 entries.

`cap` bounds order **(3)**, not order (2). A later pass that quotes "the
per-slot index" without naming the order is quoting an ill-defined number,
and the item's vector is consistent with neither (2) nor (3) nor admission.

### 7.2 Case 1 — `recognize speech` -> `wreck a nice beach`

Depth 4, so `cap = affordable_opening_width(4, 4000) = 10`
(`1+10+100+1000 = 1111 <= 4000`; the ladder is untouched above 10).

| slot | span | word | **traversal index (3)** | shortlist index (2) | admission (1) | admitting pass | rank under each of the six passes (cost / fam / rarity / quality) | pushed at cap 10 |
|---|---|---|---|---|---|---|---|---|
| 0 (depth 1) | `[0,?)` | wreck | **1** | 32 | 0 | cost | 0 / 96 / 96 / 32 | **yes** |
| 1 | | a | **0** | 97 | 4 | cost | 4 / 0 / 1 / 97 | yes |
| 2 | | nice | **0** | 0 | 1 | cost | 1 / 3 / 3 / 0 | yes |
| 3 | | beach | **2** | 1 | 8 | cost | 8 / 21 / 21 / 1 | yes |

**All four slots are pushed at the opening width, and the depth-1 tuple
`wreck` is pushed at traversal index 1.** No admission pass has to change;
the clue is already reachable, and `corpus_integration.rs:147` asserts it is
in the proposal list and is green. Case 1 needs nothing from this front.

### 7.3 Case 2 — `It's just a stupid game` -> `hits justice dupe hid came`

Depth 5, so `cap = affordable_opening_width(5, 4000) = 7`
(`1+7+49+343+2401 = 2801 <= 4000`; `w=8` gives `4681 > 4000`).

| slot | word | **traversal index (3)** | shortlist index (2) | admission (1) | admitting pass | rank under each of the six passes (cost / fam / rarity / quality) | **pushed at cap 7** |
|---|---|---|---|---|---|---|---|
| 0 (**depth 1**) | hits | **7** | 22 | 8 | cost | **7** / 72 / 72 / 22 | **NO — 7 ≮ 7** |
| 1 | justice | **0** | 0 | 0 | cost | 0 / 1 / 1 / 0 | yes |
| 2 | dupe | **13** | 64 | 3 | cost | 3 / 133 / 133 / 64 | no |
| 3 | hid | **99** | 85 | 109 | **fill** | 103 / 134 / 135 / 85 | no |
| 4 | came | **11** | 7 | 22 | familiarity | 34 / **7** / 7 / 7 | no |

**The priced negative the coordination pass asked for, with its arithmetic.**
At depth 1 the needed tuple is `hits`. Its rank under each of the six
admission passes' own orderings is 7 (cost), 72 (familiarity), 72 (rarity),
22 (quality) — the cost pass is the best of the six, so the **floor over all
six is 7**. The depth-1 cap is **7**, and the walk reads `0..min(len, cap)`,
so the test is `index < cap`, i.e. `7 < 7`, which is false. Therefore:

> **No change to any of the six admission passes can admit `hits` at depth 1
> within the production opening width. It misses by exactly one index.**

The same holds for the whole canonical clue: the floor over the six passes
is 7 / 0 / 3 / 85 / 7 against a cap of 7, so **not one of the five canonical
tuples can be brought inside the opening width by re-specifying the
admission rule**, and the two that come closest (`hits`, `came`) both have a
floor of exactly 7 = cap. What *would* admit them is a width, not a pass:

* `cap = 8` admits `hits` (traversal index 7);
* `cap = 12` also admits `came` (11);
* `cap = 14` also admits `dupe` (13);
* `cap = 100` admits `hid` (99) and is the first width at which the
  canonical wording is *fully* enumerable.

So the canonical clue needs **width 100**, against an opening width of 7
derived from `1 + 7 + 49 + 343 + 2401 = 2801 <= LEXICAL_HEAP_POP_LIMIT =
4000`. That is the sibling front's arithmetic, and this front's contribution
to it is the factor: **the width the clue needs is 100/7 ≈ 14x the width the
budget derives**, and the shortlist is not what stands in the way.

### 7.4 One general observation the sibling front should have

`hid` is admitted by the `fill` pass at admission index 109 and lands at
traversal index 99, and it is the only canonical tuple no *named* pass ranks
in the first 85. Measured across the eight corpus alignments, the
traversal order and the shortlist order disagree by up to 143 positions
(`it`: shortlist 143 -> traversal 0; `wreck`: 32 -> 1; `came`: 7 -> 11;
`hid`: 85 -> 99) and by 0 for others. Two consequences:

* a widening ladder that is justified in terms of "the shortlist index" is
  justifying itself in the wrong units — the number `cap` truncates is the
  traversal index, and it is **not** monotone in the shortlist index;
* because `contribution` divides the familiarity / shape / closed-class terms
  by the slot count, the traversal order itself depends on the segmentation's
  depth. A wider slot is not merely "further down the same list" — it is a
  differently-ordered list. Any width derivation must therefore be stated in
  traversal-index units, and re-derived per depth, not scaled from a
  shortlist-index measurement.

## 8. Verification

Pristine base `adf1672` on `scratch/0f3a17-shortlist`, `CARGO_TARGET_DIR=/workspace/target-0f3a17-shortlist`:

* `--lib`: 61 passed, 0 failed
* `--test corpus_integration`: 12 passed, **1 failed** —
  `approximate_finds_classic_madgab_resegmentation`, the item's own goal,
  pre-existing red at this base
* `--test emit_coverage`: 4 passed
* `--test approx_determinism`: 4 passed
* `--test exact_determinism`: 1 passed
* `--test no_phrase_hard_coding`: 9 passed

On the probe branch the same `--lib` suite is 65 passed / 0 failed, so the
dump instrumentation is inert. `cargo fmt`, `cargo clippy` and doctests
**cannot run on this host** and are not claimed.
