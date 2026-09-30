#!/usr/bin/env bash
# item-state.sh — the paused-recon work item's OWN current state, in one command.
#
# THE GAP, and why it is not cosmetic.
#
# scheduled.md, "Work discovery and ownership", requires of every pass:
#   "Read the selected work item's current state and handoff notes before claiming it."
# and work-items.md makes a coordinator's ability to parse the frontmatter the
# mechanism by which work is discovered at all. For the other 95 items that is
# cheap: a few KB, and census.sh enumerates their states. For THIS item it is
# not. It is 28,218 lines / 2.3 MB, because it carries a per-pass log. So the
# one file the protocol tells a pass to read FIRST is the one file in the
# repository that cannot be read without a multi-megabyte tail, and the state a
# pass needs first -- state, owner, branch, worktree, and the next action the
# previous pass left for it -- is the state buried furthest from the top.
#
# This log has raised its own size as an escalation on many passes without
# instrumenting it, which is the failure class rules 14q/14r name: the thing
# every other script is careful about (a fact that is measured) versus this one
# (a file that is consumed by eye). Five green instruments each report a
# FACT; none reported that the file they all log into is 2.3 MB of hand-typed
# history, and a pass that reads the first 2,000 lines of it would learn the
# state is owned by a coordinator from two days ago and stop there.
#
# WHAT A CLEAN RUN CLAIMS:
#   - the item's frontmatter parses as a closed leading YAML block;
#   - the eight work-items.md schema keys are present and the state is one of
#     the five allowed values;
#   - the LAST `## Pass N` heading in the file is the newest entry, and the
#     NEXT guidance is quoted from it (the log's own rule: latest entry is the
#     last section);
#   - the file's size and line count, so growth is a number and not a vibe.
#
# WHAT IT DOES NOT CLAIM: that any of the log's historical entries are true.
# That is the other five instruments' business. This is a reader for the
# handoff, not a second opinion on the facts.
#
# FAIL-CLOSED. Any of the following exits 1 rather than printing a plausible
# answer, because a reader that silently falls back to "no handoff found" when
# the handoff is present-but-unparseable manufactures exactly the wrong
# conclusion -- that a pass left nothing, when a pass left something this
# script failed to read:
#   - the item file is missing;
#   - the leading `---` block does not close (the pass-218 defect class);
#   - a required schema key is absent, or `state` is not an allowed value;
#   - no `## Pass ` heading is found at all.
#
# Pass 218's lesson is load-bearing here: that frontmatter held 35 duplicate
# keys whose unquoted values contain ": ", so any conforming reader failed with
# `mapping values are not allowed in this context`. This script must therefore
# NOT parse the block itself with a hand-rolled loop and then report success --
# it delegates to `yq`, and asserts the delegation actually happened. Pass 298
# found `yq` absent from PATH making frontmatter.sh exit 0 on a clean 0, i.e.
# an instrument that succeeds on the failure case; so the absence of yq is
# checked FIRST and is its own distinct failure, not a silent fallback.

set -u
set -o pipefail

# root is resolved, not assumed (pass 317 defect) ------------------
# census.sh and frontmatter.sh already `cd "$(git rev-parse --show-toplevel)"`.
# This script did not, so its ITEM path was relative to the CALLER's cwd. From
# the repo root that happened to resolve; from any other directory the file was
# genuinely absent and the script printed "item file not found" and exited 1.
# That is fail-closed, so it was not a false PASS -- but it was a false ALARM
# about the wrong cause, and it propagated: selfcheck.sh runs the instruments
# with the caller's inherited cwd, so `selfcheck.sh` from docs/ reported
# "item-state.sh DEAD" and then "the instrument set is not trustworthy as it
# stands". A pass following that would have "repaired" a healthy instrument.
#
# General form: an instrument that takes no path argument still names paths, and
# a relative path is an argument to the filesystem with an ambient cwd the
# script did not choose. Every instrument here must be invocation-independent:
# a standing fact must not be a function of where the coordinator happened to
# be standing.
#
# The caller's cwd is captured BEFORE the root is resolved, because a relative
# argument is relative to the caller, and resolving the root must not change
# what a relative argument means.
CALLER_PWD="$PWD"
ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" \
  || { echo "item-state: not in a work tree" >&2; exit 3; }
[ -n "$ROOT" ] || { echo "item-state: not in a work tree" >&2; exit 3; }
case "$ROOT" in
  /*) ;;
  *)  echo "item-state: not in a work tree" >&2; exit 3 ;;
esac

# The DEFAULT ITEM is joined to the resolved root. An explicit relative
# argument is tried against the CALLER's cwd first and the root second, so
# neither the documented default nor a hand-passed path depends on where the
# coordinator was standing. Absolute arguments are used as given.
ITEM="${1:-docs/work/items/w-paused-reconciliation.md}"
case "$ITEM" in
  /*) ;;
  *)
    if [ -f "$CALLER_PWD/$ITEM" ]; then
      ITEM="$CALLER_PWD/$ITEM"
    else
      ITEM="$ROOT/$ITEM"
    fi
    ;;
esac
FAILED=0

fail() { printf 'item-state: FAIL: %s\n' "$1" >&2; FAILED=1; }

[ -f "$ITEM" ] || { fail "item file not found: $ITEM"; exit 1; }

# --- yq must exist BEFORE anything else (pass 298 defect) ---------------------
if ! command -v yq >/dev/null 2>&1; then
  fail "yq not on PATH; this script will not hand-roll a YAML parse (pass 298)"
  exit 1
fi

# --- the leading frontmatter block must be closed -----------------------------
first_line="$(head -1 "$ITEM")"
[ "$first_line" = "---" ] || fail "file does not begin with a '---' frontmatter block"

# Close the block the way a conforming reader locates it: the FIRST '---' at
# column 0 on a line of its own. Not `grep -n '---' | head -2` arithmetic.
close_line="$(awk 'NR>1 && $0=="---" {print NR; exit}' "$ITEM")"
[ -n "$close_line" ] || fail "leading frontmatter block does not close"

# --- parse via yq, and assert the parse really happened -----------------------
# Duplicate keys make `yq` fail; that failure must surface, not be absorbed.
# head to close_line-1, NOT close_line: the closing '---' would make yq see a
# SECOND (null) document, and `.state` would then return the state value
# concatenated with '---' and 'null'. Found on this script's first live run.
if head -n "$((close_line - 1))" "$ITEM" | yq -P '.' - >/dev/null 2>/dev/null; then
  :
else
  fail "frontmatter is not parseable YAML (duplicate keys or unquoted ': ' values?)"
fi

# --- required schema keys, per work-items.md ---------------------------------
for key in work_item id state priority owner updated branch worktree; do
  v="$(head -n "$((close_line - 1))" "$ITEM" | yq -r ".${key} // \"\"" 2>/dev/null)"
  if [ -z "$v" ]; then
    fail "required schema key absent or null: ${key}"
  else
    printf '  %-10s %s\n' "$key" "$v"
  fi
done

# --- state must be one of the five allowed values ----------------------------
state="$(head -n "$((close_line - 1))" "$ITEM" | yq -r '.state' 2>/dev/null)"
case "$state" in
  open|working|blocked|done|superseded) : ;;
  *) fail "state '${state}' is not one of open|working|blocked|done|superseded" ;;
esac

# --- the next-pass label predicate, as ONE function ----------------------------
# PASS 339, AND IT IS THE FAILURE CLASS THIS SCRIPT WAS WRITTEN TO CATCH.
#
# This predicate used to be `/^(NEXT|### Next pass)/` -- a CLOSED ENUMERATION of
# the two spellings that had been written when it was authored. Pass 338 wrote a
# third (`**Next pass: prefer no entry at all.**`), so the predicate matched
# nothing, the script exited 1 with "the handoff is unreadable", and the two
# statements it makes -- "no NEXT guidance" and "guidance is present but spelled
# differently" -- became indistinguishable. selfcheck.sh then reported
# item-state.sh DEAD and REFUSED the whole instrument set, over a Markdown
# emphasis marker.
#
# That is this log's recurring failure mode at its worst, and it is a *reader*
# failing, not a fact: a reader that reports "no handoff" when a handoff is
# present teaches the next pass that nothing was left for it. The handoff WAS
# there, and it said to prefer no entry at all.
#
# The general form: a predicate written as a list of the spellings observed so
# far is a census, not a rule; it fails on the first spelling nobody thought of,
# and the failure is indistinguishable from the absence it exists to detect. Match
# the PROPERTY -- a line that opens a next-pass handoff, under any Markdown
# decoration -- and not an enumeration of the decorations seen so far.
# Sanctioned spellings, all three of which occur in this log today:
#     NEXT: ...            (30 entries, the majority form)
#     ### Next pass ...    (heading form)
#     **Next pass: ...**   (bold prose, pass 338)
# Leading '#'/'*'/space are stripped first, so the property is "the first
# undecorated token sequence is NEXT or Next pass", not "the line looks like one
# of these three".
next_label_line() {
  # $1 = file, $2 = heading line number. Prints the line number of the first
  # next-pass label strictly after the heading, or nothing.
  awk -v h="$2" '
    NR > h {
      s = $0
      sub(/^[#*[:space:]]+/, "", s)          # strip Markdown decoration
      # The boundary is written as an explicit character class, NOT as \b: in
      # GNU Awk \b is BACKSPACE (word boundaries are \< and \>), so a \b here
      # silently never matches and the reader fails closed on EVERY entry. That
      # is the job of the control set below -- the first version of this repair
      # shipped that bug and the control caught it before publication.
      # (No apostrophes in these comments: the awk program is single-quoted.)
      if (s ~ /^NEXT([^A-Za-z0-9_]|$)/ || s ~ /^Next[ ]+pass([^A-Za-z0-9_]|$)/) {
        print NR; exit
      }
    }
  ' "$1"
}

# PASS 339 CONTROLS. The predicate above is now the load-bearing part of the
# script and it is the part that was wrong, so it gets a control set, and the
# control calls the SAME function rather than a second copy of the regex
# (rule 262: a control written from a re-spelling of the pattern certifies the
# re-spelling). Each control is a real handoff spelling that MUST be found, plus
# a near-miss that must NOT be, plus the absence that must still fail.
# Failure modes are 0-read and 1-read, not just right-versus-wrong (rule 334).
#
# Every line number in the expectations is DERIVED from the fixture, never
# written down. The first version hardcoded them and all five controls "failed"
# against a predicate that was working, because a control that pins a line
# number fails for the reason rule 330 is about -- it certifies a layout rather
# than a property, and it reports the reader's own bookkeeping as a defect.
ctl_dir="$(mktemp -d)"
trap 'rm -rf "$ctl_dir"' EXIT
ctl_bad=""

# write_fixture <file> <label-line-text>; builds a two-pass item whose OLD entry
# carries a decoy label, so the control also proves the selection is scoped to
# the NEWEST entry.
write_fixture() {
  {
    echo "---"; echo "work_item: true"; echo "id: w-ctl"; echo "state: blocked"
    echo "---"
    echo "## Pass 1 (x)"; echo "old entry"; echo "NEXT: decoy in the OLD entry"
    echo "## Pass 2 (y)"; echo "newest entry"
    echo "$2"
    echo "the guidance body"
  } > "$1"
}

# newest_heading <file> -- derived, never assumed.
newest_heading() { grep -n '^## Pass ' "$1" | tail -1 | cut -d: -f1; }

for ctl_text in \
  'NEXT: do the thing' \
  '### Next pass' \
  '**Next pass:** do the thing' \
  '#### NEXT — the thing' \
  '  * **Next pass** — the thing' \
  'NEXT' \
; do
  write_fixture "$ctl_dir/item.md" "$ctl_text"
  # The label is the line AFTER the newest heading plus the 2 prose lines the
  # fixture puts between them; derive it by searching the fixture for the exact
  # planted text rather than counting.
  ctl_want="$(grep -nFx -- "$ctl_text" "$ctl_dir/item.md" | tail -1 | cut -d: -f1)"
  ctl_got="$(next_label_line "$ctl_dir/item.md" "$(newest_heading "$ctl_dir/item.md")")"
  if [ "$ctl_got" != "$ctl_want" ]; then
    ctl_bad="$ctl_bad '$ctl_text'->'$ctl_got'(want $ctl_want)"
  fi
done

# NEGATIVE control 1: a word that merely STARTS with NEXT must not be selected,
# and neither must prose that merely CONTAINS a next-pass phrase. The predicate
# matches a LABEL at the start of an undecorated line, with a real token
# boundary.
#
# PASS 339: this control was WRONG on its first run and a mutant proved it. It
# planted "the next passes all went fine, and NEXTWORD is a word" and expected
# no match, but that line does not BEGIN with NEXT, so it would not have matched
# a prefix-anchored predicate either -- the control could not fail, and the
# mutant with the token boundary DELETED read green through it. The line must
# start with the bait, or the control certifies nothing. This is rule 262 one
# level up: the control has to be reachable by the defect it is meant to catch.
write_fixture "$ctl_dir/item.md" "NEXTWORD is a word, and the next passes went fine"
ctl_got="$(next_label_line "$ctl_dir/item.md" "$(newest_heading "$ctl_dir/item.md")")"
[ -z "$ctl_got" ] || ctl_bad="$ctl_bad near-miss->'$ctl_got'(want empty)"

# NEGATIVE control 1b: a decorated label must still be found, so the controls
# cannot be satisfied by a predicate that only ever matches one spelling. This is
# the "the other two" of the 0-read/1-read pair in rule 334: mutant 1 loses the
# bold and heading forms, and this line is the same guard seen from the
# measurement side rather than the mutant side.
write_fixture "$ctl_dir/item.md" "**Next pass:** prefer no entry at all."
ctl_got="$(next_label_line "$ctl_dir/item.md" "$(newest_heading "$ctl_dir/item.md")")"
[ -n "$ctl_got" ] || ctl_bad="$ctl_bad decorated-label->'(want a line)'"

# NEGATIVE control 2: an entry with NO handoff must yield nothing, so the
# script's fail-closed path stays reachable and is not masked by a loose match.
{
  echo "---"; echo "work_item: true"; echo "id: w-ctl"; echo "state: blocked"
  echo "---"; echo "## Pass 2 (y)"; echo "newest entry, no handoff label at all"
} > "$ctl_dir/item.md"
ctl_got="$(next_label_line "$ctl_dir/item.md" "$(newest_heading "$ctl_dir/item.md")")"
[ -z "$ctl_got" ] || ctl_bad="$ctl_bad absence->'$ctl_got'(want empty)"

if [ -n "$ctl_bad" ]; then
  fail "control: the next-pass label predicate mis-selected:$ctl_bad -- the reader cannot be trusted; refusing"
fi
rm -rf "$ctl_dir"; trap - EXIT

# --- newest entry: the LAST '## Pass ' heading, and its NEXT guidance --------
# `grep | tail -1` rather than `head -1`: the log's own rule is that the latest
# entry is the LAST section, and an entry count is a number, not a position.
last_heading="$(grep -n '^## Pass ' "$ITEM" | tail -1)"
if [ -z "$last_heading" ]; then
  fail "no '## Pass ' heading found; the log's own rule (latest entry is the last section) cannot be applied"
else
  hno="${last_heading%%:*}"
  htext="${last_heading#*:}"
  printf '  %-10s %s\n' latest "$htext (line ${hno})"

  # The handoff a pass is required to read: the newest entry's NEXT guidance,
  # which is the block of non-heading prose after its next-pass LABEL. The
  # predicate is next_label_line() above -- one implementation, shared with the
  # controls above (pass 339), because the label's spelling is a property of the
  # prose and not a fixed enumeration of the spellings seen so far.
  # Bounded so a malformed log cannot make this print the whole file.
  next_line="$(next_label_line "$ITEM" "$hno")"
  if [ -n "$next_line" ]; then
    printf '  next action (from the newest entry, line %s):\n' "$next_line"
    # Print the WHOLE next-action block, not a fixed 5 lines.
    #
    # A fixed window of 5 lines was a real defect, found on the second live run
    # of this script (the first live run found the head -n close_line bug). The
    # block it opened is a paragraph of wrapped prose, so a line count truncates
    # mid-sentence -- and here it truncated BEFORE the last line, which is the
    # one that matters most:
    #
    #   **Blocked on the human reopen/confirm decision.**
    #
    # A reader that reports "the pause holds, the facts stand, CI does not run
    # the fence" and stops there reports the three things a pass may act on and
    # omits the one thing that says a pass must NOT act. A handoff reader that
    # can clip is a handoff reader that can be quietly wrong in the direction
    # that manufactures work.
    #
    # The block ends at the next `## ` heading or the end of the file, and is
    # hard-capped so a malformed log with no next heading cannot print the rest
    # of a 2.3 MB file.
    end_line="$(awk -v s="$next_line" '
      NR > s && /^## / { print NR; exit }
    ' "$ITEM")"
    [ -n "$end_line" ] || end_line=$(( $(wc -l < "$ITEM") + 1 ))
    if [ "$((end_line - next_line))" -gt 40 ]; then
      end_line=$(( next_line + 40 ))
      printf 'item-state: next-action block is longer than 40 lines; TRUNCATED at 40 (line %s)\n' "$end_line" >&2
    fi
    sed -n "${next_line},$((end_line - 1))p" "$ITEM" | sed 's/^/    /'
  else
    fail "newest entry '${htext}' contains NO next-pass label (NEXT: / '### Next pass' / '**Next pass**:' under any heading or emphasis decoration); the handoff is unreadable"
  fi
fi

# --- size, so growth is a measurement -----------------------------------------
lines="$(wc -l < "$ITEM" | tr -d ' ')"
bytes="$(wc -c < "$ITEM" | tr -d ' ')"
printf '  %-10s %s lines / %s bytes\n' size "${lines}" "${bytes}"

if [ "$FAILED" -ne 0 ]; then
  printf 'item-state: one or more checks FAILED; do not treat the absence of output as an empty handoff\n' >&2
  exit 1
fi

printf 'item-state: item frontmatter parses, all eight schema keys present, state allowed\n'
printf 'item-state: the block above is the NEWEST entry and its next action, read from the file\n'
printf 'item-state: it is a reader for the handoff only; the facts are census/clue-fence/agents/at-risk\n'
exit 0
