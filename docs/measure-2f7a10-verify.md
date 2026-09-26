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
In the same breath the change **broke a guard it was required to keep green**:
`approximate_pool_reaches_alternatives_past_the_opening_slot_width` passes on the
parent `fe4ad78` and **fails** on `a58a49b`, on `tickets` for
`the cat sat on the mat`, which is the same regression the pool measurement
shows independently (`tickets` 3 pool candidates -> 0). Criterion 3 fails and
criterion 5 fails. Wall clock did **not** regress.

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

## Branches

* `scratch/2f7a10-verify` — this commit plus the probe hunk. Scratch.
  **Must never be merged.**
* `madgab-pairing-2f7a10` / `origin/madgab-pairing-2f7a10` — the implementer's
  branch at `a58a49b`. Not touched, not merged into, not merged from.
* `scratch/2f7a10-base` — the baseline front. Read for the `ZZ_POOL_OUT` recipe;
  **never merged**.
