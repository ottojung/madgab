const fs=require('fs');
function load(p){return fs.readFileSync(p,'utf8').trim().split('\n').map(l=>{const f=l.split('\t');return{target:f[0],phase:+f[1],slots:+f[2],depth:+f[3],narrow:+f[4],cost:+f[5],acc:f[6]==='1',ranks:f[7],spread:+f[8],words:f[9]};});}
const A=load(process.argv[2]), B=load(process.argv[3]);
function agg(rows,label){
  const byD={}, byT={};
  for(const r of rows){
    byD[r.depth]=byD[r.depth]||{off:0,acc:0,ref:0,cost:0,n:0};
    byD[r.depth].off++; byD[r.depth][r.acc?'acc':'ref']++; if(r.acc){byD[r.depth].cost+=r.cost;byD[r.depth].n++;}
    byT[r.target]=byT[r.target]||{off:0,acc:0,ref:0,byD:{}};
    byT[r.target].off++; byT[r.target][r.acc?'acc':'ref']++;
    byT[r.target].byD[r.depth]=byT[r.target].byD[r.depth]||{off:0,acc:0,ref:0};
    byT[r.target].byD[r.depth].off++; byT[r.target].byD[r.depth][r.acc?'acc':'ref']++;
  }
  return {byD,byT,label};
}
function show(a){
  console.log('== '+a.label+' ==');
  let to=0,ta=0,tr=0;
  for(const d of Object.keys(a.byD).sort((x,y)=>x-y)){const s=a.byD[d];to+=s.off;ta+=s.acc;tr+=s.ref;
    console.log(`d${d}\toff ${s.off}\tacc ${s.acc}\tref ${s.ref}\t${(100*s.ref/s.off).toFixed(2)}%\tmeancost-acc ${s.n?(s.cost/s.n).toFixed(3):'-'}`);}
  console.log(`ALL\toff ${to}\tacc ${ta}\tref ${tr}\t${(100*tr/to).toFixed(2)}%`);
  console.log('-- per target --');
  for(const t of Object.keys(a.byT).sort()){const s=a.byT[t];
    const ds=Object.keys(s.byD).sort((x,y)=>x-y).map(d=>{const c=s.byD[d];return `d${d} ${c.off}/${c.acc}/${c.ref}`;}).join('  ');
    console.log(`${t}\toff ${s.off}\tacc ${s.acc}\tref ${s.ref}\t${(100*s.ref/s.off).toFixed(2)}%\t${ds}`);}
}
show(agg(A,process.argv[2])); show(agg(B,process.argv[3]));
