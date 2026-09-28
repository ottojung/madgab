//! The no-hard-coding fence for the two canonical Mad Gab examples.
//!
//! Scans `src/`, `web/` and `examples/`. See "What is scanned" below for the
//! exact regions and extensions, and the allowlist for the narrow exceptions.
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
//! * **The other files in `tests/`.** Only the scanned regions below are
//!   read. The acceptance tests in `tests/corpus_integration.rs` must name
//!   the phrases; that is their job.
//! * **`Cargo.toml`** and any file extension not listed in `REGIONS`.
//!
//! # What is scanned
//!
//! Three regions, named in `REGIONS` so the scope is greppable rather than
//! implied by which function happens to be called:
//!
//! | region | extensions | why it is in scope |
//! |---|---|---|
//! | `src/` | `rs` | the production region |
//! | `web/` | `js`, `html`, `css` | `web/app.js` is a real user-facing search entry point, so a hard-code committed there is as reachable as one in `src/` |
//! | `examples/` | `rs` | `examples/measure.rs` is compiled and run to measure quality, so a hard-code there would distort the numbers an integration pass reads |
//!
//! The directories are walked recursively and dotfiles are skipped, so a
//! vendored or generated subtree cannot quietly join the scan without being
//! visible in `REGIONS` first. `no_canonical_example_in_a_production_doc_comment`
//! remains `src/`-only, because the rule it enforces is about production
//! documentation rather than about coupling.
//!
//! # The allowlist
//!
//! A legitimate phrase-specific literal goes in `ALLOWLIST` below as an entry
//! naming the directory, the file, the line, *why* it is legitimate, and a
//! greppable `marker` string that must still be present on that line. There
//! is no wildcard form: an entry either names one directory, one file and one
//! line or it does not apply, and adding one is a visible, greppable edit
//! (`git grep -n RECOGNIZED tests/no_phrase_hard_coding.rs`) that a reviewer
//! sees in the diff.
//!
//! The size bound is **per region**, in `ALLOWLIST_CAPS`: `src/` is held to
//! zero, `web/` to two, `examples/` to one. `src/` is held to zero because
//! every legitimate use of the phrases there is already a region exclusion
//! (a `#[cfg(test)]` module, or `src/main.rs`'s documented CLI usage), so an
//! entry there would be a hard-code buying a pass. The other two regions hold
//! the phrases in user-facing copy and in a benchmark input, and a small named
//! cap is what keeps that from becoming a back door. A cap is raised only by
//! an explicit edit to `ALLOWLIST_CAPS`, never by accident.
//!
//! Two further controls keep the list honest:
//! `the_allowlist_is_small_and_every_entry_justifies_itself` checks each
//! entry's directory, file, line, reason and marker, and
//! `every_allowlist_entry_suppresses_a_finding_that_is_still_there` fails if
//! an entry names a line the detector no longer fires on — so an entry dies
//! with the line that justified it.
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
    code.split([';', '\n'])
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

/// A legitimate phrase-specific literal: one directory, one file, one line.
struct AllowEntry {
    /// Scanned directory the file is in, e.g. `"src"`.
    dir: &'static str,
    file: &'static str,
    line: usize,
    reason: &'static str,
    marker: &'static str,
}

/// The allowlist. `src/` has no entry at all; the others name one line each,
/// and every reason says why the phrase is user-visible text rather than
/// coupling. See the module docs for why it must stay this small.
const ALLOWLIST: &[AllowEntry] = &[
    // RECOGNIZED:
    //
    // AllowEntry { dir: "src", file: "lexical.rs", line: 0, reason: "<why>",
    //              marker: "..." },
    AllowEntry {
        dir: "web",
        file: "index.html",
        line: 20,
        reason: "the pre-filled default value of the search box: UI copy the \
                 user can clear or overwrite, not search behaviour",
        marker: "value=",
    },
    AllowEntry {
        dir: "web",
        file: "index.html",
        line: 21,
        reason: "the same search box's placeholder hint, on the next line of \
                 that one element: it names a canonical example to show the \
                 expected input shape",
        marker: "placeholder=",
    },
    AllowEntry {
        dir: "examples",
        file: "measure.rs",
        line: 30,
        reason: "the head of the benchmark harness's 24-target list: it \
                 reports aggregate quality and never special-cases one target",
        marker: "const TARGETS",
    },
];

/// The directories the fence reads, each with the largest number of
/// allowlist entries it may hold.
///
/// `src/` is the production region and is held to **zero**: every legitimate
/// use of the phrases there is already a region exclusion (a `#[cfg(test)]`
/// module, or `src/main.rs`'s documented CLI usage), so an entry there would
/// be a hard-code buying a pass. `web/` and `examples/` are the regions the
/// fence extension added, where the phrases legitimately appear as user-facing
/// copy and as a benchmark input, and they are held to a small named cap
/// rather than to nothing. The caps are the size bound: they grow only by an
/// explicit, visible edit here, never by accident.
const ALLOWLIST_CAPS: &[(&str, usize)] = &[("src", 0), ("web", 2), ("examples", 1)];

fn allowed(dir: &str, file: &str, line: usize) -> Option<&'static str> {
    let file = Path::new(file)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file);
    ALLOWLIST
        .iter()
        .find(|e| e.dir == dir && e.file == file && e.line == line)
        .map(|e| e.reason)
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every file of a scanned directory with the given extension, recursively.
/// `src/` and `examples/` hold Rust source; `web/` holds the browser assets
/// that can carry coupling into a user's search.
fn files_in(dir: &'static str, ext: &'static str) -> Vec<PathBuf> {
    fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {dir:?}: {e}"));
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if name.starts_with('.') {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                walk(&path, ext, out);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&manifest_dir().join(dir), ext, &mut out);
    out.sort();
    assert!(!out.is_empty(), "no {ext} files found in {dir}/");
    out
}

fn src_files() -> Vec<PathBuf> {
    files_in("src", "rs")
}

/// The regions the fence reads, as `(directory, file extension)`.
///
/// `web/` is scanned because `web/app.js` is a real user-facing search entry
/// point: a hard-code committed there is as reachable as one in `src/`.
/// `examples/` is scanned because `examples/measure.rs` is compiled and run to
/// measure quality, so a hard-code there would distort the numbers an
/// integration pass reads.
const REGIONS: &[(&str, &str)] = &[
    ("src", "rs"),
    ("web", "js"),
    ("web", "html"),
    ("web", "css"),
    ("examples", "rs"),
];

/// Every finding in the scanned regions, with allowlisted lines excluded.
fn scan_tree() -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut scanned_lines = 0usize;
    for (dir, ext) in REGIONS {
        for path in files_in(dir, ext) {
            let name = path.to_string_lossy().to_string();
            let src = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", name));
            for unit in units_of(&name, &src) {
                scanned_lines += 1;
                if let Some(f) = detect(&unit) {
                    if allowed(dir, &name, f.line).is_some() {
                        continue;
                    }
                    findings.push(f);
                }
            }
        }
    }
    assert!(scanned_lines > 0, "the scan read no lines at all");
    findings
}

#[test]
fn no_phrase_specific_hard_coding_in_src_web_or_examples() {
    let findings = scan_tree();
    if findings.is_empty() {
        return;
    }
    let mut report = String::from(
        "\nphrase-specific hard-coding detected in a scanned region \
         (src/, web/, examples/).\nEach finding names the file, the line and the \
         shape. Remove the special case;\nif an entry is genuinely legitimate, add \
         it to ALLOWLIST in tests/no_phrase_hard_coding.rs\nwith the directory, \
         the file, the line and the reason.\n",
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
    // The size bound is per scanned directory, and `src/` is held to zero.
    // Two or more entries in the *same* directory is the signal that the
    // region boundary is wrong, not that that many hard-codes are
    // legitimate: the cap is what stops the user-facing and measurement
    // regions from becoming a back door, and it was raised from zero only
    // by the fence extension recorded in docs/work/items/w-3a7f0d.md.
    for (dir, cap) in ALLOWLIST_CAPS {
        let in_dir = ALLOWLIST.iter().filter(|e| e.dir == *dir).count();
        assert!(
            in_dir <= *cap,
            "{in_dir} allowlist entries for {dir}/; the cap is {cap}. More than \
             that in one region means the region boundary is in the wrong \
             place, not that another hard-code is legitimate"
        );
    }
    // An entry naming a directory the fence does not scan buys nothing and
    // would hide a real finding from the scan.
    for entry in ALLOWLIST {
        assert!(
            ALLOWLIST_CAPS.iter().any(|(dir, _)| *dir == entry.dir),
            "allowlist entry names directory {}, which is not a scanned region",
            entry.dir
        );
        assert!(
            !entry.reason.trim().is_empty(),
            "allowlist entry for {}/{}:{} has no reason",
            entry.dir,
            entry.file,
            entry.line
        );
        assert!(
            !entry.marker.is_empty(),
            "allowlist entries need a greppable marker string from the line"
        );
        let path = manifest_dir().join(entry.dir).join(entry.file);
        assert!(
            path.exists(),
            "allowlist entry names {}/{}, which is not a file in a scanned region",
            entry.dir,
            entry.file
        );
        let src = fs::read_to_string(&path).expect("allowlisted file is readable");
        let line = src
            .lines()
            .nth(entry.line.saturating_sub(1))
            .unwrap_or_default();
        assert!(
            line.contains(entry.marker),
            "allowlist entry for {}/{}:{} expected marker {:?} on that line, found {:?}",
            entry.dir,
            entry.file,
            entry.line,
            entry.marker,
            line
        );
    }
    // Two entries for the same site would mean one is redundant.
    for (i, a) in ALLOWLIST.iter().enumerate() {
        for b in &ALLOWLIST[i + 1..] {
            assert!(
                (a.dir, a.file, a.line) != (b.dir, b.file, b.line),
                "allowlist has two entries for {}:{}:{}",
                a.dir,
                a.file,
                a.line
            );
        }
    }
}

/// Every allowlisted line is still a live phrase hit, so an entry cannot
/// quietly stop justifying itself once the file it names is edited. A unit is
/// a bracket-balanced run of lines, so the file is scanned whole and the
/// entry's line is matched against the units that report one.
#[test]
fn every_allowlist_entry_suppresses_a_finding_that_is_still_there() {
    assert!(
        !ALLOWLIST.is_empty(),
        "the allowlist is empty; nothing for this control to check"
    );
    for entry in ALLOWLIST {
        let path = manifest_dir().join(entry.dir).join(entry.file);
        let src = fs::read_to_string(&path).expect("allowlisted file is readable");
        let fires_at_entry = units_of(entry.file, &src)
            .iter()
            .any(|u| u.line == entry.line && detect(u).is_some());
        assert!(
            fires_at_entry,
            "allowlist entry {}/{}:{} does not name a line the detector still \
             fires on, so the entry is stale and should be deleted",
            entry.dir,
            entry.file,
            entry.line
        );
    }
}

/// The `hid` inside the DOM property `hidden` is a substring of a canonical
/// clue word, and the audit that filed this item listed those lines as
/// possible hits. They are not: `hidden` is one ordinary word, the fence needs
/// two words as a bare literal, and it is not a literal in either file. Pin it
/// on the real `web/` sources rather than a synthetic snippet, so a future
/// change to the unit splitter or the word thresholds that *does* start
/// firing on `hidden` is caught here.
#[test]
fn the_dom_property_hidden_is_not_a_phrase_hit() {
    let mut checked = 0usize;
    for ext in ["js", "html"] {
        for path in files_in("web", ext) {
            let name = path.to_string_lossy().to_string();
            let src = fs::read_to_string(&path).expect("web source is readable");
            for (i, line) in src.lines().enumerate() {
                if !line.contains("hidden") {
                    continue;
                }
                checked += 1;
                for unit in units_of(&name, &format!("{line}\n")) {
                    assert!(
                        detect(&unit).is_none(),
                        "the DOM property `hidden` at {}:{} was read as a canonical \
                         phrase hit; one ordinary English word is not a coupling",
                        name,
                        i + 1
                    );
                }
            }
        }
    }
    assert!(
        checked >= 6,
        "expected the `hidden` occurrences in web/app.js and web/index.html to \
         still be present, found {checked}"
    );
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

/// The written fence is stronger than the detector above: it forbids a
/// canonical example's words in production `src/` *or* `tests/`, comments
/// included, and the detector strips comments by design. The one place that
/// has to keep them is `src/main.rs`, whose doc examples are the CLI's
/// documented usage; every other production comment must be clean, so a
/// measurement recorded in a production doc comment cannot name the input it
/// was measured on.
///
/// A comment holding *three* consecutive words of a watched phrase is a
/// hard-code's fingerprint whether or not it is code, and one or two words is
/// ordinary prose (`the bad hid, we came, it was a nice beach day`), so the
/// threshold is three here for the same reason it is three above.
#[test]
fn no_canonical_example_in_a_production_doc_comment() {
    let mut report = String::new();
    for path in src_files() {
        let name = path.to_string_lossy().to_string();
        if Path::new(&name).file_name().and_then(|s| s.to_str()) == Some("main.rs") {
            continue;
        }
        let src = fs::read_to_string(&path).expect("production source is readable");
        let test = test_lines(&src.split('\n').collect::<Vec<_>>());
        for (i, line) in src.lines().enumerate() {
            if test.contains(&(i + 1)) {
                continue;
            }
            let Some(body) = line.trim_start().strip_prefix("//") else {
                continue;
            };
            let words: Vec<String> = literal_words(body);
            let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
            for c in couplings(&refs) {
                if c.matched < 3 {
                    continue;
                }
                let shown: Vec<String> = c
                    .words
                    .iter()
                    .take(c.matched)
                    .map(|w: &&str| w.to_string())
                    .collect();
                report.push_str(&format!(
                    "\n  {}:{}  [{}]\n      a production comment holds {} words of the canonical {} phrase ({})",
                    name,
                    i + 1,
                    "phrase-in-production-comment",
                    c.matched,
                    if c.is_target { "target" } else { "clue" },
                    shown.join(" "),
                ));
            }
        }
    }
    assert!(
        report.is_empty(),
        "a production doc comment names a canonical example:\n{report}"
    );
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
