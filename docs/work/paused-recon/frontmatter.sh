#!/usr/bin/env bash
# frontmatter.sh -- validate the PARSEABILITY of every census-population file's
# leading frontmatter block, as a RUNNING instrument.
#
# WHY THIS FILE EXISTS. Pass 297 left one standing fact un-instrumented: the
# block-form probe `sed -n '2,/^---$/p' FILE | yq .`. census.sh (pass 295) was
# the first script to reason about frontmatter validity, and it did so by
# STRUCTURE and IDENTITY -- it selects `work_item: true` inside a closed leading
# block and refuses on duplicate/ non-schema / missing-state / bad-state keys.
#
# That is the right mechanism for a census, and it is not a parse check. It never
# runs a YAML parser at all, on purpose (its own header says so). So the one
# question census.sh cannot answer is: would a CONFORMING YAML reader accept
# this block? For a census that is tolerable -- the census only needs to read
# `work_item` and `state` as text. For a work item a human or an agent is about
# to parse, it is the whole question, and it is asked by hand at every pass that
# touches frontmatter.
#
# WHY THE BLOCK FORM, AND NOT THE WHOLE FILE. `yq FILE` fails on 96 of 96 work
# items, well-formed ones included, because the body after the closing `---` is
# Markdown, not YAML. A detector whose healthy reading is "fails" has no power
# to detect anything. The block form `sed -n '2,/^---$/p' FILE` -- line 1 is the
# opening `---`, so the block starts at line 2 -- is the only form that
# discriminates. This script uses the block form, and asserts that its two
# controls disagree with each other, so a future edit that breaks the scoping
# cannot leave the script still reporting 0.
#
# WHAT THIS SCRIPT CLAIMS AND DOES NOT CLAIM. Two different defects, kept apart
# because three passes in a row conflated them:
#
#   * PARSE failure. Sufficient and sufficient ALONE: a `:` followed by a space
#     inside an unquoted plain scalar, e.g. `owner: pass 1; note: yes`. Measured
#     at pass 297 (yq v4.52.4) and re-measured below, every pass.
#     An apostrophe contributes nothing on its own -- the pass-295 story was
#     false and pass 297 corrected it. Do not reintroduce the apostrophe.
#   * SCHEMA violation. A duplicate key parses fine: a conforming reader takes
#     the LAST occurrence and continues (control (c) below proves this on this
#     host). A duplicate non-schema `prior_owner:`/`updated:` key is a real
#     defect -- the keys are outside the eight-key schema in
#     docs/skills/work-items.md -- but it is a defect census.sh reports, not one
#     a parser reports. A script that conflated the two would attribute parse
#     failures to a cause that cannot produce them.
#
# So: this script REFUSES on parse failure, and reports schema defects as a
# separate, non-fatal line for census.sh's arms. It does not claim that a block
# which parses here is well-formed.
#
# USAGE
#   docs/work/paused-recon/frontmatter.sh
#
# Exit status
#   0  every block parsed, the population matched the census, both controls
#      behaved
#   3  not in a work tree, or the population selector matches nothing
#   4  at least one block failed to parse
#   5  the population shrank below the census's discoverable count
#   6  a control failed -- the probe has no power, discard the run
set -uo pipefail

cd "$(git rev-parse --show-toplevel)" || { echo "frontmatter.sh: not in a work tree" >&2; exit 3; }

command -v yq >/dev/null 2>&1 || {
  echo "frontmatter.sh: yq is not on PATH -- the only discriminating probe is unavailable, refusing to report 0" >&2
  echo "frontmatter.sh: a missing instrument is not a clean result. Do NOT record this run as a measurement." >&2
  exit 3
}
YQVER=$(yq --version 2>/dev/null)

# Same population census.sh measures, and for the same reason: these two scripts
# must describe one set of files or the cross-check below is meaningless.
FILES=(docs/work/items/*.md docs/*.md)
[ -e "${FILES[0]}" ] || { echo "frontmatter.sh: docs/work/items/*.md matches nothing -- refusing" >&2; exit 3; }

# ---------------------------------------------------------------- controls
# Run FIRST, with no per-file loop over the repository, so a broken probe cannot
# be masked by a repository that happens to be clean. Both are planted under a
# throwaway tree; nothing here touches docs/ or src/.
#
# Rule 14q: a control whose result cannot be re-derived from the file it names
# manufactures confidence in the direction the conclusion already points.
PLANT=$(mktemp -d) || { echo "frontmatter.sh: mktemp failed" >&2; exit 3; }
trap 'rm -rf "$PLANT"' EXIT

mkblock() {  # mkblock FILE ; body on stdin
  { printf -- '---\n'; cat; printf -- '---\nbody text is markdown, not yaml: it has a colon-space\n'; } > "$1"
}

# (a) HEALTHY: a conforming block must PARSE.
mkblock "$PLANT/ok.md" <<'EOF'
work_item: true
id: w-000001
state: open
priority: normal
owner: null
updated: 2026-09-29T00:00:00Z
branch: null
worktree: null
EOF

# (b) THE DEFECT: a colon-space inside an unquoted plain scalar must FAIL.
#     No apostrophe anywhere in the value -- see the header.
mkblock "$PLANT/bad.md" <<'EOF'
work_item: true
id: w-000001
prior_owner: coord-0000 (pass 1; gate NO; see a re-opened block: 131 = 114)
state: open
EOF

# (c) THE DISCRIMINATOR: a duplicate key must PARSE. If this fails, the
#     script's refusal would be firing on schema defects as if they were parse
#     failures, which is precisely the pass-295 conflation.
mkblock "$PLANT/dup.md" <<'EOF'
work_item: true
id: w-000001
state: open
state: done
EOF

# (d) AN APOSTROPHE ALONE must PARSE. This is the pass-295 mechanism, planted so
#     it can only ever be falsified, not re-asserted.
mkblock "$PLANT/apos.md" <<'EOF'
work_item: true
prior_owner: coord-0000 (it's a long value with backticks and no colon-space)
state: open
EOF

# block_of FILE -- the frontmatter block, and NOTHING ELSE.
#
# `sed -n '2,/^---[ \t]*$/p' FILE` is the form this log has published for 75
# passes, and it is WRONG in a way that only shows up on a malformed file. When
# the block never closes, sed's end-pattern never matches, so it prints to END OF
# FILE and returns the whole body -- exit 0, non-empty, indistinguishable from a
# correct block. A frontmatter the census refuses as UNCLOSED would therefore be
# "validated" here. Verified by plant: on a 3-line unclosed item the published
# form returns both frontmatter lines plus nothing to mark the truncation.
#
# So the closing delimiter is located FIRST, explicitly, and its absence is a
# skip with a stated reason rather than a silent partial read. The maximum is
# generous because a legal frontmatter block is short; 200 lines is far past any
# block in this repository and bounds the scan on a file with no delimiter.
close_line() { awk 'NR>1 && /^---[ \t]*$/ { print NR; exit } NR>200 { exit }' "$1"; }
block_of()  { sed -n "2,$(( $(close_line "$1") - 1 ))p" "$1"; }
has_close() { [ -n "$(close_line "$1")" ]; }

c_ok=0; block_of "$PLANT/ok.md"    | yq . >/dev/null 2>&1 && c_ok=1
c_bad=0; block_of "$PLANT/bad.md"  | yq . >/dev/null 2>&1 && c_bad=1
c_dup=0; block_of "$PLANT/dup.md"  | yq . >/dev/null 2>&1 && c_dup=1
c_apos=0; block_of "$PLANT/apos.md" | yq . >/dev/null 2>&1 && c_apos=1

# A control that does not exercise the same filter as the measurement proves
# nothing (rule 14r). Each of these four runs the exact block-of|yq pipeline the
# census below runs; only the planted content differs.
if [ "$c_ok" -ne 1 ] || [ "$c_bad" -ne 0 ] || [ "$c_dup" -ne 1 ] || [ "$c_apos" -ne 1 ]; then
  echo "frontmatter.sh: CONTROL FAILED -- healthy parses=${c_ok} (want 1), colon-space FAILS=${c_bad} (want 1)," >&2
  echo "frontmatter.sh:   duplicate-key parses=${c_dup} (want 1), apostrophe-alone parses=${c_apos} (want 1)." >&2
  echo "frontmatter.sh: the probe cannot tell a parse failure from anything else on this host." >&2
  echo "frontmatter.sh:   DISCARD THIS RUN. Do not report a number from it." >&2
  exit 6
fi
echo "frontmatter: yq ${YQVER#yq* } | controls healthy=1 colon-space=0 duplicate=1 apostrophe=1 (all as required)"

# ------------------------------------------------------------ measurement
fail=0
n=0
unclosed=0
ERRFILE="$PLANT/yq.err"
for f in "${FILES[@]}"; do
  [ -s "$f" ] || continue
  # A file with no leading `---` is not a frontmatter carrier at all. census.sh
  # has its own structural opinion about these; here it is just "not a block",
  # and counting it as a parse failure would manufacture findings.
  head -n 1 "$f" | grep -qE '^---[ \t]*$' || continue
  # An UNCLOSED block is census.sh's UNCLOSED condition to report, not this
  # script's to report as a parse error. Skip it, and say so -- an unannounced
  # skip is how a population silently shrinks.
  if ! has_close "$f"; then
    echo "UNCLOSED $f: frontmatter opened and never closed -- not parse-checked (census.sh's arm)" >&2
    unclosed=$((unclosed + 1))
    continue
  fi
  blk=$(block_of "$f")
  n=$((n + 1))
  if ! printf '%s\n' "$blk" | yq . >/dev/null 2>"$ERRFILE"; then
    fail=$((fail + 1))
    # Name the offending line IN THE BLOCK, whose coordinate is block-line, and
    # add 1 for the file's own line 1 (the opening `---`). Pass 297's cost was
    # reading a block coordinate as a file coordinate and blaming the wrong
    # value; the arithmetic is done here so no pass has to redo it.
    #
    # yq's diagnostic is `Error: bad file '-': yaml: line N: <message>` -- the
    # line number is N, a BLOCK coordinate, buried after two prefixes. Match the
    # number, do not quote the whole line: this script's own first version
    # anchored on `^line `, which never matches, so every real failure silently
    # degraded to the "no line number, here is the whole block" fallback and
    # still exited 4. The refusal was right and the diagnosis was useless.
    ln=$(sed -n 's/.*yaml: line \([0-9][0-9]*\):.*/\1/p' "$ERRFILE" | head -n 1)
    if [ -n "${ln:-}" ]; then
      val=$(printf '%s\n' "$blk" | sed -n "${ln}p")
      echo "PARSE-FAIL $f: block line ${ln} = file line $((ln + 1)): $val" >&2
      echo "              mechanism: a ':' followed by a space inside an unquoted plain scalar" >&2
      echo "              (yq: $(head -n 1 "$ERRFILE"))" >&2
    else
      echo "PARSE-FAIL $f: yq failed without a recognisable line number; the block is:" >&2
      printf '%s\n' "$blk" | sed 's/^/              /' >&2
      echo "              (yq: $(head -n 1 "$ERRFILE"))" >&2
    fi
  fi
done
rm -f "$ERRFILE"

echo "frontmatter: ${n} leading blocks checked, ${fail} failed to parse"
if [ "$unclosed" -ne 0 ]; then
  echo "  note         ${unclosed} file(s) skipped as UNCLOSED -- NOT parse-checked; see stderr, census.sh is the authority" >&2
fi

if [ "$fail" -ne 0 ]; then
  echo "frontmatter.sh: REFUSING to report a clean frontmatter state over ${fail} block(s) a conforming YAML reader cannot parse." >&2
  echo "frontmatter.sh: a work item whose frontmatter will not parse is invisible to any tool that reads metadata." >&2
  exit 4
fi

# ------------------------------------------------- cross-check against census
# The population this script scanned must be the population census.sh counted.
# If a file moves out of $FILES, this shrinks and is caught; if a file gains a
# leading `---`, census's count and this count diverge and the run is suspect.
# Either way, do not read a shrink as "fewer problems".
cen=$(gawk '
  FNR == 1 { fm = 0; wi = 0; iswi = 0 }
  /^---[ \t]*$/ { if (fm == 0) { fm = 1; next } fm = 0; next }
  fm && /^work_item:[ \t]*true[ \t]*$/ { wi = 1; iswi = 1 }
  ENDFILE { if (wi && iswi) t++; fm = 0; wi = 0; iswi = 0 }
  END { print t + 0 }' "${FILES[@]}" 2>/dev/null)
crc=$?
# gawk exits 2 on "cannot open file" when a glob matched nothing -- a BAD
# POPULATION, not an empty queue. Read separately from the 0 it prints, because
# the zero is indistinguishable from a real empty census and the two want
# opposite responses. This script's own first version lost exactly that
# distinction and exited 5 for a directory it should have complained about.
if [ "$crc" -ne 0 ]; then
  echo "frontmatter.sh: the census cross-check could not read the population (gawk rc=${crc})." >&2
  echo "frontmatter.sh: at least one element of docs/work/items/*.md docs/*.md does not exist." >&2
  echo "frontmatter.sh: that is a BROKEN POPULATION, not an empty queue. Refusing rather than reporting 0." >&2
  exit 5
fi
if [ "${cen:-0}" -lt 1 ]; then
  echo "frontmatter.sh: the identity selector now matches 0 discoverable work items -- refusing" >&2
  exit 5
fi
if [ "$n" -lt "$cen" ]; then
  short=$((cen - n))
  echo "frontmatter.sh: scanned ${n} blocks but the census counts ${cen} discoverable items -- a shortfall of ${short}." >&2
  if [ "$unclosed" -eq "$short" ]; then
    echo "frontmatter.sh: the shortfall EQUALS the ${unclosed} UNCLOSED file(s) named on stderr, so every closed block" >&2
    echo "frontmatter.sh: was checked. This is a real unparseable-for-a-reader item, not a skipped scan. Refusing." >&2
  else
    echo "frontmatter.sh: the shortfall does NOT equal the ${unclosed} UNCLOSED file(s), so some items were scanned by" >&2
    echo "frontmatter.sh: neither arm. A population disagreement is a broken instrument, not a finding. Refusing." >&2
  fi
  exit 5
fi
echo "  population   scanned ${n} leading blocks; census discovers ${cen} work items (no item skipped)"
# `n` counts BLOCKS, `cen` counts DISCOVERABLE ITEMS, and the two are not
# expected to be equal: a file may carry a work-item-shaped header with
# `work_item: false` (docs/work/items/w-0f3a17-shortlist-rule.md) and is correctly
# still a block to parse. Published so a later pass does not read the difference
# as an unexplained delta -- pass 220 shipped exactly that inflation in the
# other direction, and pass 204's rule 14o says an unexplained delta is a
# defective measurement rather than a discovery.
if [ "$n" -ne "$cen" ]; then
  echo "  population   difference ${n} - ${cen} = $((n - cen)) block(s) that are not discoverable items (expected: work_item: false headers are parsed but not counted)"
fi

# Schema defects, reported and NOT fatal, and deliberately separate from the
# parse result above. census.sh is the authority on these; naming them here is
# so a pass editing frontmatter sees them in the same output as its parse result
# and cannot mistake one for the other.
nonkey=0
dupk=0
for f in "${FILES[@]}"; do
  [ -s "$f" ] || continue
  head -n 1 "$f" | grep -qE '^---[ \t]*$' || continue
  blk=$(sed -n '2,/^---[ \t]*$/p' "$f")
  [ -n "$blk" ] || continue
  d=$(printf '%s\n' "$blk" | sed -n 's/^\([A-Za-z_][A-Za-z0-9_-]*\)[ \t]*:.*/\1/p' | sort | uniq -d)
  if [ -n "$d" ]; then
    echo "SCHEMA $f: duplicate key(s): $(printf '%s' "$d" | tr '\n' ' ') -- parses fine, but is outside the schema in docs/skills/work-items.md" >&2
    dupk=$((dupk + 1))
  fi
  nonkey=$((nonkey + $(printf '%s\n' "$blk" | sed -n 's/^\([A-Za-z_][A-Za-z0-9_-]*\)[ \t]*:.*/\1/p' \
    | grep -vxE 'work_item|id|state|priority|owner|updated|branch|worktree' | wc -l)))
done
if [ "$dupk" -ne 0 ]; then
  echo "  schema       ${dupk} file(s) carry a duplicate key -- these PARSE (control c) and are census.sh's UNCLOSED/DUPLICATE arm, not parse failures"
fi
echo "  note         non-schema key lines present: ${nonkey} (normal here; census.sh is the authority, this is a count only)"
echo "  next         a clean parse here means a conforming reader can READ these items, not that they are well-formed"
exit 0
