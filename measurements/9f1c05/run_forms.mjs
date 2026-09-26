import { loadPool, measure, churn, isFullReseg, isSalad, base, W, AX, TOPN } from './forms.mjs';
import { FORMS } from './forms_def.mjs';
const PROBE = process.argv[2];
const TGTW = process.argv.slice(3);
const pool = loadPool('/tmp/opencode/agg/pool_01i.tsv');
const b = measure(pool, base, PROBE);
console.log(`pool ${pool.length} @ --top 50 | control: rank ${b.rank} score ${b.score.toFixed(9)} cutoff ${b.cutoff.toFixed(9)} margin ${b.margin.toFixed(9)} bandWidth ${b.bandWidth.toFixed(6)}`);
console.log('form'.padEnd(58), 'rank'.padStart(6), 'score'.padStart(10), 'cutoff'.padStart(10), 'margin'.padStart(11), 'w'.padStart(9), 'churn/50'.padStart(9), 'reseg'.padStart(6), 'salad'.padStart(6));
for (const F of FORMS) {
  const f = F.make(pool);
  const m = measure(pool, f, PROBE);
  const c = churn(b.band, m.band);
  const res = m.band.filter(r => isFullReseg(r.phrase, TGTW)).length;
  const sal = m.band.filter(isSalad).length;
  console.log(F.name.padEnd(58), String(m.rank).padStart(6), m.score.toFixed(9).padStart(10), m.cutoff.toFixed(9).padStart(10), m.margin.toFixed(9).padStart(11), m.bandWidth.toFixed(5).padStart(9), `${c.churned}`.padStart(9), `${res}`.padStart(6), `${sal}`.padStart(6));
}
