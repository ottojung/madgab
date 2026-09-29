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
# substitution, which strips trailing blank lines and under-reads by 33 on
# lib.rs): adjacency.rs 269, lexical.rs 260, approx.rs 464, lib.rs 4,242,
# wasm.rs 67, main.rs 269. lib.rs was 380 under the superseded boundary; the
# change is the blind spot closing. The other five moved by +1 each, because
# the old boundary sat on the `#[cfg(test)]` line above `mod tests` and the
# corrected one sits on the `mod tests` line itself, so the attribute line is
# now inside the region. Pass 213/214 published 268/259/463/380/67/269; those
# five figures are superseded, not contradicted.

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

  # `[^{]` rather than `\b`: this host's gawk (5.3.0) does not support the `\b`
  # word boundary in a regex -- it silently matches nothing, which would leave
  # the boundary dead and the region the WHOLE file. That is the same
  # fail-open class as the `exit`/`ENDFILE` defect above, one rule down, and it
  # is recorded because `\b` reads as obviously correct (rule 14x's lesson: an
  # instrument's own convenience syntax can be the thing that disables it).
  if (line ~ /^[[:space:]]*mod tests[^{]/) { nextfile }

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
