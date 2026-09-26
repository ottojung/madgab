# REPORT-3e7b04 — independent post-merge review of the pool-determinism fix `3d520c0`

Front `3e7b04`, branch `madgab-review-pool-3e7b04`, work item
[docs/work/items/w-3e7b04.md](docs/work/items/w-3e7b04.md), based on
`post-milestone-acceptance` at `6faf20d` (branch opened at `92a5b23`).
No merge, no rebase, nothing pushed to `main` or to
`post-milestone-acceptance`, no other branch touched. This branch pushes
exactly one commit, this file. `git diff 3d520c0..HEAD --stat` is
`docs/work/items/*.md` only (inherited from `6faf20d`, not from me);
`git diff 6faf20d..HEAD` after this commit is `REPORT-3e7b04.md` and
nothing else.

**MERGE RECOMMENDATION: ALREADY-MERGED-WITH-NOTES** — the fix is sound
and the determinism claim is true on the merged result; the notes are a
false claim in the merged test comment and in `REPORT-d3b7c2.md` §4, a
pool-membership movement materially larger than that report states, the
fact that the pool count is a function of `top_n`, the direction the
tie-break imposes, and a caveat about the strength of the new guard.

---

## 0. What I measured, and how

Commit measured: **`3d520c0`** (merge of `1bd005e` and `e3823d3`;
`3d520c0^` = `1bd005e`). The only code change is the one
`git diff 3d520c0^..3d520c0` shows: `src/lib.rs` and
`tests/approx_determinism.rs`.

`git diff 3d520c0..92a5b23 --stat` is `docs/work/items/w-2f7a10.md`,
`w-558697.md`, `w-5c8e1f.md`, `w-6b91d3.md` — documentation only. The
`src/` and `tests/` trees on this branch are byte-identical to `3d520c0`,
so the arm I measured is the branch.

Arms, all built `--release` under `/workspace` (`/tmp` is `noexec`):

| arm | how it was made | what it is |
|---|---|---|
| tip-clean | `git archive 3d520c0 \| tar -x -C /workspace/rev-3e7b04-clean` | suites + un-instrumented pool counts |
| tip-probe | same, plus the probe below | pool membership dumps |
| base-clean | `git archive 3d520c0^ \| tar -x -C /workspace/rev-3e7b04-baseclean` | base suite arm |
| base-probe | same, plus the **identical** probe | base pool membership dumps |

**Probe, declared.** The base arm has no `generate_with_pool`, and the
CLI's visible list is not the pool, so to answer Q5 I applied **one
untracked instrumentation hunk, byte-identical in both throwaway archive
arms**, immediately after the phrase-dedup `retain` in `finish` and
before `select_diverse`:

```rust
if std::env::var_os("REVIEW_POOL_DUMP").is_some() {
    eprintln!("POOLSIZE {}", clues.len());
    for c in &clues { eprintln!("POOLDUMP {}", c.phrase.to_lowercase()); }
}
```

It is a **dump switch**: the variable's *presence* is tested, its *value*
is never read, and nothing in `src/` branches on it. It is the same
shape as the pre-existing `MADGAB_TRACE_PHRASES` probe (which I also used
for the head-count tables, with a neutral non-word value, so the trace
reports the pool count rather than a rank). The hunk exists **only** in
`/workspace/rev-3e7b04-tip` and `/workspace/rev-3e7b04-base`, which are
`git archive` extractions, not checkouts and not my branch. No `src/`
file, no `tests/` file and no constant on `madgab-review-pool-3e7b04` was
touched; the Q3/Q4 head counts come from the **un-instrumented** tip-clean
binary, and the probe agrees with it wherever both were run (e.g.
`recognize speech` 15908 at `--top 50` from both paths). No
phrase-specific, word-specific, substring-specific, dictionary-lookup or
environment-value case appears anywhere, in the probe or in the report's
scripts.

**Wall clock.** Started 2026-09-26T19:19:30Z, finished 2026-09-26T20:19Z:
**59 minutes**, of which roughly 45 was 490 approximate searches across
the two arms.

---

## 1. Fence scan, re-run on `git diff 3d520c0^..3d520c0`

My own scanner (`node`, in `/tmp/opencode/fence/scan.mjs`, run over the
374-line diff): it lists every file and every added line, then flags
acceptance phrases, steering scaffolding and relaxation markers, and any
changed `const`/`static` or retention-looking literal.

```
== files touched ==
src/lib.rs
tests/approx_determinism.rs
== added lines: 242, removed lines: 20 ==
```

Acceptance phrases in **added** lines:

| phrase | added lines |
|---|---|
| `wreck a nice beach` | **0** |
| `recognize speech` | **1** — `tests/approx_determinism.rs: "recognize speech",` |
| `hits justice dupe hid came` | **0** |
| `it's just a stupid game` | **1** — `tests/approx_determinism.rs: "it's just a stupid game",` |
| `justice too bad aim` (bonus) | 0 |
| `classic madgab` (bonus) | 0 |

Both hits are entries of the new test constant
`POOL_TARGETS` at `tests/approx_determinism.rs:226-231`. **Note 1.** The
comment immediately above that constant says:

> "none of them is either of the two phrases the project's acceptance
> criteria are written in terms of, so this test cannot be satisfied by
> anything keyed to those."

That is **false as written**: two of the four entries *are* acceptance
phrases. The same false sentence is repeated in `REPORT-d3b7c2.md` §4
("Four targets, all ordinary sentences, none of them either acceptance
phrase: `recognize speech`, …, `it's just a stupid game`, …"). I report
it as found and do not soften it. What it does **not** do is steer
anything: the entries only choose which sentences the reproducibility
guard measures, `src/lib.rs` contains no acceptance phrase, and
`tests/no_phrase_hard_coding` is green on both arms (§6). The guard is
not *satisfied* by those two entries — it would be red without them and
is red on the parent. So: a documentation defect in a merged test
comment and in the merged report, not a fence breach. It is the kind of
thing that should be corrected by whoever owns `tests/`, not by this
front.

Steering scaffolding in added lines: **2**, both in the test file and
both benign —

* `tests/approx_determinism.rs`: `let Ok(target) = std::env::var(POOL_HELPER_ENV) else {`
* `tests/approx_determinism.rs`: `let out = Command::new(std::env::current_exe()…)`

`POOL_HELPER_ENV = "APPROX_POOL_HELPER_TARGET"` is test-local, is
deliberately not `MADGAB_`-prefixed, and selects a helper `#[test]` that
returns immediately unless the driver names it. **No `ZZ_`, no `zz_`, no
`MADGAB_`, no `env::var` was added in `src/`**; the three `MADGAB_*` and
`std::env` lines in the diff context are pre-existing trace code that the
diff only shifted. The removed lines contain no scaffolding either.

Relaxation markers in added lines: **0** — no `#[ignore]`, no
`#[should_panic]`, no `allow(dead_code)]`, no "skipped". And
`grep -rn "#\[ignore" tests/ src/` on the tip returns **nothing at all**,
so no test anywhere in the tree is ignored.

Confirmed untouched by the diff, as required:

* `tests/no_phrase_hard_coding.rs` — not in the diff at all.
* `approximate_finds_classic_madgab_resegmentation`
  (`tests/corpus_integration.rs:134`), `approximate_finds_recognize_speech_resegmentation`
  (`:144`) and `approximate_output_is_locked` (`:459`) — all three live
  in `tests/corpus_integration.rs`, which is not in the diff. No
  baseline, no expected-value list and no tolerance in that file changed.
* Retention constants: the only `const` lines in the diff are the four
  test-local ones (`POOL_TARGETS`, `POOL_HELPER_ENV`, `POOL_TOP_N`,
  `REPLICATES`). `SEG_STATE_KEEP`, `SEGMENTATION_KEEP`,
  `LEXICAL_GLOBAL_EMISSION_BUDGET`, `EMIT_PROFILE_MAX_DEEP`,
  `EMIT_PROFILE_RESERVE`, `STRUCTURE_FLOOR` — **none** added, removed or
  changed. `coverage_tuples` and `sweep_index` are not touched. Nothing
  in `src/adjacency.rs`, `src/approx.rs`, `src/lexical.rs` is touched.

The change is three edits inside the structural DP of
`generate_approximate` plus the plumbing needed to expose the pool size,
and one test. Nothing else.

---

## 2. Generality: why this is a property of all inputs, and what it can hurt

### The mechanism, read from the tip source

The structural DP keeps `seg_states: Vec<HashMap<(usize, usize), Vec<SegPath>>>`
(`src/lib.rs:1134-1136`), `HashMap::new()`, i.e. `RandomState`, reseeded
per process. There are exactly two places where the *order* of that map's
contents is consumed:

1. **`src/lib.rs:1162-1165`** — the map at layer `p` is drained and
   `sort_by`'ed on the key `(word_count, shared)`. The key is a
   `HashMap` key, so it is unique: this is a **total** order, and the
   arrival order into every successor bucket becomes a function of the
   state keys alone.
2. **`src/lib.rs:1235-1237`** — the same drain-and-sort on
   `seg_states[n]`, feeding `segmentations`, which becomes `by_structure`
   → `schedule`, i.e. the order the emission budget is spent in.

Inside the loop the *retention* cuts are `bucket.sort_by(…).truncate(SEG_STATE_KEEP)`
(`:1218-1222`) and `segmentations.sort_by(…).truncate(SEGMENTATION_KEEP)`
(`:1252-1255`). Both comparators are now
`cmp_desc(score).then_with(|| a.spans.cmp(&b.spans))`, and `SegPath::spans`
is `Vec<(usize, usize)>` (`src/lib.rs:736-741`) — the chain of IPA
character boundaries the path has committed to.

**Why it is a property of all inputs, not of the measured ones:**

* The two primary keys are `f64` proxies (`span_partial`,
  `span_objective`) built from `SpanExtremes` arithmetic, and they tie
  *exactly* — not approximately — whenever two paths take the same
  monotone route through the same numbers. Ties are structural, not
  input-specific; whether a target *happens* to produce a tie is a
  property of its phonetics, not of the code.
* The properties used to break the tie — **the state key
  `(word_count, shared)`**, and **the committed span boundary chain
  `Vec<(usize, usize)>`** — are *derived from the candidate itself*. They
  are not a rank, not a score, not a heuristic preference, not a lookup,
  and not a function of the target's words. Two candidates that tie on
  score are separated by what they *are*.
* `spans` is unique within a bucket. I checked the two ways that could
  fail: `span_lattice[p]` is built by draining a
  `BTreeMap<usize, Vec<FuzzyMatch>>` keyed on `end` (`src/lib.rs:813-821`,
  `:1045`), so there is at most **one** `SpanEdge` per `(p, end)`; and a
  bucket at `seg_states[edge.end]` is keyed `(next_words, next_shared)`
  where `next_words = spans.len()`, so every contributor to one bucket has
  a predecessor chain of the same length, and two contributors with
  different predecessor chains produce different extended chains. So the
  comparator is a strict total order on every bucket the DP builds, for
  every input.
* Nothing in the three edits reads the corpus, the target text, the
  configuration or the environment. `generate` → `generate_with_pool` and
  `finish` returning a tuple are signature plumbing; the `pool_size`
  capture is a `len()`.
* Even if the uniqueness argument above were wrong, determinism would
  still hold: a stable sort of *equal* elements preserves arrival order,
  and arrival order is already totalised by edit 1. The two halves of the
  fix are individually sufficient; together they are belt and braces.

So there is no input class on which the fix declines to apply. The same
three lines run for a two-word collocation and a nineteen-syllable
shred, in exact mode's absence as much as in approximate mode's presence.

### The class of input a deterministic tie-break can hurt — stated plainly

`Vec<(usize, usize)>` `Ord` is lexicographic, so among equal-score paths
the fix **always prefers the path whose first boundary is leftmost**
(and, at the state map, the one with the fewest words and the least
sharing, since `(word_count, shared)` sorts ascending). That is a
*systematic* preference, and it replaces a decision that was previously a
symmetric coin flip.

`rank` and `span_objective` are **proxies**. `SpanExtremes` carries only
`min_cost`, `max_familiarity`, `max_shape`, `min_closed`, `min_reused`,
`min_syllables`, `max_syllables` (`src/lib.rs:1045-1057`) — the final
scorer additionally applies lexical shape quality, closed-class share,
word familiarity, rhythm/edge weights, boundary novelty and the coverage
reserve. So an exact tie in the proxy can hide a real difference in the
final score. The input class at risk is therefore:

> **targets (or span regions within a target) where two paths tie exactly
> on the DP proxy and the leftmost-first-boundary path is the worse clue
> by the full scorer.**

For such a target, the parent emitted a random one of the two and the tip
always emits the same one. If the unlucky side is the better clue, the
tip is worse — **reproducibly**, which is strictly harder to detect than
the parent's noise, because the noise floor is now exactly zero and
replication can no longer average the bias away. I looked for this and
did **not** find it at the display cutoff: the `--top 50` visible list is
**byte-identical on parent and tip on 7 of 7 targets** (§5), and
`approximate_output_is_locked` is green. So there is no measured quality
regression. What the fix does is convert a random draw into a fixed
preference whose direction is arbitrary and is now permanent. That is a
**note for a later pass**, not a defect in this change: the parent had no
defined behaviour to regress from, and the tip's behaviour is at least
now measurable. If a later front wants the bias gone, the right move is
to make the *proxy* agree with the scorer (fewer exact ties), not to
re-introduce a seed.

---

## 3. Is the determinism claim true on the merged result? — yes

Release, `--approximate`, pool count read from the pre-existing
`MADGAB_TRACE_PHRASES` probe (neutral non-word value, so the trace
reports `missing candidates=N`, which is `clues.len()` after dedup and
before `select_diverse` — the same `usize` that `generate_with_pool`
returns). Not the visible top-50.

Pool count is a function of `top_n` (`top_n` feeds the retention budget),
so it is only comparable at a fixed `top_n`. **Note 3:** for `recognize
speech` the tip pool is 11538 at `--top 10`, 12607 at `--top 20`, 15908
at `--top 50`. Any pool number quoted without its `top_n` is not a
measurement. All tables below are at **`--top 20`**, the guard test's
`POOL_TOP_N`, unless stated.

**10 consecutive runs per target, tip `3d520c0`:**

| target | runs | min | max | distinct values |
|---|---|---|---|---|
| `recognize speech` (canonical) | 10 | 12607 | 12607 | `12607 ×10` |
| `a whole lot of trouble` | 10 | 12500 | 12500 | `12500 ×10` |
| `It's just a stupid game` (canonical acceptance target) | 10 | 14561 | 14561 | `14561 ×10` |
| `the quick brown fox jumps` | 10 | 13523 | 13523 | `13523 ×10` |
| `the mad gab for kids` | 10 | 15433 | 15433 | `15433 ×10` |

Identical on 5 of 5. Extended to **50 runs per target** (see §4):
**250 tip runs, span 0 on every target.** And at the **default**
`--top 10` (no `--top` flag at all), 5 consecutive tip runs each:
`recognize speech` 11538 ×5, `It's just a stupid game` 13471 ×5,
`he was a big fat man` 14423 ×5. At `--top 50`, 3 full pool dumps per
target on 7 targets: sizes identical in all three (§5).

The claim holds. Independently of the merged test, and on the merged
binary.

---

## 4. The noise floor, re-taken on `3d520c0`

`b47d02`'s ±25 accepted tuples / ±0.06 pp was measured with the
`cost_probe` instrument over `coverage_tuples` offers, on the
**non-deterministic** parent pool. That instrument is not present at
`3d520c0` and I did not re-create it, so I re-derive the floor in the
quantity the fix is about: the pool count. **What I did not measure** is
listed in §7.

Pool count, `--approximate --top 20`, 50 consecutive release runs per
target per arm (250 per arm):

| target | tip `3d520c0` (n=50) | span | pp | parent `1bd005e` (n=50) | span | pp |
|---|---|---|---|---|---|---|
| `recognize speech` | 12607 ×50 | **0** | 0.0000 | 12608 ×25; 12610 ×25 | **2** | 0.0159 |
| `a whole lot of trouble` | 12500 ×50 | **0** | 0.0000 | 12490 ×20; 12493 ×30 | **3** | 0.0240 |
| `It's just a stupid game` | 14561 ×50 | **0** | 0.0000 | 14554 ×50 | **0** | 0.0000 |
| `the quick brown fox jumps` | 13523 ×50 | **0** | 0.0000 | 13520 ×26; 13516 ×24 | **4** | 0.0296 |
| `the mad gab for kids` | 15433 ×50 | **0** | 0.0000 | 15431 ×50 | **0** | 0.0000 |

**The number the next front can use:**

* On `3d520c0` the run-to-run noise floor on the pool count is
  **0 tuples = 0.0000 pp**, measured over 250 runs on 5 targets, with no
  target showing a single flip. Any pool delta is decidable; no
  replication is needed for any count comparison.
* Equivalently, on this tip the smallest *decidable* delta is the
  smallest *expressible* one: **1 tuple**, which on these pools is
  **0.0063–0.0079 pp** (1/15908 at `--top 50`, 1/12607 at `--top 20`).
* For the record, and **only** for anyone forced to measure on the parent
  `1bd005e`: the floor there is **2–4 tuples = 0.016–0.030 pp**, and it
  is **target-dependent** — 3 of 5 targets flip within 10 runs (per-run
  flip probability 0.20–0.50), and 2 of 5 never flipped in 50 runs
  (probability 0.000 at n=50). A 1-tuple delta on the parent is
  decidable on a q>0 target at n=10 with probability `1-(1-q)^9` =
  0.87–0.99, and is **not decidable at any n** on a q=0 target.
  Do not inherit a single number from the parent; it is per target.

**The important caveat, and it is the reason I am not calling this floor
"zero" without qualification. Note 5.** A reproducible *count* is not a
reproducible *membership*. On the **parent**, at `--top 50`, target
`It's just a stupid game` returned pool size **17906 on all five runs**
while the *members* differed between runs. So a count-only assertion
would have passed on a membership-unstable pool. The merged guard
`approximate_pool_is_reproducible_across_processes` is stronger than
count-only — it also compares the visible proposal list — but it still
does not compare the full pool membership, only a `top_n` sample of it.
On the **tip** I found no such case: 3 full pool dumps per target on 7
targets, membership identical in all three. This is a statement about the
guard's strength, not an observed defect in the tip.

---

## 5. Regression check: parent `1bd005e` vs tip `3d520c0`

Release, `--approximate --top 50`, 7 targets (the required ≥6, including
`recognize speech` and the canonical target). "Parent" membership is
quoted as the **intersection over 5 parent runs** — the only well-defined
parent set, because a single parent run is not a set. "Tip" is 3 runs,
identical in all three on every target.

| target | parent size (5 runs) | tip size (3 runs) | parent-stable ∩ | lost at tip | gained at tip | `--top 50` visible |
|---|---|---|---|---|---|---|
| `It's just a stupid game` (canonical) | 17906 ×5 | 17913 ×3 | 17896 | **165** | 182 | **byte-identical** |
| `recognize speech` | 15911, 15911, 15909, 15909, 15911 | 15908 ×3 | 15341 | **31** | 598 | **byte-identical** |
| `a whole lot of trouble` | 16159, 16159, 16162, 16162, 16159 | 16169 ×3 | 15960 | **255** | 464 | **byte-identical** |
| `the quick brown fox jumps` | 17084 ×2, 17088 ×3 | 17091 ×3 | 17056 | **40** | 75 | **byte-identical** |
| `the mad gab for kids` | 18472 ×5 | 18474 ×3 | 18459 | **53** | 68 | **byte-identical** |
| `when the rain finally stopped` | 15609 ×5 | 15609 ×3 | 15609 | **32** | 32 | **byte-identical** |
| `he was a big fat man` | 19169, 19164, 19167, 19165, 19168 | 19168 ×3 | 18917 | **288** | 539 | **byte-identical** |

### `(target, word)` pairs on the parent and absent at the tip — **yes, there are**

This is the failure mode that disqualified `3b14482`, and it is present
here in the count sense. Each of the following words occurs in **every
one of 5 parent runs** and in **none of 3 tip runs**, at `--top 50`:

| target | word | in parent 5/5 | in tip 0/3 |
|---|---|---|---|
| `recognize speech` | `kohn` | yes | no |
| `a whole lot of trouble` | `drool` | yes | no |
| `the quick brown fox jumps` | `cream` | yes | no |
| `the mad gab for kids` | `glare` | yes | no |
| `when the rain finally stopped` | `delhi` | yes | no |
| `It's just a stupid game` | `sook` | yes | no |
| `he was a big fat man` | `ohh` | yes | no |

Example lost pool phrases, one per target: `"are a can i chef pete shh"`
(`recognize speech`), `"ack hole pal ahh cut rubble"`
(`a whole lot of trouble`), `"a cream crown och stumps"`
(`the quick brown fox jumps`), `"a ahmad get fork is"`
(`the mad gab for kids`), `"an air a.'s fine a lee stop"`
(`when the rain finally stopped`), `"c ohh then big-ass at an"`
(`he was a big fat man`), `"ache justice stupid give un"`
(`It's just a stupid game`). Example gained: `"beach justice too bad
a.s"`, `"each dost too bad aim"`.

**Note 2 — this is bigger than the merged report says.**
`REPORT-d3b7c2.md` §3 says the fix "moved the deduplicated pool by 1-2
candidates on three of the four targets", and §7's last bullet asks a
coordinator only to "re-measure any pool-membership number quoted on the
base" on the assumption of a 1-2 candidate move. On the real parent
`1bd005e` at `--top 20` the **count** delta is 1, 7, 7, 3–7 and 2 tuples
(`recognize speech` 12607 vs 12608–12610 (1–3 tuples); `a whole lot of trouble` 12500
vs 12490–12493; `It's just a stupid game` 14561 vs 14554; `the quick
brown fox jumps` 13523 vs 13516–13520; `the mad gab for kids` 15433 vs
15431) — so up to **10 tuples, not 2**. And at `--top 50` the
**membership** churn is **31 to 288 phrases lost and 68 to 598 gained
per target**, on 7 of 7 targets. The mechanism is the one the fix's own
source comment names: `segmentations`' order becomes the order the
emission budget is spent in, so re-deciding one tie re-funds a different
set of boundary structures and the whole downstream pool turns over. A
1-2 tuple count delta and a hundreds-scale membership delta are the same
fact seen through two denominators, and the report quoted the flattering
one. **Consequence for later work: any published reachability table
denominated in pool membership must be re-measured on `3d520c0`; treat
`1bd005e` membership as unmeasurable rather than as a baseline to
compare against.** (I could not reconcile `REPORT-d3b7c2.md`'s absolute
numbers with mine — it measured on base `8d4cab1` and reports e.g.
12605/12604/12606 for `recognize speech` where I measure 12607 on the tip
and 12608/12610 on the parent — so the parent corpus/flags differ
slightly from the one I measured. I did not chase that; my numbers are
self-consistent across both arms, which is what the comparison needs.)

**Why this is a note and not a revert candidate.** The parent has no
membership to regress *from*: it is not reproducible, so "present on the
parent" is only meaningful for the 5-run intersection, and the tip's
answer is a single well-defined set. The visible product — the `--top 50`
list users and the corpus tests see — is **byte-identical on 7 of 7
targets**. No acceptance guard moved, no score moved, and the only red
test has a byte-identical failure payload (§6). Choosing one member of a
tie necessarily changes something; the tip changed it in a way that is now
measurable, and the alternative was not "no change" but "a different
random change every run".

### Is `recognize speech` → `wreck a nice beach` still produced? — **yes**

Explicitly, both arms, `--approximate --top 50`, release CLI:

```
parent 1bd005e : 28. [0.918] wreck a nice beach
tip    3d520c0 : 28. [0.918] wreck a nice beach
```

Same rank (28, 1-based), same score. Present in the full pool dump of
both arms (parent 5/5 runs, tip 3/3 runs).
`approximate_finds_recognize_speech_resegmentation` is **green**.

---

## 6. Suites, with a `git archive 3d520c0^` base arm

Both arms are clean `git archive` extractions, no probe. Tip-clean's
`src/` and `tests/` are byte-identical to this branch's.

| suite | tip `3d520c0` | base `1bd005e` | verdict |
|---|---|---|---|
| `cargo test --release --lib` | **ok — 53 passed, 0 failed, 0 ignored** | ok — 53 passed, 0 failed, 0 ignored | no change |
| `--test no_phrase_hard_coding` | **ok — 6 passed, 0 failed** | ok — 6 passed, 0 failed | no change |
| `--test corpus_integration` | **FAILED — 10 passed, 1 failed, 0 ignored** | FAILED — 10 passed, 1 failed, 0 ignored | **pre-existing red** |
| `--test approx_determinism` | **ok — 4 passed, 0 failed** | ok — 2 passed, 0 failed | +2 (the new guard) |
| `--test exact_determinism` | **ok — 1 passed, 0 failed** | ok — 1 passed, 0 failed | no change |

**The one red, separated.** The only failure on either arm is
`approximate_finds_classic_madgab_resegmentation`, at
`tests/corpus_integration.rs:136`, `canonical clue missing from top 50`.
It is **pre-existing red**, not new red:

* It fails **identically on the parent** `1bd005e` — same test, same line,
  same message, same 10-pass/1-fail split.
* Its printed 12 proposals are **byte-identical on both arms**:
  `["it justice too bad aim", "it justice too pad aim", "it justice too
  bad same", "it justice too peg aim", "it justice too pad same", "it
  justice too bad name", "it justice too bed aim", "it justice too pig
  aim", "eat justice too bad aim", "it justice too pad name", "it justice
  too pug aim", "it justice too bad came"]`. I diffed the two captures.
  So it is not green by accident and not newly failing for a different
  reason.

The other two named tests are **green on both arms**, untouched:
`approximate_finds_recognize_speech_resegmentation` ok,
`approximate_output_is_locked` ok. `approximate_output_is_locked` is worth
one sentence: it is an exact-list lock at `--top 10` on a third target
(`I love you`), and it passing on both arms is independent evidence that
the tie-break did not disturb scored output.

The tip's `approx_determinism` has 2 more tests than the parent's, which
is exactly the 2 added (`approximate_pool_helper`,
`approximate_pool_is_reproducible_across_processes`). Nothing was removed.

`cargo fmt`, `cargo clippy` and doctests are not available on this host
(`docs/environment-notes.md`); I did not run them and make no claim about
them. I also did not run the `--release --doc` target or the `web`/wasm
build.

---

## 7. What I did **not** measure

* **The `coverage_tuples` accepted/refused instrument.** `b47d02`'s
  ±25 accepted tuples / ±0.06 pp is a different quantity — acceptance
  *rate* over coverage-reserve offers — and the `cost_probe` that produced
  it is not on `3d520c0`. I did not re-instrument `coverage_tuples`, so
  that figure is neither confirmed nor re-derived; §4's floor is the pool
  count only. **A front that needs an acceptance-rate noise floor must
  re-create that instrument on `3d520c0` from scratch.**
* **`src/adjacency.rs`.** `d3b7c2` §7 left its `HashSet`s unaudited. I
  did not audit them either, and I did not instrument them. This remains
  an open loose end from the merged item, and I have no evidence either
  way.
* **Why I did not reconcile `REPORT-d3b7c2.md`'s absolute numbers** with
  mine (its base `8d4cab1` vs my parent `1bd005e`, and it reports
  `recognize speech` 12605/12604/12606 where I measure 12607 / 12608–12610
  at `--top 20`). Both of my arms are self-consistent, which is what the
  comparison needs, so I spent the budget on replicates instead. A
  coordinator may want to know why the bases differ.
* **Targets beyond the 7 above** (Q5) and the 5 above (Q3/Q4). No sweep,
  no `MAX_DEEP` row, no `c81e55` guard-margin re-measurement, no
  reachability table.
* **Membership at `--top 10` or `--top 20`.** Membership dumps are
  `--top 50` only; the count tables at `--top 20` have no membership
  counterpart.
* **`env::var`-gated probe paths in production** (`MADGAB_TRACE_*`): I
  used `MADGAB_TRACE_PHRASES` as a read-only observer and did not verify
  that the presence of the probe cannot perturb the pool beyond the
  `12607 == 12607` cross-check in §0.
* **The `w-558697` front.** It is live on `3d520c0` and lands no code; I
  did not read its branch tip as a base and did not touch it.
* **wasm / `web` build, `examples/measure.rs`, benchmarks.**

---

## 8. Verdict, restated

`3d520c0` is sound. It is a three-line, constant-free, phrase-free,
input-free total-order fix to two `HashMap` drains whose order fed a
stable sort and a `truncate`; its mechanism is a property of the code and
not of any target, and it holds for all inputs. The determinism claim is
independently true on the merged binary: 250 runs, 5 targets, pool span
0, plus 15 runs at the default `top_n` and 21 full pool dumps at
`--top 50`. The `--top 50` visible output is byte-identical to the parent
on 7 of 7 targets, `recognize speech` → `wreck a nice beach` is still
produced at rank 28 with an unchanged score, the three named tests are in
exactly the state the repository already records (one pre-existing red
with a byte-identical payload, two green), and nothing was relaxed,
skipped, re-baselined or ignored.

The notes a later pass should carry:

1. **A false claim in a merged comment and in `REPORT-d3b7c2.md` §4**:
   `tests/approx_determinism.rs:222-225` and its report paragraph assert
   that none of `POOL_TARGETS` is an acceptance phrase; two of the four
   are (`recognize speech`, `it's just a stupid game`). Documentation
   defect, no steering effect, but it is the kind of sentence a later
   reader uses as a fence argument. Owner of `tests/` should fix the
   wording; this front does not touch `tests/`.
2. **The pool moved much more than advertised** — 1 to 10 tuples by count
   at `--top 20`, and 31–288 phrases lost / 68–598 gained per target by
   membership at `--top 50`, on 7 of 7 targets, with named
   `(target, word)` pairs (§5). Re-measure, do not re-baseline against
   `1bd005e`.
3. **The pool count is a function of `top_n`** (11538 / 12607 / 15908 for
   `recognize speech` at 10 / 20 / 50). A pool number without its
   `top_n` is not a measurement.
4. **The tie-break has a direction** — leftmost first boundary, then
   fewest words, then least sharing — and the DP proxy it breaks on omits
   axes the final scorer applies, so a systematically favoured class of
   tied inputs is now permanently favoured rather than randomly. No
   quality regression is measurable at `--top 50`; the cost is that a
   future regression in this direction would be invisible to replication,
   because the floor is zero.
5. **A reproducible count is not a reproducible membership** — observed
   on the parent at `--top 50`. The merged guard compares the count and
   the visible `top_20` sample, not the full pool. No such case on the
   tip in 21 dumps.

None of these is a blocker. If a later pass decides note 2 or note 4
should be escalated, the right vehicle is a new work item — this front
proposes no revert, and `3d520c0`'s own effect on the acceptance guards
is nil.

**MERGE RECOMMENDATION: ALREADY-MERGED-WITH-NOTES**
