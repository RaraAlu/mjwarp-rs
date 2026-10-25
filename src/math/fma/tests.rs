//! `math::fma` 的探针与正确性测试（默认构建可跑；无 GPU/NVRTC 依赖）。
//!
//! 探针模板（QA 复核口径，团队通用）：输入生成器参数写死、对硬件 FMA 0 差异、
//! 记录 ≥5 条差异样本与「残余率仅对该分布/规模有效」声明。这里的差异样本来自
//! `f32::mul_add`（非硬件路径）对硬件 FMA，用于证明探针有判别力。

use super::*;

/// QA 复核口径的输入生成器：LCG `mult/inc/seed` 固定，映射 `f32(top24)/2^24*3-1 ∈ [-1,2)`。
const LCG_MULT: u64 = 6_364_136_223_846_793_005;
const LCG_INC: u64 = 1_442_695_040_888_963_407;
const LCG_SEED: u64 = 0x2026_1025_0000_0001;
const SAMPLE_1M: usize = 1_000_000;

fn next_state(state: &mut u64) {
    *state = state.wrapping_mul(LCG_MULT).wrapping_add(LCG_INC);
}

fn operand(state: &mut u64) -> f32 {
    next_state(state);
    let top24 = ((*state >> 40) as u32) & 0x00ff_ffff;
    (top24 as f32) / 16_777_216.0 * 3.0 - 1.0
}

fn generated(count: usize) -> Vec<(f32, f32, f32)> {
    let mut state = LCG_SEED;
    let mut triples = Vec::with_capacity(count);
    for _ in 0..count {
        triples.push((
            operand(&mut state),
            operand(&mut state),
            operand(&mut state),
        ));
    }
    triples
}

fn sample_4096() -> &'static [(f32, f32, f32)] {
    static SAMPLES: OnceLock<Vec<(f32, f32, f32)>> = OnceLock::new();
    SAMPLES.get_or_init(|| generated(4096))
}

fn sample_1m() -> &'static [(f32, f32, f32)] {
    static SAMPLES: OnceLock<Vec<(f32, f32, f32)>> = OnceLock::new();
    SAMPLES.get_or_init(|| generated(SAMPLE_1M))
}

/// 边界集：次正规、近半 ULP tie、指数差极大、抵消到次正规/零、±0、溢出、NaN/Inf。
fn boundary_cases() -> Vec<(f32, f32, f32)> {
    let min_subnormal = f32::from_bits(1);
    let max_subnormal = f32::from_bits(0x007f_ffff);
    let min_normal = f32::from_bits(0x0080_0000);
    let half_min_subnormal = f32::from_bits(0x0000_0001); // 2^-149
    let one_plus_ulp = f32::from_bits(0x3f80_0001); // 1+2^-23
    let one_minus_ulp = f32::from_bits(0x3f7f_ffff); // 1-2^-23
    let pow2_24 = f32::from_bits(0x4b80_0000); // 2^24
    let small_24 = f32::from_bits(0x3380_0000); // 2^-24
    let small_47 = f32::from_bits(0x2b80_0000); // 2^-47
    let mut cases = vec![
        // ±0 组合
        (0.0, 0.0, 0.0),
        (-0.0, 0.0, 0.0),
        (0.0, -0.0, 0.0),
        (-0.0, -0.0, -0.0),
        (0.0, 1.0, -0.0),
        (-1.0, -0.0, 0.0),
        // 普通/精确值
        (1.0, 1.0, -1.0),
        (-1.0, 1.0, 1.0),
        (one_plus_ulp, one_minus_ulp, -1.0),
        (one_plus_ulp, one_plus_ulp, -(1.0 + 2.0 * 2f32.powi(-23))),
        // 近半 ULP 的 tie（精确和 = 1 + 1.5*2^-23）
        (1.0, 1.0, 1.5 * small_24),
        (1.0, 1.0, 2.5 * small_24),
        (1.0, 1.0, small_24 + small_47),
        (1.0, 1.0, small_24 - small_47),
        (one_plus_ulp, 1.0, small_24),
        // 指数差极大（c 远大于积 / 积远大于 c）
        (two_pow_neg_100(), two_pow_neg_100(), 1.0e30),
        (1.0e30, 1.0e30, two_pow_neg_100()),
        (min_subnormal, min_subnormal, 1.0),
        (1.0, 1.0, min_subnormal),
        // 抵消、到次正规/零
        (min_normal, -1.0, 1.0),
        (max_subnormal, 1.0, -1.0),
        (two_pow_neg_75(), two_pow_neg_75(), -two_pow_neg_150()),
        (two_pow_neg_65(), 1.0, -two_pow_neg_65()),
        (two_pow_neg_24(), two_pow_neg_24(), -two_pow_neg_48()),
        (two_pow_neg_149(), 1.0, -two_pow_neg_149()),
        (half_min_subnormal, 1.0, -half_min_subnormal),
        // 溢出边界
        (f32::MAX, 2.0, 0.0),
        (f32::MAX, 1.0, f32::MAX),
        (f32::MAX, -1.0, f32::MAX),
        (-f32::MAX, 2.0, 0.0),
        (1.0e30, 1.0e30, 0.0),
        (max_subnormal, max_subnormal, 0.0),
        // 近 1.0 的极端
        (one_minus_ulp, pow2_24, 0.0),
        (one_plus_ulp, pow2_24, 0.0),
        // 特殊值
        (f32::INFINITY, 1.0, 0.0),
        (f32::INFINITY, 0.0, 0.0),
        (f32::INFINITY, 1.0, f32::NEG_INFINITY),
        (f32::NEG_INFINITY, 1.0, f32::INFINITY),
        (f32::NAN, 1.0, 1.0),
        (1.0, f32::NAN, 1.0),
        (1.0, 1.0, f32::NAN),
        (f32::INFINITY, f32::NAN, 1.0),
    ];
    // 生成一批「近半 ULP」样本：向 1.0 的 ulp 边界附近逼近。
    for step in 0..8u32 {
        let value = f32::from_bits(0x3f80_0000 + step);
        cases.push((
            value,
            1.0,
            2f32.powi(-24) * (1.0 + 2f32.powi(-23) * step as f32),
        ));
        cases.push((
            value,
            -1.0,
            -2f32.powi(-24) * (1.0 + 2f32.powi(-23) * step as f32),
        ));
    }
    cases
}

fn two_pow_neg_24() -> f32 {
    2f32.powi(-24)
}

fn two_pow_neg_48() -> f32 {
    2f32.powi(-48)
}

fn two_pow_neg_65() -> f32 {
    2f32.powi(-65)
}

fn two_pow_neg_75() -> f32 {
    2f32.powi(-75)
}

fn two_pow_neg_100() -> f32 {
    2f32.powi(-100)
}

fn two_pow_neg_149() -> f32 {
    f32::from_bits(1)
}

fn two_pow_neg_150() -> f32 {
    // 2^-150 不可由 f32 精确表示；用 2^-149 的一半（算符里只用于「抵消到次正规」场景）。
    f32::from_bits(1) * 0.5
}

#[cfg(target_arch = "x86_64")]
fn hardware(a: f32, b: f32, c: f32) -> f32 {
    // SAFETY: 测试在本机 CPUID 已确认 FMA 的机器上运行（缺失时由调用方跳过）。
    unsafe { super::hw_fma(a, b, c) }
}

fn same(left: f32, right: f32) -> bool {
    if left.is_nan() || right.is_nan() {
        left.is_nan() && right.is_nan()
    } else {
        left.to_bits() == right.to_bits()
    }
}

fn digest(triples: &[(f32, f32, f32)], f: impl Fn(f32, f32, f32) -> f32) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &(a, b, c) in triples {
        let result = f(a, b, c);
        // NaN payload 不是 IEEE 语义的一部分：统一折叠为规范 quiet NaN，便于跨路径比 digest。
        let bits = if result.is_nan() {
            f32::NAN.to_bits()
        } else {
            result.to_bits()
        };
        hash ^= u64::from(bits);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn count_mismatches(triples: &[(f32, f32, f32)], f: impl Fn(f32, f32, f32) -> f32) -> usize {
    triples
        .iter()
        .filter(|&&(a, b, c)| !same(f(a, b, c), hardware(a, b, c)))
        .count()
}

fn print_samples(triples: &[(f32, f32, f32)], f: impl Fn(f32, f32, f32) -> f32, wanted: usize) {
    let mut printed = 0;
    for &(a, b, c) in triples {
        let got = f(a, b, c);
        let want = hardware(a, b, c);
        if !same(got, want) {
            println!(
                "  SAMPLE a={:08x} b={:08x} c={:08x} | got={:08x} want={:08x}",
                a.to_bits(),
                b.to_bits(),
                c.to_bits(),
                got.to_bits(),
                want.to_bits()
            );
            printed += 1;
            if printed == wanted {
                break;
            }
        }
    }
}

fn print_path() {
    println!(
        "PATH cpuid_fma={} active_path={}",
        hardware_fma_available(),
        active_path()
    );
}

fn require_hardware() -> bool {
    if hardware_fma_available() {
        true
    } else {
        println!("SKIP 本机无 FMA，硬件对照测试不执行（兜底路径由逻辑/已知向量覆盖）");
        false
    }
}

#[test]
fn soft_matches_hardware_on_generated_4096() {
    print_path();
    if !require_hardware() {
        return;
    }
    let samples = sample_4096();
    let mismatches = count_mismatches(samples, soft_fma);
    println!(
        "4096 soft vs hw: mismatches={mismatches} soft_digest={:016x} hw_digest={:016x}",
        digest(samples, soft_fma),
        digest(samples, hardware)
    );
    assert_eq!(mismatches, 0, "兜底路径对硬件 FMA 差异应为 0（4096 组）");
}

#[test]
fn soft_matches_hardware_on_generated_1m() {
    print_path();
    if !require_hardware() {
        return;
    }
    let samples = sample_1m();
    let mismatches = count_mismatches(samples, soft_fma);
    println!(
        "1e6 soft vs hw: mismatches={mismatches} soft_digest={:016x} hw_digest={:016x}",
        digest(samples, soft_fma),
        digest(samples, hardware)
    );
    assert_eq!(mismatches, 0, "兜底路径对硬件 FMA 差异应为 0（1e6 组）");
    println!("RESIDUAL_RATE 该分布/规模（LCG seed=0x2026102500000001，1e6 组）残余率 = 0");
}

#[test]
fn soft_matches_hardware_on_boundary_cases() {
    print_path();
    if !require_hardware() {
        return;
    }
    let cases = boundary_cases();
    let mut mismatches = 0;
    for &(a, b, c) in &cases {
        let got = soft_fma(a, b, c);
        let want = hardware(a, b, c);
        if !same(got, want) {
            mismatches += 1;
            println!(
                "  BOUNDARY MISMATCH a={:08x} b={:08x} c={:08x} got={:08x} want={:08x}",
                a.to_bits(),
                b.to_bits(),
                c.to_bits(),
                got.to_bits(),
                want.to_bits()
            );
        }
    }
    println!("boundary cases={} mismatches={mismatches}", cases.len());
    assert_eq!(mismatches, 0, "兜底路径对硬件 FMA 边界集差异应为 0");
}

#[test]
fn dispatch_matches_hardware_and_soft_on_same_samples() {
    print_path();
    if !require_hardware() {
        return;
    }
    let samples = sample_4096();
    let boundary = boundary_cases();
    let mut all = samples.to_vec();
    all.extend_from_slice(&boundary);
    let dispatch_mismatches = count_mismatches(&all, fma_f32);
    let soft_mismatches = count_mismatches(&all, soft_fma);
    println!(
        "dispatch vs hw={dispatch_mismatches} soft vs hw={soft_mismatches} dispatch_digest={:016x} soft_digest={:016x}",
        digest(&all, fma_f32),
        digest(&all, soft_fma)
    );
    assert_eq!(
        dispatch_mismatches, 0,
        "主路（按 CPUID 派发）对硬件 FMA 差异应为 0"
    );
    assert_eq!(soft_mismatches, 0, "兜底对硬件 FMA 差异应为 0");
    assert_eq!(
        digest(&all, fma_f32),
        digest(&all, soft_fma),
        "主路与兜底同批样本应逐位一致"
    );
}

#[test]
fn forced_soft_seam_matches_hardware() {
    print_path();
    if !require_hardware() {
        return;
    }
    super::test_seam::enable_from_env();
    let forced = std::env::var_os("MJWARP_FORCE_SOFT_FMA")
        .map(|value| value != "0")
        .unwrap_or(false);
    println!("SEAM forced_soft_from_env={forced}");
    if !forced {
        println!(
            "SEAM 未启用（MJWARP_FORCE_SOFT_FMA 未设置）；跳过 seam 断言，主路/兜底由其它测试覆盖"
        );
        return;
    }
    let samples = sample_4096();
    let boundary = boundary_cases();
    let mut all = samples.to_vec();
    all.extend_from_slice(&boundary);
    let mismatches = count_mismatches(&all, fma_f32);
    println!(
        "SEAM path={} samples={} mismatches={mismatches} seam_digest={:016x} hw_digest={:016x}",
        if super::test_seam::forced_soft() {
            "software(seam)"
        } else {
            "hardware"
        },
        all.len(),
        digest(&all, fma_f32),
        digest(&all, hardware)
    );
    super::test_seam::set_forced(false);
    assert_eq!(mismatches, 0, "seam 强制兜底路径对硬件 FMA 差异应为 0");
}

/// 判别力自证：非硬件 FMA 的 `f32::mul_add`（如 GNU 默认构建的库回退）与硬件 FMA 的差异样本。
/// 该测试不因差异数是否为零失败（MSVC 的 fmaf 可能已正确）；差异样本用于证据。
#[test]
fn mul_add_library_fallback_discrimination() {
    print_path();
    if !require_hardware() {
        return;
    }
    let samples = sample_4096();
    let mismatches = samples
        .iter()
        .filter(|&&(a, b, c)| !same(a.mul_add(b, c), hardware(a, b, c)))
        .count();
    println!("mul_add(f32) vs hw: mismatches={mismatches} / 4096");
    print_samples(samples, |a, b, c| a.mul_add(b, c), 5);
    if mismatches == 0 {
        println!("mul_add(f32) 在本构建已等价于硬件 FMA（无差异样本）");
    }
}
