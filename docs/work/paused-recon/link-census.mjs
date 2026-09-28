// Broken-relative-link census for the paused MadGab programme.
//
// Why this exists (rule 53 of docs/work/items/w-paused-reconciliation.md):
// the standing one-liner installed by rule 52 matched only links ENDING in .md,
// so a broken link that omits its extension is invisible to the very pattern
// meant to detect broken links.
//
// This detector is required to prove it can fail before its number is believed,
// and to prove it is not merely reporting a property of one repository. Both
// controls run on a SYNTHETIC FIXTURE the script builds for itself, never on
// the target repository: a control whose result depends on the target's own
// contents is a fingerprint of that target, not evidence about this detector.
// (Rule 55: the version this replaces compared the two patterns on the target
// and exited 1 unless the target happened to contain an extensionless broken
// link, so it could not report a broken .md link anywhere except MadGab, and
// called a clean repository a failure.)
//
// Usage: node docs/work/paused-recon/link-census.mjs [repoRoot]
// Exit code 0 = census complete (both controls passed). Exit 1 = control failed.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REPO = process.argv[2] ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

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

const docsFiles = REPO ? walk(path.join(REPO, "docs"), []).sort() : [];

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

const census = (files, re) => {
  const broken = new Map();
  for (const f of files) {
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

// --- CONTROLS (rule 55) ----------------------------------------------------
// Both controls run on a synthetic fixture written to a temp dir, so what they
// measure is the DETECTOR and not the contents of the repository being scanned.
//
//   C1 (can it see what the anchored pattern cannot?) a fixture holding one
//      anchored broken link and one extensionless broken link must yield
//      strictly more edges under ANY than under ANCHORED.
//   C2 (can it report a clean repository as clean?) a fixture whose links all
//      resolve must yield zero broken edges under both patterns. The version
//      this replaces had no C2, which is why it failed on exactly this class.
const fixture = (name, files) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `linkcensus-${name}-`));
  for (const [rel, body] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, "docs", rel)), { recursive: true });
    fs.writeFileSync(path.join(root, "docs", rel), body);
  }
  return root;
};

const mixed = fixture("mixed", {
  "a.md": "# A\n\n[anchored broken](gone.md)\n[extensionless broken](also-gone)\n",
  "b.md": "# B\n\n[resolves](a.md#top)\n",
});
const clean = fixture("clean", {
  "a.md": "# A\n\n[resolves](b.md)\n[resolves with anchor](b.md#section)\n",
  "b.md": "# B\n\n# section\n",
});

const controlResults = [];
for (const [label, root, want] of [
  ["C1 mixed fixture", mixed, (any, anc) => edges(any) > edges(anc)],
  ["C2 clean fixture", clean, (any, anc) => edges(any) === 0 && edges(anc) === 0],
]) {
  const files = walk(path.join(root, "docs"), []);
  const ok = want(census(files, ANY), census(files, ANCHORED));
  if (!ok) {
    console.error(`CONTROL FAILED (${label}): detector is not trustworthy; number withheld.`);
    process.exit(1);
  }
  controlResults.push(label);
}

const anchored = census(docsFiles, ANCHORED);
const full = census(docsFiles, ANY);

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
console.log(`controls: ${controlResults.join(", ")} - PASSED`);
