# TEMP profiling scaffolding — strip before committing

Everything here is measurement-only, **except one change** flagged below.

## The one real change

`src/lib.rs`, in `prune_partials` (tail of the function):

```rust
// before
let mut out: Vec<Partial> = selected.into_iter().map(|i| items[i].clone()).collect();
out.sort_by(|a, b| {
    let am = a.metrics(target_boundaries, target_words, total_len, true);
    let bm = b.metrics(target_boundaries, target_words, total_len, true);
    cmp_desc(am.combined, bm.combined)
});

// after
let mut picked: Vec<usize> = selected.into_iter().collect();
picked.sort_by(|&a, &b| cmp_desc(metrics[a].combined, metrics[b].combined));
picked.into_iter().map(|i| items[i].clone()).collect()
```

This removes an O(k log k) re-run of `Partial::metrics` in the sort comparator.
Measured: 3.8x–4.9x faster end-to-end, byte-identical output on the 8 benchmark
targets at `--approximate --top 20`. Revert it with `git checkout src/lib.rs`
if you want a pure-instrumentation tree.

## Everything else

* `src/lib.rs`: `pub mod prof` (between `mod approx;` and the `wasm` cfg) and
  `TEMP-PROF` / `TEMP-PROF-BEGIN` / `TEMP-PROF-END` comments.
* `src/approx.rs`: `TEMP-PROF` comments only (4 `prof::T::new` calls).
* `src/main.rs`: one line, `madgab::prof::dump();` after the clue loop.

All of it is inert unless `MADGAB_PROF=1` is set in the environment.

## Reproducing

```sh
export PATH="/gnu/store/myghlzn9m8d9ccj19ygf7bbmlyflpvld-ripgrep-15.1.0/bin:$PATH"
export PATH="/gnu/store/04s5rn5sclrf933rbfp8262gr24yxdmi-profile/bin:$PATH"
cargo build --release --offline
REPS=3 OUT=prof/results-new.txt bash prof/run.sh
python3 prof/summarize.py prof/results-new.txt
```

* `prof/madgab-baseline` — pristine HEAD binary, used as the output oracle.
* `prof/baseline/N.out` — reference output for target N.
* `prof/targets.txt` — the 8 benchmark targets.
* `prof/scale.txt` — length-scaling ladder.
* `prof/sum-baseline.txt` — baseline phase breakdown.
* `prof/sum-exp6.txt` — breakdown after the fix above.
* `prof/sum-exp7.txt` — breakdown inside `Partial::metrics` (inflated: 4 extra
  `Instant::now()` per `metrics()` call).
