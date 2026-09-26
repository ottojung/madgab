import fs from 'node:fs';
const STRUCTURE_FLOOR = 3;

function load(path) {
  return fs.readFileSync(path, 'utf8').trim().split('\n').map((l) => {
    const t = l.split('\t');
    return { score: +t[1], phrase: JSON.parse(t[2]), cuts: JSON.parse(t[3]) };
  });
}
const S = (c) => c.cuts.slice(0, -1).join(',');

function orderOf(pool) {
  const o = pool.map((_, k) => k);
  o.sort((a, b) => (pool[b].score - pool[a].score) ||
    (pool[a].phrase < pool[b].phrase ? -1 : pool[a].phrase > pool[b].phrase ? 1 : 0));
  return o;
}
function shareCap(topN, available) {
  const floor = Math.min(STRUCTURE_FLOOR, Math.max(available, 1));
  return Math.max(Math.ceil(topN / floor), 2, Math.ceil(topN / Math.max(available, 1)));
}

// coverageBudget: number of slots the representative phase may claim.
function run(order, pool, structs, topN, coverageBudget) {
  const taken = new Array(pool.length).fill(false);
  const counts = new Map(), represented = new Set(), picked = [];
  const admit = (i) => { picked.push(i); taken[i] = true; const s = structs[i]; counts.set(s, (counts.get(s) ?? 0) + 1); };
  const cap = shareCap(topN, new Set(structs).size);
  for (const i of order) {
    if (picked.length >= Math.min(coverageBudget, topN)) break;
    if (!represented.has(structs[i])) { represented.add(structs[i]); admit(i); }
  }
  const coverageSlots = picked.length;
  for (const i of order) {
    if (picked.length === topN) break;
    if (taken[i]) continue;
    if ((counts.get(structs[i]) ?? 0) < cap) admit(i);
  }
  for (const i of order) {
    if (picked.length === topN) break;
    if (!taken[i]) admit(i);
  }
  const pos = new Map(order.map((v, k) => [v, k]));
  const fillRank = 1 + Math.max(...picked.map((i) => pos.get(i)));
  const inU = new Set();
  const byS = new Map();
  for (const i of order) {
    const s = structs[i], n = (byS.get(s) ?? 0) + 1; byS.set(s, n);
    if (n <= cap) inU.add(i);
  }
  let uBeyond = 0;
  for (const i of inU) if (pos.get(i) + 1 > fillRank) uBeyond++;
  return {
    cap, coverageSlots, depthSlots: picked.length - coverageSlots,
    structures: new Set(picked.map((i) => structs[i])).size,
    fillRank, underCapBeyond: uBeyond, underCapTotal: inU.size,
    worst: Math.min(...picked.map((i) => pool[i].score)),
    phrases: picked.map((i) => pool[i].phrase),
    cuts: picked.map((i) => structs[i]),
  };
}

const guard = {
  pool1: 'wreck a nice beach', pool2: 'hits justice dupe hid came',
};
for (const [key, file] of Object.entries({ pool1: 'pool1.tsv', pool2: 'pool2.tsv', pool_love10: 'pool_love10.tsv' })) {
  const pool = load(file);
  const structs = pool.map(S);
  const ord = orderOf(pool);
  const topN = key === 'pool_love10' ? 10 : 50;
  const cap = shareCap(topN, new Set(structs).size);
  console.log(`== ${file} top_n=${topN} cap=${cap} pool=${pool.length} structures=${new Set(structs).size}`);
  for (const c of [topN, topN - 1, topN - cap, Math.ceil(topN * 2 / 3), Math.ceil(topN / 2), 1]) {
    if (c < 1 || c > topN) continue;
    const r = run(ord, pool, structs, topN, c);
    const g = guard[key];
    console.log(`  coverage=${String(c).padStart(3)} depth=${String(r.depthSlots).padStart(3)} structures=${String(r.structures).padStart(3)} fillRank=${String(r.fillRank).padStart(5)} underCapBeyond=${r.underCapBeyond} worst=${r.worst.toFixed(6)} guard_present=${g ? r.phrases.includes(g) : 'n/a'}`);
  }
}
