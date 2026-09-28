#!/usr/bin/env bash
# TEMP profiling harness
set -u
export PATH="/gnu/store/myghlzn9m8d9ccj19ygf7bbmlyflpvld-ripgrep-15.1.0/bin:$PATH"
BIN=${BIN:-./target/release/madgab}
REPS=${REPS:-3}
OUT=${OUT:-prof/results.txt}
: > "$OUT"
i=0
while IFS= read -r t; do
  i=$((i+1))
  for r in $(seq 1 "$REPS"); do
    echo "### target=$i rep=$r phrase=$t" >> "$OUT"
    MADGAB_PROF=1 "$BIN" --approximate --top 20 "$t" 2>&1 >/dev/null \
      | rg '^(PROF|PROFSTAT|corpus)' >> "$OUT"
  done
done < prof/targets.txt
echo "wrote $OUT"
