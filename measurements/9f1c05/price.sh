#!/bin/bash
# End-to-end pricing sweep for front 9f1c05, run against the THROWAWAY apply
# arm in /workspace/rev-9f1c05-apply (a `git archive` extraction of this
# branch, patched by measurements/9f1c05/harness-apply.patch; never committed
# to any branch).
#
# Per (target, form): pass 1 captures the visible list, then the pool count via
# the production trace gate with a neutral non-word value (so the trace reports
# the pool size rather than a rank), then pass 2 re-runs with that visible list
# named in MADGAB_AGG_VIS so the arm reports each visible member's axis vector.
# Offline, that yields visible churn, full-resegmentation count and
# function-word-salad count without touching select_diverse.
#
# FORM 1 IS DELIBERATELY ABSENT.  It is disqualified on admissibility: it
# returns scores outside the documented Clue::score in [0,1] contract (observed
# rank-49 cutoff 5.813541148).  See REPORT-9f1c05.md.  Per the 21:30Z
# coordinator pass, no further budget is spent on it.
set -u
BIN=/workspace/rev-9f1c05-apply/target/release/madgab
OUT=/tmp/opencode/agg/price
mkdir -p $OUT
TOPN=50
export TIMEFORMAT='%R'
while IFS=$'\t' read -r id target; do
  for form in 0 2 3; do
    tag="$OUT/${id}_f${form}"
    timeout 300 env MADGAB_AGG_FORM=$form "$BIN" --approximate --top $TOPN "$target" \
      2>"$tag.err" >"$tag.out"
    grep -oP '^\s*\d+\.\s+\[[0-9.]+\]\s+\K.*' "$tag.out" > "$tag.vis"
    timeout 300 env MADGAB_AGG_FORM=$form MADGAB_TRACE_PHRASES="zqjx" \
      "$BIN" --approximate --top $TOPN "$target" 2>&1 >/dev/null \
      | grep -oP 'missing candidates=\K[0-9]+' > "$tag.pool"
    V=$(tr '\n' '\n' < "$tag.vis")
    timeout 300 env MADGAB_AGG_FORM=$form MADGAB_AGG_VIS="$V" \
      "$BIN" --approximate --top $TOPN "$target" 2>"$tag.visaxes" >/dev/null
    grep '^AGGVIS' "$tag.visaxes" | cut -f2- > "$tag.axes"
  done
  echo "swept $id $target"
done < /tmp/opencode/agg/index.tsv
echo SWEEP-DONE
