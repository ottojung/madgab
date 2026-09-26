# w-2f7a10 independent verification — post-change measurement

SCRATCH. Measurement and review only. Branch `scratch/2f7a10-verify`, forked
from `a58a49b` ("w-2f7a10: sample the traversal's index space as points, not
diagonals"), which is the implementer `2f7a11`'s landed commit, pushed as
`origin/madgab-pairing-2f7a10`. Nothing is merged here, `scratch/2f7a10-base`
is never merged, and nothing is pushed to `main`.

## Headline

**The fix did not land.** On the default approximate path, release build, the
requested wording `Hits Justice Dupe Hid Came` is in the deduplicated pool
**0 times** — the target the item exists to move from 0 to non-zero is still 0.

**And the change cost pool breadth on every target measured, and broke a guard
it was required to keep green.** This is a first-class finding, not a footnote,
and it is set out in full in section 1b below:

* `approximate_pool_reaches_alternatives_past_the_opening_slot_width` passes on
  the parent `fe4ad78` and **fails** on `a58a49b`, on `tickets` for
  `the cat sat on the mat` — the same regression the pool measurement finds
  independently of the test (`tickets` 3 pool candidates -> 0).
* Aggregate non-leading word reachability falls on **all six** targets, from
  9657 distinct non-target words reachable at position >= 1 to 8546, a loss of
  **1111 words (-11.5%)**. This is a systematic narrowing, not a reshuffle.
* Three previously-reachable deep words are gone (`tickets` 3 -> 0,
  `capture` 1 -> 0, `copper` 1 -> 0) and **none of the six designated witnesses
  improved**.
* The visible `--top 50` output is **byte-identical** on every target checked, so
  `select_diverse` masks the entire pool loss from the user and from
  `approximate_output_is_locked`. The guard is the only thing that catches it.

Criterion 3 fails, criterion 5 fails. Wall clock did **not** regress, replicated
across two independent sessions.

## Instrumentation on this branch, and its non-intrusiveness

One hunk in `src/lib.rs`, immediately before `select_diverse(clues,
self.config.top_n)` in `Generator::generate`'s approximate path, ported from
`scratch/2f7a10-base` (read from that branch; **not** merged):

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

It sits *after* the sort and *after* the `phrase_signature` dedup at
`src/lib.rs:1877`, and *before* `select_diverse` at `src/lib.rs:1925`. So the
dump is the **deduplicated pool the release binary actually produces on the
default path**, in global score order, rank 0 = best. It reads nothing, writes
no counter, re-budgets nothing, re-samples nothing, and is unreachable from the
accumulation branch — it is past the whole search, immediately upstream of the
selector.

Non-intrusiveness, checked rather than asserted:

```sh
cargo build --release
B=./target/release/madgab
$B --approximate --top 50 "It's just a stupid game" > /tmp/ni_off.txt 2>/tmp/ni_off.err
ZZ_POOL_OUT=/tmp/pool_ni.txt $B --approximate --top 50 "It's just a stupid game" > /tmp/ni_on.txt 2>/tmp/ni_on.err
cmp /tmp/ni_off.txt /tmp/ni_on.txt          # -> STDOUT BYTE-IDENTICAL
cmp /tmp/ni_off.err /tmp/ni_on.err          # -> differs at line 3 ONLY, and that
                                           #    line is the timing report
```

stdout (the 50 visible clues) is byte-identical with the probe unset and set,
and the same `cmp` was run for all six further targets with no mismatch. stderr
differs only in `(corpus loaded in 448ms; search 1316ms)` vs
`(corpus loaded in 403ms; search 1358ms)`, i.e. in the two timing numbers and
nowhere else.

Fence: `git grep ZZ_POOL_OUT post-milestone-acceptance -- src tests examples`
is **empty** (exit 1). The probe exists only on this scratch branch.

## Admissibility of every number on this page

Admissible — default approximate path, release build, membership of the
deduplicated pool the release binary produced, offsets verbatim from
`madgab::Clue::cuts`:

* every pool size, distinct-structure count and visible-cutoff score;
* every pool-membership count, best-member rank, best-member score, gap, fill
  rank, per-word slot-position histogram and pairwise co-occurrence count;
* the wall clock, which is end-to-end process time of the default path plus the
  binary's own `search NNNms` line, on the real release binary.

**Not** used anywhere on this page: no shortlist rank, no re-budgeted harness,
no widened enumeration, no `select_diverse`-side reasoning. The one probe is a
dump of the pool the unmodified search built.

## 1. CRITERION 3 — post-change membership, default path, release

### The five-word target

`It's just a stupid game`, `--approximate --top 50`, release, default path:

```sh
ZZ_POOL_OUT=/tmp/p_stupid.txt MADGAB_TRACE_PHRASES="hits justice dupe hid came" \
  ./target/release/madgab --approximate --top 50 "It's just a stupid game" > /tmp/v_stupid.txt
```

```text
pool (deduplicated)                17637      (parent: 17906)
distinct structures                  333      (parent:   333, unchanged)
visible score cutoff at rank 49   0.9156918881684132
  "it justice too day mm", cuts [2,10,14,18,19]
fill rank                             105      (parent:  104)
```

```text
canonical structure [3,10,13,15,19]
  members in the pool                  48      (parent:  47)
  best member          "each justice too add gain"
  best-member global pool rank        3171      (parent: 3108)
  best member score  0.8781284673865422      (parent: 0.8781284673865422, BYTE-IDENTICAL)
  gap to the visible cutoff        -0.037563      (parent: -0.037563)
```

```text
requested wording "Hits Justice Dupe Hid Came"
  members in the deduplicated pool       0      <-- TARGET 0 -> non-zero NOT MET
  global rank                       n/a
  score                             n/a
```

Corroborated by the default path's own trace, which is the same predicate the
probe uses (case-insensitive whole-phrase equality against the deduplicated
pool, `src/lib.rs:1884`):

```text
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17637
MADGAB_TRACE raw_cutoff rank=49 score=0.915691888 phrase="it justice too day mm"
```

`missing candidates=17637` equals the pool size, so **no** candidate matches.

**What the change did to this structure** (diffed against the parent's dump of
the same structure): 14 wordings added, 15 removed, net 47 -> 48. Every added
wording varies slot 3 and/or slot 4 only, and **all 48 members keep `each` or a
near neighbour in the leading slot and `justice` in slot 1** — the pairing shape
is now expressed, but never over the two words the item is about. Best member's
score being byte-identical to the parent's confirms again that the reserve
changed *which wordings are enumerated* and not how any of them scores.

### The seam that is actually still closed

Per-word and pairwise membership over the whole 17637-candidate pool, position
index within the clue (`0` = leading slot):

```text
word      members   best rank  slot positions
hits          52         984   0:50  1:2
justice     8011           0   1:4197  2:2896  3:918
dupe           9        2328   2:4  3:5
hid            4        7272   0:3  3:1
came          174          11   3:27  4:72  5:53  6:16  7:6
```

(parent, for contrast: `hits` 46 members all at slot 0; `dupe` 14 members at
slots 2:6 3:7 4:1; `hid` 5 members at slots 0:4 4:1. So `hits` did gain two
non-leading occurrences and `hid` one, but `dupe` **lost five** members.)

```text
co-occurrence in any single pool candidate
  hits + dupe    0
  hits + hid     0
  hid  + dupe    0
  dupe + came    0
  hid  + add     2   (best pool rank 12531, "hid see justice too add gave", cuts [2,3,10,13,15,19])
```

So the pairing is *still* never enumerated, and the pool-membership evidence
says the remaining blocker is **not** the diagonal-vs-point defect the commit
fixed. Enumerating all 9 `dupe` occurrences and all 4 `hid` occurrences:

```text
dupe: cuts [4,7,11,14,18,19] [2,10,14,16,19] [4,10,14,18,19] [3,8,10,13,19]
      cuts [1,3,10,13,14,19] [4,10,14,19]     [2,3,10,14,18,19] [3,10,14,19]
      cuts [2,8,9,13,19]
hid:  cuts [2,11,13,15,19] [2,3,10,13,15,19] [2,3,10,13,15,19] [2,3,10,15,18,19]
```

`hits` is fine — it is the leading word of 44 pool candidates whose `cuts[0]` is
3, including the canonical structure. But `dupe`, whenever it sits in
within-clue position 2 (which is where the requested wording needs it), is in a
candidate whose `cuts[2]` is **14 or 19, never 13**, and it is never in the same
candidate as `hits` at all. So on the admissible evidence the requested tuple
is blocked by a **per-slot candidate-list / offset-coverage** constraint on
`dupe` at offset 13 in within-clue slot 2, downstream of the class order and the
diagonal rule. Fixing the tuple *shape* cannot reach it.

### Six real targets, same table the baseline took

`a whole lot of trouble` … `what are you going to do`, each
`ZZ_POOL_OUT=… ./target/release/madgab --approximate --top 50 "<target>"`:

```text
target                          pool  structures  cutoff(rank 49)  fill rank   (parent pool)
a whole lot of trouble          15713      326     0.895695875        49          (16151)
the cat sat on the mat          15864      330     0.893683280        50          (16262)
put it back on the shelf        15257      314     0.894965791        51          (15633)
when the rain finally stopped   15120      352     0.914965805        60          (15610)
he was a big fat man            18992      313     0.895134791        49          (19164)
what are you going to do        14033      309     0.905780912        54          (14544)
```

The six cutoff scores reproduce the baseline **byte-identically**; every pool
size is *smaller* than the parent's (by 438, 398, 376, 490, 172, 511), and the
structure counts are unchanged on five of six and down one on
`the cat sat on the mat` (330 -> 330, unchanged; `when the rain finally stopped`
352 -> 352, unchanged; `he was a big fat man` 313 -> 313, unchanged;
`put it back on the shelf` 314 -> 314, unchanged; `what are you going to do`
309 -> 309, unchanged; `a whole lot of trouble` 326 -> 326, unchanged — all six
structure counts reproduce exactly).

```text
target                        witness   members  best global rank  slot confinement
a whole lot of trouble        delve          1             15613  non-leading (slot 7) ONLY
                              dwell          1             15605  non-leading (slot 7) ONLY
                              pulp           1             15603  non-leading (slot 7) ONLY
                              misfit         0                 -  UNREACHABLE
the cat sat on the mat        tickets        0                 -  UNREACHABLE  <-- was 3 on the parent
                              tossed         3              7998  non-leading (slot 1) ONLY
                              capture        0                 -  UNREACHABLE  <-- was 1 on the parent
                              misfit         0                 -  UNREACHABLE
put it back on the shelf      louis          8             14635  non-leading (slot 6) ONLY
                              taught        29              2348  LEADING SLOT 0 ONLY  (was 28/0)
                              beef           4             14657  non-leading (slot 7) ONLY
                              fool           5             10288  non-leading (4:1 7:4)
                              misfit         0                 -  UNREACHABLE
when the rain finally stopped aar           13             15076  non-leading (slot 3) ONLY
                              mist           2              4791  non-leading (3:1 5:1)
                              copper         0                 -  UNREACHABLE  <-- was 1 on the parent
                              misfit         0                 -  UNREACHABLE
he was a big fat man          honour         1             18688  non-leading (slot 6) ONLY
                              inner          1             18683  non-leading (slot 6) ONLY
                              misfit         0                 -  UNREACHABLE
what are you going to do      perdue         2             13928  non-leading (slot 7) ONLY
                              duper          2             13921  non-leading (slot 7) ONLY
                              dougie         5             10014  non-leading (2:3 7:2)
                              misfit         0                 -  UNREACHABLE
```

Read plainly, this is a **mixed and net-negative** result for generality:

* Five of the six named witnesses still land in a non-leading slot. `delve`,
  `louis`, `aar`, `honour`, `perdue` reproduce the parent's member counts
  exactly.
* `tickets` — the word the second guard test asserts on — went **3 -> 0**.
  `capture` and `copper`, the baseline's corroborating non-leading witnesses,
  also went to 0. Three previously-reachable deep alternatives became
  unreachable, and none of the six named witnesses improved.
* `tossed` improved 1 -> 3 and `dougie` 2 -> 5, so the change is not inert; but
  it is a reshuffle of *which* deep words are enumerated, and it cost three
  reachabilities to buy two.
* `taught` on `put it back on the shelf` is 29 members and still at the
  **leading** slot in all 29 — the confinement the baseline warned about, and
  the change did not touch it.
* `misfit` is **0 on all six targets**, exactly as the baseline recorded, so my
  own zeros stay visible: reachability is still not uniform.

Pool-level wording churn on the canonical target: **2506 wordings added, 2775
removed** relative to the parent, for a net pool 269 smaller.

## 2. CRITERION 7 — wall clock, interleaved against the parent, one session

Both arms built **pristine** from `git archive` of each commit — no probe in
either, so the comparison is search-vs-search and not probe-vs-no-probe:

```sh
git archive a58a49b | tar -x -C /workspace/wc-verify/new
git archive fe4ad78 | tar -x -C /workspace/wc-verify/base
# cargo build --release in each
node /tmp/wc.js    # 4 reps per target per arm, INTERLEAVED, arm order
                   # alternated each rep, one discarded warm-up of each binary
```

`/tmp` is mounted `noexec` on this host, so the two build trees had to live
under `/workspace/wc-verify/`; that is why the recipe differs from the
baseline's.

End-to-end process time, 4 reps, interleaved, plus the binary's own
`search NNNms` line (the less noisy of the two):

```text
target                        parent e2e (med)   a58a49b e2e (med)   d(e2e)   parent search med   a58a49b search med   d
a whole lot of trouble            1.99 2.06 1.90 1.87   1.94   2.15 2.18 1.96 2.39   2.17   +0.22 s      1350 ms            1524.5 ms   +174.5 ms (+12.9%)
the cat sat on the mat            2.39 2.67 2.29 2.12   2.34   2.30 3.20 2.33 2.08   2.31   -0.02 s      1702.5 ms            1679.5 ms   -23.0 ms ( -1.4%)
put it back on the shelf          2.38 2.37 2.25 2.18   2.31   2.78 2.50 2.26 2.12   2.38   +0.07 s      1627.0 ms            1711.5 ms   +84.5 ms ( +5.2%)
when the rain finally stopped     2.49 2.54 2.67 2.57   2.55   2.80 2.63 2.70 2.58   2.67   +0.11 s      1949.5 ms            2016.0 ms   +66.5 ms ( +3.4%)
he was a big fat man              2.16 1.94 2.06 1.93   2.00   2.29 2.01 2.04 2.03   2.04   +0.04 s      1360.0 ms            1386.5 ms   +26.5 ms ( +1.9%)
what are you going to do          2.17 1.97 2.13 1.87   2.05   1.99 2.47 2.11 1.99   2.05   +0.00 s      1421.0 ms            1431.5 ms   +10.5 ms ( +0.7%)
It's just a stupid game (ref)     1.90 2.64 2.18 2.10   2.14   3.05 2.02 2.11 2.02   2.06   -0.08 s      1517.5 ms            1446.0 ms   -71.5 ms ( -4.7%)
```

**No wall-clock regression.** Every median `search` delta lies between -4.7% and
+5.2% except one, and the arm-to-arm spread on each target overlaps heavily
(e.g. `when the rain finally stopped`, the noisiest target, parent 1862-2047 ms
vs a58a49b 1944-2173 ms — overlapping ranges, +3.4% on the median).

The single outlier, `a whole lot of trouble` at +12.9% on median search, was
re-measured with **8 further interleaved reps** and did not survive:

```text
parent  2.01 1.99 1.83 1.77 1.82 1.85 1.82 1.82 1.97   med 1.83  mean 1.88
a58a49b 1.96 1.98 1.97 1.80 1.81 1.82 1.80 1.86 1.83   med 1.83  mean 1.87
```

Identical. So the answer to criterion 7 is **no regression, and none of the
noisy targets (`when the rain finally stopped`, `what are you going to do`, the
stupid-game reference) moved outside the parent's own spread** — the 1.79-2.82 s
column the coordinator quoted from the baseline session is not comparable to
this one and is not reused as a comparison here.

The wall clock being free is not evidence that the change is cheap in the
traversal; it is evidence that the emitted-tuple *count* did not grow, which is
consistent with the pool being 269 *smaller* on the canonical target.

## 3. THE TWO GUARDS, re-measured after the change

```sh
cargo test --release --test corpus_integration
```

Full result on `a58a49b` + probe: **9 passed; 2 failed**.

```text
approximate_finds_recognize_speech_resegmentation               ok    PASS
approximate_pool_reaches_alternatives_past_the_opening_slot_width  FAILED   FAIL
approximate_finds_classic_madgab_resegmentation                  FAILED  (the item's own target, unchanged)
approximate_output_is_locked                                     ok    (NOT re-baselined)
approximate_pool_reaches_matches_deep_in_a_span                 ok
approximate_proposals_are_predominantly_content_words           ok
approximate_list_is_not_one_resegmentation                      ok
known_madgab_pair_is_searchable                                  ok
approximate_mode_runs_end_to_end                                 ok
canonical_words_have_expected_narrow_ipa                        ok
generates_for_a_real_phrase                                      ok
```

Each guard on its own merits:

* **`approximate_finds_recognize_speech_resegmentation` — PASS.** The
  "wreck a nice beach" guard survives the pool change. This was the guard most
  at risk from a pool-changing fix, and it held.

* **`approximate_pool_reaches_alternatives_past_the_opening_slot_width` —
  FAIL**, and the change is what broke it. Same test, same release build, run
  against the pristine parent in the same session shape for attribution:

  ```text
  a58a49b: the cat sat on the mat: "tickets" missing from a pool of 17355 clues,
            so a real alternative below the traversal's opening slot width was not reached
  fe4ad78: ok
  ```

  This is criterion 5's second half, and it is failed. It is also the same fact
  my pool measurement found independently and without the test: `tickets` goes
  from 3 members in the parent's deduplicated pool to **0** in `a58a49b`'s
  (`capture` 1 -> 0, `copper` 1 -> 0 as well). The fix traded a guard for the
  single-word reachability the guard exists to protect, in exchange for a
  pairing that did not materialise.

## 4. SUITES

```text
cargo test --release --lib                        51 passed; 0 failed
cargo test --release --test corpus_integration     9 passed; 2 failed
                                                   approximate_output_is_locked                    ok (not re-baselined)
                                                   approximate_finds_recognize_speech_resegmentation ok
                                                   approximate_pool_reaches_alternatives_past_the_opening_slot_width FAILED
                                                   approximate_finds_classic_madgab_resegmentation FAILED
cargo test --release --test exact_determinism      1 passed; 0 failed
cargo test --release --test approx_determinism     2 passed; 0 failed
cargo test --release --test no_phrase_hard_coding  6 passed; 0 failed
```

`approx_determinism` is green, so the change is deterministic. The
`no_phrase_hard_coding` fence is green, corroborating the coordinator's
generality review from the other direction. Both `corpus_integration` failures
are accounted for: one is the item's own target and was already failing on the
parent, the other is new and is a guard.

`cargo fmt`, `cargo clippy`, doctests and `wasm32` builds **do not exist on this
host** (`docs/environment-notes.md`) and are not claimed.

## What is not established here

* Nothing about `select_diverse`, admission order, the share cap or axis
  weights; closed territories, not touched and not re-litigated.
* No line-level trace *inside* `coverage_tuples` / `build`'s budget check. The
  `dupe`-at-offset-13 confinement above is established from pool membership and
  from an exhaustive enumeration of the 9 and 4 candidates that contain those
  words — not from a trace of the emission budget, so the *rejecting quantity*
  is still unnamed.
* No claim about whether a corrected reserve would recover `tickets` and
  `capture`. That is the experiment the next measurement should be.

## The single most useful next measurement

Since the fix did not land and it cost a guard, the next measurement should be
**per-slot candidate-list and offset coverage for `dupe` at within-clue slot 2,
on the default path, in the deduplicated pool** — specifically, over the whole
pool, the set of `cuts[2]` values that occur in candidates whose within-clue
position 2 is `dupe`, and the rank/width of `dupe` in the slot-2 candidate list
for the segmentation whose span covers offsets 10..13. That distinguishes the
two remaining hypotheses that pool membership alone cannot separate: (a)
`dupe` is simply absent from the slot-2 list at the width that covers offset 13,
in which case no tuple-space change can ever produce the requested wording and
the whole front is aimed at the wrong seam; and (b) it is present but the
traversal never offers that index, in which case the reserve is still the
problem and the fix is under-reserved rather than mis-shaped. This is the
admissible next step because it is answerable from the same deduplicated pool
dump plus the already-existing `MADGAB_TRACE_SPANS` / `MADGAB_TRACE_WORDS`
hooks, with no harness widening and no re-budgeting.

## 1b. THE POOL REGRESSION, as a first-class finding

Parent `fe4ad78` vs `a58a49b`, both default approximate path, both release, both
the deduplicated pool the binary produced, offsets from `Clue::cuts`. Parent
column from `scratch/2f7a10-base`'s dump of the same target; new column from
this front's dump. Designated witness in bold.

```text
target                        pool(parent->new)  structs(p->n)  cutoff rank49 (p->n)      fill rank (p->n)
a whole lot of trouble          16151 -> 15713     326 -> 326     0.895695875 -> same      49 -> 49
the cat sat on the mat          16262 -> 15864     330 -> 330     0.893683280 -> same      50 -> 50
put it back on the shelf        15633 -> 15257     314 -> 314     0.894965791 -> same      51 -> 51
when the rain finally stopped   15610 -> 15120     352 -> 352     0.914965805 -> same      60 -> 60
he was a big fat man            19164 -> 18992     313 -> 313     0.895134791 -> same      49 -> 49
what are you going to do        14544 -> 14033     309 -> 309     0.905780912 -> same      54 -> 54
```

Every structure count and every visible cutoff score is **byte-identical** to the
parent. Every pool is **smaller**, by 438 / 398 / 376 / 490 / 172 / 511, and
every fill rank is unchanged except the canonical target (104 -> 105).

Designated and corroborating witnesses, membership in the deduplicated pool:

```text
target                        witness      parent      new      verdict
a whole lot of trouble        delve           1  ->      1     unchanged, non-leading (slot 7)
                              dwell           1  ->      1     unchanged, non-leading (slot 7)
                              pulp            1  ->      1     unchanged, non-leading (slot 7)
the cat sat on the mat        tickets         3  ->      0     REGRESSED TO ZERO
                              tossed          1  ->      3     improved 1 -> 3
                              capture         1  ->      0     REGRESSED TO ZERO
put it back on the shelf      louis           8  ->      8     unchanged, non-leading (slot 6)
                              taught         28  ->     29     +1, but still LEADING SLOT 0 ONLY
                              beef            4  ->      4     unchanged, non-leading (slot 7)
                              fool            4  ->      5     improved 4 -> 5
when the rain finally stopped aar            13  ->     13     unchanged, non-leading (slot 3)
                              copper          1  ->      0     REGRESSED TO ZERO
                              mist            1  ->      2     improved 1 -> 2
he was a big fat man          honour          1  ->      1     unchanged, non-leading (slot 6)
                              inner           1  ->      1     unchanged, non-leading (slot 6)
what are you going to do      perdue          2  ->      2     unchanged, non-leading (slot 7)
                              duper           2  ->      2     unchanged, non-leading (slot 7)
                              dougie          2  ->      5     improved 2 -> 5
(all targets)                 misfit          0  ->      0     unchanged, UNREACHABLE everywhere
```

**Regressed to zero: `tickets` (3 -> 0), `capture` (1 -> 0), `copper` (1 -> 0).**
**Improved: `tossed` (1 -> 3), `dougie` (2 -> 5), `fool` (4 -> 5), `mist` (1 -> 2),
`taught` (28 -> 29, but still leading-slot-only so it buys no non-leading
reachability).**
**Of the six designated witnesses — `delve`, `tickets`, `louis`, `aar`, `honour`,
`perdue` — five are unchanged and one (`tickets`) is destroyed. None improved.**

The aggregate, which is the finding rather than any single word:

```text
target                          distinct non-target words   of which reachable at position >= 1
a whole lot of trouble              1766 -> 1634  (-132)        1619 -> 1481  (-138)
the cat sat on the mat              1773 -> 1575  (-198)        1662 -> 1460  (-202)
put it back on the shelf            1760 -> 1565  (-195)        1533 -> 1334  (-199)
when the rain finally stopped       2231 -> 1973  (-258)        2035 -> 1801  (-234)
he was a big fat man                1580 -> 1446  (-134)        1421 -> 1283  (-138)
what are you going to do            1499 -> 1290  (-209)        1387 -> 1187  (-200)
TOTAL                                10609 -> 9483 (-1126)       9657 ->  8546 (-1111, -11.5%)
```

**Breadth falls on every single target, by 138 to 234 non-leading words each.**
A reshuffle would show wins and losses roughly balancing in the aggregate; this
is a one-directional loss of 11.5% of non-leading word reachability. The change
is described as widening coverage — sampling "points of a class's index
rectangle" instead of "diagonals" — and on the real pools it **narrows** coverage
by roughly a ninth, because the per-slot rotations make each class emit fewer
*distinct* deep words and the shallower classes, which were cheap breadth, now
compete for a reserve whose depth-4 classes are funded from the first tuple.

The user-visible consequence is nil and the internal consequence is severe:

```sh
for t in "It's just a stupid game" "the cat sat on the mat" "put it back on the shelf"; do
  /workspace/wc-verify/base/target/release/madgab --approximate --top 50 "$t" > /tmp/cmp_b.txt
  /workspace/wc-verify/new/target/release/madgab  --approximate --top 50 "$t" > /tmp/cmp_n.txt
  cmp /tmp/cmp_b.txt /tmp/cmp_n.txt && echo "IDENTICAL: $t"
done
```

```text
VISIBLE OUTPUT BYTE-IDENTICAL: It's just a stupid game
VISIBLE OUTPUT BYTE-IDENTICAL: the cat sat on the mat
VISIBLE OUTPUT BYTE-IDENTICAL: put it back on the shelf
```

`select_diverse` absorbs the entire 269-candidate pool loss on the canonical
target and the entire `tickets` loss on `the cat sat on the mat` without emitting
a single different visible clue. That is why `approximate_output_is_locked` still
passes, and it is why the pool-level guard is the only instrument in this
repository that can see the change at all. **A change that is invisible in the
product and visible only as a broken pool invariant has bought nothing and cost
the invariant.**

## 2. The canonical membership number, one line

> **The requested wording: ABSENT, 0 pool members.**

Taken from the deduplicated pool the **release** binary produced on the
**default** approximate path for `It's just a stupid game` with `--top 50`,
offsets verbatim from `madgab::Clue::cuts`, via a read-only env-gated dump
(`ZZ_POOL_OUT`) placed after the `phrase_signature` dedup and before
`select_diverse`, with the probe proven non-intrusive (stdout byte-identical with
and without it, on all seven targets). The default path's own trace agrees
independently, using the same predicate the probe uses:

```text
MADGAB_TRACE raw phrase="hits justice dupe hid came" missing candidates=17637
```

`missing candidates` equals the pool size 17637, so zero candidates match. This
agrees with the implementer's own arm, which reports every pair of the wording's
words still at 0 after the change. No further explanation of provenance is
needed because the number is not in dispute; the disagreement in this queue is
about the cause, and section 1's `dupe` offset-coverage finding is the cause this
front can evidence.

## 3. Review: is the reserve change defensible as a general improvement on its own terms?

The brief puts four things in the change's favour and one against. Taking each
on its own evidence.

**(a) "It makes a 4-deep off-modal pairing reachable where 3 was the max."**
True as a property of `coverage_tuples`, and it is asserted:
`depth_profile_reserve_emits_pairings_not_only_diagonals` passes on all 64
phases of the synthetic widths `[SPAN_SHORTLIST, 7, SPAN_SHORTLIST,
SPAN_SHORTLIST, 93]` (`cargo test --release --lib` 51 passed, 0 failed). But this
is the weak point of the whole change: **the property is demonstrated only on
synthetic widths, and it is never observed to fire on a real pool.** On the
canonical target the 4-deep capability produced 14 added wordings, and every one
of the resulting 48 members of `[3,10,13,15,19]` still leads with `each` or a
near neighbour and keeps `justice` in slot 1. On the six real targets no new
designated witness appeared. The unit test proves the *shape* is expressible; no
measurement in this front shows it being *used* for a pairing on any real input.
That is a real gap between the claim and the evidence, and it is the same gap
that let the change ship with the item's own target still at 0.

**(b) "It keeps the visible output identical."** True, and verified directly:
byte-identical `--top 50` output on all three targets checked, including
`the cat sat on the mat`, whose pool lost `tickets`. This is not the safety it
appears to be — section 1b shows the identical visible output coexists with an
11.5% loss of pool breadth. Identical visible output means the change is
*undetectable in the product*, not that it is safe.

**(c) "It keeps the guard green."** **It does not.**
`approximate_pool_reaches_alternatives_past_the_opening_slot_width` passes on
`fe4ad78` and fails on `a58a49b`. This claim is false as stated, and it is the
single hardest fact against the branch.

**(d) "The wall clock is flat."** True, and now replicated. Session 1 median
`search` deltas ran -4.7% to +12.9%; session 2 ran -10.2% to +12.8%, with the
sign **flipping per target between sessions** (`he was a big fat man` +1.9% then
-10.0%; `what are you going to do` +0.7% then -10.2%; `the cat sat on the mat`
-1.4% then +12.8%). Two sessions, 8 reps per target per arm, opposite signs: the
true effect is indistinguishable from zero and per-target noise on this host is
+-10-13%. Consistent with the pool getting *smaller*, not larger. This is the
change's strongest genuine result.

**(e) "It costs pool breadth and at least one non-canonical witness."**
Understated. It costs breadth on **all six** targets, **-1111 non-leading words
(-11.5%)** in aggregate, and it costs **three** witnesses to zero
(`tickets`, `capture`, `copper`), one of which is the word a green guard test
asserts on. It buys no improvement in any designated witness and no appearance of
the requested wording.

### Verdict

**Not defensible as a general improvement, on this evidence.** Not because the
idea is wrong — making the reserve emit points rather than diagonals of a
class's index rectangle is a correct diagnosis of a real defect, and the depth
interleave plus `EMIT_PROFILE_MAX_DEEP = 4` is a coherent way to attack it. It
is not defensible because, on the pools that exist, the change delivers **none**
of its claimed benefit at the item's target (0 members, unchanged) while
measurably **narrowing** coverage by about a ninth and breaking the invariant
that exists to prevent exactly that. The cost is measured, repeated and
one-directional; the benefit is asserted on synthetic widths and unobserved in
the product. A change whose only demonstrated effect is a measurable loss should
not be integrated on the strength of a unit test.

I record no view on whether the idea should be attempted again with a different
budget or a different rotation; that is a later pass's call, and the numbers
below are what such a pass would need.

### The measurement that would settle the breadth question

**Not** another wall clock, and not another pool dump. The question is whether
the 11.5% breadth loss is intrinsic to sampling points-with-rotations or an
artefact of the reserve being spent on depth-4 classes from the first tuple.
The settling measurement is a **reserve-size sweep on the default path**:
measure aggregate non-leading word reachability (the 9657/8546 metric above) and
`tickets` membership on the canonical target and on `the cat sat on the mat`, for
`EMIT_PROFILE_RESERVE` at its current value and at 1.5x and 2x, plus the current
`EMIT_PROFILE_MAX_DEEP` of 4 and 3, all against the parent as control. That
yields a four-cell answer to "is the loss a depth-cap cost or a budget cost".
It is admissible as stated — same default path, same release build, same
deduplicated pool dump, no re-budgeted harness, no shortlist ranks — because the
constants are the object under test and the emitted tuples are still filtered by
`build`'s additive `total_budget` comparison, which is the bound the work item
requires the deeper enumeration to be derived from. If breadth is restored at
2x reserve without the requested wording appearing, the loss is a budget cost and
the front is viable. If breadth stays down at every setting, the rotation scheme
itself is the cost and the approach needs rethinking before it is retried.

## 4b. Note on the `/tmp/opencode` build path for criterion 7

A later coordinator pass recorded that `/tmp` is `noexec` but `/tmp/opencode` is
exec-capable, and directed that the two arms be built under `/tmp/opencode/wc`.
**That is not the case on this host, and the build fails there.** Evidence:

```text
$ git archive fe4ad78 | tar -x -C /tmp/opencode/wc/base   # extraction succeeds
$ cargo build --release   # in /tmp/opencode/wc/base
error: failed to run custom build command for `proc-macro2 v1.0.107`
Caused by: could not execute process `.../build-script-build` (never executed)
Caused by: Permission denied (os error 13)

$ cat /proc/mounts | grep tmpfs
tmpfs /tmp tmpfs rw,nosuid,nodev,noexec,relatime,inode64 0 0

$ ls -l .../build-script-build
-rwxr-xr-x 2 lubko lubko 462024 ...        # mode 0755, yet EACCES on exec
$ printf '#!/bin/sh\necho EXEC-OK\n' > /tmp/opencode/wc/probe.sh; chmod +x ...
$ /tmp/opencode/wc/probe.sh
bash: .../probe.sh: Permission denied
```

`/tmp/opencode` is a plain subdirectory of the same `noexec` tmpfs and has no
mount of its own, so the exec bit is set and the kernel still refuses. This is
the same finding as the first pass, now confirmed with a minimal exec probe as
well as a cargo build.

The criterion 7 measurement was therefore taken with the coordinator's prescribed
recipe and only the parent directory changed, forced by `noexec`:

```sh
git archive a58a49b | tar -x -C /workspace/wc-verify/new    # a58a49b, pristine
git archive fe4ad78 | tar -x -C /workspace/wc-verify/base   # fe4ad78, pristine
(cd /workspace/wc-verify/new  && cargo build --release)
(cd /workspace/wc-verify/base && cargo build --release)
```

No worktree, no checkout, nothing in the repository mutated — `git status` is
clean and `git log -1` is `781fb5f` after all timing runs. Arm provenance
verified by hashing `src/lib.rs` in each build tree against the git blob of its
commit:

```text
base  worktree fbeb85dbd69ddf55   fe4ad78 git blob fbeb85dbd69ddf55   MATCH
new   worktree 1f0a63eab0e7564a   a58a49b git blob 1f0a63eab0e7564a   MATCH
base  ZZ_POOL_OUT present: false
new   ZZ_POOL_OUT present: false
```

Both arms carry **no probe**, so the comparison is search against search and not
probe against no-probe. Section 2's numbers are replicated in a second
independent session (`/tmp/wc.js`, 4 reps per target per arm, interleaved, arm
order alternated each rep, one discarded warm-up per binary):

```text
target                        parent search med   a58a49b search med    delta
a whole lot of trouble              1436 ms            1449 ms     +13 ms   ( +0.9%)
the cat sat on the mat              1613 ms            1820 ms    +207 ms   (+12.8%)
put it back on the shelf            1635 ms            1717 ms     +82 ms   ( +5.0%)
when the rain finally stopped       2087 ms            2105.5 ms   +18.5 ms  ( +0.9%)
he was a big fat man                1597 ms            1437.5 ms  -159.5 ms (-10.0%)
what are you going to do            1605 ms            1442 ms    -163 ms   (-10.2%)
It's just a stupid game             1488 ms            1584.5 ms   +96.5 ms  ( +6.5%)
```

Signs flip against session 1 on three of seven targets, in both directions, so
the per-target noise band is +-10-13% and the effect is not distinguishable from
zero. `a whole lot of trouble`, the only target that showed +12.9% in session 1,
came in at +0.9% in session 2 and at med 1.83 s / mean 1.87 s against the
parent's med 1.83 s / mean 1.88 s over 8 further interleaved reps. **No wall
clock regression, on two independent sessions.**

## Branches

* `scratch/2f7a10-verify` — this commit plus the probe hunk. Scratch.
  **Must never be merged.**
* `madgab-pairing-2f7a10` / `origin/madgab-pairing-2f7a10` — the implementer's
  branch at `a58a49b`. Not touched, not merged into, not merged from.
* `scratch/2f7a10-base` — the baseline front. Read for the `ZZ_POOL_OUT` recipe;
  **never merged**.
