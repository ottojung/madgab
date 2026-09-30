#!/usr/bin/env bash
# branches.sh — does every BRANCH the reader-facing section names actually exist,
# and does its stated location match reality?
#
# Pass 330 added refs.sh, which checks links, instrument paths and self-pointers in
# the reader-facing section of this log. It checks three kinds of pointer. A branch
# name is a fourth, and it was the one the standing human list is made of: the list
# a human is told to act on is four branch names and what to do with each.
#
# That gap was live, and the pass-330 entries carry the defect it let through. Five
# pass entries, the newest of which is the handoff, tell a human to delete
# `review/drop-dead-trace-env-on-main` and add "(present only as `audit/…`)". The
# branch IS a live remote head, `refs/heads/review/drop-dead-trace-env-on-main` =
# 66e28ff, so the parenthetical sends a human looking for a local-only artefact,
# and a "delete the three superseded branches" instruction whose target is not where
# the log says it is will leave a live remote branch behind.
#
# The class is the pass-329/330 one exactly. Those passes established that a handoff
# pointer is an instrument and needs a control; refs.sh policed the pointers that
# are file paths and left the ones that are ref names. A pointer into a ref namespace
# decays the same way a pointer into a file does — a branch deleted, or renamed, or
# merged away, leaves the prose naming a thing that is gone.
#
# WHAT A CLEAN RUN CLAIMS
#
# For every branch-shaped token in the reader-facing section, this resolves it
# against all three places a branch can live and reports which:
#   local   refs/heads/<name>                       (a branch someone checked out)
#   audit   refs/remotes/audit/<name>              (the local mirror of remote heads)
#   remote  refs/heads/<name> on origin             (what a human can actually push/delete)
#
# It claims a branch is FINE when it resolves somewhere, and it reports the location
# so the prose can be checked against it by eye. It does NOT police the prose's
# wording — "present only as audit/…" is a claim about which of the three, and a
# keyword scan cannot adjudicate an English aside (see refs.sh's v1/v2, the pass-317
# defect: a check that condemns correct text is worse than no check). What it CAN
# decide, and does, is the resolvable half: a named branch that exists nowhere.
#
# A branch that resolves only as `remote` is reported as a distinct class, because
# that is the case a human acting on this log has to get right: a `git branch -D`
# will not touch it, and only a push will. The pass-330 defect is an instance.
#
# Usage:  branches.sh [ITEM-PATH]
# The argument exists so the failure cases can be PLANTED, as in refs.sh and
# selfcheck.sh. See the plants in the pass-331 entry.

set -uo pipefail

ITEM="${1:-}"

# The repo root is resolved, not inherited — selfcheck.sh's pass-317 fix, for the
# same reason refs.sh does it.
if ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" && [ -n "$ROOT" ] && [ "${ROOT#/}" != "$ROOT" ]; then
  cd "$ROOT" || { echo "branches: cannot cd to $ROOT" >&2; exit 3; }
fi

[ -n "$ITEM" ] || ITEM="docs/work/items/w-paused-reconciliation.md"
case "$ITEM" in
  /*) ;;
  *) ITEM="$(pwd)/$ITEM" ;;
esac

if [ ! -f "$ITEM" ]; then
  echo "branches: MISSING item $ITEM" >&2
  echo "branches: a missing document is not a document with no broken pointers" >&2
  exit 2
fi

# --- the population: identical to refs.sh, derived not hard-coded -------------
FIRSTPASS="$(grep -n '^## Pass ' "$ITEM" | head -1 | cut -d: -f1)"
[ -n "$FIRSTPASS" ] || FIRSTPASS=1

prose="$(mktemp)"
trap 'rm -f "$prose"' EXIT
head -n "$(( FIRSTPASS - 1 ))" "$ITEM" | awk '
  NR==1 && /^---[ \t]*$/ { fm=1; next }
  fm && /^---[ \t]*$/ { fm=0; next }
  fm { next }
  { print }
' | grep -vE '^(prior_owner|updated|owner|parent_item|superseded_by):' >"$prose"

nprose="$(wc -l <"$prose" | tr -d " ")"
BODY_LINES="$(wc -l <"$ITEM" | tr -d " ")"
if [ "$FIRSTPASS" -gt "$BODY_LINES" ]; then
  echo "branches: BROKEN POPULATION — first '## Pass ' heading at line $FIRSTPASS of $BODY_LINES" >&2
  echo "branches: the heading was probably renamed; a check over the whole body is not a pass" >&2
  exit 2
fi
if [ "$nprose" -lt 5 ]; then
  echo "branches: BROKEN POPULATION — the reader-facing section extracted $nprose line(s)" >&2
  echo "branches: a check over nothing is not a pass" >&2
  exit 2
fi

# --- collect the branch-shaped tokens -----------------------------------------
# Only the NAMESPACE/leaf shape. A bare word is not a branch name, and this file's
# standing section is full of prose; matching bare words is the pass-317 defect.
# The namespaces are the ones this repository's branches actually use. Deriving that
# list from the remote instead of hard-coding it would be better, but it would also
# mean a branch in a namespace nobody has used yet is invisible to the check that
# exists to catch exactly that — so the list is stated here, and its coverage is
# this file's comment, not a figure.
#
# `tmp/` and `scratch/` and `archive/` are included because the standing section
# cites branches in them; a token in the section is a claim the section makes.
# The leading lookbehind is load-bearing and its absence was this script's first
# live run producing THREE FALSE POSITIVES. `\b` fires after a `/`, so the tokens
# in the standing section's Worktrees row — the pruned worktree paths
# `/tmp/opencode/verify/w` and `/tmp/opencode/zzcheck`, and the directory row
# `madgab-scratch/examples/` — were all read as branch names in the `tmp/` and
# `scratch/` namespaces and reported UNRESOLVED. That is refs.sh's v1 defect wearing
# a different hat: the check fired on correct text. The first control run is what
# exposed it, and the plants below keep it exposed.
#
# Two classes of token are therefore excluded by construction rather than by
# post-filtering, because a post-filter cannot tell them from real ones:
#   * a token whose PRECEDING character is `/` or `.` or `-` or a word character
#     — it is a PATH COMPONENT, and this repository's worktrees live under
#     `/workspace/…` and `/tmp/…` while its directory rows are written
#     `madgab-scratch/examples/`;
#   * a token whose FOLLOWING character is `/` or a word character — a branch name
#     never ends in a separator, and a trailing `/` is how a DIRECTORY is written
#     in this log. The trailing lookahead is not redundant with the leading one:
#     a backticked `` `scratch/examples/` `` begins at a token boundary, so the
#     lookbehind does not see it, and without the lookahead the leaf is captured
#     bare and reported as an unresolved branch. That was caught by plant 2.
#
# Branch names here are 1, 2 or 3 slash-separated components (108/98/2 of the 208
# remote heads), so the leaf is matched loosely and the two BOUNDARY assertions,
# not the depth, are what do the work.
names="$(
  grep -oP '(?<![/.\w-])(review|scratch|recovery|archive|wip|mp2|tmp)/[A-Za-z0-9._-]+(?:/[A-Za-z0-9._-]+)*(?![/\w-])' "$prose" \
  | sed 's/[.,;)]*$//' \
  | sort -u
)"

nbranch=0; missing=0; remoteonly=0; nshaclaim=0; shabad=0
: > /tmp/.branches.$$.missing
: > /tmp/.branches.$$.remoteonly
: > /tmp/.branches.$$.shabad
trap 'rm -f "$prose" /tmp/.branches.$$.missing /tmp/.branches.$$.remoteonly /tmp/.branches.$$.shabad' EXIT

# --- the TIP claim: a branch name published WITH a sha is a claim about a commit --
# Pass 331's gate resolves branch NAMES. It does not resolve the sha the standing
# section publishes beside four of them, and that is the half a human acts on: the
# list says merge `review/drop-dead-trace-and-fence` = 8c88a59, delete
# `review/drop-dead-trace-env` (a29f3d7), `…-on-main` (66e28ff),
# `review/run-clue-fence-in-ci` (6edff83). A name that exists at a DIFFERENT commit
# still passes every name-level check, and a human merging "the branch" gets a commit
# the log never described.
#
# This is rule 323 ("a review branch is its tip AND its base") stated as an
# instrument instead of as advice. Rule 323 was written after pass 323 found
# review/drop-dead-trace-env parented on the accumulation line rather than main, and
# three passes described that branch as "prepared and validated" without either
# noticing. The defect survived because prose describing a branch is not a check on it.
#
# The sha is read from the SECTION, never hard-coded, so this cannot itself become
# the stale figure rule 14k exists to prevent: correct the sha in the prose and this
# follows it. The bare-prefix `git rev-parse <branch>^{commit}` is the shape that
# resolves regardless of which of the three namespaces the branch lives in, which is
# what a remote-only branch needs.
#
# The trailing-context filter is what keeps this to genuine NAME+sha pairs: the sha
# must sit within the same line as, and shortly after, the branch token. A sha in
# the same section with no branch of its own (the log's own commit shas, the
# origin/main tip) is not a branch claim and is not this check's business.
#
# SCOPE, and it is the whole design: the HUMAN LIST only. The reader-facing section
# also contains the historical recovery narrative ("## Third recovery pass", "## Fourth
# recovery pass"), where `recovery/probe-scaffolding-2026-09-28` = `51ebdd1` and later
# = `0a12e33` are RECORDS OF WHAT THE BRANCH WAS at the time each pass pushed it. The
# branch has since advanced to 2408c25, correctly. Checking those lines as claims reads
# a true history as a false defect — and an unscoped first run of this check did
# exactly that, reporting 1 tip mismatch on a branch that has never been wrong. The
# human list is delimited by its own heading and runs to the next `## ` heading; a
# record of the past is not an instruction to the future.
HUMANLIST="$(mktemp)"
trap 'rm -f "$prose" "$HUMANLIST" /tmp/.branches.$$.missing /tmp/.branches.$$.remoteonly /tmp/.branches.$$.shabad' EXIT
awk '
  /^## Current gate status/ { inh=1 }
  inh && /^## / && !/^## Current gate status/ { inh=0 }
  inh { print }
' "$prose" >"$HUMANLIST"
nsha_lines="$(wc -l <"$HUMANLIST" | tr -d " ")"

sha_claims="$(
  grep -oP '(?<![/.\w-])(review|scratch|recovery|archive|wip|mp2|tmp)/[A-Za-z0-9._-]+(?:/[A-Za-z0-9._-]+)*(?![/\w-])[^`]{0,12}`\s*\(?[=]?\s*\(?`\s*[0-9a-f]{7,40}' "$HUMANLIST" 2>/dev/null \
  | awk '{
      line = $0
      # The sha is the trailing hex run; the branch is the token before it.
      if (!match(line, /`[ \t]*\(?[=]?[ \t]*\(?`[ \t]*[0-9a-f]{7,40}$/)) next
      head_ = substr(line, 1, RSTART - 1)
      sha   = substr(line, RSTART, RLENGTH)
      gsub(/^[^0-9a-f]+/, "", sha)
      b = ""
      while (match(head_, /(review|scratch|recovery|archive|wip|mp2|tmp)\/[A-Za-z0-9._\/-]+/)) {
        cand = substr(head_, RSTART, RLENGTH); sub(/[.,;)]*$/, "", cand)
        b = cand
        head_ = substr(head_, RSTART + RLENGTH)
      }
      if (b != "") print b "\t" sha
    }' \
  | sort -u
)"

if [ -n "$sha_claims" ]; then
  while IFS="$(printf '\t')" read -r b sha; do
    [ -n "$b" ] || continue
    nshaclaim=$((nshaclaim + 1))
    actual="$(git rev-parse --verify --quiet "$b^{commit}" 2>/dev/null || true)"
    if [ -z "$actual" ]; then
      # Unresolvable here — the name-level pass above already reports this class, and
      # re-reporting it as a sha failure would double-count one defect as two.
      continue
    fi
    if [ "${actual#${sha}}" = "$actual" ] && [ "${sha#${actual}}" = "$sha" ]; then
      printf 'branches: SHA MISMATCH  %s  prose says %s, branch is %s\n' \
        "$b" "$sha" "$(printf '%s' "$actual" | cut -c1-7)" >&2
      shabad=$((shabad + 1))
      printf '%s\t%s\t%s\n' "$b" "$sha" "$(printf '%s' "$actual" | cut -c1-7)" >>"/tmp/.branches.$$.shabad"
    fi
  done <<EOF
$sha_claims
EOF
fi

# The BASE claim is the other half of rule 323 and cannot be phrased as prose+sha:
# the log says "one commit on `main` (`0267ade`)" for the merge target, so the check
# is that the merge target's PARENT is origin/main. That is what pass 323 got wrong
# (parented on the accumulation line, dragging 397 log commits), it is the whole
# content of that pass's finding, and it is decidable by exactly one command.
#
# SCOPE: `review/` only, deliberately. The first run of this check applied to every
# branch the section names and reported 3 base mismatches, all three of them FALSE —
# `recovery/probe-scaffolding-2026-09-28` (5 commits, parented on its own recovery
# line) and `recovery/unreachable-merge-content-2026-09-28` (37 commits) are
# recovery archives that are SUPPOSED to be off main, and `review/drop-dead-trace-env`
# is the pass-323 defect the standing section already names and explicitly forbids
# merging. Rule 323 is about a branch a human is told to MERGE; a branch the human is
# told to DELETE has no base to be right about. A check that condemns correct text is
# worse than no check (refs.sh's v1, the pass-317 defect), so the population is the
# namespace the human list actually merges from.
base_bad=0
for mb in $(printf '%s\n' "$sha_claims" | grep '^review/' | cut -f1 | sort -u); do
  mbparent="$(git rev-parse --verify --quiet "$mb^1^{commit}" 2>/dev/null || true)"
  mbmain="$(git rev-parse --verify --quiet 'origin/main^{commit}' 2>/dev/null || true)"
  [ -n "$mbparent" ] && [ -n "$mbmain" ] || continue
  # Only adjudicate a branch that is a SINGLE commit ahead of main. A branch several
  # commits deep is not a prepared one-commit review branch, and naming it would
  # report the standing section's own decision (merge only the composed one; the
  # 397-commit one is named and forbidden) as a defect of the branch. The count is
  # measured from origin/main, NOT as `$mbparent..$mb` — that range is the branch's
  # own last commit and is therefore always exactly 1, which is how this check's
  # first scoping attempt flagged review/drop-dead-trace-env, a branch the prose
  # already forbids merging, on the strength of a filter that could not fail.
  if [ "$mbparent" != "$mbmain" ]; then
    ncommits="$(git rev-list --count "$mbmain..$mb" 2>/dev/null || echo '?')"
    if [ "$ncommits" = "1" ]; then
      printf 'branches: BASE MISMATCH   %s  parent is %s, origin/main is %s (rule 323: a review branch is its tip AND its base)\n' \
        "$mb" "$(printf '%s' "$mbparent" | cut -c1-7)" "$(printf '%s' "$mbmain" | cut -c1-7)" >&2
      base_bad=$((base_bad + 1))
    fi
  fi
done

# ---------------------------------------------------------------------------
# PAYLOAD (rule 333, pass 334). The fourth checkable claim.
#
# Gate 7 adjudicates a branch's NAME, TIP and BASE. The standing merge sentence
# makes a fourth claim — the payload: "one commit on main, one file, +2/-4: it
# deletes the dead MADGAB_TRACE_* env block and adds the no_phrase_hard_coding
# fence as a CI step". That is the part a human actually merges, and until this
# block it was prose sitting between two machine checks, so a reader could not
# tell which parts of that sentence were gated.
#
# POPULATION, stated before the code (rule 332): the `N file(s) changed, +A/-B`
# claims inside the `## Current gate status` block, compared against
# `git diff --shortstat origin/main <branch>` for each `review/` branch gate 7
# has already established is parented on `origin/main`. The numbers are READ
# FROM THE SECTION, never hard-coded (rule 14k): correct +2/-4 in the prose to
# +2/-5 and this goes red, which is the entire point.
#
# FAIL-CLOSED ON AMBIGUITY: the block publishes exactly one payload claim today.
# A second claim makes the population ambiguous, and this reports the ambiguity
# and goes red rather than picking one. A check that guesses is the pass-326
# class — `branch-containment.sh` exists to answer a question with arguments and
# an argument-less reader who cannot see which argument was supplied.
#
# The sign is matched as a CHARACTER CLASS covering both the ASCII hyphen-minus
# and U+2212 MINUS SIGN. The section publishes the Unicode one (`+2/−4`, verified
# present at the byte level), so an ASCII-only pattern reads 0 claims and would
# report "no payload claim published" — a vacuous pass, the refs.sh v1 defect.
# ---------------------------------------------------------------------------
payload_bad=0
nclaim=0
# Declared BEFORE the payload loop, because the loop is what fills it. Declaring
# it after the loop — which is where the content block that reads it sits — wipes
# the value and the content check silently adjudicates nothing, reporting its own
# zero-population guard and going red on a section that is exactly right. Found by
# running, not by reading, and the third defect in this one block.
payload_branch=""

# Both branches this publishes for are single-commit review branches on
# origin/main; the loop reuses the base check's scoping so the pass-323
# mis-based branch (397 commits) is never payload-adjudicated, for the reason
# given above it.
for mb in $(printf '%s\n' "$sha_claims" | grep '^review/' | cut -f1 | sort -u); do
  mbparent="$(git rev-parse --verify --quiet "$mb^1^{commit}" 2>/dev/null || true)"
  mbmain="$(git rev-parse --verify --quiet 'origin/main^{commit}' 2>/dev/null || true)"
  [ -n "$mbparent" ] && [ -n "$mbmain" ] || continue
  [ "$mbparent" = "$mbmain" ] || continue

  # The claim is located by the branch it is ABOUT, and scoped to the few lines
  # after the branch token — not to the rest of the section, and not to the
  # blank-line-delimited paragraph either.
  #
  # This block has now had two scoping defects, both in the same direction
  # (too much text considered), and both found before the check was trusted:
  #
  #  - v1 stopped at the next `## ` heading, but the whole human list lives
  #    under ONE heading, so "after the branch token" meant "the next 172
  #    lines". A branch named inside the `branch-containment.sh` command block
  #    swept up a `1 file, +2/-2` from an unrelated at-risk table row.
  #  - v2 switched to blank-line-delimited paragraphs, which is the right idea
  #    and still too coarse in this document: the standing facts are a Markdown
  #    TABLE, and table rows are not separated by blank lines, so a dozen rows
  #    are one paragraph and a branch named in any of them captures a claim from
  #    any of the others.
  #
  # The scope that is actually correct is the one the prose uses: the claim sits
  # on the SAME LINE as the branch token or within the next few lines of the same
  # sentence, and a blank line ends it. That is a line window, which is what this
  # implements — WINDOW lines, and never across a blank line.
  #
  # SPELLING: the section publishes "one file, +2/-4" — a WORD for the file count
  # and a bare signed pair, NOT the `git diff --shortstat` wording this check was
  # first written against. A pattern built from the tool's phrasing reads 0
  # claims on a section that plainly publishes one, and the first run of this
  # block did exactly that: 0 claims, 0 defects, exit 0 — a vacuous pass, which
  # is the refs.sh v1 defect and the reason the zero-claim guard below exists.
  # Accept the published spelling, in either sign's Unicode, and nothing else.
  claim="$(awk -v br="$mb" '
    BEGIN { WINDOW = 4; left = 0 }
    /^## / { inh = ($0 ~ /^## Current gate status/); left = 0; next }
    !inh   { next }
    {
      line = $0
      gsub(/\xe2\x88\x92/, "-", line)        # U+2212 -> ASCII for the number
      hits = (left > 0)
      if (!hits && index(line, "`" br "`")) { left = WINDOW; hits = 1 }
      if (hits && out == "") {
        if (match(line, /([0-9]+|one|two|three|four|five|six|seven|eight|nine|ten) files?( changed)?, \+[0-9]+\/-[0-9]+/)) {
          out = substr(line, RSTART, RLENGTH); left = 0
        } else if (line ~ /^[[:space:]]*$/) { left = 0 }
        else if (left > 0) { left-- }
      }
    }
    END { if (out != "") print out }
  ' "$HUMANLIST")"

  [ -n "$claim" ] || continue
  nclaim=$((nclaim + 1))
  # The branch the payload claim was read FOR. The content check below reuses
  # this rather than re-deriving "a review branch on origin/main", because two
  # branches qualify as parented on origin/main — the composed merge branch and
  # `review/run-clue-fence-in-ci`, a constituent the human list tells a human to
  # DELETE — and the human list asserts the merge sentence's effect for the
  # first and says nothing about the second. Scoping to "parented on
  # origin/main" adjudicated the effect claim against a branch it was never made
  # about, which is a false defect; scoping to the branch the claim is ABOUT is
  # the same fix the payload check needed for its scoping, one level on.
  payload_branch="$mb"

  # The file count may be spelled as a word; map the published ones. An
  # unrecognised token is NOT defaulted to 1 — it aborts the claim, which the
  # zero-claim guard then reports, rather than inventing agreement.
  case "$claim" in
    one*)   cfiles=1 ;;
    two*)   cfiles=2 ;;
    three*) cfiles=3 ;;
    four*)  cfiles=4 ;;
    five*)  cfiles=5 ;;
    six*)   cfiles=6 ;;
    seven*) cfiles=7 ;;
    eight*) cfiles=8 ;;
    nine*)  cfiles=9 ;;
    ten*)   cfiles=10 ;;
    *)      cfiles="$(printf '%s' "$claim" | sed 's/ files\?.*//')" ;;
  esac
  cins="$(printf '%s' "$claim" | sed 's/.*, +//; s/\/-.*//')"
  cdel="$(printf '%s' "$claim" | sed 's/.*\/-//')"

  # Sanity: the parsed numbers must be non-empty integers. If a future edit to
  # the prose changes the shape, the comparison below would read "" != "1" and
  # report a mismatch — technically red but for the wrong reason, and a reader
  # would go looking for a repository change that did not happen. Say which.
  for tok in "$cfiles" "$cins" "$cdel"; do
    case "$tok" in
      ''|*[!0-9]*)
        printf 'branches: UNPARSED CLAIM %s  could not read the payload numbers out of "%s" (rule 333: a claim this check cannot parse is not a claim it checked)\n' \
          "$mb" "$claim" >&2
        payload_bad=$((payload_bad + 1))
        claim=""
        ;;
    esac
  done
  [ -n "$claim" ] || continue

  # `git diff --shortstat` LEADS WITH A SPACE (" 1 file changed, ..."). Trim it
  # ONCE, up front, rather than inside each field's own sed: trimming per-field
  # is what let the `^` anchor below silently miss, because the raw string is
  # " 1 file changed, ..." and so `s/^[0-9]* files\? changed, //` did not match,
  # returning the whole string as the "insertion count".
  shortstat="$(git diff --shortstat "$mbmain" "$mb" 2>/dev/null)"
  shortstat="${shortstat#"${shortstat%%[![:space:]]*}"}"
  # A check that reports a difference a reader cannot SEE is worse than one that
  # reports none, because it sends them hunting a repository change that does
  # not exist — the first version printed "section says 1 file(s)/+2/-4, live
  # shortstat says  1 file(s)/+2/-4", identical to the eye, differing only in an
  # invisible space (rule 332).
  lfiles="$(printf '%s' "$shortstat" | sed 's/ files\? changed.*//')"
  # `s/.*, //` is GREEDY and takes the LAST comma, so anchoring to the start of
  # the string is what makes these two fields distinct: insertion is the field
  # after the FILE COUNT, deletion is the field after the LAST comma.
  lins="$(printf '%s' "$shortstat" | sed 's/^[0-9]* files\? changed, //; s/ insertions\?.*//')"
  ldel="$(printf '%s' "$shortstat" | sed 's/.*, //; s/ deletions\?.*//')"

  if [ "$cfiles" != "$lfiles" ] || [ "$cins" != "$lins" ] || [ "$cdel" != "$ldel" ]; then
    printf 'branches: PAYLOAD MISMATCH %s  section says %s file(s)/+%s/-%s, live shortstat says %s file(s)/+%s/-%s (rule 333: the payload is what a human merges)\n' \
      "$mb" "$cfiles" "$cins" "$cdel" "$lfiles" "$lins" "$ldel" >&2
    payload_bad=$((payload_bad + 1))
  else
    printf 'branches: payload OK       %s  %s matches the live shortstat\n' "$mb" "$claim"
  fi
done

# Ambiguity guard. This is the fail-closed half of the population, and it is
# checked even when every individual claim matched: a check that silently
# adjudicates the first of two contradicting claims is a check that published a
# verdict on a population it had not identified.
if [ "$nclaim" -gt 1 ]; then
  printf 'branches: AMBIGUOUS PAYLOAD  %s payload claim(s) in the human list; this check adjudicates the single composed merge branch and REFUSES to pick one (rule 326 class)\n' \
    "$nclaim" >&2
  payload_bad=$((payload_bad + 1))
fi

# ZERO-CLAIM GUARD, and this is the check that matters most, because 0 is also
# what a broken matcher returns. The first run of this block matched the
# `git diff --shortstat` wording rather than the section's own and read 0
# claims — then printed "0 payload defects" and exited 0, which is the
# refs.sh v1 shape: an instrument that cannot fail is not evidence. So the
# guard is stated as a positive requirement, not as an inference: a branch the
# human list tells a human to MERGE must produce a payload claim. The merge
# branch is identified as the single-commit review branch on origin/main, which
# is the same set the base check established, so this cannot fire on a branch
# the human is told to delete.
merge_expected=0
for mb in $(printf '%s\n' "$sha_claims" | grep '^review/' | cut -f1 | sort -u); do
  mbparent="$(git rev-parse --verify --quiet "$mb^1^{commit}" 2>/dev/null || true)"
  mbmain="$(git rev-parse --verify --quiet 'origin/main^{commit}' 2>/dev/null || true)"
  [ -n "$mbparent" ] && [ -n "$mbmain" ] || continue
  [ "$mbparent" = "$mbmain" ] || continue
  if [ "$(git rev-list --count "$mbmain..$mb" 2>/dev/null)" = "1" ]; then
    merge_expected=$((merge_expected + 1))
  fi
done
if [ "$merge_expected" -gt 0 ] && [ "$nclaim" -eq 0 ]; then
  printf 'branches: NO PAYLOAD CLAIM  the human list names %s single-commit review branch(es) on origin/main to merge, and no payload claim was read for any of them; a matcher that finds nothing is indistinguishable from a matcher that is broken, so this is a defect, not a pass (rule 332/333)\n' \
    "$merge_expected" >&2
  payload_bad=$((payload_bad + 1))
fi

# The named test target must exist under tests/. The section says the branch
# "adds the no_phrase_hard_coding fence as a CI step"; a CI step naming a target
# that does not exist is a claim about a payload that cannot be true, and it is
# decidable by the same class of check. Read from the section, not hard-coded.
#
# The population is read as "a backticked name immediately followed by `passed`",
# which is the shape the section's run-evidence sentence actually uses
# ("`corpus_integration` 12 passed / 0 failed", "`no_phrase_hard_coding` 9 passed").
# The first version grepped the two literal strings instead, which is a HARD-CODED
# population wearing a read-from-the-section hat: a plant that renamed the CI step
# left the same name in the evidence sentence, the check still found it, and it
# reported OK. Deriving the population from the sentence's own grammar is what
# makes a future rename a red run rather than a silent pass.
#
# The character class is [A-Za-z0-9_], NOT [a-z0-9_]. A lowercase-only class does
# not report a renamed target — it EXCLUDES it, so a plant that renamed
# `corpus_integration` to `corpus_integ_TYPO` shrank the population to one entry
# and the check stayed green. A population filter must be wider than the names it
# is meant to catch, or a violation is indistinguishable from an absence.
#
# The count printed here is the CLAIMED pass count, so it must mean what the
# section's sentence means. Pass 335 found it did not: the original counted
# `^[[:space:]]*#\[test\]`, and tests/no_phrase_hard_coding.rs contains one such
# attribute at column 5 INSIDE a raw string of synthetic source used as test
# data, so the gate printed 10 where the section's own run evidence says 9
# passed. A raw text count cannot see a string literal — the same blindness the
# clue fence calls rule 14x. Two fixes, and the second is the load-bearing one:
#
#   1. count only column-0 attributes, which is what a Rust integration-test
#      target actually runs, and report the indented ones separately rather than
#      folding them in or dropping them silently;
#   2. RECONCILE the count against the number the section claims. The claim
#      "no_phrase_hard_coding 9 passed" is a checkable statement about the file
#      and was previously printed beside a tool number that disagreed with it,
#      with nothing saying which was right. A check that reports a number
#      nobody compared is decoration.
#
# A mismatch is a DEFECT (red), not a note. If a target ever grows a real
# nested-`mod` test, this goes red and says so in the same line, which is the
# correct direction: the standing section's run evidence would then be stale
# and a human has to re-run it.
ntarget=0
while IFS=$'\t' read -r tgt claimed claimed_ignored; do
  [ -n "$tgt" ] || continue
  ntarget=$((ntarget + 1))
  if [ ! -f "tests/$tgt.rs" ]; then
    printf 'branches: MISSING TARGET   %s  the human list names tests/%s.rs and that file does not exist\n' \
      "$tgt" "$tgt" >&2
    payload_bad=$((payload_bad + 1))
  else
    # `grep -c` PRINTS its 0 and EXITS 1 on no match, so the old `|| echo 0`
    # fallback appended a second 0 and the variable held "0\n0". Take the first
    # field instead; never let a fallback add a line to a count.
    ntests="$(grep -c '^#\[test\]' "tests/$tgt.rs" 2>/dev/null | head -1)"
    ntests="${ntests:-0}"
    nindent="$(grep -c '^[[:space:]][[:space:]]*#\[test\]' "tests/$tgt.rs" 2>/dev/null | head -1)"
    nindent="${nindent:-0}"
    printf 'branches: target OK        tests/%s.rs exists, %s #[test] (claimed %s passed' \
      "$tgt" "$ntests" "$claimed"
    if [ "$claimed_ignored" != "0" ]; then
      printf ' + %s ignored' "$claimed_ignored"
    fi
    printf ')'
    if [ "$nindent" -gt 0 ]; then
      printf '; %s indented #[test] not counted — in this repo that one is synthetic source inside a string literal' \
        "$nindent"
    fi
    printf '\n'
    # The unit is passed + ignored, because a `#[ignore]`d test is still a test
    # the runner accounted for. Comparing the count against `passed` alone
    # reddened corpus_integration by exactly its one ignored regression, which
    # the human list states correctly — and a check that reddens on a correct
    # document is a check that teaches its reader to ignore it.
    expect=$(( claimed + claimed_ignored ))
    if [ "$ntests" -ne "$expect" ]; then
      printf 'branches: COUNT MISMATCH   tests/%s.rs has %s top-level #[test] but the human list records "%s passed + %s ignored" = %s; one of the two is stale, and the run evidence has to be re-run, not reconciled on paper (pass 335)\n' \
        "$tgt" "$ntests" "$claimed" "$claimed_ignored" "$expect" >&2
      payload_bad=$((payload_bad + 1))
    fi
  fi
done <<EOF
$(cat "$HUMANLIST" | grep -oE '`[A-Za-z0-9_]+` [0-9]+ passed( / [0-9]+ failed( / [0-9]+ ignored)?)?' \
  | sed 's/`//g' \
  | awk '{
      name = $1; passed = $2; ignored = 0
      for (i = 1; i <= NF; i++) if ($i == "ignored") ignored = $(i - 1)
      printf "%s\t%s\t%s\n", name, passed, ignored
    }' | sort -u)
EOF

# Same zero-population guard as the payload claim, for the same reason: 0 targets
# read is what a broken matcher returns, and it must not read as a clean pass.
if [ "$ntarget" -eq 0 ]; then
  printf 'branches: NO TARGET CLAIM  no test target was read out of the human list run-evidence sentence, so the "adds the fence as a CI step" claim is unadjudicated; 0 is what a broken matcher returns, so this is a defect, not a pass\n' >&2
  payload_bad=$((payload_bad + 1))
fi

# ---------------------------------------------------------------------------
# PAYLOAD CONTENT (rule 336, pass 336). What the shortstat cannot see.
#
# The merge sentence has now been gated on its branch, its tip, its base, its
# FILE COUNT, its insertion and deletion counts, and the existence of the test
# targets its run evidence names. Every one of those is a COUNT or a NAME. The
# sentence's two remaining assertions are not:
#
#   "it deletes the dead MADGAB_TRACE_* env block"
#   "and adds the no_phrase_hard_coding fence as a CI step"
#
# Both are decidable, and neither was being decided. A branch whose diff is
# `1 file changed, +2/-4` can be that diff for a hundred unrelated reasons --
# reformat the file, tighten a clippy lint, bump an action version -- and would
# pass every check above. That is the same defect rule 323 found for the base
# and rule 334 found for the payload size, one level in: a claim that has never
# been measured is a claim that has never been tested, and a human merging on
# the strength of it is trusting prose.
#
# It is also the direction that matters most here. Pass 315 is the finding that
# these two halves exist: CI configured three MADGAB_TRACE_* variables that the
# accepted code no longer reads, and the one place the canonical clue is
# hard-coded outside the tests is the unfenced .github/. A merge that is +2/-4
# on one file and does neither half leaves both defects live while satisfying
# every gate in this script.
#
# POPULATION, read from the section's own grammar and never hard-coded
# (rule 14k, and the pass-325 lesson about a hard-coded population wearing a
# read-from-the-section hat): the two assertions have fixed grammatical shapes.
#
#   A  "deletes the dead `<TOKEN>` env block"   -> the token is a name, and
#      `*` in it is the prose's own glob, not a character to match literally.
#   B  "adds the `<TARGET>` fence as a CI step" -> the target is a test target,
#      and the existing NO TARGET CLAIM guard already proves the file exists, so
#      this check only has to decide whether the DIFF adds the step.
#
# Neither is allowed to fall back to a default. An unreadable assertion is a
# defect, exactly as an unreadable payload claim is (UNPARSED CLAIM above): a
# check that substitutes a plausible value for a value it could not read
# manufactures agreement.
#
# FAIL-CLOSED ON AMBIGUITY, as everywhere else in this block: two assertion A
# claims, or two assertion B claims, make the population ambiguous and are
# reported rather than resolved by picking one.
# ---------------------------------------------------------------------------
content_bad=0
nclaim_del=0
nclaim_step=0

# --- assertion A: the dead env block is actually gone ----------------------
# Scoped to the branch the payload claim was read FOR, which is the merge
# branch. The reason is above, at `payload_branch=`: a second review branch is
# also parented on origin/main and is a constituent the human list tells a human
# to delete, so "parented on origin/main" is not the merge branch and running
# the merge sentence's effect claim against it manufactures a false defect.
if [ -n "$payload_branch" ]; then
  mb="$payload_branch"
  mbmain="$(git rev-parse --verify --quiet 'origin/main^{commit}' 2>/dev/null || true)"
  [ -n "$mbmain" ] || mbmain=""

  if [ -n "$mbmain" ]; then
  del_token="$(grep -oE 'deletes the dead `[A-Za-z0-9_*]+` env block' "$HUMANLIST" 2>/dev/null \
    | head -1 | sed 's/.*`\(.*\)`.*/\1/')"
  step_token="$(grep -oE 'adds the `[A-Za-z0-9_]+` fence as a CI step' "$HUMANLIST" 2>/dev/null \
    | head -1 | sed 's/.*`\(.*\)`.*/\1/')"

  # Ambiguity first, before either is used, so a doubled claim cannot be
  # silently adjudicated by the `head -1` above.
  #
  # These count OCCURRENCES, not lines. `grep -c` counts matching LINES, so two
  # identical claims written on the SAME line — the shape this document's
  # prose actually takes, since the merge sentence wraps mid-claim — read as 1
  # and the guard stayed silent. The first version of this line used `grep -coE`
  # and its own plant (a second claim appended to the same sentence) passed green.
  # The count that matters for ambiguity is how many times the assertion is
  # MADE, so it is `grep -o | wc -l`.
  ndel_all="$(grep -oE 'deletes the dead `[A-Za-z0-9_*]+` env block' "$HUMANLIST" 2>/dev/null | wc -l | tr -d ' ')"
  ndel_all="${ndel_all:-0}"
  nstep_all="$(grep -oE 'adds the `[A-Za-z0-9_]+` fence as a CI step' "$HUMANLIST" 2>/dev/null | wc -l | tr -d ' ')"
  nstep_all="${nstep_all:-0}"
  if [ "$ndel_all" -gt 1 ] || [ "$nstep_all" -gt 1 ]; then
    printf 'branches: AMBIGUOUS CONTENT  the human list makes %s deletion claim(s) and %s CI-step claim(s); this check adjudicates one of each and REFUSES to pick (rule 326 class)\n' \
      "$ndel_all" "$nstep_all" >&2
    content_bad=$((content_bad + 1))
    # Not `continue`: this is an `if`, not a loop, so `continue` is a syntax error
    # at RUNTIME (bash parses the whole file, so `bash -n` does not catch it) and
    # it would skip the end of the block. Nesting the remainder is the fix.
  else

  # --- A ---
  if [ "$ndel_all" -eq 0 ]; then
    printf 'branches: NO DELETION CLAIM  the human list states no "deletes the dead `<X>` env block" assertion, so the deletion half of the merge sentence is unadjudicated; 0 is also what a broken matcher returns, so this is a defect, not a pass\n' >&2
    content_bad=$((content_bad + 1))
  else
    nclaim_del=$((nclaim_del + 1))
    # The prose's `*` is a glob covering the three variables, so the decidable
    # form is the shared PREFIX, read off the token rather than assumed. If a
    # future edit publishes a token with no `*`, the prefix is the whole token
    # and the test is exact -- which is the right degradation.
    del_prefix="${del_token%\*}"
    [ -n "$del_prefix" ] || { del_prefix="$del_token"; }
    # Three conditions, and all three are needed. (1) the diff removes at least
    # one such line, so the claim has not been satisfied by doing nothing;
    # (2) the diff ADDS none, so it was not renamed into a live form; (3) none
    # survive in the branch's version of any changed file, so the env block is
    # gone rather than merely edited once. Condition (3) is the one a deletion
    # claim is actually about, and it is the only one of the three that a
    # reformatting diff would fail.
    del_files="$(git diff --name-only "$mbmain" "$mb" -- 2>/dev/null)"
    # grep -c exits 1 on no match, and the `|| echo 0` fallback appends a
    # SECOND line to a count (the pass-335 defect, in a third instrument). Never
    # let a fallback add a line to a count: filter the stream first and let the
    # filtered pipeline's own wc do the counting. An earlier draft of this block
    # counted per changed file with `grep -c ... || echo 0` inside a loop and
    # hit exactly that, producing a literal two-line value that aborted the
    # arithmetic with a syntax error -- found by RUNNING the block, not by
    # reading it, which is the only reason it was found at all.
    nrem="$(git diff "$mbmain" "$mb" 2>/dev/null | grep '^-' | grep -v '^---' | grep -c "$del_prefix" | head -1)"
    nadd="$(git diff "$mbmain" "$mb" 2>/dev/null | grep '^+' | grep -v '^+++' | grep -c "$del_prefix" | head -1)"
    nrem="${nrem:-0}"; nadd="${nadd:-0}"
    # Survivors: the token anywhere in each changed file AT THE BRANCH TIP.
    surv=0
    if [ -n "$del_files" ]; then
      for f in $del_files; do
        s="$(git show "$mb:$f" 2>/dev/null | grep -c "$del_prefix" | head -1)"
        surv=$((surv + ${s:-0}))
      done
    fi
    if [ "$nrem" -eq 0 ]; then
      printf 'branches: CONTENT MISMATCH  %s  the human list says the merge "deletes the dead %s env block", but the diff removes no line containing %s\n' \
        "$mb" "$del_token" "$del_prefix" >&2
      content_bad=$((content_bad + 1))
    elif [ "$nadd" -gt 0 ]; then
      printf 'branches: CONTENT MISMATCH  %s  the human list says the merge "deletes the dead %s env block", but the diff also ADDS %s line(s) containing %s -- it renamed the block rather than removing it\n' \
        "$mb" "$del_token" "$nadd" "$del_prefix" >&2
      content_bad=$((content_bad + 1))
    elif [ "$surv" -gt 0 ]; then
      printf 'branches: CONTENT MISMATCH  %s  the human list says the merge "deletes the dead %s env block", but %s line(s) containing %s survive in the branch at %s\n' \
        "$mb" "$del_token" "$surv" "$del_prefix" "$(printf '%s' "$mb" | cut -c1-7)" >&2
      content_bad=$((content_bad + 1))
    else
      printf 'branches: content OK       %s  "deletes the dead %s env block": %s line(s) removed, 0 added, 0 survive at the branch tip\n' \
        "$mb" "$del_token" "$nrem"
    fi
  fi

  # --- B ---
  if [ "$nstep_all" -eq 0 ]; then
    printf 'branches: NO CI-STEP CLAIM  the human list states no "adds the `<X>` fence as a CI step" assertion, so that half of the merge sentence is unadjudicated; 0 is also what a broken matcher returns, so this is a defect, not a pass\n' >&2
    content_bad=$((content_bad + 1))
  else
    nclaim_step=$((nclaim_step + 1))
    # The step must be an ADDED line in a file under .github/workflows/ naming
    # the target. A CI step that was already there is not what this branch
    # "adds", and a step elsewhere in the repo is not a CI step -- scoping to
    # the workflow directory is what makes the assertion mean what it says.
    wf="$(git diff "$mbmain" "$mb" -- .github/workflows/ 2>/dev/null | grep '^+' | grep -v '^+++' | grep -c -- "--test $step_token" | head -1)"
    wf="${wf:-0}"
    if [ "$wf" -eq 0 ]; then
      printf 'branches: CONTENT MISMATCH  %s  the human list says the merge "adds the %s fence as a CI step", but the diff adds no line under .github/workflows/ naming --test %s\n' \
        "$mb" "$step_token" "$step_token" >&2
      content_bad=$((content_bad + 1))
    else
      printf 'branches: content OK       %s  "adds the %s fence as a CI step": %s added workflow line(s) naming --test %s\n' \
        "$mb" "$step_token" "$wf" "$step_token"
    fi
  fi
  fi
fi
fi

# The same zero-population guard, one level out: a section that names a merge
# branch but carries neither assertion is the vacuous-pass shape twice over.
if [ "$merge_expected" -gt 0 ] && [ "$nclaim_del" -eq 0 ] && [ "$nclaim_step" -eq 0 ]; then
  printf 'branches: NO CONTENT CLAIM  the human list names %s merge branch(es) but asserts nothing about what their diff DOES; the shortstat says how many lines changed and not one thing about them\n' \
    "$merge_expected" >&2
  content_bad=$((content_bad + 1))
fi

# ---------------------------------------------------------------------------
# Pass 340: the standing section's SCHEDULER INSTRUCTION is a claim about this
# instrument set, and it was the one part of the section no instrument read.
#
# Everything gated above is read out of the `## Current gate status` block: a
# branch, a tip, a base, a shortstat, a test count, a diff. The paragraph that
# tells a scheduled coordinator what its correct pass is ALSO lives in that
# block, and it names the gate set in prose: "run the six gates above". Gate 7
# was added at pass 331, so that sentence has been wrong since the very pass
# that made it wrong -- the table under it has seven rows and the instruction
# says six, for nine passes, and no gate can see it because no gate read the
# sentence. This is rule 14k's exact shape in the highest-traffic prose in the
# document: a count carried in words beside a structure that changes.
#
# So: the number of gates is TAKEN FROM THE TABLE, and the number word in the
# instruction is reconciled against it. Both halves are read, neither is
# hard-coded, and adding an eighth gate makes this go red until the prose is
# corrected -- which is the point.
# ---------------------------------------------------------------------------
selfcheck_bad=0

# The table: rows whose first cell is the gate's own ordinal.
gate_rows="$(grep -cE '^\|[[:space:]]*[0-9]+[[:space:]]*\|' "$prose" || true)"
case "$gate_rows" in ''|*[!0-9]*) gate_rows=0 ;; esac

# The claim: the only place the section says "N gates above" is the scheduler
# instruction, and the "above" is what makes it a claim about the table.
#
# ABSENCE OF A NUMERAL IS THE GREEN CASE, and that is the durable form rather
# than a concession. The standing sentence was repaired at pass 340 to stop
# carrying a count at all -- it tells the reader to take the number from the
# table's rows. A check that demanded a numeral there would have re-created
# the defect it was written to remove, by making the number load-bearing
# again. So a numeral is RECONCILED when present and its absence is reported as
# the intended shape.
#
# The two spellings are ONE alternation, not two piped greps, and that is a
# defect this check had on its first run. Split into
# `grep -oiE '[a-z]+|[0-9]+ (bare )?gates above'`, the first alternative wins
# at the position of "six", consumes it, and the gates-above alternative is
# never tried -- so the claim read EMPTY, the spelling table fell through to
# `gate_words=-1`, and the check printed UNREADABLE while standing next to a
# sentence it could plainly see. Found by running it, not by reading it, which
# is rule 334's standing lesson reached inside a single extraction.
gate_claim="$(grep -oiE '(six|seven|eight|nine|ten|[0-9]+) (bare )?gates above' "$prose" \
  | head -1 | sed -E 's/ (bare )?gates above$//' || true)"
[ -n "$gate_claim" ] || gate_claim=""

gate_words=0
if [ -z "$gate_claim" ]; then
  gate_words=0   # no numeral carried: the intended shape, reconciled as green below
else
case "$gate_claim" in
  one|1)    gate_words=1 ;;
  two|2)    gate_words=2 ;;
  three|3)  gate_words=3 ;;
  four|4)   gate_words=4 ;;
  five|5)   gate_words=5 ;;
  six|6)    gate_words=6 ;;
  seven|7)  gate_words=7 ;;
  eight|8)  gate_words=8 ;;
  nine|9)   gate_words=9 ;;
  ten|10)   gate_words=10 ;;
  *)        gate_words=-1 ;;
esac
fi

if [ "$gate_rows" -lt 1 ]; then
  printf 'branches: NO GATE TABLE  the reader-facing section carries no numbered gate rows, so the instruction that names them cannot be checked against anything\n' >&2
  selfcheck_bad=$((selfcheck_bad + 1))
elif [ "$gate_words" -lt 0 ]; then
  printf 'branches: UNREADABLE GATE COUNT  the instruction names "%s gates above"; the spelling is not one this check knows\n' "$gate_claim" >&2
  selfcheck_bad=$((selfcheck_bad + 1))
elif [ "$gate_words" -gt 0 ] && [ "$gate_words" -ne "$gate_rows" ]; then
  printf 'branches: GATE COUNT STALE  the scheduler instruction says "%s gates above" and the table has %s row(s)\n' \
    "$gate_claim" "$gate_rows" >&2
  printf 'branches: the instruction is the sentence every scheduled pass acts on, and a gate that disagrees with the table it points at is the oldest defect class in this log (rule 14k)\n' >&2
  selfcheck_bad=$((selfcheck_bad + 1))
fi

# The second half of the same sentence, and the same defect: it told the next
# pass to find the closed classes in the newest NEXT block "item 4". That NEXT
# ends at (3). An ORDINAL into a block whose length is not a property of this
# file decays by one every time a pass appends -- rule 330's shape, in an
# index instead of a line number. A standing pointer into the newest entry must
# name a HEADING or a searchable string, never a position.
#
# The scan runs over the section with CODE SPANS STRIPPED, and that is not a
# softening. A backticked `item 4` in this section is a worked example in prose
# explaining why the ordinal is wrong; the live defect was the unbackticked
# `"Next action for the next pass", item 4`, which instructs a reader to go
# look. An instrument that cannot tell a token from a mention of that token
# condemns its own documentation -- which is exactly what happened on this
# check's first run against the sentence written to fix it, and it is rule
# 14x (a `#[test]` counted inside a string literal) reached inside a prose
# scan. The plants below cover the unbackticked form in both directions.
prose_nocode="$(sed -E 's/`[^`]*`//g' "$prose")"
if grep -qE 'item [0-9]+' <<<"$prose_nocode"; then
  printf 'branches: DECAYING ORDINAL POINTER  the standing section points into the newest NEXT block by position ("%s")\n' \
    "$(grep -oE 'item [0-9]+' <<<"$prose_nocode" | head -1)" >&2
  printf 'branches: the newest NEXT block is append-shaped and its length is not a property of this file; point at the block, not at a position in it (rule 330)\n' >&2
  selfcheck_bad=$((selfcheck_bad + 1))
fi

if [ "$selfcheck_bad" -eq 0 ]; then
  if [ -n "$gate_claim" ]; then
    printf 'branches: selfcheck OK       scheduler instruction reconciles: "%s gates above" vs %s row(s) in the table; no ordinal pointer into the newest NEXT\n' \
      "$gate_claim" "$gate_rows"
  else
    printf 'branches: selfcheck OK       scheduler instruction carries NO gate count (the intended shape; take it from the %s table rows); no ordinal pointer into the newest NEXT\n' \
      "$gate_rows"
  fi
fi

remote_heads=""
if git remote get-url origin >/dev/null 2>&1; then
  # Full-form ls-remote per rule 14p, one invocation. A bare-prefix form against
  # `--heads` is the rule-14j shape; both are fine, this is the one that cannot
  # be satisfied by a stale local mirror.
  remote_heads="$(git ls-remote --heads origin 2>/dev/null)"
fi
[ -n "$remote_heads" ] || remote_heads=""

while IFS= read -r b; do
  [ -n "$b" ] || continue
  nbranch=$((nbranch + 1))

  loc=""
  git show-ref --verify --quiet "refs/heads/$b"      && loc="local"
  git show-ref --verify --quiet "refs/remotes/audit/$b" && loc="${loc:+$loc,}audit"
  isremote=0
  if [ -n "$remote_heads" ] && printf '%s\n' "$remote_heads" | grep -q "refs/heads/$b\$"; then
    loc="${loc:+$loc,}remote"
    isremote=1
  fi

  if [ -z "$loc" ]; then
    printf 'branches: UNRESOLVED BRANCH  %s  (not local, not mirrored, not a remote head)\n' "$b" >&2
    missing=$((missing + 1))
    printf '%s\n' "$b" >>"/tmp/.branches.$$.missing"
  elif [ "$isremote" -eq 1 ] && ! git show-ref --verify --quiet "refs/heads/$b"; then
    # REMOTE-ONLY: a live remote head with NO local branch. This is the class the
    # pass-330 defect lives in -- a human told this branch is a local artefact
    # runs `git branch -D`, which cannot fire, and the branch survives.
    #
    # The condition is on the ABSENCE of a local branch, not on $loc equalling the
    # single string "remote". An earlier version of this line tested
    # `[ "$loc" = "remote" ]`, and it was wrong in the direction that matters most:
    # the branch also has a local `audit/` mirror, so $loc read "audit,remote" and
    # the branch was printed in the ORDINARY list with no flag at all. The
    # instrument did not flag the exact case it was written for while reporting
    # itself clean -- rule 298's shape reached from a new direction, and found by
    # reading the script's own output against its own header rather than by a
    # plant. Plant 7 below is what keeps it fixed.
    printf '  %-40s REMOTE-ONLY     -> %s   (no local branch; `git branch -D` cannot remove it)\n' \
      "$b" "$(printf '%s\n' "$remote_heads" | awk -v n="refs/heads/$b" '$2==n{print substr($1,1,7)}')"
    remoteonly=$((remoteonly + 1))
    printf '%s\n' "$b" >>"/tmp/.branches.$$.remoteonly"
  else
    printf '  %-40s %s\n' "$b" "$loc"
  fi
done <<EOF
$names
EOF

# The verdict is derived from the defect counts, never from a variable each block
# has to remember to touch. refs.sh's first working version printed and exited 0;
# the plant below is the only reason that is known.
status=0
[ "$missing" -gt 0 ] && status=1
[ "$shabad" -gt 0 ] && status=1
[ "$base_bad" -gt 0 ] && status=1
[ "$payload_bad" -gt 0 ] && status=1
[ "$content_bad" -gt 0 ] && status=1
[ "$selfcheck_bad" -gt 0 ] && status=1

echo "branches: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
if [ "$remoteonly" -gt 0 ]; then
  echo "branches: ${nbranch} branch name(s); ${remoteonly} resolve REMOTE-ONLY (a local delete will not touch them)"
  echo "branches:   -> $(tr '\n' ' ' <"/tmp/.branches.$$.remoteonly")"
fi
echo "branches: ${nshaclaim} branch+sha claim(s) checked against the live commit; ${shabad} tip mismatch(es), ${base_bad} base mismatch(es)"
echo "branches: ${nclaim} payload claim(s) checked against the live shortstat; ${payload_bad} payload defect(s)"
echo "branches: ${nclaim_del} deletion claim(s) and ${nclaim_step} CI-step claim(s) checked against the live diff; ${content_bad} content defect(s)"

if [ "$status" -eq 0 ]; then
  echo "branches: ${nbranch} branch name(s) resolve; 0 unresolved"
  echo "branches: every branch the section names exists, at the commit it names, on the base it names, with the payload it claims and the effect it claims"
  exit 0
fi

echo "branches: ${nbranch} branch name(s), ${missing} unresolved, ${selfcheck_bad} scheduler-instruction defect(s)" >&2
if [ "$missing" -gt 0 ]; then
  echo "branches: REFUSING — a named branch that exists nowhere sends a human to delete nothing" >&2
fi
if [ "$selfcheck_bad" -gt 0 ]; then
  echo "branches: REFUSING — the section's own instruction to a scheduled pass disagrees with the section; a reader acting on it is acting on a stale count" >&2
fi
exit 1
