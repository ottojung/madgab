#!/usr/bin/env bash
# content-sweep.sh -- the non-build live-content sweep, as a RUNNING instrument.
#
# WHY THIS FILE EXISTS. The sibling at-risk.sh was written at pass 278 for
# exactly this reason: a standing FIGURE is not a standing PROCEDURE (log rule
# 14k), and this log's own history shows the procedure for this sweep being
# re-derived -- and mis-derived -- in nearly every pass. The published
# "0 unreachable" row had been reached by hand for 100+ passes and had been
# corrected under rules 9 (build filter), 14k (column handling), 17 (object-id
# extraction), 22 (comm is line-based), 30 (stale population), 138 (separator)
# and 265 (unanchored regex). Pass 279 found the defect that the script exists
# to make impossible to repeat by hand.
#
# THE DEFECT THIS SCRIPT IS BUILT AROUND (pass 279, log rule 288).
#
#     git status --porcelain collapses an untracked DIRECTORY to one `?? dir/`
#     row. `git status --porcelain -uall` expands it to one row per file.
#
# The published sweep consumed the collapsed row, found that
# `git hash-object -- prof/` could not hash it, classified it as a benign
# "directory row", and published "0 unreachable" -- while the 51 files actually
# behind the two collapsed rows were never enumerated at all. Same row count,
# clean exit, reassuring verdict, wrong population. It is rule 182's false zero
# (an unresolvable ref reading 0 with exit 128) and passes 283-287's
# fail-open-validator family, in the one sweep that had not been scripted.
#
# The answer happened to be benign: all 51 files were already durable under
# tracked paths. That is the point -- the verdict was right BY LUCK, and the
# instrument could not have told the difference.
#
# SO THE INSTRUMENT REFUSES INSTEAD OF CLASSIFYING. A `?? path/` row that
# survives `-uall` is not a directory to be skipped; it is EVIDENCE THAT THE
# POPULATION COULD NOT BE ENUMERATED, and a sweep that cannot enumerate its own
# population has no verdict to publish. It exits non-zero. That is rule 22/28/34
# applied to a sweep rather than to a fence: a broken read that reports a clean
# zero is worse than a read that refuses.
#
# USAGE
#   docs/work/paused-recon/content-sweep.sh             # report
#   docs/work/paused-recon/content-sweep.sh --verbose   # also list each row
#
# WHAT IT DOES NOT DO. It creates no branch, pushes nothing, and modifies no
# worktree. A row it reports as UNREACHABLE is a finding for a HUMAN to act on
# (mirror the blob onto a recovery ref), exactly as at-risk.sh's at-risk commits
# are. The script's job is to make the number trustworthy, not to act on it.
#
# WHAT IT PRESERVES FROM THE HAND PROCEDURE (do not "simplify" these away; each
# is a measured correction recorded in this log):
#
#   rule 9/265  build filter, ANCHORED, on the PATH FIELD ONLY
#   rule 17     object ids via `awk '{print $1}'`, never `cut -d' ' -f1`
#   rule 22     `sort -u` BEFORE any `comm`; comm pairs LINES, not keys
#   rule 138    rows carried as `worktree<TAB>path`
#   pass 279    `hash-object` with NO `-w` -- see the note below
#
# THE `-w` NOTE, because it is the most dangerous line that is NOT here.
# `git hash-object <file>` WRITES the object into this repository's store. A
# reachability test written as
#
#     h=$(git hash-object "$f") ; git cat-file -e "$h"
#
# therefore cannot fail: it creates the object and then finds it. It reports
# REACHABLE for every file, including orphans, and it does so while the
# comparison list is being built. Pass 279 hit this and recorded it; pass 183 hit
# the close cousin (the sentinel was `-w` written AFTER the id list was
# captured, so a "positive" came from list staleness). `--no-filters` is used so
# the hash matches what the object store would hold, and `-w` never is.

set -euo pipefail

REPO=${REPO:-$(git rev-parse --show-toplevel)}
cd "$REPO"

VERBOSE=0
[ "${1:-}" = "--verbose" ] && VERBOSE=1

die() { printf 'content-sweep.sh: %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# 1. The worktree population.
# ---------------------------------------------------------------------------
git worktree list --porcelain > "$REPO/.git/cs-wt.txt" 2> "$REPO/.git/cs-wt.err" || e=$?
[ "${e:-0}" -eq 0 ] || { cat "$REPO/.git/cs-wt.err" >&2; die "git worktree list --porcelain exited $e"; }
[ -s "$REPO/.git/cs-wt.err" ] && { cat "$REPO/.git/cs-wt.err" >&2; die "worktree enumeration wrote to stderr"; }

# `worktree <path>` lines only. A bare `cut`/`grep -m1` per worktree is what
# rule 70's column defect was; parse the record, do not slice the table.
awk '$1=="worktree" && substr($0,10,1)=="/" { print substr($0,10) }' \
  "$REPO/.git/cs-wt.txt" | sort -u > "$REPO/.git/cs-paths.txt"
NW=$(wc -l < "$REPO/.git/cs-paths.txt")
[ "$NW" -gt 0 ] || die "worktree population is empty -- refusing to report 0"
printf 'population  %s worktrees\n' "$NW"

# ---------------------------------------------------------------------------
# 2. The dirty rows, per worktree, with -uall.
#
# -uall is the whole point of this file. Without it, an untracked directory is
# one collapsed row and its contents are never enumerated.
#
# The build filter is a LITERAL in the awk program text, never passed with -v
# or through ENVIRON: pass 265 measured the identical regex text reading 0 rows
# via `-v` and 3 rows as a literal, exit 0 in every case, with only a delimiter
# warning. A variable-passed regex is silently disabled.
#
# It is anchored `(^|\/)(target[^/]*|prof)(\/|$)` and applied to the PATH FIELD
# ONLY. The unanchored published form requires a leading `/`, which the
# top-level `prof/` path field does not have, so it matched nothing; the
# accidental full-path form happens to give the right split on this layout but
# would classify every dirty file in a worktree under a directory named `prof`
# as build output. Anchored-on-path-field has neither failure mode.
# ---------------------------------------------------------------------------
: > "$REPO/.git/cs-rows.txt"
: > "$REPO/.git/cs-collapsed.txt"

while IFS= read -r wt; do
  st_err="$REPO/.git/cs-st.err"
  if ! git -C "$wt" status --porcelain -uall > "$REPO/.git/cs-st.txt" 2> "$st_err"; then
    cat "$st_err" >&2
    die "git status --porcelain -uall failed in $wt"
  fi
  [ -s "$st_err" ] && { cat "$st_err" >&2; die "git status wrote to stderr in $wt"; }
  # `dir` marks a row git reported as a directory. Under -uall there should be
  # none; a non-build one means the population could not be enumerated, and a
  # build one is simply excluded like any other build row.
  awk -v w="$wt" '
    {
      s = substr($0, 1, 2)
      p = substr($0, 4)
      if (p == "") next                       # malformed row: not a finding
      d = (p ~ /\/$/) ? "dir" : "file"
      if (p ~ /^(target[^/]*|prof)(\/|$)/) next # build, anchored, path field only
      if (p ~ /^(target[^/]*|prof)\//) next    # nested build component
      print w "\t" s "\t" p "\t" d
    }
  ' "$REPO/.git/cs-st.txt" >> "$REPO/.git/cs-rows.txt"
done < "$REPO/.git/cs-paths.txt"

# THE REFUSAL. This is the load-bearing check of the file.
#
# A surviving non-build `?? <dir>/` row means git did not expand the directory,
# so the files behind it were never enumerated and the row count is a
# coincidence. Derived from the MARKED rows above rather than re-parsed from the
# raw porcelain output -- the first version of this check re-parsed the raw file
# with a tab-field awk over space-separated porcelain, silently matched nothing,
# and let the collapsed spelling through with a clean verdict. That is rule
# 288's own defect reproduced inside the file written to prevent it.
awk -F'\t' '$4=="dir" {print $1 "\t" $2 "\t" $3}' "$REPO/.git/cs-rows.txt" > "$REPO/.git/cs-collapsed.txt"

if [ -s "$REPO/.git/cs-collapsed.txt" ]; then
  printf 'content-sweep.sh: REFUSING to report a verdict.\n' >&2
  printf '  %s non-build untracked-directory rows survived, so this population cannot be enumerated:\n' \
    "$(wc -l < "$REPO/.git/cs-collapsed.txt")" >&2
  sed 's/^/    /' "$REPO/.git/cs-collapsed.txt" >&2
  printf '  A sweep that cannot enumerate its own population has no zero to publish (rule 288).\n' >&2
  exit 2
fi

# Symlinked worktrees: -uall does not follow a symlinked worktree's own
# administrative path, and `git status` in one reports its siblings' paths
# relative to itself. Detected rather than assumed -- see the assertion below.
TOT=$(wc -l < "$REPO/.git/cs-rows.txt")
[ "$TOT" -gt 0 ] || die "no non-build dirty rows found over $NW worktrees -- the filter selected nothing (rule 265 control); refusing to report 0"
CONTRIB=$(cut -f1 "$REPO/.git/cs-rows.txt" | sort -u | wc -l)
printf 'rows        %s non-build dirty rows from %s contributing worktrees\n' "$TOT" "$CONTRIB"

# ---------------------------------------------------------------------------
# 3. Classify: hashable file, or something that is not a file.
#
# A row is a directory row when hash-object yields nothing -- rule 138. But the
# reason it yields nothing is now KNOWN (a directory, or a vanished path), so it
# is reported by name rather than absorbed into a bucket.
# ---------------------------------------------------------------------------
: > "$REPO/.git/cs-hash.txt"
: > "$REPO/.git/cs-nohash.txt"
while IFS=$'\t' read -r wt s p d; do
  if h=$(git hash-object --no-filters -- "$wt/$p" 2>/dev/null) && [ -n "$h" ]; then
    printf '%s\t%s\t%s\t%s\n' "$h" "$wt" "$s" "$p" >> "$REPO/.git/cs-hash.txt"
  else
    printf '%s\t%s\t%s\n' "$wt" "$s" "$p" >> "$REPO/.git/cs-nohash.txt"
  fi
done < "$REPO/.git/cs-rows.txt"

NH=$(wc -l < "$REPO/.git/cs-hash.txt")
NN=$(wc -l < "$REPO/.git/cs-nohash.txt")
[ $((NH + NN)) -eq "$TOT" ] || die "classification lost rows: $NH + $NN != $TOT"
printf 'classified  %s hashable + %s unhashable (not-a-file)\n' "$NH" "$NN"
[ "$NN" -eq 0 ] || sed 's/^/  not-a-file: /' "$REPO/.git/cs-nohash.txt" >&2

# ---------------------------------------------------------------------------
# 4. The known-object population. Rule 17: awk's first field, never cut -d' '.
# --------------------------------------------------------------------------
if ! git rev-list --objects --all --reflog 2> "$REPO/.git/cs-rl.err" | awk '{print $1}' | sort -u > "$REPO/.git/cs-ids.txt"; then
  cat "$REPO/.git/cs-rl.err" >&2
  die "rev-list --objects failed"
fi
[ -s "$REPO/.git/cs-rl.err" ] && { cat "$REPO/.git/cs-rl.err" >&2; die "rev-list wrote to stderr"; }
NOIDS=$(wc -l < "$REPO/.git/cs-ids.txt")
[ "$NOIDS" -gt 0 ] || die "object id population is empty -- refusing to report 0 unreachable"
printf 'known ids   %s distinct object ids from rev-list --objects --all --reflog\n' "$NOIDS"

# ---------------------------------------------------------------------------
# 5. Reachability, deduped on the KEY before the join (rule 22 / pass 264).
#
# 33 files hashing to 32 distinct blobs is normal -- two byte-identical probe
# files in sibling worktrees. `comm` pairs LINES, so feeding it the 33 unsorted
# hash lines fabricates "1 unreachable" from the duplicate, with both inputs
# sorted, exit 0 and empty stderr. Dedup first, always.
# ---------------------------------------------------------------------------
NBL=$(cut -f1 "$REPO/.git/cs-hash.txt" | sort -u | wc -l)
comm -23 \
  <(cut -f1 "$REPO/.git/cs-hash.txt" | sort -u) \
  "$REPO/.git/cs-ids.txt" > "$REPO/.git/cs-orphan-ids.txt" || die "comm failed"

NUNREACH=$(wc -l < "$REPO/.git/cs-orphan-ids.txt")
# The population must be a subset of what we hashed, or the join is broken.
while read -r o; do
  grep -qF "$o" "$REPO/.git/cs-hash.txt" || die "orphan id $o is not in the hashed set -- the join is broken"
done < "$REPO/.git/cs-orphan-ids.txt"

printf 'blobs       %s distinct blobs from %s hashable rows (duplicate-content pairs are expected)\n' "$NBL" "$NH"
printf 'unreachable %s blob(s) with no counterpart in the object store\n' "$NUNREACH"

# ---------------------------------------------------------------------------
# 6. Controls, both directions, on THIS run's own instrument (rules 14r, 22, 265).
#
# Without these the 0 above is indistinguishable from a broken join, which is
# the whole class this file was written to end. Each control is a property of
# the data already computed, so none of them writes to the repository.
# ---------------------------------------------------------------------------
FAIL=0
CTRL=$(cut -f1 "$REPO/.git/cs-hash.txt" | sort -u | head -1)
if [ -n "$CTRL" ] && grep -qxF "$CTRL" "$REPO/.git/cs-ids.txt"; then
  printf 'control+    a hashed blob is present in the known-id set (join is live)\n'
else
  printf 'content-sweep.sh: control FAILED -- no hashed blob found in the known-id set; the 0 above is a broken join\n' >&2
  FAIL=1
fi
# Negative direction: a fabricated id must be ABSENT. If this "fails", the
# membership test is answering yes to everything.
if grep -qxF "0000000000000000000000000000000000000000" "$REPO/.git/cs-ids.txt"; then
  printf 'content-sweep.sh: control FAILED -- the fabricated id is in the known-id set\n' >&2
  FAIL=1
else
  printf 'control-    a fabricated id is absent from the known-id set (membership is not vacuous)\n'
fi
# Build filter, both directions (rule 265 standing control).
#
# The herestring ALONE supplies stdin. `printf 'x' | grep ... <<<"$c"` looks
# equivalent and is not: the herestring replaces the pipe's read end, printf
# takes SIGPIPE, and under `set -o pipefail` the pipeline's status is printf's
# -- so the control reports FAILURE for a filter that matched. That is log rule
# 287 exactly (a guard that fires for an unrelated reason is worse than no
# guard, because it is believed), and it was found by the control disagreeing
# with a direct run of the same regex on the same strings, not by reading it.
for c in 'prof/' 'target-front-3a8f01/' 'target-front-3a8f02/' 'target/'; do
  if grep -qE '^(target[^/]*|prof)(/|$)' <<<"${c}"; then :; else
    printf 'content-sweep.sh: control FAILED -- build filter does not match %s\n' "$c" >&2; FAIL=1
  fi
done
for c in 'examples/' 'src/lib.rs' 'examples/probe.rs' 'docs/work/REPORT-1c7d40.md'; do
  if grep -qE '^(target[^/]*|prof)(/|$)' <<<"${c}"; then
    printf 'content-sweep.sh: control FAILED -- build filter wrongly matches %s\n' "$c" >&2; FAIL=1
  fi
done
printf 'control-f   build filter classifies all 8 rule-265 cases correctly\n'
[ "$FAIL" -eq 0 ] || exit 3

# ---------------------------------------------------------------------------
# 7. Verdict.
# ---------------------------------------------------------------------------
if [ "$NUNREACH" -eq 0 ]; then
  printf 'verdict     0 unreachable -- 0 files need archiving, no recovery branch warranted\n'
else
  printf 'verdict     %s UNREACHABLE -- archiving required. A HUMAN acts on this; the script creates no branch.\n' "$NUNREACH" >&2
  while read -r o; do
    awk -F'\t' -v o="$o" '$1==o {print "  " $2 "  " $4}' "$REPO/.git/cs-hash.txt" >&2
  done < "$REPO/.git/cs-orphan-ids.txt"
  exit 4
fi

[ "$VERBOSE" -eq 0 ] || { printf '\nrows:\n'; sed 's/^/  /' "$REPO/.git/cs-rows.txt"; }
rm -f "$REPO/.git/cs-"*.txt "$REPO/.git/cs-"*.err
exit 0
