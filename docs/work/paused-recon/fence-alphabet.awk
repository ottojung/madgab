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
#   gawk -f docs/work/paused-recon/fence-alphabet.awk --decomposed
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
# This file matches; it does not decide. Pipe its output through your own counter
# and keep the regex public in whatever you publish (rule 25: a count carries its
# population, so publish the matcher with the number).

BEGIN {
  if (decomposed) {
    # Word level. `came` alone is unavoidably broad -- it is the only word here
    # that occurs in ordinary English in this codebase -- which is exactly why
    # (b) is reported beside (a) and never instead of it.
    print "(hits|justice|dupe|hid|came|wreck|beach|recognize|speech|stupid|game)"
  } else {
    # Phrase level, either side of either example, case-insensitive.
    # The apostrophe in "It's" is matched as `.` so a typographic variant
    # (U+2019) cannot open a spelling gap.
    print "(wreck a nice beach|hits justice dupe hid came|recognize speech|it.s just a stupid game)"
  }
}
