// c81e55 analysis.  Reads a probe TSV and answers:
//  A. the reserve's class-walk allocation as a function of the slot count
//  B. the score distribution of reserve-built vs traversal-built tuples
//  C. the score of the requested clue and where it would have to rank
//  D. the score-rank of the requested clue's words inside their slots
const fs = require("fs");

const AX = { SIM: 0.25, NOV: 0.15, WNOV: 0.15, FAM: 0.10, RHY: 0.30, SHAPE: 0.05 };
const CLOSED_CLASS_WEIGHT = 0.10;

function rhythm(lo, hi, target) {
  const closest = target < lo ? lo - target : target > hi ? target - hi : 0;
  const spread = Math.max(1, hi - lo);
  return Math.exp(-closest / spread);
}
function closedPenalty(closed, words) {
  if (words <= 0) return 0;
  const s = Math.min(1, Math.max(0, closed / words));
  return s * s;
}

function load(path) {
  const lines = fs.readFileSync(path, "utf8").split("\n");
  const segs = new Map();
  const alts = new Map();
  const tuples = [];
  for (const l of lines) {
    if (!l) continue;
    const f = l.split("\t");
    if (f[0] === "SEG") {
      const m = f[9].match(/^([0-9\-]+(?:,[0-9\-]+)*),([0-9]+(?:,[0-9]+)*)$/);
      if (!m) throw new Error("bad SEG field: " + f[9]);
      segs.set(+f[1], {
        id: +f[1], phase: +f[2], nslots: +f[3], wordCount: +f[4],
        targetSyll: +f[5], shared: +f[6], tic: +f[7], novelty: +f[8],
        spans: m[1].split(","), widths: m[2].split(",").map(Number),
      });
    } else if (f[0] === "ALT") {
      const k = +f[1];
      if (!alts.has(k)) alts.set(k, []);
      if (!alts.get(k)[+f[2]]) alts.get(k)[+f[2]] = [];
      alts.get(k)[+f[2]][+f[3]] = { cost: +f[4], reused: +f[5], fam: +f[6], closed: +f[7], shape: +f[8], syl: +f[9], word: f[10], i: +f[3] };
    } else if (f[0] === "TUPLE") {
      tuples.push({ seg: +f[1], src: f[2], ok: f[3] === "1", idx: f[4].split(",").map(Number) });
    }
  }
  return { segs, alts, tuples };
}

// exact reconstruction of `bound` for a *complete* index tuple
function score(D, segId, idx) {
  const s = D.segs.get(segId);
  const A = D.alts.get(segId);
  let cost = 0, reused = 0, fam = 0, closed = 0, shape = 0, syl = 0;
  for (let k = 0; k < idx.length; k++) {
    const a = A[k][idx[k]];
    cost += a.cost; reused += a.reused; fam += a.fam; closed += a.closed; shape += a.shape; syl += a.syl;
  }
  const wc = s.wordCount;
  return AX.SIM * (1 - Math.min(1, Math.max(0, cost / 4)))
    + AX.NOV * s.novelty
    + AX.WNOV * (1 - reused / wc)
    + AX.FAM * (fam / wc)
    - CLOSED_CLASS_WEIGHT * closedPenalty(closed, wc)
    + AX.RHY * rhythm(syl, syl, s.targetSyll)
    + AX.SHAPE * (shape / wc);
}

function cost(D, segId, idx) {
  const A = D.alts.get(segId);
  let c = 0;
  for (let k = 0; k < idx.length; k++) c += A[k][idx[k]].cost;
  return c;
}

module.exports = { load, score, cost, rhythm, closedPenalty, AX, CLOSED_CLASS_WEIGHT };

if (require.main === module) {
  const D = load(process.argv[2]);
  const segs = [...D.segs.values()];
  console.log("segmentations:", segs.length, " tuples offered:", D.tuples.length);
  const byN = new Map();
  for (const s of segs) {
    if (!byN.has(s.nslots)) byN.set(s.nslots, []);
    byN.get(s.nslots).push(s);
  }
  console.log("\n--- A. slot-count distribution and the class-walk prefix sums ---");
  console.log("nslots  #segs   cum1 cum2 cum3 cum4  deepest stratum reachable with allowance 16");
  for (const n of [...byN.keys()].sort((a, b) => a - b)) {
    let cum = 0, deep = 0;
    for (let k = 1; k <= n; k++) {
      const comb = nCr(n, k);
      if (cum + comb > 16) break;
      cum += comb; deep = k;
    }
    console.log(`${String(n).padStart(5)}  ${String(byN.get(n).length).padStart(6)}   ${[1, 2, 3, 4].map(k => String(nCr(n, k)).padStart(4)).join(" ")}   depth ${deep}`);
  }
  function nCr(n, k) { let r = 1; for (let i = 0; i < k; i++) r = r * (n - i) / (i + 1); return Math.round(r); }

  console.log("\n--- B. score of every offered tuple, by source and depth ---");
  const buckets = new Map();
  for (const t of D.tuples) {
    if (!t.ok) continue;
    const d = t.idx.filter(x => x !== 0).length;
    const k = t.src + "/d" + d;
    if (!buckets.has(k)) buckets.set(k, []);
    buckets.get(k).push(score(D, t.seg, t.idx));
  }
  for (const k of [...buckets.keys()].sort()) {
    const v = buckets.get(k).slice().sort((a, b) => a - b);
    const q = p => v[Math.min(v.length - 1, Math.floor(p * v.length))];
    console.log(`${k.padEnd(14)} n=${String(v.length).padStart(6)}  mean ${(v.reduce((a, b) => a + b, 0) / v.length).toFixed(4)}  p10 ${q(0.1).toFixed(4)}  p50 ${q(0.5).toFixed(4)}  p90 ${q(0.9).toFixed(4)}  max ${v[v.length - 1].toFixed(4)}`);
  }

  console.log("\n--- C. per-target top-50 cut and score band ---");
  const top = fs.readFileSync(process.argv[3], "utf8").split("\n").map(s => s.trim()).filter(Boolean);
  const scores = top.map(l => parseFloat(l.match(/\[([0-9.]+)\]/)[1]));
  console.log("top50 n=" + scores.length + "  best " + Math.max(...scores).toFixed(4) + "  50th " + scores[scores.length - 1].toFixed(4) + "  band " + (Math.max(...scores) - scores[scores.length - 1]).toFixed(4));
}
