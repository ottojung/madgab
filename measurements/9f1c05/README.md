# Front `9f1c05` — measurement corpus and harness (SCRATCH, NEVER MERGE)

**This branch is measurement-only and must never be merged into
`post-milestone-acceptance`, and nothing here may ever reach `main`.** It exists
so that a later pass can re-derive every number in
`git show madgab-aggform:REPORT-9f1c05.md` without re-running this agent's
process. The report and its conclusions live on `madgab-aggform`; this branch
holds only the scripts, the target index, the harness patches and the derived
summary tables.

The raw pool dumps are **not** committed: 16 targets x ~4 MB is ~58 MB of
regenerable output. `price.sh` and the two commands in §3 below regenerate them
byte-for-byte from a clean build, because the pool is reproducible on this head
(noise floor exactly 0, front `3e7b04`).

## 1. What is here

| file | what it is |
|---|---|
| `index.tsv` | the 16 targets, `id<TAB>target`, in sweep order |
| `lib.mjs` | pool loader that reproduces production `finish()`: sort by score desc, phrase asc, then dedup on `phrase_signature`; plus the `c81e55` full-resegmentation rule and the closed-class-share helper |
| `forms_def.mjs` | the seven candidate aggregation forms, each a function of the seven axis values (and, where the form needs a scale, of the pool's own per-axis statistics) |
| `forms.mjs` | `measure` / `churn` / rank-and-cutoff helpers |
| `run_forms.mjs` | offline driver: evaluates every form against a captured pool TSV |
| `price.sh` | end-to-end driver against the patched apply arm |
| `harness-dump.patch` | the pool-dump hunk applied to a `git archive HEAD` extraction |
| `harness-apply.patch` | the form-application + injection + visible-dump hunks, same extraction |
| `price-summary.tsv` | the derived per-target per-form table (churn, reseg, salad, pool) |
| `forms-summary.tsv` | the derived offline per-form table on the canonical pool |

## 2. Provenance and fences

Both patches apply to a `git archive` extraction of the branch head, never to a
checkout and never to any branch. No word, phrase, target, dictionary entry or
corpus lookup is named in either patch; both are presence-tested switches whose
values are either a form index, a list of index tuples, or a list of phrases
this harness itself produced one pass earlier. With every switch unset the
patched binary's visible stdout is byte-identical to production, and
`tests/no_phrase_hard_coding` is green because it reads only `src/` of a branch
that carries none of this.

## 3. Reproducing the pool capture

```sh
# 1. two throwaway arms, outside any checkout
mkdir -p /workspace/rev-9f1c05-dump /workspace/rev-9f1c05-apply
git archive madgab-aggform | tar -x -C /workspace/rev-9f1c05-dump
git archive madgab-aggform | tar -x -C /workspace/rev-9f1c05-apply
cd /workspace/rev-9f1c05-dump   && patch -p1 < .../harness-dump.patch   && cargo build --release
cd /workspace/rev-9f1c05-apply  && patch -p1 < .../harness-apply.patch  && cargo build --release

# 2. capture one pool per target: release binary, default approximate path,
#    --approximate --top 50, axis dump on stderr
while IFS=$'\t' read -r id target; do
  MADGAB_AGG_DUMP=1 /workspace/rev-9f1c05-dump/target/release/madgab \
    --approximate --top 50 "$target" 2>pool_$id.tsv >vis_$id.txt
done < index.tsv
```

**`top_n` is 50 for every pool count and every rank anywhere in this
measurement, on both arms of every comparison.** The pool is a strong function
of `top_n` (13471 / 14561 / 17913 at `--top 10` / `--top 20` / `--top 50` for the
canonical target on this head), so a rank here is not a rank at another
`top_n`.

**Validation that this capture is the production pool.** The seven pool counts
`3e7b04` published at `--top 50` on `3d520c0` are reproduced exactly by this
capture (17913, 15908, 16169, 17091, 18474, 15609, 19168 — 7 of 7), and the
`axes::`-weighted sum of each dumped axis vector reproduces the production
score to **7.5e-13** on all 16 targets. `run_forms.mjs` re-checks both.

## 4. Reading `harness-apply.patch`

It changes `Partial::metrics` so that the *way the seven axis values are
combined* is selectable, and nothing else. It deliberately does **not** touch
`span_score_bound`, `partial_span_score`, `complete_span_score`, `select_diverse`,
the share cap, `STRUCTURE_FLOOR`, `coverage_tuples`, `sweep_index`,
`EMIT_PROFILE_MAX_DEEP` or `EMIT_PROFILE_RESERVE`. That incompleteness is
itself one of the report's findings and is discussed there: `metrics().combined`
is also the search's `incumbent` threshold at `src/lib.rs:785-800`, consumed by
the span-path cut at `:1194`, so any form that is not an upper-bounding affine
re-expression of the old objective silently truncates the search. The
per-form pool counts in `price-summary.tsv` are the measurable signature of that
truncation.
