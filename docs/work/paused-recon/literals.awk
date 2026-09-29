#!/usr/bin/awk -f
# literals.awk -- PASS 311. Emit each statement's string-literal content, so a
# canonical clue assembled from ADJACENT LITERALS can be detected.
#
# WHY THIS FILE EXISTS. `fence.awk` (rule 14x) strips comments and the test
# module but PRESERVES string content verbatim, which is right. The detection
# half in `clue-fence.sh` then ran `grep` over that region, and `grep` is
# LINE-SCOPED: it cannot match across a newline. Rust concatenates adjacent
# string literals, so a source file that respects a line-length limit writes
#
#     const CLUE: &str = "Hi"
#         "ts J"
#         ...
#         "ame";
#
# and a fence whose unit of detection is the line sees no clue word at all.
# Measured at pass 310: such a plant reads 0/0 under BOTH the joined and the
# per-word form, so the per-word form is NOT the independent backstop pass 309
# described -- it is line-scoped too. The information is present in the region
# (fence.awk kept it); only the per-line measurement discards it.
#
# So this pass changes the MEASUREMENT, not any file under src/. Nothing in the
# accepted implementation is touched.
#
# TWO FORMS PER STATEMENT, because a split can land in two different places:
#
#   EXACT   literals concatenated with NO separator -- the way the compiler
#           concatenates them -- so a MID-WORD split reassembles.
#   SPACED  the same literals joined by one space, so a split at a WORD
#           boundary also reassembles. Rust would not insert that space, but the
#           pattern is whitespace-flexible and a hard-code written either way is
#           the same hard-code, so both are emitted and the caller matches the
#           phrase with [[:space:]]+ between words.
#
# The forms are emitted ALTERNATELY on consecutive lines, so a caller's
# `-c` count is over statements twice over. The caller asserts the count is
# never equal to the emitted line count (rule 292's fail-open signature) and
# that the emitted line count is non-zero (a scanner that matches nothing
# reports a clean 0, which is the pass-286/pass-298 failure mode).
#
# LIMITS, STATED RATHER THAN IMPLIED. The scanner is not a Rust parser:
#   - `\"` inside a literal is not understood, so a literal containing an
#     escaped quote would end the scan early. The caller ASSERTS that no
#     production region contains one, so the limitation is unreachable here
#     rather than silently load-bearing.
#   - raw strings (`r#"..."#`) are not understood either, and the caller
#     asserts their absence the same way.
#   - a statement is delimited by `;`. A literal inside a block that ends
#     without one (a `const` inside a macro expansion, say) is emitted at
#     END, which under-detects rather than over-detects.
# These are the fail-safe directions: a limitation here can only make a hit
# disappear, never invent one, and the three controls in clue-fence.sh cover
# the two splits that matter.
{
  line = $0
  rest = line
  while (match(rest, /"[^"]*"/)) {
    lit = substr(rest, RSTART + 1, RLENGTH - 2)
    exact  = exact lit
    spaced = spaced " " lit
    rest   = substr(rest, RSTART + RLENGTH)
  }
  if (index(line, ";") > 0) {
    if (exact != "") { print exact; print spaced }
    exact = ""; spaced = ""
  }
}
END { if (exact != "") { print exact; print spaced } }
