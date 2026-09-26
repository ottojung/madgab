---
work_item: true
id: w-5d03af
state: done
owner: agent-a1b2c305
updated: 2026-09-26T09:40:00Z
priority: normal
branch: madgab-exact-determinism
worktree: /workspace/madgab-exact-determinism
---

# Make exact-mode search deterministic

## Goal

`SearchMode::Exact` returns a different set of clues from one process to the
next for the same target and configuration. Exact mode is the mode with the
strongest reproducibility claim, so this should be fixed.

## Context

Found while reviewing [w-4b1e07](w-4b1e07.md). Eight consecutive runs of

```sh
target/release/madgab --top 20 "I love you"
```

on `post-milestone-acceptance` produced two different outputs
(`5e3c036e...` five times, `e029334...` three times). Approximate mode is
deterministic as of commit 6250ba3, so this is a pre-existing exact-mode
defect, not a regression from that work.

Probable cause: `phonetics-rs` 0.3.1 `Corpus::from_json` iterates
`entry.ipa` (a `HashMap<String, String>`) and calls `trie.insert(p.clone())`
for every source, so the order of terminations at a shared trie node
follows `HashMap` iteration order. `Trie::words_starting_at` then yields
those pronunciations in that order, and `insert_top_k` breaks ties by
beam position, so which of two equal-`cheap_score` hypotheses survives
depends on the process's hash seed. That is third-party code; the fix
belongs in `generate_exact` in `src/lib.rs`.

## Completion criteria

- Repeated runs of the same exact-mode invocation produce byte-identical
  output.
- A regression test covers it. A subprocess-based test is the honest
  boundary here, because the defect is a property of the process's hash
  seed and cannot be reproduced inside a single process; if a
  same-process test is preferred, it must at least assert that the search
  does not depend on `HashMap` iteration order.
- No change to the *set* of exact-mode candidates beyond removing the
  order dependence.
- `cargo test --release` is green.

Not yet implemented. If the fix has to sort `words_starting_at` output,
check that it does not cost measurable time in exact mode; exact mode is
already the fast path and the sort key should be a plain
`(consumed, word, ipa)` ordering.

## Handoff / notes

Not yet started. If the fix has to sort `words_starting_at` output, check
that it does not cost measurable time in exact mode; exact mode is already
the fast path and the sort key should be a plain
`(consumed, word, ipa)` ordering.

---

## Handoff — Antonina agent a1b2c305

Branch `madgab-exact-determinism`, one commit on top of `d46d154`, touching
`src/lib.rs` (`generate_exact` only) and adding `tests/exact_determinism.rs`.

### Confirmed root cause

Verified by reading the vendored source, not taken on faith:

`/home/lubko/.cargo/registry/src/index.crates.io-*/phonetics-rs-0.3.1/src/transcriptions.rs`

- `RawEntry.ipa` is `HashMap<String, String>`, and `Corpus::from_json`
  iterates `for (source, ipa) in entry.ipa` and the outer
  `for (word, entry) in raw` (also a `HashMap`), pushing into
  `entries`/`by_word` and then `trie.insert(p.clone())`.
- `Trie::insert` appends to `Node.terminations: Vec<Pronunciation>`, so the
  order of terminations at any shared node follows that hash iteration
  order. (The `BTreeMap` in `Node::children` is fine; the nondeterminism
  is the `terminations` vec, not the child edges.)
- `Trie::words_starting_at` yields `Node::terminations` in slice order.
- `generate_exact` fed those into `insert_top_k`, which replaces the beam
  only on a *strictly* greater `cheap_score` and otherwise leaves the
  incumbent in place — i.e. ties are broken by arrival order, hence by the
  process hash seed.

So the defect is exactly as hypothesised, and it is third-party; the fix
belongs in our code. `generate_approximate` does not use
`words_starting_at` (it uses `approx::FuzzyLexicon`, whose shortlists are
sorted), which is why 6250ba3 left approximate mode deterministic.

### Fix

`generate_exact` now collects the trie walk for each target position,
filters by `min_word_ipa_chars` as before, sorts by
`(consumed, word, ipa)`, and only then feeds the beam. The walk does not
depend on the beam hypothesis, so it is hoisted out of the per-`Partial`
inner loop and the sort is paid once per target position, not once per
hypothesis. Two entries that compare equal under that key (same consumed
span, spelling and IPA) extend every partial into an identical `Partial`,
so the key is total for our purposes and the candidate *set* is unchanged.

### Determinism before the fix (reproduced)

Recipe (run from the worktree root, release binary built at `d46d154`):

```sh
cargo build --release
for i in 1 2 3 4 5 6 7 8; do
  target/release/madgab --top 20 "I love you" 2>/dev/null | sha256sum
done
```

Output — two distinct results across eight runs:

```
a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
8cb0b0b139eb5e42f7240c42bf3fb96ca4ab62b3e950943632d65fbc6aadcb61  -
a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
8cb0b0b139eb5e42f7240c42bf3fb96ca4ab62b3e950943632d65fbc6aadcb61  -
```

Extended to 24 runs: 15× `a55a0649…`, 9× `8cb0b0b1…`. The two outputs
differ by one clue and the rank shift it causes:

```
$ diff before-A.txt before-B.txt
2,12c2,11
<  2. [0.915] i'll of hugh      # present in A only
<  3. [0.910] i'll of u
...
```

### Determinism after the fix

```sh
cargo build --release
for i in $(seq 1 24); do
  target/release/madgab --top 20 "I love you" 2>/dev/null | sha256sum
done | sort | uniq -c -w64
```

```
     24 a55a06494a993330f5fa73ce178d346b1b2e9ea959dbcc916e18ab0147d61fff  -
```

24/24 byte-identical, and the surviving output is bit-for-bit one of the
two pre-fix outputs — i.e. the fix removes the order dependence without
changing the candidate set away from an already-reachable one.

Wider sweep, 10 runs each, hashing stdout:

```
DETERMINISTIC --top 20 I love you
DETERMINISTIC --top 20 It's just a stupid game
DETERMINISTIC --top 20 --beam 16 --max-rarity 20000 recognize speech
DETERMINISTIC --top 20 --min-word-len 3 I love you
DETERMINISTIC --top 20 --approximate I love you   (untouched, still green)
```

### Regression test

`tests/exact_determinism.rs` —
`exact_mode_is_reproducible_across_processes`. It runs the real binary
eight times via `env!("CARGO_BIN_EXE_madgab")` with `--top 20 "I love you"`
(default mode is `SearchMode::Exact`) and asserts stdout is byte-identical.
It compares stdout only: stderr carries the corpus-load and search timings,
which are wall-clock and vary by design.

Subprocess boundary is the honest one — the defect is a function of the
process hash seed, so no in-process test can observe it. It is a real
`tests/` integration test, not a doctest (rustdoc is not installed here).

Negative control — the same test against the unfixed `src/lib.rs`:

```sh
git stash push src/lib.rs
cargo test --release --test exact_determinism
```

```
test exact_mode_is_reproducible_across_processes ... FAILED
assertion `left == right` failed: exact-mode output for "I love you" differed on run 3 of 8
test result: FAILED. 0 passed; 1 failed
```

So the test genuinely covers the defect.

### Sort cost in exact mode

Measured with a 20-word target whose search is ~20–30 ms (so the sort is
visible rather than lost in sub-millisecond noise), 21 runs each side, same
machine, binary rebuilt for each side:

```
AFTER  runs=21 min=25 median=29 max=40
BEFORE runs=21 min=26 median=30 max=41
```

Indistinguishable — the sort is a wash, and the `min_word_ipa_chars` filter
now runs before the sort rather than during the inner loop. For the
original `"I love you"` target the search is 0 ms both before and after.
Timing script: `/tmp/opencode/w5d03af/bench.js` (host-local scratch, not
committed).

### `cargo test --release`

```
cargo test --release --lib      → 11 passed; 0 failed
cargo test --release --test exact_determinism → 1 passed; 0 failed
cargo test --release --test corpus_integration → 4 passed; 2 FAILED
```

The two failures are **pre-existing on the base commit `d46d154`** and are
approximate-mode only — `approximate_finds_classic_madgab_resegmentation`
and `approximate_finds_recognize_speech_resegmentation` in
`tests/corpus_integration.rs`, i.e. the deliverable of the still-open
[w-4b1e07](w-4b1e07.md). Verified by re-running that test file with
`src/lib.rs` stashed: identical failure list and identical printed
proposal lists. They are outside this front (approximate mode) and outside
this item (determinism), so they were left alone. Because of them the
"cargo test --release is green" criterion is **not** met repo-wide on this
branch; it is met for everything this front owns.

### Item state

Left `state: open`. The determinism criteria are objectively verified and
committed on this branch, but the item's "`cargo test --release` is green"
criterion is not met repo-wide, because of the two pre-existing
approximate-mode failures documented above, which belong to w-4b1e07. I did
not mark the item done on the strength of "green everywhere" when it is not.

### Not verifiable on this host

- `cargo fmt`, `cargo clippy` and doctests are not installed, so none were
  run. The diff is hand-formatted to the surrounding style; the file has
  since had long lines rewrapped by hand, so expect `cargo fmt` to differ.
- `docs/environment-notes.md` does not exist in this worktree (only
  `docs/continuation-approximate-search.md`, `docs/skills/`,
  `docs/work/`), so its instructions could not be consulted. Everything
  here follows the constraints in the item text and the prompt.
- `grep`/`sed`/`awk`/`python3` are absent, as stated; all ad-hoc scripting
  was done with `node`.
- No `git push` to `main`, and nothing merged into `main`.
## Pass 2026-09-26T09:40Z (coordinator, integration review)

Integrated into `post-milestone-acceptance` as a merge of
`origin/madgab-exact-determinism` (`1a242b8`). `main` untouched.

Review performed before integrating:

- read the full `src/lib.rs` diff (35 lines). It sorts each target
  position's corpus trie walk by `(consumed, word, ipa)` and hoists the
  walk out of the per-hypothesis loop. General, no target-specific case,
  and confined to `generate_exact`, so it cannot interact with the four
  approximate fronts editing the same file.
- `git grep -i -E "wreck|beach|recognize|justice|stupid|dupe|came|hid"`
  over the branch hits only `src/lib.rs` unit tests, explanatory comments,
  and pre-existing `src/main.rs` doc examples. Nothing in production code.
- the new `tests/exact_determinism.rs` is a real subprocess test, which is
  the correct boundary for a process-hash-seed defect, and it compares
  stdout only because stderr carries wall-clock timings.
- validation on the integrated tree is recorded below.

On the `cargo test --release` criterion: the two failures in
`tests/corpus_integration.rs` (`approximate_finds_classic_madgab_resegmentation`,
`approximate_finds_recognize_speech_resegmentation`) reproduce identically on
the base commit `d46d154`, are approximate-mode only, and are owned by
[w-4b1e07](w-4b1e07.md). This front is therefore closed `done` on its own
criteria; repo-wide green-ness is carried by w-4b1e07, not re-opened here.
`cargo fmt`, `cargo clippy` and doctests remain unverifiable on this host,
per [../../environment-notes.md](../../environment-notes.md).