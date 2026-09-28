// Broken-relative-link census for the paused MadGab programme.
//
// Why this exists (rule 53 of docs/work/items/w-paused-reconciliation.md):
// the standing one-liner installed by rule 52 matched only links ENDING in .md,
// so a broken link that omits its extension is invisible to the very pattern
// meant to detect broken links. This detector is required to prove it can fail
// before its number is believed: it runs a NEGATIVE CONTROL on every invocation
// and refuses to report a census if the control does not behave.
//
// Usage: node docs/work/paused-recon/link-census.mjs [repoRoot]
// Exit code 0 = census complete (control passed). Exit code 1 = control failed.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO = process.argv[2] ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const DOCS = path.join(REPO, "docs");

function walk(dir, out) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name === ".git" || e.name === "target" || e.name.startsWith("target-")) continue;
      walk(p, out);
    } else if (e.name.endsWith(".md")) out.push(p);
  }
  return out;
}

const docsFiles = walk(DOCS, []).sort();

// The rule 52 pattern, kept verbatim so the comparison is honest. Rule 53's
// finding is that this pattern cannot see an extensionless broken link.
const ANCHORED = /\]\(([^)\s]+\.md)(#[^)]*)?\)/g;
// This detector's pattern: any relative link, extension optional.
const ANY = /\]\(([^)\s)#][^)\s]*)(#[^)]*)?\)/g;

const linkTargets = (src, link) => {
  const p = link.split("#")[0];
  if (!p || /^(https?:|mailto:)/.test(p)) return null;
  return p.startsWith("/") ? path.join(REPO, p) : path.resolve(path.dirname(src), p);
};

const census = (re) => {
  const broken = new Map();
  for (const f of docsFiles) {
    const txt = fs.readFileSync(f, "utf8");
    const seen = new Set();
    for (const m of txt.matchAll(re)) seen.add(m[1]);
    for (const l of [...seen].sort()) {
      const abs = linkTargets(f, l);
      if (abs && !fs.existsSync(abs)) {
        if (!broken.has(f)) broken.set(f, []);
        broken.get(f).push(l);
      }
    }
  }
  return broken;
};

const edges = (m) => [...m.values()].reduce((a, v) => a + v.length, 0);

// --- NEGATIVE CONTROL (rule 53) -------------------------------------------
// Two extensionless broken links are known to exist in the repository. If the
// census under test does not report strictly MORE edges than the anchored
// pattern, the census is anchored and its "0 broken" is an undercount, so its
// number must not be reported.
const anchored = census(ANCHORED);
const full = census(ANY);
const controlOK = edges(full) > edges(anchored);
if (!controlOK) {
  console.error("NEGATIVE CONTROL FAILED: census is spelling-dependent; number withheld.");
  process.exit(1);
}

// --- repair proposal -------------------------------------------------------
const repoMd = walk(REPO, []).sort();
const propose = (src, link) => {
  const base = path.basename(link.split("#")[0]);
  const stem = base.replace(/\.md$/, "");
  const hits = repoMd.filter(
    (f) => path.basename(f) === base || path.basename(f, ".md") === stem || path.basename(f, ".md") === "w-" + stem
  );
  const set = new Map();
  for (const h of hits) set.set(h, path.relative(path.dirname(src), h));
  const uniq = [...set.values()].sort((a, b) => a.length - b.length);
  return uniq.length === 1 ? uniq[0] : null;
};

const fixable = new Map();
const unresolvable = [];
for (const [f, links] of full) {
  for (const l of links) {
    const t = propose(f, l);
    if (t) {
      if (!fixable.has(f)) fixable.set(f, new Map());
      fixable.get(f).set(l, t);
    } else unresolvable.push([path.relative(REPO, f), l]);
  }
}

const rel = (f) => path.relative(REPO, f);
console.log(`repo: ${REPO}`);
console.log(`docs .md files scanned: ${docsFiles.length}`);
console.log(`anchored-pattern broken edges: ${edges(anchored)} (undercount by construction)`);
console.log(`broken edges, extension optional: ${edges(full)} in ${full.size} files`);
console.log(`uniquely repairable by existing target: ${[...fixable.values()].reduce((a, m) => a + m.size, 0)}`);
console.log(`ambiguous or phantom: ${unresolvable.length}`);
for (const [f, l] of unresolvable) console.log(`  ${f}: ${l}`);
console.log("negative control: PASSED");
