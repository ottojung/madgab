//! `madgab` — Mad Gab puzzle generator CLI.
//!
//! Reads a target English phrase, prints ranked Mad-Gab-style clue
//! candidates: phrases whose IPA matches the target but whose words
//! re-syllabify the phoneme stream.
//!
//! Usage:
//!
//!     madgab "It's just a stupid game"
//!     madgab --top 20 --max-rarity 50000 "Coors light"
//!     madgab --approximate "recognize speech" --pool-rank "wreck a nice beach"
//!     madgab --transcribe "It's just a stupid game"   # IPA-only debug
//!
//! Build is a single binary that embeds the 15 MB transcription
//! corpus, so `madgab` runs anywhere without a separate data file.

use std::process::ExitCode;

use madgab::{Generator, GeneratorConfig, SearchMode};

// The IPA dictionary comes from the open-english-pronouncing-dictionary
// crate (a.k.a. OpenEPD). That crate carries the data and a thin loader;
// we just point at its embedded constant so the binary still has zero
// on-disk dependencies.
use open_english_pronouncing_dictionary::CORPUS_JSON;

const USAGE: &str = "\
madgab — Mad Gab puzzle generator

Usage:
  madgab [options] <target phrase>

Options:
  --top N            Return top N candidates (default 10).
  --max-rarity R     Drop corpus words rarer than R (default 50000).
  --beam K           Beam width during search (default 64).
  --min-word-len N   Skip clue words with fewer than N IPA chars (default 1).
  --approximate      Allow small phonetic substitutions (/t/→/d/, /ɪ/→/i/, etc.)
                     Lets the generator find clues whose phonemes don't exactly
                     match the target. Defaults are sensible; tune with the next
                     two flags if needed.
  --per-word-budget COST   Approximate-mode: max substitution cost per clue word
                           (default 0.5).
  --total-budget COST      Approximate-mode: max total substitution cost (default 1.5).
  --pool-rank        Also report each proposal's rank in the scored candidate
                     pool, not just its display position. Costs a second search.
  --pool-rank CLUE   Placed AFTER the target phrase: report only where CLUE
                     stands in the scored pool for that target. Prints either a
                     rank, or an explicit absent-from-pool statement. Absent is
                     a finding, not a failure: it exits 3, while a real error
                     exits 1 or 2. Costs a second search.
  --transcribe       Print the target's IPA stream and exit.
  --help             This message.
";

/// The **display position** of a proposal: its 1-based position in the printed
/// list, after the search's selection policy has narrowed the pool to `--top`.
///
/// The **pool rank** of a proposal: its 1-based position in the deduplicated,
/// score-ordered candidate pool the search actually built, before that
/// selection policy ran.
///
/// These are different coordinates and must never be printed under one label.
/// A clue can be display position 1 and pool rank 400; a clue can be absent
/// from the display and present in the pool; and a clue can be missing from
/// both, which is a search question rather than a display question. Until the
/// CLI reported both, every rank figure in this project's records was ambiguous
/// between them, and "the binary shows rank 27" could mean either.
fn render_row(
    display_position: usize,
    clue: &madgab::Clue,
    pool_rank: Option<usize>,
    pool_size: usize,
) -> String {
    match pool_rank {
        // Default shape, byte-identical to what this tool has always printed.
        None => format!("{display_position:2}. [{:.3}] {}", clue.score, clue.phrase),
        // Labelled shape: the score and the pool rank are named fields inside
        // the one bracket, so no consumer has to guess which is which, and the
        // phrase still starts at the first `]` exactly as before.
        Some(rank) => format!(
            "{display_position:2}. [score {:.3}, pool rank {rank} of {pool_size}] {}",
            clue.score, clue.phrase
        ),
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let mut config = GeneratorConfig {
        mode: SearchMode::Exact,
        ..GeneratorConfig::default()
    };
    let mut approximate_per_word = 0.5_f64;
    let mut approximate_total = 1.5_f64;
    let mut transcribe_only = false;
    let mut show_pool_rank = false;

    while let Some(flag) = args.first().cloned() {
        match flag.as_str() {
            "--top" => {
                args.remove(0);
                let v = args.remove(0).parse::<usize>().unwrap_or(config.top_n);
                config.top_n = v;
            }
            "--max-rarity" => {
                args.remove(0);
                let v = args.remove(0).parse::<f64>().ok();
                config.max_rarity = v;
            }
            "--beam" => {
                args.remove(0);
                config.beam_width = args.remove(0).parse::<usize>().unwrap_or(config.beam_width);
            }
            "--min-word-len" => {
                args.remove(0);
                config.min_word_ipa_chars =
                    args.remove(0).parse::<usize>().unwrap_or(config.min_word_ipa_chars);
            }
            "--approximate" => {
                args.remove(0);
                config.mode = SearchMode::Approximate {
                    per_word_budget: approximate_per_word,
                    total_budget: approximate_total,
                };
            }
            "--per-word-budget" => {
                args.remove(0);
                approximate_per_word = args.remove(0).parse::<f64>().unwrap_or(approximate_per_word);
                if matches!(config.mode, SearchMode::Approximate { .. }) {
                    config.mode = SearchMode::Approximate {
                        per_word_budget: approximate_per_word,
                        total_budget: approximate_total,
                    };
                }
            }
            "--total-budget" => {
                args.remove(0);
                approximate_total = args.remove(0).parse::<f64>().unwrap_or(approximate_total);
                if matches!(config.mode, SearchMode::Approximate { .. }) {
                    config.mode = SearchMode::Approximate {
                        per_word_budget: approximate_per_word,
                        total_budget: approximate_total,
                    };
                }
            }
            "--pool-rank" => {
                args.remove(0);
                show_pool_rank = true;
            }
            "--transcribe" => {
                args.remove(0);
                transcribe_only = true;
            }
            other if other.starts_with("--") => {
                eprintln!("madgab: unknown flag {other:?}");
                return ExitCode::from(2);
            }
            _ => break,
        }
    }

    // A trailing `--pool-rank` — one that appears *after* the target phrase,
    // as in `madgab --approximate "<target>" --pool-rank "<clue>"` — is a
    // query about one clue rather than an annotation of every row. The
    // pre-target form above stays the annotate-every-row form, so the two
    // spellings are distinguishable by position alone and neither changes
    // what the other means.
    //
    // Splitting on the literal token rather than reworking the flag loop keeps
    // the default path bit-for-bit: with no trailing token, `target` is
    // computed exactly as it always was.
    let mut query_for: Option<String> = None;
    if let Some(at) = args.iter().position(|a| a == "--pool-rank") {
        let rest = args.split_off(at);
        // Drop the token itself; whatever follows is the clue to look for.
        let clue = rest[1..].join(" ");
        if clue.trim().is_empty() {
            // A trailing form with no clue is a mistake, not a request: the
            // pre-target form is the one that takes no argument.
            eprintln!("madgab: --pool-rank after the target needs a clue to look for");
            eprintln!("\n{USAGE}");
            return ExitCode::from(2);
        }
        query_for = Some(clue);
    }

    if args.is_empty() {
        eprintln!("madgab: missing <target phrase>\n\n{USAGE}");
        return ExitCode::from(2);
    }

    let target = args.join(" ");

    // Loading the corpus dominates startup (~1s). Don't do it in the
    // --transcribe path if we don't need to — but we do need it
    // because transcribe() uses the corpus, so just press on.
    let started_load = std::time::Instant::now();
    let generator = match Generator::from_json(CORPUS_JSON, config.clone()) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("madgab: failed to load corpus: {e}");
            return ExitCode::from(1);
        }
    };
    let load_ms = started_load.elapsed().as_millis();

    if transcribe_only {
        match generator.corpus().transcribe(&target) {
            Some(ipa) => {
                println!("{ipa}");
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("madgab: at least one word in {target:?} has no transcription");
                ExitCode::from(1)
            }
        }
    } else {
        let started_search = std::time::Instant::now();
        let (clues, pool_size) = generator.generate_with_pool(&target);
        let search_ms = started_search.elapsed().as_millis();

        if clues.is_empty() {
            eprintln!("madgab: no clue coverings found for {target:?}");
            eprintln!("  (try lowering --min-word-len or raising --max-rarity)");
            return ExitCode::from(1);
        }

        // Pool ranks come from a second search. `generate_with_pool` reports
        // the pool's *size* only, so the pool's contents — and therefore any
        // proposal's coordinate inside it — need `generate_pool`. The search
        // is deterministic (tests/approx_determinism.rs, tests/exact_determinism.rs),
        // so the second run's pool is the first run's pool, and the ranks
        // below describe the printed rows. This is why the flag is opt-in:
        // the default path pays no extra search for the size it now reports.
        let started_pool = std::time::Instant::now();
        let needs_pool = show_pool_rank || query_for.is_some();
        let pool: Option<Vec<madgab::Clue>> = if needs_pool {
            Some(generator.generate_pool(&target))
        } else {
            None
        };
        // Annotation of the printed rows happens only for the pre-target
        // `--pool-rank`; a query about one clue leaves the rows alone, so a
        // query never turns into an annotation.
        let pool_ranks: Option<Vec<Option<usize>>> = match (show_pool_rank, &pool) {
            (true, Some(p)) => Some(pool_ranks_of(p, &clues)),
            _ => None,
        };
        let pool_ms = started_pool.elapsed().as_millis();

        let ipa = generator
            .corpus()
            .transcribe(&target)
            .unwrap_or_else(|| "?".to_string());
        eprintln!("target: {target}");
        eprintln!("IPA:    /{ipa}/");
        eprintln!("(corpus loaded in {load_ms}ms; search {search_ms}ms)");
        eprintln!(
            "(pool: {pool_size} scored candidates, {} displayed; expansion {:.1}x)",
            clues.len(),
            if clues.is_empty() {
                0.0
            } else {
                pool_size as f64 / clues.len() as f64
            }
        );
        // The second search is the whole cost of this flag, so it is reported
        // for the query form as well as the annotate form. A surface whose
        // price is only knowable by timing it from outside cannot be judged
        // against the alternative of not asking the question.
        if needs_pool {
            eprintln!("(pool-rank run: second search {pool_ms}ms)");
        }
        eprintln!();

        for (i, clue) in clues.iter().enumerate() {
            println!(
                "{}",
                render_row(
                    i + 1,
                    clue,
                    pool_ranks.as_ref().and_then(|r| r[i]),
                    pool_size,
                )
            );
        }

        match query_for {
            None => ExitCode::SUCCESS,
            Some(clue) => report_query_rank(&clue, pool.as_deref().unwrap_or(&[]), &clues, pool_size),
        }
    }
}

/// Exit code for "the search succeeded and the clue is not in the pool".
///
/// Deliberately distinct from `1` (a real failure: corpus load, or no clue
/// coverings at all) and from `2` (a usage error). Absence is a *finding* —
/// the pool was built and the clue is not in it — and this repository's
/// standing confusion is exactly the kind this separates: "was not generated"
/// and "generated but did not make the display" are different facts, and a
/// caller that reads either as "the run failed" is wrong.
const ABSENT_FROM_POOL: u8 = 3;

/// Where a user-supplied clue stands in the scored pool, or the statement
/// that it is not there.
///
/// The two outcomes are worded so that neither can be misread as the other
/// or as a failure: a hit names a rank *and* says whether it also made the
/// display, and a miss says the clue was **not generated** into a pool of a
/// stated size, which is a statement about emission rather than about
/// ranking.
fn report_query_rank(
    clue: &str,
    pool: &[madgab::Clue],
    displayed: &[madgab::Clue],
    pool_size: usize,
) -> ExitCode {
    let wanted = clue.trim().to_lowercase();
    let wanted = wanted.split_whitespace().collect::<Vec<_>>().join(" ");
    let match_in = |list: &[madgab::Clue]| {
        list.iter().position(|c| {
            c.phrase
                .trim()
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                == wanted
        })
    };

    match match_in(pool) {
        Some(at) => {
            let display_position = match_in(displayed).map(|i| i + 1);
            match display_position {
                Some(d) => println!(
                    "pool-rank: {clue:?} is pool rank {} of {pool_size}; displayed at {d}",
                    at + 1
                ),
                None => println!(
                    "pool-rank: {clue:?} is pool rank {} of {pool_size}; NOT in the display \
                     (generated, then ranked out of the top {} shown)",
                    at + 1,
                    displayed.len()
                ),
            }
            ExitCode::SUCCESS
        }
        None => {
            eprintln!(
                "pool-rank: {clue:?} is ABSENT from the pool: not generated at all, out of a \
                 pool of {pool_size} scored candidates."
            );
            eprintln!(
                "  This is a finding, not a failure: the run succeeded and the clue was never \
                 built."
            );
            eprintln!("  Absence is not the same as ranking out of the display.");
            ExitCode::from(ABSENT_FROM_POOL)
        }
    }
}

/// Map each displayed proposal to its 1-based position in `pool`.
///
/// The pool is phrase-deduplicated, so the mapping is one-to-one; `None` is
/// returned for a displayed proposal with no pool entry, which should not
/// happen, and is reported as such rather than being papered over with a
/// guess at a rank.
fn pool_ranks_of(pool: &[madgab::Clue], displayed: &[madgab::Clue]) -> Vec<Option<usize>> {
    let rank_of = |phrase: &str| -> Option<usize> {
        pool.iter()
            .position(|c| c.phrase == phrase)
            .map(|i| i + 1)
    };
    displayed.iter().map(|c| rank_of(&c.phrase)).collect()
}
