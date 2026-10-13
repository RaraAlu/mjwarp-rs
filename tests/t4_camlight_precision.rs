//! 核对相机光源内部量与输出类型。
//! 只读既存产物；不重放物理数值或编译CUDA。
#![cfg(feature = "cuda-probe")]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

const MAP: &str = include_str!("../docs/evidence/t4-g03-retest/camlight-internal/map.json");

fn digest(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

fn read(path: &str, expected: &str, normalized: bool) -> String {
    let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let actual = if normalized {
        digest(text.replace("\r\n", "\n"))
    } else {
        digest(bytes)
    };
    assert_eq!(actual, expected, "source drift: {path}");
    text.trim_end_matches('\0').replace("\r\n", "\n")
}

fn map() -> Value {
    serde_json::from_str(MAP).unwrap()
}

fn source(map: &Value, index: usize) -> String {
    let row = &map["sources"][index];
    read(
        row["path"].as_str().unwrap(),
        row["sha256"].as_str().unwrap(),
        true,
    )
}

fn entry<'a>(text: &'a str, name: &str) -> &'a str {
    let marker = format!(".visible .entry {name}(");
    let tail = text.split_once(&marker).unwrap().1;
    tail.split(".visible .entry ").next().unwrap()
}

fn tokens(line: &str) -> Vec<&str> {
    line.split([' ', '\t', ',', ';'])
        .filter(|part| !part.is_empty())
        .collect()
}

fn zero_before(code: &[Vec<&str>], register: &str, end: usize) -> bool {
    // 编译器复用前一归一化的零寄存器。
    // 这里只追踪已冻结块的mov链，不分析任意控制流。
    let definition = code[..end]
        .iter()
        .enumerate()
        .rfind(|(_, op)| op.get(1) == Some(&register) && !op[0].starts_with(['.', '@']));
    match definition {
        Some((index, op)) if op[0] == "mov.f32" => {
            op[2] == "0f00000000" || (op[2].starts_with('%') && zero_before(code, op[2], index))
        }
        _ => false,
    }
}

fn normalization_sites(body: &str) -> usize {
    let lines: Vec<_> = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect();
    let code: Vec<_> = lines.iter().map(|line| tokens(line)).collect();
    let mut sites = 0;
    for (index, sqrt) in code
        .iter()
        .enumerate()
        .filter(|(_, op)| op[0] == "sqrt.rn.f32")
    {
        let predicate = &code[index + 1];
        assert_eq!(predicate[0], "setp.leu.f32");
        assert_eq!(predicate[2..], [sqrt[1], "0f00000000"]);
        let divisions: Vec<_> = code
            .iter()
            .enumerate()
            .filter(|(_, op)| op[0] == "div.rn.f32" && op.get(3) == sqrt.get(1))
            .collect();
        assert_eq!(divisions.len(), 3);
        let first_division = divisions[0].0;
        let last_division = divisions[2].0;
        let prefix = &code[index + 2..first_division];
        let branch = prefix
            .iter()
            .find(|op| op[0] == format!("@{}", predicate[1]))
            .unwrap();
        assert_eq!(branch[1], "bra");
        let label = format!("{}:", branch[2]);
        let label_index = lines.iter().position(|line| *line == label).unwrap();
        assert!(label_index > last_division);
        // 拒绝把向量除法替换为四元数的倒数乘法。
        assert!(
            !code
                .iter()
                .any(|op| op[0] == "rcp.rn.f32" && op.get(2) == sqrt.get(1))
        );
        for (_, division) in divisions {
            assert!(zero_before(&code, division[1], first_division));
        }
        sites += 1;
    }
    for op in code {
        assert!(
            !op[0]
                .split('.')
                .any(|t| matches!(t, "f64" | "f16" | "f16x2" | "bf16" | "bf16x2" | "tf32"))
        );
    }
    sites
}

#[test]
fn maps_eighteen_internal_groups_and_four_public_f32_outputs() {
    let map = map();
    for index in 0..8 {
        source(&map, index);
    }
    let product = source(&map, 0);
    let smooth = source(&map, 4);
    let types = source(&map, 5);
    let mut ids = BTreeSet::new();
    for row in map["groups"].as_array().unwrap() {
        assert!(ids.insert(row["id"].as_str().unwrap()));
        assert_eq!(row["physical_scalar"], "f32");
        assert!(product.contains(row["product"].as_str().unwrap()));
        assert!(smooth.contains(row["upstream"].as_str().unwrap()));
    }
    assert_eq!(ids.len(), 18);
    let outputs = map["outputs"].as_array().unwrap();
    assert_eq!(outputs.len(), 4);
    for row in outputs {
        let name = row["name"].as_str().unwrap();
        assert_eq!(row["scalar"], "f32");
        assert!(product.contains(&format!("pub {name}: &'a [f32]")));
        assert!(product.contains(row["product"].as_str().unwrap()));
        assert!(types.contains(row["upstream"].as_str().unwrap()));
    }
    assert!(product.contains("mat[0]=x.x;mat[1]=y.x;mat[2]=z.x;"));
    assert!(product.contains("mat[6]=x.z;mat[7]=y.z;mat[8]=z.z;"));
    assert!(product.contains("bool invalid=(mode==3 || mode==4) && t<0;"));
    let light = smooth
        .split_once("def _light_local_to_global(")
        .unwrap()
        .1
        .split("@event_scope")
        .next()
        .unwrap();
    let invalid = light
        .split_once("if invalid_target:")
        .unwrap()
        .1
        .split("elif light_mode")
        .next()
        .unwrap();
    assert!(invalid.contains("return"));
    assert!(!invalid.contains("wp.normalize"));
    let vec = source(&map, 6);
    let specialized = vec
        .split_once("normalize(vec_t<3, Type> a)")
        .unwrap()
        .1
        .split("template <")
        .next()
        .unwrap();
    assert!(specialized.contains("a.c[0] / l, a.c[1] / l, a.c[2] / l"));
    assert!(specialized.contains("l > Type(kEps)"));
    assert!(source(&map, 7).contains("#define kEps 0.0f"));
}

#[test]
fn binds_existing_product_and_generated_upstream_normalization_artifacts() {
    let map = map();
    let capture = &map["product_capture"];
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
        let product = source(&map, 0);
        let template = product
            .split_once("const CAMLIGHT_CUDA: &str = r#\"")
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
        assert_eq!(
            normalization_sites(entry(&ptx, name)),
            row["normalization_sites"].as_u64().unwrap() as usize
        );
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
        assert_eq!(receipt["fmad"], true);
        assert_eq!(receipt["useFastMath"], false);
    }
    let capture = &map["upstream_capture"];
    let prefix = capture["prefix"].as_str().unwrap();
    let cuda = read(
        &format!("{prefix}.cu"),
        capture["cuda_sha256"].as_str().unwrap(),
        false,
    );
    let ptx = read(
        &format!("{prefix}.sm89.ptx"),
        capture["ptx_sha256"].as_str().unwrap(),
        false,
    );
    for row in capture["entries"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let generated = cuda
            .split_once(&format!("void {name}("))
            .unwrap()
            .1
            .split("extern \"C\" __global__")
            .next()
            .unwrap();
        assert!(generated.contains("wp::vec_t<3, wp::float32>"));
        assert!(!generated.contains("wp::float64"));
        assert_eq!(
            generated.matches("wp::normalize(").count(),
            row["normalization_sites"].as_u64().unwrap() as usize
        );
        assert_eq!(
            normalization_sites(entry(&ptx, name)),
            row["normalization_sites"].as_u64().unwrap() as usize
        );
    }
}

#[test]
fn checks_two_stage_dependencies_and_keeps_numerical_admission_open() {
    let map = map();
    let resident = source(&map, 1);
    let update = resident
        .split_once("pub fn update_camlight(")
        .unwrap()
        .1
        .split("pub fn ")
        .next()
        .unwrap();
    assert!(update.contains("self.require_input(data, data.ready.rigid, \"rigid\")?;"));
    assert!(update.contains("self.require_input(data, data.ready.com, \"com\")?;"));
    let rigid = update.find("device.rigid_kernel.launch(").unwrap();
    let com = update.find("device.com_kernel.launch(").unwrap();
    let ready = update.find("data.ready.camlight = true;").unwrap();
    assert!(rigid < com && com < ready);
    assert!(!update.contains("readback"));
    let product = source(&map, 0);
    assert!(product.contains("else if(cm[c]==4 && ct[c]>=0)"));
    assert!(product.contains("else if(lm[l]==4 && lt[l]>=0)"));
    for key in [
        "new_cuda_compilation",
        "new_gpu_execution",
        "physical_formula_changed",
        "np01_complete",
        "np02_complete",
        "full_equivalence",
    ] {
        assert_eq!(map["boundaries"][key], false);
    }
}
