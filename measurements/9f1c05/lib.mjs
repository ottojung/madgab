import fs from 'fs';
export const AX=['SIM','NOV','WNOV','FAM','RHY','SHAPE','CLOSED'];
export const W=[0.25,0.15,0.15,0.10,0.30,0.05,-0.15];
export function loadPool(f){
  const raw=fs.readFileSync(`/tmp/opencode/agg/pool_${f}.tsv`,'utf8').split('\n').filter(x=>x.startsWith('AGGDUMP'));
  const rows=raw.map(l=>{const c=l.split('\t');return{phrase:c[1],v:c.slice(2,9).map(Number),score:+c[9],syl:+c[10],sig:c[11]};});
  // production finish(): sort desc by score, tie -> phrase asc; then dedup on signature (first wins)
  rows.sort((a,b)=> b.score-a.score || (a.phrase<b.phrase?-1:a.phrase>b.phrase?1:0));
  const seen=new Set(); const pool=[];
  for(const r of rows){ if(seen.has(r.sig))continue; seen.add(r.sig); pool.push(r); }
  return pool;
}
export function base(r){return W[0]*r.v[0]+W[1]*r.v[1]+W[2]*r.v[2]+W[3]*r.v[3]+W[4]*r.v[4]+W[5]*r.v[5]+W[6]*r.v[6];}
export function rankAndCutoff(pool,scoreFn,topn=50){
  const s=pool.map(r=>scoreFn(r));
  const idx=pool.map((r,i)=>i).sort((a,b)=> s[b]-s[a] || (pool[a].phrase<pool[b].phrase?-1:1));
  return {s,idx};
}
export function rankOf(idx,pool,phrase){return idx.findIndex(i=>pool[i].phrase===phrase);}
