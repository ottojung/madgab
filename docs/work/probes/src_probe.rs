//! THROWAWAY measurement scaffolding for front 558697.  Not proposed for
//! integration.  Every entry point is inert unless `MADGAB_558697` is set,
//! and the module names no word, phrase, target or dictionary entry: the
//! only inputs are a variant tag, a weight and a list of integer index
//! tuples, all from the environment.

use std::cell::RefCell;
use std::sync::OnceLock;

/// Objective variant tag: `punch` / `foot` / `econ` / `land`, or empty.
pub fn variant() -> &'static str {
    static V: OnceLock<String> = OnceLock::new();
    V.get_or_init(|| std::env::var("MADGAB_558697").unwrap_or_default())
        .as_str()
}

pub fn enabled() -> bool {
    !variant().is_empty()
}

/// Additive weight for the candidate axis, in score units.
pub fn weight() -> f64 {
    static W: OnceLock<f64> = OnceLock::new();
    *W.get_or_init(|| {
        std::env::var("MADGAB_558697_W")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.10)
    })
}

/// Per-target constants the candidate axes need: IPA length, the target's
/// syllable count and the target's word count.  Set once per search.
thread_local! {
    static SHAPE: RefCell<(f64, f64, f64)> = const { RefCell::new((0.0, 0.0, 0.0)) };
}

pub fn set_target_shape(total_len: f64, target_syllables: f64, target_words: f64) {
    SHAPE.with(|s| *s.borrow_mut() = (total_len, target_syllables, target_words));
}

pub fn shape() -> (f64, f64, f64) {
    SHAPE.with(|s| *s.borrow())
}

/// Index tuples to force into the pool, as `seg:t0,t1,...` separated by `;`.
pub fn injects() -> &'static Vec<(usize, Vec<usize>)> {
    static I: OnceLock<Vec<(usize, Vec<usize>)>> = OnceLock::new();
    I.get_or_init(|| match std::env::var("MADGAB_558697_INJECT") {
        Ok(s) => s
            .split(';')
            .filter(|p| !p.trim().is_empty())
            .filter_map(|p| {
                let mut it = p.split(',').map(|x| x.trim());
                let seg: usize = it.next()?.parse().ok()?;
                let tuple: Vec<usize> = it.map(|x| x.parse::<usize>().unwrap_or(0)).collect();
                Some((seg, tuple))
            })
            .collect(),
        Err(_) => Vec::new(),
    })
}

/// Per-retained-clue axis dump, for correlation / spread analysis.
thread_local! {
    static ROWS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[allow(clippy::too_many_arguments)]
pub fn row(
    score: f64,
    similarity: f64,
    novelty: f64,
    word_novelty: f64,
    familiarity: f64,
    rhythm: f64,
    shape: f64,
    closed: f64,
    v_punch: f64,
    v_foot: f64,
    v_econ: f64,
    v_land: f64,
) {
    ROWS.with(|r| {
        r.borrow_mut().push(format!(
            "{score:.9}\t{similarity:.6}\t{novelty:.6}\t{word_novelty:.6}\t{familiarity:.6}\t{rhythm:.6}\t{shape:.6}\t{closed:.6}\t{v_punch:.6}\t{v_foot:.6}\t{v_econ:.6}\t{v_land:.6}"
        ))
    });
}

pub fn flush() {
    let path = match std::env::var("MADGAB_558697_DUMP") {
        Ok(p) => p,
        Err(_) => return,
    };
    let mut out = String::from("score\tsim\tbnovel\twnov\tfam\trhythm\tshape\tclosed\tpunch\tfoot\tecon\tland\n");
    ROWS.with(|r| {
        for line in r.borrow().iter() {
            out.push_str(line);
            out.push('\n');
        }
        r.borrow_mut().clear();
    });
    if let Ok(mut f) = std::fs::File::create(&path) {
        use std::io::Write;
        let _ = f.write_all(out.as_bytes());
    }
}
