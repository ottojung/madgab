const fs=require("fs");
const L=fs.readFileSync(process.argv[2],"utf8").split("\n").filter(Boolean);
const pool=L.filter(l=>l.startsWith("POOL\t")).map(l=>{const p=l.split("\t");return{rank:+p[1],score:+p[2],cuts:p[3],w:p.slice(4).join("\t").toLowerCase().split(" ")}});
const vis=L.filter(l=>l.startsWith("VIS\t")).map(l=>{const p=l.split("\t");return{visrank:+p[1],poolrank:+p[2]}});
const hdr=["target","pool","structs","cutoff(score@49)","fill","witness","members","bestRank","bestScore","topStruct","memInTopStruct","slotConfineInTopStruct"];
const rows=[["#", ...hdr]];
const meta={pool:pool.length,structs:new Set(pool.map(c=>c.cuts)).size,cutoff:pool[49].score,fill:vis[vis.length-1].poolrank};
for(const w of process.argv[3].split("|").filter(Boolean)){
 const occ=[];for(const c of pool){const i=c.w.indexOf(w);if(i>=0)occ.push({c,i});}
 if(!occ.length){rows.push([process.argv[4],meta.pool,meta.structs,meta.cutoff.toFixed(9),meta.fill,w,0,"absent","absent","-","0","ABSENT"]);continue;}
 const best=occ[0];
 const byS={};for(const o of occ){(byS[o.c.cuts]=byS[o.c.cuts]||[]).push(o.i);}
 const top=Object.entries(byS).sort((a,b)=>b[1].length-a[1].length)[0];
 const sl={};top[1].forEach(i=>sl[i]=(sl[i]||0)+1);
 const keys=Object.keys(sl).sort((a,b)=>a-b);
 const conf=keys.length===1?("SINGLE-SLOT-"+keys[0]):("slots "+keys.join(","));
 rows.push([process.argv[4],meta.pool,meta.structs,meta.cutoff.toFixed(9),meta.fill,w,occ.length,best.c.rank,best.c.score.toFixed(9),top[0],top[1].length,conf+"  ["+JSON.stringify(sl)+"]"]);
}
const wd=[0,1,2,3,4,5,6,7,8,9,10,11].map(i=>Math.max(...rows.map(r=>String(r[i]).length)));
for(const r of rows)console.log(r.map((c,i)=>String(c).padEnd(wd[i])).join("  "));
