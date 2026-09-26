#!/bin/bash
# probe run: $1 = arm dir, $2 = out tsv
set -e
ARM=$1
OUT=$2
rm -f "$OUT"
i=0
while IFS= read -r t; do
  i=$((i+1))
  MADGAB_COST_PROBE="$OUT" "$ARM/target/release/madgab" --approximate --top 50 ${EXTRA:-} "$t" > /dev/null
done <<'EOF'
It's just a stupid game
recognize speech
a whole lot of trouble
the cat sat on the mat
put it back on the shelf
when the rain finally stopped
he was a big fat man
what are you going to do
she had a lot of money
there is no way to know
an old man in a big hat
my brother has a red car
EOF
wc -l "$OUT"
