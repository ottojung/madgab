#!/usr/bin/env bash
# at-risk-content.sh -- is the CONTENT of the at-risk commits durable on origin?
#
# WHY THIS FILE EXISTS (pass 313, log rule 14af).
#
#     at-risk.sh answers "which commits are held by no ref?". It does not answer
#     "is any of their content lost?", and it explicitly declines to:
#     "a residual is a HUMAN judgement, not an automatic action."
#
# For 100+ passes that judgement was made by ASSERTION, not by measurement. Pass
# 273 measured the at-risk 88 once and found 7 of them were this log's own
# superseded pass drafts, but the question was never put to the CONTENT: are the
# blobs these commits introduce present on the origin side at all?
#
# The answer matters because the two classes are genuinely different. A commit
# held by no ref is NOT lost work if every non-build blob it introduces already
# exists somewhere under an origin-mirrored ref. It is unbacked HISTORY, not
# lost CONTENT -- which is what the log has been saying in prose since rule 42
# without an instrument that could tell the difference.
#
# The method, and why each step is here:
#
#   1. at-risk population = (--all --reflog) minus (--all), i.e. reflog-only.
#      at-risk.sh's own population, so the two agree by construction.
#   2. origin-side object set = `rev-list --objects` over the audit mirror,
#      which is a LOCAL MIRROR of the remote's refs/heads/* (rule: fetch it with
#      the bare-prefix form and NO --prune, or you delete it -- pass 202).
#   3. for each at-risk commit, its tree entries; the blobs whose sha is absent
#      from the origin-side set are the candidates.
#   4. THE BUILD FILTER IS COMPONENT-WISE, not a path prefix. `target-after/`,
#      `target-base/`, `prof/` and `target/` all start with "target"/"prof" but
#      only as a PATH COMPONENT. An anchored `^target/` misses target-after/
#      entirely -- which is exactly how 320 of this pass's entries would have
#      been miscounted as source. See log rule 288 and the pass-279 note in
#      content-sweep.sh.
#
# It PRINTS, it does not gate. A non-zero result is a finding for a human, not a
# build failure: the correct response to "N non-build blobs are absent from
# origin" is to archive them, which is a judgement, not a script.
#
# USAGE
#   docs/work/paused-recon/at-risk-content.sh            # the verdict
#   docs/work/paused-recon/at-risk-content.sh --verbose  # list the absent paths

set -uo pipefail

REPO="$(git rev-parse --show-toplevel)"
cd "$REPO" || exit 1

VERBOSE=0
[ "${1:-}" = "--verbose" ] && VERBOSE=1

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

fail() { echo "at-risk-content: $*" >&2; exit 1; }

# --- controls run FIRST, and a broken control is a refusal, not a warning ----
# Without these the verdict below is a number nobody can calibrate.
git rev-parse --verify HEAD >/dev/null 2>&1 || fail "no HEAD; not a repository"
[ -s docs/work/paused-recon/at-risk.sh ] || fail "sibling at-risk.sh missing; population would be undefined"

echo "at-risk-content: repo $REPO"

# --- step 1: the at-risk population, the same way at-risk.sh defines it -------
git rev-list --all --reflog | sort -u > "$TMP/base"
git rev-list --all | sort -u > "$TMP/refs"
comm -23 "$TMP/base" "$TMP/refs" > "$TMP/atrisk"
ATRISK=$(wc -l < "$TMP/atrisk")

# --- step 2: the origin-side object set, from the audit mirror --------------
# The mirror MUST be a mirror of the remote's real heads. If the fetch has never
# run, `refs/remotes/audit` can be empty and every blob reads absent -- a false
# alarm in the safe-looking direction. So assert the population is non-trivial
# before believing anything (log rule 14g: assert cardinality inline).
git for-each-ref --format='%(refname)' refs/remotes/audit refs/remotes/audit-tag > "$TMP/auditrefs"
AUDIT_REFS=$(wc -l < "$TMP/auditrefs")
[ "$AUDIT_REFS" -ge 100 ] || fail "audit mirror has only $AUDIT_REFS refs; fetch it first
  git fetch origin '+refs/heads/*:refs/remotes/audit/*'   (NO --prune: pass 202 deleted 203 refs with it)"
git rev-list --objects --stdin < "$TMP/auditrefs" 2>/dev/null | awk '{print $1}' | sort -u > "$TMP/originobjs"
ORIGIN_OBJS=$(wc -l < "$TMP/originobjs")
[ "$ORIGIN_OBJS" -gt 0 ] || fail "origin-side object set is empty; the mirror has no objects"

echo "at-risk-content: at-risk commits (reflog-only) = $ATRISK"
echo "at-risk-content: origin-mirror refs = $AUDIT_REFS, objects = $ORIGIN_OBJS"

# --- step 3: which blobs do the at-risk commits introduce, and which are new --
if [ "$ATRISK" -eq 0 ]; then
  echo "at-risk-content: nothing at risk; verdict is 0 absent non-build blobs"
  exit 0
fi
while read -r c; do git ls-tree -r "$c" 2>/dev/null; done < "$TMP/atrisk" \
  | awk '{print $3, $4}' | sort -u > "$TMP/entries"
ENTRIES=$(wc -l < "$TMP/entries")
awk '{print $1}' "$TMP/entries" | sort -u > "$TMP/blobs"
BLOBS=$(wc -l < "$TMP/blobs")
comm -23 "$TMP/blobs" "$TMP/originobjs" > "$TMP/absent"
ABSENT_BLOBS=$(wc -l < "$TMP/absent")
grep -F -f "$TMP/absent" "$TMP/entries" 2>/dev/null | awk '{print $2}' | sort -u > "$TMP/absentpaths" || true
[ -f "$TMP/absentpaths" ] || : > "$TMP/absentpaths"

# --- step 4: the build filter, COMPONENT-WISE -------------------------------
# A path is build output if ANY of its components begins with target/prof.
# `target-after/` and `target-base/` are the two in this repository; an anchored
# `^target/` reads both as source.
grep -vE '(^|/)(target|prof)[-a-zA-Z0-9_]*/' "$TMP/absentpaths" > "$TMP/nonbuild" || true
NONBUILD=$(wc -l < "$TMP/nonbuild")

[ "$VERBOSE" -eq 1 ] && sed 's/^/  /' "$TMP/nonbuild"

echo "at-risk-content: distinct blobs introduced by the at-risk set = $BLOBS"
echo "at-risk-content: blobs ABSENT from the origin side = $ABSENT_BLOBS"
echo "at-risk-content: distinct paths carrying an absent blob = $(wc -l < "$TMP/absentpaths")"
echo "at-risk-content: of those, NON-BUILD (component-wise filter) = $NONBUILD"

# --- controls on the verdict itself, both directions ------------------------
# A known-present blob must read present; a fabricated one must read absent.
# Both are checked against the SAME originobjs set the verdict used, so a
# control cannot pass by exercising a different population (log rule 14r).
KNOWN=$(grep -m1 'docs/work/recovery/.*\.blob$' "$TMP/entries" | awk '{print $1}')
if [ -n "${KNOWN:-}" ] && comm -12 <(echo "$KNOWN") "$TMP/originobjs" | grep -q .; then
  echo "at-risk-content: control known-present ${KNOWN:0:8} -> present (fires)"
elif [ -n "${KNOWN:-}" ]; then
  fail "control FAILED: known-present ${KNOWN:0:8} reads absent; the origin set is not what it claims"
fi
FAKE=0000000000000000000000000000000000000000
if comm -23 <(echo "$FAKE") "$TMP/originobjs" | grep -q .; then
  echo "at-risk-content: control fabricated-absent -> absent (fires)"
else
  fail "control FAILED: a fabricated sha reads PRESENT; the comparison is broken"
fi

if [ "$NONBUILD" -eq 0 ]; then
  echo "at-risk-content: VERDICT 0 non-build blobs absent from origin."
  echo "at-risk-content:   The at-risk commits are unbacked HISTORY, not lost CONTENT."
  echo "at-risk-content:   No recovery branch is warranted on content grounds."
else
  echo "at-risk-content: VERDICT $NONBUILD non-build blobs are absent from the origin side."
  echo "at-risk-content:   That IS lost content. A HUMAN decides whether to archive it"
  echo "at-risk-content:   (see docs/work/recovery/ for the precedent) -- this script only reports."
fi
exit 0
