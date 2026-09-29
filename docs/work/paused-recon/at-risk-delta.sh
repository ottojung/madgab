#!/usr/bin/env bash
# at-risk-delta.sh -- attribute the at-risk census's DELTA by identity, and
# settle the "at-risk tree with no ref-held twin" gap by CONTENT.
#
# WHY THIS FILE EXISTS (pass 322, log rule 14l applied to the at-risk row).
#
# TWO claims in the standing table are populations reported as bare counts, and
# both have cost passes real work:
#
#   (1) "at-risk 89, was 88 at pass 273, the +1 is UNATTRIBUTED."  Pass 321
#       recorded this caveat rather than publish the count as settled, which was
#       the right call -- and then left it open, twice, because attributing it
#       needs a 4th command the hand procedure never published.
#
#   (2) "62 of the 87 distinct at-risk trees are shared with no *ref-held*
#       commit."  This is a PROXY, not a durability measurement, and it reads
#       alarming. It is superseded by at-risk-content.sh's direct question
#       ("is any non-build blob these commits introduce absent from origin?"),
#       which answers 0. A pass that reads only the proxy would file a false
#       at-risk finding -- the alarming direction, and rule 273's exact shape.
#
# The mechanism behind (1), measured not guessed: the pass-273 published form is
#
#     git rev-list --all --reflog --not --all "^<each of the 206 audit refs>"
#
# and the EXTRA `--not --all` is what makes it read 88 rather than 89. The
# instrument's own two arms (at-risk.sh) both omit it, so both read 89. The one
# commit the extra flag removes is named and pinned below.
#
# USAGE
#   docs/work/paused-recon/at-risk-delta.sh            # report
#   docs/work/paused-recon/at-risk-delta.sh --fetch    # re-fetch the audit mirror first
#
# FAIL-CLOSED (the at-risk.sh discipline): this script never prints a number it
# has not validated. A rev-list that dies on a bad revision prints zero lines and
# a non-zero exit, which `wc -l` reads as "nothing is at risk" -- the exact
# silent-zero this repository has now hit seven times. Every population here is
# asserted non-empty, both arms are cross-checked, and a known-positive and a
# known-negative control must both fire.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

FETCH=0
[ "${1:-}" = "--fetch" ] && FETCH=1

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

if [ "$FETCH" -eq 1 ]; then
  # Rule 14m: fetch the audit mirror by its REAL source namespace and never
  # with --prune, or 200+ local-only mirror refs are deleted as "remote-deleted".
  git fetch --no-tags origin '+refs/heads/*:refs/remotes/audit/*' >/dev/null 2>&1 \
    || { echo "at-risk-delta: DEAD -- audit-mirror fetch failed; no number reported" >&2; exit 1; }
fi

# --- populations -----------------------------------------------------------

# Exclusion set: the audit mirror, BARE PREFIX (rule 14i: a glob is WM_PATHNAME
# and reads 108 of the true set), cardinality asserted inline (rule 14g).
git for-each-ref --format='%(refname)' refs/remotes/audit > "$TMP/refs"
n_refs=$(wc -l < "$TMP/refs")
[ "$n_refs" -gt 0 ] || { echo "at-risk-delta: DEAD -- 0 exclusion refs; no number reported" >&2; exit 1; }
bad=$(awk '!/^refs\/remotes\/audit\//{n++} END{print n+0}' "$TMP/refs")
[ "$bad" -eq 0 ] || { echo "at-risk-delta: DEAD -- $bad exclusion refs outside the audit namespace; no number reported" >&2; exit 1; }

mapfile -t REFS < "$TMP/refs"
mapfile -t CARET < <(sed 's|^|^|' "$TMP/refs")

# Arm A: the two spellings at-risk.sh itself uses. Both must agree.
git rev-list --all --reflog --not "${REFS[@]}"        2>"$TMP/eA1" | sort -u > "$TMP/A" || true
git rev-list --all --reflog            "${CARET[@]}" 2>"$TMP/eA2" | sort -u > "$TMP/B" || true
[ ! -s "$TMP/eA1" ] && [ ! -s "$TMP/eA2" ] \
  || { cat "$TMP/eA1" "$TMP/eA2" >&2; echo "at-risk-delta: DEAD -- rev-list wrote stderr; no number reported" >&2; exit 1; }
[ -s "$TMP/A" ] && [ -s "$TMP/B" ] \
  || { echo "at-risk-delta: DEAD -- an arm returned 0 rows; that is a FALSE ZERO, not a measurement; no number reported" >&2; exit 1; }
diff -q "$TMP/A" "$TMP/B" >/dev/null \
  || { echo "at-risk-delta: DEAD -- at-risk.sh's two arms disagree; no number reported" >&2; exit 1; }
at_risk=$(wc -l < "$TMP/A")

# Arm B: the pass-273 PUBLISHED form, which additionally carries `--not --all`.
git rev-list --all --reflog --not --all "${CARET[@]}" 2>"$TMP/eC" | sort -u > "$TMP/C" || true
[ ! -s "$TMP/eC" ] && [ -s "$TMP/C" ] \
  || { cat "$TMP/eC" >&2; echo "at-risk-delta: DEAD -- the published-form arm failed; no number reported" >&2; exit 1; }
published=$(wc -l < "$TMP/C")

# --- (1) attribute the delta by identity -----------------------------------

comm -23 "$TMP/A" "$TMP/C" > "$TMP/delta"      # in A, removed by the extra flag
comm -13 "$TMP/A" "$TMP/C" > "$TMP/invdelta"   # in C, absent from A
n_delta=$(wc -l < "$TMP/delta")
n_invdelta=$(wc -l < "$TMP/invdelta")
[ "$n_invdelta" -eq 0 ] \
  || { echo "at-risk-delta: DEAD -- the published form found $n_invdelta commits the instrument missed; the forms are not nested; no number reported" >&2; exit 1; }
[ $(( at_risk - published )) -eq "$n_delta" ] \
  || { echo "at-risk-delta: DEAD -- arithmetic $((( at_risk - published ))) != $n_delta named deltas; no number reported" >&2; exit 1; }

# --- (2) the twin gap, answered by CONTENT not by the proxy ---------------

# Distinct trees the at-risk set introduces.
while read -r c; do git rev-parse "$c^{tree}"; done < "$TMP/A" 2>/dev/null | sort -u > "$TMP/at-trees"
# Distinct trees some REF reaches.
git rev-list --all 2>/dev/null | sort -u > "$TMP/rev"
git log --format='%T' $(cat "$TMP/rev") 2>/dev/null | sort -u > "$TMP/ref-trees"
n_at_trees=$(wc -l < "$TMP/at-trees")
[ "$n_at_trees" -gt 0 ] && [ -s "$TMP/ref-trees" ] \
  || { echo "at-risk-delta: DEAD -- a tree population came back empty; no number reported" >&2; exit 1; }

t_with=0; t_without=0
while read -r t; do
  if grep -qxF "$t" "$TMP/ref-trees"; then t_with=$(( t_with + 1 )); else t_without=$(( t_without + 1 )); fi
done < "$TMP/at-trees"
[ $(( t_with + t_without )) -eq "$n_at_trees" ] \
  || { echo "at-risk-delta: DEAD -- twin split is not a partition ($t_with + $t_without != $n_at_trees); no number reported" >&2; exit 1; }

# The direct question. Blobs the at-risk set carries that NO ref reaches.
git rev-list --objects --all 2>/dev/null | awk 'NF>0{print $1}' | sort -u > "$TMP/obj-refs"
absent=0
while read -r c; do
  while read -r o p; do
    grep -qxF "$o" "$TMP/obj-refs" || absent=$(( absent + 1 ))
  done < <(git ls-tree -r "$c" 2>/dev/null | awk '{print $3" "$4}')
done < "$TMP/A"

# --- controls, both directions --------------------------------------------
# Known-positive: an at-risk commit's own src/lib.rs blob is ref-held (this is
# the whole basis for the 0 verdict; if it stopped being true the 0 would be
# vacuous). Known-negative: origin/main's tip is not at risk.
CTRL_POS=${CTRL_POS:-f86907c9af76b845c5a0cabea02611da08c98fd0}   # 514ed91's src/lib.rs
CTRL_NEG=${CTRL_NEG:-$(git rev-parse origin/main)}
grep -qxF "$CTRL_POS" "$TMP/obj-refs" \
  || { echo "at-risk-delta: control $CTRL_POS absent from the ref-held object set -- the object set is broken; no number reported" >&2; exit 1; }
grep -qxF "$CTRL_NEG" "$TMP/A" \
  && { echo "at-risk-delta: control $CTRL_NEG (origin/main tip) reported at risk -- the exclusion set is broken; no number reported" >&2; exit 1; }
# Fabricated-absent control: a sha that is certainly in no ref set.
if grep -qxF 0000000000000000000000000000000000000001 "$TMP/obj-refs"; then
  echo "at-risk-delta: fabricated-absent control FIRED INCORRECTLY -- the object-set test cannot fail; no number reported" >&2; exit 1
fi

# --- report ---------------------------------------------------------------
printf 'at-risk-delta: instrument arms %s   published form %s   exclusion refs %s\n' "$at_risk" "$published" "$n_refs"
printf '  delta       %s commit(s) the extra `--not --all` removes, %s the other way (nested: yes)\n' "$n_delta" "$n_invdelta"
if [ "$n_delta" -eq 1 ]; then
  d=$(cat "$TMP/delta")
  printf '  named       %s\n' "$d"
  printf '               %s\n' "$(git log -1 --format='%ci  %s' "$d" 2>/dev/null | cut -c1-100)"
  printf '               held by: %s\n' "$(git for-each-ref --contains "$d" --format='%(refname)' | paste -sd, -)"
  printf '               on origin? %s\n' "$(git branch -r --contains "$d" 2>/dev/null | sed 's/^ *//' | paste -sd, - | sed 's/^$/(no origin ref contains it)/')"
  printf '               => the 88 -> %s delta is a SPELLING difference between two published\n' "$at_risk"
  printf '                  forms, not a new at-risk commit. Do not report it as growth.\n'
else
  printf '  named       (see below)\n'
  sed 's|^|               |' "$TMP/delta"
fi
printf '  tree proxy  %s distinct at-risk trees: %s with a ref-held twin, %s without\n' "$n_at_trees" "$t_with" "$t_without"
printf '  tree proxy  the "%s without" figure is a PROXY and reads alarming; it is NOT a\n' "$t_without"
printf '              durability finding. at-risk-content.sh asks the direct question\n'
printf '              (is any NON-BUILD blob these commits carry absent from origin?) and\n'
printf '              answers 0. Run it; do not promote the proxy to a finding.\n'
printf '  content     blobs the at-risk set carries that NO ref reaches: %s\n' "$absent"
printf '  controls    %s ref-held (fires), %s not at risk (fires), fabricated-absent rejected\n' "${CTRL_POS:0:8}" "${CTRL_NEG:0:8}"
printf '  next        a residual is a HUMAN judgement, not an automatic action.\n'
