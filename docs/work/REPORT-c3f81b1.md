# REPORT-c3f81b1 — independent review of `1973f05` (front `madgab-reserve-c3f81a`)

**VERDICT: INTEGRATE.**

Reviewed range: `b04380d..1973f05` (source change `170fbb3..1973f05`, `src/lib.rs` + the work item's
own record). Reviewer branch `madgab-review-reserve`, worktree `/workspace/madgab-review-reserve`,
reviewed range left unchanged (no `src/` edit landed on it; the only mutation red-check was done on
`scratch/review-c3f81a` in `/workspace/probe-wt` with its own `CARGO_TARGET_DIR`).

---

## 1. Generality — the derivation is real, and it is not a re-parameterised 3

### The derivation, from `src/lib.rs` as it stands at `1973f05`

* `sweep_index` (`src/lib.rs:463-477`) returns `None` immediately when `width <= floor`, where
  `floor = LEXICAL_BRANCH_STAGE_0` (= 10, `src/lib.rs:141`). So for any subset whose **narrowest**
  slot is at or below the floor there is no index above the floor for the subset to take, and the
  function yields no coordinate at all.
* `coverage_tuples` (`src/lib.rs:591-664`) computes exactly that `narrowest` per subset
  (`src/lib.rs:614-618`) and calls `sweep_index(narrowest, …)` for the subset's members. When it
  returns `None` the `t` loop `break`s with `best` still `None`, and the final
  `if let Some((_, tuple)) = best { out.push(tuple) }` **does not push** (`src/lib.rs:660-662`).
  A subset that cannot be funded therefore produces nothing.
* The reserve is charged only by `out.len()`. The only two charges are
  `if out.len() >= reserve { return out; }` at the top of the subset loop (`src/lib.rs:606-608`) and
  the push itself, so an unfundable subset costs **zero** of `EMIT_PROFILE_RESERVE` — the free-skip
  claim is true, not merely plausible.
* At the call site (`src/lib.rs:1832-1866`) `profile_emitted` and `spent_emissions` are incremented
  only for a tuple that came back from `coverage_tuples` and survived `build`, so again nothing is
  charged for a skipped subset.
* Hence: a `k`-deep subset is fundable only if **every** one of its `k` slots is wider than the
  floor, and the deepest tier the reserve can place anything at is the number of slots wider than
  the floor. `funded_slot_depth` (`src/lib.rs:246-251`) is precisely that count. **The bound holds.**

### It is not `3` in disguise — measured, not argued

The one thing a re-parameterised cap would show is `funded_slot_depth` reproducing the old constant
on the segmentations that matter. It does not. I instrumented the call site on
`scratch/review-c3f81a` (`CARGO_TARGET_DIR=/workspace/probe-target`, debug `--lib`, one
`zz_review_probe_dump` test) and dumped `(slot_count, funded_slot_depth)` for **1,024
segmentations** over five real targets (`It's just a stupid game`, `recognize speech`,
`can you hear me now`, `I love you`, `in the middle of the night`):

```
(2,0):1  (2,1):6  (2,2):4  (3,1):6  (3,2):39 (3,3):31
(4,2):18 (4,3):57 (4,4):146
(5,3):16 (5,4):79 (5,5):270
(6,4):2  (6,5):53 (6,6):167
(7,7):125 (8,8):4
min funded: Some(0)   where funded < slots: 277/1024
```

The bound takes **every value 0..8**, the floor binds in 277 of 1,024 segmentations, and on the
targets the front is about the cap is 3 nowhere in particular: 4-slot segmentations get 4
(146 times), 5-slot get up to 5, 6-slot get up to 6. A re-parameterised 3 would show `(n,3)` for
every `n >= 3`. It does not.

### Floor unreachable — safe, and it really happens

`funded_slot_depth == 0` occurs (the `(2,0)` row above). Path: `max_deep = 0`, the loop
`for deep in 1..=0` never runs, `coverage_tuples` returns an empty vector, `profile_emitted` stays
0, `emit_allowance = emit_allowance.saturating_sub(0)` is unchanged, and the traversal receives its
full `LEXICAL_COMBINATIONS_PER_SEGMENTATION` share. No indexing, no panic, no reserve consumed, no
emission budget spent. This is also why `approximate_output_is_locked` and the three
`approximate_pool_reaches_*` guards do not regress: the whole-search emission total is **identical
at both heads** for every one of the sixteen priced targets, and the pool moves by +0…+19.
Measured directly (see §4): the rank-50 cut-off is byte-identical at base and head
(`0.913250211242` for `recognize speech`, `0.912970544846` for `It's just a stupid game`), i.e. the
change re-orders which subsets are funded inside the fixed envelope and adds a handful of
candidates without displacing a printed proposal.

### No phrase, clue, word list, target literal or threshold fitted to a canonical case

`git grep -i -n -E "wreck|beach|recognize|justice|stupid|dupe|came|hid|net" 1973f05 -- src` gives 64
lines. Diffed line-for-line against the same grep at `170fbb3`: **the only difference is line
numbers**. No new hit. The added hits would be new lines, and there are none. All existing hits are
in pre-existing `#[cfg(test)]` fixtures, the `src/lexical.rs` test word lists, the `src/main.rs` CLI
usage examples, and ordinary English in comments ("phonetically", "candidate came from cells", the
`beach` rarity fixture at `src/lib.rs:4260-4272`).

The two new tests contain **no** canonical phrase and no threshold fitted to one: the fixture is
twelve ordinary sentences (`"a whole lot of trouble"`, `"the cat sat on the mat"`, …), and the
expected value is a function of the targets' own word counts. `tests/no_phrase_hard_coding.rs`
(9/9) enforces the same at the boundary.

---

## 2. Tests — both new tests are discriminating, and neither is self-referential

**`the_reserve_depth_bound_is_the_slots_the_traversal_leaves_room_in`** (`src/lib.rs:5217-5255`).
Over 1..=12 slots with 0..=4 of them pinned at the floor, it recomputes the expectation **inline
from the widths** and never calls `funded_slot_depth`. Red-check, on `scratch/review-c3f81a`
(`CARGO_TARGET_DIR=/workspace/probe-target`), reintroducing `.min(3)` in the production function:

```
4 slots, the first 0 at the floor: the bound is the count of slots the
traversal leaves room in, so it cannot be a constant
  left: 3   right: 4
```

**`a_target_whose_best_wording_is_deep_in_one_slot_gets_a_deep_tuple`** (`src/lib.rs:5257-5324`).
Same mutation, same target dir:

```
"a whole lot of trouble" (5 words): the reserve placed a tuple deep in 3
slots, and a target of 5 words admits a tuple deep in 4.
```

Both red independently reproduced on this host; the branch's own red-check report matches.

**Not self-referential (the `w-6d2af3` / `w-3c9d17` trap).** The property test reads **no** shared
axis with the rule: not `funded_slot_depth`, not the slot widths, not `LEXICAL_BRANCH_STAGE_0`, not
`SPAN_SHORTLIST`, and not the emission budget. Its expectation is `min word count - 1` over its own
fixture, a quantity the production bound cannot reach by construction; and its measurement is a
test-only counter of non-zero coordinates, a quantity the old cap could not exceed. The rule can
only move the number by actually placing a deeper tuple, and the test can only move the expectation
by being handed a different sentence. Verified red above. This is the correct shape and is the
inverse of the defect the repo paid for.

**Hygiene.** `git diff --name-only 170fbb3 1973f05` = `docs/work/items/w-c3f81a.md`, `src/lib.rs`.
No `tests/` change at all, so nothing could have been re-pinned. `git grep -i -E "ZZ_|zz_"` over
`src/` and `tests/` at `1973f05`: **no hits** — no probe instrumentation, no `zz_probe`, no
survey-dump test. `examples/` contains only the pre-existing `measure.rs`, which is not in the diff;
no probe binary landed. The one new `#[cfg(test)]` counter, `DEEPEST_PROFILE_COORDINATES`, is a
*permanent* part of the depth instrumentation (sitting beside `DEEPEST_PROFILE`,
`DEEPEST_TRAVERSAL`, `DEEPEST_ADJACENCY`, all permanent) with a doc comment, and it is read by a
committed test — that is instrumentation the tests depend on, not a temporary probe.

---

## 3. The criterion-5 fence, run here, single-threaded, at both heads

`CARGO_TARGET_DIR=/workspace/rev-target` for the head (`/workspace/madgab-review-reserve` at
`1973f05`) and `CARGO_TARGET_DIR=/workspace/base-target` for the base (worktree `/workspace/base-wt`
at `170fbb3`). Separate directories, no sharing — this is the trap the front documented, and I hit
its other face first: `/tmp/opencode/...` is `noexec` here, which made `cargo` fail to exec
`proc-macro2`'s build script. All runs `-j1`; corpus_integration filtered runs are also
`--test-threads=1`.

| command / test | base `170fbb3` | head `1973f05` |
|---|---|---|
| `cargo test --lib` | **61 passed, 0 failed** (58.5 s) | **63 passed, 0 failed** (160.9 s) |
| `cargo test --test emit_coverage` (all 4) | **4 passed, 0 failed** (24.1 s) | **4 passed, 0 failed** (24.9 s) |
| `approximate_pool_reaches_matches_deep_in_a_span` | pass | pass |
| `approximate_pool_reaches_alternatives_past_the_opening_slot_width` | pass | pass |
| `approximate_pool_reaches_resegmentations_deeper_than_one_walk` | pass | pass |
| `approximate_output_is_locked` (not re-baselined) | pass | pass |
| `cargo test --test no_phrase_hard_coding` | 9 passed | 9 passed |
| `cargo test --test approx_determinism` | 4 passed | 4 passed |
| `cargo test --test exact_determinism` | 1 passed | 1 passed |
| `approximate_finds_classic_madgab_resegmentation` | **FAIL** | **FAIL** |

**Criterion 5 (pool reach): green at both heads, not re-pinned.** All three
`approximate_pool_reaches_*` pass on `1973f05`, and `tests/` is untouched by the diff, so
`approximate_output_is_locked` is the same assertion as at base — and it passes on both.

**The one failure is pre-existing and I did not touch it.** `approximate_finds_classic_madgab_resegmentation`
fails **identically at base and head**, with the identical message and the identical 12-clue list:

```
canonical clue missing from top 50; got: ["it said thus test oop day", "it said thus 'cause too day",
"it said thus death too day", "it said thus tough too day", "it said thus test oop gave",
"each thus 'cause too bad aim", "it said thus 'cause too gave", "it said thus test oop dame",
"it said thus 'cause too dame", "it said thus tess too day", "each thus 'cause too bad same",
"it justice too bad same"]
```

Not attributed to this front, not re-pinned, not opened as new work here. **Reported only.**

**One host limitation to record honestly:** a whole-binary `cargo test --test corpus_integration`
run (all 13) is **SIGKILLed (signal 9) on this host at both heads** — memory, not a test failure.
The per-test and per-filter results above are therefore the complete criterion-5 picture I could
obtain; every test in that file was exercised individually and none regressed.

---

## 4. Reproduction of the front's central numbers

Release build, defaults, public `Generator::generate_with_pool` / `generate_pool` / `generate`,
top 50, `CARGO_TARGET_DIR=/workspace/probe-target` (head) and `/workspace/base-target` (base),
probe `examples/zz_review_probe.rs` living only in the throwaway worktrees.

| measurement | base `170fbb3` | head `1973f05` | front's claim |
|---|---|---|---|
| `recognize speech` pool | **18,270** | **18,289** | 18,270 → 18,289 (+19) ✔ both |
| `It's just a stupid game` pool | **18,936** | **18,949** | 18,936 → 18,949 (+13) ✔ both |
| `recognize speech` rank-50 cut-off | 0.913250211242 | 0.913250211242 | 0.918796440893 ✘ |
| `It's just a stupid game` rank-50 cut-off | 0.912970544846 | 0.912970544846 | 0.915121574454 ✘ |
| `wreck a nice beach` in `recognize speech` pool | rank **26** | rank **26** | "still produced" ✔ |

* **The pool sizes reproduce to the candidate, on both heads, and match the front's `@3`/`@4`
  columns exactly.** The +19 / +13 deltas are the front's, independently measured. This is the
  strongest single piece of evidence for the change and it corroborates the pricing table.
* **The rank-50 cut-off literals do not reproduce on this host**: 0.913250211242 / 0.912970544846,
  not 0.918796440893 / 0.915121574454. The cut-off is **identical at base and head**, so this is a
  host difference in the printed-list selection, not an effect of the change, and the pool
  literals — the ones the fence depends on — do reproduce. Recorded as the item's traps require.
* **Canonical case 1 still produces `wreck a nice beach`** in approximate mode: present in the
  production pool for `recognize speech` at **rank 26**, at both heads. (It is not in the printed
  top 50 at either head; the pool is the production pool by `generate_pool`'s own contract, and the
  item is asking for production, not for the printed list.)
* **Canonical case 2's alignment is still absent** from the `It's just a stupid game` pool at both
  heads (`hits justice dupe hid came` → `None`), while each of its five words is individually
  present. The front's honest negative is confirmed.

---

## 5. Criterion 4 — no residual hard-coding

Covered in §1. New-line-for-line diff of the banned-token grep is empty; `tests/no_phrase_hard_coding.rs`
9/9 at both heads.

---

## 6. Not verified here (not claimed as satisfied)

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and doctests **cannot run on this
host** (no `rustfmt`/`clippy`/`rustdoc`; see `docs/environment-notes.md`). I did not run them and do
not report those criteria as met. `cargo fmt` is the one real residual risk of a ~50-line src diff
whose new function is four lines — an integration pass should run it where a toolchain exists.

---

## 7. Observations for the coordinator (none blocking)

1. **`--lib` wall time roughly triples**, 58.5 s → 160.9 s, essentially all of it the new
   12-target property test (~10 s/target). Legitimate, but it is a real CI cost and the fixture
   could be halved without weakening the claim. Worth a coordinator note, not a change I made.
2. **The property test's name overclaims slightly.** It asserts that the reserve *places* a tuple
   with `>= W-1` non-zero coordinates for a `W`-word target; it does not establish that the
   target's *best available wording* is deep in one slot, which is what the name says. The
   assertion itself is sound, general and discriminating; the name is the imprecise part.
3. **Two existing tests now derive their ladder width from `funded_slot_depth(&[SPAN_SHORTLIST; 8])`**,
   which evaluates to 8 only because `SPAN_SHORTLIST` (160) exceeds the floor (10). The comment
   claims "no segmentation in the corpus is wider than this many slots" — measured max is **exactly
   8** (four 8-slot segmentations in 1,024), so the comment is currently true with zero margin. A
   9-slot segmentation would silently under-test the rate ladder. Cosmetic, test-only, and the
   `sweep_rate` coprimality claim is only ever asked for `member < 8`.
4. **Not a defect, but state it for the record:** the new bound removes a backstop, not a budget.
   The reserve is still bounded by `out.len() >= reserve` inside `coverage_tuples`, so
   breadth-before-depth truncation is what actually limits a run; the old `3` was never load-bearing
   for affordability (F2's zero-emission result confirms this on sixteen targets).

---

## 8. The durable residual finding, stated precisely enough to become the next item

**The blocker for the canonical case-2 alignment is not tuple depth. It is that the coverage reserve
is uniform in each slot's *marginal* index and has essentially zero *joint* coordinate coverage.**

Reproduced on this head (instrumented dump, `It's just a stupid game`, 240 segmentations,
`CARGO_TARGET_DIR=/workspace/probe-target`):

```
reserve emissions by slot count:            {4: 165, 5: 1104, 6: 1660}
four-plus-deep reserve tuples by slot count: {5: 34}
```

**All 34 four-deep tuples on the canonical target come from five-slot segmentations** — exactly the
slot count the canonical alignment (`hits justice dupe hid came`, indices 7/0/22/99/11) needs. The
depth cap was never the cause, and this number is now confirmed rather than asserted.

* **Which consumer:** `coverage_tuples` (`src/lib.rs:591-664`), the coverage reserve — the component
  that decides *which* per-slot candidate combinations enter the enumerated lattice. It is not
  `affordable_opening_width` (per-slot width, `w-9e2b41`) and not `SPAN_SHORTLIST` retention
  (`w-7b40d2`).
* **Which budget:** the per-subset sample width `EMIT_PROFILE_SAMPLE = 8` (`src/lib.rs:373`), drawn
  *around one strided position* per subset per phase by `sweep_index`. A joint coordinate set like
  {7, 22, 99, 11} is one point in a ~150⁴ product; a sample that is uniform per marginal has joint
  coverage of essentially zero. The reserve's own slice, `EMIT_PROFILE_RESERVE = 16` of
  `LEXICAL_COMBINATIONS_PER_SEGMENTATION = 64`, is *not* the binding constraint — F2 shows depth-4
  placement costs zero emissions — and neither is `LEXICAL_GLOBAL_EMISSION_BUDGET` (16,384, already
  saturated and unchanged).
* **What a general rule would have to key on:** the **joint distribution of a subset's member
  coordinates within a phase**, not the depth of the subset and not any per-slot width. Concretely,
  a rule that varies the *phase* or the *rate* per member within a subset — rather than rotating
  every member of a subset by one shared rate and then sampling a fixed 8 positions around a single
  strided draw — would buy joint coverage at the same marginal coverage and the same emission cost.
  It must be stated and priced with the same discipline: the trigger is a measurable per-subset
  property (distinct coordinate vectors per subset per phase, versus the current one), and the
  acceptance property is *presence* of specific joint alignments in the production pool, not a
  change in any depth or marginal statistic. Note the front correctly declined to raise
  `EMIT_PROFILE_SAMPLE`, which is a shortlist-fill surface (`w-7b40d2`) — so a next front must be
  explicitly chartered on the sampling *rule* rather than the sample *width*, or it will duplicate
  `w-7b40d2`.

**Integration recommendation: take `1973f05`.** Do not read it as clearing the milestone blocker for
the canonical case-2 alignment — it explicitly does not, and `1973f05`'s own residual finding is the
more useful output. If the coordinator prefers to keep the branch's own pricing and record intact,
`b04380d` is self-contained and reviews on its own.
