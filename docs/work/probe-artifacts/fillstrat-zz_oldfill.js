const fs = require("fs");
let s = fs.readFileSync("src/lib.rs", "utf8");
const start = s.indexOf("        let fill_slots = SPAN_SHORTLIST.saturating_sub(selected.len());");
const marker =
  "        selected.sort_by(|a, b| cmp_desc(quality(a), quality(b)));\n        selected\n    }";
const end = s.indexOf(marker);
if (start < 0 || end < 0) throw new Error("markers");
const old = `        for m in &by_quality {
            if selected.len() >= SPAN_SHORTLIST {
                break;
            }
            if seen_words.insert(m.word_idx) {
                selected.push(*m);
            }
        }
`;
fs.writeFileSync("src/lib.rs", s.slice(0, start) + old + s.slice(end));
console.log("fill reverted to the global quality walk");
