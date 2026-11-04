//! 用户签名的判据 allowlist（2026-11-04 22:14，签名级）在渲染验证用例里的**显式实现**：
//! 仅 cam2 的 1 个像素 `(x=134, y=128)`（参考 seg = `(flexid 0, 9)`）允许 depth
//! `ULP <= MAX_DEPTH_ULP(4)` **且** `beyond4 = 0`；其余全部严格位等。
//! 禁止「总数减 1」等隐式做法，也不得把放宽扩到相机级/区域级/条件式。
//!
//! 卡：`2026-11-04-render-r06-s3b-flex-allowlist-assert-fix`（p80；零 GPU、只改用例）。
//! 溯源：attempt5 `6e9aa585` FAILED_ROUND —— 旧断言严于签名 allowlist（观测 cam2 depth
//! `(1,0)` 落在允许范围）；本模块把签名 allowlist 实现进断言，其余保持 bit-first。
//! 签名/allowlist 落库由 Arch 治理卡承载；本模块只记录用例实现。

use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, ulp_distance};

/// allowlist 只作用于该相机（cam2）。
pub(crate) const ALLOWLIST_CAMERA: usize = 2;
/// allowlist 像素列（cam2 行主序 flat = `ALLOWLIST_PIXEL_Y * width + ALLOWLIST_PIXEL_X`）。
pub(crate) const ALLOWLIST_PIXEL_X: usize = 134;
/// allowlist 像素行。
pub(crate) const ALLOWLIST_PIXEL_Y: usize = 128;
/// allowlist 像素的参考图元：flexid 0。
pub(crate) const ALLOWLIST_FLEXID: i32 = 0;
/// 参考 seg 的 object type（`mjOBJ_FLEX = 9`）。
pub(crate) const ALLOWLIST_SEG_TYPE: i32 = 9;

/// 单相机深度断言（GPU 用例与 CPU 自证共用同一实现路径）。
///
/// - 严格区：逐像素 `bits == expected_bits`；任何失配立即失败，失败信息打印
///   `(mismatched, beyond, max_ulp, bits)` + flat/x/y 定位（bits 为首个失配像素的渲染位型）。
/// - allowlist 像素（仅 `ALLOWLIST_CAMERA` 且 flat 命中）：允许 `ULP <= MAX_DEPTH_ULP`
///   且 `beyond4 = 0`；失败信息额外打印该像素 bits/expected/ulp；并校验参考 seg 确实是
///   `(flexid 0, 9)`，防止 allowlist 被误挂到别的图元。
#[rustfmt::skip]
pub(crate) fn assert_depth_allowlisted(camera: usize, width: usize, rendered: &[f32], expected_bits: &[u32], reference_seg: &[[i32; 2]]) {
    assert_eq!(rendered.len(), expected_bits.len(), "cam{camera} depth 长度");
    assert_eq!(expected_bits.len(), reference_seg.len(), "cam{camera} depth/seg 长度");
    let allowlist_flat = if camera == ALLOWLIST_CAMERA { ALLOWLIST_PIXEL_Y * width + ALLOWLIST_PIXEL_X } else { usize::MAX };
    assert!(allowlist_flat == usize::MAX || allowlist_flat < expected_bits.len(), "allowlist 像素 ({ALLOWLIST_PIXEL_X},{ALLOWLIST_PIXEL_Y}) 超出 cam{camera} 尺寸");
    let (mut strict_mismatched, mut strict_beyond, mut strict_max_ulp, mut strict_first) = (0usize, 0usize, 0i64, None::<(usize, u32, u32, i64)>);
    let mut allowlist_observed = None::<(u32, u32, i64)>;
    for (flat, (value, expected)) in rendered.iter().zip(expected_bits).enumerate() {
        let (bits, ulp) = (value.to_bits(), ulp_distance(value.to_bits(), *expected));
        if flat == allowlist_flat {
            assert_eq!(reference_seg[flat], [ALLOWLIST_FLEXID, ALLOWLIST_SEG_TYPE], "cam{camera} allowlist 像素 ({ALLOWLIST_PIXEL_X},{ALLOWLIST_PIXEL_Y}) 的参考 seg 不是 (flexid{ALLOWLIST_FLEXID}, {ALLOWLIST_SEG_TYPE})");
            allowlist_observed = Some((bits, *expected, ulp));
            continue;
        }
        if bits != *expected {
            strict_mismatched += 1;
            strict_beyond += usize::from(ulp > MAX_DEPTH_ULP);
            strict_max_ulp = strict_max_ulp.max(ulp);
            if strict_first.is_none() {
                strict_first = Some((flat, bits, *expected, ulp));
            }
        }
    }
    if let Some((flat, bits, _expected, _ulp)) = strict_first {
        panic!("cam{camera} depth 严格区失配：mismatched={strict_mismatched}, beyond={strict_beyond}, max_ulp={strict_max_ulp}, bits={bits:#010x} (flat={flat}, x={}, y={})", flat % width, flat / width);
    }
    assert_eq!(strict_beyond, 0, "cam{camera} depth 严格区 beyond={strict_beyond}");
    if let Some((bits, expected, ulp)) = allowlist_observed {
        let beyond = usize::from(ulp > MAX_DEPTH_ULP);
        assert_eq!(beyond, 0, "cam{camera} allowlist 像素 ({ALLOWLIST_PIXEL_X},{ALLOWLIST_PIXEL_Y}) 超 4 ULP：mismatched=1, beyond={beyond}, max_ulp={ulp}, bits={bits:#010x}, expected={expected:#010x}");
        assert!(ulp <= MAX_DEPTH_ULP, "cam{camera} allowlist 像素 ({ALLOWLIST_PIXEL_X},{ALLOWLIST_PIXEL_Y}) ULP 越界：mismatched=1, beyond={beyond}, max_ulp={ulp}, bits={bits:#010x}");
    }
}

/// 只读统计行（成功路径打印；不改变任何断言/阈值）：逐相机 `(mismatched, beyond4, max_ulp)`，
/// cam2 额外返回 allowlist 像素明细行（bits/expected/ULP/beyond4/flexid）。
#[rustfmt::skip]
pub(crate) fn format_depth_stats(camera: usize, width: usize, rendered: &[f32], expected_bits: &[u32]) -> (String, Option<String>) {
    let (mut mismatched, mut beyond, mut max_ulp) = (0usize, 0usize, 0i64);
    for (value, expected) in rendered.iter().zip(expected_bits) {
        let ulp = ulp_distance(value.to_bits(), *expected);
        mismatched += usize::from(value.to_bits() != *expected);
        beyond += usize::from(ulp > MAX_DEPTH_ULP);
        max_ulp = max_ulp.max(ulp);
    }
    let line = format!("R06-S3b depth cam{camera}: mismatched={mismatched} beyond4={beyond} max_ulp={max_ulp}");
    let flat = ALLOWLIST_PIXEL_Y * width + ALLOWLIST_PIXEL_X;
    let allow = if camera == ALLOWLIST_CAMERA && flat < expected_bits.len() {
        let (bits, expected) = (rendered[flat].to_bits(), expected_bits[flat]);
        let ulp = ulp_distance(bits, expected);
        Some(format!("R06-S3b allowlist cam{camera} ({},{}): bits={bits:#010x} expected={expected:#010x} ulp={ulp} beyond4={} flexid={ALLOWLIST_FLEXID}", ALLOWLIST_PIXEL_X, ALLOWLIST_PIXEL_Y, usize::from(ulp > MAX_DEPTH_ULP)))
    } else { None };
    (line, allow)
}

/// 打印统计行（GPU 成功路径调用；CPU 自证用 `format_depth_stats` 断言格式）。
#[rustfmt::skip]
pub(crate) fn print_depth_stats(camera: usize, width: usize, rendered: &[f32], expected_bits: &[u32]) {
    let (line, allow) = format_depth_stats(camera, width, rendered, expected_bits);
    println!("{line}");
    if let Some(allow) = allow { println!("{allow}"); }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// cam2 allowlist 像素的 flat index（与内核 depth 行主序一致）。
    #[rustfmt::skip]
    const CPU_ALLOWLIST_FLAT: usize = ALLOWLIST_PIXEL_Y * 320 + ALLOWLIST_PIXEL_X;

    /// 合成 cam2 深度/seg：全像素位等，仅 `mutations` 指定像素按 `delta` 改动位型；
    /// 参考 seg 在 allowlist 像素置为 `(flexid0, 9)`。
    #[rustfmt::skip]
    fn cpu_depth_case(mutations: &[(usize, i64)]) -> (Vec<f32>, Vec<u32>, Vec<[i32; 2]>) {
        let width = 320usize;
        let pixels = width * 240;
        let expected: Vec<u32> = (0..pixels).map(|i| 0x4000_0000u32.wrapping_add((i as u32) & 0x3ff)).collect();
        let mut rendered_bits = expected.clone();
        let mut rendered = Vec::with_capacity(pixels);
        for (flat, bits) in rendered_bits.iter_mut().enumerate() {
            for (target, delta) in mutations {
                if *target == flat { *bits = (expected[flat] as i64 + delta) as u32; }
            }
            rendered.push(f32::from_bits(*bits));
        }
        let mut reference_seg = vec![[0i32, 0i32]; pixels];
        reference_seg[CPU_ALLOWLIST_FLAT] = [ALLOWLIST_FLEXID, ALLOWLIST_SEG_TYPE];
        (rendered, expected, reference_seg)
    }

    #[rustfmt::skip]
    fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
        if let Some(text) = payload.downcast_ref::<String>() { return text.clone(); }
        if let Some(text) = payload.downcast_ref::<&str>() { return (*text).to_string(); }
        "<non-string panic>".to_string()
    }

    /// 期望生产断言 panic；捕获并返回 panic 信息（临时静默 hook，避免污染测试输出）。
    #[rustfmt::skip]
    fn catch_assert_failure<F: FnOnce() + std::panic::UnwindSafe>(body: F) -> String {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = std::panic::catch_unwind(body);
        std::panic::set_hook(previous);
        panic_message(outcome.expect_err("生产断言必须失败"))
    }

    /// 自证 ①：allowlist 像素在 `(1,0)` 且 ULP≤4 → PASS（走生产断言路径）。
    #[test]
    #[rustfmt::skip]
    fn allowlist_pixel_within_4_ulp_passes_cpu() {
        let (rendered, expected, reference_seg) = cpu_depth_case(&[(CPU_ALLOWLIST_FLAT, 3)]);
        let mismatched = rendered.iter().zip(&expected).filter(|(v, e)| v.to_bits() != **e).count();
        assert_eq!(mismatched, 1, "自证前提：allowlist 像素应恰好 1 个 mismatched");
        assert_eq!(rendered[CPU_ALLOWLIST_FLAT].to_bits(), expected[CPU_ALLOWLIST_FLAT].wrapping_add(3), "自证前提：该像素 ULP=3");
        assert_depth_allowlisted(ALLOWLIST_CAMERA, 320, &rendered, &expected, &reference_seg);
    }

    /// 自证 ②：allowlist 像素 `beyond4 >= 1`（ULP=5）→ 必须失败，且失败信息包含该像素明细。
    #[test]
    #[rustfmt::skip]
    fn allowlist_pixel_beyond_4_ulp_fails_cpu() {
        let (rendered, expected, reference_seg) = cpu_depth_case(&[(CPU_ALLOWLIST_FLAT, 5)]);
        let message = catch_assert_failure(|| assert_depth_allowlisted(ALLOWLIST_CAMERA, 320, &rendered, &expected, &reference_seg));
        assert!(message.contains("超 4 ULP"), "失败信息应打印 beyond/ulp/bits：{message}");
        assert!(message.contains("mismatched=1"), "失败信息应打印 mismatched：{message}");
    }

    /// 自证 ③：除 allowlist 像素外任一像素 mismatched=1 → 必须失败（证明没有放宽到相机级）。
    #[test]
    #[rustfmt::skip]
    fn non_allowlisted_pixel_mismatch_fails_cpu() {
        let other = 100usize;
        let (rendered, expected, reference_seg) = cpu_depth_case(&[(CPU_ALLOWLIST_FLAT, 3), (other, 1)]);
        let message = catch_assert_failure(|| assert_depth_allowlisted(ALLOWLIST_CAMERA, 320, &rendered, &expected, &reference_seg));
        assert!(message.contains("严格区失配"), "失败信息应打印严格区明细：{message}");
        assert!(message.contains("bits=0x"), "失败信息应打印失配像素 bits：{message}");
        // 严格区**不统计** allowlist 像素（该像素单独判定），所以这里严格区只应看到那 1 个非白名单失配。
        assert!(message.contains("mismatched=1"), "严格区应只统计到 1 个非白名单失配：{message}");
        assert!(message.contains("flat=100"), "失败信息应定位到非白名单像素行：{message}");
    }

    /// 附：allowlist 只作用于 `ALLOWLIST_CAMERA`（cam0/cam1 同 flat 像素照旧严格）。
    #[test]
    #[rustfmt::skip]
    fn allowlist_is_camera_scoped_cpu() {
        let (rendered, expected, reference_seg) = cpu_depth_case(&[(CPU_ALLOWLIST_FLAT, 3)]);
        for camera in [0usize, 1usize] {
            let message = catch_assert_failure(|| assert_depth_allowlisted(camera, 320, &rendered, &expected, &reference_seg));
            assert!(message.contains(&format!("cam{camera} depth 严格区失配")), "cam{camera} 不应受 allowlist 放宽：{message}");
        }
    }
    /// 自证 ④：统计打印格式在 CPU 合成输入上可断言（成功路径只读打印，不改变断言）。
    #[test]
    #[rustfmt::skip]
    fn depth_stats_lines_format_cpu() {
        let (rendered, expected, _reference_seg) = cpu_depth_case(&[(CPU_ALLOWLIST_FLAT, 3)]);
        let (line, allow) = format_depth_stats(ALLOWLIST_CAMERA, 320, &rendered, &expected);
        assert!(line.contains("R06-S3b depth cam2: mismatched=1 beyond4=0 max_ulp=3"), "{line}");
        let allow = allow.expect("cam2 应有 allowlist 明细行");
        assert!(allow.contains("bits=0x") && allow.contains("ulp=3") && allow.contains("beyond4=0") && allow.contains("flexid=0"), "{allow}");
        assert!(format_depth_stats(0, 320, &rendered, &expected).1.is_none(), "cam0 不应打印 allowlist 行");
    }
}
