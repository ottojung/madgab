const fs=require('fs');
function load(p){return fs.readFileSync(p,'utf8').trim().split('\n').map(l=>{const f=l.split('\t');return{target:f[0],phase:+f[1],slots:+f[2],depth:+f[3],narrow:+f[4],cost:+f[5],acc:f[6]==='1',ranks:f[7],spread:+f[8],words:f[9]};});}
const files=process.argv.slice(2);
for(const f of files){
  const rows=load(f);
  const d={};let to=0,ta=0,tr=0;let d4=0;
  for(const r of rows){d[r.depth]=d[r.depth]||[0,0];d[r.depth][0]++;d[r.depth][1]+=r.acc?1:0;to++;ta+=r.acc?1:0;tr+=r.acc?0:1;if(r.depth>=4)d4++;}
  const s=Object.keys(d).sort().map(k=>`d${k} ${d[k][0]}/${d[k][1]}/${d[k][0]-d[k][1]} ${(100*(d[k][0]-d[k][1])/d[k][0]).toFixed(2)}%`).join(' | ');
  console.log(`${f}\ttotal ${to}/${ta}/${tr} (${(100*tr/to).toFixed(2)}%)  depth>=4 offers: ${d4}\t${s}`);
}
