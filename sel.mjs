import fs from 'node:fs';

const STRUCTURE_FLOOR = 3;

function load(path) {
  const lines = fs.readFileSync(path, 'utf8').trim().split('\n');
  return lines.map((l) => {
    const t = l.split('\t');
    return { i: +t[0], score: +t[1], phrase: t[2], cuts: JSON.parse(t[3]) };
  });
}

function structOf(c) {
  return c.cuts.slice(0, c.cuts.length - 1).join(',');
}

function order(pool) {
  const o = pool.map((c, k) => k);
  o.sort((a, b) => {
    const d = pool[b].score - pool[a].score;
    if (d !== 0) return d;
    return pool[a].phrase < pool[b].phrase ? -1 : pool[a].phrase > pool[b].phrase ? 1 : 0;
  });
  return o;
}

function shareCap(topN, available) {
  const floor = Math.min(STRUCTURE_FLOOR, Math.max(available, 1));
  return Math.max(Math.ceil(topN / floor), 2, Math.ceil(topN / Math.max(available, 1)));
}

// current policy
function current(order, pool, structs, topN) {
  const taken = new Array(pool.length).fill(false);
  const counts = new Map();
  const represented = new Set();
  const picked = [];
  const admit = (i) => { picked.push(i); taken[i] = true; const s = structs[i]; counts.set(s, (counts.get(s) ?? 0) + 1); };
  const cutoff = order.slice(0, topN);
  for (const i of cutoff) {
    if (picked.length === topN) break;
    if (!represented.has(structs[i])) { represented.add(structs[i]); admit(i); }
  }
  const available = new Set(structs).size;
  const cap = shareCap(topN, available);
  for (const i of order) {
    if (picked.length === topN) break;
    if (taken[i]) continue;
    if ((counts.get(structs[i]) ?? 0) < cap) admit(i);
  }
  for (const i of order) {
    if (picked.length === topN) break;
    if (!taken[i]) admit(i);
  }
  return { picked, cap, available, fillRank: fillRank(picked, order) };
}

// coverage-first policy
function coverageFirst(order, pool, structs, topN) {
  const taken = new Array(pool.length).fill(false);
  const counts = new Map();
  const represented = new Set();
  const picked = [];
  const admit = (i) => { picked.push(i); taken[i] = true; const s = structs[i]; counts.set(s, (counts.get(s) ?? 0) + 1); };
  for (const i of order) {
    if (picked.length === topN) break;
    if (!represented.has(structs[i])) { represented.add(structs[i]); admit(i); }
  }
  const available = new Set(structs).size;
  const cap = shareCap(topN, available);
  for (const i of order) {
    if (picked.length === topN) break;
    if (taken[i]) continue;
    if ((counts.get(structs[i]) ?? 0) < cap) admit(i);
  }
  for (const i of order) {
    if (picked.length === topN) break;
    if (!taken[i]) admit(i);
  }
  return { picked, cap, available, fillRank: fillRank(picked, order) };
}

function fillRank(picked, order) {
  const pos = new Map(order.map((v, k) => [v, k]));
  return 1 + Math.max(...picked.map((i) => pos.get(i)));
}

function visiblePhrases(res, pool) {
  return res.picked.map((i) => pool[i].phrase).sort();
}

function metrics(name, res, pool, structs, order, topN) {
  const shown = new Set(res.picked.map((i) => structs[i]));
  const counts = new Map();
  for (const i of res.picked) counts.set(structs[i], (counts.get(structs[i]) ?? 0) + 1);
  const fr = res.fillRank;
  let underCapBeyond = 0;
  let underCapBefore = 0;
  let repBeyond = 0;
  for (let r = 0; r < order.length; r++) {
    const s = structs[order[r]];
    const c = counts.get(s) ?? 0;
    if (c < res.cap) {
      if (r + 1 <= fr) underCapBefore++; else underCapBeyond++;
    }
    if (shown.has(s) && c < res.cap && r + 1 > fr) repBeyond++;
  }
  const worst = Math.min(...res.picked.map((i) => pool[i].score));
  // "under-cap" as w-7b2d40 measured it: within-structure rank <= cap.
  // This set depends only on the pool and the cap, not on the order.
  const inU = new Set();
  const byStruct = new Map();
  order.forEach((i) => {
    const s = structs[i];
    const n = (byStruct.get(s) ?? 0) + 1;
    byStruct.set(s, n);
    if (n <= res.cap) inU.add(i);
  });
  const shownSet = new Set(res.picked);
  let uBeyond = 0, uBefore = 0;
  for (const i of inU) {
    const pos = order.indexOf(i) + 1;
    if (pos > fr) uBeyond++; else uBefore++;
  }
  return {
    name, pool: pool.length, structures: res.available, cap: res.cap,
    visibleStructures: shown.size, fillRank: fr,
    underCapSetSize: inU.size,
    underCapInvisibleBeyondFillRank: uBeyond,
    underCapInvisibleWithinFillRank: uBefore,
    underCapInvisibleBeyondAllStructures: underCapBeyond,
    underCapInvisibleBeyondShownStructures: repBeyond,
    underCapVisibleOrBefore: underCapBefore,
    worstVisibleScore: worst,
    _shown: shownSet.size,
  };
}

const targets = process.argv.slice(2);
for (const t of targets) {
  const [poolPath, visPath, topNStr] = t.split(',');
  const topN = +topNStr;
  const pool = load(poolPath);
  const structs = pool.map(structOf);
  const ord = order(pool);
  const now = current(ord, pool, structs, topN);
  const cov = coverageFirst(ord, pool, structs, topN);
  // validate current against the real CLI output
  const real = fs.readFileSync(visPath, 'utf8').split('\n')
    .map((l) => l.trim().match(/^\d+\.\s\[[^\]]*\]\s(.*)$/))
    .filter(Boolean).map((m) => m[1]).sort();
  const got = visiblePhrases(now, pool);
  const ok = JSON.stringify(real) === JSON.stringify(got);
  console.log(`== ${poolPath} top_n=${topN} model_matches_cli=${ok}`);
  if (!ok) {
    console.log('  real:', real.slice(0, 5).join(' | '));
    console.log('  model:', got.slice(0, 5).join(' | '));
    console.log('  realN', real.length, 'modelN', got.length);
    console.log('  missing', real.filter((p) => !got.includes(p)).slice(0, 5));
    console.log('  extra', got.filter((p) => !real.includes(p)).slice(0, 5));
  }
  console.log('  ' + JSON.stringify(metrics('current', now, pool, structs, ord, topN)));
  console.log('  ' + JSON.stringify(metrics('coverage-first', cov, pool, structs, ord, topN)));
}
