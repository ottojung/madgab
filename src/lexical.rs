//! Closed-class (function) word detection for candidate clues.
//!
//! A Mad Gab answer has to be *readable* as an answer, not merely
//! pronounceable.  Every objective axis the search optimises is defined
//! over phones, word boundaries and *frequency*, and frequency is the one
//! that actively points the wrong way: `the`, `a`, `it` and `each` are
//! among the most common words in English, so the familiarity axis
//! *rewards* exactly the determiner salad nobody would accept as a
//! puzzle answer.  A closed-class membership test is the missing
//! lexical-class signal, and it is orthogonal to the other axes:
//!
//! * `word_novelty` asks "is this a different word from the target's?"
//!   — a closed-class word can be perfectly novel here and still useless.
//! * `familiarity` asks "is this a common word?" — a determiner scores
//!   1.0 and is therefore *penalised* by the new axis.  The new axis is
//!   close to anti-correlated with it, which is exactly the point.
//! * `lexical_shape_quality` only looks at word length, so it happens to
//!   catch one- and two-letter items but has no idea that `each` is a
//!   determiner while `beach` is not.
//!
//! ## The list
//!
//! The list is the standard closed-class vocabulary of English,
//! partitioned by the category each group belongs to, so it reads as a
//! linguistic claim rather than a word dump.  Membership covers the
//! *lemmas* of those categories; inflected and clitic forms are derived
//! by rule (see [`is_closed_class`]) rather than enumerated, because the
//! productive morphology of function words is what actually shows up in
//! a clue ("i'll", "can't", "he's", "them").
//!
//! Boundaries worth stating explicitly, since they are judgement calls:
//!
//! * **Determiners** include the quantifiers (`some`, `many`, `few`,
//!   `all`, `half`).  They are closed-class, and a clue built from
//!   `much`, `many`, `little` is not a puzzle answer.
//! * **Pronouns** include the indefinite quantified nominals
//!   (`somebody`, `anything`, `nothing`) for the same reason.
//! * **Auxiliaries** include the modals and the semantically empty
//!   verbs `be`, `do` and `have` in every inflected form.
//! * **Conjunctions** include the subordinators (`although`, `because`,
//!   `unless`) and the complementiser `that`.
//! * **Numerals** include the plain cardinals, the tens, the scale words
//!   and the first ten ordinals.
//! * **Particles** are the Phrasal-Verb particles (Quirk et al. 1985).
//!   English has no single authoritative list, so the set is the one
//!   that is small and phonetically robust: the directional and
//!   completive particles that genuinely form phrasal verbs.  Bare
//!   adverbs (`now`, `very`, `also`, `only`) are deliberately *excluded*:
//!   they are an open class, and penalising them would fight the
//!   rhythm and acoustic axes with no linguistic support.
//! * **Existential `there`** is included, because `there is` carries no
//!   content; **manner adverbs** are not.
//!
//! Words that are homographs of function words but genuine content words
//! in their own right (`cant`, `wont`, `lets`, `her`, `round`, `will`,
//! `may`, `like`) are treated as closed class only in the spelling that
//! the linguistics licenses: `cant` (the verb) is content, `can't` is
//! the negated auxiliary.  See [`is_closed_class`].

use std::collections::HashSet;
use std::sync::OnceLock;

/// Articles: the definite and indefinite articles.
const ARTICLES: &[&str] = &["a", "an", "the"];

/// Demonstrative determiners and pronouns.
const DEMONSTRATIVES: &[&str] = &["this", "that", "these", "those"];

/// Possessive determiners.  The corresponding forms are also pronouns,
/// so they are listed in both groups.
const POSSESSIVE_DETERMINERS: &[&str] =
    &["my", "your", "his", "her", "its", "our", "their", "theirs"];

/// Non-pronominal determiners, quantifiers and partitives.
const QUANTIFIERS: &[&str] = &[
    // every / each
    "each", "every", "either", "neither", "both", "all", "another",
    // some / any
    "some", "any", "no", "much", "many", "few", "little", "several",
    "enough", "half", "most", "more", "less", "least",
    // such
    "such", "what", "which", "whose", "whichever",
];

/// Personal and possessive pronouns, plus the reflexive/emphatic series.
const PRONOUNS: &[&str] = &[
    "i", "me", "mine", "myself", "we", "us", "ours", "ourselves", "you",
    "your", "yours", "yourself", "yourselves", "he", "him", "his", "himself",
    "she", "her", "hers", "herself", "it", "its", "itself", "they", "them",
    "their", "theirs", "themselves", "who", "whom", "whoever", "whomever",
    "one", "ones",
];

/// Indefinite quantified nominals and pronominal `some-`/`any-`/`no-`
/// forms.  Traditional grammars class most of these as pronouns.
const INDEFINITE_PRONOUNS: &[&str] = &[
    "someone", "somebody", "something", "somewhere", "anyone", "anybody",
    "anything", "anywhere", "everyone", "everybody", "everything",
    "everywhere",     "nobody", "nothing", "nowhere", "none",
];

/// Prepositions, including the two-word-cum-one types that a clue can
/// reach through a single resegmentation.
const PREPOSITIONS: &[&str] = &[
    "aboard", "about", "above", "across", "after", "against", "along",
    "alongside", "amid", "amidst", "among", "amongst", "around", "at",
    "before", "behind", "below", "beneath", "beside", "besides", "between",
    "beyond", "by", "concerning", "despite", "down", "during", "except",
    "excluding", "for", "from", "in", "inside", "into", "minus", "near",
    "of", "off", "on", "onto", "opposite", "out", "outside", "over", "past",
    "per", "plus", "regarding", "round", "save", "through", "throughout",
    "till", "to", "toward", "towards", "under", "underneath", "unlike",
    "until", "unto", "up", "upon", "versus", "via", "with", "within",
    "without",
];

/// Coordinating, adversative, and subordinating conjunctions, plus the
/// complementiser `that`.
const CONJUNCTIONS: &[&str] = &[
    "and", "but", "or", "nor", "yet", "so", "if", "unless", "though",
    "although", "because", "since", "than", "as", "while", "whereas", "when",
    "where",     "wherever", "whether", "once", "lest", "provided", "excepting",
    "supposing", "granted", "given", "let's",
];

/// The semantically empty verbs in every inflected form, plus the modals
/// and the semi-modals.
const AUXILIARIES_AND_MODALS: &[&str] = &[
    // be
    "be", "am", "is", "are", "was", "were", "been", "being", "'s", "'re",
    // do
    "do", "does", "did", "doing",
    // have
    "have", "has", "had", "having",
    // modals and semi-modals
    "will", "would", "shall", "should", "can", "could", "may", "might",
    "must", "ought", "need", "dare", "used",
    // clitic and contracted auxiliaries
    "'ll", "'ve", "'m", "'d",
    // irregular negations of the modals
    "shan't", "won't", "can't", "cannot", "ain't", "'t",
];

/// Negators and the negative-polarity existential determiners.
const NEGATORS: &[&str] = &["not", "n't", "never", "neither", "nor", "no"];

/// Existential and locative `there`, which carries no content in
/// `there is` / `there were`.
const EXISTENTIALS: &[&str] = &["there"];

/// Cardinal numerals, the tens, the scale words, and the low ordinals.
const NUMERALS: &[&str] = &[
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
    "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
    "sixteen", "seventeen", "eighteen", "nineteen", "twenty", "thirty",
    "forty", "fifty", "sixty", "seventy", "eighty", "ninety", "hundred",
    "thousand", "million", "billion", "trillion", "dozen", "first", "second",
    "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
    "tenth", "eleventh", "twelfth",
];

/// Phrasal-Verb particles (Quirk et al. 1985).  A deliberately small and
/// phonetically robust subset: the directional and completive particles.
const PARTICLES: &[&str] = &[
    "away", "aside", "apart", "back", "backward", "down", "forward", "in",
    "off", "on", "out", "over", "through", "together", "up",
];

/// Every closed-class lemma, as a set.
static LEMMAS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn lemmas() -> &'static HashSet<&'static str> {
    LEMMAS.get_or_init(|| {
        let mut set: HashSet<&'static str> = HashSet::new();
        for group in [
            ARTICLES,
            DEMONSTRATIVES,
            POSSESSIVE_DETERMINERS,
            QUANTIFIERS,
            PRONOUNS,
            INDEFINITE_PRONOUNS,
            PREPOSITIONS,
            CONJUNCTIONS,
            AUXILIARIES_AND_MODALS,
            NEGATORS,
            EXISTENTIALS,
            NUMERALS,
            PARTICLES,
        ] {
            set.extend(group.iter().copied());
        }
        set
    })
}

/// Is `word` a closed-class English word?
///
/// Membership is decided on the lemma, and the productive morphology of
/// function words is derived rather than tabulated:
///
/// 1. the word is a lemma;
/// 2. the word is a lemma plus an enclitic clitic — `i`+`'ll` = `i'll`,
///    `he`+`'s` = `he's`, `we`+`'re` = `we're`, `i`+`'ve`, `they`+`'d`;
/// 3. the word is a lemma plus the negative clitic `n't` —
///    `do`+`n't` = `don't`, `is`+`n't` = `isn't`.
/// (4) is only tried when the word actually contains an apostrophe, so
/// the genuine content-word homographs `cant`, `wont`, `wont`, `im`,
/// `lets`, `d'ye`-style spellings and `aunt` are *not* caught.  A
/// closed-class word never appears in a clue without its apostrophe.
///
/// The empty (zero-word) clitics `''s`/`'ll`/`'ve`/`'re`/`'m`/`'d` are
/// listed as lemmas in [`AUXILIARIES_AND_MODALS`] so that a clue
/// consisting of a bare clitic is recognised too.
pub fn is_closed_class(word: &str) -> bool {
    // The corpus keys follow the dictionary's display convention, so a
    // function word can be spelled with trailing punctuation (`i.`,
    // `a.'s`).  Strip the non-alphabetic margins before testing, exactly
    // as `clean_input_word` does for target words.
    let trimmed = word.trim().trim_matches(|c: char| !c.is_alphabetic());
    let lower = trimmed.to_lowercase();
    if lower.is_empty() {
        return false;
    }
    if matches_spelling(&lower) {
        return true;
    }
    // Only a genuine contraction licenses the apostrophe-free spelling
    // (`can't` -> `cant`), and only for the negative clitic: `isn't` is
    // the lemma `is` plus `n't`, whereas a bare `cant` is the verb.
    if let Some(base) = lower.strip_suffix("n't") {
        return lemmas().contains(base);
    }
    false
}

/// Does this exact spelling denote a closed-class word?
fn matches_spelling(lower: &str) -> bool {
    let set = lemmas();
    if set.contains(lower) {
        return true;
    }
    // Clitic forms of a closed-class lemma: i'll, he's, we're, i've,
    // they'd, that's, there's.
    for clitic in ["'s", "'ll", "'ve", "'re", "'m", "'d"] {
        if let Some(base) = lower.strip_suffix(clitic) {
            if set.contains(base) {
                return true;
            }
        }
    }
    // The negative clitic on a written apostrophe: don't, isn't, can't.
    if let Some(base) = lower.strip_suffix("n't") {
        if set.contains(base) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_ordinary_closed_class_words() {
        for word in [
            // articles, determiners, quantifiers
            "a", "an", "the", "this", "that", "these", "those", "some", "any",
            "no", "each", "every", "either", "neither", "another", "both",
            "all", "half", "much", "many", "few", "little", "such", "what",
            "which", "whose", "my", "your", "his", "its", "our", "their",
            // pronouns
            "i", "me", "mine", "myself", "we", "us", "ours", "you", "he",
            "him", "hers", "it", "itself", "they", "them", "theirs", "who",
            "whom", "someone", "anybody", "everything", "nothing", "nobody",
            // prepositions
            "of", "in", "on", "at", "by", "for", "with", "about", "to",
            "from", "into", "through", "over", "under", "between", "without",
            // conjunctions
            "and", "but", "or", "nor", "so", "yet", "if", "unless", "though",
            "although", "because", "while", "when", "as", "than",
            // auxiliaries and modals
            "be", "am", "is", "are", "was", "were", "been", "being", "do",
            "does", "did", "have", "has", "had", "will", "would", "shall",
            "should", "can", "could", "may", "might", "must",
            // negation and existential
            "not", "never", "neither", "there",
            // numerals
            "zero", "one", "two", "three", "twelve", "twenty", "hundred",
            "thousand", "first", "second", "third", "tenth",
            // particles
            "up", "down", "out", "off", "away", "together", "back",
        ] {
            assert!(is_closed_class(word), "expected {word:?} to be closed class");
        }
    }

    #[test]
    fn rejects_ordinary_content_words() {
        for word in [
            // the words of ordinary phrases
            "wreck", "nice", "beach", "hits", "justice", "dupe", "hid",
            "came", "recognize", "speech", "game", "stupid", "love",
            "lot", "tess", "thus", "bad", "two_bad", "cut", "pad", "aim",
            "view", "trouble", "whole",
            // open-class words that merely happen to be short
            "ate", "bee", "ode", "odd", "ore", "owe", "row", "saw", "sea",
            "tea", "too", "cat", "kit", "hat", "mat", "rat", "sat", "bat",
            "ace", "aid", "air", "arc", "arm", "art", "ash", "axe", "bay",
            "bed", "bid", "bit", "bog", "bow", "box", "boy", "bud", "bug",
            "cap", "car", "cot", "cow", "cry", "cut", "dam", "dig", "dip",
            "doe", "dry", "ear", "end", "eye", "fan", "far", "fat", "fig",
            "fin", "fir", "fit", "fox", "fun", "gap", "gem", "gum", "ham",
            "hay", "hen", "hip", "hit", "hot", "ice", "ink", "ink", "jam",
            "jaw", "jet", "job", "joy", "key", "kid", "lab", "lad", "lap",
            "law", "lay", "leg", "lid", "lip", "lit", "log", "man", "map",
            "met", "mix", "mop", "mow", "mud", "mug", "nap", "net", "nil",
            "nod", "nut", "oak", "oat", "oil", "old", "owl", "pad", "pan",
            "pat", "pay", "pen", "pet", "pie", "pig", "pin", "pit", "pod",
            "pop", "pot", "pub", "rag", "ram", "rap", "ray", "red", "rib",
            "rid", "rim", "rip", "rob", "rod", "rot", "rub", "rug", "rum",
            "run", "sad", "sap", "say", "see", "set", "shy", "sin", "sip",
            "sir", "sit", "ski", "sky", "son", "sow", "spy", "sub",
            "sum", "sun", "tag", "tan", "tap", "tar", "tax", "tee",
            "tie", "tin", "tip", "toe", "ton", "top", "tow", "toy", "try",
            "tub", "tug", "urn", "use", "van", "vat", "vet", "vie", "vow",
            "wag", "war", "wax", "way", "web", "wet", "wig", "win", "wit",
            "wok", "woo", "wow", "yam", "yaw", "yen", "yes", "yew", "zoo",
            // open-class adverbs: an open class, deliberately not penalised
            "now", "very", "also", "only", "here", "always", "often",
            "still", "even", "quite", "well", "fast", "soon", "hard",
            // content nouns, verbs and adjectives
            "child", "hand", "head", "face", "night", "light", "run",
            "runs", "ran", "jump", "jumps", "walk", "big", "small", "red",
            "blue", "good", "best", "slow", "beach", "speech", "justice",
        ] {
            assert!(
                !is_closed_class(word),
                "expected {word:?} to be a content word"
            );
        }
    }

    #[test]
    fn catches_inflected_and_clitic_forms_of_function_words() {
        for word in [
            "i'll", "I'll", "I'll", "i've", "i'm", "i'd", "we're", "they're",
            "he's", "she's", "it's", "that's", "there's", "let's", "you'll",
            "don't", "doesn't", "didn't", "isn't", "aren't", "wasn't",
            "weren't", "hasn't", "haven't", "hadn't", "won't", "can't",
            "couldn't", "shouldn't", "wouldn't", "mustn't", "ain't",
            "shan't", "oughtn't",
        ] {
            assert!(
                is_closed_class(word),
                "expected the inflected function word {word:?} to be caught"
            );
        }
    }

    #[test]
    fn does_not_catch_content_words_that_merely_end_like_a_clitic() {
        for word in [
            // -'d / -'ll / -'ve / -'m look like clitics, but the stem is a
            // content word so the spelling is a content word.
            "herd", "card", "hard", "heard", "beard", "aimed", "seemed",
            // -n't homographs: the verb `cant`, not the negation `can't`.
            "cant", "wont", "dont", "isnt", "arent", "wasnt", "hasnt",
            "havent", "couldnt", "shouldnt", "wouldnt", "mustnt",
            // -'s lookalikes with a plural or lexical stem
            "years", "days", "hours", "states", "games", "works", "kids",
            // apostrophe-free content words that are not contractions
            "aim", "aunt", "ocean", "idea", "area", "quiet", "diet",
            // ordinary inflected content words
            "aims", "aimed", "aiming", "runs", "running", "jumps", "jumping",
            "watches", "watches'", "boxes", "glasses",
        ] {
            assert!(
                !is_closed_class(word),
                "content word {word:?} must not be treated as closed class"
            );
        }
    }

    #[test]
    fn is_case_insensitive_and_trims_punctuation() {
        assert!(is_closed_class("The"));
        assert!(is_closed_class("THE"));
        assert!(is_closed_class(" Each "));
        assert!(is_closed_class("Each,"));
        assert!(!is_closed_class(""));
        assert!(!is_closed_class("   "));
    }
}
