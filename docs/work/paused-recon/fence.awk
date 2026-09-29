# Fence: hard-coded canonical-phrase detection in the PRODUCTION region.
#
# Region (rule 14n): src/*.rs, everything above the first #[cfg(test)]
# boundary, with comments stripped (// line and /* */ block, the latter
# tracked across lines). A file with no #[cfg(test)] boundary is therefore
# measured over its whole file -- not exempt.
#
# Alphabet (rule 14v): derived by READING the document that defines the
# property (docs/accepted-state-2026-09-27.md lines 25 and 31), not recalled.
# Both sides of both canonical examples, both spellings of the clue
# (contiguous and decomposed), matched case-insensitively.
#
# This script exits 2 on any read error rather than passing an empty string
# to the matcher: a stage that fails open produces a zero indistinguishable
# from a passing result (rule 22/28/34 family).

BEGIN {
  inblock = 0
}

{
  line = $0

  # Region boundary: stop at the test fence.
  if (line ~ /^[[:space:]]*#\[cfg\(test\)\]/) { exit }

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
}
