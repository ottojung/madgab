/* Web Worker: loads the wasm-pack bundle off the main thread so
 * generation never blocks the UI. Message protocol:
 *   in:  { type: "generate", target, topN, approximate }
 *   out: { type: "ready" } | { type: "result", json } | { type: "error", message }
 */
import init, { WasmGenerator } from "./pkg/madgab.js";

let generator = null;

try {
  await init();
  generator = new WasmGenerator();
  self.postMessage({ type: "ready" });
} catch (err) {
  self.postMessage({ type: "error", message: `Failed to load engine: ${err?.message || err}` });
}

self.onmessage = (event) => {
  const msg = event.data;
  if (msg.type !== "generate" || generator === null) return;
  try {
    const json = generator.generate(msg.target, msg.topN, msg.approximate);
    self.postMessage({ type: "result", json });
  } catch (err) {
    self.postMessage({ type: "error", message: `Generation failed: ${err?.message || err}` });
  }
};
