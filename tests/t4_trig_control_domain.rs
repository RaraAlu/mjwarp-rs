//! 独立核对极分解的三角控制域。
//! 不求三角输出，不运行完整迭代或设备。
#![cfg(feature = "cuda-probe")]

use serde_json::Value;
use sha2::{Digest, Sha256};

const CONVERGENCE: f32 = 1.0e-6;
const LARGE_ANGLE: f32 = f32::from_bits(0x47ce_4780);
const AUDIT: &str = include_str!("../docs/evidence/t4-g03-retest/face-trig/audit.json");

fn pinned(row: &Value) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join(row["path"].as_str().unwrap())).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let hash = if row["normalization"] == "LF" {
        format!("{:x}", Sha256::digest(text.replace("\r\n", "\n")))
    } else {
        format!("{:x}", Sha256::digest(bytes))
    };
    assert_eq!(hash, row["sha256"].as_str().unwrap());
    text.trim_end_matches('\0').replace("\r\n", "\n")
}

#[derive(Debug, PartialEq)]
enum Branch {
    Converged,
    Fast,
    Slow,
    Infinite,
    Unordered,
}

fn branch(w: f32) -> Branch {
    if w < CONVERGENCE {
        Branch::Converged
    } else {
        let angle = 0.5_f32 * w;
        // PTX的ltu包含无序；不把NaN误算为整数归约。
        if angle.is_nan() {
            Branch::Unordered
        } else if angle.abs() < LARGE_ANGLE {
            Branch::Fast
        } else if angle.is_infinite() {
            Branch::Infinite
        } else {
            Branch::Slow
        }
    }
}

#[test]
fn classifies_convergence_range_reduction_and_nonfinite_boundaries() {
    assert_eq!(CONVERGENCE.to_bits(), 0x3586_37bd);
    assert_eq!(LARGE_ANGLE, 105_615.0);
    for w in [0.0, -0.0, f32::from_bits(1), CONVERGENCE.next_down()] {
        assert_eq!(branch(w), Branch::Converged);
    }
    for w in [
        CONVERGENCE,
        CONVERGENCE.next_up(),
        (2.0 * LARGE_ANGLE).next_down(),
    ] {
        assert_eq!(branch(w), Branch::Fast);
    }
    for w in [2.0 * LARGE_ANGLE, (2.0 * LARGE_ANGLE).next_up(), f32::MAX] {
        assert_eq!(branch(w), Branch::Slow);
    }
    assert_eq!(branch(f32::INFINITY), Branch::Infinite);
    for bits in [0x7fc0_0000, 0xffc0_1234, 0x7f80_0001] {
        assert_eq!(branch(f32::from_bits(bits)), Branch::Unordered);
    }
    // 负有限值不是长度的可达值；不把它当作物理样本。
}

#[test]
fn proves_active_finite_half_angles_stay_normal_and_scale_exactly() {
    // 活跃有限长度的指数域为107至254。
    // 半乘只减指数，不改尾数；结果指数至少106。
    let mut checked = 0;
    let mut check = |w: f32| {
        assert!(w.is_finite() && w >= CONVERGENCE);
        let angle = 0.5_f32 * w;
        assert!(angle.is_normal() && angle.is_sign_positive());
        assert_eq!(angle.to_bits(), w.to_bits() - (1 << 23));
        assert_eq!((2.0_f32 * angle).to_bits(), w.to_bits());
        checked += 1;
    };
    for exponent in 107..=254 {
        for mantissa in [0, 1, 0x3f_ffff, 0x7f_fffe, 0x7f_ffff] {
            let w = f32::from_bits((exponent << 23) | mantissa);
            if w >= CONVERGENCE {
                check(w);
            }
        }
    }
    check(CONVERGENCE);
    check(CONVERGENCE.next_up());
    assert_eq!(checked, 740);
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[test]
fn witnesses_large_angles_from_finite_nondegenerate_half_unit_corners() {
    // 四角沿产品槽序；只推导首轮角度，不执行极分解。
    let corners: [[f32; 3]; 4] = [
        [-0.5, 0.25, -0.5],
        [-0.5, -0.25, 0.5],
        [0.5, 0.25, -0.5],
        [0.5, -0.25, 0.5],
    ];
    let mut t1 = [0.0_f32; 3];
    let mut t2 = [0.0_f32; 3];
    for (slot, point) in corners.iter().enumerate() {
        let g0 = if slot / 2 == 0 { -0.5 } else { 0.5 };
        let g1 = if slot % 2 == 0 { -0.5 } else { 0.5 };
        for k in 0..3 {
            assert!(point[k].is_finite() && point[k].abs() <= 0.5);
            t1[k] += point[k] * g0;
            t2[k] += point[k] * g1;
        }
    }
    assert_eq!(t1, [1.0, 0.0, 0.0]);
    assert_eq!(t2, [0.0, -0.5, 1.0]);
    let normal = cross(t1, t2);
    assert_eq!(normal, [0.0, -1.0, -0.5]);
    assert_eq!(normal.iter().map(|v| v * v).sum::<f32>(), 1.25);
    let numerator = [
        cross([1.0, 0.0, 0.0], t1),
        cross([0.0, 1.0, 0.0], t2),
        cross([0.0, 0.0, 1.0], normal),
    ];
    let trace = (t1[0] + t2[1]) + normal[2];
    assert_eq!(trace, 0.0);
    let omega: [f32; 3] = std::array::from_fn(|k| {
        ((numerator[0][k] + numerator[1][k]) + numerator[2][k]) / (trace.abs() + 1.0e-10_f32)
    });
    let w = ((omega[0] * omega[0] + omega[1] * omega[1]) + omega[2] * omega[2]).sqrt();
    assert!(w.is_finite());
    assert_eq!(branch(w), Branch::Slow);
    println!(
        "finite geometry witness: w={w}, half_angle={}, determinant=1.25",
        0.5_f32 * w
    );
}

fn instruction(line: &str) -> Vec<&str> {
    line.split([' ', '\t', ',', ';'])
        .filter(|s| !s.is_empty())
        .collect()
}

fn check_control(ptx: &str, entry: &str) {
    let tail = ptx
        .split_once(&format!(".visible .entry {entry}("))
        .unwrap()
        .1;
    let body = tail.split(".visible .entry ").next().unwrap();
    let lines: Vec<_> = body
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("//"))
        .collect();
    let code: Vec<_> = lines.iter().map(|line| instruction(line)).collect();
    let convergence: Vec<_> = code
        .iter()
        .enumerate()
        .filter(|(_, op)| op.first() == Some(&"setp.lt.f32") && op.last() == Some(&"0f358637BD"))
        .collect();
    assert_eq!(convergence.len(), 1);
    let (start, conv) = convergence[0];
    assert_eq!(
        code[start + 1][..2],
        [format!("@{}", conv[1]), "bra".into()]
    );
    let half = code
        .iter()
        .skip(start + 2)
        .find(|op| {
            op.first() == Some(&"mul.f32")
                && op.get(2) == conv.get(2)
                && op.last() == Some(&"0f3F000000")
        })
        .unwrap();
    let abs = code
        .iter()
        .find(|op| op.first() == Some(&"abs.f32") && op.get(2) == half.get(1))
        .unwrap();
    let fast = code
        .iter()
        .find(|op| {
            op.first() == Some(&"setp.ltu.f32")
                && op.get(2) == abs.get(1)
                && op.last() == Some(&"0f47CE4780")
        })
        .unwrap();
    let guards: Vec<_> = code
        .iter()
        .enumerate()
        .filter(|(_, op)| {
            op.first() == Some(&"setp.eq.f32")
                && op.get(2) == abs.get(1)
                && op.last() == Some(&"0f7F800000")
        })
        .collect();
    let chains: Vec<_> = code
        .iter()
        .enumerate()
        .filter(|(_, op)| op.first() == Some(&"cvt.rn.f64.s64"))
        .collect();
    assert_eq!((guards.len(), chains.len()), (4, 4));
    let mut allowed_f64 = Vec::new();
    for ((guard_index, guard), (chain_index, convert)) in guards.iter().zip(&chains) {
        let fast_jump = code[..*guard_index]
            .iter()
            .rfind(|op| op.first() == Some(&format!("@{}", fast[1]).as_str()))
            .unwrap();
        assert_eq!(fast_jump[1], "bra");
        let fast_label = format!("{}:", fast_jump[2]);
        let fast_index = lines.iter().position(|line| *line == fast_label).unwrap();
        assert!(*guard_index < *chain_index && *chain_index < fast_index);
        let infinity_jump = &code[guard_index + 1];
        assert_eq!(infinity_jump[..2], [format!("@{}", guard[1]), "bra".into()]);
        let infinity_label = format!("{}:", infinity_jump[2]);
        let infinity_index = lines
            .iter()
            .position(|line| *line == infinity_label)
            .unwrap();
        let infinity_block = &code[infinity_index + 1..infinity_index + 5];
        assert_eq!(infinity_block[0][..1], ["mov.f32"]);
        assert_eq!(infinity_block[0][2], "0f00000000");
        assert_eq!(infinity_block[1][..1], ["mul.rn.f32"]);
        assert_eq!(infinity_block[1][2], half[1]);
        assert_eq!(infinity_block[1][3], infinity_block[0][1]);
        assert_eq!(infinity_block[3], ["bra.uni", fast_jump[2]]);
        let assembled = &code[chain_index - 1];
        assert_eq!(assembled[0], "bfi.b64");
        assert_eq!(assembled[1], convert[2]);
        let multiply = &code[chain_index + 1];
        assert!(matches!(multiply[0], "mul.f64" | "mul.rn.f64"));
        assert_eq!(multiply[2..], [convert[1], "0d3BF921FB54442D19"]);
        let narrow = &code[chain_index + 2];
        assert_eq!(narrow[0], "cvt.rn.f32.f64");
        assert_eq!(narrow[2], multiply[1]);
        allowed_f64.extend([*chain_index, chain_index + 1, chain_index + 2]);
    }
    for (index, op) in code.iter().enumerate() {
        if op[0].split('.').any(|part| part == "f64") {
            assert!(
                allowed_f64.contains(&index),
                "unmapped f64: {}",
                lines[index]
            );
        }
    }
}

#[test]
fn maps_current_and_frozen_ptx_control_and_library_mixed_precision() {
    let audit: Value = serde_json::from_str(AUDIT).unwrap();
    let sources: Vec<_> = audit["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(pinned)
        .collect();
    assert!(sources[0].contains("if(w<1.0e-6f) break;"));
    assert!(sources[0].contains("sinf(0.5f*w)"));
    assert!(sources[0].contains("cosf(0.5f*w)"));
    assert!(sources[1].contains("if w < 1.0e-6:"));
    assert!(sources[1].contains("half_w = 0.5 * w"));
    assert_eq!(sources[1].matches("wp.sin(half_w)").count(), 3);
    assert!(sources[1].contains("wp.cos(half_w)"));
    assert!(sources[2].contains("float sin(float x) { return ::sinf(x); }"));
    assert!(sources[2].contains("float cos(float x) { return ::cosf(x); }"));
    for row in audit["ptx"].as_array().unwrap() {
        check_control(&pinned(row), row["entry"].as_str().unwrap());
    }
}
