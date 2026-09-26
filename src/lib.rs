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

        // beam[p] = Pareto-banded coverings of [0..p). BeamPos
        // keeps per-(word-count, cost-tier) cells. The beam priority
        // is the generic prefix objective (same four components and
        // weights as the final score, evaluated on the covered
        // target prefix), so beam order and terminal retention
        // optimize the same scalar. The priority is non-additive, so
        // there is no lazy arithmetic gate: every budget-admissible
        // extension is built (scored exactly once, in extend_approx)
        // and admitted by BeamPos::insert. Roomy beams stay
        // affordable, which is what lets valid mid-pack parses
        // survive alongside hundreds of near-tie rivals.
        let k = self.config.beam_width;
        let mut beam: Vec<BeamPos> = (0..=n).map(|_| BeamPos::new(k)).collect();
        beam[0].insert(Partial::empty());
        // Terminal retention is by final score (see CompletionTop):
        // every distinct closed parse contends for a generous
        // capped pool, then the pool is sorted, deduped and
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
            // Take ownership of the beam-at-p so we can mutate beam[p..] freely.
            // No lazy gate: cheap_score is the non-additive prefix
            // objective, so no arithmetic shortcut can predict it
            // exactly. Build each budget-admissible extension (which
            // scores itself with that same objective) and let insert()
            // arbitrate. This keeps gate and priority consistent by
            // construction.
            let here = beam[p].take_entries();
            for partial in &here {
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
                    let next = partial.extend_approx(
                        &m.word,
                        &m.ipa,
                        m.rarity,
                        m.consumed,
                        m.cost,
                        &target_boundaries,
                        &target_words,
                    );
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
/// prefix objective score used to prune the beam. Target-aligned
/// cuts (per-word consumed target spans) are stored on `spans` so
/// both prefix and final scoring compare clue cuts against target
/// boundaries in the same target coordinate space.
#[derive(Debug, Clone)]
struct Partial {
    words: Vec<ClueWord>,
    /// Target chars consumed per word, parallel to `words`. In Exact
    /// mode each span equals its word's IPA length; in Approximate
    /// mode indels make them differ.
    spans: Vec<usize>,
    /// Accumulated substitution cost across all words so far.
    /// Always zero in Exact mode.
    sub_cost_total: f64,
    /// Beam priority. In Approximate mode this is the generic prefix
    /// objective (same four components and weights as the final
    /// score, evaluated on the covered target prefix), recomputed
    /// after every extension. In Exact mode it keeps the historical
    /// additive cheap score.
    cheap_score: f64,
    /// Cached clone-detection key: one `\x1e`-joined token per word,
    /// each token `normalized-word\x1fipa`. Both halves matter: the
    /// IPA half keeps acoustically distinct parses in separate slots,
    /// while the normalized-word half keeps lexically distinct
    /// homophones ("wreck" vs "wreak" → same sound, different words)
    /// in separate slots so either spelling can win downstream. Only
    /// exact same normalized word + same IPA path duplicates collapse
    /// (best score wins the slot). Separators are ASCII control codes
    /// that can appear in neither half, so token boundaries are
    /// unambiguous.
    key: String,
}

impl Partial {
    fn empty() -> Self {
        Self {
            words: Vec::new(),
            spans: Vec::new(),
            sub_cost_total: 0.0,
            cheap_score: 0.0,
            key: String::new(),
        }
    }

    /// Normalized word form: lowercase, alphanumeric only.
    /// Used for the lexical half of the clone key (so "It's" and
    /// "its" share one identity) and kept available for reuse.
    #[allow(dead_code)]
    fn norm_word(word: &str) -> String {
        word.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// Target-aligned cuts: cumulative consumed spans. Falls back to
    /// clue IPA lengths when spans are absent (legacy/Exact paths
    /// where consumed equals IPA length by construction).
    fn cuts(&self) -> Vec<usize> {
        if self.spans.len() == self.words.len() {
            let mut out = Vec::with_capacity(self.spans.len());
            let mut cum = 0_usize;
            for s in &self.spans {
                cum += *s;
                out.push(cum);
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.words.len());
            let mut cum = 0_usize;
            for w in &self.words {
                cum += w.ipa.chars().count();
                out.push(cum);
            }
            out
        }
    }

    /// Generic objective: same four normalized components and weights
    /// as the final score, evaluated on the covered target prefix
    /// `cum`. Boundary novelty compares target-aligned consumed-span
    /// cuts against target boundaries within the covered prefix via
    /// symmetric Jaccard (union-empty novelty = 0); word novelty is
    /// the reused-target-word fraction in the partial; similarity and
    /// length signals match the final definitions. Used for both
    /// prefix (beam priority) and final (terminal retention) scoring
    /// so the two objectives are consistent.
    fn objective_score(
        words: &[ClueWord],
        cuts: &[usize],
        sub_cost_total: f64,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
    ) -> f64 {
        let cum = cuts.last().copied().unwrap_or(0);
        let target_inner: HashSet<usize> = target_boundaries
            .iter()
            .copied()
            .filter(|b| *b < cum)
            .collect();
        let clue_inner: HashSet<usize> =
            cuts.iter().copied().filter(|c| *c < cum).collect();
        let inter = target_inner.intersection(&clue_inner).count() as f64;
        let union = target_inner.union(&clue_inner).count() as f64;
        let novelty = if union == 0.0 {
            0.0
        } else {
            1.0 - (inter / union)
        };
        let reused = words
            .iter()
            .filter(|w| target_words.contains(&w.word.to_lowercase()))
            .count() as f64;
        let word_novelty = 1.0 - (reused / words.len().max(1) as f64);
        let avg_word_ipa_len = if words.is_empty() {
            0.0
        } else {
            words.iter().map(|w| w.ipa.chars().count()).sum::<usize>() as f64
                / words.len() as f64
        };
        let length_signal = (avg_word_ipa_len / 4.0).min(1.0);
        let similarity = (1.0 - sub_cost_total / 4.0).clamp(0.0, 1.0);
        0.40 * similarity + 0.35 * novelty + 0.15 * word_novelty + 0.10 * length_signal
    }

    fn prefix_score(&self, target_boundaries: &[usize], target_words: &HashSet<String>) -> f64 {
        Self::objective_score(
            &self.words,
            &self.cuts(),
            self.sub_cost_total,
            target_boundaries,
            target_words,
        )
    }

    /// One clone-key token: normalized word + IPA with unambiguous
    /// separators (see [`Partial::key`]).
    fn step_key(word: &str, ipa: &str) -> String {
        format!("{}\x1f{ipa}", Self::norm_word(word))
    }

    /// Clone key for `self` extended by one word.
    fn extended_key(&self, word: &str, ipa: &str) -> String {
        let tok = Self::step_key(word, ipa);
        if self.key.is_empty() {
            tok
        } else {
            format!("{}\x1e{tok}", self.key)
        }
    }

    fn extend(&self, p: &Pronunciation, consumed: usize, word_sub_cost: f64) -> Self {
        Self::extend_words_exact(self, &p.word, &p.ipa, p.rarity, consumed, word_sub_cost)
    }

    fn extend_approx(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        consumed: usize,
        word_sub_cost: f64,
        target_boundaries: &[usize],
        target_words: &HashSet<String>,
    ) -> Self {
        let mut words = self.words.clone();
        words.push(ClueWord {
            word: word.to_string(),
            ipa: ipa.to_string(),
            rarity,
            sub_cost: word_sub_cost,
        });
        let mut spans = self.spans.clone();
        spans.push(consumed);
        let sub_cost_total = self.sub_cost_total + word_sub_cost;
        // Non-additive prefix objective, recomputed whole.
        let mut cum = 0_usize;
        let mut cuts = Vec::with_capacity(spans.len());
        for s in &spans {
            cum += *s;
            cuts.push(cum);
        }
        let cheap_score = Self::objective_score(
            &words,
            &cuts,
            sub_cost_total,
            target_boundaries,
            target_words,
        );
        Self {
            words,
            spans,
            sub_cost_total,
            cheap_score,
            key: self.extended_key(word, ipa),
        }
    }

    /// Historical Exact-mode additive cheap step, preserved as-is.
    fn step_cheap_exact(ipa_len: usize, rarity: Option<f64>) -> f64 {
        let word_term = -0.35 + (ipa_len as f64).min(6.0) / 6.0 * 0.10;
        let rarity_penalty = match rarity {
            Some(r) if r > 5_000.0 => -(0.05 * (r / 5_000.0).log10()).min(0.15),
            _ => 0.0,
        };
        word_term + rarity_penalty
    }

    fn extend_words_exact(
        &self,
        word: &str,
        ipa: &str,
        rarity: Option<f64>,
        consumed: usize,
        word_sub_cost: f64,
    ) -> Self {
        let step = Self::step_cheap_exact(ipa.chars().count(), rarity);
        let mut spans = self.spans.clone();
        spans.push(consumed);
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
            spans,
            sub_cost_total: self.sub_cost_total + word_sub_cost,
            cheap_score: self.cheap_score + step,
            key: self.extended_key(word, ipa),
        }
    }

    /// The final clue score: the generic objective evaluated on the
    /// whole target (target-aligned cuts, symmetric Jaccard).
    fn final_score(&self, target_boundaries: &[usize], target_words: &HashSet<String>) -> f64 {
        Self::objective_score(
            &self.words,
            &self.cuts(),
            self.sub_cost_total,
            target_boundaries,
            target_words,
        )
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
/// slot per distinct IPA parse). Stale heap entries (from
/// superseded clones) are skipped lazily at drain time. Capped so
/// the final re-rank (sort + diversity selection) stays
/// interactive even when the beam produces hundreds of thousands
/// of completions; the cap is generous (thousands) so mid-pack
/// valid parses survive to re-ranking.
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
}

impl CompletionTop {
    fn new(cap: usize) -> Self {
        Self {
            cap,
            heap: std::collections::BinaryHeap::new(),
            seq: 0,
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, candidate: Partial, score: f64) {
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

    fn drain_sorted(mut self) -> Vec<Partial> {
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
    /// Clone key -> (index, cheap). Clone keys are IPA sequences;
    /// same-sound spellings share one slot, best cheap wins.
    keys: HashMap<String, (usize, f64)>,
    /// (word count, cost tier) -> member indices. Each cell is an
    /// independent quota: word counts and cost tiers can never steal
    /// each other's slots, so deep or slightly-off parses always
    /// keep representation no matter how crowded other cells get.
    cell_members: HashMap<(usize, u8), Vec<usize>>,
    cell_min: HashMap<(usize, u8), (usize, f64)>,
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
        self.ending_counts.clear();
        std::mem::take(&mut self.entries)
    }

    /// Beam cell key: word count (segmentation depth) by
    /// acoustic-cost tier (clean / near / far parses never evict
    /// each other).
    fn cell_of(cand_words: usize, cand_sub: f64) -> (usize, u8) {
        (cand_words, cost_tier(cand_sub))
    }

    /// Removed lazy gate: cheap_score is the non-additive prefix
    /// objective, so no arithmetic shortcut can predict it exactly.
    /// Kept for reference/tests; the search loop builds every
    /// budget-admissible extension and lets `insert` arbitrate, which
    /// keeps gate and priority consistent by construction.
    #[allow(dead_code)]
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
    ) -> bool {
        // Clone key mirrors Partial::extended_key exactly: one
        // `norm\x1fipa` token per word, `\x1e`-joined.
        let tok = format!("{}\x1f{match_ipa}", Partial::norm_word(match_word));
        let key = if partial_key_empty {
            tok
        } else {
            format!("{partial_key}\x1e{tok}")
        };
        if self.k == 0 {
            return false;
        }
        let cell = Self::cell_of(cand_words, cand_sub);
        let cell_count = self.cell_members.get(&cell).map_or(0, Vec::len);
        if cell_count >= self.cell_cap
            && !self.cell_admits(cell, cell_count, cand_cheap, cand_sub)
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
                return match self.keys.get(&key) {
                    Some(&(_, cheap)) => cand_cheap > cheap,
                    None => false,
                };
            }
        }
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
    /// only strict improvements (or displacing an epsilon-dominated
    /// member) get in.
    fn cell_admits(
        &self,
        cell: (usize, u8),
        cell_count: usize,
        cand_cheap: f64,
        cand_sub: f64,
    ) -> bool {
        if cell_count < self.cell_cap {
            return true;
        }
        if cell_count < self.cell_hard {
            return match self.cell_min.get(&cell) {
                Some(&(_, min)) => cand_cheap > min - EPSILON,
                // Inconsistent (should not happen); admit.
                None => true,
            };
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
    /// displacement), so early function words like the "a" in "wreck a
    /// nice beach" are unaffected; they just cannot evict parses built
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
    /// Cells grow freely to their soft share; past it, only
    /// non-epsilon-dominated candidates join (up to the hard cap),
    /// and settled cells admit strict improvements or
    /// epsilon-dominating displacements. Mirrors
    /// [`BeamPos::would_admit`].
    fn insert(&mut self, candidate: Partial) {
        if self.k == 0 {
            return;
        }
        let cell = Self::cell_of(candidate.words.len(), candidate.sub_cost_total);
        if let Some(&(idx, cheap)) = self.keys.get(&candidate.key) {
            if candidate.cheap_score <= cheap {
                return;
            }
            // Improvement for a held clone slot: swap in place. The
            // slot keeps its cell even if the improved acoustics
            // would tier differently — relocating across cells
            // breaks the member-vector accounting (duplicate-index
            // buildup), while the score/cheap update itself is what
            // matters for downstream expansion.
            self.replace(idx, candidate);
            return;
        }
        let cell_count = self.cell_members.get(&cell).map_or(0, Vec::len);
        if cell_count < self.cell_cap {
            self.push_new(candidate);
            return;
        }
        if Self::last_ipa_len(&candidate) < Self::PICO_LEN {
            // PicoWord in a full cell: room is gone, near-tie growth
            // stays closed to PicoWords (that is the swamp
            // mechanism), and substantial members are immune — only
            // a same-class slot may turn over to a better PicoWord.
            let victim = self
                .cell_members
                .get(&cell)
                .and_then(|members| {
                    members
                        .iter()
                        .filter(|&&i| Self::last_ipa_len(&self.entries[i]) < Self::PICO_LEN)
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
                self.replace(idx, candidate);
            }
            return;
        }
        if cell_count < self.cell_hard {
            // Near-tie extension: join unless the whole kept cluster
            // beats the candidate by more than EPSILON.
            let near_tie = match self.cell_min.get(&cell) {
                Some(&(_, min)) => candidate.cheap_score > min - EPSILON,
                None => true,
            };
            if near_tie {
                self.push_new(candidate);
                return;
            }
        }
        // Settled cell: displace the worst on a strict heuristic
        // improvement, else a member the candidate epsilon-dominates,
        // else (last resort) a first-of-ending displacement: a
        // candidate whose last-word sound is absent from the cell
        // joins by displacing the cell worst if it is within
        // ENDING_EPS of it. Endings are the perceptually salient
        // part of a clue; this guarantees every distinct ending a
        // foothold without growing the cell.
        let victim = self
            .cell_min
            .get(&cell)
            .filter(|(_, min)| candidate.cheap_score > *min)
            .map(|(idx, _)| *idx)
            .or_else(|| self.dominates_some(cell, candidate.cheap_score, candidate.sub_cost_total));
        if let Some(idx) = victim {
            self.replace(idx, candidate);
            return;
        }
        // Last resort: first-of-ending novelty displaces the cell
        // worst (mirrors the gate's novelty check).
        if let Some(end_ipa) = candidate.words.last().map(|w| w.ipa.clone()) {
            if self.novel_ending(cell, &end_ipa, candidate.cheap_score) {
                if let Some(&(idx, _)) = self.cell_min.get(&cell) {
                    self.replace(idx, candidate);
                }
            }
        }
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
    }

    /// Replace entry at `idx` in place (other indices unchanged).
    /// Usually the slot keeps its cell (clone improvements,
    /// cell-worst displacements); the clone tier-change path may
    /// move a slot across cells, in which case member vectors,
    /// minima and the ending census all follow the entry.
    fn replace(&mut self, idx: usize, candidate: Partial) {
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
        // Refresh minima that may have moved. A stale low minimum
        // would make the gate skip admittable candidates, so when
        // the evicted entry was the minimum, rescan. The new cell
        // always learns the replacement (it may be its new best).
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
            spans: vec![2; nwords],
            sub_cost_total: cost,
            cheap_score: cheap,
            key: key.to_string(),
        }
    }

    /// Settled-cell admission is purely score-ordered: a substantial
    /// candidate above the cell min displaces the min member, with no
    /// cost condition. cheap_score already internalizes phonetic
    /// similarity/cost and the cell key already buckets cost tiers, so
    /// a further cand_sub <= victim.sub_cost requirement would
    /// double-count cost and veto objectively better parses. Below-min
    /// candidates can still arrive via near-tie growth (below the hard
    /// cap) or first-of-ending novelty; the raw-cost Pareto helper
    /// survives only as a fallback for those paths and the dead gate.
    #[test]
    fn settled_cell_score_improvement_replaces_min_regardless_of_cost() {
        fn member(key: &str, cheap: f64, sub: f64) -> Partial {
            Partial {
                words: (0..3)
                    .map(|i| ClueWord {
                        word: format!("{key}w{i}"),
                        ipa: "abc".to_string(),
                        rarity: None,
                        sub_cost: 0.0,
                    })
                    .collect(),
                spans: vec![2; 3],
                sub_cost_total: sub,
                cheap_score: cheap,
                key: key.to_string(),
            }
        }
        let mut beam = BeamPos::new(16); // cell_cap 8, hard cap 64
        // Fill cell (3,1) (3 words, sub 0.30 = tier 1) to the hard cap
        // with rising scores; the min stays the first arrival.
        for i in 0..64 {
            beam.insert(member(&format!("m{i:02}"), 0.80 + i as f64 * 0.0005, 0.30));
        }
        assert_eq!(beam.entries.len(), 64);
        let min_before = beam.cell_min.get(&(3, 1)).map(|(_, m)| *m).unwrap();
        assert!((min_before - 0.80).abs() < 1e-12);
        // Higher score but HIGHER cost, same tier: must displace the
        // min directly, with no cost veto.
        beam.insert(member("newbie", 0.85, 0.45));
        assert_eq!(beam.entries.len(), 64, "hard cap must hold");
        assert!(
            beam.entries.iter().any(|p| p.key == "newbie"),
            "better scorer must displace worst despite higher cost"
        );
        assert!(
            !beam.entries.iter().any(|p| p.key == "m00"),
            "previous min must be evicted"
        );
        let min_after = beam.cell_min.get(&(3, 1)).map(|(_, m)| *m).unwrap();
        assert!(
            min_after > min_before,
            "min must rise after strict-improvement displacement"
        );
        // Lower score: refused even with LOWER cost (ending "abc" is
        // already held, so no novelty fallback either).
        beam.insert(member("cheapskate", 0.79, 0.05));
        assert_eq!(beam.entries.len(), 64);
        assert!(
            !beam.entries.iter().any(|p| p.key == "cheapskate"),
            "worse scorer must not displace"
        );
        // Equal score with higher cost: no strict improvement, no
        // domination, no novel ending -> refused.
        beam.insert(member("equal", min_after, 0.45));
        assert_eq!(beam.entries.len(), 64);
        assert!(
            !beam.entries.iter().any(|p| p.key == "equal"),
            "equal scorer must not displace"
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

    /// Clone identity covers the lexical word as well as its IPA:
    /// two distinct words sharing one IPA ("wreck"/"wreak") must
    /// coexist in a beam position, while an exact same word + same
    /// IPA duplicate collapses (and a better score replaces a worse
    /// one in the shared slot). Without the lexical half, the second
    /// homophone spelling is refused as a "clone" and can never reach
    /// completion — a core Mad Gab semantic, independent of any
    /// particular phrase.
    #[test]
    fn beam_keeps_homophone_spellings_in_distinct_slots() {
        let tb: Vec<usize> = vec![];
        let tw: HashSet<String> = HashSet::new();
        let root = Partial::empty();
        let a = root.extend_approx("wreck", "ɹɛk", None, 3, 0.0, &tb, &tw);
        let b = root.extend_approx("wreak", "ɹɛk", None, 3, 0.0, &tb, &tw);
        assert_ne!(a.key, b.key, "homophone spellings must not share a clone key");
        let mut beam = BeamPos::new(16);
        beam.insert(a);
        beam.insert(b);
        assert_eq!(
            beam.entries.len(),
            2,
            "distinct homophone spellings must both remain in the beam"
        );
        // Exact same word + IPA duplicate collapses to one slot.
        let mut beam2 = BeamPos::new(16);
        let lo = root.extend_approx("wreck", "ɹɛk", None, 3, 0.0, &tb, &tw);
        let mut hi = lo.clone();
        hi.cheap_score += 1.0;
        beam2.insert(lo);
        beam2.insert(hi.clone());
        assert_eq!(beam2.entries.len(), 1, "exact duplicates must collapse");
        assert!(
            (beam2.entries[0].cheap_score - hi.cheap_score).abs() < 1e-12,
            "better score must replace worse in the shared slot"
        );
        // Worse duplicate does not displace the holder.
        let mut beam3 = BeamPos::new(16);
        beam3.insert(hi.clone());
        let lo2 = root.extend_approx("wreck", "ɹɛk", None, 3, 0.0, &tb, &tw);
        beam3.insert(lo2);
        assert_eq!(beam3.entries.len(), 1);
        assert!(
            (beam3.entries[0].cheap_score - hi.cheap_score).abs() < 1e-12,
            "worse duplicate must not displace the holder"
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
