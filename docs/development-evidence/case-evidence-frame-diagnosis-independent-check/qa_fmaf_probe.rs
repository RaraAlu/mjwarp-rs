// QA independent probe for f32::mul_add build semantics.
// Own implementation for the independent check of the case_evidence frame
// diagnosis; not derived from the delivery probe source.
//
// Build matrix (probe is compiled separately for each cell):
//   rustc <toolchain> --edition 2024 -C opt-level=0 -C debug-assertions=on \
//         [-C target-feature=+fma] -o probe.exe qa_fmaf_probe.rs
//
// Input generator (documented so the statistics are reproducible):
//   64-bit LCG: state <- state * 6364136223846793005 + 1442695040888963407
//   seed = 0x2026102500000001
//   unit  = f32(top 24 bits) / 2^24                (exact, in [0,1))
//   value = unit * 3 - 1                           (in [-1,2))
//   case i: a, b, c = next three values (LCG order a,b,c)
//
// References per case:
//   r_hw   : IEEE single-rounding FMA, computed through
//            #[target_feature(enable = "fma")] fma.rn (CPU must expose FMA3)
//   r_no   : non-fused a*b + c (black_box forced rounding between the steps)
//   r_f64  : (a as f64 * b as f64 + c as f64) as f32 (f64 double rounding)
//
// The value under test is a.mul_add(b, c) from the build being run.

use std::hint::black_box;
use std::env;

const LCG_MUL: u64 = 6_364_136_223_846_793_005;
const LCG_INC: u64 = 1_442_695_040_888_963_407;
const LCG_SEED: u64 = 0x2026_1025_0000_0001;

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new() -> Self {
        Lcg { state: LCG_SEED }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(LCG_MUL).wrapping_add(LCG_INC);
        self.state
    }
    fn next_value(&mut self) -> f32 {
        // Top 24 bits -> exact integer in [0, 2^24) -> exact f32 in [0,1).
        let unit = ((self.next_u64() >> 40) as u32) as f32 / ((1u32 << 24) as f32);
        unit * 3.0 - 1.0
    }
}

/// True hardware IEEE FMA via inline asm (`vfmadd231ss d, a, b` = a*b + d).
/// Inline asm is emitted verbatim: it does not depend on crate-level
/// `target-feature=+fma`, so the reference is identical in all four builds
/// (the `#[target_feature]`-only variant is NOT reliable at opt-level 0:
/// the non-inlined `f32::mul_add` shim still falls back to the libm `fmaf`).
#[inline]
unsafe fn fma_hw(a: f32, b: f32, c: f32) -> f32 {
    let mut acc = c;
    unsafe {
        core::arch::asm!(
            "vfmadd231ss {acc}, {a}, {b}",
            acc = inout(xmm_reg) acc,
            a = in(xmm_reg) a,
            b = in(xmm_reg) b,
            options(nostack, nomem, pure),
        );
    }
    acc
}

fn unfused(a: f32, b: f32, c: f32) -> f32 {
    let product = black_box(a * b);
    black_box(product + c)
}

fn f64_double_round(a: f32, b: f32, c: f32) -> f32 {
    let product = black_box(f64::from(a) * f64::from(b));
    let sum = black_box(product + f64::from(c));
    sum as f32
}

fn ordered(value: f32) -> i64 {
    let bits = value.to_bits();
    if bits & 0x8000_0000 != 0 {
        0x8000_0000_i64 - i64::from(bits)
    } else {
        i64::from(bits)
    }
}

fn ulp_distance(left: f32, right: f32) -> u64 {
    (ordered(left) - ordered(right)).unsigned_abs()
}

fn fnv1a(mut digest: u64, value: u32) -> u64 {
    for byte in value.to_le_bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100_0000_01b3);
    }
    digest
}

const SAMPLE_CAP: usize = 5;

struct Comparison {
    name: &'static str,
    diffs: u64,
    max_ulp: u64,
    dup_ulp1: u64,
    dup_ulp_ge2: u64,
    samples: Vec<(u32, u32, u32, u32, u32, u64)>,
    worst: Option<(u32, u32, u32, u32, u32, u64)>,
}

impl Comparison {
    fn new(name: &'static str) -> Self {
        Comparison {
            name,
            diffs: 0,
            max_ulp: 0,
            dup_ulp1: 0,
            dup_ulp_ge2: 0,
            samples: Vec::new(),
            worst: None,
        }
    }

    fn record(&mut self, a: f32, b: f32, c: f32, got: f32, want: f32) {
        if got.to_bits() != want.to_bits() {
            self.diffs += 1;
            let ulps = ulp_distance(got, want);
            self.max_ulp = self.max_ulp.max(ulps);
            if ulps == 1 {
                self.dup_ulp1 += 1;
            } else {
                self.dup_ulp_ge2 += 1;
            }
            let sample = (
                a.to_bits(),
                b.to_bits(),
                c.to_bits(),
                got.to_bits(),
                want.to_bits(),
                ulps,
            );
            if self.worst.map_or(true, |current| ulps > current.5) {
                self.worst = Some(sample);
            }
            if self.samples.len() < SAMPLE_CAP {
                self.samples.push(sample);
            }
        }
    }
}

fn main() {
    let n: usize = env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1_000_000);
    if !std::is_x86_feature_detected!("fma") {
        println!("ERROR cpu_fma3_missing");
        std::process::exit(2);
    }

    let global_fma = cfg!(target_feature = "fma");
    let mut comparisons = [
        Comparison::new("muladd_vs_hw_fma"),
        Comparison::new("muladd_vs_unfused"),
        Comparison::new("muladd_vs_f64_double"),
        Comparison::new("hw_fma_vs_f64_double"),
        Comparison::new("hw_fma_vs_unfused"),
        Comparison::new("unfused_vs_f64_double"),
    ];
    let (mut d_mul, mut d_hw, mut d_no, mut d_64) = (0xcbf2_9ce4_8422_2325u64, 0xcbf2_9ce4_8422_2325u64,
                                                   0xcbf2_9ce4_8422_2325u64, 0xcbf2_9ce4_8422_2325u64);
    let mut lcg = Lcg::new();
    for _ in 0..n {
        let a = lcg.next_value();
        let b = lcg.next_value();
        let c = lcg.next_value();
        let mul = black_box(a.mul_add(b, c));
        let hw = unsafe { black_box(fma_hw(a, b, c)) };
        let no = black_box(unfused(a, b, c));
        let f64r = black_box(f64_double_round(a, b, c));
        d_mul = fnv1a(d_mul, mul.to_bits());
        d_hw = fnv1a(d_hw, hw.to_bits());
        d_no = fnv1a(d_no, no.to_bits());
        d_64 = fnv1a(d_64, f64r.to_bits());
        comparisons[0].record(a, b, c, mul, hw);
        comparisons[1].record(a, b, c, mul, no);
        comparisons[2].record(a, b, c, mul, f64r);
        comparisons[3].record(a, b, c, hw, f64r);
        comparisons[4].record(a, b, c, hw, no);
        comparisons[5].record(a, b, c, no, f64r);
    }

    println!(
        "PROBE n={n} global_target_feature_fma={global_fma} cpu_fma3=true"
    );
    for comparison in &comparisons {
        println!(
            "CMP {} diffs={} max_ulp={} ulp_eq_1={} ulp_ge_2={}",
            comparison.name, comparison.diffs, comparison.max_ulp, comparison.dup_ulp1, comparison.dup_ulp_ge2
        );
        for (a, b, c, got, want, ulps) in &comparison.samples {
            println!(
                "SAMPLE {} a_bits={a:08x} b_bits={b:08x} c_bits={c:08x} got_bits={got:08x} want_bits={want:08x} ulp={ulps}",
                comparison.name
            );
        }
        if let Some((a, b, c, got, want, ulps)) = comparison.worst {
            println!(
                "SAMPLE_WORST {} a_bits={a:08x} b_bits={b:08x} c_bits={c:08x} got_bits={got:08x} want_bits={want:08x} ulp={ulps}",
                comparison.name
            );
        }
    }
    println!("DIGEST muladd={d_mul:016x} hw_fma={d_hw:016x} unfused={d_no:016x} f64_double={d_64:016x}");

    // Known vectors (values are mathematical, not copied from any probe source).
    let known: [(f32, f32, f32); 5] = [
        (f32::from_bits(0x3f80_0001), f32::from_bits(0x3f7f_ffff), -1.0),
        (1.0, 1.0, -1.0),
        (3.0, 7.0, -21.0),
        (0.1, 0.2, 0.3),
        (f32::from_bits(0x3f80_0001), 1.0, -1.0),
    ];
    for (a, b, c) in known {
        let mul = a.mul_add(b, c);
        let no = unfused(a, b, c);
        let hw = unsafe { fma_hw(a, b, c) };
        let f64r = f64_double_round(a, b, c);
        println!(
            "KNOWN a_bits={:08x} b_bits={:08x} c_bits={:08x} muladd={:08x} unfused={:08x} hw={:08x} f64={:08x}",
            a.to_bits(), b.to_bits(), c.to_bits(), mul.to_bits(), no.to_bits(), hw.to_bits(), f64r.to_bits()
        );
    }
}
