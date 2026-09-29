#!/usr/bin/env bash
# refs.sh — do this log's own cross-references resolve?
#
# Pass 329 recorded rule 329: "a handoff pointer is an instrument, and it needs a
# control like any other." Passes 327 and 328 each told the next pass "use the
# gate list at the top of this file"; there was none there, and it survived two
# passes because no control in this directory asserts that a pointer RESOLVES —
# every other control runs an instrument and asserts on its output.
#
# It stayed broken after the fix too, which is the part worth recording. Pass 329
# copied the gate list to the top and, in the same paragraph, cited it as
# "line 9081". Line 9081 is a closing code fence; the list is at 9116. The number
# was wrong on arrival, and it is wrong-forever by construction: a line number in
# an append-only file is stale the moment the next entry lands. So the standing
# section now cites a HEADING, and this script is the control that keeps it
# doing so.
#
# SCOPE, and what a clean run claims
#
# It claims three things about the READER-FACING SECTION of this log:
#   1. every relative Markdown link target in it resolves to a real file;
#   2. every instrument path it names exists on disk;
#   3. it contains no self-pointer — a line number attributed to THIS file.
# It claims nothing about the 9,000-line body, which is append-only history and
# will always contain stale internal citations. That is fine: history is not an
# instruction to the next pass, and a check that condemned it would be this
# directory's recurring defect (see pass 317) rather than a control.
#
# Usage:  refs.sh [ITEM-PATH]
# The argument exists so the failure cases can be PLANTED, exactly as
# selfcheck.sh's instrument-directory argument does. See the plants in the
# pass-330 entry.

set -uo pipefail

ITEM="${1:-}"

# The repo root is resolved, not inherited (selfcheck.sh's pass-317 fix, same
# defect: this script hands paths to nothing, but `git rev-parse` and the item's
# own relative links both depend on where the caller was standing).
if ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" && [ -n "$ROOT" ] && [ "${ROOT#/}" != "$ROOT" ]; then
  cd "$ROOT" || { echo "refs: cannot cd to $ROOT" >&2; exit 3; }
fi

[ -n "$ITEM" ] || ITEM="docs/work/items/w-paused-reconciliation.md"
case "$ITEM" in
  /*) ;;
  *) ITEM="$(pwd)/$ITEM" ;;
esac

if [ ! -f "$ITEM" ]; then
  echo "refs: MISSING item $ITEM" >&2
  echo "refs: a missing document is not a document with no broken pointers" >&2
  exit 2
fi

DIR="$(dirname "$ITEM")"
status=0

# --- the population: the preamble, without its preserved history ---------------
# Derived, not hard-coded, so this script does not become the stale pointer it
# exists to catch. The population is the WHOLE preamble — everything before the
# first `## Pass ` heading — minus two kinds of preserved text:
#
#   * the frontmatter block (schema keys). Note it ends at line 10 on this item;
#     the several-KB `prior_owner:` / `updated:` run that follows is NOT
#     frontmatter, it is displaced history that passes 218/252/289 moved into
#     the body verbatim rather than dropping. It is excluded by its key prefix
#     for the same reason it is excluded from the census: it is a record of what
#     past passes said, not an instruction to the next one.
#   * the append-only log body, which will always contain stale internal
#     citations because that is what history is.
#
# If the first `## Pass ` heading is ever renamed, the extraction runs away into
# the body; the population check below refuses rather than reporting a vacuous
# clean. That fail-closed direction is the point.
FIRSTPASS="$(grep -n '^## Pass ' "$ITEM" | head -1 | cut -d: -f1)"
[ -n "$FIRSTPASS" ] || FIRSTPASS=1
extract() {
  head -n "$(( FIRSTPASS - 1 ))" "$ITEM" | awk '
    NR==1 && /^---[ \t]*$/ { fm=1; next }
    fm && /^---[ \t]*$/ { fm=0; next }
    fm { next }
    { print }
  ' | grep -vE '^(prior_owner|updated|owner|parent_item|superseded_by):'
}

slurp="$(mktemp)"; prose="$(mktemp)"
trap 'rm -f "$slurp" "$prose"' EXIT
extract >"$prose"

nprose="$(wc -l <"$prose" | tr -d " ")"
BODY_LINES="$(wc -l <"$ITEM" | tr -d " ")"
if [ "$FIRSTPASS" -gt "$BODY_LINES" ]; then
  echo "refs: BROKEN POPULATION — first '## Pass ' heading at line $FIRSTPASS of $BODY_LINES" >&2
  echo "refs: the heading was probably renamed; a check over the whole body is not a pass" >&2
  exit 2
fi
if [ "$nprose" -lt 5 ]; then
  echo "refs: BROKEN POPULATION — the reader-facing section extracted $nprose line(s)" >&2
  echo "refs: '## Current gate status' was probably renamed; a check over nothing is not a pass" >&2
  exit 2
fi

# --- 1. relative markdown links ------------------------------------------------
links=0; badlinks=0
while IFS= read -r target; do
  [ -n "$target" ] || continue
  links=$((links + 1))
  path="${target%%#*}"                      # the FILE must resolve; an anchor is not checked
  case "$path" in
    http://*|https://*|mailto:*) continue ;;
  esac
  if [ ! -e "$DIR/$path" ]; then
    printf 'refs: BROKEN LINK  %s -> %s\n' "$target" "$path" >&2
    badlinks=$((badlinks + 1))
  fi
done < <(grep -oE '\]\((\.\.?/)[^)]*\)' "$prose" | sed -E 's/^\]\(//; s/\)$//' | sort -u)

# --- 2. instrument paths named in the section ---------------------------------
# Checked even when only mentioned, because the section's purpose is to say which
# files to run; naming a file that is not there is the defect rule 329 is about.
insts=0; badinsts=0
while IFS= read -r path; do
  [ -n "$path" ] || continue
  insts=$((insts + 1))
  if [ ! -e "$path" ]; then
    printf 'refs: MISSING INSTRUMENT  %s\n' "$path" >&2
    badinsts=$((badinsts + 1))
  fi
done < <(grep -oE 'docs/work/paused-recon/[A-Za-z0-9._-]+' "$prose" | sort -u)

# --- 3. self-pointers into this file (rule 330) --------------------------------
# TWO EARLIER VERSIONS OF THIS CHECK EXISTED AND BOTH ARE WRONG, recorded here
# because a check that condemns correct text is the pass-317 defect in a new
# place, and is worse than shipping no check at all.
#
# v1 banned every `line N` in the standing section. Wrong: the section
# LEGITIMATELY cites line numbers in OTHER files — src/lib.rs:3597, the
# fence.awk region boundaries, src/approx.rs:1041 — and those citations are
# precisely what lets a reader re-derive the fence result instead of trusting it.
#
# v2 kept only the claims attributed to "this file"/"this log". STILL wrong, and
# undecidably so: over the whole file it fired 6 times, 5 on the verbatim
# prior_owner history (8 KB on single lines, containing the attribution words and
# dozens of correct figures about other files) and 1 on the sentence explaining
# the fix it was making. A keyword scan cannot tell a live claim from a quoted
# one. The general form is rule 14k approached from the other side: a
# self-referential measurement in prose is not policable by grep, so it must not
# be published in prose in the first place.
#
# What IS decidable is structure, which is what the extraction above buys. Inside
# the reader-facing section a `line N` that also names this file is a self-pointer
# and is banned outright rather than checked for correctness — there is no
# correct value, because the number is stale the moment the next entry lands.
selfptr=0
while IFS= read -r hit; do
  [ -n "$hit" ] || continue
  case "$hit" in
    *"this file"*|*"this log"*|*"this item"*|*"this section"*|*"this document"*|*"w-paused-reconciliation"*)
      printf 'refs: SELF-POINTER  %s\n' "$hit" >&2
      selfptr=$((selfptr + 1)) ;;
  esac
done < <(grep -E '\b(lines?) [0-9][0-9,]*' "$prose" | tr -s ' ' || true)
if [ "$selfptr" -gt 0 ]; then
  printf 'refs: %s self-pointer(s) into this file (rule 330: cite a heading, not a line)\n' "$selfptr" >&2
fi

# Each defect class sets the verdict. They are NOT folded into $status above,
# because a first version of this script incremented $badlinks/$badinsts,
# printed "BROKEN LINK" on stderr, and then still exited 0: the summary block
# tested $status, which nothing had touched. The plant below caught it, which
# is the only reason it is known. A detector that reports and does not fail is
# the same shape as pass 298's instrument that "succeeded on the failure case".
[ "$badlinks" -gt 0 ] || [ "$badinsts" -gt 0 ] || [ "$selfptr" -gt 0 ] && status=1

# --- summary -------------------------------------------------------------------
# Printed on BOTH paths: a clean run is never silent, a failing one is never
# quiet. selfcheck.sh anchors on the clean line, so this summary is the
# invariant a future pass can register.
if [ "$status" -eq 0 ]; then
  echo "refs: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
  echo "refs: ${links} relative link(s) resolve, ${insts} instrument path(s) resolve, 0 self-pointers"
  echo "refs: cross-references resolve"
  exit 0
fi

echo "refs: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
echo "refs: ${links} relative link(s), ${badlinks} broken; ${insts} instrument path(s), ${badinsts} missing; ${selfptr} self-pointer(s)"
echo "refs: REFUSING — a pointer that does not resolve sends the next pass to the wrong line" >&2
exit 1
