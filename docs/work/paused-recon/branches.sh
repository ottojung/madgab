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

echo "branches: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
if [ "$remoteonly" -gt 0 ]; then
  echo "branches: ${nbranch} branch name(s); ${remoteonly} resolve REMOTE-ONLY (a local delete will not touch them)"
  echo "branches:   -> $(tr '\n' ' ' <"/tmp/.branches.$$.remoteonly")"
fi
echo "branches: ${nshaclaim} branch+sha claim(s) checked against the live commit; ${shabad} tip mismatch(es), ${base_bad} base mismatch(es)"

if [ "$status" -eq 0 ]; then
  echo "branches: ${nbranch} branch name(s) resolve; 0 unresolved"
  echo "branches: every branch the section names exists, at the commit it names, on the base it names"
  exit 0
fi

echo "branches: ${nbranch} branch name(s), ${missing} unresolved" >&2
echo "branches: REFUSING — a named branch that exists nowhere sends a human to delete nothing" >&2
exit 1
