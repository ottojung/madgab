//! wasm-bindgen wrapper over [`crate::Generator`].
//!
//! The browser UI never touches the generator directly: `web/worker.js`
//! loads the wasm-pack bundle in a Web Worker, constructs one
//! `WasmGenerator` (corpus load happens once), and calls `generate`
//! off the main thread. Results cross the worker boundary as JSON
//! strings to keep the binding surface trivial.

use wasm_bindgen::prelude::*;

use crate::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

/// Long-lived generator instance shared by all `generate` calls on
/// the page. Build once; the corpus load dominates startup, so reuse
/// matters.
#[wasm_bindgen]
pub struct WasmGenerator {
    generator: Generator,
}

#[wasm_bindgen]
impl WasmGenerator {
    /// Load the embedded corpus with interactive defaults. Throws a
    /// JS error if the bundled corpus fails to parse (should never
    /// happen — same constant the native CLI uses).
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmGenerator, JsValue> {
        let generator = Generator::from_json(CORPUS_JSON, GeneratorConfig::default())
            .map_err(|e| JsValue::from_str(&format!("corpus load failed: {e}")))?;
        Ok(Self { generator })
    }

    /// Transcribe `target` to IPA. Returns an empty string when any
    /// word has no transcription.
    pub fn transcribe(&self, target: &str) -> String {
        self.generator
            .corpus()
            .transcribe(target)
            .unwrap_or_default()
    }

    /// Generate the top `top_n` clues for `target` and return them as
    /// a JSON string: `{ ipa, clues: [{ phrase, ipa, score, words }] }`.
    ///
    /// `approximate` selects [`SearchMode::approximate`] vs
    /// [`SearchMode::Exact`]; budgets stay at the documented defaults
    /// (per-word 0.5, total 1.5) to keep the UI surface small.
    /// `top_n` is clamped to 1..=50.
    pub fn generate(&mut self, target: &str, top_n: usize, approximate: bool) -> String {
        let mut config = self.generator.config().clone();
        config.top_n = top_n.clamp(1, 50);
        config.mode = if approximate {
            SearchMode::approximate()
        } else {
            SearchMode::Exact
        };
        self.generator.set_config(config);
        let clues = self.generator.generate(target);
        let ipa = self
            .generator
            .corpus()
            .transcribe(target)
            .unwrap_or_default();
        serde_json::json!({ "ipa": ipa, "clues": clues }).to_string()
    }
}
