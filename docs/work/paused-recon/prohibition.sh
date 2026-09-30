#!/usr/bin/env bash
# Gate 9 -- the "do not merge" prohibition's OWN figures, reconciled against live git.
#
# WHY THIS EXISTS (pass 363, rule 363).
#
# Gate 7 (branches.sh) reconciles the standing section's claims about the branch a
# human is told to MERGE: its tip, its base, its payload, its effect. Gate 8
# (figures.sh) reconciles the census table's own row figures. Pass 362's NEXT (3)
# said the remaining unreconciled prose is "in the human-item paragraphs above the
# gate table, which is prose about a merge no gate adjudicates end to end", and
# named this script's subject as the place to look.
#
# It is. This file is the only instrument that reads the PROHIBITION paragraph --
# the one that says **do not merge `a29f3d7`** -- and it carries THREE figures
# (a commit count, an insertion count, a deleted-line count) on which the whole
# prohibition rests. None of them was read by any instrument, and one of them is
# FALSE on arrival in its own population (see the finding below).
#
# WHY IT MATTERS MORE THAN A NORMAL UNGATED FIGURE. A stale count in a table is
# trivia. A stale count in the sentence that forbids a merge is the sentence a
# human trusts INSTEAD of re-deriving, and a wrong one in the *over-claim*
# direction is the direction that keeps a human from merging -- so the error is
# safe and invisible, which is exactly the pair this directory keeps finding.
#
# POPULATION, STATED BEFORE THE CODE (rule 332). The `## Current gate status`
# block of w-paused-reconciliation.md, and within it the one sentence that
# PROHIBITS a merge. The block is located by its own heading, never by a line
# number (rule 330). The sentence is located by its own grammar: a bolded
# `Do not merge <token>` instruction. The three figures are located by shapes
# taken from the sentence's own units -- `<n> ... commits`, `(<n> insertions)`,
# `to deliver <n> deleted lines` -- and the branch is located from the
# backticked token the prohibition names. NO FIGURE AND NO BRANCH IS HARD-CODED
# (rule 14k): correct 397 to 398 in the prose and this goes red, which is the
# entire point.
#
# CLASS, NOT SPELLING (rule 362). Each figure is read as a digit run and then
# *normalised* (separators removed) rather than matched against the exact
# comma-grouped spelling the document happens to use. A matcher for "44,136"
# would read 0 claims the moment a human wrote "44136", and a check that reads 0
# claims on a section that plainly makes three is the fail-open direction
# (pass 317). The same applies to the line WRAP: the sentence is joined before
# extraction, so where the prose breaks the line is not part of the claim.
#
# FAIL-CLOSED (rules 334/346/351). Absent block, absent prohibition, absent
# branch token, an unresolvable branch, or any of the three figures unreadable
# is BROKEN POPULATION and this exits 3. A figure that cannot be read is NEVER
# reported as agreeing.
#
# WHAT IS CHECKED, and the one thing that is only REPORTED.
#   1. the prohibition names a branch that resolves to a live commit
#   2. ...and that commit is NOT parented on origin/main -- the paragraph's own
#      stated reason for the prohibition ("parented on this log rather than on
#      main"). A prohibition that names an already-mergeable branch is a
#      prohibition with no reason, and that is decidable.
#   3. <n> commits      == git rev-list --count origin/main..<branch>
#   4. (<n> insertions)  == the insertion field of git diff --shortstat
#                              origin/main...<branch>
#   5. to deliver <n> deleted lines
#                         == the deletion field of the branch's OWN single
#                            commit (git diff --shortstat <branch>^1 <branch>),
#                            NOT of the three-dot range -- the sentence is about
#                            what the branch delivers, and the three-dot range
#                            also reports 175 deletions, which is a different
#                            number for a different population (rule 14l).
#   - REPORTED, not gated: how many of the counted commits actually touch this
#     log file. See the finding; the prose conflated "commits" with "log
#     commits" and the gate prints the true split on both paths so the
#     over-claim stays visible rather than silent (pass 351).

set -uo pipefail

BASE=origin/main

# RESOLVE THE ROOT FIRST, THEN JOIN THE DEFAULT ITEM TO IT. A coordinator that
# runs this gate from anywhere but the repository root used to get
# "BROKEN POPULATION work item docs/work/items/w-paused-reconciliation.md not
# found" -- a sentence about ITS OWN cwd wearing the costume of a defect in the
# document it is supposed to audit (pass 317's shape). It fails closed, so it
# cannot authorise a merge, but it sends the reader to the document instead of
# to the invocation. figures.sh, item-state.sh, selfcheck.sh and compact-log.sh
# all resolve the root and none of them has this failure; gate 9 was the only
# instrument in the directory that assumed where the reader was standing.
# An explicit argument is honoured as given (absolute, else caller-cwd-relative),
# exactly as item-state.sh does, so planting a broken population still works.
CALLER_PWD="$PWD"
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
case "$ROOT" in /*) ;; *) ROOT="" ;; esac
[ -n "$ROOT" ] && cd "$ROOT" || true

fails=0
broken=0

die_broken() {
  printf 'prohibition: BROKEN POPULATION %s -- refusing, no figure is reported as agreeing\n' "$1" >&2
  printf 'prohibition: exit 3\n' >&2
  exit 3
}

ITEM="${1:-}"
if [ -z "$ITEM" ]; then
  ITEM=docs/work/items/w-paused-reconciliation.md
  case "$ROOT" in /*) ITEM="$ROOT/$ITEM" ;; esac
else
  case "$ITEM" in
    /*) ;;
    *) if [ -f "$CALLER_PWD/$ITEM" ]; then ITEM="$CALLER_PWD/$ITEM"; else ITEM="$ITEM"; fi ;;
  esac
fi

[ -f "$ITEM" ] || die_broken "work item $ITEM not found"

# --- locate the block by its heading (rule 330: never by a line number) --------
block="$(awk '
  /^## Current gate status/ { inb=1; next }
  inb && /^## / { inb=0 }
  inb { print }
' "$ITEM")"

[ -n "$block" ] || die_broken "no '## Current gate status' block found"

# --- locate the prohibition sentence by its own grammar, and JOIN its lines -----
# The sentence wraps mid-figure in the document as it stands; joining first is
# what makes the line break not part of the claim.
prose="$(printf '%s\n' "$block" \
  | tr '\n' ' ' \
  | sed 's/  */ /g' \
  | grep -o '\*\*Do not merge [^:]*\*\*[^.]*\.' \
  | head -1)"

[ -n "$prose" ] || die_broken "no '**Do not merge <token>**' prohibition sentence in the block"

# --- the branch token the prohibition names, from the sentence's own grammar ----
branch="$(printf '%s' "$prose" | grep -o '`[0-9a-f]\{7,40\}`' | head -1 | tr -d '`')"
[ -n "$branch" ] || die_broken "the prohibition names no backticked commit token"

sha="$(git rev-parse --verify --quiet "${branch}^{commit}" 2>/dev/null || true)"
[ -n "$sha" ] || die_broken "the prohibition's token $branch does not resolve to a commit"

main_sha="$(git rev-parse --verify --quiet "${BASE}^{commit}" 2>/dev/null || true)"
[ -n "$main_sha" ] || die_broken "$BASE does not resolve to a commit"

# --- the three figures, normalised to bare digit runs -------------------------
norm() { printf '%s' "$1" | tr -d ' ,'; }

n_commits="$(printf '%s' "$prose" \
  | grep -oE '[0-9][0-9,]*([ ]+[A-Za-z][^.,()]*)? commits' | head -1 \
  | grep -o '[0-9][0-9,]*' | head -1)"
n_ins="$(printf '%s' "$prose" \
  | grep -o '([0-9][0-9,]* insertions)' | head -1 \
  | grep -o '[0-9][0-9,]*' | head -1)"
n_del="$(printf '%s' "$prose" \
  | grep -o 'to deliver [0-9][0-9,]* deleted lines' | head -1 \
  | grep -o '[0-9][0-9,]*' | head -1)"

for pair in "commit-count:$n_commits" "insertion-count:$n_ins" "deleted-line-count:$n_del"; do
  name="${pair%%:*}"; val="${pair#*:}"
  [ -n "$val" ] || die_broken "the prohibition's $name could not be read from the sentence"
done

# --- the live measurements ----------------------------------------------------
m_commits="$(git rev-list --count "${main_sha}..${sha}" 2>/dev/null || echo '?')"

# DEFECT FOUND BY RUNNING THIS FILE, not by reading it, and recorded because it
# is the same shape as branches.sh's own comment above: `git diff --shortstat`
# emits a LEADING SPACE (" 73 files changed, 44136 insertions(+), ..."), so an
# anchored `^[0-9]* files\? changed, ` cannot match and the substitution is a
# NO-OP that returns the WHOLE string. The first run of this file read the
# figure as "73 files changed, 44136" against a claim of 44136 and went RED on
# a document that is exactly right -- the check condemning the truth, which
# pass 317's rule calls worse than shipping no check at all. The general form is
# branches.sh rule 332 taken one field further: a two-token-wide git variable is
# TRIMMED, and a matcher over it is not trusted because it looks anchored.
trim() { local s="$1"; s="${s#"${s%%[![:space:]]*}"}"; s="${s%"${s##*[![:space:]]}"}"; printf '%s' "$s"; }

range_shortstat="$(trim "$(git diff --shortstat "${main_sha}...${sha}" 2>/dev/null || true)")"
m_ins=''
if [ -n "$range_shortstat" ]; then
  m_ins="$(printf '%s' "$range_shortstat" | sed 's/^[0-9][0-9]* files\? changed, //; s/ insertions\?.*//')"
fi
own_shortstat="$(trim "$(git diff --shortstat "${sha}^1" "$sha" 2>/dev/null || true)")"
m_del=''
if [ -n "$own_shortstat" ]; then
  m_del="$(printf '%s' "$own_shortstat" | sed 's/.*[ ,]\([0-9][0-9]*\) deletions\?.*/\1/')"
fi

[ -n "$m_ins" ] || die_broken "git diff --shortstat $main_sha...$branch produced no insertion count"
[ -n "$m_del" ] || die_broken "git diff --shortstat $branch^1..$branch produced no deletion count"

printf 'prohibition: sentence = %s\n' "$prose"
printf 'prohibition: population = the prohibition paragraph inside the %s block of %s\n' \
  "'## Current gate status'" "$ITEM"
printf 'prohibition: branch %s resolves to %s\n' "$branch" "$(printf '%s' "$sha" | cut -c1-7)"

# --- check 1: the stated REASON -- parented on this log, not on main -----------
parent="$(git rev-parse --verify --quiet "${sha}^1^{commit}" 2>/dev/null || true)"

# The base's NAME as a reader would write it, DERIVED from the live ref rather
# than spelled. The document says "`main`", the ref is "origin/main", and the
# first version of this extractor interpolated the ref name and therefore found
# no reason span on the REAL sentence -- a check that went red on a document that
# is exactly right (pass 317). The same class as pass 362's rule, applied to a
# ref: derive the token the prose uses from the ref, do not assume the two are
# spelled alike.
base_short="${BASE##*/}"
printf 'prohibition: reason  %s is parented on %s; %s is %s\n' \
  "$branch" \
  "$(printf '%s' "${parent:-<root>}" | cut -c1-7)" \
  "$BASE" "$(printf '%s' "$main_sha" | cut -c1-7)"
if [ "$parent" = "$main_sha" ]; then
  printf 'prohibition: DEFECT  the prohibited branch IS parented on %s, so "parented on this log rather than on main" is false and the prohibition has no stated reason\n' "$BASE" >&2
  fails=$((fails + 1))
fi

# ...and the SENTENCE must actually assert the denial. The check above is on the
# live repository, so it stays green however the prose is worded -- and a plant
# that rewrote "parented on this log rather than on `main`" to "parented on
# main" read GREEN, which is a fail-open in the check I wrote minutes earlier
# and is the direction this directory treats as worst.
#
# The predicate is over the CLASS of the claim, not its spelling (rule 362): the
# base name is taken from the LIVE base the check already resolved, and the
# requirement is a NEGATOR somewhere between the parentage claim and that name.
# Any negator condemns a direct assertion; no negator means the sentence claims
# main-parentage outright. No list of the wordings the document happens to use
# is consulted, so a reworded sentence that still denies main-parentage passes
# and one that stops denying it fails.
# The clause is delimited by the SENTENCE's own grammar -- a comma or a full stop
# -- and the clause is selected by containing the base name, not by containing
# any particular verb. The first version selected it by the literal word
# "parent", which is a SPELLING: plant J reworded the reason to "it sits on
# this log, not on `main`", denied main-parentage exactly as before, and went
# RED. That is rule 362 fired on the instrument I had just written from it, and
# it is the fail-closed direction -- correct to be suspicious, wrong to condemn
# a true claim -- so the repair is to stop naming the verb.
# WINDOW, and this is the second fail-open this one check contained, found by a
# plant rather than by reading: the clause is taken from the WHOLE sentence,
# whose first clause is the instruction "**Do not merge `a29f3d7`**" -- and that
# clause contains the word "not". So a sentence reading "parented on `main`"
# with no denial at all satisfied the negator search on the IMPERATIVE and went
# GREEN. The window is now the sentence's ASSERTION -- everything after the
# instruction's own colon -- so the imperative can never supply a negator for a
# reason it does not state. General form, and it is the same one as rules
# 334/335: a pattern must not be able to match the carrier of the claim in order
# to satisfy the claim.
assertion="${prose#*:*}"
reason_clause="$(printf '%s' "$assertion" \
  | tr '.,' '\n\n' \
  | grep -F "$base_short" \
  | head -1)"
if [ -n "$reason_clause" ]; then
  if printf '%s' "$reason_clause" | grep -qiE 'rather than|instead of|not |never |no '; then
    printf 'prohibition:   ok   %-22s the sentence denies %s-parentage in its own clause\n' "reason-asserted" "$BASE"
  else
    printf 'prohibition: DEFECT  %-22s the sentence asserts %s-parentage with no denial; the live branch is parented on %s, so the prose contradicts the repository\n' \
      "reason-asserted" "$BASE" "$(printf '%s' "$parent" | cut -c1-7)" >&2
    fails=$((fails + 1))
  fi
else
  printf 'prohibition: DEFECT  %-22s the prohibition sentence never names %s, so it gives no reason a reader could check\n' \
    "reason-asserted" "$BASE" >&2
  fails=$((fails + 1))
fi

# --- checks 2-4: the three figures -------------------------------------------
check() {
  label="$1"; claimed="$2"; measured="$3"
  c="$(norm "$claimed")"; m="$(norm "$measured")"
  if [ -z "$m" ] || [ "$m" = '?' ]; then
    printf 'prohibition: DEFECT  %s: claimed %s, MEASUREMENT UNAVAILABLE (not an agreement)\n' \
      "$label" "$claimed" >&2
    fails=$((fails + 1)); return
  fi
  if [ "$c" = "$m" ]; then
    printf 'prohibition:   ok   %-22s claimed %-8s measured %s\n' "$label" "$claimed" "$measured"
  else
    printf 'prohibition: DEFECT  %-22s claimed %-8s measured %s\n' "$label" "$claimed" "$measured" >&2
    fails=$((fails + 1))
  fi
}

check "commit-count"        "$n_commits" "$m_commits"
check "insertion-count"     "$n_ins"      "$m_ins"
check "deleted-line-count"  "$n_del"      "$m_del"

# --- REPORTED, not gated: how many of the counted commits touch THIS log ------
# Printed on BOTH paths, always. This is pass 351's distinction: "not checked"
# and "not even mentioned" must both be visible.
log_commits=0
while read -r c; do
  [ -n "$c" ] || continue
  if git diff-tree --no-commit-id --name-only -r "$c" 2>/dev/null \
       | grep -qx 'docs/work/items/w-paused-reconciliation.md'; then
    log_commits=$((log_commits + 1))
  fi
done <<EOF
$(git rev-list "${main_sha}..${sha}" 2>/dev/null)
EOF
# Pass 365. These two lines are the ONLY place the 381/16 split exists anywhere in
# this repository, they print on BOTH paths always, and pass 364 proved that a
# printf format string is invisible to every document plant -- fourteen of them
# all read rc, none read the sentence. So the sentence is built here, gated, and
# only then printed (rule 364: an instrument's own output text is part of what
# it asserts, and pass 364's own NEXT named a double space after "reported" as
# the regression signal while quoting the double-spaced form as correct, so the
# criterion could not discriminate).
line_share=$(printf 'reported: log-commit share of those %s commit(s): %s touch this log file and %s do not' \
  "$m_commits" "$log_commits" "$((m_commits - log_commits))")
line_noun=$(printf 'reported: commit noun; the prose says "%s commits"; if it says "log commits" that is a POPULATION claim, and the true log-commits figure is %s' \
  "$n_commits" "$log_commits")
for _l in share noun; do
  eval "_t=\$line_$_l"
  case "$_t" in
    'reported: '?*)
      # Shortest-prefix strip, so a SECOND space survives in _r and is visible.
      # A pattern that searched the whole line for '  ' would not catch it: the
      # defect this pass repairs is adjacent to the label, and plant A (see the
      # pass-365 entry) proved that form reads GREEN on exactly this case.
      _r=${_t#reported: }
      case "$_r" in
        ' '*)
          printf 'prohibition: DEFECT reported-%s-shape -- the label is followed by two spaces, so the word after it reads as a missing value: %s\n' \
            "$_l" "$_t" >&2
          fails=$((fails + 1))
          ;;
        *)
          printf 'prohibition:   ok   reported-%s-shape  %s\n' "$_l" "$_r"
          ;;
      esac
      ;;
    *)
      printf 'prohibition: DEFECT reported-%s-shape -- the sentence does not begin with the "reported:" label: %s\n' \
        "$_l" "$_t" >&2
      fails=$((fails + 1))
      ;;
  esac
done
printf 'prohibition: %s\n' "$line_share"
printf 'prohibition: %s\n' "$line_noun"
printf 'prohibition: scope    the insertion count is the three-dot range; the deleted-line count is the branch OWN single commit (they are different populations)\n'
printf 'prohibition: not gated  whether a29f3d7 is still the right branch to prohibit (that is the containment claim, branch-containment.sh) and whether the merge has happened (gate 7)\n'

if [ "$fails" -gt 0 ]; then
  printf 'prohibition: %s defect(s)\n' "$fails" >&2
  printf 'prohibition: exit 1\n' >&2
  exit 1
fi
printf 'prohibition: 3 figure(s) + 1 reason reconciled, 0 defect(s)\n'
printf 'prohibition: the prohibition paragraph agrees with live git\n'
exit 0
