const fs=require('fs');
const rows=fs.readFileSync(process.argv[2],'utf8').trim().split('\n').map(l=>{const f=l.split('\t');return{target:f[0],phase:+f[1],slots:+f[2],depth:+f[3],narrow:+f[4],cost:+f[5],acc:f[6]==='1',ranks:f[7],spread:+f[8],words:f[9]};});
const d4=rows.filter(r=>r.depth>=4);
console.log('depth>=4 offers',d4.length,'acc',d4.filter(r=>r.acc).length);
// slot-set distribution
const setc={},setacc={};
for(const r of d4){const s=r.ranks.split(' ').map(x=>x.split(':')[0]).join(',');setc[s]=(setc[s]||0)+1;if(r.acc)setacc[s]=(setacc[s]||0)+1;}
console.log('distinct slot sets offered:',Object.keys(setc).length);
const arr=Object.entries(setc).sort((a,b)=>b[1]-a[1]);
for(const [s,c] of arr) console.log(`  slots{${s}}  offered ${c}  acc ${setacc[s]||0}`);
// narrowest width distribution of accepted d4
const w={};for(const r of d4) if(r.acc){const b=Math.min(99,Math.floor(r.narrow/10)*10);w[b]=(w[b]||0)+1;}
console.log('accepted d4 by narrowest-width bucket:');
for(const k of Object.keys(w).sort((a,b)=>a-b)) console.log(`  ${k}-${+k+9}: ${w[k]}`);
// requested shape
const want=['hits','dupe','hid','came'];
const hits=rows.filter(r=>{const ws=r.words.split(' ');return ws.length===4&&ws.every((x,i)=>x===want[i]);});
console.log('rows whose four deep words are exactly hits dupe hid came:',hits.length, hits.filter(r=>r.acc).length);
const slots035=hits.filter(r=>r.ranks.split(' ').map(x=>x.split(':')[0]).join(',')==='0,3,4,5');
console.log('  of which slot set 0,3,4,5:',slots035.length,'accepted',slots035.filter(r=>r.acc).length);
for(const r of slots035.slice(0,5)) console.log('   ',r.target,r.ranks,r.cost,r.acc,r.words);
// any 4-deep offer at slots 0,3,4,5 for the canonical target?
const s035=d4.filter(r=>r.target==="It's_just_a_stupid_game"&&r.ranks.split(' ').map(x=>x.split(':')[0]).join(',')==='0,3,4,5');
console.log("canonical d4 offers at slots{0,3,4,5}:",s035.length,'acc',s035.filter(r=>r.acc).length);
const ex=s035.filter(r=>r.acc).slice(0,5); for(const r of ex) console.log('   ',r.ranks,r.cost,r.words);
