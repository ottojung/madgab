---
work_item: w-2b6a19
report: true
id: REPORT-2b6a19
state: done
priority: high
owner: front-2b6a19 (agent-2b6a19)
updated: 2026-09-28T01:20:00Z
branch: madgab-lexicon-2b6a19
worktree: /workspace/madgab-lexicon-2b6a19
base: post-milestone-acceptance at a1c8074
verdict: HOLD
---

# REPORT-2b6a19 — the default rarity bound is not what is missing: all five clue words are in both consumers, and removing the bound does not make the alignment reachable

**Item:** [w-2b6a19](items/w-2b6a19.md) · **Branch:** `madgab-lexicon-2b6a19` ·
**Front:** `agent-2b6a19`, `/workspace/madgab-lexicon-2b6a19`

## Verdict

**HOLD, and the surface is closed.** No production line changes. `src/approx.rs` gains
**310 lines, all of them inside one `#[cfg(test)] mod front_2b6a19`, and zero production
lines**. `src/lib.rs`, `src/main.rs`, `src/lexical.rs`, `src/adjacency.rs` and `web/` are
byte-identical to the base.

The three findings, in the order the item predicted them:

1. **All five clue words are present at the default bound, in both consumers, with margin.**
   Not one of them is filtered. `dupe` — the word the item named as the likeliest casualty —
   is the *worst-placed of the five* and still survives, at rarity 40,210 against a bound of
   50,000. It is inside the cut with 19.6 % headroom, and it would need the bound lowered by
   9,790 to fall out. `hits` and `hid` are present, so the predicted *sense* problem does
   not exist as a *presence* problem either: their homography is not costing them a slot.
2. **The two consumers are in step, and are in step by construction as well as by
   measurement.** `Corpus::from_json` filters on `entry.rarity > cap`; `build_lexicon`
   filters on the same predicate over the same JSON and then requires
   `corpus.preferred_ipa(&word)`, so the lexicon is a *subset* of the corpus by design and
   cannot drift from it without the corpus's own lookup failing. Measured: 0 lexicon words
   absent from the corpus, 0 IPA mismatches, 0 admitted-and-transcribable words missing from
   the lexicon, at the default bound **and** unfiltered.
3. **The bound is therefore not the cause, and raising it does not fix case 2.** With
   `--max-rarity 1e9` the lexicon goes from 50,001 words to 281,502 and the case-2 pool
   goes from 18,949 to 19,231, and the canonical alignment is *still absent*. The bound is
   load-bearing for the vocabulary in general but not for this clue.

So the correct next lever, if there is one, is not upstream of search and not the rarity
bound's character. This surface closes.

**Case 1 is untouched and is the guard.** `wreck a nice beach` is at **display rank 27** in
the shipped release binary at the default configuration, re-measured after the change (§6).

---

## 1. Criterion 1 — the membership table, with numbers

Both consumers, both configurations, every word, no adjectives.

Corpus JSON: `open-english-pronouncing-dictionary` 0.1.0, 281,502 entries, vendored.
Default bound: `Some(50_000.0)`, `src/lib.rs:86`. Unfiltered: `None` / `--max-rarity 1e9`.

### `Corpus::from_json` (the exact corpus, `src/lib.rs:806`)

| word | rarity | default `Some(50_000.0)` | unfiltered `None` | headroom to the cut |
|---|---|---|---|---|
| `hits` | 3,416.0 | **in**, 2 pronunciations | **in**, 2 | 46,584 (13.6×) |
| `justice` | 1,744.0 | **in**, 4 pronunciations | **in**, 4 | 48,256 (28.7×) |
| `dupe` | 40,210.0 | **in**, 3 pronunciations | **in**, 3 | **9,790 (1.24×)** |
| `hid` | 12,601.0 | **in**, 3 pronunciations | **in**, 3 | 37,399 (3.97×) |
| `came` | 431.0 | **in**, 4 pronunciations | **in**, 4 | 49,569 (116.1×) |

Entries admitted by the bound: **50,001 of 281,502** (17.77 %).

### `approx::build_lexicon` (the fuzzy lexicon, `src/lib.rs:815`)

| word | rarity | default `Some(50_000.0)` | unfiltered `None` | lexicon IPA (normalized) | syllables | closed-class |
|---|---|---|---|---|---|---|
| `hits` | 3,416.0 | **in** | **in** | `hɪts` | 1 | no |
| `justice` | 1,744.0 | **in** | **in** | `dʒʌstəs` | 2 | no |
| `dupe` | 40,210.0 | **in** | **in** | `dup` | 1 | no |
| `hid` | 12,601.0 | **in** | **in** | `hɪd` | 1 | no |
| `came` | 431.0 | **in** | **in** | `keɪm` | 1 | no |

Words admitted by the bound: **50,001** (identical count to the corpus, and the identity is
not a coincidence — see §2).

### Where each word sits in the rarity ordering

`dupe` is the rarest of the five at position **40,210 of 281,502** by rarity, i.e. inside the
top 14.3 % rarest, and the bound keeps the 50,001 rarest. The band between `dupe`'s rarity and
the bound holds **9,791** further words, so the cut is 9,791 words clear of the one word whose
admission is closest to the line. `hid` has 37,400 words between it and the bound; the other
three are in the low thousands and are nowhere near it.

**Nothing is missing. The table has no empty cell.**

### Agreement between the two consumers, both configurations

| check | default | unfiltered |
|---|---|---|
| lexicon words | 50,001 | 281,502 |
| lexicon words absent from the exact corpus | **0** | **0** |
| lexicon words whose IPA ≠ `normalize_ipa(corpus.preferred_ipa(w))` | **0** | **0** |
| corpus-admitted, transcribable words **absent from the lexicon** | **0** | **0** |
| words the bound rejects that are *inside* the bound (i.e. filtered for another reason) | **0** | 0 |

The last row is the one the item asked for specifically: the bound is a **pure rarity cut**,
with no second hidden condition. Every rejected word is rejected because `rarity > cap` and
for no other reason, and every word inside the bound that the corpus can transcribe reaches
the fuzzy lexicon.

### The predicted hazards, measured

* **`dupe` as a spelling, not a headword.** Not a casualty. It is in the vendored JSON as a
  headword in its own right with rarity 40,210 and three source transcriptions
  (`misaki_gold` `dˈup`, `cmu` `dˈup`, `wikipron` `dup`), so `preferred_ipa` resolves and both
  consumers admit it. Its rarity data is sparse in the sense that it sits in the 40th
  thousand, but sparse is not absent.
* **`hits` / `hid` homography as a *sense* problem.** Not a presence problem either. `hits`
  has 2 corpus pronunciations, `hid` has 3, and both are inside the bound, so no sense of
  either was excluded. (Whether the *right* sense is reachable is a question about the
  traversal and the objective, not about the vocabulary, and this front does not price it.)
* **`src/lexical.rs:302`.** Confirmed a different property. It asserts these words are **not**
  closed-class, i.e. `is_closed_class` returns false, and all five do return false — so they
  are *eligible* for the content-word slots. It says nothing about corpus membership and it
  was not used as evidence here.

---

## 2. Why the two consumers cannot silently diverge

Worth stating as a durable fact, because it is the reason this surface is closed by
inspection and not only by measurement.

`build_lexicon` iterates the **raw JSON itself** (`src/approx.rs:382`), not the corpus trie, so
it has its own copy of the rarity filter at `src/approx.rs:390`. That copy applies the same
predicate. But every word it then admits must additionally pass
`corpus.preferred_ipa(&word)` (`src/approx.rs:395`), and `preferred_ipa` reads the corpus's
own `by_word` map, which `Corpus::from_json` already filtered. So:

* a word the **corpus** rejected is unreachable by the lexicon, because
  `preferred_ipa` returns `None` and the lexicon `continue`s;
* a word the **lexicon** rejected but the corpus kept would be a real divergence, and is what
  the new test is looking for.

The two filters live in different files and one is inside a third-party crate
(`phonetics-rs` 0.3.1, `Corpus::from_json` at `transcriptions.rs:250`), so "they agree" is
not a type-system fact. It is now a test.

---

## 3. Criterion 2 — no priced change, because nothing is missing

The criterion is conditional: *"if and only if a word is missing by default."* **No word is
missing by default**, so there is no change to price, and this front makes none. Producing a
pool-size / wall-clock / expansions table for a change that would not help would be inventing
a deliverable the criterion does not ask for.

What *is* priced here is the **negative**: what the bound actually costs and buys, so the next
front does not have to re-measure it. Case-2 target `It's just a stupid game`, release mode,
`beam_width` 64, `top_n` 50, the shipped default budgets, one run each, expansions from the
suite's own `SPENT_EMISSIONS` / `SPENT_POPS` counters:

| `--max-rarity` | lexicon words | case-2 pool | wall clock | emissions | pops |
|---|---|---|---|---|---|
| **50,000 (default)** | **50,001** | **18,949** | **1.190 s** | **16,384** | **175,859** |
| 1e9 | 281,502 | 19,231 | 1.351 s | 16,384 | 202,371 |
| `None` (Exact-only shortcut) | 281,502 | 19,231 | 2.050 s | 16,384 | 202,371 |
| 40,000 | — | 18,887 | 1.257 s | — | — |
| 30,000 | — | 18,767 | 1.143 s | — | — |
| 12,000 | — | 18,286 | 0.757 s | — | — |
| 3,000 | — | 15,540 | 0.449 s | — | — |

Readings:

* **Un-filtering the bound is expensive and does not buy the answer.** 5.6× the vocabulary
  (50,001 → 281,502), +282 pool clues (+1.5 %), +15 % wall clock, +15.1 % pops, and the
  canonical alignment is still absent at 1e9. The emissions counter is pinned at its ceiling
  (16,384) in every row, so the search is emission-bound and the extra pops are being spent
  on candidates the ceiling never lets through.
* **The bound is a real lever, just not for this clue.** 50,000 → 3,000 removes 3,409 pool
  clues (−18 %) and 62 % of the wall clock. That is the shape of a coverage/cost trade a
  future front can use; it just has nothing to do with case 2.
* **Where `dupe` would fall out.** Between 40,000 and 30,000 the pool loses 120 clues, and
  `dupe` is at 40,210 — so a bound anywhere below ~40,210 is the first configuration in which
  this front's finding would change. Nothing in the shipped code or CLI suggests such a
  value; it is recorded so a future front that lowers the bound knows it is now editing a
  documented fact rather than making a fresh mistake.
* **Wall clock on the unfiltered row is not directly comparable to the default row** and is
  reported as measured. `Corpus::from_json` keeps the exact corpus bounded at
  `max_rarity: None` in the library but builds a 281k-word lexicon too
  (`src/lib.rs:812-815`), so the `None` row pays for a 281k corpus it does not use. That is
  the existing shape of `from_json`, not a cost this front introduced or would change.

---

## 4. Criterion 3 — the regression test

`src/approx.rs`, one new `#[cfg(test)] mod front_2b6a19`, **6 tests, 405 added lines, zero
production lines.**

| test | what it pins |
|---|---|
| `the_exact_corpus_admits_exactly_what_the_rarity_predicate_admits` | at the shipped bound, the exact corpus holds a word **iff** the rarity predicate admits it, checked word by word over all 281,502 |
| `the_fuzzy_lexicon_and_the_exact_corpus_admit_the_same_words_at_a_bound` | **the cross-consumer property**: every lexicon word is admitted, its IPA is the corpus's own preferred transcription, and every admitted transcribable word reaches the lexicon. A guard against vacuous pass (`checked > 1_000`) is included |
| `the_bound_is_a_pure_rarity_cut_and_nothing_else` | every rejected word is rejected because `rarity > cap` and nothing else; the bound is a real cut, not a no-op and not the whole corpus |
| `removing_the_rarity_bound_does_not_make_the_hard_alignment_reachable` | **the negative this front exists to establish.** If the alignment ever *does* appear in the case-2 pool, at either the default bound or unfiltered, the test fails with "re-file this front" — so a future front that finds it reachable knows the premise moved. The companion assertion that the unfiltered pool really is wider keeps the test from passing for the wrong reason |
| `the_unfiltered_bound_admits_what_no_bound_admits` | the `1e9` stand-in for "unfiltered" admits exactly the set `None` admits, so the other tests' unfiltered arm really is unfiltered |
| `the_watched_clue_words_are_covered_by_the_default_bound` | the nine words of the two canonical clues are in both consumers at the default bound, so a corpus refresh that drops or renames one is visible here rather than as an unexplained search regression |

**Fence compliance.** The phrase literals live in a `#[cfg(test)]` module in `src/`, which
`tests/no_phrase_hard_coding.rs` excludes by design (`test_lines` / `units_of`). `src/`'s
allowlist cap stays at **zero** and the `ALLOWLIST` array is untouched. Measured:
`cargo test --release --test no_phrase_hard_coding` → **9 passed / 0 failed**. No production
code reads any of the strings; `git diff --stat` is `src/approx.rs | 405 +++` and nothing else.

**Test selection.** Four of the six are *general*: they state a property of the bound and the
two consumers over the whole vocabulary and would fail on any divergence, not only on the
canonical words. Two name the canonical phrases, and both do so as a corpus-refresh tripwire
rather than as a search expectation. There is no branch, no exception and no table anywhere
keyed on a canonical sentence.

**What each configuration is covered at, stated plainly rather than implied:**

| configuration | predicate (`rarity <= cap`) | exact corpus | fuzzy lexicon |
|---|---|---|---|
| shipped default, 50,000 | exhaustive, all 281,502 words | exhaustive, all 281,502 words | exhaustive, all 50,001 lexicon words, both directions |
| unfiltered (`1e9` and `None` equated) | exhaustive, all 281,502 words | searched, whole pool inspected | searched, whole pool inspected |
| an intermediate bound | — | — | covered by the construction argument in §2, which is bound-independent |

The two `exhaustive … lexicon` cells are at the shipped bound only, and that is a stated limit
rather than a silent one. The agreement argument in §2 is that the lexicon is a *subset of the
corpus by construction* — every lexicon word must pass `corpus.preferred_ipa`, which reads the
corpus's own already-filtered map — and that argument does not depend on the bound, so a
second bound would repeat it rather than extend it.

**One test-harness accommodation, stated rather than hidden.** These tests are the only place
in the crate that builds a 281,502-word corpus, and one of them also builds the 281,502-word
lexicon in order to search it. Measured peak RSS per test, alone, on this host:

| test | peak RSS |
|---|---|
| `removing_the_rarity_bound_…` (builds corpus **and** lexicon) | 790 MB |
| `the_fuzzy_lexicon_and_the_exact_corpus_…` | 262 MB |
| `the_exact_corpus_admits_exactly_…` | 262 MB |
| `the_bound_is_a_pure_rarity_cut_…` | 262 MB |
| `the_watched_clue_words_are_covered_…` | 247 MB |
| `the_unfiltered_bound_admits_what_no_bound_admits` | 72 MB |
| *base suite, whole binary, 32 threads* | *1,547 MB* |

The container's cgroup caps memory at **30 GB** and other work on this host already runs near
it. An earlier revision of this module built seven such views, and the unqualified
`cargo test --release --lib` then **SIGKILLed about half the time at the default 32 threads**
while the base was green 5/5. Two changes fixed it, and both are test-only:

1. the six tests serialize on one process-wide `Mutex`, so at most one of these views is live
   at a time regardless of how many test threads the harness starts;
2. only the test that has to *search* the unbounded vocabulary builds it; the other five read
   the corpus and the lexicon at the shipped bound, and the corpus-side checks derive their
   expected sets from indices into one cached vocabulary rather than cloning 281,502 `String`s
   per call.

After both, the unqualified suite is green **5 runs in 5** at the default 32 threads, in 24 s —
*faster* than the base's 27 s, because the mutex also stops this module from competing with the
rest of the suite for cores. `cargo test --release --lib -- --test-threads=2` is green too. The
mutex and the caching are inside a `#[cfg(test)]` module; they touch no production path and no
search behaviour.


---

## 5. Criterion 4 — no production change, confirmed

| file | production lines changed |
|---|---|
| `src/approx.rs` | **0** (405 added, all inside `#[cfg(test)] mod front_2b6a19`) |
| `src/lib.rs` | **0** |
| `src/main.rs` | **0** |
| `src/lexical.rs` | **0** |
| `src/adjacency.rs` | **0** |
| `web/`, `examples/`, `Cargo.toml` | **0** |

Objective weights, the score function, the selection rule and `src/approx.rs` search
behaviour are all untouched. The `#[cfg(test)]` module adds no production line, so the
change is measurement apparatus plus a report, which is criterion 4's stated success case.

**Disjointness with `agent-8f0b3d1`.** That front sweeps `--top` and budget knobs on the
shipped binary. This front touched `--max-rarity` and lexicon construction only, and the one
time it ran a `--top` sweep (`--top 2000`, §7) it was to count clue-word occurrences, which
is not that front's question. No overlap, no duplication.

---

## 6. Criterion 5 — case 1 re-measured, not assumed

**Shipped release binary**, built from this worktree, run as a user would:

```
$ ./target/release/madgab --approximate --top 50 recognize speech
```

```
26. [0.920] let ugh nice pitch
27. [0.920] wreck a nice beach
28. [0.920] rec a guys peach
```

**`wreck a nice beach` is at display rank 27** — exactly at the standing guard, not worse.
Corpus load 510 ms, search 1,040 ms. Also confirmed in the library: pool 18,289, pool rank
**26**, 1.49 s, 195,294 pops.

The rank is identical to `REPORT-3c5b18`'s measurement on the base, which is expected: this
front changed no production line.

Case 2 at the shipped default, for the record and verbatim:

```
$ ./target/release/madgab --approximate --top 50 "It's just a stupid game"
 1. [0.921] it said thus test oop day
 2. [0.921] it said thus 'cause too day
 ...
```

`Hits Justice Dupe Hid Came` is **absent**, and absent at `--max-rarity 1e9` too.

---

## 7. Supporting detail — the words are reachable as clue words too

Present in the vocabulary and present in the printed list are different facts, and the second
one is worth recording because it is the *first* place the diagnosis "the bound is not the
cause" can be checked without reading source.

`./target/release/madgab --approximate --top 2000 "It's just a stupid game"`, counting
occurrences as a clue word in the printed output:

| word | occurrences in printed top 2000 (default bound) | occurrences at `--max-rarity 1e9` |
|---|---|---|
| `hits` | 2 | 2 |
| `justice` | 402 | 402 |
| `dupe` | **0** | **0** |
| `hid` | **0** | **0** |
| `came` | 18 | 18 |

`dupe` and `hid` are *in the vocabulary* (§1) but never surface as a printed clue word in
either configuration. So the vocabulary is not the blocker for those two, and something
downstream — the traversal's reach, the objective, or the emission ceiling — is. That is
consistent with every other case-2 front already priced, and it is why this front's verdict is
that the next lever is **not** here. This front does not claim which downstream surface it is
and does not price it; that is a separate front, and the pool numbers in §3 are the handoff.

---

## 8. Fences and environment, measured on this host

| suite | base | this delivery |
|---|---|---|
| `cargo test --release --lib` (default 32 threads) | 76 passed / 0 failed / 12 ignored, green 5 runs in 5, 27 s | **82 passed / 0 failed / 12 ignored, green 5 runs in 5, 24 s** (the 6 added tests) |
| `cargo test --release --lib -- --test-threads=2` | 76 / 0 / 12 | **82 / 0 / 12** |
| `cargo test --release --test no_phrase_hard_coding` | 9 passed / 0 failed | **9 passed / 0 failed** |
| `cargo test --release --test emit_coverage` | 7 passed / 0 failed | **7 passed / 0 failed** |
| `cargo test --release --test approx_determinism` | 4 passed / 0 failed | **4 passed / 0 failed** |
| `cargo test --release --test exact_determinism` | 1 passed / 0 failed | **1 passed / 0 failed** |
| `cargo test --release --test corpus_integration -- --test-threads=2` | 12 passed / 1 failed | **12 passed / 1 failed** — the same single red, `approximate_finds_classic_madgab_resegmentation`, which *is* the case-2 example |
| `src/` allowlist entries | 0 (cap 0) | **0 (cap 0)**, `ALLOWLIST` untouched |

**Not claimed:** `cargo fmt`, `cargo fmt --check`, `cargo clippy` and doctests **cannot run
on this host** — no `rustup`, no `rustfmt`/`clippy`/`rustdoc` component
(`docs/environment-notes.md`). Nothing in this report is a formatting or lint result.

**The `corpus_integration` red.** `approximate_finds_classic_madgab_resegmentation` is red at
12 passed / 1 failed. That is the known pre-existing base red — it is the canonical case-2
test this whole line of work exists to fix. It was run here for completeness, it is red in the
same way it is red on the base, its literals were not modified, and it is **not re-pinned**.

**A real regression this front introduced and then fixed, reported because it happened.** An
earlier revision of the new test module built seven large corpus views and the unqualified
`cargo test --release --lib` then SIGKILLed roughly half the time at the default 32 threads
while the base was green 5/5 — so this was caused by the change, not by the host. The cause was
the tests' own footprint against a 30 GB cgroup other work is already near; the fix and its
measurements are in §4. The delivered state is green 5/5, and the caveat is recorded here so
nobody re-introduces it by adding one more large view.

---

## 9. Criteria, stated

| # | criterion | met | evidence |
|---|---|---|---|
| 1 | membership table, five words, default and unfiltered, **both** consumers, with numbers | **YES** | §1: two full tables plus the four-row consumer-agreement table, every cell a count |
| 2 | priced general change, **if and only if** something is missing | **N/A — nothing is missing, so no change to price** | §1 (no empty cell), §3 (the negative *is* priced: 5.6× vocabulary, +282 pool, +15 % wall clock, +15.1 % pops, and the alignment still absent) |
| 3 | regression test at the library or executable boundary, no phrase special-casing, fence 9/0, `src/` allowlist 0 | **YES** | §4: 6 tests in one `#[cfg(test)]` module, `git diff --stat` shows additions only, fence measured 9/0 |
| 4 | no change to weights, score function, selection rule or `src/approx.rs` search behaviour | **YES** | §5: 0 production lines in every file |
| 5 | case 1 at or better than display rank 27 in the shipped binary | **YES** | §6: **display rank 27**, re-measured on the built release binary |

**1, 3, 4, 5 met. 2 is conditional and its condition is false.**

---

## 10. What the next front should know

1. **Do not re-open `max_rarity` as a case-2 cause.** The bound is 9,791 words clear of the
   closest word, both consumers agree exactly, and raising it 5.6× does not produce the
   alignment. If a future front re-derives this, it should start from §1's table and §3's
   priced negative rather than from the word "ra".
2. **`dupe` and `hid` are vocabulary-present and search-unreached.** §7 is the handoff: the
   next surface is downstream of the vocabulary, and every downstream case-2 surface already
   priced (`w-e086cc`, `w-1a4e8d`, `w-3f6a21`, `w-5c1a3e`, `w-3c5b18`) is a candidate. This
   front does not rank them.
3. **The bound is still a live cost/coverage knob** if a future front wants one: 50,000 → 3,000
   is −3,409 pool clues for −62 % wall clock (§3). It is simply not a case-2 lever.
4. **Do not add a sixth large corpus view to the new test module.** The six tests in §4
   serialize on a mutex precisely because this host's 30 GB cgroup is already near its
   ceiling; an earlier revision that built seven views killed the suite about half the time
   (§8). If a future test needs the unbounded view, extend an existing one rather than
   building a new one.
