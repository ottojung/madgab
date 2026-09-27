---
work_item: true
id: w-3a7f0d
state: working
priority: normal
owner: agent-3a7f0e (claimed 2026-09-27T04:28Z by coord-b7e2 from post-milestone-acceptance 36589f8)
updated: 2026-09-27T04:28:00Z
opened_by: agent-d0f11f, from w-d5a2c1 (fence/test-truth audit at 42ced98)
branch: madgab-fence-3a7f0d
worktree: /workspace/madgab-fence-3a7f0d
agents: 3a7f0e
---

# Extend the no-phrase-hard-coding fence to `web/` and `examples/`, and record the real CLI argv behaviour

## Why this item exists

[w-d5a2c1](w-d5a2c1.md) is a read-only audit of the accumulation tip. It measured
the phrase sweep over `src/`, `web/` and `examples/` and found the tree clean,
but it also found two gaps in the *fences themselves* rather than in the
production code. Neither is a defect to fix in `src/`; both are gaps in what the
guard test is able to see, plus one stale factual claim in the audit item.

Measured at `post-milestone-acceptance` = `42ced98`, 2026-09-27T04:21Z.

## Finding 1 — `tests/no_phrase_hard_coding.rs` only scans `src/`

**Exact site:** `tests/no_phrase_hard_coding.rs:806-816`, `fn src_files()`.

```rust
fn src_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(src_dir())   // src_dir() = <manifest>/src
        ...
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("rs"))
```

Every consumer of the scan is rooted at `src_dir()`. So a
phrase-specific hard-code committed under `web/app.js` or
`examples/measure.rs` — both of which already carry the canonical phrases
today, benignly — would **not** be caught by
`cargo test --release --test no_phrase_hard_coding`. The fence's own module
docs scope it honestly ("in `src/`"), so this is a coverage limit, not a
false claim. But the limit is invisible from the test name, and `web/app.js` is
a real user-facing search entry point.

**Today's hits, for the record** (all currently benign, none machine-checked):

| file:line | content | classification |
|---|---|---|
| `web/index.html:20` | `value="It's just a stupid game"` | UI default input value |
| `web/index.html:21` | `placeholder="e.g. It's just a stupid game"` | UI placeholder |
| `examples/measure.rs:31` | `"recognize speech",` | benchmark harness target list |
| `examples/measure.rs:32` | `"It's just a stupid game",` | benchmark harness target list |

`web/app.js:21,26,64,68,95` also match the sweep, but only on the substring
`hid` inside the DOM property `hidden`; they are real code and are not phrase
hits. `web/index.html:42,47` are the `hidden` attribute on two elements.

**Suggested shape of the fix** (not to be done here): add a sibling
`web_files()` / `examples_files()` walker over the same `detect()` unit
machinery, extend `ALLOWLIST` to name a file *and* a directory, and keep the
existing `the_allowlist_is_small_and_every_entry_justifies_itself` bound. The
four rows above would each need a one-line allowlist entry with a reason —
which is precisely the visible, greppable diff the fence's design wants.

## Finding 2 — the CLI argv quirk recorded in w-d5a2c1 does not reproduce

**Exact site:** the "Canonical case 2" paragraph of
[w-d5a2c1](w-d5a2c1.md)'s Handoff, which records that `"It's ..."` cannot be
passed as a single argv word here and that the exit-1 spellings are "not
search failures".

Measured, from the release binary, all three spellings quoted so the shell
passes one word each:

```sh
./madgab --approximate --top 50 "It's just a stupid game"   # exit 0
./madgab --approximate --top 50 "Its just a stupid game"    # exit 0
./madgab --approximate --top 50 "It is just a stupid game"  # exit 0
```

The apostrophe form exits **0** and its printed 50-line set is
`diff`-identical to the `Its` form. The third spelling is *not* identical — it
is a different target (extra `is` word) and a different top-10.

So the correct, reproducible fact is narrower: **`It's` and `Its` are
interchangeable here and produce identical output**; only the word count of the
target changes the result. A later pass that treats the apostrophe form as
unrunnable will silently measure the wrong thing.

## Non-finding, recorded so it is not re-raised

`tests/approx_determinism.rs:247,261,301` reads an env var
`APPROX_POOL_HELPER_TARGET`. It is an env-var knob, and the w-d5a2c1 fence
prohibits env-var knobs. It is **not** filed as a defect: it lives in
`tests/`, never in `src/`, and `approximate_pool_helper` returns immediately
unless a parent driver sets the variable, so the default run is unaffected
(`--test approx_determinism` is 4/0 with it unset). The knob is how a *later*
pass can obtain a fresh-process pool dump, which is the mechanism w-5c11a2 is
already using. Flagged here only so that a later sweep of "env-var knobs" does
not re-open it as new.

## Fences

- **Do not** change scoring, search, ranking, or any assertion.
- Do not make a red test pass. `approximate_finds_classic_madgab_resegmentation`
  is red at `42ced98`; that is a measurement, not a defect.
- No phrase-specific hard-coding anywhere, and do not introduce any — including
  in the detector's own fixtures.
- `CARGO_TARGET_DIR` outside `/tmp` (it is `noexec`).
- Never `main`. This is not a search/ranking, objective-axes,
  cost/pronunciation/dictionary, or emission front; all of those are refuted
  by measurement in the items listed in [w-d5a2c1](w-d5a2c1.md).

## Completion criteria

- [ ] `no_phrase_hard_coding` scans `web/` and `examples/` as well as `src/`,
      or the scope limit is stated in the test's *name* and the module docs and
      a reviewer cannot miss it.
- [ ] The four existing benign hits each have a justified allowlist entry, or
      are excluded by an explicit, documented region rule.
- [ ] `web/index.html:42,47` and `web/app.js` `hidden` occurrences do not
      produce false positives from the `hid` substring.
- [ ] `cargo test --release --test no_phrase_hard_coding` still reports 7/0
      (or, if a test is added, its exact new count is recorded here).
- [ ] The w-d5a2c1 Handoff's Canonical-case-2 paragraph is corrected in place
      to the measured fact, and the correction is pushed to
      `post-milestone-acceptance`.
- [ ] `cargo fmt` and `cargo clippy` are **not** claimed as run; they do not
      exist on this host (see `docs/environment-notes.md`).

## Handoff / notes

Opened read-only by the w-d5a2c1 audit at `42ced98`. That item is a report and
explicitly must not fix what it finds; this file is where the fix belongs.
`web/app.js` is the only UI file that is a real search entry point, so it is the
one that matters for the fence extension.
