# Fence: hard-coded canonical-phrase detection in the PRODUCTION region.
#
# Region (rule 14n): src/*.rs, everything above the first #[cfg(test)]
# boundary, with comments stripped (// line and /* */ block, the latter
# tracked across lines). A file with no #[cfg(test)] boundary is therefore
# measured over its whole file -- not exempt.
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
# Region-line counts to expect on the six production files, measured
# 2026-09-29 (each with `wc -l` on this script's stdout -- NOT via command
# substitution, which strips trailing blank lines and under-reads by 33 on
# lib.rs): adjacency.rs 268, lexical.rs 259, approx.rs 463, lib.rs 380,
# wasm.rs 67, main.rs 269.

BEGIN {
  inblock = 0
  thisfile = ""
}

FNR == 1 {
  thisfile = FILENAME
  emitted = 0
  inblock = 0
  pastfence = 0
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
  line = $0

  if (line ~ /^[[:space:]]*#\[cfg\(test\)\]/) { nextfile }

  if (inblock) {
    p = index(line, "*/")
    if (p == 0) next
    line = substr(line, p + 2)
    inblock = 0
  }

  out = ""
  rest = line
  while (1) {
    a = index(rest, "//")
    b = index(rest, "/*")
    c = index(rest, "*/")

    if (b > 0 && (a == 0 || b < a)) {
      out = out substr(rest, 1, b - 1)
      d = index(substr(rest, b), "*/")
      if (d == 0) { inblock = 1; rest = ""; break }
      rest = substr(substr(rest, b), d + 2)
      continue
    }

    if (a > 0) { out = out substr(rest, 1, a - 1) }
    else       { out = out rest }

    if (c > 0) { inblock = 1 }
    rest = ""
    break
  }

  print out
  emitted++
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
