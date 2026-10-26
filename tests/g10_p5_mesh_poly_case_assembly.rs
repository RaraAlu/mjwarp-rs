//! G10@P5 M0b：mesh multicontact 32 项 case bins 的 CPU 装配与同一来源校验（不建 CUDA context）。
//!
//! 覆盖：
//! 1. bins ↔ `mesh-cases-manifest.json` 的 sha256/count 逐项一致；
//! 2. 32 项装配后 9 个 mesh_poly slice 的长度/指针与 bins 一致，且满足修正后的 M0a
//!    `validate_mesh_poly` 规则（本测试为规则镜像；真实校验在 M2 窗口 `run()` 内执行）；
//! 3. 旧 G5 23 项 case 目录缺 9 个 bin → 空 slice + null sidecar 语义，不报错；
//! 4. 同一来源：`xml_sha256` / `qpos_bits` / `geom_xpos` / `geom_xmat` 与 M1 GPU 参考逐位一致。
//!
//! 运行：`cargo test --locked --test g10_p5_mesh_poly_case_assembly`

#[path = "support/mesh_poly_case_support.rs"]
mod support;

use sha2::{Digest, Sha256};

use support::{
    INPUT_BINS, MESH_POLY_BINS, MeshCaseInputs, g5_cases_root, load_case, m0b_cases_root,
    m0b_manifest, m1_reference, qa_reference,
};

const M0B_CASES: [&str; 2] = ["mesh-box", "mesh-cylinder"];
const OLD_G5_CASES: [&str; 10] = [
    "sphere-box-shallow",
    "sphere-box-deep",
    "capsule-box",
    "ellipsoid-box",
    "hfield-sphere",
    "hfield-capsule",
    "sdf-box",
    "cylinder-box",
    "box-box-margin",
    "mesh-mesh",
];

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn f32_bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn json_f32_bits(value: &serde_json::Value) -> Vec<u32> {
    match value {
        serde_json::Value::Array(items) => items.iter().flat_map(json_f32_bits).collect(),
        serde_json::Value::Number(number) => {
            vec![(number.as_f64().expect("数值") as f32).to_bits()]
        }
        other => panic!("参考字段不是数字/数组：{other}"),
    }
}

/// 修正后 M0a `validate_mesh_poly` 的规则镜像（与 `abi.rs` 逐条同构；此处为测试侧复算）。
fn assert_m0a_mesh_poly_rules(case: &str, inputs: &MeshCaseInputs) {
    let lengths = inputs.mesh_poly_lengths();
    assert!(
        lengths.iter().all(|length| *length > 0),
        "{case}: 9 个 mesh_poly 数组必须齐全（长度 {lengths:?}）"
    );
    let nmesh = inputs.mesh_polynum.len();
    assert_eq!(
        inputs.mesh_polyadr.len(),
        nmesh,
        "{case}: polynum/polyadr 长度须一致"
    );
    assert!(
        inputs.mesh_polynormal.len().is_multiple_of(3),
        "{case}: polynormal 长度须为 3 的倍数"
    );
    let nmeshpoly = inputs.mesh_polynormal.len() / 3;
    assert_eq!(
        inputs.mesh_polyvertadr.len(),
        nmeshpoly,
        "{case}: polyvertadr 长度须为 nmeshpoly"
    );
    assert_eq!(
        inputs.mesh_polyvertnum.len(),
        nmeshpoly,
        "{case}: polyvertnum 长度须为 nmeshpoly"
    );
    assert_eq!(
        inputs.mesh_polymapadr.len(),
        inputs.mesh_polymapnum.len(),
        "{case}: polymapadr/polymapnum 长度须一致"
    );
    let nmeshvert = inputs.mesh_polymapadr.len();

    let mut total_polygons = 0usize;
    for index in 0..nmesh {
        let (adr, num) = (inputs.mesh_polyadr[index], inputs.mesh_polynum[index]);
        assert!(adr >= 0 && num >= 0, "{case}: polyadr/polynum 不得为负");
        assert!(
            (adr as usize) + (num as usize) <= nmeshpoly,
            "{case}: mesh poly 区间超出 nmeshpoly"
        );
        total_polygons += num as usize;
    }
    assert_eq!(
        total_polygons, nmeshpoly,
        "{case}: sum(polynum) != nmeshpoly"
    );

    let mut total_vertices = 0usize;
    for index in 0..nmeshpoly {
        let (adr, num) = (
            inputs.mesh_polyvertadr[index],
            inputs.mesh_polyvertnum[index],
        );
        assert!(
            adr >= 0 && num >= 0,
            "{case}: polyvertadr/polyvertnum 不得为负"
        );
        assert!(
            (adr as usize) + (num as usize) <= inputs.mesh_polyvert.len(),
            "{case}: mesh_polyvert 区间越界"
        );
        total_vertices += num as usize;
    }
    assert_eq!(
        total_vertices,
        inputs.mesh_polyvert.len(),
        "{case}: sum(polyvertnum) != len(mesh_polyvert)"
    );

    let mut total_maps = 0usize;
    for index in 0..nmeshvert {
        let (adr, num) = (inputs.mesh_polymapadr[index], inputs.mesh_polymapnum[index]);
        assert!(
            adr >= 0 && num >= 0,
            "{case}: polymapadr/polymapnum 不得为负"
        );
        assert!(
            (adr as usize) + (num as usize) <= inputs.mesh_polymap.len(),
            "{case}: mesh_polymap 区间越界"
        );
        total_maps += num as usize;
    }
    assert_eq!(
        total_maps,
        inputs.mesh_polymap.len(),
        "{case}: sum(polymapnum) != len(mesh_polymap)"
    );
}

#[test]
fn m0b_bins_match_manifest_sha_and_counts() {
    let manifest = m0b_manifest();
    let arrays_order: Vec<&str> = manifest["arrays_order"]
        .as_array()
        .expect("arrays_order")
        .iter()
        .map(|value| value.as_str().expect("数组名"))
        .collect();
    assert_eq!(
        arrays_order,
        INPUT_BINS.to_vec(),
        "manifest 数组顺序须与 32 项口径一致"
    );

    let directory = m0b_cases_root();
    for case in M0B_CASES {
        let record = &manifest["cases"][case];
        let arrays = record["arrays"].as_object().expect("case arrays");
        assert_eq!(arrays.len(), 32, "{case}: 应声明 32 个数组");
        for name in INPUT_BINS {
            let bytes = std::fs::read(directory.join(case).join(format!("{name}.bin")))
                .unwrap_or_else(|error| panic!("{case}/{name}.bin 读取失败：{error}"));
            let expected = &arrays[name];
            assert_eq!(
                sha256_hex(&bytes),
                expected["sha256"].as_str().expect("sha256"),
                "{case}: {name}.bin sha256 与 manifest 不一致"
            );
            assert_eq!(
                bytes.len() as u64 / 4,
                expected["count"].as_u64().expect("count"),
                "{case}: {name}.bin 元素数与 manifest 不一致"
            );
        }
        // 关键 9 项的真实长度（cube：nmesh=1 / nmeshpoly=6 / nmeshvert=8 / 24 / 24）。
        let lengths = [
            arrays["mesh_polynum"]["count"].as_u64().unwrap(),
            arrays["mesh_polyadr"]["count"].as_u64().unwrap(),
            arrays["mesh_polynormal"]["count"].as_u64().unwrap(),
            arrays["mesh_polyvertadr"]["count"].as_u64().unwrap(),
            arrays["mesh_polyvertnum"]["count"].as_u64().unwrap(),
            arrays["mesh_polyvert"]["count"].as_u64().unwrap(),
            arrays["mesh_polymapadr"]["count"].as_u64().unwrap(),
            arrays["mesh_polymapnum"]["count"].as_u64().unwrap(),
            arrays["mesh_polymap"]["count"].as_u64().unwrap(),
        ];
        assert_eq!(
            lengths,
            [1, 1, 18, 6, 6, 24, 8, 8, 24],
            "{case}: mesh_poly 数组长度应为真实 cube 形状"
        );
    }
}

#[test]
fn m0b_assembly_matches_m0a_sidecar_contract() {
    for case in M0B_CASES {
        let inputs = load_case(&m0b_cases_root().join(case));
        assert!(
            inputs.mesh_poly_present(),
            "{case}: 32 项 case 应构造 sidecar"
        );
        assert_eq!(
            inputs.mesh_poly_lengths(),
            [1, 1, 18, 6, 6, 24, 8, 8, 24],
            "{case}: 装配后 9 个 slice 长度须与 bins 一致"
        );
        assert_eq!(inputs.geom_xpos.len(), 2 * 3, "{case}: geom_xpos 长度");
        assert_eq!(inputs.geom_xmat.len(), 2 * 9, "{case}: geom_xmat 长度");
        assert_eq!(inputs.meta.nworld, 1, "{case}: meta nworld");
        assert_eq!(inputs.meta.ngeom, 2, "{case}: meta ngeom");

        // sidecar 的 9 个指针直接来自 bins 缓冲区（`CollisionGpuInputs` 借用同一 Vec）。
        let view = inputs.inputs();
        assert!(std::ptr::eq(
            view.mesh_polynum.as_ptr(),
            inputs.mesh_polynum.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polyadr.as_ptr(),
            inputs.mesh_polyadr.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polynormal.as_ptr(),
            inputs.mesh_polynormal.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polyvertadr.as_ptr(),
            inputs.mesh_polyvertadr.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polyvertnum.as_ptr(),
            inputs.mesh_polyvertnum.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polyvert.as_ptr(),
            inputs.mesh_polyvert.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polymapadr.as_ptr(),
            inputs.mesh_polymapadr.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polymapnum.as_ptr(),
            inputs.mesh_polymapnum.as_ptr()
        ));
        assert!(std::ptr::eq(
            view.mesh_polymap.as_ptr(),
            inputs.mesh_polymap.as_ptr()
        ));

        assert_m0a_mesh_poly_rules(case, &inputs);
    }
}

#[test]
fn old_g5_cases_keep_23_bin_compat() {
    let reference = qa_reference();
    let declared_count = reference["cases"]["sphere-box-shallow"]["input_bins"]
        .as_object()
        .expect("input_bins")
        .len();
    assert_eq!(declared_count, 23, "旧 QA 参考应只声明 23 项输入");

    for case in OLD_G5_CASES {
        let directory = g5_cases_root().join(case);
        let inputs = load_case(&directory);
        assert!(
            !inputs.mesh_poly_present(),
            "{case}: 旧 23 项 case 不应构造 sidecar"
        );
        assert_eq!(
            inputs.mesh_poly_lengths(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            "{case}: 旧 case 的 9 个 slice 应为空"
        );
        // 旧 case 的 bins 只覆盖前 23 项；9 个 mesh_poly 文件不存在（不是空文件）。
        for name in MESH_POLY_BINS {
            assert!(
                !directory.join(format!("{name}.bin")).exists(),
                "{case}: 旧 case 目录不应出现 {name}.bin"
            );
        }
        // sha 守卫按声明集合执行：旧参考声明的 23 项都必须能在 32 项口径中找到。
        let declared = reference["cases"][case]["input_bins"]
            .as_object()
            .unwrap_or_else(|| panic!("{case}: QA input_bins"));
        for name in declared.keys() {
            assert!(
                INPUT_BINS.contains(&name.as_str()),
                "{case}: 声明的输入 {name} 不在 32 项 INPUT_BINS 内"
            );
        }
    }
}

#[test]
fn m0b_same_source_matches_m1_reference() {
    let manifest = m0b_manifest();
    let reference = m1_reference();
    let fixture_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/g10-p5-mesh-multicontact");

    for case in M0B_CASES {
        let record = &manifest["cases"][case];
        let case_reference = &reference["cases"][case];

        // `xml_sha256`：manifest 记录 == 现场重算 == M1 参考。
        let xml_bytes =
            std::fs::read(fixture_root.join(format!("{case}.xml"))).expect("fixture xml");
        let xml_sha = sha256_hex(&xml_bytes);
        assert_eq!(
            record["xml_sha256"].as_str().expect("xml_sha256"),
            xml_sha,
            "{case}: manifest xml_sha256 与现场重算不一致"
        );
        assert_eq!(
            xml_sha,
            case_reference["xml_sha256"]
                .as_str()
                .expect("参考 xml_sha256"),
            "{case}: xml_sha256 与 M1 参考不一致"
        );

        // `qpos_bits`：manifest 记录 == M1 参考（逐位）。
        let qpos_bits: Vec<u64> = record["qpos_bits"]
            .as_array()
            .expect("qpos_bits")
            .iter()
            .map(|value| value.as_u64().expect("bit"))
            .collect();
        let reference_qpos_bits: Vec<u64> = case_reference["qpos_bits"]
            .as_array()
            .expect("参考 qpos_bits")
            .iter()
            .map(|value| value.as_u64().expect("bit"))
            .collect();
        assert_eq!(
            qpos_bits, reference_qpos_bits,
            "{case}: qpos_bits 与 M1 参考不一致"
        );

        // `geom_xpos` / `geom_xmat`：bins 位型 == M1 GPU 参考位型。
        let directory = m0b_cases_root().join(case);
        let inputs = load_case(&directory);
        let reference_xpos = json_f32_bits(&case_reference["output"]["geom_xpos"]);
        let reference_xmat = json_f32_bits(&case_reference["output"]["geom_xmat"]);
        assert_eq!(
            f32_bits(&inputs.geom_xpos),
            reference_xpos,
            "{case}: geom_xpos 与 M1 参考不是逐位一致"
        );
        assert_eq!(
            f32_bits(&inputs.geom_xmat),
            reference_xmat,
            "{case}: geom_xmat 与 M1 参考不是逐位一致"
        );

        // 交叉校验结论记录（manifest 内自洽）。
        for key in [
            "xml_sha256",
            "qpos_bits",
            "geom_xpos_bits",
            "geom_xmat_bits",
            "collision_count",
        ] {
            assert_eq!(
                record["cross_check"][key]["match"].as_bool(),
                Some(true),
                "{case}: manifest cross_check.{key} 应为 true"
            );
        }
        assert_eq!(
            record["cpu_selfcheck"]["result"].as_str(),
            Some("PASS"),
            "{case}: manifest cpu_selfcheck 应为 PASS"
        );
    }
}
