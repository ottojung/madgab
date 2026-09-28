//! w-5e2d42 measurement probe: per-span shortlist membership for the
//! canonical case-2 target and five named words.
//!
//! Run with:  cargo run --release --example zz_5e2d42_spans
//!
//! No phrase is hard-coded in src/; this probe only reads the public
//! `span_candidates` / `span_shortlists` API at the default budgets.

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TARGET: &str = "It's just a stupid game";
const WORDS: &[&str] = &["hits", "justice", "dupe", "hid", "came"];

fn main() {
    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::Approximate {
                per_word_budget: 0.5,
                total_budget: 1.5,
            },
            top_n: 50,
            ..GeneratorConfig::default()
        },
    )
    .unwrap();
    let cands = g.span_candidates(TARGET, 0.5);
    let shorts = g.span_shortlists(TARGET, 0.5);
    let (printed, pool_size) = g.generate_with_pool(TARGET);
    let pool = g.generate_pool(TARGET);
    println!("pool {} printed {}", pool.len(), printed.len());
    println!("pool_size_reported {pool_size}");

    {
        use std::collections::BTreeMap;
        let mut by: BTreeMap<(usize, usize), Vec<f64>> = BTreeMap::new();
        for c in &shorts {
            by.entry((c.start, c.end)).or_default().push(c.cost);
        }
        let mut offered: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for c in &cands {
            *offered.entry((c.start, c.end)).or_default() += 1;
        }
        println!("--- shortlist span census");
        let mut rows: Vec<String> = Vec::new();
        for (k, v) in &by {
            let min = v.iter().cloned().fold(f64::INFINITY, f64::min);
            let cheap = v.iter().filter(|c| **c <= 1.5 * min).count();
            let off_cheap = cands
                .iter()
                .filter(|c| c.start == k.0 && c.end == k.1 && c.cost <= 1.5 * min)
                .count();
            rows.push(format!(
                "  span [{:?}) width={} offered={} min_cost={:.10} cheap1.5x_retained={} cheap1.5x_offered={}",
                k, v.len(), offered[k], min, cheap, off_cheap
            ));
        }
        rows.sort();
        for r in rows {
            println!("{r}");
        }
    }
    for w in WORDS {
        println!("--- {w}");
        let mut rows: Vec<(usize, usize, Option<f64>, Option<f64>, usize)> =
            Vec::new();
        let spans: std::collections::BTreeSet<(usize, usize)> = cands
            .iter()
            .map(|c| (c.start, c.end))
            .chain(shorts.iter().map(|c| (c.start, c.end)))
            .collect();
        for (s, e) in spans {
            let offer: Vec<&madgab::SpanCandidate> = cands
                .iter()
                .filter(|c| c.start == s && c.end == e)
                .collect();
            let in_short = shorts
                .iter()
                .find(|c| c.start == s && c.end == e && c.word == *w)
                .map(|c| c.cost);
            let cost = offer
                .iter()
                .find(|c| c.word == *w)
                .map(|c| c.cost);
            if cost.is_none() && in_short.is_none() {
                continue;
            }
            // rank of the word by cost within the span's offered candidates
            let mut by_cost = offer.clone();
            by_cost.sort_by(|a, b| {
                a.cost.partial_cmp(&b.cost).unwrap()
            });
            let rank = by_cost.iter().position(|c| c.word == *w);
            rows.push((
                s,
                e,
                cost,
                in_short,
                rank.map(|r| r + 1).unwrap_or(0),
            ));
        }
        for (s, e, cost, in_short, rank) in rows {
            println!(
                "  span [{s},{e}) offered={} costrank={rank} cost={:?} shortlist={}",
                cands
                    .iter()
                    .filter(|c| c.start == s && c.end == e)
                    .count(),
                cost,
                if in_short.is_some() { "YES" } else { "no" }
            );
        }
        let in_pool = pool
            .iter()
            .filter(|c| c.words.iter().any(|t| t.word == *w))
            .count();
        println!("  pool wordings containing it: {in_pool}");
    }
}
