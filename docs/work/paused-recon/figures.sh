#!/usr/bin/env bash
# figures.sh — do the census TABLE's own row figures agree with the live gates?
#
# WHY THIS FILE EXISTS. Pass 361's rule: "a check on the MACHINERY that reads a
# document is not a check on the CLAIM the document makes about itself." It
# hardened the newest-entry SENTENCE and correctly noted that the other
# reader-facing claims are unchecked "by the same argument", naming among them
# the census table's own row figures:
#
#   | Work items        | ... **0 `open` / 0 `working`**, 1 `blocked`, **83 `done`**, ...
#   | `main`            | untouched: `origin/main` = `0267ade`, ... HEAD is `post-milestone-acceptance`
#   | Worktrees         | **125 registered, 125 live**, and `git worktree prune -n -v` is **empty**
#   | MadGab ... agents | **0 running in a MadGab cwd** ...
#
# Every one of those figures is DECIDABLE against a gate that already prints the
# live value beside it (census.sh, agents.sh, and git for the other two), and
# until now nothing compared them. Gates 1-7 all read this section; none of them
# read its NUMBERS. That is the same shape as gate 7's run-evidence check before
# pass 335 and as the delete-list swap before pass 345: a document claim carried
# by prose while an instrument printed a different figure next to it.
#
# SCOPE: the four rows whose figure has ONE shape, and what each is reconciled
# against.
#
#   Work items   the five backticked state names and the bolded total, against
#                census.sh's five counts and its TOTAL.
#   `main`       the `origin/main` short sha, the HEAD branch, AND the row's own
#                instruction "Do not pin HEAD to a SHA in this row".
#   Worktrees    the bolded "N registered, M live" pair and the `prune -n -v`
#                EMPTY claim, against .git/worktrees, the directories, and git.
#   Agents alive the bolded "N running in a MadGab cwd" clause, against agents.sh.
#
# WHAT IT DOES NOT CLAIM. The at-risk rows (90/89) and the production-fence row
# are deliberately NOT anchored: at-risk.sh is slow and its family is closed on
# content since pass 184, and the fence row's headline is a narrative about a
# repaired regex rather than a figure with one shape. Each unanchored row is
# PRINTED on both paths, so "not checked" is visible rather than silent (the
# pass-351 distinction). Adding one is future work, not an omission.
#
# WHY NOT EVERY NUMBER IN THOSE ROWS IS A CLAIM. The rows are append-only prose.
# The Work items row deliberately carries "11 superseded / 95 total" and a "98"
# from passes that were WRONG, and the at-risk row carries the superseded 88 and
# 89. Matching a bare number would either find a dozen or condemn the document for
# recording its own history (pass 317). So each claim is anchored to the shape
# that states it as a CURRENT figure, documented per row above.
#
# WHY THE CLAIM IS EXTRACTED, NEVER HARD-CODED. A hard-coded 96 would be a check
# that goes red when the census legitimately moves — selfcheck.sh plant H's
# defect. Correcting the row's 83 to 84 turns this red; that is a plant.
#
# FAIL-CLOSED, per rules 334/351/346. The table is a population: absent,
# non-contiguous, or missing any of the four labels is BROKEN POPULATION and
# refuses. A row whose figure cannot be read is never reported as agreeing.
#
# FOUR DEFECTS FOUND BY RUNNING THIS FILE, none visible to reading it. All four
# are recorded at their fixes; the general form is in the last one.
#   (1) the population was the SPAN between the first and last required label,
#       and the four labels are not contiguous — the two agent rows sit ABOVE
#       `Work items` — so the span cut them off and the script reported "the row
#       vanished", a sentence about its own race rather than about the document.
#       Fixed by taking the whole contiguous run of `|` lines and requiring it.
#   (2) the sha was extracted with a matcher anchored to END OF LINE, while the
#       row carries a backticked `0267ade`; the correct document read as a
#       BROKEN POPULATION, i.e. the check condemned the truth (pass 317). Fixed by
#       delimiting on the row's own backticks.
#   (3) and (4) quoting: a `$( )` nested inside a `$( )` with double-quoted
#       variables at both levels, and an awk program whose `""` terminated the
#       shell's single quote. `bash -n` reported all of them at END OF FILE, up to
#       three hundred lines from the mistake, because a quote that closed early
#       leaves an unbalanced `$(` behind. See the note on `q()` below.
#
# Usage:  figures.sh [ITEM-PATH]
# The argument exists so the failure cases can be PLANTED, as refs.sh's does.
#
# Exit: 0 reconciled; 1 a figure disagrees or a population is broken; 3 input
# error (missing item, unresolvable root, a gate that would not run).

set -uo pipefail

ITEM="${1:-}"
if ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" && [ -n "$ROOT" ] && [ "${ROOT#/}" != "$ROOT" ]; then
  cd "$ROOT" || { echo "figures: cannot cd to $ROOT" >&2; exit 3; }
fi
[ -n "$ITEM" ] || ITEM="docs/work/items/w-paused-reconciliation.md"
case "$ITEM" in /*) ;; *) ITEM="$(pwd)/$ITEM" ;; esac
if [ ! -f "$ITEM" ]; then
  echo "figures: MISSING item $ITEM" >&2
  exit 3
fi

status=0
defects=0
bad() { printf 'figures: DEFECT  %s\n' "$*" >&2; defects=$((defects + 1)); }

# THIS PASS'S FOURTH AND FIFTH DEFECTS, and the one worth keeping: the first
# three fixes still did not parse, and the cause was a `$( ... "$tbl" ... )`
# nested inside a `$( ... )`, plus an awk program containing `""` inside a shell
# single-quoted string. Bash allows nesting, but a double-quoted variable at the
# inner level closes the outer quote, and `bash -n` then reports the failure at
# END OF FILE, hundreds of lines away, because a quote that closed early leaves
# an unbalanced `$(` behind. So the rule this file is written to: **never
# re-quote a variable the outer level already quoted, and never put a `$( )`
# inside a quoted argument** — compute into a variable on its own line and
# interpolate the bare name.
#
# The repair for that then introduced a FIFTH defect of the opposite kind, and
# it is why the helper is gone: I replaced the assignments with a `q VAR VALUE`
# helper built on `read`, which reads STANDARD INPUT and ignores its arguments
# entirely — so every variable it was meant to set came back EMPTY and the
# population check correctly reported that it had found no table at all. A helper
# whose parameter list it ignores is worse than no helper, and the failure was
# silent in the direction that looks like the document being broken. Plain
# assignments are unambiguous and are what this uses.

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
PROSE="$work/table"

# --- the population: the census TABLE inside the READER-FACING section --------
# Located by SHAPE (`^\|`) rather than by a line number, which is stale the
# moment the next pass appends (rule 330). Two scoping decisions, both measured:
#
# (a) SCOPED TO THE PREAMBLE. A whole-file `^\|` scan finds 39 separate runs,
#     because the append-only log body carries its own tables. refs.sh's own scope
#     banner already says the body "is not an instruction to the next pass", and a
#     check that condemned it would be this directory's recurring defect (pass
#     317). So the population is bounded by the first `## Pass ` heading, exactly
#     as refs.sh bounds its own. The first run measured on that scope is the
#     census table at lines 283-296.
# (b) THE RUN CONTAINING `Work items`, not the first run. The preamble holds two
#     runs (283-296 and 351-359), and taking the first happened to be right for
#     the wrong reason — the census table is not defined as "the first table".
#     Requiring the run that CONTAINS a required label is a definition by content.
# Contiguity is required within the chosen run, so a table that got split in two
# is refused rather than half-reconciled.
firstpass="$(grep -n '^## Pass ' "$ITEM" | head -1 | cut -d: -f1)"
if [ -z "$firstpass" ]; then firstpass=1; fi
# ONE awk pass finds the run, rather than a shell loop that closes each run with
# a copy of the same test — the first version of this block had the test written
# TWICE (once at each run boundary, once for the final run), which is exactly the
# pass-336 defect this directory records: the same wrong scope written twice, and
# the tail case left to a hand-copied variant. A single pass that tracks the run
# in progress cannot have a tail case.
#
# The first version was also wrong in a quieter way, and it is worth recording
# because it reported itself as a claim about the DOCUMENT: it walked a list of
# line numbers and, at each gap, re-derived the run's extent from shell variables
# that were one iteration stale, so the last run was never tested and no run ever
# matched. It printed "no table in the reader-facing section contains a row
# labelled 'Work items'" — a statement about the document, which is exactly right,
# and false.
n="$(awk -v lim="$firstpass" -F'|' '
  NR < lim && /^\|/ {
    if (inrun == 0) { runlo = NR; inrun = 1 }
    runhi = NR
    l = $2
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", l)
    if (l == "Work items") hit = 1
    next
  }
  {
    if (inrun == 1 && hit == 1) { print runlo " " runhi; exit }
    inrun = 0; hit = 0
  }
  END {
    if (inrun == 1 && hit == 1) { print runlo " " runhi }
  }' "$ITEM" | head -1)"
if [ -z "$n" ]; then
  echo "figures: BROKEN POPULATION — no table in the reader-facing section contains a row labelled 'Work items'" >&2
  echo "figures: a check that cannot find its subject is not a pass" >&2
  exit 3
fi
tstart="${n%% *}"; tend="${n##* }"
if [ "$tend" -lt "$((tstart + 3))" ]; then
  echo "figures: BROKEN POPULATION — the run containing 'Work items' is only $((tend - tstart + 1)) line(s)" >&2
  exit 3
fi

# row LABEL -> that row's text. Returns the FIRST matching row, because
# `MadGab Antonina agents alive` legitimately appears twice (a superseded row and
# its correction) and preferring one silently would be a choice this check has no
# standing to make.
row() {
  awk -F'|' -v want="$1" '
    { l = $2
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", l)
      if (l == want) { print; exit } }
  ' "$PROSE"
}
sed -n "${tstart},${tend}p" "$ITEM" >"$PROSE"
for req in 'Work items' '`main`' 'Worktrees' 'MadGab Antonina agents alive'; do
  if [ -z "$(row "$req")" ]; then
    echo "figures: BROKEN POPULATION — the census table has no row labelled '$req'" >&2
    echo "figures: a check that cannot find its subject is not a pass" >&2
    exit 3
  fi
done

# --- 1. Work items vs census.sh ------------------------------------------------
if ! docs/work/paused-recon/census.sh >"$work/census" 2>/dev/null; then
  echo "figures: census.sh did not exit 0 — refusing to reconcile a figure against a gate that failed" >&2
  exit 3
fi
cval() { awk -v s="$1" '$1 == s { print $2; exit }' "$work/census"; }
c_open="$(cval open)"; c_working="$(cval working)"; c_blocked="$(cval blocked)"
c_done="$(cval done)"; c_super="$(cval superseded)"; c_total="$(cval TOTAL)"
wi="$(row 'Work items')"
if [ -z "$c_open$c_working$c_blocked$c_done$c_super$c_total" ]; then
  bad "census.sh printed no state counts; the row cannot be reconciled against nothing"
else
  for pair in "open:$c_open" "working:$c_working" "blocked:$c_blocked" \
              "done:$c_done" "superseded:$c_super"; do
    state="${pair%%:*}"; live="${pair##*:}"
    n="$(printf '%s' "$wi" | grep -oE "[0-9]+ .$state." | head -1 | cut -d' ' -f1)"
    case "$n" in
      '')     bad "the 'Work items' row states no current figure for '$state'" ;;
      "$live") : ;;
      *)       bad "$state: row says ${n:-none}, census.sh says $live" ;;
    esac
  done
  # the total's own shape is the BOLDED one; the row also carries unbolded
  # population figures (95, 98) that are deliberately not claims
  wt="$(printf '%s' "$wi" | grep -oE '\*\*[0-9]+\*\* total' | head -1 | grep -oE '[0-9]+' | head -1)"
  case "$wt" in
    '')        bad "the 'Work items' row states no bolded total" ;;
    "$c_total") : ;;
    *)          bad "total: row says ${wt:-none}, census.sh says $c_total" ;;
  esac
fi

# --- 2. `main` vs git ---------------------------------------------------------
mrow="$(row '`main`')"
if [ -z "$mrow" ]; then
  bad "the 'main' row vanished between the population check and the read"
else
  live_sha="$(git rev-parse --short origin/main 2>/dev/null)"
  # delimited by the row's own backticks, NOT by end-of-line: see defect (2)
  row_sha="$(printf '%s' "$mrow" | grep -oE '`origin/main` = `[0-9a-f]{7,40}`' | head -1 | grep -oE '[0-9a-f]{7,40}')"
  case "$row_sha" in
    '')          bad "the 'main' row states no origin/main sha" ;;
    "$live_sha") : ;;
    *)            bad "origin/main: row says ${row_sha:-none}, git says ${live_sha:-unresolvable}" ;;
  esac

  live_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)"
  row_branch="$(printf '%s' "$mrow" | grep -oiE 'HEAD is `[A-Za-z0-9._/-]+`' | head -1 | tr -d '`' | awk '{print $NF}')"
  case "$row_branch" in
    '')             bad "the 'main' row states no HEAD branch" ;;
    "$live_branch") : ;;
    *)               bad "HEAD: row says ${row_branch:-none}, git says ${live_branch:-unresolvable}" ;;
  esac

  # The row carries its OWN instruction — "Do not pin HEAD to a SHA in this row"
  # — and the reason is rule 330: a SHA there is stale the moment the next pass
  # commits. A claim about a document is a claim about a document (rule 25), and
  # this one is decidable, so it is checked rather than trusted.
  #
  # THE GUARD IS "no sha-shaped token other than origin/main's", NOT a pattern
  # matching one spelling of a pin. The first version looked for
  # `HEAD (is )?@?<hex>` and its own plant — `HEAD is \`post-...\` @ e158d2d` —
  # read GREEN, because the backticked branch name sits between `is` and the sha.
  # That is rule 288 exactly: a boundary predicate with a control only at the
  # spelling this repository happens to use, so it is green on the one spelling
  # the plant chose. The decidable form removes the origin/main claim from the
  # row and requires NOTHING sha-shaped to remain, which condemns a pin in any
  # spelling (`HEAD is e158d2d`, `HEAD @ e158d2d`, a bare `e158d2d` in a
  # parenthetical) without naming any of them.
  if printf '%s' "$mrow" | grep -qiE 'do not pin HEAD to a SHA'; then
    rest="$(printf '%s' "$mrow" | sed -E 's/`origin\/main` = `[0-9a-f]{7,40}`//')"
    extra="$(printf '%s' "$rest" | grep -oiE '(^|[^0-9a-z-])[0-9a-f]{7,40}([^0-9a-z-]|$)' | tr -d ' ')"
    if [ -n "$extra" ]; then
      bad "the 'main' row forbids pinning HEAD to a SHA, and carries sha-shaped token(s): $extra"
    fi
  else
    bad "the 'main' row no longer states its 'do not pin HEAD to a SHA' instruction; the check cannot run"
  fi
fi

# --- 3. Worktrees vs .git/worktrees and git -----------------------------------
wtrow="$(row 'Worktrees')"
if [ -z "$wtrow" ]; then
  bad "the 'Worktrees' row vanished between the population check and the read"
else
  live_reg="$(ls -1 .git/worktrees 2>/dev/null | wc -l)"
  live_reg="${live_reg// /}"
  pair="$(printf '%s' "$wtrow" | grep -oE '\*\*[0-9]+ registered, [0-9]+ live\*\*' | head -1)"
  row_reg="$(printf '%s' "$pair" | grep -oE '[0-9]+' | sed -n 1p)"
  row_live="$(printf '%s' "$pair" | grep -oE '[0-9]+' | sed -n 2p)"
  if [ -z "$row_reg" ] || [ -z "$row_live" ]; then
    bad "the 'Worktrees' row states no 'N registered, M live' figure"
  else
    if [ "$row_reg" != "$live_reg" ]; then
      bad "registered worktrees: row says $row_reg, .git/worktrees has $live_reg"
    fi
    # "live" is a property of the DIRECTORIES, a different population from the
    # registrations -- pass 188's note records `git worktree list` reading 126
    # against .git/worktrees reading 125, and that is not a delta. So live is
    # measured as registered-and-present, and the row's own equality of the two
    # is the claim under test.
    live_present=0
    for d in .git/worktrees/*/; do
      [ -d "$d" ] || continue
      live_present=$((live_present + 1))
    done
    if [ "$row_live" != "$live_present" ]; then
      bad "live worktrees: row says $row_live, $live_present of the $live_reg registered admin dirs are present"
    fi
  fi
  prune_out="$(git worktree prune -n -v 2>/dev/null)"
  if ! printf '%s' "$wtrow" | grep -qiE 'prune -n -v`? is \*?\*?empty'; then
    bad "the 'Worktrees' row no longer states that 'git worktree prune -n -v' is empty"
  elif [ -n "$prune_out" ]; then
    prune_n="$(printf '%s' "$prune_out" | wc -l)"
    prune_n="${prune_n// /}"
    bad "the 'Worktrees' row says prune -n -v is empty, and git reports $prune_n prunable registration(s)"
  fi
fi

# --- 4. MadGab agents alive vs agents.sh --------------------------------------
arow="$(row 'MadGab Antonina agents alive')"
if [ -z "$arow" ]; then
  bad "the 'MadGab Antonina agents alive' row vanished between the population check and the read"
else
  if ! docs/work/paused-recon/agents.sh >"$work/agents" 2>/dev/null; then
    bad "agents.sh did not exit 0 — refusing to reconcile the agents row against a gate that failed"
  else
    live_nt="$(awk -F'= ' '/non-terminal madgab agents =/ { print $2; exit }' "$work/agents")"
    row_nt="$(printf '%s' "$arow" | grep -oE '\*\*[0-9]+ running in a MadGab cwd\*\*' | head -1 | grep -oE '[0-9]+' | head -1)"
    case "$row_nt" in
      '')         bad "the 'MadGab Antonina agents alive' row states no bolded 'N running in a MadGab cwd' figure" ;;
      "$live_nt") : ;;
      *)           bad "non-terminal MadGab agents: row says ${row_nt:-none}, agents.sh says ${live_nt:-nothing}" ;;
    esac
  fi
fi

# --- summary ------------------------------------------------------------------
checked=4
skipped="at-risk commits (at-risk.sh is slow; closed on content since pass 184), at-risk non-build content, production fence (no single-figure headline)"
if [ "$defects" -gt 0 ]; then
  echo "figures: census table = lines ${tstart}-${tend} of this item"
  echo "figures: $checked row(s) reconciled, $defects defect(s)"
  echo "figures: not anchored, and deliberately so: $skipped" >&2
  echo "figures: REFUSING — the standing section's own figures are not what the gates measure" >&2
  status=1
else
  item_base="$(basename "$ITEM")"
  echo "figures: census table = $((tend - tstart + 1)) line(s) of $item_base (lines ${tstart}-${tend})"
  echo "figures: $checked row(s) reconciled, 0 defect(s): work items, main, worktrees, madgab agents alive"
  echo "figures: not anchored, and deliberately so: $skipped"
  echo "figures: the census table's row figures agree with the live gates"
  status=0
fi
exit "$status"
