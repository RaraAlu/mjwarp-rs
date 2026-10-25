// QA independent probe for the exact f32 FMA helper (src/math/fma.rs).
//
// Self-written for the independent check; it does NOT use src/math/fma/tests.rs
// or any delivery probe/script. It INCLUDES the product implementation under test
// (src/math/fma.rs) at crate root so that the real `fma_f32` dispatch and the
// real `#[cfg(test)]` seam can be exercised; the sibling `tests.rs` in this
// directory is an empty stub only to satisfy the implementation's
// `#[cfg(test)] mod tests;` declaration.
//
// Reference: inline asm `vfmadd231ss` (hardware IEEE FMA). NOT a
// #[target_feature] wrapper around f32::mul_add (that falls back to the same
// non-faithful libm path at opt-level 0).
//
// Generated input set (team template): LCG mult=6364136223846793005,
// inc=1442695040888963407, seed=0x2026102500000001; value =
// f32(top24(state))/2^24*3-1 in [-1,2). A curated boundary set is added
// (ties, subnormals, overflow, +-0, inf/nan, cancellation, huge exponent gaps).
//
// Build: rustc --edition 2024 --test -C opt-level=0 -C debug-assertions=on \
//        [-C target-feature=+fma] -o <exe> qa_fma_probe.rs
// Run:   <exe> --nocapture --test-threads=1        (add MJWARP_FORCE_SOFT_FMA=1 to
//        additionally verify the env-driven seam gate)

include!("fma_snapshot.rs");

use std::hint::black_box;

const QA_LCG_MUL: u64 = 6_364_136_223_846_793_005;
const QA_LCG_INC: u64 = 1_442_695_040_888_963_407;
const QA_LCG_SEED: u64 = 0x2026_1025_0000_0001;

struct QaLcg {
    state: u64,
}

impl QaLcg {
    fn new() -> Self {
        QaLcg { state: QA_LCG_SEED }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(QA_LCG_MUL).wrapping_add(QA_LCG_INC);
        self.state
    }
    fn next_value(&mut self) -> f32 {
        let unit = ((self.next_u64() >> 40) as u32) as f32 / ((1u32 << 24) as f32);
        unit * 3.0 - 1.0
    }
}

#[inline]
unsafe fn qa_hw_fma(a: f32, b: f32, c: f32) -> f32 {
    let mut acc = c;
    unsafe {
        core::arch::asm!(
            "vfmadd231ss {acc}, {a}, {b}",
            acc = inout(xmm_reg) acc,
            a = in(xmm_reg) a,
            b = in(xmm_reg) b,
            options(pure, nomem, nostack, preserves_flags)
        );
    }
    acc
}

fn qa_ordered(value: f32) -> i64 {
    let bits = value.to_bits();
    if bits & 0x8000_0000 != 0 {
        0x8000_0000_i64 - i64::from(bits)
    } else {
        i64::from(bits)
    }
}

fn qa_ulp(left: f32, right: f32) -> i64 {
    (qa_ordered(left) - qa_ordered(right)).abs()
}

fn qa_fnv(mut digest: u64, value: u32) -> u64 {
    for byte in value.to_le_bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100_0000_01b3);
    }
    digest
}

const QA_SAMPLE_CAP: usize = 6;

#[derive(Default)]
struct QaStat {
    diffs: u64,
    max_ulp: i64,
    samples: Vec<(u32, u32, u32, u32, u32, i64)>,
    digest: u64,
}

impl QaStat {
    fn new() -> Self {
        QaStat { diffs: 0, max_ulp: 0, samples: Vec::new(), digest: 0xcbf2_9ce4_8422_2325 }
    }
    fn record(&mut self, a: f32, b: f32, c: f32, got: f32, want: f32) {
        self.digest = qa_fnv(self.digest, got.to_bits());
        let both_nan = got.is_nan() && want.is_nan();
        if got.to_bits() != want.to_bits() && !both_nan {
            self.diffs += 1;
            let ulps = if got.is_finite() && want.is_finite() {
                qa_ulp(got, want)
            } else {
                i64::MAX
            };
            self.max_ulp = self.max_ulp.max(ulps);
            if self.samples.len() < QA_SAMPLE_CAP {
                self.samples.push((a.to_bits(), b.to_bits(), c.to_bits(), got.to_bits(), want.to_bits(), ulps));
            }
        }
    }
    fn print(&self, label: &str, n: usize) {
        println!(
            "STAT {label} n={n} diffs={} max_ulp={} digest={:016x}",
            self.diffs, self.max_ulp, self.digest
        );
        for (a, b, c, got, want, ulps) in &self.samples {
            println!("SAMPLE {label} a_bits={a:08x} b_bits={b:08x} c_bits={c:08x} got_bits={got:08x} want_bits={want:08x} ulp={ulps}");
        }
    }
}

fn qa_generated(n: usize, got: impl Fn(f32, f32, f32) -> f32) -> QaStat {
    let mut stat = QaStat::new();
    let mut lcg = QaLcg::new();
    for _ in 0..n {
        let a = black_box(lcg.next_value());
        let b = black_box(lcg.next_value());
        let c = black_box(lcg.next_value());
        let got = got(a, b, c);
        let want = unsafe { qa_hw_fma(black_box(a), black_box(b), black_box(c)) };
        stat.record(a, b, c, got, want);
    }
    stat
}

fn qa_boundary_cases() -> Vec<(&'static str, f32, f32, f32)> {
    let bits = f32::from_bits;
    let one_plus = bits(0x3f80_0001); // 1 + 2^-23
    let one_minus = bits(0x3f7f_ffff); // 1 - 2^-24
    let half_ulp = bits(0x3380_0000); // 2^-24
    let third_ulp = bits(0x33c0_0000); // 3*2^-24
    let min_normal = bits(0x0080_0000);
    let min_sub = bits(0x0000_0001);
    let max_finite = bits(0x7f7f_ffff);
    let inf = f32::INFINITY;
    let nan = f32::NAN;
    vec![
        ("tie_1p0plushalf", 1.0, 1.0, half_ulp),
        ("tie_1p0plusthird", 1.0, 1.0, third_ulp),
        ("tie_known_337ffffe", one_plus, one_minus, -1.0),
        ("tie_known_33800000", one_plus, 1.0, -1.0),
        ("exact_2minus46", one_plus, 1.0 - 2.0f32.powi(-23), -1.0),
        ("cancel_to_zero", 3.0, 7.0, -21.0),
        ("cancel_to_sub", min_normal, 0.5, -min_normal * 0.5 + min_sub),
        ("sub_product", min_sub, min_sub, 0.0),
        ("sub_plus_normal", min_sub, 1.0, min_normal),
        ("normal_from_sub", min_normal, 0.5, min_normal * 0.5),
        ("carry_sub_to_normal", min_normal * 0.5, 1.0, min_normal * 0.5),
        ("overflow_exact_max", max_finite, 1.0, 0.0),
        ("overflow_product", max_finite, 2.0, 0.0),
        ("overflow_cancel", max_finite, 2.0, -max_finite),
        ("overflow_then_sub", max_finite, 2.0, max_finite * -1.0),
        ("neg_zero_mul", -0.0, 1.0, 0.0),
        ("neg_zero_add", -0.0, 1.0, -0.0),
        ("pos_zero_add_neg", 0.0, 1.0, -0.0),
        ("zero_product_neg_c", 0.0, 1.0, -1.0),
        ("negzero_times_one_plus_two", -0.0, 1.0, 2.0),
        ("one_times_zero_plus_five", 1.0, 0.0, 5.0),
        ("zero_times_negone_plus_half", 0.0, -1.0, 0.5),
        ("zero_b_zero_c_nonzero", 0.0, -0.0, 1.0),
        ("inf_times_zero", inf, 0.0, 1.0),
        ("inf_times_one_plus_inf", inf, 1.0, inf),
        ("inf_times_one_minus_inf", inf, 1.0, -inf),
        ("c_inf_finite_ab", 1.0, 2.0, inf),
        ("c_neg_inf_finite_ab", 1.0, 2.0, -inf),
        ("zero_a_c_inf", 0.0, 1.0, inf),
        ("nan_propagate_a", nan, 1.0, 1.0),
        ("nan_propagate_c", 1.0, 1.0, nan),
        ("huge_gap_pos", bits(0x7f00_0000), bits(0x7f00_0000), bits(0x0080_0000)),
        ("huge_gap_neg", bits(0x7f00_0000), bits(0x7f00_0000), bits(0x8080_0000)),
        ("tiny_product_big_c", bits(0x0080_0000), bits(0x0080_0000), 1.0e30),
        ("big_product_tiny_c", 1.0e30, 1.0, bits(0x0080_0000)),
        ("near_half_c", 1.0, 1.0, bits(0x337f_ffff)),
        ("near_half_c2", 1.0, 1.0, bits(0x3380_0001)),
        ("denorm_max_plus", bits(0x007f_ffff), 1.0, bits(0x0000_0001)),
        ("denorm_carry", bits(0x007f_ffff), 1.0, bits(0x007f_ffff)),
        ("sign_mix_1", -1.0, 1.0, 1.0),
        ("sign_mix_2", -1.0, -1.0, -1.0),
        ("sign_mix_3", 1.0, -1.0, -1.0),
        ("ulp_round_up", bits(0x3f7f_ffff), bits(0x3f7f_ffff), bits(0x0000_0001)),
        ("tie_subnormal_half", bits(0x0000_0003), 1.0, 0.0),
        ("subnormal_tie_2", bits(0x0000_0005), 0.5, 0.0),
        ("exact_43690", 7.0, 0.1, bits(0x3f00_0000)),
    ]
}

fn qa_boundary_row(stat: &mut QaStat, name: &str, a: f32, b: f32, c: f32, got: f32, want: f32) -> bool {
    let both_nan = got.is_nan() && want.is_nan();
    let mismatch = got.to_bits() != want.to_bits() && !both_nan;
    if mismatch {
        stat.record(a, b, c, got, want);
        println!(
            "BOUNDARY_MISMATCH name={name} a_bits={:08x} b_bits={:08x} c_bits={:08x} got_bits={:08x} want_bits={:08x}",
            a.to_bits(), b.to_bits(), c.to_bits(), got.to_bits(), want.to_bits()
        );
    }
    mismatch
}

fn qa_boundary(stat: &mut QaStat, got: impl Fn(f32, f32, f32) -> f32) -> (usize, usize) {
    let cases = qa_boundary_cases();
    let mut mismatches = 0;
    for (name, a, b, c) in &cases {
        let g = got(*a, *b, *c);
        let w = unsafe { qa_hw_fma(*a, *b, *c) };
        if qa_boundary_row(stat, name, *a, *b, *c, g, w) {
            mismatches += 1;
        }
    }
    (cases.len(), mismatches)
}

fn qa_relmat_chain(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    // Same 3-step k-ascending FMA chain as the product's
    // relmat::upstream_relative_matrix (row-major a^T * b), routed through the
    // product helper under test.
    let mut out = [0.0f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            let sum = fma_f32(a[row], b[column], 0.0);
            let sum = fma_f32(a[3 + row], b[3 + column], sum);
            out[row * 3 + column] = fma_f32(a[6 + row], b[6 + column], sum);
        }
    }
    out
}

#[test]
fn qa_exact_fma_probe() {
    if !std::arch::is_x86_feature_detected!("fma") {
        println!("ERROR no_fma3_cpu");
        return;
    }

    // env-driven seam gate (product test seam; env only compiled under cfg(test))
    test_seam::enable_from_env();
    let seam_env = test_seam::forced_soft();
    println!(
        "INFO cpuid_fma={} active_path={} seam_env={}",
        hardware_fma_available(),
        active_path(),
        seam_env
    );
    // env-driven seam behavioural check: with the env set, the product entry must
    // dispatch to the software path; with it unset it must use hardware.
    {
        let got = fma_f32(0.0, 1.0, -1.0);
        let want = unsafe { qa_hw_fma(0.0, 1.0, -1.0) };
        println!(
            "SEAM_ENV_CHECK seam_env={} fma_f32(0,1,-1)_bits={:08x} hw_bits={:08x} equal={}",
            seam_env,
            got.to_bits(),
            want.to_bits(),
            got.to_bits() == want.to_bits()
        );
    }

    // A) dispatch path with seam OFF (real production dispatch: CPUID -> hw asm)
    test_seam::set_forced(false);
    let dispatch_4096 = qa_generated(4096, fma_f32);
    let dispatch_1m = qa_generated(1_000_000, fma_f32);

    // B) seam-forced fallback path through the same product entry
    test_seam::set_forced(true);
    let seam_4096 = qa_generated(4096, fma_f32);
    let seam_1m = qa_generated(1_000_000, fma_f32);

    // C) direct software implementation vs hardware (same function seam forces)
    let soft_4096 = qa_generated(4096, soft_fma);
    let soft_1m = qa_generated(1_000_000, soft_fma);

    // D) boundary set through both paths
    test_seam::set_forced(false);
    let mut boundary_dispatch = QaStat::new();
    let (boundary_n, boundary_dispatch_mismatches) = qa_boundary(&mut boundary_dispatch, fma_f32);
    test_seam::set_forced(true);
    let mut boundary_seam = QaStat::new();
    let (_, boundary_seam_mismatches) = qa_boundary(&mut boundary_seam, fma_f32);

    // E) discriminate power: GNU default f32::mul_add vs hardware FMA
    let muladd_4096 = qa_generated(4096, |a, b, c| black_box(a.mul_add(b, c)));

    // F) path A vs path B identical?
    let mut cross = QaStat::new();
    let mut lcg = QaLcg::new();
    for _ in 0..1_000_000 {
        let a = lcg.next_value();
        let b = lcg.next_value();
        let c = lcg.next_value();
        test_seam::set_forced(false);
        let from_dispatch = fma_f32(a, b, c);
        test_seam::set_forced(true);
        let from_seam = fma_f32(a, b, c);
        cross.record(a, b, c, from_dispatch, from_seam);
    }

    // G) relmat chain analogue: does the fallback corrupt the product's 3-FMA chain?
    let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let rz90 = [0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    let ry45 = [
        0.70710677, 0.0, 0.70710677, 0.0, 1.0, 0.0, -0.70710677, 0.0, 0.70710677,
    ];
    for (name, m1, m2) in [
        ("identity_x_identity", identity, identity),
        ("rz90_x_identity", rz90, identity),
        ("rz90_x_rz90", rz90, rz90),
        ("ry45_x_rz90", ry45, rz90),
    ] {
        test_seam::set_forced(false);
        let hardware_path = qa_relmat_chain(&m1, &m2);
        test_seam::set_forced(true);
        let seam_path = qa_relmat_chain(&m1, &m2);
        let mut cells_mismatched = 0;
        for cell in 0..9 {
            if hardware_path[cell].to_bits() != seam_path[cell].to_bits() {
                cells_mismatched += 1;
            }
        }
        println!(
            "RELMAT_CHAIN name={name} cells_mismatched={cells_mismatched}/9 hw_cell0={:08x} seam_cell0={:08x}",
            hardware_path[0].to_bits(),
            seam_path[0].to_bits()
        );
    }

    test_seam::set_forced(seam_env);

    dispatch_4096.print("dispatch_4096", 4096);
    dispatch_1m.print("dispatch_1e6", 1_000_000);
    seam_4096.print("seam_4096", 4096);
    seam_1m.print("seam_1e6", 1_000_000);
    soft_4096.print("soft_4096", 4096);
    soft_1m.print("soft_1e6", 1_000_000);
    boundary_dispatch.print("boundary_dispatch", boundary_n);
    boundary_seam.print("boundary_seam", boundary_n);
    muladd_4096.print("muladd_4096", 4096);
    cross.print("dispatch_vs_seam_1e6", 1_000_000);
    println!(
        "SUMMARY dispatch_1e6_diffs={} seam_1e6_diffs={} soft_1e6_diffs={} boundary_dispatch={} boundary_seam={} muladd_diffs={} cross={}",
        dispatch_1m.diffs, seam_1m.diffs, soft_1m.diffs,
        boundary_dispatch_mismatches, boundary_seam_mismatches, muladd_4096.diffs, cross.diffs
    );
}
