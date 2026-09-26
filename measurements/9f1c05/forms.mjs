// Offline aggregation-form evaluator for front 9f1c05.
// Operates on a CAPTURED pool (see REPORT s2 for how it was captured).  No
// re-running of the search per experiment.  Nothing here names a phrase as a
// special case: the requested wording is selected by an index computed from
// the data, and every form is a function of the seven axis values only.
import fs from 'fs';

export const AX = ['SIM', 'NOV', 'WNOV', 'FAM', 'RHY', 'SHAPE', 'CLOSED'];
export const W = [0.25, 0.15, 0.15, 0.10, 0.30, 0.05, -0.15];
export const TOPN = 50;

export function loadPool(file) {
  const raw = fs.readFileSync(file, 'utf8').split('\n').filter(x => x.startsWith('AGGDUMP')).map(l => l.split('\t'));
  const rows = raw.map(c => ({ phrase: c[1], v: c.slice(2, 9).map(Number), score: +c[9], syl: +c[10], sig: c[11] }));
  rows.sort((a, b) => b.score - a.score || (a.phrase < b.phrase ? -1 : a.phrase > b.phrase ? 1 : 0));
  const seen = new Set(), pool = [];
  for (const r of rows) { if (seen.has(r.sig)) continue; seen.add(r.sig); pool.push(r); }
  return pool;
}

// A stable identity of a candidate for churn accounting: the phrase's spelling
// signature, i.e. what the production deduplicator itself treats as one clue.
export const id = r => r.sig;
export const base = r => W.reduce((s, w, i) => s + w * r.v[i], 0);

export function rawTop(pool, scoreFn, topn = TOPN) {
  const idx = pool.map((_, i) => i).sort((a, b) => scoreFn(pool[b]) - scoreFn(pool[a]) || (pool[a].phrase < pool[b].phrase ? -1 : 1));
  return idx;
}

// `wreck a nice beach`-style guard probe: the phrase under test is passed in
// by the caller as a plain string; the code has no knowledge of it.
export function measure(pool, scoreFn, probe, topn = TOPN) {
  const idx = rawTop(pool, scoreFn, topn);
  const pi = pool.findIndex(r => r.phrase === probe);
  const rk = pi < 0 ? -1 : idx.indexOf(pi);
  const band = idx.slice(0, topn);
  const s = scoreFn;
  const cut = s(pool[band[topn - 1]]);
  return {
    rank: rk,
    score: rk < 0 ? null : s(pool[rk]),
    cutoff: cut,
    margin: rk < 0 ? null : s(pool[rk]) - cut,
    bandTop: s(pool[band[0]]),
    bandWidth: s(pool[band[0]]) - cut,
    band: band.map(i => pool[i]),
    bandPhrases: band.map(i => pool[i].phrase),
  };
}

export function churn(a, b) {
  const A = new Set(a.map(id)), B = new Set(b.map(id));
  let kept = 0; for (const x of A) if (B.has(x)) kept++;
  return { kept, churned: A.size - kept, sizeA: A.size, sizeB: B.size };
}

// c81e55's full-resegmentation rule, verbatim: two words are the same word if
// equal, or both >= 3 letters and one is a prefix of the other; a proposal is a
// full resegmentation if no clue word is the same word as any target word.
export function isFullReseg(phrase, targetWords) {
  const cw = phrase.toLowerCase().split(/\s+/).filter(Boolean);
  const tw = targetWords.map(w => w.toLowerCase());
  const same = (a, b) => a === b || (a.length >= 3 && b.length >= 3 && (a.startsWith(b) || b.startsWith(a)));
  return !cw.some(c => tw.some(t => same(c, t)));
}

// Function-word-salad count.  Definition stated in the report: a visible
// proposal is a salad when the closed-class share of its words exceeds 1/3,
// i.e. strictly more than one word in three is a function word.  The share is
// axis value 7 is `closed_penalty` = (closed/words)^2, so share = sqrt(v[6]).
export function closedShare(r) { return Math.sqrt(r.v[6]); }
export const isSalad = r => closedShare(r) > 1 / 3 + 1e-12;
