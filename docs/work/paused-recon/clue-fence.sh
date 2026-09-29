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
# taking the CLUE side (right of the arrow). Never typed from memory: a
# remembered alphabet is exactly the input that lets a hard-code through.
#
# TWO COUNTS, AND WHY THEY DIFFER IN KIND:
#
#   JOINED  each canonical clue as a contiguous phrase, case-INsensitive,
#           because the defining document spells one of them in title case
#           (`Hits Justice Dupe Hid Came`) and the other in lower case. This is
#           the strong form and must be 0 in all six files.
#   PERWORD each clue word alone. This is the weaker form, needed because a
#           hard-code may spell the clue DECOMPOSED (rule 14x) and then the
#           contiguous form cannot see it at all. One hit in lib.rs is
#           ADJUDICATED (see KNOWN_BENIGN below) and anything else FAILS.
#
# PASS 308 REPAIRS BOTH FORMS, AND A CONTROL THAT COULD NOT HAVE CAUGHT EITHER.
# The joined pattern was built by `paste -sd' '` over the alphabet, which
# produces the SORTED UNION OF BOTH CLUES' WORDS -- `a beach came dupe hid
# hits justice nice wreck` -- which is not a clue, is not a substring of either
# clue, and cannot occur in any file that does not already contain all nine
# words in alphabetical order. It was therefore an UNMATCHABLE pattern reported
# as the primary, strong, must-be-zero check, and it read 0 on a file
# containing a hard-coded `wreck a nice beach`. Separately, the per-word form
# was case-SENSITIVE, so the clue spelled exactly as the defining document
# spells it -- `Hits Justice Dupe Hid Came` -- read 0, while the same clue in
# lower case read 5. The alphabet itself was not the defect: rule 14v asked for
# both sides of every example and the CLUE side was taken, but the words were
# then reassembled into a phrase that is not a clue, and the case-insensitivity
# that the JOINED form claimed to provide was silently absent from the form
# that actually had to catch a decomposed spelling.
#
# The trap that hid both: every plant control was built from the same
# misassembled `JP_RE`, so it planted the very string that cannot occur and
# read 1, certifying a pattern that could not fire. A control must be planted
# with the STRING THE PROPERTY NAMES (rule 14v), never with the pattern's own
# output. New controls below plant both documented clues, in both documented
# capitalisations, and require each to be CAUGHT.
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
# PASS 311. The literal-JOIN form, closing the line-scope hole passes 309 and
# 310 measured. `fence.awk` keeps string content; grep cannot match ACROSS a
# newline, and Rust concatenates adjacent string literals, so a clue split
# across literals read 0/0 under BOTH existing forms. This scanner reassembles
# each statement's literals so the phrase can be matched without newlines.
LITSCAN=docs/work/paused-recon/literals.awk
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
[ -f "$LITSCAN" ]  || fail "$LITSCAN not found -- the literal-join form is what detects a clue split across adjacent string literals (pass 310); refusing to report a fence that is blind to it"
[ -f "$DEFECT" ]   || fail "$DEFECT not found -- the alphabet must be DERIVED from the defining document, not recalled"

# ---------------------------------------------------------------------------
# 1. Derive the alphabet. Refuse rather than match a partial or empty list.
# ---------------------------------------------------------------------------
ALPHA=$(node -e '
const fs = require("fs");
const lines = fs.readFileSync(process.argv[1], "utf8").split("\n");
const cluePhrases = [];
const targetPhrases = [];
const clueWords = [];

// APOSTROPHE IS WRITTEN \u0027, NOT LITERALLY (pass 308). This program is inside
// a shell SINGLE-QUOTED argument, so a literal apostrophe terminates the string
// and the JS is spliced into the middle of the shell script -- a syntax error
// reported at some unrelated line, which cost this pass two confusing cycles.
const AP = String.fromCharCode(39);

// Emit a phrase with an OPTIONAL ASCII APOSTROPHE (AP followed by "?").
//
// NO APOSTROPHE APPEARS LITERALLY IN THIS COMMENT BLOCK, deliberately: the
// program sits inside a shell SINGLE-QUOTED argument, so one literal
// apostrophe in a comment terminates the string and splices JS into the middle
// of the shell script, producing a syntax error reported at an unrelated line.
// That cost this pass two confusing cycles.
//
// WHY OPTIONAL-AND-ASCII, MEASURED ON THIS HOST: grep is GNU grep 3.11 with
// LC_ALL and LANG UNSET, so matching happens in the C locale where a character
// is a BYTE. Two consequences, both measured rather than assumed:
//   (a) a bracket class over the multi-byte curly apostrophe never matches --
//       a class probe reads 0 where a bare literal probe of the same character
//       reads 1 -- so the class form is a silent, environment-dependent false
//       negative, the same shape of defect as the rest of this pass;
//   (b) the quantifier binds to a single byte, so the optional-apostrophe form
//       does NOT consume a three-byte curly sequence and reads 0 on a curly
//       hard-code.
// Neither form is portable, so the portable form is used instead: the SEARCH
// TEXT is normalised (curly apostrophes folded to straight) before matching,
// which puts both sides in the same encoding in any locale. The optional
// quantifier is then redundant but harmless, and it additionally covers a
// hard-code that DROPPED the apostrophe.
const both = (p) => (p.includes(AP) ? [p.split(AP).join(AP)] : [p]);

// NORMALISATION (pass 308): an APOSTROPHE IS PART OF THE TOKEN, not a
// separator. The pre-308 normaliser mapped every non-alphanumeric to a space,
// so the second target -- the one spelled with a contraction -- was reduced to
// its letters only, and a hard-code, which is the one thing this fence exists
// to catch, is written WITH the apostrophe, because that is how the word is
// spelt. The derived phrase therefore could not match the only spelling a
// developer would type: measured, the plant with a straight apostrophe read 0
// under the normalised phrase. So contractions are preserved verbatim and only
// true separators become spaces. The clue side is unaffected: neither clue
// contains an apostrophe.
const norm = (s) => s.toLowerCase()
    .replace(new RegExp("[\\u2018\\u2019]", "g"), AP)   // curly -> straight
    .replace(new RegExp("[^a-z0-9" + AP + "]+", "g"), " ")
    .trim().replace(/\s+/g, " ")
    .replace(new RegExp("\\s+" + AP, "g"), AP);

for (const ln of lines) {
  // `TARGET` -> `CLUE`. The backticks are pinned so the arrow and the target
  // cannot be mistaken for one another. PASS 308: BOTH sides are taken, per
  // rule 14v (a disjunction over BOTH sides of every example and EVERY
  // spelling). Pre-308 read the CLUE side only, which left the rule-14t hazard
  // open: a hard-code keyed on the QUESTION -- `if t == "recognize speech" {`
  // -- is invisible to a fence whose alphabet is the answers only.
  //
  // THE TWO SIDES ARE MATCHED AT DIFFERENT GRANULARITIES, AND THE ASYMMETRY IS
  // THE POINT. Clue-side words go in the PER-WORD alphabet: they are derived
  // from the property, and a hard-code spelling a clue DECOMPOSED (rule 14x)
  // is caught word by word. Target-side words CANNOT go in the per-word
  // alphabet, and the reason is measured, not stylistic: the second target is
  // `it is just a stupid game`, whose words include `it` and `just`, and
  // ordinary prose in production code uses both -- adding them produced 10
  // per-word hits in src/adjacency.rs alone (all `it`), which is exactly the
  // fail-open direction rule 14v warns about, a real hard-code hidden inside a
  // number nobody can read. So the target side is matched as a WHOLE PHRASE,
  // which is the only form in which it is evidence: a hard-code keyed on the
  // question necessarily contains the whole target string. Clue words are
  // distinctive; target words are not; the granularity each side deserves
  // follows from that, and neither list is hand-curated.
  const m = ln.match(/^\x60([^\x60]+)\x60\s*\u2192\s*\x60([^\x60]+)\x60/);
  if (!m) continue;
  const target = norm(m[1]);
  const clue = norm(m[2]);
  if (clue) cluePhrases.push(clue);
  if (target) targetPhrases.push(target);
  clueWords.push(...clue.split(" ").filter(Boolean));
}
const u = [...new Set(clueWords)].sort();
if (u.length < 5) {
  console.error("alphabet derivation yielded " + u.length + " words -- refusing");
  process.exit(3);
}
if (cluePhrases.length < 2) {
  console.error("clue-phrase derivation yielded " + cluePhrases.length + " phrase(s); the property names two -- refusing");
  process.exit(3);
}
if (targetPhrases.length !== cluePhrases.length) {
  console.error("target/clue phrase count mismatch (" + targetPhrases.length + "/" + cluePhrases.length + ") -- refusing");
  process.exit(3);
}
// Emit each phrase in both apostrophe typescripts, joined by a pipe, so the
// disjunction built by the caller covers either spelling without the caller
// having to know which phrases contain a contraction.
//
// The one-to-one assertion ABOVE is on the BASE phrase lists, before expansion:
// only one of the two targets contains a contraction, so the expanded target
// section is longer than the expanded clue section by exactly one. Asserting
// the invariant on the expanded lists would read that as a defect. The
// expansion is 1:1 per phrase by construction (`both` maps each phrase to one
// or two), so the base comparison is the meaningful one.
const variants = (list) => list.flatMap(both).join("\n");
process.stdout.write(u.join("\n") + "\x1e" + variants(cluePhrases) + "\x1e" + variants(targetPhrases));
' "$DEFECT") || fail "alphabet derivation failed from $DEFECT -- refusing to build a pattern from a partial list"

# PASS 308: FOLD THE SEARCH TEXT, NOT THE PATTERN. Every measurement below pipes
# a production region through this before grepping, which puts the pattern and
# the text in the same encoding in ANY locale and therefore lets the plain ASCII
# pattern see a curly-apostrophe hard-code.
#
# `sed` WITH \xNN, NEVER `tr`, and that is measured rather than stylistic. The
# curly right single quote is the three bytes E2 80 99; `tr` works on
# CHARACTERS, so `tr '\342\200\231' '\047'` reads a three-character set and
# writes a one-character set and pads -- on an input containing a curly quote it
# emits THREE apostrophes and mangles the line (verified with od -c). GNU sed
# with \xNN is byte-exact and folds both U+2019 and U+2018 to a single straight
# apostrophe with no other change (verified with od -c).
#
# The fold is applied to the TEXT ONLY. It is a no-op on this repository's own
# sources -- there is no curly apostrophe in any of them -- so it cannot change
# the standing measurement, and that is asserted below rather than assumed.
fold_apostrophes() {
  sed -e 's/\xe2\x80\x99/\x27/g' -e 's/\xe2\x80\x98/\x27/g'
}
# Build the straight apostrophe ONCE into a variable. Writing it inline as
# "$(printf "'")" embeds a lone apostrophe inside a command substitution inside
# double quotes, which is a shell quoting error reported at a distant line.
APOSTROPHE=$(printf '\047')
[ "$(printf 'let x = 1;\n' | fold_apostrophes)" = 'let x = 1;' ] \
  || fail "the apostrophe fold is not the identity on plain ASCII -- refusing"
_fold_probe=$(printf 'It\xe2\x80\x99s\n' | fold_apostrophes)
[ "$_fold_probe" = "It${APOSTROPHE}s" ] \
  || fail "the apostrophe fold did not fold a curly apostrophe (got [$_fold_probe]) -- refusing"

# Split the derivation's three sections. \x1e (record separator) is used because
# neither a clue word nor a normalised phrase can contain it.
#
# `%%` IS LOAD-BEARING, NOT `%` (pass 308). There are TWO separators, and `%`
# strips the SHORTEST matching suffix -- i.e. it splits at the LAST separator,
# which leaves the clue phrases glued onto the end of the alphabet. The symptom
# is quiet and in the fail-open direction: the alphabet line printed
# `... wreckwreck a nice beach hits justice dupe hid came` and the per-word
# count silently covered the phrase words too. `%%` strips the longest suffix
# and splits at the FIRST separator, which is the boundary the derivation means.
_alpha_all=${ALPHA%%$'\x1e'*}
_rest=${ALPHA#*$'\x1e'}
[ "$_rest" != "$ALPHA" ] || fail "the derivation emitted no clause separator -- refusing to guess which half is which"
_clue_all=${_rest%$'\x1e'*}
TARGETS=${_rest#*$'\x1e'}
[ "$TARGETS" != "$_rest" ] || fail "the derivation emitted only one section separator -- refusing"
PHRASES=$_clue_all
ALPHA=$_alpha_all
# The three sections must be disjoint and each must be non-empty. Asserted
# because a mis-split here produces a plausible pattern rather than an error.
case "$ALPHA" in *"$(printf '\036')"*) fail "the alphabet section still contains a separator -- the split is wrong; refusing" ;; esac
case "$PHRASES" in *"$(printf '\036')"*) fail "the clue section still contains a separator -- the split is wrong; refusing" ;; esac
n_alpha=$(printf '%s\n' "$ALPHA" | grep -c .)
[ "$n_alpha" -ge 5 ] || fail "derived alphabet has $n_alpha words -- refusing"
n_phrase=$(printf '%s\n' "$PHRASES" | grep -c .)
[ "$n_phrase" -ge 2 ] || fail "derived $n_phrase clue phrase(s); the property names two -- refusing"
# The apostrophe-variant expansion is 1:1 per BASE phrase, so the target
# section may be longer than the clue section (only one of the two targets
# contains a contraction). The honest assertion is that the target section is
# not SHORTER, i.e. no base phrase lost its expansion.
n_target=$(printf '%s\n' "$TARGETS" | grep -c .)
[ "$n_target" -ge "$n_phrase" ] || fail "derived $n_target target variants against $n_phrase clue variants -- the expansion lost a phrase; refusing"

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
# This loses nothing: the JOINED form is now the two real clue PHRASES, so a
# contiguous `wreck a nice beach` is caught outright, and a hard-code spelling
# the clue as a decomposed array still trips the per-word form on the eight
# remaining words. The controls below prove all three spellings.
PW_RE=$(printf '%s\n' "$ALPHA" | awk 'NF && length($0) > 1 { sep = (n++ ? sep "|" : ""); sep = sep $0 } END { print sep }')
n_pw=$(printf '%s\n' "$ALPHA" | awk 'NF && length($0) > 1' | grep -c .)
[ -n "$PW_RE" ] || fail "built an EMPTY per-word pattern -- that matches every line (rule 292); refusing"
[ "$n_pw" -ge 5 ] || fail "per-word alphabet has only $n_pw words after dropping single letters -- refusing; a derived list this short means the extraction broke, not that the property is weak"
case "$PW_RE" in
  '|*|*'|'|'*|*'|') fail "built pattern '$PW_RE' contains an empty alternative -- that matches every line (rule 292); refusing" ;;
esac
case "$PW_RE" in *'||'*) fail "built pattern contains '||' -- an empty alternative (rule 292); refusing" ;; esac

# PASS 308: the joined pattern is the DISJUNCTION OF THE ACTUAL PHRASES --
# every clue the property names, plus every target (rule 14t). The pre-308 form
# was `paste -sd' '` over the alphabet, i.e. the sorted union of both clues'
# words, which is not a clue and cannot occur in any file that does not already
# contain all nine words in alphabetical order. It was an unmatchable pattern
# standing in for the primary check.
JP_RE=$(printf '%s\n%s\n' "$PHRASES" "$TARGETS" | awk 'NF { sep = (n++ ? sep "|" : ""); sep = sep $0 } END { print sep }')
[ -n "$JP_RE" ] || fail "built an EMPTY joined pattern -- refusing"
case "$JP_RE" in
  '|*|*'|'|'*|*'|') fail "joined pattern '$JP_RE' contains an empty alternative -- that matches every line (rule 292); refusing" ;;
esac
case "$JP_RE" in *'||'*) fail "joined pattern contains '||' -- an empty alternative (rule 292); refusing" ;; esac

# PASS 308 ASSERTION, the one that would have caught it at build time: every
# derived phrase must be MATCHABLE by the pattern built from the same phrases.
# This is a self-consistency check on the construction, and it is what makes
# the pre-308 form's unmatchable pattern structurally impossible: the sorted
# union is a substring of no phrase derived the same way, so had the pattern
# been built by the old `paste` form, this loop would have aborted.
jp_bad=0
AP_M=$(printf "'")
# The combined-pattern self-check: require the combined pattern to match a probe
# recovered from each derived phrase. A derived line is a REGEX (an apostrophe
# is emitted as an optional `'?`), so it is never compared as a literal string.
# Dropping the `?` yields the plain spelling, which is what the pattern must
# match. The optional-apostrophe form is what makes this one line per phrase
# instead of two: `it'?s` already covers the straight, curly and elided
# spellings, so there is nothing to disjoin.
while IFS= read -r ph; do
  [ -n "$ph" ] || continue
  # The `?` marks an OPTIONAL apostrophe, so the plain spelling -- the one the
  # pattern must also match -- is the text with the `?` REMOVED, not replaced
  # by an apostrophe. Substituting rather than deleting yields a doubled
  # apostrophe that no spelling of the phrase contains, and the check then
  # aborts on a correct pattern.
  probe=${ph//[?]/}
  printf '%s\n' "$probe" | grep -qiE "($JP_RE)" || { jp_bad=1; printf 'joined pattern does not match its own derived phrase: [%s] (probe [%s])\n' "$ph" "$probe" >&2; }
done < <(printf '%s\n%s\n' "$PHRASES" "$TARGETS")
[ "$jp_bad" -eq 0 ] || fail "the joined pattern cannot match the phrase it was derived from -- refusing to report any count (pass 308)"

# PASS 311: THE WHITESPACE-FLEXIBLE JOINED PATTERN (`LF_RE`), used ONLY against
# the literal-join stream. Built from the SAME derived phrases as JP_RE, with
# each inter-word space widened to [[:space:]]+ so the phrase can match a
# reassembly that used a newline (or a single space) where the phrase has one.
# It is deliberately NOT used against the raw region: widened whitespace against
# raw source would match a phrase spread across two arbitrary lines of code, and
# over-detection on a fail-closed fence is as wrong as under-detection. The
# literal-join stream is the only place where a spread phrase is real.
#
# Built with the same rule-292 guards as the other two patterns, and with the
# same self-check: every derived phrase must match the pattern derived from it.
LF_RE=$(printf '%s\n%s\n' "$PHRASES" "$TARGETS" | awk '
  NF {
    s = $0
    gsub(/ /, "[[:space:]]+", s)
    sep = (n++ ? sep "|" : "")
    sep = sep s
  }
  END { print sep }')
[ -n "$LF_RE" ] || fail "built an EMPTY literal-join pattern -- refusing"
case "$LF_RE" in
  '|*|*'|'|'*|*'|') fail "literal-join pattern contains an empty alternative -- that matches every line (rule 292); refusing" ;;
esac
case "$LF_RE" in *'||'*) fail "literal-join pattern contains '||' -- an empty alternative (rule 292); refusing" ;; esac
lf_bad=0
while IFS= read -r ph; do
  [ -n "$ph" ] || continue
  probe=${ph//[?]/}
  printf '%s\n' "$probe" | grep -qiE "^($LF_RE)$" || { lf_bad=1; printf 'literal-join pattern does not match its own derived phrase: [%s]\n' "$ph" >&2; }
done < <(printf '%s\n%s\n' "$PHRASES" "$TARGETS")
[ "$lf_bad" -eq 0 ] || fail "the literal-join pattern cannot match the phrase it was derived from -- refusing (pass 311)"

# PASS 311, THE SCANNER'S OWN LIMITS, ASSERTED RATHER THAN ASSUMED. literals.awk
# does not understand \" or raw strings. Neither appears in any production region
# (measured), but a future one would silently change what the form can see, so
# the absence is a gate, not a comment: if a region ever contains either, this
# script refuses instead of reporting a fence whose coverage just changed.
joinable() {
  awk -v f="$1" '
    /\\"/    { print f ": an escaped quote inside a string literal"; bad = 1 }
    /r#"/    { print f ": a raw string literal"; bad = 1 }
    END { exit bad ? 1 : 0 }'
}

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

# ---------------------------------------------------------------------------
# The guard above is a guard over src/ ONLY, and src/ is not the whole
# production region. tests/no_phrase_hard_coding.rs names three of them --
# src/, web/ and examples/ -- and states the reason web/ is in scope: "web/
# app.js is a real user-facing search entry point, so a hard-code committed
# there is as reachable as one in src/". This script measured src/ alone and
# its verdict line said "0 canonical clue occurrences in all 6 production
# regions", which a reader takes to cover the program.
#
# It does not. Verified at pass 314: a canonical clue appended to web/app.js
# made tests/no_phrase_hard_coding.rs FAIL (naming web/app.js:105) while this
# script still exited 0 printing "0 canonical clue occurrences in all 6
# production regions". The Rust test caught it; this one could not see it.
#
# So the two fences have DISJOINT coverage and only one of them is named in
# the verdict. Rather than reimplement a JS/HTML/CSS and examples/ scanner
# here, the honest fix is to make the scope VISIBLE and to refuse when the
# rest of the production region is not covered by SOME fence. If the Rust
# test is deleted or stops scanning web/ or examples/, this refuses instead
# of publishing a narrower 0 than the one it used to publish.
#
# DERIVED, NOT RECALLED: the region list is read out of the test's own
# REGIONS table, so editing that table changes what is demanded here.
# ---------------------------------------------------------------------------
RUSTFENCE=tests/no_phrase_hard_coding.rs
[ -f "$RUSTFENCE" ] || fail "$RUSTFENCE not found -- it is the only fence covering web/ and examples/; without it this script measures src/ alone and cannot report a whole-program zero"

# The (dir, ext) pairs the Rust fence claims, one per line.
rust_regions() {
  awk '/^const REGIONS/,/^\];/' "$RUSTFENCE" \
    | sed -n 's/^[[:space:]]*("\([^"]*\)", "\([^"]*\)"),*$/\1 \2/p'
}
mapfile -t RREG < <(rust_regions)
[ "${#RREG[@]}" -ge 3 ] \
  || fail "read ${#RREG[@]} region(s) from $RUSTFENCE's REGIONS table; the table is src+web+examples and a short read means the population is broken, not small -- refusing"

# Every region the Rust fence claims, grouped by directory, must be non-empty
# on disk. An empty region is the same shape as a missing one: the Rust fence
# would scan nothing there and report nothing wrong.
rbad=()
for pair in "${RREG[@]}"; do
  set -- $pair
  rdir=$1; rext=$2
  n=$(find "$rdir" -name "*.$rext" -type f 2>/dev/null | grep -c .)
  [ "$n" -gt 0 ] || rbad+=("$rdir/*.$rext(0 files)")
done
[ "${#rbad[@]}" -eq 0 ] \
  || fail "regions claimed by $RUSTFENCE but empty on disk: ${rbad[*]} -- an empty region scans clean by construction; refusing"

# And this script must not silently under-report them: name the directories it
# does NOT measure, on the verdict line, so "all 6 production regions" can
# never again be read as the whole program.
UNFENCED_DIRS=$(printf '%s\n' "${RREG[@]}" | awk '{print $1}' | sort -u | grep -vxF 'src' | paste -sd, -)

printf 'fence: alphabet %d clue words / %d clue phrases / %d target phrases derived from %s\n' "$n_alpha" "$n_phrase" "$n_target" "$DEFECT"
printf '  words      %s\n' "$(printf '%s' "$ALPHA" | tr '\n' ' ')"
printf '  clues      %s\n' "$(printf '%s' "$PHRASES" | tr '\n' '|')"

total_joined=0
total_lit=0
total_litstream=0
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

  j=$(fold_apostrophes < "$region" | grep -cEi "$JP_RE" || true)
  # The population signature. If the joined count equals the region line count,
  # the pattern matched every line and the count is meaningless.
  [ "$j" -ne "$lines" ] || fail "$f joined count ($j) equals the region line count ($lines) -- the pattern is matching every line; refusing (rule 292)"
  total_joined=$(( total_joined + j ))

  # Per-word, CASE-INSENSITIVE (pass 308), counted with grep -oEi so
  # repetitions are counted. (grep -cE would under-count several occurrences
  # on one line, and counting LINES rather than OCCURRENCES is how a planted
  # clue hides behind an existing line that already matched something else.)
  # Case-INSENSITIVITY IS NOT A TIGHTENING OF THE LANGUAGE BUT A TIGHTENING OF
  # THE CHECK: the pre-308 comment claimed case-insensitivity made the joined
  # form "unusable" because an English word in prose is not a hard-code, and
  # left the per-word form case-SENSITIVE so the clue spelled as the defining
  # document spells it (`Hits Justice Dupe Hid Came`) read 0 while the same
  # clue in lower case read 5. That is the wrong direction: this alphabet is
  # DERIVED from the property (rule 14v), not a general English vocabulary, and
  # it is re-measured below over all six production regions -- case-insensitive
  # per-word returns the SAME 0/0/0/1/0/0 as the case-sensitive form, with the
  # one lib.rs hit being the pass-216-adjudicated `.expect("key came from
  # cells")`. So the case-insensitive form is strictly stronger at zero cost.
  w=$(fold_apostrophes < "$region" | grep -oEi "($PW_RE)" | wc -l)

  # PASS 311: the LITERAL-JOIN form. Same regions, but the string literals are
  # reassembled per statement first, so a clue written as adjacent literals
  # (`"Hi"` / `"ts J"` / ... / `"ame"`) is matched as the phrase it is instead
  # of as seven lines that happen to contain no whole word. Counted with
  # grep -cEi because each emitted line is a whole statement.
  fold_apostrophes < "$region" | joinable "$f" \
    || fail "$f contains a string form the literal-join scanner does not understand (escaped quote or raw string) -- its coverage would silently differ from every other pass; refusing"
  litstream=$(mktemp)
  fold_apostrophes < "$region" | awk -f "$LITSCAN" > "$litstream"
  n_litstream=$(wc -l < "$litstream")
  # A scanner that emits nothing reports a clean 0, which is the pass-286 /
  # pass-298 failure mode (an instrument that succeeds while measuring nothing).
  #
  # MEASURED, NOT ASSUMED: `src/adjacency.rs` has NO string literal at all in
  # its production region, so an EMPTY stream is the correct reading for that
  # file and an earlier version of this gate failed on it. A per-file
  # non-empty assertion is therefore wrong, and it is wrong in the useful
  # direction: it would have demanded output from a file that has nothing to
  # output. So the invariant is split in two:
  #   per file    quotes == 0  =>  stream == 0  (consistency, both directions)
  #   across files the aggregate must be NON-ZERO (the scanner is measuring)
  n_quotes=$(grep -c '"' < "$region" || true)
  if [ "$n_quotes" -eq 0 ] && [ "$n_litstream" -ne 0 ]; then
    fail "$f has no string literal in its production region but the scanner emitted $n_litstream line(s) -- the scanner is inventing content; refusing"
  fi
  total_litstream=$(( total_litstream + n_litstream ))
  l=$(tr -s ' \t' ' ' < "$litstream" | grep -cEi "($LF_RE)" || true)
  rm -f "$litstream"
  # The population signature again, on the new population: a count equal to the
  # emitted line count means the pattern matched every statement. It is only
  # that signature on a NON-EMPTY population -- on an empty one, 0 == 0 is the
  # correct reading and not a fail-open signature. (Measured: the first version
  # of this gate aborted on src/adjacency.rs for exactly that reason.)
  if [ "$n_litstream" -gt 0 ]; then
    [ "$l" -ne "$n_litstream" ] \
      || fail "$f literal-join count ($l) equals the literal stream line count ($n_litstream) -- the pattern is matching every statement; refusing (rule 292)"
  fi
  total_lit=$(( total_lit + l ))
  # Any literal-join hit is reported with the statement that matched, so a real
  # hard-code is inspectable rather than a bare number.
  if [ "$l" -gt 0 ]; then
    printf '  %-18s region %-5s LITERAL-JOIN %-3s  UNEXPLAINED -- a canonical clue assembled from adjacent string literals:\n' "$f" "$lines" "$l"
    awk -f "$LITSCAN" < <(fold_apostrophes < "$region") | tr -s ' \t' ' ' | grep -Ei "($LF_RE)" | head -5 | sed 's/^/      /'
    fail "$f has $l literal-join canonical clue occurrence(s) -- a clue assembled from adjacent string literals is the same hard-code; refusing"
  fi
  b=0
  if [ "$w" -gt 0 ]; then
    # Every per-word hit must be adjudicated. Named file+line, not a count.
    got=$(fold_apostrophes < "$region" | grep -nEio "($PW_RE)" | head -5)
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
# PASS 311: the same must hold for the clue ASSEMBLED from adjacent string
# literals, which is the spelling a source file writes when it respects a
# line-length limit. Pass 310 measured that spelling at 0/0 under both
# pre-existing forms, so this assertion is the one the item's 0 did not make.
[ "$total_lit" -eq 0 ] \
  || fail "the canonical clue appears $total_lit time(s) once adjacent string literals are reassembled -- a clue split across literals is the same hard-code; refusing (pass 311)"
# The scanner must have measured SOMETHING somewhere. Aggregate-level because
# one of the six production files legitimately has no string literal at all
# (src/adjacency.rs, measured), so only the total can distinguish "no hard-code"
# from "the form is not running".
[ "$total_litstream" -gt 0 ] \
  || fail "the literal-join scanner emitted 0 lines across all ${#FILES[@]} production regions -- the form would report a clean 0 while measuring nothing; refusing (pass 311)"
printf '  literal-join stream: %d statement-forms scanned across %d regions (src/adjacency.rs has no string literal; measured, not assumed)\n' "$total_litstream" "${#FILES[@]}"

# ---------------------------------------------------------------------------
# 4. Controls, in BOTH directions. A fence that cannot fail is decoration.
# ---------------------------------------------------------------------------
# Run on a COPY of a production file, never on the repository's own src/.
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# PASS 308: the plant literal is the FIRST REAL CLUE PHRASE, read out of the
# defining document -- NOT the pattern's own output. Pre-308 set
# `JP_PROBE="$JP_RE"`, i.e. it planted whatever the pattern happened to be,
# which is why a pattern that could not match anything still passed every
# control: the control and the measurement were the same string (rule 262).
JP_PROBE=$(printf '%s\n' "$PHRASES" | grep . | head -1)
[ -n "$JP_PROBE" ] || fail "no clue phrase to plant -- refusing"

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

# PASS 308, THE CONTROL THAT MATTERS: EVERY clue the property names, planted as
# a CONTIGUOUS literal, must be CAUGHT BY THE JOINED FORM. Pre-308 there was one
# control, it planted a non-clue, and it read 1 -- so the unmatchable pattern
# was certified. Rule 14v: plant every string the property names, not one
# representative. Iterated over the derived phrases so the control set cannot
# drift from the property document.
c_phrase_expect=$n_phrase
c_each=0; c_each_fail=""
while IFS= read -r ph; do
  [ -n "$ph" ] || continue
  lit=${ph//[?]/}
  { printf 'const HARD: &str = "%s";\n' "$lit"; cat src/adjacency.rs; } > "$TMP/each.rs"
  n=$("$STRIPPER" "$TMP/each.rs" 2>/dev/null | grep -cEi "$JP_RE" || true)
  c_each=$(( c_each + 1 ))
  [ "$n" -ge 1 ] || c_each_fail="$c_each_fail [$ph => $n]"
done <<< "$PHRASES"
[ -z "$c_each_fail" ] \
  || fail "control: these canonical clues were planted as contiguous literals and NOT caught:$c_each_fail -- the joined form does not fire on the string the property names; refusing"

# PASS 308, THE TITLE-CASE CONTROL: the defining document spells one clue in
# title case, and that is the spelling a developer would most likely copy. The
# pre-308 per-word form was case-SENSITIVE and read 0 on it, so a hard-code in
# the document's OWN capitalisation was invisible. This plants the second
# phrase in UPPER CASE, which differs from the derived lower-case phrase only
# in case, and requires the per-word form to catch it.
UP_PROBE=$(printf '%s\n' "$PHRASES" | grep . | sed -n '2p' | tr 'a-z' 'A-Z')
[ -n "$UP_PROBE" ] || fail "no second clue phrase to plant in upper case -- refusing"
{ printf 'const HARD: [&str; 5] = [%s];\n' "$(printf '%s' "$UP_PROBE" | awk '{for(i=1;i<=NF;i++) printf "%s\"%s\"", (i>1 ? ", " : ""), $i}')"; cat src/adjacency.rs; } > "$TMP/upper.rs"
c_upper=$("$STRIPPER" "$TMP/upper.rs" 2>/dev/null | grep -oEi "($PW_RE)" | wc -l)

# PASS 308, THE TARGET-SIDE CONTROL (rule 14t): a hard-code keyed on the
# QUESTION must also be caught. `if t == "recognize speech" { ... }` names no
# clue word at all, so a fence built from the answers alone reads 0 on exactly
# the shape a developer is most likely to write. The target phrases come from
# the same derivation (the other side of the arrow), so this control cannot
# drift from the property document.
TGT_J="$TARGETS"
c_tgt=0; c_tgt_n=0
while IFS= read -r tp; do
  [ -n "$tp" ] || continue
  c_tgt_n=$(( c_tgt_n + 1 ))
  # A derived line may be a regex alternation (apostrophe character class), so
  # the PLANTED literal must be the plain spelling, never the pattern text. A
  # control that plants its own pattern fragment is rule 262 again: it would
  # plant a `?` quantifier and read 0 for a reason that has nothing to do with
  # the hard-code it is pretending to be.
  lit=${tp//[?]/}
  { printf 'if t == "%s" { return best(); }\n' "$lit"; cat src/adjacency.rs; } > "$TMP/tgt.rs"
  n=$("$STRIPPER" "$TMP/tgt.rs" 2>/dev/null | grep -cEi "$JP_RE" || true)
  [ "$n" -ge 1 ] && c_tgt=$(( c_tgt + 1 )) || printf '         target-side phrase NOT caught: [%s]\n' "$lit"
done <<< "$TGT_J"
[ "$c_tgt" -eq "$c_tgt_n" ] \
  || fail "control: $(( c_tgt_n - c_tgt )) of $c_tgt_n target-side hard-code(s) were planted and NOT caught -- the fence is blind to a hard-code keyed on the question rather than the answer (rule 14t); refusing"

# PASS 308, THE ENCODING CONTROL. The straight-apostrophe plant above passes
# even when the curly spelling is invisible, because the straight and curly
# forms differ only in encoding -- so the control set must contain BOTH, or the
# suite certifies a fence with a known hole in it. This plants each target with
# the curly apostrophe and requires it to be caught, which is what the text fold
# exists to make possible. A fence that passes this and fails nothing else is
# encoding-independent; before the fold, this plant read 0.
c_curly=0; c_curly_n=0
while IFS= read -r tp; do
  [ -n "$tp" ] || continue
  # Only a phrase that actually CONTAINS an apostrophe gets the curly variant;
  # testing the raw derived line for one is what keeps the control non-vacuous
  # (comparing a stripped line with itself would silently pass every time).
  case "$tp" in *"$APOSTROPHE"*) ;; *) continue ;; esac
  c_curly_n=$(( c_curly_n + 1 ))
  curly=${lit/$APOSTROPHE/$'\xe2\x80\x99'}
  { printf 'if t == "%s" { return best(); }\n' "$curly"; cat src/adjacency.rs; } > "$TMP/curly.rs"
  n=$("$STRIPPER" "$TMP/curly.rs" 2>/dev/null | fold_apostrophes | grep -cEi "$JP_RE" || true)
  [ "$n" -ge 1 ] && c_curly=$(( c_curly + 1 )) || printf '         curly-apostrophe plant NOT caught: [%s]\n' "$curly"
done <<< "$TGT_J"
[ "$c_curly_n" -ge 1 ] \
  || fail "control: no target phrase contains an apostrophe, so the encoding control is vacuous -- refusing"
[ "$c_curly" -eq "$c_curly_n" ] \
  || fail "control: $(( c_curly_n - c_curly )) of $c_curly_n curly-apostrophe hard-code(s) were planted and NOT caught -- the fence matches a spelling, not the hard-code; refusing (pass 308)"

# PASS 311, THE LINE-SCOPE CONTROLS. These are the controls passes 308 and 309
# did not have, and their absence is why the hole survived them: every earlier
# plant put the clue on ONE line, so a line-scoped fence passed every one of
# them. Each plant below splits the phrase across literals the way a
# line-length-respecting source file does, and the pre-311 forms are MEASURED
# on the same plant, so the control reports both numbers and cannot be read as
# certifying a form that was never deficient.
#
# midword: the split lands INSIDE a word (`"Hi"` / `"ts J"` / ... / `"ame"`).
#   This defeats the joined form (not contiguous) AND the per-word form (no
#   line holds a whole clue word) -- measured 0/0 at pass 310, which is the
#   case the existing eleven controls could not reach.
# wordbound: the split lands BETWEEN words, in a clue of a different length
#   from the first, so it cannot be confused with a restatement of midword.
# targetside: the split is on the TARGET side of the arrow, i.e. the hard-code
#   a developer writes as `if t == "recognize " "speech"` (rule 14t).
CUE_MID=${JP_PROBE// / }
# Reassemble the probe into six fragments, at least one of which splits a word
# in half, by construction rather than by hand: the cut points are computed from
# the probe's own length so this control cannot drift from the property doc.
LJ_MID=$TMP/midword.rs
{
  printf 'const CLUE: &str = '
  n=${#JP_PROBE}; a=$(( n / 6 )); b=$(( 2 * n / 6 )); c=$(( 3 * n / 6 )); d=$(( 4 * n / 6 )); e=$(( 5 * n / 6 ))
  printf '"%s"\n  "%s"\n  "%s"\n  "%s"\n  "%s"\n  "%s";\n' \
    "${JP_PROBE:0:$a}" "${JP_PROBE:$a:$((b-a))}" "${JP_PROBE:$b:$((c-b))}" \
    "${JP_PROBE:$c:$((d-c))}" "${JP_PROBE:$d:$((e-d))}" "${JP_PROBE:$e}"
  cat src/adjacency.rs
} > "$LJ_MID"
LJ_WORDB=$TMP/wordbound.rs
{
  printf 'const CLUE: &str = "%s"\n  "%s";\n' "${JP_PROBE:0:$a}" "${JP_PROBE:$a}"
  cat src/adjacency.rs
} > "$LJ_WORDB"
LJ_TGT=$TMP/targetside.rs
{
  printf 'if t == "%s"\n  "%s" { return best(); }\n' "${TGT_J%%$'\n'*}" "${TGT_J#*$'\n'}"
  cat src/adjacency.rs
} > "$LJ_TGT"

# The plants must be caught by the NEW form, and the OLD forms' readings are
# printed beside them so the gap stays visible instead of being closed silently.
# DOUBLE quotes, deliberately. This was written `'($LF_RE)'` first and read 0
# on a plant the same pattern caught when pasted literally: single quotes do not
# expand, so the "pattern" was the seven characters `($LF_RE)` and the control
# failed while the measurement over the real regions passed -- a control
# defeated by its own quoting, which is the same class as the pass-308
# control/measurement identity, reached from the other direction.
LJ_RE="($LF_RE)"
lj_mid=$("$STRIPPER" "$LJ_MID" 2>/dev/null | awk -f "$LITSCAN" | tr -s ' \t' ' ' | grep -cEi "$LJ_RE" || true)
lj_mid_jp=$("$STRIPPER" "$LJ_MID" 2>/dev/null | grep -cEi "$JP_RE" || true)
lj_mid_pw=$("$STRIPPER" "$LJ_MID" 2>/dev/null | grep -oEi "($PW_RE)" | wc -l)
lj_wb=$("$STRIPPER"  "$LJ_WORDB" 2>/dev/null | awk -f "$LITSCAN" | tr -s ' \t' ' ' | grep -cEi "$LJ_RE" || true)
lj_wb_jp=$("$STRIPPER" "$LJ_WORDB" 2>/dev/null | grep -cEi "$JP_RE" || true)
lj_tgt=$("$STRIPPER" "$LJ_TGT" 2>/dev/null | awk -f "$LITSCAN" | tr -s ' \t' ' ' | grep -cEi "$LJ_RE" || true)
# A hard-code planted INSIDE the test module must still not be caught: the new
# form reads the same stripped region, so the boundary has to hold in both
# directions or the form is simply a bigger hole.
{ cat src/adjacency.rs; printf '\nconst HARD: &str = "%s"\n  "%s";\n' "${JP_PROBE:0:$a}" "${JP_PROBE:$a}"; } > "$TMP/ljbelow.rs"
lj_below=$("$STRIPPER" "$TMP/ljbelow.rs" 2>/dev/null | awk -f "$LITSCAN" | tr -s ' \t' ' ' | grep -cEi "$LJ_RE" || true)

c_above=$("$STRIPPER" "$TMP/above.rs"    2>/dev/null | grep -cEi "$JP_RE" || true)
c_below=$("$STRIPPER" "$TMP/below.rs"    2>/dev/null | grep -cEi "$JP_RE" || true)
c_mt=$("$STRIPPER"    "$TMP/mtblock.rs"  2>/dev/null | grep -cEi "$JP_RE" || true)
c_sl=$("$STRIPPER"    "$TMP/slashstr.rs" 2>/dev/null | grep -cEi "$JP_RE" || true)
c_decomp=$("$STRIPPER" "$TMP/decomp.rs"  2>/dev/null | grep -oEi "($PW_RE)" | wc -l)
# The expectation is the PER-WORD alphabet size, not the joined one: the array
# spells all nine clue words, and the per-word form deliberately does not count
# the single-letter `a`. So 8 is the correct reading here, and a control that
# expected 9 would be testing the wrong thing.
c_decomp_n=$n_pw

printf '  controls    rule-292 self-check: pattern does not match "zzz"\n'
printf '             every derived clue, planted contiguous -> %s/%s caught (must be all)\n' "$c_each" "$c_phrase_expect"
printf '             target-side hard-codes (rule 14t)     -> %s/%s caught (must be all)\n' "$c_tgt" "$c_tgt_n"
printf '             same, with a CURLY apostrophe       -> %s/%s caught (must be all)\n' "$c_curly" "$c_curly_n"
printf '             clue in UPPER CASE, decomposed    -> %s per-word hits (must be >= 1)\n' "$c_upper"
printf '             planted above mod tests  -> %s (must be 1)\n' "$c_above"
printf '             planted below mod tests  -> %s (must be 0: the boundary is real)\n' "$c_below"
printf '             mod tests in /* */      -> %s (must be 1)\n' "$c_mt"
printf '             // inside a string      -> %s (must be 1)\n' "$c_sl"
printf '             decomposed array        -> %s per-word hits (must be %s; the joined form cannot see this, rule 14x)\n' "$c_decomp" "$c_decomp_n"
printf '  pass 311    literal-join form, clue split MID-WORD  -> %s (must be >= 1); joined form read %s, per-word read %s (the pass-310 hole)\n' "$lj_mid" "$lj_mid_jp" "$lj_mid_pw"
printf '             literal-join form, split at a WORD bound  -> %s (must be >= 1); joined form read %s\n' "$lj_wb" "$lj_wb_jp"
printf '             literal-join form, TARGET side split     -> %s (must be >= 1)\n' "$lj_tgt"
printf '             literal-join form, planted BELOW mod tests -> %s (must be 0: the boundary holds for the new form too)\n' "$lj_below"

[ "$c_upper" -ge 1 ] || fail "control: the clue planted in UPPER CASE as a decomposed array produced 0 per-word hits -- the per-word form is case-sensitive again, so a hard-code in the defining document's own capitalisation is invisible; refusing (pass 308)"
[ "$c_above" -eq 1 ] || fail "control: the planted clue ABOVE the boundary was not detected (read $c_above) -- the fence does not fire on a real hard-code; refusing"
[ "$c_below" -eq 0 ] || fail "control: the planted clue BELOW the boundary WAS detected (read $c_below) -- the test-module boundary is not being honoured; refusing"
[ "$c_mt"    -eq 1 ] || fail "control: 'mod tests' inside a block comment moved the boundary (read $c_mt); refusing"
[ "$c_sl"    -eq 1 ] || fail "control: '//' inside a string literal ate the hard-code (read $c_sl); refusing"
[ "$c_decomp" -eq "$c_decomp_n" ] || fail "control: the decomposed array produced $c_decomp per-word hits, expected $c_decomp_n -- the per-word form is not seeing every clue word; refusing"
# PASS 311 assertions. The three plants are the spellings the pre-311 forms
# could not see; the fourth keeps the new form from becoming a boundary hole.
[ "$lj_mid" -ge 1 ] \
  || fail "control: a canonical clue split MID-WORD across adjacent string literals produced $lj_mid literal-join hits (joined read $lj_mid_jp, per-word read $lj_mid_pw) -- the literal-join form is blind to the spelling this fence exists to catch; refusing (pass 311)"
[ "$lj_wb" -ge 1 ] \
  || fail "control: a canonical clue split at a WORD boundary across adjacent literals produced $lj_wb literal-join hits; refusing (pass 311)"
[ "$lj_tgt" -ge 1 ] \
  || fail "control: a target-side hard-code split across adjacent literals produced $lj_tgt literal-join hits -- the form is blind to a hard-code keyed on the question (rule 14t); refusing (pass 311)"
[ "$lj_below" -eq 0 ] \
  || fail "control: a literal-join plant BELOW the test-module boundary WAS detected (read $lj_below) -- the new form does not honour the boundary; refusing (pass 311)"

printf '  verdict     0 canonical clue occurrences in all %d src/ regions; %d adjudicated benign per-word hit\n' "${#FILES[@]}" "$benign_hits"
# The scope half, stated every run. "all N production regions" read as
# whole-program coverage is exactly the misreading pass 314 measured: a
# canonical clue in web/app.js left this line printing 0. The rest of the
# production region is fenced by tests/no_phrase_hard_coding.rs, not here.
printf '  scope       this script measures src/ ONLY (%d regions). NOT measured here: %s\n' "${#FILES[@]}" "${UNFENCED_DIRS:-(none)}"
printf '  scope       those directories are fenced by %s; this script refuses if that file or its region table disappears.\n' "$RUSTFENCE"
printf '  note        CI does not run this fence. .github/workflows/test.yml runs tests and clippy only.\n'
printf '              That gap is a HUMAN decision (first raised at pass 267), not a pass action.\n'
exit 0
