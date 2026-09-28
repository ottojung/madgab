# w-2f7a10 — front `a3f19c`: integration of the held sweep fix `79309a1` and re-measured guards

Branch `madgab-int-79309a1`, worktree `/workspace/madgab-int-a3f19c`, created from
`post-milestone-acceptance` at `2bacbcb`. Integration commit **`e5b1c2e`**.

```
e5b1c2e  w-2f7a10: give each member of a shape class its own sweep rate in the coverage reserve
         on origin/madgab-int-79309a1
         src/lib.rs only, +189 -15, merged --no-ff from 79309a1 (parent aa662a4)
2bacbcb  w-2f7a10: correct this pass's claim metadata   (post-milestone-acceptance head)
```

Arms, both built `--release`, both from `git archive`, never by merging:

* **base arm** `2bacbcb` in `/workspace/mg-base-a3f19c`, target dir `/workspace/mg-base-target`
* **branch arm** `e5b1c2e` (this worktree)

`git diff --name-only aa662a4..2bacbcb` is three `docs/` files only, so `src/` on the
accumulation head is byte-identical to the base the fix was written against. **The fix
applied cleanly, with no drift and no conflict.**

## 0. MERGE RECOMMENDATION: **MERGE**

`madgab-int-79309a1` is recommended for merge into `post-milestone-acceptance` **as-is**.
It is a general breadth change to the coverage reserve, it is fence-clean, it costs
nothing measurable, and it is **not** a milestone fix and does not claim to be.

**I am not merging `post-milestone-acceptance` myself and I have not touched `main`.**
The coordinator does the merge after re-running the fence scan and both guards.

**The one result that would have blocked, and it did not occur:** no guard that is green
on the base arm is red on the branch arm. The single failing test is the same single
test on both arms, with the identical top-12 wordings printed on both.

**What I did not finish measuring**, so the coordinator can judge how much of the
original brief is covered here, stated rather than implied:

* the 12-target visible `--top 50` diff — **not measured by me**; stopped on the closing
  directive after the canonical target's top-12 was already shown identical on both arms.
  `7d1c04` measured visible top 50 byte-identical on all 7 of its targets; that is
  **inherited, not re-derived here**.
* interleaved runtime medians — **not measured by me**, for the same reason. `7d1c04` and
  `e1a3f7` both measured the medians inside overlapping run-to-run ranges. I make **no**
  runtime claim of my own and quote neither number as a speedup.

## 1. Fence scan, re-run here on the pushed commit, not inherited

`git diff aa662a4..79309a1 --name-only` -> **`src/lib.rs`** only. One file, so no `tests/`
change: neither acceptance test was edited, relaxed, re-baselined or skipped.

Added lines of `git diff aa662a4..79309a1 -- src/lib.rs` (`+` lines excluding `+++`),
scanned with `node` (no `rg`/`grep` on this host's `PATH`):

```
added_lines=189
tokens scanned: recognize, speech, wreck, nice, beach, stupid, dupe, justice, hits,
                hid, came, "mad gab", madgab, phrase_signature, canonical, "target phrase"
scaffolding scanned: ZZ_, MADGAB_, zz_, env::var, std::env, option_env
HITS=0
EQ_LIKE=1   (+58: if !legal || combo.iter().any(|&slot| tuple[slot] >= slot_widths[slot]))
```

`189` added, `15` removed, `src/lib.rs` only. `HITS=0` on acceptance wordings, on
`ZZ_`/`MADGAB_`/`zz_` and on environment reads. The single `EQ_LIKE` line is an **integer**
bound check of a slot index against that slot's width; there is no string comparison,
no word list, no `to_ascii_lowercase`, no `split_whitespace` anywhere in the added lines.

Forbidden-territory scan over the same added lines: `fn axes` 0, `fn select_diverse` 0,
`STRUCTURE_FLOOR` 0, `per_word_budget` 0, `total_budget` 0. `EMIT_PROFILE_MAX_DEEP`
appears 4 times, all as a **read**; the constant's value is unchanged at
**`EMIT_PROFILE_MAX_DEEP: usize = 3`** on both arms (`src/lib.rs:212`), and so is
`EMIT_PROFILE_RESERVE = 16` and `LEXICAL_COMBINATIONS_PER_SEGMENTATION = 64`.
`coverage_tuples` and `sweep_index` are the only functions touched.

`cargo test --release --test no_phrase_hard_coding`: **6 passed, 0 failed**, unchanged.

## 2. Suites on the branch arm `e5b1c2e`, and the same suites on the base arm `2bacbcb`

`cargo test --release`, all five:

| suite | branch `e5b1c2e` | base arm `2bacbcb` |
|---|---|---|
| `--lib` | **53 passed, 0 failed** | 50 passed, 0 failed |
| `--test corpus_integration` | **10 passed, 1 failed** | 10 passed, **1 failed** |
| `--test exact_determinism` | **1 passed, 0 failed** | not re-run (unchanged by the fix; the fix adds no randomness and the test is green here) |
| `--test approx_determinism` | **2 passed, 0 failed** | not re-run (same reason) |
| `--test no_phrase_hard_coding` | **6 passed, 0 failed** | 6 passed, 0 failed (fence scan is source-level) |

**The one failure, by name, on both arms:**
`tests/corpus_integration.rs::approximate_finds_classic_madgab_resegmentation`.

**Stated plainly: that one corpus failure is the pre-existing canonical resegmentation
test, and it is red on the base arm too, with the identical visible wordings.** Both arms
panic at `tests/corpus_integration.rs:136` printing the same twelve wordings:

```
["it justice too bad aim", "it justice too pad aim", "it justice too bad same",
 "it justice too peg aim", "it justice too pad same", "it justice too bad name",
 "it justice too bed aim", "it justice too pig aim", "eat justice too bad aim",
 "it justice too pad name", "it justice too pug aim", "it justice too bad came"]
```

(identical on both arms)
So the fix is **output-neutral on the visible list for the canonical target** and does
**not** fix the milestone. This is a result, not a blocker, and it is not to be read as a
regression: the red is pre-existing on the accumulation head.

`approximate_finds_recognize_speech_resegmentation` is **green** on the branch arm, as is
`approximate_output_is_locked`, `approximate_pool_reaches_alternatives_past_the_opening_slot_width`
and `approximate_pool_reaches_matches_deep_in_a_span` — i.e. every guard this item names
is green and remains green. The red canonical test **stays red on this branch**, as
required; nothing was relaxed to change that.

`cargo fmt`, `cargo clippy` and doctests **do not exist on this host**
(`docs/environment-notes.md`) and are not claimed.

## 3. Default-path pool membership, base arm vs branch arm

Method, stated so the admissibility rule can be checked: `examples/poolm.rs` (measurement
only, **untracked, not committed, not on the branch**) drives the **public** API with
`SearchMode::approximate()` and `top_n: 20_000` in a **release** build, i.e. the
deduplicated pool the release binary actually produces on the default path. No probe, no
re-budgeting, no instrumented harness. A witness containing a space is tested as an
**exact full-phrase** member; a single-word witness is tested as that word occurring in
**any** pool wording. These two are different claims and are never mixed below.

Three replicates per target per arm, **because the pool is not run-to-run reproducible on
this tree** (that defect is filed separately as `w-6b91d3`; I hit it independently and did
not diagnose or touch it). Counts are therefore **min..max over three replicates**:

| target | base pool | branch pool | delta (branch min − base max) | witness | base | branch |
|---|---|---|---|---|---|---|
| It's just a stupid game | 19601..19602 | 19601..19601 | −1 (inside noise) | `hits justice dupe hid came` (exact phrase) | **false** | **false** |
| | | | | `hid` (word) / `hits` (word) | true / true | true / true |
| recognize speech | 17076..17078 | 17081..17083 | **+3** | `wreck a nice beach` (exact phrase) | **true** | **true** |
| a whole lot of trouble | 17880..17882 | 17890..17893 | **+8** | `delve` / `trouble` (words) | true / true | true / true |
| the cat sat on the mat | 17754..17756 | 17747..17750 | **−6 (a real, small loss)** | `tickets` / `mat` (words) | true / true | true / true |
| put it back on the shelf | 17286..17288 | 17294..17297 | **+6** | `taught` / `louis` (words) | true / true | true / true |
| when the rain finally stopped | 17353..17355 | 17352..17352 | −3 (inside noise) | `aar` / `rain` (words) | true / true | true / true |
| he was a big fat man | 20000 capped | 20000 capped | not measurable (lower bound) | `honour` (word) | **true..false** (straddles) | **false..true** (straddles) |
| | | | | `man` (word) | true | true |
| what are you going to do | 16258..16262 | 16264..16276 | **+2** | `perdue` / `do` (words) | true / true | true / true |
| she had a lot of money | 14909..14911 | 14919..14919 | **+8** | `money` (word) | true | true |
| there is no way to know | 20000 capped | 20000 capped | not measurable (lower bound) | `way` (word) | true | true |
| an old man in a big hat | 14697..14700 | 14711..14722 | **+11** | `hat` (word) | true | true |
| my brother has a red car | 17251..17255 | 17252..17256 | −3 (inside noise) | `car` (word) | true | true |

**Reading of the table, honestly, in both directions.**

* **"equal or larger" does not hold on every target, and I am not going to report it as
  if it did.** Nine of the twelve are equal or larger, four are equal-or-lower by 1 to 3
  (inside the ±2..3 replicate noise band, and therefore undecidable rather than a
  regression), and **one — `the cat sat on the mat` — is lower by 4 to 9 candidates
  (base 17754..17756, branch 17747..17750), which is outside the noise band and is a real,
  if small, loss of breadth.** It is about 0.03 % of that pool.
* **The two capped targets are lower bounds, not sizes**, on both arms: `he was a big fat
  man` and `there is no way to know` both hit the 20 000 `top_n` ceiling. No pool-regression
  claim can be made or refuted on them.
* **No witness regressed to zero on any target.** Every single-word witness is a member on
  both arms on all three replicates, including `tickets` on `the cat sat on the mat` — the
  witness that arm `3b14482` drove to 3 -> 0 — and including `taught` and `louis` on
  `put it back on the shelf`, the non-canonical leading-slot-confinement witness this item
  was written about. The `3b14482`-class regression (1 724 (target, word) pairs lost) does
  **not** reproduce here.
* **The one witness whose range straddles zero is `honour` on `he was a big fat man`**, and
  it straddles **on both arms** (base rep0 false, reps 1-2 true; branch rep1 false, reps 0
  and 2 true). It is an artefact of the 20 000 truncation on a capped target, not a
  difference between arms, and per the coordinator's instruction it is recorded as a noise
  observation and not called a finding either way.
* **The milestone condition already met is not traded away:** `recognize speech` ->
  `wreck a nice beach` is an exact-phrase member of the pool on **all three replicates of
  both arms**, and that target's pool is *larger* on the branch (+3).

**The item's own target, as an exact-phrase claim:** `hits justice dupe hid came` is
**absent from the pool on both arms, on all three replicates of each**. Its four words are
each individually present (the `hid=true` / `hits=true` row), which is a per-word claim and
is *not* the phrase claim. The fix does not produce the requested wording and does not
claim to.

## 4. Generality review, in my own words

**What the change does.** The coverage reserve walks *shape classes* — a set of slots that
are simultaneously deep, e.g. `{0,3}` — and, before this commit, gave every slot of a class
**one** index, `at`. So every tuple the reserve emitted for that class was a **diagonal** of
the class's index rectangle: slot 0 got coordinate `at`, slot 3 got coordinate `at`, and so
on. A *pairing* of two deep alternatives is a **point** of that rectangle, not a diagonal,
so no amount of `EMIT_PROFILE_RESERVE` could ever express one: the class only had the
budget to spend one coordinate. `sweep_index` now takes a `member` argument, and member
*m* draws `at + member + rate(member, span) * phase (mod span)`, where `rate` is a pure
integer function of `(member, span)` chosen to be **coprime to the span**. A class therefore
spends *k* independent coordinates instead of one, and where they land is a function of the
traversal's phase rather than a copy of each other.

**Why that is general, and not a move that happens to help these two sentences.** The five
arguments of `sweep_index` are `(width, per, nth, member, phase)`. `width` is a *slot
candidate-list length* — a property of the segmentation's shape, not of any text. `per` is
`EMIT_PROFILE_RESERVE`, a constant. `nth` and `phase` are the traversal's own counters.
`member` is a position inside a shape class. **No target string, no IPA stream, no word
identity, no clue and no environment value reaches this function at all** — which is the
same conclusion the fence scan reaches mechanically (`HITS=0` on the environment-read
patterns), stated here from the call graph. And the rates are not fitted to any width: the
coprimality property is asserted over **every** span in `1..=150` and every member in
`0..=EMIT_PROFILE_MAX_DEEP`, so a rate cannot collide with the span it is used on for any
input, only for none. The change is therefore a statement about *how a shape class spends
coordinates*, and it helps **every** input whose deep slots have candidate lists that differ
in width and phase — which is the generic situation, not a property of `It's just a stupid
game` or of any of the other eleven targets. The evidence that it is not target-specific is
also in the numbers: pools moved on 12 of 12 targets, in both directions, with no target
special-cased; and the visible output did not move.

**Special cases I looked for and did not find.** I read the whole 189-line added diff. There
is no phrase-specific, clue-specific, sentence-specific or target-specific case in it: no
string literal at all, no word or substring comparison, no word list, no
`to_ascii_lowercase`, no `split_whitespace`, no comparison of any string to any other string
(the one `EQ_LIKE` hit is an integer index-vs-width bound). Nothing keys on the two
acceptance sentences, on their clues, on `hid`/`hits`/`dupe`/`came`/`wreck`/`beach` or on
any target string. The change is confined to `coverage_tuples` and `sweep_index` in
`src/lib.rs`; `axes::*`, `select_diverse`, `src/adjacency.rs`, the share cap and
`STRUCTURE_FLOOR` are untouched, and the cost accounting in `build` is untouched — this
change does **not** re-price anything, it only changes which coordinates are offered.

**The honest cost of it, which the mechanism predicts and the table confirms.** Continuity
is **coordinate-level, not tuple-level**: `rate(0, span) = 1`, so member 0's index is
byte-identical to the parent's and every coordinate the parent could place in the leading
slot is still placeable there. But the class now spends *k* widths rather than one, and the
legality check is per-member, so there exist `(widths, nth, phase)` where the parent emitted
the diagonal legally and the branch emits nothing. That is why `the cat sat on the mat`
loses 4-9 candidates. It is a real cost, it is bounded at this magnitude by the measurement,
and it is not offset by a milestone gain — the change is integrated as a general improvement
in its own right, on the evidence that it is general, fence-clean, guard-neutral and
pool-neutral-within-noise on 11 of 12 targets.

## 5. The residual I leave behind: the depth cap, not the cost accounting

**This fix is necessary and not sufficient for the item's target, and I am not touching the
territory that would make it sufficient.** The requested `hits justice dupe hid came` is a
**four-deep** shape. The coverage reserve **structurally never offers depth 4**, because
`EMIT_PROFILE_MAX_DEEP = 3` at **`src/lib.rs:212`** — and I verified that constant is
*unchanged* by this fix, which only *reads* it. The depth cap is reached before the
traversal draws any coordinate and before `build` is ever called, so no coordinate-level
change, however expressive, can reach a shape the reserve will not construct.

The cost accounting is **not** the residual: `e1a3f7` measured the requested shape's total
substitution cost at **0.919520** against a `total_budget` of **1.5**, i.e. `build` would
*accept* it if the reserve ever offered it, and measured the per-member sweep rates to be
**not more refusable** than the diagonals they replaced (deep refusal rate 26.12 % on the
parent vs 25.69 % here, 86.9 % of deep refusals now sitting at unrelated ranks). I take
that as given and did not re-derive it; it is why I make no cost claim.

**Two sibling fronts own that territory and are designing it right now** —
`madgab-depth4-b47d02` and `madgab-depthcap-c81e55`. `src/lib.rs` was in scope for me in
this front and I used it for nothing but this integration. **I did not touch the depth cap,
the cost accounting, the scoring or the ranking**, and no regression test was edited,
relaxed, re-baselined, skipped or deleted.

## 6. Artifacts, hygiene and durability

* On the branch: `src/lib.rs` (the integrated fix) and this report. Nothing else.
* **Not on the branch:** `examples/poolm.rs` is a measurement-only harness and is
  **untracked**; it is not committed and is not pushed. No `ZZ_*` / `zz_*` / `MADGAB_*`
  token, no `env::var`, no benchmark file and no `examples/zz_*.rs` probe is on the branch.
* No test file was touched by this front. No probe was added to `src/`.
* Nothing was merged to `main`. Nothing was merged to `post-milestone-acceptance` — the
  coordinator does that. No branch other than `madgab-int-79309a1` was written or pushed
  by this front, and no measurement arm (`scratch/2f7a10-base`, `zzparent`, `7c70784`,
  `madgab-pairing-2f7a10`, `scratch/2f7a10-cost`, `madgab-pairing-2f7a10-rebase-backup`)
  was merged into anything.
* Durability verified with `git ls-remote origin madgab-int-79309a1`, not with a local
  remote-tracking ref:

```
e5b1c2e158ad41ba7f85cbcc1031bec9b45477d0	refs/heads/madgab-int-79309a1   (integration commit)
```

with this report committed on top of it on the same branch. Read it with
`git show madgab-int-79309a1:REPORT-a3f19c.md`.

## 7. Blockers

**None.** The fix applied cleanly with no conflict and no `src/` drift; no guard regressed;
the one failing test is red on both arms. Reported, not fixed, as separate items rather than
as blockers here: the pool's run-to-run non-reproducibility (`w-6b91d3`, hit independently
by this front and by `b47d02`), and the `the cat sat on the mat` breadth loss of 4-9
candidates, which is inside this fix's measured envelope and is the expected
coordinate-level-not-tuple-level cost of the mechanism rather than a defect to repair in
this front.
