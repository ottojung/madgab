# Unreachable-object recovery — 2026-09-28

Provenance for `docs/work/unreachable-patches/`. Archived by coordinator pass
`coord-3d5f` (nineteenth pass) on its own branch per standing rule 5. **Pushed,
not merged, and not a merge candidate.**

These patches come from the object class that standing rules 10, 11 and 12 do
not cover: **objects held by no ref and no reflog.** `git rev-list --all` only
sees commits a ref reaches, so a commit that has been left behind by a removed
ref and an expired reflog is invisible to it — and equally invisible to the
file sweep, because the sweep asks whether live files hash to *reachable*
blobs. Such objects are held by nothing and are the **first** things `git gc`
prunes. See the new standing rule 13.

## `0088d27c-w-b3e91a-src-lib-rs.diff`

* **Commit:** `0088d27c06c4555069efb2389756f1d26c4415b8`
* **Subject:** `w-b3e91a: correct the emission-ceiling funding claim and instrument both ceilings`
* **Authored:** 2026-09-27T05:43:50Z
* **Parent:** `f2b2f1b00ace84734e26714719d14f4140f13e11`
* **Content:** one file, `src/lib.rs` (248 872 bytes), +8.8 KB of instrumentation
  that instruments both emission ceilings.
* **Held by:** nothing. `git for-each-ref --contains 0088d27c` returns nothing,
  and no reflog entry mentions it. `git fsck --unreachable` lists it.
* **Item:** `w-b3e91a`, which is **closed** — `04132a5` records the emission
  front as a priced negative, and `a49fed3` merged its branch. So the *result* of
  this front is safe; what was lost is the instrumented `src/lib.rs` itself.

## What was deliberately *not* archived

`202aef9fb4abe226e6e0043552c93ce9e66be9ca`, a stash untracked-files commit from
`madgab-approx-runtime` dated 2026-09-26, is unreachable in the same way. It
holds 18 blobs: the `prof/{results,sum}*.txt` harness outputs and the two 30 MB
instrumented binaries `prof/madgab-prof` and `prof/madgab-baseline`.

It is **not** recovered, for two reasons already on record rather than new
judgement: the text files are regenerable outputs whose inputs are durable at
`docs/work/probe-inputs/` (standing rule 8), and **the live worktree still
contains all 18 files**, so nothing is actually at risk from this object. It is
recorded here so a later pass does not re-derive it as a gap.

## Fence note — read this before ever promoting any of it

`0088d27c` is scratch instrumentation and contains canonical phrases as probe
literals. It sits under `docs/`, which `tests/no_phrase_hard_coding.rs` does not
scan — that fence walks `src/`, `web/` and `examples/` only — and
`ALLOWLIST_CAPS` is unchanged. This is documentation of a past measurement, not
production coupling. **If any of it is ever promoted, its phrase literals must be
removed as part of that promotion, not waived.**
