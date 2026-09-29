#!/usr/bin/env bash
# selfcheck.sh — does the paused-recon instrument set still run?
#
# Pass 298 named the gap this closes: "no instrument checks that these five
# scripts themselves still run: a future edit to any of them would be discovered
# by a pass that happens to run it, not automatically." Every other standing
# fact has an instrument; the instruments themselves had none.
#
# WHAT A CLEAN RUN HERE CLAIMS, AND WHAT IT DOES NOT
#
# It claims: each instrument is executable, parses under bash, exits 0, and
# PRINTS the invariant line it exists to produce.
# It does not claim: that the printed invariant is CORRECT. That is the
# instruments' own business, established by their own plants (passes 278, 280,
# 281, 293, 294, 298). This script is a liveness and fail-closed check, not a
# second opinion on the facts.
#
# WHY "exit 0" IS NOT ENOUGH, in this directory's own history:
#   - pass 298 plant G: `yq` absent from PATH made frontmatter.sh exit 0 on a
#     clean 0. An instrument can succeed on the failure case.
#   - pass 298 defect 3: a missing population and an empty queue were the same
#     gawk exit-0 output.
#   - pass 286: fence.awk at mode 100644 with no shebang had an UNREACHABLE
#     empty-region abort, and a hard-coded plant read as a clean 0.
# So the assertion here is on the OUTPUT, not the exit status, and the two
# failure modes are named apart because they want opposite responses:
#   DEAD   (nonzero exit)     -> the instrument is broken; do not trust it.
#   SILENT (exit 0, no line)  -> the instrument is worse than broken: it
#                                reports success while measuring nothing.
#                                Same response as DEAD, different diagnosis.
#
# Usage:  selfcheck.sh [INSTRUMENT-DIR]     (default: this script's directory)
# The argument exists so the failure cases can be PLANTED: point this at a
# throwaway directory of fake instruments and it must refuse each one.

set -uo pipefail

DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"

# instrument :: the invariant its output must contain for a clean run.
# Split on '::' because several invariant lines contain spaces and colons.
#
# THE ANCHORS ARE DELIBERATELY FIGURE-FREE, and plant H is why. A first version
# anchored on "census: 96 work items" and on "verdict 0 canonical clue
# occurrences"; a census that LEGITIMATELY moved to 97 was then reported SILENT,
# i.e. a healthy instrument condemned — and worse, a fence that genuinely found
# a hard-coded clue (its whole purpose) would have been condemned the same way,
# hiding the one alarm this directory exists to raise. An anchor must name the
# SHAPE of the invariant, never its current value. A liveness check that pins a
# moving figure is a standing fact that will be reported as broken.
INSTRUMENTS=(
  "census.sh::work items (identity: work_item:true"
  "clue-fence.sh::canonical clue occurrences in all"
  "agents.sh::non-terminal madgab agents ="
  "at-risk.sh::at-risk: "
  "frontmatter.sh::failed to parse"
)

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

status=0

# The population check: a directory that is not the instrument set is a BROKEN
# POPULATION, not a passing run. Same distinction as pass 298's census defect 3
# and this script's own DEAD/SILENT split — a thing that failed to be measured
# must never read as a thing that measured zero.
missing=0
for entry in "${INSTRUMENTS[@]}"; do
  name="${entry%%::*}"
  [ -f "$DIR/$name" ] || { echo "selfcheck: MISSING $name in $DIR" >&2; missing=$((missing+1)); }
done
if [ "$missing" -gt 0 ]; then
  echo "selfcheck: BROKEN POPULATION — $missing of ${#INSTRUMENTS[@]} instruments absent from $DIR" >&2
  echo "selfcheck: a missing instrument is not a clean result" >&2
  exit 2
fi

echo "selfcheck: instrument dir $DIR"
echo "selfcheck: $(( ${#INSTRUMENTS[@]} - missing )) of ${#INSTRUMENTS[@]} instruments present"

for entry in "${INSTRUMENTS[@]}"; do
  name="${entry%%::*}"
  want="${entry#*::}"
  path="$DIR/$name"
  base="$(basename "$path")"

  # --- static checks -----------------------------------------------------
  # mode 755 and a shebang are load-bearing here, not hygiene: pass 286's
  # fence.awk at 100644 with no shebang could not execute its own abort, so a
  # hard-coded plant read as a clean 0. Check the bits, then check the parse.
  mode="$(stat -c %a "$path" 2>/dev/null || echo '?')"
  shebang="$(head -1 "$path" 2>/dev/null | cut -c1-2)"
  notes=""
  [ "$mode" = "755" ] || notes="$notes NOT-EXECUTABLE(mode=$mode)"
  [ "$shebang" = "#!" ] || notes="$notes NO-SHEBANG"
  if ! parse_err="$(bash -n "$path" 2>&1)"; then
    notes="$notes DOES-NOT-PARSE:$(printf '%s' "$parse_err" | head -1)"
  fi
  if [ -n "$notes" ]; then
    printf '  %-16s STATIC-FAIL%s\n' "$base" "$notes"
    status=1
    continue
  fi

  # --- run it ------------------------------------------------------------
  # stdout, stderr and the exit code are captured SEPARATELY. Reading only
  # stdout, or only the exit code, is pass 298's defect 3 verbatim: the two
  # carry different information and either alone is a half-measurement.
  if "$path" >"$tmp/out" 2>"$tmp/err"; then rc=0; else rc=$?; fi
  errlines="$(wc -l <"$tmp/err" | tr -d ' ')"

  if [ "$rc" -ne 0 ]; then
    printf '  %-16s DEAD      exit=%s stderr_lines=%s\n' "$base" "$rc" "$errlines"
    status=1
    continue
  fi

  # --- the assertion that matters ----------------------------------------
  # exit 0 is not a result. The instrument must PRINT its invariant.
  if grep -qF -- "$want" "$tmp/out"; then
    printf '  %-16s OK        prints its invariant (exit 0, stderr_lines=%s)\n' "$base" "$errlines"
  else
    printf '  %-16s SILENT    exit 0 but no line containing %s\n' "$base" "\"$want\""
    echo "selfcheck: $base exited 0 while measuring nothing — worse than dead" >&2
    status=1
  fi
done

if [ "$status" -eq 0 ]; then
  echo "selfcheck: all ${#INSTRUMENTS[@]} instruments executable, parsing, exiting 0, and printing their invariant"
  echo "selfcheck: this is LIVENESS only; each instrument's correctness is its own plants'"
else
  echo "selfcheck: REFUSING — the instrument set is not trustworthy as it stands" >&2
  echo "selfcheck: a standing fact measured by a dead or silent instrument is NOT a measurement" >&2
fi
exit "$status"
