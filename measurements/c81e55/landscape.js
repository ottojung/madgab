// c81e55: the objective's landscape around the requested clue.
const { load, score } = require("./analyze.js");
const D = load(process.argv[2]);
const target = process.argv[3].split(" ");

const wi = new Map();
for (const [segId, slots] of D.alts) {
  slots.forEach((alts, s) => { const m = new Map(); for (const a of alts) m.set(a.word, a.i); wi.set(segId + ":" + s, m); });
}
let rows = [];
for (const [segId, slots] of D.alts) {
  if (slots.length !== target.length) continue;
  const idx = []; let ok = true;
  for (let s = 0; s < target.length; s++) { const m = wi.get(segId + ":" + s); if (!m || !m.has(target[s])) { ok = false; break; } idx.push(m.get(target[s])); }
  if (ok) rows.push({ segId, idx, s: score(D, segId, idx), w: slots.map((a, k) => a[idx[k]].word) });
}
for (const r of rows) {
  console.log(`\n=== seg ${r.segId}: requested ${r.w.join(" ")}  bound ${r.s.toFixed(4)} ===`);
  console.log("slot  req_idx req_word      argmax_idx argmax_word    argmax_bound   gain   #cands_better");
  let worse = 0;
  for (let k = 0; k < r.idx.length; k++) {
    const A = D.alts.get(r.segId)[k];
    let bi = 0, bs = -1;
    for (let i = 0; i < A.length; i++) {
      const t = r.idx.slice(); t[k] = i;
      const v = score(D, r.segId, t);
      if (v > bs) { bs = v; bi = i; }
    }
    let better = 0;
    for (let i = 0; i < A.length; i++) {
      const t = r.idx.slice(); t[k] = i;
      if (score(D, r.segId, t) > r.s) better++;
    }
    worse += better;
    console.log(String(k).padStart(4), String(r.idx[k]).padStart(8), A[r.idx[k]].word.padEnd(13),
      String(bi).padStart(10), A[bi].word.padEnd(14), bs.toFixed(4).padStart(12),
      (bs - r.s).toFixed(4).padStart(7), String(better).padStart(8), `  (slot width ${A.length})`);
  }
  console.log("  single-slot substitutions of the requested wording that beat it, summed over slots:", worse);
  // the best 2-slot greedy ascent
  let cur = r.idx.slice(), curS = r.s, steps = 0;
  for (;;) {
    let best = null;
    for (let k = 0; k < cur.length; k++) {
      const A = D.alts.get(r.segId)[k];
      for (let i = 0; i < A.length; i++) {
        const t = cur.slice(); t[k] = i;
        const v = score(D, r.segId, t);
        if (v > curS + 1e-12 && (!best || v > best.v)) best = { k, i, v };
      }
    }
    if (!best) break;
    cur[best.k] = best.i; curS = best.v; steps++;
  }
  console.log("  greedy single-slot ascent:", steps, "steps to", curS.toFixed(4), "->",
    cur.map((i, k) => D.alts.get(r.segId)[k][i].word).join(" "));
}
