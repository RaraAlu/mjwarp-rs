//! 肌腱唤醒的混合标量与整数输出映射。
//! 不运行唤醒物理循环，不重放旧状态打包测试。
#![cfg(feature = "cuda-probe")]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

const MAP: &str = include_str!("../docs/evidence/t4-g03-retest/tendon-wake-internal/map.json");

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
    text.replace("\r\n", "\n")
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

fn tokens(line: &str) -> Vec<&str> {
    line.split([' ', '\t', ',', ';'])
        .filter(|v| !v.is_empty())
        .collect()
}

#[test]
fn maps_eighteen_mixed_scalar_groups_and_five_integer_outputs() {
    let map = map();
    let src = sources(&map);
    let mut ids = BTreeSet::new();
    for row in map["groups"].as_array().unwrap() {
        assert!(ids.insert(row["id"].as_str().unwrap()));
        assert!(
            src[row["source"].as_u64().unwrap() as usize]
                .contains(row["product"].as_str().unwrap())
        );
        if let Some(anchor) = row["upstream"].as_str() {
            assert!(src[4].contains(anchor));
        }
    }
    assert_eq!(ids.len(), 18);
    for row in map["outputs"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let declaration = if name.starts_with("tree_") {
            format!("pub {name}: &'a [i32]")
        } else {
            format!("pub {name}: i32")
        };
        assert!(src[0].contains(&declaration));
        assert!(src[5].contains(row["upstream"].as_str().unwrap()));
        assert_eq!(row["scalar"], "i32");
    }
    assert_eq!(map["outputs"].as_array().unwrap().len(), 5);
    assert!(src[1].contains("pub const TREE_FULLY_AWAKE: i32 = -11;"));
    assert!(src[4].contains("K_AWAKE_VAL = -(1 + types.MJ_MINAWAKE)"));
    assert!(src[5].contains("MJ_MINAWAKE = mujoco.mjMINAWAKE"));
    // 符号映射不证明绑定常量已在上游内核中实例化。
    for key in [
        "upstream_binding_constant_instantiation_verified",
        "upstream_generated_sleep_artifact_available",
        "parallel_semantics_verified",
    ] {
        assert_eq!(map["precision"][key], false);
    }
}

#[test]
fn inspects_existing_ptx_float_distances_and_signed_integer_updates() {
    let map = map();
    let capture = &map["capture"];
    for row in capture["entries"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let prefix = format!(
            "{}/{}-{}",
            capture["directory"].as_str().unwrap(),
            name,
            capture["ptx_sha256"].as_str().unwrap()
        );
        let cuda = read(
            &format!("{prefix}.cu"),
            capture["cuda_sha256"].as_str().unwrap(),
            false,
        );
        let source = &sources(&map)[0];
        let template = source
            .split_once("const TENDON_WAKE_CUDA: &str = r#\"")
            .unwrap()
            .1
            .split_once("\"#;")
            .unwrap()
            .0;
        assert!(cuda.contains(template));
        let ptx = read(
            &format!("{prefix}.ptx"),
            capture["ptx_sha256"].as_str().unwrap(),
            false,
        );
        let body = entry(&ptx, name);
        let code: Vec<_> = body
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .map(tokens)
            .collect();
        assert!(code.iter().any(|op| op[0] == "ld.global.u32"));
        assert!(code.iter().any(|op| op[0] == "st.global.u32"));
        for op in &code {
            assert!(
                !op[0]
                    .split('.')
                    .any(|t| matches!(t, "f64" | "f16" | "f16x2" | "bf16" | "bf16x2" | "tf32"))
            );
            if op[0].split('.').any(|t| t == "f32") {
                assert!(matches!(
                    op[0],
                    "ld.global.f32" | "sub.f32" | "setp.lt.f32" | "setp.geu.f32"
                ));
                assert_eq!(name, "tendon_wake");
            }
        }
        if name == "tendon_wake" {
            let subtract: Vec<_> = code.iter().filter(|op| op[0] == "sub.f32").collect();
            assert_eq!(subtract.len(), 2);
            assert_eq!(subtract[0][2], subtract[1][3]); // 同一实时长度。
            let low = code.iter().find(|op| op[0] == "setp.lt.f32").unwrap();
            let high_skip = code.iter().find(|op| op[0] == "setp.geu.f32").unwrap();
            assert_eq!(low[2], subtract[0][1]);
            assert_eq!(high_skip[2], subtract[1][1]);
            assert_eq!(low[3], high_skip[3]); // 同一边距；高侧跳过条件含无序。
            assert_eq!(code.iter().filter(|op| op[0] == "ld.global.f32").count(), 4);
            assert_eq!(code.iter().filter(|op| op[0] == "rem.u32").count(), 2);
            assert!(code.iter().any(|op| op[0] == "min.s32"));
            assert!(
                code.iter()
                    .any(|op| op[0] == "mov.u32" && op.last() == Some(&"-11"))
            );
            assert!(
                code.iter()
                    .any(|op| op[0] == "setp.lt.s32" && op.last() == Some(&"0"))
            );
            assert!(!code.iter().any(|op| op[0].starts_with("atom.")));
        }
        let receipt: Value = serde_json::from_str(&read(
            &format!("{prefix}.json"),
            row["receipt_sha256"].as_str().unwrap(),
            false,
        ))
        .unwrap();
        assert_eq!(receipt["entry"], name);
        assert_eq!(receipt["sourceSha256"], capture["cuda_sha256"]);
        assert_eq!(receipt["ptxSha256"], capture["ptx_sha256"]);
        assert_eq!(receipt["passed"], true);
        assert_eq!(receipt["useFastMath"], false);
    }
}

#[test]
fn maps_transaction_publication_and_diagnostic_only_readback_conversion() {
    let map = map();
    let src = sources(&map);
    let update = src[2]
        .split_once("pub fn update_tendon_wake(")
        .unwrap()
        .1
        .split("pub fn ")
        .next()
        .unwrap();
    let copy = update.find("device.copy_kernel.launch(").unwrap();
    let wake = update.find("device.wake_kernel.launch(").unwrap();
    let swap = update
        .find("std::mem::swap(&mut out.state, &mut out.scratch);")
        .unwrap();
    assert!(copy < wake && wake < swap);
    assert!(update.contains("data.ready.require(data.ready.tendon, \"tendon\")?;"));
    assert!(src[2].contains("state: TransferBuffer<i32>"));
    assert!(src[2].contains("scratch: TransferBuffer<i32>"));
    assert!(src[2].contains(".write_range(4 + range.start, &state.packed())?;"));
    let readback = src[2]
        .split_once("fn read_output<T:")
        .unwrap()
        .1
        .split("\n}")
        .next()
        .unwrap();
    assert!(readback.contains("Result<Vec<T>, TransferError>"));
    assert!(readback.contains("Ok(values)"));
    assert!(src[3].contains("if !(*value).into().is_finite()"));
    assert!(src[0].contains("if(!m[6]||nt==0) return;"));
    assert!(
        src[0].find("awake[i]=asleep[i]<0").unwrap()
            > src[0].find("for(int t=0;t<nt;++t)").unwrap()
    );
    for key in [
        "np01_complete",
        "np02_complete",
        "full_sleep_acceptance",
        "product_formula_changed",
    ] {
        assert_eq!(map["boundaries"][key], false);
    }
}

#[test]
fn checks_binary32_strict_limit_boundaries_without_running_wake_cycles() {
    fn active(length: f32) -> bool {
        length - 0.0_f32 < 0.125_f32 || 2.0_f32 - length < 0.125_f32
    }
    for (length, expected) in [
        (0.125_f32.next_down(), true),
        (0.125, false),
        (0.125_f32.next_up(), false),
        (1.875_f32.next_down(), false),
        (1.875, false),
        (1.875_f32.next_up(), true),
    ] {
        assert_eq!(active(length), expected);
    }
    // 高侧geu跳过与严格小于互补；NaN不是本轮物理样本。
    for distance in [0.125_f32.next_down(), 0.125, 0.125_f32.next_up(), f32::NAN] {
        let strict = distance < 0.125_f32;
        let geu = distance.is_nan() || distance >= 0.125_f32;
        assert_eq!(strict, !geu);
    }
}
