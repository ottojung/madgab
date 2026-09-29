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

ITEM="${1:-docs/work/items/w-paused-reconciliation.md}"
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
  # which is the block of non-heading prose after its last 'NEXT'/'### Next'
  # label. Bounded so a malformed log cannot make this print the whole file.
  next_line="$(awk -v h="$hno" '
    NR>h && /^(NEXT|### Next pass)/ { print NR; exit }
  ' "$ITEM")"
  if [ -n "$next_line" ]; then
    printf '  next action (from the newest entry, line %s):\n' "$next_line"
    sed -n "${next_line},$((next_line + 4))p" "$ITEM" | sed 's/^/    /'
  else
    fail "newest entry '${htext}' has no NEXT / '### Next pass' guidance; the handoff is unreadable"
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
