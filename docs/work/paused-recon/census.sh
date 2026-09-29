#!/usr/bin/env bash
# census.sh -- the MadGab work-item census, as a RUNNING instrument.
#
# WHY THIS FILE EXISTS. agents.sh, at-risk.sh, content-sweep.sh and clue-fence.sh
# were written for one reason: a standing figure that every pass re-derives by
# hand is a figure, not a measurement (rule 14k). The census was the last one
# still hand-typed, and it is the figure a future pass is most likely to make a
# WORK DECISION from -- "0 open / 0 working" is what the pause rests on.
#
# The hand procedure is exactly the shape of the failures this log already owns:
#
#   1. IDENTITY, not grep. `grep -rl work_item docs/` finds
#      docs/skills/work-items.md, whose ```yaml example header contains a literal
#      `work_item: true` (rule 34 -- four live instances). A per-file
#      `head -20 | grep` loop reports a phantom open item from the skills doc.
#      Pass 203 shipped that phantom's arithmetic (98) for a full pass.
#   2. The FRONTMATTER, not the file. A file may carry a work-item-shaped header
#      with `work_item: false` (docs/work/items/w-0f3a17-shortlist-rule.md) and
#      is correctly NOT discoverable; adding its `state: done` inflated the
#      total by one (pass 220).
#   3. The frontmatter must be VALID. This item's own frontmatter regressed
#      three times (passes 218, 252, 285-289) with duplicate non-schema
#      `prior_owner:` / `updated:` keys whose unquoted values contain ": ",
#      so any conforming YAML reader fails with `mapping values are not allowed
#      in this context`. A census run over unparseable frontmatter reports a
#      plausible count for a file no reader can see.
#
# So this script REFUSES to report a number over frontmatter it has not
# validated, in the same spirit as at-risk.sh and clue-fence.sh.
#
# USAGE
#   docs/work/paused-recon/census.sh
#
# Exit status: 0 on a reported number, non-zero on instrument failure.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)" || { echo "census.sh: not in a work tree" >&2; exit 3; }

# The population is fixed and asserted against the tree, so a missing or
# renamed directory is an abort rather than a smaller number read as a change.
FILES=(docs/work/items/*.md docs/*.md)
[ -e "${FILES[0]}" ] || { echo "census.sh: docs/work/items/*.md matches nothing -- refusing" >&2; exit 3; }
[ -e "docs/skills/work-items.md" ] || { echo "census.sh: the skills doc is missing -- the rule-34 trap cannot be excluded" >&2; exit 3; }

# Schema, from docs/skills/work-items.md. Anything else in the frontmatter is a
# non-schema key -- the defect this file's own frontmatter has regressed on
# three times -- so it is reported, not silently tolerated.
SCHEMA="work_item id state priority owner updated branch worktree"
ALLOWED="open working blocked done superseded"

gawk -v schema="$SCHEMA" -v allowed="$ALLOWED" '
function split_schema(s,   a, n, i) { n = split(s, a, " "); for (i = 1; i <= n; i++) keys[a[i]] = 1 }
BEGIN { split_schema(schema); alts = allowed; gsub(/ /, "|", alts) }
FNR == 1 { infm = 0; fm = 0; wi = 0; hass = 0; nf = 0; blank = 0 }
# A leading `---` opens the frontmatter ONLY at the very first line. Opening on
# any `---` later in a file is how a horizontal rule becomes a "frontmatter".
FNR == 1 && /^---[ \t]*$/ { infm = 1; fm = 1; next }
fm && /^---[ \t]*$/ { infm = 0; fm = 0; closefm = 1; next }
fm {
  nf++
  if ($0 ~ /^[ \t]*$/) { blank++; next }
  if ($0 ~ /^---[ \t]*$/) next
  if ($0 !~ /^[A-Za-z_][A-Za-z0-9_-]*[ \t]*:/) {
    printf "MALFORMED %s:%d: frontmatter line is not a key: %s\n", FILENAME, FNR, $0 > "/dev/stderr"
    badfm++
    next
  }
  k = $0; sub(/[ \t]*:.*$/, "", k)
  if (k in seen) {
    printf "DUPLICATE %s:%d: key \"%s\" already present in this frontmatter\n", FILENAME, FNR, k > "/dev/stderr"
    badfm++
  }
  seen[k] = 1
  # A non-schema key is reported ONCE PER FILE, not once per key. Extra keys
  # (`opened_by`, `base`, `source_item`, `agents`, `reviews`, ...) are a normal
  # and long-standing convention across the items here, so 120 of them is
  # not a finding; naming each one trains a pass to read the stderr as noise,
  # which is how the load-bearing DUPLICATE line below gets skimmed past. The
  # count is still published so a sudden jump is visible.
  if (!(k in keys)) { if (!(FILENAME in nonkeyfile)) { printf "NON-SCHEMA %s: carries keys outside the work-items.md schema (not counted, not fatal)\n", FILENAME > "/dev/stderr"; nonkeyfile[FILENAME] = 1; nonkey++ } }
  if (k == "work_item") { iswi = ($0 ~ /^work_item:[ \t]*true[ \t]*$/); wi = 1 }
  if (k == "state") { st = $0; sub(/^state:[ \t]*/, "", st); sub(/[ \t]*$/, "", st); hass = 1; seen_state = st }
  next
}
ENDFILE {
  # A file that never closed its frontmatter is unparseable, whatever it counted.
  if (fm) { printf "UNCLOSED %s: frontmatter opened and never closed\n", FILENAME > "/dev/stderr"; badfm++ }
  if (wi && iswi) {
    total++
    if (!hass) { printf "NOSTATE %s: work_item:true with no state key\n", FILENAME > "/dev/stderr"; nostate++ }
    # The alternation is DERIVED from the allowed list, not hand-typed twice.
    # (A first draft read `(^|open working ...|s$)`, whose leading `^` matches
    # the empty string, so the guard approved every value including `inprogress`
    # -- a fail-open refusal, which is the most expensive kind: the arm exists
    # precisely to be unable to be wrong, and it reported itself as passing.)
    else if (seen_state !~ ("^(" alts ")$")) {
      printf "BADSTATE %s: state \"%s\" is not one of %s\n", FILENAME, seen_state, allowed > "/dev/stderr"; badstate++
    } else { c[seen_state]++ }
  } else if (wi) {
    shaped++
  }
  delete seen; infm = 0; fm = 0; wi = 0; iswi = 0; hass = 0; seen_state = ""; closefm = 0
}
END {
  if (badfm || nostate || badstate) {
    printf "census.sh: REFUSING to report a census over %d unparseable or invalid frontmatters (%d structural, %d missing state, %d bad state).\n", badfm, badfm, nostate, badstate > "/dev/stderr"
    printf "census.sh: a count taken over frontmatter no conforming reader can parse is a figure, not a measurement.\n" > "/dev/stderr"
    exit 4
  }
  printf "census: %d work items (identity: work_item:true inside a closed leading frontmatter)\n", total
  ns = split(allowed, o, " ")
  for (i = 1; i <= ns; i++) printf "  %-12s %d\n", o[i], c[o[i]] + 0
  printf "  %-12s %d\n", "TOTAL", total
  if (shaped) printf "  note          %d work-item-shaped header(s) with work_item not true are correctly NOT counted\n", shaped
  if (nonkey) printf "  note          %d file(s) carry non-schema frontmatter keys; normal here, listed once each on stderr, not counted\n", nonkey
}
' "${FILES[@]}"
rc=$?

# REFUSAL, after the fact: a census of zero items on a repository with a populated
# items directory means the selector matched nothing, not that the queue is empty.
if [ "$rc" -ne 0 ]; then exit "$rc"; fi
tot=$(gawk -v schema="$SCHEMA" -v allowed="$ALLOWED" '
BEGIN { n = split(schema, a, " "); for (i = 1; i <= n; i++) keys[a[i]] = 1 }
FNR==1 { fm=0; wi=0; iswi=0 }
/^---[ \t]*$/ { if (fm==0) { fm=1; next } fm=0; next }
fm && /^work_item:[ \t]*true[ \t]*$/ { wi=1; iswi=1 }
ENDFILE { if (wi && iswi) t++; fm=0; wi=0; iswi=0 }
END { print t+0 }' "${FILES[@]}")
if [ "${tot:-0}" -lt 1 ]; then
  echo "census.sh: the selector matched 0 discoverable work items in a populated repository -- refusing to report an empty queue" >&2
  exit 5
fi

# CONTROL, in both directions, using the SAME selector as the measurement
# (rule 14r: a control must exercise the filter the measurement uses, or it
# proves nothing).
#
# docs/skills/work-items.md carries a complete work-item-shaped header inside a
# ```yaml fence, at lines 11-20, including `work_item: true` and `state: open`.
# A fence-blind selector therefore finds a phantom OPEN item, which is how
# `1 open` reaches a pass (rule 34, four live instances). The census is correct
# here for a reason that is easy to lose: docs/skills/ is NOT in $FILES. That is
# a property of the file list, so it is asserted rather than assumed.
SKILLS=docs/skills/work-items.md
if [ -e "$SKILLS" ]; then
  # (a) the population really does exclude the skills doc
  for f in "${FILES[@]}"; do
    if [ "$f" = "$SKILLS" ]; then
      echo "census.sh: control fired -- $SKILLS is in the measured population, so its fenced example can be counted" >&2
      exit 6
    fi
  done
  # (b) the selector, applied to that file alone, reports zero
  sel=$(gawk '
    FNR == 1 && /^---[ \t]*$/ { fm = 1; next }
    fm && /^---[ \t]*$/ { fm = 0; next }
    fm && /^work_item:[ \t]*true[ \t]*$/ { n++ }
    ENDFILE { fm = 0 }
    END { print n + 0 }' "$SKILLS")
  # (c) a deliberately fence-BLIND selector on the same file DOES report one,
  #     which is what proves the exclusion is doing work rather than the file
  #     being empty. Without (c), (b) is vacuous.
  blind=$(gawk '/^work_item:[ \t]*true[ \t]*$/ { n++ } END { print n + 0 }' "$SKILLS")
  if [ "${sel:-x}" != "0" ] || [ "${blind:-0}" -lt 1 ]; then
    echo "census.sh: control failed -- selector reads ${sel:-x} (must be 0), fence-blind reads ${blind:-0} (must be >=1). The trap is not where this control thinks it is; discard the run." >&2
    exit 6
  fi
  echo "  control       skills-doc fenced example: selector 0 / fence-blind ${blind} (trap live, correctly excluded)"
fi

echo "  next          0 open / 0 working is the figure the pause rests on; compare against the previous pass before acting on it"
exit 0
