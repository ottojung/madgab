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
# WHY AN INSTRUMENT MUST BE RUN WITH ITS OWN ARGUMENTS
#
# A THIRD and THIRD defect, found on this script's first live run (pass 300)
# and not by a plant but by the real repository, which is the more expensive
# kind to find late.
#
# at-risk.sh REFUSES ON ITS DEFAULT PATH, by design: if the audit mirror is
# stale it exits 1 and tells the caller to re-run with --fetch, because
# reporting a figure off a stale mirror is the pass-187 failure that pass 293
# exists to prevent. This script ran every instrument with no arguments, so it
# always measured at-risk.sh in its REFUSAL mode and reported a provably
# healthy instrument as DEAD.
#
# The trigger is structural, not incidental: every pass pushes its commit, and
# that push is precisely what staleness the mirror detects. The state pass 299
# verified and published (all five exit 0) lasts only until its own commit
# lands, so the false alarm was guaranteed on every pass from 300 onward, and
# this script's verdict was permanently red. A liveness check that is always
# red is a liveness check that gets ignored — the same loss of alarm value as
# pass 293's stale mirror reported as a clean 91, reached from the opposite
# direction.
#
# The fix is to run each instrument the way its own contract documents, NOT to
# recognise the refusal and wave it through. A "maybe it is fine" branch here
# would be the fail-open direction (rules 14q, 14r): a genuinely broken
# instrument that happens to print to stderr would be excused by the same test
# that excuses a healthy one. Measuring in the fully-measuring mode instead
# keeps the check fail-closed — a broken at-risk.sh still reads DEAD below,
# with or without --fetch. The --fetch side effect is the same idempotent,
# no---prune mirror refresh (rule 14a) that every pass performs anyway.
#
# Usage:  selfcheck.sh [INSTRUMENT-DIR]     (default: this script's directory)
# The argument exists so the failure cases can be PLANTED: point this at a
# throwaway directory of fake instruments and it must refuse each one.

set -uo pipefail

DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"

# --- the repo root is resolved, not inherited (pass 317 defect) ----------------
# This script runs the instruments as children, so it handed them the CALLER's
# cwd. Every instrument resolves its own root now, so each is
# invocation-independent on its own; running them from the resolved root as well
# means the liveness verdict itself cannot be a function of where the
# coordinator was standing. That is the whole defect, twice: the same relative
# path read as a real file from the root and as a real failure from anywhere
# else.
#
# An explicit DIR argument still wins -- it is resolved against the CALLER's
# cwd first, so the plant mechanism below (a throwaway directory of fake
# instruments) keeps working from wherever the caller was standing.
case "$DIR" in
  /*) ;;
  *)  DIR="$(pwd)/$DIR" ;;
esac
if ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" && [ -n "$ROOT" ] && [ "${ROOT#/}" != "$ROOT" ]; then
  cd "$ROOT" || { echo "selfcheck: cannot cd to $ROOT" >&2; exit 3; }
fi

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
# A THIRD field carries the arguments the instrument's OWN contract documents.
# It is empty for instruments that take none, and it is load-bearing, not
# decoration: see "WHY AN INSTRUMENT MUST BE RUN WITH ITS OWN ARGUMENTS" below.
INSTRUMENTS=(
  "census.sh::work items (identity: work_item:true::"
  "clue-fence.sh::canonical clue occurrences in all::"
  "agents.sh::non-terminal madgab agents =::"
  "at-risk.sh::at-risk: ::--fetch"
  "at-risk-content.sh::of those, NON-BUILD (component-wise filter) =::"
  "frontmatter.sh::failed to parse::"
  "item-state.sh::item frontmatter parses::"
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
  rest="${entry#*::}"
  want="${rest%%::*}"
  args="${rest#*::}"
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
  #
  # $args is the instrument's own documented invocation, unquoted, so that a
  # multi-word remedy would split into words. It is empty for four of the five.
  if [ -n "$args" ]; then
    if "$path" $args >"$tmp/out" 2>"$tmp/err"; then rc=0; else rc=$?; fi
  else
    if "$path" >"$tmp/out" 2>"$tmp/err"; then rc=0; else rc=$?; fi
  fi
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
