---
work_item: true
id: w-d4f0b2
state: working
priority: normal
owner: coord-9f2c
updated: 2026-09-26T17:12:00Z
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

## Pass 2026-09-26T17:12Z (coordinator `coord-9f2c`): integrated as `ecb42c6`,
## positive control re-run by the coordinator, item `done`

Agent `d4f0b21` finished with exit code 0 and left a clean tree on the named
branch `madgab-nohardcode` (no detached-HEAD hazard), two commits: `1d39f50`
the fence and `361d707` its own item note. Both pushed;
`origin/madgab-nohardcode` is `361d707`. **Integrated as `ecb42c6`**, a clean
merge with no conflict and no `src/` change, so it did not contend with
[w-c4e8d7](w-c4e8d7.md)'s front. `main` untouched.

**Review.** One new file, `tests/no_phrase_hard_coding.rs`, 1070 lines, `std`
only, no `Cargo.toml` change — inside this item's collision boundary exactly.
Detection is about *behavioural coupling* and reports the matched shape, as
criterion 1 and the context section require: the shapes it looks for are a
literal full clue string, a comparison against a normalised target string, a
whole-sentence equality test, a lookup table keyed by the target text, and a
per-word or per-substring special case. Test modules and comments are excluded
by rule (`test_modules_and_comments_are_not_scanned`), and
`the_allowlist_is_small_and_every_entry_justifies_itself` is a guard on the
allowlist itself, which is the reviewable-and-greppable requirement.

**Criterion 2's positive control was re-run by this coordinator rather than
taken from the agent's report**, because that is the criterion a fence lives or
dies by and because [w-9d4e17](w-9d4e17.md)'s test was rejected in this queue
for lacking one. A temporary comparison-against-target-text line was appended
to `src/approx.rs` on the **integrated** tree and the fence failed with file,
line and reason:

```text
/workspace/madgab/src/approx.rs:554  [comparison-against-target-text]
    "wreck a nice beach" spells out a whole clue phrase and is compared
    against a runtime value
no_phrase_specific_hard_coding_in_src ... FAILED
```

`src/approx.rs` was then restored from a copy taken beforehand and
`git status` showed an empty `src/` diff, and the fence went back to 6/6. So
the fence has a demonstrated catch, not just a demonstrated pass.

**Criterion 5, on the integrated tree, after [w-c4e8d7](w-c4e8d7.md)'s 331-line
`src/lib.rs` diff was already merged in:** `cargo test --release --test
no_phrase_hard_coding` is **6 passed, 0 failed** (0.01 s, no release build or
corpus needed, as the context section required). That is the stronger form of
the criterion, because the fence is green against the largest `src/` change in
this queue and not only against the tree it was written on. `--lib` 50/50 and
`--test corpus_integration` 10 passed / 1 failed
(`approximate_finds_classic_madgab_resegmentation`) were measured by this
coordinator on the same head.

**Item state: `done`.** All six criteria are objectively verified. The one
criterion that needed independent evidence — the positive control — was
reproduced rather than trusted, and it fired. This item is a fence, so `done`
means the fence exists and works, not that the milestone is met; the milestone
is [w-4b1e07](w-4b1e07.md)'s to close and is still open.

No further work is owed by this item. A later pass that adds a legitimate
allowlist entry is expected, and the test's own module docs say how; that is
maintenance of a landed fence, not a new front.
