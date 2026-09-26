---
work_item: true
id: w-7c4a91
state: superseded
priority: high
owner: null
updated: 2026-09-26T12:40:00Z
branch: post-milestone-acceptance
worktree: null
superseded_by: w-4b1e07
---

# Continue approximate-search acceptance work

This document is an active repository work item. It is intentionally discoverable by scheduled orchestrators without relying on GitHub Issues. Follow [skills/work-items.md](skills/work-items.md) for claiming, handoff, and completion.

## Superseded (2026-09-26, coordinator coord-5e1d)

The work moved to [work/items/w-4b1e07.md](work/items/w-4b1e07.md), which
carries the live measurements, blocker list and pass history. Its
completion criteria are the same acceptance goal, restated with the
constraints this document predates. The archival branch list, the
milestone tag and the strongest-findings notes below are kept as history
and are still accurate, but this file is no longer the place to record
progress: a coordinator discovering it now should read
[w-4b1e07](work/items/w-4b1e07.md) instead.

Two completion criteria in the list at the end of this document cannot be
verified on this host at all (`cargo fmt --check`, `cargo clippy`, and the
doctest/`wasm32` parts of `cargo test`/`cargo build`); see
[environment-notes.md](environment-notes.md). Use
`cargo test --release --lib` and `cargo test --release --test <name>` and
say plainly which criteria were unverifiable.

## Goal

Continue the approximate-search quality work until the normal approximate generator produces both canonical acceptance cases without phrase-specific hardcoding:

- `It's just a stupid game` → `hits justice dupe hid came`
- `recognize speech` → `wreck a nice beach`

Both should appear in default approximate mode with `top_n = 50`. Keep the search generic, deterministic, and interactive.

## Stable milestone

- tag: `approximate-search-milestone-2026-09-25`
- commit: `c0ecd7ce756fe524a918c17b133a3ab39b3500d2`

## Main continuation branch

Continue from `post-milestone-acceptance`. Use the GitHub branch head as source of truth.

## Archived local experiments

All meaningful previously-local source/test work was preserved on GitHub:

- `archive/local-bound-2026-09-25` — `b987b5e5ba5b22f17932111f571e47cd07ca3ea9`
- `archive/local-ci-2026-09-25` — `46efa5e66eb740bced948319482668d7d62cad80`
- `archive/local-clone-2026-09-25` — `3132e619f72b69672988fa7a696621df737a73a7`
- `archive/local-costalign-2026-09-25` — `4864116db3f7f49fb16341e16b9770403ba0abe7`
- `archive/local-dag-2026-09-25` — `bd9be8322f165abeec0906e8b4e2d57acff8c53d`
- `archive/local-diag-current-2026-09-25` — `9a7959e36afe5bda709e3b1560b37b8d78365a76`
- `archive/local-diag-joint-2026-09-25` — `8533fe5bb8ac88b99b21e5771542deb5a0170fb7`
- `archive/local-dp-2026-09-25` — `1fc8867706a8d2226cdec3a14beb7bb4466f6d19`
- `archive/local-ending-2026-09-25` — `9add95a22295435b44403581d3465b714f60b8fc`
- `archive/local-familiarity2-2026-09-25` — `0cd60b1850fc15e83c4926b7bc17cd14fbf21cdb`
- `archive/local-final-2026-09-25` — `477a66408ef3b360e9f024e624c023346cdd262f`
- `archive/local-global-2026-09-25` — `d5d74f22b346aaec7ae17735d3a38591221a0cb0`
- `archive/local-late-2026-09-25` — `3d0ec432aadf7fe9b4ff8d334d00a93a820081b9`
- `archive/local-late2-2026-09-25` — `f7f3a883b1bf4e5f759aea120dddad964eb70c5b`
- `archive/local-mine-2026-09-25` — `ee72c274f6456bc2f47d194a5abdcfb428c3af3a`
- `archive/local-objective-2026-09-25` — `2878fd266bc94fb384b6759cbf38683c3477ff14`
- `archive/local-oracle-2026-09-25` — `8f48e59452165e5bf95891f98ada9f9e2d36882c`
- `archive/local-proxy-2026-09-25` — `5382f6d907202604e30573756687516258d20446`
- `archive/local-quality-2026-09-25` — `0fcfae108bb4d5c071844d646b5eb30648c43e49`
- `archive/local-state-2026-09-25` — `f6a95af70b8e3488f495a2bdd4ffeeeecef09e5f`
- `archive/local-web-2026-09-25` — `5c1a116a7343a983e7b84d9aabc73ddf995496bc`

These branches are archival references. Inspect/cherry-pick useful ideas; do not merge them wholesale.

The generated `target-hcollapse/` directory was intentionally not pushed because it is build output, not source work.

## Strongest findings

1. Both canonical word sequences are reachable in the fuzzy matcher/lattice. The blocker is search/ranking.
2. In the archived CI experiment, `wreck a nice beach` reached the raw final candidate list around rank 42 but was removed by final MMR diversity.
3. The classic target segmentation survived structural search, and the canonical words were present in their span alternatives; the full phrase was lost in lexical combination enumeration.
4. Increasing global beam/reservoir sizes caused 60–100+ second runtimes and is not acceptable.
5. Online admission/eviction reserves were order-sensitive. Prefer deferred/order-independent selection.
6. Structural segmentation survival and lexical choice should remain separated, with both stages tightly bounded.
7. Final diversity should not discard genuinely top-scoring candidates simply because they overlap with earlier picks.

## Suggested continuation

Inspect especially:

- `archive/local-ci-2026-09-25` for k-best lexical-combination and final-selection diagnostics.
- `archive/local-dag-2026-09-25` and `archive/local-dp-2026-09-25` for bounded lattice/DP approaches.
- `archive/local-quality-2026-09-25` for pre-consolidation search experiments.
- `archive/local-bound-2026-09-25`, `archive/local-objective-2026-09-25`, and `archive/local-costalign-2026-09-25` for score/bound alignment ideas.

A promising direction remains:

1. Build approximate word/span matches once.
2. Retain a small order-independent portfolio of structurally distinct segmentations.
3. For a fixed segmentation, exploit additive final-score components and enumerate bounded k-best lexical products.
4. Rerank only bounded complete candidates with the exact final scorer.
5. Preserve a score-ranked core before diversity, or scale diversity penalty to the score spread.
6. Benchmark every iteration in release mode.

## Completion criteria

- Both exact acceptance regressions pass unignored with default approximate mode / `top_n = 50`.
- No production hardcoding of either phrase or its words.
- Reasonable outputs for at least `I love you` and `hello there how was your day`.
- Interactive release-mode runtime.
- `cargo fmt --check`
- `cargo test --no-fail-fast`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release`
- `cargo build --release --target wasm32-unknown-unknown`
- CI green before merge.
