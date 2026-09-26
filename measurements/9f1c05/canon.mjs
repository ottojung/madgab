import { loadPool, measure, churn, isFullReseg, isSalad, base, W, AX, TOPN } from './forms.mjs';
import { FORMS } from './forms_def.mjs';
const OFF = {F0:'OFF-A',F1:'OFF-B',F2:'OFF-C',F3:'OFF-D',F4:'OFF-E',F5:'OFF-F',F6:'OFF-G'};
const pool = loadPool('pool_01i.tsv');
const PROBE='hits justice dupe hid came';
const b = measure(pool, base, PROBE);
console.log(`# canonical target, injected pool ${pool.length} @ --top 50`);
console.log(`# control OFF-A: rank ${b.rank} score ${b.score.toFixed(9)} cutoff ${b.cutoff.toFixed(9)} margin ${b.margin.toFixed(9)} bandWidth ${b.bandWidth.toFixed(6)}`);
console.log('form\tname\trank\tscore\tcutoff\tmargin\tbandWidth\tchurn50\treseg50\tsalad50\tmaxScore');
for (const F of FORMS) {
  const f=F.make(pool); const m=measure(pool,f,PROBE); const c=churn(b.band,m.band);
  const mx=Math.max(...pool.map(r=>f(r)));
  console.log([OFF[F.name.slice(0,2)], F.name, m.rank, m.score.toFixed(9), m.cutoff.toFixed(9),
    m.margin.toFixed(9), m.bandWidth.toFixed(6), c.churned,
    m.band.filter(r=>isFullReseg(r.phrase,["it's","just","a","stupid","game"])).length,
    m.band.filter(isSalad).length, mx.toFixed(6)].join('\t'));
}
console.log('\n# per-axis standing of the requested wording inside the emitted pool of', pool.length);
const t=pool.find(r=>r.phrase===PROBE);
for(let i=0;i<7;i++){const a=pool.map(r=>r.v[i]);
 const below=a.filter(x=>x<t.v[i]).length;
 const band=measure(pool,base,PROBE).band.map(r=>r.v[i]);
 console.log(AX[i].padEnd(7), 'clue',t.v[i].toFixed(6), 'poolPct',(100*below/a.length).toFixed(2),
  '| band50 min',Math.min(...band).toFixed(6),'max',Math.max(...band).toFixed(6),
  '| clue<bandMin?', t.v[i]<Math.min(...band)?'YES':'no',
  '| clue>bandMax?', t.v[i]>Math.max(...band)?'YES':'no');}
