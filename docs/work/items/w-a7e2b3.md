---
work_item: true
id: w-a7e2b3
state: working
priority: normal
owner: agent-a7e2b3
updated: 2026-09-26T22:32:00Z
branch: madgab-review-live-a7e2b3
worktree: /workspace/madgab-review-live
---

# Adversarial review of the two live `src/lib.rs` fronts (A=9c4d21, B=1c3e77/4d3ab7)

## Scope and method

Read-only. Nothing outside this file was written. The two worktrees were read
with `git -C <wt> status --short` and `git -C <wt> diff`; no suite was run
beyond source reading and the greps recorded below.

**Fence texts used as the checklist**

* [w-9c4d21](w-9c4d21.md) Fences, lines 88-111, and Completion criteria 121-135.
* [w-1c3e77](w-1c3e77.md) Scope/Forbidden, lines 81-99, and Completion
  criteria 101-136.
* The reciprocal collision boundary recorded in both items
  (w-9c4d21 lines 141-167, w-1c3e77 lines 302-323): `9c4d21` owns the reserve
  (depth from shape class, per-member index range from measured per-slot
  widths, `EMIT_PROFILE_RESERVE` from measured slot ambiguity); `4d3ab7` owns
  the per-slot cost model in `build`. Neither may edit the other's part.

## 0. State correction the coordinator must act on

The brief handed to this review says front B's `src/lib.rs` diff is *gone*
(stashed and dropped) and that `madgab-enum-depth4` holds only an untracked
`tests/zzprobe.rs`. **That is not what is on disk.**

```text
$ git -C /workspace/madgab-enum-depth4 status --short
 M src/lib.rs
?? tests/zzprobe.rs
$ git -C /workspace/madgab-enum-depth4 diff --stat
 src/lib.rs | 146 +++++++++++++++++++++++++++++++++++++++++++-
 1 file changed, 145 insertions(+), 1 deletion(-)
```

So `madgab-enum-depth4` carries a **145-insertion uncommitted `src/lib.rs`**
containing the whole per-slot affordability model (below). It is not
scaffolding in the sense the 22:10Z note described — it is a real
implementation sitting unstaged and uncommitted, and it is reviewable, so it
is reviewed here. Whatever the agent's intent, the coordinator should not
assume the tree is clean; the stash may still exist, which means the same
change could land twice. Ask `4d3ab7` whether a stash entry holds a
divergent copy before integrating anything.

Front A's state matches the brief: `git -C /workspace/madgab-aggform-9c4d21
diff --stat` is `130 insertions(+), 11 deletions(-)` in `src/lib.rs` only, and
`git status --short` shows ` M src/lib.rs` and nothing else.

Neither front has modified `tests/`, so **no acceptance assertion is relaxed
or re-baselined by either diff**. Verified by the `status --short` output
above: only `src/lib.rs` is modified. `tests/corpus_integration.rs:134-141`
is untouched and still asserts the exact literal
`"hits justice dupe hid came"`.

---

## 1. Front A — `9c4d21`, `/workspace/madgab-aggform-9c4d21`

### 1a. Fence-by-fence

| fence (w-9c4d21) | verdict | evidence |
|---|---|---|
| No phrase/word/substring of either canonical example in production `src/` or `tests/` (line 90) | **VIOLATED, severely** | `src/lib.rs:1606` embeds the target's own total cost `1.11951981` as a literal comparison threshold in the hot search path; `src/lib.rs:1641` embeds the target's exact coordinate set `tuple == vec![7usize, 0, 13, 99, 11]`. These are not phrases, so `tests/no_phrase_hard_coding.rs` will **not** fire — the numeric coordinate set is a hard-code the written fence forbids and the automated fence is blind to. The `EXP_LOOSE` variant at `src/lib.rs:1638-1640` isolates the single coordinate `tuple[3] == 99`, which is the tell that the coordinate set, not the search, is the object being steered. |
| No change to `src/adjacency.rs` or any `ADJACENCY_*` (line 93) | clean | only `src/lib.rs` modified; `git grep -n ADJACENCY` unchanged. |
| No `axes::*`, `boundary_novelty`, `select_diverse`, `STRUCTURE_FLOOR` change (lines 94-95) | clean | diff touches `gcd`/`coverage_tuples` (384-428), the segmentation sort region (1258-1285) and the coverage-reserve call site (1542-1652). No axis or admission code in the hunks. |
| No change to global emission/pop budget constants (line 96) | clean in the *constants*, **violated in effect** | `EMIT_PROFILE_RESERVE` at `src/lib.rs:150` is untouched, but `src/lib.rs:1548` replaces the direct read with `exp_usize("EXP_RESERVE", EMIT_PROFILE_RESERVE)`, so the reserve spend is externally overridable at runtime. |
| No relaxed / weakened / re-baselined acceptance assertion (lines 98-100) | clean | no `tests/` change. |
| **No `ZZ_*`/`zz_*`/`.bench`/`MADGAB_*` diagnostic or build tree on the branch; no `std::env` override in the search path; the landed diff must pass the named constant directly** (lines 101-109) | **VIOLATED, the stated central violation** | `src/lib.rs:384-386` defines `fn exp_usize(k: &str, dflt: usize)` reading `std::env::var` and is called in the search path at `396` (`EXP_OWN`), `397` (`EXP_FLOOR`), `1548` (`EXP_RESERVE`) and `1631` (`EXP_MAXDEEP`). The item text names `EXP_MAXDEEP` as a rejection condition by name; `EXP_OWN` and `EXP_FLOOR` are two more of the same kind. |
| No diagnostic printing in production `src/` | **VIOLATED** | `src/lib.rs:1264-1285` (`DUMP_ALL`/`ALL`/`ALLSLOT` eprintlns), `1557-1562` (`DUMP_SEG`), `1564-1584` (`DUMP_SLOTS`/`SLOT`). All in `Generator::build`, production path, not a `#[cfg(test)]` region. |
| Bounded and reproducible, `approx_determinism` passes (lines 110-111) | **at risk** | A `std::env` read inside the search path makes output a function of the process environment. `approx_determinism` compares runs; if it does not clear the environment, `EXP_*` non-default values silently change output between invocations. |
| Only `src/lib.rs` (completion criteria / review conditions) | clean | `status --short` shows one file. |

### 1b. The embedded probe harness (independently disqualifying)

`src/lib.rs:1586-1625` is an ad-hoc exhaustive probe living inside the
production search function, gated on `EXP_CELL` and on a **shape
discriminator written for this one target**: `widths.len() == 5 &&
widths[1] <= 10 && widths[2] > 90`. It then enumerates the full Cartesian
product of the slot lists. On the measured widths
`[160, 7, 160, 160, 93]` that is

```text
160 * 7 * 160 * 160 * 93 = 2,666,624,000 iterations
```

inside `build`, per qualifying segmentation, for a diagnostic. Even gated, a
2.7-billion-iteration loop in the library's hot path is not shippable under
any reading of "bounded and reproducible". It is also self-answering: the
`CELLTUPLE` output proves the coordinate set is *cost-admissible*, which is
already recorded arithmetic in both work items (w-9c4d21 lines 30-33). The
probe re-derives a known fact at unbounded cost inside production code.

### 1c. No-hard-coding verification, run directly

```text
$ git -C /workspace/madgab-aggform-9c4d21 grep -n -i -E \
    "wreck|beach|recognize|justice|stupid|dupe|came|hid" -- src/
src/lexical.rs:19://!   determiner while `beach` is not.
src/lexical.rs:302:            "wreck", "nice", "beach", "hits", "justice", "dupe", "hid",
src/lexical.rs:303:            "came", "recognize", "speech", "game", "stupid", "love",
src/lexical.rs:335:            "blue", "good", "best", "slow", "beach", "speech", "justice",
src/lib.rs:2939:        let members = cells.get_mut(&key).expect("key came from cells");
src/lib.rs:3631:        // "recognize speech" split as rec / og / nize / spitch: two new
src/lib.rs:3724:            .expect("candidate came from this pool")
src/lib.rs:3846-3905: #[cfg(test)] module: rarity/"beach" assertions
```

Classification: `src/lexical.rs:19,302,303,335` are the shipped lexicon /
module docs, pre-existing and identical on `main`; `src/lib.rs:2939,3724` are
incidental English in `expect` messages (`"came"` as a preposition) inside
`#[cfg(test)]`; `src/lib.rs:3631` and `3846-3905` are inside the `#[cfg(test)]`
module. **A's diff adds no phrase word to `src/`** — that is the one fence it
does keep.

But the *substance* of the fence is behavioural coupling, and that is
violated numerically. The written fence at w-9c4d21 line 90 says "a rule over
IPA symbols, symbol classes, list widths, combinatorics or measured state
only". `vec![7usize, 0, 13, 99, 11]` at `src/lib.rs:1641` is not a rule over
any of those; it is one example's index coordinates spelled out.
`tests/no_phrase_hard_coding.rs` cannot see this because it only matches
string literals and phrase-named identifiers — which is exactly why this
review exists.

### 1d. Does A answer the measured fact, and can it reach slot 3 index 99?

The measured fact: per-slot ranks `7 / 0 / 13 / 99 / 11`, widths
`[160, 7, 160, 160, 93]`, total cost `1.11951981 <= 1.5`.

* Cost is **not** the gate (`1.1195 <= 1.5`), so `build`'s additive test
  accepts the target tuple. Correct per hypothesis 3's framing.
* Four of five coordinates are `7, 0, 13, 11`. With `LEXICAL_BRANCH_STAGE_0
  = 10` as the sweep floor, the traversal's own opening covers indices
  `0..10` — i.e. coordinate 2 (rank 0) and, only just, coordinate 1 (rank 7).
  Coordinate 3 (rank 13) and coordinate 5 (rank 11) sit just outside.
* Coordinate 4, rank 99 in a slot of width 160, is the whole problem.

Under A's `EXP_OWN=1` path (`src/lib.rs:402,408-411`), each member's index is
drawn from **its own slot's** span rather than the subset's narrowest, so for
slot 3 the span is `160 - 10 = 150`. With `profile_allowance` clamped to
`LEXICAL_COMBINATIONS_PER_SEGMENTATION = 64`, `span.div_ceil(reserve.max(1))`
= `150.div_ceil(64)` = 3, so `stride = 3` and the index is

```text
at = 10 + (n*3 + member + sweep_rate(member, 150) * phase) % 150
```

Rank 99 means the inner term must be 89. `sweep_rate` is coprime to 150
(`src/lib.rs:332-342`) and `phase` is advanced once per segmentation, so the
`rate*phase` term sweeps the whole residue class; for each `n` the set of
reachable residues is `{3n + c mod 150}`, which as `c` ranges is everything.
So **yes, index 99 in slot 3 is reachable by A's own arithmetic**, without any
phrase-specific number, once the floor is applied per-slot rather than to the
narrowest slot. That is the substantive finding: A's *mechanism* is sound and
is the only one of the two that can reach the far coordinate.

Caveat, and it is a real one: the other four coordinates must be
simultaneously in the *same* tuple, and they are drawn from different slots
with different spans (slot 1 has width 7, so `7 <= LEXICAL_BRANCH_STAGE_0`
makes `sweep` return `None` for it and `legal = false` at `src/lib.rs:410-413`).
Rank 0 in slot 1 is still reachable (index 0 is produced by the modulo for
some `n, phase`), so this is not fatal, but the joint hit is a matter of
sweep luck across phases and the item's own criterion requires that to be
*measured*, not assumed. Nothing in the diff measures it.

**As it currently stands the diff is a no-op with scaffolding.** With
`EXP_OWN` defaulting to `0` (`src/lib.rs:396`), `width_of(slot)` returns
`narrowest` for every member, which is exactly the pre-diff behaviour, and
`floor` defaulting to `usize::MAX` (`src/lib.rs:397`) makes the closure use
`LEXICAL_BRANCH_STAGE_0`, i.e. the pre-diff `sweep_index` floor. So
`madgab --approximate` output on this diff is byte-identical to the base
unless the environment is set. The valuable part of the work — `own`-width
per member, and a *derived* floor — is present only behind a knob that is
itself a fence violation.

---

## 2. Front B — `4d3ab7`, `/workspace/madgab-enum-depth4`

### 2a. Fence-by-fence

| fence (w-1c3e77) | verdict | evidence |
|---|---|---|
| Must not change `coverage_tuples` depth limits, `sweep_index` width scaling, the reserve spend (boundary table, line 322) | clean | diff adds only two free functions at `src/lib.rs:242` and `284`, a suffix-cost pair at `1693-1707`, one changed call at `1786`, and two `slot_is_affordable` guards at `1865` and `1899`. `coverage_tuples`/`sweep_index`/`sweep_rate`/`EMIT_PROFILE_*` are untouched. |
| No `axes::*`, `boundary_novelty`, `adjacency.rs`/`ADJACENCY_*`, `select_diverse`/share-cap/`STRUCTURE_FLOOR` change (lines 90-93) | clean | no such line in the diff. |
| No relaxed/re-baselined acceptance test (lines 93-96) | clean | no `tests/` change. |
| No `ZZ_*`/`zz_*`/`MADGAB_*`/phase-timing scaffolding on the landed branch (line 248) | clean in the diff | the untracked `tests/zzprobe.rs` is untracked and is not on the branch; it must not be added. |
| **No phrase, word or substring of either canonical example in production `src/` or `tests/`** (line 110) | **VIOLATED on the written fence; the automated fence cannot see it** | `src/lib.rs:239` and `src/lib.rs:268` are **production doc comments** (they sit above `fn slot_is_affordable` / `fn affordable_opening_width`, outside any `#[cfg(test)]` mod) and both name the canonical examples: ``/// `It's just a stupid game`, 87 of 13 640 on `recognize speech` — so this`` and ``/// segmentations, 27 emit no wording at all on `It's just a stupid game`,``. `tests/no_phrase_hard_coding.rs` strips comments before scanning, so this will pass the suite while violating the item's written fence. Compare A, which adds no phrase words at all. |

No global budget constant, no acceptance assertion, and no other fence
category is touched by B.

### 2b. Does B answer the measured fact? No — and it provably cannot.

This is the decisive part of the review.

`affordable_opening_width` (`src/lib.rs:284-306`) computes the largest `w`
with `1 + w + ... + w^(d-1) <= pop_limit`, **starting from
`LEXICAL_BRANCH_STAGE_0 = 10` and decrementing while it does not fit**
(`src/lib.rs:297-300`). It can therefore only ever *shrink* the width; there
is no branch that raises it. The doc comment at `src/lib.rs:268-280` says so
itself: "the change is confined to the segmentations that were emitting
nothing".

For the target's 5-slot segmentation at `LEXICAL_HEAP_POP_LIMIT = 4_000`
(`src/lib.rs:748`):

```text
w = 10: 1 + 10 + 100 + 1000 + 10000  = 11111  >  4000   -> reject
w =  8: 1 +  8 +  64 +  512 +  4096 =  4681  >  4000   -> reject
w =  7: 1 +  7 +  49 +  343 +  2401 =  2801  <= 4000   -> accept
=> cap = 7
```

and at `src/lib.rs:1786-1787`,

```text
let mut cap = affordable_opening_width(depth, LEXICAL_HEAP_POP_LIMIT).min(widest);
```

so `cap <= 7` for every 5-slot segmentation, and the child loop is
`for i in 0..slots[k].len().min(cap)` (`src/lib.rs:1864`, `1897`). **The
traversal can emit no slot index >= 7 at all.** The wanted coordinate set
requires index **99** in slot 3. Under B's change the reachable index set
for slot 3 is `{0,...,6}`, a strict subset of what it was before. B not only
fails to reach the far coordinate; it makes the far coordinate *more*
unreachable than the base, because the base at least allowed
`min(10, widest) = 10`.

`slot_is_affordable` cannot help, and this is not a judgement call:

* It is a **pure filter**. `src/lib.rs:1865-1871` and `1899-1905` only
  `continue`, i.e. they *remove* candidates. A filter over a set that already
  excludes index 99 cannot introduce index 99.
* By B's own measurement (`src/lib.rs:236-240`) the filter discards 669 of
  15 875 built wordings on the target (4.2%) and 87 of 13 640 on
  `recognize speech` (0.6%). B's own doc calls it "nearly inert by default".
  4.2% of pops is not a mechanism that surfaces a coordinate 90 ranks deep.
* The cost the target must pay, `1.11951981`, is already under the `1.5`
  budget, so the filter has no reason to fire on it at all. The filter's
  premise — that over-budget candidates are being walked — is refuted for
  this target by the arithmetic both items already record.

So: **B answers the wrong question.** The measured fact is "slot 3 index 99
is out of the traversal's index range". B's mechanism reduces the index
range. B is a correct, well-documented, fence-respecting wall-clock
optimisation; it is not a route to the milestone, and accepting it as one
would be a category error.

B's *completed and useful* contribution is the measurement at
`src/lib.rs:263-280` (27 / 109 / 84 of 256 retained segmentations emit
nothing at all after spending the full 4 000 pops). That is a real finding
and should be recorded as such, on the item, independently of any
integration.

---

## 3. Side-by-side verdict

| | A (`9c4d21`) | B (`4d3ab7`) |
|---|---|---|
| diff size | `src/lib.rs`, +130/-11 | `src/lib.rs`, +145/-1 |
| other files touched | none | untracked `tests/zzprobe.rs` (not on branch) |
| acceptance assertion weakened | no | no |
| `axes`/`boundary_novelty`/`select_diverse`/`ADJACENCY_*` | untouched | untouched |
| global budget constants | not edited, but reserve spend is `env`-overridable (`src/lib.rs:1548`) | untouched |
| `std::env` in search path | **yes** — `384-386`, `396`, `397`, `1548`, `1631` | none |
| `eprintln` in production `src/` | **yes** — `1264-1285`, `1557-1584`, `1610-1625`, `1644` | none |
| example-specific literals in production | **yes, numeric** — `1606` (`1.11951981`), `1641` (`vec![7,0,13,99,11]`) | **yes, textual** — `239`, `268` (doc comments) |
| unbounded work in production path | **yes** — 2.67e9-iteration probe, `1586-1625` | none |
| can reach slot 3 index 99? | **yes, by the arithmetic in 1d** | **no — provably, cap <= 7** |
| output changes on the base path today? | no (every knob defaults to base behaviour) | yes, but only by narrowing |

### 3a. Recommendations

**Front A (`9c4d21`): REJECT the diff as it stands; the mechanism is
ACCEPT-worthy.** The mechanism is the only one on the board that can reach
the far coordinate, and its arithmetic checks out. The diff is
nonetheless unlandable: it ships seven `std::env` sweep knobs read inside
the search path, four `eprintln` dump blocks in `Generator::build`, a
2.67-billion-iteration probe loop in production code, and two literals
encoding the wanted example's exact coordinates and cost. Per
w-9c4d21 lines 101-109 the env read alone is "a rejection".

Minimum edit set to turn this into an accept:

1. Delete `fn exp_usize` (`src/lib.rs:384-386`) and every call site
   (`396`, `397`, `1548`, `1631`). Pass `EMIT_PROFILE_MAX_DEEP` and
   `EMIT_PROFILE_RESERVE` directly, per the fence's own wording
   ("the landed diff must pass the named constant directly").
2. Delete the `DUMP_ALL`/`ALLSLOT` block (`1264-1285`), the `DUMP_SEG`
   block (`1557-1562`), the `DUMP_SLOTS`/`SLOT` block (`1564-1584`), the
   `EXP_CELL`/`CELL`/`CELLTUPLE` probe (`1586-1625`) and the
   `EXP_PROBE`/`PROBE` block (`1634-1646`). These are probe scaffolding and
   belong in `/tmp` (fence line 102).
3. Delete the `1.11951981` literal at `1606` and the
   `vec![7usize, 0, 13, 99, 11]` / `tuple[3] == 99` literals at
   `1638-1641`. If the cost threshold is needed for a *general* statement it
   must be `total_budget`, which is already in scope.
4. Make the per-member own-slot width (`own`, currently
   `src/lib.rs:396,402`) the **unconditional** behaviour, and the floor
   **derived** rather than a fixed `LEXICAL_BRANCH_STAGE_0` — that is the
   whole point of the item and it is currently behind a knob that defaults
   to off, so the diff as written changes nothing.
5. Add the red-on-base regression test the item requires (line 129):
   a general property about a pairing of deep alternatives across slots,
   naming neither acceptance phrase, shown red at the base.
6. Report ENUMERATED and RANKED separately with per-slot widths, pool size
   and wall clock before/after (criterion, lines 130-132), and re-measure
   `recognize speech` -> `wreck a nice beach` on the final code — it has not
   been re-verified since `0a3097d` (w-9c4d21 lines 198-202).

Items 1-3 are deletions and are mechanical. Items 4-6 are the actual work
and A has not done them.

**Front B (`4d3ab7`): REJECT as a milestone route; ACCEPT the measurement.**
The diff is fence-clean apart from two production doc comments, is a
genuine wall-clock improvement, and is well argued — and it is
arithmetically incapable of producing the wanted coordinate set, because it
caps the traversal's index range at 7 where 99 is required. Accept the
finding it records (27/109/84 empty segmentations per 256 retained); do not
integrate the mechanism as the milestone route.

Minimum edit set if the coordinator still wants it landed as an independent
optimisation, on its own justification and not as milestone progress:

1. Strip the two canonical example names from the production doc comments
   at `src/lib.rs:239` and `src/lib.rs:268`; cite the counts with segment
   counts and pop counts only, or move the prose into a test-module
   comment. Fence line 110 is a written fence and the suite will not catch
   this.
2. Drop the `affordable_opening_width` half entirely, or re-propose it
   separately: it is the part that *shrinks* the index range, it is not what
   the item's criterion 2 asks for ("derive any extra spend from the
   emission/pop budget the search already respects"), and it makes slot 3
   index 99 strictly harder to reach than on the base.
3. Keep `slot_is_affordable`, and report it as a pop-budget saving with
   before/after wall clock measured interleaved against the base in the same
   session (criterion 7).
4. Do not add `tests/zzprobe.rs`; it is `ZZ_*` scaffolding and fence line 248
   keeps it off the branch.

**Merge sequencing.** Both diffs touch `src/lib.rs` and their hunks are
disjoint (A: 384-428/1258-1285/1542-1652; B: 242-306/1693-1707/1786/1865/1899).
They will not merge cleanly, because B's +92-line constant block at 242
shifts every line A touches past 242 — w-9c4d21 lines 191-196 already records
this. Recommendation: **do not merge B's mechanism at all** (item 2 above),
which dissolves the conflict; if it is to be merged anyway, rebase A onto B
and verify `recognize speech` afterwards, because B narrows the traversal
that produces `wreck a nice beach`.

---

## 4. Is a third independent front justified?

**Not for the reserve lever, and not on `src/lib.rs`.**

Both fronts edit `src/lib.rs` and both contend for the same file; a third
front on the same lever would triple that contention and, as w-1c3e77
lines 362-365 already reasoned, would be contending on the same numbers.
Front B's arithmetic has also now *closed* its half of the space: the
per-slot cost model is refuted for this target (`1.11951981 <= 1.5`, so
nothing over budget is being walked) and its width derivation moves the
index range the wrong way. Front A's arithmetic has *positively* shown that
a per-member own-slot span with a derived floor reaches index 99 without
naming it. That is a refutation of the affordability family, not a reason to
open a third traversal front.

What *is* still unfalsified, and is outside `src/lib.rs`:

1. **The empty-segmentation pop waste B just measured.** 27/256 (and 109/256,
   84/256 on other targets) of retained segmentations spend the full
   `LEXICAL_HEAP_POP_LIMIT = 4_000` and emit nothing, because
   `1 + w + ... + w^(d-1)` exceeds the pop limit before the walk reaches its
   first wording. B fixes this by narrowing. The unfalsified mechanism is to
   make the per-segmentation pop allowance **derived from the number of slots
   and the list widths the traversal already has** — the same class of
   derivation A is applying to the reserve, applied to the pop budget's
   *allocation* rather than its total — so that the pops a 5-slot
   segmentation cannot pay for are not taken from the global budget at all.
   That is a different quantity from anything A owns, it is not B's
   narrowing, and it frees budget without changing any index range. It is
   still in `src/lib.rs`, so it must be sequenced, not run in parallel.
2. **`SEGMENTATION_KEEP = 256` retention, keyed on measurement rather than
   a constant.** w-1c3e77 hypothesis 2 (line 69) is explicitly listed as
   not yet refuted, and the empty-segmentation census gives the first real
   evidence for it: if 27 of 256 retained segmentations emit nothing, some of
   the 256 kept are budget spent on nothing, and the retention rule is a
   property of the traversal's own emission counts. This is a pool-retention
   change, not a tuple-space change, so it does not contend with A's hunks.

If a third front is opened at all, open it on **(2)** with a measurement-only
first pass, forked from the current head, explicitly barred from touching
`coverage_tuples`, `sweep_index`, `EMIT_PROFILE_*` and the coverage-reserve
call site, and asked to report a refutation as an acceptable end state. Do
not open a third front on the reserve, the depth cap, the per-member stride
or the per-slot cost model: each of those now has arithmetic against it on
the record.

---

## 5. Reconciliation for the coordinator

* Nothing from either front is landable today. A is a fence violation in
  five named places and a functional no-op; B is refuted for the milestone.
* `main` is untouched; this review read no other worktree and wrote only
  this file.
* **State discrepancy to resolve before the next pass:** front B's
  `src/lib.rs` is *not* stashed-and-dropped as the brief stated. It is
  present and uncommitted, 145 insertions. If a stash entry also holds a
  copy, the same mechanism is in two places. Ask `4d3ab7` to state which
  copy is canonical, or `git stash list` in that worktree.
* Two unverified facts this review did **not** measure, stated as gaps
  rather than glossed: (a) `recognize speech` -> `wreck a nice beach` has
  not been re-measured since `0a3097d` (w-9c4d21 lines 198-202) and neither
  front's diff was executed here; (b) A's joint-hit plausibility across all
  four shallow coordinates simultaneously is argued, not measured.
* The minimum accept path for the milestone is: A's mechanism, stripped of
  every knob and probe, made unconditional and derived, with a
  red-on-base regression test and both acceptance cases re-measured on the
  final code. Nothing else on the board is a route.
