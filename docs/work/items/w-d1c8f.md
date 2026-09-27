---
work_item: true
id: w-d1c8f
state: done
priority: high
owner: agent-d1c8f1 (claimed 2026-09-27T04:53Z by coord-4e21 from post-milestone-acceptance 4b51dce)
updated: 2026-09-27T07:05:00Z
opened_by: coord-4e21 (reconciliation pass 2026-09-27T04:52Z-04:55Z)
branch: madgab-review-d6c-d1c8f
worktree: /workspace/madgab-review-d6c
reviews: madgab-d6c-9d4e10 (36589f8..4f3442c) — the delivered w-9d4e10 D6c implementation
diff_under_review: 36589f8..4f3442c
verdict: pass-with-follow-up
---

# Adversarial review of the D6c per-phone `SIMILARITY` change (4f3442c)

## VERDICT: **pass-with-follow-up**

`4f3442c` **may be integrated into `post-milestone-acceptance`.** The change
is the specification, applied literally and arithmetically exactly; the
canonical case-1 non-regression is not merely claimed but independently
measured and holds; the fence sweep is unchanged from the audited tree; and
every fence I was asked to check is intact. Five follow-ups are recorded below.
**None of them is a blocker and none requires a further implementation front
before landing** — F1 is a documentation/number-bookkeeping obligation and
F2 is a small test-coverage suggestion for whoever next touches `src/lib.rs`.

**A process note, and a correction to my own first pass on it.** My review
worktree is based at `4f3442c`, which is an ancestor of nothing on the
accumulation branch, so **`docs/work/items/w-d1c8f.md` was not present in this
worktree when the agent started**, and I initially recorded that as a missing
item. That was wrong, and the reason is worth stating because it will recur:
the item *was* opened, at `2d19836` on `post-milestone-acceptance`, and this
review branch (`4f3442c`, off `36589f8`) simply does not contain that commit.
**The item exists, with all eight questions, at
`origin/post-milestone-acceptance:docs/work/items/w-d1c8f.md`.** All eight are
answered below, including Q8, which the dispatch text did not spell out. The
practical lesson for the coordinator: a review worktree based at the *reviewed
tip* cannot see the item that dispatched it, so the agent must fetch the
accumulation branch to find its own instructions. I read them only after the
substantive work was done, and re-checked my findings against all eight
questions; nothing in the verdict changed.

## Answer index — the item's eight questions

| # | question (as opened at `2d19836`) | answered in | verdict |
|---|---|---|---|
| 1 | Is the deleted test's removal a legitimate replacement or a re-baseline in disguise? | *Q1 — The deleted test* | **legitimate replacement of an obsolete specification**, not a re-baseline. Proven in both directions by experiment. |
| 2 | Is `SIMILARITY_COST_PER_PHONE = 0.30` used as specified, with no re-sweep or re-tuning smuggled in? | *The remaining verifications → the constant*; *the arithmetic* | **as specified.** One constant change in the whole diff, all eight weights unchanged, character-for-character match to the spec. |
| 3 | Are the three enumeration-side `SIMILARITY_PER_WORD` proxies really untouched, and is leaving them defensible? | *The remaining verifications → the three proxies* | **untouched at src/lib.rs:954, :1065, :3340; leaving them is defensible** on the measured test that catches it, which I ran by name and which is green. |
| 4 | Is anything else in the diff? Account for every hunk. | *Accounting for `git diff 36589f8..4f3442c`, hunk by hunk* | **nothing else.** Ten hunks across four files, every one accounted for; 241 lines in `src/lib.rs` is comment rewriting, not logic. |
| 5 | Was `approximate_output_is_locked` re-locked against measured new behaviour, and is the w-4e2b19 contradiction gone? | *The remaining verifications → the re-lock* | **re-locked against measured behaviour, not re-baselined.** The block is recomputable from the constant and explains its own two unmoved entries. |
| 6 | Re-run and classify the fence sweep; is the hit set byte-identical to `42ced98`? | *Q3 — The fence sweep* | **byte-identical, confirmed** by SHA-256 of the normalised hit set. 49 hits, none in a decision. One counting error in the implementer's report. |
| 7 | Canonical case 1 must not regress — measure it yourself. | *Q2 — Canonical case 1* | **not regressed.** Measured through the shipped release executable: rank 27, 0.920, against a self-built rank-28 / 0.918 baseline. |
| 8 | Are the unmeasured predictions a legitimate deferral or a blocker? | *Q8 — The unmeasured predictions* | **legitimate deferral, not a blocker.** I measured them anyway: both cutoffs confirm exactly, and the pool size does not — which is F1. |

Two findings beyond the eight questions, both integration-critical, are in
*Two findings the dispatch did not ask for*: **the item's own
"cherry-pick `4f3442c` alone" advice is wrong and I proved it**, and a
test-merge of the branch into the current `post-milestone-acceptance` is clean
and validates at 9/0 on the fence test that w-3a7f0d extended after this
branch's base.

---

## Method and toolchain

`export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"` in every shell
(`git` 2.54.0, `grep`, `sed`, `awk`, `date` all present once exported).
`CARGO_TARGET_DIR` outside `/tmp` throughout (`/tmp` is `noexec`):
`/workspace/cargo-target-review-d6c` for the tree under review and
`/workspace/cargo-target-review-d6c-base` for a from-scratch baseline build of
`36589f8`. Release build throughout.

**This was read-only with respect to `src/` and `tests/`.** Nothing in
`/workspace/madgab-review-d6c` was modified — `git status` is clean and
`git diff HEAD -- src tests` is empty. Three disposable worktrees were created
outside the review tree purely to build a baseline and to reproduce the
implementer's red-before/green-after claims; all three and their `scratch/*`
branches were removed, and their target directories deleted. Agent
`1f6c41`'s `madgab-baseline-1f6c40` worktree and branch were not touched
(still `a5bcce2`) and no contention with it occurred.

**`cargo fmt` and `cargo clippy` do not exist on this host and were not run.**
Confirmed directly rather than assumed: `cargo fmt --version` → `error: no
such command: fmt`, `cargo clippy --version` → `error: no such command:
clippy`, and `command -v rustfmt clippy-driver rustdoc` finds none of them.
There is no `rustup`; the toolchain was built from a source tarball
(`docs/environment-notes.md`). `cargo test --doc` therefore cannot run either,
and no `wasm32-unknown-unknown` build is checkable. These are stated as
**unavailable**, never as passing.

---

## Q1 — Is deleting `similarity_is_scored_per_word_not_per_candidate` a
## legitimate replacement of an obsolete specification, or a re-baseline in
## disguise?

**It is a legitimate replacement of an obsolete specification. It is not a
re-baseline in disguise.** I reached that on my own reading of both tests plus
a direct experiment, not on the implementer's justification.

### What the two tests actually say

The deleted test's core claim was that `combined` is **flat in word count at
equal *per-word* cost**: `combined(n, cost) == combined(2, cost)` for
`n ∈ {3,4,6,8}`. Its own doc comment framed this as the deliberate removal of
the *pre*-per-word bug ("It charged a long clue once for a per-word property").
In other words the test asserted that the axis divides total cost by the
**word count**.

The specification D6c replaces that divisor with the **target's phone count**.
Under the specified formula, `n` words each at per-word cost `c` carry a total
cost of `n·c` and are charged for all of it, so `combined(n, c)` is *not* flat
in `n` and cannot be. The deleted assertion is the literal negation of the
change the item was opened to build. A test that asserts the negation of the
specification it ships with is obsolete by construction, not by preference.

### The experiment

I restored the deleted test verbatim into the `4f3442c` tree (a disposable
worktree, since this front is read-only) and ran it. It is **red**, with
precisely the message the implementer quoted:

```
test tests::similarity_is_scored_per_word_not_per_candidate ... FAILED
3 words at 0.05 per word scored 0.5980468750000001
  against 0.5986979166666667 for 2 words at the same per-word cost
```

That is not a test that "went red for an unrelated reason" — it goes red on
the axis arithmetic and nothing else.

### Why it is not a re-baseline

Three positive properties separate this from a disguised re-baseline, all
checked by me:

1. **The replacement is red-before / green-after, and the deleted test was
   not.** I transplanted the *new* test into the `36589f8` tree — adding only
   the `Metrics.similarity` field needed to observe the axis, leaving the
   per-word formula in place. It is **red** there, with exactly the quoted
   message:
   `the cut into words changed SIMILARITY at equal per-phone cost:
   0.8 over 3 words vs 0.9 over 6 words`, and it is **green** at `4f3442c`.
   A re-baseline is a test that accepts whatever the code now does; this pair
   is a specification that discriminates between the two readings, and I
   verified the discrimination in both directions independently.
2. **The replacement is strictly stronger on the property that matters.** The
   old test asserted flatness at equal *per-word* cost. The new one asserts
   flatness in the **cut** at equal *total* cost (three eight-phone words vs
   six four-phone words over the same 24 phones) — a property the old code
   **cannot** satisfy and the new code must. It also keeps the old test's
   surviving ranking claim in stronger form: it pins the axis maximum
   (`similarity == 1.0` at zero cost) and strict monotonicity
   (`m_worse < m_wide`) at equal word count.
3. **The surviving claims of the old test were carried over, not dropped.**
   The old test's ranking assertions (`worse per-word cost ranks strictly
   below`; `the axis stopped responding to quality`) are re-asserted on the
   corrected axis. The old test's *other* claim — "spreading one total cost
   over more words was penalised" — is obsolete in exactly the same way as the
   main loop and is correctly not carried over.

Nothing was relaxed: the deleted test's tolerance was `1e-12` on a `combined`
difference, and the replacement uses exact `assert_eq!` on the axis. Nothing
was renamed in place, nothing was `#[ignore]`d (`git grep -n '#\[ignore' --
src tests` returns nothing), and the lib count is 58 before and 58 after
because one test was removed and one added plus the refold helper's
mechanical update.

### Two honest limits, neither a spec loss, both worth recording

- **Coverage narrowed slightly, in a place that is still covered elsewhere.**
  The old test asserted through `combined`, i.e. end-to-end through the
  objective; the new one asserts the `Metrics.similarity` field in isolation.
  The wiring of the axis into `combined` is *not* orphaned:
  `incremental_aggregates_match_a_full_refold` re-derives `combined` from the
  eight axes including the per-phone `similarity`, and `Metrics` derives
  `PartialEq`, so the axis's contribution to `combined` is still pinned — and
  the implementer correctly moved that test's local `refold` helper to the
  per-phone formula, which is necessary for it to keep meaning anything.
- **The axis's clamp at `0.0` is no longer pinned by any test.** The old test
  asserted `combined(4, 0.40) > combined(4, 4.0)` — "a clue whose words cost a
  full unit each should reach the axis's floor". The replacement asserts the
  **ceiling** (`== 1.0`) but not the **floor**. This is the one genuine,
  small coverage regression, and it is follow-up F2.

---

## Q2 — Canonical case 1, measured myself through the shipped release
## executable

I built the release binary and ran it. I did not trust the implementer's
numbers and I did not reuse their target directory.

```sh
export PATH="$GUIX_PROFILE/bin:$HOME/.local/bin:$PATH"
export CARGO_TARGET_DIR=/workspace/cargo-target-review-d6c
cargo build --release
$CARGO_TARGET_DIR/release/madgab --approximate --top 50 "recognize speech"
```

**The implementer's claim is CONFIRMED.**

| | result at `4f3442c` | claimed |
|---|---|---|
| `wreck a nice beach` in the printed 50? | **yes** | yes |
| printed rank | **27** | 27 |
| printed score (3 dp, as the CLI prints) | **0.920** | 0.920 |
| printed 50 word-count classes | **`{4: 50}`** | `{4: 50}` |
| proposals printed | 50 | 50 |

It sits at line 27 of the CLI's own numbered output:
`27. [0.920] wreck a nice beach`.

### The "not regressed from rank 28" claim, checked against a baseline I built

I also built `36589f8` from scratch in a separate target directory and ran the
same command. **`wreck a nice beach` is at rank 28, score 0.918,
distribution `{4: 50}`.** So it moved **28 → 27**: **not regressed**, and
marginally *improved* by one rank, exactly as claimed.

### Exact score, through the public boundary

The CLI prints 3 decimals, so I took the 6-decimal figure through the public
`generate_pool` on both trees (disposable worktrees, since this front cannot
touch `tests/`):

| | `36589f8` | `4f3442c` | w-5c11a2 **predicted** |
|---|---|---|---|
| `wreck a nice beach` score | 0.9183133827599373 | **0.9199502875218423** | 0.918313 → **0.919950** |

The prediction is reproduced **exactly**. Independently, my `36589f8` baseline
reproduces w-5c11a2's recorded 0.918313383 to 9 decimals, which is a check
that my instrument and my baseline build are the ones w-5c11a2 measured.

### Word-count-class distribution of the printed 50, both canonical cases,
### before and after

Every cell of this table is my own measurement, run through the shipped
executable, on a baseline I built myself. **All seven targets match the
implementer's table exactly, in every cell.**

| target | before `36589f8` | after `4f3442c` |
|---|---|---|
| `recognize speech` (canonical case 1) | `{4: 50}` | `{4: 50}` |
| `It's just a stupid game` (canonical case 2) | `{6: 50}` | **`{5: 6, 6: 44}`** |
| `a sturdy green cardigan` | `{7: 50}` | **`{6: 2, 7: 48}`** |
| `we should probably leave before midnight` | `{10: 50}` | `{10: 50}` |
| `she borrowed another excellent hat` | `{8: 1, 9: 1, 10: 48}` | `{8: 1, 9: 3, 10: 46}` |
| `the morning newspaper arrived late` | `{8: 25, 9: 25}` | `{8: 41, 9: 9}` |
| `please pour some cold water` | `{6: 50}` | `{6: 50}` |

**Canonical case 2's printed 50 is `{5: 6, 6: 44}`: the single-length
monopoly breaks.** That is the general property the design buys, and it is
real. The specification predicted `{5: 5, 6: 45}`; the implementer's
off-by-one disclosure of that drift is honest and is the expected kind of
drift when the design is priced on a different tree.

**The honest limit the implementer stated is real and I confirm it:** the
monopoly does *not* break on every short target. Two of the five neutral
targets are still a single class after the change
(`we should probably leave before midnight` `{10: 50}`, `please pour some
cold water` `{6: 50}`). D6c removes the span-length term in `SIMILARITY`; it
does not touch `RHYTHM`, whose degenerate band w-5c11a2 named as the separate
structural reason. The implementer did not oversell this.

### Cutoffs and the pool, which the implementer did **not** measure

The implementer recorded the two 6-decimal cutoff predictions as *not
verified*, because measuring them exactly would have required adding an
instrument the item forbids. I discharged that gap, because w-5c11a2's own
specification **explicitly instructed the builder** to re-measure the pool
"rather than trusting the 'pool size is unchanged' line above it", and that
instruction was not carried out. Through the public `generate_pool` at default
settings on both trees:

| quantity | `36589f8` | `4f3442c` | delta | w-5c11a2 predicted |
|---|---|---|---|---|
| case-1 pool size | 18,242 | **18,270** | **+28** | "unchanged by construction" |
| case-1 score-ordered 50th | 0.917025536 | **0.918796441** | +0.001771 | 0.917026 → 0.918796 (+0.001771) |
| case-1 `wreck` score | 0.918313383 | **0.919950288** | +0.001637 | 0.918313 → 0.919950 |
| case-2 pool size | 18,917 | **18,936** | **+19** | "unchanged by construction" |
| case-2 score-ordered 50th | 0.917129199 | **0.915121574** | −0.002008 | 0.917129 → 0.915122 (−0.002007) |

Two results, and the second is the substantive one:

- **Every predicted score figure reproduces to six decimals.** The
  implemented design is arithmetically the specified design.
- **The pool size is NOT unchanged: it grew by 28 (case 1) and 19 (case 2).**
  w-5c11a2's "unchanged by construction" reasoning was wrong — a change to
  the score is *not* a pure re-ordering, because the enumeration-side proxies
  the item left on the old scale now rank differently, which changes which
  candidates survive. w-5c11a2 predicted this risk in prose and warned the
  builder about it in exactly these words; the builder did not measure it.
  This is follow-up **F1**. It is not a defect in the change — the property
  bought is unaffected and every acceptance criterion still holds — but any
  later figure that assumes an 18,917 / 18,242 pool is now wrong by that much,
  including w-5c11a2's hypothetical ranks 9,249 and 9,661.

*A methodological warning for whoever reads this next, because it nearly
misled me:* my first pool measurement reused one `CARGO_TARGET_DIR` for two
different worktrees and returned **bit-identical** pre-D6c scores on the
post-D6c tree. It was a stale-artifact problem, not a code problem. Any
cross-tree comparison here needs a **separate target directory per tree**, and
a suspiciously exact equality across a behaviour-changing diff is a reason to
re-measure, not to conclude.

---

## Q3 — The fence sweep, re-run and classified by me

```sh
git grep -i -n -E "wreck|beach|recognize|justice|stupid|dupe|came|hid" -- src web examples
```

**49 hits.** I ran the same sweep at `42ced98` and compared. The hit set is
**byte-identical** to w-d5a2c1's at `42ced98`, modulo line-number shifts: I
normalised away the line numbers and the `42ced98:` prefix and compared with
`cmp`, and the two normalised files have the same SHA-256
(`f0f21e861e807f828f52ed36ac5d12a69168fdf2dce01835823f07dca4a33cb9`).
**The claim is CONFIRMED**: this change added no hit and removed none.

**One correction to the implementer's bookkeeping.** They reported
"src — 30 hits, web — 9 hits, examples — 3 hits" (42 total). The actual
counts are **src 37, web 9, examples 3, total 49**. Their "byte-identical"
claim is nonetheless correct, and the discrepancy is a counting slip in the
report, not a hidden hit. This is follow-up F3.

### Classification of all 49 hits, by category

I checked by eye as well as by the test, because `no_phrase_hard_coding.rs`
strips comments and cannot see an example-naming doc comment.

| file | hits | classification |
|---|---|---|
| `src/lexical.rs` | 4 | 1 doc comment (L19, contrasts `beach` with a determiner); 3 in the `#[cfg(test)] mod tests` exemplar lists (L302, L303, L335) — negative assertions, pre-existing and untouched |
| `src/lib.rs` | 31 | **2 real code**, both `expect()` panic prose carrying an ordinary English `came`: L3091 `"key came from cells"`, L3949 `"candidate came from this pool"`. 2 comments (L3856 quotes a target while explaining a split; L5038 is a doc comment matching `came` inside `became`). The remaining **27 are all in `#[cfg(test)]`** (L4144–5583): corpus-rarity contrasts, spelling-normalisation, `phrase_signature`, `TargetPhrase` reuse and stem cases, `reachability_corpus()`, and the reachability budget walk |
| `src/main.rs` | 2 | doc comment — the CLI's own usage examples |
| `web/app.js` | 5 | the DOM property `hidden` — a substring match on `hid`, not a phrase. `app.js` *is* a search entry point, but all five are show/hide of the results and error elements |
| `web/index.html` | 4 | 2 UI default input value / placeholder (not read by the search); 2 real code, the `hidden` attribute |
| `examples/measure.rs` | 3 | 1 doc comment; 2 benchmark-harness input list, a separate binary's inputs and not a library input |

**There is no phrase-specific hard-coding for any of these examples anywhere.**
No phrase literal in `src/` participates in a decision. The only real-code
hits in the whole sweep are two English `came`s in panic messages and a
`hidden` DOM property, none of which is a clue, a target, or a branch
condition. The new test text introduces no hit: the synthetic words are
`w0`, `w1`, …, the IPA string is `aeiouy`, the target is `alpha bravo charlie
delta` and the neutral target is `a sturdy green cardigan` — none matches the
pattern, which is exactly what the byte-identical comparison shows
independently.

`tests/no_phrase_hard_coding.rs` is **7 passed; 0 failed** (see the table
below), but as noted that is the weaker of the two checks and I did not rely
on it alone.

---

## The remaining verifications

### The constant is the specified one, from the swept band, not re-tuned

`axes::SIMILARITY_COST_PER_PHONE: f64 = 0.30` — exactly the value w-5c11a2
specified and the value its swept safe band `[0.15, 0.35]` recommended.

**Not re-swept and not re-tuned**, established structurally rather than by
trust: the diff contains **exactly one constant change**, and **all eight axis
weights are unchanged** — `SIMILARITY 0.25`, `NOVELTY 0.15`,
`WORD_NOVELTY 0.15`, `FAMILIARITY 0.10`, `RHYTHM 0.30`, `SHAPE 0.05`,
`CLOSED_CLASS`, `PUNCH 0.10`. There is no second candidate value anywhere in
the branch, so no sweep was performed and the safe band was not walked.

### The arithmetic matches w-5c11a2's implementation-ready specification
### exactly

w-5c11a2's *Exact lines* block specifies, character for character:

```rust
let cost_per_phone = self.sub_cost_total / total_len.max(1) as f64;
let similarity =
    (1.0 - cost_per_phone / axes::SIMILARITY_COST_PER_PHONE).clamp(0.0, 1.0);
```

and `pub const SIMILARITY_COST_PER_PHONE: f64 = 0.30;`. That is what landed,
with no signature change (`total_len` was already a parameter). And as Q2
shows, the *behavioural* arithmetic reproduces every one of w-5c11a2's
predicted figures to six decimals. Both the literal and the numeric check pass.

### The three enumeration-side `SIMILARITY_PER_WORD` proxies are genuinely
### untouched, and leaving them is defensible

Genuinely untouched — none of the three appears in the diff, and
`axes::SIMILARITY_PER_WORD = SIMILARITY / 4.0` is unchanged:

- `SlotAlt::contribution` — **src/lib.rs:954**
- `quality` in `generate_approximate` — **src/lib.rs:1065**
- `partial_span_score` — **src/lib.rs:3340** (body confirmed still on the
  per-word scale)

Leaving them is defensible, on evidence rather than assertion:
[w-9c6f2b](w-9c6f2b.md) measured that re-deriving all three on the correct
marginal **loses** `approximate_pool_reaches_matches_deep_in_a_span`, and I
ran that test by name on `4f3442c`: **ok**. The divergence is documented
in-code with the correct marginal (`-SIMILARITY * weight / (phones *
SIMILARITY_COST_PER_PHONE)`) and the reason it is being left to the front that
owns enumeration. Scope discipline held: the builder did not smuggle an
enumeration change into an objective front.

### `approximate_output_is_locked` was re-locked against measured new
### behaviour, not re-baselined to whatever the code printed

The distinguishing evidence is that the new comment block is **checkable
independently of the printout**, which a re-baseline cannot be:

- it states an exact, derived per-axis bound — `SIMILARITY * cost / 4.5`, from
  `1/3 - 1/1.8` where `1.8` is six phones × `0.30` — that a reviewer can
  recompute from the constant alone;
- it names **two entries that did not move and explains why** (their total
  edit cost exceeds both normalisers, so the axis clamps to `0.0` on either
  denominator). A re-baseline has nothing to explain, because under a
  re-baseline everything simply moves to whatever printed;
- it **discloses the churn as a price rather than an improvement** — "nine of
  the ten phrases are different, eight of the ten scores moved" — and
  explicitly disclaims "neither a regression nor an improvement claim";
- the w-4e2b19 self-contradiction is gone, and the whole block was replaced
  with measured text for the new axis rather than left to describe the old
  one.

This is the standard the lock comment exists to meet, and it meets it.

### No cap or emission site was touched

`git diff 36589f8..4f3442c -- src/lib.rs | grep -E 'cap|affordable_opening_width|LEXICAL_HEAP_POP_LIMIT'`
on added lines returns **nothing**. Both fenced sites are still at the exact
line numbers the fences cite — `src/lib.rs:1877` (`affordable_opening_width(depth,
LEXICAL_HEAP_POP_LIMIT)`) and `src/lib.rs:1955` (`slots[k].len().min(cap)`) —
so the emission blocker priced in w-7c1f64 and w-5c11a2 is untouched and
still binding.

### No budget was widened

`total_budget` default `1.5` is unchanged, and every gate that reads it
(src/lib.rs:858, 1406, 1770, 5457, 5559) is absent from the diff. No widened
run is reported as production behaviour.

### No `zz*` file, no `eprintln!` probe, no `env::var` knob, no `#[ignore]`

```text
git ls-files | grep -E '(^|/)zz'        -> no output
git grep -n '#\[ignore' -- src tests    -> no output
git grep -n 'env::var' -- src           -> no output
git grep -n -E 'eprintln!|dbg!' -- src  -> src/main.rs only (pre-existing CLI
                                          error/progress output), and
                                          src/main.rs is not in the diff at all
```

Every test run below reports `0 ignored`.

### Clean build

A forced full release rebuild emits **0 warnings**. In particular the new
`Metrics.similarity` field — read only by tests — produces no `dead_code`
diagnostic.

---

## Validation, exactly as run, on `madgab-review-d6c-d1c8f` at `4f3442c`

| command | result |
|---|---|
| `cargo test --release --lib` | **ok. 58 passed; 0 failed; 0 ignored** |
| `cargo test --release --test corpus_integration` | **FAILED. 12 passed; 1 failed; 0 ignored** — `approximate_finds_classic_madgab_resegmentation` only |
| `cargo test --release --test exact_determinism` | **ok. 1 passed; 0 failed; 0 ignored** |
| `cargo test --release --test approx_determinism` | **ok. 4 passed; 0 failed; 0 ignored** (incl. the re-locked `approximate_output_is_locked`) |
| `cargo test --release --test emit_coverage` | **ok. 4 passed; 0 failed; 0 ignored** (was 3; +1 is the new regression test) |
| `cargo test --release --test no_phrase_hard_coding` | **ok. 7 passed; 0 failed; 0 ignored** |

**Every count matches the implementer's table exactly.** Named tests checked
individually, not only by count:

- `approximate_finds_recognize_speech_resegmentation` — **ok** (canonical
  case 1 is green)
- `approximate_finds_classic_madgab_resegmentation` — **FAILED**, and this is
  the correct end state: the binding blocker is emission (`cap = 7` against a
  required slot index of 99), upstream of any objective value. The design was
  never expected to reach this alignment (`-0.094351`) and nothing here was
  tuned to try.
- `approximate_pool_reaches_matches_deep_in_a_span` — **ok**, the test
  w-5c11a2 named as the risk of leaving the enumeration proxies alone
- `the_other_canonical_resegmentation_is_still_proposed` — **ok**
- `a_short_multi_syllable_proposal_set_is_not_one_word_count_class` — **ok**

`cargo fmt` and `cargo clippy` **do not exist on this host and were not run**;
neither does `rustdoc`, so no doctest could run. These are stated as
unavailable and must not be reported as passing by the coordinator on landing.

---

## Q8 — The unmeasured predictions: legitimate deferral, or an untested claim
## that should block?

**The deferral was legitimate as stated, and it did not need to block. But I
measured the deferred quantities anyway, and the measurement found something
the deferral was hiding — so the deferral was right about the method and
wrong about the risk.**

### Why the deferral was legitimate

The agent declined to verify three 6-decimal figures (case-2 cutoff
`-0.002007`, case-1 cutoff `+0.001771`, the alignment's exact residual gap)
on the grounds that the CLI prints 3 decimals and resolving them exactly would
require adding a measurement instrument the item forbids. That reasoning is
sound and it is the reasoning this project has been burned by before: the
instrument goes in the crate, the number becomes unreproducible, and the next
pass re-derives it. Recording "not measured" instead of "confirmed" was the
correct discipline, and Q2's measurement shows it was not covering for an
error — every one of the two cutoff figures the agent *could* have printed at
3 decimals is exactly right.

The claim about the alignment is also correctly scoped: the design was never
expected to reach canonical case 2, so its absence is not a defect, and the
acceptance test being red is the correct end state.

### What measuring it anyway turned up

I discharged the gap through the public `generate_pool` on a disposable
worktree (Q2's table). The two cutoff predictions reproduce **exactly**:
case-1 score-ordered 50th `0.917025536 → 0.918796441` (predicted `+0.001771`,
actual `+0.001771`) and case-2 `0.917129199 → 0.915121574` (predicted
`-0.002007`, actual `-0.002008`). Those are not untested claims; they are
confirmed claims the implementer under-claimed.

**But the pool size, which w-5c11a2 tied to those same predictions, is wrong,
and the implementer did not measure it.** See F1 below. w-5c11a2's
specification said in terms that the builder "must … re-measure the pool
rather than trusting the 'pool size is unchanged' line above it", and the
builder did not. This is the one place where the deferral left a real gap, and
it is a gap in *w-5c11a2's* prediction rather than in the change.

**Verdict on Q8: legitimate deferral, correctly disclosed, not a blocker.**
The change stands on the measured acceptance criteria. The residual risk it
created is documentary and is F1.

---

## Accounting for `git diff 36589f8..4f3442c`, hunk by hunk

`src/lib.rs` shows 137 insertions and 104 deletions — the 241 the dispatch
flagged as a lot for one formula. It is a lot, and **all of it is accounted
for**: six hunks, and the bulk is comment rewriting, not logic.

| # | file | hunk | lines | what it is |
|---|---|---|---|---|
| 1 | `src/lib.rs` | `@2587,37 +2587,45` | 30/26 | The `metrics` doc comment rewritten for the per-phone reading, **plus the two-line formula change itself** (`cost_per_word`/`SIMILARITY_COST_PER_WORD` → `cost_per_phone`/`SIMILARITY_COST_PER_PHONE`). The comment rewrite is the reason the hunk looks large; only 4 lines are code. |
| 2 | `src/lib.rs` | `@2700,6 +2708,7` | 1/0 | `Metrics { combined,` gains `similarity,`. The one structural addition. |
| 3 | `src/lib.rs` | `@2776,14 +2785,20` | 11/8 | `SIMILARITY_COST_PER_WORD: f64 = 1.0` → `SIMILARITY_COST_PER_PHONE: f64 = 0.30`, with the constant's doc comment updated and the standing note about `SIMILARITY_PER_WORD` kept. |
| 4 | `src/lib.rs` | `@2796,6 +2811,10` | 4/0 | The `similarity: f64` field declared on `struct Metrics` with a comment saying why it exists (so a test can assert the axis without the seven word-count-dependent terms). |
| 5 | `src/lib.rs` | `@4332,15 +4351,16` and `@4386,6 +4406,7` and `@4415,7 +4436,7` | 9/6 | `incremental_aggregates_match_a_full_refold`: its local `refold` helper re-derives `metrics`, so it was moved to the per-phone formula and given the `total_len` parameter. **Necessary, not optional** — leaving it on the per-word formula would have made the test assert a different axis than the one in production. |
| 6 | `src/lib.rs` | `@5563,78 +5584,6` | 0/72 | The deletion of `similarity_is_scored_per_word_not_per_candidate`. Q1. |
| 7 | `src/lib.rs` | `@5701,4 +5650,88` | 84/0 | The `synthetic_clue` helper and `similarity_is_charged_per_phone_not_per_word`. Q1. |
| 8 | `tests/corpus_integration.rs` | `@554,42 +554,47` | 33/28 | The `approximate_output_is_locked` comment block replaced with measured new text and the expected table updated. The **only** test whose expectations changed. |
| 9 | `tests/emit_coverage.rs` | `@172,3 +172,37` | 34/0 | The new `a_short_multi_syllable_proposal_set_is_not_one_word_count_class`. |
| 10 | `docs/work/items/w-9d4e10.md` | — | 386/16 | The implementer's own report. Documentation. |

**Hunks 1, 3 and 5 are the change. Hunks 2, 4 and 7 are the test seam for it.
Hunks 6 and 8 are the test deletion and the re-lock, judged in Q1 and below.
Hunk 9 is the second required test. Hunk 10 is documentation.** There is
nothing unaccounted for, and in particular there is no hunk touching `PUNCH`,
`RHYTHM`, any weight, any budget, or either emission site.

---

## Two findings the dispatch did not ask for, both integration-critical

### The item's own cherry-pick advice is wrong, and following it wastes a pass
### and creates pressure toward a re-baseline

The item's handoff says the next coordinator can integrate "with a merge or a
**cherry-pick of `4f3442c` alone**, since `4679cf9` is a coordinator durability
snapshot and not part of the reviewed change."

**That is not true, and I tested it.** `4f3442c`'s own stat is:

```text
 docs/work/items/w-9d4e10.md | 402 +++++++++++++++++++++++++++++++++--
 src/lib.rs                  |  72 --------
 tests/corpus_integration.rs |  61 ++++---
```

— `src/lib.rs` is **deletions only**, and `tests/emit_coverage.rs` is **absent**.
The actual per-phone formula, the `Metrics.similarity` field, the new unit test
and the new integration test all live in **`4679cf9`**:

```text
4679cf9  src/lib.rs | 169 +++++++++++++++++++++++++++++-----------
         tests/emit_coverage.rs | 34 ++++++++
```

I cherry-picked `4f3442c` alone onto `36589f8` and measured the result:
`SIMILARITY_COST_PER_PHONE` appears **0 times** — the tree is still
per-`word` — and `emit_coverage` runs **3** tests, not 4. The build is clean
and `--lib` passes 57/0, so it is not a loud compile failure; it is a tree that
looks nearly right and is not. `approximate_output_is_locked` does fail loudly
(`left` = the old per-word table, `right` = the new table), which is the one
thing standing between a coordinator and a re-baseline of a test the item
explicitly forbids re-baselining.

**Recommendation: merge `madgab-d6c-9d4e10` (the branch tip, which contains
both commits), or cherry-pick `4679cf9` *then* `4f3442c`. Do not cherry-pick
`4f3442c` alone.** The implementer's own request to "squash `4679cf9` into the
tip on landing" is the right instinct for a squash-merge and the wrong
instinct for a selective cherry-pick.

### The merge is clean and I validated the integrated tree end to end

`post-milestone-acceptance` has moved well past this change's base, so I
test-merged in a disposable detached worktree (no branch, nothing pushed, no
effect on `post-milestone-acceptance` or `main`):

- the **only** conflict is `docs/work/items/w-9d4e10.md` — a documentation
  conflict, resolvable by taking either side;
- `src/lib.rs`, `tests/corpus_integration.rs` and `tests/emit_coverage.rs` all
  merge **cleanly and automatically**;
- on the merged tree: `--lib` **58/0**, `corpus_integration` **12 passed / 1
  failed** (only `approximate_finds_classic_madgab_resegmentation`, the
  expected red), `exact_determinism` **1/0**, `approx_determinism` **4/0**,
  `emit_coverage` **4/0**;
- **`no_phrase_hard_coding` is 9 passed, not 7.** This matters: [w-3a7f0d]
  extended that fence to `web/` and `examples/` at `2d8449c`, *after* the
  `36589f8` base this branch forked from, so the merged tree inherits the
  stronger fence. **It passes.** The extended fence therefore confirms, on the
  integrated tree, that the D6c code adds no phrase-specific hard-coding in
  `web/` or `examples/` either — a check that did not exist when the
  implementer reported 7/7, and that I would not have run had I not test-merged.

### A small reporting error inherited from w-5c11a2, worth correcting

`approximate_output_is_locked` lives in **`tests/corpus_integration.rs`**, not
in `tests/approx_determinism.rs`. `w-5c11a2`, `w-9d4e10` and this review's
own dispatch text all say `--test approx_determinism … (incl. the re-locked
`approximate_output_is_locked`)`. Running that filter on `approx_determinism`
gives `running 0 tests … 4 filtered out`. The counts are all still right —
`approx_determinism` really is 4/0 and `corpus_integration` really is 12/1 —
but the parenthetical attribution is wrong, and it is how a reviewer ends up
believing the re-lock was checked on a test target that does not contain it.
The lock is verified, by running it where it actually lives.


- **F1 — record the real pool size.** The pool is **not** unchanged: 18,242 →
  **18,270** (case 1) and 18,917 → **18,936** (case 2), measured through
  `generate_pool` at default settings. w-5c11a2 predicted "unchanged by
  construction" and simultaneously instructed the builder to re-measure it;
  the builder disclosed not measuring it. Any downstream figure that assumes
  the old pool size — including w-5c11a2's hypothetical ranks 9,249 and 9,661
  — is stale by these amounts. This is a documentation obligation on landing,
  not a code change.
- **F2 — the similarity axis's `0.0` clamp is unpinned.** The deleted test
  asserted the axis reaches its floor; the replacement asserts its ceiling
  but not its floor. A one-line assertion in the existing new test would close
  it. Not worth a front on its own; fold into the next touch of `src/lib.rs`.
- **F3 — the fence-sweep counts in w-9d4e10 are wrong** (30/9/3 = 42; actual
  37/9/3 = 49). The "byte-identical to `42ced98`" claim itself is correct and
  I verified it independently. Correct the table when the item is closed.
- **F4 — merge the branch tip, or both commits. Do NOT cherry-pick `4f3442c`
  alone.** Proven above: `4f3442c` alone leaves the tree on the per-**word**
  formula with the new integration test missing, and puts a red re-lock in
  front of a coordinator who is under explicit instruction not to re-baseline
  a test. Merge `madgab-d6c-9d4e10` (tip `4f3442c`, containing `4679cf9` and
  `4f3442c`), or cherry-pick `4679cf9` then `4f3442c`. This supersedes F4 as
  previously drafted, which merely asked for `4679cf9` to be squashed — that
  is right for a squash-merge and wrong for a selective cherry-pick.
- **F5 — `w-9d4e10` is still `state: working` with its last checkbox unticked.**
  The checkbox is unticked by design because the agent was forbidden to push
  to `post-milestone-acceptance`. Set the state to `done` on landing, and
  record the real pool size from F1 in it.
- **F6 — correct the lock's test-file attribution in `w-5c11a2` and
  `w-9d4e10`.** `approximate_output_is_locked` is in
  `tests/corpus_integration.rs`, not `tests/approx_determinism.rs`. The counts
  are right; the attribution is not. Documentation only.

## Deliberately not opened, per instruction

No new measurement front. No re-derivation of the refutations already recorded
in [w-d3f7a1](w-d3f7a1.md) — the objective, cost, pronunciation, dictionary,
`GAP_COST`, retention and emission fronts all stay shut. The constant was not
re-swept. **Nothing in `src/` or `tests/` was modified on this review branch**
(`git diff HEAD -- src tests` is empty; the only file in the final commit is
this item). All scratch work — a `36589f8` baseline build, three disposable
worktrees for the red-before/green-after and cherry-pick experiments, and a
test-merge — was done in worktrees outside the review tree on `scratch/*`
branches or detached, and every one of them, its branch and its target
directory has been removed. `main` was not touched, and `4f3442c` was **not**
merged into `post-milestone-acceptance` by this agent; the test-merge was a
detached, uncommitted-to-any-branch rehearsal and integration remains the
coordinator's job. Agent `1f6c41`'s worktree and branch
(`madgab-baseline-1f6c40`) were not touched and no contention occurred.

## Next action for the coordinator

1. **Land `madgab-d6c-9d4e10` (tip `4f3442c`) into
   `post-milestone-acceptance`.** The verdict is pass-with-follow-up; nothing
   in F1–F6 must be done first, and none of them requires a new front. **Merge
   the branch, or cherry-pick `4679cf9` and then `4f3442c` — never `4f3442c`
   alone (F4).** I test-merged it: the only conflict is the
   `docs/work/items/w-9d4e10.md` documentation conflict; all three code files
   merge automatically; and the merged tree is 58/0, 12/1 (expected red),
   1/0, 4/0, 4/0, and 9/0 on the now-extended `no_phrase_hard_coding`.
2. **On landing, fold F1's pool numbers and F6's attribution fix into
   `w-5c11a2` and `w-9d4e10`,** and set `w-9d4e10` to `done`.
3. **Open no front for the D6c design itself.** It is built, measured, and
   correct. `w-5c11a2`'s priced negative stands: the canonical case-2 alignment
   is still out, the binding blocker is still emission at `src/lib.rs:1955`
   against `cap = 7` from `src/lib.rs:1877`, and no objective change reaches
   it.
4. **The two items D6c deliberately left are still open and still worth
   having:** the falsified `PUNCH` doc comment at `src/lib.rs:2673-2675`
   (it claims the axis "asks the clue to be as short-worded as the target is",
   which canonical case 1 — two words in, four words out — falsifies), and a
   read-only `Clue::axes()` accessor. The latter is the durable fix for the
   mechanism behind every prior reversal, and this review is fresh evidence for
   it: discharging Q2 and Q8 cost a full baseline build and a `generate_pool`
   harness that had to be written from `Clue`'s published fields, when a
   published eight-axis decomposition would have made both a one-line read.
5. **Fix the review-worktree base, not the code.** A review worktree based at
   the *reviewed tip* cannot see the work item that dispatched it, because the
   item lands on the accumulation branch, which is a descendant. This agent
   initially recorded its own item as missing. Give review fronts a base that
   contains the dispatch commit, or tell them in the item's own metadata to
   `git fetch origin post-milestone-acceptance` and read
   `docs/work/items/w-d1c8f.md` there before starting. It cost a wrong
   intermediate conclusion, which is exactly the failure mode this project keeps
   paying for.
