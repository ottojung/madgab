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

use std::collections::BTreeMap;
use std::hash::{BuildHasher, Hasher};

use phonetics::transcriptions::{Corpus, Pronunciation};
use serde::Serialize;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Deterministic 64-bit FNV-1a hasher (fixed offset basis, no
/// per-process seed): all hash maps/sets in the search use it so
/// results never depend on `RandomState` seeding. Iteration order
/// over hash maps is still unspecified, so every order-sensitive
/// decision additionally breaks ties by content (total orders) —
/// but the fixed seed removes the largest nondeterminism source
/// and makes runs reproducible bit-for-bit.
#[derive(Clone, Default)]
struct FnvBuild;
struct FnvHasher(u64);
const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
impl Hasher for FnvHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}
impl BuildHasher for FnvBuild {
    type Hasher = FnvHasher;
    fn build_hasher(&self) -> FnvHasher {
        FnvHasher(FNV_OFFSET)
    }
}
/// Hash map/sets with deterministic iteration seeds. Prefer these
/// over `std` maps everywhere in the search.
type FastMap<K, V> = std::collections::HashMap<K, V, FnvBuild>;
type FastSet<K> = std::collections::HashSet<K, FnvBuild>;

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
    /// Number of clue candidates kept per beam position in Exact
    /// mode. Higher = better quality but quadratically more memory
    /// and time. Approximate mode instead retains per-hypothesis-
    /// family floors (see `BeamPos`), independent of this setting.
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
        let mut recovery = self.generate_once(target, false);
        if matches!(self.config.mode, SearchMode::Approximate { .. }) {
            recovery.extend(self.recover_approx(target));
        }
        recovery.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        recovery.dedup_by(|a, b| a.phrase == b.phrase);
        select_diverse(recovery, self.config.top_n)
    }

    fn recover_approx(&self, target: &str) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_normalized_with_boundaries(&self.corpus, target)
        else {
            return Vec::new();
        };
        let target_words: FastSet<String> = target
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
        let (per_word_budget, total_budget) = match self.config.mode {
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => (per_word_budget, total_budget),
            SearchMode::Exact => return Vec::new(),
        };
        let mut states: Vec<Vec<Vec<Partial>>> = (0..=n)
            .map(|_| (0..8).map(|_| Vec::new()).collect())
            .collect();
        states[0].push(vec![Partial::empty()]);
        let mut completed = Vec::new();
        for p in 0..n {
            let matches = self.approx_trie.words_approximately_starting_at(
                &self.approx_entries,
                &chars,
                p,
                per_word_budget,
                500,
            );
            let current = states[p].clone();
            for by_words in &current {
                for partial in by_words {
                    for m in &matches {
                        if m.word_len < self.config.min_word_ipa_chars
                            || m.consumed < self.config.min_word_ipa_chars
                            || partial.sub_cost_total + m.cost > total_budget + 1e-9
                        {
                            continue;
                        }
                        let norm = Partial::norm_word(&m.word);
                        let stem = Partial::stem_word(&norm);
                        let reuse = if target_words.iter().any(|t| {
                            Partial::stems_match(&stem, &Partial::stem_word(&Partial::norm_word(t)))
                        }) {
                            0.10
                        } else {
                            0.0
                        };
                        let next = partial.extend_approx(
                            &m.word, &m.ipa, m.rarity, m.consumed, m.cost, 0.0, reuse,
                        );
                        let end = p + m.consumed;
                        if end == n {
                            let score = next.final_score(&target_boundaries, &target_words);
                            completed.push((next, score));
                        } else if end < n {
                            let words = next.words.len();
                            let cell = &mut states[end][words.min(7)];
                            cell.push(next);
                            cell.sort_by(|a, b| {
                                b.recovery_score(&target_boundaries)
                                    .partial_cmp(&a.recovery_score(&target_boundaries))
                                    .unwrap_or(std::cmp::Ordering::Equal)
                                    .then_with(|| a.key.cmp(&b.key))
                            });
                            cell.truncate(128);
                        }
                    }
                }
            }
            for words in &mut states[p] {
                words.sort_by(|a, b| {
                    b.recovery_score(&target_boundaries)
                        .partial_cmp(&a.recovery_score(&target_boundaries))
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.key.cmp(&b.key))
                });
                words.truncate(128);
            }
        }
        completed.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.key.cmp(&b.0.key))
        });
        completed.dedup_by(|a, b| a.0.key == b.0.key && a.0.lex == b.0.lex);
        if let Ok(phrase) = std::env::var("MADGAB_TRACE_PHRASE") {
            let needle = phrase.to_lowercase();
            if let Some((rank, (partial, score))) =
                completed.iter().enumerate().find(|(_, (partial, _))| {
                    partial
                        .words
                        .iter()
                        .map(|w| w.word.to_lowercase())
                        .collect::<Vec<_>>()
                        .join(" ")
                        == needle
                })
            {
                eprintln!(
                    "madgab recovery phrase={phrase:?} rank={rank} score={score:.6} words={}",
                    partial.words.len()
                );
            } else {
                eprintln!("madgab recovery phrase={phrase:?} rank=absent");
            }
        }
        completed
            .into_iter()
            .take(self.config.top_n.saturating_mul(64).max(8192))
            .map(|(p, _)| p.into_clue(&target_ipa, &target_boundaries, &target_words))
            .collect()
    }

    fn generate_once(&self, target: &str, _recovery: bool) -> Vec<Clue> {
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
        let target_words: FastSet<String> = target
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_end_matches(['.', ',', '!', '?'])
                    .to_string()
            })
            .collect();
        // Stemmed target vocabulary for the incremental
        // word-novelty penalty: "it's" and "its" count as the same
        // recycled word, as do "recognize" and "recognizes" (see
        // `Partial::stem_word`). Without stemming, inflected
        // parrot paths dodge the penalty in transit even though the
        // final scorer demotes them.
        let target_words_stem: FastSet<String> = target
            .split_whitespace()
            .map(|w| {
                Partial::stem_word(&Partial::norm_word(
                    w.to_lowercase().trim_end_matches(['.', ',', '!', '?']),
                ))
            })
            .collect();
        let chars: Vec<char> = target_ipa.chars().collect();
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        // beam[p] = deferred coverings of [0..p). Each position
        // holds per-hypothesis-family floors (see `BeamPos`):
        // every (segmentation depth, acoustic tier, last-word
        // sound) family keeps its best prefixes, and contributes
        // its best to expansion — deterministically and independent
        // of arrival order, so mid-pack genuine resegmentations
        // survive alongside hundreds of near-tie rivals.
        let mut beam: Vec<BeamPos> = (0..=n).map(|_| BeamPos::new()).collect();
        beam[0].insert(Partial::empty());
        // Terminal retention is by final score within per-word-count
        // strata (see CompletionTop): every distinct closed parse
        // contends, then the pool is sorted, deduped and
        // diversity-selected to top_n.
        let completion_cap = self.config.top_n.saturating_mul(64).max(8192);
        let mut completed = CompletionTop::new(completion_cap);

        let (per_word_budget, total_budget) = match self.config.mode {
            SearchMode::Approximate {
                per_word_budget,
                total_budget,
            } => (per_word_budget, total_budget),
            SearchMode::Exact => unreachable!(),
        };

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
            // Per-match step constants: everything about a step's
            // cheap contribution is partial-independent, so
            // precompute once per position with the same helper the
            // beam priority uses (no drift between gate and extend).
            // Candidate cheap = partial.cheap + step: a few flops, no
            // allocation, and the gate usually skips the clone.
            let mut mconst: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(matches.len());
            // Normalized + stemmed match words, once per position:
            // the gates need them allocation-free on the hot pair
            // path (beam clone gate uses norms, terminal pre-gate
            // uses stems).
            let mnorm: Vec<String> = matches
                .iter()
                .map(|m| Partial::norm_word(&m.word))
                .collect();
            let mstem: Vec<String> = mnorm.iter().map(|w| Partial::stem_word(w)).collect();
            for m in &matches {
                // Resyllabification lean, DISABLED (0.0): a per-link
                // bonus for word ends that move away from target
                // boundaries was meant as incremental novelty, but
                // measurement showed it punishing genuine links that
                // land on target boundaries (canonical
                // resegmentations reuse some boundaries — final
                // novelty only needs *some* reshuffle) while
                // subsidizing over-segmented salads (+bonus per extra
                // word). Boundary luck must never outweigh acoustic
                // and lexical quality; novelty is judged globally by
                // the final scorer instead. Anti-parrot pressure comes
                // from the reuse penalty.
                let boundary_bonus = 0.0;
                let reuse_penalty = if target_words_stem.iter().any(|t| {
                    Partial::stems_match(&Partial::stem_word(&Partial::norm_word(&m.word)), t)
                }) {
                    0.10
                } else {
                    0.0
                };
                mconst.push((
                    Partial::step_cheap(
                        m.consumed,
                        m.rarity,
                        m.cost,
                        boundary_bonus,
                        reuse_penalty,
                    ),
                    m.cost,
                    boundary_bonus,
                    reuse_penalty,
                ));
            }
            // Expansion shortlist for position p (each position
            // expands exactly once): the closing zone expands the
            // whole pool (exact final-score arbitration replaces
            // heuristic selection there); elsewhere the per-family
            // floors decide.
            let closing = n - p <= CLOSING_SPAN;
            let here = if closing {
                beam[p].take_all()
            } else {
                beam[p].take_selected()
            };
            for partial in &here {
                let remaining_budget = total_budget - partial.sub_cost_total;
                for (mi, m) in matches.iter().enumerate() {
                    if m.cost > remaining_budget + 1e-9 {
                        continue;
                    }
                    let (c_cheap, m_cost, boundary_bonus, reuse_penalty) = mconst[mi];
                    let cand_sub = partial.sub_cost_total + m_cost;
                    if cand_sub > total_budget + 1e-9 {
                        continue;
                    }
                    let end = p + m.consumed;
                    let terminal = end == n;
                    // Lazy gate first (allocation-free refusal path).
                    // Terminal parses skip it: they are retained by
                    // final score below, and the heuristic gate cannot
                    // see closing novelty.
                    if !terminal {
                        let cand_cheap = partial.cheap_score + c_cheap;
                        if !beam[end].would_admit(
                            partial,
                            &m.ipa,
                            &mnorm[mi],
                            partial.words.len() + 1,
                            cand_sub,
                            cand_cheap,
                        ) {
                            continue;
                        }
                    }
                    if terminal {
                        // Terminal pre-score: the exact final score
                        // without building the extended partial. The
                        // family floor usually refuses it (most
                        // single-character closings lose), skipping
                        // the clone, full score, and insert.
                        let score = partial.terminal_score_exact(
                            &target_boundaries,
                            m.ipa.chars().count(),
                            m.rarity,
                            reuse_penalty,
                            m.cost,
                        );
                        if !completed.would_admit_ub(partial.words.len() + 1, &mstem[mi], score) {
                            continue;
                        }
                        let next = partial.extend_approx(
                            &m.word,
                            &m.ipa,
                            m.rarity,
                            m.consumed,
                            m.cost,
                            boundary_bonus,
                            reuse_penalty,
                        );
                        // The pre-score is bit-identical to scoring
                        // the built partial; keep the single source
                        // of truth by scoring once via the pre-score
                        // path (verified by
                        // `terminal_score_matches_final_score`).
                        completed.insert(next, score);
                    } else {
                        let next = partial.extend_approx(
                            &m.word,
                            &m.ipa,
                            m.rarity,
                            m.consumed,
                            m.cost,
                            boundary_bonus,
                            reuse_penalty,
                        );
                        beam[end].insert(next);
                    }
                }
            }
        }

        let completed_sorted = completed.drain_sorted();
        let mut clues: Vec<Clue> = completed_sorted
            .into_iter()
            .map(|p| p.into_clue(&target_ipa, &target_boundaries, &target_words))
            .collect();
        clues.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        clues.dedup_by(|a, b| a.phrase == b.phrase);
        select_diverse(clues, self.config.top_n)
    }

    /// Historical exact tiling over raw (stress-bearing) IPA.
    fn generate_exact(&self, target: &str) -> Vec<Clue> {
        let Some((target_ipa, target_boundaries)) =
            transcribe_with_boundaries(&self.corpus, target)
        else {
            return Vec::new();
        };
        let target_words: FastSet<String> = target
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
        let raw: FastMap<String, RawApproxEntry> = match serde_json::from_str(json) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let mut out: Vec<Self> = Vec::new();
        let mut seen: FastSet<(String, String)> = FastSet::default();
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
    ipa: FastMap<String, String>,
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
    dist_cache: std::cell::RefCell<FastMap<(char, char), f64>>,
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
            dist_cache: std::cell::RefCell::new(FastMap::default()),
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
        let mut best: FastMap<(usize, usize), f64> = FastMap::default();
        // (entry_idx, consumed) -> best cost; one row per alignment.
        let mut hits: FastMap<(usize, usize), f64> = FastMap::default();
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
        // Full total order (cost, word, consumed, IPA, rarity): the
        // input comes from a HashMap, so any residual tie must break
        // deterministically or downstream results vary run to run.
        out.sort_by(|a, b| {
            a.cost
                .partial_cmp(&b.cost)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.word.cmp(&b.word))
                .then_with(|| a.consumed.cmp(&b.consumed))
                .then_with(|| a.ipa.cmp(&b.ipa))
                .then_with(|| {
                    a.rarity
                        .unwrap_or(f64::INFINITY)
                        .partial_cmp(&b.rarity.unwrap_or(f64::INFINITY))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        let mut seen: FastSet<(String, usize)> = FastSet::default();
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
    /// Total order over matches: `out` arrives in HashMap order, so
    /// any residual tie would otherwise leak run-to-run
    /// nondeterminism into the search. Cost leads (cheapest wins
    /// the shortlist); rarity closes the order so even
    /// same-spelling pronunciation variants compare
    /// deterministically.
    fn match_ord(a: &ApproxMatch, b: &ApproxMatch) -> std::cmp::Ordering {
        a.cost
            .partial_cmp(&b.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.word.cmp(&b.word))
            .then_with(|| a.consumed.cmp(&b.consumed))
            .then_with(|| a.ipa.cmp(&b.ipa))
            .then_with(|| {
                familiarity(a)
                    .partial_cmp(&familiarity(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
    let mut by_span: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, m) in out.iter().enumerate() {
        by_span.entry(m.consumed).or_default().push(i);
    }
    const PER_SPAN_COST: usize = 32;
    const PER_SPAN_FAM: usize = 32;
    let mut kept: FastSet<usize> = FastSet::default();
    // Familiarity-axis picks: exempt from over-cap trimming
    // below (trimming them would defeat the union).
    let mut fam_kept: FastSet<usize> = FastSet::default();
    for idxs in by_span.values() {
        // Total-order tiebreaks everywhere below: `out` arrives in
        // HashMap order, so any residual tie would otherwise leak
        // run-to-run nondeterminism into the search.
        let mut by_cost = idxs.clone();
        by_cost.sort_by(|&a, &b| match_ord(&out[a], &out[b]));
        let mut by_fam = idxs.clone();
        by_fam.sort_by(|&a, &b| {
            familiarity(&out[a])
                .partial_cmp(&familiarity(&out[b]))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| match_ord(&out[a], &out[b]))
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
        // holds nothing else. Total order: `kept` iterates in
        // HashMap order, so cost-only comparison would pick
        // nondeterministically among ties.
        let victim = kept
            .iter()
            .filter(|&&i| out[i].consumed == span && !fam_kept.contains(&i))
            .max_by(|&&a, &&b| match_ord(&out[a], &out[b]))
            .or_else(|| {
                kept.iter()
                    .filter(|&&i| out[i].consumed == span)
                    .max_by(|&&a, &&b| match_ord(&out[a], &out[b]))
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
    let mut kept_set: FastSet<usize> = kept_idx.iter().copied().collect();
    let mut rest: Vec<ApproxMatch> = Vec::new();
    for (i, m) in out.into_iter().enumerate() {
        if kept_set.remove(&i) {
            shortlist.push(m);
        } else {
            rest.push(m);
        }
    }
    // Fill remaining slots cheapest-first.
    rest.sort_by(match_ord);
    for m in rest {
        if shortlist.len() >= cap {
            break;
        }
        shortlist.push(m);
    }
    // Deterministic order for downstream iteration.
    shortlist.sort_by(match_ord);
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
    /// Running cheap score (per-word rarity penalties plus word-length
    /// bonuses) used only to prune the beam. The final Clue score
    /// replaces this with the full novelty-aware computation.
    cheap_score: f64,
    /// Cached clone-detection key: the space-joined stripped IPA of
    /// the words so far. The acoustic half of the dedup identity
    /// (see `lex`): same-sound paths share it.
    key: String,
    /// Cached lexical identity: the space-joined normalized
    /// ([`Partial::norm_word`]) word sequence so far. Corpus keys
    /// carry case/punctuation/source variants of the same lexical
    /// item ("its" vs "it's" → both "its"), which still collapse to
    /// one slot — but lexically different homophones ("to"/"too"/
    /// "two", "hits"/"hitz"-style rivals) stay distinct slots even
    /// when their IPA sequences match. Dedup identity is the
    /// (`key`, `lex`) pair: collapsing purely by pronunciation
    /// would let one wording silently overwrite a different Mad Gab
    /// hypothesis that happens to sound the same, losing lexical
    /// hypotheses the final word-novelty scorer must see.
    lex: String,
    /// Running count of recycled target words (stem-aware, mirroring
    /// the final word-novelty term): lets terminal pre-scoring bound
    /// word-novelty without iterating words.
    reuse_count: u32,
    /// Running sum of per-word familiarity (see
    /// [`Partial::familiarity01`]): lets terminal pre-scoring use the
    /// exact lexical term without iterating words.
    lex_sum: f64,
    /// Running total of word IPA characters: lets terminal
    /// pre-scoring use the exact length signal without iterating.
    ipa_total: usize,
}

impl Partial {
    fn empty() -> Self {
        Self {
            words: Vec::new(),
            sub_cost_total: 0.0,
            cheap_score: 0.0,
            key: String::new(),
            lex: String::new(),
            reuse_count: 0,
            lex_sum: 0.0,
            ipa_total: 0,
        }
    }

    /// Normalized word form for clone detection: lowercase,
    /// alphanumeric characters only.
    fn norm_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// Familiarity of one clue word in [0, 1] from corpus
    /// frequency rank: everyday words score ~1, mid-frequency words
    /// decay gently on a log scale, unattested strings (fragments,
    /// abbreviations, letter-names like "p", "ch", "tch") score 0.
    /// Feeds the final lexical-plausibility term; the beam's
    /// per-link rarity penalty mirrors this direction in transit.
    fn familiarity01(rarity: Option<f64>) -> f64 {
        match rarity {
            Some(r) if r <= 5_000.0 => 1.0,
            Some(r) => (1.0 - 0.35 * (r / 5_000.0).log10()).max(0.0),
            None => 0.0,
        }
    }
    /// Morphological stem of a normalized word: strips common
    /// English inflectional suffixes ('s, es, s, ed, d, ing) so
    /// novelty checks see through inflections ("recognizes",
    /// "recognized" both recycle target "recognize"). Applied to
    /// BOTH clue and target words, so consistency matters more than
    /// linguistic perfection; over-stemming ("bus" → "bu") only
    /// risks equating two non-words alike, which never decides a
    /// real clue ranking. The z→s fold canonicalizes -ize/-ise
    /// spelling variants ("recognize"/"recognise") to one stem;
    /// for a sound puzzle, voicing-only spelling pairs are close
    /// enough to count as the same word for novelty purposes.
    /// Identity (clone keys, output phrases) is never folded.
    fn stem_word(normed: &str) -> String {
        let folded: String = normed
            .chars()
            .map(|c| if c == 'z' { 's' } else { c })
            .collect();
        let mut s = folded;
        for suffix in ["'s", "ies", "es", "ed", "ing", "s", "d"] {
            if s.len() > suffix.len() + 2 && s.ends_with(suffix) {
                s.truncate(s.len() - suffix.len());
                if suffix == "ies" {
                    s.push('y');
                }
                break;
            }
        }
        s
    }

    /// Stem-aware equality: equal stems match, tolerating one
    /// dropped silent e ("recogniz" from "recognized" matches
    /// "recognize"). Catches regular inflections whose suffix strip
    /// exposes a bare consonant.
    fn stems_match(a: &str, b: &str) -> bool {
        a == b || format!("{a}e") == b || a == format!("{b}e")
    }

    fn extend(&self, p: &Pronunciation, consumed: usize, word_sub_cost: f64) -> Self {
        Self::extend_words(
            self,
            &p.word,
            &p.ipa,
            p.rarity,
            word_sub_cost,
            0.0,
            0.0,
            consumed,
        )
    }

    fn recovery_score(&self, target_boundaries: &[usize]) -> f64 {
        let count = self.words.len().max(1) as f64;
        let total = self.ipa_total;
        let mut clue_end = 0usize;
        let mut shared = 0usize;
        let mut ti = 0usize;
        for w in &self.words {
            clue_end += w.ipa.chars().count();
            if clue_end >= total {
                break;
            }
            while ti < target_boundaries.len() && target_boundaries[ti] < clue_end {
                ti += 1;
            }
            if ti < target_boundaries.len() && target_boundaries[ti] == clue_end {
                shared += 1;
            }
        }
        let novelty = 1.0 - shared as f64 / target_boundaries.len().max(1) as f64;
        let similarity = (1.0 - self.sub_cost_total / 4.0).clamp(0.0, 1.0);
        let word_novelty = 1.0 - self.reuse_count as f64 / count;
        let length_signal = (total as f64 / count / 4.0).min(1.0);
        let lexical = self.lex_sum / count;
        0.40 * similarity
            + 0.30 * novelty
            + 0.15 * word_novelty
            + 0.05 * length_signal
            + 0.10 * lexical
    }

    #[allow(clippy::too_many_arguments)]
    fn extend_approx(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        consumed: usize,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
    ) -> Self {
        Self::extend_words(
            self,
            word,
            ipa,
            rarity,
            word_sub_cost,
            boundary_bonus,
            reuse_penalty,
            consumed,
        )
    }

    /// One step of the beam priority, shared by `extend_words` and
    /// the search loop's lazy pre-filter so the gate can never drift
    /// from the priority it predicts (a drifted gate could skip
    /// admittable candidates). See `extend_words` for the rationale
    /// of each term.
    fn step_cheap(
        covered_len: usize,
        rarity: Option<f64>,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
    ) -> f64 {
        // The length earn-back keys on COVERED target characters
        // (evidence), not word IPA length: word IPA includes
        // inserted segments that cover nothing, so earning back on
        // IPA length would subsidize hallucinating extra sounds.
        let word_term = -0.35 + (covered_len as f64).min(6.0) / 6.0 * 0.10;
        // Familiarity penalty on a log-frequency scale: prefixes
        // built from obscure vocabulary (rare words, abbreviations,
        // fragments with no corpus frequency) are weak Mad Gab
        // hypotheses — the final clue score's lexical-plausibility
        // term demotes them, so the beam mirrors that direction
        // here; without it, common-word genuine resegmentations
        // ("dupe"-class) drown among thousands of exotic-spelling
        // near-ties that the final scorer would reject anyway.
        // Unknown frequency (None) pays the max: unattested strings
        // are the least plausible clue words. The penalty starts
        // beyond the default vocabulary horizon (50k, the default
        // `max_rarity` cap): mid-frequency resegmentation words must
        // compete on acoustics, not frequency.
        let rarity_penalty = match rarity {
            Some(r) if r > 50_000.0 => -(0.06 * (r / 50_000.0).log10()).min(0.12),
            Some(_) => 0.0,
            None => -0.12,
        };
        // Word-length, familiarity, resyllabification and reuse
        // terms, plus the final-score-aligned phonetic cost: the cost term leans
        // cells toward clean links (without it, high-cost junk with
        // long common words outranks genuine low-cost
        // resegmentations), while hard budgets and cost-tier cells
        // keep slightly-off close matches alive beside exact ones. A
        // heavier price would bury legitimate mid-cost links; a
        // lighter one lets junk flood the cells (and blow up memory
        // via unbounded downstream fan-out).
        word_term + rarity_penalty - 0.10 * word_sub_cost + boundary_bonus - reuse_penalty
    }

    /// Shared beam priority step, kept as the incremental mirror of
    /// the final clue score. Each extra word pays a segmentation
    /// penalty (net negative per word, so fewer/longer parses outrank
    /// over-segmented ones covering the same span — the old per-word
    /// length *bonus* did the opposite and let degenerate tiny-word
    /// paths dominate); covering more target characters earns back a
    /// small fraction of it; phonetic edits pay their final-score marginal cost (full weight would
    /// over-prune slightly-off close matches that the final scorer
    /// still ranks highly); obscure words pay a log-frequency
    /// penalty mirroring the final lexical-plausibility term;
    /// `boundary_bonus` rewards word ends that resyllabify
    /// away from target boundaries (incremental novelty); and
    /// `reuse_penalty` charges clue words that recycle a target word
    /// (incremental word-novelty — without it, parrot paths like
    /// "its just ..." outrank genuine resyllabifications for the
    /// same span even though the final scorer demotes them).
    #[allow(clippy::too_many_arguments)]
    fn extend_words(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        word_sub_cost: f64,
        boundary_bonus: f64,
        reuse_penalty: f64,
        covered_len: usize,
    ) -> Self {
        let step = Self::step_cheap(
            covered_len,
            rarity,
            word_sub_cost,
            boundary_bonus,
            reuse_penalty,
        );
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
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            cheap_score: self.cheap_score + step,
            key: if self.key.is_empty() {
                ipa.to_string()
            } else {
                format!("{} {ipa}", self.key)
            },
            lex: if self.lex.is_empty() {
                Self::norm_word(word)
            } else {
                format!("{} {}", self.lex, Self::norm_word(word))
            },
            reuse_count: self.reuse_count + u32::from(reuse_penalty > 0.0),
            lex_sum: self.lex_sum + Self::familiarity01(rarity),
            ipa_total: self.ipa_total + ipa.chars().count(),
        }
    }

    /// Exact final score of the one-word terminal extension of this
    /// partial by a match, WITHOUT building the extended partial:
    /// similarity, word-novelty and lexical terms come from running
    /// aggregates plus the match's own data; novelty replays the
    /// boundary walk over borrowed word IPAs (no allocation — the
    /// old HashSet version cost microseconds per call, and terminal
    /// pairs number in the hundreds of thousands). Bit-identical to
    /// extending then calling [`Partial::final_score`]: same terms,
    /// same operation order. Lets terminal retention skip the clone
    /// for parses that cannot beat their completion family's floor.
    fn terminal_score_exact(
        &self,
        target_boundaries: &[usize],
        match_ipa_len: usize,
        match_rarity: Option<f64>,
        match_reuse_penalty: f64,
        match_sub_cost: f64,
    ) -> f64 {
        let nwords = self.words.len() + 1;
        let nwords_f = nwords as f64;
        // Novelty replays final_score's boundary walk without
        // allocating: partial word IPA lengths in order, then the
        // closing match. Both boundary lists ascend, so a
        // two-pointer walk counts the shared inner boundaries
        // exactly as the HashSet version does.
        let mut ti = 0usize;
        let mut target_inner = 0usize;
        let tb = target_boundaries;
        // Count target inners below the eventual clue total: the
        // total is the partial total plus the match length, summed
        // in word order exactly as final_score does (float sum
        // order matters for bit-identity).
        let mut total = 0_usize;
        for w in &self.words {
            total += w.ipa.chars().count();
        }
        total += match_ipa_len;
        while ti < tb.len() && tb[ti] < total {
            target_inner += 1;
            ti += 1;
        }
        // Walk clue boundaries (partial words, then the match end
        // which equals the total and is excluded as non-inner).
        let mut ci = 0usize;
        let mut c2 = 0usize;
        ti = 0;
        for w in &self.words {
            c2 += w.ipa.chars().count();
            if c2 >= total {
                break;
            }
            while ti < tb.len() && tb[ti] < c2 {
                ti += 1;
            }
            if ti < tb.len() && tb[ti] == c2 {
                ci += 1;
            }
        }
        let shared = ci;
        let denom = target_inner.max(1) as f64;
        let novelty = 1.0 - (shared as f64 / denom);
        let reused = (self.reuse_count + u32::from(match_reuse_penalty > 0.0)) as f64 / nwords_f;
        let word_novelty = 1.0 - reused;
        let avg_word_ipa_len = total as f64 / nwords_f;
        let length_signal = (avg_word_ipa_len / 4.0).min(1.0);
        let similarity = (1.0 - (self.sub_cost_total + match_sub_cost) / 4.0).clamp(0.0, 1.0);
        let lexical = (self.lex_sum + Self::familiarity01(match_rarity)) / nwords_f;
        0.40 * similarity
            + 0.30 * novelty
            + 0.15 * word_novelty
            + 0.05 * length_signal
            + 0.10 * lexical
    }

    /// The final clue score for a closed parse: phonetic
    /// similarity, boundary novelty (reshuffled word boundaries),
    /// stem-aware word-novelty (no recycled target words, inflections
    /// included), word-length signal, and lexical plausibility
    /// (familiar real words, not fragments). Computable only
    /// once the parse is complete, so the beam cannot prune on it
    /// mid-parse — but terminal retention can and does (see
    /// [`CompletionTop`]).
    fn final_score(&self, target_boundaries: &[usize], target_words: &FastSet<String>) -> f64 {
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
        let target_inner: FastSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|b| *b < cum)
            .collect();
        let clue_inner: FastSet<usize> = clue_boundaries
            .iter()
            .copied()
            .filter(|b| *b < cum)
            .collect();
        let shared = target_inner.intersection(&clue_inner).count() as f64;
        let denom = target_inner.len().max(1) as f64;
        let novelty = 1.0 - (shared / denom);

        // Word-novelty: penalty if the clue reuses any target word,
        // compared stem-aware so morphological parrots
        // ("recognizes" for target "recognize") count as recycled:
        // without this, inflected parrots score full novelty while
        // sounding identical to the target.
        let target_stems: FastSet<String> = target_words
            .iter()
            .map(|w| Self::stem_word(&Self::norm_word(w)))
            .collect();
        let reused = self
            .words
            .iter()
            .filter(|w| {
                let s = Self::stem_word(&Self::norm_word(&w.word));
                target_stems.iter().any(|t| Self::stems_match(&s, t))
            })
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

        // Lexical plausibility: average word familiarity. Real Mad
        // Gab clues use familiar words; fragment salads ("p ch",
        // "pee tch") and abbreviation piles score near zero here
        // even when they sound close, so they sink below genuine
        // resegmentations. Weighted modestly: familiarity must not
        // overrule acoustics (mid-frequency resegmentation words
        // like "dupe" stay competitive), only break near-ties
        // against unattested strings.
        let lexical = self
            .words
            .iter()
            .map(|w| Self::familiarity01(w.rarity))
            .sum::<f64>()
            / self.words.len().max(1) as f64;

        0.40 * similarity
            + 0.30 * novelty
            + 0.15 * word_novelty
            + 0.05 * length_signal
            + 0.10 * lexical
    }

    fn into_clue(
        self,
        target_ipa: &str,
        target_boundaries: &[usize],
        target_words: &FastSet<String>,
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
/// word-overlap only, no lexical content. Words compare stemmed
/// (see `Partial::stem_word`) so inflectional twins ("recognizes
/// peach" vs "recognized peach") collapse instead of each
/// flooding the list. Input must be score-sorted and
/// phrase-deduped (as the caller provides).
fn select_diverse(clues: Vec<Clue>, top_n: usize) -> Vec<Clue> {
    if top_n == 0 || clues.is_empty() {
        return Vec::new();
    }
    // Precompute per-clue sorted stemmed-word and bigram lists
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
                .map(|w| Partial::stem_word(&Partial::norm_word(w)))
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
    // Pairwise overlap between two clues (symmetric inputs).
    let pair_overlap = |a: usize, b: usize| -> f64 {
        let uw = frac(&bags[a].words, &bags[b].words, word_lens[a]);
        let bw = frac(&bags[a].bigrams, &bags[b].bigrams, bigram_lens[a]);
        0.5 * uw + 0.5 * bw
    };
    let mut remaining: Vec<usize> = (0..clues.len()).collect();
    let mut picked: Vec<usize> = Vec::with_capacity(top_n.min(clues.len()));
    // Closest-picked overlap per candidate, updated incrementally:
    // each round only compares against the NEWLY picked clue and
    // keeps the running max. Identical outcome to comparing against
    // all picked every round (max is associative), but linear per
    // round instead of quadratic — the completion pool holds
    // thousands, where the naive form dominates runtime.
    let mut max_overlap: Vec<f64> = vec![0.0; clues.len()];
    // Clue object moves out of `clues` at the end; work by index.
    while picked.len() < top_n && !remaining.is_empty() {
        if let Some(&last) = picked.last() {
            for &i in remaining.iter() {
                let o = pair_overlap(i, last);
                if o > max_overlap[i] {
                    max_overlap[i] = o;
                }
            }
        }
        let mut best_pos = 0;
        let mut best_val = f64::NEG_INFINITY;
        for (pos, &i) in remaining.iter().enumerate() {
            // Closest-picked overlap (classic MMR): unlike a running
            // union (whose penalties saturate as picked vocabulary
            // grows and bury everything late), every comparison
            // stays local, so a structurally different parse is
            // judged only against its nearest neighbor.
            let v = clues[i].score - MMR_LAMBDA * max_overlap[i];
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

/// Terminal shortlist ranked by final score, floored by clue
/// family (word count × last-word stem): a global top-by-score pool
/// degenerates into a monoculture — hundreds of near-twin parses
/// ("neck egg ninth peach/pitch/pete's/...") crowd out every
/// structurally different parse, so a distinctive genuine
/// resegmentation can be evicted even at a respectable score. Every
/// family keeps its best few completions; the final sort + MMR
/// selection then arbitrate across families. Bounded
/// (families×floor), deterministic, generic word-count + stem only.
struct CompletionTop {
    tops: BTreeMap<usize, CompletionInner>,
}

/// Per-completion-family floor: each (word-count, last-stem)
/// family retains this many of its best completions. Sized to keep
/// genuine resegmentations while bounding total volume: loose
/// terminal retention floods final selection with near-twin
/// variants that bury distinctive parses under sheer count, so the
/// floor stays tight and lets the hotspot families keep only their
/// best representatives.
const COMPLETION_FAMILY_KEEP: usize = 8;

impl CompletionTop {
    fn new(_total_cap: usize) -> Self {
        Self {
            tops: BTreeMap::new(),
        }
    }

    fn insert(&mut self, candidate: Partial, score: f64) {
        let nwords = candidate.words.len();
        self.tops
            .entry(nwords)
            .or_insert_with(CompletionInner::new)
            .insert(candidate, score);
    }

    /// Pre-gate for a terminal candidate scored by
    /// [`Partial::terminal_score_exact`] without building it:
    /// admits unless the family's floor is full and the score cannot
    /// reach its minimum. Sound: scores are non-negative, so bit
    /// order matches numeric order; anything else proceeds to
    /// extend+insert, which arbitrate exactly (including ties and
    /// clone improvements). Allocation-free (borrowed stem lookup,
    /// cached minima).
    fn would_admit_ub(&self, nwords: usize, last_stem: &str, ub: f64) -> bool {
        let Some(inner) = self.tops.get(&nwords) else {
            return true;
        };
        let Some((min, len)) = inner.family_stats(last_stem) else {
            return true;
        };
        len < COMPLETION_FAMILY_KEEP || ub.to_bits() >= min
    }

    fn drain_sorted(self) -> Vec<Partial> {
        let mut v: Vec<(u64, u64, Partial)> = Vec::new();
        for (_, inner) in self.tops {
            v.extend(inner.drain_entries());
        }
        v.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        v.into_iter().map(|(_, _, p)| p).collect()
    }
}

/// One stratum of [`CompletionTop`]: per-last-stem family floors
/// with clone suppression (one slot per distinct (pronunciation,
/// lexical-sequence) parse, so lexically different homophones never
/// overwrite each other). Each family's content is exactly its
/// top-[`COMPLETION_FAMILY_KEEP`] by (score, clone id): a pure
/// function of arrivals, never of arrival order. Scores ride along
/// in the member tuples so floor checks never hash the pool map.
/// Capped so the final re-rank (sort + diversity selection) stays
/// interactive.
struct CompletionInner {
    seq: u64,
    /// Clone key → `(score_bits, seq, entry)`.
    map: FastMap<CloneId, (u64, u64, Partial)>,
    /// Last-word stem → members as (clone id, score bits).
    families: FastMap<String, Vec<ScoredId>>,
}

impl CompletionInner {
    fn new() -> Self {
        Self {
            seq: 0,
            map: FastMap::default(),
            families: FastMap::default(),
        }
    }

    /// Last-word stem family of a completion.
    fn family_of(candidate: &Partial) -> String {
        candidate
            .words
            .last()
            .map(|w| Partial::stem_word(&Partial::norm_word(&w.word)))
            .unwrap_or_default()
    }

    /// Minimum score bits plus member count in a family
    /// (borrowed lookup, cached scores): backs the terminal pre-gate.
    fn family_stats(&self, stem: &str) -> Option<(u64, usize)> {
        self.families.get(stem).map(|members| {
            let min = members
                .iter()
                .map(|(_, bits)| *bits)
                .min()
                .unwrap_or(u64::MAX);
            (min, members.len())
        })
    }

    /// Worst member of a family: lowest score bits, largest clone
    /// id tiebreak (admitted set is exactly top-K by (score, id)).
    fn worst_of(members: &[ScoredId]) -> Option<CloneId> {
        members
            .iter()
            .max_by(|a, b| {
                // Worst = smallest bits; tiebreak = largest id
                // evicted first. Scores are non-negative, so bit
                // order matches numeric order.
                b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
            })
            .map(|(id, _)| id.clone())
    }

    fn insert(&mut self, candidate: Partial, score: f64) {
        // Scores are finite (bounded arithmetic in final_score).
        let bits = score.to_bits();
        let id = (candidate.key.clone(), candidate.lex.clone());
        let family = Self::family_of(&candidate);
        if let Some(slot) = self.map.get_mut(&id) {
            if bits > slot.0 {
                self.seq += 1;
                *slot = (bits, self.seq, candidate);
                if let Some(ms) = self.families.get_mut(&family) {
                    if let Some(slot) = ms.iter_mut().find(|m| m.0 == id) {
                        slot.1 = bits;
                    }
                }
            }
            return;
        }
        self.seq += 1;
        self.map.insert(id.clone(), (bits, self.seq, candidate));
        self.families
            .entry(family.clone())
            .or_default()
            .push((id, bits));
        // Enforce the family floor.
        if let Some(members) = self
            .families
            .get(&family)
            .filter(|m| m.len() > COMPLETION_FAMILY_KEEP)
        {
            let members = members.clone();
            if let Some(worst) = Self::worst_of(&members) {
                if let Some(ms) = self.families.get_mut(&family) {
                    if let Some(pos) = ms.iter().position(|m| m.0 == worst) {
                        ms.swap_remove(pos);
                    }
                }
                self.map.remove(&worst);
            }
        }
    }

    fn drain_entries(mut self) -> Vec<(u64, u64, Partial)> {
        self.map.drain().map(|(_, v)| v).collect()
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

/// Clone identity: (word-IPA sequence, normalized word sequence).
/// Shared by beam pools and completion retention so both dedup on
/// the same (pronunciation, lexical-sequence) pair.
type CloneId = (String, String);
/// Scored clone identity: clone id plus score bits, so floor
/// ordering scans scores without hashing the entry map.
type ScoredId = (CloneId, u64);

/// One beam position: a deferred, order-independent candidate pool.
///
/// Retention is per-hypothesis-family floors, not global top-K:
/// every (segmentation-depth, cost-tier, last-word-sound) family
/// keeps its best [`GROUP_POOL_KEEP`] prefixes in the pool, and
/// contributes its best [`GROUP_SELECT_KEEP`] to expansion. Dense
/// near-tie swarms (hundreds of acoustically indistinguishable
/// prefixes) fill only their own family's floor instead of crowding
/// out thin genuine families — the beam cannot resolve a 0.05-wide
/// band, so it keeps representatives of every family and lets
/// later links plus the final scorer decide.
///
/// Bounding is online but order-independent: each family's content
/// is exactly its top-K by (cheap score, clone id), a pure function
/// of the candidate multiset — arrival order only affects
/// intermediate states, never the final set. (A batch
/// accumulate-everything-then-select would need hundreds of
/// megabytes per position at real fan-in; the outcome here is
/// identical to batch floors.) The hot pair loop pre-filters
/// through [`BeamPos::would_admit`] (arithmetic + hash lookups, no
/// allocation) and only clones a `Partial` on admission; floors
/// refuse ~99% of pairs before cloning, which funds the wide
/// fan-in the floors must see.
struct BeamPos {
    /// Clone key (word-IPA sequence, normalized word sequence) ->
    /// covering. Clone-suppressed: best cheap score wins.
    pool: FastMap<CloneId, Partial>,
    /// Hypothesis families: stratum cell (word count, cost tier)
    /// -> last-word IPA -> members as (clone id, cheap-score bits).
    /// Each family's membership is its top-K; vectors stay tiny (≤
    /// [`GROUP_POOL_KEEP`]). Cheap bits ride along so the hot gate
    /// scans scores without touching the pool map (hashing two
    /// Strings per member would dominate pair cost). Two levels so
    /// the gate looks families up by borrowed `&str` without
    /// allocating.
    groups: FastMap<(usize, u8), FastMap<String, Vec<ScoredId>>>,
}

/// Per-family pool floor: each hypothesis family retains this many
/// of its best prefixes. Sized ABOVE the true depth of the densest
/// near-tie families (~20-30 distinct prefixes): when the floor
/// exceeds every family's membership, refusal vanishes — admission
/// is pure clone-suppression, hence trivially arrival-order
/// independent, and every distinct hypothesis survives to expansion
/// (the closing zone) or selection. Bounds the pool to roughly
/// families×keep transient entries (each position drains).
const GROUP_POOL_KEEP: usize = 16;

/// Per-family expansion floor: each family contributes this many of
/// its best prefixes to expansion outside the closing zone.
/// Covers the observed within-group ranks of genuine
/// resegmentations with margin, while swarms cannot exceed it.
/// Funded by strict admission gates (most pairs refused before
/// cloning) and exact terminal pre-scoring.
const GROUP_SELECT_KEEP: usize = 5;

/// Closing-zone span: positions within this many target characters
/// of the end expand the WHOLE pooled set, not the selection
/// shortlist. Rationale: this close to a complete parse, retention
/// arbitrates by exact final score (terminal pre-score + family
/// floors), never by the heuristic cheap priority — so there is no
/// need to pre-filter with it. Mid-search floors exist because
/// partial parses cannot be final-scored; within the zone every
/// expansion is one or two links from a scored completion.
/// Six covers two average content words: the canonical
/// penultimate prefixes sit inside it, while earlier links rank
/// top-tier for selection. Bounded: the pool itself
/// is floored, and terminal pairs skip cloning/scoring/insertion
/// via exact pre-scoring unless competitive.
const CLOSING_SPAN: usize = 6;

/// Global per-position safety cap. Family floors bound the pool in
/// practice (~8k); if degenerate input ever exceeds this, whole
/// worst families are dropped deterministically (never partial
/// families — a family's floor is atomic).
const POOL_SAFETY: usize = 32768;

impl BeamPos {
    fn new() -> Self {
        Self {
            pool: FastMap::default(),
            groups: FastMap::default(),
        }
    }

    fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    /// Beam cell key: word count (segmentation depth) by
    /// acoustic-cost tier (clean / near / far parses never compete
    /// with each other for family floors).
    fn cell_of(cand_words: usize, cand_sub: f64) -> (usize, u8) {
        (cand_words, cost_tier(cand_sub))
    }

    /// Worst member of a family group: lowest cheap score,
    /// clone-id-descending tiebreak (so the admitted set is exactly
    /// the top-K by (cheap, id) — deterministic under ties).
    fn worst_of(members: &[ScoredId]) -> Option<CloneId> {
        members
            .iter()
            .max_by(|a, b| {
                let (ca, cb) = (f64::from_bits(a.1), f64::from_bits(b.1));
                // Worst = smallest cheap; tiebreak = largest id
                // evicted first.
                cb.partial_cmp(&ca)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            })
            .map(|(id, _)| id.clone())
    }

    /// Lazy pre-filter for a candidate that has not been built yet.
    /// Admits clone improvements and candidates that make their
    /// family's top-[`GROUP_POOL_KEEP`]; refuses everything else
    /// before any cloning. Mirrors [`BeamPos::insert`] exactly (a
    /// drifted gate could skip admittable candidates). Pure function
    /// of pool content: arrival order cannot matter. Allocation-free
    /// on the refusal path (family lookup by borrowed `&str`, min
    /// scan over pool references); only admitted candidates pay for
    /// clone-key assembly.
    fn would_admit(
        &self,
        partial: &Partial,
        match_ipa: &str,
        match_norm: &str,
        cand_words: usize,
        cand_sub: f64,
        cand_cheap: f64,
    ) -> bool {
        let cell = Self::cell_of(cand_words, cand_sub);
        let members = self
            .groups
            .get(&cell)
            .and_then(|by_end| by_end.get(match_ipa));
        let Some(members) = members else {
            // New family (or new cell): room. Clone check below.
            return self.clone_allows(partial, match_ipa, match_norm, cand_cheap);
        };
        if members.len() < GROUP_POOL_KEEP {
            return self.clone_allows(partial, match_ipa, match_norm, cand_cheap);
        }
        // Full family: compare against its worst without allocating
        // (cheap scores ride along in the member tuples; compared
        // as floats — bit patterns do not order negatives).
        let mut worst_cheap = f64::INFINITY;
        let mut worst_id: Option<&(String, String)> = None;
        for (id, bits) in members {
            let cheap = f64::from_bits(*bits);
            let replace =
                cheap < worst_cheap || (cheap == worst_cheap && worst_id.is_some_and(|w| id > w));
            if replace {
                worst_cheap = cheap;
                worst_id = Some(id);
            }
        }
        if cand_cheap < worst_cheap {
            return false;
        }
        if cand_cheap > worst_cheap {
            return self.clone_allows(partial, match_ipa, match_norm, cand_cheap);
        }
        // Exact cheap tie: the newcomer displaces the worst iff
        // its clone id is smaller (insert evicts min-cheap-max-id,
        // so anything else would be cloned only to be evicted).
        // Clone improvements take the clone-check path instead.
        let id = Self::assemble_id(partial, match_ipa, match_norm);
        if let Some(stored) = self.pool.get(&id) {
            return cand_cheap > stored.cheap_score;
        }
        match worst_id {
            Some(worst_id) => id < *worst_id,
            // Inconsistent (empty group entry); admit.
            None => true,
        }
    }

    /// Assemble a clone id from a partial plus one match step.
    fn assemble_id(partial: &Partial, match_ipa: &str, match_norm: &str) -> (String, String) {
        let key = if partial.key.is_empty() {
            match_ipa.to_string()
        } else {
            format!("{} {}", partial.key, match_ipa)
        };
        let lex = if partial.lex.is_empty() {
            match_norm.to_string()
        } else {
            format!("{} {}", partial.lex, match_norm)
        };
        (key, lex)
    }

    /// Clone check by assembled id: admits unless a pooled clone is
    /// at least as good.
    fn clone_allows_id(&self, id: &(String, String), cand_cheap: f64) -> bool {
        match self.pool.get(id) {
            Some(p) => cand_cheap > p.cheap_score,
            None => true,
        }
    }

    /// Clone check assembling the id (admit path only).
    fn clone_allows(
        &self,
        partial: &Partial,
        match_ipa: &str,
        match_norm: &str,
        cand_cheap: f64,
    ) -> bool {
        self.clone_allows_id(
            &Self::assemble_id(partial, match_ipa, match_norm),
            cand_cheap,
        )
    }

    /// Admit a built candidate under clone suppression and family
    /// floors, evicting the family's worst past the floor.
    /// Deterministic outcome (per-family top-K) regardless of
    /// arrival order. Mirrors [`BeamPos::would_admit`]: anything the
    /// gate refuses is also refused here.
    fn insert(&mut self, candidate: Partial) {
        let id = (candidate.key.clone(), candidate.lex.clone());
        let bits = candidate.cheap_score.to_bits();
        let new_end = candidate
            .words
            .last()
            .map(|w| w.ipa.clone())
            .unwrap_or_default();
        let new_family = Self::cell_of(candidate.words.len(), candidate.sub_cost_total);
        match self.pool.get(&id) {
            Some(p) => {
                if candidate.cheap_score <= p.cheap_score {
                    return;
                }
                // Clone improvement: swap in place, relocating
                // across families if the improved acoustics tier
                // differently, and refreshing the cached score.
                let old = self.pool.insert(id.clone(), candidate).expect("present");
                let old_end = old.words.last().map(|w| w.ipa.clone()).unwrap_or_default();
                let old_family = Self::cell_of(old.words.len(), old.sub_cost_total);
                if (old_family, &old_end) != (new_family, &new_end) {
                    if let Some(by_end) = self.groups.get_mut(&old_family) {
                        if let Some(members) = by_end.get_mut(&old_end) {
                            if let Some(pos) = members.iter().position(|m| m.0 == id) {
                                members.swap_remove(pos);
                            }
                        }
                    }
                    self.groups
                        .entry(new_family)
                        .or_default()
                        .entry(new_end.clone())
                        .or_default()
                        .push((id, bits));
                } else if let Some(members) = self
                    .groups
                    .get_mut(&new_family)
                    .and_then(|by_end| by_end.get_mut(&new_end))
                {
                    if let Some(slot) = members.iter_mut().find(|m| m.0 == id) {
                        slot.1 = bits;
                    }
                }
            }
            None => {
                self.pool.insert(id.clone(), candidate);
                self.groups
                    .entry(new_family)
                    .or_default()
                    .entry(new_end.clone())
                    .or_default()
                    .push((id, bits));
            }
        }
        // Enforce the family floor: evict its worst past the keep.
        let over = self
            .groups
            .get(&new_family)
            .and_then(|by_end| by_end.get(&new_end))
            .map_or(0, Vec::len)
            > GROUP_POOL_KEEP;
        if over {
            let members = self.groups[&new_family][&new_end].clone();
            if let Some(worst_id) = Self::worst_of(&members) {
                if let Some(ms) = self
                    .groups
                    .get_mut(&new_family)
                    .and_then(|by_end| by_end.get_mut(&new_end))
                {
                    if let Some(pos) = ms.iter().position(|m| m.0 == worst_id) {
                        ms.swap_remove(pos);
                    }
                }
                self.pool.remove(&worst_id);
            }
        }
        // Global safety: drop whole worst families (never partial
        // ones) if degenerate input overflows the cap.
        if self.pool.len() > POOL_SAFETY {
            self.enforce_safety();
        }
    }

    /// Drop whole worst families until back under half the safety
    /// cap. Families order by (best cheap, family key): total,
    /// deterministic. Expected never to fire (floors bound the pool
    /// far below it).
    fn enforce_safety(&mut self) {
        let mut fams: Vec<((usize, u8), String, f64)> = self
            .groups
            .iter()
            .flat_map(|(cell, by_end)| {
                by_end.iter().map(|(end, members)| {
                    let best = members
                        .iter()
                        .map(|(_, bits)| f64::from_bits(*bits))
                        .fold(f64::NEG_INFINITY, f64::max);
                    (*cell, end.clone(), best)
                })
            })
            .collect();
        fams.sort_by(|a, b| {
            a.2.partial_cmp(&b.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
                .then_with(|| a.1.cmp(&b.1))
        });
        for (cell, end, _) in fams {
            if self.pool.len() <= POOL_SAFETY / 2 {
                break;
            }
            let members = self
                .groups
                .get_mut(&cell)
                .and_then(|by_end| by_end.remove(&end));
            if let Some(members) = members {
                for (id, _) in members {
                    self.pool.remove(&id);
                }
            }
        }
    }

    /// Deterministically select the expansion shortlist and drain
    /// the pool: every hypothesis family's best
    /// [`GROUP_SELECT_KEEP`] prefixes, gathered across all families
    /// in deterministic group order. No global budget: the floors
    /// bound the shortlist, and the strict admission gates fund it
    /// by refusing most pairs before cloning. Each position expands
    /// exactly once, by this or by [`BeamPos::take_all`].
    /// Drain the whole pooled set for closing-zone expansion (see
    /// [`CLOSING_SPAN`]): every pooled prefix expands, selection
    /// floors do not apply. Deterministic order (quality first).
    fn take_all(&mut self) -> Vec<Partial> {
        let mut out: Vec<Partial> = self.pool.drain().map(|(_, p)| p).collect();
        self.groups.clear();
        out.sort_by(|a, b| {
            b.cheap_score
                .partial_cmp(&a.cheap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.key.cmp(&b.key))
                .then_with(|| a.lex.cmp(&b.lex))
        });
        out
    }
    fn take_selected(&mut self) -> Vec<Partial> {
        if self.pool.is_empty() {
            self.groups.clear();
            return Vec::new();
        }
        // Deterministic group order; members best-first with total
        // tiebreaks.
        let mut fams: Vec<((usize, u8), String)> = self
            .groups
            .iter()
            .flat_map(|(cell, by_end)| by_end.keys().map(|end| (*cell, end.clone())))
            .collect();
        fams.sort();
        let mut out = Vec::new();
        for (cell, end) in &fams {
            let Some(members) = self.groups.get(cell).and_then(|by_end| by_end.get(end)) else {
                continue;
            };
            let mut sorted = members.clone();
            sorted.sort_by(|a, b| {
                let (ca, cb) = (f64::from_bits(a.1), f64::from_bits(b.1));
                cb.partial_cmp(&ca)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            for (id, _) in sorted.into_iter().take(GROUP_SELECT_KEEP) {
                if let Some(p) = self.pool.remove(&id) {
                    out.push(p);
                }
            }
        }
        self.pool.clear();
        self.groups.clear();
        // Deterministic expansion order (quality first; ties break
        // on the full dedup identity).
        out.sort_by(|a, b| {
            b.cheap_score
                .partial_cmp(&a.cheap_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.key.cmp(&b.key))
                .then_with(|| a.lex.cmp(&b.lex))
        });
        out
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
            cheap_score: cheap,
            key: key.to_string(),
            lex: key.to_string(),
            reuse_count: 0,
            lex_sum: 0.0,
            ipa_total: 0,
        }
    }

    /// Lexically explicit partial: `words`/`ipas` must pair up; the
    /// dedup identity is the (IPA sequence, normalized word
    /// sequence) pair.
    fn partial_lex(cheap: f64, cost: f64, words: &[&str], ipas: &[&str]) -> Partial {
        assert_eq!(words.len(), ipas.len());
        let mut p = Partial {
            words: words
                .iter()
                .zip(ipas.iter())
                .map(|(w, i)| ClueWord {
                    word: w.to_string(),
                    ipa: i.to_string(),
                    rarity: None,
                    sub_cost: 0.0,
                })
                .collect(),
            sub_cost_total: cost,
            cheap_score: cheap,
            key: String::new(),
            lex: String::new(),
            reuse_count: 0,
            lex_sum: 0.0,
            ipa_total: 0,
        };
        p.key = p
            .words
            .iter()
            .map(|w| w.ipa.clone())
            .collect::<Vec<_>>()
            .join(" ");
        p.lex = p
            .words
            .iter()
            .map(|w| Partial::norm_word(&w.word))
            .collect::<Vec<_>>()
            .join(" ");
        p
    }

    #[test]
    fn boundary_luck_never_outweighs_acoustic_quality() {
        // Same-span links: A covers more target characters with
        // clean acoustics but lands on a target boundary; B covers
        // less with costly acoustics but lands off-boundary. A must
        // win: the resyllabification lean is deliberately smaller
        // than real acoustic/evidence gaps, so junk salads cannot
        // outrank genuine resegmentations on boundary luck alone
        // (and per-word bonuses must not subsidize
        // over-segmentation). Familiarity is an explicit
        // log-frequency penalty mirroring the final
        // lexical-plausibility term (see
        // `familiarity_penalty_ranks_exotic_below_common`).
        let a = Partial::step_cheap(4, Some(2_000.0), 0.15, 0.0, 0.0);
        let b = Partial::step_cheap(2, Some(45_000.0), 0.60, 0.05, 0.0);
        assert!(
            a > b,
            "on-boundary clean link lost to off-boundary junk: {a} vs {b}"
        );
    }

    #[test]
    fn stemming_sees_through_inflections() {
        // Inflected parrots must read as recycled: recognizes,
        // recognized (via silent-e tolerance) and runs all match
        // their roots, while unrelated words do not. Short words
        // are guard-protected from over-stripping.
        assert!(Partial::stems_match(
            &Partial::stem_word("recognizes"),
            &Partial::stem_word("recognize"),
        ));
        assert!(Partial::stems_match(
            &Partial::stem_word("recognized"),
            &Partial::stem_word("recognize"),
        ));
        assert!(Partial::stems_match(
            &Partial::stem_word("runs"),
            &Partial::stem_word("run"),
        ));
        assert!(!Partial::stems_match(
            &Partial::stem_word("beach"),
            &Partial::stem_word("peach"),
        ));
        assert_eq!(Partial::stem_word("hid"), "hid");
        assert_eq!(Partial::stem_word("a"), "a");
    }

    #[test]
    fn final_score_demotes_inflected_parrots_and_fragments() {
        // Two generic quality properties of the final objective:
        // (a) an inflected target word ("recognizes" for target
        // "recognize") counts as recycled; (b) a parse of familiar
        // words outscores an acoustically identical parse of
        // unattested fragments.
        fn partial(words: Vec<WordSpec>, sub: f64) -> Partial {
            let mut key = String::new();
            let mut lex = String::new();
            let mut ws = Vec::new();
            for (w, ipa, rarity, sc) in words {
                ws.push(ClueWord {
                    word: w.to_string(),
                    ipa: ipa.to_string(),
                    rarity,
                    sub_cost: sc,
                });
                key = if key.is_empty() {
                    ipa.to_string()
                } else {
                    format!("{key} {ipa}")
                };
                let n = Partial::norm_word(w);
                lex = if lex.is_empty() {
                    n
                } else {
                    format!("{lex} {n}")
                };
            }
            Partial {
                words: ws,
                sub_cost_total: sub,
                cheap_score: 0.0,
                key,
                lex,
                reuse_count: 0,
                lex_sum: 0.0,
                ipa_total: 0,
            }
        }
        let bounds = vec![4, 8];
        // (a) parrot inflection is recycled word-novelty-wise.
        let parrot = partial(
            vec![("recognizes", "ɹɛkəgnaɪzəz", Some(13_520.0), 0.3)],
            0.3,
        );
        let target: FastSet<String> = ["recognize"].iter().map(|s| s.to_string()).collect();
        let plain = partial(vec![("peach", "pitʃ", Some(4_000.0), 0.3)], 0.3);
        assert!(
            plain.final_score(&bounds, &target) > parrot.final_score(&bounds, &target),
            "inflected parrot was not demoted"
        );
        // (b) familiar words beat same-sounding unattested
        // spellings at equal acoustics (isolates lexical
        // plausibility: identical spans, boundaries, novelty).
        let familiar = partial(
            vec![
                ("beach", "bitʃ", Some(3_000.0), 0.15),
                ("peach", "pitʃ", Some(4_000.0), 0.15),
            ],
            0.3,
        );
        let fragments = partial(
            vec![("bch", "bitʃ", None, 0.15), ("pch", "pitʃ", None, 0.15)],
            0.3,
        );
        let target2: FastSet<String> = ["zz", "yy"].iter().map(|s| s.to_string()).collect();
        assert!(
            familiar.final_score(&bounds, &target2) > fragments.final_score(&bounds, &target2),
            "fragment salad was not demoted below real words"
        );
    }

    /// (word, IPA, rarity, sub-cost) spec for synthetic partials.
    type WordSpec<'a> = (&'a str, &'a str, Option<f64>, f64);

    #[test]
    fn terminal_score_matches_final_score() {
        // Bit-identity between the allocation-free terminal
        // pre-score and extend-then-final_score: the retention gate
        // relies on exactness (a mismatch would wrongly refuse
        // completions). Exercises multiword prefixes, recycled
        // words, fragments, and boundary-sharing spans.
        let bounds = vec![3, 7, 10];
        let target: FastSet<String> = ["aa", "bb", "cc"].iter().map(|s| s.to_string()).collect();
        let stems: FastSet<String> = target
            .iter()
            .map(|w| Partial::stem_word(&Partial::norm_word(w)))
            .collect();
        let cases: Vec<Vec<WordSpec>> = vec![
            vec![
                ("wreck", "ɹɛk", Some(10_201.0), 0.0),
                ("a", "ə", Some(4.0), 0.0),
            ],
            vec![("recognizes", "ɹɛkəgnaɪzəz", Some(13_520.0), 0.3)],
            vec![("x", "ks", None, 0.6), ("eh", "ɛ", None, 0.15)],
            vec![("aa", "ɑ", Some(100.0), 0.0)],
        ];
        let closings: Vec<(&str, &str, Option<f64>, f64)> = vec![
            ("nice", "naɪs", Some(2_000.0), 0.75),
            ("beach", "bitʃ", Some(3_000.0), 0.60),
            ("p", "p", None, 0.60),
        ];
        for words in &cases {
            let mut partial = Partial::empty();
            for (w, ipa, rarity, sub) in words {
                let reuse = if stems
                    .iter()
                    .any(|t| Partial::stems_match(&Partial::stem_word(&Partial::norm_word(w)), t))
                {
                    0.10
                } else {
                    0.0
                };
                partial =
                    partial.extend_approx(w, ipa, *rarity, ipa.chars().count(), *sub, 0.0, reuse);
            }
            for (cw, cipa, crarity, csub) in &closings {
                let reuse = if stems
                    .iter()
                    .any(|t| Partial::stems_match(&Partial::stem_word(&Partial::norm_word(cw)), t))
                {
                    0.10
                } else {
                    0.0
                };
                let pre = partial.terminal_score_exact(
                    &bounds,
                    cipa.chars().count(),
                    *crarity,
                    reuse,
                    *csub,
                );
                let built = partial
                    .extend_approx(cw, cipa, *crarity, cipa.chars().count(), *csub, 0.0, reuse)
                    .final_score(&bounds, &target);
                assert!(
                    pre.to_bits() == built.to_bits(),
                    "pre-score {pre} != final {built} for {:?} + {cw}",
                    words.iter().map(|w| w.0).collect::<Vec<_>>(),
                );
            }
        }
    }

    #[test]
    fn familiarity_penalty_ranks_exotic_below_common() {
        // Same span, same acoustics, same boundaries: a link built
        // from an unattested fragment must lose to one built from
        // familiar words. Pins the lexical-plausibility direction of
        // the beam priority (mirrored in the final score).
        let common = Partial::step_cheap(4, Some(2_000.0), 0.30, 0.0, 0.0);
        let exotic = Partial::step_cheap(4, None, 0.30, 0.0, 0.0);
        assert!(
            common > exotic,
            "unattested fragment was not penalized: {common} vs {exotic}"
        );
    }

    #[test]
    fn beam_pool_is_order_independent() {
        // The same candidate multiset inserted in opposite orders
        // must yield the same expansion shortlist: retention is a
        // pure function of content (per-family top-K floors, clone
        // suppression), never of arrival order.
        fn shortlist(order: &[i32]) -> Vec<String> {
            let mut beam = BeamPos::new();
            for &o in order.iter() {
                beam.insert(partial_with(
                    0.0 - 0.01 * o as f64,
                    if o % 3 == 0 { 0.15 } else { 0.0 },
                    2,
                    &format!("key {o}"),
                ));
            }
            let mut sel = beam.take_selected();
            sel.sort_by(|a, b| a.key.cmp(&b.key));
            sel.into_iter().map(|p| p.key).collect()
        }
        let fwd: Vec<i32> = (0..200).collect();
        let mut rev = fwd.clone();
        rev.reverse();
        assert_eq!(shortlist(&fwd), shortlist(&rev));
    }

    #[test]
    fn beam_family_floors_cap_swarms_but_keep_thin_families() {
        // 60 same-family prefixes (one ending sound, one stratum):
        // the pool floor keeps the best 8, expansion takes the best
        // 3 — while a thin family in another stratum keeps its lone
        // member through both stages.
        let mut beam = BeamPos::new();
        for i in 0..60 {
            beam.insert(partial_with(
                0.0 - 0.001 * i as f64,
                0.0,
                2,
                &format!("swarm {i}"),
            ));
        }
        beam.insert(partial_with(-0.5, 0.15, 3, "thin tier different"));
        // Pool: swarm family capped at GROUP_POOL_KEEP; thin family
        // whole (1 member).
        let swarm_kept = beam.pool.values().filter(|p| p.words.len() == 2).count();
        assert_eq!(
            swarm_kept, GROUP_POOL_KEEP,
            "swarm family must be capped at the pool floor"
        );
        assert!(
            beam.pool.values().any(|p| p.key == "thin tier different"),
            "thin family must survive pooling whole"
        );
        let sel = beam.take_selected();
        let swarm_sel = sel.iter().filter(|p| p.words.len() == 2).count();
        assert_eq!(
            swarm_sel, GROUP_SELECT_KEEP,
            "swarm family must be capped at the selection floor"
        );
        assert!(
            sel.iter().any(|p| p.key == "thin tier different"),
            "thin family must reach expansion"
        );
    }

    #[test]
    fn beam_gate_mirrors_floor_admission() {
        // The lazy gate must agree with insert: a candidate worse
        // than its full family's floor is refused before cloning; a
        // better one is admitted; clone improvements are always
        // admitted. (A drifted gate would skip admittable
        // candidates.) Family here is ((2, tier0), "ab").
        let mut beam = BeamPos::new();
        for i in 0..GROUP_POOL_KEEP {
            let w = format!("w{i}");
            beam.insert(partial_lex(
                0.0 - 0.001 * i as f64,
                0.0,
                &[&w[..], "tail"],
                &["ab", "ab"],
            ));
        }
        // Prefix "pre" + match "tail"/"ab": candidate words 2, sub
        // 0.0, family ((2, tier0), "ab"), at its floor of 8; worst
        // kept is rank 7 at cheap ≈ -0.007.
        let prefix = partial_lex(-0.40, 0.0, &["pre"], &["ab"]);
        assert!(
            !beam.would_admit(&prefix, "ab", "zzz", 2, 0.0, -0.50),
            "below-floor candidate must be refused"
        );
        assert!(
            beam.would_admit(&prefix, "ab", "aaa", 2, 0.0, -0.001),
            "above-floor candidate must be admitted"
        );
        // Clone improvement: same id as member 0 ("w0 tail" /
        // "ab ab") with better cheap.
        let prefix0 = partial_lex(0.0, 0.0, &["w0"], &["ab"]);
        assert!(
            beam.would_admit(&prefix0, "ab", "tail", 2, 0.0, 0.50),
            "clone improvement must be admitted"
        );
        assert!(
            !beam.would_admit(&prefix0, "ab", "tail", 2, 0.0, -0.50),
            "clone inferior must be refused"
        );
    }

    #[test]
    fn beam_keeps_lexically_distinct_homophones() {
        // Same pronunciation, different words: dedup identity is the
        // (IPA, lexical-sequence) pair, so neither wording may
        // overwrite the other in the pool or the expansion
        // shortlist. Only exact lexical+IPA duplicates collapse.
        let mut beam = BeamPos::new();
        beam.insert(partial_lex(-0.30, 0.0, &["to"], &["tu"]));
        beam.insert(partial_lex(-0.31, 0.0, &["two"], &["tu"]));
        beam.insert(partial_lex(-0.32, 0.0, &["too"], &["tu"]));
        // Exact duplicate of an existing path: collapses, keeping
        // the better cheap score.
        beam.insert(partial_lex(-0.29, 0.0, &["to"], &["tu"]));
        assert_eq!(beam.pool.len(), 3, "homophones must hold distinct slots");
        let sel = beam.take_selected();
        let mut got: Vec<String> = sel.iter().map(|p| p.lex.clone()).collect();
        got.sort();
        assert_eq!(
            got,
            vec!["to".to_string(), "too".to_string(), "two".to_string()]
        );
        assert!(
            sel.iter()
                .find(|p| p.lex == "to")
                .is_some_and(|p| (p.cheap_score - (-0.29)).abs() < 1e-12),
            "duplicate collapse must keep the better cheap score"
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
