# 8a1d47 measurement harness (scratch branch, never merge)

Reproduces every number in `docs/work/items/w-8a1d47.md`'s Resolution.

* `src/lib.rs` carries two things the accumulation branch must never see:
  the `ZZ_POOL_OUT` pool-dump hook, and the coverage-first admission order
  (`5c156b1`), with F1 (`b6ddcb0`) and F2 (`c15b51a`) ported on top.
* `coverage-first.patch` is the selection change alone, against `fc5296f`.
* `f1.patch` is the F2 port alone, against `fc5296f`.
* `cell.mjs` emits one row of the 2x2 from a pool dump plus a CLI run.
* `sweep.mjs` / `sel.mjs` are the coverage-budget sweep and the selector model.
  The model reproduces the shipped `select_diverse` visible list exactly on all
  seven targets (validated against the CLI before any variant was scored).

Build each cell by applying the wanted combination to `fc5296f`, then:

    ZZ_POOL_OUT=/tmp/pool.tsv cargo run --release -- --approximate --top 50 "<target>"
    node cell.mjs /tmp/pool.tsv <visible.txt> "<the requested wording>"
