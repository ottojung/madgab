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
  # at-risk-delta.sh is registered on its SHAPE -- that it names a delta and
  # classifies the tree proxy -- and NOT on either figure. Both are counts over
  # populations that move with this log's own commits, and rule 14k is exactly
  # "a standing figure is not a standing procedure": pinning "named 514ed91"
  # would go red the moment that commit is finally pushed, and pinning
  # "62 without" would go red on any commit added. The instrument is fail-closed
  # on its own controls, so a green liveness check here is the honest one.
  # (pass 322; guard added at pass 327)
  #
  # Pass 327 added a GROWTH GUARD that exits 3 when either arm exceeds the
  # count the previous pass recorded, because the old verdict line hard-coded
  # "88" and absorbed real growth as a "spelling difference". A guard that fires
  # on live data would turn this liveness check red for a reason that is not a
  # defect, so the guard is disarmed HERE ONLY, by pinning PREV_* above the live
  # arms. That keeps the self-referential discipline of the other registrations
  # (this can never go stale as the counts move) while still exercising the
  # guard's comparison and exit-0 path. The `--prev-arms/--prev-pub` flags are
  # ARGUMENTS, not an environment prefix, because this harness invokes
  # `"$path" $args` — the script comes FIRST, so there is nowhere to put a
  # `NAME=v` prefix and `env NAME=v CMD` degenerates into the script being run
  # with `env`, `NAME=v` and `CMD` as $1 $2 $3. (That was tried and it does not
  # work: at-risk-delta.sh tests `[ "${1:-}" = --fetch ]`, so the assignments
  # never bind, the guard stays armed and this check stays red for a reason
  # that is not a defect. Recording it because it is the second time in this
  # file's history that a plausible-looking fix was wrong in a way that only
  # showed up on execution — rule 14q.) The guard's FIRED case is exercised in
  # the pass-327 entry beside this instrument, where both directions are shown.
  "at-risk-delta.sh::is a PROXY and reads alarming::--prev-arms 99999 --prev-pub 99999"
  "frontmatter.sh::failed to parse::"
  "item-state.sh::item frontmatter parses::"
  # branch-containment.sh is registered on a SELF-REFERENTIAL case -- base,
  # composed and constituent are all origin/main -- and NOT on the live human
  # review branches, which would couple this instrument set's health to the
  # human list's lifetime. That coupling is the pass-317 defect in a new place:
  # a human who merges `review/drop-dead-trace-and-fence` and deletes the three
  # superseded branches would turn a healthy selfcheck red and send the next
  # pass to "repair" an instrument that is fine. The self-referential case still
  # exercises everything that can rot -- argument parsing, ref resolution across
  # the audit mirror, the rule-323 base check, the merge-tree invocation, and the
  # contained path -- and it can never go stale. The DISCRIMINATING controls
  # (NOT CONTAINED, and the bogus-ref refusal) cannot be registered this way at
  # all, because each needs a real pair of branches that differ; they are
  # exercised in the pass-325 entry beside the instrument. (pass 325)
  "branch-containment.sh::CONTAINED and mergeable::origin/main origin/main origin/main"
  # refs.sh is registered on its CLEAN line, "cross-references resolve", and
  # takes no arguments here so it measures THIS item, the standing one. That is
  # the coupling pass 325 warned about for branch-containment.sh, and it is
  # accepted deliberately: this instrument's entire subject IS the standing
  # item's own pointers, so registering it on a throwaway copy would check
  # nothing. If a human's own edit breaks a pointer in the standing section, a
  # red selfcheck here is the correct answer, not a false alarm -- the fix is to
  # repair the pointer, which is the cheap action. Its FAILING cases (broken
  # link, missing instrument, self-pointer) are exercised in the pass-330 entry
  # beside it, in both directions, because pass 320's lesson is that registering
  # an instrument is not the same as having planted it.
  "refs.sh::cross-references resolve::"
  # branches.sh is registered on its CLEAN line, "every branch the section names
  # exists", for the same reason and with the same accepted coupling as refs.sh
  # above: its subject IS the standing item's own branch pointers. Unlike
  # refs.sh it is NOT a free-standing check, because it consults the remote
  # (`git ls-remote --heads origin`) to decide what resolves. That makes it the
  # one instrument here whose answer can change without anything in the
  # repository changing -- a human who deletes a branch the standing section names
  # will turn this red. That is the correct answer and not a false alarm, for
  # exactly the reason given above: the fix is to update the standing section,
  # which is the cheap action, and a stale branch name in a handoff is the defect
  # this instrument exists to catch. Its FAILING cases (a branch that exists
  # nowhere, a renamed `## Pass ` heading, a missing item) and its
  # FALSE-POSITIVE direction (worktree paths and directory rows, which are not
  # branches) are planted in the pass-331 entry beside it, because pass 320's
  # lesson is that registering an instrument is not the same as having planted it.
  "branches.sh::every branch the section names exists::"
  # compact-log.sh is DELIBERATELY absent from this list, and the reason is
  # worth recording because registering it looked obviously right.
  #
  # selfcheck liveness-checks the instruments that print a STANDING FACT, and it
  # does that by running each one bare and requiring exit 0. compact-log.sh is
  # not such an instrument: it is a one-shot maintenance action, and it is
  # correctly fail-closed once it has run -- a second run refuses with "archive
  # already exists". Registering it therefore turned correct behaviour into a red
  # selfcheck ("compact-log.sh DEAD exit=1") on a healthy script, which is the
  # exact failure this log records as the pass-317 defect: an instrument set that
  # condemns something healthy and sends the next pass to "repair" it.
  #
  # A dry run on a FRESH item does exit 0, so the alternative -- liveness-check it
  # only while the archive is absent -- would make the instrument set's health
  # depend on whether a one-shot job has already run. That is a worse coupling
  # than not checking it. It is exercised instead by its own --apply path, which
  # verifies its own output and refuses to write unless the split is a proven
  # line-multiset partition of the original. (pass 320)
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
