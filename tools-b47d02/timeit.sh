#!/bin/bash
# interleaved timing, probe off
A=$1; B=$2; N=${3:-6}
for t in "It's just a stupid game" "recognize speech"; do
  echo "== $t"
  echo -n "cap4:"
  echo -n "cap3:"
  for i in $(seq 1 $N); do
    if [ $((i%2)) -eq 1 ]; then
      x=$("$A/target/release/madgab" --approximate --top 50 "$t" 2>&1 | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o 'search [0-9]*ms' | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o '[0-9]*')
      y=$("$B/target/release/madgab" --approximate --top 50 "$t" 2>&1 | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o 'search [0-9]*ms' | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o '[0-9]*')
    else
      y=$("$B/target/release/madgab" --approximate --top 50 "$t" 2>&1 | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o 'search [0-9]*ms' | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o '[0-9]*')
      x=$("$A/target/release/madgab" --approximate --top 50 "$t" 2>&1 | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o 'search [0-9]*ms' | /gnu/store/05vy146ycavwjm8va1d8qym188s14vzv-grep-3.11/bin/grep -o '[0-9]*')
    fi
    echo -n " $x"
    echo -n " $y"
  done
  echo
done
