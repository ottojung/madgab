//! Mad Gab puzzle generator.
//!
//! The generator searches for English word sequences whose connected
//! pronunciation is close to a target phrase while preferring a
//! genuinely different lexical/word-boundary parse.

use std::collections::{HashMap, HashSet};

use phonetics::transcriptions::{Corpus, Pronunciation};
use serde::Serialize;

mod approx;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// One candidate Mad Gab clue.
#[derive(Debug, Clone, Serialize)]
pub struct Clue {
    pub phrase: String,
    pub ipa: String,
    pub words: Vec<ClueWord>,
    /// Composite score in [0, 1] — higher is better.
    pub score: f64,
}

/// One word inside a candidate clue.
#[derive(Debug, Clone, Serialize)]
pub struct ClueWord {
    pub word: String,
    pub ipa: String,
    pub rarity: Option<f64>,
    /// Phonetic edit cost against the target span consumed by this word.
    pub sub_cost: f64,
}

/// Search behavior.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    Exact,
    Approximate {
        per_word_budget: f64,
        total_budget: f64,
    },
}

impl SearchMode {
    pub fn approximate() -> Self {
        Self::Approximate {
            per_word_budget: 0.5,
            total_budget: 1.5,
        }
    }
}

/// Search configuration.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub beam_width: usize,
    pub top_n: usize,
    pub max_rarity: Option<f64>,
    pub mode: SearchMode,
    pub min_word_ipa_chars: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            beam_width: 64,
            top_n: 10,
            max_rarity: Some(50_000.0),
            mode: SearchMode::Exact,
            min_word_ipa_chars: 1,
        }
    }
}

/// A reusable Mad Gab generator.
pub struct Generator {
    corpus: Corpus,
    config: GeneratorConfig,
    /// Iteration-friendly view of the same preferred pronunciations.
    /// The corpus trie remains authoritative for Exact mode.
    fuzzy_lexicon: approx::FuzzyLexicon,
}

impl Generator {
    pub fn from_json(
        json: &str,
        config: GeneratorConfig,
    ) -> Result<Self, phonetics::transcriptions::Error> {
        let corpus = Corpus::from_json(json, config.max_rarity)?;

        // The common/default configuration is rarity-bounded, so this
        // stays around 50k words. Avoid building a second 280k-word view
        // for callers that explicitly request an unfiltered Exact-only
        // corpus (the integration corpus probes do this).
        let fuzzy_lexicon = if config.max_rarity.is_some()
            || matches!(config.mode, SearchMode::Approximate { .. })
        {
            approx::build_lexicon(json, &corpus, config.max_rarity)
        } else {
            approx::FuzzyLexicon::empty()
        };

        Ok(Self {
            corpus,
            config,
            fuzzy_lexicon,
        })
    }

    pub fn corpus(&self) -> &Corpus {
        &self.corpus
    }

    pub fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: GeneratorConfig) {
        self.config = config;
    }

    /// Generate ranked clue candidates for target.
    pub fn generate(&self, target: &str) -> Vec<Clue> {
        match self.config.mode {
            SearchMode::Exact => self.generate_exact(target),
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => self.generate_approximate(target, per_word_budget, total_budget),
        }
    }

    fn generate_exact(&self, target: &str) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_with_boundaries(&self.corpus, target, false)
        else {
            return Vec::new();
        };
        let target_words = target_word_set(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        let mut beam: Vec<Vec<Partial>> = vec![Vec::new(); n + 1];
        beam[0].push(Partial::empty());

        for p in 0..n {
            if beam[p].is_empty() {
                continue;
            }
            let here = std::mem::take(&mut beam[p]);
            for partial in &here {
                for (consumed, pronunciation) in self.corpus.trie.words_starting_at(&chars, p) {
                    if pronunciation.ipa.chars().count() < self.config.min_word_ipa_chars {
                        continue;
                    }
                    let next = partial.extend_pronunciation(pronunciation, consumed, 0.0);
                    insert_top_k(&mut beam[p + consumed], next, self.config.beam_width);
                }
            }
        }

        self.finish(
            std::mem::take(&mut beam[n]),
            &target_ipa,
            &target_boundaries,
            &target_words,
        )
    }

    fn generate_approximate(
        &self,
        target: &str,
        per_word_budget: f64,
        total_budget: f64,
    ) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_with_boundaries(&self.corpus, target, true)
        else {
            return Vec::new();
        };
        let target_words = target_word_set(target);
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 || self.fuzzy_lexicon.is_empty() {
            return Vec::new();
        }

        // Candidate word/span alignments depend only on the target and
        // per-word edit budget, not on a particular beam hypothesis.
        // Build this expensive lattice once.
        let lattice: Vec<Vec<approx::FuzzyMatch>> = (0..n)
            .map(|p| {
                self.fuzzy_lexicon.matches_at(
                    &chars,
                    p,
                    per_word_budget,
                    self.config.min_word_ipa_chars,
                )
            })
            .collect();

        let mut beam: Vec<Vec<Partial>> = vec![Vec::new(); n + 1];
        beam[0].push(Partial::empty());

        for p in 0..n {
            if beam[p].is_empty() {
                continue;
            }
            let here = prune_partials(
                std::mem::take(&mut beam[p]),
                self.config.beam_width,
                &target_boundaries,
                &target_words,
                n,
            );

            for partial in &here {
                let remaining = total_budget - partial.sub_cost_total;
                if remaining < -1e-9 {
                    continue;
                }
                for m in &lattice[p] {
                    if m.cost > remaining + 1e-9 {
                        continue;
                    }
                    let word = self.fuzzy_lexicon.word(m.word_idx);
                    let next = partial.extend_fuzzy(word, m.consumed, m.cost);
                    let q = p + m.consumed;
                    if q > n {
                        continue;
                    }
                    beam[q].push(next);

                    // Intermediate hypotheses must stay tight for
                    // interactive search. Completed hypotheses are
                    // different: they will never be expanded again, so
                    // retain a much larger bounded pool and let the real
                    // final scorer + diversity selector decide among
                    // them. Premature completion pruning loses exactly
                    // the globally-good parses beam search is meant to
                    // approximate.
                    let keep = if q == n {
                        self.config
                            .top_n
                            .saturating_mul(128)
                            .max(1024)
                            .min(8192)
                    } else {
                        self.config.beam_width.max(1)
                    };
                    if beam[q].len() > keep.saturating_mul(2) {
                        let reduced = prune_partials(
                            std::mem::take(&mut beam[q]),
                            keep,
                            &target_boundaries,
                            &target_words,
                            n,
                        );
                        beam[q] = reduced;
                    }
                }
            }
        }

        let final_keep = self
            .config
            .top_n
            .saturating_mul(128)
            .max(1024)
            .min(8192);
        let completed = prune_partials(
            std::mem::take(&mut beam[n]),
            final_keep,
            &target_boundaries,
            &target_words,
            n,
        );
        self.finish(
            completed,
            &target_ipa,
            &target_boundaries,
            &target_words,
        )
    }

    fn finish(
        &self,
        completed: Vec<Partial>,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
    ) -> Vec<Clue> {
        let mut clues: Vec<Clue> = completed
            .into_iter()
            .map(|p| p.into_clue(target_ipa, target_boundaries, target_words))
            .collect();

        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.phrase.cmp(&b.phrase))
        });

        let mut seen = HashSet::new();
        clues.retain(|c| seen.insert(c.phrase.to_lowercase()));
        select_diverse(clues, self.config.top_n)
    }
}

// -----------------------------------------------------------------
// Transcription and scoring
// -----------------------------------------------------------------

fn transcribe_with_boundaries(
    corpus: &Corpus,
    phrase: &str,
    normalize: bool,
) -> Option<(String, Vec<usize>)> {
    let mut out = String::new();
    let mut boundaries = Vec::new();
    for word in phrase.split_whitespace() {
        let key = clean_input_word(word);
        let ipa = corpus.preferred_ipa(&key)?;
        if normalize {
            out.push_str(&approx::normalize_ipa(ipa));
        } else {
            out.push_str(ipa);
        }
        boundaries.push(out.chars().count());
    }
    Some((out, boundaries))
}

fn clean_input_word(word: &str) -> String {
    word.to_lowercase()
        .trim_end_matches(['.', ',', '!', '?', ';', ':'])
        .to_string()
}

fn normalized_word(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn target_word_set(target: &str) -> HashSet<String> {
    target
        .split_whitespace()
        .map(normalized_word)
        .collect()
}

#[derive(Debug, Clone)]
struct Partial {
    words: Vec<ClueWord>,
    sub_cost_total: f64,
    cheap_score: f64,
    /// Target-stream offsets consumed at clue word boundaries.
    cuts: Vec<usize>,
    /// Stable path key for duplicate suppression in approximate beams.
    key: String,
}

impl Partial {
    fn empty() -> Self {
        Self {
            words: Vec::new(),
            sub_cost_total: 0.0,
            cheap_score: 0.0,
            cuts: Vec::new(),
            key: String::new(),
        }
    }

    fn extend_pronunciation(
        &self,
        p: &Pronunciation,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        self.extend_parts(&p.word, &p.ipa, p.rarity, consumed, word_sub_cost)
    }

    fn extend_fuzzy(
        &self,
        word: &approx::FuzzyWord,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        self.extend_parts(
            &word.word,
            &word.ipa,
            word.rarity,
            consumed,
            word_sub_cost,
        )
    }

    fn extend_parts(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        let len = ipa.chars().count();
        let word_bonus = (len as f64).min(6.0) / 6.0;
        let rarity_penalty = match rarity {
            Some(r) if r > 5_000.0 => -((r / 50_000.0).min(1.0)),
            _ => 0.0,
        };

        let mut words = self.words.clone();
        words.push(ClueWord {
            word: word.to_string(),
            ipa: ipa.to_string(),
            rarity,
            sub_cost: word_sub_cost,
        });

        let mut cuts = self.cuts.clone();
        let end = cuts.last().copied().unwrap_or(0) + consumed;
        cuts.push(end);

        let step_key = format!("{}\u{1f}{}", word.to_lowercase(), ipa);
        let key = if self.key.is_empty() {
            step_key
        } else {
            format!("{} {}", self.key, step_key)
        };

        Self {
            words,
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            cheap_score: self.cheap_score + word_bonus + rarity_penalty - word_sub_cost,
            cuts,
            key,
        }
    }

    fn metrics(
        &self,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
        total_len: usize,
        partial: bool,
    ) -> Metrics {
        let similarity = (1.0 - self.sub_cost_total / 4.0).clamp(0.0, 1.0);
        let novelty = boundary_novelty(
            &self.cuts,
            target_boundaries,
            total_len,
            partial,
        );

        let reused = self
            .words
            .iter()
            .filter(|w| target_words.contains(&normalized_word(&w.word)))
            .count() as f64;
        let word_novelty = 1.0 - reused / self.words.len().max(1) as f64;

        let familiarity = if self.words.is_empty() {
            0.0
        } else {
            self.words
                .iter()
                .map(|w| word_familiarity(w.rarity))
                .sum::<f64>()
                / self.words.len() as f64
        };

        let avg_word_ipa_len = self
            .words
            .iter()
            .map(|w| w.ipa.chars().count())
            .sum::<usize>() as f64
            / self.words.len().max(1) as f64;
        let length_signal = (avg_word_ipa_len / 4.0).min(1.0);

        let combined = 0.45 * similarity
            + 0.25 * novelty
            + 0.10 * word_novelty
            + 0.15 * familiarity
            + 0.05 * length_signal;

        Metrics {
            combined,
            novelty,
            familiarity,
            word_novelty,
        }
    }

    fn into_clue(
        self,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
    ) -> Clue {
        let total_len = target_ipa.chars().count();
        let score = self
            .metrics(target_boundaries, target_words, total_len, false)
            .combined;
        Clue {
            phrase: self
                .words
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            ipa: target_ipa.to_string(),
            words: self.words,
            score,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Metrics {
    combined: f64,
    novelty: f64,
    familiarity: f64,
    word_novelty: f64,
}

/// Symmetric segmentation novelty: Jaccard distance between target
/// inner word boundaries and clue inner boundaries.
///
/// The old score only asked which target boundaries disappeared. It
/// therefore gave zero novelty to a useful split that preserved an
/// original boundary (for example splitting one target word into
/// several clue words). Jaccard distance rewards both added and
/// removed boundaries.
fn boundary_novelty(
    cuts: &[usize],
    target_boundaries: &[usize],
    total_len: usize,
    partial: bool,
) -> f64 {
    let covered = cuts.last().copied().unwrap_or(0);

    let target_inner: HashSet<usize> = target_boundaries
        .iter()
        .copied()
        .filter(|&b| {
            b < total_len && (!partial || b <= covered)
        })
        .collect();
    let clue_inner: HashSet<usize> = cuts
        .iter()
        .copied()
        .filter(|&c| c < total_len)
        .collect();

    let union = target_inner.union(&clue_inner).count();
    if union == 0 {
        return 0.0;
    }
    let shared = target_inner.intersection(&clue_inner).count();
    1.0 - shared as f64 / union as f64
}

fn word_familiarity(rarity: Option<f64>) -> f64 {
    let Some(r) = rarity.filter(|r| r.is_finite() && *r > 0.0) else {
        return 0.0;
    };
    let lo = 100.0_f64.log10();
    let hi = 50_000.0_f64.log10();
    (1.0 - (r.max(1.0).log10() - lo) / (hi - lo)).clamp(0.0, 1.0)
}

// -----------------------------------------------------------------
// Beam retention
// -----------------------------------------------------------------

/// Exact mode keeps its original cheap top-K behavior.
fn insert_top_k(beam: &mut Vec<Partial>, candidate: Partial, k: usize) {
    if k == 0 {
        return;
    }
    if beam.len() < k {
        beam.push(candidate);
        return;
    }
    let (worst_idx, worst_score) = beam
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.cheap_score
                .partial_cmp(&b.cheap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, p)| (i, p.cheap_score))
        .unwrap();
    if candidate.cheap_score > worst_score {
        beam[worst_idx] = candidate;
    }
}

/// Approximate mode keeps a bounded *portfolio* rather than a single
/// scalar top-K. This matters because a locally expensive word can be
/// the key to a globally excellent resegmentation.
///
/// We reserve representatives across segmentation-depth / cost cells,
/// then fill the rest round-robin from independent objective rankings:
/// overall score, boundary novelty, lexical familiarity, phonetic cost,
/// and target-word novelty.
fn prune_partials(
    candidates: Vec<Partial>,
    k: usize,
    target_boundaries: &[usize],
    target_words: &HashSet<String>,
    total_len: usize,
) -> Vec<Partial> {
    if k == 0 || candidates.is_empty() {
        return Vec::new();
    }

    // Exact path duplicates (same words + same pronunciations) can be
    // generated through multiple edit alignments. Keep the better one.
    let mut dedup: HashMap<String, Partial> = HashMap::new();
    for candidate in candidates {
        match dedup.get(&candidate.key) {
            Some(old)
                if old
                    .metrics(target_boundaries, target_words, total_len, true)
                    .combined
                    >= candidate
                        .metrics(target_boundaries, target_words, total_len, true)
                        .combined => {}
            _ => {
                dedup.insert(candidate.key.clone(), candidate);
            }
        }
    }

    let items: Vec<Partial> = dedup.into_values().collect();
    if items.len() <= k {
        return items;
    }

    let metrics: Vec<Metrics> = items
        .iter()
        .map(|p| p.metrics(target_boundaries, target_words, total_len, true))
        .collect();

    let mut selected = HashSet::new();

    // First protect up to two representatives from each structural
    // (word-count, acoustic-cost-band) cell. This prevents the huge
    // family of zero-cost/local optima from erasing every moderately
    // edited resegmentation.
    let mut cells: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (i, p) in items.iter().enumerate() {
        let band = ((p.sub_cost_total / 0.25) + 1e-9).floor() as usize;
        cells
            .entry((p.words.len().min(16), band.min(16)))
            .or_default()
            .push(i);
    }
    let mut protected = Vec::new();
    for members in cells.values_mut() {
        members.sort_by(|&a, &b| {
            metrics[b]
                .combined
                .partial_cmp(&metrics[a].combined)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        protected.extend(members.iter().take(2).copied());
    }
    protected.sort_by(|&a, &b| {
        metrics[b]
            .combined
            .partial_cmp(&metrics[a].combined)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in protected.into_iter().take(k / 2) {
        selected.insert(i);
    }

    let mut orders: Vec<Vec<usize>> = Vec::new();
    let indices: Vec<usize> = (0..items.len()).collect();

    let mut combined = indices.clone();
    combined.sort_by(|&a, &b| cmp_desc(metrics[a].combined, metrics[b].combined));
    orders.push(combined);

    let mut novelty = indices.clone();
    novelty.sort_by(|&a, &b| cmp_desc(metrics[a].novelty, metrics[b].novelty));
    orders.push(novelty);

    let mut familiarity = indices.clone();
    familiarity.sort_by(|&a, &b| cmp_desc(metrics[a].familiarity, metrics[b].familiarity));
    orders.push(familiarity);

    let mut acoustic = indices.clone();
    acoustic.sort_by(|&a, &b| {
        items[a]
            .sub_cost_total
            .partial_cmp(&items[b].sub_cost_total)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    orders.push(acoustic);

    let mut lexical = indices;
    lexical.sort_by(|&a, &b| cmp_desc(metrics[a].word_novelty, metrics[b].word_novelty));
    orders.push(lexical);

    let mut rank = 0;
    while selected.len() < k {
        let mut added = false;
        for order in &orders {
            if let Some(&i) = order.get(rank) {
                added |= selected.insert(i);
                if selected.len() == k {
                    break;
                }
            }
        }
        if !added && orders.iter().all(|o| rank >= o.len()) {
            break;
        }
        rank += 1;
    }

    let mut out: Vec<Partial> = selected.into_iter().map(|i| items[i].clone()).collect();
    out.sort_by(|a, b| {
        let am = a.metrics(target_boundaries, target_words, total_len, true);
        let bm = b.metrics(target_boundaries, target_words, total_len, true);
        cmp_desc(am.combined, bm.combined)
    });
    out
}

fn cmp_desc(a: f64, b: f64) -> std::cmp::Ordering {
    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
}

// -----------------------------------------------------------------
// Final proposal diversity
// -----------------------------------------------------------------

const MMR_LAMBDA: f64 = 0.20;

fn select_diverse(clues: Vec<Clue>, top_n: usize) -> Vec<Clue> {
    if top_n == 0 || clues.is_empty() {
        return Vec::new();
    }
    if clues.len() <= top_n {
        return clues;
    }

    let words: Vec<Vec<String>> = clues
        .iter()
        .map(|c| {
            c.phrase
                .split_whitespace()
                .map(normalized_word)
                .collect()
        })
        .collect();

    let bigrams: Vec<HashSet<String>> = words
        .iter()
        .map(|ws| {
            ws.windows(2)
                .map(|w| format!("{} {}", w[0], w[1]))
                .collect()
        })
        .collect();

    let word_sets: Vec<HashSet<String>> =
        words.iter().map(|ws| ws.iter().cloned().collect()).collect();

    let mut remaining: Vec<usize> = (0..clues.len()).collect();
    let mut picked = Vec::with_capacity(top_n.min(clues.len()));

    while picked.len() < top_n && !remaining.is_empty() {
        let mut best_pos = 0;
        let mut best_value = f64::NEG_INFINITY;

        for (pos, &i) in remaining.iter().enumerate() {
            let mut max_overlap: f64 = 0.0;
            for &j in &picked {
                let word_overlap =
                    directional_overlap(&word_sets[i], &word_sets[j]);
                let bigram_overlap =
                    directional_overlap(&bigrams[i], &bigrams[j]);
                max_overlap = max_overlap.max(0.5 * word_overlap + 0.5 * bigram_overlap);
            }

            let value = clues[i].score - MMR_LAMBDA * max_overlap;
            if value > best_value {
                best_value = value;
                best_pos = pos;
            }
        }

        picked.push(remaining.remove(best_pos));
    }

    let mut slots: Vec<Option<Clue>> = clues.into_iter().map(Some).collect();
    picked
        .into_iter()
        .map(|i| slots[i].take().expect("picked once"))
        .collect()
}

fn directional_overlap(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() {
        return 0.0;
    }
    a.intersection(b).count() as f64 / a.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = r#"{
        "cat":  { "rarity": 100, "ipa": { "cmu": "kæt" }, "alt_display": "CAT" },
        "kit":  { "rarity": 200, "ipa": { "cmu": "kɪt" } },
        "at":   { "rarity": 80,  "ipa": { "cmu": "æt" } },
        "ka":   { "rarity": 5000,"ipa": { "cmu": "kæ" } }
    }"#;

    #[test]
    fn transcribes_known_phrase() {
        let g = Generator::from_json(TINY, GeneratorConfig::default()).unwrap();
        assert_eq!(g.corpus().transcribe("cat"), Some("kæt".to_string()));
    }

    #[test]
    fn generates_completions_for_a_tiny_corpus() {
        let cfg = GeneratorConfig {
            min_word_ipa_chars: 2,
            ..GeneratorConfig::default()
        };
        let g = Generator::from_json(TINY, cfg).unwrap();
        let clues = g.generate("cat");
        assert!(!clues.is_empty());
        assert!(clues.iter().any(|c| c.phrase == "cat"));
    }

    #[test]
    fn empty_when_target_word_unknown() {
        let g = Generator::from_json(TINY, GeneratorConfig::default()).unwrap();
        assert!(g.generate("orange").is_empty());
    }

    #[test]
    fn jaccard_boundary_novelty_rewards_added_cuts() {
        let target = vec![9, 14];
        let clue = vec![3, 4, 9, 14];
        let n = boundary_novelty(&clue, &target, 14, false);
        assert!((n - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn acceptance_sequences_are_reachable_in_fuzzy_lattice() {
        use open_english_pronouncing_dictionary::CORPUS_JSON;

        fn check(target: &str, expected: &[&str]) {
            let g = Generator::from_json(
                CORPUS_JSON,
                GeneratorConfig {
                    mode: SearchMode::approximate(),
                    ..GeneratorConfig::default()
                },
            )
            .unwrap();

            let (ipa, _) = transcribe_with_boundaries(g.corpus(), target, true).unwrap();
            let chars: Vec<char> = ipa.chars().collect();

            // Keep every acoustically valid alignment of the required
            // word sequence. This checks matcher/budget reachability
            // independently of phrase-beam pruning.
            let mut states = vec![(0usize, 0.0f64)];
            for &wanted in expected {
                let mut next = Vec::new();
                let mut seen = HashSet::new();

                for &(pos, total) in &states {
                    if pos >= chars.len() {
                        continue;
                    }
                    for m in g
                        .fuzzy_lexicon
                        .matches_at(&chars, pos, 0.5, 1)
                    {
                        let word = g.fuzzy_lexicon.word(m.word_idx);
                        if word.word.eq_ignore_ascii_case(wanted)
                            && total + m.cost <= 1.5 + 1e-9
                            && seen.insert((pos + m.consumed, (total + m.cost).to_bits()))
                        {
                            next.push((pos + m.consumed, total + m.cost));
                        }
                    }
                }

                assert!(
                    !next.is_empty(),
                    "{target:?}: required word {wanted:?} has no continuation after states {states:?}"
                );
                states = next;
            }

            assert!(
                states.iter().any(|&(pos, _)| pos == chars.len()),
                "{target:?}: required sequence ends at {states:?}, target len {}",
                chars.len()
            );
        }

        check(
            "It's just a stupid game",
            &["hits", "justice", "dupe", "hid", "came"],
        );
        check(
            "recognize speech",
            &["wreck", "a", "nice", "beach"],
        );
    }
}
