//! Mad Gab puzzle generator.
//!
//! Mad Gab takes an English phrase and re-presents it as a sequence of
//! different English words whose concatenated pronunciation is similar
//! to the original. "It's just a stupid game" becomes "Hits Justice
//! Dupe Hid Came" — same phoneme stream, totally different lexical
//! parse.
//!
//! This crate's [`Generator`] takes a target phrase and returns a
//! ranked list of candidate clues. It leans on three capabilities
//! from `phonetics-rs`:
//!
//!   * `Corpus::transcribe` to turn the target into an IPA string
//!   * `Corpus::trie::words_starting_at` to enumerate every English
//!     word whose IPA matches a given prefix of the target
//!   * `phonetics::similarity` to score how close a candidate clue
//!     sounds to the target
//!
//! The search itself is a beam-DP over phoneme positions: at each
//! position we keep the best K coverings reachable so far, and at
//! each step we extend each beam entry by every word in the trie
//! that fits. Polynomial time in the phoneme stream length.

use std::collections::{BTreeMap, HashMap, HashSet};

use phonetics::transcriptions::{Corpus, Pronunciation};
use serde::Serialize;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// One candidate Mad Gab clue.
#[derive(Debug, Clone, Serialize)]
pub struct Clue {
    /// The clue as a space-joined English phrase.
    pub phrase: String,
    /// The IPA stream the clue covers. Always equal to the target's
    /// IPA in [`SearchMode::Exact`] mode.
    pub ipa: String,
    /// Per-word components, in order.
    pub words: Vec<ClueWord>,
    /// Composite score in [0, 1] — higher is a better Mad Gab clue.
    pub score: f64,
}

/// One word inside a candidate clue.
#[derive(Debug, Clone, Serialize)]
pub struct ClueWord {
    /// English headword.
    pub word: String,
    /// IPA span this word's transcription covers (which may differ
    /// from the target's span in Approximate mode).
    pub ipa: String,
    /// Frequency rank from the corpus, if known.
    pub rarity: Option<f64>,
    /// Accumulated substitution cost relative to the target's span
    /// of the IPA stream this word covers. Always 0.0 in Exact mode.
    pub sub_cost: f64,
}

/// Search behavior.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    /// Each clue word's IPA must exactly match its span of the
    /// target stream. The clue is a re-syllabification of the same
    /// phonemes; phonetic similarity is by construction 1.0.
    Exact,
    /// Each clue word's IPA is allowed to differ from its span of
    /// the target by up to `per_word_budget` of accumulated
    /// phonetic-distance cost; the whole clue's accumulated cost is
    /// capped at `total_budget`. Lets the generator find clues
    /// whose phonemes don't exactly match — /t/→/d/, /ɪ/→/i/, etc.
    Approximate {
        /// Maximum substitution cost per single trie-walked word.
        per_word_budget: f64,
        /// Maximum substitution cost summed across the whole clue.
        total_budget: f64,
    },
}

impl SearchMode {
    /// A sensible Approximate default — per-word slack wide enough
    /// to admit single weak-indel resegmentations (a dropped or
    /// inserted /s/ beside a close substitution costs ~0.75), with
    /// a total cap that still keeps the clue recognizable.
    pub fn approximate() -> Self {
        Self::Approximate {
            per_word_budget: 0.75,
            total_budget: 1.5,
        }
    }
}

/// Search configuration. Defaults are tuned for an interactive
/// `madgab "..."` invocation.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Number of clue candidates kept per beam position. Higher =
    /// better quality but quadratically more memory and time.
    pub beam_width: usize,
    /// How many candidates to return.
    pub top_n: usize,
    /// Maximum corpus rarity (= least common word allowed). Lower
    /// values mean a smaller, faster-to-build trie and clues that
    /// use more familiar vocabulary. None loads everything.
    pub max_rarity: Option<f64>,
    /// Search mode (only Exact is implemented for v0.1.0).
    pub mode: SearchMode,
    /// Reject clue words shorter than this many IPA characters.
    /// Without a floor, the beam fills with degenerate
    /// single-vowel paths like "a a a a a a a".
    pub min_word_ipa_chars: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            beam_width: 64,
            top_n: 10,
            // The rebuilt fused corpus ranks ~280k words; the cap
            // covers roughly the top 20% by frequency, which is wide
            // enough to keep canonical Mad Gab clue words like
            // "dupe" (rank ~40k) but tight enough to keep the trie
            // small.
            max_rarity: Some(50_000.0),
            mode: SearchMode::Exact,
            // 1 keeps legitimate single-segment morphemes ("a", "I",
            // interjections like "uh", "ah", "sh") in play. The
            // corpus build filters out fragment-only entries
            // ('s, 't, 'd) so we don't pay for them in noise.
            min_word_ipa_chars: 1,
        }
    }
}

/// A reusable Mad Gab generator. Build once from a corpus; ask for
/// many phrases.
pub struct Generator {
    corpus: Corpus,
    config: GeneratorConfig,
    /// Stress-stripped mirror of the corpus used only by Approximate
    /// mode. `phonetics-rs`' trie supports substitution-only fuzzy
    /// walks over raw (stress-bearing) IPA, which cannot reach
    /// near-homophones whose segment counts differ (insertions /
    /// deletions) or whose lexical stress marks fall on different
    /// segments. This index stores stress-stripped forms and is
    /// queried with a small edit-tolerant (match / substitute /
    /// insert / delete) trie walk instead.
    approx_trie: ApproxTrie,
    approx_entries: Vec<ApproxEntry>,
}

impl Generator {
    /// Build a generator from raw corpus JSON. The JSON shape is the
    /// one `phonetics::transcriptions::Corpus` expects.
    pub fn from_json(
        json: &str,
        config: GeneratorConfig,
    ) -> Result<Self, phonetics::transcriptions::Error> {
        let corpus = Corpus::from_json(json, config.max_rarity)?;
        let approx_entries = ApproxEntry::from_json(json, config.max_rarity);
        let approx_trie = ApproxTrie::build(&approx_entries);
        Ok(Self {
            corpus,
            config,
            approx_trie,
            approx_entries,
        })
    }

    /// Access the underlying corpus (handy for transcription
    /// debugging and tests).
    pub fn corpus(&self) -> &Corpus {
        &self.corpus
    }

    /// Configuration in effect.
    pub fn config(&self) -> &GeneratorConfig {
        &self.config
    }

    /// Replace the search configuration (beam width, top N, mode…).
    /// The parsed corpus/trie is untouched, so this is cheap — the
    /// wasm wrapper uses it per `generate` call.
    pub fn set_config(&mut self, config: GeneratorConfig) {
        self.config = config;
    }

    pub fn generate(&self, target: &str) -> Vec<Clue> {
        // Exact mode keeps the historical raw-IPA behavior. Approximate
        // mode works on a stress-stripped stream so lexical stress
        // marks (which legitimately differ between near-homophones)
        // never block or tax a match.
        let is_approx = matches!(self.config.mode, SearchMode::Approximate { .. });
        if !is_approx {
            return self.generate_exact(target);
        }
        let Some((target_ipa, target_boundaries)) =
            transcribe_normalized_with_boundaries(&self.corpus, target)
        else {
            return Vec::new();
        };
        // Normalized target vocabulary for the word-novelty axis
        // (final and prefix alike): "it's" and "its" count as the
        // same recycled word.
        let target_words: HashSet<String> = target
            .split_whitespace()
            .map(|w| Partial::norm_word(w.to_lowercase().trim_end_matches(['.', ',', '!', '?'])))
            .collect();
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        // beam[p] = Pareto-banded coverings of [0..p). BeamPos
        // keeps per-(word-count, cost-tier) cells with incremental
        // stats so the hot pair loop pre-filters arithmetically
        // (clone only on admission). Roomy beams stay affordable,
        // which is what lets valid mid-pack parses survive
        // alongside hundreds of near-tie rivals.
        let k = self.config.beam_width;
        let mut beam: Vec<BeamPos> = (0..=n).map(|_| BeamPos::new(k)).collect();
        beam[0].insert(Partial::empty());
        // Terminal retention is by final score (see CompletionTop):
        // every distinct closed parse contends for a generous
        // capped pool, then the pool is sorted, deduped and
        // diversity-selected to top_n. The pool is deliberately roomy
        // (tens of thousands): near-tie resegmentation families number
        // in the thousands, and a mid-pack true resegmentation only
        // reaches the MMR/diversity stage if the pool holds it — a
        // tight pool keeps parrot twins and buries it. One merged
        // pool; MMR over it stays interactive (linearithmic in pool
        // size times top_n).
        let completion_cap = self.config.top_n.saturating_mul(64).max(32768);
        let mut completed = CompletionTop::new(completion_cap);

        let (per_word_budget, total_budget) = match self.config.mode {
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => (per_word_budget, total_budget),
            SearchMode::Exact => unreachable!(),
        };

        // TEMPORARY completion diagnostic (env-driven, generic): if
        // `MADGAB_TRACE_COMPLETE` holds a space-separated word
        // sequence, log whenever exactly that word sequence (matched
        // case-insensitively per word) completes, plus the pool floor
        // at drain. No literal words in source.
        let trace_complete: Vec<String> =
            std::env::var("MADGAB_TRACE_COMPLETE").map_or(Vec::new(), |s| {
                s.split_whitespace().map(|w| w.to_lowercase()).collect()
            });
        // Reference (score, raw_nov, fam, cost) of the traced
        // completion, captured above and used for the pool
        // distribution diagnostic at drain.
        let mut trace_ref: Option<(f64, f64, f64, f64)> = None;

        for p in 0..n {
            if beam[p].is_empty() {
                continue;
            }
            // The edit-tolerant trie walk depends only on the position
            // and the per-word budget, not on the partial path, so run
            // it once per position (not once per beam entry) and filter
            // per partial by remaining total budget below.
            let mut matches = self.approx_trie.words_approximately_starting_at(
                &self.approx_entries,
                &chars,
                p,
                per_word_budget,
                250,
            );
            matches.retain(|m| {
                m.word_len >= self.config.min_word_ipa_chars
                    && m.consumed >= self.config.min_word_ipa_chars
            });
            if matches.is_empty() {
                continue;
            }
            // Take ownership of the beam-at-p so we can mutate beam[p..] freely.
            let here = beam[p].take_entries();
            if !trace_complete.is_empty() {
                // Longest env-traced prefix present at this position
                // (case-insensitive word match), with its metrics.
                let mut best: Option<(usize, f64, f64, f64, f64)> = None;
                for partial in &here {
                    let mut j = 0usize;
                    while j < trace_complete.len()
                        && j < partial.words.len()
                        && partial.words[j].word.to_lowercase() == trace_complete[j]
                    {
                        j += 1;
                    }
                    if j > 0 && best.as_ref().is_none_or(|b| j > b.0) {
                        best = Some((
                            j,
                            partial.sub_cost_total,
                            partial.cheap_score,
                            partial.novelty,
                            partial.familiarity,
                        ));
                    }
                }
                if let Some((j, sub, cheap, nov, fam)) = best {
                    eprintln!(
                        "TRACE-PREFIX pos={p} n={} len={j} sub={sub:.3} cheap={cheap:.3} nov={nov:.3} fam={fam:.3}",
                        here.len(),
                    );
                }
            }
            for partial in &here {
                // Length of traced prefix held by this partial (only
                // exact-prefix holders can extend the trace).
                let held = if trace_complete.is_empty() {
                    usize::MAX
                } else {
                    let mut j = 0usize;
                    while j < trace_complete.len()
                        && j < partial.words.len()
                        && partial.words[j].word.to_lowercase() == trace_complete[j]
                    {
                        j += 1;
                    }
                    if j == partial.words.len() {
                        j
                    } else {
                        usize::MAX
                    }
                };
                let remaining_budget = total_budget - partial.sub_cost_total;
                for m in matches.iter() {
                    if m.cost > remaining_budget + 1e-9 {
                        continue;
                    }
                    let cand_sub = partial.sub_cost_total + m.cost;
                    if cand_sub > total_budget + 1e-9 {
                        continue;
                    }
                    let end = p + m.consumed;
                    let terminal = end == n;
                    // Partial-estimate gate (no drift: identical to
                    // the heuristic stored by `extend_approx`).
                    // Terminal parses skip it and are retained by
                    // final score.
                    let (cand_bound, _cand_nov, cand_raw, cand_fam) = Partial::candidate_metrics(
                        partial,
                        cand_sub,
                        end,
                        &m.word,
                        m.ipa.chars().count(),
                        m.rarity,
                        &target_boundaries,
                        n,
                        &target_words,
                    );
                    if !terminal
                        && !beam[end].would_admit(
                            &partial.key,
                            partial.key.is_empty(),
                            &m.word,
                            &m.ipa,
                            partial.words.len() + 1,
                            cand_sub,
                            cand_bound,
                            cand_raw,
                            cand_fam,
                            m.cost,
                        )
                    {
                        if held != usize::MAX
                            && held < trace_complete.len()
                            && m.word.to_lowercase() == trace_complete[held]
                        {
                            eprintln!(
                                "TRACE-EXT REJECT p={p} end={end} len={} word_cost={:.3} sub={:.3} cheap={:.3} rawnov={:.3} fam={:.3} {}",
                                held + 1,
                                m.cost,
                                cand_sub,
                                cand_bound,
                                cand_raw,
                                cand_fam,
                                beam[end].trace_cell_stats(
                                    partial.words.len() + 1,
                                    cand_sub,
                                    cand_bound,
                                    cand_raw,
                                    cand_fam,
                                    m.cost,
                                    &partial.words
                                ),
                            );
                        }
                        continue;
                    }
                    if held != usize::MAX
                        && held < trace_complete.len()
                        && m.word.to_lowercase() == trace_complete[held]
                    {
                        eprintln!(
                            "TRACE-EXT ADMIT p={p} end={end} len={} word_cost={:.3} sub={:.3} cheap={:.3} rawnov={:.3} fam={:.3} terminal={terminal} {}",
                            held + 1,
                            m.cost,
                            cand_sub,
                            cand_bound,
                            cand_raw,
                            cand_fam,
                            beam[end].trace_cell_stats(
                                partial.words.len() + 1,
                                cand_sub,
                                cand_bound,
                                cand_raw,
                                cand_fam,
                                m.cost,
                                &partial.words
                            ),
                        );
                    }
                    let next = partial.extend_approx(
                        &m.word,
                        &m.ipa,
                        m.rarity,
                        m.consumed,
                        m.cost,
                        &target_boundaries,
                        n,
                        &target_words,
                    );
                    if terminal {
                        let score = next.final_score(&target_boundaries, &target_words);
                        if !trace_complete.is_empty()
                            && next.words.len() == trace_complete.len()
                            && next
                                .words
                                .iter()
                                .zip(trace_complete.iter())
                                .all(|(w, t)| w.word.to_lowercase() == *t)
                        {
                            eprintln!(
                                "TRACE-COMPLETE score={score:.4} sub={:.3} key={:?}",
                                next.sub_cost_total, next.key,
                            );
                            trace_ref = Some((
                                score,
                                next.raw_novelty,
                                next.familiarity,
                                next.sub_cost_total,
                            ));
                        }
                        completed.insert(next, score);
                    } else {
                        // TEMPORARY traced-insert probe (generic;
                        // env-hook only): log the insert decision for
                        // traced extensions.
                        let traced_here = held != usize::MAX
                            && held < trace_complete.len()
                            && m.word.to_lowercase() == trace_complete[held];
                        let trace_cheap = next.cheap_score;
                        let trace_dbg = if traced_here {
                            beam[end].trace_debug_for(partial.words.len() + 1, cand_sub)
                        } else {
                            String::new()
                        };
                        let why = beam[end].insert(next);
                        if traced_here {
                            eprintln!(
                                "TRACE-INSERT p={p} end={end} len={} why={why} cheap={:.3} {}",
                                held + 1,
                                trace_cheap,
                                trace_dbg,
                            );
                        }
                    }
                }
            }
        }

        let partials = completed.drain_sorted_traced(&trace_complete, trace_ref);
        // Axis champions by proxy axes (the drain is score-sorted, so
        // a strict scan keeps the highest-score holder of each best):
        // best raw boundary novelty, best familiarity, lowest cost.
        // Phrases are captured here because `into_clue` consumes the
        // partials below.
        let mut champ_phrases: Vec<String> = Vec::new();
        {
            let mut best_nov = f64::NEG_INFINITY;
            let mut best_fam = f64::NEG_INFINITY;
            let mut best_cost = f64::INFINITY;
            let mut phrases = [None, None, None];
            for p in &partials {
                if p.raw_novelty > best_nov {
                    best_nov = p.raw_novelty;
                    phrases[0] = Some(
                        p.words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>().join(" "),
                    );
                }
                if p.familiarity > best_fam {
                    best_fam = p.familiarity;
                    phrases[1] = Some(
                        p.words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>().join(" "),
                    );
                }
                if p.sub_cost_total < best_cost {
                    best_cost = p.sub_cost_total;
                    phrases[2] = Some(
                        p.words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>().join(" "),
                    );
                }
            }
            for ph in phrases.into_iter().flatten() {
                if !champ_phrases.contains(&ph) {
                    champ_phrases.push(ph);
                }
            }
        }
        let mut clues: Vec<Clue> = partials
            .into_iter()
            .map(|p| p.into_clue(&target_ipa, &target_boundaries, &target_words))
            .collect();
        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        clues.dedup_by(|a, b| a.phrase == b.phrase);
        // Multiobjective seats: pull the champions' phrases out of the
        // MMR pool, MMR the rest to the remaining seats, then append
        // the champions in score order. One merged pool; bounded;
        // generic word-overlap only.
        let (champ_clues, rest_clues): (Vec<Clue>, Vec<Clue>) =
            clues.into_iter().partition(|c| champ_phrases.contains(&c.phrase));
        let seats = self.config.top_n.min(champ_clues.len().min(FINAL_QUOTA));
        let mut champs_sorted = champ_clues;
        champs_sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        champs_sorted.truncate(seats);
        let rest_n = self.config.top_n.saturating_sub(champs_sorted.len());
        let mut out = select_diverse(rest_clues, rest_n);
        out.extend(champs_sorted);
        out
    }

    /// Historical exact tiling over raw (stress-bearing) IPA.
    fn generate_exact(&self, target: &str) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_with_boundaries(&self.corpus, target)
        else {
            return Vec::new();
        };
        let target_words: HashSet<String> = target
            .split_whitespace()
            .map(|w| Partial::norm_word(w.to_lowercase().trim_end_matches(['.', ',', '!', '?'])))
            .collect();
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
                    if consumed < self.config.min_word_ipa_chars {
                        continue;
                    }
                    let next = partial.extend(pronunciation, consumed, 0.0);
                    insert_top_k(&mut beam[p + consumed], next, self.config.beam_width);
                }
            }
        }
        let mut completed = std::mem::take(&mut beam[n]);
        let mut clues: Vec<Clue> = completed
            .drain(..)
            .map(|p| p.into_clue(&target_ipa, &target_boundaries, &target_words))
            .collect();
        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        clues.dedup_by(|a, b| a.phrase == b.phrase);
        clues.truncate(self.config.top_n);
        clues
    }
}

// -----------------------------------------------------------------
// Internals
// -----------------------------------------------------------------

/// Like `Corpus::transcribe` but also returns the running set of
/// char offsets at each word boundary, so we can later score how
/// much a candidate clue rearranges them.
fn transcribe_with_boundaries(corpus: &Corpus, phrase: &str) -> Option<(String, Vec<usize>)> {
    let mut out = String::new();
    let mut boundaries: Vec<usize> = Vec::new();
    for word in phrase.split_whitespace() {
        let key = word
            .to_lowercase()
            .trim_end_matches(['.', ',', '!', '?', ';', ':'])
            .to_string();
        let ipa = corpus.preferred_ipa(&key)?;
        out.push_str(ipa);
        boundaries.push(out.chars().count());
    }
    Some((out, boundaries))
}

/// Normalized transcription for Approximate mode: preferred IPA per
/// word with lexical stress marks stripped and ASCII `g` folded to
/// IPA `ɡ`, so stress placement (which legitimately differs between
/// near-homophones) neither blocks nor taxes a match. Boundaries are
/// measured in the normalized stream, consistent with the
/// stress-stripped clue word IPAs stored in [`ApproxEntry`].
fn transcribe_normalized_with_boundaries(
    corpus: &Corpus,
    phrase: &str,
) -> Option<(String, Vec<usize>)> {
    let mut out = String::new();
    let mut boundaries: Vec<usize> = Vec::new();
    for word in phrase.split_whitespace() {
        let key = word
            .to_lowercase()
            .trim_end_matches(['.', ',', '!', '?', ';', ':'])
            .to_string();
        let ipa = corpus.preferred_ipa(&key)?;
        out.push_str(&normalize_ipa_str(ipa));
        boundaries.push(out.chars().count());
    }
    Some((out, boundaries))
}

/// Strip suprasegmental stress/length marks and fold IPA `ɡ` to
/// ASCII `g`. Applied identically to target streams and clue entries.
/// The fold direction matters: `phonetics-rs`' acoustic tables key
/// the velar stop as ASCII `g` (its own tests transcribe "dog" as
/// "dɔg"), so folding toward `ɡ` would price every g-involved
/// substitution at the 1.0 fallback instead of ~0.15.
fn normalize_ipa_str(s: &str) -> String {
    s.chars()
        .filter(|&c| c != 'ˈ' && c != 'ˌ' && c != 'ː')
        .map(|c| if c == 'ɡ' { 'g' } else { c })
        .collect()
}

/// Indel (insertion/deletion) price for one segment, mirroring the
/// listener-confusion tiers: routinely dropped/inserted weak phonemes
/// (/ə/, /h/) are cheap, everything else pays a full gap-open.
fn indel_cost(c: char) -> f64 {
    match c {
        'ə' | 'h' | 'ʔ' | 'ɦ' => 0.15,
        _ => 0.60,
    }
}

/// One stress-stripped pronunciation entry backing Approximate mode.
#[derive(Debug, Clone)]
struct ApproxEntry {
    word: String,
    /// Stress-stripped, g-folded IPA.
    ipa: String,
    chars: Vec<char>,
    rarity: Option<f64>,
}

impl ApproxEntry {
    /// Parse the same JSON shape `Corpus` expects and mirror its
    /// `max_rarity` filtering, keeping every source pronunciation as
    /// its own entry (deduplicated). Parse failures (foreign corpora
    /// in unit tests use the same shape, so this succeeds there too)
    /// yield an empty list rather than an error; the main `Corpus`
    /// parse remains the authoritative fallible one.
    fn from_json(json: &str, max_rarity: Option<f64>) -> Vec<Self> {
        let raw: HashMap<String, RawApproxEntry> = match serde_json::from_str(json) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let mut out: Vec<Self> = Vec::new();
        let mut seen: HashSet<(String, String)> = HashSet::new();
        for (word, entry) in raw {
            if let (Some(rarity), Some(cap)) = (entry.rarity, max_rarity) {
                if rarity > cap {
                    continue;
                }
            }
            for ipa in entry.ipa.values() {
                if ipa.is_empty() {
                    continue;
                }
                let norm = normalize_ipa_str(ipa);
                if norm.is_empty() {
                    continue;
                }
                if !seen.insert((word.clone(), norm.clone())) {
                    continue;
                }
                out.push(Self {
                    word: word.clone(),
                    chars: norm.chars().collect::<Vec<_>>(),
                    ipa: norm,
                    rarity: entry.rarity,
                });
            }
        }
        out
    }
}

#[derive(Debug, serde::Deserialize, Default)]
struct RawApproxEntry {
    #[serde(default)]
    rarity: Option<f64>,
    #[serde(default)]
    ipa: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct ApproxNode {
    children: BTreeMap<char, usize>,
    /// Indices into the entry list terminating here.
    terminations: Vec<usize>,
}

/// Stress-stripped trie with an edit-tolerant (match / substitute /
/// insert / delete) fuzzy walk. Substitution prices come from
/// `phonetics::distance` on single segments; indels use
/// [`indel_cost`]. Stress marks never enter the index at all.
#[derive(Debug, Default)]
struct ApproxTrie {
    nodes: Vec<ApproxNode>,
    /// Cache of single-segment substitution prices,
    /// `phonetics::distance` on one-char strings, which the fuzzy
    /// walk would otherwise recompute (with fresh allocations) for
    /// every trie step. The IPA segment alphabet is tiny, so this
    /// stays small; interior mutability keeps the walk API
    /// call-site clean (all use is single-threaded).
    dist_cache: std::cell::RefCell<HashMap<(char, char), f64>>,
}

/// One fuzzy-trie hit: `consumed` target chars covered starting at the
/// query offset, at `cost`, by the entry's word/IPA.
#[derive(Clone)]
struct ApproxMatch {
    consumed: usize,
    word_len: usize,
    word: String,
    ipa: String,
    rarity: Option<f64>,
    cost: f64,
}

impl ApproxTrie {
    fn build(entries: &[ApproxEntry]) -> Self {
        let mut nodes: Vec<ApproxNode> = vec![ApproxNode::default()];
        for (idx, e) in entries.iter().enumerate() {
            let mut node = 0_usize;
            for &c in &e.chars {
                let next = if let Some(&n) = nodes[node].children.get(&c) {
                    n
                } else {
                    let n = nodes.len();
                    nodes.push(ApproxNode::default());
                    nodes[node].children.insert(c, n);
                    n
                };
                node = next;
            }
            nodes[node].terminations.push(idx);
        }
        Self {
            nodes,
            dist_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    /// Single-segment substitution price with memoization (see
    /// [`ApproxTrie::dist_cache`]).
    fn seg_dist(&self, a: char, b: char) -> f64 {
        if a == b {
            return 0.0;
        }
        if let Some(&d) = self.dist_cache.borrow().get(&(a, b)) {
            return d;
        }
        let d = phonetics::distance(&a.to_string(), &b.to_string());
        self.dist_cache.borrow_mut().insert((a, b), d);
        d
    }

    /// Edit-tolerant prefix lookup: all entries whose stripped IPA
    /// aligns to `chars[offset..]` within `budget`, via substitution,
    /// insertion (trie segment with no target coverage), and deletion
    /// (target segment with no trie coverage). Insertions are capped
    /// at 2 per word and deletions are bounded by the budget, keeping
    /// the walk polynomial; results are capped to the cheapest few
    /// hundred per position for interactive latency.
    fn words_approximately_starting_at(
        &self,
        entries: &[ApproxEntry],
        chars: &[char],
        offset: usize,
        budget: f64,
        cap: usize,
    ) -> Vec<ApproxMatch> {
        if self.nodes.is_empty() {
            return Vec::new();
        }
        // Cheapest-first (Dijkstra) queue over (node, target_pos):
        // step costs are non-negative, so the first pop of a state
        // carries its minimum cost and later (costlier) revisits are
        // pruned before expanding. A LIFO stack rediscovers the same
        // states over and over (each cheaper arrival re-expands the
        // whole subtree); ordering cuts the walk to roughly one
        // expansion per reachable state with identical results
        // (all recorded costs are true minima either way). Costs
        // are non-negative, so `to_bits` preserves numeric order;
        // the sequence number keeps ties deterministic.
        // Frames: (cost-bits, seq, node, target_pos, insertions_used).
        // Cost round-trips exactly through `to_bits`/`from_bits`,
        // keeping the heap tuple `Ord` (f64 is not).
        let mut heap: std::collections::BinaryHeap<(
            std::cmp::Reverse<u64>,
            u64,
            usize,
            usize,
            u8,
        )> = std::collections::BinaryHeap::new();
        let mut seq = 0u64;
        heap.push((std::cmp::Reverse(0.0f64.to_bits()), seq, 0, offset, 0));
        seq += 1;
        // (node, target_pos) -> best cost seen; dominates revisits.
        let mut best: HashMap<(usize, usize), f64> = HashMap::new();
        // (entry_idx, consumed) -> best cost; one row per alignment.
        let mut hits: HashMap<(usize, usize), f64> = HashMap::new();
        // Hard cap on target chars a single word may cover: longest
        // indexed word plus room for deletions paid out of the budget.
        let max_extra_del = (budget / 0.15).ceil() as usize + 2;

        while let Some((bits, _, node, tpos, ins)) = heap.pop() {
            let cost = f64::from_bits(bits.0);
            if cost > budget + 1e-9 {
                continue;
            }
            if tpos > chars.len() {
                continue;
            }
            if let Some(&b) = best.get(&(node, tpos)) {
                if cost >= b - 1e-12 {
                    continue;
                }
            }
            best.insert((node, tpos), cost);

            let consumed = tpos - offset;
            for &eidx in &self.nodes[node].terminations {
                if consumed == 0 {
                    continue;
                }
                hits.entry((eidx, consumed))
                    .and_modify(|b| *b = b.min(cost))
                    .or_insert(cost);
            }

            // Deletion: skip a target segment without advancing the trie.
            if tpos < chars.len() {
                let dc = indel_cost(chars[tpos]);
                let nc = cost + dc;
                if nc <= budget + 1e-9 && consumed < 12 + max_extra_del {
                    heap.push((std::cmp::Reverse(nc.to_bits()), seq, node, tpos + 1, ins));
                    seq += 1;
                }
            }

            for (&trie_c, &child) in &self.nodes[node].children {
                // Insertion: consume a trie segment without covering target.
                if ins < 2 {
                    let ic = indel_cost(trie_c);
                    let nc = cost + ic;
                    if nc <= budget + 1e-9 {
                        heap.push((std::cmp::Reverse(nc.to_bits()), seq, child, tpos, ins + 1));
                        seq += 1;
                    }
                }
                // Match / substitution: advance both sides.
                if tpos < chars.len() {
                    let step = self.seg_dist(chars[tpos], trie_c);
                    let nc = cost + step;
                    if nc <= budget + 1e-9 {
                        heap.push((std::cmp::Reverse(nc.to_bits()), seq, child, tpos + 1, ins));
                        seq += 1;
                    }
                }
            }
        }

        let mut out: Vec<ApproxMatch> = hits
            .into_iter()
            .map(|((eidx, consumed), cost)| {
                let e = &entries[eidx];
                ApproxMatch {
                    consumed,
                    word_len: e.chars.len(),
                    word: e.word.clone(),
                    ipa: e.ipa.clone(),
                    rarity: e.rarity,
                    cost,
                }
            })
            .collect();
        // Dedup identical (word, consumed) keeping cheapest. The
        // sort is total (cost, word, IPA, consumed) so the HashMap
        // iteration order above cannot leak nondeterminism into which
        // duplicate survives — same input must give same output.
        out.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.word.cmp(&b.word))
                .then_with(|| a.ipa.cmp(&b.ipa))
                .then_with(|| a.consumed.cmp(&b.consumed))
        });
        let mut seen: HashSet<(String, usize)> = HashSet::new();
        out.retain(|m| seen.insert((m.word.clone(), m.consumed)));
        shortlist_diverse(out, cap)
    }
}

/// Bounded multi-objective shortlist: a cheapest-only cap
/// systematically drops expensive-but-common resegmentation words,
/// so keep a bounded union of acoustic-cost elites and
/// lexical-familiarity elites, stratified by consumed target span
/// (span lengths are incomparable strategies — a short exact word
/// must not evict a longer near-match covering more ground).
/// Remaining slots are filled cheapest-first; over-cap trims shrink
/// the largest spans first so every consumed length keeps
/// representation.
fn shortlist_diverse(out: Vec<ApproxMatch>, cap: usize) -> Vec<ApproxMatch> {
    fn familiarity(m: &ApproxMatch) -> f64 {
        m.rarity.unwrap_or(f64::INFINITY)
    }
    let mut by_span: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, m) in out.iter().enumerate() {
        by_span.entry(m.consumed).or_default().push(i);
    }
    const PER_SPAN_COST: usize = 32;
    const PER_SPAN_FAM: usize = 32;
    let mut kept: HashSet<usize> = HashSet::new();
    // Familiarity-axis picks: exempt from over-cap trimming
    // below (trimming them would defeat the union).
    let mut fam_kept: HashSet<usize> = HashSet::new();
    for idxs in by_span.values() {
        let mut by_cost = idxs.clone();
        by_cost.sort_by(|&a, &b| {
            out[a]
                .cost
                .partial_cmp(&out[b].cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| out[a].word.cmp(&out[b].word))
                .then_with(|| out[a].ipa.cmp(&out[b].ipa))
        });
        let mut by_fam = idxs.clone();
        by_fam.sort_by(|&a, &b| {
            familiarity(&out[a])
                .partial_cmp(&familiarity(&out[b]))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    out[a]
                        .cost
                        .partial_cmp(&out[b].cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| out[a].word.cmp(&out[b].word))
                .then_with(|| out[a].ipa.cmp(&out[b].ipa))
        });
        for &i in by_cost.iter().take(PER_SPAN_COST) {
            kept.insert(i);
        }
        for &i in by_fam.iter().take(PER_SPAN_FAM) {
            kept.insert(i);
            fam_kept.insert(i);
        }
    }
    // Over-cap stratified trim on the kept index set: shrink
    // the largest spans first by highest acoustic cost so every
    // consumed length keeps representation. Familiarity-axis
    // picks are exempt (removing them would defeat the union);
    // only if they alone exceed the cap are they trimmed too.
    while kept.len() > cap {
        let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
        for &i in &kept {
            *counts.entry(out[i].consumed).or_default() += 1;
        }
        let Some((&span, _)) = counts.iter().max_by_key(|(_, &c)| c) else {
            break;
        };
        // Highest cost in the span, preferring non-familiarity
        // picks; fall back to familiarity picks when the span
        // holds nothing else. Fully tie-broken (cost, word, IPA,
        // span) so HashSet iteration order cannot leak
        // nondeterminism — even true twins (same word/IPA/cost at
        // different spans) resolve identically every run.
        let victim = kept
            .iter()
            .filter(|&&i| out[i].consumed == span && !fam_kept.contains(&i))
            .max_by(|&&a, &&b| {
                out[a]
                    .cost
                    .partial_cmp(&out[b].cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| out[a].word.cmp(&out[b].word))
                    .then_with(|| out[a].ipa.cmp(&out[b].ipa))
                    .then_with(|| out[a].consumed.cmp(&out[b].consumed))
            })
            .or_else(|| {
                kept.iter()
                    .filter(|&&i| out[i].consumed == span)
                    .max_by(|&&a, &&b| {
                        out[a]
                            .cost
                            .partial_cmp(&out[b].cost)
                            .unwrap_or(std::cmp::Ordering::Equal)
                            .then_with(|| out[a].word.cmp(&out[b].word))
                            .then_with(|| out[a].ipa.cmp(&out[b].ipa))
                            .then_with(|| out[a].consumed.cmp(&out[b].consumed))
                    })
            })
            .copied();
        if let Some(v) = victim {
            kept.remove(&v);
        } else {
            break;
        }
    }
    let mut shortlist: Vec<ApproxMatch> = Vec::new();
    // Drain via swap-remove style: take kept indices out of `out`.
    let mut kept_idx: Vec<usize> = kept.into_iter().collect();
    kept_idx.sort_unstable();
    let mut kept_set: HashSet<usize> = kept_idx.iter().copied().collect();
    let mut rest: Vec<ApproxMatch> = Vec::new();
    for (i, m) in out.into_iter().enumerate() {
        if kept_set.remove(&i) {
            shortlist.push(m);
        } else {
            rest.push(m);
        }
    }
    // Fill remaining slots cheapest-first (total order keeps the
    // HashMap-derived input order from leaking nondeterminism).
    rest.sort_by(|a, b| {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.word.cmp(&b.word))
            .then_with(|| a.ipa.cmp(&b.ipa))
            .then_with(|| a.consumed.cmp(&b.consumed))
    });
    for m in rest {
        if shortlist.len() >= cap {
            break;
        }
        shortlist.push(m);
    }
    // Deterministic order for downstream iteration.
    shortlist.sort_by(|a, b| {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.word.cmp(&b.word))
            .then_with(|| a.ipa.cmp(&b.ipa))
            .then_with(|| a.consumed.cmp(&b.consumed))
    });
    shortlist
}

/// A partially-constructed clue: the words chosen so far plus the
/// running cheap score used to prune the beam. Boundaries (the
/// per-word char-offset cuts) are derived from `words` at scoring
/// time.
#[derive(Debug, Clone)]
struct Partial {
    words: Vec<ClueWord>,
    /// Accumulated substitution cost across all words so far.
    /// Always zero in Exact mode.
    sub_cost_total: f64,
    /// Beam ranking signal. Exact mode: running cheap score.
    /// Approximate mode: prefix heuristic aligned with
    /// `final_score` (same 0.40/0.35/0.15/0.10 weights over
    /// similarity, Jaccard cut novelty, word novelty, length
    /// signal); ranking only, not a bound.
    cheap_score: f64,
    /// Cached prefix boundary novelty (coverage-shrunk Jaccard over
    /// stored target-aligned cuts). Ranking signal inside
    /// `cheap_score`.
    novelty: f64,
    /// Raw (unshrunk) prefix Jaccard novelty over stored cuts.
    /// Reservoir axis (b): shrinkage exists to stop novel-cut junk
    /// from outranking on the scalar, but for *protection* the raw
    /// resegmentation signal is the right one — a genuine valley
    /// prefix that paid cost early must not lose its novelty foothold
    /// to coverage discounting. Junk sheltered this way is harmless:
    /// the final scorer still demotes it.
    raw_novelty: f64,
    /// Cached lexical familiarity: smooth normalized log-frequency
    /// average over chosen words' rarity. Powers axis (c).
    familiarity: f64,
    /// Substitution cost of the most recent word (0.0 for the empty
    /// root). Powers reservoir axis (e): the most expensive last
    /// link. A near-budget link marks a committed valley hypothesis —
    /// few peers can afford it, and only the final scorer (with the
    /// full parse in hand) can tell valley from junk — so a bounded
    /// shelter keeps them. Junk sheltered this way is harmless: the
    /// final scorer still demotes it.
    last_cost: f64,
    /// Cumulative target-stream cuts (end offsets) per word.
    /// Approximate mode only; empty in Exact mode. Target-aligned
    /// segmentation history used for final novelty scoring and for
    /// the prefix heuristic. Never part of the beam cell key.
    cuts: Vec<usize>,
    /// Cached clone-detection key: the space-joined per-step
    /// `(word, IPA)` pairs of the words so far. Exact duplicate
    /// corpus records (same word through different sources) collapse
    /// to one slot, but homophones with identical pronunciation and
    /// different lexical identity stay distinct: lexical alternatives
    /// are the puzzle output, and word identity feeds
    /// familiarity/word-novelty scoring, so collapsing by sound alone
    /// would reject valid clues behind a same-sounding rival.
    key: String,
    /// Cached normalized word sequence of the prefix (all words but
    /// the last, lowercased alphanumerics joined by spaces; empty for
    /// prefixes shorter than two words). Powers the beam's
    /// prefix-family quota: members sharing a prefix differ only in
    /// the last word (same cuts, same novelty), so the beam can
    /// resolve them against each other instead of letting one
    /// prolific prefix hog the cell. Empty keys are exempt (early
    /// short links like single function words must never be capped
    /// by family).
    prefix_key: String,
}

impl Partial {
    fn empty() -> Self {
        Self {
            words: Vec::new(),
            sub_cost_total: 0.0,
            cheap_score: 0.55,
            novelty: 0.5,
            raw_novelty: 0.5,
            familiarity: 0.0,
            last_cost: 0.0,
            cuts: Vec::new(),
            key: String::new(),
            prefix_key: String::new(),
        }
    }

    /// Normalized word form for target-word reuse detection:
    /// lowercase, alphanumeric characters only.
    fn norm_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// One clone-key step: collision-safe `(word, IPA)` pair. The
    /// word part is lowercased only — apostrophes are meaningful
    /// ("its" vs "it's" stay distinct) — and `\x1f` separates the
    /// fields so no word/IPA concatenation can collide.
    fn step_key(word: &str, ipa: &str) -> String {
        format!("{}\x1f{}", word.to_lowercase(), ipa)
    }

    /// Clone key of the path extended by one step.
    fn join_key(base: &str, word: &str, ipa: &str) -> String {
        let step = Self::step_key(word, ipa);
        if base.is_empty() {
            step
        } else {
            format!("{base} {step}")
        }
    }

    fn extend(&self, p: &Pronunciation, _consumed: usize, word_sub_cost: f64) -> Self {
        Self::extend_words(self, &p.word, &p.ipa, p.rarity, word_sub_cost, 0.0, 0.0)
    }

    /// Axis projections for the beam reservoir (function items so
    /// the quota sets can share one implementation).
    fn raw_novelty_proj(p: &Partial) -> f64 {
        p.raw_novelty
    }
    fn familiarity_proj(p: &Partial) -> f64 {
        p.familiarity
    }
    fn sub_cost_proj(p: &Partial) -> f64 {
        p.sub_cost_total
    }
    fn last_cost_proj(p: &Partial) -> f64 {
        p.last_cost
    }
    fn cheap_proj(p: &Partial) -> f64 {
        p.cheap_score
    }

    /// Normalized prefix key for a word list: normalized words of    /// all but the last, joined; empty for lists shorter than three
    /// words (family quotas start at length three so early short
    /// links are never capped by family).
    fn prefix_key_for(words: &[ClueWord]) -> String {
        if words.len() < 3 {
            return String::new();
        }
        words[..words.len() - 1]
            .iter()
            .map(|w| Self::norm_word(&w.word))
            .collect::<Vec<_>>()
            .join(" ")
    }
    /// Smooth normalized log-frequency familiarity of one word:
    /// 1.0 at rarity <= 100, 0.0 at rarity >= 50k (or unknown), linear
    /// in log10 between. No cutoffs or lexical cases.
    fn word_familiarity(rarity: Option<f64>) -> f64 {
        let r = match rarity {
            Some(r) if r.is_finite() => r,
            _ => return 0.0,
        };
        let lo = 2.0f64; // log10(100)
        let hi = 50_000f64.log10();
        (1.0 - (r.max(1.0).log10() - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    fn avg_familiarity(words: &[ClueWord]) -> f64 {
        if words.is_empty() {
            return 0.0;
        }
        words.iter().map(|w| Self::word_familiarity(w.rarity)).sum::<f64>()
            / words.len() as f64
    }

    /// Coverage-shrunk prefix Jaccard novelty over stored cuts,
    /// plus the raw (unshrunk) value for reservoir protection.
    fn prefix_novelty(
        cuts: &[usize],
        target_boundaries: &[usize],
        total_len: usize,
    ) -> (f64, f64) {
        let cum = cuts.last().copied().unwrap_or(0);
        let mut t_prefix = 0usize;
        for &b in target_boundaries {
            if b < total_len && b <= cum {
                t_prefix += 1;
            }
        }
        let c_len = cuts.len();
        let mut shared = 0usize;
        for &c in cuts {
            for &b in target_boundaries {
                if b < total_len && b <= cum && b == c {
                    shared += 1;
                    break;
                }
            }
        }
        let union = t_prefix + c_len - shared.min(t_prefix.min(c_len));
        let raw = if union == 0 { 0.0 } else { 1.0 - (shared as f64 / union as f64) };
        let coverage = if total_len == 0 {
            0.0
        } else {
            (cum as f64 / total_len as f64).clamp(0.0, 1.0)
        };
        (coverage * raw + (1.0 - coverage) * 0.5, raw)
    }

    #[allow(clippy::too_many_arguments)]
    fn extend_approx(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        consumed: usize,
        word_sub_cost: f64,
        target_boundaries: &[usize],
        total_len: usize,
        target_words: &HashSet<String>,
    ) -> Self {
        let base = self.cuts.last().copied().unwrap_or(0);
        let end = base + consumed;
        let mut cuts = self.cuts.clone();
        cuts.push(end);
        let sub_total = self.sub_cost_total + word_sub_cost;
        let mut words = self.words.clone();
        words.push(ClueWord {
            word: word.to_string(),
            ipa: ipa.to_string(),
            rarity,
            sub_cost: word_sub_cost,
        });
        let cheap = Self::partial_heuristic(
            sub_total,
            &cuts,
            &words,
            target_boundaries,
            total_len,
            target_words,
        );
        let novelty = Self::prefix_novelty(&cuts, target_boundaries, total_len);
        let familiarity = Self::avg_familiarity(&words);
        let prefix_key = Self::prefix_key_for(&words);
        Self {
            words,
            sub_cost_total: sub_total,
            cheap_score: cheap,
            novelty: novelty.0,
            raw_novelty: novelty.1,
            familiarity,
            last_cost: word_sub_cost,
            cuts,
            key: Self::join_key(&self.key, word, ipa),
            prefix_key,
        }
    }

    /// Partial ranking heuristic aligned with [`Partial::final_score`]:
    /// same four axes with the same weights (0.40 phonetic
    /// similarity from accumulated cost, 0.35 Jaccard boundary-set
    /// novelty over the prefix covered so far, 0.15 word novelty
    /// among words chosen so far, 0.10 average-IPA-length signal).
    /// Computed on the prefix only (target cuts clipped to the
    /// covered prefix, clue cuts = stored target-aligned cuts), so
    /// it is a ranking signal, not an upper bound. Generic
    /// arithmetic only.
    fn partial_heuristic(
        sub_total: f64,
        cuts: &[usize],
        words: &[ClueWord],
        target_boundaries: &[usize],
        total_len: usize,
        target_words: &HashSet<String>,
    ) -> f64 {
        let sim = (1.0 - sub_total / 4.0).clamp(0.0, 1.0);
        let cum = cuts.last().copied().unwrap_or(0);
        // Prefix target cuts: inner boundaries at or before the
        // frontier (the frontier itself counts when it coincides
        // with a target cut — that alignment is shared, not novel).
        let mut t_prefix = 0usize;
        for &b in target_boundaries {
            if b < total_len && b <= cum {
                t_prefix += 1;
            }
        }
        let c_len = cuts.len();
        let mut shared = 0usize;
        for &c in cuts {
            for &b in target_boundaries {
                if b < total_len && b <= cum && b == c {
                    shared += 1;
                    break;
                }
            }
        }
        let union = t_prefix + c_len - shared.min(t_prefix.min(c_len));
        let raw_novelty = if union == 0 { 0.0 } else { 1.0 - (shared as f64 / union as f64) };
        // Coverage shrinkage: on a short prefix the Jaccard sets
        // hold 1-3 cuts, so raw novelty jumps in steps of 0.33+
        // (0.12+ after weighting) — wider than the beam's EPSILON
        // near-tie band — and acoustically weak junk with a novel
        // cut would outrank genuine resegmentations on noise. Shrink
        // toward the neutral prior 0.5 by the uncovered fraction:
        // early ranking rests on the smooth axes (similarity,
        // length), and the term converges to the final weight at
        // completion (cum == total_len). Ranking heuristic only.
        let coverage = if total_len == 0 {
            0.0
        } else {
            (cum as f64 / total_len as f64).clamp(0.0, 1.0)
        };
        let novelty = coverage * raw_novelty + (1.0 - coverage) * 0.5;
        let nwords = words.len().max(1) as f64;
        let reused = words
            .iter()
            .filter(|w| target_words.contains(&Self::norm_word(&w.word)))
            .count() as f64;
        let word_novelty = 1.0 - (reused / nwords);
        let avg_len = if words.is_empty() {
            0.0
        } else {
            words.iter().map(|w| w.ipa.chars().count()).sum::<usize>() as f64 / nwords
        };
        let length_signal = (avg_len / 4.0).min(1.0);
        0.40 * sim + 0.35 * novelty + 0.15 * word_novelty + 0.10 * length_signal
    }

    /// Candidate metrics without building the next Partial
    /// (mirrors `extend_approx` exactly so the lazy gate cannot
    /// drift): returns (heuristic, shrunk novelty, raw novelty,
    /// familiarity).
    #[allow(clippy::too_many_arguments)]
    fn candidate_metrics(
        partial: &Partial,
        cand_sub: f64,
        end: usize,
        cand_word: &str,
        cand_ipa_len: usize,
        cand_rarity: Option<f64>,
        target_boundaries: &[usize],
        total_len: usize,
        target_words: &HashSet<String>,
    ) -> (f64, f64, f64, f64) {
        let mut cuts = partial.cuts.clone();
        cuts.push(end);
        let (novelty, raw_novelty) = Self::prefix_novelty(&cuts, target_boundaries, total_len);
        let n = partial.words.len() + 1;
        let fam_sum: f64 = partial
            .words
            .iter()
            .map(|w| Self::word_familiarity(w.rarity))
            .sum::<f64>()
            + Self::word_familiarity(cand_rarity);
        let familiarity = fam_sum / n as f64;
        let cheap = Self::candidate_heuristic(
            partial,
            cand_sub,
            end,
            cand_word,
            cand_ipa_len,
            target_boundaries,
            total_len,
            target_words,
        );
        (cheap, novelty, raw_novelty, familiarity)
    }

    /// Candidate heuristic without building the next Partial
    /// (mirrors `extend_approx` exactly so the lazy gate cannot
    /// drift): same prefix computation with the candidate word/cut
    /// appended.
    #[allow(clippy::too_many_arguments)]
    fn candidate_heuristic(
        partial: &Partial,
        cand_sub: f64,
        end: usize,
        cand_word: &str,
        cand_ipa_len: usize,
        target_boundaries: &[usize],
        total_len: usize,
        target_words: &HashSet<String>,
    ) -> f64 {
        let sim = (1.0 - cand_sub / 4.0).clamp(0.0, 1.0);
        let cum = end;
        let mut t_prefix = 0usize;
        for &b in target_boundaries {
            if b < total_len && b <= cum {
                t_prefix += 1;
            }
        }
        // Clue cuts = partial cuts + end; count shared by testing
        // each against the prefix target set.
        let mut shared = 0usize;
        for &c in partial.cuts.iter().chain(std::iter::once(&end)) {
            for &b in target_boundaries {
                if b < total_len && b <= cum && b == c {
                    shared += 1;
                    break;
                }
            }
        }
        let c_len = partial.cuts.len() + 1;
        let union = t_prefix + c_len - shared.min(t_prefix.min(c_len));
        let raw_novelty = if union == 0 { 0.0 } else { 1.0 - (shared as f64 / union as f64) };
        // Coverage shrinkage shared with `partial_heuristic` (see
        // it): the gate must mirror the stored priority exactly.
        let coverage = if total_len == 0 {
            0.0
        } else {
            (cum as f64 / total_len as f64).clamp(0.0, 1.0)
        };
        let novelty = coverage * raw_novelty + (1.0 - coverage) * 0.5;
        let nwords = (partial.words.len() + 1) as f64;
        let mut reused = partial
            .words
            .iter()
            .filter(|w| target_words.contains(&Self::norm_word(&w.word)))
            .count() as f64;
        if target_words.contains(&Self::norm_word(cand_word)) {
            reused += 1.0;
        }
        let word_novelty = 1.0 - (reused / nwords);
        let total_ipa: usize = partial
            .words
            .iter()
            .map(|w| w.ipa.chars().count())
            .sum::<usize>()
            + cand_ipa_len;
        let avg_len = total_ipa as f64 / nwords;
        let length_signal = (avg_len / 4.0).min(1.0);
        0.40 * sim + 0.35 * novelty + 0.15 * word_novelty + 0.10 * length_signal
    }

    /// One step of the beam priority, shared by `extend_words` and
    /// the search loop's lazy pre-filter so the gate can never drift
    /// from the priority it predicts (a drifted gate could skip
    /// admittable candidates). See `extend_words` for the rationale
    /// of each term.
    fn step_cheap(
        ipa_len: usize,
        rarity: Option<f64>,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
    ) -> f64 {
        let word_term = -0.35 + (ipa_len as f64).min(6.0) / 6.0 * 0.10;
        // Familiarity tie-break on a log-frequency scale, kept small
        // on purpose: the final clue score has no rarity term, so a
        // strong beam penalty would systematically bury valid
        // mid-frequency resegmentation words ("dupe"-class) that the
        // final scorer ranks highly. This only orders near-ties
        // toward familiar vocabulary.
        let rarity_penalty = match rarity {
            Some(r) if r > 5_000.0 => -(0.05 * (r / 5_000.0).log10()).min(0.15),
            _ => 0.0,
        };
        // Word-length, familiarity, resyllabification and reuse
        // terms, plus half the phonetic cost: the cost term leans
        // cells toward clean links (without it, high-cost junk with
        // long common words outranks genuine low-cost
        // resegmentations), while hard budgets, cheapest-first
        // arrival, cost-tier cells and epsilon-dominance keep
        // slightly-off close matches alive beside exact ones. A
        // heavier price would bury legitimate mid-cost links; a
        // lighter one lets junk flood the cells (and blow up memory
        // via unbounded downstream fan-out).
        word_term + rarity_penalty - 0.5 * word_sub_cost + boundary_bonus - reuse_penalty
    }

    /// Shared beam priority step, kept as the incremental mirror of
    /// the final clue score. Each extra word pays a segmentation
    /// penalty (net negative per word, so fewer/longer parses outrank
    /// over-segmented ones covering the same span — the old per-word
    /// length *bonus* did the opposite and let degenerate tiny-word
    /// paths dominate); longer words earn back a small fraction of
    /// it; phonetic edits pay half their cost (full weight would
    /// over-prune slightly-off close matches that the final scorer
    /// still ranks highly); rare words pay a small log-frequency
    /// penalty; `boundary_bonus` rewards word ends that resyllabify
    /// away from target boundaries (incremental novelty); and
    /// `reuse_penalty` charges clue words that recycle a target word
    /// (incremental word-novelty — without it, parrot paths like
    /// "its just ..." outrank genuine resyllabifications for the
    /// same span even though the final scorer demotes them).
    fn extend_words(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
    ) -> Self {
        let step = Self::step_cheap(
            ipa.chars().count(),
            rarity,
            word_sub_cost,
            boundary_bonus,
            reuse_penalty,
        );
        let mut grown = self.words.clone();
        grown.push(ClueWord {
            word: word.to_string(),
            ipa: ipa.to_string(),
            rarity,
            sub_cost: word_sub_cost,
        });
        let prefix_key = Self::prefix_key_for(&grown);
        Self {
            words: grown,
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            cheap_score: self.cheap_score + step,
            novelty: 0.5,
            raw_novelty: 0.5,
            familiarity: 0.0,
            last_cost: word_sub_cost,
            cuts: Vec::new(),
            key: Self::join_key(&self.key, word, ipa),
            prefix_key,
        }
    }

    /// The final clue score for a closed parse: novelty (reshuffled
    /// word boundaries), word-novelty (no recycled target words),
    /// word-length signal, and phonetic similarity. Computable only
    /// once the parse is complete, so the beam cannot prune on it
    /// mid-parse — but terminal retention can and does (see
    /// [`CompletionTop`]).
    fn final_score(&self, target_boundaries: &[usize], target_words: &HashSet<String>) -> f64 {
        // Target-aligned cuts when available (approximate mode with
        // indel history); otherwise reconstruct from clue IPA lens
        // (exact mode).
        let (cum, clue_boundaries): (usize, Vec<usize>) =
            if self.cuts.len() == self.words.len() && !self.words.is_empty() {
                let c = *self.cuts.last().unwrap();
                (c, self.cuts.clone())
            } else {
                let mut c = 0_usize;
                let mut b: Vec<usize> = Vec::with_capacity(self.words.len());
                for w in &self.words {
                    c += w.ipa.chars().count();
                    b.push(c);
                }
                (c, b)
            };

        // Boundary novelty as Jaccard distance over inner cuts:
        // rewards both removed and added boundaries, and is 0 only
        // for the same segmentation. The end-of-phrase boundary is
        // shared by construction, so exclude it (identical empty
        // inner sets score 0).
        let target_inner: HashSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|b| *b < cum)
            .collect();
        let clue_inner: HashSet<usize> = clue_boundaries
            .iter()
            .copied()
            .filter(|b| *b < cum)
            .collect();
        let shared = target_inner.intersection(&clue_inner).count() as f64;
        let union = target_inner.union(&clue_inner).count() as f64;
        let novelty = if union == 0.0 { 0.0 } else { 1.0 - (shared / union) };

        // Word-novelty: penalty if the clue reuses any target word
        // (normalized: "its" recycles "it's").
        let reused = self
            .words
            .iter()
            .filter(|w| target_words.contains(&Self::norm_word(&w.word)))
            .count() as f64;
        let word_novelty = 1.0 - (reused / self.words.len().max(1) as f64);

        // Word-length signal: prefer fewer/longer words, the
        // signature of a real Mad Gab clue.
        let avg_word_ipa_len = self
            .words
            .iter()
            .map(|w| w.ipa.chars().count())
            .sum::<usize>() as f64
            / self.words.len().max(1) as f64;
        let length_signal = (avg_word_ipa_len / 4.0).min(1.0);

        // Approximate-mode similarity: penalize total substitution
        // cost. In Exact mode sub_cost_total is 0, so similarity is
        // exactly 1.0 and this term is constant — the discrimination
        // remains on the novelty/length axes as before.
        let similarity = (1.0 - self.sub_cost_total / 4.0).clamp(0.0, 1.0);
        // Smooth lexical familiarity over chosen words' rarity
        // (normalized log-frequency average). Modest weight, taken
        // from the old average-length term (0.10 -> 0.05) so total
        // weight is unchanged; similarity stays the largest weight
        // and Jaccard novelty is retained.
        let familiarity = Self::avg_familiarity(&self.words);

        0.45 * similarity + 0.30 * novelty + 0.10 * word_novelty + 0.05 * length_signal
            + 0.10 * familiarity
    }

    fn into_clue(
        self,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
    ) -> Clue {
        let score = self.final_score(target_boundaries, target_words);
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

/// Diversity penalty weight for [`select_diverse`]: how much score
/// a candidate loses per unit of word/bigram overlap with its
/// closest already-picked clue. Large enough that near-identical
/// inflections of one resegmentation ("recognizes peach/pitch/
/// pits/...") collapse to their best representative, small enough
/// that genuinely better clues still outrank diverse-but-weaker
/// ones.
const MMR_LAMBDA: f64 = 0.25;

/// Diversity-aware final selection for Approximate mode (classic
/// maximal-marginal-relevance): pick the best clue, then
/// repeatedly the clue maximizing `score - LAMBDA * overlap`,
/// where overlap is the maximum (over already-picked clues) of
/// blended unigram+bigram word reuse. Twins of an early pick are
/// demoted while structurally different parses are judged only
/// against their nearest neighbor, so they surface; the single
/// best clue always ranks first. Deterministic; generic
/// word-overlap only, no lexical content. Input must be
/// score-sorted and phrase-deduped (as the caller provides).
fn select_diverse(clues: Vec<Clue>, top_n: usize) -> Vec<Clue> {
    if top_n == 0 || clues.is_empty() {
        return Vec::new();
    }
    // Precompute per-clue sorted lowercase word and bigram lists
    // once; pairwise overlap is a linear two-pointer walk.
    struct Bags {
        words: Vec<String>,
        bigrams: Vec<String>,
    }
    fn sorted(mut v: Vec<String>) -> Vec<String> {
        v.sort();
        v.dedup();
        v
    }
    let bags: Vec<Bags> = clues
        .iter()
        .map(|c| {
            let words: Vec<String> = c
                .phrase
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .collect();
            let bigrams: Vec<String> = words
                .windows(2)
                .map(|w| format!("{} {}", w[0], w[1]))
                .collect();
            Bags {
                words: sorted(words),
                bigrams: sorted(bigrams),
            }
        })
        .collect();
    /// Fraction of `a` also present in `b` (both sorted).
    fn frac(a: &[String], b: &[String], a_len: usize) -> f64 {
        if a_len == 0 {
            return 0.0;
        }
        let (mut i, mut j, mut hit) = (0usize, 0usize, 0usize);
        while i < a.len() && j < b.len() {
            match a[i].cmp(&b[j]) {
                std::cmp::Ordering::Equal => {
                    hit += 1;
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
            }
        }
        hit as f64 / a_len as f64
    }
    // Original (pre-dedup-count) lengths for denominators.
    let word_lens: Vec<usize> = clues
        .iter()
        .map(|c| c.phrase.split_whitespace().count())
        .collect();
    let bigram_lens: Vec<usize> = word_lens.iter().map(|&n| n.saturating_sub(1)).collect();
    let mut remaining: Vec<usize> = (0..clues.len()).collect();
    let mut picked: Vec<usize> = Vec::with_capacity(top_n.min(clues.len()));
    // Clue object moves out of `clues` at the end; work by index.
    while picked.len() < top_n && !remaining.is_empty() {
        let mut best_pos = 0;
        let mut best_val = f64::NEG_INFINITY;
        for (pos, &i) in remaining.iter().enumerate() {
            // Closest-picked overlap (classic MMR): unlike a running
            // union (whose penalties saturate as picked vocabulary
            // grows and bury everything late), every comparison
            // stays local, so a structurally different parse is
            // judged only against its nearest neighbor.
            let mut overlap = 0.0;
            for &j in &picked {
                let uw = frac(&bags[i].words, &bags[j].words, word_lens[i]);
                let bw = frac(&bags[i].bigrams, &bags[j].bigrams, bigram_lens[i]);
                let o = 0.5 * uw + 0.5 * bw;
                if o > overlap {
                    overlap = o;
                }
                if overlap >= 1.0 {
                    break;
                }
            }
            let v = clues[i].score - MMR_LAMBDA * overlap;
            if v > best_val {
                best_val = v;
                best_pos = pos;
            }
        }
        picked.push(remaining.remove(best_pos));
    }
    // Preserve pick order (MMR rank), not score order.
    let mut slots: Vec<Option<Clue>> = clues.into_iter().map(Some).collect();
    let mut out = Vec::with_capacity(picked.len());
    for i in picked {
        out.push(slots[i].take().expect("each index picked once"));
    }
    out
}

/// Insert `candidate` into a top-K beam, keeping the K highest-cheap-
/// score entries. Stable enough for our purposes.
fn insert_top_k(beam: &mut Vec<Partial>, candidate: Partial, k: usize) {
    if beam.len() < k {
        beam.push(candidate);
        return;
    }
    // Find the weakest entry; replace if the candidate is stronger.
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

/// Terminal shortlist ranked by final score: a min-heap of live
/// score bits plus a key→entry table for clone suppression (one
/// slot per distinct word+IPA parse). Per-axis reserves (see
/// [`AxisReserve`]) shelter objective champions below the score
/// floor; the drain merges everything into one pool.
/// Reserve seats for axis champions in the final pick: one each
/// for boundary novelty, lexical familiarity, and phonetic similarity.
/// Small and generic; guarantees Pareto-style representation in the
/// returned clues even when the composite score floor would bury a
/// resegmentation the beam worked to complete.
const FINAL_QUOTA: usize = 3;

/// Small per-axis reserve inside terminal retention: the top few
/// hundred completions by one objective survive regardless of
/// composite score, so a high-novelty resegmentation below the score
/// floor still reaches final re-ranking. One beam traversal, one
/// merged pool at drain; bounded.
struct AxisReserve {
    cap: usize,
    /// Min-heap of `(stored_axis, seq)` → key, where `stored_axis`
    /// is ordered so HIGHER always means better (`to_bits` for
    /// keep-highest axes, bitwise-NOT bits for keep-lowest). The heap
    /// top is the live worst; stale entries (superseded clones) are
    /// skipped lazily.
    heap: std::collections::BinaryHeap<(std::cmp::Reverse<(u64, u64)>, String)>,
    seq: u64,
    /// Clone key → `(stored_axis, seq, score_bits, entry)`.
    map: HashMap<String, (u64, u64, u64, Partial)>,
    /// True for keep-lowest axes (stored bits are bitwise-NOT).
    flip: bool,
}

impl AxisReserve {
    fn new(cap: usize, flip: bool) -> Self {
        Self {
            cap,
            heap: std::collections::BinaryHeap::new(),
            seq: 0,
            map: HashMap::new(),
            flip,
        }
    }

    fn stored(&self, axis: f64) -> u64 {
        let bits = axis.to_bits();
        if self.flip { !bits } else { bits }
    }

    fn insert(&mut self, candidate: Partial, score: f64, axis: f64) {
        if self.cap == 0 {
            return;
        }
        let stored = self.stored(axis);
        let bits = score.to_bits();
        if let Some(slot) = self.map.get_mut(&candidate.key) {
            if stored > slot.0 {
                self.seq += 1;
                let key = slot.3.key.clone();
                *slot = (stored, self.seq, bits, candidate);
                self.heap.push((std::cmp::Reverse((stored, self.seq)), key));
            }
            return;
        }
        if self.map.len() >= self.cap {
            while let Some((std::cmp::Reverse((b, s)), k)) = self.heap.pop() {
                match self.map.get(&k) {
                    Some((ab, as_, _, _)) if *ab == b && *as_ == s => {
                        if stored <= b {
                            self.heap.push((std::cmp::Reverse((b, s)), k));
                            return;
                        }
                        self.map.remove(&k);
                        break;
                    }
                    _ => continue,
                }
            }
        }
        if self.map.len() < self.cap {
            self.seq += 1;
            self.heap
                .push((std::cmp::Reverse((stored, self.seq)), candidate.key.clone()));
            self.map.insert(
                candidate.key.clone(),
                (stored, self.seq, bits, candidate),
            );
        }
    }

    fn drain_into(self, out: &mut HashMap<String, (u64, u64, Partial)>) {
        for (key, (_, _, score_bits, partial)) in self.map {
            match out.get(&key) {
                Some((bits, _, _)) if *bits >= score_bits => {}
                _ => {
                    out.insert(key, (score_bits, 0, partial));
                }
            }
        }
    }

    /// Cheap admission peek (no clone): true when the axis value
    /// would survive. Drains stale heap tops as a side effect.
    fn would_admit(&mut self, axis: f64) -> bool {
        if self.cap == 0 {
            return false;
        }
        if self.map.len() < self.cap {
            return true;
        }
        let stored = self.stored(axis);
        while let Some((std::cmp::Reverse((b, s)), k)) = self.heap.pop() {
            match self.map.get(&k) {
                Some((ab, as_, _, _)) if *ab == b && *as_ == s => {
                    let admits = stored > b;
                    self.heap.push((std::cmp::Reverse((b, s)), k));
                    return admits;
                }
                _ => continue,
            }
        }
        true
    }
}

/// Stale heap entries (from superseded clones) are skipped lazily at
/// drain time. The main cap is generous (thousands) so mid-pack valid
/// parses survive to re-ranking; per-axis reserves shelter objective
/// champions below the score floor.
struct CompletionTop {
    cap: usize,
    /// Min-heap of `(score_bits, seq, key)`. Scores are in [0, 1],
    /// where `to_bits` preserves numeric order; `seq`
    /// disambiguates ties so every push is unique. Entries whose
    /// map slot no longer matches are stale (clone superseded or
    /// evicted) and skipped on pop.
    heap: std::collections::BinaryHeap<(std::cmp::Reverse<(u64, u64)>, String)>,
    seq: u64,
    /// Clone key → `(score_bits, seq, entry)`.
    map: HashMap<String, (u64, u64, Partial)>,
    /// Per-axis reserves (raw novelty, familiarity, sub cost),
    /// merged into one pool at drain.
    reserves: [AxisReserve; 3],
}

impl CompletionTop {
    fn new(cap: usize) -> Self {
        Self {
            cap,
            heap: std::collections::BinaryHeap::new(),
            seq: 0,
            map: HashMap::new(),
            // Small bounded reserves per objective axis: raw boundary
            // novelty and familiarity keep the highest, phonetic cost
            // keeps the lowest (flip). Sized to comfortably shelter
            // below-floor champions (a merged pool near ten thousand
            // holds thousands of near-tie resegmentation variants, so
            // a few hundred slots cannot cover an axis tail).
            reserves: [
                AxisReserve::new(2048, false),
                AxisReserve::new(2048, false),
                AxisReserve::new(2048, true),
            ],
        }
    }

    fn insert(&mut self, candidate: Partial, score: f64) {
        // Axis values ride along before any move: raw resegmentation
        // novelty, lexical familiarity, accumulated phonetic cost.
        let axes = [
            candidate.raw_novelty,
            candidate.familiarity,
            candidate.sub_cost_total,
        ];
        if self.main_would_admit(&candidate.key, score) {
            self.main_insert(candidate, score);
            return;
        }
        // Below the score floor: shelter axis champions in the
        // reserves (clone only on admission).
        for (reserve, axis) in self.reserves.iter_mut().zip(axes) {
            if reserve.would_admit(axis) {
                reserve.insert(candidate.clone(), score, axis);
            }
        }
    }

    /// Cheap main-pool admission peek (no clone). Clone improvements
    /// always pass; new keys pass while room lasts or past the live
    /// floor.
    fn main_would_admit(&mut self, key: &str, score: f64) -> bool {
        if self.cap == 0 {
            return false;
        }
        if self.map.contains_key(key) {
            return true;
        }
        if self.map.len() < self.cap {
            return true;
        }
        let bits = score.to_bits();
        while let Some((std::cmp::Reverse((b, s)), k)) = self.heap.pop() {
            match self.map.get(&k) {
                Some((lb, ls, _)) if *lb == b && *ls == s => {
                    let admits = bits > b;
                    self.heap.push((std::cmp::Reverse((b, s)), k));
                    return admits;
                }
                _ => continue,
            }
        }
        true
    }

    /// Infallible main-pool insert (admission pre-checked by
    /// `main_would_admit`).
    fn main_insert(&mut self, candidate: Partial, score: f64) {
        if self.cap == 0 {
            return;
        }
        // Scores are finite (bounded arithmetic in final_score).
        let bits = score.to_bits();
        if self.cap == 0 {
            return;
        }
        // Scores are finite (bounded arithmetic in final_score).
        let bits = score.to_bits();
        if let Some(slot) = self.map.get_mut(&candidate.key) {
            if bits > slot.0 {
                self.seq += 1;
                let key = slot.2.key.clone();
                *slot = (bits, self.seq, candidate);
                self.heap.push((std::cmp::Reverse((bits, self.seq)), key));
            }
            return;
        }
        if self.map.len() >= self.cap {
            // Evict the live minimum to make room, if the newcomer
            // beats it.
            while let Some((std::cmp::Reverse((b, s)), k)) = self.heap.pop() {
                match self.map.get(&k) {
                    Some((lb, ls, _)) if *lb == b && *ls == s => {
                        if bits <= b {
                            // Newcomer is no better; restore evicted.
                            self.heap.push((std::cmp::Reverse((b, s)), k));
                            return;
                        }
                        self.map.remove(&k);
                        break;
                    }
                    // Stale heap entry; keep draining.
                    _ => continue,
                }
            }
        }
        if self.map.len() < self.cap {
            self.seq += 1;
            self.heap
                .push((std::cmp::Reverse((bits, self.seq)), candidate.key.clone()));
            self.map
                .insert(candidate.key.clone(), (bits, self.seq, candidate));
        }
    }

    /// Merged drain: main score pool plus all axis reserves,
    /// deduped by clone key keeping the best score, then sorted.
    /// One merged completion pool; bounded.
    fn drain_merged(mut self) -> Vec<Partial> {
        for reserve in self.reserves {
            reserve.drain_into(&mut self.map);
        }
        let mut v: Vec<(u64, u64, Partial)> = self.map.drain().map(|(_, v)| v).collect();
        v.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        v.into_iter().map(|(_, _, p)| p).collect()
    }

    fn drain_sorted(self) -> Vec<Partial> {
        self.drain_merged()
    }

    /// TEMPORARY: word-sequence presence probe over a key→entry
    /// table. Generic; env-hook only.
    fn pool_has_trace(
        map: &HashMap<String, (u64, u64, Partial)>,
        trace: &[String],
    ) -> bool {
        !trace.is_empty()
            && map.values().any(|(_, _, p)| {
                p.words.len() == trace.len()
                    && p.words
                        .iter()
                        .zip(trace.iter())
                        .all(|(w, t)| w.word.to_lowercase() == *t)
            })
    }

    /// TEMPORARY traced drain: same as merged drain plus a one-line
    /// pool summary (size, score floor, whether the env-traced word
    /// sequence survived). Generic; env-hook only.
    fn drain_sorted_traced(
        self,
        trace: &[String],
        tref: Option<(f64, f64, f64, f64)>,
    ) -> Vec<Partial> {
        let v = self.drain_merged_traced(trace, tref);
        v
    }

    fn drain_merged_traced(
        mut self,
        trace: &[String],
        tref: Option<(f64, f64, f64, f64)>,
    ) -> Vec<Partial> {
        // TEMPORARY: per-pool traced presence + reserve live floors,
        // before merging. Generic; env-hook only.
        let in_main = Self::pool_has_trace(&self.map, trace);
        let mut reserve_hits = Vec::new();
        let mut reserve_floors = Vec::new();
        for r in &mut self.reserves {
            let mut floor = f64::INFINITY;
            for (stored, _, _, _) in r.map.values() {
                let axis = if r.flip {
                    f64::from_bits(!stored)
                } else {
                    f64::from_bits(*stored)
                };
                if axis < floor {
                    floor = axis;
                }
            }
            reserve_floors.push(floor);
            reserve_hits.push(r.map.values().any(|(_, _, _, p)| {
                !trace.is_empty()
                    && p.words.len() == trace.len()
                    && p.words
                        .iter()
                        .zip(trace.iter())
                        .all(|(w, t)| w.word.to_lowercase() == *t)
            }));
        }
        eprintln!(
            "TRACE-POOLS main={in_main} reserves={reserve_hits:?} reserve_floors={reserve_floors:.4?} sizes=[{}, {}, {}, {}]",
            self.map.len(),
            self.reserves[0].map.len(),
            self.reserves[1].map.len(),
            self.reserves[2].map.len(),
        );
        for reserve in self.reserves {
            reserve.drain_into(&mut self.map);
        }
        let mut floor = f64::INFINITY;
        let mut traced = false;
        for (bits, _, _) in self.map.values() {
            let s = f64::from_bits(*bits);
            if s < floor {
                floor = s;
            }
        }
        if !trace.is_empty() {
            for (_, _, p) in self.map.values() {
                if p.words.len() == trace.len()
                    && p.words
                        .iter()
                        .zip(trace.iter())
                        .all(|(w, t)| w.word.to_lowercase() == *t)
                {
                    traced = true;
                    break;
                }
            }
        }
        // Joint-distribution diagnostic vs the traced completion's own
        // (score, raw_nov, fam, cost): how many pool entries beat it
        // per axis and on all four (dominators).
        let mut above_score = 0usize;
        let mut above_nov = 0usize;
        let mut above_fam = 0usize;
        let mut below_cost = 0usize;
        let mut dominators = 0usize;
        if let Some((ts, tn, tf, tc)) = tref {
            for (bits, _, p) in self.map.values() {
                let s = f64::from_bits(*bits);
                let bs = s > ts;
                let bn = p.raw_novelty > tn;
                let bf = p.familiarity > tf;
                let bc = p.sub_cost_total < tc - 1e-9;
                if bs {
                    above_score += 1;
                }
                if bn {
                    above_nov += 1;
                }
                if bf {
                    above_fam += 1;
                }
                if bc {
                    below_cost += 1;
                }
                if bs && bn && bf && bc {
                    dominators += 1;
                }
            }
            eprintln!(
                "TRACE-DIST ref=({ts:.4},{tn:.3},{tf:.3},{tc:.3}) above_score={above_score} above_nov={above_nov} above_fam={above_fam} below_cost={below_cost} dominators={dominators}",
            );
        }
        eprintln!(
            "TRACE-POOL size={} floor={floor:.4} traced_completed={}",
            self.map.len(),
            tref.is_some(),
        );
        let mut v: Vec<(u64, u64, Partial)> = self.map.drain().map(|(_, v)| v).collect();
        v.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        v.into_iter().map(|(_, _, p)| p).collect()
    }
}

/// Acoustic-cost tier of an accumulated substitution cost. Tier 0
/// is exactly clean (no phonetic edits — typically parrot or exact
/// parses); tier 1 is a slight deviation (about one weak edit —
/// a single close substitution or weak indel); tier 2 compounds
/// edits. The tier-1/tier-2 cut keeps slightly-off genuine
/// resegmentations alive beside zero-cost competitors covering the
/// same span: a parse that paid for an extra weak insertion plus a
/// substitution must not compete head-to-head with near-exact
/// parses for the same cell, since the final scorer still ranks
/// such parses highly and only it can arbitrate.
fn cost_tier(sub_cost_total: f64) -> u8 {
    if sub_cost_total <= 1e-9 {
        0
    } else if sub_cost_total < 0.5 {
        1
    } else {
        2
    }
}

/// One beam position with incremental statistics. A flat
/// `Vec<Partial>` plus linear-scan insert costs O(beam) per pair;
/// with roomy beams (512+) and wide match fan-in that is
/// quadratic and interactive use dies. `BeamPos` keeps per-cell
/// counts/minima incrementally, so the hot pair loop pre-filters
/// through [`BeamPos::would_admit`] (a few flops and hash lookups,
/// no allocation) and only clones a `Partial` on admission.
/// Entry indices are stable (pushed or replaced in place, never
/// removed), so cached indices stay valid. Minima are refreshed by
/// rescan only when the minimum itself is evicted.
/// Near-tie epsilon for beam admission: candidates within this of
/// a cell's (or band's) worst kept score are plausible hypotheses
/// the beam cannot resolve, so it keeps them all and lets the final
/// scorer decide. Covers the observed sub-0.15 gaps between genuine
/// resegmentations and near-exact rivals sharing a cell.
const EPSILON: f64 = 0.20;

/// Hard capacity multiplier over the soft shares: near-tie
/// extension never grows a cell past this, bounding per-position
/// memory even when a cluster is wide. Sized generously because
/// real near-tie clusters run into the hundreds on a full-size
/// corpus (dozens of resegmentation variants times pronunciation
/// variants of the same words): truncating them would evict genuine
/// mid-pack parses that the final scorer ranks highly, and only the
/// hard ceiling — not quality order — may stop an undominated
/// candidate.
const HARD_MULT: usize = 8;

/// Reservoir quota per objective axis inside a full cell: the top-Q
/// members by each axis are protected from eviction. Small (single
/// digits): valley prefixes that paid cost early typically sit just
/// outside the very top on the scalar while tied on novelty, so the
/// quota needs room for near-ties, not just winners; the hard cap
/// still bounds the cell.
const RES_Q: usize = 6;

/// Prefix-family protection depth: top-FAM_KEEP per prefix stay
/// immune; members below that in big families are ordinary
/// displacement victims. Small families (at most FAM_KEEP) keep
/// everything, so rare prefixes are never trimmed. Deliberately
/// narrower than the RES_Q admission cap: admission bounds family
/// SIZE for diversity, protection bounds IMMUNITY against churn.
/// A valley prefix in a small family is therefore immune everywhere
/// except saturated-margin displacement, which independently spares
/// small families (see below).
const FAM_KEEP: usize = 3;

/// Merit margin for saturated-cell displacement: twins within a
/// thousandth coexist (incumbency wins ties — a strictly-better
/// newcomer by less still bounces off a saturated cell), while
/// clear wins re-enter contention. Small and generic — it only
/// arbitrates the saturated-cell fallback, never roomy growth or
/// unprotected victims (junk falls to any strict improvement).
/// Sized an order above twin noise with headroom below the
/// smallest genuine valley gaps either side.
const MERIT_MARGIN: f64 = 0.005;

/// Novelty slack for first-of-ending admission: a candidate whose
/// last-word sound is absent from its cell joins (displacing the
/// cell worst) if it is within this of the kept worst. Endings are
/// the perceptually salient part of a clue, so every distinct
/// ending deserves a foothold; the slack keeps out true junk while
/// admitting legitimate resegmentation endings that lose a close
/// scalar fight. Wider than EPSILON on purpose (novelty is rarer
/// than near-ties).
const ENDING_EPS: f64 = 0.50;

struct BeamPos {
    entries: Vec<Partial>,
    /// Clone key -> (index, cheap). Clone keys are per-step
    /// `(word, IPA)` sequences; exact duplicate records share one
    /// slot (best cheap wins) while homophones stay distinct.
    keys: HashMap<String, (usize, f64)>,
    /// (word count, cost tier) -> member indices. Each cell is an
    /// independent quota: word counts and cost tiers can never steal
    /// each other's slots, so deep or slightly-off parses always
    /// keep representation no matter how crowded other cells get.
    cell_members: HashMap<(usize, u8), Vec<usize>>,
    cell_min: HashMap<(usize, u8), (usize, f64)>,
    /// Per-cell best cheap (index, value). Anchors the settled-cell
    /// epsilon window: past the hard cap the beam arbitrates against
    /// its best, not its worst. Maintained exactly like `cell_min`
    /// (a stale low max would wrongly refuse admittable candidates).
    cell_max: HashMap<(usize, u8), (usize, f64)>,
    /// ((word count, cost tier), last-word IPA) -> member count.
    /// Powers ending-novelty admission: the first prefix with a
    /// given ending sound always finds a foothold in its cell.
    ending_counts: HashMap<((usize, u8), String), usize>,
    k: usize,
    cell_cap: usize,
    cell_hard: usize,
}

impl BeamPos {
    fn new(k: usize) -> Self {
        Self {
            entries: Vec::new(),
            keys: HashMap::new(),
            cell_members: HashMap::new(),
            cell_min: HashMap::new(),
            cell_max: HashMap::new(),
            ending_counts: HashMap::new(),
            k,
            cell_cap: (k / 8).max(8),
            cell_hard: (k / 8).max(8) * HARD_MULT,
        }
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drain entries for expansion; resets all stats (each position
    /// expands exactly once).
    fn take_entries(&mut self) -> Vec<Partial> {
        self.keys.clear();
        self.cell_members.clear();
        self.cell_min.clear();
        self.cell_max.clear();
        self.ending_counts.clear();
        std::mem::take(&mut self.entries)
    }

    /// Beam cell key: word count (segmentation depth) by
    /// acoustic-cost tier (clean / near / far parses never evict
    /// each other).
    fn cell_of(cand_words: usize, cand_sub: f64) -> (usize, u8) {
        (cand_words, cost_tier(cand_sub))
    }

    /// Lazy pre-filter for a candidate that has not been built yet.
    /// Conservative: returns true whenever admission is possible, so
    /// [`BeamPos::insert`] re-verifies. Key assembly (one small
    /// allocation) happens only after the arithmetic gates pass.
    /// Mirrors [`BeamPos::insert`] exactly.
    #[allow(clippy::too_many_arguments)]
    fn would_admit(
        &self,
        partial_key: &str,
        partial_key_empty: bool,
        match_word: &str,
        match_ipa: &str,
        cand_words: usize,
        cand_sub: f64,
        cand_cheap: f64,
        cand_nov: f64,
        cand_fam: f64,
        cand_last: f64,
    ) -> bool {
        if self.k == 0 {
            return false;
        }
        let cell = Self::cell_of(cand_words, cand_sub);
        let cell_count = self.cell_members.get(&cell).map_or(0, Vec::len);
        if cell_count >= self.cell_cap
            && !self.cell_admits(cell, cell_count, cand_cheap, cand_sub, cand_nov, cand_fam, cand_last)
            // Last resort (mirrors insert): a first-of-ending sound
            // joins if within ENDING_EPS of the kept worst.
            // PicoWord endings stay under pico policy below.
            && (match_ipa.chars().count() < Self::PICO_LEN
                || !self.novel_ending(cell, match_ipa, cand_cheap))
        {
            // PicoWord candidates may still pass as clone
            // improvements (checked below): mirror insert(), which
            // checks clones before cell policy. Substantial
            // candidates are refused here exactly as insert()
            // refuses them.
            let pico = match_ipa.chars().count() < Self::PICO_LEN;
            if !pico {
                return false;
            }
            let takes_slot = self.cell_members.get(&cell).is_some_and(|ms| {
                ms.iter().any(|&i| {
                    let p = &self.entries[i];
                    Self::last_ipa_len(p) < Self::PICO_LEN && cand_cheap > p.cheap_score
                })
            });
            if !takes_slot {
                let key = Partial::join_key(
                    &if partial_key_empty {
                        String::new()
                    } else {
                        partial_key.to_string()
                    },
                    match_word,
                    match_ipa,
                );
                return match self.keys.get(&key) {
                    Some(&(_, cheap)) => cand_cheap > cheap,
                    None => false,
                };
            }
        }
        let key = Partial::join_key(
            &if partial_key_empty {
                String::new()
            } else {
                partial_key.to_string()
            },
            match_word,
            match_ipa,
        );
        match self.keys.get(&key) {
            Some(&(_, cheap)) => cand_cheap > cheap,
            None => true,
        }
    }

    /// Cell admission shared by the gate and [`BeamPos::insert`]:
    /// below the soft share everything joins; past it, a candidate
    /// joins (up to the hard cap) unless the whole kept cluster
    /// beats it by more than EPSILON — near-tie hypotheses the
    /// beam cannot resolve are all kept, and only uniformly worse
    /// candidates are refused, so low-cost resegmentation prefixes
    /// can never be squeezed out by one scalar. Past the hard cap,
    /// a candidate gets in when it sits inside the epsilon window
    /// below the cell best (near-best hypotheses), enters a
    /// protected reservoir set, strictly improves, or displaces a
    /// member it epsilon-dominates.
    fn cell_admits(
        &self,
        cell: (usize, u8),
        cell_count: usize,
        cand_cheap: f64,
        cand_sub: f64,
        cand_nov: f64,
        cand_fam: f64,
        cand_last: f64,
    ) -> bool {
        if cell_count < self.cell_cap {
            return true;
        }
        // Reservoir axis: a candidate that would enter any protected
        // top-Q set is always admittable (up to the hard cap; past it
        // `insert` evicts an unprotected member).
        if self.would_enter_protected(cell, cand_cheap, cand_nov, cand_fam, cand_sub, cand_last) {
            return true;
        }
        if cell_count < self.cell_hard {
            return match self.cell_min.get(&cell) {
                Some(&(_, min)) => cand_cheap > min - EPSILON,
                // Inconsistent (should not happen); admit.
                None => true,
            };
        }
        // Settled cell: inside the epsilon window below the best, a
        // strict heuristic improvement, or an epsilon-dominating
        // displacement. The window is max-anchored on purpose: past
        // the hard cap the beam must arbitrate against its best, not
        // its worst — otherwise a low floor admits unbounded junk
        // while genuine near-best prefixes wait behind champions.
        if let Some(&(_, max)) = self.cell_max.get(&cell) {
            if cand_cheap > max - EPSILON {
                return true;
            }
        }
        // Settled cell: strict heuristic improvement displaces the
        // worst, or the candidate displaces a member it
        // epsilon-dominates (acoustically redundant beside it).
        if let Some(&(_, min)) = self.cell_min.get(&cell) {
            if cand_cheap > min {
                return true;
            }
        }
        self.dominates_some(cell, cand_cheap, cand_sub).is_some()
    }

    /// True when the candidate sits inside the settled-cell epsilon
    /// window (within EPSILON of the cell best). Shared by the gate
    /// and [`BeamPos::insert`].
    fn in_window(&self, cell: (usize, u8), cand_cheap: f64) -> bool {
        match self.cell_max.get(&cell) {
            Some(&(_, max)) => cand_cheap > max - EPSILON,
            // Inconsistent (should not happen); admit.
            None => true,
        }
    }

    /// First-of-ending novelty shared by the gate and
    /// [`BeamPos::insert`]: true when no kept member shares the
    /// candidate's last-word sound and the candidate is within
    /// ENDING_EPS of the kept worst. The minimum check comes first
    /// so the common refusal path costs no allocation.
    fn novel_ending(&self, cell: (usize, u8), end_ipa: &str, cand_cheap: f64) -> bool {
        let min = match self.cell_min.get(&cell) {
            Some(&(_, min)) => min,
            // Inconsistent (should not happen); admit.
            None => return true,
        };
        if cand_cheap <= min - ENDING_EPS {
            return false;
        }
        self.ending_counts
            .get(&(cell, end_ipa.to_string()))
            .copied()
            .unwrap_or(0)
            == 0
    }

    /// Per-axis strictly-better tallies of the candidate against a
    /// cell's kept members: [cheap, raw novelty, familiarity,
    /// accumulated cost (lower better), last-link cost]. Returns
    /// `None` when the cell holds fewer than RES_Q members (every
    /// candidate trivially contends there). Shared by the gate
    /// champion test, the insert band edition, and the strictly-best
    /// rule below.
    #[allow(clippy::too_many_arguments)]
    fn axis_strict_counts(
        &self,
        cell: (usize, u8),
        cand_cheap: f64,
        cand_raw: f64,
        cand_fam: f64,
        cand_sub: f64,
        cand_last: f64,
    ) -> Option<[usize; 5]> {
        let members = self.cell_members.get(&cell)?;
        if members.len() < RES_Q {
            return None;
        }
        let mut better = [0usize; 5];
        for &i in members {
            let p = &self.entries[i];
            if p.cheap_score > cand_cheap {
                better[0] += 1;
            }
            if p.raw_novelty > cand_raw {
                better[1] += 1;
            }
            if p.familiarity > cand_fam {
                better[2] += 1;
            }
            if p.sub_cost_total < cand_sub - 1e-9 {
                better[3] += 1;
            }
            if p.last_cost > cand_last + 1e-9 {
                better[4] += 1;
            }
        }
        Some(better)
    }

    /// True when the candidate strictly exceeds all kept members on
    /// at least one reservoir axis (zero kept members at-or-better).
    /// Such an arrival is an objective extreme — the only newcomer
    /// that may displace the cell worst even in a saturated cell —
    /// because extremes are rare (they must beat every kept member
    /// outright, ties excluded) and trading the worst scalar for a
    /// new extreme improves diversity and quality at once. This is
    /// what saves valley prefixes whose single distinguishing trait
    /// is being the cheapest, most novel, most familiar, or most
    /// committed link around. Note the strictness: merely tying the
    /// best does not qualify (tie crowds are handled by the banded
    /// champion rule instead).
    #[allow(clippy::too_many_arguments)]
    fn is_strictly_best(
        &self,
        cell: (usize, u8),
        cand_cheap: f64,
        cand_raw: f64,
        cand_fam: f64,
        cand_sub: f64,
        cand_last: f64,
    ) -> bool {
        let Some(members) = self.cell_members.get(&cell) else {
            return false;
        };
        if members.len() < RES_Q {
            return false;
        }
        // Per-axis: true when no member is at-or-better.
        let mut dominated = [false; 5];
        for &i in members {
            let p = &self.entries[i];
            if p.cheap_score >= cand_cheap {
                dominated[0] = true;
            }
            if p.raw_novelty >= cand_raw {
                dominated[1] = true;
            }
            if p.familiarity >= cand_fam {
                dominated[2] = true;
            }
            if p.sub_cost_total <= cand_sub + 1e-9 {
                dominated[3] = true;
            }
            if p.last_cost >= cand_last - 1e-9 {
                dominated[4] = true;
            }
            if dominated.iter().all(|&d| d) {
                return false;
            }
        }
        true
    }

    /// True when the candidate would rank in the top-Q of any
    /// reservoir axis (cheap, raw novelty, familiarity, lowest
    /// accumulated cost, highest last-link cost) of the cell: fewer
    /// than Q kept members are strictly better on that axis. Strict
    /// comparison is deliberate — ties always enter, so a valley
    /// prefix tied with the kept cluster on novelty still earns its
    /// slot, and displacement (one in, one out, cheapest unprotected
    /// first) keeps the best of each tie crowd without growing the
    /// cell. This is the GATE version (over-admitting by design);
    /// [`BeamPos::insert`] re-verifies full cells with the bounded
    /// tie-band edition below. Cheap scan over bounded cell members.
    fn would_enter_protected(
        &self,
        cell: (usize, u8),
        cand_cheap: f64,
        cand_raw: f64,
        cand_fam: f64,
        cand_sub: f64,
        cand_last: f64,
    ) -> bool {
        match self.axis_strict_counts(cell, cand_cheap, cand_raw, cand_fam, cand_sub, cand_last) {
            None => true,
            Some(better) => better.iter().any(|&b| b < RES_Q),
        }
    }

    /// Insert-side edition of the champion test: like
    /// [`BeamPos::would_enter_protected`], but ties join only a
    /// bounded band (fewer than 2Q at-or-better per axis). Unbounded
    /// tie crowds (dozens of raw-1.0 early prefixes) must not churn
    /// protected champions out of full cells; the band admits a
    /// valley prefix tied at a high absolute value with few members
    /// above while refusing degenerate tie floods.
    fn would_enter_banded(
        &self,
        cell: (usize, u8),
        cand_cheap: f64,
        cand_raw: f64,
        cand_fam: f64,
        cand_sub: f64,
        cand_last: f64,
    ) -> bool {
        let Some(members) = self.cell_members.get(&cell) else {
            return true;
        };
        if members.len() < RES_Q {
            return true;
        }
        let mut strict = [0usize; 5];
        let mut weak = [0usize; 5];
        for &i in members {
            let p = &self.entries[i];
            if p.cheap_score > cand_cheap {
                strict[0] += 1;
            }
            if p.cheap_score >= cand_cheap {
                weak[0] += 1;
            }
            if p.raw_novelty > cand_raw {
                strict[1] += 1;
            }
            if p.raw_novelty >= cand_raw {
                weak[1] += 1;
            }
            if p.familiarity > cand_fam {
                strict[2] += 1;
            }
            if p.familiarity >= cand_fam {
                weak[2] += 1;
            }
            if p.sub_cost_total < cand_sub - 1e-9 {
                strict[3] += 1;
            }
            if p.sub_cost_total <= cand_sub + 1e-9 {
                weak[3] += 1;
            }
            if p.last_cost > cand_last + 1e-9 {
                strict[4] += 1;
            }
            if p.last_cost >= cand_last - 1e-9 {
                weak[4] += 1;
            }
        }
        strict
            .iter()
            .zip(weak.iter())
            .any(|(&s, &w)| s < RES_Q && w < 2 * RES_Q)
    }

    /// Protected indices of a cell: union over axes of the top-Q
    /// plus the bounded tie band (a member is protected when fewer
    /// than Q are strictly better AND fewer than 2Q are at-or-better
    /// on that axis), plus the prefix-family quota — but only for
    /// members inside the epsilon window below the cell best. This
    /// matches [`BeamPos::would_enter_banded`]: an admitted candidate
    /// that earned its slot keeps it; degenerate tie crowds stay
    /// unprotected and turn over cheapest-first. The window gate is
    /// what keeps victims available: members the beam already
    /// resolved against (below the band) are always displaceable, so
    /// full cells concentrate instead of freezing, and floor-sitting
    /// valleys are never stranded without a victim to displace.
    /// Bounded (under 2Q per axis plus families).
    fn protected_set(&self, cell: (usize, u8)) -> HashSet<usize> {
        const AXES: [(bool, fn(&Partial) -> f64); 5] = [
            (true, Partial::cheap_proj),
            (true, Partial::raw_novelty_proj),
            (true, Partial::familiarity_proj),
            (false, Partial::sub_cost_proj),
            (true, Partial::last_cost_proj),
        ];
        let Some(members) = self.cell_members.get(&cell) else {
            return HashSet::new();
        };
        let mut out: HashSet<usize> = self.family_protected(members);
        out.extend(self.band_protected(members, &AXES));
        // Window gate: immunity requires standing inside the
        // competitive band. Below-window members were resolved
        // against — they stay displaceable no matter their axis or
        // family standing (this is also what guarantees victims
        // exist without any global protection budget).
        let floor = self
            .cell_max
            .get(&cell)
            .map_or(f64::NEG_INFINITY, |(_, m)| *m - EPSILON);
        out.retain(|&i| self.entries[i].cheap_score > floor);
        out
    }

    /// Prefix-family quota: within a cell, members sharing a
    /// normalized word prefix (all words but the last; only for
    /// parses of three or more words) differ only in the last word —
    /// same cuts, same novelty — so the beam can resolve them against
    /// each other at admission time. Admission caps families at
    /// RES_Q; protection covers the whole kept family, so one
    /// prolific prefix can never hog the cell while small families
    /// (a rare resegmentation prefix with few continuations) keep
    /// everything. Bounded. Generic word identity only, no lexical
    /// content.
    fn family_protected(&self, members: &[usize]) -> HashSet<usize> {
        let mut out: HashSet<usize> = HashSet::new();
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for &i in members {
            let key = self.entries[i].prefix_key.as_str();
            if key.is_empty() {
                continue;
            }
            groups.entry(key).or_default().push(i);
        }
        for (_, mut idxs) in groups {
            idxs.sort_by(|&a, &b| {
                self.entries[b]
                    .cheap_score
                    .partial_cmp(&self.entries[a].cheap_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for &i in idxs.iter().take(FAM_KEEP) {
                out.insert(i);
            }
        }
        out
    }

    /// Number of cell members sharing a prefix key (empty keys count
    /// nothing — short parses are exempt from family quotas).
    fn family_count(&self, cell: (usize, u8), prefix: &str) -> usize {
        if prefix.is_empty() {
            return 0;
        }
        self.cell_members.get(&cell).map_or(0, |members| {
            members
                .iter()
                .filter(|&&i| self.entries[i].prefix_key == prefix)
                .count()
        })
    }

    /// Band-protection core shared by the full and non-cheap
    /// protection sets below: union of top-Q plus bounded tie bands
    /// over the given axes.
    fn band_protected(
        &self,
        members: &[usize],
        axes: &[(bool, fn(&Partial) -> f64)],
    ) -> HashSet<usize> {
        let mut out: HashSet<usize> = HashSet::new();
        for &(higher_better, proj) in axes {
            let mut ranked: Vec<(f64, usize)> =
                members.iter().map(|&i| (proj(&self.entries[i]), i)).collect();
            if higher_better {
                ranked.sort_by(|a, b| {
                    b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
                });
            } else {
                ranked.sort_by(|a, b| {
                    a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            let mut g0 = 0usize;
            while g0 < ranked.len() {
                let mut g1 = g0 + 1;
                while g1 < ranked.len() && ranked[g1].0 == ranked[g0].0 {
                    g1 += 1;
                }
                if g0 < RES_Q && g1 < 2 * RES_Q {
                    for (_, idx) in &ranked[g0..g1] {
                        out.insert(*idx);
                    }
                }
                g0 = g1;
            }
        }
        out
    }

    /// Prefix-family gate: true when the candidate's prefix family
    /// already holds RES_Q members in the cell. A saturated family
    /// refuses newcomers outright — within-family cheap gaps are
    /// sub-epsilon noise (same prefix, cuts and novelty; only the
    /// last word's acoustics differ), which the beam cannot resolve,
    /// so first-come stands and upgrades arrive through the
    /// clone-improvement path. Small families always have room.
    /// Generic word identity only, no lexical content.
    fn family_is_full(&self, cell: (usize, u8), candidate: &Partial) -> bool {
        if candidate.prefix_key.is_empty() {
            return false;
        }
        let Some(members) = self.cell_members.get(&cell) else {
            return false;
        };
        let mut count = 0usize;
        for &i in members {
            if self.entries[i].prefix_key == candidate.prefix_key {
                count += 1;
                if count >= RES_Q {
                    return true;
                }
            }
        }
        false
    }

    /// TEMPORARY prefix-match predicate shared by the env-hook
    /// traces (generic; no literal words).
    fn trace_matches(old: &Partial) -> bool {
        use std::sync::OnceLock;
        static T: OnceLock<Vec<String>> = OnceLock::new();
        let t = T.get_or_init(|| {
            std::env::var("MADGAB_TRACE_EVICT").map_or(Vec::new(), |s| {
                s.split_whitespace().map(|w| w.to_lowercase()).collect()
            })
        });
        if t.is_empty() || old.words.is_empty() {
            return false;
        }
        let mut j = 0usize;
        while j < t.len()
            && j < old.words.len()
            && old.words[j].word.to_lowercase() == t[j]
        {
            j += 1;
        }
        j > 0 && j == old.words.len()
    }

    /// TEMPORARY eviction trace helper (generic; env-hook only).
    /// The caller pre-filters with `trace_matches` and passes
    /// forensics (family size, protection state, coverage).
    #[allow(clippy::too_many_arguments)]
    fn trace_evicted(
        old: &Partial,
        new: &Partial,
        why: &str,
        fam_size: usize,
        is_prot: bool,
        coverage: usize,
    ) {
        // `j` doubles as the matched length (the caller guarantees a
        // full-prefix match).
        let j = old.words.len();
        eprintln!(
            "TRACE-EVICT len={j} why={why} old_cheap={:.3} old_sub={:.3} new_cheap={:.3} famsize={fam_size} isprot={is_prot} coverage={coverage} old={:?} new={:?}",
            old.cheap_score,
            old.sub_cost_total,
            new.cheap_score,
            old.words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>(),
            new.words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>(),
        );
    }

    /// TEMPORARY cell census for the insert probe (generic;
    /// env-hook only): member count, min/max cheap, protected count,
    /// below-window count, unprotected count.
    fn trace_cell_debug(&self, cell: (usize, u8)) -> String {
        let Some(members) = self.cell_members.get(&cell) else {
            return "empty".to_string();
        };
        let prot = self.protected_set(cell);
        let max = self.cell_max.get(&cell).map_or(f64::NAN, |(_, m)| *m);
        let min = self.cell_min.get(&cell).map_or(f64::NAN, |(_, m)| *m);
        let below = members
            .iter()
            .filter(|&&i| self.entries[i].cheap_score <= max - EPSILON)
            .count();
        let unprot = members
            .iter()
            .filter(|&&i| !prot.contains(&i))
            .count();
        format!(
            "n={} min={min:.3} max={max:.3} prot={} below={below} unprot={unprot}",
            members.len(),
            prot.len(),
        )
    }

    /// TEMPORARY cell census by candidate shape (generic; env-hook
    /// only). Wraps `trace_cell_debug` for probe call sites that do
    /// not name cells.
    fn trace_debug_for(&self, words_len: usize, sub: f64) -> String {
        self.trace_cell_debug(Self::cell_of(words_len, sub))
    }

    /// TEMPORARY diagnostic accessor: cheap held under a clone
    /// key, if any. Generic; env-hook only.
    fn key_cheap(&self, key: &str) -> Option<f64> {
        self.keys.get(key).map(|&(_, cheap)| cheap)
    }

    /// Same-prefix test for victim selection: two parses share a
    /// prefix family (and must never displace each other outside the
    /// clone path) exactly when both keys are non-empty and equal.
    /// Empty keys (short parses without a family) never match — early
    /// cells keep churning normally.
    fn same_family(a: &Partial, b: &Partial) -> bool {
        !a.prefix_key.is_empty() && a.prefix_key == b.prefix_key
    }

    /// True when a member's prefix family is small (at most
    /// FAM_KEEP sharing a non-empty prefix): rare prefixes are never
    /// fallback victims anywhere. Members without a prefix (short
    /// parses) never count as small-family.
    fn is_small_family(&self, cell: (usize, u8), idx: usize) -> bool {
        let key = self.entries[idx].prefix_key.as_str();
        if key.is_empty() {
            return false;
        }
        self.family_count(cell, key) <= FAM_KEEP
    }

    /// Weakest member outside a small prefix family: the saturated
    /// and extreme fallback victim. Members of small families (at
    /// most FAM_KEEP sharing a non-empty prefix — rare prefixes,
    /// including valley prefixes like a lone resegmentation path)
    /// are never fallback victims; everyone else contends by scalar
    /// (worst first). Members without a prefix (short parses) have
    /// no family and are always eligible. Generic; bounded scan.
    fn min_nonsmall(&self, cell: (usize, u8)) -> Option<usize> {
        let members = self.cell_members.get(&cell)?;
        let mut sizes: HashMap<&str, usize> = HashMap::new();
        for &i in members {
            let key = self.entries[i].prefix_key.as_str();
            if !key.is_empty() {
                *sizes.entry(key).or_default() += 1;
            }
        }
        members
            .iter()
            .filter(|&&i| {
                let key = self.entries[i].prefix_key.as_str();
                key.is_empty() || sizes.get(key).copied().unwrap_or(0) > FAM_KEEP
            })
            .min_by(|&&a, &&b| {
                self.entries[a]
                    .cheap_score
                    .partial_cmp(&self.entries[b].cheap_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    /// Weakest scalar (lowest cheap) member outside the protected set.
    fn worst_unprotected(
        &self,
        cell: (usize, u8),
        protected: &HashSet<usize>,
        candidate: &Partial,
    ) -> Option<usize> {
        self.cell_members.get(&cell).and_then(|members| {
            members
                .iter()
                .filter(|&&i| !protected.contains(&i) && !Self::same_family(&self.entries[i], candidate))
                .min_by(|&&a, &&b| {
                    self.entries[a]
                        .cheap_score
                        .partial_cmp(&self.entries[b].cheap_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
        })
    }

    /// Weakest unprotected member below the epsilon window (if any).
    /// Evicting these first concentrates full cells instead of
    /// churning them: a newcomer inside the window displaces a member
    /// the beam already resolved against, never a peer. Same-family
    /// members are never victims (only clone improvements displace
    /// within a family).
    fn worst_below_window(
        &self,
        cell: (usize, u8),
        protected: &HashSet<usize>,
        candidate: &Partial,
    ) -> Option<usize> {
        let max = self.cell_max.get(&cell).map_or(f64::NAN, |(_, m)| *m);
        if !max.is_finite() {
            return None;
        }
        self.cell_members.get(&cell).and_then(|members| {
            members
                .iter()
                .filter(|&&i| {
                    !protected.contains(&i)
                        && !Self::same_family(&self.entries[i], candidate)
                        && self.entries[i].cheap_score <= max - EPSILON
                })
                .min_by(|&&a, &&b| {
                    self.entries[a]
                        .cheap_score
                        .partial_cmp(&self.entries[b].cheap_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
        })
    }

    /// TEMPORARY trace helper: one-line summary of the target cell
    /// vs the candidate on all four reservoir axes (top-4 thresholds
    /// + cell counts). Generic; env-hook only.
    fn trace_cell_stats(
        &self,
        cand_words: usize,
        cand_sub: f64,
        cand_cheap: f64,
        cand_nov: f64,
        cand_fam: f64,
        cand_last: f64,
        cand_prefix: &[ClueWord],
    ) -> String {
        let cell = Self::cell_of(cand_words, cand_sub);
        let Some(members) = self.cell_members.get(&cell) else {
            return format!("cell={cell:?} empty cap={} hard={}", self.cell_cap, self.cell_hard);
        };
        let mut cheap: Vec<f64> = members.iter().map(|&i| self.entries[i].cheap_score).collect();
        cheap.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let mut nov: Vec<f64> = members.iter().map(|&i| self.entries[i].raw_novelty).collect();
        nov.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let mut fam: Vec<f64> = members.iter().map(|&i| self.entries[i].familiarity).collect();
        fam.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let mut cost: Vec<f64> = members.iter().map(|&i| self.entries[i].sub_cost_total).collect();
        cost.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut last: Vec<f64> = members.iter().map(|&i| self.entries[i].last_cost).collect();
        last.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let t = |v: &[f64]| {
            if v.len() >= RES_Q {
                format!("{:.3}", v[RES_Q - 1])
            } else {
                "n/a".to_string()
            }
        };
        let min = self.cell_min.get(&cell).map_or(f64::NAN, |(_, m)| *m);
        let max = self.cell_max.get(&cell).map_or(f64::NAN, |(_, m)| *m);
        // TEMPORARY diagnostic: lexical family composition — distinct
        // normalized-word sequences vs members (variant explosion
        // check). Generic; env-hook only.
        let mut fam_count: HashMap<String, usize> = HashMap::new();
        for &i in members {
            let seq = self.entries[i]
                .words
                .iter()
                .map(|w| Partial::norm_word(&w.word))
                .collect::<Vec<_>>()
                .join(" ");
            *fam_count.entry(seq).or_default() += 1;
        }
        let nfam = fam_count.len();
        let maxfam = fam_count.values().copied().max().unwrap_or(0);
        // TEMPORARY diagnostic: pico-ended count + per-(n-1)-prefix
        // family stats (weak-evidence rival share). Generic; env-hook
        // only.
        let mut pico = 0usize;
        let mut pre_count: HashMap<String, usize> = HashMap::new();
        for &i in members {
            let ws = &self.entries[i].words;
            if ws.last().map(|w| w.ipa.chars().count() < 3).unwrap_or(false) {
                pico += 1;
            }
            let pre = ws
                .iter()
                .take(ws.len().saturating_sub(1))
                .map(|w| Partial::norm_word(&w.word))
                .collect::<Vec<_>>()
                .join(" ");
            *pre_count.entry(pre).or_default() += 1;
        }
        let npre = pre_count.len();
        let maxpre = pre_count.values().copied().max().unwrap_or(0);
        // TEMPORARY diagnostic: size/best/worst of the CANDIDATE's own
        // prefix family (caller passes the holder's words, i.e. the
        // candidate prefix). Generic; env-hook only.
        let (famsize, fammin, fammax) = {
            let key = cand_prefix
                .iter()
                .map(|w: &ClueWord| Partial::norm_word(&w.word))
                .collect::<Vec<_>>()
                .join(" ");
            let mut n = 0usize;
            let mut lo = f64::INFINITY;
            let mut hi = f64::NEG_INFINITY;
            for &i in members {
                let ws = &self.entries[i].words;
                if ws.len() != cand_prefix.len() + 1 {
                    continue;
                }
                let k = ws[..ws.len() - 1]
                    .iter()
                    .map(|w| Partial::norm_word(&w.word))
                    .collect::<Vec<_>>()
                    .join(" ");
                if k == key {
                    n += 1;
                    let c = self.entries[i].cheap_score;
                    if c < lo {
                        lo = c;
                    }
                    if c > hi {
                        hi = c;
                    }
                }
            }
            (n, lo, hi)
        };
        // TEMPORARY diagnostic counts: per-axis strictly-better /
        // at-or-better tallies for the candidate (generic; env-hook
        // only).
        let mut sb = [0usize; 5];
        let mut wb = [0usize; 5];
        for &i in members {
            let p = &self.entries[i];
            if p.cheap_score > cand_cheap {
                sb[0] += 1;
            }
            if p.cheap_score >= cand_cheap {
                wb[0] += 1;
            }
            if p.raw_novelty > cand_nov {
                sb[1] += 1;
            }
            if p.raw_novelty >= cand_nov {
                wb[1] += 1;
            }
            if p.familiarity > cand_fam {
                sb[2] += 1;
            }
            if p.familiarity >= cand_fam {
                wb[2] += 1;
            }
            if p.sub_cost_total < cand_sub - 1e-9 {
                sb[3] += 1;
            }
            if p.sub_cost_total <= cand_sub + 1e-9 {
                wb[3] += 1;
            }
            if p.last_cost > cand_last + 1e-9 {
                sb[4] += 1;
            }
            if p.last_cost >= cand_last - 1e-9 {
                wb[4] += 1;
            }
        }
        format!(
            "cell={cell:?} n={} cap={} hard={} min={min:.3} max={max:.3} fams={nfam} maxfam={maxfam} pico={pico} npre={npre} maxpre={maxpre} myfam={famsize} myfamrange=[{fammin:.3},{fammax:.3}] top4[cheap>={} nov>={} fam>={} cost<={} last>={}] cand[cheap={cand_cheap:.3} nov={cand_nov:.3} fam={cand_fam:.3} sub={cand_sub:.3} last={cand_last:.3}] ranks5[sb={sb:?} wb={wb:?}]",
            members.len(),
            self.cell_cap,
            self.cell_hard,
            t(&cheap),
            t(&nov),
            t(&fam),
            t(&cost),
            t(&last),
        )
    }

    /// Weakest member the candidate strictly dominates (candidate
    /// at least as good on both axes and strictly better on one),
    /// if any. Deliberately strict (no EPSILON slack): eviction must
    /// never replace a better member with a worse newcomer — the
    /// epsilon slack lives in admission (near-tie growth), not in
    /// displacement. This hysteresis keeps the beam stable: without
    /// it, worse newcomers churn mid-pack members out on
    /// cost-technicalities even though the final scorer would rank
    /// the evicted parse as highly as its replacement.
    fn dominates_some(&self, cell: (usize, u8), cand_cheap: f64, cand_sub: f64) -> Option<usize> {
        self.cell_members.get(&cell).and_then(|members| {
            members
                .iter()
                .filter(|&&i| {
                    let p = &self.entries[i];
                    cand_cheap >= p.cheap_score
                        && cand_sub <= p.sub_cost_total + 1e-9
                        && (cand_cheap > p.cheap_score || cand_sub < p.sub_cost_total - 1e-9)
                })
                .min_by(|&&a, &&b| {
                    self.entries[a]
                        .cheap_score
                        .partial_cmp(&self.entries[b].cheap_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
        })
    }

    /// Evidence threshold for beam displacement: clue words shorter
    /// than this many IPA characters carry too little phonetic evidence
    /// to evict established parses. Single-segment words ("a", "I",
    /// "uh") and two-segment function words ("to", "ad", "at", "of")
    /// match almost everywhere, so letting them displace freely lets
    /// combinatorial PicoWord salads churn genuine multi-segment
    /// resegmentations out of every cell they touch — even though the
    /// final scorer ranks those resegmentations highly. PicoWords still
    /// compete freely for room and among themselves (same-class
    /// displacement), so early single-vowel function words are
    /// unaffected; they just cannot evict parses built
    /// on more phonetic evidence, nor grow full cells by near-tie.
    /// Generic word-shape rule, no lexical content.
    const PICO_LEN: usize = 3;

    /// IPA length of a partial's most recent word: the evidence weight
    /// of its latest extension. Empty partials (beam roots) count as
    /// full evidence.
    fn last_ipa_len(p: &Partial) -> usize {
        p.words
            .last()
            .map(|w| w.ipa.chars().count())
            .unwrap_or(usize::MAX)
    }

    /// Full admission with clone-suppression and diversity caps.
    /// Cells grow freely to their soft share; past it, near-ties and
    /// reservoir champions join up to the hard cap; settled cells
    /// admit objective extremes, window/champion members, dominators
    /// and novel endings — never bare scalar improvements. Mirrors
    /// [`BeamPos::would_admit`].
    /// TEMPORARY: returns a decision tag for the env-hook trace.
    fn insert(&mut self, candidate: Partial) -> &'static str {
        if self.k == 0 {
            return "k0";
        }
        let cell = Self::cell_of(candidate.words.len(), candidate.sub_cost_total);
        if let Some(&(idx, cheap)) = self.keys.get(&candidate.key) {
            if candidate.cheap_score <= cheap {
                return "clone-dup";
            }
            // Improvement for a held clone slot: swap in place. The
            // slot keeps its cell even if the improved acoustics
            // would tier differently — relocating across cells
            // breaks the member-vector accounting (duplicate-index
            // buildup), while the score/cheap update itself is what
            // matters for downstream expansion.
            self.replace(idx, candidate, "clone-improve");
            return "clone-improve";
        }
        let cell_count = self.cell_members.get(&cell).map_or(0, Vec::len);
        // Protection snapshots are computed lazily: roomy growth
        // pushes never displace, so they skip the sorts entirely.
        // Every displacement below consults the snapshot (never a
        // stale one — it is built after the clone check on current
        // members, and nothing mutates between build and use).
        let mut cached_prot: Option<HashSet<usize>> = None;
        // Objective-extreme flag for the growth/full paths below:
        // extremes bypass tie bands (they exceed rather than tie).
        let extreme = self.is_strictly_best(
            cell,
            candidate.cheap_score,
            candidate.raw_novelty,
            candidate.familiarity,
            candidate.sub_cost_total,
            candidate.last_cost,
        );
        // Prefix-family gate (uniform across phases): a saturated
        // family refuses newcomers outright (its members resolve
        // internally through clone improvements); small families
        // always have room. PicoWord candidates skip this gate
        // (their same-class rule below is family-aware on its own).
        if Self::last_ipa_len(&candidate) >= Self::PICO_LEN
            && self.family_is_full(cell, &candidate)
        {
            return "family-shut";
        }
        if extreme && cell_count >= self.cell_hard {
            if cached_prot.is_none() {
                cached_prot = Some(self.protected_set(cell));
            }
            // Borrow ends before `replace` (the set lives in the
            // local cache, not in `self`). Small families and
            // same-family twins are exempt from the min fallback
            // (uniform rarity rule). A miss falls through to the
            // normal and saturated rules below (an extreme that
            // cannot displace here may still qualify there).
            let victim = self
                .worst_unprotected(cell, cached_prot.as_ref().unwrap(), &candidate)
                .or_else(|| {
                    self.cell_members.get(&cell).and_then(|members| {
                        members
                            .iter()
                            .filter(|&&i| {
                                !Self::same_family(&self.entries[i], &candidate)
                                    && !self.is_small_family(cell, i)
                            })
                            .min_by(|&&a, &&b| {
                                self.entries[a]
                                    .cheap_score
                                    .partial_cmp(&self.entries[b].cheap_score)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .copied()
                    })
                });
            if let Some(victim) = victim {
                self.replace(victim, candidate, "extreme");
                return "extreme-hit";
            }
        }
        if cell_count < self.cell_cap {
            // Roomy cell: join freely (family-saturated substantial
            // candidates were already refused above; PicoWords in
            // roomy cells compete freely as before).
            self.push_new(candidate);
            return "cap-push";
        }
        if Self::last_ipa_len(&candidate) < Self::PICO_LEN {
            // PicoWord in a full cell: room is gone, near-tie growth
            // stays closed to PicoWords (that is the swamp
            // mechanism), and substantial members are immune — only a
            // same-class, unprotected slot of a DIFFERENT family may
            // turn over to a better PicoWord (same-family turnover
            // would churn near-twins; the family admission cap
            // already bounds same-prefix PicoWords).
            if cached_prot.is_none() {
                cached_prot = Some(self.protected_set(cell));
            }
            let protected: &HashSet<usize> = cached_prot.as_ref().unwrap();
            let victim = self
                .cell_members
                .get(&cell)
                .and_then(|members| {
                    members
                        .iter()
                        .filter(|&&i| {
                            !protected.contains(&i)
                                && !Self::same_family(&self.entries[i], &candidate)
                                && Self::last_ipa_len(&self.entries[i]) < Self::PICO_LEN
                        })
                        .min_by(|&&a, &&b| {
                            self.entries[a]
                                .cheap_score
                                .partial_cmp(&self.entries[b].cheap_score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .copied()
                })
                .filter(|&idx| candidate.cheap_score > self.entries[idx].cheap_score);
            if let Some(idx) = victim {
                self.replace(idx, candidate, "pico");
                return "pico-hit";
            }
            return "pico-miss";
        }
        if cell_count < self.cell_hard {
            // Near-tie extension: join unless the whole kept cluster
            // beats the candidate by more than EPSILON. Reservoir
            // champions always join (bounded by the hard cap), as do
            // objective extremes (which the band might miss inside a
            // tie crowd).
            let near_tie = match self.cell_min.get(&cell) {
                Some(&(_, min)) => candidate.cheap_score > min - EPSILON,
                None => true,
            };
            if near_tie
                || extreme
                || self.would_enter_banded(
                    cell,
                    candidate.cheap_score,
                    candidate.raw_novelty,
                    candidate.familiarity,
                    candidate.sub_cost_total,
                    candidate.last_cost,
                )
            {
                self.push_new(candidate);
                return "growth-push";
            }
        }
        // Full cell (family gate handled above): settle by strict
        // scalar improvement over the worst non-exempt member, or by
        // objective extreme. An objective extreme always earns its
        // slot by displacing the worst member it may (an unprotected
        // one, else the cell worst — extremes are rare enough that
        // this cannot churn). Otherwise the candidate must strictly
        // beat the worst non-exempt cheap to displace it; below-window
        // members go first since they fell out of the competitive
        // band. Protected champions (axis bands, families, all gated
        // on the epsilon window) are never evicted except by an
        // objective extreme; a saturated cell with no unprotected
        // members freezes rather than churn. Deliberately narrow:
        // past the hard cap, scalar gaps among dozens of kept members
        // are noise the beam cannot resolve — but a strict
        // improvement over the floor is signal, and it is exactly how
        // valley prefixes that paid cost early re-enter contention.
        // The growth phase below the hard cap already arbitrates
        // scalar near-ties.
        if extreme {
            if cached_prot.is_none() {
                cached_prot = Some(self.protected_set(cell));
            }
            let victim = self
                .worst_unprotected(cell, cached_prot.as_ref().unwrap(), &candidate)
                .or_else(|| {
                    // Extreme last resort: the worst member outside a
                    // small family (rare prefixes never fall, not even
                    // to extremes; same-family twins never fall to each
                    // other outside clone improvements either).
                    self.cell_members.get(&cell).and_then(|members| {
                        members
                            .iter()
                            .filter(|&&i| {
                                !Self::same_family(&self.entries[i], &candidate)
                                    && !self.is_small_family(cell, i)
                            })
                            .min_by(|&&a, &&b| {
                                self.entries[a]
                                    .cheap_score
                                    .partial_cmp(&self.entries[b].cheap_score)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .copied()
                    })
                });
            if let Some(victim) = victim {
                self.replace(victim, candidate, "full-extreme");
                return "full-extreme-hit";
            }
            // Miss falls through to the normal and saturated rules
            // below (an extreme that cannot displace here may still
            // qualify there).
        }
        if cached_prot.is_none() {
            cached_prot = Some(self.protected_set(cell));
        }
        let protected: &HashSet<usize> = cached_prot.as_ref().unwrap();
        let victim = self
            .worst_below_window(cell, protected, &candidate)
            .or_else(|| self.worst_unprotected(cell, protected, &candidate));
        if let Some(victim) = victim {
            if candidate.cheap_score > self.entries[victim].cheap_score {
                self.replace(victim, candidate, "strict");
                return "full-hit";
            }
            return "full-reject-below-victim";
        }
        // Saturated cell: every member is protected. Two narrow
        // doors remain, both margin-gated so near-tie twins coexist
        // instead of churning sideways (incumbency wins ties).
        // (1) A newcomer beating the floor by more than MERIT_MARGIN
        // earns its slot; above-margin gaps ratchet the floor upward
        // in stable steps, and the floor stalls within a margin of
        // the best (no newcomer can beat a floor that close to the
        // ceiling by a margin), so near-best valleys are never
        // reached. (2) A reservoir champion (strict top-Q on an axis
        // with a bounded tie crowd) landing strictly below the floor
        // by more than MERIT_MARGIN but within EPSILON of it earns
        // its slot too — its axis merit outweighs a small scalar
        // deficit, while the epsilon cap bounds dilution (a far-below
        // champion is still refused) and the margin blocks twin churn
        // from below. Small families are exempt from floor duty on
        // both doors (rare prefixes never fall), as are the
        // candidate's own twins: saturation means the quotas already
        // spoke for everyone else, and only clear wins reopen the
        // question. This is what lets genuine valley prefixes
        // dislodge floor junk instead of bouncing off a frozen cell.
        if let Some(victim) = self.cell_members.get(&cell).and_then(|members| {
            members
                .iter()
                .filter(|&&i| {
                    !Self::same_family(&self.entries[i], &candidate)
                        && !self.is_small_family(cell, i)
                })
                .min_by(|&&a, &&b| {
                    self.entries[a]
                        .cheap_score
                        .partial_cmp(&self.entries[b].cheap_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
        }) {
            let floor = self.entries[victim].cheap_score;
            if candidate.cheap_score > floor + MERIT_MARGIN {
                self.replace(victim, candidate, "saturated-hit");
                return "saturated-hit";
            }
            if candidate.cheap_score < floor - MERIT_MARGIN
                && candidate.cheap_score > floor - EPSILON
                && self.would_enter_banded(
                    cell,
                    candidate.cheap_score,
                    candidate.raw_novelty,
                    candidate.familiarity,
                    candidate.sub_cost_total,
                    candidate.last_cost,
                )
            {
                self.replace(victim, candidate, "saturated-merit");
                return "saturated-merit";
            }
        }
        // Everything protected and no objective extreme: freeze.
        "freeze"
    }

    fn push_new(&mut self, candidate: Partial) {
        let idx = self.entries.len();
        let cell = Self::cell_of(candidate.words.len(), candidate.sub_cost_total);
        let cheap = candidate.cheap_score;
        self.keys.insert(candidate.key.clone(), (idx, cheap));
        self.cell_members.entry(cell).or_default().push(idx);
        if let Some(w) = candidate.words.last() {
            *self.ending_counts.entry((cell, w.ipa.clone())).or_default() += 1;
        }
        self.entries.push(candidate);
        Self::lower_min(&mut self.cell_min, cell, idx, cheap);
        Self::raise_max(&mut self.cell_max, cell, idx, cheap);
    }

    /// Replace entry at `idx` in place (other indices unchanged).
    /// Usually the slot keeps its cell (clone improvements,
    /// cell-worst displacements); the clone tier-change path may
    /// move a slot across cells, in which case member vectors,
    /// minima and the ending census all follow the entry.
    fn replace(&mut self, idx: usize, candidate: Partial, why: &'static str) {
        // TEMPORARY eviction forensics (generic; env-hook only):
        // when the evicted entry holds a traced prefix, report its
        // family size and protection state so disappearances can be
        // attributed. Protection work runs only on traced hits.
        if Self::trace_matches(&self.entries[idx]) {
            let old_cell = Self::cell_of(
                self.entries[idx].words.len(),
                self.entries[idx].sub_cost_total,
            );
            let fam_size = self.family_count(old_cell, &self.entries[idx].prefix_key);
            let prot = self.protected_set(old_cell);
            Self::trace_evicted(
                &self.entries[idx],
                &candidate,
                why,
                fam_size,
                prot.contains(&idx),
                prot.len(),
            );
        }
        let old = std::mem::replace(&mut self.entries[idx], candidate);
        self.keys.remove(&old.key);
        let new_key = self.entries[idx].key.clone();
        let new_cheap = self.entries[idx].cheap_score;
        self.keys.insert(new_key, (idx, new_cheap));
        let old_cell = Self::cell_of(old.words.len(), old.sub_cost_total);
        let new_cell = Self::cell_of(
            self.entries[idx].words.len(),
            self.entries[idx].sub_cost_total,
        );
        if old_cell != new_cell {
            if let Some(members) = self.cell_members.get_mut(&old_cell) {
                if let Some(pos) = members.iter().position(|&i| i == idx) {
                    members.swap_remove(pos);
                }
            }
            self.cell_members.entry(new_cell).or_default().push(idx);
            self.refresh_cell_min(old_cell);
            self.refresh_cell_max(old_cell);
        }
        // Ending census follows the words, not the slot.
        let old_end = old.words.last().map(|w| w.ipa.clone());
        let new_end = self.entries[idx].words.last().map(|w| w.ipa.clone());
        if old_end != new_end {
            if let Some(o) = old_end {
                if let Some(c) = self.ending_counts.get_mut(&(old_cell, o)) {
                    *c = c.saturating_sub(1);
                }
            }
            if let Some(n) = new_end {
                *self.ending_counts.entry((new_cell, n)).or_default() += 1;
            }
        }
        // Refresh minima and maxima that may have moved. A stale low
        // minimum would make the gate skip admittable candidates, so
        // when the evicted entry was the minimum, rescan; likewise a
        // stale low maximum would wrongly refuse window candidates.
        // The new cell always learns the replacement (it may be its
        // new best or worst).
        let cell_min_was_this = self.cell_min.get(&new_cell).is_some_and(|(i, _)| *i == idx);
        if cell_min_was_this {
            self.refresh_cell_min(new_cell);
        } else {
            match self.cell_min.get(&new_cell) {
                Some(&(_, min)) if new_cheap < min => {
                    self.cell_min.insert(new_cell, (idx, new_cheap));
                }
                None => {
                    self.cell_min.insert(new_cell, (idx, new_cheap));
                }
                _ => {}
            }
        }
        let cell_max_was_this = self.cell_max.get(&new_cell).is_some_and(|(i, _)| *i == idx);
        if cell_max_was_this {
            self.refresh_cell_max(new_cell);
        } else {
            match self.cell_max.get(&new_cell) {
                Some(&(_, max)) if new_cheap > max => {
                    self.cell_max.insert(new_cell, (idx, new_cheap));
                }
                None => {
                    self.cell_max.insert(new_cell, (idx, new_cheap));
                }
                _ => {}
            }
        }
    }

    fn lower_min(
        map: &mut HashMap<(usize, u8), (usize, f64)>,
        cell: (usize, u8),
        idx: usize,
        cheap: f64,
    ) {
        match map.get(&cell) {
            Some(&(_, min)) if min <= cheap => {}
            _ => {
                map.insert(cell, (idx, cheap));
            }
        }
    }

    fn refresh_cell_min(&mut self, cell: (usize, u8)) {
        let best = self.cell_members.get(&cell).and_then(|members| {
            members
                .iter()
                .map(|&i| (i, self.entries[i].cheap_score))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        match best {
            Some(v) => {
                self.cell_min.insert(cell, v);
            }
            None => {
                self.cell_min.remove(&cell);
            }
        }
    }

    fn raise_max(
        map: &mut HashMap<(usize, u8), (usize, f64)>,
        cell: (usize, u8),
        idx: usize,
        cheap: f64,
    ) {
        match map.get(&cell) {
            Some(&(_, max)) if max >= cheap => {}
            _ => {
                map.insert(cell, (idx, cheap));
            }
        }
    }

    fn refresh_cell_max(&mut self, cell: (usize, u8)) {
        let best = self.cell_members.get(&cell).and_then(|members| {
            members
                .iter()
                .map(|&i| (i, self.entries[i].cheap_score))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        match best {
            Some(v) => {
                self.cell_max.insert(cell, v);
            }
            None => {
                self.cell_max.remove(&cell);
            }
        }
    }
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
        // "cat" IPA = "kæt". Completions whose word IPAs concatenate
        // to "kæt": just the single word "cat" itself (in our tiny
        // corpus). "ka" + "t" doesn't work — "t" alone isn't a word.
        let clues = g.generate("cat");
        assert!(!clues.is_empty(), "expected at least one covering");
        assert!(clues.iter().any(|c| c.phrase == "cat"));
    }

    #[test]
    fn empty_when_target_word_unknown() {
        let g = Generator::from_json(TINY, GeneratorConfig::default()).unwrap();
        let clues = g.generate("orange");
        assert!(clues.is_empty());
    }
}

#[cfg(test)]
mod pareto_tests {
    use super::*;

    fn entry(word: &str, ipa: &str, rarity: f64) -> ApproxEntry {
        ApproxEntry {
            word: word.to_string(),
            ipa: ipa.to_string(),
            chars: ipa.chars().collect(),
            rarity: Some(rarity),
        }
    }

    fn query(words: Vec<ApproxEntry>, target: &str, budget: f64, cap: usize) -> Vec<ApproxMatch> {
        let trie = ApproxTrie::build(&words);
        let chars: Vec<char> = target.chars().collect();
        trie.words_approximately_starting_at(&words, &chars, 0, budget, cap)
    }

    #[test]
    fn shortlist_keeps_costly_familiar_word_in_span() {
        // 40 rare words match "abcde" exactly (cost 0); one familiar
        // word matches with a weak deletion (cost 0.6, near the
        // budget). A pure cheapest-first cap of 20 would drop it at
        // rank 41; the familiarity axis must retain it.
        let mut entries: Vec<ApproxEntry> = (0..40)
            .map(|i| entry(&format!("rarew{i}"), "abcde", 45_000.0))
            .collect();
        entries.push(entry("common", "abcd", 300.0));
        let ms = query(entries, "abcde", 0.75, 20);
        assert!(
            ms.iter().any(|m| m.word == "common"),
            "familiar near-budget word was dropped: {:?}",
            ms.iter().map(|m| m.word.clone()).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn shortlist_keeps_alternative_spans() {
        // 60 rare single-segment matches plus one lone 7-segment
        // match: a small cap must still cover both spans.
        let mut entries: Vec<ApproxEntry> = (0..60)
            .map(|i| entry(&format!("x{i}"), "a", 40_000.0))
            .collect();
        entries.push(entry("longword", "abcdefg", 20_000.0));
        let ms = query(entries, "abcdefg", 0.75, 10);
        assert!(
            ms.iter().any(|m| m.word == "longword"),
            "minority-span match was dropped"
        );
    }

    fn partial_with(cheap: f64, cost: f64, nwords: usize, key: &str) -> Partial {
        let words: Vec<ClueWord> = (0..nwords)
            .map(|i| ClueWord {
                word: format!("w{i}"),
                ipa: "ab".to_string(),
                rarity: None,
                sub_cost: 0.0,
            })
            .collect();
        let prefix_key = Partial::prefix_key_for(&words);
        Partial {
            words,
            sub_cost_total: cost,
            cheap_score: cheap,
            novelty: 0.5,
            raw_novelty: 0.5,
            familiarity: 0.0,
            last_cost: 0.0,
            cuts: Vec::new(),
            key: key.to_string(),
            prefix_key,
        }
    }

    /// Prefix-family quota: one prolific prefix cannot hog a cell,
    /// while small families keep everything. Uses synthetic words
    /// (no lexicon content).
    #[test]
    fn beam_caps_prefix_family_but_keeps_small_ones() {
        // Three-word partials; IPAs are 3+ chars so the PicoWord rule
        // stays out of the way. Novelty differs per group so
        // tie-band protection behaves like in real cells.
        fn fam3(first: &str, second: &str, last: &str, cheap: f64, nov: f64) -> Partial {
            let words = vec![
                ClueWord {
                    word: first.to_string(),
                    ipa: "abc".to_string(),
                    rarity: None,
                    sub_cost: 0.0,
                },
                ClueWord {
                    word: second.to_string(),
                    ipa: "def".to_string(),
                    rarity: None,
                    sub_cost: 0.0,
                },
                ClueWord {
                    word: last.to_string(),
                    ipa: "ghi".to_string(),
                    rarity: None,
                    sub_cost: 0.0,
                },
            ];
            let prefix_key = Partial::prefix_key_for(&words);
            let key = format!("{first} {second} {last}");
            Partial {
                words,
                sub_cost_total: 0.0,
                cheap_score: cheap,
                novelty: nov,
                raw_novelty: nov,
                familiarity: 0.0,
                last_cost: 0.0,
                cuts: Vec::new(),
                key,
                prefix_key,
            }
        }
        // BeamPos::new(8): cell_cap=8, hard=64. One saturated family
        // ("pa pb", novelty 0.5) plus a crowd of diverse filler with
        // higher novelty, filling the (3, tier-0) cell to the hard
        // cap.
        let mut beam = BeamPos::new(8);
        for i in 0..RES_Q {
            beam.insert(fam3("pa", "pb", &format!("m{i}"), 0.9 - 0.01 * i as f64, 0.5));
        }
        for i in 0..(64 - RES_Q) {
            beam.insert(fam3(
                &format!("f{i}"),
                "g",
                &format!("b{i}"),
                0.80,
                0.9,
            ));
        }
        assert_eq!(beam.entries.len(), 64);
        // A worse same-family newcomer is refused (family saturated).
        beam.insert(fam3("pa", "pb", "zz", 0.10, 0.5));
        assert_eq!(
            beam.entries.len(),
            64,
            "saturated family must refuse a worse member"
        );
        assert!(
            !beam.entries.iter().any(|p| p.words[2].word == "zz"),
            "refused member must stay out"
        );
        // A better same-family newcomer is likewise refused: the
        // family is saturated and first-come stands (within-family
        // upgrades arrive through clone improvements, not eviction).
        beam.insert(fam3("pa", "pb", "top", 0.99, 0.5));
        assert!(
            !beam.entries.iter().any(|p| p.words[2].word == "top"),
            "saturated family must refuse even a better member"
        );
        // Open families proceed normally: a lone prefix joins a roomy
        // cell (small families are never capped).
        let mut beam2 = BeamPos::new(8);
        beam2.insert(fam3("kc", "kd", "lone", -5.0, 0.5));
        assert!(
            beam2.entries.iter().any(|p| p.words[2].word == "lone"),
            "small-family member must join a roomy cell"
        );
    }

    #[test]
    fn beam_preserves_low_cost_prefix_under_parrot_pressure() {
        // A band of zero-cost parrot prefixes (best heuristic
        // quality) must not squeeze out a slightly costlier genuine
        // resegmentation while the beam still has room: unfilled
        // beams admit (clone-checked) and caps only arbitrate once
        // full, so thin strategies survive alongside the mainstream.
        let mut beam = BeamPos::new(16);
        for i in 0..8 {
            beam.insert(partial_with(
                0.0 - 0.01 * i as f64,
                0.0,
                2,
                &format!("parrot {i}"),
            ));
        }
        assert_eq!(beam.entries.len(), 8);
        // Beam still has room: worse parrots and the resegmentation
        // both admit without evicting anyone.
        beam.insert(partial_with(-0.5, 0.0, 2, "parrot tail"));
        // Low-cost resegmentation: worse heuristic quality than the
        // parrots, but a distinct acoustic hypothesis the final
        // scorer must see.
        beam.insert(partial_with(-0.45, 0.15, 2, "re seg"));
        assert!(
            beam.entries.iter().any(|p| p.key == "re seg"),
            "low-cost resegmentation prefix was squeezed out by parrots"
        );
    }

    /// Clone identity is (word, IPA) per step: two different words
    /// with identical pronunciation coexist in one beam cell, while
    /// an exact same word+IPA duplicate collapses to one slot.
    /// Uses synthetic two-letter words (no lexicon content).
    #[test]
    fn beam_keeps_homophones_distinct_but_collapses_duplicates() {
        fn one_word(word: &str, ipa: &str, cheap: f64) -> Partial {
            let mut p = Partial::empty();
            p.words.push(ClueWord {
                word: word.to_string(),
                ipa: ipa.to_string(),
                rarity: None,
                sub_cost: 0.0,
            });
            p.cheap_score = cheap;
            p.key = Partial::join_key("", word, ipa);
            p
        }
        let mut beam = BeamPos::new(64);
        beam.insert(one_word("aa", "ab", 0.5));
        beam.insert(one_word("bb", "ab", 0.4));
        assert_eq!(
            beam.entries.len(),
            2,
            "different words with identical IPA must coexist"
        );
        beam.insert(one_word("aa", "ab", 0.3));
        assert_eq!(
            beam.entries.len(),
            2,
            "exact same word+IPA duplicate must collapse"
        );
        // A strictly better exact duplicate improves the held slot.
        beam.insert(one_word("aa", "ab", 0.6));
        assert_eq!(beam.entries.len(), 2);
        assert!(
            beam.entries
                .iter()
                .any(|p| p.words.len() == 1 && p.words[0].word == "aa" && p.cheap_score == 0.6),
            "better duplicate should improve the held clone slot"
        );
    }
}

/// Shortlist recall for the canonical resegmentation words: the
/// per-position candidate shortlist (bounded union of acoustic-cost
/// and lexical-familiarity elites per consumed span) must emit each
/// keystone word of the classic multiword parses at its path
/// position and cost. This pins the trie/shortlist end of the chain
/// (beam retention is pinned by `pareto_tests` and the corpus
/// integration tests); it fails if quotas or budgets ever starve a
/// keystone word behind hundreds of cheaper same-position matches.
#[cfg(test)]
mod shortlist_recall_tests {
    use super::*;
    use open_english_pronouncing_dictionary::CORPUS_JSON;

    fn approx_gen() -> Generator {
        Generator::from_json(
            CORPUS_JSON,
            GeneratorConfig {
                max_rarity: Some(50_000.0),
                mode: SearchMode::Approximate {
                    per_word_budget: 0.75,
                    total_budget: 1.5,
                },
                ..GeneratorConfig::default()
            },
        )
        .unwrap()
    }

    /// (target, trie position, wanted word, max acceptable cost):
    /// every row must appear in the shortlist at that position.
    const RECALL_ROWS: &[(&str, usize, &str, f64)] = &[
        ("It's just a stupid game", 0, "hits", 0.2),
        ("It's just a stupid game", 3, "justice", 0.1),
        ("It's just a stupid game", 10, "dupe", 0.2),
        ("It's just a stupid game", 13, "hid", 0.4),
        ("It's just a stupid game", 15, "came", 0.2),
        ("recognize speech", 0, "wreck", 0.1),
        ("recognize speech", 3, "a", 0.1),
        ("recognize speech", 4, "nice", 0.8),
        ("recognize speech", 9, "beach", 0.8),
    ];

    #[test]
    fn shortlist_emits_resegmentation_keystones() {
        let g = approx_gen();
        for (target, pos, want, max_cost) in RECALL_ROWS {
            let (tipa, _) = transcribe_normalized_with_boundaries(g.corpus(), target).unwrap();
            let chars: Vec<char> = tipa.chars().collect();
            let ms = g.approx_trie.words_approximately_starting_at(
                &g.approx_entries,
                &chars,
                *pos,
                0.75,
                250,
            );
            assert!(
                ms.iter()
                    .any(|m| m.word.to_lowercase() == *want && m.cost <= *max_cost),
                "{want:?} missing from pos-{pos} shortlist for {target:?} ({} hits)",
                ms.len(),
            );
        }
    }
}
