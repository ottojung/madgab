import { loadPool, measure, base, TOPN } from './forms.mjs';
import { FORMS } from './forms_def.mjs';
import fs from 'fs';
const idx = fs.readFileSync('index.tsv','utf8').trim().split('\n').map(l=>l.split('\t'));
console.log('id\ttarget\tpool@50\tform\trank\tscore\tcutoff\tmargin\tbandWidth\tinBand');
for (const [id,t] of idx) {
  const pool = loadPool(`pool_${id}.tsv`);
  for (const F of FORMS) {
    const f = F.make(pool);
    const m = measure(pool, f, 'hits justice dupe hid came');
    console.log([id, JSON.stringify(t), pool.length, F.name.split(' ')[0],
      m.rank, m.score===null?'':m.score.toFixed(9), m.cutoff.toFixed(9),
      m.margin===null?'ABSENT':m.margin.toFixed(9), m.bandWidth.toFixed(5),
      m.rank>=0&&m.rank<TOPN?'YES':'no'].join('\t'));
  }
}
