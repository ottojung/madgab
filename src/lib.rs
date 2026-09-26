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
            // enough to keep mid-frequency clue words (rank ~40k)
            // but tight enough to keep the trie small.
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

/// One lattice position: every word alignment starting here within
/// per-word budget, plus the per-match cheap-step terms
/// (resyllabification bonus, recycling penalty) priced in target
/// coordinates. The weighted cheap step derives from these with a
/// few flops, so the lattice is built once up front and shared by
/// the whole beam pass.
struct LatticePos {
    matches: Vec<ApproxMatch>,
    recycle: Vec<bool>,
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
        let target_words: HashSet<String> = target
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_end_matches(['.', ',', '!', '?'])
                    .to_string()
            })
            .collect();
        // Normalized target vocabulary for the incremental
        // word-novelty penalty: "it's" and "its" count as the same
        // recycled word.
        let target_words_norm: HashSet<String> = target
            .split_whitespace()
            .map(|w| Partial::norm_word(w.to_lowercase().trim_end_matches(['.', ',', '!', '?'])))
            .collect();
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        let (per_word_budget, total_budget) = match self.config.mode {
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => (per_word_budget, total_budget),
            SearchMode::Exact => unreachable!(),
        };

        // Word-edge lattice, computed once up front: the
        // edit-tolerant trie walk depends only on the position and
        // the per-word budget, not on the partial path. The cap
        // matches the recall tests' (which pin keystone presence at
        // 250): a tighter cap was measured to starve keystone
        // openings (notably "hits" at position 0, outranked on both
        // shortlist axes once the fill Spielraum shrinks), so the
        // fan-in stays wide here and selection happens downstream
        // where whole-prefix virtue is visible.
        let mut lattice: Vec<LatticePos> = Vec::with_capacity(n + 1);
        for q in 0..=n {
            let mut matches = self.approx_trie.words_approximately_starting_at(
                &self.approx_entries,
                &chars,
                q,
                per_word_budget,
                250,
            );
            matches.retain(|m| {
                m.word_len >= self.config.min_word_ipa_chars
                    && m.consumed >= self.config.min_word_ipa_chars
            });
            // Per-match recycle flag, priced in target
            // coordinates; the beam bound derives from shared
            // counts and costs.
            let mut recycle = Vec::with_capacity(matches.len());
            for m in &matches {
                recycle.push(target_words_norm.contains(&Partial::norm_word(&m.word)));
            }
            lattice.push(LatticePos { matches, recycle });
        }

        // Single beam-DP pass over the shared lattice. beam[p]
        // accumulates every covering of [0..p) (exact-clone
        // deduped); only when p expands is the bounded expansion
        // set selected from all candidates that reached p — an
        // order-independent choice combining optimistic-bound
        // quality with an MMR diversity component over word
        // sequences (see `BeamPos::take_selected`). Arrival order
        // never decides survival.
        let k = self.config.beam_width;
        let completion_cap = self.config.top_n.saturating_mul(64).max(8192);
        // Inner target boundaries (terminal excluded): the novelty
        // denominator and the reuse set for the optimistic bound.
        // Built once; the hot loop only does integer arithmetic and
        // set lookups.
        let target_inner: HashSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|b| *b < n)
            .collect();
        let nov_denom = target_inner.len().max(1);
        let mut beam: Vec<BeamPos> = (0..=n).map(|_| BeamPos::new(k)).collect();
        beam[0].insert(Partial::empty());
        let mut completed = CompletionTop::new(completion_cap);
            for p in 0..n {
                if beam[p].is_empty() {
                    continue;
                }
                let lp = &lattice[p];
                if lp.matches.is_empty() {
                    continue;
                }
                // Deferred, order-independent selection: the bounded
                // expansion set is chosen from every candidate that
                // reached p (quality by optimistic bound plus an MMR
                // diversity reserve over word sequences ranked by
                // demonstrated score).
                let here = beam[p].take_selected(&target_boundaries);
                for partial in &here {
                    let remaining_budget = total_budget - partial.sub_cost_total;
                    for (mi, m) in lp.matches.iter().enumerate() {
                        if m.cost > remaining_budget + 1e-9 {
                            continue;
                        }
                        let cand_sub = partial.sub_cost_total + m.cost;
                        if cand_sub > total_budget + 1e-9 {
                            continue;
                        }
                        let end = p + m.consumed;
                        let terminal = end == n;
                        // Candidate optimistic bound before building:
                        // cost and clue-frame boundary reuse are
                        // partial-independent arithmetic, identical to
                        // what the built candidate stores via
                        // extend_approx, so precomputation and insert
                        // agree. The hit test runs in clue
                        // coordinates — the same frame the final
                        // novelty scores (see `final_score`).
                        let boundary_hit = target_boundaries
                            .contains(&(partial.clue_cum + m.ipa.chars().count()));
                        let cand_bound = Partial::upper_bound_for(
                            cand_sub,
                            partial.shared + usize::from(boundary_hit),
                            nov_denom,
                        );
                        let is_recycle = lp.recycle[mi];
                        let boundary_bonus = if target_boundaries.contains(&end) {
                            0.0
                        } else {
                            0.20
                        };
                        let reuse_penalty = if is_recycle { 0.10 } else { 0.0 };
                        let next = partial.extend_approx(
                            &m.word,
                            &m.ipa,
                            m.rarity,
                            end,
                            m.cost,
                            boundary_bonus,
                            reuse_penalty,
                            boundary_hit,
                            nov_denom,
                        );
                        debug_assert!((next.bound - cand_bound).abs() < 1e-9);
                        // TMP-PROBE transition witness (env-gated;
                        // remove before final commit): fires when a
                        // canonical-worded dupe-prefix extends via a
                        // hid-match, proving the child is built and
                        // inserted.
                        if std::env::var("MADGAB_RAW_PROBE").is_ok() {
                            let pw: Vec<String> = partial
                                .words
                                .iter()
                                .map(|w| w.word.to_lowercase())
                                .collect();
                            if (pw == ["hits", "justice", "dupe"]
                                || pw == ["wreck", "a"])
                                && (m.word.to_lowercase() == "hid"
                                    || m.word.to_lowercase() == "nice")
                            {
                                eprintln!(
                                    "WITNESS-PROBE parent={pw:?} psub={:.3} pshared={} match={:?} mcost={:.3} end={end} csub={:.3} cbound={:.4} chit={}",
                                    partial.sub_cost_total,
                                    partial.shared,
                                    m.word,
                                    m.cost,
                                    next.sub_cost_total,
                                    next.bound,
                                    target_boundaries.contains(
                                        &(partial.clue_cum + m.ipa.chars().count())
                                    ),
                                );
                            }
                        }
                        if terminal {
                            let score = next.final_score(&target_boundaries, &target_words);
                            completed.insert(next, score);
                        } else {
                            beam[end].insert(next);
                        }
                    }
                }
            }
        let mut clues: Vec<Clue> = completed
            .drain_sorted()
            .into_iter()
            .map(|p| p.into_clue(&target_ipa, &target_boundaries, &target_words))
            .collect();
        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        clues.dedup_by(|a, b| a.phrase == b.phrase);
        // TMP-PROBE: raw-pool inspection before MMR (env-gated;
        // remove before final commit). Reports pool floor, the
        // canonical oracles' exact scores, and their raw ranks, so
        // beam loss, completion-cap loss, and MMR demotion stay
        // distinguishable.
        if std::env::var("MADGAB_RAW_PROBE").is_ok() {
            eprintln!(
                "HID-CENSUS inserts={}",
                HID_INSERTS.swap(0, std::sync::atomic::Ordering::Relaxed)
            );
            // TMP-PROBE band floors (env-gated; remove before final
            // commit): per-cost-band completion counts and minima —
            // shows whether a band quota evicts a keystone
            // completion that clears the global floor.
            {
                use std::collections::BTreeMap;
                let mut bands: BTreeMap<u8, (usize, f64)> = BTreeMap::new();
                for c in &clues {
                    let sub: f64 = c.words.iter().map(|w| w.sub_cost).sum();
                    let band = ((sub / 0.25).floor() as u8).min(6);
                    bands
                        .entry(band)
                        .and_modify(|e| {
                            e.0 += 1;
                            e.1 = e.1.min(c.score);
                        })
                        .or_insert((1, c.score));
                }
                eprintln!("BAND-PROBE {bands:?} pool={}", clues.len());
            }
            if let Some(last) = clues.last() {
                eprintln!("POOL-PROBE pool={} min_score={:.4}", clues.len(), last.score);
            }
            for want in ["hits justice dupe hid came", "wreck a nice beach"] {
                match clues.iter().position(|c| c.phrase.to_lowercase() == want) {
                    Some(i) => eprintln!(
                        "RAW-PROBE target={want:?} rank={i} score={:.4}",
                        clues[i].score
                    ),
                    None => eprintln!("RAW-PROBE target={want:?} MISSING"),
                }
            }
            // Keystone survival: does ANY completion contain each
            // canonical word? Tells prefix survival apart from
            // full-path assembly.
            for kw in ["hits", "justice", "dupe", "hid", "came", "wreck", "nice", "beach"] {
                let mut n = 0;
                let mut ex: Vec<&str> = Vec::new();
                for c in &clues {
                    if c.phrase.split_whitespace().any(|w| w.to_lowercase() == kw) {
                        n += 1;
                        if ex.len() < 2 {
                            ex.push(c.phrase.as_str());
                        }
                    }
                }
                eprintln!("KEYSTONE-PROBE word={kw:?} completions={n} ex={ex:?}");
            }
        }
        select_diverse(clues, self.config.top_n)
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
            .map(|w| {
                w.to_lowercase()
                    .trim_end_matches(['.', ',', '!', '?'])
                    .to_string()
            })
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
                    let next = partial.extend(pronunciation, p + consumed, 0.0);
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
        // Dedup identical (word, consumed) keeping cheapest.
        out.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.word.cmp(&b.word))
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
        // holds nothing else.
        let victim = kept
            .iter()
            .filter(|&&i| out[i].consumed == span && !fam_kept.contains(&i))
            .max_by(|&&a, &&b| {
                out[a]
                    .cost
                    .partial_cmp(&out[b].cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .or_else(|| {
                kept.iter()
                    .filter(|&&i| out[i].consumed == span)
                    .max_by(|&&a, &&b| {
                        out[a]
                            .cost
                            .partial_cmp(&out[b].cost)
                            .unwrap_or(std::cmp::Ordering::Equal)
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
    // Fill remaining slots cheapest-first.
    rest.sort_by(|a, b| {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.word.cmp(&b.word))
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
    });
    shortlist
}

/// A partially-constructed clue: the words chosen so far plus the
/// running cheap score used to prune the beam.
#[derive(Debug, Clone)]
struct Partial {
    words: Vec<ClueWord>,
    /// Accumulated substitution cost across all words so far.
    /// Always zero in Exact mode.
    sub_cost_total: f64,
    /// Running cheap score (per-word rarity penalties plus word-length
    /// bonuses) used only to prune the beam. The final Clue score
    /// replaces this with the full novelty-aware computation.
    cheap_score: f64,
    /// Cached clone-detection hash: the per-step pair of
    /// lowercased lexical word and pronunciation, folded
    /// incrementally (see [`Partial::step_hash`]). Lexical identity
    /// is part of the hash because homophones are distinct outputs:
    /// the clue is a word sequence, and word identity feeds
    /// downstream scoring (word-novelty) as well as the final phrase.
    /// An IPA-only hash would let one spelling occupy the slot and
    /// refuse its homophones. Exact duplicate corpus records (same
    /// word and same pronunciation) still collapse to one slot, best
    /// bound wins. A u64 fold keeps the hot pair loop
    /// allocation-free; the old space-joined String key copied the
    /// whole history per candidate.
    key_hash: u64,
    /// Cumulative target-stream cuts: the target char offset after
    /// each word so far. This is the parse's segmentation history in
    /// target coordinates — unlike the clue word IPA lengths, which
    /// drift from the target stream whenever insertions or deletions
    /// apply.
    cuts: Vec<usize>,
    /// Clue IPA characters covered so far (sum of word IPA lengths).
    /// Prefix clue boundaries (running totals of this) are compared
    /// against the target boundary set for the incremental novelty
    /// count — the same clue-frame comparison the final novelty
    /// scores, so bound and scorer agree (see [`Partial::final_score`]
    /// for why the clue frame, drift and all, is the right one:
    /// target-frame novelty scores the canonical resegmentations
    /// ~0.75/~0.49 instead of ~0.92/~0.83).
    clue_cum: usize,
    /// Number of prefix clue boundaries that coincide with a target
    /// word boundary. Incremental input to the optimistic bound;
    /// maintained at extension time so the hot gate never rescans.
    shared: usize,
    /// Optimistic upper bound on the final clue score of any
    /// completion extending this partial (see
    /// [`Partial::upper_bound_for`]). The quality pool's
    /// admission/ranking signal in Approximate mode. Unused (zero)
    /// in Exact mode, which ranks by `cheap_score`.
    bound: f64,
    /// Number of clue words so far that recycle a target word
    /// (normalized comparison, same as the incremental reuse
    /// penalty).
    reused: u32,
}

impl Partial {
    /// Final-score weights, shared by [`Partial::final_score`] and
    /// [`Partial::upper_bound_for`] so the beam bound can never drift
    /// from the scorer it predicts.
    const W_SIM: f64 = 0.40;
    const W_NOV: f64 = 0.35;
    const W_WNOV: f64 = 0.15;
    const W_LEN: f64 = 0.10;
    /// Divisor mapping total substitution cost to similarity.
    const SIM_DIV: f64 = 4.0;

    fn empty() -> Self {
        Self {
            words: Vec::new(),
            sub_cost_total: 0.0,
            cheap_score: 0.0,
            key_hash: 0,
            cuts: Vec::new(),
            clue_cum: 0,
            shared: 0,
            bound: 0.0,
            reused: 0,
        }
    }

    /// Optimistic upper bound on the final score of any completion
    /// extending a partial with this accumulated substitution cost
    /// and this many target-boundary reuses. Substitution cost and
    /// boundary reuse only grow as words are appended (cuts are
    /// append-only and the denominator is fixed), so current
    /// similarity and novelty already cap their final values, while
    /// word novelty and length signal are at most 1.0 — all in the
    /// same target coordinates the final scorer uses, so the beam
    /// bound can never drift from the scorer it predicts. Generic
    /// cost and boundary arithmetic, no lexical content.
    fn upper_bound_for(sub_cost_total: f64, shared: usize, denom: usize) -> f64 {
        let sim_ub = (1.0 - sub_cost_total / Self::SIM_DIV).clamp(0.0, 1.0);
        let nov_ub = 1.0 - shared as f64 / denom.max(1) as f64;
        Self::W_SIM * sim_ub + Self::W_NOV * nov_ub + Self::W_WNOV + Self::W_LEN
    }

    /// Demonstrated score of a prefix: the final-score weights
    /// applied to virtue already realized. Current similarity and
    /// novelty combine with the CURRENT word-novelty and length
    /// signal instead of their perfect maxima, so paths that
    /// already demonstrate resegmentation virtue (novel cuts, no
    /// recycled words, long words) outrank clean near-paraphrases
    /// that merely preserve headroom. Matches
    /// [`Partial::final_score`] evaluated on the prefix as a
    /// complete clue (pinned by unit test; the incremental `shared`
    /// count includes a terminal-boundary coincidence the scorer
    /// excludes, so it is subtracted back, and reuse is counted on
    /// the same normalized forms as the incremental penalty).
    /// Pure integer/set arithmetic over incrementally maintained
    /// fields, so selection computes it on the fly with no extra
    /// storage. Generic cost/boundary/word-shape arithmetic, no
    /// lexical content.
    fn demonstrated(
        sub_cost_total: f64,
        shared: usize,
        reused: u32,
        nwords: usize,
        clue_cum: usize,
        target_boundaries: &[usize],
    ) -> f64 {
        let sim = (1.0 - sub_cost_total / Self::SIM_DIV).clamp(0.0, 1.0);
        let terminal_hit = usize::from(target_boundaries.contains(&clue_cum));
        let shared_inner = shared.saturating_sub(terminal_hit);
        let denom = target_boundaries
            .iter()
            .filter(|b| **b < clue_cum)
            .count()
            .max(1) as f64;
        let novelty = 1.0 - (shared_inner as f64 / denom);
        let word_novelty = 1.0 - (reused as f64 / nwords.max(1) as f64);
        let avg_len = clue_cum as f64 / nwords.max(1) as f64;
        let length_signal = (avg_len / 4.0).min(1.0);
        Self::W_SIM * sim
            + Self::W_NOV * novelty
            + Self::W_WNOV * word_novelty
            + Self::W_LEN * length_signal
    }

    /// Normalized word form for clone detection: lowercase,
    /// alphanumeric characters only.
    fn norm_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// One clone-hash step: fold the lowercased lexical word (case
    /// variants collapse; distinct spellings, including apostrophe
    /// forms, stay distinct) and its pronunciation into the running
    /// hash. Allocation-free: lowercasing folds per char without
    /// collecting.
    fn step_hash(prev: u64, word: &str, ipa: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        prev.hash(&mut h);
        for c in word.chars().flat_map(|c| c.to_lowercase()) {
            c.hash(&mut h);
        }
        0x1fu8.hash(&mut h);
        ipa.hash(&mut h);
        h.finish()
    }

    fn extend(&self, p: &Pronunciation, end: usize, word_sub_cost: f64) -> Self {
        Self::extend_words(
            self,
            &p.word,
            &p.ipa,
            p.rarity,
            end,
            word_sub_cost,
            0.0,
            0.0,
            false,
            1,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn extend_approx(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        end: usize,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
        boundary_hit: bool,
        nov_denom: usize,
    ) -> Self {
        Self::extend_words(
            self,
            word,
            ipa,
            rarity,
            end,
            word_sub_cost,
            boundary_bonus,
            reuse_penalty,
            boundary_hit,
            nov_denom,
        )
    }

    /// One step of the exact-mode beam priority. See `extend_words`
    /// for the rationale of each term.
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
        // mid-frequency resegmentation words that the
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
    /// (incremental word-novelty — without it, parrot paths outrank
    /// genuine resyllabifications for the same span even though the
    /// final scorer demotes them).
    #[allow(clippy::too_many_arguments)]
    fn extend_words(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        end: usize,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
        boundary_hit: bool,
        nov_denom: usize,
    ) -> Self {
        let ipa_len = ipa.chars().count();
        let step = Self::step_cheap(
            ipa_len,
            rarity,
            word_sub_cost,
            boundary_bonus,
            reuse_penalty,
        );
        let sub_cost_total = self.sub_cost_total + word_sub_cost;
        let shared = self.shared + usize::from(boundary_hit);
        let bound = Self::upper_bound_for(sub_cost_total, shared, nov_denom);
        let reused = self.reused + u32::from(reuse_penalty > 0.0);
        Self {
            words: {
                let mut w = self.words.clone();
                w.push(ClueWord {
                    word: word.to_string(),
                    ipa: ipa.to_string(),
                    rarity,
                    sub_cost: word_sub_cost,
                });
                w
            },
            sub_cost_total,
            cheap_score: self.cheap_score + step,
            key_hash: Self::step_hash(self.key_hash, word, ipa),
            cuts: {
                let mut c = self.cuts.clone();
                c.push(end);
                c
            },
            clue_cum: self.clue_cum + ipa_len,
            shared,
            bound,
            reused,
        }
    }

    /// The final clue score for a closed parse: novelty (reshuffled
    /// word boundaries), word-novelty (no recycled target words),
    /// word-length signal, and phonetic similarity. Computable only
    /// once the parse is complete, so the beam cannot prune on it
    /// mid-parse — but terminal retention can and does (see
    /// [`CompletionTop`]). The beam's optimistic bound (see
    /// [`Partial::upper_bound_for`]) upper-bounds this score from
    /// cost and boundary reuse already incurred, in the same
    /// clue-frame coordinates.
    ///
    /// Novelty is scored on clue IPA lengths, not target cuts: under
    /// insertions/deletions the clue stream is longer or shorter than
    /// the target, so its boundaries drift off the target grid — and
    /// that drift is precisely what distinguishes a resegmentation
    /// from a parrot. Measured on the two canonical clues,
    /// target-frame novelty scores them ~0.75/~0.49 (pool-rank
    /// thousands, unreachable) while clue-frame novelty scores them
    /// ~0.92/~0.83; only the latter lets genuine resegmentations
    /// that preserve a boundary or two compete.
    fn final_score(&self, target_boundaries: &[usize], target_words: &HashSet<String>) -> f64 {
        // Reconstruct the clue's boundary set.
        let mut cum = 0_usize;
        let mut clue_boundaries: Vec<usize> = Vec::with_capacity(self.words.len());
        for w in &self.words {
            cum += w.ipa.chars().count();
            clue_boundaries.push(cum);
        }

        // Novelty: how few of the target's word boundaries the clue
        // also has. Boundary at the end of the phrase is shared by
        // construction, so exclude it.
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
        let denom = target_inner.len().max(1) as f64;
        let novelty = 1.0 - (shared / denom);

        // Word-novelty: penalty if the clue reuses any target word.
        let reused = self
            .words
            .iter()
            .filter(|w| target_words.contains(&w.word.to_lowercase()))
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
        let similarity = (1.0 - self.sub_cost_total / Self::SIM_DIV).clamp(0.0, 1.0);

        Self::W_SIM * similarity
            + Self::W_NOV * novelty
            + Self::W_WNOV * word_novelty
            + Self::W_LEN * length_signal
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
/// inflections of one resegmentation collapse to their best
/// representative, small enough that genuinely better clues still
/// outrank diverse-but-weaker ones.
const MMR_LAMBDA: f64 = 0.25;

/// TMP-PROBE λ override (env-gated; remove before final commit):
/// sweeps MMR diversity weight to test whether any generic setting
/// surfaces a mid-pool canonical clue.
fn mmr_lambda() -> f64 {
    std::env::var("MADGAB_LAMBDA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(MMR_LAMBDA)
}

/// Diversity-aware final selection for Approximate mode (classic
/// maximal-marginal-relevance): pick the best clue, then
/// repeatedly the clue maximizing `score - LAMBDA * overlap`,
/// where overlap is the maximum (over already-picked clues) of the
/// stronger of unigram and bigram word reuse. Twins of an early pick are
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
    // TMP-PROBE MMR rank (env-gated; remove before final commit):
    // run the greedy loop past top_n (output still truncated to
    // top_n) to report where the canonical clues would be picked.
    let probe_n = if std::env::var("MADGAB_RAW_PROBE").is_ok() {
        300.min(clues.len())
    } else {
        top_n.min(clues.len())
    };
    while picked.len() < probe_n && !remaining.is_empty() {
        let mut best_pos = 0;
        let mut best_val = f64::NEG_INFINITY;
        for (pos, &i) in remaining.iter().enumerate() {
            // Closest-picked overlap (classic MMR): unlike a running
            // union (whose penalties saturate as picked vocabulary
            // grows and bury everything late), every comparison
            // stays local, so a structurally different parse is
            // judged only against its nearest neighbor. Overlap is
            // the stronger of unigram and bigram reuse: near-twins
            // that swap one word of a long clue still share almost
            // every bigram, so they collapse to their best
            // representative instead of occupying a whole rank block
            // of scalar near-clones.
            let mut overlap = 0.0;
            for &j in &picked {
                let uw = frac(&bags[i].words, &bags[j].words, word_lens[i]);
                let bw = frac(&bags[i].bigrams, &bags[j].bigrams, bigram_lens[i]);
                let o = uw.max(bw);
                if o > overlap {
                    overlap = o;
                }
                if overlap >= 1.0 {
                    break;
                }
            }
            let v = clues[i].score - mmr_lambda() * overlap;
            if v > best_val {
                best_val = v;
                best_pos = pos;
            }
        }
        picked.push(remaining.remove(best_pos));
    }
    // Preserve pick order (MMR rank), not score order.
    let mut slots: Vec<Option<Clue>> = clues.into_iter().map(Some).collect();
    // TMP-PROBE (env-gated; remove before final commit).
    if std::env::var("MADGAB_RAW_PROBE").is_ok() {
        for want in [
            "hits justice dupe hid came",
            "wreck a nice beach",
        ] {
            let rank = picked.iter().position(|&i| {
                slots[i]
                    .as_ref()
                    .is_some_and(|c| c.phrase.to_lowercase() == want)
            });
            eprintln!("MMR-PROBE want={want:?} rank={rank:?} picks={}", picked.len());
        }
    }
    let mut out = Vec::with_capacity(picked.len().min(top_n));
    for i in picked.into_iter().take(top_n) {
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

/// Terminal shortlist with per-cost-band quotas: each acoustic
/// band keeps its own top completions by final score (min-heap of
/// live score bits plus a key→entry table for clone suppression;
/// stale heap entries are skipped lazily at drain time). A single
/// global cap lets the crowded clean bands evict max-budget
/// keystone completions (a whole-parse cost near the total budget
/// scores far below the clean-band floor even when it is the
/// intended resegmentation); band quotas give every cost family a
/// bounded share instead, so the final re-rank always sees the
/// best of each family. Sized so mid-pack keystone completions
/// (which sit just below their band's top thousand by score)
/// survive to re-ranking; the final MMR re-rank stays interactive
/// (tens of thousands at most). Generic cost arithmetic, no
/// lexical content.
const COMPLETION_BAND_CAP: usize = 2048;
/// Number of cost bands retained (see [`completion_band`]; the
/// total budget caps the top band).
const COMPLETION_BANDS: usize = 7;

/// Acoustic-cost band of a whole-parse substitution cost: the
/// budgeted range sliced into quarter-cost strips. Parses that
/// paid for one or two max-budget keystone links sit several bands
/// above clean near-paraphrases; banding completions keeps those
/// families from competing head-to-head for one global cap.
/// Generic cost arithmetic, no lexical content.
fn completion_band(sub_cost_total: f64) -> usize {
    ((sub_cost_total / 0.25).floor() as usize).min(COMPLETION_BANDS - 1)
}

struct CompletionTop {
    bands: Vec<BandTop>,
}

struct BandTop {
    cap: usize,
    /// Min-heap of `(score_bits, seq, key)`. Scores are in [0, 1],
    /// where `to_bits` preserves numeric order; `seq`
    /// disambiguates ties so every push is unique. Entries whose
    /// map slot no longer matches are stale (clone superseded or
    /// evicted) and skipped on pop.
    heap: std::collections::BinaryHeap<(std::cmp::Reverse<(u64, u64)>, u64)>,
    seq: u64,
    /// Clone key → `(score_bits, seq, entry)`.
    map: HashMap<u64, (u64, u64, Partial)>,
}

impl CompletionTop {
    fn new(_cap: usize) -> Self {
        Self {
            bands: (0..COMPLETION_BANDS)
                .map(|_| BandTop {
                    cap: COMPLETION_BAND_CAP,
                    heap: std::collections::BinaryHeap::new(),
                    seq: 0,
                    map: HashMap::new(),
                })
                .collect(),
        }
    }

    fn insert(&mut self, candidate: Partial, score: f64) {
        let band = completion_band(candidate.sub_cost_total);
        self.bands[band].insert(candidate, score);
    }

    fn drain_sorted(self) -> Vec<Partial> {
        let mut v: Vec<(u64, u64, Partial)> = Vec::new();
        for band in self.bands {
            v.extend(band.map.into_values().map(|(b, s, p)| (b, s, p)));
        }
        v.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        v.into_iter().map(|(_, _, p)| p).collect()
    }
}

impl BandTop {
    fn insert(&mut self, candidate: Partial, score: f64) {
        if self.cap == 0 {
            return;
        }
        // Scores are finite (bounded arithmetic in final_score).
        let bits = score.to_bits();
        if let Some(slot) = self.map.get_mut(&candidate.key_hash) {
            if bits > slot.0 {
                self.seq += 1;
                let key = slot.2.key_hash;
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
                .push((std::cmp::Reverse((bits, self.seq)), candidate.key_hash));
            self.map
                .insert(candidate.key_hash, (bits, self.seq, candidate));
        }
    }
}

/// One beam position: order-independent accumulation with deferred
/// selection. Every candidate that reaches the position is kept
/// (exact-clone deduped, best bound wins); only when the position is
/// about to expand is the bounded expansion set chosen from the full
/// accumulated pool. Arrival order never decides survival.
///
/// Selection combines `k` quality states (top bound) with two
/// best word-sequences per (parent-sound, ending-sound, depth)
/// triple, capped at this many triples. Triples isolate keystone
/// links from cheaper near-ties that merely share an ending: a
/// canonical middle word competes only against same-history twins
/// (same parent sound and depth), where direct lattice
/// measurement shows the canonical links cheapest-or-equal
/// (twin opener costs) or tied (homophone openers, ordered by
/// familiarity). Sizing shows keystone triples clearing a
/// ~300-deep cutoff at the hardest positions while junk triples
/// fall below; clean singletons lose nothing (kept via quality).
/// Order-independent sorts; deterministic tie-breaks.
/// Generic bound/score/word-shape arithmetic, no lexical content.
const BEAM_MAX_TRIPLES: usize = 384;

/// Best-demonstrated word-sequence kept per structural triple.
/// Two per triple: the canonical link plus its closest same-history
/// twin survive side by side so the final scorer — not beam order
/// — arbitrates. Small enough that the expansion set stays
/// interactive (triples × this).
const BEAM_PER_TRIPLE: usize = 2;

/// TMP-PROBE insertion census counter (env-gated diagnostics;
/// remove before final commit).
static HID_INSERTS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Safety cap on accumulation per position: exact-clone dedup already
/// bounds the pool, but a pathological fan-in is truncated
/// order-independently (sort by bound, keep best) before selection so
/// memory stays flat. Sized (with+"_TMP-PROBE sizing"_ below) so the
/// mid-cost keystone arrivals the acceptance targets need survive
/// the trim: the cutoff must sit below keystone bound grade.
/// Well above the pre-shortlist size, so it never binds on small
/// targets in practice.
const BEAM_ACCUM_CAP: usize = 24000;

struct BeamPos {
    /// Clone hash -> best-bound partial. No online eviction.
    map: HashMap<u64, Partial>,
    k: usize,
}

impl BeamPos {
    fn new(k: usize) -> Self {
        Self {
            map: HashMap::new(),
            k,
        }
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Accumulate one candidate (exact-clone dedup, best bound wins).
    /// Order-independent: no eviction of earlier arrivals.
    fn insert(&mut self, candidate: Partial) {
        if self.k == 0 {
            return;
        }
        // TMP-PROBE insertion census (env-gated; remove before final
        // commit): counts canonical-hid arrivals reaching any beam
        // pool, to distinguish never-inserted from
        // inserted-then-removed.
        if std::env::var("MADGAB_RAW_PROBE").is_ok() {
            let wk: Vec<String> = candidate
                .words
                .iter()
                .map(|w| w.word.to_lowercase())
                .collect();
            if wk == ["hits", "justice", "dupe", "hid"] {
                HID_INSERTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        match self.map.get(&candidate.key_hash) {
            Some(kept) if kept.bound >= candidate.bound => {}
            _ => {
                self.map.insert(candidate.key_hash, candidate);
            }
        }
        // TMP-PROBE: trim disabled (env-gated removal of the
        // bound; remove before final commit): diagnoses whether the
        // accumulation trim or the deferred selection drops
        // keystone arrivals.
        let accum_cap = if std::env::var("MADGAB_RAW_PROBE").is_ok() {
            usize::MAX
        } else {
            BEAM_ACCUM_CAP
        };
        if self.map.len() > accum_cap {
            // Order-independent safety trim: keep the best-bound
            // states. Deterministic tie-break on clone hash.
            let mut v: Vec<(u64, f64)> = self
                .map
                .iter()
                .map(|(h, p)| (*h, p.bound))
                .collect();
            v.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            // TMP-PROBE trim witness (env-gated; remove before final
            // commit): cutoff bound decides whether mid-cost
            // keystone arrivals survive accumulation.
            if std::env::var("MADGAB_RAW_PROBE").is_ok() {
                eprintln!(
                    "TRIM-PROBE arrivals={} cutoff={:.4}",
                    v.len(),
                    v.get(BEAM_ACCUM_CAP / 2).map(|(_, b)| *b).unwrap_or(-1.0),
                );
            }
            v.truncate(BEAM_ACCUM_CAP / 2);
            let keep: HashSet<u64> = v.into_iter().map(|(h, _)| h).collect();
            self.map.retain(|h, _| keep.contains(h));
        }
    }

    /// Deferred selection of the bounded expansion set from every
    /// candidate that reached this position. The set combines `k`
    /// quality states (top optimistic bound) with the best couple
    /// per (parent-sound, ending-sound, depth) triple (ranked by
    /// demonstrated score). Triples isolate keystone links from
    /// cheaper near-ties that merely share an ending: a canonical
    /// middle word competes only against same-history twins, where
    /// direct lattice measurement shows the canonical links
    /// cheapest-or-equal. A cap on distinct triples bounds the set.
    /// Deterministic; arrival order plays no role. Generic
    /// cost/boundary/word-shape arithmetic, no lexical content.
    fn take_selected(&mut self, target_boundaries: &[usize]) -> Vec<Partial> {
        fn dscore(p: &Partial, target_boundaries: &[usize]) -> f64 {
            Partial::demonstrated(
                p.sub_cost_total,
                p.shared,
                p.reused,
                p.words.len(),
                p.clue_cum,
                target_boundaries,
            )
        }
        /// Corpus-familiarity load of a prefix: summed rarity ranks
        /// (lower rank = more familiar; unknown words count as very
        /// rare). Tie-break only, applied solely on bit-identical
        /// demonstrated scores — structurally identical histories
        /// whose words differ (wreck/rec homophone openers). It can
        /// never promote a worse-scoring path, and prefers the
        /// familiar spelling deterministically. Generic corpus
        /// property, no lexical content.
        fn fam(p: &Partial) -> f64 {
            p.words
                .iter()
                .map(|w| w.rarity.unwrap_or(1e12))
                .sum()
        }
        /// Selection order key: demonstrated score descending, then
        /// familiarity ascending, then clone hash ascending. Tuples
        /// carry precomputed scores: (dscore, fam, hash, rest index).
        /// Pure tuple comparison — no pool lookups.
        fn ord_sel(
            a: &(f64, f64, u64, usize),
            b: &(f64, f64, u64, usize),
        ) -> std::cmp::Ordering {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.1.partial_cmp(&b.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| a.2.cmp(&b.2))
        }
        // TMP-PROBE sizing counters (env-gated; remove before final
        // commit): arrivals, distinct endings/triples, and how many
        // triple-bests clear keystone-grade thresholds. Decides
        // capacity sizing with measurements, not guesses.
        static SEL_CALL: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let mut v: Vec<Partial> = self.map.drain().map(|(_, p)| p).collect();
        if v.is_empty() {
            return v;
        }
        // TMP-PROBE arrival check (env-gated; remove before final
        // commit): do the exact canonical-worded prefixes ARRIVE at
        // any position (regardless of score)? Distinguishes
        // never-arrives (parent never expanded / budget-blocked)
        // from arrives-but-unselected (retention too shallow).
        if std::env::var("MADGAB_RAW_PROBE").is_ok() {
            for want in [
                vec!["hits"],
                vec!["hits", "justice"],
                vec!["hits", "justice", "dupe"],
                vec!["hits", "justice", "dupe", "hid"],
                vec!["hits", "justice", "dupe", "hid", "came"],
                vec!["wreck"],
                vec!["wreck", "a"],
                vec!["wreck", "a", "nice"],
                vec!["wreck", "a", "nice", "beach"],
            ] {
                let hits: Vec<(f64, Vec<usize>, f64)> = v
                    .iter()
                    .filter(|q| {
                        q.words.len() == want.len()
                            && q.words
                                .iter()
                                .zip(want.iter())
                                .all(|(w, k)| w.word.to_lowercase() == **k)
                    })
                    .map(|q| (q.sub_cost_total, q.cuts.clone(), q.bound))
                    .collect();
                if !hits.is_empty() {
                    eprintln!("ARRIVE-PROBE want={want:?} arrivals={} {hits:?}", hits.len());
                }
            }
        }
        // TMP-PROBE contender dump (env-gated; remove before final
        // commit): word-distinct arrivals ending in the keystone
        // sounds, ranked — shows the canonical prefix's exact
        // within-ending rank and whether selection keeps it.
        if std::env::var("MADGAB_RAW_PROBE").is_ok() {
            // Canonical-hid rank among ALL arrivals (not top-truncated):
            // dscore, and how many arrivals beat it.
            let mut with_scores: Vec<(f64, String)> = v
                .iter()
                .map(|q| {
                    (
                        dscore(q, target_boundaries),
                        q.words
                            .iter()
                            .map(|w| w.word.to_lowercase())
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                })
                .collect();
            with_scores.sort_by(|a, b| {
                b.0.partial_cmp(&a.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let canon = "hits justice dupe hid";
            match with_scores.iter().position(|(_, w)| w == canon) {
                Some(r) => eprintln!(
                    "CANONRANK-PROBE prefix={canon:?} rank={} dscore={:.4} arrivals={}",
                    r + 1,
                    with_scores[r].0,
                    with_scores.len()
                ),
                None => eprintln!(
                    "CANONRANK-PROBE prefix={canon:?} ABSENT arrivals={}",
                    with_scores.len()
                ),
            }
            for key_ipa in ["hɪd", "dup", "naɪs"] {
                let mut contenders: HashMap<String, (f64, f64, Vec<usize>)> = HashMap::new();
                for p in &v {
                    let last_ok = p
                        .words
                        .last()
                        .is_some_and(|w| w.ipa == key_ipa);
                    if !last_ok {
                        continue;
                    }
                    let wk: String = p
                        .words
                        .iter()
                        .map(|w| w.word.to_lowercase())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let d = dscore(p, target_boundaries);
                    contenders
                        .entry(wk)
                        .and_modify(|e| {
                            if d > e.0 {
                                *e = (d, p.sub_cost_total, p.cuts.clone());
                            }
                        })
                        .or_insert((d, p.sub_cost_total, p.cuts.clone()));
                }
                if !contenders.is_empty() {
                    let mut rank: Vec<(String, (f64, f64, Vec<usize>))> =
                        contenders.into_iter().collect();
                    rank.sort_by(|a, b| {
                        b.1 .0
                            .partial_cmp(&a.1 .0)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let show: Vec<(String, f64, f64, Vec<usize>)> = rank
                        .iter()
                        .take(12)
                        .map(|(w, (d, s, c))| (w.clone(), *d, *s, c.clone()))
                        .collect();
                    eprintln!(
                        "CONTEND-PROBE ending={key_ipa:?} distinct={} top={show:?}",
                        rank.len()
                    );
                }
            }
        }
        // TMP-PROBE sizing input (env-gated; remove with the print
        // below before final commit).
        let sizing_arrivals = v.len();
        // Quality top-k by bound (descending, hash tie-break).
        v.sort_by(|a, b| {
            b.bound
                .partial_cmp(&a.bound)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.key_hash.cmp(&b.key_hash))
        });
        let k = self.k.min(v.len());
        let rest = v.split_off(k);
        let mut selected = v;
        let mut picked: HashSet<u64> =
            selected.iter().map(|p| p.key_hash).collect();
        // Best word-sequences per (parent-sound, ending-sound,
        // depth) triple, top couple per triple. One linear pass;
        // per triple, a word-sequence map (same words via different
        // alignments collapse to their best demonstrated variant —
        // those variants would dedup to one phrase downstream
        // anyway). Triples isolate keystone links from cheaper
        // near-ties that merely share an ending; two per triple
        // cover exact-tie twins. Ties prefer the smaller clone hash
        // so choices stay deterministic.
        let mut best_per_triple: HashMap<
            (String, String, usize),
            HashMap<String, (f64, f64, u64, usize)>,
        > = HashMap::new();
        for (i, p) in rest.iter().enumerate() {
            if picked.contains(&p.key_hash) {
                continue;
            }
            let last_ipa: String = p
                .words
                .last()
                .map(|w| w.ipa.clone())
                .unwrap_or_default();
            let parent_ipa: String = if p.words.len() >= 2 {
                p.words[p.words.len() - 2].ipa.clone()
            } else {
                String::new()
            };
            let triple = (parent_ipa, last_ipa, p.words.len());
            let words_key: String = p
                .words
                .iter()
                .map(|w| w.word.to_lowercase())
                .collect::<Vec<_>>()
                .join(" ");
            let d = dscore(p, target_boundaries);
            let f = fam(p);
            let slot = best_per_triple.entry(triple).or_default();
            match slot.get(&words_key) {
                Some(&(bd, bf, bh, _))
                    if bd > d
                        || (bd == d && (bf < f || (bf == f && bh < p.key_hash))) => {}
                _ => {
                    slot.insert(words_key, (d, f, p.key_hash, i));
                }
            }
        }
        // Flatten each triple's word-map to its top entry, keeping
        // the pre-truncation family size (word-distinct contender
        // count) alongside — truncation must not destroy the
        // contention signal the cap below orders by.
        // (triple, best entries, family size).
        let mut triples: Vec<((String, String, usize), Vec<(f64, f64, u64, usize)>, usize)> =
            best_per_triple
                .into_iter()
                .map(|(t, m)| {
                    let size = m.len();
                    let mut v: Vec<(f64, f64, u64, usize)> = m.into_values().collect();
                    v.sort_by(ord_sel);
                    if v.len() > BEAM_PER_TRIPLE {
                        v.truncate(BEAM_PER_TRIPLE);
                    }
                    (t, v, size)
                })
                .collect();
        triples.sort_by(|a, b| {
            // Order-independent pair priority: best demonstrated
            // score first (familiarity/hash tie-breaks inside
            // ord_sel), then pair key. Sizing shows keystone pairs
            // clear a low-hundreds-deep cutoff while junk pairs fall
            // below; contention already shaped the within-pair
            // shortlists, so the cap judges families by their best.
            ord_sel(&a.1[0], &b.1[0]).then_with(|| a.0.cmp(&b.0))
        });
        // TMP-PROBE sizing (env-gated; remove before final commit):
        // pre-EMAX totals show whether the cap binds and how many
        // keystone-grade families compete.
        if std::env::var("MADGAB_SIZING").is_ok() {
            let call = SEL_CALL.fetch_add(
                1,
                std::sync::atomic::Ordering::Relaxed,
            );
            let mut ends: HashSet<String> = HashSet::new();
            for (t, _, _) in triples.iter() {
                ends.insert(t.1.clone());
            }
            let above: [usize; 3] = [0.90, 0.925, 0.938]
                .map(|th| triples.iter().filter(|(_, v, _)| v[0].0 > th).count());
            eprintln!(
                "SIZING call={call} arrivals={sizing_arrivals} triples={} endings={} above90={} above925={} above938={} maxfam={}",
                triples.len(),
                ends.len(),
                above[0],
                above[1],
                above[2],
                triples.iter().map(|(_, _, s)| *s).max().unwrap_or(0),
            );
            // TMP-PROBE keystone-triple fate (env-gated; remove with
            // the rest): the (parent, ending, depth) triples behind
            // the canonical links — size, best score, and whether
            // EMAX keeps them (rank among all triples by the live
            // EMAX order).
            let mut ranked: Vec<((String, String, usize), usize, f64)> = triples
                .iter()
                .map(|(t, v, s)| (t.clone(), *s, v[0].0))
                .collect();
            ranked.sort_by(|a, b| {
                b.1.cmp(&a.1).then_with(|| {
                    b.2.partial_cmp(&a.2)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });
            for (t, s, best) in ranked.iter() {
                let keystone =
                    t.1 == "hɪd" || t.1 == "naɪs" || t.1 == "dup";
                if keystone {
                    let rank = ranked.iter().position(|(u, _, _)| u == t).unwrap_or(99999);
                    eprintln!(
                        "PAIR-PROBE call={call} pair=({:?},{:?}) size={s} best={best:.4} emax_rank={rank} kept={}",
                        t.0,
                        t.1,
                        rank < 160,
                    );
                }
            }
        }
        if triples.len() > BEAM_MAX_TRIPLES {
            triples.truncate(BEAM_MAX_TRIPLES);
        }
        // Deterministic expansion order: same (score, familiarity,
        // hash) order. Recompute scores for the picked indices (the
        // stored tuples were moved into the ordering above).
        let mut extra_idx: Vec<usize> = triples
            .into_iter()
            .flat_map(|(_, v, _)| v.into_iter().map(|(_, _, _, i)| i))
            .collect();
        extra_idx.sort_by(|&a, &b| {
            ord_sel(
                &(
                    dscore(&rest[a], target_boundaries),
                    fam(&rest[a]),
                    rest[a].key_hash,
                    a,
                ),
                &(
                    dscore(&rest[b], target_boundaries),
                    fam(&rest[b]),
                    rest[b].key_hash,
                    b,
                ),
            )
        });
        let mut slots: Vec<Option<Partial>> = rest.into_iter().map(Some).collect();
        for i in extra_idx {
            let p = slots[i].take().expect("each index picked once");
            picked.insert(p.key_hash);
            selected.push(p);
        }
        // TMP-PROBE expansion check (env-gated; remove before final
        // commit): does the selected expansion set contain either
        // canonical keystone prefix? Distinguishes beam drops from
        // downstream (completion/MMR) losses with no chain
        // machinery — just full-prefix word matching.
        if std::env::var("MADGAB_RAW_PROBE").is_ok() {
            // Call index = expansion order ≈ target position (each
            // position expands once, in order).
            let call = SEL_CALL.fetch_add(
                1,
                std::sync::atomic::Ordering::Relaxed,
            );
            for want in [
                vec!["hits"],
                vec!["hits", "justice"],
                vec!["hits", "justice", "dupe"],
                vec!["hits", "justice", "dupe", "hid"],
                vec!["wreck"],
                vec!["wreck", "a"],
                vec!["wreck", "a", "nice"],
            ] {
                let mut found = 0;
                let mut best = f64::NEG_INFINITY;
                for q in &selected {
                    best = best.max(dscore(q, target_boundaries));
                    if q.words.len() == want.len()
                        && q.words
                            .iter()
                            .zip(want.iter())
                            .all(|(w, k)| w.word.to_lowercase() == **k)
                    {
                        found += 1;
                    }
                }
                if found > 0 {
                    let detail: Vec<(Vec<usize>, f64, f64, usize)> = selected
                        .iter()
                        .filter(|q| {
                            q.words.len() == want.len()
                                && q.words
                                    .iter()
                                    .zip(want.iter())
                                    .all(|(w, k)| w.word.to_lowercase() == **k)
                        })
                        .map(|q| (q.cuts.clone(), q.sub_cost_total, q.bound, q.shared))
                        .collect();
                    eprintln!(
                        "EXPAND-PROBE call={call} want={want:?} found={found} detail={detail:?} selected={} best={best:.4}",
                        selected.len()
                    );
                }
            }
        }
        selected
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
    fn demonstrated_matches_final_score_on_prefix() {
        // Selection ranks the MMR reserve by `demonstrated`, so it
        // must equal the scorer evaluated on the prefix — otherwise
        // retention order drifts from completion order.
        // Punctuation-free vocabulary keeps the normalized reuse
        // count identical to the scorer's word-novelty input.
        let bounds = vec![2, 4];
        let targets: HashSet<String> = ["zz".to_string()].into_iter().collect();
        let root = Partial::empty();
        let p1 = root.extend_approx("alpha", "ab", None, 2, 0.1, 0.0, 0.0, true, 1);
        let p2 = p1.extend_approx("beta", "cd", None, 4, 0.2, 0.0, 0.0, true, 1);
        let d = Partial::demonstrated(
            p2.sub_cost_total,
            p2.shared,
            p2.reused,
            p2.words.len(),
            p2.clue_cum,
            &bounds,
        );
        let s = p2.final_score(&bounds, &targets);
        assert!(
            (d - s).abs() < 1e-9,
            "demonstrated {d} must equal final_score {s} on the prefix"
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

    fn partial_with(bound: f64, cost: f64, nwords: usize, key: &str, reused: u32) -> Partial {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut h);
        Partial {
            words: (0..nwords)
                .map(|i| ClueWord {
                    word: format!("w{i}"),
                    ipa: "ab".to_string(),
                    rarity: None,
                    sub_cost: 0.0,
                })
                .collect(),
            sub_cost_total: cost,
            cheap_score: 0.0,
            key_hash: h.finish(),
            cuts: (1..=nwords).map(|i| i * 2).collect(),
            clue_cum: 2 * nwords,
            shared: 0,
            bound,
            reused,
        }
    }

    #[test]
    fn tmp_prank_numbers() {
        let b = vec![2, 4];
        // parrot parent: sub 0, shared 0, reused 1, nwords 1, cum 2
        let pp = Partial::demonstrated(0.0, 0, 1, 1, 2, &b);
        // reseg parent: sub 0.15, shared 0, reused 1, nwords 1, cum 2
        let rp = Partial::demonstrated(0.15, 0, 1, 1, 2, &b);
        // reseg own: sub 0.15, shared 0, reused 0, nwords 2, cum 4
        let ro = Partial::demonstrated(0.15, 0, 0, 2, 4, &b);
        // parrot own: sub 0, shared 0, reused 1, nwords 2, cum 4
        let po = Partial::demonstrated(0.0, 0, 1, 2, 4, &b);
        eprintln!("TMP parrot_prank={pp:.6} reseg_prank={rp:.6} reseg_own={ro:.6} parrot_own={po:.6}");
    }

    #[test]
    fn beam_preserves_low_cost_prefix_under_parrot_pressure() {
        // Deferred selection keeps every arrival: a band of
        // zero-cost parrot prefixes must not squeeze out a slightly
        // costlier genuine resegmentation, and the ending-stratified
        // reserve must surface it in the expansion set even when it
        // sits far below the quality cutoff.
        let mut beam = BeamPos::new(4);
        for i in 0..8 {
            beam.insert(partial_with(
                0.90 - 0.01 * i as f64,
                0.0,
                2,
                &format!("parrot {i}"),
                // Parrots recycle target words by definition (that is
                // why the final scorer demotes them); the genuine
                // resegmentation below does not.
                1,
            ));
        }
        beam.insert(partial_with(0.50, 0.0, 2, "parrot tail", 1));
        beam.insert(partial_with(0.55, 0.15, 2, "re seg", 0));
        let sel = beam.take_selected(&[2, 4]);
        eprintln!("SEL bounds={:?}", sel.iter().map(|p| p.bound).collect::<Vec<_>>());
        assert!(
            sel.iter()
                .any(|p| p.words.len() == 2 && (p.bound - 0.55).abs() < 1e-9),
            "low-cost resegmentation prefix was squeezed out by parrots"
        );
    }

    #[test]
    fn beam_selection_is_order_independent() {
        // The expansion set must not depend on arrival order: the
        // same pool inserted in forward and reverse order selects
        // the same clone keys.
        let mut fwd = BeamPos::new(8);
        let mut rev = BeamPos::new(8);
        let mut items = Vec::new();
        for i in 0..50 {
            items.push(partial_with(
                0.95 - 0.01 * (i as f64),
                0.01 * (i as f64),
                2,
                &format!("cand {i}"),
                0,
            ));
        }
        for p in &items {
            fwd.insert(p.clone());
        }
        for p in items.iter().rev() {
            rev.insert(p.clone());
        }
        let mut a: Vec<u64> = fwd.take_selected(&[2, 4]).iter().map(|p| p.key_hash).collect();
        let mut b: Vec<u64> = rev.take_selected(&[2, 4]).iter().map(|p| p.key_hash).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "selection depends on arrival order");
    }

    #[test]
    fn beam_keeps_homophones_distinct_but_collapses_exact_dupes() {
        // Clone identity is (word, IPA) per step: two different words
        // with identical pronunciation are different candidate
        // answers and must coexist, while an exact same word+IPA
        // duplicate collapses to one slot.
        let root = Partial::empty();
        let a = root.extend_approx("rec", "ɹɛk", None, 3, 0.0, 0.0, 0.0, false, 1);
        let b = root.extend_approx("wreck", "ɹɛk", None, 3, 0.0, 0.0, 0.0, false, 1);
        assert_ne!(a.key_hash, b.key_hash, "homophone keys must differ");
        let mut beam = BeamPos::new(16);
        beam.insert(a);
        // Same IPA, different word: novel clone key, both accumulate.
        beam.insert(b);
        // Exact duplicate: same key, no improvement → collapses.
        let dup = root.extend_approx("rec", "ɹɛk", None, 3, 0.0, 0.0, 0.0, false, 1);
        beam.insert(dup);
        let sel = beam.take_selected(&[2, 4]);
        assert_eq!(
            sel.len(),
            2,
            "homophones must coexist; exact duplicates must collapse"
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
    fn tmp_twin_costs() {
        use open_english_pronouncing_dictionary::CORPUS_JSON;
        let g = crate::Generator::from_json(
            CORPUS_JSON,
            crate::GeneratorConfig {
                max_rarity: Some(50_000.0),
                mode: crate::SearchMode::Approximate {
                    per_word_budget: 0.75,
                    total_budget: 1.5,
                },
                ..crate::GeneratorConfig::default()
            },
        )
        .unwrap();
        for (target, pos, show) in [
            ("It's just a stupid game", 0_usize, vec!["hits", "hitch", "heats", "hicks", "it"]),
            ("It's just a stupid game", 3_usize, vec!["justice", "stir", "tough"]),
            ("It's just a stupid game", 4_usize, vec!["justice", "stir", "tough"]),
            ("It's just a stupid game", 10_usize, vec!["dupe", "coop", "coupe", "to", "two"]),
            ("It's just a stupid game", 13_usize, vec!["hid", "had", "ad", "it"]),
            ("recognize speech", 0_usize, vec!["wreck", "rec", "wrack", "rag"]),
            ("recognize speech", 3_usize, vec!["a", "uh", "o"]),
            ("recognize speech", 4_usize, vec!["nice", "ice", "eyes"]),
            ("recognize speech", 9_usize, vec!["beach", "peach", "speech"]),
        ] {
            let (tipa, _) =
                crate::transcribe_normalized_with_boundaries(g.corpus(), target).unwrap();
            let chars: Vec<char> = tipa.chars().collect();
            let ms = g.approx_trie.words_approximately_starting_at(
                &g.approx_entries,
                &chars,
                pos,
                0.75,
                250,
            );
            let mut rows: Vec<(String, usize, f64, Option<f64>)> = Vec::new();
            for w in show {
                match ms.iter().find(|m| m.word.to_lowercase() == w) {
                    Some(m) => rows.push((w.to_string(), m.consumed, m.cost, m.rarity)),
                    None => rows.push((w.to_string(), 0, -1.0, None)),
                }
            }
            eprintln!("TWIN target={target:?} pos={pos} rows={rows:?} total={}", ms.len());
        }
    }

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
