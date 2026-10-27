// QA probe: does serde_json (workspace features, no float_roundtrip) parse the
// decimal text of the oracle f64 values back to the exact bit pattern?
use std::io::{BufRead, BufReader};

fn main() {
    let path = std::env::args().nth(1).expect("cases file");
    let file = std::fs::File::open(path).expect("open cases");
    let mut total = 0u64;
    let mut bad = 0u64;
    let mut shown = 0u64;
    for line in BufReader::new(file).lines() {
        let line = line.expect("line");
        let mut parts = line.split_whitespace();
        let text = parts.next().expect("text");
        let expect = u64::from_str_radix(parts.next().expect("bits"), 16).expect("hex");
        let parsed: f64 = serde_json::from_str(text).expect("parse");
        total += 1;
        if parsed.to_bits() != expect {
            bad += 1;
            if shown < 8 {
                shown += 1;
                println!("MISMATCH {text} parsed={:016x} expected={expect:016x}", parsed.to_bits());
            }
        }
    }
    println!("total={total} mismatched={bad}");
}
