// MECHANICAL SNAPSHOT of src/math/fma.rs for the independent probe.
// Only inner-doc prefix '//!' -> '//' at line starts was changed (rustc E0753);
// all other bytes are identical. See qa_fma_snapshot.json.
// 目标无关的 f32 精确 FMA（IEEE-754 单次正确舍入的 `a*b+c`）。
//
// # 契约
// - **全 f32、正确舍入、无 f64**：本模块只出现 f32 与整数运算，不使用 f64 工作区；
//   契约依据 `docs/numerical-precision-contract.md` L313/L344/L454。
// - 语义：`fma_f32(a,b,c)` = IEEE-754 单次正确舍入的 `a*b+c`（round-to-nearest-even），
//   与 GPU 的 `fma.rn.f32` 同语义。
//
// # 路径选择（每进程只判定一次）
// 1. 主路：`is_x86_feature_detected!("fma")` 为真时走内联汇编 `vfmadd231ss`（单指令、IEEE 正确
//    舍入）；CPUID 结果由 `OnceLock<bool>` 缓存，**每进程只检测一次**，缓存后不再读 CPUID。
// 2. 兜底：无 FMA 的机器自动退到**纯软件正确舍入**（f32 位型分解 + 整数精确乘积/求和 + RNE
//    舍入；无 f64、无平台假设）。该路径也用于在 FMA 机器上通过测试 seam 验证兜底正确性。
//
// # 合规论证（NP01 类型映射：整数中间量不构成类型偏离）
// - 兜底路径使用整数，**仅用于「精确模拟硬件 FMA 的单次舍入」**：整数只承载 f32 位型分解后的
//   有效位乘积/和（576 位精确整数），不引入任何新的浮点精度层级，也不产生第二个浮点结果；
//   `fma_f32` 的输入与输出类型始终是 `f32`。
// - 契约禁止的是「内部 f64 工作区/升精度规避原生误差」（L313/L344/L454）；整数精确运算不是
//   浮点升精度，结果是对 `f32` 单次正确舍入的实现，因此不进入 NP01 的「实际精度偏离」计数。
// - NP01 审计记录：`src/math/fma.rs`（整数中间量，结果 f32，2026-10-25）——见
//   `docs/numerical-precision-contract.md` 的 NP01 段注记与
//   `docs/development-evidence/exact-f32-fma-relmat/` 的 `f64-audit.log`。
//
// # 测试 seam（仅测试构建存在，产品构建不编译、不读取）
// `MJWARP_FORCE_SOFT_FMA` 只在 `#[cfg(test)]` 的测试模块里读取并置位进程内标志；
// 产品构建（debug/release/默认特性）不包含任何 env 读取或 seam 分支（见证据的源码/二进制检查）。
//
// # 兜底正确舍入依据（tie / 次正规 / 溢出 / 特殊值）
// - 输入按 f32 位型精确分解 `(-1)^s * mant * 2^exp`（次正规按 `mant*2^-149`）；
// - 乘积 `mant_a*mant_b` 用 u64、与 c 对齐后用 576 位整数精确相加/相减，**无中间舍入**；
// - 最后按 RNE 舍入到 f32：保留 24 位，guard=次高位、sticky=其余低位；tie（guard=1、sticky=0）
//   取偶；次正规按 2^-149 单位舍入，进位到 2^-126 时升级为最小正规数；超出 `f32::MAX` 按 RNE
//   得 ±Inf；零按 IEEE 符号规则（`+0 + -0 = +0`）。因此 tie/边界与硬件 FMA 舍入判定一致
//   （由 4096/1e6/边界集对硬件 FMA 0 差异验证）。
//
// # 安全前提
// 内联汇编 `vfmadd231ss` 只在 `is_x86_feature_detected!("fma")` 为真时执行；调用点已确认该 CPU
// 支持 FMA。汇编仅操作 xmm 标量寄存器，不访问内存、不改变标志位、不弹栈。

use std::sync::OnceLock;

const LIMBS: usize = 9;
const SIGN_BIT: u32 = 0x8000_0000;
const EXP_MASK: u32 = 0x7f80_0000;
const FRAC_MASK: u32 = 0x007f_ffff;

#[cfg(target_arch = "x86_64")]
static HAS_FMA: OnceLock<bool> = OnceLock::new();

/// 本机（当前进程）是否支持并选择硬件 FMA 主路；结果缓存一次。
pub(crate) fn hardware_fma_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        *HAS_FMA.get_or_init(|| std::arch::is_x86_feature_detected!("fma"))
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// 证据用：当前构建实际会选择哪条路径（不含测试 seam）；仅测试构建使用。
#[cfg(test)]
pub(crate) fn active_path() -> &'static str {
    if hardware_fma_available() {
        "hardware"
    } else {
        "software"
    }
}

/// 生产入口：有 FMA 走硬件指令，否则走纯软件正确舍入。
#[inline]
pub(crate) fn fma_f32(a: f32, b: f32, c: f32) -> f32 {
    #[cfg(test)]
    {
        if test_seam::forced_soft() {
            return soft_fma(a, b, c);
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        if hardware_fma_available() {
            // SAFETY: `hardware_fma_available()` 已确认本 CPU 支持 FMA；汇编仅操作 xmm 标量。
            return unsafe { hw_fma(a, b, c) };
        }
    }
    soft_fma(a, b, c)
}

/// 主路：`vfmadd231ss`（单指令 FMA，IEEE 正确舍入）。
///
/// # Safety
/// 调用者必须已确认 `is_x86_feature_detected!("fma")` 为真。
#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn hw_fma(a: f32, b: f32, c: f32) -> f32 {
    let mut accumulator = c;
    // SAFETY: 见函数安全前提；`vfmadd231ss` 只写目标 xmm 寄存器，不读写内存、不弹栈、不改标志位。
    unsafe {
        core::arch::asm!(
            "vfmadd231ss {acc}, {a}, {b}",
            acc = inout(xmm_reg) accumulator,
            a = in(xmm_reg) a,
            b = in(xmm_reg) b,
            options(pure, nomem, nostack, preserves_flags)
        );
    }
    accumulator
}

/// 测试专用 seam：仅 `#[cfg(test)]` 编译，产品构建不包含任何 env 读取。
#[cfg(test)]
mod test_seam {
    use std::sync::atomic::{AtomicBool, Ordering};

    static FORCED_SOFT: AtomicBool = AtomicBool::new(false);

    /// 由测试模块读取 `MJWARP_FORCE_SOFT_FMA` 并置位进程内标志。
    pub(super) fn enable_from_env() {
        let enabled = std::env::var_os("MJWARP_FORCE_SOFT_FMA")
            .map(|value| value != "0")
            .unwrap_or(false);
        FORCED_SOFT.store(enabled, Ordering::Relaxed);
    }

    pub(super) fn forced_soft() -> bool {
        FORCED_SOFT.load(Ordering::Relaxed)
    }

    pub(super) fn set_forced(enabled: bool) {
        FORCED_SOFT.store(enabled, Ordering::Relaxed);
    }
}

/// 软件正确舍入 FMA（无 f64；f32 位型 + 整数精确运算）。
pub(crate) fn soft_fma(a: f32, b: f32, c: f32) -> f32 {
    let a = Decomposed::new(a);
    let b = Decomposed::new(b);
    let c = Decomposed::new(c);
    if a.kind == Kind::Nan || b.kind == Kind::Nan || c.kind == Kind::Nan {
        return f32::NAN;
    }
    let a_inf = a.kind == Kind::Inf;
    let b_inf = b.kind == Kind::Inf;
    if a_inf || b_inf {
        if (a_inf && b.is_zero()) || (b_inf && a.is_zero()) {
            return f32::NAN;
        }
        let sign = a.sign ^ b.sign;
        if c.kind == Kind::Inf {
            return if c.sign != sign { f32::NAN } else { inf(sign) };
        }
        return inf(sign);
    }
    if c.kind == Kind::Inf {
        return c.bits as f32;
    }
    if a.is_zero() || b.is_zero() {
        let product_sign = a.sign ^ b.sign;
        if c.is_zero() {
            return zero(if product_sign == c.sign {
                product_sign
            } else {
                false
            });
        }
        return c.bits as f32;
    }
    let product_sign = a.sign ^ b.sign;
    if c.is_zero() {
        let mut magnitude = [0u64; LIMBS];
        add_shifted(&mut magnitude, a.mant * b.mant, 0);
        return round_to_f32(&magnitude, a.exp + b.exp, product_sign);
    }

    let product_mant = a.mant * b.mant;
    let product_exp = a.exp + b.exp;
    let common_exp = product_exp.min(c.exp);
    let product_shift = (product_exp - common_exp) as usize;
    let c_shift = (c.exp - common_exp) as usize;
    if product_sign == c.sign {
        let mut magnitude = [0u64; LIMBS];
        add_shifted(&mut magnitude, product_mant, product_shift);
        add_shifted(&mut magnitude, c.mant, c_shift);
        round_to_f32(&magnitude, common_exp, product_sign)
    } else {
        let mut product = [0u64; LIMBS];
        add_shifted(&mut product, product_mant, product_shift);
        let mut summand = [0u64; LIMBS];
        add_shifted(&mut summand, c.mant, c_shift);
        match compare(&product, &summand) {
            std::cmp::Ordering::Equal => zero(false),
            std::cmp::Ordering::Greater => {
                sub_shifted(&mut product, c.mant, c_shift);
                round_to_f32(&product, common_exp, product_sign)
            }
            std::cmp::Ordering::Less => {
                sub_shifted(&mut summand, product_mant, product_shift);
                round_to_f32(&summand, common_exp, c.sign)
            }
        }
    }
}

fn inf(sign: bool) -> f32 {
    f32::from_bits(if sign { SIGN_BIT } else { 0 } | EXP_MASK)
}

fn zero(sign: bool) -> f32 {
    f32::from_bits(if sign { SIGN_BIT } else { 0 })
}

#[derive(Clone, Copy)]
struct Decomposed {
    sign: bool,
    mant: u64,
    exp: i32,
    kind: Kind,
    bits: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Zero,
    Finite,
    Inf,
    Nan,
}

impl Decomposed {
    fn new(value: f32) -> Self {
        let bits = value.to_bits();
        let sign = bits & SIGN_BIT != 0;
        let exponent = ((bits & EXP_MASK) >> 23) as i32;
        let fraction = (bits & FRAC_MASK) as u64;
        if exponent == 0xff {
            let kind = if fraction != 0 { Kind::Nan } else { Kind::Inf };
            return Self {
                sign,
                mant: 0,
                exp: 0,
                kind,
                bits,
            };
        }
        if exponent == 0 {
            let kind = if fraction == 0 {
                Kind::Zero
            } else {
                Kind::Finite
            };
            return Self {
                sign,
                mant: fraction,
                exp: -149,
                kind,
                bits,
            };
        }
        Self {
            sign,
            mant: fraction | 0x80_0000,
            exp: exponent - 150,
            kind: Kind::Finite,
            bits,
        }
    }

    fn is_zero(self) -> bool {
        self.kind == Kind::Zero
    }
}

fn add_shifted(destination: &mut [u64; LIMBS], value: u64, shift: usize) {
    if value == 0 {
        return;
    }
    let limb = shift / 64;
    let offset = shift % 64;
    let low = value << offset;
    let high = if offset == 0 {
        0
    } else {
        value >> (64 - offset)
    };
    add_limb(destination, limb, low);
    if high != 0 {
        add_limb(destination, limb + 1, high);
    }
}

fn add_limb(destination: &mut [u64; LIMBS], index: usize, value: u64) {
    let mut carry = value;
    let mut position = index;
    while carry != 0 && position < LIMBS {
        let (sum, overflow) = destination[position].overflowing_add(carry);
        destination[position] = sum;
        carry = u64::from(overflow);
        position += 1;
    }
}

fn sub_shifted(destination: &mut [u64; LIMBS], value: u64, shift: usize) {
    if value == 0 {
        return;
    }
    let limb = shift / 64;
    let offset = shift % 64;
    let low = value << offset;
    let high = if offset == 0 {
        0
    } else {
        value >> (64 - offset)
    };
    sub_limb(destination, limb, low);
    if high != 0 {
        sub_limb(destination, limb + 1, high);
    }
}

fn sub_limb(destination: &mut [u64; LIMBS], index: usize, value: u64) {
    let mut borrow = value;
    let mut position = index;
    while borrow != 0 && position < LIMBS {
        let (difference, underflow) = destination[position].overflowing_sub(borrow);
        destination[position] = difference;
        borrow = u64::from(underflow);
        position += 1;
    }
}

fn compare(left: &[u64; LIMBS], right: &[u64; LIMBS]) -> std::cmp::Ordering {
    for index in (0..LIMBS).rev() {
        match left[index].cmp(&right[index]) {
            std::cmp::Ordering::Equal => {}
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

fn highest_bit(magnitude: &[u64; LIMBS]) -> Option<usize> {
    for index in (0..LIMBS).rev() {
        if magnitude[index] != 0 {
            return Some(index * 64 + (63 - magnitude[index].leading_zeros() as usize));
        }
    }
    None
}

fn bit_at(magnitude: &[u64; LIMBS], position: usize) -> bool {
    let limb = position / 64;
    limb < LIMBS && magnitude[limb] >> (position % 64) & 1 != 0
}

fn any_below(magnitude: &[u64; LIMBS], limit: usize) -> bool {
    if limit == 0 {
        return false;
    }
    let limb = limit / 64;
    let offset = limit % 64;
    for value in magnitude.iter().take(limb) {
        if *value != 0 {
            return true;
        }
    }
    offset > 0 && limb < LIMBS && magnitude[limb] & ((1u64 << offset) - 1) != 0
}

fn extract(magnitude: &[u64; LIMBS], shift: usize, count: usize) -> u32 {
    let mut result = 0u32;
    for index in 0..count {
        if bit_at(magnitude, shift + index) {
            result |= 1 << index;
        }
    }
    result
}

/// 把 `magnitude * 2^exponent` 按 RNE 舍入为 f32。
fn round_to_f32(magnitude: &[u64; LIMBS], exponent: i32, sign: bool) -> f32 {
    let Some(highest) = highest_bit(magnitude) else {
        return zero(sign);
    };
    let mut value_exp = exponent + highest as i32;
    if value_exp > 127 {
        return inf(sign);
    }
    if value_exp >= -126 {
        let mut mantissa = if highest >= 23 {
            let shift = highest - 23;
            let truncated = extract(magnitude, shift, 24);
            if shift == 0 {
                truncated
            } else {
                let guard = bit_at(magnitude, shift - 1);
                let sticky = any_below(magnitude, shift - 1);
                if guard && (sticky || truncated & 1 != 0) {
                    let rounded = truncated + 1;
                    if rounded == 1 << 24 {
                        value_exp += 1;
                        1 << 23
                    } else {
                        rounded
                    }
                } else {
                    truncated
                }
            }
        } else {
            extract(magnitude, 0, 24) << (23 - highest)
        };
        if value_exp > 127 {
            return inf(sign);
        }
        if mantissa >= 1 << 24 {
            mantissa >>= 1;
            value_exp += 1;
            if value_exp > 127 {
                return inf(sign);
            }
        }
        let biased = (value_exp + 127) as u32;
        let bits = (if sign { SIGN_BIT } else { 0 }) | (biased << 23) | (mantissa & FRAC_MASK);
        return f32::from_bits(bits);
    }
    // 次正规：按 2^-149 单位舍入。
    let shift = exponent + 149;
    let mantissa = if shift >= 0 {
        let value = u64::from(extract(magnitude, 0, 24)) << shift;
        if value >= 1 << 24 {
            1 << 24
        } else {
            value as u32
        }
    } else {
        let drop = (-shift) as usize;
        let truncated = extract(magnitude, drop, 24);
        let guard = drop > 0 && bit_at(magnitude, drop - 1);
        let sticky = drop > 1 && any_below(magnitude, drop - 1);
        if guard && (sticky || truncated & 1 != 0) {
            truncated + 1
        } else {
            truncated
        }
    };
    if mantissa >= 1 << 23 {
        return f32::from_bits((if sign { SIGN_BIT } else { 0 }) | 0x0080_0000);
    }
    if mantissa == 0 {
        return zero(sign);
    }
    f32::from_bits((if sign { SIGN_BIT } else { 0 }) | mantissa)
}

#[cfg(test)]
mod tests;
