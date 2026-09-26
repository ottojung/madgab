//! w-7b2d40 scratch measurement: what bounds a proposal's visibility?
//!
//! Runs the approximate search for each target, dumps the retained pool
//! and the selection outcome via the MADGAB_SEL_TRACE hook in
//! `select_diverse`, and reports the numbers that decide membership:
//! pool size, distinct structures, cap, the global visible cutoff, how
//! many pool candidates sit below it, how many of those are nevertheless
//! visible, and the within-structure score rank of every visible entry.

use std::collections::HashMap;
use std::time::Instant;

use madgab::{Generator, GeneratorConfig, SearchMode};
use open_english_pronouncing_dictionary::CORPUS_JSON;

const TOP_N: usize = 50;

const TARGETS: &[&str] = &[
    "recognize speech",
    "It's just a stupid game",
    "a whole lot of trouble",
    "the cat sat on the mat",
    "put it back on the shelf",
    "when the rain finally stopped",
];

/// The canonical clue, matched loosely (case/punctuation) against pool
/// phrases.  Not a production code path: this harness only.
const CANONICAL: &[(&str, &str)] = &[("It's just a stupid game", "hits justice dupe hid came")];

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

struct Entry {
    /// Position in the descending-score order, i.e. the global rank.
    rank: usize,
    /// The pool index the trace reported, which is what PICK lines refer to.
    index: usize,
    score: f64,
    structure: Vec<usize>,
    phrase: String,
}

fn main() {
    let trace = std::env::temp_dir().join("w7b2d40-sel-trace.txt");
    let trace = trace.to_str().unwrap().to_string();
    let inject = std::env::args().nth(1);

    let g = Generator::from_json(
        CORPUS_JSON,
        GeneratorConfig {
            mode: SearchMode::approximate(),
            top_n: TOP_N,
            beam_width: 64,
            ..GeneratorConfig::default()
        },
    )
    .expect("corpus should parse");

    let only = std::env::var("MADGAB_ONLY").ok();
    let mut total_seconds = 0.0;
    for target in TARGETS {
        if let Some(o) = &only {
            if norm(o) != norm(target) {
                continue;
            }
        }
        if let Some(spec) = &inject {
            std::env::set_var("MADGAB_INJECT", spec);
        }
        std::env::set_var("MADGAB_SEL_TRACE", &trace);
        let started = Instant::now();
        let visible = g.generate(target);
        let seconds = started.elapsed().as_secs_f64();
        total_seconds += seconds;
        std::env::remove_var("MADGAB_SEL_TRACE");
        std::env::remove_var("MADGAB_INJECT");

        let text = std::fs::read_to_string(&trace).expect("trace");
        let mut lines = text.lines();
        let stats = lines.next().expect("stats line");
        let mut parts = stats.split_whitespace();
        let mut field = |k: &str| -> String {
            parts
                .find(|p| p.starts_with(&format!("{k}=")))
                .map(|p| p[k.len() + 1..].to_string())
                .unwrap_or_else(|| "?".into())
        };
        let pool_n: usize = field("pool").parse().unwrap();
        let available: usize = field("available").parse().unwrap();
        let cap: usize = field("cap").parse().unwrap();
        let cutoff_score: f64 = field("cutoff_score").parse().unwrap();

        let mut pool: Vec<Entry> = Vec::with_capacity(pool_n);
        // The C lines are emitted in `order`, i.e. already in descending
        // score order, so the *emission position* is the global rank.  The
        // first field is the pool index and must not be used as the rank:
        // doing so once reported eight same-structure siblings as ranking
        // above a candidate they score below.
        for (pos, l) in lines.enumerate() {
            if let Some(rest) = l.strip_prefix("C\t") {
                let f: Vec<&str> = rest.split('\t').collect();
                pool.push(Entry {
                    rank: pos,
                    index: f[0].parse().unwrap(),
                    score: f[1].parse().unwrap(),
                    structure: if f[2].is_empty() {
                        Vec::new()
                    } else {
                        f[2].split(',').map(|x| x.parse().unwrap()).collect()
                    },
                    phrase: f[3].to_string(),
                });
            }
        }
        assert_eq!(pool.len(), pool_n, "trace parse");
        // the pool is in descending score order by construction
        assert!(
            pool.windows(2).all(|w| w[0].score >= w[1].score),
            "pool not in descending score order"
        );

        // within-structure score rank: how many pool candidates share this
        // structure and score strictly higher.
        let mut better: HashMap<&Vec<usize>, usize> = HashMap::new();
        for e in &pool {
            *better.entry(&e.structure).or_default() += 1;
        }
        let mut within: Vec<usize> = vec![0; pool_n];
        let mut seen: HashMap<&Vec<usize>, usize> = HashMap::new();
        for e in &pool {
            let c = seen.entry(&e.structure).or_default();
            within[e.rank] = *c + 1;
            *c += 1;
        }
        let _ = better;

        // PICK lines report pool indices; translate them to global ranks.
        let mut by_index: HashMap<usize, usize> = HashMap::new();
        for e in &pool {
            by_index.insert(e.index, e.rank);
        }
        let picked: Vec<usize> = {
            let mut v = Vec::new();
            for l in text.lines() {
                if let Some(r) = l.strip_prefix("PICK\t") {
                    let idx: usize = r.parse().unwrap();
                    v.push(*by_index.get(&idx).expect("PICK names a pool entry"));
                }
            }
            v
        };
        let picked_set: std::collections::HashSet<usize> = picked.iter().copied().collect();

        let below_cutoff = pool_n - TOP_N.min(pool_n);
        let below_and_visible = (0..pool_n)
            .filter(|&r| r >= TOP_N && picked_set.contains(&r))
            .count();

        println!("=== target {target:?}  ({seconds:.2}s)");
        println!(
            "  pool={pool_n}  distinct_structures={available}  cap={cap}  visible={}",
            picked.len()
        );
        println!("  global cutoff: rank {TOP_N} score {cutoff_score:.6}");
        println!(
            "  pool candidates below cutoff = {below_cutoff};  of those visible = {below_and_visible} ({:.1}% of below-cutoff, {:.2}% of pool)",
            100.0 * below_and_visible as f64 / below_cutoff.max(1) as f64,
            100.0 * below_and_visible as f64 / pool_n as f64
        );
        let mut w: Vec<usize> = picked.iter().map(|&r| within[r]).collect();
        w.sort_unstable();
        println!(
            "  within-structure rank over visible list: min {} median {} max {} ; ranks {:?}",
            w.first().copied().unwrap_or(0),
            w.get(w.len() / 2).copied().unwrap_or(0),
            w.last().copied().unwrap_or(0),
            w
        );
        let mut by_struct: HashMap<&Vec<usize>, usize> = HashMap::new();
        for &r in &picked {
            *by_struct.entry(&pool[r].structure).or_default() += 1;
        }
        let mut shapes: Vec<(Vec<usize>, usize)> = by_struct.into_iter().map(|(k, v)| (k.to_vec(), v)).collect();
        shapes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("  visible structure fill (structure: shown / pool members / cap):");
        for (s, n) in shapes.iter().take(12) {
            let members = pool.iter().filter(|e| e.structure == *s).count();
            println!("    {:?}: {n} / {members} / {cap}", s);
        }
        println!("  visible structures = {} of {available} available", shapes.len());

        // how far below the cutoff the visible list already reaches, and
        // what a cutoff-free step 1 would cost: rank the pool's distinct
        // structures by their best member's score.
        let worst = picked.iter().copied().max().unwrap_or(0);
        let best_in_worst = picked
            .iter()
            .filter(|&&r| within[r] == 1)
            .copied()
            .max()
            .unwrap_or(0);
        println!(
            "  lowest-scoring visible entry: global rank {worst} score {:.6} ; highest global rank among rank-1 (structure representatives) = {best_in_worst}",
            pool[worst].score
        );
        // Is the cap the ONLY gate on a below-cutoff candidate, or does the
        // walk simply run out of slots first?  The largest global rank that
        // is visible bounds it: any candidate past that rank is invisible
        // *whatever* its within-structure rank, because the list is already
        // full when the walk reaches it.  This separates "under the cap"
        // from "reachable at all".
        let mut fill_rank = 0usize;
        for &r in &picked {
            fill_rank = fill_rank.max(r);
        }
        let at_fill_under_cap = (0..=fill_rank)
            .filter(|&r| !picked_set.contains(&r) && within[r] < cap)
            .count();
        let beyond_fill_under_cap = (fill_rank + 1..pool_n)
            .filter(|&r| within[r] < cap)
            .count();
        println!(
            "  list is FULL at global rank {fill_rank} (score {:.6}); slots run out before the walk finishes.",
            pool[fill_rank].score
        );
        println!(
            "  under-cap candidates (within-structure rank < {cap}) that are still INVISIBLE: {} at or before the fill rank, {beyond_fill_under_cap} beyond it",
            at_fill_under_cap
        );

        let mut reps: Vec<(usize, f64, Vec<usize>)> = Vec::new();
        {
            let mut best_of: HashMap<&Vec<usize>, &Entry> = HashMap::new();
            for e in &pool {
                best_of
                    .entry(&e.structure)
                    .or_insert(e)
                    .score
                    .max(best_of[&e.structure].score);
            }
            for (s, e) in best_of {
                reps.push((e.rank, e.score, s.to_vec()));
            }
        }
        reps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        println!(
            "  cutoff-free step 1: best-member score of distinct structure #1 = {:.6}, #6 = {:.6}, #17 = {:.6}, #50 = {:.6}",
            reps[0].1,
            reps[5].1,
            reps[16].1,
            reps[49].1
        );
        if let Some(pos) = reps.iter().position(|(_, _, s)| *s == vec![3, 10, 13, 15]) {
            println!(
                "  canonical structure [3,10,13,15] would be representative #{} of {} at score {:.6} (visible structure count today = {})",
                pos + 1,
                reps.len(),
                reps[pos].1,
                shapes.len()
            );
        }

        for (t, want) in CANONICAL {
            if t != target {
                continue;
            }
            // probe the canonical alignment's structure directly
            let canonical: Vec<usize> = vec![3, 10, 13, 15];
            let injected_visible = pool
                .iter()
                .find(|e| norm(&e.phrase) == norm(want))
                .map(|e| picked_set.contains(&e.rank))
                .unwrap_or(false);
            let members: Vec<&Entry> = pool
                .iter()
                .filter(|e| e.structure == canonical)
                .collect();
            let score = 0.8199012907736722_f64;
            let above = members.iter().filter(|e| e.score > score).count();
            println!(
                "  PROBE structure {canonical:?}: pool members {} ; members scoring above 0.819901291 = {above} ; within-structure rank of the injected clue = {} ; cap = {cap} ; global rank of the injected clue = {} ; cutoff score = {cutoff_score:.6} ; score gap to cutoff = {:+.9} ; VISIBLE = {}",
                members.len(),
                above + 1,
                pool_n,
                score - cutoff_score,
                injected_visible,
            );
            if above > 0 {
                let mut v: Vec<(usize, f64, String)> = members
                    .iter()
                    .filter(|e| e.score > score)
                    .map(|e| (e.rank, e.score, e.phrase.clone()))
                    .collect();
                v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                for (r, s, p) in v.iter().take(3) {
                    println!("    first above: rank {r} score {s:.9} {p:?}");
                }
            }
            // ENUMERATED, RANKED and VISIBLE as three separate numbers: they
            // have been conflated before in this repository, and only the
            // last one is what the user sees.
            let hit = pool.iter().find(|e| norm(&e.phrase) == norm(want));
            match hit {
                Some(e) => {
                    println!(
                        "  CANONICAL {want:?}: ENUMERATED=yes RANKED={}/{} (score {:.9}, cutoff rank {TOP_N} at {cutoff_score:.6}, gap {:+.9}) VISIBLE={} within_structure_rank={} cap={} structure={:?} structure_pool_members={}",
                        e.rank + 1,
                        pool_n,
                        e.score,
                        e.score - cutoff_score,
                        picked_set.contains(&e.rank),
                        within[e.rank],
                        cap,
                        e.structure,
                        pool.iter().filter(|x| x.structure == e.structure).count(),
                    );
                }
                None => {
                    println!(
                        "  CANONICAL {want:?}: ENUMERATED=no RANKED=n/a VISIBLE=no (pool={pool_n}, canonical structure {canonical:?} has {} members, best member score {:.6}, first member at global rank {})",
                        members.len(),
                        reps.iter().find(|(_, _, s)| *s == canonical).map(|r| r.1).unwrap_or(f64::NAN),
                        reps.iter().find(|(_, _, s)| *s == canonical).map(|r| r.0).unwrap_or(usize::MAX),
                    );
                }
            }
        }
        let _ = visible;
        println!();
    }
    println!("total search seconds = {total_seconds:.2}");
}
