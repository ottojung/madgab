# Fence: hard-coded canonical-phrase detection in the PRODUCTION region.
#
# SCOPE, corrected at pass 214: this script is the REGION STRIPPER only.
# It does NOT match, count or detect canonical phrases. It emits each file's
# production region (comments removed) to stdout and nothing else; the caller
# pipes that through its own matcher. Pass 213 published this script as
# matching "both spellings of the clue (contiguous and decomposed)"; it
# contains no matcher and no decomposed handling whatsoever, so a decomposed
# array such as ["hits","justice","dupe","hid","came"] emits VERBATIM and any
# contiguous-phrase regex reads it 0. See rule 14x.
#
# Alphabet (rule 14v) belongs to the CALLER's matcher, not to this file: derive
# it by READING the document that defines the property
# (docs/accepted-state-2026-09-27.md lines 25 and 31), never from recall.
#
# This script exits non-zero rather than passing an empty string through to a
# matcher: a stage that fails open produces a zero indistinguishable from a
# passing result (rule 22/28/34 family). Concretely, it ABORTS if any input
# file yields an EMPTY production region, which is the failure mode that would
# otherwise turn a broken read into a clean fence 0.
#
# REGION (rules 14n, 14y -- CORRECTED at pass 215, this materially CHANGES the
# lib.rs count): everything above the TEST MODULE BOUNDARY, with comments
# stripped (// line and /* */ block, the latter tracked across lines). A file
# with no `mod tests` is measured over its whole file -- not exempt.
#
# The boundary is `mod tests`, NOT the first `#[cfg(test)]`. Pass 214 (and every
# pass before it) used the first `#[cfg(test)]`, which is WRONG in general and
# badly wrong on this repository: a `#[cfg(test)]` attribute is a PER-ITEM
# attribute, so it also marks test-only helper FUNCTIONS that sit in the middle
# of otherwise production code. In src/lib.rs the first `#[cfg(test)]` is on
# line 381, annotating the test-only helper `first_leaf_frontier`, while the
# actual test module does not begin until line 4243. The old rule therefore cut
# the fence's region at line 381 and EXCLUDED 3,861 lines of real production
# code -- including the whole `impl Generator` at line 801. A hard-code planted
# at line 382 reads 0 under BOTH spellings; the corrected region reads it.
# The other five files are unaffected (their first `#[cfg(test)]` is already the
# line above their `mod tests`).
#
# Region-line counts to expect on the six production files, measured
# 2026-09-29 (each with `wc -l` on this script's stdout -- NOT via command
# substitution): adjacency.rs 269, lexical.rs 260, approx.rs 464, lib.rs 4,242,
# wasm.rs 67, main.rs 269. lib.rs was 380 under the superseded boundary; the
# change is the blind spot closing. The other five moved by +1 each, because
# the old boundary sat on the `#[cfg(test)]` line above `mod tests` and the
# corrected one sits on the `mod tests` line itself, so the attribute line is
# now inside the region. Pass 213/214 published 268/259/463/380/67/269; those
# five figures are superseded, not contradicted.
#
# THE "NOT via command substitution" WARNING IS A REAL OFF-BY-ONE, and the
# magnitude published beside it (pass 214: "under-reads by 33 on lib.rs") was
# WRONG -- it was itself a command-substitution reading, and it propagated for
# 34 passes. Re-measured 2026-09-29 (pass 249) on this script's stdout:
#
#     file          pipe | wc -l    $( ) cmdsub    difference
#     adjacency.rs      269            268          1
#     lexical.rs        260            259          1
#     approx.rs         464            463          1
#     lib.rs           4242           4241          1
#     wasm.rs            67             66          1
#     main.rs           269            268          1
#
# The difference is exactly 1 in every file, not 33. Cause: `$( ... )` strips the
# single trailing newline, and the LAST line of every production region is the
# non-blank `#[cfg(test)]` (four files) or `}` (wasm.rs, main.rs), so `wc -l`
# counts one fewer newline-terminated line. The old figure of 33 was read off a
# region that ended in blank lines, which is not what this fence emits.
#
# CONSEQUENCE FOR THE LOG, and why this matters beyond cosmetics: passes 247 and
# 248 published "Region counts 268/259/463/4241/66/268" -- the command-substitution
# form -- while THIS FILE, correctly, documents 269/260/464/4242/67/269. So for
# two passes the log's own region counts disagreed with its own instrument by
# exactly one in all six files, in the direction that understates the region, and
# neither pass noticed because the six numbers were internally consistent and the
# invariant they gate (phrase 0 / decomp 0-0-0-1-0-0) is unaffected by one line.
# A figure that is off by one in EVERY file is not six coincidences; it is a
# defect in the measurement, and the tell is uniformity (rule 14ao).
#
# PUBLISH region counts ONLY in the `| wc -l` form, and re-derive them from this
# file's numbers above. A pass that gets 268/259/463/4241/66/268 is reading
# through command substitution, not observing a change in the region.
#
# ---------------------------------------------------------------------------
# STRING AND CHAR LITERALS ARE SCANNED (rule 14aq, pass 254). Until pass 254
# this file located `//`, `/*` and `*/` with bare `index()` -- that is, it
# treated those two characters as comment syntax wherever they appeared,
# INCLUDING inside a string literal. It also tested the `mod tests` boundary
# against the RAW line, before any comment stripping, so a `mod tests` line
# inside a block comment moved the boundary. Both are fail-open, and both were
# measured, not conjectured:
#
#   plant                                                    before   after
#   `mod tests {` inside a /* */ block comment at lib.rs:1-3     0        1
#     (before: region collapsed 4242 -> 1 line, exit 0 -- the fence was
#      measuring ONE line and reporting a clean 0, with no abort, because
#      `emitted` was 1 and only an EMPTY region aborts)
#   `const U: &str = "see http://x";` then a clue literal,
#     same line                                                0        1
#   `const S: &str = "mod tests {";` then a clue literal,
#     same line                                          (not tested)   1
#
# The first is the dangerous one and it needs no adversarial intent: an
# ordinary doc comment that says "mod tests below" is a natural thing to write,
# and its effect was to reduce the whole fence to a single line while still
# exiting 0. That is rule 22/28/34's exact shape -- a broken read that is
# indistinguishable from a passing one.
#
# The fix is `strip()`, which walks each line CHARACTER BY CHARACTER carrying
# inblock / instr / inchar / inraw, and emits TWO projections:
#
#   RES    comments removed, string CONTENT PRESERVED -- this is what the
#          caller matches, because a hard-coded clue IS a string literal and
#          blanking it would be the defect rather than the fix;
#   RESNC  comments removed and string content removed -- this is what the
#          `mod tests` BOUNDARY is tested against.
#
# Two projections because the two consumers need opposite answers about strings,
# and a single projection gets one of them wrong in the fail-open direction.
# The region counts in this file are UNCHANGED by this repair, and that is the
# evidence it is a repair rather than a loosening: the new stripper's output is
# BYTE-IDENTICAL to the old one's on all six production files (`diff` clean),
# so no region grew and none shrank. The only behavioural differences are on
# the three plants above, which previously read 0.
#
# STANDING CONTROLS -- all four must be run, because each covers a different
# fail-open path and no one of them implies the others:
#   1. `mod tests {` inside a block comment, plant a clue literal at lib.rs:300
#      -> phrase 1 (this is the pass-254 regression control)
#   2. a `//` inside a string literal, plant a clue literal later on the same
#      line -> phrase 1
#   3. a `mod tests {` string literal, plant a clue literal later on the same
#      line -> phrase 1, and the region must NOT shrink
#   4. the U+2019 apostrophe form of the target, not the ASCII one (pass 253)

BEGIN {
  inblock = 0
  thisfile = ""
}

FNR == 1 {
  thisfile = FILENAME
  emitted = 0
  inblock = 0
  pastfence = 0
  instr = 0
  inchar = 0
  inraw = 0
  rawhash = ""
}

# Is the `'` at position i a CHAR LITERAL opener, or a Rust LIFETIME?
# A char literal closes within a few characters ('a', '\n', '\'', '"', '\u{1}').
# A lifetime does not (&'a str, &'static str). Getting this wrong in the
# permissive direction leaves `inchar` stuck ON, and every subsequent `//` on
# that line and after it is eaten as string content -- a fence that silently
# stops seeing the code. Default to LIFETIME when undecided.
function ischar(s, i,   j, c) {
  if (substr(s, i + 1, 1) == "\\") {
    for (j = i + 2; j <= length(s); j++) {
      c = substr(s, j, 1)
      if (c == "\\") { j++; continue }
      return (c == "'") ? 1 : 0
    }
    return 0
  }
  return (substr(s, i + 1, 2) ~ /^.'$/) ? 1 : 0
}

# Strip ONE line of Rust into two projections, updating inblock / instr /
# inchar / inraw as a side effect:
#
#   RES    comment content removed, STRING CONTENT PRESERVED. This is what the
#          caller matches, because a hard-coded clue IS a string literal and
#          must survive the stripper.
#   RESNC  comment content removed AND string content removed. The region
#          BOUNDARY is tested against this, so text inside a string cannot move
#          the boundary.
#
# Two projections because the two consumers need opposite answers about strings,
# and each single-projection mistake fails open -- see rule 14aq for the two
# measured instances of exactly that.
function strip(s,   i, n, c, d, j, h, o, onc) {
  o = ""; onc = ""; n = length(s); i = 1
  while (i <= n) {
    c = substr(s, i, 1); d = substr(s, i + 1, 1)

    if (inblock) {
      if (c == "*" && d == "/") { inblock = 0; i += 2; continue }
      i++; continue
    }

    if (inraw) {
      # Raw string: no backslash escapes. The terminator is `"` followed by
      # exactly as many `#` as the opener had.
      if (c == "\"") {
        j = i + 1
        while (substr(s, j, 1) == "#") j++
        if (j - i - 1 == length(rawhash)) { inraw = 0; rawhash = ""; o = o "\""; i = j; continue }
      }
      o = o c; i++; continue
    }

    if (instr) {
      if (c == "\\") { o = o c substr(s, i + 1, 1); i += 2; continue }
      if (c == "\"") { o = o c; instr = 0; i++; continue }
      o = o c; i++; continue
    }

    if (inchar) {
      if (c == "\\") { o = o c substr(s, i + 1, 1); i += 2; continue }
      if (c == "'") { o = o c; inchar = 0; i++; continue }
      o = o c; i++; continue
    }

    if (c == "/" && d == "/") break              # line comment: the rest is not code
    if (c == "/" && d == "*") { inblock = 1; i += 2; continue }

    if (c == "\"") { o = o c; onc = onc c; instr = 1; i++; continue }
    if (c == "'")  { o = o c; onc = onc c; if (ischar(s, i)) inchar = 1; i++; continue }

    # Raw string opener r"..." / r#"..."#, and the b / br prefixed forms.
    if (c == "r" && (d == "\"" || d == "#")) {
      h = ""; j = i + 1
      while (substr(s, j, 1) == "#") { h = h "#"; j++ }
      if (substr(s, j, 1) == "\"") { inraw = 1; rawhash = h; o = o c; onc = onc c; i = j + 1; continue }
    }
    if (c == "b" && (d == "\"" || (d == "r" && (substr(s, i + 2, 1) == "\"" || substr(s, i + 2, 1) == "#")))) {
      o = o c; onc = onc c; i++; continue
    }

    o = o c; onc = onc c; i++
  }
  RES = o
  RESNC = onc
}

# Region boundary: stop at the test fence. This uses `nextfile`, NOT `exit`.
# `exit` at the boundary terminates the whole run, which skips ENDFILE for that
# file, so an empty region was never detected -- and a test fence on line 1 is
# the ordinary shape of a file that has no production code at all, so the abort
# this script advertises fired only for a zero-byte file and not for the case it
# was written for (rule 22/28/34 family: the control must exercise the same path
# the measurement does).
# No `pastfence` rule is needed: `nextfile` moves to the next input file, so the
# remaining lines of this one are never re-entered. `pastfence` is kept only as
# the FNR==1 reset marker.
{
  strip($0)

  # `[^{]` rather than `\b`: this host's gawk (5.3.0) does not support the `\b`
  # word boundary in a regex -- it silently matches nothing, which would leave
  # the boundary dead and the region the WHOLE file. That is the same
  # fail-open class as the `exit`/`ENDFILE` defect above, one rule down, and it
  # is recorded because `\b` reads as obviously correct (rule 14x's lesson: an
  # instrument's own convenience syntax can be the thing that disables it).
  #
  # Tested against RESNC, not the raw line: a string literal reading
  # `mod tests {` must not be able to move the boundary (rule 14aq).
  if (RESNC ~ /^[[:space:]]*mod tests[^{]/) { nextfile }

  # RES keeps string CONTENT -- a hard-coded clue IS a string literal, so
  # blanking it here would be the defect, not the fix.
  print RES
  # Count a line as production CODE only if the stripped projection is not
  # blank (pass 283). `emitted++` unconditionally counted a line whose entire
  # content was a comment, because strip() reduces such a line to "". A file
  # with no code at all therefore emitted one empty line, `emitted` was 1, and
  # the abort below never fired -- the exact "non-empty region, vacuous match
  # count" shape pass 281 flagged and pass 281/282 could not construct from
  # outside. Measured this pass: a comment-only file reads rc=0 with empty
  # stderr, a test-fence-on-line-1 file correctly aborts rc=2, and the two
  # disagreed, so the guard was not doing the job its comment claims.
  if (RES ~ /[^[:space:]]/) emitted++
}

ENDFILE {
  # FILENAME, not thisfile: a zero-byte file never reaches an FNR==1 rule, so
  # thisfile is unset for exactly the case most worth naming in the message.
  if (emitted == 0) {
    printf("fence.awk: %s produced an EMPTY production region -- aborting rather than emitting a clean 0\n", FILENAME) > "/dev/stderr"
    abort = 1
  }
}

# The region rule leaves via `exit` at the test boundary, which would otherwise
# discard the abort status, so the failure is carried out through END instead.
END {
  if (abort) exit 2
}
