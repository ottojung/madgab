const fs=require('fs');
const rows=fs.readFileSync(process.argv[2],'utf8').trim().split('\n').map(l=>{const f=l.split('\t');return{target:f[0],phase:+f[1],slots:+f[2],depth:+f[3],narrow:+f[4],cost:+f[5],acc:f[6]==='1',ranks:f[7],spread:+f[8],words:f[9]};});
const tgt=process.argv[3];
const g={};
for(const r of rows) if(r.target===tgt) (g[r.phase]=g[r.phase]||[]).push(r);
const ks=Object.keys(g).map(Number).sort((a,b)=>a-b);
console.log('phases',ks.length);
for(const k of ks.slice(0,6)){
  const comp={};for(const r of g[k]) comp[r.depth]=(comp[r.depth]||0)+1;
  console.log('phase',k,'n',g[k].length,JSON.stringify(comp),'widths? narrow',[...new Set(g[k].map(r=>r.narrow))].join(','));
  console.log('   shapes:', g[k].map(r=>r.ranks).join(' | '));
}
// depth composition across all phases
const all={};for(const r of rows) if(r.target===tgt){all[r.depth]=all[r.depth]||{off:0};all[r.depth].off++;}
console.log('total by depth',JSON.stringify(all));
