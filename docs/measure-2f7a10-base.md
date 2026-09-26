# w-2f7a10 measurement front — baseline on the parent head

SCRATCH. Measurement only. Nothing here is a fix, and nothing here may be
merged. Branch `scratch/2f7a10-base`, forked from `post-milestone-acceptance`
at `fe4ad78` ("w-2f7a10: claim the pairing front"). The implementer
`2f7a11` works in `/workspace/madgab-pairing` on `madgab-pairing-2f7a10` and
nothing here touches it.

No production behaviour is changed, no test assertion is modified, no
`axes::*` / `select_diverse` / `SPAN_*` / `EMIT_*` constant is touched, and no
budget is re-set. The only edit is a read-only dump hook.

## What is instrumented, and where

One hunk, `src/lib.rs`, immediately **before** `select_diverse(clues,
self.config.top_n)` in `Generator::generate`'s approximate path:

```rust
#[cfg(not(target_arch = "wasm32"))]
if let Ok(path) = std::env::var("ZZ_POOL_OUT") {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (i, c) in clues.iter().enumerate() {
        let _ = writeln!(out, "{}\t{:.17}\t{:?}\t{:?}", i, c.score, c.phrase, c.cuts);
    }
    std::fs::write(&path, out).expect("zz pool dump");
}
```

It sits *after* the sort at `src/lib.rs:1780` and after the
`phrase_signature` dedup at `src/lib.rs:1791-1792`, and *before*
`select_diverse` at `src/lib.rs:1821`. So the dump is the **deduplicated pool
the release binary actually produces on the default path**, in global score
order, with rank 0 = best. Nothing about the search is widened, re-budgeted or
re-sampled to produce it.

Offsets are `madgab::Clue::cuts`, taken verbatim from the dump. They are not
re-derived from clue-word IPA lengths and not re-derived from the target.

Non-intrusiveness, checked rather than asserted: the 50 visible clues for
`It's just a stupid game` are byte-identical with `ZZ_POOL_OUT` unset and set
(`cmp /tmp/a.txt /tmp/b.txt`).

## Admissibility of each number

Admissible (default path, release build, deduplicated pool, offsets from
`Clue::cuts`):

* every pool size, distinct-structure count and visible-cutoff score below;
* every pool-membership count, best-member rank, best-member score, gap and
  fill rank below;
* the wall clock below, which is end-to-end process time of the default path.

**Not** admissible and **not** used here: nothing. No shortlist rank, no
per-slot candidate index, no traversal trace, no re-budgeted harness. There is
no instrumented-or-wider harness on this branch at all — the single probe is a
dump of the pool the real binary built.

## Reproduction

```sh
cd /workspace/madgab-pairbase
git checkout scratch/2f7a10-base
cargo build --release

# canonical target: pool dump + the default-path trace
MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
  ZZ_POOL_OUT=/tmp/pool_stupid.txt \
  ./target/release/madgab --approximate --top 50 "It's just a stupid game"

# the 50 visible clues, to map visible slots back to pool ranks
./target/release/madgab --approximate --top 50 "It's just a stupid game" > /tmp/vis_stupid.txt

# six further real targets (i = 1..6)
for t in "a whole lot of trouble" "the cat sat on the mat" \
         "put it back on the shelf" "when the rain finally stopped" \
         "he was a big fat man" "what are you going to do"; do
  i=$((i+1))
  ZZ_POOL_OUT=/tmp/t$i.txt ./target/release/madgab --approximate --top 50 "$t" > /tmp/t${i}_vis.txt
done

# wall clock, end-to-end process time, 4 reps each
node -e 'const cp=require("child_process");
for(const t of ["a whole lot of trouble","the cat sat on the mat",
  "put it back on the shelf","when the rain finally stopped",
  "he was a big fat man","what are you going to do"]){
  const r=[];for(let k=0;k<4;k++){const s=process.hrtime.bigint();
    cp.execFileSync("./target/release/madgab",["--approximate","--top","50",t],{stdio:"ignore"});
    r.push((Number(process.hrtime.bigint()-s)/1e9).toFixed(2));}
  console.log(t,r.join(" "));}'
```

Analysis scripts used (throwaway, under `/tmp`, not committed):
`/tmp/an.js` (pool summary + structure drill-down), `/tmp/fillrank.js`
(visible slot -> pool rank mapping), `/tmp/w.js` (per-word membership and slot
positions), `/tmp/deep.js` (deepest non-target words per target). Each is a
few lines of `node` over the tab-separated dump.

## 1. The five-word input of the second canonical acceptance example

`It's just a stupid game`, `--approximate --top 50`, release, default path,
offsets from `madgab::Clue::cuts`:

```text
pool (deduplicated)                17906
distinct structures               333
visible score cutoff at rank 49   0.915691888
  (pool rank 49, "it justice too day mm", cuts [2,10,14,18,19])

MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17906
```

```text
canonical structure [3, 10, 13, 15, 19]
  members in the pool                 47
  best member wording                 "each justice too add gain"
  best-member global pool rank        3108
  best member score                   0.8781284673865422
  gap to the visible cutoff           -0.037563
  fill rank                            104   (the 50th and last visible slot
                                               sits at pool rank 104)
  the requested wording               NOT among the 47
  visible slots held by this structure   0
```

Both members of the pair, as pool membership on this path (for the later
pass's benefit, and it is the sharpest number on this page):

```text
"dupe" in pool        14   best pool rank 4061  slot positions 2:6 3:7 4:1
                           (every one of the 14 is at a NON-LEADING slot)
"hid"  in pool         5   best pool rank 7504  slot positions 0:4 4:1
                           (4 of the 5 are at the LEADING slot 0)

co-occurrence in any single pool candidate:
  hits + dupe        0
  hid  + dupe        0
  hits + hid         0
  dupe + came        0
  too  + dupe        1     pool rank 12260  "each justice too dupe add gave"
                            cuts [3,10,11,13,15,19]
```

So the seam the work item names is reproduced exactly: each word is reachable
on its own, `dupe` at 14 pool candidates and always off the traversal's best
index, `hid` at 5, and **the two are never enumerated in the same structure**.
The requested wording is not a scoring or selection question; it is a
tuple-space question. `too + dupe` occurring exactly once, and then at cuts
`[3,10,11,13,15,19]` rather than the canonical `[3,10,13,15,19]`, is the
nearest thing in the pool to the requested pairing.

Pool size is **17906** here. `w-c4e8d7`'s end-state note records 17907 for the
same tree; `MADGAB_TRACE ... missing candidates=17906` and the 17906 dump
lines agree, and `src/` is byte-identical between `58a45b0` and `fe4ad78`, so
17907 in that record looks like an off-by-one in the note rather than a
different tree. Treat 17906 as the number a later pass must be compared
against. Every other canonical number (333, 0.915691888, 47, 3108,
0.8781284673865422, -0.037563, 104, "each justice too add gain") reproduces
exactly.

## 2. Six real targets other than the two acceptance examples

The general question — is a deep alternative reachable in a **non-leading**
slot, and how many pool candidates is it in? — answered on the default path,
release build, for six targets that are neither `It's just a stupid game` nor
`recognize speech`. "Deep" is defined here with no reference to either
acceptance phrase: a pool word that is not one of the target's own words, and
"non-leading" is a word position `>= 1` in the clue (position 0 is the slot
the traversal is best at).

```text
target                          pool  structures  cutoff(rank 49)  fill rank
a whole lot of trouble           16151      326     0.895695875        49
the cat sat on the mat           16262      330     0.893683280        50
put it back on the shelf         15633      314     0.894965791        51
when the rain finally stopped    15610      352     0.914965805        60
he was a big fat man             19164      313     0.895134791        49
what are you going to do         14544      309     0.905780912        54
```

```text
target                        deep alt reachable in a non-leading slot?  members  best rank  slot pos
a whole lot of trouble         yes                                        1      16051      7  ("delve")
the cat sat on the mat         yes                                        3       1672      1  ("tickets")
put it back on the shelf       yes                                        8      15011      6  ("louis")
when the rain finally stopped  yes                                       13      15572      3  ("aar")
he was a big fat man           yes                                        1      18860      6  ("honour")
what are you going to do       yes                                        2      14438      7  ("perdue")
```

**Yes on all six.** A second witness per target, for corroboration, and one
negative case worth recording because it is the shape a naive reading of the
seam would predict:

```text
target                        word        members  best rank  slot positions  non-leading
a whole lot of trouble        dwell          1      16043      7:1             1
                              pulp           1      16041      7:1             1
                              misfit         0          -        -             0
the cat sat on the mat        tossed         1      15837      2:1             1
                              capture        1      15781      1:1             1
put it back on the shelf      taught        28       2311      0:28            0   <- leading only
                              beef           4      15033      7:4             4
                              fool           4      15034      7:4             4
when the rain finally stopped copper         1      15536      3:1             1
                              mist           1      15523      5:1             1
he was a big fat man          inner          1      18855      6:1             1
what are you going to do      duper          2      14430      7:2             2
                              dougie         2      14422      7:2             2
```

Aggregate over the whole pool, non-target words only:

```text
target                        non-target words in pool  of which reachable at position >= 1
a whole lot of trouble                          1766                        1619
the cat sat on the mat                          1773                        1662
put it back on the shelf                        1760                        1533
when the rain finally stopped                   2231                        2035
he was a big fat man                            1580                        1421
what are you going to do                        1499                        1387
```

Two things a later pass should not misread. First, `taught` on
`put it back on the shelf` is reachable in 28 pool candidates and is at the
**leading** slot in all 28 — a word can be well-populated and still confined
to slot 0, which is the same confinement the canonical `hid` shows, and it is
why the "non-leading" half of the question is the half worth generalising.
Second, `misfit` is 0 on `a whole lot of trouble`: not every deep word is
reachable, so a later pass must report its own zeros and not infer reachability
from "the reserve is now a uniform sweep".

## 3. Wall clock, `--approximate --top 50`, this parent head

End-to-end process time of the release binary, 4 reps per target, this session,
on `fe4ad78` + the read-only `ZZ_POOL_OUT` hunk:

```text
a whole lot of trouble            1.82  1.84  1.76  1.75
the cat sat on the mat            1.99  1.92  1.96  1.94
put it back on the shelf          1.95  1.96  1.94  1.93
when the rain finally stopped     2.82  2.46  2.46  2.26
he was a big fat man              1.79  1.72  1.74  1.75
what are you going to do          1.82  1.83  2.95  2.04
It's just a stupid game (ref)     2.68  3.44  2.98  2.82
```

This is a **baseline only**. It is not comparable to the 1.77-2.51 s column of
`w-7b2d40`, and `w-c4e8d7` already recorded that column as host noise by
measuring the parent at 1.82-3.21 s in the same session. A later pass must
measure interleaved against *this* build in *its* session, not against either
column.

## 4. Suites on this branch, unchanged code plus the dump hunk

```text
cargo test --release --lib                        50 passed; 0 failed
cargo test --release --test corpus_integration    10 passed; 1 failed
  approximate_output_is_locked                              ok   (NOT re-baselined)
  approximate_finds_recognize_speech_resegmentation        ok   (the guard)
  approximate_pool_reaches_alternatives_past_the_opening_slot_width  ok
  approximate_finds_classic_madgab_resegmentation           FAILED (unchanged)
cargo test --release --test approx_determinism     2 passed
cargo test --release --test exact_determinism      1 passed
```

The single failure is the acceptance test this work item exists to fix. It is
untouched here: `tests/` has an empty diff on this branch.

`cargo fmt`, `cargo clippy`, doctests and `wasm32` builds do not exist on this
host (see `docs/environment-notes.md`) and are not claimed.

## What is not established here

* No line-level diagnosis of where the pairing is dropped. This front measured
  the pool; it did not instrument `coverage_tuples`, `sweep_index`,
  `EMIT_PROFILE_MAX_DEEP`, `EMIT_PROFILE_RESERVE` or the emission budget, so
  the rejecting quantity and the count of tuples the traversal does and does
  not offer for `[3,10,13,15,19]` are still open. That is the implementer's
  criterion 1 and it is not answered by this page.
* No claim that the fix is cheap. The wall clock above is a baseline, not
  headroom.
* Nothing about selection, admission order, share cap or axis weights, which
  are closed territories and were not touched or re-litigated.

## Branches

* `scratch/2f7a10-base` — this commit. Scratch. **Must never be merged.**
* `madgab-pairing-2f7a10` (`/workspace/madgab-pairing`) — the implementer
  `2f7a11`'s branch. Not touched, not merged into, not merged from.
