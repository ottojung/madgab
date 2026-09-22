/* Minimal no-build UI glue: form -> Web Worker -> render. */

const form = document.getElementById("form");
const targetEl = document.getElementById("target");
const modeEl = document.getElementById("mode");
const topnEl = document.getElementById("topn");
const goBtn = document.getElementById("go");
const statusEl = document.getElementById("status");
const resultsEl = document.getElementById("results");
const ipaEl = document.getElementById("ipa");
const cluesEl = document.getElementById("clues");
const errorEl = document.getElementById("error");

let ready = false;
let busy = false;

const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });

function setStatus(text) {
  statusEl.textContent = text;
  statusEl.hidden = text === "";
}

function showError(text) {
  errorEl.textContent = text;
  errorEl.hidden = text === "";
}

worker.onmessage = (event) => {
  const msg = event.data;
  if (msg.type === "ready") {
    ready = true;
    goBtn.disabled = false;
    setStatus("");
    return;
  }
  if (msg.type === "result") {
    busy = false;
    goBtn.disabled = false;
    setStatus("");
    renderResult(JSON.parse(msg.json));
    return;
  }
  if (msg.type === "error") {
    busy = false;
    ready = false;
    goBtn.disabled = true;
    setStatus("");
    showError(msg.message);
  }
};

worker.onerror = (event) => {
  busy = false;
  ready = false;
  goBtn.disabled = true;
  setStatus("");
  showError(`Worker failed: ${event.message || "unknown error"}`);
};

function renderResult({ ipa, clues }) {
  showError("");
  if (!clues || clues.length === 0) {
    resultsEl.hidden = true;
    showError("No clue coverings found for that phrase. Try shorter words or approximate mode.");
    return;
  }
  resultsEl.hidden = false;
  ipaEl.textContent = ipa ? `/${ipa}/` : "";
  cluesEl.innerHTML = "";
  for (const clue of clues) {
    const li = document.createElement("li");
    const score = document.createElement("span");
    score.className = "score";
    score.textContent = `[${Number(clue.score).toFixed(3)}]`;
    li.append(score, document.createTextNode(` ${clue.phrase}`));
    cluesEl.appendChild(li);
  }
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  if (!ready) {
    return;
  }
  if (busy) return;
  const target = targetEl.value.trim();
  if (!target) {
    showError("Enter a target phrase first.");
    return;
  }
  busy = true;
  goBtn.disabled = true;
  showError("");
  resultsEl.hidden = true;
  setStatus("Generating…");
  worker.postMessage({
    type: "generate",
    target,
    topN: Math.min(50, Math.max(1, Number(topnEl.value) || 10)),
    approximate: modeEl.value === "approximate",
  });
});
