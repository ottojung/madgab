import fs from 'fs';
import { isFullReseg } from './forms.mjs';
const idx = fs.readFileSync('index.tsv', 'utf8').trim().split('\n').map(l => l.split('\t'));
const FORMS = [0, 2, 3];
const FN = { 0: 'F0 linear (control)', 2: 'F2 hard gate on WORD_NOVELTY', 3: 'F3 strict lexicographic' };
const rd = p => fs.readFileSync(p, 'utf8').split('\n').map(x => x.trim()).filter(Boolean);
console.log(['id', 'target', 'pool@50', 'form', 'visibleChurn/50', 'kept', 'axesMatched', 'fullReseg/50', 'salad/50', 'poolDelta'].join('\t'));
for (const [id, t] of idx) {
  const tw = t.toLowerCase().split(/\s+/).filter(w => /[a-z0-9']/.test(w));
  const base = new Set(rd(`price/${id}_f0.vis`));
  const pool0 = +rd(`price/${id}_f0.pool`)[0];
  for (const f of FORMS) {
    const vis = rd(`price/${id}_f${f}.vis`);
    const vs = new Set(vis);
    const ax = rd(`price/${id}_f${f}.axes`).map(l => l.split('\t'));
    const seen = new Map();
    for (const c of ax) if (!seen.has(c[0])) seen.set(c[0], c.slice(1, 8).map(Number));
    const members = vis.map(p => seen.get(p)).filter(Boolean);
    const R = vis.filter(p => isFullReseg(p, tw)).length;
    const S = members.filter(m => Math.sqrt(m[6]) > 1 / 3 + 1e-12).length;
    const pool = +rd(`price/${id}_f${f}.pool`)[0];
    let kept = 0; for (const x of base) if (vs.has(x)) kept++;
    console.log([id, JSON.stringify(t), pool0, FN[f], base.size - kept, kept, members.length, R, S, pool - pool0].join('\t'));
  }
}
