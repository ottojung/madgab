//! PROBE INSTRUMENTATION - scratch branch only.
use std::sync::Mutex;
use std::sync::OnceLock;

pub static SLOTS: Mutex<Vec<(String, usize, Vec<Vec<(usize,String,f64)>>)>> = Mutex::new(Vec::new());
pub fn note_slots(t: String, id: usize, slots: Vec<Vec<(usize,String,f64)>>) { SLOTS.lock().unwrap().push((t,id,slots)); }
pub fn dump_slots() {
    let g = SLOTS.lock().unwrap();
    let mut out = String::new();
    for (t,id,slots) in g.iter() {
        out.push_str(&format!("SEG\t{}\t{}\t{}\n", serde_escape(t), id, slots.len()));
        for (k,sl) in slots.iter().enumerate() {
            for (i,(wi,w,c)) in sl.iter().enumerate() {
                out.push_str(&format!("  {}\t{}\t{}\t{:.4}\n", k, i, serde_escape(w), c));
            }
        }
    }
    let path = std::env::var("MADGAB_PROBE_SLOTS").unwrap_or_else(|_| "/tmp/probe_slots.tsv".into());
    std::fs::write(path, out).unwrap();
}
pub struct Rec {
    pub target: String,
    pub depth: usize,
    pub widths: Vec<usize>,
    pub cap: usize,
    pub max_pushed: Vec<usize>,
    pub n_pushed: Vec<usize>,
    pub profile: Vec<Vec<usize>>,
    pub profile_outside: usize,
    pub emitted: usize,
    pub popped: usize,
    pub stages: usize,
    pub hit: Option<(usize, usize, f64, Vec<usize>)>,
    pub spans: Vec<(usize, usize)>,
    pub minbound: Option<f64>,
    pub target_bound: Option<f64>,
}

fn store() -> &'static Mutex<Vec<Rec>> {
    static S: OnceLock<Mutex<Vec<Rec>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn note(r: Rec) {
    store().lock().unwrap().push(r);
}

pub fn reset() {
    store().lock().unwrap().clear();
}

pub fn dump() {
    let g = store().lock().unwrap();
    let mut out = String::new();
    out.push_str("[\n");
    for (n, r) in g.iter().enumerate() {
        if n > 0 { out.push_str(",\n"); }
        out.push_str(&format!(
            "{{\"target\":{},\"depth\":{},\"widths\":{:?},\"cap\":{},\"max_pushed\":{:?},\"n_pushed\":{:?},\"profile\":{:?},\"profile_outside\":{},\"emitted\":{},\"popped\":{},\"stages\":{},\"hit\":{},\"spans\":{},\"minbound\":{},\"tb\":{}}}",
            serde_escape(&r.target), r.depth, r.widths, r.cap, r.max_pushed, r.n_pushed,
            r.profile, r.profile_outside, r.emitted, r.popped, r.stages,
            serde_opt(&r.hit),
            format!("[{}]", r.spans.iter().map(|(a,b)| format!("({},{})",a,b)).collect::<Vec<_>>().join(",")),
            fmt_f(r.minbound), fmt_f(r.target_bound)));
    }
    out.push_str("\n]\n");
    let path = std::env::var("MADGAB_PROBE_OUT").unwrap_or_else(|_| "/tmp/probe.json".into());
    std::fs::write(path, out).unwrap();
}

fn fmt_f(x: Option<f64>) -> String { match x { None => "null".to_string(), Some(v) => format!("{:.9}", v) } }

fn serde_opt(h: &Option<(usize, usize, f64, Vec<usize>)>) -> String {
    match h { None => "null".to_string(), Some((a,b,c,d)) => format!("[{},{},{:.6},{:?}]", a,b,c,d) }
}

fn serde_escape(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
