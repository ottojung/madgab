// c81e55: how many visible proposals are *full* resegmentations (no target word kept)?
const fs = require("fs");
const M = "measurements/c81e55";
const w = s => s.toLowerCase().replace(/[^a-z]/g, "");
// two words are "the same word" if one is a prefix of the other with >=3 letters
// on both sides, so a contraction ("it's") and its clipping ("it") count as shared.
const same = (a, b) => a === b || (a.length >= 3 && b.length >= 3 && (a.startsWith(b) || b.startsWith(a)));
const rows = [];
for (const f of fs.readdirSync(M).filter(x => x.startsWith("tgt_")).sort()) {
  const target = fs.readFileSync(M + "/" + f, "utf8").trim();
  const tset = target.toLowerCase().split(/\s+/).map(w).filter(Boolean);
  const top = fs.readFileSync(M + "/" + f.replace("tgt_", "").replace(".txt", ".top50"), "utf8")
    .split("\n").map(s => s.trim()).filter(Boolean);
  let full = 0; const kept = new Map();
  for (const l of top) {
    const m = l.match(/\]\s*(.*)$/);
    if (!m) continue;
    const ws = m[1].toLowerCase().split(/\s+/).map(w).filter(Boolean);
    const shared = ws.filter(x => tset.some(t => same(t, x)));
    if (shared.length === 0) full++;
    for (const s of shared) kept.set(s, (kept.get(s) || 0) + 1);
  }
  rows.push([target, top.length, full, [...kept.entries()].map(([k, v]) => k + ":" + v).join(" ")]);
}
rows.sort((a, b) => a[2] - b[2]);
console.log("target".padEnd(32), "n".padStart(3), "full-reseg".padStart(11), "  target words retained, with counts");
for (const r of rows) console.log(r[0].padEnd(32), String(r[1]).padStart(3), String(r[2]).padStart(11), "  " + r[3]);
