//! MEASUREMENT-ONLY probe (front c81e55).  Throwaway.
//!
//! Gated on `MADGAB_C81E55`; unset, every call returns immediately and no
//! file is opened.  It writes one TSV file recording, per segmentation, the
//! per-slot score components of every candidate, plus every index tuple the
//! coverage reserve and the per-segmentation traversal actually offered, so
//! that the *score* of a tuple that was never built can be reconstructed
//! offline and compared with the score of the tuples that were.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub const ENV: &str = "MADGAB_C81E55";

fn with_sink(f: impl FnOnce(&mut BufWriter<File>)) {
    use std::sync::Mutex;
    static SINK: Mutex<Option<BufWriter<File>>> = Mutex::new(None);
    let mut guard = SINK.lock().unwrap();
    if guard.is_none() {
        if let Ok(path) = std::env::var(ENV) {
            let path = PathBuf::from(path);
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(file) = File::create(path) {
                *guard = Some(BufWriter::new(file));
            }
        }
    }
    if let Some(w) = guard.as_mut() {
        f(w);
    }
}

fn fmt_f(x: f64) -> String {
    format!("{:.6}", x)
}

/// One `SEG` row: the segmentation's shape.  Columns:
/// `SEG id phase nslots word_count target_syllables shared target_inner_count
/// novelty spans widths`
pub fn segmentation(
    seg_index: usize,
    phase: usize,
    spans: &[(usize, usize)],
    word_count: f64,
    target_syllables: usize,
    shared: usize,
    target_inner_count: usize,
    novelty: f64,
    widths: &[usize],
) {
    let spans_str: Vec<String> = spans
        .iter()
        .map(|sp| format!("{}-{}", sp.0, sp.1))
        .collect();
    let widths_str: Vec<String> = widths.iter().map(|x| x.to_string()).collect();
    with_sink(|w| {
    let _ = writeln!(
        w,
        "SEG\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{},{}",
        seg_index,
        phase,
        spans.len(),
        fmt_f(word_count),
        target_syllables,
        shared,
        target_inner_count,
        fmt_f(novelty),
        spans_str.join(","),
        widths_str.join(",")
    );
    });
}

/// One `ALT` row: the score components of one candidate of one slot.
/// `cost reused familiarity closed shape syllables`
pub fn alt(seg_index: usize, slot: usize, i: usize, c: (f64, usize, f64, usize, f64, usize), word: &str) {
    with_sink(|w| {
    let _ = writeln!(
        w,
        "ALT\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        seg_index,
        slot,
        i,
        fmt_f(c.0),
        c.1,
        fmt_f(c.2),
        c.3,
        fmt_f(c.4),
        c.5,
        word
    );
    });
}

/// One `TUPLE` row: an index tuple offered to the pool, with its source
/// (`reserve` or `traversal`) and whether `build` accepted it.
pub fn tuple(seg_index: usize, source: &str, accepted: bool, t: &[usize]) {
    let parts: Vec<String> = t.iter().map(|x| x.to_string()).collect();
    with_sink(|w| {
    let _ = writeln!(
        w,
        "TUPLE\t{}\t{}\t{}\t{}",
        seg_index,
        source,
        u8::from(accepted),
        parts.join(",")
    );
    });
}

pub fn flush() {
    with_sink(|w| {
        let _ = w.flush();
    });
}

/// MEASUREMENT-ONLY.  Reads `MADGAB_C81E55_INJECT` = `<seg_index>,<i>,<i>,...`
/// and asks the caller to push that index tuple for that segmentation into
/// the pool through the normal `build`, so the *production* scorer and the
/// production `select_diverse` rank it.  Nothing is hard-coded: the tuple
/// comes from the environment and the only word this module knows is none.
pub fn inject(seg_index: usize) -> Option<Vec<usize>> {
    let spec = std::env::var("MADGAB_C81E55_INJECT").ok()?;
    let mut out = None;
    for part in spec.split(";") {
        let mut it = part.split(",");
        let want = it.next()?.trim().parse::<usize>().ok()?;
        if want == seg_index {
            out = Some(it.filter_map(|x| x.trim().parse::<usize>().ok()).collect());
        }
    }
    out
}
