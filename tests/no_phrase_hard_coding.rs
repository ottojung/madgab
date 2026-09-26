//! The no-hard-coding fence for the two canonical Mad Gab examples.
//!
//! `tests/corpus_integration.rs` asserts that approximate search finds
//!
//! - `"It's just a stupid game"` -> `"hits justice dupe hid came"`, and
//! - `"recognize speech"` -> `"wreck a nice beach"`.
//!
//! The itinerary forbids solving those two examples by special-casing them,
//! and the only enforcement today is a human reading a diff. This file makes
//! the forbidden shortcut fail the suite instead, at the moment the milestone
//! is blocked and the shortcut looks most attractive.
//!
//! # What is detected
//!
//! The point is *behavioural coupling to one of those two examples*, not a
//! vocabulary. A bare occurrence of `came`, `hid`, `bad` or `aim` in
//! production code is normal English, and a rule that fired on one would be
//! noise the next reviewer deletes. So nothing here matches a single word on
//! its own. What it matches is a phrase taking one of the shapes a hard-code
//! actually takes, and it reports *which shape* so a failure is actionable:
//!
//! | shape | what it looks like in code |
//! |---|---|
//! | `whole-sentence-equality` | a canonical clue/target literal standing alone as the value: `return "wreck a nice beach";`, `const X: &str = "..."` |
//! | `lookup-keyed-by-target-text` | a full phrase literal used as a `match` arm, map key, or table cell: `"wreck a nice beach" => ...` |
//! | `comparison-against-target-text` | `if clue == "hits justice dupe hid came"`, i.e. the literal is compared against a runtime value |
//! | `comparison-against-normalized-target` | the same, but against the *normalized* form: `if sig == "wreckanicebeach"` |
//! | `full-phrase-literal` | any other literal spelling out a whole canonical phrase, including one spread over a list: `vec!["hits", "justice", "dupe", "hid", "came"]`, `format!("wreck a nice beach")` |
//! | `substring-special-case` | a string method fed a multi-word piece of a canonical phrase: `clue.contains("nice beach")` |
//! | `phrase-substring-literal` | any other literal holding *three* consecutive words of a canonical phrase: `"wreck a nice"` |
//! | `identifier-named-after-phrase` | no literal at all: `fn wreck_a_nice_beach()`, `const RECOGNIZESPEECH` |
//! | `runtime-normalization-comparison` | no literal at all: `if phrase_signature(&clue) == self.expected_signature`, i.e. "is this the example I know?" |
//!
//! Two thresholds do the work of keeping this a coupling test rather than a
//! vocabulary test:
//!
//! * **A single word never fires.** `came`, `hid`, `bad`, `aim` and `wreck`
//!   are ordinary words that production code may legitimately contain.
//! * **Two words fire only where a literal is used as a key** — inside a
//!   string method call such as `contains`. Two words are a collocation
//!   (`"stupid game"`), and a collocation can appear in prose by chance. As a
//!   bare literal a coupling needs three consecutive words, which chance
//!   does not produce.
//!
//! Multi-line literals are covered: `vec![` on one line and `]` three lines
//! later is one unit, and the finding is reported at the unit's first line. A
//! phrase spread across several literals of one statement is covered too,
//! since no single literal in `vec!["hits", "justice", ...]` holds it.
//!
//! A unit reports at most one finding — the highest-priority shape that
//! matched — because one hard-coded function is one problem to fix, not five.
//! The unit is a bracket-balanced statement, so a whole function body is one
//! unit and the finding is reported at its `fn` line.
//!
//! The last two shapes exist because a hard-code need not contain the phrase
//! in a string at all; a name or a signature comparison is enough to couple
//! the search to one example.
//!
//! # What is excluded, and why
//!
//! * **`#[cfg(test)]` modules inside `src/`.** `src/lexical.rs` and
//!   `src/lib.rs` legitimately name the phrases when asserting that stems,
//!   signatures and IPA collapse correctly. The scan stops at the `mod` a
//!   top-level `#[cfg(test)]` introduces and resumes at its closing brace, so
//!   production code between two test modules is still scanned.
//! * **Comments and doc comments, including the `src/main.rs` doc examples.**
//!   The documented usage of the CLI names the two canonical examples, and
//!   prose about a phrase is not behaviour. Comment text is stripped before
//!   anything is inspected.
//! * **The other files in `tests/`.** Only `src/` is read. The acceptance
//!   tests in `tests/corpus_integration.rs` must name the phrases; that is
//!   their job.
//! * **`Cargo.toml`, `examples/`, `web/`.** Not the production region.
//!
//! # The allowlist
//!
//! A legitimate phrase-specific literal in production code goes in
//! `ALLOWLIST` below as an entry naming the file, the line, and *why* it is
//! legitimate. There is no wildcard form: an entry either names one file and
//! one line or it does not apply, and adding one is a visible, greppable
//! edit (`git grep -n RECOGNIZED tests/no_phrase_hard_coding.rs`) that a
//! reviewer sees in the diff.
//!
//! The allowlist is expected to stay short, ideally empty. It is empty today
//! and should stay empty: every legitimate use of the phrases is already
//! covered by the region exclusions above, so a new entry means either a new
//! kind of legitimate coupling (argue for it in review, and keep it to one
//! line) or, more likely, a hard-code that is trying to buy a pass. If two
//! or more entries ever appear, the region boundary above is probably in the
//! wrong place and should be widened instead.
//!
//! # Running it
//!
//! ```text
//! cargo test --release --test no_phrase_hard_coding
//! ```
//!
//! It reads source and does nothing else: no corpus, no release build of
//! the library, no search. It takes milliseconds.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The two canonical targets, as normalized word sequences.
const CANONICAL_TARGETS: &[&[&str]] = &[
    &["its", "just", "a", "stupid", "game"],
    &["recognize", "speech"],
];

/// The canonical clue each target must produce, as normalized word sequences.
const CANONICAL_CLUES: &[&[&str]] = &[&["hits", "justice", "dupe", "hid", "came"], &[
    "wreck", "a", "nice", "beach",
]];

/// Longest common subsequence length between two normalized word sequences.
fn word_overlap(literal: &[&str], phrase: &[&str]) -> usize {
    let mut prev = vec![0usize; phrase.len() + 1];
    let mut cur = vec![0usize; phrase.len() + 1];
    for w in literal {
        for (j, p) in phrase.iter().enumerate() {
            cur[j + 1] = if w == p {
                prev[j] + 1
            } else {
                cur[j].max(prev[j + 1])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
        cur.iter_mut().for_each(|v| *v = 0);
    }
    prev[phrase.len()]
}

/// A phrase the fence is watching, and how much of it a literal spells out.
#[derive(Clone)]
struct Coupling {
    /// Normalized words of the watched phrase.
    words: Vec<&'static str>,
    /// `true` for a target phrase, `false` for a canonical clue.
    is_target: bool,
    /// `true` if the literal spells out the whole phrase.
    whole: bool,
    /// Words matched, for the message.
    matched: usize,
}

fn couplings(literal: &[&str]) -> Vec<Coupling> {
    let mut out = Vec::new();
    for phrase in CANONICAL_TARGETS.iter().chain(CANONICAL_CLUES.iter()) {
        let n = word_overlap(literal, phrase);
        // Two words is already far past chance: it cannot happen to an
        // ordinary sentence, whereas one word happens constantly.
        if n >= 2 {
            out.push(Coupling {
                words: phrase.to_vec(),
                is_target: CANONICAL_TARGETS.contains(phrase),
                whole: n >= phrase.len(),
                matched: n,
            });
        }
    }
    out
}

/// Normalized, whitespace-split words of a string literal's contents.
fn literal_words(contents: &str) -> Vec<String> {
    contents
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// The double-quoted literals in a code string, as `(start, end, body)`,
/// where `end` is the index of the closing quote. Char literals are skipped so
/// that `c == '"'` is not read as a string.
fn literals(code: &str) -> Vec<(usize, usize, String)> {
    let chars: Vec<char> = code.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\'' {
            // A char literal or a lifetime: skip to its closer.
            i += 1;
            while i < chars.len() && chars[i] != '\'' {
                i += if chars[i] == '\\' { 2 } else { 1 };
            }
            i += 1;
            continue;
        }
        if chars[i] == '"' {
            let start = i;
            i += 1;
            let mut body = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                }
                body.push(chars[i]);
                i += 1;
            }
            let end = i;
            i += 1;
            out.push((start, end, body));
        } else {
            i += 1;
        }
    }
    out
}

/// Everything a detector needs from one contiguous run of source lines.
#[derive(Debug, Clone)]
struct Unit {
    file: String,
    /// 1-based line where the unit starts.
    line: usize,
    /// Source with comments and literal contents blanked out, so the
    /// remaining text is the *code* around the literals.
    code: String,
    /// The literals themselves, in order of appearance.
    literals: Vec<String>,
}

impl Unit {
    fn new(file: &str, line: usize, raw: &str) -> Unit {
        let stripped = strip_comments(raw);
        let spans = literals(&stripped);
        let lits: Vec<String> = spans.iter().map(|(_, _, body)| body.clone()).collect();
        // Replace each literal, quotes included, with a marker, so the text
        // that is left is the code *around* the phrase.
        let mut chars: Vec<char> = stripped.chars().collect();
        for (start, end, _) in spans.into_iter().rev() {
            let hi = (end + 1).min(chars.len());
            chars.splice(start..hi, "<literal>".chars());
        }
        let code: String = chars.into_iter().collect();
        Unit {
            file: file.to_string(),
            line,
            code,
            literals: lits,
        }
    }

}

/// Blank out comment text, preserving length and character positions so
/// that the literals and the code around them can still be located. Handles
/// `//`, `///`, `//!` and `/* ... */` (including a block comment
/// spanning lines), and does not mistake a `//` inside a string or a char
/// literal for a comment. Literal *contents* are left alone: the caller
/// consumes them.
fn strip_comments(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out: Vec<char> = chars.clone();
    let mut i = 0;
    let mut state = State::Code;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied().unwrap_or('\0');
        match state {
            State::Code => {
                if c == '/' && next == '/' {
                    out[i] = ' ';
                    out[i + 1] = ' ';
                    i += 2;
                    state = State::LineComment;
                    continue;
                }
                if c == '/' && next == '*' {
                    out[i] = ' ';
                    out[i + 1] = ' ';
                    i += 2;
                    state = State::BlockComment;
                    continue;
                }
                if c == '\'' {
                    i += 1;
                    while i < chars.len() && chars[i] != '\'' {
                        i += if chars[i] == '\\' { 2 } else { 1 };
                    }
                    i += 1;
                    continue;
                }
                if c == '"' {
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        i += if chars[i] == '\\' { 2 } else { 1 };
                    }
                    i += 1;
                    continue;
                }
            }
            State::LineComment => {
                if c == '\n' {
                    state = State::Code;
                } else {
                    out[i] = ' ';
                }
            }
            State::BlockComment => {
                if c == '*' && next == '/' {
                    out[i] = ' ';
                    out[i + 1] = ' ';
                    i += 2;
                    state = State::Code;
                    continue;
                }
                if c != '\n' {
                    out[i] = ' ';
                }
            }
        }
        i += 1;
    }
    out.into_iter().collect()
}

#[derive(PartialEq, Clone, Copy)]
enum State {
    Code,
    LineComment,
    BlockComment,
}

/// One place where the fence fired.
#[derive(Debug)]
struct Finding {
    file: String,
    line: usize,
    shape: &'static str,
    detail: String,
}

/// One-based line numbers that are inside a `#[cfg(test)]` item, and so are
/// test code rather than production code.
fn test_lines(lines: &[&str]) -> HashSet<usize> {
    let mut marked = HashSet::new();
    let indent_of = |l: &str| l.len() - l.trim_start().len();
    for (i, line) in lines.iter().enumerate() {
        if line.trim() != "#[cfg(test)]" {
            continue;
        }
        let indent = indent_of(line);
        let next = lines[i + 1..]
            .iter()
            .position(|l| !l.trim().is_empty())
            .map(|d| i + 1 + d);
        let Some(next) = next else { continue };
        let header = lines[next];
        if indent_of(header) == indent && header.trim_start().starts_with("mod ") {
            // A test module: everything up to its own closing brace, which is
            // the first line at the same indent consisting only of `}`.
            for (j, l) in lines.iter().enumerate().skip(next) {
                marked.insert(j + 1);
                if indent_of(l) == indent && l.trim() == "}" {
                    break;
                }
            }
        } else {
            // A `#[cfg(test)]` attribute on a single statement.
            marked.insert(i + 1);
        }
    }
    marked
}

/// Lines of one source file, split into units: a run of lines forming one
/// logical expression, so a `vec![` opened on one line and closed three
/// lines later is scanned as a whole. Units inside a `#[cfg(test)]` item are
/// dropped.
fn units_of(file: &str, src: &str) -> Vec<Unit> {
    let lines: Vec<&str> = src.split('\n').collect();
    let test = test_lines(&lines);
    let mut units: Vec<Unit> = Vec::new();
    let mut start: Option<usize> = None;
    let mut depth: i32 = 0;
    let mut text = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if start.is_none() {
            start = Some(idx);
        }
        text.push_str(line);
        text.push('\n');
        depth += bracket_delta(&strip_comments(line));
        if depth <= 0 {
            let first = start.unwrap() + 1;
            if !test.contains(&first) {
                units.push(Unit::new(file, first, &text));
            }
            depth = 0;
            start = None;
            text.clear();
        }
    }
    if let Some(first) = start.map(|s| s + 1) {
        if !test.contains(&first) {
            units.push(Unit::new(file, first, &text));
        }
    }
    units
}

fn bracket_delta(code: &str) -> i32 {
    let mut delta = 0i32;
    let mut in_str = false;
    let mut prev_escape = false;
    for c in code.chars() {
        if in_str {
            if prev_escape {
                prev_escape = false;
            } else if c == '\\' {
                prev_escape = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '[' | '{' | '(' => delta += 1,
            ']' | '}' | ')' => delta -= 1,
            _ => {}
        }
    }
    delta
}

/// Phrase-shaped things this unit does: whole phrases, and partial runs.
struct Phrases {
    /// (literal, coupling) pairs spelling out a whole watched phrase.
    whole: Vec<(String, Coupling)>,
    /// (literal, coupling) pairs holding two or more words of one.
    partial: Vec<(String, Coupling)>,
    /// Any literal spelled without separators, i.e. a normalized phrase.
    normalized_full: Option<String>,
}

fn phrase_coupling(unit: &Unit) -> Phrases {
    let mut whole = Vec::new();
    let mut partial = Vec::new();
    let mut normalized_full = None;
    for lit in &unit.literals {
        let words = literal_words(lit);
        let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        for c in couplings(&refs) {
            if c.whole {
                whole.push((lit.clone(), c));
            } else {
                partial.push((lit.clone(), c));
            }
        }
        // A phrase with no separator characters at all is its normalized form:
        // `wreckanicebeach` rather than `wreck a nice beach`. Note that
        // `<[&str] as Concat>::concat()` joins with no separator, so this
        // compares normalized spellings, not spaced ones.
        let flat: String = refs.concat();
        let no_separators = !lit.is_empty() && lit.chars().all(|c| c.is_alphanumeric());
        for phrase in CANONICAL_TARGETS.iter().chain(CANONICAL_CLUES.iter()) {
            if no_separators && flat == phrase.concat() {
                normalized_full = Some(lit.clone());
                whole.push((
                    lit.clone(),
                    Coupling {
                        words: phrase.to_vec(),
                        is_target: CANONICAL_TARGETS.contains(phrase),
                        whole: true,
                        matched: phrase.len(),
                    },
                ));
            }
        }
    }
    // A phrase can be spread over several literals in one statement, which is
    // what `vec!["hits", "justice", "dupe", "hid", "came"]` looks like: no
    // single literal holds the phrase, but the statement does.
    let combined: Vec<String> = unit
        .literals
        .iter()
        .flat_map(|l| literal_words(l))
        .collect();
    let combined_refs: Vec<&str> = combined.iter().map(|s| s.as_str()).collect();
    let spelled: Vec<String> = unit
        .literals
        .iter()
        .map(|l| format!("{l:?}"))
        .collect();
    for c in couplings(&combined_refs) {
        if c.whole {
            whole.push((spelled.join(", "), c));
        }
    }
    Phrases {
        whole,
        partial,
        normalized_full,
    }
}
fn looks_like_lookup(code: &str) -> bool {
    code.contains("=>")
        || code.contains("match ")
        || code.contains("insert(")
        || code.contains(".get(")
        || code.contains("HashMap")
        || code.contains("BTreeMap")
}

fn looks_like_comparison(code: &str) -> bool {
    code.contains("==") || code.contains("!=")
}

fn looks_like_substring_call(code: &str) -> bool {
    [
        "contains(",
        "starts_with(",
        "ends_with(",
        "strip_prefix(",
        "strip_suffix(",
        "replace(",
        "replacen(",
        "split(",
        "splitn(",
        "trim_matches(",
        "find(",
    ]
    .iter()
    .any(|m| code.contains(m))
}

/// Is the literal handed back verbatim, with nothing computed from it? True
/// for `return "...";`, `let x = "...";` and a `const NAME: &str = "...";`,
/// false for anything that inspects, transforms or compares it.
fn is_standalone_value(code: &str) -> bool {
    let Some((before, _)) = code.split_once("<literal>") else {
        return false;
    };
    let head = before.trim();
    let head_ok = head.is_empty()
        || head.ends_with("return")
        || head.ends_with('=')
        || ["let ", "const ", "static ", "fn ", "pub "]
            .iter()
            .any(|k| head.contains(k));
    let after = code
        .rsplit_once("<literal>")
        .map(|(_, a)| a.trim())
        .unwrap_or("");
    let after_ok = after
        .chars()
        .all(|c| c == ';' || c == '}' || c == ',' || c.is_whitespace());
    head_ok && after_ok
}

/// Identifiers on this unit, as normalized word sequences, so that
/// `fn wreck_a_nice_beach` reads as `wreck a nice beach`.
fn identifier_words(code: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut words: Vec<String> = Vec::new();
    for c in code.chars() {
        if c.is_alphanumeric() {
            current.push(c);
        } else if c == '_' {
            if !current.is_empty() {
                words.push(current.to_lowercase());
            }
            current.clear();
        } else {
            if !current.is_empty() {
                words.push(current.to_lowercase());
            }
            current.clear();
            if !words.is_empty() {
                out.push(std::mem::take(&mut words));
            }
        }
    }
    if !current.is_empty() {
        words.push(current.to_lowercase());
    }
    if !words.is_empty() {
        out.push(words);
    }
    out
}

/// A statement that compares a normalized form against a value that was
/// remembered rather than recomputed, which is how a hard-code recognizes one
/// target without ever naming it in a string. Comparing two normalized forms
/// to each other is ordinary de-duplication, so exactly one normalization
/// call is required.
fn runtime_normalization_comparison(code: &str) -> bool {
    const CALLS: &[&str] = &[
        "phrase_signature(",
        "normalized_word(",
        "signature(",
        "normalize(",
        "normalize_ipa(",
    ];
    // Count calls without double counting: `phrase_signature(` also contains
    // `signature(`, so take the earliest match and resume after it.
    let count_calls = |s: &str| {
        let mut n = 0;
        let mut at = 0;
        while at < s.len() {
            let hit = CALLS
                .iter()
                .filter_map(|c| s[at..].find(c).map(|i| (at + i, c.len())))
                .min_by_key(|(i, _)| *i);
            match hit {
                Some((i, len)) => {
                    n += 1;
                    at = i + len;
                }
                None => break,
            }
        }
        n
    };
    code.split(|c| c == ';' || c == '\n')
        .filter(|s| s.contains("==") || s.contains("!="))
        .any(|s| count_calls(s) == 1 && !s.contains("<literal>"))
}

/// Run the detector over one unit. Returns at most one finding: the highest
/// priority shape that matched, since a single hard-code line is one problem.
fn detect(unit: &Unit) -> Option<Finding> {
    let phrases = phrase_coupling(unit);
    let make = |shape: &'static str, detail: String| Finding {
        file: unit.file.clone(),
        line: unit.line,
        shape,
        detail,
    };

    if let Some((lit, coupling)) = phrases.whole.first() {
        let kind = if coupling.is_target { "target" } else { "clue" };
        let quoted = format!("{:?}", lit);
        if phrases.normalized_full.is_some() {
            if looks_like_comparison(&unit.code) {
                return Some(make(
                    "comparison-against-normalized-target",
                    format!(
                        "{quoted} is the normalized {kind} phrase and is compared \
                         against a runtime value; a canonical example is being \
                         recognized by its normalized text"
                    ),
                ));
            }
            return Some(make(
                "full-phrase-literal",
                format!("{quoted} spells out the whole normalized {kind} phrase"),
            ));
        }
        if looks_like_lookup(&unit.code) {
            return Some(make(
                "lookup-keyed-by-target-text",
                format!(
                    "{quoted} spells out a whole {kind} phrase and is used as a \
                     key in a table, arm or lookup"
                ),
            ));
        }
        if looks_like_comparison(&unit.code) {
            return Some(make(
                "comparison-against-target-text",
                format!(
                    "{quoted} spells out a whole {kind} phrase and is compared \
                     against a runtime value"
                ),
            ));
        }
        if is_standalone_value(&unit.code) {
            return Some(make(
                "whole-sentence-equality",
                format!(
                    "the whole {kind} phrase {quoted} is produced or bound as \
                     the value, with nothing computed from it"
                ),
            ));
        }
        return Some(make(
            "full-phrase-literal",
            format!("{quoted} spells out a whole {kind} phrase"),
        ));
    }

    let in_substring_call = looks_like_substring_call(&unit.code);
    // Two words is a collocation and can occur by chance in ordinary prose;
    // three consecutive words of a canonical phrase cannot. Inside a string
    // method call the literal is being used as a key rather than as prose, so
    // two words is already conclusive there.
    let partial: Vec<(String, Coupling)> = phrases
        .partial
        .iter()
        .filter(|(_, c)| c.matched >= 3 || (c.matched >= 2 && in_substring_call))
        .cloned()
        .collect();
    if let Some((lit, coupling)) = partial.first() {
        let quoted = format!("{:?}", lit);
        let shown: Vec<String> = coupling
            .words
            .iter()
            .take(coupling.matched)
            .map(|w: &&str| w.to_string())
            .collect();
        if in_substring_call {
            return Some(make(
                "substring-special-case",
                format!(
                    "a string method is handed {quoted}, which holds {} of the \
                     canonical {} phrase ({})",
                    coupling.matched,
                    if coupling.is_target { "target" } else { "clue" },
                    shown.join(" ")
                ),
            ));
        }
        return Some(make(
            "phrase-substring-literal",
            format!(
                "{quoted} holds {} of the canonical {} phrase ({})",
                coupling.matched,
                if coupling.is_target { "target" } else { "clue" },
                shown.join(" ")
            ),
        ));
    }

    for words in identifier_words(&unit.code) {
        let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        for c in couplings(&refs) {
            if !c.whole {
                continue;
            }
            let kind = if c.is_target { "target" } else { "clue" };
            return Some(make(
                "identifier-named-after-phrase",
                format!(
                    "an identifier on this line is named after the canonical \
                     {} phrase ({})",
                    kind,
                    c.words.join(" ")
                ),
            ));
        }
        let flat: String = refs.concat();
        for phrase in CANONICAL_TARGETS.iter().chain(CANONICAL_CLUES.iter()) {
            if flat.len() >= 8 && flat == phrase.concat() {
                return Some(make(
                    "identifier-named-after-phrase",
                    format!(
                        "an identifier on this line is named {} — the \
                         normalized canonical phrase",
                        flat
                    ),
                ));
            }
        }
    }

    if runtime_normalization_comparison(&unit.code) {
        return Some(make(
            "runtime-normalization-comparison",
            "a normalized form is compared here against a value that was not \
             recomputed, which is how a hard-code recognizes one target without \
             naming it in a string"
                .to_string(),
        ));
    }

    None
}

/// A legitimate phrase-specific literal, one file and one line.
struct AllowEntry {
    file: &'static str,
    line: usize,
    reason: &'static str,
    marker: &'static str,
}

/// The allowlist. Empty on purpose; see the module docs for why.
const ALLOWLIST: &[AllowEntry] = &[
    // RECOGNIZED:
    //
    // AllowEntry { file: "lexical.rs", line: 0, reason: "<why>", marker: "..." },
];

fn allowed(file: &str, line: usize) -> Option<&'static str> {
    let file = Path::new(file)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file);
    ALLOWLIST
        .iter()
        .find(|e| e.file == file && e.line == line)
        .map(|e| e.reason)
}

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn src_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(src_dir())
        .unwrap_or_else(|e| panic!("reading src/: {e}"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("rs"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .rs files found in src/");
    files
}

/// Every finding in the production region of `src/`.
fn scan_tree() -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut scanned_lines = 0usize;
    for path in src_files() {
        let name = path.to_string_lossy().to_string();
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", name));
        for unit in units_of(&name, &src) {
            scanned_lines += 1;
            if let Some(f) = detect(&unit) {
                if allowed(&name, f.line).is_some() {
                    continue;
                }
                findings.push(f);
            }
        }
    }
    assert!(scanned_lines > 0, "the scan read no lines of src/");
    findings
}

#[test]
fn no_phrase_specific_hard_coding_in_src() {
    let findings = scan_tree();
    if findings.is_empty() {
        return;
    }
    let mut report = String::from(
        "\nphrase-specific hard-coding detected in the production region of src/.\n\
         Each finding names the file, the line and the shape. Remove the special \
         case;\nif an entry is genuinely legitimate, add it to ALLOWLIST in \
         tests/no_phrase_hard_coding.rs\nwith the file, the line and the reason.\n",
    );
    for f in &findings {
        report.push_str(&format!("\n  {}:{}", f.file, f.line));
        report.push_str(&format!("  [{}]\n      {}", f.shape, f.detail));
    }
    report.push('\n');
    panic!("{report}");
}

#[test]
fn the_allowlist_is_small_and_every_entry_justifies_itself() {
    // Two entries is the signal that the region boundary is wrong, not that
    // two hard-codes are legitimate.
    assert!(
        ALLOWLIST.len() <= 1,
        "the allowlist has {} entries; it should stay empty or hold exactly one \
         justified case",
        ALLOWLIST.len()
    );
    for entry in ALLOWLIST {
        assert!(
            !entry.reason.trim().is_empty(),
            "allowlist entry for {}:{} has no reason",
            entry.file,
            entry.line
        );
        assert!(
            !entry.marker.is_empty(),
            "allowlist entries need a greppable marker string from the line"
        );
        let path = src_dir().join(entry.file);
        assert!(
            path.exists(),
            "allowlist entry names {}, which is not a file in src/",
            entry.file
        );
        let src = fs::read_to_string(&path).expect("allowlisted src file is readable");
        let line = src
            .lines()
            .nth(entry.line.saturating_sub(1))
            .unwrap_or_default();
        assert!(
            line.contains(entry.marker),
            "allowlist entry for {}:{} expected marker {:?} on that line, found {:?}",
            entry.file,
            entry.line,
            entry.marker,
            line
        );
    }
}

#[test]
fn the_fence_watches_both_canonical_examples() {
    // If the watched phrase data ever drifts, the fence would silently stop
    // watching. Pin it.
    assert_eq!(CANONICAL_TARGETS.len(), 2);
    assert_eq!(CANONICAL_CLUES.len(), 2);
    let mut seen: HashSet<String> = HashSet::new();
    for phrase in CANONICAL_TARGETS.iter().chain(CANONICAL_CLUES.iter()) {
        assert!(phrase.len() >= 2, "a watched phrase must have two or more words");
        let joined = phrase.join(" ");
        assert!(
            seen.insert(joined.clone()),
            "{joined} appears twice in the watched phrase data"
        );
    }
    for target in CANONICAL_TARGETS {
        let joined = target.join(" ");
        assert!(
            couplings(target)
                .iter()
                .any(|c| c.whole),
            "the fence does not recognise the canonical target {joined}"
        );
    }
    for clue in CANONICAL_CLUES {
        let joined = clue.join(" ");
        assert!(
            couplings(clue)
                .iter()
                .any(|c| c.whole),
            "the fence does not recognise the canonical clue {joined}"
        );
    }
}

/// The detector is only worth having if it fires, so this is a permanent
/// positive control: one synthetic unit per documented shape, each of which
/// must be detected and must be reported as that shape. These snippets are
/// not in `src/`; they exist so the fence's detection power is itself
/// regression-tested rather than being a claim in a commit message.
#[test]
fn the_detector_catches_every_documented_shape() {
    let cases: &[(&str, &str)] = &[
        (
            "whole-sentence-equality",
            "    return \"hits justice dupe hid came\";",
        ),
        (
            "lookup-keyed-by-target-text",
            "    \"wreck a nice beach\" => vec![\"wreck\", \"a\", \"now\"], \"default\" => vec![],",
        ),
        (
            "comparison-against-target-text",
            "    if clue == \"wreck a nice beach\" { return true; }",
        ),
        (
            "comparison-against-normalized-target",
            "    if signature == \"wreckanicebeach\" { return true; }",
        ),
        (
            "whole-sentence-equality",
            "    const CANONICAL: &str = \"hits justice dupe hid came\";",
        ),
        (
            "full-phrase-literal",
            "    let joined = format!(\"hits justice dupe hid came\", n);",
        ),
        (
            "substring-special-case",
            "    if clue.contains(\"nice beach\") { return; }",
        ),
        (
            "phrase-substring-literal",
            "    let _ = \"wreck a nice\";",
        ),
        ("identifier-named-after-phrase", "    fn wreck_a_nice_beach() {}"),
        (
            "runtime-normalization-comparison",
            "    if phrase_signature(&clue) == self.expected_signature { return; }",
        ),
        // A multi-line literal is one unit, and is reported at its first line.
        (
            "full-phrase-literal",
            "    let canned = vec![\n        \"hits\",\n        \"justice\",\n        \
             \"dupe\",\n        \"hid\",\n        \"came\",\n    ];",
        ),
    ];

    for (shape, snippet) in cases {
        let raw = format!("fn f() {{\n{snippet}\n}}\n");
        let mut fired = None;
        for unit in units_of("synthetic.rs", &raw) {
            if let Some(f) = detect(&unit) {
                fired = Some(f);
                break;
            }
        }
        let fired = fired.unwrap_or_else(|| {
            panic!("synthetic hard-code was not detected at all:\n{snippet}\nexpected shape {shape}")
        });
        assert_eq!(
            fired.shape, *shape,
            "synthetic hard-code detected as the wrong shape:\n{snippet}"
        );
    }
}

/// The negative control, in the same place: legitimate source of the same
/// kinds must not fire. If this fails, the fence is a word list.
#[test]
fn the_detector_stays_quiet_on_ordinary_english_and_real_production_code() {
    let innocent: &[&str] = &[
        "    if expected.kind came from cells { return; }",
        "    let comment = \"a key came from cells\";",
        "    if word == \"aim\" || word == \"bad\" || word == \"hid\" { }",
        "    if clue.contains(\"a\") { return; }",
        "    // wrecked aim: the bad hid, we came, it was a nice beach day",
        "    let bad = if depth == 0 { 0 } else { 1 };",
        "    if phrase_signature(a) == phrase_signature(b) && flag { }",
        "    let x = \"stupid game\";",
        "    let y = \"recognize\";",
    ];
    for snippet in innocent {
        let raw = format!("fn f() {{\n{snippet}\n}}\n");
        for unit in units_of("synthetic.rs", &raw) {
            assert!(
                detect(&unit).is_none(),
                "fence fired on ordinary source, which makes it noise:\n{snippet}"
            );
        }
    }
}

/// The regions the module docs claim are excluded really are excluded.
#[test]
fn test_modules_and_comments_are_not_scanned() {
    let raw = "\
//! Module docs naming recognize speech and wreck a nice beach.
fn production() {
    // An ordinary comment about hits justice dupe hid came.
    let _ = 1;
}
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        assert_eq!(phrase_signature(\"recognize speech\"), \"recognize speech\");
        let _ = \"wreck a nice beach\";
    }
}
fn more_production() {
    let _ = 2;
}
";
    let units = units_of("synthetic.rs", raw);
    for unit in &units {
        assert!(
            detect(unit).is_none(),
            "fence fired on a comment or a #[cfg(test)] module body at line {}",
            unit.line
        );
    }
    // And production code outside the test module really was scanned: the
    // units covering it survived, and no unit covers the test module body.
    assert!(units.iter().any(|u| u.code.contains("let _ = 1")));
    assert!(units.iter().any(|u| u.code.contains("let _ = 2")));
    assert!(!units.iter().any(|u| u.code.contains("phrase_signature")));
}
