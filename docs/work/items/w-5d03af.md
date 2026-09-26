---
work_item: true
id: w-5d03af
state: open
priority: normal
owner: null
updated: 2026-09-26T07:05:00Z
branch: null
worktree: null
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

## Handoff / notes

Not yet started. If the fix has to sort `words_starting_at` output, check
that it does not cost measurable time in exact mode; exact mode is already
the fast path and the sort key should be a plain
`(consumed, word, ipa)` ordering.
