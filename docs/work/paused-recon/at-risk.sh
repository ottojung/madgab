#!/usr/bin/env bash
# at-risk.sh -- the at-risk-commit census, as a RUNNING instrument.
#
# WHY THIS FILE EXISTS. The census has been re-derived by hand in 90+ passes, and
# its published procedure has been corrected 15 separate times (log rules 14, 14a
# .. 14i, 14j, 14l, 14n, 14x, 14y, 280). Every one of those corrections was a
# SPELLING defect, not a change in what was being measured, and every one of them
# produced a plausible number with a clean exit status:
#
#   rule 14b  shell `^` glued to an unquoted list  -> 1,072 rows (1,000x over-report)
#   rule 14d  arms written without `--reflog`     -> 1 instead of 88 (wrong population)
#   rule 14g  `$REFS` empty in the second shell   -> 1,121 (silent full-baseline)
#   rule 14h  `--not` before a `^` list            -> 1,122 (double negation)
#   rule 14i  `for-each-ref 'refs/remotes/audit/*'`-> 108 of 204 (WM_PATHNAME)
#   rule 182  an unresolvable ref in the list     -> 0 rows, exit 128 (FALSE ZERO)
#
# The last one is the dangerous direction and it is the reason this is a script
# and not more prose: `git rev-list` prints `fatal: bad revision ...` on stderr
# and emits ZERO lines, and `wc -l` reads that as "nothing is at risk" -- a clean
# result that suppresses the only useful recurring work while the programme is
# paused. A rule can be quoted; a script cannot be mis-transcribed.
#
# This script therefore does three things the hand procedure kept failing to do:
#   1. refuses to report a number it has not validated (exit codes, cardinality,
#      two independent arms, and a known-positive / known-negative control);
#   2. treats a fatal-empty-stdout as INSTRUMENT FAILURE, never as a zero result;
#   3. pins the two exclusions that hand passes got wrong independently -- the
#      per-ref caret prefix (14b) and the explicit positive start `--all
#      --reflog` (14a/14d), neither of which may be dropped.
#
# USAGE
#   docs/work/paused-recon/at-risk.sh            # report
#   docs/work/paused-recon/at-risk.sh --fetch    # re-fetch the audit mirror first
#
# BOTH FORMS NOW VERIFY MIRROR FRESHNESS (pass 293). The bare form used to skip
# that check and did so well: pass 293 ran it first and got a formatted,
# internally consistent, control-passing "ref-held 3, at-risk 91" where the truth
# is ref-held 1, 89. The bare form now REFUSES on a stale mirror; --fetch
# refreshes and then verifies as before.
#
# `--fetch` is rule 14a: the audit namespace is a LOCAL MIRROR of the remote's
# ordinary refs/heads/* under a renamed destination, so a stale mirror puts the
# previous pass's own freshly pushed commit into the at-risk set (this happened
# in pass 187, which read 2 where the truth was 1, and again in pass 293, which
# read 3 where the truth was 1). It also passes --prune NEVER
# (pass 202: `--prune` against a mirror whose source namespace does not exist
# remotely deleted 203 of 204 local refs).
#
# WHAT IT DOES NOT DO. It creates no branch and pushes nothing. A residual
# at-risk commit is a HUMAN judgement, not an automatic action; the script's job
# is to make the number trustworthy so that judgement can be made on it.

set -euo pipefail

REPO=${REPO:-$(git rev-parse --show-toplevel)}
cd "$REPO"

# The remote namespace that the local audit mirror is built from. The mirror
# lives under refs/remotes/audit* purely so it cannot collide with the ordinary
# origin tracking refs.
SRC_NS='refs/heads/*'
# The DESTINATION pattern needs its own trailing `/*`. `$SRC_NS:$DST` without it
# is not a no-op or a cosmetic difference -- git rejects it (exit 128, "invalid
# refspec") and fetches NOTHING, so the mirror stays stale while the operator
# believes it was refreshed. That is the worst possible pairing: the safety
# action silently does nothing, and the census then reports the previous pass's
# own freshly pushed commit as at-risk. This exact bug shipped in this script's
# first draft and was caught only because the run was repeated after a push.
# Two spellings, deliberately kept distinct:
#   DST_FETCH  the refspec destination PATTERN  (needs the trailing /*)
#   DST        the bare PREFIX for for-each-ref (must NOT have one -- rule 14j:
#              for-each-ref matches with WM_PATHNAME, and a glob there silently
#              enumerates only the depth-4 refs and drops every nested namespace)
DST_FETCH='refs/remotes/audit/*'
DST='refs/remotes/audit'

# Expected ref count. NOT a constant to be trusted blindly: it is asserted, and
# the script REFUSES to report if it moved (rule 14g -- an inline cardinality
# assertion is the only check that has ever caught a live enumeration defect).
# The count grows by one per pass, because each pass pushes its own log commit
# to post-milestone-acceptance and that commit is in the exclusion set. A human
# raising this number must confirm the delta is this log's own history.
#
# 205 -> 206 at pass 320, and the delta was confirmed rather than assumed, in
# the way the comment above demands. The count is 205 mirrored heads plus the
# one peeled tag, i.e. 206; and the set difference against the previous
# expectation is exactly ONE ref, `refs/remotes/audit/review/drop-dead-trace-env`
# (a29f3d7) -- the review branch pass 319 pushed, which is this log's own
# history, not a lost or mirrored-elsewhere work branch. Two further checks
# agree and are worth recording because rule 14g's assertion is blind to
# freshness by construction and only these two speak to it:
#   - the mirror's ref NAMES are identical to `git ls-remote --heads origin`
#     after normalising `refs/heads/` away -- `comm -23` and `comm -13` both
#     empty -- so the +1 is on the remote, not a stale local artifact;
#   - removing that one ref from the enumeration returns exactly 205, so the
#     delta is that ref and not a change in the tag arm or in the enumeration
#     spelling.
#
# 206 -> 207 at pass 321, delta confirmed in the same way and for the same
# reason. The count is 206 mirrored heads plus the one peeled tag, i.e. 207, and
# the single new ref is `refs/remotes/audit/review/run-clue-fence-in-ci` (6edff83)
# -- the review branch pass 321 pushed for human item (c), so this log's own
# history, not a lost or mirrored-elsewhere work branch. Both controls re-run
# and both agree:
#   - the mirror's ref NAMES are identical to `git ls-remote --heads origin`
#     after normalising `refs/heads/` away -- `comm -23` and `comm -13` both
#     empty over 206 rows each -- so the +1 is on the remote, not a stale local
#     artifact;
#   - removing exactly that one ref from the enumeration returns exactly 206,
#     so the delta is that ref and not a change in the tag arm or the spelling.
# NOTE for the next pass: this pass's FIRST attempt at the name comparison
# reported a ~180-row difference, in the dangerous direction, because it
# normalised with `%(refname:strip=4)`. The audit mirror is
# `refs/remotes/audit/<name>` -- three components -- so strip=4 eats the branch
# name and leaves an empty string for every ref; the empty key sorts first and
# `comm` then reports nearly every remote branch as absent from the mirror. The
# correct normalisation is `sed 's|^refs/remotes/audit/||'`, or `strip=3`.
#
# PASS 323 RAISES THIS 207 -> 208. The mirror was 206 heads + 1 tag = 207 entries;
# this pass pushed ONE new branch, `review/drop-dead-trace-env-on-main` (66e28ff),
# taking it to 207 heads + 1 tag = 208. Per pass 322's standing instruction the
# delta was confirmed BEFORE raising, in this order and not another:
#   1. by ref NAME with the sed normalisation above -- `comm -13` and `comm -23`
#      both 0 against `ls-remote --heads`, i.e. 0 missing and 0 phantom, so the
#      mirror is a faithful mirror and the growth is real rather than a prune scar
#      (rule 14m: had this been a phantom, the right move would be to fix the
#      mirror, NOT to raise the number);
#   2. by removing the single candidate -- `git update-ref -d
#      refs/remotes/audit/review/drop-dead-trace-env-on-main` returns the
#      instrument to green at 206+1, which proves the delta is exactly that one
#      ref and nothing else.
# NOTE for whoever raises this next: the deletion test is only sound if you
# re-fetch afterwards. Deleting the mirror ref and LEAVING it deleted makes this
# pass's own branch tip 66e28ff read as at-risk and ref-held (at-risk went
# 90 = 1 + 89 to 91 = 2 + 89 while the ref was absent) -- the self-inflicted class
# of rule 273, reached here by removing a ref rather than by amending a commit.
# `git fetch origin '+refs/heads/*:refs/remotes/audit/*'` (never --prune, rule
# 14m) restores it and the count returns to 90 = ref-held 1 + reflog-only 89.
#
# PASS 324 RAISES THIS 208 -> 209, for the same reason and by the same two-step
# confirmation. This pass pushed ONE new branch, `review/drop-dead-trace-and-fence`
# (8c88a59), taking 208 heads + 1 tag = 209.
#
# NOTE THE ORDER OF THE TWO NUMBERS, because this pass read them in the opposite
# order on arrival and it briefly looked like a phantom. `git for-each-ref refs/
# remotes/audit | wc -l` counts HEADS ONLY and read 208, and `ls-remote --heads |
# wc -l` also read 208, so the two agreed and the instrument still failed at 209.
# The 209 is heads + TAG, and this is the second time in three passes that a
# head-only count has been the number a reader checks first (pass 261 amended the
# 204 for exactly this reason). The instrument counts the union of the mirror and
# the tag namespace, so HEAD it with the TOTAL -- `git for-each-ref --format=
# '%(refname)' refs/remotes/audit refs/remotes/audit-tag | wc -l` -- and never
# with a heads-only `ls-remote --heads`, which cannot see the tag that a
# `refs/heads/*` refspec also cannot fetch. Rule 14l again: a count carries its
# population, and here the two populations differ by one and agree on nothing.
#
# Confirmed before raising, in pass 322's order and not another:
#   1. by ref NAME, both sides normalised identically (`sed 's|^refs/remotes/
#      audit/||'` against `sed 's|.*\trefs/heads/||'`, per pass 323's correction):
#      mirror 208 heads, `ls-remote --heads` 208, `comm -13` 0 missing and
#      `comm -23` 0 phantom -- a faithful mirror, so the growth is real and not a
#      prune scar (rule 14m);
#   2. by removing the single candidate -- `git update-ref -d refs/remotes/
#      audit/review/drop-dead-trace-and-fence` returns the instrument to green at
#      207+1, proving the delta is exactly that one ref; re-fetched immediately
#      afterwards per the note above, and the count returned to 209.
EXPECT_REFS=209

# WHY NOT TO "REFRESH THE MIRROR" WITH A NARROWED SOURCE (measured 2026-09-30,
# this pass's own self-inflicted red; EXPECT_REFS was NOT raised).
#
# A pass wanting to look at the `review/*` branches is tempted to run
#   git fetch origin 'refs/heads/review/*:refs/remotes/audit/*'
# instead of the whole-namespace form. That exits 0, looks like a refresh, and
# SILENTLY QUADRUPLES the mirror: git's refspec `*` matches the pattern's PREFIX,
# so `refs/heads/review/drop-dead-trace-and-fence` is written to
# `refs/remotes/audit/drop-dead-trace-and-fence` -- the `review/` component is
# STRIPPED -- while `refs/remotes/audit/review/drop-dead-trace-and-fence` already
# exists from the whole-namespace fetch. Four branches become eight refs, the
# cardinality assertion fires at 213 != 209, and at-risk.sh (and therefore
# selfcheck, which reports 12 of 13) goes red on a mirror that is otherwise a
# faithful mirror of origin.
#
# The failure is invisible in the fetch output, which reports "[new branch]"
# exactly as it would for a genuine addition, and the four phantoms are byte-equal
# to their real counterparts (`bare=8c88a59 review=8c88a59 same=yes` for all
# four), so nothing looks wrong until the count disagrees. A narrowed source
# refspec is a MIRROR-CORRUPTING action, not a cheaper refresh.
#
# Repair, and it is `update-ref -d`, never a re-fetch: deleting the four phantoms
# returns the count to 209 and the report to 90 = ref-held 1 + reflog-only 89
# (disjoint), with both controls behaving (514ed91 present, 0267ade absent). A
# re-fetch does NOT repair it -- the whole-namespace fetch cannot un-write the
# stripped names, and re-running the narrowed form reproduces them.
#
# Note this is NOT the pass-202 `--prune` shape: nothing was lost, because every
# phantom's object is still held by its real `review/` mirror ref. The lesson is
# the mirror's own subject, not the refs: EXPECT_REFS is a cardinality assertion,
# so a ref that is a faithful duplicate is still a defect in the instrument's
# input, and the honest response is to fix the input (rule 14m), never to raise
# the expectation to match a corrupted mirror.

# Known-good / known-bad controls. These are the arms' discriminators: a census
# that cannot tell these two apart is reporting a constant, not a measurement.
#   CTRL_POSITIVE 514ed91 -- held by exactly refs/heads/scratch-3f8c62-landed;
#                          its non-build content is durable on
#                          origin/recovery/at-risk-2026-09-29.
#   CTRL_NEGATIVE 0267ade -- origin/main's tip; must never be at risk.
CTRL_POSITIVE=514ed91
CTRL_NEGATIVE=0267ade

fail() { printf 'at-risk.sh: %s\n' "$*" >&2; exit 1; }

# Concurrency: every scratch path below lives in a private mktemp -d, never a
# fixed /tmp name. A shared name let two simultaneous runs of this instrument
# (or of it alongside at-risk-delta.sh) read each other's partially-written
# files, which surfaced as a fabricated "ref cardinality 10 != expected 209"
# and as selfcheck.sh reporting this instrument DEAD. The bare-gate set is run
# in parallel by design, so the collision was reachable, not hypothetical.
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

if [ "${1:-}" = "--fetch" ]; then
  # Rule 14a, amended at pass 202: fetch the mirror by its REAL source namespace
  # and never pass --prune to a mirror.
  # Capture git's own stderr: a bare "fetch failed" is not a diagnosis, and the
  # whole reason this branch exists is that a failure must never be silent.
  if ! git fetch origin "+${SRC_NS}:${DST_FETCH}" >$TMP/fetch 2>&1; then
    cat $TMP/fetch >&2
    fail "audit fetch failed -- refusing to report against a stale mirror"
  fi
  # A fetch can exit 0 and fetch NOTHING. The measured instance: a refspec whose
  # source pattern matches no remote ref at all is not an error to git -- it is an
  # empty result -- so the mirror silently stays stale while the operator is told
  # it was refreshed. That is the pass-202 --prune disaster's exact shape (a
  # safety action that appears to work and does nothing), and the consequence is
  # the one this file exists to prevent: the previous pass's own pushed commit is
  # reported as at-risk. So the fetch is VERIFIED, not trusted: the remote's
  # actual tip of the accumulation branch must now be present in the mirror.
  probe=$(git ls-remote origin refs/heads/post-milestone-acceptance | awk '{print $1}')
  [ -n "$probe" ] || fail "could not read origin's post-milestone-acceptance tip; refusing to report"
  if ! git rev-parse --verify --quiet "refs/remotes/audit/post-milestone-acceptance" >/dev/null; then
    fail "fetch reported success but the mirror has no post-milestone-acceptance ref -- the refspec matched nothing; refusing to report against a stale mirror"
  fi
  mirrored=$(git rev-parse refs/remotes/audit/post-milestone-acceptance)
  [ "$mirrored" = "$probe" ] \
    || fail "fetch left the mirror at $mirrored while origin is at $probe -- the refresh was partial; refusing to report"
  printf 'audit mirror re-fetched from %s to %s (no --prune), verified at %s\n' \
    "$SRC_NS" "$DST_FETCH" "$mirrored"
else
  # PASS 293 FIX -- the DEFAULT path used to report against a stale mirror, and
  # did so well. This pass ran the bare form first and got a clean, formatted,
  # self-consistent, control-passing report of "ref-held 3 + reflog-only 88 =
  # 91", where the truth is ref-held 1 and 89. Every internal check agreed: the
  # two arms matched, the partition summed, the known-positive was present and
  # the known-negative absent, and EXPECT_REFS was satisfied at 205. The two
  # spurious members were pass 292's own pushed commits (ec06efa, 59f6b05),
  # which the mirror had not yet picked up.
  #
  # Note WHICH checks failed to catch it, because that is the generalisable part:
  #   - the controls are about the EXCLUSION SET's width, not its freshness, and
  #     a too-narrow set still passes both of them;
  #   - EXPECT_REFS counts REFS, and a fast-forward of one ref does not change
  #     the count -- rule 14g's cardinality assertion is blind to staleness by
  #     construction;
  #   - the arms agreeing proves only that two spellings of the same stale
  #     input agree, which is what they are for.
  # The file's own header already named this failure ("pass 187, which read 2
  # where the truth was 1"), and the mitigation was documented but only wired
  # into the OPTIONAL flag. An operator running the documented default form got
  # no protection at all.
  #
  # So: staleness is now checked in BOTH modes, and in the default mode a stale
  # mirror is a refusal, not a number. Refusing is the whole point -- the
  # alternative is publishing a plausible, wrong, safety-relevant figure.
  origin_tip=$(git ls-remote origin refs/heads/post-milestone-acceptance | awk '{print $1}')
  [ -n "$origin_tip" ] || fail "could not read origin's post-milestone-acceptance tip; refusing to report"
  if ! git rev-parse --verify --quiet "refs/remotes/audit/post-milestone-acceptance" >/dev/null; then
    fail "audit mirror has no post-milestone-acceptance ref -- the exclusion set cannot be trusted; re-run with --fetch"
  fi
  mirrored_tip=$(git rev-parse refs/remotes/audit/post-milestone-acceptance)
  [ "$mirrored_tip" = "$origin_tip" ] \
    || fail "audit mirror is STALE: mirror at $mirrored_tip, origin at $origin_tip -- the previous pass's own pushed commits would be reported as at-risk (this is the pass-187 failure, unfixed on the default path until pass 293). Re-run with --fetch"
  printf 'audit mirror verified fresh against origin (no fetch needed) at %s\n' "$mirrored_tip"
fi

# ---------------------------------------------------------------------------
# Build the exclusion set. Sanctioned spelling, rule 14j: a BARE PREFIX with no
# glob, because for-each-ref matches with WM_PATHNAME and 'refs/remotes/audit/*'
# enumerates only the depth-4 refs and silently drops every nested namespace.
#
# '%(*objectname)%(objectname)' is the PEELED object: an annotated tag's own
# objectname is a tag object, not a commit. Both the caret and --not spellings
# accept a commit id but neither reliably accepts a tag object in this position,
# and a tag in the list is exactly what makes the run fatal-empty (see below).
# NOTE: the caret prefix is added by `sed` in the ARGS below, never inside the
# --format string. Putting it in --format yields '^(ref)', which is not valid
# revision syntax: git rejects it with `fatal: bad revision` and emits ZERO
# lines, which is the pass-182 false zero wearing a new disguise.
# ---------------------------------------------------------------------------
git for-each-ref --format='%(*objectname)%(objectname)' "$DST" "$DST-tag" \
  > $TMP/refs

n_refs=$(grep -c . $TMP/refs || true)
[ "$n_refs" -eq "$EXPECT_REFS" ] \
  || fail "ref cardinality $n_refs != expected $EXPECT_REFS -- the enumeration or the mirror moved; confirm the delta is this log's own pushed history, then raise EXPECT_REFS deliberately"

# Every entry must be a bare 40-hex id. This is the check that would have caught
# the '^(ref)' form: it is malformed, and the run that used it reported 0.
bad=$(awk '!/^[0-9a-f]{40}$/{n++; if (n==1) first=$0} END{print n+0}' $TMP/refs)
[ "$bad" -eq 0 ] \
  || fail "$bad malformed exclusion entries, first: $(awk '!/^[0-9a-f]{40}$/{print; exit}' $TMP/refs) -- refusing to run (a malformed revision makes rev-list emit 0 rows and exit 128)"

# ---------------------------------------------------------------------------
# The population. `--all --reflog` explicitly, always (rule 14a: a stateless
# exclusion spelling is not a population; rule 14d: an arm written without
# --reflog measures a strictly smaller population and reads as a wrong answer).
# ---------------------------------------------------------------------------
# NOTE the `|| true` on every pipeline below, and WHY. Under `set -e -o pipefail`
# a failing `git rev-list` aborts the script at the pipeline, so the explicit
# exit-code checks after it never run and the operator sees a bare exit 128 with
# no message. That is this file's own fail-open shape reproduced inside the file
# written to prevent it: `set -e` is not a check, it is an abort, and an abort
# that carries no diagnosis is indistinguishable from a crash. Each pipeline
# therefore always completes, and the exit code is examined deliberately.
git rev-list --all --reflog 2>$TMP/err | sort -u > $TMP/base || b=$?
: "${b:=0}"
[ "$b" -eq 0 ] && [ ! -s $TMP/err ] \
  || { cat $TMP/err >&2; fail "baseline rev-list exit $b -- instrument failure, not a result"; }
[ -s $TMP/base ] || fail "empty baseline -- instrument failure, not a zero result"
base=$(wc -l < $TMP/base)

# Two arms in DIFFERENT spelling families (rule 14c): one --not, one per-ref
# caret. Agreement between two arms of the same family cannot detect a bug they
# share; agreement between these two, PLUS the controls below, can.
# The pipeline's status is captured INSIDE the `||` block, where `$?` is still the
# pipeline's. Two other spellings were tried and both are wrong, in the direction
# that matters: `... | sort -u > f || true` followed by `e=${PIPESTATUS[0]}` on a
# later line reads the status of `true` (always 0), and reading PIPESTATUS
# immediately after a `set -e`-protected pipeline never executes at all. Either
# way a run that died with `fatal: bad object` reported "exit 0" -- a
# self-reporting validator that misreports its own instrument is worse than none.
# `set -o pipefail` is what makes `$?` here the right code rather than sort's.
git rev-list --all --reflog --not $(cat $TMP/refs) 2>$TMP/e1 \
  | sort -u > $TMP/arm1 || e1=$?
: "${e1:=0}"
git rev-list --all --reflog $(sed 's|^|^|' $TMP/refs) 2>$TMP/e2 \
  | sort -u > $TMP/arm2 || e2=$?
: "${e2:=0}"

[ "$e1" -eq 0 ] && [ ! -s $TMP/e1 ] || { cat $TMP/e1 >&2; fail "arm1 (--not) exit $e1 with stderr -- no number reported"; }
[ "$e2" -eq 0 ] && [ ! -s $TMP/e2 ] || { cat $TMP/e2 >&2; fail "arm2 (caret) exit $e2 with stderr -- no number reported"; }

diff -q $TMP/arm1 $TMP/arm2 >/dev/null \
  || fail "arms disagree -- no number reported"

at_risk=$(wc -l < $TMP/arm1)

# Split: held by no ref at all (reflog-only) vs held by some ref.
git rev-list --all 2>/dev/null | sort -u > $TMP/refsonly
comm -23 $TMP/base $TMP/refsonly > $TMP/reflogonly
reflog_only=$(wc -l < $TMP/reflogonly)
ref_held=$(( at_risk - reflog_only ))

# The split must be a partition of the at-risk set: both parts non-negative and
# summing back. A negative part means the two populations were not nested, which
# is a broken instrument and not a finding -- and note the direction: a negative
# part still PRINTS, so without this the script would report "ref-held -1207"
# as though it were a measurement. Assert the arithmetic, do not display it.
[ "$ref_held" -ge 0 ] && [ "$reflog_only" -ge 0 ] \
  && [ $(( ref_held + reflog_only )) -eq "$at_risk" ] \
  && [ "$reflog_only" -le "$base" ] && [ "$(wc -l < $TMP/refsonly)" -le "$base" ] \
  || fail "split is not a partition (ref-held $ref_held, reflog-only $reflog_only, at-risk $at_risk, baseline $base) -- the refs-only population is not nested in the baseline; no number reported"

# Controls. A census that cannot separate these two is a constant, not a
# measurement, and the whole point of the script is that the number is one.
grep -q "^$CTRL_POSITIVE" $TMP/arm1 \
  || fail "control $CTRL_POSITIVE absent from the at-risk set -- the exclusion set is too wide; no number reported"
grep -q "^$CTRL_NEGATIVE" $TMP/arm1 \
  && fail "control $CTRL_NEGATIVE (origin/main tip) reported at risk -- the exclusion set is broken; no number reported"

printf 'at-risk: %s total = ref-held %s + reflog-only %s (disjoint)\n' \
  "$at_risk" "$ref_held" "$reflog_only"
printf '  population  baseline(--all --reflog) %s   refs-only(--all) %s   exclusion refs %s\n' \
  "$base" "$(wc -l < $TMP/refsonly)" "$n_refs"
printf '  arms        --not and per-ref caret agree, both exit 0, both stderr empty\n'
printf '  controls    %s present, %s absent\n' "$CTRL_POSITIVE" "$CTRL_NEGATIVE"
printf '  at risk:\n'
sed 's/^/    /' $TMP/arm1
printf '  next        inspect each holder with `git for-each-ref --contains <sha>`;\n'
printf '              a residual is a HUMAN judgement, not an automatic action.\n'
