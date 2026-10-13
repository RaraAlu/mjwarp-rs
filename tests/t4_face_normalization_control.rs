//! 核对独立归一化控制域。
//! 不求极分解，不重放物理样本。
#![cfg(feature = "cuda-probe")]

use serde_json::Value;
use sha2::{Digest, Sha256};

const PRIOR: &str = include_str!("../docs/evidence/t4-g03-retest/face-trig/audit.json");

fn read(path: &str, hash: &str, lf: bool) -> String {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("pinned source exists");
    let text = String::from_utf8(bytes.clone()).expect("UTF-8 source");
    let normalized = text.replace("\r\n", "\n");
    let input = if lf { normalized.as_bytes() } else { &bytes };
    assert_eq!(format!("{:x}", Sha256::digest(input)), hash, "{path}");
    normalized.trim_end_matches('\0').to_owned()
}

fn pinned(row: &Value) -> String {
    read(
        row["path"].as_str().unwrap(),
        row["sha256"].as_str().unwrap(),
        row["normalization"] == "LF",
    )
}

fn instructions(text: &str, entry: &str) -> Vec<Vec<String>> {
    let body = text
        .split_once(&format!(".visible .entry {entry}("))
        .expect("exact forward entry")
        .1
        .split(".visible .entry ")
        .next()
        .unwrap();
    body.lines()
        .map(|line| {
            line.split([' ', '\t', ',', ';'])
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .filter(|op: &Vec<String>| !op.is_empty())
        .collect()
}

fn contains(code: &[Vec<String>], expected: &[&str]) -> bool {
    code.iter().any(|op| op == expected)
}

// 固定既有入口的寄存器与块。
// 此检查不比较平方和运算图。
fn check(code: &[Vec<String>], upstream: bool) -> Result<(), &'static str> {
    let (length, sum, pred, inverse, values, outputs) = if upstream {
        (
            "%f221",
            "%f603",
            "%p101",
            "%f604",
            ["%f217", "%f218", "%f219", "%f220"],
            ["%f702", "%f707", "%f708", "%f709"],
        )
    } else {
        (
            "%f99",
            "%f332",
            "%p49",
            "%f337",
            ["%f95", "%f96", "%f97", "%f98"],
            ["%f358", "%f357", "%f356", "%f359"],
        )
    };
    let opcode = if upstream {
        "setp.leu.f32"
    } else {
        "setp.gt.f32"
    };
    let start = code
        .iter()
        .position(|op| op == &["sqrt.rn.f32", length, sum])
        .ok_or("missing normalization length")?;
    let code = &code[start..];
    if code
        .get(1)
        .is_none_or(|op| op != &[opcode, pred, length, "0f00000000"])
    {
        return Err("wrong ordered/unordered predicate");
    }
    let branch = if upstream {
        "$L__BB42_105"
    } else {
        "$L__BB0_64"
    };
    if !contains(code, &[&format!("@{pred}"), "bra", branch]) {
        return Err("wrong normalization branch");
    }
    let inverse_at = code
        .iter()
        .position(|op| op == &["rcp.rn.f32", inverse, length])
        .ok_or("wrong reciprocal source or precision")?;
    let end = if upstream {
        "$L__BB42_105:"
    } else {
        "$L__BB0_63:"
    };
    let end_at = code
        .iter()
        .position(|op| op == &[end])
        .ok_or("missing block boundary")?;
    if inverse_at >= end_at {
        return Err("reciprocal outside positive branch");
    }
    let positive = &code[inverse_at + 1..end_at];
    for (output, input) in outputs.iter().zip(values) {
        if !contains(positive, &["mul.f32", output, inverse, input]) {
            return Err("wrong component or reciprocal reuse");
        }
    }
    if upstream {
        // 生成器每轮先清零，再设置单位回退。
        // 外层测试另核清零位置。
        for op in [
            ["mov.f32", "%f709", "0f3F800000"],
            ["mov.f32", "%f707", "%f702"],
            ["mov.f32", "%f708", "%f702"],
        ] {
            if !contains(&code[..inverse_at], &op) {
                return Err("wrong upstream identity fallback");
            }
        }
    } else {
        let fallback = &code[end_at..];
        for op in [
            ["mov.f32", "%f356", "0f00000000"],
            ["mov.f32", "%f359", "0f3F800000"],
            ["mov.f32", "%f357", "%f356"],
            ["mov.f32", "%f358", "%f356"],
        ] {
            if !contains(fallback, &op) {
                return Err("wrong product identity fallback");
            }
        }
    }
    Ok(())
}

#[test]
fn proves_inverse_predicates_include_unordered_identity_fallback() {
    let mut values = vec![0.0, -0.0, f32::INFINITY, f32::NEG_INFINITY];
    for exponent in 0..=254 {
        for mantissa in [0, 1, 0x7f_ffff] {
            let bits = (exponent << 23) | mantissa;
            values.extend([f32::from_bits(bits), f32::from_bits(bits | 0x8000_0000)]);
        }
    }
    for bits in [0x7fc0_0000, 0xffc0_1234, 0x7f80_0001, 0xff80_0001] {
        values.push(f32::from_bits(bits));
    }
    for length in values {
        let product_positive = length > 0.0;
        // PTX leu包含无序，不等于普通le。
        let upstream_identity = length <= 0.0 || length.is_nan();
        assert_eq!(product_positive, !upstream_identity);
    }
    // 负长度不可达；代表位型不证明物理可达性。
}

#[test]
fn distinguishes_reciprocal_multiply_and_ieee_boundary_classes() {
    let component = 5.0_f32;
    let length = 3.0_f32;
    assert_ne!(
        (component * (1.0 / length)).to_bits(),
        (component / length).to_bits()
    );
    let tiny = f32::from_bits(1);
    assert_eq!((tiny * tiny).sqrt(), 0.0);
    assert_eq!((f32::MAX * f32::MAX).sqrt(), f32::INFINITY);
    assert_eq!((1.0_f32 / f32::INFINITY).to_bits(), 0);
    assert!((f32::INFINITY * (1.0_f32 / f32::INFINITY)).is_nan());
    assert!((1.0_f32 / tiny).is_infinite());
    // 这里只核IEEE阶段，不把合成量当上游参考。
}

#[test]
fn binds_sources_and_existing_ptx_normalization_control() {
    let prior: Value = serde_json::from_str(PRIOR).unwrap();
    let product = pinned(&prior["sources"][0]);
    let support = pinned(&prior["sources"][1]);
    let builtin = pinned(&prior["sources"][2]);
    let quat = read(
        "target/mjwarp-parallel/venv/Lib/site-packages/warp/native/quat.h",
        "1ef29e23117b3da391c12d5e4959f52f8cc83ccb2c8b5ced474a63499a20ccd2",
        false,
    );
    assert!(
        product
            .contains("if(n>0.0f) {float inv_n=1.0f/n;for(int k=0;k<4;++k) q[k]=next[k]*inv_n;}")
    );
    assert!(product.contains("else {q[0]=q[1]=q[2]=0.0f;q[3]=1.0f;}"));
    assert!(support.contains("cell_quat = wp.normalize(qrot * cell_quat)"));
    assert!(builtin.contains("#define kEps 0.0f"));
    assert!(quat.contains("if (l > Type(kEps))"));
    assert!(quat.contains("Type inv_l = Type(1) / l;"));
    assert!(quat.contains("q.x * inv_l, q.y * inv_l, q.z * inv_l, q.w * inv_l"));
    for (i, row) in prior["ptx"].as_array().unwrap().iter().enumerate() {
        let code = instructions(&pinned(row), row["entry"].as_str().unwrap());
        if i == 1 {
            let zero = code
                .iter()
                .position(|op| op == &["mov.f32", "%f702", "0f00000000"])
                .unwrap();
            let product = code
                .iter()
                .position(|op| op == &["mul.f32", "%f582", "%f706", "%f701"])
                .unwrap();
            // 零初值必须先于本轮乘积与归一化。
            let norm = code
                .iter()
                .position(|op| op == &["sqrt.rn.f32", "%f221", "%f603"])
                .unwrap();
            assert!(zero < norm);
            assert!(zero < product && product < norm);
            assert_eq!(
                code.iter()
                    .filter(
                        |op| op.get(1).map(String::as_str) == Some("%f702") && op[0] == "mov.f32"
                    )
                    .count(),
                1
            );
        }
        check(&code, i == 1).unwrap();
    }
}

#[test]
fn rejects_wrong_predicate_reciprocal_component_and_fallback() {
    let prior: Value = serde_json::from_str(PRIOR).unwrap();
    for (i, row) in prior["ptx"].as_array().unwrap().iter().enumerate() {
        let text = pinned(row);
        let changes = if i == 0 {
            [
                ("setp.gt.f32 \t%p49", "setp.ge.f32 \t%p49"),
                ("%f337, %f99;", "%f337, %f47;"),
                ("%f358, %f337, %f95;", "%f358, %f337, %f96;"),
                ("%f359, 0f3F800000;", "%f359, 0f00000000;"),
            ]
        } else {
            [
                ("setp.leu.f32 \t%p101", "setp.le.f32 \t%p101"),
                ("%f604, %f221;", "%f604, %f167;"),
                ("%f702, %f604, %f217;", "%f702, %f604, %f218;"),
                ("%f709, 0f3F800000;", "%f709, 0f00000000;"),
            ]
        };
        for (from, to) in changes {
            assert!(text.contains(from));
            let corrupt = text.replace(from, to);
            let code = instructions(&corrupt, row["entry"].as_str().unwrap());
            assert!(check(&code, i == 1).is_err(), "accepted corruption: {from}");
        }
    }
}
