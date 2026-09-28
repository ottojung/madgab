---
work_item: true
id: w-c1d3a7
state: open
priority: high
owner: agent-c1d3a7
updated: 2026-09-26T00:00:00Z
branch: madgab-adjacency
worktree: /workspace/madgab-adjacency
---

# An adjacency / neighbourhood operator over the emitted pool: make depth in one
# slot additive instead of multiplicative

## Goal

Land a general neighbourhood (adjacency) operator in the approximate search so
that a wording which is jointly excellent but **locally expensive in one slot**
is reachable at an affordable budget, and so that the canonical Mad Gab
resegmentation `hits justice dupe hid came` for `It's just a stupid game` appears
in the visible top 50.

## Context

The mechanism this item owns, and the arithmetic behind it.

The approximate search enumerates, for one segmentation, the Cartesian product
of its slots' candidate lists with a bounded best-first traversal. A wording is
proposed only when the traversal *reaches* it by cumulative cost, so reaching a
wording that is deep in one slot costs the sum of every better-bound node in
front of it. Depth in one slot is therefore **multiplicative** in reach:
`64` emissions of a segmentation buy the cheap corner and essentially nothing
else.

**Measured facts already in the tree** (all reproduced below, none of them
assumed):

* The canonical resegmentation, forced to index 0 of all five of its slots, is
  a genuine in-pool candidate — emission 23,962 / pop 28,930 of its own
  segmentation, structural rank 151 of 256. Only the *path* to it is expensive.
* By best-first order it needs **>= 404,081 emissions** of its own segmentation
  against a per-segmentation allowance of **64**
  ([w-8f3c61](w-8f3c61.md) §8).
* The cost-model front was **refuted by measurement**
  ([w-3b8e15](w-3b8e15.md)), so the order itself is not going to move.
* The global budget is **inert 1x..256x** ([w-be6d21](w-be6d21.md)), so raising
  it buys nothing.
* The integrated staged width schedule **fires 0 times** on multi-clause
  targets ([w-9d4e17](w-9d4e17.md)) — which is exactly the canonical target's
  shape.

The depth-profile reserve that *is* integrated (w-9d4e17) walks whole depth
profiles from the index-0 corner; it samples the index-tuple space, but it
starts from index 0 in every non-profile slot and so cannot reach a wording
whose *every* slot is deep. The operator this item adds starts from wordings
**already emitted into the pool** and substitutes one slot at a time. That is
what makes depth in one slot **additive**: one substitution away from a
parent that the traversal did reach, rather than the whole product in front of
it.

## Mechanism

From the wordings already emitted for a segmentation, for each emitted
wording, for each slot, substitute that slot's alternative, re-score, and admit
to the pool if the build succeeds and the tuple is new. Bounded by a named
allowance so the pool stays inside the global emission budget, and by a named
per-slot width so a long list cannot turn the operator into a second product
enumeration.

## Completion criteria

- [ ] The operator is **general**: no phrase-specific case. The canonical words
      (`hits`, `justice`, `dupe`, `hid`, `came`, `wreck`, `nice`, `beach`, and
      their substrings) must never appear in production `src/`. Measurement
      harnesses read their targets from the environment and live in `/tmp`.
- [ ] No `axes::*` move.
- [ ] `approximate_output_is_locked` is not re-baselined without a justified
      before/after table in this item.
- [ ] `cargo test --release --lib`, `--test corpus_integration`,
      `--test exact_determinism`, `--test approx_determinism` run and are
      reported. The single known failure
      `approximate_finds_classic_madgab_resegmentation` is this item's target.
- [ ] `wreck a nice beach` for `recognize speech` still appears, and its **rank
      is recorded**.
- [ ] ENUMERATED and RANKED are reported **separately**, always.
- [ ] The operator lands in `src/lexical.rs` or a new module, with a small
      clearly marked call site. Agent `5f1c04` owns the traversal loop in
      `src/lib.rs`; if the call site must touch that region, this item says so
      explicitly rather than editing silently.
- [ ] No `zz_*` test, sweep script, `.bench/` tree or `MADGAB_*` diagnostic
      left on the branch.

## Fences

- A refutation with numbers is an acceptable and useful outcome. Several
  families in this repository have closed that way. Do not manufacture a
  favourable number.
- `cargo fmt`, `cargo clippy` and doctests do not exist on this host; see
  [../../environment-notes.md](../../environment-notes.md). Do not report them
  as satisfied.

## Handoff / notes

Filed by coordinator pass, forked from `post-milestone-acceptance` at `769733d`
onto branch `madgab-adjacency`. Baseline recorded on arrival: `cargo test
--release --lib` 39 passed / 0 failed; `--test corpus_integration` 9 passed /
1 failed, the failure being `approximate_finds_classic_madgab_resegmentation`.
`approximate_finds_recognize_speech_resegmentation` passes at baseline.
