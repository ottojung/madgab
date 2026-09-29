#!/usr/bin/env bash
# branch-containment.sh -- decide, by EXECUTION, whether a composed review
# branch really carries every effect of the branches it was composed from.
#
# WHY THIS FILE EXISTS. The human list in w-paused-reconciliation.md is a
# human-facing change set: passes 319/321/323 prepared one edit each, and pass
# 324 composed two of them into ONE branch (`review/drop-dead-trace-and-fence`
# = 8c88a59) on the finding that a human must not be handed three branches plus
# an ordering constraint between them (rule 324: per-branch checks cannot see a
# hazard that lives BETWEEN branches).
#
# That composition is only safe if the composed branch carries the union of the
# constituent effects. Pass 324 asserted it -- "all three are strictly contained
# in the composed branch and none carries anything unique" -- but published NO
# COMMAND that decides it, and every command a reader would reach for first
# reports FALSE on this repository's live branches. Measured at pass 325, all
# three wrong in the same direction (they say "not contained" for a branch that
# is):
#
#   1. `git cherry <composed> <constituent>`   -> reports every constituent
#      commit as `+` (unmerged), because `cherry` compares PATCH IDENTITY
#      against the upstream and the composed branch has a DIFFERENT patch: it
#      deletes the env block AND adds the fence step, while each constituent
#      does one of those. Pass 325 ran it and got a 397-line wall of `+` rows
#      for a29f3d7 -- a result indistinguishable from "nothing is contained".
#   2. `diff <git show composed:file> <git show constituent:file>` -> "differs",
#      for the same reason: the composed file legitimately has the OTHER
#      constituent's edit as well. Containment is about the constituent's
#      EFFECT being present, not about the two files being byte-identical.
#   3. `git apply --check --reverse <constituent.patch>` against the composed
#      file -> "patch does not apply", and this one is subtler: the two edits
#      are ADJACENT hunks 5 lines apart, so removing one changes the context
#      lines the other's patch matches on. Pass 325 built this and it fails on
#      both constituents.
#
# So the claim that protects a human from merging the wrong branch could not be
# checked by any of the three obvious instruments, and a reader who trusted one
# would conclude the composed branch is missing work. That is rule 14q again --
# a control whose result cannot be re-derived manufactures confidence in the
# direction the conclusion already points -- and rule 273: a population
# published as a count has not been examined.
#
# WHAT IT DOES INSTEAD. It asks the question in the only form git can answer
# exactly: does a real three-way merge of the constituents, on the shared base,
# produce the composed branch's TREE? If it does, the composed branch carries
# the union of their effects and there is nothing to merge after it. A tree
# hash is a whole-content identity, not a line-wise judgement, so it cannot be
# fooled by adjacency, hunk offsets, or by a constituent that happens to be an
# ancestor.
#
# WHAT IT DOES NOT DO. It never merges, never creates a branch, never pushes.
# And it answers only "does the composed branch subsume these constituents on
# the given base" -- it does NOT certify the composed branch is correct or
# wanted, and a residual is a human judgement (same as at-risk.sh).
#
# USAGE
#   docs/work/paused-recon/branch-containment.sh <base> <composed> <constituent>...
#
#   base       the commit both sides are diffed against (normally origin/main)
#   composed   the single branch the human is invited to merge
#   constituent  one or more branches that were folded into it

set -euo pipefail

REPO=${REPO:-$(git rev-parse --show-toplevel)}
cd "$REPO"

fail() { printf 'branch-containment: %s\n' "$*" >&2; exit 1; }

[ "$#" -ge 3 ] || fail "usage: $0 <base> <composed> <constituent>..."

# Resolve a branch NAME to a ref that actually exists here. .git/config fetches
# only post-milestone-acceptance, so a review branch is normally NOT present as
# `refs/remotes/origin/<name>` and a naive `git rev-parse <name>` fails for a
# reason that has nothing to do with containment -- it means "never fetched",
# not "does not exist". The audit mirror (rule 10/14a) holds the remote's real
# heads; never pass --prune to it (rule 14m). Resolution is by identity of the
# name, not by loose prefix.
resolve() {
  local n=$1
  git rev-parse --verify --quiet "$n^{commit}" >/dev/null 2>&1 && { printf '%s' "$n"; return 0; }
  git rev-parse --verify --quiet "refs/remotes/audit/$n^{commit}" >/dev/null 2>&1 \
    && { printf 'refs/remotes/audit/%s' "$n"; return 0; }
  git rev-parse --verify --quiet "origin/$n^{commit}" >/dev/null 2>&1 \
    && { printf 'origin/%s' "$n"; return 0; }
  return 1
}

BASE=$(resolve "$1") || fail "no such commit: $1 (and it is in neither the audit mirror nor origin) -- a missing ref is an input error, not a containment result"
COMPOSED=$(resolve "$2") || fail "no such commit: $2 (and it is in neither the audit mirror nor origin) -- a missing ref is an input error, not a containment result"
shift 2
CONSTITUENTS=()
for c in "$@"; do
  CONSTITUENTS+=("$(resolve "$c")") \
    || fail "no such commit: $c (and it is in neither the audit mirror nor origin) -- a missing ref is an input error, not a containment result"
done

# git rev-parse --verify prints to stderr and exits 128 for an unknown name.
# A missing branch here is NOT a containment result: it is an input error, and
# letting it read as "0 constituents contained" would be a false zero in the
# direction that hides work (the pass-182 class this repo has hit for real).
for r in "$BASE" "$COMPOSED" "${CONSTITUENTS[@]}"; do
  git rev-parse --verify --quiet "$r^{commit}" >/dev/null \
    || fail "no such commit: $r -- refusing to report (a missing ref is an input error, not a containment result)"
done

# Rule 323, applied to the composed branch: a review branch is its tip AND its
# base. If the composed branch is parented on the accumulation line rather than
# on the release line, merging it delivers the composed edit plus every log
# commit since -- which is the exact defect that survived passes 319, 321 and
# 323 before it was found. Check it first, because it invalidates the whole
# composition if it fails.
comp_off=$(git rev-list --count "$BASE..$COMPOSED")
printf 'composition   base=%s composed=%s (%s commit(s) ahead of base)\n' \
  "$(git rev-parse --short "$BASE")" "$(git rev-parse --short "$COMPOSED")" "$comp_off"

# The base verdict is a separate, LOAD-BEARING fact and it gates the final
# verdict. Pass 325's first draft reported it as a printed annotation and then
# let a CONTAINED verdict stand on a branch that fails it -- so running the
# instrument on a29f3d7 printed "a human needs to merge the composed branch
# ONLY" for a branch 397 commits off the release line, which is the exact
# advice rule 323 exists to prevent. Containment and base are independent
# questions and a green one must never mask a red one.
base_ok=0
if [ "$(git merge-base "$COMPOSED" "$BASE")" = "$(git rev-parse "$BASE")" ]; then
  base_ok=1
  printf 'base check    OK: composed is based on the release line\n'
else
  printf 'base check    FAIL (rule 323): composed is NOT based on the release line -- merging it\n'
  printf '              would carry %s commit(s) of extra history. The containment verdict\n' "$comp_off"
  printf '              below is measured and may be correct, but DO NOT MERGE THIS BRANCH.\n'
fi

# The decision itself. A three-way merge over the constituents' OWN merge bases
# -- not over BASE -- because a constituent may predate the release line, and
# forcing a common base would either conflict spuriously or silently rebase
# away the very edit being tested. Per rule 274 an unresolvable argument must
# not read as an empty result, so the exit code is captured and checked.
# NOTE ON THE MERGE SHAPE, because pass 325 got this wrong twice here and the
# errors are instructive.
#
#   (a) `git merge-tree --write-tree` takes exactly TWO branch arguments on this
#       git. A third is a usage error and exits non-zero, which the check below
#       reports as a refused measurement -- an instrument defect wearing the
#       costume of a safety refusal.
#   (b) The obvious fix -- merge the constituents together and compare the
#       resulting TREE to the composed tree -- also fails, and the reason is
#       that merge-tree's arguments must be COMMITS, so feeding an accumulated
#       tree back in needs a throwaway commit. Avoided entirely below.
#
# The pairwise form answers the containment question directly and more sharply:
# merge EACH constituent into the composed branch, separately, and require
# every one of those merges to be a no-op on the tree. That is precisely "the
# constituent's effect is already present", it needs no intermediate commit, and
# unlike one merged total it NAMES the constituent that is not contained rather
# than only reporting that something is not.
composed_tree=$(git rev-parse "$COMPOSED^{tree}")
contained=1
for c in "${CONSTITUENTS[@]}"; do
  if ! out=$(git merge-tree --write-tree "$COMPOSED" "$c" 2>/tmp/bc.merge); then
    cat /tmp/bc.merge >&2
    fail "merge-tree failed -- refusing to report (rule 274: a fatal must not read as a zero)"
  fi
  nt=$(printf '%s\n' "$out" | head -1)
  name=$(git rev-parse --abbrev-ref "$c" 2>/dev/null || echo "$c")
  if [ "$nt" = "$composed_tree" ]; then
    printf 'constituent   %-42s -> contained (merge is a no-op)\n' "$name"
  else
    contained=0
    printf 'constituent   %-42s -> ADDS something the composed branch lacks:\n' "$name"
    git diff --stat "$composed_tree" "$nt" | sed 's/^/                /'
  fi
done

printf 'tree          %s tree %s\n' "$(git rev-parse --short "$COMPOSED")" "${composed_tree:0:7}"

if [ "$contained" = 1 ] && [ "$base_ok" = 1 ]; then
  printf 'VERDICT       CONTAINED and mergeable -- the composed branch carries every constituent\n'
  printf "              effect and sits on the release line. A human needs to merge the composed\n"
  printf '              branch ONLY; the constituents carry nothing unique and must not be merged\n'
  printf '              alongside it.\n'
elif [ "$contained" = 1 ]; then
  printf 'VERDICT       CONTAINED BUT NOT MERGEABLE -- the constituents ARE subsumed, so nothing is\n'
  printf '              lost by not merging them, but the composed branch FAILS the base check above.\n'
  printf '              Do not merge it as it stands; re-prepare it on the release line first.\n'
else
  # Not a failure of the instrument -- a real difference the human must read.
  # Name the paths rather than asserting a verdict about intent.
  printf 'VERDICT       NOT CONTAINED -- at least one constituent is not subsumed. Paths:\n'
  printf '              This may be a genuine extra change in the composed branch, or a\n'
  printf '              constituent that edits in a different order. READ THE DIFF before\n'
  printf '              concluding anything; do not merge either branch on this number alone.\n'
fi

# The constituent-base sanity check, reported rather than enforced: a
# constituent based deep in the log is exactly the pass-323 hazard, and it is
# worth seeing in the same output as the verdict instead of in a second command.
for c in "${CONSTITUENTS[@]}"; do
  mb=$(git merge-base "$c" "$BASE")
  printf 'constituent   %-40s base=%s (%s commit(s) ahead of %s)\n' \
    "$(git rev-parse --abbrev-ref "$c" 2>/dev/null || echo "$c")" \
    "${mb:0:7}" "$(git rev-list --count "$mb..$c")" "${BASE:0:7}"
done
