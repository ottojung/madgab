const fs=require("fs");
const L=fs.readFileSync(process.argv[2],"utf8").split("\n").filter(Boolean);
const pool=L.filter(l=>l.startsWith("POOL\t")).map(l=>{const p=l.split("\t");return{rank:+p[1],score:+p[2],cuts:p[3],phrase:p.slice(4).join("\t")}});
const vis=L.filter(l=>l.startsWith("VIS\t")).map(l=>{const p=l.split("\t");return{visrank:+p[1],poolrank:+p[2],score:+p[3],cuts:p[4],phrase:p.slice(5).join("\t")}});
const out={pool:pool.length,visible:vis.length,distinctStructures:new Set(pool.map(c=>c.cuts)).size,
 rawCutoff:{phrase:pool[49].phrase,score:pool[49].score,rank:49},
 fillRank:vis[vis.length-1].poolrank,
 worstVisible:vis[vis.length-1].phrase};
if(process.argv[4]){
 const S=process.argv[4].split(",");
 const mem=pool.filter(c=>c.cuts===S.join(","));
 out.structure=S.join(",");out.members=mem.length;
 out.bestMember=mem.length?{phrase:mem[0].phrase,score:mem[0].score,rank:mem[0].rank}:null;
 const want=process.argv[5].toLowerCase().split(" ");
 out.requested=pool.filter(c=>c.phrase.toLowerCase()===want.join(" ")).length;
 const byWord={};
 for(const w of want){const s={};let n=0;
  for(const c of mem){const t=c.phrase.toLowerCase().split(" ");const i=t.indexOf(w);if(i>=0){s[i]=(s[i]||0)+1;n++;}}
  byWord[w]={members:n,bySlot:s};}
 out.perWord=byWord;
 const pairs={};
 for(let i=0;i<want.length;i++)for(let j=i+1;j<want.length;j++){
  const a=want[i],b=want[j];let n=0;const st={};
  for(const c of mem){const t=c.phrase.toLowerCase().split(" ");const i1=t.indexOf(a),i2=t.indexOf(b);
   if(i1>=0&&i2>=0){n++;st[i1+"/"+i2]=(st[i1+"/"+i2]||0)+1;}}
  pairs[a+"+"+b]={members:n,slotPairs:st};}
 out.pairs=pairs;
}
const ws=process.argv[3].split("|").filter(Boolean);
out.witnesses={};
for(const w of ws){
 let members=0,bestRank=null,bestScore=null,slots={},confs=new Set();
 for(const c of pool){const t=c.phrase.toLowerCase().split(" ");const i=t.indexOf(w);if(i<0)continue;
  members++;slots[i]=(slots[i]||0)+1;confs.add(c.cuts);
  if(bestRank===null){bestRank=c.rank;bestScore=c.score;}}
 out.witnesses[w]={members,bestRank,bestScore,bySlot:slots,distinctStructures:confs.size,
   confinement:Object.keys(slots).length===0?"absent":(Object.keys(slots).length===1?("single-slot-"+Object.keys(slots)[0]):"multi-slot")};}
console.log(JSON.stringify(out,null,1));
