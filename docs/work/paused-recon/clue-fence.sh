#!/usr/bin/env bash
# clue-fence.sh -- the canonical-phrase hard-coding fence, as a RUNNING instrument.
#
# WHY THIS FILE EXISTS. `fence.awk` is the REGION STRIPPER only (rule 14x, pass
# 213): it emits each file's production region with comments removed and string
# content PRESERVED, and it matches nothing. The detection half -- deriving the
# clue alphabet, building the pattern, counting hits -- lived entirely in the
# CALLER, which meant it was re-derived by hand in every pass. That hand half
# has now failed open TWICE, in two different directions, and both times the
# output was a clean exit 0 and a plausible number:
#
#   pass 290  `grep -c` over a region that the fence had just collapsed to one
#             line; the region count itself was nearly published as the result.
#   pass 292  the pattern was built with `tr`/`paste -sd'|'`, which leaves a
#             TRAILING `|` -- an empty final alternative. In POSIX ERE an empty
#             alternative matches every line, so the per-word count came back as
#             269 / 260 / 464 / 4242 / 67 / 269: exactly the region line count of
#             every file. The signal had become its own denominator.
#
# Rule 292 states the general form and this script ENFORCES it: any count
# computed by `grep -cE "$(derived list)"` is only meaningful if the derivation
# cannot emit an empty element, and a count equal to the population size is a
# fail-open signature that should be ASSERTED, not noticed. That assertion is
# the difference between this file and the procedure it replaces.
#
# THE ALPHABET IS DERIVED, NOT RECALLED (rule 14v). It is read out of the
# document that defines the property:
#
#     docs/accepted-state-2026-09-27.md, the two `TARGET -> CLUE` lines
#
# taking the CLUE side (right of the arrow), lowercased, split on non-letters,
# de-duplicated. Never typed from memory: a remembered alphabet is exactly the
# input that lets a hard-code through.
#
# TWO COUNTS, AND WHY THEY DIFFER IN KIND:
#
#   JOINED  the canonical clue as a contiguous phrase, case-INsensitive, because
#           the defining document spells one of them in title case
#           (`Hits Justice Dupe Hid Came`) and the other in lower case. This is
#           the strong form and must be 0 in all six files.
#   PERWORD each clue word alone, case-SENSITIVE. This is deliberately the
#           weaker form: case-insensitivity is not a tightening, it is a
#           different and unusable check, because an English word in ordinary
#           prose is not a hard-code. One hit in lib.rs is ADJUDICATED (see
#           KNOWN_BENIGN below) and anything else FAILS.
#
# So the script is fail-CLOSED: joined != 0 fails, and per-word != exactly the
# adjudicated set fails. A new hard-code cannot hide behind an existing allowance
# because the allowance names its file, its line and its reason.
#
# USAGE
#   docs/work/paused-recon/clue-fence.sh              # report
#   docs/work/paused-recon/clue-fence.sh --self-test  # also run the plants
#
# Exit status: 0 only if every control behaved AND the fence is clean.

set -uo pipefail

REPO=${REPO:-$(git rev-parse --show-toplevel)}
cd "$REPO"

STRIPPER=docs/work/paused-recon/fence.awk
DEFECT=docs/accepted-state-2026-09-27.md
REGION=docs/work/paused-recon/fence.awk

# The six production files. A file added to src/ without being added here is
# invisible to the fence, so the list is asserted against the tree rather than
# trusted: if src/ grows, this script refuses rather than reporting a clean
# fence over a subset.
FILES=(src/adjacency.rs src/lexical.rs src/approx.rs src/lib.rs src/wasm.rs src/main.rs)

# Region line counts published by fence.awk's own header. They are asserted, not
# assumed: rule 14ao -- a figure that is off by one in EVERY file is a defect in
# the measurement, and the tell is uniformity. Passes 247/248 published the
# command-substitution form of these (268/259/463/4241/66/268) for two passes
# without noticing, so the check is a guard rather than a comment.
declare -A EXPECT_REGION=(
  [src/adjacency.rs]=269 [src/lexical.rs]=260 [src/approx.rs]=464
  [src/lib.rs]=4242 [src/wasm.rs]=67 [src/main.rs]=269
)

# Adjudicated benign per-word hit. `came` as the ordinary English past tense in
# a panic message; adjudicated at pass 216 and deliberately not re-opened. The
# line is named so a SECOND hit, or a hit elsewhere, is a failure rather than a
# silent pass.
KNOWN_BENIGN="lib.rs:3597 .expect(\"key came from cells\")"

fail() { printf 'clue-fence.sh: %s\n' "$*" >&2; exit 1; }

[ -x "$STRIPPER" ] || fail "$STRIPPER is not executable -- the stripper must be run as a program (pass 286: 100644 with no shebang made its empty-region abort unreachable)"
[ -f "$DEFECT" ]   || fail "$DEFECT not found -- the alphabet must be DERIVED from the defining document, not recalled"

# ---------------------------------------------------------------------------
# 1. Derive the alphabet. Refuse rather than match a partial or empty list.
# ---------------------------------------------------------------------------
ALPHA=$(node -e '
const fs = require("fs");
const lines = fs.readFileSync(process.argv[1], "utf8").split("\n");
const words = [];
for (const ln of lines) {
  // `TARGET` -> `CLUE`, CLUE side only. The backticks are pinned so the
  // arrow and the target cannot be mistaken for one another.
  const m = ln.match(/^\x60[^\x60]+\x60\s*\u2192\s*\x60([^\x60]+)\x60/);
  if (m) words.push(...m[1].toLowerCase().split(/[^a-z]+/).filter(Boolean));
}
const u = [...new Set(words)].sort();
if (u.length < 5) {
  console.error("alphabet derivation yielded " + u.length + " words -- refusing");
  process.exit(3);
}
process.stdout.write(u.join("\n"));
' "$DEFECT") || fail "alphabet derivation failed from $DEFECT -- refusing to build a pattern from a partial list"

n_alpha=$(printf '%s\n' "$ALPHA" | grep -c .)
[ "$n_alpha" -ge 5 ] || fail "derived alphabet has $n_alpha words -- refusing"

# ---------------------------------------------------------------------------
# 2. Build the pattern WITHOUT the rule-292 trap, and assert it cannot fail open.
# ---------------------------------------------------------------------------
# `tr` + `paste -sd'|'` leaves a trailing `|`, i.e. an empty final alternative,
# which matches every line. The join is done in awk with an explicit separator
# between elements, so an empty element is impossible by construction rather
# than checked after the fact -- and the empty-element case is refused below as
# well, because belt and braces is the point.
# THE PER-WORD FORM DROPS SINGLE-LETTER WORDS, and the derivation is what
# surfaced why. The clue alphabet read out of the defining document contains
# `a` -- the article in `wreck a nice beach`. A one-letter word is not evidence
# of anything: it matches every English sentence, so including it turns the
# per-word count into 74/…/ on adjacency.rs alone and the check is dead in the
# fail-open direction (a real hard-code would sit inside a number nobody can
# read). So the per-word form is built from the DERIVED list with single-letter
# words removed -- a derived, inspectable rule, not a hand-typed exception list
# that could quietly drop a real word.
#
# This loses nothing: the JOINED form keeps all nine words including `a`, so a
# contiguous `wreck a nice beach` is still caught outright, and a hard-code
# spelling the clue as a decomposed array still trips the per-word form on the
# eight remaining words. The control below proves both.
PW_RE=$(printf '%s\n' "$ALPHA" | awk 'NF && length($0) > 1 { sep = (n++ ? sep "|" : ""); sep = sep $0 } END { print sep }')
n_pw=$(printf '%s\n' "$ALPHA" | awk 'NF && length($0) > 1' | grep -c .)
[ -n "$PW_RE" ] || fail "built an EMPTY per-word pattern -- that matches every line (rule 292); refusing"
[ "$n_pw" -ge 5 ] || fail "per-word alphabet has only $n_pw words after dropping single letters -- refusing; a derived list this short means the extraction broke, not that the property is weak"
[ -n "$PW_RE" ] || fail "built an EMPTY per-word pattern -- that matches every line (rule 292); refusing"
case "$PW_RE" in
  '|*|*'|'|'*|*'|') fail "built pattern '$PW_RE' contains an empty alternative -- that matches every line (rule 292); refusing" ;;
esac
case "$PW_RE" in *'||'*) fail "built pattern contains '||' -- an empty alternative (rule 292); refusing" ;; esac

JP_RE=$(printf '%s\n' "$ALPHA" | awk 'NF { print }' | paste -sd' ' -)
[ -n "$JP_RE" ] || fail "built an EMPTY joined pattern -- refusing"

# RULE 292 SELF-CHECK, and the one that would have caught pass 292's live false
# measurement: the pattern must not match a word that is not in the alphabet.
# A pattern that matches everything is self-refuting, so this is asserted.
if printf 'zzz\n' | grep -qE "^[^a-zA-Z]*($PW_RE)"; then
  fail "the derived pattern matches 'zzz', a word not in the alphabet -- it has an empty or over-broad alternative (rule 292); refusing to report any count"
fi
# And the population self-check: a hit count equal to the region line count is
# the fail-open signature pass 292 published. Asserted on the positive control
# below, and recorded per file as a guard.

# ---------------------------------------------------------------------------
# 3. The fence itself.
# ---------------------------------------------------------------------------
# The file list must cover src/. A new production file that nobody added here
# would be silently unfenced.
declare -A declared=()
for f in "${FILES[@]}"; do declared["$f"]=1; done
missing=()
while IFS= read -r f; do
  if [ -z "${declared[$f]+x}" ]; then missing+=("$f"); fi
done < <(find src -name '*.rs' -type f | sort)
[ "${#missing[@]}" -eq 0 ] \
  || fail "src/ contains ${#missing[@]} file(s) not in this script's list: ${missing[*]} -- a production file outside the fence is an unfenced production file; refusing to report a clean fence over a subset"

printf 'fence: alphabet %d words derived from %s\n' "$n_alpha" "$DEFECT"
printf '  words      %s\n' "$(printf '%s' "$ALPHA" | tr '\n' ' ')"

total_joined=0
benign_hits=0
for f in "${FILES[@]}"; do
  # fence.awk ABORTS on an empty region (rc=2) -- that refusal is load-bearing,
  # so it is allowed to propagate rather than being folded into a count.
  region=$(mktemp)
  if ! "$STRIPPER" "$f" > "$region" 2>/tmp/clue-fence.err; then
    rc=$?
    cat /tmp/clue-fence.err >&2
    fail "the stripper refused on $f (rc=$rc) -- an empty or unreadable production region is instrument failure, not a clean fence"
  fi
  lines=$(wc -l < "$region")
  exp=${EXPECT_REGION[$f]}
  [ "$lines" -eq "$exp" ] \
    || fail "$f region is $lines lines, fence.awk publishes $exp -- publish region counts ONLY in the \`| wc -l\` form (rule 14ao); refusing to report"

  j=$(grep -cEi "$JP_RE" "$region" || true)
  # The population signature. If the joined count equals the region line count,
  # the pattern matched every line and the count is meaningless.
  [ "$j" -ne "$lines" ] || fail "$f joined count ($j) equals the region line count ($lines) -- the pattern is matching every line; refusing (rule 292)"
  total_joined=$(( total_joined + j ))

  # Per-word, case-SENSITIVE, counted with grep -oE so repetitions are counted.
  # (grep -cE would under-count several occurrences on one line, and counting
  # LINES rather than OCCURRENCES is how a planted clue hides behind an
  # existing line that already matched something else.)
  w=$(grep -oE "($PW_RE)" "$region" | wc -l)
  b=0
  if [ "$w" -gt 0 ]; then
    # Every per-word hit must be adjudicated. Named file+line, not a count.
    got=$(grep -nEo "($PW_RE)" "$region" | head -5)
    if [ "$f" = "src/lib.rs" ] && [ "$w" -eq 1 ] && grep -qE '\.expect\("key came from' "$region"; then
      b=1
      benign_hits=$(( benign_hits + 1 ))
      printf '  %-18s region %-5s joined 0  per-word %s  ADJUDICATED (pass 216: %s)\n' "$f" "$lines" "$w" "$KNOWN_BENIGN"
    else
      printf '  %-18s region %-5s joined %-3s per-word %s  UNEXPLAINED:\n' "$f" "$lines" "$j" "$w"
      printf '%s\n' "$got" | sed 's/^/      /'
      fail "$f has $w per-word clue-word occurrences that are not the single adjudicated benign hit -- this is either a hard-code or a new adjudication, and both need a human (rule 14v: the alphabet is derived, so a hit is a real hit)"
    fi
  else
    printf '  %-18s region %-5s joined %-3s per-word 0\n' "$f" "$lines" "$j"
  fi
  rm -f "$region"
done

[ "$total_joined" -eq 0 ] \
  || fail "the joined canonical clue appears $total_joined time(s) in the production regions -- a hard-coded clue is a hard-code; refusing"

# ---------------------------------------------------------------------------
# 4. Controls, in BOTH directions. A fence that cannot fail is decoration.
# ---------------------------------------------------------------------------
# Run on a COPY of a production file, never on the repository's own src/.
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
JP_PROBE="$JP_RE"

# positive: the clue planted ABOVE the test-module boundary must be caught
{ printf 'const HARD: &str = "%s";\n' "$JP_PROBE"; cat src/adjacency.rs; } > "$TMP/above.rs"
# negative: the same literal BELOW the boundary must NOT be caught, or the
# boundary is dead and the region has collapsed to something too small
{ cat src/adjacency.rs; printf '\nconst HARD: &str = "%s";\n' "$JP_PROBE"; } > "$TMP/below.rs"
# decomposed array: the joined form cannot see it (rule 14x) -- so the per-word
# form is what must catch it, and the count must equal the number of words
ARR=$(printf '%s\n' "$ALPHA" | awk 'NF { printf "%s\"%s\"", (n++ ? ", " : ""), $0 }')
{ printf 'const A: [&str; %s] = [%s];\n' "$(printf '%s\n' "$ALPHA" | grep -c .)" "$ARR"; cat src/adjacency.rs; } > "$TMP/decomp.rs"
# `mod tests {` inside a BLOCK COMMENT must not move the boundary (rule 14aq)
{ printf '/*\nmod tests {\n*/\nconst HARD: &str = "%s";\n' "$JP_PROBE"; cat src/adjacency.rs; } > "$TMP/mtblock.rs"
# `//` inside a string literal must not eat the rest of the line (rule 14aq)
{ printf 'const U: &str = "see http://x"; const HARD: &str = "%s";\n' "$JP_PROBE"; cat src/adjacency.rs; } > "$TMP/slashstr.rs"

c_above=$("$STRIPPER" "$TMP/above.rs"    2>/dev/null | grep -cEi "$JP_RE" || true)
c_below=$("$STRIPPER" "$TMP/below.rs"    2>/dev/null | grep -cEi "$JP_RE" || true)
c_mt=$("$STRIPPER"    "$TMP/mtblock.rs"  2>/dev/null | grep -cEi "$JP_RE" || true)
c_sl=$("$STRIPPER"    "$TMP/slashstr.rs" 2>/dev/null | grep -cEi "$JP_RE" || true)
c_decomp=$("$STRIPPER" "$TMP/decomp.rs"  2>/dev/null | grep -oE "($PW_RE)" | wc -l)
# The expectation is the PER-WORD alphabet size, not the joined one: the array
# spells all nine clue words, and the per-word form deliberately does not count
# the single-letter `a`. So 8 is the correct reading here, and a control that
# expected 9 would be testing the wrong thing.
c_decomp_n=$n_pw

printf '  controls    rule-292 self-check: pattern does not match "zzz"\n'
printf '             planted above mod tests  -> %s (must be 1)\n' "$c_above"
printf '             planted below mod tests  -> %s (must be 0: the boundary is real)\n' "$c_below"
printf '             mod tests in /* */      -> %s (must be 1)\n' "$c_mt"
printf '             // inside a string      -> %s (must be 1)\n' "$c_sl"
printf '             decomposed array        -> %s per-word hits (must be %s; the joined form cannot see this, rule 14x)\n' "$c_decomp" "$c_decomp_n"

[ "$c_above" -eq 1 ] || fail "control: the planted clue ABOVE the boundary was not detected (read $c_above) -- the fence does not fire on a real hard-code; refusing"
[ "$c_below" -eq 0 ] || fail "control: the planted clue BELOW the boundary WAS detected (read $c_below) -- the test-module boundary is not being honoured; refusing"
[ "$c_mt"    -eq 1 ] || fail "control: 'mod tests' inside a block comment moved the boundary (read $c_mt); refusing"
[ "$c_sl"    -eq 1 ] || fail "control: '//' inside a string literal ate the hard-code (read $c_sl); refusing"
[ "$c_decomp" -eq "$c_decomp_n" ] || fail "control: the decomposed array produced $c_decomp per-word hits, expected $c_decomp_n -- the per-word form is not seeing every clue word; refusing"

printf '  verdict     0 canonical clue occurrences in all %d production regions; %d adjudicated benign per-word hit\n' "${#FILES[@]}" "$benign_hits"
printf '  note        CI does not run this fence. .github/workflows/test.yml runs tests and clippy only.\n'
printf '              That gap is a HUMAN decision (first raised at pass 267), not a pass action.\n'
exit 0
