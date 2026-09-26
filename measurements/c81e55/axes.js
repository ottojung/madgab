// c81e55: can any reweighting of the six existing axes put the requested clue
// in the visible band, and what does it cost the green guard?
const fs = require("fs");
const { load, AX, CLOSED_CLASS_WEIGHT, rhythm, closedPenalty } = require("./analyze.js");

const D = load(process.argv[2]);
const topFile = process.argv[3];
const requested = process.argv[4].split(" ");
const guard = process.argv[5] ? process.argv[5].split(" ") : null;

function parts(D, segId, idx) {
  const s = D.segs.get(segId), A = D.alts.get(segId);
  let cost = 0, reused = 0, fam = 0, closed = 0, shape = 0, syl = 0;
  for (let k = 0; k < idx.length; k++) { const a = A[k][idx[k]]; cost += a.cost; reused += a.reused; fam += a.fam; closed += a.closed; shape += a.shape; syl += a.syl; }
  const wc = s.wordCount;
  return {
    sim: 1 - Math.min(1, Math.max(0, cost / 4)),
    nov: s.novelty, wnov: 1 - reused / wc, fam: fam / wc,
    closed: closedPenalty(closed, wc), shape: shape / wc,
    rhy: rhythm(syl, syl, s.targetSyll), cost,
  };
}
// per-segmentation minimum achievable total substitution cost: the cost of the
// all-argmin tuple, i.e. the floor the additive total_budget is measured against.
const floor = new Map();
for (const [segId, slots] of D.alts) {
  let c = 0;
  for (const alts of slots) c += alts.reduce((m, a) => Math.min(m, a.cost), Infinity);
  floor.set(segId, c);
}

const W = { sim: AX.SIM, nov: AX.NOV, wnov: AX.WNOV, fam: AX.FAM, rhy: AX.RHY, shape: AX.SHAPE, closed: -CLOSED_CLASS_WEIGHT };
const keys = Object.keys(W);
const dot = (p, w) => keys.reduce((a, k) => a + w[k] * p[k], 0);

const top = fs.readFileSync(topFile, "utf8").split("\n").map(x => x.trim()).filter(Boolean);
const printed = top.map(l => parseFloat(l.match(/\[([0-9.]+)\]/)[1])).sort((a, b) => b - a);
// The production gate is the *raw* rank-49 cutoff, read from MADGAB_TRACE_PHRASES
// on the unpatched default path, not the post-select_diverse visible floor.
const cut = parseFloat(process.env.RAW_CUT || printed[printed.length - 1]);

const wi = new Map();
for (const [segId, slots] of D.alts) slots.forEach((alts, s) => { const m = new Map(); for (const a of alts) m.set(a.word, a.i); wi.set(segId + ":" + s, m); });
function find(words) {
  let best = null;
  for (const [segId, slots] of D.alts) {
    if (slots.length !== words.length) continue;
    const idx = []; let ok = true;
    for (let s = 0; s < words.length; s++) { const m = wi.get(segId + ":" + s); if (!m || !m.has(words[s])) { ok = false; break; } idx.push(m.get(words[s])); }
    if (ok) { const p = parts(D, segId, idx); const v = dot(p, W); if (!best || v > best.v) best = { segId, idx, p, v }; }
  }
  return best;
}
const req = find(requested);
console.log("=== baseline weights ===", JSON.stringify(W));
console.log("cut (50th visible)          ", cut.toFixed(6));
console.log("requested, baseline weights ", req.v.toFixed(6), "  axes:", JSON.stringify(Object.fromEntries(keys.map(k => [k, +req.p[k].toFixed(4)]))));
console.log("  its cost", req.p.cost.toFixed(4), "segmentation floor", floor.get(req.segId).toFixed(4));

// accepted tuples, to know what a reweighting does to the visible band
const acc = D.tuples.filter(t => t.ok).map(t => ({ t, p: parts(D, t.seg, t.idx) }));

// candidate variant: similarity measured *relative to the segmentation floor*
function scoreRel(p, segId, w) {
  const rel = 1 - Math.min(1, Math.max(0, (p.cost - floor.get(segId)) / 4));
  return keys.reduce((a, k) => a + w[k] * (k === "sim" ? rel : p[k]), 0);
}
console.log("\n=== variant S1: similarity relative to the segmentation's cost floor ===");
{
  const w = { ...W };
  const band = acc.map(x => scoreRel(x.p, x.t.seg, w)).sort((a, b) => b - a);
  const r = scoreRel(req.p, req.segId, w);
  const rrank = acc.filter(x => scoreRel(x.p, x.t.seg, w) > r).length;
  console.log("requested", r.toFixed(6), "rank", rrank + 1, "of", acc.length, " 50th accepted", band[49].toFixed(6));
}

// minimum-L1 reweighting that lifts the requested clue above the cut, by grid
console.log("\n=== minimum perturbation of the six axis weights that lifts the requested clue ===");
{
  // grid over a simplex of perturbations: move t of mass out of (sim, nov, fam)
  // proportionally and give it to (wnov, rhy, shape), and separately try
  // targeted single-axis moves.  Report the cheapest by total variation.
  let best = null;
  const base = { ...W };
  const donors = ["sim", "nov", "fam", "closed"];
  const takers = ["wnov", "rhy", "shape", "nov", "sim"];
  for (let ti = 1; ti <= 100; ti++) {
    const t = ti / 100;
    for (const d of donors) for (const k of takers) {
      const w = { ...base };
      const move = t * w[d];
      w[d] -= move; w[k] += move;
      const v = dot(req.p, w);
      if (v < cut) continue;
      // what does it do to the guard?
      let gv = null;
      if (guard) { const g = find(guard); if (g) gv = dot(g.p, w); }
      const tv = move;
      if (!best || tv < best.tv) best = { tv, d, k, v, gv, w };
    }
  }
  if (best) {
    console.log("cheapest total-variation move:", (best.tv * 100).toFixed(1) + "% of the mass,",
      "from", best.d, "to", best.k, "-> requested", best.v.toFixed(6),
      best.gv === null ? "" : " guard " + best.gv.toFixed(6) + (best.gv >= cut ? " (kept)" : " (LOST)"));
    console.log("  resulting weights:", JSON.stringify(Object.fromEntries(keys.map(k => [k, +best.w[k].toFixed(4)]))));
    const r2 = acc.filter(x => dot(x.p, best.w) > best.v).length;
    console.log("  requested rank under those weights:", r2 + 1, "of", acc.length);
  } else console.log("  no single donor/taker move of up to 100% of the mass reaches the cut");
}
