---
work_item: true
id: w-d4e8b1
state: done
priority: high
owner: coord-5f2b (claimed 2026-09-27T20:26Z; front agent-d4e8b1 launched 20:26Z, terminal succeeded, verdict HOLD) / integration by coord-c1d4a (reconciliation pass 2026-09-27T20:58Z-21:20Z, report integrated as 31e3073)
updated: 2026-09-27T21:20:00Z
branch: madgab-phon-d4e8b1
worktree: /workspace/madgab-phon-d4e8b1
opened_by: coord-5f2b (reconciliation pass 2026-09-27T20:21Z-20:26Z)
source_item: docs/work/OBSTRUCTION-MAP.md §3 conclusion (all three remaining shapes now priced; the argued next move is a general search-quality front, not another case-2 reach attempt)
base: 44b427f (post-milestone-acceptance, pushed)
---

# Price the phonetic-cost term: why the canonical case-2 alignment scores where it does

## Goal

Decompose and price **only the phonetic-cost term** of the approximate score, and answer one
question: is the canonical case-2 alignment (`hits justice dupe hid came` for
`It's just a stupid game`) held out of the proposal set by a **general pronunciation /
phonetic-distance defect**, or is its deficit fully explained by the two coordinates already
priced (objective axes, and stage-4 slot/heap retention)?

This is a **measurement-and-pricing front**. A general, phrase-free rule is the deliverable if one
exists and survives pricing; a priced negative is an equally acceptable deliverable.

## Why this front now

[../OBSTRUCTION-MAP.md](../OBSTRUCTION-MAP.md) §3 is now exhausted on the search side: shape 1 went
priced-negative at `91c0e35` ([REPORT-3e91a4.md](../REPORT-3e91a4.md)), shape 2 is being priced by
[ w-2f1c03](w-2f1c03.md) (agent `2f1c031`, live), and shape 3 was priced by
[REPORT-9e2b41.md](../REPORT-9e2b41.md) and [REPORT-5d9c04.md](../REPORT-5d9c04.md). The map's own
stated conclusion is that if all three fail the honest next move is a **general search-quality
item** and landing [w-9b4a15](w-9b4a15.md)'s `SHAPE -> PARSIMONY` head fix — not another case-2
reach attempt. The phonetic-cost term is the one large component of the score that **no front has
priced**: `REPORT-3a8c05.md` priced five objective axes and `REPORT-6b2e19.md` decomposed score
structure, but neither isolated the per-phone cost model, and the code itself flags it as a suspect
site (`src/lib.rs:2943` "edit cost for all of them, so at equal per-phone phonetic", `src/lib.rs:3016`
"phonetically excellent and humanly unusable", `src/lib.rs:3810` and `src/lib.rs:4124` on alignments
that consume or discard material). The deficit being explained is concrete and repeatedly measured:
the canonical case-2 alignment scores **0.8199** against a **0.9295** cutoff, a gap of **0.0944**
(see [w-3a8c05.md](w-3a8c05.md) and [w-4d7c12.md](w-4d7c12.md)); the green case-1 control
(`wreck a nice beach` for `recognize speech`) sits at 0.91995, inside the cutoff, at pool rank 27.

## The question to answer

* For the canonical case-2 alignment, how much of the 0.0944 gap is the **phonetic-cost** term
  versus every other term, measured term by term on the same candidate with everything else held
  fixed? Same measurement for the case-1 control as a green reference.
* Is the phonetic term's behaviour **general and correct** for near-homophonic spellings in general, or
  does it have a systematic, phrase-free defect (per-phone cost asymmetry, unnormalised segment
  counts, a cost floor/ceiling interacting with alignment length, a corpus/transcription
  normalisation gap, or an ordering effect) that penalises this *shape* of alignment for a reason
  unrelated to how homophonic it actually is?
* If a defect exists, what is the **cheapest general rule** that removes it, priced in candidate
  visits and wall clock, and what is the counterfactual across the other targets (both canonical
  cases plus a spread of ordinary clues) — does the green case-1 control survive?

## Method and deliverables

1. Read [../OBSTRUCTION-MAP.md](../OBSTRUCTION-MAP.md) first, then the reports behind the rows it
   cites that bear on scoring: [REPORT-3a8c05.md](../REPORT-3a8c05.md),
   [REPORT-6b2e19.md](../REPORT-6b2e19.md), [REPORT-1c7d40.md](../REPORT-1c7d40.md) §1, and
   [w-4d7c12.md](w-4d7c12.md). Do not re-derive a coordinate they already priced.
2. Reproduce the baseline first: pool 18,949; canonical 0.8207695329; cutoff 0.9151215745; gap
   0.0943520415; green case 0.9199502875 at pool rank 27. If your reproduction disagrees, record the
   disagreement rather than proceeding on the quoted numbers.
3. Instrument behind `#[cfg(test)]` only, one probe at a time, and record every number.
4. Write `docs/work/REPORT-d4e8b1.md` with a term-by-term table, the general rule (if any) stated as a
   function of the span, its slots, its candidate set and its phone strings **only**, the
   counterfactual table, and exactly **one** end-state verdict: `INTEGRATE` or `HOLD`.

## Constraints

* **No phrase-specific hard-coding.** No special-casing `hid`, `dupe`, `hits`, `came`, `justice`, or
  any literal token, clue, or sentence, and no per-input branch. A rule is only acceptable if it is
  expressible over general properties and helps other inputs. The `no_phrase_hard_coding` test
  (9/9) is a hard fence.
* **Non-contending surface.** This front owns the **phonetic/pronunciation cost term only**. Do **not**
  touch the objective axes (owned by the live front `agent-9b4a153` in
  `/workspace/madgab-parsimony-9b4a15`, which is editing `src/lib.rs`), the reserve sweep or
  shortlist or width (owned by [w-2f1c03](w-2f1c03.md) and the priced reports), or the emission
  order. If the measurement shows the answer lives on one of those surfaces, say so and stop — that
  is a valid HOLD.
* **Memory fence.** The host OOM-killed two `w-9b4a15` predecessors today and `agent-9b4a153` is
  live. Run **one** test suite at a time; never run two suites concurrently. Do not use
  `/workspace/madgab-parsimony-9b4a15`.
* Validation fence on any change: `cargo test --release --lib` 74/0, `emit_coverage` 4/4,
  `approx_determinism` 4/4, `exact_determinism` 1/1, `no_phrase_hard_coding` 9/9, and
  `corpus_integration` 12/1 with the known pre-existing red
  `approximate_finds_classic_madgab_resegmentation` **not to be re-pinned**. `cargo fmt` and
  `cargo clippy` cannot run on this host — see [../environment-notes.md](../../environment-notes.md) —
  so do not claim them.
* Commit and push `madgab-phon-d4e8b1` as soon as any real change is green, with probe
  instrumentation confined to an unpushed scratch branch. **Do not self-merge** into
  `post-milestone-acceptance`; leave integration to a review front.
* If the tree is red at a prompt boundary, fix the build or move the dump behind `#[cfg(test)]`
  after the `Partial -> Clue` conversion in `finish` before continuing.

## Completion criteria

1. The baseline above is reproduced, or the disagreement with it is recorded with both numbers.
2. A term-by-term decomposition of the canonical case-2 alignment's 0.0944 deficit, with the
   phonetic-cost term's share quantified, plus the same measurement for the case-1 control.
3. Either (a) a **general**, phrase-free candidate rule for the phonetic term, stated as a function
   of the span, its slots, its candidate set and its phone strings, with a priced counterfactual
   over both canonical cases and a spread of ordinary clues, covered by a red/green test at the
   library or executable boundary; or (b) a **priced negative** stating that the deficit is not
   attributable to the phonetic term, with the numbers that show it.
4. `docs/work/REPORT-d4e8b1.md` committed and pushed on `madgab-phon-d4e8b1`, ending in exactly one
   `INTEGRATE` or `HOLD`, with the run-one-suite-at-a-time lesson and the phrase fence observed.
5. The branch is pushed and un-integrated, with the report and any `INTEGRATE` production commit
   identified by hash for a later review front.

## Handoff / notes

Opened by coordinator pass `coord-5f2b` at 20:21Z. Claimed and launched immediately: agent
`d4e8b1` in `/workspace/madgab-phon-d4e8b1` on branch `madgab-phon-d4e8b1`, off
`post-milestone-acceptance` at `44b427f`, with this item's full contract as the prompt.

At launch the host state was: `agent-2f1c031` (w-2f1c03) and `agent-9b4a153` (w-9b4a15) running;
`agent-3e91a41` terminal and integrated at `91c0e35`/`27f986e`. Live fronts occupy non-contending
surfaces: this one (phonetic cost), `2f1c031` (docs-only reserve coverage), `9b4a153` (objective
axes in `src/lib.rs`). The canonical case-2 alignment remains **absent, 0 of 18,949**; the green
case-1 control remains inside the top 50 at pool rank 27. Blocker remains ONE
(`approximate_finds_classic_madgab_resegmentation`, red at base, not to be re-pinned).

**Next action for a later pass:** inspect `agent-d4e8b1`'s status and log. On a terminal `INTEGRATE`,
launch an independent read-only review front before integrating anything; on `HOLD`, integrate the
report as a priced negative and let the map's conclusion stand.

### Front result (agent-d4e8b1, terminal `succeeded`, exit 0, 1 prompt, 35m)

**Verdict: HOLD** — a priced negative, which criterion 3(b) accepts as a full deliverable. Report
[../REPORT-d4e8b1.md](../REPORT-d4e8b1.md) pushed as `ecbf346` on `madgab-phon-d4e8b1`; probe
instrumentation is confined to the **unpushed** scratch branch `phon-probe-d4e8b1` (`fc3a930`),
all behind `#[cfg(test)]`. Nothing was self-merged.

* Baseline reproduced exactly: pool 18,949 / 18,289, canonical 0.8207695329, cutoff
  0.9151215745, gap 0.0943520415, green case 0.9199502875 at pool rank 27, canonical absent.
  One recorded disagreement with this item's fence: `cargo test --release --lib` is **75/0**
  (9 ignored) on `44b427f`, not the 74/0 quoted here; zero failures either way.
* The **phonetic-cost term is 3.54% of the deficit** (`SIMILARITY` +0.0033385585 of
  0.0943520415). Its entire achievable range is +0.0491017462, and a *perfect* pronunciation of
  the same wording at the same cuts still scores 0.8698712792 — **0.0452502953 short**. That
  residual is `NOVELTY` +0.05, `FAMILIARITY` +0.0252, `PUNCH` +0.02, `CLOSED_CLASS` −0.0042: the
  already-priced objective axes, i.e. another front's surface, so the front stopped there — a
  valid HOLD under the non-contending clause.
* Five phrase-free cost models × two normalisers were priced over both canonical cases plus ten
  ordinary targets. The only model that reaches the cutoff does so by making the term vacuous
  (the segment-substitution component is **0.000000 across all 18,949 pool clues**) and it drops
  the green case from rank 27 to **1006**. The term also does not punish the canonical's shape:
  mean raw similarity *rises* with word count and the canonical sits at the 25.65th percentile of
  five-word pool clues.

### Coordinator integration (coord-c1d4a, pass 2026-09-27T20:58Z-21:20Z)

* `ecbf346` merged `--no-ff` into `post-milestone-acceptance` as `31e3073` and pushed (head
  `515f8bd` then `see log`). The branch is docs-only by construction: `git diff --stat
  8fab932..ecbf346` is `docs/work/REPORT-d4e8b1.md` alone, +326 lines, zero `src/` lines, so no
  review front was needed for a docs-only priced negative.
* Re-validated the integrated head independently: `cargo test --release --lib` **75 passed /
  0 failed / 12 ignored**; the other fences re-run this pass are recorded on
  [w-4b1e07](w-4b1e07.md)'s latest pass note.
* The item closes `done`: all four of its shapes of criterion 3 are satisfied by the priced
  negative, criterion 1 (baseline reproduced) and criterion 4 (report pushed, un-integrated when
  the front stopped) are verified in the report and above. The scratch branch
  `phon-probe-d4e8b1` stays unpushed by design.
* **Successor, already opened by this pass:** the objective-axis residue the front declined to
  touch (the `NOVELTY`/`FAMILIARITY`/`PUNCH` 0.0952 of the deficit) is exactly what
  [w-3f8c62](w-3f8c62.md) lands — the word-count parsimony axis priced by REPORT-9b4a15 §6 —
  and the reserve-side residue REPORT-2f1c03 identified (the per-member modulus in
  `coverage_tuples`) is recorded there as a later, non-blocking front.
