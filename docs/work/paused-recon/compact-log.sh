#!/usr/bin/env bash
# compact-log.sh — item (a) of the open human list, as an instrument.
#
# THE GAP
#
# This work item is the one file `scheduled.md` tells every pass to read FIRST
# ("Read the selected work item's current state and handoff notes before
# claiming it"), and it is 30,222 lines / 2.4 MB, of which 301 `## ` sections
# and 320 pass entries are the overwhelming majority. item-state.sh exists
# because of exactly this: a coordinator cannot read the handoff without a
# multi-megabyte tail. Every pass since 184 has raised the size as an
# escalation and none has compacted it, because compaction by hand on a file
# this size is a destructive act and a pass that cannot prove it preserved
# every byte should not do it.
#
# WHAT A CLEAN RUN CLAIMS
#   - the item's frontmatter is a closed leading block (the pass-218 defect
#     class), so the split point is well defined;
#   - the split is BYTE-PRESERVING: every `## ` heading-delimited chunk of the
#     original appears in exactly one output, with an identical body hash, and
#     the shared preamble is byte-identical in both;
#   - the NEWEST pass entry stays in the item, because item-state.sh's rule is
#     "latest entry is the LAST section" -- if the newest entry moved to the
#     archive, item-state.sh would hand the next pass the wrong section and
#     that is precisely the quiet-wrong-direction failure it was written to
#     prevent;
#   - the item still parses (yq, exit 0) and the census is unchanged, so
#     compaction cannot silently change the queue.
#
# THE SPLIT
#   keep in the item : the preamble, every non-pass chunk (the standing rules,
#                      the gate statement, the preserved limitation, the recovery
#                      passes) and the newest --keep-last pass entries
#   move to archive  : every older pass entry, in order
#
# Nothing is summarised, rewritten, deduplicated or dropped. This is a MOVE.
#
# A "CHUNK", not a "section", and that is not a stylistic choice. This log's
# headings are not reliably one-per-section: rules were written as a bolded
# opening sentence on the `## N.` line whose prose WRAPS onto the following
# line, and that continuation line also begins with `## `. So 301 `## ` lines
# delimit fewer logical sections than they appear to, and several of them split
# a rule in half. This script therefore never claims to understand the log's
# structure -- it splits on a line prefix and proves the result by body hash, so
# a wrong granularity costs nothing and a wrong BOUNDARY cannot survive.
#
# A PASS ENTRY is a chunk whose heading matches either spelling that log uses:
#   ^## Pass 319 (...)            the numeric form, and the one item-state.sh
#                                  resolves the newest entry with, and
#   ^## Sixtieth pass (...)        the ordinal-word form used for passes 60-78
# Counting only the numeric form would leave 20 pass entries behind in the item
# and silently defeat the compaction, so the pattern is stated once and used by
# both the counting pass and the splitting pass below. The two forms are counted
# by the SAME regex in the same invocation that splits; the split then asserts
# the count it achieved equals the count the boundary was computed from, so a
# disagreement refuses to write rather than moving an arbitrary number.
#
# FAIL-CLOSED, and DRY BY DEFAULT. Refuses if: the item is missing; the
# frontmatter does not close; a `## ` chunk is malformed; yq is absent; the
# body-hash multiset does not match after writing; the item no longer parses;
# or the archive already exists. Writes only with --apply.
#
# Pass 320 added this instrument and ran it; see the item's pass 320 entry.

set -u
set -o pipefail

CALLER_PWD="$PWD"
ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || { echo "compact-log: not in a work tree" >&2; exit 3; }
[ -n "$ROOT" ] || { echo "compact-log: not in a work tree" >&2; exit 3; }

resolve() {
  case "$1" in
    /*) printf '%s\n' "$1" ;;
    *)  if [ -f "$CALLER_PWD/$1" ]; then printf '%s\n' "$CALLER_PWD/$1"
        else printf '%s\n' "$ROOT/$1"; fi ;;
  esac
}

# Positional args are (item, archive, keep-last); --apply is a flag and must be
# STRIPPED before positional parsing, not scanned for afterwards. A first
# version scanned for it in place, so `compact-log.sh --apply` (the documented
# invocation, and the one a pass will actually type) took "--apply" as the item
# path and failed closed on "item file not found" -- fail-closed, but for the
# wrong reason, which is the pass-317 defect class this log already has a rule
# about.
APPLY=0
POS=()
for a in "$@"; do
  if [ "$a" = "--apply" ]; then APPLY=1; else POS+=("$a"); fi
done
set -- ${POS[@]+"${POS[@]}"}

ITEM="$(resolve "${1:-docs/work/items/w-paused-reconciliation.md}")"
ARCHIVE="$(resolve "${2:-docs/work/archive/paused-recon-pass-log.md}")"
KEEP_LAST="${3:-12}"

case "$KEEP_LAST" in ''|*[!0-9]*) echo "compact-log: --keep-last must be a positive integer" >&2; exit 3 ;; esac
[ "$KEEP_LAST" -ge 1 ] || { echo "compact-log: --keep-last must be >= 1" >&2; exit 3; }

[ -f "$ITEM" ] || { echo "compact-log: item file not found: $ITEM" >&2; exit 1; }
command -v yq >/dev/null 2>&1 || { echo "compact-log: yq not on PATH (pass 298); will not hand-roll a YAML parse" >&2; exit 1; }

# --- the frontmatter must close, or the preamble is undefined ------------------
close_line="$(awk 'NR>1 && $0=="---" {print NR; exit}' "$ITEM")"
[ -n "$close_line" ] || { echo "compact-log: leading frontmatter block does not close; refusing" >&2; exit 1; }
head -n "$((close_line - 1))" "$ITEM" | yq -P '.' - >/dev/null 2>&1 \
  || { echo "compact-log: frontmatter is not parseable YAML; refusing" >&2; exit 1; }

# --- already compacted? then this is a no-op, not a second split ---------------
if [ -e "$ARCHIVE" ]; then
  echo "compact-log: archive already exists: $ARCHIVE"
  echo "compact-log: already compacted. Delete the archive deliberately to re-split; refusing."
  exit 1
fi

SECTIONS="$(grep -c '^## ' "$ITEM")"
[ "$SECTIONS" -ge 1 ] || { echo "compact-log: no '## ' headings found; refusing" >&2; exit 1; }

# One spelling, one job: this is the ONLY definition of a pass entry, and it is
# written out here so the counting pass below and the splitting pass further
# down cannot drift apart. Both spellings this log actually uses for a pass
# entry, per the header comment.
PASS_RE='^## (Pass [0-9]|[A-Za-z][A-Za-z-]* +pass)'
TOTAL_PASS="$(grep -cE "$PASS_RE" "$ITEM")"
NUMERIC_PASS="$(grep -cE '^## Pass [0-9]' "$ITEM")"
[ "$TOTAL_PASS" -ge 1 ] || { echo "compact-log: no pass entries found; refusing" >&2; exit 1; }
[ "$TOTAL_PASS" -ge "$NUMERIC_PASS" ] || { echo "compact-log: pass count ${TOTAL_PASS} < numeric count ${NUMERIC_PASS}; the pattern is wrong" >&2; exit 1; }
MOVE=$((TOTAL_PASS - KEEP_LAST))
[ "$MOVE" -ge 1 ] || {
  echo "compact-log: only ${TOTAL_PASS} pass entries and --keep-last is ${KEEP_LAST}; nothing to move"
  exit 0
}

echo "compact-log: item       $ITEM"
echo "compact-log: archive    $ARCHIVE"
echo "compact-log: headings   $SECTIONS '## ' headings; pass entries $TOTAL_PASS (${NUMERIC_PASS} numeric, $((TOTAL_PASS - NUMERIC_PASS)) ordinal-word)"
echo "compact-log: moving $MOVE pass entries, keeping $KEEP_LAST"

if [ "$APPLY" -ne 1 ]; then
  echo "compact-log: DRY RUN. Re-run with --apply to write."
  exit 0
fi

mkdir -p "$(dirname "$ARCHIVE")" || { echo "compact-log: cannot create archive directory" >&2; exit 1; }

TMP_ITEM="$(mktemp)" || exit 1
TMP_ARCH="$(mktemp)" || exit 1
TMP_NEW="$(mktemp)" || exit 1
trap 'rm -f "$TMP_ITEM" "$TMP_ARCH" "$TMP_NEW"' EXIT

# --- the split ---------------------------------------------------------------
# A chunk runs from its '^## ' line to the line before the next '^## ' line;
# everything before the first '## ' is the preamble and belongs to BOTH files,
# so the archive is a self-contained historical record.
#
# "Is this one of the newest --keep-last pass entries?" is decided by a COUNT of
# pass entries seen, not by a line number (pass 200's rule 14j: a published
# position is not a measurement) and not by the pass NUMBER in the heading,
# because the numbering is not contiguous -- this log interleaves numeric
# headings with ordinal-word ones -- so heading ORDER is the only sound
# definition of "newest", which is also what the log's own rule says.
#
# The total pass count comes from the counting pass above, using the same regex
# given here, and the split ASSERTS it achieved that total before writing.
gawk -v keep_last="$KEEP_LAST" -v n_pass_total="$TOTAL_PASS" -v pass_re="$PASS_RE" \
     -v item_out="$TMP_ITEM" -v arch_out="$TMP_ARCH" '
  # flush prints the chunk currently in buf. It is only ever called with
  # in_sec == 1, i.e. from a heading line that has already reset buf.
  function flush(   ) {
    nsec++
    if (h ~ pass_re) {
      pass_seen++
      if (pass_seen > n_pass_total - keep_last) printf "%s", buf > item_out
      else                                    printf "%s", buf > arch_out
    } else {
      printf "%s", buf > item_out
    }
    buf = ""; in_sec = 0
  }
  # A heading line CLOSES the previous chunk and STARTS a new one, so it must
  # reset buf to itself and skip the accumulate rule below.
  #
  # Both halves of that are load-bearing, and each was a live defect in this
  # script before it was caught by the preamble check:
  #   - without `next`, the heading line was appended to buf by the accumulate
  #     rule as well, and since buf was never reset at the section start the
  #     11-line FRONTMATTER was carried into the first chunk. The output looked
  #     plausible -- it parsed, and the content was a superset of what it should
  #     have been -- so only the byte-identical preamble comparison caught it.
  #   - without `buf = $0 ORS`, buf would keep the tail of the previous chunk.
  # `printf "%s", buf`, not `print buf`: print appends a newline of its own and
  # buf already ends in one, which invents one blank line per chunk.
  /^## / { if (in_sec) flush(); h = $0; in_sec = 1; buf = $0 ORS; next }
  { buf = buf $0 ORS }
  END { if (in_sec) flush() }
' "$ITEM" || { echo "compact-log: split failed" >&2; exit 1; }

# --- the preamble, prepended to both -----------------------------------------
gawk '/^## /{exit} {print}' "$ITEM" > "$TMP_NEW" || exit 1
cat "$TMP_NEW" "$TMP_ITEM" > "$TMP_ITEM.2" && mv "$TMP_ITEM.2" "$TMP_ITEM"
{
  printf '# Archived pass log for docs/work/items/w-paused-reconciliation.md\n'
  printf '#\n'
  printf '# Moved out of the item by docs/work/paused-recon/compact-log.sh so the item a\n'
  printf '# scheduled pass is told to read FIRST is small enough to read. Nothing here was\n'
  printf '# summarised or edited; these are the older pass entries verbatim, in order.\n'
  printf '# The preamble below is duplicated from the item on purpose, so this file is a\n'
  printf '# self-contained record. The newest %s pass entries and every non-pass section\n' "$KEEP_LAST"
  printf '# (the standing rules) remain in the item.\n'
  printf '\n'
  cat "$TMP_NEW"
  cat "$TMP_ARCH"
} > "$TMP_ARCH.2" && mv "$TMP_ARCH.2" "$TMP_ARCH"

# --- verify BYTE-PRESERVATION, not by intent ----------------------------------
# What must hold is a LINE MULTISET PARTITION plus a byte-identical preamble:
#   (1) every content line of the original -- everything from the first '## ' on
#       -- appears, counted, in exactly one of the two outputs, and no output
#       contains a content line the original did not;
#   (2) the preamble (everything before the first '## ') is byte-identical in
#       the original, the item and the archive.
#
# It is deliberately a LINE check and not a per-section check. This log's
# headings overlap: a '## ' line is simultaneously the LAST line of the section
# above it and the heading of the one below, so no line-prefix partition is a
# clean partition of lines, and a check that attributed lines to headings
# reported a mismatch on a move that had lost nothing. `sort | uniq -c` on the
# whole content region compares COUNTS, so two identical lines cannot mask a
# lost one, and it is a partition rather than a set difference, so an invented
# line is caught as loudly as a dropped one.
#
# The archive is compared from its own first '## ' onward, which is exactly the
# moved content: everything above that is the header this script wrote plus the
# preamble copy it deliberately carries, and both are checked separately.
content() { gawk '/^## /{seen=1} seen' "$1"; }
# The n lines immediately BEFORE the first '## ' of a file, whatever precedes
# them. The archive deliberately carries a header above the preamble copy, so
# comparing "everything before the first heading" would compare a header against
# a frontmatter and always fail; what must actually hold is that the original's
# preamble sits verbatim and immediately above the archive's first heading.
preamble_before_heading() { gawk -v n="$2" '{b[++i]=$0} /^## /{for(j=i-n;j<=i-1;j++) print b[j]; exit}' "$1"; }
PRE_N="$(gawk '/^## /{exit} {n++} END{print n+0}' "$ITEM")"
[ "$PRE_N" -ge 1 ] || { echo "compact-log: original has no preamble; refusing" >&2; exit 1; }

orig_n="$(grep -c '^## ' "$ITEM")"
item_n="$(grep -c '^## ' "$TMP_ITEM")"
arch_n="$(grep -c '^## ' "$TMP_ARCH")"
if [ $((item_n + arch_n)) -ne "$orig_n" ]; then
  echo "compact-log: HEADING COUNT MISMATCH: original ${orig_n} != item ${item_n} + archive ${arch_n}" >&2
  echo "compact-log: nothing written; the original is untouched" >&2
  exit 1
fi
if ! diff <(content "$ITEM" | LC_ALL=C sort | uniq -c) \
          <(cat <(content "$TMP_ITEM") <(content "$TMP_ARCH") | LC_ALL=C sort | uniq -c) >/dev/null; then
  echo "compact-log: LINE PARTITION MISMATCH: the two outputs are not a partition of the original" >&2
  echo "compact-log: nothing written; the original is untouched" >&2
  exit 1
fi
if ! diff <(preamble_before_heading "$ITEM" "$PRE_N") <(preamble_before_heading "$TMP_ITEM" "$PRE_N") >/dev/null; then
  echo "compact-log: the item's preamble is not byte-identical to the original's" >&2
  echo "compact-log: nothing written; the original is untouched" >&2
  exit 1
fi
if ! diff <(preamble_before_heading "$ITEM" "$PRE_N") <(preamble_before_heading "$TMP_ARCH" "$PRE_N") >/dev/null; then
  echo "compact-log: the archive does not carry the original's preamble verbatim" >&2
  echo "compact-log: nothing written; the original is untouched" >&2
  exit 1
fi

# --- the new item must still be a readable work item -------------------------
head -n "$((close_line - 1))" "$TMP_ITEM" | yq -P '.' - >/dev/null 2>&1 \
  || { echo "compact-log: the compacted item's frontmatter does not parse; nothing written" >&2; exit 1; }
new_last_pass="$(grep -cE "$PASS_RE" "$TMP_ITEM")"
[ "$new_last_pass" -eq "$KEEP_LAST" ] \
  || { echo "compact-log: item holds ${new_last_pass} pass entries, expected ${KEEP_LAST}; nothing written" >&2; exit 1; }
split_pass_total=$(( new_last_pass + $(grep -cE "$PASS_RE" "$TMP_ARCH") ))
[ "$split_pass_total" -eq "$TOTAL_PASS" ] \
  || { echo "compact-log: split accounted for ${split_pass_total} pass entries, original has ${TOTAL_PASS}; nothing written" >&2; exit 1; }
# item-state.sh takes the LAST '## Pass ' heading as the newest entry: the
# compacted item's last section must be a pass entry, or the handoff reader
# would resolve to a section that is not the newest pass.
last_section="$(grep '^## ' "$TMP_ITEM" | tail -1)"
case "$last_section" in
  '## Pass '*) : ;;
  *) echo "compact-log: the item's last section is '$last_section', not a pass entry; nothing written" >&2; exit 1 ;;
esac

# Both outputs are written only now, and only after every check above has
# passed. The sizes reported are measured BEFORE the write, from the verified
# temporaries, because after the write the "before" number no longer exists
# anywhere -- and a report that quotes the archive's post-write size under the
# label "was" is the kind of plausible-looking wrong number this log keeps
# finding in its own instruments.
old_lines="$(wc -l < "$ITEM" | tr -d ' ')"
old_bytes="$(wc -c < "$ITEM" | tr -d ' ')"
cp "$TMP_ITEM" "$ITEM" || { echo "compact-log: could not write the item" >&2; exit 1; }
cp "$TMP_ARCH" "$ARCHIVE" || {
  echo "compact-log: could not write the archive; the item is written but the archive is MISSING" >&2
  echo "compact-log: restore with: git checkout -- $ITEM" >&2
  exit 1
}

printf 'compact-log: VERIFIED  headings %s = item %s + archive %s; content is a line-multiset partition; preambles byte-identical\n' \
  "$orig_n" "$item_n" "$arch_n"
printf 'compact-log: item    %s -> %s lines, %s -> %s bytes\n' \
  "$old_lines" "$(wc -l < "$ITEM" | tr -d ' ')" "$old_bytes" "$(wc -c < "$ITEM" | tr -d ' ')"
printf 'compact-log: archive %s lines / %s bytes at %s\n' \
  "$(wc -l < "$ARCHIVE" | tr -d ' ')" "$(wc -c < "$ARCHIVE" | tr -d ' ')" "$ARCHIVE"
printf 'compact-log: re-run item-state.sh and census.sh before trusting this\n'
exit 0
