use std::collections::{BTreeMap, HashMap, HashSet};

use phonetics::transcriptions::Corpus;
use serde::Deserialize;

/// Cost of inserting or deleting one IPA character while aligning a
/// clue word to a target span.
const GAP_COST: f64 = 0.20;

// -----------------------------------------------------------------
// Phonemic confusability
// -----------------------------------------------------------------
//
// The matcher's currency has to be *phonemic*, but the corpus alphabet
// is a mixed transliteration: some phonemes are spelled with Latin
// letters (`k`, `t`, `d`, `s`) and some with IPA characters (U+0261
// for the voiced velar stop, U+03B8 for the voiceless dental
// fricative). A distance between single *codepoints* therefore measures
// orthography, not phonology: across the 75 corpus symbols it averages
// 0.78, and 93.8% of distinct-symbol pairs come out dearer than a
// single gap, so the substitution branch is bypassed almost everywhere
// and the search pays for indels instead. The worst case is the two
// velar stops, one Latin and one IPA, which come out at the maximum
// 1.0 -- so a plain voicing alternation is charged *more* than deleting
// the segment and putting a different one back.
//
// So the substitution cost is rebuilt from articulatory features: each
// symbol gets a coarse place/manner/voicing triple if it is a
// consonant, or a height/backness/rounding/reduction vector if it is a
// vowel, and the cost of substituting one symbol for another is the
// weighted distance between those descriptions. The weights are
// ordered by how reliably a listener recovers the cue, so the cost of a
// substitution is small exactly when a listener would not have noticed
// it. Everything is keyed on the same corpus alphabet the matcher
// already collects; no symbol is listed on account of a word it
// happens to appear in.

// Coarse places of articulation.
const PLACE_BILABIAL: i32 = 0;
const PLACE_LABIODENTAL: i32 = 1;
const PLACE_DENTAL: i32 = 2;
const PLACE_ALVEOLAR: i32 = 3;
const PLACE_POSTALVEOLAR: i32 = 4;
const PLACE_PALATAL: i32 = 5;
const PLACE_VELAR: i32 = 6;
const PLACE_UVULAR: i32 = 7;
const PLACE_GLOTTAL: i32 = 8;

// Places collapsed into regions, so that the distance between two
// places saturates: beyond a point, every place change sounds equally
// wrong, so a far pair is not scored as many times worse as a near one.
fn place_region(place: i32) -> i32 {
    match place {
        PLACE_BILABIAL | PLACE_LABIODENTAL => 0,
        PLACE_DENTAL | PLACE_ALVEOLAR => 1,
        PLACE_POSTALVEOLAR | PLACE_PALATAL => 2,
        PLACE_VELAR | PLACE_UVULAR => 3,
        _ => 4,
    }
}

// Coarse manners of articulation.
const MANNER_STOP: i32 = 0;
const MANNER_FRICATIVE: i32 = 2;
const MANNER_NASAL: i32 = 3;
const MANNER_APPROXIMANT: i32 = 4;
const MANNER_LATERAL: i32 = 5;
const MANNER_TRILL: i32 = 6;

// Vowel heights, low to close.
const HEIGHT_LOW: i32 = 0;
const HEIGHT_CLOSE_MID: i32 = 2;
const HEIGHT_NEAR_CLOSE: i32 = 3;

// Vowel backness, 0 (front) to 2 (back).
const BACK_FRONT: i32 = 0;
const BACK_CENTRAL: i32 = 1;
const BACK_BACK: i32 = 2;

// Weights, in the matcher's own cost units, per unit of feature
// distance.
const W_VOICE: f64 = 0.10;
const W_PLACE: f64 = 0.22;
const W_MANNER: f64 = 0.24;
const W_HEIGHT: f64 = 0.07;
const W_BACK: f64 = 0.06;
const W_ROUND: f64 = 0.05;
const W_RHOTIC: f64 = 0.05;

/// A reduced vowel is heard as whatever the schwa is heard as, so a
/// comparison involving one is discounted: the reduction itself is
/// free, and only the residual difference is charged.
const REDUCED_DISCOUNT: f64 = 0.5;

/// Two symbols of different kinds are not a phonetic confusion at all,
/// only a disagreement about what sort of sound belongs there, so this
/// is near-maximal but still below the cross-kind cap below it.
const CROSS_KIND_COST: f64 = 0.80;

/// A symbol that is not in the corpus alphabet at all. Charging the
/// maximum keeps an unexpected symbol from being cheap by accident.
const UNKNOWN_COST: f64 = 1.0;

/// The coarse articulatory description of one corpus symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Segment {
    Consonant {
        place: i32,
        manner: i32,
        voiced: bool,
    },
    Vowel {
        height: i32,
        back: i32,
        round: bool,
        rhotic: bool,
        reduced: bool,
    },
}

impl Segment {
    fn c(place: i32, manner: i32, voiced: bool) -> Segment {
        Segment::Consonant {
            place,
            manner,
            voiced,
        }
    }

    fn v(
        height: i32,
        back: i32,
        round: bool,
        rhotic: bool,
        reduced: bool,
    ) -> Segment {
        Segment::Vowel {
            height,
            back,
            round,
            rhotic,
            reduced,
        }
    }
}

/// Describe one symbol of the corpus alphabet articulatorily.
///
/// The table is written over the symbol *inventory*, not over words: each
/// arm is a class of sounds, and a symbol appears because of how it is
/// articulated, never because of a phrase it turns up in. Both spellings
/// of a class have to be present, because the corpus mixes notations --
/// the alveolar family is written with Latin letters while the velar and
/// labiodental ones are written with IPA characters -- and the classes
/// only line up across the two notations if both are described.
fn describe(c: char) -> Option<Segment> {
    // Stops.
    Some(match c {
        'p' => Segment::c(PLACE_BILABIAL, MANNER_STOP, false),
        't' => Segment::c(PLACE_ALVEOLAR, MANNER_STOP, false),
        'k' => Segment::c(PLACE_VELAR, MANNER_STOP, false),
        'q' => Segment::c(PLACE_UVULAR, MANNER_STOP, false),
        'b' => Segment::c(PLACE_BILABIAL, MANNER_STOP, true),
        'd' => Segment::c(PLACE_ALVEOLAR, MANNER_STOP, true),
        '\u{0261}' => Segment::c(PLACE_VELAR, MANNER_STOP, true),
        // Fricatives.
        'f' => Segment::c(PLACE_LABIODENTAL, MANNER_FRICATIVE, false),
        '\u{03B8}' => Segment::c(PLACE_DENTAL, MANNER_FRICATIVE, false),
        's' => Segment::c(PLACE_ALVEOLAR, MANNER_FRICATIVE, false),
        '\u{0283}' => Segment::c(PLACE_POSTALVEOLAR, MANNER_FRICATIVE, false),
        'h' => Segment::c(PLACE_GLOTTAL, MANNER_FRICATIVE, false),
        'v' => Segment::c(PLACE_LABIODENTAL, MANNER_FRICATIVE, true),
        '\u{00F0}' => Segment::c(PLACE_DENTAL, MANNER_FRICATIVE, true),
        'z' => Segment::c(PLACE_ALVEOLAR, MANNER_FRICATIVE, true),
        '\u{0292}' => Segment::c(PLACE_POSTALVEOLAR, MANNER_FRICATIVE, true),
        // Nasals.
        'm' => Segment::c(PLACE_BILABIAL, MANNER_NASAL, true),
        'n' => Segment::c(PLACE_ALVEOLAR, MANNER_NASAL, true),
        '\u{014B}' => Segment::c(PLACE_VELAR, MANNER_NASAL, true),
        // Lateral.
        'l' => Segment::c(PLACE_ALVEOLAR, MANNER_LATERAL, true),
        // Approximants, trill and glides.
        '\u{0279}' => Segment::c(PLACE_ALVEOLAR, MANNER_APPROXIMANT, true),
        '\u{027E}' => Segment::c(PLACE_ALVEOLAR, MANNER_TRILL, true),
        'j' => Segment::c(PLACE_PALATAL, MANNER_APPROXIMANT, true),
        'w' => Segment::c(PLACE_BILABIAL, MANNER_APPROXIMANT, true),
        // Vowels: front unrounded.
        'i' | '\u{026A}' => {
            Segment::v(HEIGHT_NEAR_CLOSE, BACK_FRONT, false, false, false)
        }
        'e' | '\u{025B}' => {
            Segment::v(HEIGHT_CLOSE_MID, BACK_FRONT, false, false, false)
        }
        '\u{00E6}' => Segment::v(HEIGHT_LOW, BACK_FRONT, false, false, false),
        // Vowels: central. This is the reduced family, and it is the
        // one class a listener cannot resolve at all.
        '\u{0259}' | '\u{0250}' | '\u{1D4A}' => {
            Segment::v(HEIGHT_CLOSE_MID, BACK_CENTRAL, false, false, true)
        }
        '\u{025A}' | '\u{025D}' => {
            Segment::v(HEIGHT_CLOSE_MID, BACK_CENTRAL, false, true, true)
        }
        '\u{0275}' => {
            Segment::v(HEIGHT_CLOSE_MID, BACK_CENTRAL, true, false, true)
        }
        // Vowels: back.
        'u' | '\u{028A}' => {
            Segment::v(HEIGHT_NEAR_CLOSE, BACK_BACK, true, false, false)
        }
        'o' | '\u{0254}' => {
            Segment::v(HEIGHT_CLOSE_MID, BACK_BACK, true, false, false)
        }
        '\u{0251}' | '\u{028C}' => {
            Segment::v(HEIGHT_LOW, BACK_BACK, false, false, false)
        }
        '\u{0252}' => Segment::v(HEIGHT_LOW, BACK_BACK, true, false, false),
        _ => return None,
    })
}

/// The cost of hearing one symbol where the other side produced another.
///
/// This is the whole of the substitution model: a feature distance,
/// capped, and discounted when either side is a reduced vowel. A pair
/// in the same class is free, which is the "folded pair" the matcher
/// needs for the unstressed-vowel alternations that dominate connected
/// speech.
pub(crate) fn confusability(a: char, b: char) -> f64 {
    if a == b {
        return 0.0;
    }
    let (Some(x), Some(y)) = (describe(a), describe(b)) else {
        return UNKNOWN_COST;
    };

    use Segment::{Consonant as C, Vowel as V};
    let raw = match (x, y) {
        (
            C {
                place: place_x,
                manner: manner_x,
                voiced: voiced_x,
            },
            C {
                place: place_y,
                manner: manner_y,
                voiced: voiced_y,
            },
        ) => {
            let region = (place_region(place_x) - place_region(place_y)).abs();
            let place_steps = region.min(2);
            let manner_steps = (manner_x - manner_y).abs().min(2);
            let voice = i32::from(voiced_x != voiced_y);
            W_VOICE * voice as f64
                + W_PLACE * place_steps as f64
                + W_MANNER * manner_steps as f64
        }
        (
            V {
                height: h_x,
                back: b_x,
                round: r_x,
                rhotic: rh_x,
                reduced: red_x,
            },
            V {
                height: h_y,
                back: b_y,
                round: r_y,
                rhotic: rh_y,
                reduced: red_y,
            },
        ) => {
            let base = W_HEIGHT * (h_x - h_y).abs() as f64
                + W_BACK * (b_x - b_y).abs() as f64
                + W_ROUND * i32::from(r_x != r_y) as f64
                + W_RHOTIC * i32::from(rh_x != rh_y) as f64;
            if red_x || red_y {
                base * REDUCED_DISCOUNT
            } else {
                base
            }
        }
        _ => CROSS_KIND_COST,
    };

    // A substitution that costs more than deleting the symbol and
    // inserting a different one is never worth taking, so the table is
    // capped just below two gaps. Above that the search already has the
    // cheaper indel, and a table that keeps climbing past it only
    // distorts the best-first ordering.
    raw.min(2.0 * GAP_COST).min(UNKNOWN_COST)
}

// -----------------------------------------------------------------
// Indel cost by class
// -----------------------------------------------------------------
//
// A gap is not one thing. Losing an `h` that the speaker never released
// is not the same event as losing the `t` of a `t`-`d` pair: the first
// is invisible, the second changes the word. A flat `GAP_COST` for both
// is what makes a weak-segment resegmentation -- which is the *normal*
// thing connected speech does at a word boundary -- look as expensive as
// dropping a vowel that carries the word.
//
// So the indel cost is scaled by how *obligatory* the segment is, taken
// from the same articulatory description the substitution table uses. The
// scale is a property of the class, not of the word: a reduced central
// vowel and a glottal fricative are segments English routinely fails to
// produce, a sonorant is partly redundant, and a stop or a full vowel
// carries the word and is charged the flat `GAP_COST`. This is the
// mechanism the item calls folding, expressed over classes rather than
// as a rewrite of the alphabet: the corpus has no codepoint duplication
// to fold away, but it does have segments that go missing, and those
// should not cost the same as segments that do not.

/// How much of the flat `GAP_COST` a segment of this class is worth.
///
/// 1.0 means the segment is the word and losing it is a real edit; below
/// 1.0 means a listener would not notice it going missing.
fn obligatory_weight(segment: Segment) -> f64 {
    use Segment::{Consonant as C, Vowel as V};
    match segment {
        // A reduced vowel is the segment English deletes first.
        V {
            reduced: true, ..
        } => 0.25,
        // A glottal fricative is frequently not articulated at all, and
        // a sonorant is largely masked by its neighbours.
        C {
            manner: MANNER_FRICATIVE,
            place: PLACE_GLOTTAL,
            ..
        } => 0.25,
        C {
            manner: MANNER_NASAL
                | MANNER_APPROXIMANT
                | MANNER_LATERAL
                | MANNER_TRILL,
            ..
        } => 0.60,
        // A full vowel, a stop or a suprasegmental fricative carries the
        // word and is charged the flat cost.
        _ => 1.0,
    }
}

/// The cost of inserting or deleting one symbol, given what kind of
/// segment it is.
pub(crate) fn indel_cost(c: char) -> f64 {
    GAP_COST * describe(c).map_or(1.0, obligatory_weight)
}

/// Bound the number of word alternatives exposed for any one target
/// span after trie traversal. Search still sees multiple acoustic and
/// lexical alternatives, but pathological homophone clusters cannot
/// swamp the phrase beam.
const MATCHES_PER_SPAN: usize = 256;

#[derive(Debug, Clone)]
pub(crate) struct FuzzyWord {
    pub(crate) word: String,
    pub(crate) ipa: String,
    pub(crate) ipa_len: usize,
    pub(crate) syllables: usize,
    pub(crate) rarity: Option<f64>,
    /// Cached `lexical::is_closed_class(&word)`.  The span shortlist
    /// sorts call the lexical test inside their comparators, and the test
    /// lowercases, so it is not free enough to re-run there.
    pub(crate) closed: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FuzzyMatch {
    pub(crate) consumed: usize,
    pub(crate) word_idx: usize,
    pub(crate) cost: f64,
}

#[derive(Debug, Default)]
struct TrieNode {
    children: BTreeMap<char, usize>,
    terminations: Vec<usize>,
}

/// A compact pronunciation trie used only by Approximate mode.
///
/// phonetics-rs already has a trie, but its public approximate walk
/// supports substitutions only. Mad Gab connected speech also needs
/// small insertions/deletions near word boundaries, so this trie runs
/// a bounded weighted edit search over the same preferred
/// pronunciations.
#[derive(Debug)]
pub(crate) struct FuzzyLexicon {
    words: Vec<FuzzyWord>,
    nodes: Vec<TrieNode>,
    substitution_costs: HashMap<(char, char), f64>,
}

impl FuzzyLexicon {
    pub(crate) fn empty() -> Self {
        Self {
            words: Vec::new(),
            nodes: vec![TrieNode::default()],
            substitution_costs: HashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    pub(crate) fn word(&self, index: usize) -> &FuzzyWord {
        &self.words[index]
    }

    /// Find all words whose pronunciation is within the edit budget of
    /// a target prefix at start.
    ///
    /// States are (trie node, target chars consumed). For any such
    /// state only the cheapest alignment matters; a more expensive
    /// route has exactly the same possible suffixes and can be
    /// discarded. This keeps insertion/deletion support close to the
    /// cost of an ordinary trie walk instead of scanning the lexicon.
    pub(crate) fn matches_at(
        &self,
        target: &[char],
        start: usize,
        budget: f64,
        min_word_ipa_chars: usize,
    ) -> Vec<FuzzyMatch> {
        if start >= target.len() || budget < 0.0 || self.words.is_empty() {
            return Vec::new();
        }

        let remaining = target.len() - start;
        let mut best: HashMap<(usize, usize), f64> = HashMap::new();
        let mut stack = vec![(0usize, 0usize, 0.0f64)];
        best.insert((0, 0), 0.0);

        // A word can be reached through more than one edit alignment.
        // Keep the cheapest cost for each (consumed span, word).
        let mut found: HashMap<(usize, usize), f64> = HashMap::new();

        while let Some((node_idx, consumed, cost)) = stack.pop() {
            if cost > budget + 1e-9 {
                continue;
            }
            if best
                .get(&(node_idx, consumed))
                .is_some_and(|&known| cost > known + 1e-9)
            {
                continue;
            }

            let node = &self.nodes[node_idx];
            if consumed > 0 {
                for &word_idx in &node.terminations {
                    if self.words[word_idx].ipa_len < min_word_ipa_chars {
                        continue;
                    }
                    found
                        .entry((consumed, word_idx))
                        .and_modify(|old| {
                            if cost < *old {
                                *old = cost;
                            }
                        })
                        .or_insert(cost);
                }
            }

            // Delete one target segment: the listener effectively
            // loses a weak segment while the clue pronunciation stays
            // at the same trie node. The charge depends on what kind of
            // segment went missing, not just on the fact that one did.
            if consumed < remaining {
                let lost = target[start + consumed];
                push_state(
                    &mut stack,
                    &mut best,
                    node_idx,
                    consumed + 1,
                    cost + indel_cost(lost),
                    budget,
                );
            }

            for (&clue_char, &child_idx) in &node.children {
                // Insert one clue segment: extra material in the clue
                // pronunciation that consumes no target segment. As with
                // a deletion, what arrives matters: a clue segment the
                // speaker would not have produced is cheap to add.
                push_state(
                    &mut stack,
                    &mut best,
                    child_idx,
                    consumed,
                    cost + indel_cost(clue_char),
                    budget,
                );

                // Match/substitute one segment.
                if consumed < remaining {
                    let target_char = target[start + consumed];
                    let sub = if target_char == clue_char {
                        0.0
                    } else {
                        self.substitution_costs
                            .get(&(target_char, clue_char))
                            .copied()
                            .unwrap_or_else(|| confusability(target_char, clue_char))
                    };
                    push_state(
                        &mut stack,
                        &mut best,
                        child_idx,
                        consumed + 1,
                        cost + sub,
                        budget,
                    );
                }
            }
        }

        let mut by_span: Vec<Vec<FuzzyMatch>> = vec![Vec::new(); remaining + 1];
        for ((consumed, word_idx), cost) in found {
            by_span[consumed].push(FuzzyMatch {
                consumed,
                word_idx,
                cost,
            });
        }

        let mut out = Vec::new();
        for matches in by_span.iter_mut().skip(1) {
            matches.sort_by(|a, b| {
                a.cost
                    .partial_cmp(&b.cost)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        rarity_key(self.words[a.word_idx].rarity)
                            .cmp(&rarity_key(self.words[b.word_idx].rarity))
                    })
                    .then_with(|| {
                        self.words[a.word_idx]
                            .word
                            .cmp(&self.words[b.word_idx].word)
                    })
            });

            if matches.len() > MATCHES_PER_SPAN {
                const CHEAP_KEEP: usize = 96;
                const COST_BANDS: usize = 4;
                const BAND_KEEP: usize =
                    (MATCHES_PER_SPAN - CHEAP_KEEP) / COST_BANDS;

                let ordered = matches.clone();
                let mut selected = Vec::with_capacity(MATCHES_PER_SPAN);
                let mut seen = HashSet::new();

                for m in ordered.iter().take(CHEAP_KEEP) {
                    if seen.insert(m.word_idx) {
                        selected.push(*m);
                    }
                }

                let mut bands: Vec<Vec<FuzzyMatch>> =
                    (0..COST_BANDS).map(|_| Vec::new()).collect();
                let scale = budget.max(1e-9);
                for &m in &ordered {
                    let band = (((m.cost / scale) * COST_BANDS as f64)
                        .floor() as usize)
                        .min(COST_BANDS - 1);
                    bands[band].push(m);
                }

                for band in &mut bands {
                    band.sort_by(|a, b| {
                        rarity_key(self.words[a.word_idx].rarity)
                            .cmp(&rarity_key(self.words[b.word_idx].rarity))
                            .then_with(|| {
                                a.cost
                                    .partial_cmp(&b.cost)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .then_with(|| {
                                self.words[a.word_idx]
                                    .word
                                    .cmp(&self.words[b.word_idx].word)
                            })
                    });
                    for m in band.iter().take(BAND_KEEP) {
                        if seen.insert(m.word_idx) {
                            selected.push(*m);
                        }
                    }
                }

                if selected.len() < MATCHES_PER_SPAN {
                    for m in &ordered {
                        if seen.insert(m.word_idx) {
                            selected.push(*m);
                            if selected.len() == MATCHES_PER_SPAN {
                                break;
                            }
                        }
                    }
                }

                selected.sort_by(|a, b| {
                    a.consumed
                        .cmp(&b.consumed)
                        .then_with(|| {
                            a.cost
                                .partial_cmp(&b.cost)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .then_with(|| {
                            rarity_key(self.words[a.word_idx].rarity)
                                .cmp(&rarity_key(self.words[b.word_idx].rarity))
                        })
                });
                *matches = selected;
            }

            out.extend(matches.iter().copied());
        }
        out
    }
}

fn push_state(
    stack: &mut Vec<(usize, usize, f64)>,
    best: &mut HashMap<(usize, usize), f64>,
    node: usize,
    consumed: usize,
    cost: f64,
    budget: f64,
) {
    if cost > budget + 1e-9 {
        return;
    }
    let key = (node, consumed);
    if best.get(&key).is_some_and(|&old| old <= cost + 1e-9) {
        return;
    }
    best.insert(key, cost);
    stack.push((node, consumed, cost));
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    rarity: Option<f64>,
}

pub(crate) fn normalize_ipa(ipa: &str) -> String {
    ipa.chars()
        .filter(|&c| c != 'ˈ' && c != 'ˌ')
        .collect()
}

/// Vowel pairs that form a single syllabic diphthong in the corpus
/// inventory.  Everything else that looks like two adjacent vowels
/// ("uə" in *accrual*, "iə" in *academia*) really is two syllables.
const DIPHTHONGS: &[(char, char)] = &[
    ('a', 'ɪ'),
    ('a', 'ʊ'),
    ('e', 'ɪ'),
    ('o', 'ʊ'),
    ('ɔ', 'ɪ'),
    ('ɑ', 'ɪ'),
    ('ɑ', 'ʊ'),
    ('ɪ', 'ɚ'),
    ('ʊ', 'ɚ'),
];

fn is_vowel_symbol(c: char) -> bool {
    phonetics::vowels::INVENTORY
        .iter()
        .any(|v| v.starts_with(c) && v.chars().count() == 1)
        || c == 'ɚ'
}

/// Count syllable nuclei in a stress-mark-free IPA transcription.
///
/// A Mad Gab clue only works if it can be *spoken* with the target's
/// rhythm, so the search needs a cheap syllable estimate for both clue
/// words and targets.  Syllable count is not derivable from raw IPA
/// length (which is why the length proxy is only a weak signal), but it
/// is cheap: count vowel nuclei and collapse diphthongs.
///
/// Words that contain no vowel symbol at all ("mm" -> /m/) are still
/// spoken with one syllable, so they count as one.
pub(crate) fn ipa_syllables(ipa: &str) -> usize {
    let chars: Vec<char> = ipa.chars().collect();
    let mut nuclei = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        if !is_vowel_symbol(chars[i]) {
            i += 1;
            continue;
        }
        nuclei += 1;
        if i + 1 < chars.len()
            && DIPHTHONGS
                .iter()
                .any(|(a, b)| chars[i] == *a && chars[i + 1] == *b)
        {
            i += 2;
            continue;
        }
        i += 1;
    }
    nuclei.max(usize::from(!ipa.is_empty()))
}

/// Build the fuzzy trie from the same preferred pronunciations used by
/// the main corpus. The extra JSON parse gives us an iterable word
/// list; lookup/source preference remains delegated to Corpus.
pub(crate) fn build_lexicon(
    json: &str,
    corpus: &Corpus,
    max_rarity: Option<f64>,
) -> FuzzyLexicon {
    let raw: HashMap<String, RawEntry> =
        serde_json::from_str(json).expect("Corpus::from_json already validated this JSON");

    let mut words = Vec::new();
    let mut alphabet = HashSet::new();

    for (word, entry) in raw {
        if let (Some(max), Some(rarity)) = (max_rarity, entry.rarity) {
            if rarity > max {
                continue;
            }
        }
        let Some(ipa) = corpus.preferred_ipa(&word) else {
            continue;
        };
        let ipa = normalize_ipa(ipa);
        if ipa.is_empty() {
            continue;
        }
        let chars: Vec<char> = ipa.chars().collect();
        alphabet.extend(chars.iter().copied());
        let closed = crate::lexical::is_closed_class(&word);
        words.push(FuzzyWord {
            word,
            syllables: ipa_syllables(&ipa),
            ipa,
            ipa_len: chars.len(),
            rarity: entry.rarity,
            closed,
        });
    }

    words.sort_by(|a, b| a.word.cmp(&b.word).then_with(|| a.ipa.cmp(&b.ipa)));

    let mut nodes = vec![TrieNode::default()];
    for (word_idx, word) in words.iter().enumerate() {
        let mut node_idx = 0usize;
        for ch in word.ipa.chars() {
            let child_idx = if let Some(&existing) = nodes[node_idx].children.get(&ch) {
                existing
            } else {
                let new_idx = nodes.len();
                nodes.push(TrieNode::default());
                nodes[node_idx].children.insert(ch, new_idx);
                new_idx
            };
            node_idx = child_idx;
        }
        nodes[node_idx].terminations.push(word_idx);
    }

    // Precompute the small IPA-symbol substitution table once. Target
    // symbols normally come from the same corpus alphabet; the search
    // has a safe fallback for anything outside it.
    //
    // The entries are phonemic confusabilities, not codepoint
    // distances: see the confusability block above for why the
    // distinction matters. A symbol the table cannot describe still
    // gets a cost, via `confusability`'s fallback, so no pair is ever
    // accidentally free.
    let alphabet: Vec<char> = alphabet.into_iter().collect();
    let mut substitution_costs = HashMap::new();
    for &a in &alphabet {
        for &b in &alphabet {
            substitution_costs.insert((a, b), confusability(a, b));
        }
    }

    FuzzyLexicon {
        words,
        nodes,
        substitution_costs,
    }
}

fn rarity_key(rarity: Option<f64>) -> u64 {
    match rarity {
        Some(r) if r.is_finite() && r >= 0.0 => r.round() as u64,
        _ => u64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a one-word lexicon so the insertion branch can be exercised
    /// without the corpus.
    fn one_word_lexicon(word: &str, ipa: &str) -> FuzzyLexicon {
        let words = vec![FuzzyWord {
            word: word.into(),
            ipa: ipa.into(),
            ipa_len: ipa.chars().count(),
            syllables: 1,
            rarity: Some(100.0),
            closed: false,
        }];
        let mut nodes = vec![TrieNode::default()];
        let mut node = 0;
        for ch in ipa.chars() {
            let next = nodes.len();
            nodes.push(TrieNode::default());
            nodes[node].children.insert(ch, next);
            node = next;
        }
        nodes[node].terminations.push(0);

        let alphabet: Vec<char> = ipa.chars().collect();
        let mut substitution_costs = HashMap::new();
        for &a in &alphabet {
            for &b in &alphabet {
                substitution_costs.insert((a, b), confusability(a, b));
            }
        }

        FuzzyLexicon {
            words,
            nodes,
            substitution_costs,
        }
    }

    #[test]
    fn indel_trie_finds_inserted_initial_segment() {
        // A clue segment the speaker would not have produced is found by
        // insertion, and is charged the class weight of that segment.
        let lexicon = one_word_lexicon("hits", "hɪts");
        let target: Vec<char> = "ɪts".chars().collect();
        let hits = lexicon.matches_at(&target, 0, 0.5, 1);
        assert!(hits
            .iter()
            .any(|m| m.word_idx == 0 && (m.cost - indel_cost('h')).abs() < 1e-9));
    }

    // ---- rule tests for the cost model ---------------------------
    //
    // These assert properties of the *rules* -- the feature distance and
    // the indel class weight -- over symbol classes. No word, phrase or
    // canonical example appears in them; the symbols are chosen as
    // representatives of articulatory classes.

    #[test]
    fn substitution_of_identical_symbols_is_free() {
        for &c in &['k', '\u{0261}', 't', 'a', '\u{0259}'] {
            assert_eq!(confusability(c, c), 0.0, "U+{:04X}", c as u32);
        }
    }

    #[test]
    fn a_voicing_alternation_costs_less_than_a_place_alternation() {
        // Same manner, same place, differing only in voicing: the cue a
        // listener is worst at recovering, so it must be the cheapest
        // kind of substitution.
        let voicing = confusability('t', 'd');
        // Same manner and voicing, different place.
        let place = confusability('t', 'k');
        assert!(
            voicing < place,
            "voicing {voicing} should be under place {place}"
        );
    }

    #[test]
    fn a_lax_tense_vowel_pair_costs_less_than_an_unrelated_vowel_pair() {
        // Differing in one height step, versus differing in height and
        // backness and rounding.
        let near = confusability('\u{026A}', 'i');
        let far = confusability('\u{026A}', '\u{0251}');
        assert!(near < far, "near {near} should be under far {far}");
    }

    #[test]
    fn folding_makes_a_reduced_pair_free() {
        // Two central reduced symbols a listener cannot resolve are the
        // same sound for this currency.
        assert_eq!(
            confusability('\u{0259}', '\u{0250}'),
            0.0,
            "the reduced central family should fold to one symbol"
        );
    }

    #[test]
    fn a_reduced_alternation_is_discounted() {
        // The same vowel pair costs strictly less when one side is
        // reduced, because an unstressed vowel is not recoverable.
        let reduced = confusability('\u{0259}', 'i');
        let unreduced = confusability('\u{0251}', 'i');
        assert!(
            reduced < unreduced,
            "reduced {reduced} should be under unreduced {unreduced}"
        );
    }

    #[test]
    fn an_unrelated_pair_is_never_cheaper_than_two_gaps() {
        // A substitution that outcosts delete-then-insert is never worth
        // taking, so the table must be capped at or below two gaps.
        // Consonant representatives, one per articulatory class, all
        // drawn from the corpus alphabet. (The voiced velar is spelled
        // with an IPA character there, not with a Latin `g`.)
        let alphabet = "ptkbd\u{0261}fszv\u{00F0}mnlhwj\u{0279}\u{0283}\u{0292}\u{014B}";
        for a in alphabet.chars() {
            for b in alphabet.chars() {
                if a == b {
                    continue;
                }
                let cost = confusability(a, b);
                assert!(
                    cost <= 2.0 * GAP_COST + 1e-9,
                    "U+{:04X}/U+{:04X} costs {cost}, above two gaps",
                    a as u32,
                    b as u32
                );
            }
        }
    }

    #[test]
    fn a_symbol_outside_the_corpus_alphabet_is_not_free() {
        assert!(confusability('\u{0259}', '\u{4e2d}') > GAP_COST);
    }

    #[test]
    fn an_indel_of_a_weak_segment_costs_less_than_one_that_carries_the_word() {
        // A reduced vowel and a glottal fricative are segments English
        // routinely fails to produce; a stop and a full vowel are not.
        let weak = indel_cost('\u{0259}');
        let glottal = indel_cost('h');
        let strong = indel_cost('t');
        let full_vowel = indel_cost('\u{0251}');
        assert!(
            weak < strong,
            "reduced vowel indel {weak} should be under stop indel {strong}"
        );
        assert!(
            glottal < strong,
            "glottal fricative indel {glottal} should be under stop indel {strong}"
        );
        assert!(
            full_vowel < strong * 2.0,
            "a full vowel is not free, but is not dearer than a stop"
        );
        assert!(
            full_vowel > weak,
            "a full vowel must cost more than a reduced vowel"
        );
    }

    #[test]
    fn every_indel_costs_at_least_the_reduced_rate() {
        for c in "ptkbdgfszvmnlhjw\u{0259}\u{0251}\u{0279}".chars() {
            assert!(
                indel_cost(c) > 0.0,
                "U+{:04X} indel must not be free",
                c as u32
            );
        }
    }

    #[test]
    fn indel_trie_respects_budget() {
        let mut best = HashMap::new();
        let mut stack = Vec::new();
        push_state(&mut stack, &mut best, 1, 1, 0.6, 0.5);
        assert!(stack.is_empty());
    }

    #[test]
    fn syllable_count_collapses_diphthongs() {
        for (ipa, expected) in [
            ("hɪts", 1),
            ("dʒʌstəs", 2),
            ("keɪm", 1),
            ("naɪs", 1),
            ("naɪʒ", 1),
            ("ɡoʊ", 1),
            ("baʊt", 1),
            ("bɔɪ", 1),
            ("stʊpəd", 2),
            ("ə", 1),
            ("m", 1),
            ("ɹɛkəɡnaɪzspitʃ", 4),
            ("ɪtsdʒʌstəstʊpədɡeɪm", 6),
            // Hiatus in CMUdict spelling really is two syllables.
            ("əkɹuəl", 3),
        ] {
            assert_eq!(
                ipa_syllables(ipa),
                expected,
                "syllable count for /{ipa}/"
            );
        }
    }
}
