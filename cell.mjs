import fs from 'node:fs';
// usage: node cell.mjs <pool.tsv> <vis.txt> <wanted phrase>
const [poolPath, visPath, wanted] = process.argv.slice(2);
const pool = fs.readFileSync(poolPath, 'utf8').trim().split('\n').map((l) => {
  const t = l.split('\t');
  return { score: +t[1], phrase: JSON.parse(t[2]), cuts: JSON.parse(t[3]) };
});
const S = (c) => c.cuts.slice(0, -1).join(',');
const ord = pool.map((_, k) => k).sort((a, b) =>
  (pool[b].score - pool[a].score) ||
  (pool[a].phrase < pool[b].phrase ? -1 : pool[a].phrase > pool[b].phrase ? 1 : 0));
const pos = new Map(ord.map((v, k) => [v, k]));
const vis = fs.readFileSync(visPath, 'utf8').split('\n')
  .map((l) => l.trim().match(/^\d+\.\s\[[^\]]*\]\s(.*)$/))
  .filter(Boolean).map((m) => m[1]);
const visRank = new Map(vis.map((p, k) => [p, k + 1]));

const structOrder = [];
const seen = new Set();
for (const i of ord) { const s = S(pool[i]); if (!seen.has(s)) { seen.add(s); structOrder.push(s); } }

const ci = pool.findIndex((c) => c.phrase === wanted);
const out = {
  pool: pool.length,
  structures: structOrder.length,
  visibleStructures: new Set(vis.map((p) => S(pool.find((c) => c.phrase === p)))).size,
  cutoffScore: +pool[ord[49]].score.toFixed(6),
  inPool: ci >= 0,
};
if (ci >= 0) {
  const st = S(pool[ci]);
  const sib = ord.filter((i) => S(pool[i]) === st);
  out.wantedGlobalRank = pos.get(ci) + 1;
  out.wantedWithinStructureRank = sib.indexOf(ci) + 1;
  out.structureMembers = sib.length;
  out.structureBestMemberRank = structOrder.indexOf(st) + 1;
  out.structureBestMemberScore = +pool[ord.find((i) => S(pool[i]) === st)].score.toFixed(6);
  out.wantedScore = +pool[ci].score.toFixed(9);
  out.visibleRank = visRank.get(wanted) ?? null;
  out.green = visRank.has(wanted);
} else {
  out.green = false;
  out.wordsInPool = Object.fromEntries(wanted.split(' ').map((w) => [w, pool.filter((c) => c.phrase.split(' ').includes(w)).length]));
}
console.log(JSON.stringify(out));
