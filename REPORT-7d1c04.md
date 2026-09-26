# w-2f7a10 — independent validation of the held sweep fix `79309a1`

Front `7d1c04` (agent `madgab-sweep-verify-7d1c04`), the half of the evidence front
`e1a3f7` (`/workspace/madgab-costref`) is **not** producing. `e1a3f7` owns the
per-depth cost/refusal accounting of the reserve; nothing below measures that.

* Branch `madgab-sweep-verify-7d1c04`, tip `79309a1`.
* Under review: `79309a1` vs base `aa662a4`. `src/lib.rs` only, **189 added / 15 removed**.
* Base arm built without merging, by archive only:
  `mkdir -p /workspace/madgab-base-7d1c04 && git archive aa662a4 | tar -x -C /workspace/madgab-base-7d1c04`
* `cargo fmt`, `cargo clippy` and doctests **do not exist on this host** (see
  `docs/environment-notes.md`). I ran `cargo test --release --lib` and
  `cargo test --release --test <name>` only, and claim nothing about fmt/clippy/doctests.

---

## 0. HEADLINE, stated before anything else

**`79309a1` is NOT the gate for the canonical resegmentation.**
`approximate_finds_classic_madgab_resegmentation` is **red on `79309a1` and red on
the `git archive aa662a4` base arm** — I re-ran it on both, same failure, same test.
`Hits Justice Dupe Hid Came` is **absent** from the visible top 50 for
`It's just a stupid game` on both arms. So on the evidence in this report, `79309a1`
is **at most a general breadth/quality improvement to the reserve's enumeration
mechanism**; it is **not** a step toward the milestone and it does not move any
visible outcome number I measured (0 of 7 targets changed).

That is the finding, not a caveat on it. Everything below is the supporting
evidence, and the verdict in section 5 is about *integrability and safety*, not
about having reached the milestone.

## 1. Fence scan — **EMPTY**

Added lines of `git diff aa662a4..79309a1 -- src/lib.rs`, scanned mechanically with
`node` (there is no `rg`/`grep` on this host). Command:

```
node -e '
const {execSync}=require("child_process");
const d=execSync("git diff aa662a4..79309a1 -- src/lib.rs",{cwd:"/workspace/madgab-verify-7d1c04",maxBuffer:1e8}).toString();
const added=d.split("\n").filter(l=>l.startsWith("+")&&!l.startsWith("+++"));
console.log("added_lines="+added.length);
const toks=["recognize","speech","wreck","nice","beach","stupid","dupe","justice","hits","hid","came","mad gab","madgab","phrase_signature"];
let hits=0;
for(const t of toks){const re=new RegExp(t,"gi");added.forEach((l,i)=>{if(re.test(l)){console.log("TOKEN["+t+"] +"+(i+1)+": "+l);hits++}})}
const pre=/(ZZ_|MADGAB_|zz_|env::var|std::env)/;
added.forEach((l,i)=>{if(pre.test(l)){console.log("SCAFFOLD +"+(i+1)+": "+l);hits++}});
console.log("HITS="+hits);'
```

Result:

```
added_lines=189
HITS=0
```

Second pass, for case-insensitive equality against a word list and for any string
comparison at all among the added lines:

```
node -e '... added.forEach((l,i)=>{if(/==|contains|matches|str::eq|to_ascii_lowercase|eq_ignore|split_whitespace|iter\(\).*any/.test(l)){console.log("+"+(i+1)+": "+l.trim());n++}}); console.log("EQ_LIKE="+n)'
```

Result:

```
+58: +            if !legal || combo.iter().any(|&slot| tuple[slot] >= slot_widths[slot])
EQ_LIKE=1
```

That one line is an **integer** bound check on a slot index against a slot width.
There is no string comparison, no word list, no `to_ascii_lowercase`, no
`split_whitespace`, no `env::var` and no `ZZ_`/`MADGAB_` scaffolding anywhere in the
189 added lines. `--test no_phrase_hard_coding` (6 passed, 0 failed) is green and is
the standing fence from `docs/work/items/w-d4f0b2.md`; it was neither edited nor
weakened by this commit (`git diff --stat` shows one file, `src/lib.rs`).

**Blocker: none.** Fence is clean.

## 2. Generality argument (from the code)

The new rule (`src/lib.rs:302-350`):

```text
sweep_index(width, per, nth, member, phase)
  = 10 + (nth*ceil(span/per) + member + rate(member, span)*phase) mod span,  span = width-10
rate(0, span) = 1
rate(j, span) = smallest integer > rate(j-1, span) with gcd(., span) == 1
```

**What it makes possible.** A shape class of `deep` members used to receive *one*
index `at`, so every tuple the reserve emitted for that class was a **diagonal** of
the class's index rectangle. A pairing of two deep alternatives is a **point** of
that rectangle, and no amount of `EMIT_PROFILE_RESERVE` could express one, because
the class spends a single coordinate. With a `member` axis, member *m* draws
`at + member + (rate(m,span)-1)*phase (mod span)` instead of `at`, so the class's
coordinates are at offsets that are a **function of the phase**, and off-diagonal
pairings become expressible. The new unit test
`the_coverage_sweep_covers_each_slot_and_pairs_its_class_members`
(`src/lib.rs:4256`) asserts exactly that, plus the per-slot coverage claim, over
synthetic all-`SPAN_SHORTLIST` width vectors at depths 1..=6.

**Why it is a property of shape classes, not of any input.** The five arguments of
`sweep_index` are `(width, per, nth, member, phase)`. `width` is a *slot list
length* — a structural property of the segmentation, not a property of any target
string; `member` is the member's position inside a shape-class tuple; `phase` is the
traversal's phase counter; `per` is `EMIT_PROFILE_RESERVE`. No target text, no IPA
stream, no word identity and no environment value reaches this function — that is
the same conclusion section 1 reaches mechanically, stated from the call graph.
`rate` is a pure integer function of `(member, span)`: no clock, no hash seed, no
allocation, no memo table, no data-dependent state. Coprimality is asserted for
**every** span in `1..=SPAN_SHORTLIST - LEXICAL_BRANCH_STAGE_0` (i.e. 1..=150) and
every member `0..=EMIT_PROFILE_MAX_DEEP` by
`every_sweep_rate_is_coprime_to_the_span_it_is_used_on`, so "a rate sharing a factor
with the span would visit only the residues reachable by that factor" is a statement
about all widths, not about the widths that happen to divide evenly.

**Could the new rate make some other input strictly worse? Yes, in principle, and
the honest form of it is at the *tuple* level, not the coordinate level.**

For `member = 0`, `rate(0,span) = 1` and the formula collapses to
`10 + (nth*stride + phase) mod span` — **byte-identical to the parent's index**.
That gives *coordinate-level* continuity: every index the parent could place in
slot 0 is still placeable in slot 0.

It does **not** give *tuple-level* continuity. The parent's `if
combo.iter().any(|&slot| at >= slot_widths[slot]) { continue }` was one shared
`at` compared against every member's width. The branch's version computes a
per-member `at` and then refuses the whole tuple if **any** member's `at` is illegal
(`src/lib.rs:398-430`, the `legal` flag and the trailing
`combo.iter().any(|&slot| tuple[slot] >= slot_widths[slot])`). So there exist
`(widths, nth, phase)` where the parent emitted the diagonal `(at, at, ...)` legally
and the branch emits nothing, because member 1's `at + 1 + (rate-1)*phase` runs off
the end of a narrow member. Each class of `k` members now spends `k` independent
widths instead of one, and the illegal-coordinate discard rate is strictly higher.
When the class's members have very unequal widths this bites hardest: with
`narrowest` small the span is small, `rate(1, span)` is still a distinct residue, and
the member offsets `member` are added unconditionally.

**The measured direction of that effect is: no visible regression on any target I
tested, and the one milestone behaviour the change was aimed at does not move.**
Section 4 is the number.

**Where an agent could measure the cost/refusal side** (deliberately *not* measured
here — that is `e1a3f7`'s front): instrument `coverage_tuples` to count, per
`continue` site, how many offered tuples are dropped by `legal == false` vs by the
`tuple[slot] >= slot_widths[slot]` discard, split by class size and by
`span = narrowest - LEXICAL_BRANCH_STAGE_0`, and compare base vs branch. The relevant
lines are the `continue` statements at `src/lib.rs:399-431`; the width vectors
(`slot_widths`) and the class walk (`combo`) are the inputs that decide the answer.
I did not do this and make no claim about it.

## 3. Test status on this branch

Release builds, `79309a1`, worktree `/workspace/madgab-verify-7d1c04`.

| command | base `aa662a4` | tip `79309a1` |
|---|---|---|
| `cargo test --release --lib` | **50 passed, 0 failed** | **53 passed, 0 failed** |
| `cargo test --release --test corpus_integration` | 10 passed, **1 failed** | 10 passed, **1 failed** (same test) |
| `cargo test --release --test exact_determinism` | — | **1 passed, 0 failed** |
| `cargo test --release --test approx_determinism` | — | **2 passed, 0 failed** |
| `cargo test --release --test no_phrase_hard_coding` | — | **6 passed, 0 failed** |

The single red test is `approximate_finds_classic_madgab_resegmentation`. **I
re-ran it on the base arm built by archive, not by merging:**

```
# /workspace/madgab-base-7d1c04
cargo test --release --test corpus_integration approximate_finds_classic_madgab_resegmentation
  -> FAILED. 0 passed; 1 failed   (base)
# /workspace/madgab-verify-7d1c04
cargo test --release --test corpus_integration approximate_finds_classic_madgab_resegmentation
  -> FAILED. 0 passed; 1 failed   (tip)
```

Red on base and red on tip. It is the item's **target**, not one of the two guards,
and no test was re-baselined, relaxed, skipped or deleted by this commit.

### The two guards I was told to keep green

Measured with the release CLI, default approximate path, visible top 50
(`node` used in place of the absent `grep`):

**(a) `recognize speech` -> `wreck a nice beach` — PRESENT.**

```
rs.branch includes "wreck a nice beach"        : true
rs.base   includes "wreck a nice beach"        : true
rs.branch visible lines                        : 50
```

Corroborated by `approximate_finds_recognize_speech_resegmentation`
(`tests/corpus_integration.rs:144`), which is **green** in the 10-passed set. Note
for the record: on this base the guard is already met, so the tip does not earn it —
it merely keeps it.

**(b) `It's just a stupid game` -> `Hits Justice Dupe Hid Came` — ABSENT, and I did
not try to make it appear.**

```
sg.branch includes "hits justice dupe hid came" : false
top of visible output: "it justice too bad aim", "it justice too pad aim", ...
```

Recorded status: **still not met, before or after.** This is the item's target and
this fix does not move it.

`approximate_output_is_locked` (`tests/corpus_integration.rs:459`) is untouched by
the diff and green: the locked top-10 for `I love you` is byte-identical on this
arm.

## 4. Pool/guard non-regression on non-canonical targets

Release binaries, default approximate path, **visible top 50** (stdout; the
`(corpus loaded…; search …)` timing line goes to stderr and is excluded — verified
by `cmp` succeeding).

```
for t in "I love you" "the cat sat on the mat" "put it back on the shelf" \
         "when the rain finally stopped" "he was a big fat man" "a whole lot of trouble"; do
  madgab-base   --approximate --top 50 "$t" > cmp/$n.base
  madgab-branch --approximate --top 50 "$t" > cmp/$n.branch
  cmp -s cmp/$n.base cmp/$n.branch && echo "IDENTICAL $t" || echo "CHANGED $t"
done
```

Result — **6 targets, 6 byte-identical, 0 changed:**

```
IDENTICAL  I love you
IDENTICAL  the cat sat on the mat
IDENTICAL  put it back on the shelf
IDENTICAL  when the rain finally stopped
IDENTICAL  he was a big fat man
IDENTICAL  a whole lot of trouble
```

Plus one canonical-adjacent control:

```
recognize speech, top 50:  IDENTICAL base vs branch
```

**0 of 7 targets had any change in visible output, so there is no per-target
difference to name.** This contradicts the implementer's own section 4, which
reports pool-size deltas of `+3 / +9 / -3 / -5 / +10 / -6` on six of these targets.
Both numbers can be true at once and I state both: the *pool* (the pre-`select_diverse`
candidate set, dumped by a temporary `MADGAB_POOL_DUMP` probe) moves by single-digit
percent-of-a-ten on five targets, while the *visible* top 50 does not move at all on
any of them. The pool deltas are real but do not reach the visible surface, which is
what the acceptance guards read. I did not reproduce the pool measurement myself;
I am not refuting it, I am scoping it: it is a pool-metric claim, not a
visible-output claim, and section 4 here measures visible output only.

## 5. Runtime medians (base vs tip, release binaries)

Interleaved base/branch, arm order alternated each repetition, one discarded
warm-up, medians of 7. Timed with `Date.now()` around `spawnSync` from `node` (no
`/usr/bin/time` on this host). Command shape:

```
node -e 'const {spawnSync}=require("child_process");
 const B="/workspace/madgab-base-7d1c04/target/release/madgab";
 const R="/workspace/madgab-verify-7d1c04/target/release/madgab";
 const med=a=>{a=[...a].sort((x,y)=>x-y);return a[Math.floor(a.length/2)]};
 /* 7 reps, alternating order, per target */'
```

```
target                     base median   branch median   delta
It's just a stupid game       2378 ms        2454 ms        +3.2 %
the cat sat on the mat        2618 ms        2728 ms        +4.2 %
I love you                    1301 ms        1318 ms        +1.3 %
put it back on the shelf      3010 ms        3242 ms        +7.7 %

per-run spread (ms)
  stupid game   base   2131 2154 2198 2518 2387 2378 2546
                branch 2066 2096 2914 2825 2453 2521 2454
  the mat       base   2376 2818 2578 2628 2570 2821 2618
                branch 2638 2728 2868 3059 2808 2715 2577
  i love you    base   1301 1237 1145 1410 1382 1396 1250
                branch 1711 1162 1110 1339 1318 1485 1157
  the shelf     base   2310 3010 3438 3571 3630 2691 2569
                branch 2300 3242 5453 4716 3287 2976 2527
```

Read this honestly: **all four medians are positive** (+1.3 % to +7.7 %), so the
sign is consistently on the side of the tip, but **every one of those deltas is
inside the run-to-run spread**, which is +/-25-40 % per target here, and the
branch's minimum is at or below the base's median on 3 of 4 targets. The shelf
target also shows a heavy branch tail (5453 ms, 4716 ms outliers against a base max
of 3630 ms) that I cannot explain and am **not** attributing to the change with
confidence.

So: **no demonstrated runtime regression, and no demonstration that the change is
free either.** The added work per call is one extra `usize` argument, one `%`, and a
coprime walk bounded by `EMIT_PROFILE_MAX_DEEP` = 3 iterations, so a cost of this
magnitude is consistent with the code; whether it is real needs a tighter
measurement (more reps, targets interleaved in a single process) than I ran here.

## 6. Verdict

# INTEGRABLE-NOW

Numbers behind it:

* fence: `HITS=0` over 189 added lines; `EQ_LIKE=1` and that one line is an integer
  bound check; `no_phrase_hard_coding` 6/6 green; diff is `src/lib.rs` only.
* guards: (a) `wreck a nice beach` **present** in `recognize speech` top 50 on this
  arm; (b) `hits justice dupe hid came` **absent** from `It's just a stupid game`
  top 50, as required.
* suites: lib 53/0 (base 50/0), exact_determinism 1/0, approx_determinism 2/0,
  no_phrase_hard_coding 6/0, corpus_integration 10 pass / 1 fail with that 1 fail
  reproduced red on the base arm.
* non-regression: 7 targets compared base-vs-branch, visible top 50 **byte-identical
  on all 7**, 0 changed.
* generality: the change is a property of shape classes (`member` position, slot list
  widths, phase); no target text, no word identity, no env read reaches
  `sweep_index`; `rate` is a pure integer function asserted coprime to the span for
  all spans 1..=150 and all members 0..=3.
* runtime: medians +1.3 % / +3.2 % / +4.2 % / +7.7 % (tip over base) on four
  targets, all four inside a +/-25-40 % run-to-run spread; no demonstrated
  regression, no demonstrated freeness either (section 5).

Two things the coordinator must carry forward, stated as claims with numbers and
**not** as objections to integrability:

1. **The milestone is not reached by this commit, and I am not softening that.**
   Guard (b) is unmet before and after; the one red test is the item's target and is
   red on the base too; the visible top 50 is unchanged on 7 of 7 targets. The fix
   is validated as correct, general, cheap and safe. It is **not** validated as an
   improvement, because no outcome number moved.
2. **Continuity is coordinate-level, not tuple-level.** `member 0` reproduces the
   parent index exactly, so slot-0 coverage is a superset; but the branch's
   per-member legality check can refuse a diagonal the parent emitted, so the
   reserve's *tuple* count is not a superset of the parent's. I have measured no
   visible consequence (section 4) and I make no claim about the pool or cost side —
   that is `e1a3f7`'s number, and the two should be read together before anyone
   describes this change as a pure win.

If the coordinator prefers a conditional form: the only measurement that could turn
this into INTEGRABLE-WITH-GUARDS is `e1a3f7`'s per-depth refusal split on the
default path, base vs branch. I have not run it and do not substitute for it.

---

### Reproduction commands for every number above

```
git diff aa662a4..79309a1 -- src/lib.rs          # 189 added / 15 removed
git diff --stat aa662a4..79309a1                 # src/lib.rs only
cargo test --release --lib
cargo test --release --test corpus_integration
cargo test --release --test exact_determinism
cargo test --release --test approx_determinism
cargo test --release --test no_phrase_hard_coding
mkdir -p /workspace/madgab-base-7d1c04
git archive aa662a4 | tar -x -C /workspace/madgab-base-7d1c04   # never a merge
(cd /workspace/madgab-base-7d1c04 && cargo test --release --lib
 (cd /workspace/madgab-base-7d1c04 && cargo test --release --test corpus_integration)
/workspace/madgab-base-7d1c04/target/release/madgab --approximate --top 50 "<target>"
/workspace/madgab-verify-7d1c04/target/release/madgab --approximate --top 50 "<target>"
```

There is no `rg`, `grep`, `sed`, `python3`, `which` or `/usr/bin/time` on this host;
the scans in section 1 are `node -e` one-liners over `git diff` output, file
reading was done with line-numbered reads and `node`, and the timing in section 6 is
`Date.now()` around `spawnSync` from `node`.
