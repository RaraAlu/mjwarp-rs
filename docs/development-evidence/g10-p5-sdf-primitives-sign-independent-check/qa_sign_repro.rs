//! 独立最小复现：改前 `f32::signum()` / 手写三分支 sign vs 上游 `wp.sign`（`x < 0 ? -1 : 1`）。
//!
//! 由 `qa_sign_check.py` 用 rustc 编译并运行；输出以 u32 位型（十六进制）打印，避免 NaN/±0 的显示歧义。

fn old_signum(x: f32) -> f32 {
    x.signum()
}

fn old_compute_grad_sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn new_sign(x: f32) -> f32 {
    if x < 0.0 {
        -1.0
    } else {
        1.0
    }
}

fn upstream_expected(x: f32) -> f32 {
    // 与 pinned warp include/warp-headers/builtin.h:765 的 `x < T(0) ? -1 : 1` 逐字同构。
    if x < 0.0 {
        -1.0
    } else {
        1.0
    }
}

fn bits(x: f32) -> String {
    format!("{:08x}", x.to_bits())
}

fn main() {
    let inputs: [(&str, f32); 7] = [
        ("-0.0", -0.0),
        ("0.0", 0.0),
        ("NaN", f32::NAN),
        ("-1.5", -1.5),
        ("2.5", 2.5),
        ("-inf", f32::NEG_INFINITY),
        ("inf", f32::INFINITY),
    ];

    for (name, x) in inputs {
        let old = old_signum(x);
        let new = new_sign(x);
        let up = upstream_expected(x);
        let same = old.to_bits() == up.to_bits();
        println!(
            "SIGN name={} in={} old={} new={} upstream={} old_matches_upstream={} new_matches_upstream={}",
            name, bits(x), bits(old), bits(new), bits(up), same,
            new.to_bits() == up.to_bits()
        );
        let old_cg = old_compute_grad_sign(x);
        let new_cg = new_sign(x);
        println!(
            "CG name={} in={} old={} new={} upstream={} old_matches_upstream={} new_matches_upstream={}",
            name, bits(x), bits(old_cg), bits(new_cg), bits(up),
            old_cg.to_bits() == up.to_bits(), new_cg.to_bits() == up.to_bits()
        );
    }

    // radial_field 形态：field 基线 [0.25, 0.5, 0.75] 逐分量乘 sign(x[i])。
    let field = [0.25_f32, 0.5, 0.75];
    let cases: [(&str, [f32; 3]); 3] = [
        ("all_neg_zero", [-0.0, -0.0, -0.0]),
        ("mixed_zero", [0.0, -0.0, 1.0]),
        ("nan_component", [f32::NAN, -1.0, 0.0]),
    ];
    for (name, x) in cases {
        let old = [
            field[0] * old_signum(x[0]),
            field[1] * old_signum(x[1]),
            field[2] * old_signum(x[2]),
        ];
        let new = [
            field[0] * new_sign(x[0]),
            field[1] * new_sign(x[1]),
            field[2] * new_sign(x[2]),
        ];
        println!(
            "RF name={} old=[{},{},{}] new=[{},{},{}]",
            name, bits(old[0]), bits(old[1]), bits(old[2]),
            bits(new[0]), bits(new[1]), bits(new[2])
        );
    }

    // compute_grad 形态：gradient 初值 + max_grad * sign(max_val)。
    let gradient = [1.0_f32, 2.0, 3.0];
    let max_grad = [10.0_f32, 20.0, 30.0];
    for (name, max_val) in [("0.0", 0.0_f32), ("-0.0", -0.0), ("NaN", f32::NAN), ("5.0", 5.0), ("-5.0", -5.0)] {
        let old_s = old_compute_grad_sign(max_val);
        let new_s = new_sign(max_val);
        let old_g = [
            gradient[0] + max_grad[0] * old_s,
            gradient[1] + max_grad[1] * old_s,
            gradient[2] + max_grad[2] * old_s,
        ];
        let new_g = [
            gradient[0] + max_grad[0] * new_s,
            gradient[1] + max_grad[1] * new_s,
            gradient[2] + max_grad[2] * new_s,
        ];
        println!(
            "GRAD name={} old_sign={} new_sign={} old_grad=[{},{},{}] new_grad=[{},{},{}]",
            name, bits(old_s), bits(new_s),
            bits(old_g[0]), bits(old_g[1]), bits(old_g[2]),
            bits(new_g[0]), bits(new_g[1]), bits(new_g[2])
        );
    }
}
