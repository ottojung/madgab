const fs=require('fs');
for(const f of process.argv.slice(2)){
const rows=fs.readFileSync(f,'utf8').trim().split('\n').map(l=>{const x=l.split('\t');return{target:x[0],phase:+x[1],depth:+x[3]};});
const g={};for(const r of rows){const k=r.target+'|'+r.phase;g[k]=g[k]||[];g[k].push(r.depth);}
const ks=Object.keys(g);
const withD3=ks.filter(k=>g[k].some(d=>d>=3)).length;
const withD4=ks.filter(k=>g[k].some(d=>d>=4)).length;
const maxd={};for(const k of ks){const m=Math.max(...g[k]);maxd[m]=(maxd[m]||0)+1;}
console.log(f,'\n  segmentation-phases:',ks.length,' with a depth>=3 offer:',withD3,' with depth>=4:',withD4);
console.log('  max depth per phase:',JSON.stringify(maxd));
}
