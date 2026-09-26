# w-6ad4c1 — front `6ad4c1`: land `PUNCH` as an additive scoring axis

Branch `madgab-punch-6ad4c1`, opened by `coord-b3d9` at `ddcb4da`
(`post-milestone-acceptance`) and **not amended and not re-based**: the branch tip
is `ddcb4da` + the two commits below, published by push, never by force-push.
Release build throughout, `./target/release/madgab --approximate`.

**The one-line summary.** `PUNCH` — the share of a clue's words with one syllable
or fewer — is landed as a seventh additive axis in the bounded non-positive form
`+ w·(v − 1)`, `w = 0.10`, with the six existing weights untouched. It reproduces
front `558697`'s measured census on all thirteen targets exactly, keeps both guards
green, costs no measurable wall clock, and — contrary to what the item expected —
**needs no re-baseline of `approximate_output_is_locked` at all**, because the
lock's target band turns out to be entirely monosyllabic. It **does not close the
primary milestone** and this report does not claim it does.

---

## 0. This does not close the primary milestone, and the report says so first

`approximate_finds_classic_madgab_resegmentation` is **red on this branch**, before
and after, on this branch's base and on its base's base. `hits justice dupe hid
came` is still not emitted by the search on `It's just a stupid game`, still
absent from the deduplicated pool at `--top 50`.

The reason is measured, not guessed, and it is inherited from
`git show madgab-axis-558697:REPORT-558697.md`: on the canonical target the
requested wording and **every one of the 50 visible proposals by baseline score
carry `v = 0.800` exactly** (mean 0.8000, sd 0.0000). The axis is *saturated on
both sides of the comparison*, so it adds an identical constant to the clue and to
the band and can only reshuffle below the band. `558697` measured the consequence
as the requested wording moving from rank **9865 to 9999 of 17 913 at `--top 50`
on `3d520c0`** — 134 ranks the **wrong** way. I did not re-measure that rank; see
§6 for why, and for exactly what I did instead.

So the justification for landing `PUNCH` is **general output quality**, and it is
the one measured fix for the 0-of-50 full-resegmentation rate on the canonical
target that survives review: **0 → 30** (§3).

**Every number in this report is at a stated `--top_n`.** The pool is a strong
function of `top_n` (front `3e7b04`: 13 471 / 14 561 / 17 913 on the canonical
target at `--top 10` / `--top 20` / `--top 50` on `3d520c0`), and the visible-list
noise floor on this head is **exactly 0** (`3e7b04`, 3 replicates × 13 targets
byte-identical). Every comparison below is therefore stated as a replicate range
`X-X`, which is the honest way to write "no run-to-run variation at all here", and
every delta is a real delta rather than a comparison against a noise band. I
re-took the base arm's floor myself (13 targets × 3 replicates, both arms
identical run to run) and did not inherit a number I had not seen.

---

## 1. The diff

Committed delta of this branch: `src/lib.rs` (49 insertions, 2 deletions) and this
report. `git diff --name-only` is `src/lib.rs`; nothing else. No test file is
touched at all — see §5 for why, which is the one place this branch departs from
what the item anticipated.

Three edits, all in `src/lib.rs`:

**(a) one accumulator on the candidate the scorer already carries** (`src/lib.rs:2211`)

```rust
    /// Clue words of one syllable or fewer, for the `PUNCH` axis.  The
    /// syllable count is already computed on the extension path for
    /// `syllables`, so this is a comparison against a value the
    /// candidate already holds, not a second count.
    punch_count: usize,
```

**(b) one comparison per word extension** (`src/lib.rs:2344`)

```rust
        let syllables = approx::ipa_syllables(ipa);
        …
            syllables: self.syllables + syllables,
            // `ipa_syllables` floors an empty transcription at zero and a
            // non-empty one at one, so `== 1` is "one syllable or fewer"
            // and a word with no IPA at all is not counted as punch.
            punch_count: self.punch_count + usize::from(syllables == 1),
```

**(c) one new weight constant and one term in the combined score**
(`src/lib.rs:2415`, `:2524`)

```rust
+ axes::CLOSED_CLASS * closed_penalty
+ axes::PUNCH * (punch - 1.0);
…
    pub const PUNCH: f64 = 0.10;
```

**Not touched, and the additive claim is exactly this list.** `SIMILARITY` 0.25,
`NOVELTY` 0.15, `WORD_NOVELTY` 0.15, `FAMILIARITY` 0.10, `RHYTHM` 0.30,
`SHAPE` 0.05, `CLOSED_CLASS` −0.15 are all unmodified, and no existing axis'
*definition* is modified — the similarity axis in particular is left on its
absolute `1 − cost/4` form, which is the thing front `9a41d3`'s M7a changed
(`MERGE RECOMMENDATION: HOLD`: it moved `wreck a nice beach` from raw rank 27 with
margin +0.001267657 to raw rank 278 with margin −0.006909329, red-flipped a green
guard, and `MERGE RECOMMENDATION: HOLD`). None of the six lines appears in the
diff. Also absent from the diff: `coverage_tuples`, `sweep_index`,
`EMIT_PROFILE_MAX_DEEP`, `EMIT_PROFILE_RESERVE`, `select_diverse`, the share cap,
`STRUCTURE_FLOOR`, and every other identifier the item fences.

## 2. The representation, and the alternative I rejected

**The problem, as measured rather than asserted.** The six existing weights sum to
exactly 1.00 and `CLOSED_CLASS` is the only signed one, so the objective's maximum
is exactly 1.0 and there is **zero headroom**. `558697` measured the consequence
(`REPORT-558697.md` §0): at weight 1e-9 `approximate_output_is_locked` still passes
because the delta is below print precision; at every weight at or above 0.05 all
four of its candidates redden it, and `generates_for_a_real_phrase` (which asserts
`0.0..=1.0`) goes red with it.

**What I landed: `+ w·(v − 1)`, `w = 0.10`.** The term is non-positive by
construction, so the maximum of the objective is unchanged at 1.0 and the
`Clue::score ∈ [0,1]` upper bound needs no renormalisation hack, no weight moved
and no axis rescaled. It is **ordering-identical to `+ w·v`**: the same constant
`−w` comes off every candidate, so no comparison between two candidates changes.
That is what makes it a *representation* of the same axis rather than a different
one, and the census in §3 is `558697`'s census reproduced on a different commit
through this form, which is the evidence for it.

**The alternatives I rejected, and why:**

1. **`+ w·v` (naive).** Rejected on the measurement: it breaks the documented
   upper bound at every weight above print precision and reddens two green tests
   for a reason that has nothing to do with the axis's merit. This is the same
   blocker that put M7a on HOLD.
2. **Renormalise the six existing weights to leave room (e.g. scale by
   `1/(1+w)`).** Rejected on the additivity rule, criterion 3: it moves six
   existing weights and rescales every existing axis. It is M7a's failure mode
   with extra steps, and M7a's verdict is the precedent.
3. **Rescale one axis by a small epsilon to make room.** Same rejection, and worse
   in one respect — it makes the loss *target-dependent*, so a reviewer would have
   to trust that the epsilon is small enough everywhere.
4. **A weight small enough to hide below print precision (the `1e-9` check).**
   Rejected as not an axis: it changes no ordering at all, so it would buy
   nothing and cost a constant. The interesting question is whether the *shape* is
   worth landing, and the answer is yes at 0.10, not at 1e-9.
5. **Clamp the total into `[0,1]` after summing.** Rejected: a clamp is a
   non-additive, order-violating projection on the objective, and it would make
   the score a function of *where the candidate sits* rather than of what it is.

**The one honest cost of the bounded form, stated rather than absorbed:** it makes
the term non-positive, so the *lower* end of the documented range moves down by up
to `w`. The lower bound was already soft before this change — `CLOSED_CLASS` is
subtracted, so a clue that is nothing but function words already scored as low as
−0.15 — and no test asserts it. The change makes a soft bound softer (−0.25 rather
than −0.15 in the worst case). I did not clamp, because clamping is alternative 5.

## 3. What it buys, per target

`--approximate --top 50`, release, 3 replicates per target per arm, both arms at
`top_n = 50`. Base arm is a `git archive ddcb4da | tar -x` extraction built
separately. Census rule is `c81e55`'s verbatim: two words are the same word if
`a === b` or both are ≥ 3 letters and one is a prefix of the other; a visible
proposal is a *full resegmentation* if none of its words is the same word as any
of the target's. `turnover` = how many of the 50 visible phrases are not in the
base visible 50.

| target | full resegs base | full resegs `PUNCH` | range (3 reps) | turnover /50 | pool base → `PUNCH` |
|---|---|---|---|---|---|
| **It's just a stupid game** (canonical) | **0** | **30** | 30-30 / 0-0 | 31 | 17 913 → 17 827 |
| recognize speech | 44 | 44 | 44-44 | 2 | 15 908 → 15 926 |
| the cat sat on the mat | 29 | **20** | 20-20 / 29-29 | 39 | 16 255 → 16 639 |
| when the rain finally stopped | 31 | **50** | 50-50 / 31-31 | 21 | 15 609 → 14 781 |
| an old man in a big hat | 35 | **50** | 50-50 / 35-35 | 50 | 13 041 → 13 119 |
| what are you going to do | 49 | **50** | 50-50 / 49-49 | 50 | 14 548 → 13 929 |
| there is no way to know | 50 | 50 | 50-50 | 43 | 20 119 → 19 885 |
| she had a lot of money | 50 | 50 | 50-50 | 50 | 13 341 → 13 366 |
| a whole lot of trouble | 50 | 50 | 50-50 | 50 | 16 169 → 16 173 |
| he was a big fat man | 50 | 50 | 50-50 | 39 | 19 168 → 19 229 |
| my brother has a red car | 50 | 50 | 50-50 | 48 | 15 649 → 15 509 |
| put it back on the shelf | 50 | 50 | 50-50 | 17 | 15 642 → 15 826 |
| the other seven | 15 | **22** | 22-22 / 15-15 | 34 | 20 023 → 20 106 |
| I love you (the lock target) | 50 | 50 | 50-50 | 1 | 9 338 → 9 346 |

Twelve ordinary targets plus the canonical one plus the lock target; the item asks
for at least six ordinary. **This table is `REPORT-558697.md` §2 C1's table and
§2's turnover column, reproduced row for row and number for number**, on a
different commit, through the bounded representation instead of the naive one.
That is the strongest evidence I have that the landing is the axis that front
measured, and I did not have to re-derive any of its design.

**The cost, kept in the report and not dropped: `the cat sat on the mat` goes
29 → 20, a loss of 9.** It is the one target in the set whose every word is
already monosyllabic, and the mechanism is visible in the definition: the axis
asks the clue to be as short-worded as the target is, so it has nothing to buy
there and a polysyllabic near-homophone of `cat`, `mat` or `sat` is now charged for.
Six of the twelve ordinary targets were already at 50/50 and stay there, so the
net census movement over the twelve is strongly positive, but a user searching a
target made only of short words can get a *worse* list. A front that wants to
remove that would have to make the axis target-relative (compare the clue's
monosyllabic share with the **target's**), which is a different axis and one I did
not measure.

**Also worth stating: `PUNCH` is substantially redundant with the existing
objective.** `558697` measured `r(PUNCH, score)` at a median of ≈ 0.70 over the
whole pool on thirteen targets. What it actually does is reorder near-ties, which
is why turnover is 17–50 of 50 on targets whose census does not move at all. A
reviewer should read the census column as the axis's value and the turnover column
as its churn.

## 4. Cost, and the guards

**Cost of computing `PUNCH`: no new traversal pass, no new scan, no new data.**

* `Partial` gains one `usize` (8 bytes) on a struct that already carries five
  running aggregates for exactly this reason (`reused_count`, `familiarity_sum`,
  `shape_sum`, `syllables`, `closed`).
* `extend_parts` gains one `usize` addition and one `==` comparison per word
  extension. `approx::ipa_syllables(ipa)` was **already being called on that line**
  for `syllables`; the diff hoists it into a local and reuses the value. There is
  **no second syllable count** anywhere.
* `metrics` gains one `f64` division and one multiply-add per call. `metrics`
  already runs in every beam comparison; no traversal-visible quantity is touched.
* No allocation, no I/O, no dictionary lookup, no environment read, no new pass
  over the word list, no new pass over the pool.

**Wall clock**, arms interleaved inside each replicate, 8 pairs per arm per
target, `--approximate --top 50`, the search-ms the binary itself reports:

| target | base `ddcb4da` | branch `6ad4c1` | delta |
|---|---|---|---|
| It's just a stupid game | 1410 / **1427** / 1452 | 1392 / **1426** / 1479 | median **−0.1 %** |
| recognize speech | 1267 / **1300** / 1336 | 1250 / **1266** / 1300 | median **−2.6 %** |

min / median / max. Both deltas are inside the within-arm spread and the sign
favours the branch, so the honest reading is **no measurable cost** — consistent
with `558697` §2 C1.

**Guard 1 — `wreck a nice beach` for `recognize speech`.** Verified in three
configurations, on the branch, with the release binary, plus by the test.

| configuration | base `ddcb4da` | branch `6ad4c1` |
|---|---|---|
| CLI defaults (no `--top`, i.e. `top_n` 10 / `beam` 64) | raw rank 27, score 0.918313383, rank-9 cutoff 0.920068538, margin **−0.001755155** | raw rank **27**, score **0.918313383**, rank-9 cutoff **0.920068538**, margin **−0.001755155** |
| `--top 50` | raw rank 27, score 0.918313383, rank-49 cutoff 0.917045726 (`red 'cause i.'s peach`), margin **+0.001267657** | raw rank **27**, score **0.918313383**, rank-49 cutoff **0.917025536** (`rec a guys beach`), margin **+0.001287847** |
| visible list at `--top 50` | 28th of 50 | **28th of 50** |
| `approximate_finds_recognize_speech_resegmentation` | ok | **ok** |

Three things to be precise about, because two of them are not what the item's
wording implies:

* **At the CLI's default configuration the guard phrase is *not* in the visible
  top 10 — on either arm.** The default `top_n` is 10, and `wreck a nice beach`
  sits at raw rank 27, so it is below the default display cutoff. That is
  pre-existing and identical on base; this change neither caused nor fixed it. The
  guard is *in the pool at raw rank 27 with an unchanged score* at the CLI
  defaults, which is the property that can actually be checked there.
* The guard's **score is bit-identical to base**, because the entire visible band
  on `recognize speech` is monosyllabic, so `v = 1.0` and the term is exactly
  `0.10 · 0.0 = 0`. I measured this: all 48 proposals shared between the two
  visible 50s at `--top 50` have a score delta of exactly `0.000000`.
* The `--top 50` **margin improves by 2.0e-5** (+0.001267657 → +0.001287847) and
  the rank-49 candidate changes from `red 'cause i.'s peach` to `rec a guys
  beach`. That is `558697`'s number to the digit, from a second commit and through
  the bounded form. The guard is not traded away, and it is not bought either: at
  a margin of 0.0013 a change of 2e-5 is noise dressed as a gain, and I am not
  claiming it as a result.

**Guard 2 — `approximate_finds_recognize_speech_resegmentation`**: **green**, see
§5.

## 5. Suites, and the output lock

All on the branch, release:

| suite | branch `6ad4c1` | base arm (`git archive ddcb4da`) | verdict |
|---|---|---|---|
| `--release --lib` | **ok — 53 passed, 0 failed, 0 ignored** | ok — 53 passed, 0 failed, 0 ignored | no change |
| `--test corpus_integration` | **FAILED — 10 passed, 1 failed, 0 ignored** | FAILED — 10 passed, 1 failed, 0 ignored | **pre-existing red** |
| `--test no_phrase_hard_coding` | **ok — 6 passed, 0 failed** | ok — 6 passed, 0 failed | no change |
| `--test approx_determinism` | **ok — 4 passed, 0 failed** | ok — 4 passed, 0 failed | no change |
| `--test exact_determinism` | **ok — 1 passed, 0 failed** | ok — 1 passed, 0 failed | no change |

**The one red is pre-existing, shown to be pre-existing and not excused.** The
only failure on either arm is
`approximate_finds_classic_madgab_resegmentation` at
`tests/corpus_integration.rs:136`, `canonical clue missing from top 50`, on the
same line with the same message and the same 10-pass/1-fail split. Its printed 12
proposals are the 12 base proposals **plus** the new full resegmentation
`it said thus test oop dame` in second place, which is the §3 census improvement
showing up in the failure payload. `9a41d3` and `558697` both recorded the same
red on their parents; it is the milestone this item does not claim to close.

**The output lock: no re-baseline was needed, and I did not make one.** The item
said a re-baseline "is unavoidable for any scorer change", with the side-by-side
evidence attached. I ran the side-by-side expecting to need it, and the honest
finding is that this axis does not move the lock at all, so re-baselining would
have been a *change* rather than a re-baseline. `approximate_output_is_locked`
**passes unmodified**, and here is the per-target old-versus-new comparison it
asks for.

`approximate_output_is_locked` locks one target, `I love you`, at
`mode: approximate, top_n: 10`, as `{:.6} {phrase}` strings. Side by side,
`--approximate --top 10` on the release binaries (3 dp, which is what the CLI
prints; the test's 6 dp assertion passes unmodified, which is the 6 dp evidence):

| rank | OLD `ddcb4da` | NEW `6ad4c1` | |
|---|---|---|---|
| 1 | 0.938 `isle a view` | 0.938 `isle a view` | identical |
| 2 | 0.938 `aisle a view` | 0.938 `aisle a view` | identical |
| 3 | 0.937 `i.'s a view` | 0.937 `i.'s a view` | identical |
| 4 | 0.937 `eye a view` | 0.937 `eye a view` | identical |
| 5 | 0.932 `how ill view` | 0.932 `how ill view` | identical |
| 6 | 0.932 `now ill view` | 0.932 `now ill view` | identical |
| 7 | 0.931 `isle of new` | 0.931 `isle of new` | identical |
| 8 | 0.930 `aisle of new` | 0.930 `aisle of new` | identical |
| 9 | 0.930 `i.'s of new` | 0.930 `i.'s of new` | identical |
| 10 | 0.930 `yeah ill view` | 0.930 `yeah ill view` | identical |

**Byte-identical, on both the score strings and the proposal membership.**
`approximate_output_is_locked` was not edited, relaxed, skipped or `#[ignore]`d,
and **no test file is in this branch's diff at all**.

The mechanism, and it is worth a reviewer knowing because it is the reason the
lock survived: the lock target's visible band is **entirely monosyllabic**
(`isle, aisle, i.'s, eye, how, now, view, of, new, yeah`), so `v = 1.0` on every
one of the locked strings and the term is exactly zero. I measured this at
`--top 50` on the same target: all 49 proposals shared between the two arms have a
score delta of exactly `0.000000`. The bounded form is *not* what saved the lock —
`+ w·v` would have added exactly `+0.10` to every one of them and reddened the
test. **The bounded form is what saved the `Clue::score ∈ [0,1]` contract**, and
the lock survived for the separate, lucky reason that this particular target's
answers are all monosyllables. A different lock target would have needed the
re-baseline, and the re-baseline is still the right call the next time one is
needed — just not here.

**Membership churn elsewhere, stated because the lock is not the whole story.** The
13-target visible lists do turn over (§3's turnover column: 2 to 50 of 50), and
the pools move by −828 to +384 candidates at `--top 50`. The item's rule is that a
lock whose proposal *membership* changes without a matching argument is a
rejection, so the argument is: the locked target's membership does **not** change;
the churn is on twelve unmeasured-by-any-lock ordinary targets, and it is the
intended effect of the axis, visible as the census movement in §3. Proposal
membership on the one locked target is unchanged.

## 6. What I did **not** measure

* **The requested wording's rank and margin after the change.** `558697` measured
  it by *injecting* a segmentation tuple through `build` with throwaway probe
  scaffolding; that scaffolding is not on this branch, and the item forbids
  committing it. The trace gate cannot do it either, because the wording is not in
  the pool, so the trace reports `missing candidates` and no rank. What I can say
  without re-deriving it: the two forms are ordering-identical, so 558697's
  9865 → 9999 of 17 913 at `--top 50` on `3d520c0` is the expected direction and
  magnitude here, on a different commit with a different pool. **I did not verify
  it and I am not restating it as my own measurement.** I did not judge it worth
  landing injection machinery to re-derive an ordering-identical number.
* **Saturation of the requested wording itself.** `558697` measured it at 0.800
  on the clue and 0.800 across the band. I re-measured the *band* half
  independently: on the canonical target all 19 proposals shared between the two
  visible 50s have a score delta of exactly `−0.020000`, i.e. `v = 0.800` on every
  one of them, and 31 of the 50 visible slots turned over. I did not measure the
  clue half.
* **A target-relative variant** (`v` measured against the **target's** own
  monosyllabic share rather than as an absolute), which is the obvious way to buy
  the §3 `the cat sat on the mat` loss back. Not implemented, not measured. It is
  a different axis with a different definition and it is the first thing I would
  measure next.
* **Weights other than 0.10.** No sweep, no sensitivity analysis, and I make no
  claim that the census is monotone in the weight. `558697` measured 0.05–0.20 for
  its own purposes; I did not repeat it.
* **Any interaction with `select_diverse`, the share cap or `STRUCTURE_FLOOR`.**
  The §3 visible lists are the *visible* output, so the diversity policy is
  downstream of everything I measured and I did not characterise its response to a
  reordered band. All three are untouched in the diff.
* **The structural DP's proxies.** `partial_span_score`, `complete_span_score`,
  `span_score_bound` and `SlotAlt::contribution` were deliberately **not** given a
  `PUNCH` term, so the segmentation search still orders structures on the old six
  axes. That is deliberate and it is `558697`'s implementation: adding a
  non-positive axis to `span_score_bound` would only loosen an already-sound
  bound, and adding it to the two ranking proxies would change the search's
  structure ordering, which is a second, unmeasured change. A reviewer should know
  the proxies and the final scorer now disagree by up to `w`; the disagreement is
  a bounded constant, so it cannot change which final score is highest among
  candidates the search actually reaches, only which structures it explores.
* **Composition with M7a.** Nothing measured on top of front `9a41d3`'s branch.
  M7a is on HOLD and I did not touch it.
* **Targets beyond the 14 in §3**, `--top 10` and `--top 20` membership, and the
  wasm/web build. `cargo fmt`, `cargo clippy` and doctests do not exist on this
  host (`docs/environment-notes.md`); I did not run them and claim nothing about
  them.
* **Whether the §3 churn is an improvement in any sense other than the census
  count.** Turnover of 50 of 50 on six targets means the user sees an almost
  entirely different list of answers for the same sentence. The census says some
  of those are better; nothing here says the *rest* are not worse, and `9a41d3`
  showed what a scorer change looks like when it quietly degrades two targets
  (`ass` at ranks 3 and 7, `shit` at rank 42). I did not read the new visible
  lists for quality regressions, and a reviewer who cares should spot-check
  `put it back on the shelf` and `there is no way to know`, the two targets where
  the census was already saturated and only the churn moved.

**Fence scan.** `tests/no_phrase_hard_coding` is **6/6** with the patch applied.
Over the added `src/` lines: no target word, no acceptance phrase, no substring of
one, no dictionary entry, no `env::var`, no `MADGAB_`-prefixed name, no
`ZZ_`/`zz_` identifier, no branch on a specific word or phrase. The only literals
introduced are `0`, `1`, `1.0` and `0.10`. The mechanism in the comment is stated
as a property of *any* sentence, not of any of these.

## 7. Artifacts and reproduction

Committed: `src/lib.rs` and this report, nothing else. Untracked and outside the
repository: `/workspace/opencode/run13.sh` (13-target × 3-replicate runner),
`census.js` (`c81e55`'s verbatim full-resegmentation rule), `turn.js` (visible
turnover), `out_base/` / `out_punch/` / `out_b/` (visible stdout, stderr and
per-target pool dumps for both arms), `/workspace/opencode/base6a/` (the
`git archive ddcb4da` base arm, built and tested separately). No probe, no gate
and no scaffolding was needed for anything in this report, and there is nothing to
clean up in the worktree.

```
cargo build --release
# per-target census + turnover, 3 replicates, both arms
./target/release/madgab --approximate --top 50 "It's just a stupid game"
# the guard, at the CLI defaults and at --top 50
MADGAB_TRACE_PHRASES="wreck a nice beach" madgab --approximate "recognize speech"
MADGAB_TRACE_PHRASES="wreck a nice beach" madgab --approximate --top 50 "recognize speech"
# the lock, old vs new
madgab --approximate --top 10 "I love you"
# the pre-existing red, on both arms
git archive ddcb4da | tar -x -C ../base6a && cd ../base6a && cargo test --release --test corpus_integration
```

## 8. Recommendation

`PUNCH` is one general additive term, computed from a value the scorer already
computes, with no weight moved, no axis rescaled, no similarity-axis change, no
probe, no constant tuned toward an acceptance phrase, and no test touched. It
reproduces the design front's per-target census exactly on a different commit
through a different (bounded) representation. It keeps both guards green, it costs
no measurable wall clock, it needs no lock re-baseline, and it fixes a
user-visible defect — a 0-of-50 full-resegmentation rate on a five-word target —
that is its own reason for existing.

It also does not close the milestone, moves the requested wording the wrong way by
inherited measurement, costs 9 on one ordinary target, is substantially redundant
with the existing objective, and churns 17–50 of 50 visible proposals on twelve
unlocked targets. Those are the reviewer's to weigh, and they are all in this
report rather than in a footnote. On the evidence here the axis is worth having
and the integration risk is confined to the visible lists; nothing in the guarded
surface regresses.

Reviewer's checklist, in the order I would check it: criterion 3 (additive —
`git diff ddcb4da..HEAD -- src/lib.rs`, six weight lines, one added line in the
sum), criterion 5 (both guards, CLI defaults, §4), the lock's zero delta and its
mechanism (§5), and the `−9` on `the cat sat on the mat` (§3).

MERGE RECOMMENDATION: MERGE
