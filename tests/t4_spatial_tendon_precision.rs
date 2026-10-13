//! 空间肌腱site链与混合输出的独立精度映射。
//! 不重放肌腱物理样本或旧打包目标。
#![cfg(feature = "cuda-probe")]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};
const MAP: &str = include_str!("../docs/evidence/t4-g03-retest/spatial-tendon-internal/map.json");

fn read(path: &str, hash: &str, lf: bool) -> String {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let bytes = if lf {
        text.replace("\r\n", "\n").into_bytes()
    } else {
        bytes
    };
    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        hash,
        "drift: {path}"
    );
    text.trim_end_matches('\0').replace("\r\n", "\n")
}
fn map() -> Value {
    serde_json::from_str(MAP).unwrap()
}
fn sources(map: &Value) -> Vec<String> {
    map["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            read(
                r["path"].as_str().unwrap(),
                r["sha256"].as_str().unwrap(),
                true,
            )
        })
        .collect()
}
fn entry<'a>(ptx: &'a str, name: &str) -> &'a str {
    ptx.split_once(&format!(".visible .entry {name}("))
        .unwrap()
        .1
        .split(".visible .entry ")
        .next()
        .unwrap()
}
fn instructions(body: &str) -> Vec<Vec<&str>> {
    body.lines()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("//"))
        .map(|s| {
            s.split([' ', '\t', ',', ';'])
                .filter(|s| !s.is_empty())
                .collect()
        })
        .collect()
}
fn reductions(body: &str) -> usize {
    let code = instructions(body);
    let mut allowed = BTreeSet::new();
    let mut count = 0;
    for (i, op) in code
        .iter()
        .enumerate()
        .filter(|(_, op)| op[0] == "cvt.rn.f64.s64")
    {
        let multiply = &code[i + 1];
        let narrow = &code[i + 2];
        assert!(matches!(multiply[0], "mul.f64" | "mul.rn.f64"));
        assert_eq!(multiply[2..], [op[1], "0d3BF921FB54442D19"]);
        assert_eq!(narrow[0], "cvt.rn.f32.f64");
        assert_eq!(narrow[2], multiply[1]);
        allowed.extend([i, i + 1, i + 2]);
        count += 1;
    }
    for (i, op) in code.iter().enumerate() {
        if op[0].split('.').any(|t| t == "f64") {
            assert!(allowed.contains(&i));
        }
        assert!(
            !op[0]
                .split('.')
                .any(|t| matches!(t, "f16" | "f16x2" | "bf16" | "bf16x2" | "tf32"))
        );
    }
    count
}

#[test]
fn maps_sixteen_site_chain_groups_and_six_mixed_outputs() {
    let map = map();
    let src = sources(&map);
    let mut ids = BTreeSet::new();
    for row in map["groups"].as_array().unwrap() {
        assert!(ids.insert(row["id"].as_str().unwrap()));
        assert!(
            src[row["p_source"].as_u64().unwrap() as usize]
                .contains(row["product"].as_str().unwrap())
        );
        assert!(
            src[row["u_source"].as_u64().unwrap() as usize]
                .contains(row["upstream"].as_str().unwrap())
        );
    }
    assert_eq!(ids.len(), 16);
    assert_eq!(map["outputs"].as_array().unwrap().len(), 6);
    for row in map["outputs"].as_array().unwrap() {
        assert!(src[0].contains(&format!(
            "pub {}: &'a [{}]",
            row["name"].as_str().unwrap(),
            row["scalar"].as_str().unwrap()
        )));
        assert!(src[0].contains(row["product"].as_str().unwrap()));
        assert!(src[5].contains(row["upstream"].as_str().unwrap()));
    }
    assert!(src[0].contains("scratch+13*i+7"));
    assert!(src[0].contains("scratch+13*i+10"));
    assert!(src[1].contains("struct SV { float x,y,z; }"));
}

#[test]
fn binds_generated_artifacts_and_maps_physical_library_f64() {
    let map = map();
    let src = sources(&map);
    let product = &map["product_capture"];
    for row in product["entries"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let prefix = format!(
            "{}/{}-{}",
            product["directory"].as_str().unwrap(),
            name,
            product["ptx_sha256"].as_str().unwrap()
        );
        let cuda = read(
            &format!("{prefix}.cu"),
            product["cuda_sha256"].as_str().unwrap(),
            false,
        );
        for (index, symbol) in [(0, "SPATIAL_TENDON_CUDA"), (1, "TENDON_WRAP_CUDA")] {
            let template = src[index]
                .split_once(&format!("const {symbol}: &str = r#\""))
                .unwrap()
                .1
                .split_once("\"#;")
                .unwrap()
                .0;
            assert!(cuda.contains(template));
        }
        let ptx = read(
            &format!("{prefix}.ptx"),
            product["ptx_sha256"].as_str().unwrap(),
            false,
        );
        let body = entry(&ptx, name);
        assert_eq!(
            reductions(body),
            row["library_reductions"].as_u64().unwrap() as usize
        );
        assert!(!body.contains("atom.global.add.f32"));
        if name == "spatial_moment" {
            assert!(body.contains("0f26901D7D"));
        }
        let receipt: Value = serde_json::from_str(&read(
            &format!("{prefix}.json"),
            row["receipt_sha256"].as_str().unwrap(),
            false,
        ))
        .unwrap();
        assert_eq!(receipt["entry"], name);
        assert_eq!(receipt["passed"], true);
        assert_eq!(receipt["sourceSha256"], product["cuda_sha256"]);
        assert_eq!(receipt["ptxSha256"], product["ptx_sha256"]);
    }
    let upstream = &map["upstream_capture"];
    let prefix = upstream["prefix"].as_str().unwrap();
    let cuda = read(
        &format!("{prefix}.cu"),
        upstream["cuda_sha256"].as_str().unwrap(),
        false,
    );
    let ptx = read(
        &format!("{prefix}.sm89.ptx"),
        upstream["ptx_sha256"].as_str().unwrap(),
        false,
    );
    for row in upstream["entries"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        assert!(cuda.contains(name));
        let body = entry(&ptx, name);
        assert_eq!(
            reductions(body),
            row["library_reductions"].as_u64().unwrap() as usize
        );
        if name.starts_with("_spatial_site_tendon_") {
            assert!(body.contains("atom.global.add.f32"));
            assert!(body.contains("0f26901D7D"));
        }
    }
    assert_eq!(
        map["precision"]["library_f64_affects_physical_outputs"],
        true
    );
    assert_eq!(map["precision"]["library_f64_is_diagnostic"], false);
}

#[test]
fn distinguishes_zero_and_tiny_normalization_branches() {
    let map = map();
    let src = sources(&map);
    let normalize = src[4]
        .split_once("def normalize_with_norm(")
        .unwrap()
        .1
        .split("@wp.func")
        .next()
        .unwrap();
    assert!(normalize.contains("if norm == 0.0:"));
    assert!(normalize.contains("return x / norm, norm"));
    assert!(src[1].contains("return n!=0 ? SV{a.x/n,a.y/n,a.z/n} : a;"));
    assert!(src[1].contains("return n<1e-15f ? SV{1,0,0}"));
    let norm = (1.0e-16_f32 * 1.0e-16_f32).sqrt();
    assert!(norm > 0.0 && norm < 1.0e-15_f32);
    assert_eq!(1.0e-15_f32.to_bits(), 0x2690_1d7d);
    // 正微小长度：前两者求方向，第三者回退X轴。
    let regular = [0.0_f32, 1.0e-16_f32 / norm, 0.0];
    let fallback = [1.0_f32, 0.0, 0.0];
    assert_ne!(regular, fallback);
    // 零向量：前两者保留输入；方向回退不保留负零。
    let zero = [-0.0_f32, 0.0, -0.0];
    assert_ne!(zero.map(f32::to_bits), fallback.map(f32::to_bits));
}

#[test]
fn checks_stage_order_without_admitting_atomic_or_device_equivalence() {
    let map = map();
    let src = sources(&map);
    let update = src[2]
        .split_once("pub fn update_spatial_tendons(")
        .unwrap()
        .1
        .split("pub fn ")
        .next()
        .unwrap();
    let site = update.find("device.site_kernel.launch(").unwrap();
    let moment = update.find("device.moment_kernel.launch(").unwrap();
    let wrap = update.find("device.wrap_kernel.launch(").unwrap();
    let ready = update.find("data.ready.spatial_tendon = true;").unwrap();
    assert!(site < moment && moment < wrap && wrap < ready);
    assert!(update.contains("data.ready.attached"));
    assert!(update.contains("data.ready.com"));
    for key in [
        "np01_complete",
        "np02_complete",
        "full_equivalence",
        "product_formula_changed",
        "accumulation_order_equivalent",
        "full_sphere_cylinder_inside_formula_audited",
    ] {
        assert_eq!(map["boundaries"][key], false);
    }
}
