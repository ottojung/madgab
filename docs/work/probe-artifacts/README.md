# Second-pass recovery — measurement artifacts the first archive missed

Companion to [../probes/](../probes/) and [../probe-patches/](../probe-patches/) from
`51ebdd1`. The first archive pass enumerated dirty worktrees but skipped files that were
distinct from an already-archived name, files under a non-`src` untracked directory, and
raw measurement output. Those are recovered here. MadGab development is still **paused**
(see [../../accepted-state-2026-09-27.md](../../accepted-state-2026-09-27.md)); this is
durability of past measurements only, not a front and not a candidate for integration.

## Probe sources not previously archived

| archived copy | live worktree path | why the first pass missed it |
|---|---|---|
| `floor-5e2d42-probe_measure.rs` | `/workspace/floor-5e2d42-probe/examples/measure.rs` | second `examples/` entry in that worktree; only `zz_5e2d42_spans.rs` was taken |
| `floor-5e2d42-probe_zz_5e2d42_spans.rs` | `/workspace/floor-5e2d42-probe/examples/zz_5e2d42_spans.rs` | **same basename as an already-archived file but different content** — the base-side probe and the measurement probe were different programs. The archived copy is the base-side one (`15f4a6c5…`); this is the measurement one (`661d335e…`). Recovering it under a disambiguated name is the whole point |
| `probe-0f3a17_src_probe.rs` | `/workspace/probe-0f3a17/src/probe.rs` | same trap: `src/probe.rs` was already archived from `madgab-axis-558697` (`6549c937…`). This worktree's copy is a different 71-line slot-recorder (`d78cc4c7…`) |
| `fillstrat-zz_oldfill.js` | `/workspace/madgab-fillstrat-probe/zz_oldfill.js` | untracked at the worktree root, outside `examples/` |

`approx-runtime-prof_{README,REPORT}.md` are the write-ups from
`/workspace/madgab-approx-runtime/prof/`, which is untracked as a whole directory. The
`prof/` tree is 58 MB, of which two 30 MB instrumented binaries and ~2 MB of `results-*.txt`
and `sum-*.txt` were left behind; the two markdown files carry the findings and are the part
worth keeping. `prof/README.md` documents one real `src/lib.rs` change in `prune_partials`
(cache `metrics` instead of recomputing per comparison) which was **not** carried anywhere.

## Measurement output

[../probe-output/c1d3a7-m-head200.txt](../probe-output/c1d3a7-m-head200.txt) is the first 200
lines of `/workspace/c1d3a7-instr/m.txt`, a 21,020-line / 4.0 MB `ZZMETRICS` dump of scored
candidate clues for `It's just a stupid game`. Only the head is archived — the full file is a
flat ranked list and 4 MB of it does not earn its place in a repository. The head is the
high-scoring end and is the part that carries information: it shows the top candidates
sitting near `combined: 0.75` with no exact classical clue present, which is consistent with
the accepted limitation. The tail closes with the target and IPA lines.

**Not archived:** `target-front-3a8f01/` (1.4 GB) and `target-front-3a8f02/` (1.3 GB) in
`madgab-diversity-3a8f01` and `madgab-poolrank-3a8f02`. These are Cargo `target/` directories
from the paused fronts, reproducible from the recorded source, and far too large to carry.

## Third pass — the `prof/` reproduction scripts and baseline raw output

| archived copy | live worktree path | why the first two passes missed it |
|---|---|---|
| `approx-runtime-prof_run.sh` | `/workspace/madgab-approx-runtime/prof/run.sh` | untracked directory `prof/`; the second pass took only the two markdown write-ups and dropped the harness that produced every number in them |
| `approx-runtime-prof_summarize.py` | `/workspace/madgab-approx-runtime/prof/summarize.py` | same |
| `../probe-output/approx-runtime-prof-baseline/` (24 files) | `/workspace/madgab-approx-runtime/prof/baseline/` | small per-rep `.out`/`.err`/`.target` files; the second pass's rule was "skip raw output rather than source", and this baseline half is the evidence behind the baseline column of `scale.txt` |

`run.sh` and `summarize.py` are the parts that matter: without them the two markdown
write-ups are unreproducible claims, and the second pass explicitly noted that
`prof/README.md` documents a real `src/lib.rs` change in `prune_partials` that "was **not**
carried anywhere". That change is still not in any branch — it existed only as the dirty
state of `madgab-approx-runtime` and is now at least captured as the harness that measured
it. **Still not archived** (unchanged, deliberate): the two 30 MB instrumented binaries
`prof/madgab-baseline` and `prof/madgab-prof`, the ~2 MB `results-*.txt`/`sum-*.txt`
summaries, and the `target-front-*` Cargo directories.

## Hard-coding fence

**Read this before promoting any file here.** Several archived copies contain canonical
phrases — 180 of the 200 lines in the `m.txt` head include `justice`, and two of the probe
sources reference the case-2 target. They live under `docs/`, which
`tests/no_phrase_hard_coding.rs` does not scan; that test walks `src/`, `web/` and
`examples/` only, and `ALLOWLIST_CAPS` is unchanged at `("src", 0), ("web", 2),
("examples", 1)`. This is a record of past measurements, not production coupling. **If any
of it is ever promoted into `src/`, `web/` or `examples/`, its phrase literals must be
removed as part of that promotion, not waived.**

## Fourth pass — the `prof/` harness input manifests

| archived copy | live worktree path | why the third pass missed it |
|---|---|---|
| `../probe-inputs/targets.txt` | `/workspace/madgab-approx-runtime/prof/targets.txt` | the third pass archived the harness *and* the baseline output, but not the file `run.sh` reads. `run.sh` ends in `done < prof/targets.txt`, so the archived harness is still inert without it |
| `../probe-inputs/scale.txt` | `/workspace/madgab-approx-runtime/prof/scale.txt` | the scale-series phrase list; the third pass described the baseline directory as "the `scale.txt` baseline column rests on" and archived the column without the row label |
| `../probe-inputs/scale-after.txt` | `/workspace/madgab-approx-runtime/prof/scale-after.txt` | same series, post-change run |

These are small and were skipped by the enumeration rules standing rules 6 and 7 are
written against: they are neither source, nor raw output, nor an unarchived `src/` file —
they are harness *inputs*. The sweep that found them is the prescribed one (hash every
dirty and untracked file against every reachable blob); what changed is that the count of
unaccounted-for live files went to the ones that are neither build artifacts nor the
deliberately-dropped `results-*`/`sum-*` set.

Hashes at archiving time, for the standing hash-compare check:

```
32a5d09605b99e048a429dbcd5afd03d300e632d  targets.txt
380d7f1f365660fcae848b35ffea0e137fe1f550  scale.txt
83522d546d8d90964728a04b3a531c5e3554074d  scale-after.txt
```

`targets.txt` and `scale.txt` are lists of target phrases, so the hard-coding fence above
applies to them exactly as it does to the probe sources: they stay under `docs/`, and
their phrase literals must be stripped as part of any promotion, never waived.
