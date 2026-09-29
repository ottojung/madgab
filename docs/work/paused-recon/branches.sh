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

nbranch=0; missing=0; remoteonly=0
: > /tmp/.branches.$$.missing
: > /tmp/.branches.$$.remoteonly
trap 'rm -f "$prose" /tmp/.branches.$$.missing /tmp/.branches.$$.remoteonly' EXIT

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

echo "branches: reader-facing section = ${nprose} line(s) of $(basename "$ITEM")"
if [ "$remoteonly" -gt 0 ]; then
  echo "branches: ${nbranch} branch name(s); ${remoteonly} resolve REMOTE-ONLY (a local delete will not touch them)"
  echo "branches:   -> $(tr '\n' ' ' <"/tmp/.branches.$$.remoteonly")"
fi

if [ "$status" -eq 0 ]; then
  echo "branches: ${nbranch} branch name(s) resolve; 0 unresolved"
  echo "branches: every branch the section names exists"
  exit 0
fi

echo "branches: ${nbranch} branch name(s), ${missing} unresolved" >&2
echo "branches: REFUSING — a named branch that exists nowhere sends a human to delete nothing" >&2
exit 1
