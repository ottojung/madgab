# w-5c8e1f — front `9a41d3` (M7a: the relative similarity axis)

Branch `madgab-score-reseg-9a41d3`, created at `27f9776` (`post-milestone-acceptance`).
Base for every number below: `27f9776`, **release**, default approximate path
(`./target/release/madgab --approximate --top 50`), plus the base's own
`MADGAB_TRACE_PHRASES` gate. No test was edited, relaxed, re-baselined, `#[ignore]`d
or deleted. No axis weight was moved. `main` and `post-milestone-acceptance` were
not touched. Nothing was merged.

**Headline, and it is a negative result:** M7a as specified in the brief is
implemented exactly, and it does what c81e55 §8 predicted it would do for the
canonical target — but it **breaks the green guard**. `wreck a nice beach` on
`recognize speech` goes from raw rank 27 with a **+0.001267657** margin to raw
rank 278 with a **−0.006909329** margin, and
`approximate_finds_recognize_speech_resegmentation` turns red. It also breaks
`approximate_output_is_locked`, which was expected (a scoring change moves every
printed score; c81e55 §8 said this test "will need a deliberate, recorded
decision"). Two green tests become red, so the recommendation is **HOLD**. The
change is not tuned toward green anywhere, and no test was re-baselined to hide
the result.

**A note on the brief.** `docs/work/items/w-5c8e1f.md` does not exist in this
worktree, in `27f9776`, or on any branch in this repository (`git log --all
--diff-filter=A -- docs/work/items/w-5c8e1f.md` is empty; no file in the tree
contains the string `5c8e1f`). Everything specified for M7a was recovered from
the cited evidence instead: `git show madgab-depthcap-c81e55:REPORT-c81e55.md`
§§1, 3, 5, 6 and 8, which carry the formula, the ≥3-letter shared-prefix rule,
the proposed regression test verbatim, the predicted numbers, and the guard
margins. Where this report needed something the brief alone would have supplied
— chiefly the per-target red/green census of the new guard — it was measured
here rather than assumed.

---

## 1. Reproduction before the change

`27f9776`, release, `--approximate --top 50`.

```
$ ./target/release/madgab --approximate --top 50 "It's just a stupid game"
  1. [0.925] it justice too bad aim
  2. [0.924] it justice too pad aim
  ...                                              # 50 lines, requested wording absent
$ ./target/release/madgab --approximate --top 50 "recognize speech"
  1. [0.922] yeah 'cause i.'s pitch
  ...
 28. [0.918] wreck a nice beach
```

Production trace gate, on the same binary:

```
$ MADGAB_TRACE_PHRASES="hits justice dupe hid came" madgab --approximate --top 50 "It's just a stupid game"
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17906
MADGAB_TRACE raw_cutoff rank=49 score=0.915691888 phrase="it justice too day mm"

$ MADGAB_TRACE_PHRASES="wreck a nice beach" madgab --approximate --top 50 "recognize speech"
MADGAB_TRACE raw phrase="wreck a nice beach" rank=27 score=0.918313383
MADGAB_TRACE raw_cutoff rank=49 score=0.917045726 phrase="red 'cause i.'s peach"
```

This reproduces c81e55 §1 to the digit: raw rank 27, score **0.918313383**,
cutoff **0.917045726**, margin **+0.001267657**; the requested wording absent
from 17 906 candidates (`w-6b91d3`'s known ±1–2 pool non-reproducibility; c81e55
recorded 17 907).

Visible full resegmentations in the visible 50, over 13 targets, counted with the
rule from c81e55 §6 — two words are the same word when one is a prefix of the
other with **≥3 letters on both sides**, so `it's`/`it` counts as shared, and a
proposal is a *full resegmentation* when it shares no such word with the target:

| target | full reseg / 50 | target words retained (count) |
|---|---|---|
| **It's just a stupid game** | **0 / 50** | justice 50, games 1 |
| the cat sat on the mat | 29 / 50 | matt 5, catch 8, match 6, then 3 |
| when the rain finally stopped | 31 / 50 | final 19, whence 1 |
| an old man in a big hat | 35 / 50 | many 15 |
| recognize speech | 44 / 50 | rec 6 |
| what are you going to do | 49 / 50 | wha 1 |
| I love you | 50 / 50 | — |
| a whole lot of trouble | 50 / 50 | — |
| he was a big fat man | 50 / 50 | — |
| my brother has a red car | 50 / 50 | — |
| put it back on the shelf | 50 / 50 | — |
| she had a lot of money | 50 / 50 | — |
| there is no way to know | 50 / 50 | — |

Identical to c81e55 §6, which is the independent confirmation that the harness
and the base agree.

---

## 2. What M7a is here, exactly

One axis, one formula, no weight changed:

```
similarity = 1 - (total_cost - segmentation_floor) / 4,  clamped to [0, 1]
segmentation_floor = sum over slots of (min substitution cost in that slot)
                    = suf_min_cost[0], the all-argmin tuple's additive cost
```

`src/lib.rs` diff (`27f9776..HEAD`, 75 lines changed, `+66`/`-9`):

* `Partial` gains one field, `cost_floor: f64`, and `metrics` uses
  `1 - (self.sub_cost_total - self.cost_floor) / 4` (`src/lib.rs:2341`). It is
  0.0 wherever there is no segmentation behind the candidate, which is the exact
  path (`extend_pronunciation`) and the position-indexed beam
  (`src/lib.rs:611`), so **exact mode's scoring is unchanged and
  `exact_determinism` is unaffected**.
* `segmentation_floor` is computed once per segmentation from `slots`
  (`src/lib.rs:1433`) and threaded into `build` → `extend_fuzzy` → `extend_parts`.
  It is the same fold the traversal's suffix-minimum table already performs, so
  it is free, as the brief predicted.
* The traversal's admissible `bound` is changed **in the same commit to the same
  formula** (`src/lib.rs:1583`): `1 - (cost + suf_min_cost[k] - segmentation_floor) / 4`.
  This is not a second axis change and not scope creep — leaving the bound on
  the absolute form while the score became relative would make the bound
  *under*-estimate, and the traversal's best-first emission order would stop
  being descending in final score, silently truncating the search. Admissibility
  is preserved and is checked by the `#[cfg(test)]` property at
  `src/lib.rs:4864` and by `approx_determinism`.
* No weight in `axes::*` is touched. No other axis is touched. `coverage_tuples`,
  `sweep_index`, `EMIT_PROFILE_MAX_DEEP`/`EMIT_PROFILE_RESERVE`, the emission
  budget accounting in `build`, hashing, key derivation, collection order,
  `select_diverse`, the share cap and `STRUCTURE_FLOOR` are all untouched — none
  of them appears in the diff.

---

## 3. The regression test, landed red first

`tests/corpus_integration.rs`, `approximate_proposals_include_a_full_resegmentation`:
for each of seven ordinary multi-word targets, assert the visible proposals
contain at least one **full resegmentation**, compared with the ≥3-letter
shared-prefix rule. No wording is named in the test.

Red-first was established by measurement, not asserted: with the test landed and
`src/lib.rs` reverted to `27f9776` (`git stash push src/lib.rs`), it **fails**,
panicking on the first target:

```
"It's just a stupid game": none of the 50 visible proposals is a full
resegmentation; got: ["it justice too bad aim", "it justice too pad aim", ...]
test result: FAILED. 0 passed; 1 failed
```

Per-target red/green census (the test asserts each target in turn and panics at
the first red one, so the full census is the §1/§5 harness run over the same 13
targets):

| target | parent `27f9776` | branch `9a41d3` |
|---|---|---|
| It's just a stupid game | **RED** (0/50) | **GREEN** (1/50) |
| the cat sat on the mat | green (29/50) | green (33/50) |
| when the rain finally stopped | green (31/50) | green (27/50) |
| an old man in a big hat | green (35/50) | green (39/50) |
| recognize speech | green (44/50) | green (48/50) |
| what are you going to do | green (49/50) | green (48/50) |
| a whole lot of trouble | green (50/50) | green (41/50) |

The one full resegmentation the canonical target gains is `it said thus test oop
dame` at visible rank 48. `approximate_output_is_locked` and both acceptance
tests were **not** edited, and the rule was not weakened.

---

## 4. Measurement with the production scorer

All numbers from `MADGAB_TRACE_PHRASES` on the release binary, `--approximate
--top 50`.

### 4.1 the requested wording

| target | before | after |
|---|---|---|
| It's just a stupid game | **absent** from 17 906 candidates; rank-49 cutoff 0.915691888 | **absent** from 17 906 candidates; rank-49 cutoff **0.949336019** |
| recognize speech | absent | absent (not run after; see §7) |
| the cat sat on the mat | absent from 16 257; cutoff 0.893683280 | absent from 16 258; cutoff 0.940055069 |
| an old man in a big hat | absent from 13 093; cutoff 0.887714116 | absent from 13 106; cutoff 0.949475367 |
| what are you going to do | absent from 14 561; cutoff 0.905780912 | absent from 14 558; cutoff 0.953079944 |
| when the rain finally stopped | absent from 15 605; cutoff 0.914965805 | absent from 15 609; cutoff 0.937365810 |
| there is no way to know | absent from 20 112; cutoff 0.918311609 | absent from 20 115; cutoff 0.963151107 |
| a whole lot of trouble | absent from 16 159; cutoff 0.895695875 | absent from 16 162; cutoff 0.943713698 |

The requested wording is not emitted by the search, so the trace gate has no
rank or score for it either before or after. `c81e55` §5's predicted
0.819901 → **0.833368** is a number produced by *injecting* the wording through
`build` with that branch's throwaway probe, which is not on this branch and
which I did not rebuild; see §7. The acceptance test is therefore red before and
red after, and the canonical test's status is the **expected** one.

### 4.2 the green guard — the headline

| | before | after |
|---|---|---|
| raw rank of `wreck a nice beach` on `recognize speech` | **27** | **278** |
| raw score | **0.918313383** | **0.938313383** |
| rank-49 raw cutoff | **0.917045726** (`red 'cause i.'s peach`) | **0.945222712** (`red 'cause eyes each`) |
| **margin** | **+0.001267657** | **−0.006909329** |
| in the visible 50? | yes, at 28 | **no** |

**M7a dropped the guard.** Its own score rose by exactly 0.020 — its
segmentation's floor is 0.08, i.e. 0.25 × 0.08 = 0.02 of similarity credit — but
the whole field rose further, and the guard went from 27th to 278th of ~17 000.
`approximate_finds_recognize_speech_resegmentation` is **red on this branch**
and **green on `27f9776`**. I did not re-baseline it.

### 4.3 `approximate_output_is_locked`

Also red, and its diff is entirely a scoring shift plus a reordering. For
`I love you`, locked → actual:

```
 1. 0.938335 isle a view        ->  1. 0.952274 how ill view
 2. 0.937604 aisle a view          2. 0.952274 now ill view
 3. 0.937462 i.'s a view           3. 0.950346 yeah ill view
 4. 0.936762 eye a view            4. 0.950242 a ill view
 5. 0.931877 how ill view          5. 0.948670 ivy new
 6. 0.931877 now ill view          6. 0.946417 isle see new
 7. 0.930994 isle of new           7. 0.946407 i'll see new
 8. 0.930262 aisle of new          8. 0.945685 aisle see new
 9. 0.930120 i.'s of new           9. 0.945354 ivy too
10. 0.929948 yeah ill view        10. 0.944944 how law view
```

Every printed score rose by 0.006–0.014. Nothing was re-baselined; the numbers
are reported as the diff.

### 4.4 visible top 50, before and after

Every one of the 13 targets has a **completely replaced** visible list: 0 of the
50 before are still visible after, on all 13. Full lists were captured for all
13; the new top 10 of each:

| target | new top 10 after |
|---|---|
| It's just a stupid game | 0.960 it justice too day mm; 0.957 it justice too add aim; 0.957 it justice too bad aim; 0.956 eat justice too day mm; 0.956 it justice too pad aim; 0.954 it justice too add same; 0.954 it justice too day new; 0.954 it justice too bad same; 0.953 it justice too peg aim; 0.953 it justice too pad same |
| recognize speech | 0.954 yeah 'cause i.'s each; 0.953 yeah 'cause i.'s it's; 0.952 read 'cause i.'s each; 0.951 yeah 'cause i.'s pitch; 0.951 yeah 'cause i.'s peach; 0.951 read 'cause i.'s it's; 0.950 yeah 'cause eyes each; 0.950 red 'cause i.'s each; 0.949 yeah 'cause i.'s beach; 0.949 read 'cause i.'s pitch |
| I love you | 0.952 how ill view; 0.952 now ill view; 0.950 yeah ill view; 0.950 a ill view; 0.950 eye ill view; 0.950 why ill view; 0.950 how year view; 0.950 now year view; 0.949 ivy new; 0.949 how feel view |
| a whole lot of trouble | 0.957 who la t'other -able; 0.957 who la country -able; 0.954 show la t'other -able; 0.953 show la country -able; 0.951 home la t'other -able; 0.951 who raw t'other -able; 0.951 home la country -able; 0.950 who raw country -able; 0.949 show law t'other -able; 0.949 show law country -able |
| an old man in a big hat | 0.954 yeah know law many nab get; 0.952 new all danny ready get; 0.952 yeah know law many neb get; 0.952 yeah new all danny nab get; 0.952 yeah know all danny nab get; 0.952 yeah know law danny nab get; 0.952 as know law danny nab get; 0.952 at know law danny nab get; 0.952 eh know law many nab get; 0.952 add know law many nab get |
| he was a big fat man | 0.962 here wannabe get an; 0.960 here ana see get an; 0.959 here ana see get men; 0.958 here ana if team ann; 0.958 here ana if team anne; 0.956 here wannabe get as; 0.956 here wannabe get ann; 0.956 here ana i get men; 0.956 here gonna see get an; 0.956 here wannabe get anne |
| my brother has a red car | 0.953 mile other yeah their add-on; 0.953 mile other yeah there add-on; 0.952 mile other yeah their e-car; 0.952 mile other yeah there e-car; 0.952 mire other yeah their add-on; 0.952 mire other yeah there add-on; 0.952 mile other t'other add-on; 0.951 vibe other yeah their add-on; 0.951 vibe other yeah there add-on; 0.951 mile other add their add-on |
| put it back on the shelf | 0.951 too t-back news us else; 0.950 power t-back news self; 0.950 too t-back news ass else; 0.950 pull t-back news us else; 0.950 push t-back news us else; 0.949 pull t-back news ass else; 0.949 push t-back news ass else; 0.949 too t-back news a else; 0.948 paw t-back news us else; 0.947 too t-back news s else |
| she had a lot of money | 0.948 see -able ahh come-on see; 0.948 see -able awe come-on see; 0.948 see -able ah come-on see; 0.947 see -able ahh come any; 0.947 see -able awe come any; 0.946 see -able ah come any; 0.945 see -able are come-on see; 0.944 see -able on come-on see; 0.944 see -able ahh come funny; 0.944 see saddle ahh come-on see |
| the cat sat on the mat | 0.947 that's add-on emma too; 0.946 attacks add-on man too; 0.946 attach add-on man too; 0.945 catch add-on emma too; 0.944 new attach a tons matt; 0.944 new attack chat then at; 0.944 new attach a tons met; 0.943 attacks at news matter; 0.943 attach at news matter; 0.943 new attacks a tons matt |
| there is no way to know | 0.974 very new wait no-no; 0.972 very then wait no-no; 0.972 very new wait soon oh; 0.971 very new wait own oh; 0.971 very new we tune oh; 0.971 very new weight no-no; 0.970 very new we too oh; 0.969 very new weight soon oh; 0.968 very new weight own oh; 0.968 very then weight no-no |
| what are you going to do | 0.961 one sorry go inc to-do; 0.959 won sorry go inc to-do; 0.959 one sorry go see to-do; 0.959 one sorry go think to-do; 0.958 won sorry go it to-do; 0.958 new a sorry go to-do; 0.957 won sorry go see to-do; 0.957 won sorry go think to-do; 0.957 won sorry go a to-do; 0.957 one sorry go ink to-do |
| when the rain finally stopped | 0.947 wins air a. final east opt; 0.946 wears air a. final east opt; 0.945 wins air a. fine al east opt; 0.945 wether a. final east opt; 0.945 wears air a. fine al east opt; 0.944 n.'s air a. final east opt; 0.943 wares air a. final east opt; 0.943 n.'s air a. fine al east opt; 0.942 wins air a. fine air east opt; 0.942 wears air a. fine air east opt |

Two quality regressions the numbers above make visible and which I am reporting
rather than smoothing over: the canonical target's visible 50 now contains
`shit justice too day mm` at rank 42, and `put it back on the shelf` now shows
`ass` at ranks 3 and 7. Neither is a fence violation (no target word, no
acceptance phrase, no env read) but both are the objective preferring a cheap
segmentation floor over a readable answer, which is the same mechanism that
dropped the guard.

### 4.5 full-resegmentation counts, before and after

| target | before | after |
|---|---|---|
| It's just a stupid game | 0 | **1** |
| when the rain finally stopped | 31 | 27 |
| the cat sat on the mat | 29 | 33 |
| an old man in a big hat | 35 | 39 |
| a whole lot of trouble | 50 | 41 |
| recognize speech | 44 | 48 |
| what are you going to do | 49 | 48 |
| put it back on the shelf | 50 | 49 |
| there is no way to know | 50 | 49 |
| I love you | 50 | 50 |
| he was a big fat man | 50 | 50 |
| my brother has a red car | 50 | 50 |
| she had a lot of money | 50 | 50 |

---

## 5. Which mechanism turned `approximate_finds_recognize_speech_resegmentation` red

Stated plainly: **the one axis formula itself, and specifically the fact that a
per-segmentation floor is not comparable across segmentations.** There is no
special case, no branch, no test, no wording and no environment read anywhere in
the diff (§6). The mechanism is a property of the change as specified:

`1 - cost/4` is an absolute quantity: it asks "how close are these phones". Making
it relative asks "how much worse than the best wording *this* segmentation
admits". That is a meaningful question **inside** one segmentation, where every
candidate shares a floor, and it is a meaningless comparison **across**
segmentations, where two segmentations with floors 0.08 and 0.9 produce
candidates whose similarity values are not on a common scale. The result is that
the field's absolute level stops mattering and the segmentation whose cheapest
admitted wording is cheapest in absolute terms wins the pool. On
`recognize speech` that is the `'cause i.'s` structure, which now occupies all 50
visible slots' worth of advantage over the target-faithful `wreck a nice
beach` structure; on the canonical target it is the same effect, and it is why
the target finally produces a full resegmentation at all — the one thing M7a
bought is a reshuffle toward shallow, cheap-floor segmentations, not a better
resegmentation objective.

The same mechanism explains the score band rising 0.92 → 0.95 on every target and
the `ass`/`shit` entries: the objective stopped paying for absolute closeness.

This is also a **refutation of a c81e55 prediction**. §8 predicted "`wreck a
nice beach` unchanged to within 0.001". It moved by −0.0069 relative to the
cutoff and fell out of the visible 50. That prediction is wrong as measured here,
and it is the reason this front's recommendation is HOLD rather than MERGE.

I did not attempt a repair. The brief forbids the 21.3 % mass move explicitly,
and any smaller adjustment chosen by looking at where the guard sits would be
tuning to the acceptance clue — the same offence the fence test exists to
prevent, just relocated from `src/` to a constant. The honest outcome of this
front is: M7a as specified is not integrable, and the general statement it was
meant to establish (that the similarity axis is currently a bare `4.0` constant
with no scale) is real but is not fixable by making the axis relative to a
per-segmentation floor.

---

## 6. Fence scan

Over `git diff 27f9776..HEAD -- src/`, added lines only, for the tokens
`wreck`, `beach`, `recognize`, `justice`, `stupid`, `dupe`, `hid`, `came`,
`MADGAB_`, `ZZ_`, `zz_`, `env::var`:

```
added_lines=66  HITS=0
```

(`git grep` over the added `src/` lines for target strings, phrases, clue
wordings, substrings of them, and `env::var` returns nothing; the only literals
introduced are `0.0` and the identifier `cost_floor`.) No test was edited except
the newly **added** guard; neither acceptance test nor
`approximate_output_is_locked` is in the diff at all.

No probe scaffolding is on the branch: `git diff --stat 27f9776..HEAD` is
`src/lib.rs` and `tests/corpus_integration.rs` only. All measurement harnesses
(the 13-target driver, the full-resegmentation census script, and copies of the
pre- and post-change binaries) live outside the repository in
`/workspace/m9a41d3-measure/` and are untracked. Note for the coordinator: this
worktree also contains four **untracked** `examples/zz_*.rs` files
(`zz_measure.rs`, `zz_pool.rs`, `zz_spread.rs`, `zz_visible.rs`, mtime 19:03)
that I did not author and did not commit; they are not on the branch and nothing
of mine depends on them.

---

## 7. Test suites (`cargo test --release`)

| suite | `27f9776` (parent) | `9a41d3` (this branch) |
|---|---|---|
| `--lib` | 50 passed / 0 failed (per c81e55) | **53 passed / 0 failed** |
| `--test corpus_integration` | **10 passed / 1 failed** (`approximate_finds_classic_madgab_resegmentation`) | **9 passed / 3 failed** (`approximate_finds_classic_madgab_resegmentation`, `approximate_finds_recognize_speech_resegmentation`, `approximate_output_is_locked`) |
| `--test exact_determinism` | 1 passed / 0 failed | **1 passed / 0 failed** |
| `--test approx_determinism` | 2 passed / 0 failed | **2 passed / 0 failed** |
| `--test no_phrase_hard_coding` | 6 passed / 0 failed | **6 passed / 0 failed** |

The parent's `corpus_integration` figure is the branch state with `src/` and the
new test stashed, run on this host, so the new guard is not in that count. With
the new test present and `src/` reverted, `corpus_integration` is 10 passed /
**2** failed (the acceptance test plus the new guard), which is the red-first
census of §3.

Per `docs/environment-notes.md`, `cargo fmt`, `cargo clippy` and doctests **do
not exist on this host** and are not claimed.

### The canonical test by name

`approximate_finds_classic_madgab_resegmentation` is **RED** on this branch, and
that is the **expected** outcome. `c81e55` predicted the requested wording at
0.833368, still 0.093 short of the rank-49 cutoff, and warned in terms that M7a
"does not green the milestone and should not be sold as doing so". Nothing was
tuned toward green.

---

## 8. Runtime, interleaved, 7 replicates per arm

Wall clock per `--approximate --top 50` run, arms alternated inside each
replicate, medians of 7:

| target | parent median | branch median | delta |
|---|---|---|---|
| It's just a stupid game | **2705 ms** [2699 3078 2610 2472 2723 2765 2705] | **2543 ms** [2375 2818 2483 2510 2768 2750 2543] | **−6.0 %** |
| recognize speech | **2233 ms** [2333 2251 2223 2161 2223 2233 2429] | **2293 ms** [2482 2165 2330 2168 2285 2293 2409] | **+2.7 %** |

Both are inside the 15–25 % run-to-run spread visible in the raw replicates, and
the signs are opposite on the two targets, so I read this as no demonstrated
runtime cost. The change adds one `f64` per `Partial` and one O(slots × width)
fold per segmentation, which is the reason I expected it to be free.

---

## 9. What I did **not** measure or do

* **I did not measure the requested wording's score after the change.**
  c81e55's 0.833368 came from injecting an index tuple through `build` with that
  branch's throwaway `src/probe.rs` (`MADGAB_C81E55_INJECT`). That probe is not
  on this branch, and rebuilding it would mean landing probe scaffolding, which
  the brief forbids. The trace gate reports the wording as absent because the
  search does not emit it, so the honest statement is: **rank and score after
  the change are unmeasured, and the wording is still not in the pool.**
* **I did not run the requested wording's trace on `recognize speech`** after the
  change (it is absent there too, and the canonical and five ordinary targets
  cover the claim).
* **I did not run the base/branch interleaved timing protocol with the
  instrumented harness** of c81e55 §9; I used the plain binary and `Date.now()`
  around `execFileSync`, which measures process wall clock including corpus load
  (~0.5 s of every sample, identical on both arms).
* **I did not re-derive anything c81e55 already established**: the depth-cap
  arithmetic, the reserve-shape distributions, the class-walk model, the
  reweighting sweep, the cap-at-4 prediction, or the pool non-reproducibility
  defect (`w-6b91d3`). Pool sizes here differ by ±1–3 from run to run, which is
  that defect, and I did not treat any of it as signal.
* **I did not evaluate a repaired or variant axis.** No clamp, no blend, no
  per-segmentation normalisation, no weight change. The brief forbids the 21.3 %
  mass move explicitly, and any other adjustment picked by looking at the guard's
  position is the same offence at a different constant.
* **I did not merge anything.** `post-milestone-acceptance` and `main` were not
  touched. `src/lib.rs` contains no change to anything in the collision
  boundary.
* **I did not verify `cargo fmt` / `cargo clippy` / doctests** — they do not
  exist on this host.
* **I did not measure `select_diverse` as a lever**, though the post-change
  visible lists are visibly more concentrated in one structure per target
  (`yeah 'cause i.'s` holds all 50 slots on `recognize speech`). That is a
  downstream consequence of the score change, and whether the diversity policy
  should react to it is a separate question I did not open.

---

## 10. Recommendation

M7a is implemented as specified, is one general axis formula with no special
case, and is worth having as a **measurement**: it establishes, against
c81e55 §8's contrary prediction, that making the similarity axis relative to the
segmentation floor is not guard-safe. It costs a green milestone condition
(`wreck a nice beach`, +0.00127 → −0.00691, rank 27 → 278), it leaves
`approximate_output_is_locked` red, it leaves the canonical acceptance test red
as expected, and it changes the visible top 50 completely on all 13 targets while
making two of them visibly worse. Two green tests become red. That is a HOLD, and
the branch is pushed so the numbers are readable rather than re-run.

Artifacts: `49ae9c3` (src + the red-first guard), this report on the same branch.
Read it with `git show madgab-score-reseg-9a41d3:REPORT-9a41d3.md`.

MERGE RECOMMENDATION: HOLD
