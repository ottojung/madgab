---
work_item: true
id: w-d4f0b2
state: working
priority: normal
owner: agent-d4f0b21
updated: 2026-09-26T19:05:00Z
branch: madgab-nohardcode
worktree: /workspace/madgab-nohardcode
---

# Standing fence: no phrase-specific hard-coding for the canonical examples

## Goal

Add an automated, repository-resident check that the two canonical Mad Gab
examples were **not** solved by hard-coding them, and that the hard-coding has
not crept in afterwards. The itinerary forbids it in terms, and
[../skills/itinerary-madgab.md](../skills/itinerary-madgab.md) makes it one of
the milestone's completion conditions; today that condition is verified by
hand, one diff review at a time, by whichever coordinator happens to be
running.

## Why this is worth a front now

The queue has refuted seven mechanisms in a row
([w-9d4e17](w-9d4e17.md), [w-5f1c04](w-5f1c04.md), [w-e07c42](w-e07c42.md),
[w-be6d21](w-be6d21.md), [w-3f9c02](w-3f9c02.md), [w-8a1d47](w-8a1d47.md),
[w-3b8e15](w-3b8e15.md)) and the last blocker is still open, which is exactly
the pressure under which a hard-coded exception looks attractive to whoever is
holding the last front. The fence makes the forbidden shortcut fail CI
instead of relying on review attention at exactly the moment review attention
is scarcest.

## Context

- The two acceptance examples and their clues are already named in
  `tests/corpus_integration.rs` (`approximate_finds_recognize_speech_resegmentation`,
  `approximate_finds_classic_madgab_resegmentation`) and in `src/main.rs` doc
  examples. Both are legitimate; a fence must not flag them.
- `src/` is the region that matters. The interesting negative space is
  `src/`, and within it the non-test code: unit-test modules inside `src/`
  (`#[cfg(test)]`) legitimately name the phrases, and so may doc comments and
  module docs. Whether those are excluded by a rule or by an explicit,
  reviewable allowlist is a design decision for the agent; the allowlist must
  be explicit and small, and adding an entry to it must be a visible,
  greppable act.
- Detection has to be about *behavioural coupling*, not about a bare word
  match. The words involved are ordinary English (`wreck`, `beach`, `speech`,
  `came`, `hid`, `justice`, `dupe`, `aim`, `bad`). A check that fires on any
  occurrence of `came` in `src/approx.rs` would be noise and would be deleted
  by the next agent. The check should look for the shapes a hard-code
  actually takes: a literal full clue string, a comparison against a
  normalized target string, a whole-sentence equality test, a lookup table
  keyed by the target text, and a per-word or per-substring special case.
  Report which shape matched, so a failure is actionable.
- The fence must be cheap. It is a source-scan test, not a search run, and it
  must not need a release build or a corpus.

## Scope and collision boundary

This front owns **one new file** under `tests/`, plus at most a short
paragraph in `docs/skills/itinerary-madgab.md` or a README next to it saying
what the fence is and how to run it. It is deliberately non-contending with
the live enumeration front ([w-c4e8d7](w-c4e8d7.md), agent `c4e8d70`), which
owns `src/lib.rs` and has uncommitted work in `tests/corpus_integration.rs`.

Forbidden: any edit to `src/` (including `#[cfg(test)]` modules inside
`src/`), any edit to an existing `tests/*.rs` file, any change to
`approximate_output_is_locked` or to either acceptance test, and any edit to
`Cargo.toml`'s dependencies beyond what a plain `std`-only test needs. Do not
re-baseline anything; this front changes no search behaviour at all.

## Completion criteria

1. A new `tests/*.rs` file that fails when a phrase-specific hard-code for
   either canonical example appears in the production region of `src/`, and
   passes on the current tree.
2. It is demonstrated to **catch** a hard-code, not just to pass: the agent
   must show a positive control — a temporary hard-code added to `src/`, the
   test failing with a message naming the file, the line and the matched
   shape, and the file restored afterwards. The restore must be verified with
   `git status` and the diff shown empty.
3. The rule is documented in the test's own module docs, including how to add
   a legitimate allowlist entry and why the allowlist is expected to stay
   short.
4. The check does not flag the pre-existing legitimate uses: the two
   acceptance tests in `tests/`, the `#[cfg(test)]` modules inside `src/`, and
   the `src/main.rs` doc examples. Say explicitly which regions are excluded
   and why.
5. `cargo test --release --test <new file>` is green, and
   `cargo test --release --lib` and `--test corpus_integration` are
   **unchanged** from the head this was based on: 31+ pass with
   `approximate_finds_classic_madgab_resegmentation` still the only failure in
   `corpus_integration`. `cargo fmt`, `cargo clippy` and doctests do not exist
   on this host; see [../../environment-notes.md](../../environment-notes.md).
   Do not claim them.
6. The work is on a focused branch off `post-milestone-acceptance`, pushed,
   and integrated into `post-milestone-acceptance`, never `main`.

## Handoff / notes

Claimed and launched: agent `d4f0b21`, worktree `/workspace/madgab-nohardcode`,
branch `madgab-nohardcode` from `e55ec51`. The push of this file is the claim
event.

Filed by coordinator `coord-7b3e` as a deliberately non-contending second
front while `c4e8d70` holds the milestone's critical path. It is a fence, not
a fix: if this front succeeds it changes no search output, and that is the
intended outcome, not a disappointment. Do not use it as an opportunity to
attempt the milestone.

Next action for a later fresh pass: review the new test as a diff — it must
add detection power rather than a brittle word list, and criterion 2's
positive control must be reported — then run criterion 5's suites and
integrate onto `post-milestone-acceptance`.

## Progress: the fence is written, validated and pushed

`tests/no_phrase_hard_coding.rs` (commit `1d39f50` on
`madgab-nohardcode`, pushed to `origin/madgab-nohardcode`). One new file, no
other change on the branch: `git diff` against `8fc1f6f` shows
`tests/no_phrase_hard_coding.rs` and nothing else, so criterion 5's
"unchanged from the head this was based on" is true by construction rather
than by re-measurement.

### What it detects, and why it is not a word list

The gate for this front was detection power rather than a list of common
English words, so the test watches *shapes* and reports the shape it matched
along with file and line:

- `whole-sentence-equality` — the phrase produced or bound verbatim
  (`return "hits justice dupe hid came";`)
- `lookup-keyed-by-target-text` — a full phrase used as a `match` arm, map
  key or table cell
- `comparison-against-target-text` / `comparison-against-normalized-target` —
  the phrase, spaced or normalized (`"wreckanicebeach"`), compared against a
  runtime value
- `full-phrase-literal` — any other literal spelling the whole phrase,
  including one spread over a statement's literals
  (`vec!["hits", "justice", "dupe", "hid", "came"]`) and one written
  normalized
- `substring-special-case` — a multi-word piece of a phrase handed to a
  string method (`clue.contains("nice beach")`)
- `phrase-substring-literal` — any other literal holding three consecutive
  words of a phrase
- `identifier-named-after-phrase` — no literal at all
  (`fn wreck_a_nice_beach()`, `const RECOGNIZESPEECH`)
- `runtime-normalization-comparison` — no literal at all: a normalized form
  compared against a value that was remembered rather than recomputed
  (`phrase_signature(&clue) == self.expected_signature`), which is how a
  hard-code recognizes a target without naming it

Two thresholds keep it a coupling test. A single word never fires: `came`,
`hid`, `bad`, `aim` and `wreck` are ordinary words production code may
legitimately contain. Two words fire only where a literal is being used as a
key, inside a string method call, because two words are a collocation
(`"stupid game"`) and chance produces collocations, while three consecutive
words of a canonical phrase is not something chance produces.

### Criterion 2: the positive control, and the restore

A temporary hard-code was added to `src/approx.rs` and the test run against
it. First control, a multi-statement function, reported the unit's first line:

```
  /workspace/madgab-nohardcode/src/approx.rs:465  [comparison-against-target-text]
      "hits justice dupe hid came" spells out a whole clue phrase and is compared against a runtime value
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out
```

Second control, the same special case as a one-liner, reported the exact
line:

```
  /workspace/madgab-nohardcode/src/approx.rs:464  [comparison-against-target-text]
      "hits justice dupe hid came" spells out a whole clue phrase and is compared against a runtime value
test result: FAILED.
```

`src/approx.rs` was then restored with `git checkout --`, and the restore is
verified: `git status --porcelain` lists only the untracked new test file,
`git status` reports "nothing added to commit but untracked files present",
and both `git diff` and `git diff --stat` produce no output at all.

The positive control is also permanent, not only a claim in a commit
message: `the_detector_catches_every_documented_shape` runs one synthetic
unit per documented shape and asserts each is detected *as that shape*, and
`the_detector_stays_quiet_on_ordinary_english_and_real_production_code`
asserts that nine ordinary lines — including `came`, `aim`, `bad`, `hid`,
`"stupid game"`, `"recognize"`, a comment naming every canonical word, and a
signature-to-signature de-duplication comparison — do not fire. A third test
asserts the excluded regions really are excluded, and a fourth pins the
watched phrase data so it cannot silently drift.

### Excluded regions, and why (criterion 4)

- `#[cfg(test)]` modules inside `src/`: `src/lexical.rs` and `src/lib.rs`
  name the phrases when asserting stem, signature and IPA behaviour. The scan
  drops a `#[cfg(test)]` module and resumes after its closing brace, so
  production code between two test modules is still scanned.
- All comments and doc comments, including the `src/main.rs` doc examples:
  prose that names an example is documentation, not behaviour.
- The other files in `tests/`: only `src/` is read, so
  `tests/corpus_integration.rs` can keep naming the phrases in the two
  acceptance tests, and `approximate_output_is_locked` is untouched.
- `Cargo.toml`, `examples/`, `web/`: not the production region. No dependency
  was added; the test is `std`-only.

### Allowlist (criterion 3)

`ALLOWLIST` in the test file is empty, with a `RECOGNIZED:` comment marking
where an entry goes. An entry names one file and one line, a marker string
that must still be present on that line, and a reason; there is no wildcard
form, so adding one is a greppable edit a reviewer sees in the diff. A test
fails if it ever exceeds one entry, on the reasoning that two entries means
the region boundary is in the wrong place and should be widened instead.

### Validation run (criterion 5)

- `cargo test --release --test no_phrase_hard_coding` — 6 passed, 0 failed.
- `cargo test --release --lib` — 50 passed, 0 failed.
- `cargo test --release --test corpus_integration` — 9 passed, 1 failed:
  `approximate_finds_classic_madgab_resegmentation` only, with
  `approximate_finds_recognize_speech_resegmentation` passing. That is the
  pre-existing failure the item describes.
- Also run for completeness: `--test approx_determinism` 2 passed,
  `--test exact_determinism` 1 passed.

Note on the item's own figures: it says "31+ pass", but at this commit
`--lib` reports 50 tests and `corpus_integration` reports 10 (9 pass, 1
fail). The counts do not match the text, so treat the numbers above as the
measurement and the "31+" as stale. Nothing was re-baselined.

Not verifiable on this host, per
[../../environment-notes.md](../../environment-notes.md): `cargo fmt`,
`cargo clippy`, and any doctest. No claim is made about them.

### Not done, and the next action

Criterion 6's integration onto `post-milestone-acceptance` is **not** done.
The branch was pushed to `origin/madgab-nohardcode` at `1d39f50`, as
instructed, and `madgab-nohardcode` is now 1 ahead and 3 behind
`origin/post-milestone-acceptance` (whose tip is `75ec5b7`). Integrating means
either a merge or a rebase onto a shared branch that agent `c4e8d70` is
actively working on, which is the collision this front was filed to avoid,
and it was not part of this pass's instructions.

Next action: a pass that owns `post-milestone-acceptance` should rebase or
merge `madgab-nohardcode` onto it, re-run
`cargo test --release --test no_phrase_hard_coding` plus `--lib` and
`--test corpus_integration` after the merge, and then set this item `done`.
A reviewer should read the new test as a diff and judge the thresholds
themselves: if a legitimate two-word literal in production code is ever
dropped in, the fix is an allowlist entry with a reason, not a wider net.
