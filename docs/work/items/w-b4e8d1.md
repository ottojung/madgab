---
work_item: true
id: w-b4e8d1
state: done
priority: normal
owner: agent-b4e8d1
updated: 2026-09-26T22:20:00Z
branch: madgab-review-b4e8d1
worktree: /workspace/madgab-review
---

# Review front: audit the accumulation branch for phrase-specific hard-coding, and review the adjacency WIP against the fences

## Goal

Two read-only obligations that the itinerary imposes on the milestone and that no
other front owns, kept off the region `c1d3a7` is editing so the two do not
contend:

1. **The no-hard-coding audit.** The milestone in
   [../skills/itinerary-madgab.md](../skills/itinerary-madgab.md) requires that
   the implementation "does not contain phrase-specific hard-coding for these
   examples". That has been asserted repeatedly in work-item prose and never
   *checked* against the tree on the accumulation branch. Check it, and record
   the method and the result so a later pass can re-run it in one command.
2. **A fence review of the in-flight adjacency WIP.** `c1d3a7`'s commit `c71e699`
   on `madgab-adjacency` carries an adjacency operator and its call site. Review
   it as a diff for the properties the fences in
   [w-c1d3a7](w-c1d3a7.md) require, and report defects **early**, while the agent
   can still fix them, rather than at integration time.

## Why this is useful and not a manufactured backlog

Integration currently has no reviewer: the single live front
([w-c1d3a7](w-c1d3a7.md)) is also the only thing that could review itself, and a
finished agent's own claim to generality is not review. This front is cheap,
read-only, and materially shortens the integration step that the itinerary
requires before anything lands on `post-milestone-acceptance`.

## Scope

- Read-only with respect to `src/` and `tests/`. Do **not** edit production code,
  and do not "fix" anything you find; report it.
- Your only writes are this work item and, if useful, one new work item for a
  real defect you find (see below). Do not touch `w-c1d3a7.md`; that agent owns
  it and is editing it right now, so a write from you would collide at rebase.
- Report ENUMERATED and RANKED separately wherever you report a pool result.
  Conflating them has cost a pass on this repository before.

## Part 1: the hard-coding audit

On `post-milestone-acceptance` (`fdc2769` at claim time), search all of `src/`
and `tests/` for the canonical example words and their distinctive substrings:
`wreck`, `wrecked`, `beach`, `recognize`, `recognise`, `speech`, `hits`,
`hit `, `justice`, `stupid`, `dupe`, `hid`, `came`, plus the full target strings
`just a stupid` and `it's just`. Also look for the *shape* of phrase-specific
special-casing, which is what actually matters and which a word list alone can
miss:

- any comparison of a whole input target, or of a whole target sentence, against
  a literal;
- any lookup keyed on a specific word identity rather than on a phonetic
  property, a cost, a score or a structural feature;
- any test-only constant that a production branch reads (a `#[cfg(test)]` value
  or an environment variable consulted in `src/` and used to steer the search);
  the standing exception is the `MADGAB_TRACE_*` diagnostics, which are
  observation-only and must not change a result — **verify** that they are
  observation-only by reading them, not by assuming it, and say so explicitly;
- any `zz_`/`ZZ_` identifier, `.bench` tree, or `MADGAB_*` diagnostic other than
  the standing trace ones.

Deliver the exact command or tool you used, so the next pass can re-run it
without re-deriving the method.

## Part 2: the fence review of `c71e699`

Read `c71e699` (`git -C /workspace/madgab-adjacency show c71e699`) and review
against these, which come from the fences in [w-c1d3a7](w-c1d3a7.md) and the
itinerary:

- **No phrase-specific case.** Part 1's standard applies to this diff too, at
  the level of the new code.
- **Bounded, with named arithmetic.** Any new constant, cap, pass count or
  iteration bound must have a stated derivation, and the bound must be derivable
  from an observable property of the input rather than from a chosen target.
- **No `axes::*` move** and no re-baselining of `approximate_output_is_locked`.
  Pool membership is the requirement; the score need not change.
- **No diagnostic left behind**: no `zz_`/`ZZ_`, no `MADGAB_*` outside the
  standing trace family, no `.bench`, no build tree, no scratch path.
- **No wall-clock or pool regression** that the change cannot pay for. Report
  `recognize speech` and `It's just a stupid game` search times and pool sizes
  on the merged base and, if you can build the WIP branch cheaply, on `c71e699`
  too; if the build is too expensive, say so and skip rather than guess.
- **Determinism**: the change must not introduce a HashMap/HashSet iteration
  order into anything that affects output. `--test exact_determinism` and
  `--test approx_determinism` are the gate for that, and the seeds are fixed.

`c71e699` is a WIP commit and the agent is still editing, so a WIP defect is
expected. Report what is wrong with the *current* text and say explicitly that
it is a WIP reading.

## Part 3: the blocker, confirmed from the merged head

The single known blocker is
`approximate_finds_classic_madgab_resegmentation` in
`--test corpus_integration`. It was re-measured on the merged head `fdc2769` by
the coordinator and failed with 9 passed / 1 failed. Re-run it once yourself and
record the exact failure text, so the record does not rest on a single
observation:

```sh
cd /workspace/madgab-review && cargo test --release --test corpus_integration
```

Report the failure message verbatim, including the visible wordings it got.

## Constraints

- Never merge or push to `main`. Your branch is `madgab-review-b4e8d1`; push it
  and record the remote sha.
- No `zz_*` test, sweep script, `.bench` tree or `MADGAB_*` diagnostic on your
  branch. Scratch work belongs in `/tmp`.
- `cargo fmt`, `cargo clippy` and doctests do not exist on this host — see
  [../environment-notes.md](../environment-notes.md). Do not report them.
- `cargo test --release --lib`, `--test exact_determinism` and
  `--test approx_determinism` must be reported as measured before you close.
- Do not run long measurement sweeps. This is a review front, not a measurement
  front. If something would take minutes, record it as unmeasured.

## Completion criteria

1. The hard-coding audit is done, with the method recorded as a re-runnable
   command, and a verdict that is either "clean, with the exceptions listed" or
   "defects at <file:line>", and every standing exception individually
   justified.
2. The `c71e699` fence review is written up, defect-first, each defect with a
   `file:line` and what the fence it violates is.
3. Any *real* defect in the adjacency mechanism that the agent cannot reasonably
   be expected to find is captured as a **new** work item created from
   [../work/TEMPLATE.md](../work/TEMPLATE.md) — a defect about mechanism
   behaviour is not a work item about this review, and must not die with this
   one. Link it from this item's handoff.
4. The blocker is re-measured on the merged head and its exact failure text
   recorded.
5. `--lib`, `--test corpus_integration`, `--test exact_determinism` and
   `--test approx_determinism` reported as measured, with the one known failure
   named.
6. This item updated with objective state, branch pushed, worktree clean.

## Results

All three parts were completed on 2026-09-26 by agent `b4e8d1`. The audit
target moved from `fdc2769` to `aa453c1`, which touches only
`docs/work/items/w-4b1e07.md`, so the audited `src/` and `tests/` trees are
byte-identical to the ones named at claim time.

`src/` and `tests/` were **not** edited. `w-c1d3a7.md` was **not** edited.
`main` was never touched. The worktree is clean and the branch is pushed.

---

# Part 1: the no-hard-coding audit

**Verdict: clean, with the standing exceptions listed and individually
justified below.** No phrase-specific hard-coding was found in production
code on `post-milestone-acceptance` at `aa453c1`. This is the first time the
itinerary's milestone condition "the implementation does not contain
phrase-specific hard-coding for these examples" has been *checked* against the
tree rather than asserted in prose.

## The re-runnable command

Run from the repository root on the branch under audit. One command, no
arguments, no scratch files:

```sh
cd /workspace/madgab-review && \
git grep -n -i -E '\b(wreck|wrecked|beach|recognize|recognise|speech|hits|justice|stupid|dupe|hid|came|nice)\b|just a stupid|it.s just' -- src/ tests/ ; \
git grep -n -E '(==|!=|contains\(|starts_with\(|ends_with\(|eq_ignore_ascii_case)[^;]*"' -- src/ ; \
git grep -n -E '\bzz_|\bZZ_' -- src/ tests/ ; \
git grep -n -E 'MADGAB_' -- src/ tests/ ; \
git grep -n -E '#\[cfg\(test\)\]$' -- src/ ; \
git status --porcelain
```

`git grep` is used because this host has no `grep`, no `rg` and no `python3`
in `PATH` (see [../environment-notes.md](../environment-notes.md)); `git grep`
is present and reads only tracked files, so it cannot be fooled by a stray
untracked scratch file. The final `git status --porcelain` is part of the
command on purpose: an empty result is part of the verdict.

The last invocation of the command is what this section is a transcript of.

## Result 1: canonical example words — every occurrence is test code or a doc
## comment

`git grep` returns 51 lines. Classified:

| Location | Classification |
| --- | --- |
| `src/approx.rs:473`, `511`, `512` | `#[cfg(test)] mod tests` (starts `src/approx.rs:464`) |
| `src/lexical.rs:265`, `302`, `303`, `335` | `#[cfg(test)] mod tests` (starts `src/lexical.rs:260`) |
| `src/lib.rs:3131`–`4493` (all of them) | `#[cfg(test)] mod tests` (starts `src/lib.rs:3027`) |
| `src/approx.rs:45`, `src/lexical.rs:19`, `src/main.rs:9`, `11` | `///` / `//!` doc comments; no code |
| `src/lib.rs:2439`, `3224` | the word "came" inside `.expect("key came from cells")` / `.expect("candidate came from this pool")` prose, not a word literal |
| `src/lib.rs:1810`, `2053` region | the identifier `hit` (`let hit = ...`) |
| `tests/approx_determinism.rs:61`, `62`, `67`, `68`; `tests/corpus_integration.rs:28`–`37`, `65`, `66`, `135`, `137`, `145`, `147`, `324`, `325` | test inputs and the canonical word→IPA table; tests are the right place for these and the fence in [w-c1d3a7](w-c1d3a7.md) is explicitly on production `src/` |

**No canonical word appears in any production statement in `src/`.** The
mechanism that makes this cheap to state is structural: there is no
`include_str!`/`include_bytes!` anywhere in `src/`, `tests/` or `examples/`,
and the lexicon comes from the external `open-english-pronouncing-dictionary`
crate at runtime (`Cargo.toml`). There is therefore no embedded word list in
the repository that a phrase could have been baked into, and the closed-class
tables in `src/lexical.rs:64-176` (`ARTICLES`, `PRONOUNS`, `PREPOSITIONS`, …)
contain **no** canonical clue word — the `is_closed_class` word-identity
lookup is a grammatical-class table, not a phrase table.

## Result 2: shape of phrase-specific special-casing — clean

The word list alone can miss the interesting case, so the shape was checked
separately.

**Whole-input / whole-target comparison against a literal.** The second
`git grep` returns 8 lines in all of `src/`. Of those:

- `src/lib.rs:1714` `if suffix == "ies"` — a stem-stripping rule inside
  `novelty_stem`, applied to every word.
- `src/lib.rs:1759` `w == "a" || w == "i"` — inside
  `lexical_shape_quality_reference`, which is itself `#[cfg(test)]`
  (`src/lib.rs:1754`).
- `src/lib.rs:3120`, `3252`, `3257` — `#[cfg(test)]` (`cat`, `lead0 rr ss`).
- `src/main.rs:51`, `116` — CLI flag parsing (`--help`, `-h`, `--`).

There is no comparison of a whole input target or target sentence against a
literal anywhere in production code. `TargetPhrase::new` is called with
whatever the caller passed and the target is only ever compared to itself.

**Lookup keyed on word identity rather than on a phonetic/cost/structural
property.** A `git grep` for `word ==|w ==|stem ==|.contains(&"|insert(word`
returns two production hits, both the stem-suffix rules above. The
`HashMap`/`HashSet`s in production (`src/approx.rs:383`,
`src/lib.rs:1779`, `1794`, `1795`, `2378`, `2379`, `2424`) are keyed by word
*strings* for membership and deduplication, and every one is either sorted
before use or never iterated:
`src/approx.rs:415` sorts the lexicon; `src/lib.rs:2400-2401` sorts
`dedup.into_values()` by key with the explicit comment "HashMap iteration
order varies per process"; `src/lib.rs:2434-2436` sorts `cells.keys()`
before reading them. So no word-identity-keyed map steers a result.

**Test-only constant read by a production branch.** All six `#[cfg(test)]`
attributes inside production code (`src/lib.rs:1294`, `1436`, `1694`, `1754`,
`2031`, and the `mod counters` at `1650`) were read individually. Every
non-`mod` one is a `counters::bump` or `counters::note_depth` call on a
`thread_local!` `Cell`. `counters` is `#[cfg(test)]`-gated as a whole module
(`src/lib.rs:1650`) and is *compiled out of release builds entirely*. No
counter is ever read back in production, and none of them can steer the
search. **No test-only value reaches a production branch.**

**`zz_`/`ZZ_`, `.bench`, other `MADGAB_*`.** The `zz_`/`ZZ_` grep over `src/`
and `tests/` returns **nothing**. There is no `.bench` directory and no
`zz_*` test on this branch. `git status --porcelain` is empty, so there is no
untracked scratch tree either.

## Standing exception: `MADGAB_TRACE_*` is observation-only — verified by
## reading, not assumed

The `MADGAB_TRACE` grep returns 10 lines, all of one standing family, in two
blocks. Each was read.

**`src/lib.rs:1029-1086` (`MADGAB_TRACE_SPANS`, `MADGAB_TRACE_WORDS`).**
Placed after `segmentations` is built, sorted (`:1026`) and truncated
(`:1027`). The block parses the two env values into local `spans`/`words`
vectors, computes `in_range`, `seg_rank`, `word_rank` and `word_cost` into
locals, and calls `eprintln!` (`:1052`, `:1081`). It reads `segmentations`
and `span_lattice` and `self.fuzzy_lexicon` but assigns to none of them, and
it sits *before* the enumeration loop that consumes them, so it cannot even
perturb the traversal's inputs. The whole block is
`#[cfg(not(target_arch = "wasm32"))]`, so it is not even compiled for the
wasm build.

**`src/lib.rs:1577-1602` (`MADGAB_TRACE_PHRASES`).** Placed after `clues` is
sorted (`:1563`) and deduplicated (`:1575`), and immediately before the return
expression `select_diverse(clues, self.config.top_n)` (`:1604`). It looks
phrases up in `clues` by `eq_ignore_ascii_case` for a `position`, prints the
rank or a "missing" line, prints a cutoff line, and returns. `clues` is passed
by value to `select_diverse` afterwards and is not modified.

**Conclusion, stated explicitly because the work item asked for it: the
standing `MADGAB_TRACE_*` diagnostics are observation-only.** Neither block
mutates any value that reaches the output, neither can change a rank, a pool
or a score, and both are excluded from the wasm target. They are a justified
standing exception to the "no `MADGAB_*`" fence, individually, on the evidence
above — not by family name.

## What this audit does not claim

It is a *static* audit of tracked `src/` and `tests/` on one branch. It does
not rule out a hard-coding that is spelled as a character-level coincidence, or
a rule that happens to single out the canonical inputs without naming them
(e.g. a length-based special case). The second grep is the mitigation for the
latter and found nothing, but "no rule of the shape `if <input property> {
<different behaviour> }` exists" is a much stronger claim than what a grep
establishes, so it is not made.

---

# Part 2: the fence review of the adjacency WIP

## Target correction: `c71e699` is dead text

A coordinator update mid-pass retired `c71e699`: the commit was **rewritten**,
not amended in place. Verified rather than taken on trust:

```sh
git -C /workspace/madgab-adjacency diff --stat c71e699 e5d77e3
```

returns changes in `docs/work/items/w-4b1e07.md` and
`docs/work/items/w-5f1c04.md` only. **`src/` and `tests/` are identical
between the two.** So every *code* finding I had on `c71e699` is a finding
about the current text as well, and I re-read the current text at `9767caf` to
confirm rather than carry the old reading forward.

- Findings on `c71e699` that are marked **[historical]** below were made
  against the pre-rewrite commit. They are retained for the record only.
- Findings marked **[live]** are re-verified against `9767caf`.

Reviewed: `e5d77e3` (WIP) and `9767caf` (WIP, and not yet on `origin` — read
from the local branch `/workspace/madgab-adjacency`).

> **Both readings are of a WIP.** `c1d3a7` was still editing
> `/workspace/madgab-adjacency` during this pass. Every `file:line` below is
> against `9767caf` and must be re-derived if the agent has since edited the
> file. This is exactly the early-reporting the item asked for: the agent can
> still fix all of it.

## Defects, most severe first

### D1 [live] `src/lib.rs:169-174` — `ADJACENCY_POPS` contradicts the
### invariant stated immediately above it

```rust
/// `pops` is the walk's reach and, because each pop admits at most one new
/// wording, also bounds admissions.  It has to exceed the pool the traversal
/// hands it — the pool's own wordings are the seeds and are expanded before
/// anything new is reached — so it is set above the traversal's own share of
/// the allowance rather than at it.
const ADJACENCY_POPS: usize = 24;
```

The value is 24. The traversal's own per-segmentation share is
`structure_wording_allowance` = `share_cap(top_n, STRUCTURE_FLOOR)`, measured
at **17** on a real pool by [w-6b2f04](w-6b2f04.md), plus up to
`EMIT_PROFILE_RESERVE` = 16 of pooled profile emissions
(`src/lib.rs:1411`, `1417-1418`, `1566`). 24 is not "above the traversal's own
share"; it is below the share *and* below the share plus the reserve. The
comment was true at 96 and `9767caf` lowered the value without updating it.

**Fence violated:** "The change is bounded with named arithmetic" and "the
bound must be derivable from an observable property of the input rather than
from a chosen target" (w-c1d3a7 criterion 4, and this item's Part 2). A
constant that the file's own comment says is wrong is not a derivation.

This is not cosmetic. The consequence — the walk's pop budget being spent
re-popping its own seed pool, so `admissions <= pops - |pooled|` and the
operator can admit **nothing** on a real pool — is a defect about the
*mechanism*, which is why it was filed separately as [w-6f3a91](w-6f3a91.md)
rather than left here to die with a review item. The headline is
`src/adjacency.rs:151-162` plus `165`: `admit` seeds the frontier with every
root and then runs exactly `plan.pops` pop iterations, and a pop of a root
cannot admit because the root is already in `held` (`src/adjacency.rs:176`).

### D2 [live] `src/lib.rs:1055-1094`, `1316-1354`, `1699-1727`, `1740-1752`,
### `1755-1762` — five `ZZ_SCRATCH` diagnostic blocks left in production code

```sh
git -C /workspace/madgab-adjacency grep -n -E 'ZZ_' 9767caf -- src
```

returns 11 hits across five `// ZZ_SCRATCH`-marked blocks, env-gated on
`ZZ_SEGS`, `ZZ_SLOTS`, `ZZ_ADJ`, `ZZ_SEG_MATCH` and `ZZ_TOTALS`. They are in
`src/lib.rs`, one of them (`ZZ_ADJ`, `:1699-1727`) is **inside the adjacency
admission loop** and calls `std::env::var` and allocates a `Vec<String>` per
candidate.

**Fence violated:** "No `zz_*`, `.bench`, `MADGAB_*` diagnostic or build tree
left on the branch" (w-c1d3a7 §Fences) — verbatim. These are not the standing
`MADGAB_TRACE_*` family; they are a different prefix, in production `src/`,
on a branch that is a push away from integration. Cheap to fix, and it will be
missed if the branch is integrated first. Note also the asymmetry with
Part 1: this branch currently has *worse* diagnostic hygiene than
`post-milestone-acceptance`, which has none.

### D3 [live] `src/lib.rs:1419-1429` and three now-false doc comments —
### the per-segmentation budget is no longer conserved, and the docs still
### say it is

`9767caf` stopped carving the operator's share out of the traversal's
allowance — the `emit_allowance = emit_allowance.saturating_sub(adjacency_allowance)`
line was deleted, and `src/lib.rs:1419-1427` explains the new intent
correctly. But three older comments still assert the old, conservation-based
property and now contradict both the code and each other:

- `src/lib.rs:157-158` (on `ADJACENCY_RESERVE`): "Like the profile reserve it
  is carved out before the traversal starts, so the per-segmentation total —
  and therefore the global budgets — are unchanged; it moves spend, it does not
  buy more." **No longer true.** The spend is now additive.
- `src/lib.rs:192-193` (on `EMIT_DEEP_INDEX_LADDER`): "The two spends share
  one per-segmentation allowance — the traversal is handed
  `emit_allowance - profile_emitted` — so they are not additive." **No longer
  true**, and it is now false in the *opposite* direction from the one it
  claims.
- `src/lib.rs:169` (on `ADJACENCY_POPS`): see D1.

**Fence violated:** "Bounded, with named arithmetic" (criterion 4). More
seriously, the *reason* the new funding is defensible is a single slack
measurement quoted in a comment — "the baseline leaves slack (14,239 of 16,384
spent)" — and w-c1d3a7's fence table lists **budget definition /
per-segmentation funding (w-5f1c04, w-6b2f04)** among the closed families "not
to be reopened". Changing the per-segmentation funding rule *is* reopening
that family. It may well be the right call; but it needs to be stated as a
deliberate re-opening with a before/after table, the same standard
`approximate_output_is_locked` is held to, not smuggled in via a deleted
`saturating_sub` and a comment.

### D4 [live] `src/adjacency.rs:99-116` — one allocation per *probed*
### candidate, and the work is not bounded by `per_slot`

`best_children` constructs an owned `Node`, including
`tuple: scratch.clone()`, for **every** `i in 0..width` *before* the retention
gate at `:108`. With `ADJACENCY_PER_SLOT` = 2, at most 2 clones per slot per
pop survive; the rest are dropped on the floor. Per pop the cost is
`depth × width` probes and `depth × width` allocations, with `width` bounded
by `SPAN_SHORTLIST` = 160 (`src/lib.rs:123`, applied to the edge match list at
`src/lib.rs:819`) and `depth` the span count.

The module doc at `src/adjacency.rs:30-37` claims the two named bounds "keep
it from turning into a second product enumeration". That is true of the
*frontier* and false of the *work*: `9767caf`'s change of
`ADJACENCY_PER_SLOT` 4 → 2 reduces the frontier and does not reduce the probe
count at all.

**Fences violated:** "per-candidate allocations (w-d17a62)" — a closed family,
"not to be reopened" — and "bounded with named arithmetic" (criterion 4). The
clone is also unnecessary: `scratch` is already scratch, so the clone is only
needed when a child is actually pushed, and the eviction comparison at `:111`
needs only `key`.

### D5 [live] `src/adjacency.rs:81-83` — `quantize` maps `NaN` to `0`, which
### is the *best* possible key

`(score * 1e9).round() as i64` saturates `NaN` to `0`. Since real keys are
negative and a `0` outranks every one of them, a `NaN` score would make that
node the best in the frontier. Whether `bound` can return `NaN` was **not
determined** — `Metrics` does f64 arithmetic and the surrounding code
defensively uses `partial_cmp(..).unwrap_or(Equal)` (`src/lib.rs:1565`), which
is evidence that the authors have been bitten by non-finite scores. Reported
as a latent hazard, not a confirmed defect; the cheap fix is a `total_cmp` or
an explicit non-finite check.

### D6 [live] `src/lib.rs:1681-1689` — `admit` is called for every
### segmentation and its result is a `Vec` built eagerly

`admit` runs the full walk (including all the `best_children` probes from D4)
and materialises the whole admission list, and only then does the caller
`break` at `src/lib.rs:1690-1695` after at most
`adjacency_allowance <= ADJACENCY_RESERVE` = 8 admissions. On the common path
where `pops` is small this is cheap, but combined with D1 the eager list is
mostly roots' children that are then discarded. Not a defect on its own;
recorded because it is the same cost and the same fix site as D4.

## Fences checked and found satisfied

**No phrase case.** Applied Part 1's standard to the diff. `src/adjacency.rs`
contains no vocabulary at all: it is a function of index tuples, list widths
and a scoring closure (`admit(plan, roots, widths, bound)`,
`src/adjacency.rs:133-138`). The call site reads words only to *print* them in
a diagnostic. The only canonical-word literals anywhere in the new code are in
`#[cfg(test)]` tests, and `Cargo.toml` gained no dependency and no
`include_str!`.

**Determinism — checked independently, and the coordinator's read is
correct.** The claim to check was that `src/adjacency.rs` uses `HashSet` only
for membership. Verified by reading the module and by
`git grep -n -E 'seen|held|iter\(\)|into_iter' 9767caf -- src/adjacency.rs`:
`seen` and `held` (`src/adjacency.rs:146-148`) are only ever `insert`ed
(`:155`, `:156`, `:176`, `:191`) and are **never iterated** — there is no
`for` over either and no `.iter()`/`.into_iter()` on them anywhere in the file.
Every output order comes from paths with a total order:

- `Node: Ord` (`:65-71`) compares `key` then `other.tuple.cmp(&self.tuple)`,
  which is a total order on `(i64, Vec<usize>)`, so `BinaryHeap<Node>` pop
  order is independent of push order and of any hash seed.
- roots are pushed in `pooled` order (`:151-162`), which is emission order
  (`src/lib.rs:1411`, `1566`) — deterministic.
- `best` is drained at `:117` and then explicitly `sort_by` at `:118` with the
  same total order, so `best_children` is order-independent by construction.
- `out` is push order, and push order follows pop order.

So no `HashMap`/`HashSet` iteration order reaches the output, and
`--test exact_determinism` / `--test approx_determinism` are the right gate.
**This is a genuine pass, and it is the one fence in Part 2 I can affirm
rather than merely not-contradict.** The module's own `is_deterministic` test
(`src/adjacency.rs`, in `mod tests`) is a same-process repeat, which is much
weaker than the cross-process gate, so the cross-process suites are what
actually carry this fence.

**No `axes::*` move.** The diff touches no axis constant. The operator
re-scores with the caller's own `bound` closure (`src/lib.rs:1472-1505`),
which is the *same* closure the traversal orders by, so a child's key is
computed from the same weights. Pool membership can therefore change without
the score function moving at all, which is what the fence wants.

**No re-baselining of `approximate_output_is_locked`.** The diff contains no
test-file change and no change to any expected-output constant. Not touched.

**No `.bench`, no build tree, no scratch path** in the diff, beyond the `ZZ_*`
diagnostics of D2.

## Unmeasured, deliberately

Part 2 asked for `recognize speech` and `It's just a stupid game` search
times and pool sizes on the merged base and on the WIP. **Not measured, and
recorded as unmeasured rather than guessed:**

- No pool or wall-clock figure is reported for `9767caf` at all. Building and
  running that branch is a multi-minute measurement, this is a review front
  and not a measurement front, and the branch is a moving target while
  `c1d3a7` is still editing it — any number produced now would be stale on
  arrival. The base-side figures belong to the measurement front, not this one.
- The D1 and D4 arithmetic above is derived from constants read in the source
  (`ADJACENCY_POPS`, `SPAN_SHORTLIST`, `SEGMENTATION_KEEP`,
  `EMIT_PROFILE_RESERVE`, `share_cap`, and w-6b2f04's measured breadth of 17),
  **not** from a run. It is an upper bound on cost and an argument about
  behaviour, and it is labelled as such. Confirming D1's consequence —
  `admissions == 0` on a real pool — needs one run of
  `--test corpus_integration` with an `adjacency_emitted` total, which is
  exactly the kind of thing the owner can do for free in its own worktree.

---

# Part 3: the blocker, re-measured on the merged head

Re-run once, as specified, on this branch at `aa453c1` (docs-only delta from
`fdc2769`; identical `src/` and `tests/`).

```sh
cd /workspace/madgab-review && cargo test --release --test corpus_integration
```

**Result: 9 passed, 1 failed.** The failure text, verbatim:

```text
---- approximate_finds_classic_madgab_resegmentation stdout ----

thread 'approximate_finds_classic_madgab_resegmentation' (1155656) panicked at tests/corpus_integration.rs:136:5:
canonical clue missing from top 50; got: ["it justice too bad aim", "it justice too pad aim", "it justice too bad same", "it justice too peg aim", "it justice too pad same", "it justice too bad name", "it justice too bed aim", "it justice too pig aim", "eat justice too bad aim", "it justice too pad name", "it justice too pug aim", "it justice too bad came"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

```text
failures:
    approximate_finds_classic_madgab_resegmentation

test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.94s
```

The record no longer rests on a single observation: the coordinator measured
9 passed / 1 failed on `fdc2769` and this front independently measured 9
passed / 1 failed on `aa453c1`, same test, same cause.

## What the twelve wordings it got actually say

Read as evidence, because it constrains what a fix can and cannot do:

- **`hits` was reached in every one of the twelve wordings** — `hits` is
  position 0 of the canonical clue, and the search is putting it there
  reliably. So the blocker is **not** "the operator cannot find the first
  slot". That is the cheap half.
- **`justice` is reached in all twelve** as well, in position 1. Two of five
  canonical slots are found and stable.
- The remaining three — `dupe`, `hid`, `came` — are where it fails. The
  wordings are all `... too <X> <Y>` shapes: `too` occupies the `dupe` slot
  and a near-`hid` (`bad`, `bed`, `peg`, `pig`, `pad`, `pug`) plus a
  near-`came` (`aim`, `same`, `name`) fills the last two. `dupe`/`hid` are
  short, high-confusion, low-rarity words, which is the opposite profile from
  `hits`/`justice`.
- The failures are *near misses in the same structural family*, not a
  different resegmentation entirely: all twelve are the same five-slot shape
  as the clue. The clue is not being missed by a ranking accident at rank 51+;
  it is not in the pool.
- One wording, `it justice too bad came`, has `came` correct. So `came` alone
  is reachable; it is the **combination** that is not.

**Reporting discipline, as the item requires: ENUMERATED and RANKED
separately.** The failure text is a *ranked* observation — what the top 50
*displays*. It is not evidence about what was **enumerated**. `hits` and
`justice` being in all twelve is a statement about the ranked list only. No
enumerated-pool figure is claimed here, because none was measured. Any
mechanism work that wants to say "the clue is enumerated but ranked out" needs
an enumerated count from the pool, not from this test.

---

# Test suite report, all measured on `aa453c1`

Every suite below was run on this branch. No suite is reported from memory.

| Command | Result |
| --- | --- |
| `cargo test --release --lib` | **ok. 39 passed; 0 failed** (9.44s) |
| `cargo test --release --test corpus_integration` | **FAILED. 9 passed; 1 failed** (9.94s) — the one known blocker, `approximate_finds_classic_madgab_resegmentation`, verbatim above |
| `cargo test --release --test exact_determinism` | **ok. 1 passed; 0 failed** (4.52s) |
| `cargo test --release --test approx_determinism` | **ok. 2 passed; 0 failed** (25.19s) |

The single known failure is named and is the only one. Note that
`--test approx_determinism` passing on the merged head is the *base* for the
determinism fence in Part 2: it establishes that the base is reproducible, so
a reproducibility failure introduced by the adjacency change would be
attributable to the change.

`cargo fmt`, `cargo clippy` and doctests do not exist on this host and are not
reported, per [../environment-notes.md](../environment-notes.md).

---

# Defect escalated to its own work item

**[w-6f3a91](w-6f3a91.md)** — "The adjacency walk charges its seed pool against
its own pop budget, so on a deep-seeded segmentation it can admit nothing at
all."

Filed because D1 is a defect about *mechanism behaviour*, and a mechanism
defect must not die with a review item. It carries the headline defect
(`admit`'s seed/pop accounting) **and** D4 (per-probed-candidate allocation),
because both are cost-and-behaviour properties of the same module and both
need the same before/after measurement to settle.

Also escalated out of this item and recorded here for the coordinator, but
**not** filed as a work item, because they are fence/process matters for
`c1d3a7` to settle inside [w-c1d3a7](w-c1d3a7.md) rather than separate tasks:
D2 (the five `ZZ_*` blocks), D3 (the three now-false budget doc comments and
the unstated re-opening of the closed per-segmentation-funding family), and D5
(the latent `NaN`-to-`0` in `quantize`).

---

# Blockers

None for this front. Both read-only obligations are discharged and the
blocker re-measurement is recorded. This item is `done`; nothing here is
waiting on `c1d3a7`.

One soft dependency worth a coordinator's attention, not a blocker: **the
integration step has no reviewer other than this front, and this front is
read-only.** The defects above are reported, not fixed. Integration should not
proceed on `9767caf` until D1 and D2 are addressed and the D3 budget
re-opening is stated with a before/after table.

# Next action

1. A coordinator steers [w-6f3a91](w-6f3a91.md) to `c1d3a7` — it wrote the
   module, and D1 contradicts its own stated invariant, so it is the right
   first reader. No owner is claimed on it yet; claiming is a coordinator's
   call.
2. `c1d3a7` clears D2 (five `ZZ_*` blocks out of `src/`) before the branch is
   a push away from integration.
3. `c1d3a7` decides D3 explicitly: either restore the conservation
   (`saturating_sub`) or state the re-opening of w-5f1c04/w-6b2f04's
   per-segmentation funding rule with a before/after table, and fix the three
   doc comments either way.
4. Whoever integrates re-runs the Part 1 audit command above — it is one
   command, and it is the check the milestone has been asserting in prose.
5. The blocker stays open on its own item; this front does not own it.

## Handoff / notes

Claimed by coordinator `coord-2f6a` on 2026-09-26T20:50Z from
`post-milestone-acceptance` at `fdc2769`, worktree `/workspace/madgab-review`,
branch `madgab-review-b4e8d1`, agent `b4e8d1`. Launched in parallel with
`c1d3a7`, which is still running on `madgab-adjacency`; this front is fenced away
from that worktree and from `src/`.

Board resources: `antonina board resource list` returned nothing on this host, so
no board dependency is registered for `/workspace/madgab-review`. The pushed
branch and this item are the durability mechanism, per
[../skills/resources.md](../skills/resources.md).

## Closing state, 2026-09-26T22:20Z

- Base: `post-milestone-acceptance`, fast-forwarded `fdc2769 -> aa453c1` at the
  start of the pass. The delta is `docs/work/items/w-4b1e07.md` prose only, so
  the audited `src/` and `tests/` are the ones named at claim time.
- Branch: `madgab-review-b4e8d1`. Committed and pushed. Worktree clean;
  `git status --porcelain` empty. No `zz_*`, sweep script, `.bench` tree or
  non-standing `MADGAB_*` diagnostic was created on this branch; no scratch
  file was needed outside this item and [w-6f3a91](w-6f3a91.md).
- Writes on this branch are exactly: this item, and the new
  [w-6f3a91](w-6f3a91.md). `src/` and `tests/` untouched (verified by
  `git diff --stat fdc2769..HEAD`, which lists only `docs/work/items/`).
  `w-c1d3a7.md` untouched, as instructed.
- `main` was never merged to or pushed to.
- Completion criteria 1-6 are all satisfied. Criterion 3 is satisfied by
  [w-6f3a91](w-6f3a91.md), linked above. Criterion 4 is the verbatim blocker
  text in Part 3. Criterion 5 is the four-row measured table above.

One note for whoever reads this next from repository state alone: the Part 2
readings are of `e5d77e3` and `9767caf` on the **local** branch
`/workspace/madgab-adjacency`, and `9767caf` was not on `origin` at the time
of this pass. If the agent has since amended either commit, every `file:line`
in Part 2 and in [w-6f3a91](w-6f3a91.md) must be re-derived; the findings are
about the *shape* of the mechanism and should survive re-derivation, but the
line numbers will not.
