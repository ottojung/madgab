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

# --- 4. section-NAME self-pointers (rule 350) ----------------------------------
# Rule 330 banned one SHAPE of self-pointer (a line number) and rule 330's own
# generalisation is "cite a HEADING, not a line number". This check exists because
# that generalisation has a second half nobody had written down: **a pointer to a
# heading is not thereby correct.** A heading name can be wrong in exactly the way a
# line number is — and unlike a line number it HAS a correct value, so it is
# decidable and a check is possible.
#
# THIS PASS'S DEFECT, live in the reader-facing section on arrival: the census
# table's closing paragraph sent the reader to "the latest entry, §"Declined"".
# No heading in this file has ever been named "Declined" — `grep '^#\+ .*Declined$'`
# returns 0 rows, and `grep -n '^#\+ .*[Dd]eclined'` returns only PASS HEADINGS that
# happen to contain the word. The real target is the subsection
# `### The invocation's three standing clauses are declined again, ...`. So the
# pointer named a section that has never existed, in the one paragraph whose whole
# job is to tell the human where the declining-clauses record is.
#
# Why it survived: rule 330's check matches the TOKEN `line N`, and this pointer has
# no number in it. refs.sh's own scope banner claimed "no self-pointer — a line
# number attributed to THIS file", so the claim was true as written and the reader-
# facing section still contained a self-pointer. A scope claim that names the one
# shape it checks is a shape, not a class.
#
# WHY ONLY THE QUOTED FORM IS CHECKED, and this is the load-bearing scoping
# decision rather than a convenience. Two §-forms exist in this document:
#   * §"Declined"  — a quoted NAME. Unambiguously a pointer INTO THIS FILE.
#   * §3, §4, §7   — bare NUMBERS, and every one of them points into a DIFFERENT
#     file: `OBSTRUCTION-MAP.md §3`, `... §4`, a `§7` recommendation. Checking
#     those against this file's headings would condemn correct citations of another
#     document — the pass-317 defect (a check that condemns correct text), which
#     refs.sh's own v1/v2 history says is worse than shipping no check at all.
# The discriminator is the quotes, and it is structural rather than semantic: a
# quoted name cannot be a reference to another file's numbering.
#
# 0 READS IS NOT A DEFECT HERE, and the reason is worth stating because gate 7
# makes the opposite call for the opposite reason. Gate 7 condemns 0 claims because
# the section it reads ASSERTS a merge — the section is supposed to contain one, so
# its absence is the defect. This section is under no obligation to contain a quoted
# section pointer, so 0 is not a defect ON ITS OWN. But it is not a safe resting
# state either, and pass 351 measured why: pass 350's own repair to the real pointer
# left it as a BACKTICKED leading-words citation, which the `§"..."` matcher below
# cannot see, so the population fell to 0 for a reason that had nothing to do with
# the document being pointer-free. With the pointer back in the quoted form the
# population is 1 and the check is anchored to the real instance. So the live rule is
# not "0 is fine" and not "0 is a defect": it is that 0 must be the reading of a
# document that has no such pointer, and the way to tell the two apart is to PLANT
# the defect (below) — a plant on an empty population passes for the wrong reason.
secptrs=0; badsec=0
item_headings="$(mktemp)"
# The resolution set is the file's SECTION headings with the append-only `## Pass `
# log entries EXCLUDED — refs.sh's own scope banner already says the log body "is
# not an instruction to the next pass", so a reader-facing pointer may not target
# one. Normalised to lower case with runs of space collapsed, so a citation need
# not reproduce a heading byte-exactly.
grep -E '^#+ ' "$ITEM" | grep -vE '^#+ Pass [0-9]' \
  | sed -E 's/^#+ +//' | tr 'A-Z' 'a-z' | tr -s ' ' >"$item_headings"
while IFS= read -r name; do
  [ -n "$name" ] || continue
  secptrs=$((secptrs + 1))
  # LEADING-WORDS match, not substring. A citation names a section by its OPENING
  # words, as every heading in this file is written to be cited. Substring matching
  # is fail-open here and the plant below proved it on this pass's own defect: with
  # `grep -F`, the planted §"Declined" resolved against a PASS heading containing the
  # word "declined" (e.g. "...the three invocation clauses declined on the
  # itinerary's own text") and the check went GREEN on the exact defect it was built
  # to catch. That is rule 335's shape one level up — a text matcher that cannot
  # distinguish the claim from a coincidental neighbour.
  needle="$(printf '%s' "$name" | tr 'A-Z' 'a-z' | tr -s ' ')"
  if ! awk -v n="$needle" 'index($0, n) == 1 { found = 1 } END { exit !found }' "$item_headings"; then
    printf 'refs: BROKEN SECTION POINTER  §"%s"  — no section heading in this file BEGINS with that name\n' "$name" >&2
    badsec=$((badsec + 1))
  fi
done < <(grep -oE '§"[^"]+"' "$prose" | sed -E 's/^§"//; s/"$//' | sort -u || true)

# --- §"Name" pointer QUALIFIER, added at pass 354 -----------------------------
# Rule 350 established that "a pointer to a heading is not thereby correct". The
# loop above decides the NAME. It DROPS THE QUALIFIER, and the standing section
# carries exactly one qualified pointer, in the one paragraph whose job is to
# send the reader to the declining-clauses record:
#
#   "see the newest entry's §"The invocation's three standing clauses are
#    declined again" subsection"
#
# LIVE DEFECT ON ARRIVAL, measured not inferred: that qualifier is FALSE. The
# newest entry is pass 353 (line 12015 of 12098) and contains ZERO occurrences of
# the named subsection; the newest entry that HAS one is pass 350 (line 11765).
# The loop above still read it GREEN, because the heading exists four times in
# OLDER entries (passes 343/344/345/350) and leading-words matching cannot tell
# "the one this pointer names" from "one of the four". That is rule 335's shape
# one level up: a matcher that cannot distinguish the claim from a coincidental
# neighbour of the same shape, resolving against a population the claim never
# named.
#
# So when a pointer says WHICH CONTAINER it means, the resolution set is scoped
# to that container and must resolve there as well as file-wide.
#
# The qualifier list is deliberately NARROW — a short list of container words
# this document actually uses. A qualifier this script cannot decide is IGNORED
# rather than guessed at, because guessing condemns correct text (pass 317), and
# pass 353 recorded the same boundary for the backticked `## Name` form: the
# right move here is one discriminator that survives both known failure modes,
# not a second grep that fires on prose.
#
# The context window is the pointer's own line and the two above it, because the
# governing qualifier and the pointer it governs sit in one wrapped sentence
# here. That is a stated limitation, not a claim of generality: a qualifier
# separated from its pointer by more than two lines would be ignored, and the
# 0-count below is printed precisely so that "ignored" is visible rather than
# silent.
newest_start="$(grep -n '^## Pass ' "$ITEM" | tail -1 | cut -d: -f1)"
newest_headings="$(mktemp)"
trap 'rm -f "$slurp" "$prose" "$item_headings" "$newest_headings"' EXIT
if [ -n "$newest_start" ] && [ "$newest_start" -le "$BODY_LINES" ]; then
  tail -n "+$newest_start" "$ITEM" | grep -E '^#+ ' \
    | sed -E 's/^#+ +//' | tr 'A-Z' 'a-z' | tr -s ' ' >"$newest_headings"
else
  : >"$newest_headings"
fi
qualptrs=0; badqual=0
while IFS= read -r hit; do
  [ -n "$hit" ] || continue
  pline="${hit%%$'\t'*}"
  pname="${hit#*$'\t'}"
  [ "$pline" != "$pname" ] || continue
  ctx="$(sed -n "$(( pline > 2 ? pline - 2 : 1 )),${pline}p" "$ITEM" 2>/dev/null || true)"
  printf '%s' "$ctx" | grep -qiE '(newest|latest|last|most recent|current)[[:space:]]+entr(y|ies)' || continue
  qualptrs=$((qualptrs + 1))
  qneedle="$(printf '%s' "$pname" | tr 'A-Z' 'a-z' | tr -s ' ')"
  # The container is `grep -n '^## Pass '` LAST, i.e. the highest pass number in
  # FILE order. In an append-only log those agree; the check takes the file order
  # because that is what a reader scrolling to the bottom arrives at.
  if [ -s "$newest_headings" ] \
     && ! awk -v n="$qneedle" 'index($0, n) == 1 { found = 1 } END { exit !found }' "$newest_headings"; then
    printf 'refs: BROKEN QUALIFIED POINTER  §"%s"  — the pointer says "the newest entry'"'"'s" subsection, and the newest entry (%s, line %s) has no heading beginning with that name\n' \
      "$pname" "$(sed -n "${newest_start}p" "$ITEM" | cut -c1-11)" "$newest_start" >&2
    badqual=$((badqual + 1))
  fi
done < <(awk -v lim="$FIRSTPASS" '
  NR < lim {
    rest = $0
    while (match(rest, /§"[^"]+"/)) {
      printf "%d\t%s\n", NR, substr(rest, RSTART, RLENGTH)
      rest = substr(rest, RSTART + RLENGTH)
    }
  }' "$ITEM" | sed -E 's/^([0-9]+)\t§"/\1\t/; s/"$//')

# Each defect class sets the verdict. They are NOT folded into $status above,
# because a first version of this script incremented $badlinks/$badinsts,
# printed "BROKEN LINK" on stderr, and then still exited 0: the summary block
# tested $status, which nothing had touched. The plant below caught it, which
# is the only reason it is known. A detector that reports and does not fail is
# the same shape as pass 298's instrument that "succeeded on the failure case".
[ "$badlinks" -gt 0 ] || [ "$badinsts" -gt 0 ] || [ "$selfptr" -gt 0 ] || [ "$badsec" -gt 0 ] || [ "$badqual" -gt 0 ] && status=1

# --- summary -------------------------------------------------------------------
# Printed on BOTH paths: a clean run is never silent, a failing one is never
# quiet. selfcheck.sh anchors on the clean line, so this summary is the
# invariant a future pass can register.
if [ "$status" -eq 0 ]; then
  echo "refs: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
  echo "refs: ${links} relative link(s) resolve, ${insts} instrument path(s) resolve, 0 self-pointers"
  echo "refs: ${secptrs} quoted section pointer(s) resolve, 0 broken"
  echo "refs: ${qualptrs} qualified pointer(s) checked against the newest entry (${newest_start:-none}); 0 unresolved in it"
  echo "refs: cross-references resolve"
  exit 0
fi

echo "refs: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
echo "refs: ${links} relative link(s), ${badlinks} broken; ${insts} instrument path(s), ${badinsts} missing; ${selfptr} self-pointer(s); ${secptrs} quoted section pointer(s), ${badsec} broken; ${qualptrs} qualified pointer(s), ${badqual} unresolved in the newest entry"
echo "refs: REFUSING — a pointer that does not resolve sends the next pass to the wrong line" >&2
exit 1
