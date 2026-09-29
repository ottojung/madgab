# Fence matcher: the no-hard-coding alphabet, as a FILE rather than as memory.
#
# WHY THIS FILE EXISTS (rule 14ak). fence.awk is the region stripper; it holds no
# matcher, so the matcher -- and therefore the ALPHABET -- was re-typed by each pass
# from its own recollection of the canonical examples. Rules 14t, 14u and 14v are
# three separate discoveries that this is unsafe:
#
#   14t  a fence that only searches for the ANSWER cannot detect a hard-coded
#        QUESTION -- `wreck a nice beach` was invisible to a clue-words-only fence;
#   14u  the canonical clue is stored DECOMPOSED as ["hits","justice","dupe",
#        "hid","came"], so the joined literal cannot match the form a developer
#        is most likely to write;
#   14v  a fence alphabet enumerated FROM RECALL reproduces exactly the blind spot
#        the fence exists to close, because the thing you forgot is by definition
#        not in the list you checked.
#
# All three are one defect: the alphabet lived in prose, so it was re-derived rather
# than quoted. This file makes it quotable. The derivation below is READ FROM the
# property document, never from the log's memory of it (rule 14v).
#
# SOURCE OF THE ALPHABET -- docs/accepted-state-2026-09-27.md:
#   line 25  `recognize speech` -> `wreck a nice beach`
#   line 31  `It's just a stupid game` -> `Hits Justice Dupe Hid Came`
# Cross-checked against docs/continuation-approximate-search.md lines 39-40, which
# name the same two pairs with the clue lower-cased. Four canonical strings across
# two examples, plus the decomposed storage spelling. `wreck a nice beach` is the
# ANSWER to example 1 and is exactly as canonical as either target; omitting it is
# the defect rule 14v was written about.
#
# USAGE -- the two sanctioned spellings, both required:
#
#   # (a) phrase form: every canonical string in its joined spelling, either side
#   #     of either example, matched CASE-INSENSITIVELY so the Hits-Justice
#   #     capitalisation is not a third blind spot (rule 14v).
#   gawk -f docs/work/paused-recon/fence-alphabet.awk
#
#   # (b) decomposed form: each word alone, so the array spelling is not invisible.
#   #     The variable is `decomposed`, so it must be an ASSIGNMENT BEFORE -f.
#   #     The `--decomposed` spelling published here until pass 233 does not set it:
#   #     gawk accepts the token, exits 0 and prints ARM (a)'s alphabet, so arm (b)
#   #     was unrunnable as documented and failed SILENTLY toward the weaker regex
#   #     (measured 2026-09-29, pass 233: exit 0, empty stderr, phrase regex out).
#   gawk -v decomposed=1 -f docs/work/paused-recon/fence-alphabet.awk
#
#   # THIS FILE PRINTS THE ALPHABET AND READS NO INPUT. Both sanctioned forms must
#   #   be CAPTURED, then applied by the caller to the region fence.awk produced:
#   PHRASE=$(gawk -f docs/work/paused-recon/fence-alphabet.awk)
#   DECOMP=$(gawk -v decomposed=1 -f docs/work/paused-recon/fence-alphabet.awk)
#   for f in src/adjacency.rs src/lexical.rs src/approx.rs src/lib.rs src/wasm.rs src/main.rs; do
#     gawk -f docs/work/paused-recon/fence.awk "$f" > /tmp/region.txt || continue
#     printf '%s region=%s phrase=%s decomp=%s\n' "$f" \
#       "$(wc -l < /tmp/region.txt)" \
#       "$(grep -ciE "$PHRASE" /tmp/region.txt)" \
#       "$(grep -ciE "$DECOMP" /tmp/region.txt)"
#   done
#
#   # The region count is `$(wc -l < FILE)`, NOT `$(fence.awk FILE | wc -l)`.
#   # `$( ... )` strips one trailing newline and the LAST line of every production
#   # region is non-blank, so the command-substitution form under-reads by exactly
#   # 1 in all six files. fence.awk's own header calls this out and publishes
#   # 269/260/464/4242/67/269; this file's usage block repeated the bad spelling
#   # for the whole time the header warned about it, so a pass copying the usage
#   # block would land on 268/259/463/4241/66/268 while the header says otherwise
#   # (measured 2026-09-29, pass 251). It also fails OPEN: a non-zero fence.awk
#   # exit yields an empty capture that `wc -l` still reports as 0, which is a
#   # clean-looking fence reading on an aborted region.
#   # Piping the region INTO this file (as pass 232's next action instructed) instead
#   #   of capturing its output and matching yourself yields the one line of regex
#   #   TEXT, which any counter reads as a constant 1 in all six files. Measured
#   #   2026-09-29 (pass 233): phrase=1 decomp=1 in all six regions under that
#   #   pipeline, against 0 / 0-0-0-1-0-0 under the form above.
#
# THE INVARIANT is that BOTH return 0 in the production region of all six
# production files, measured through fence.awk. The one known non-zero reading is
# `src/lib.rs:3597` `.expect("key came from cells")` under (b) -- the ordinary
# English past tense in a panic message, adjudicated benign at pass 216, do not
# re-open. A `1` anywhere else is a finding.
#
# CONTROL, required before publishing any 0 (rules 14t/14v). A fence that returns
# 0 for a planted instance of the thing it forbids is not a fence. For EVERY string
# this file names, plant it in a production-region copy and require non-zero -- not
# one representative literal. Measured 2026-09-29 (pass 232): all five spellings
# read 1, i.e. the fence discriminates rather than returning a constant.
#
# THE PLANT MUST BE CODE, NOT A COMMENT (measured 2026-09-29, pass 233). fence.awk
# strips comments by design, so a `// wreck a nice beach` plant reads 0 in BOTH arms
# and looks like a broken fence, while a string-literal or array-literal plant of the
# same text at the same line inside the region reads non-zero. Pass 232 published
# "planted at lib.rs line 300 reads 1" without recording which form it planted. At
# line 300 of a production-region copy: `const _: &str = "wreck a nice beach";`
# -> phrase 1, decomp 1; the same literal as `// ...` -> phrase 0, decomp 0. A
# decomposed array `["hits","justice","dupe","hid","came"]` reads phrase 0, decomp
# non-zero -- the pair of arms is the point (rules 14t/14u).
#
# This file PRINTS the alphabet; it neither reads input nor matches, and it does not
# decide. Apply its output to fence.awk's region with your own counter and keep the
# regex public in whatever you publish (rule 25: a count carries its population, so
# publish the matcher with the number).

BEGIN {
  if (decomposed) {
    # Word level. `came` alone is unavoidably broad -- it is the only word here
    # that occurs in ordinary English in this codebase -- which is exactly why
    # (b) is reported beside (a) and never instead of it.
    print "(hits|justice|dupe|hid|came|wreck|beach|recognize|speech|stupid|game)"
  } else {
    # Phrase level, either side of either example, case-insensitive.
    # The apostrophe in "It's" is matched as `.` -- but that covers a ONE-BYTE
    # apostrophe only, and pass 253 measured that a U+2019 typographic apostrophe
    # (three bytes: 342 200 231) is NOT covered. Cause: this host runs with LANG
    # and LC_ALL both EMPTY, so grep and gawk operate in the C locale, where `.`
    # matches a single BYTE; a three-byte character is three unmatched positions.
    # So the comment here used to assert a guarantee the regex did not make, and
    # the same commit's control (which planted only the ASCII apostrophe) read 1
    # and appeared to confirm it. The typographic alternative is therefore spelled
    # out as its own literal branch rather than folded into `.`.
    print "(wreck a nice beach|hits justice dupe hid came|recognize speech|it.s just a stupid game|it’s just a stupid game)"
  }
}
