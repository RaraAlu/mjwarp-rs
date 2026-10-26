//! G10@P5 M3：mesh-mesh 32 项 case bins 的 CPU 装配、同一来源与真实校验（不建 CUDA context）。
//!
//! 自 `tests/g10_p5_mesh_poly_case_assembly.rs` 纯搬移（仅为满足单文件 500 行上限；
//! 测试名、断言文本与数据块保持不变）。覆盖：
//! 1. M3 bins ↔ `mesh-case-manifest.json` 的 sha256/count 逐项一致（32 项）；
//! 2. 9 个 sidecar slice 长度按 `nmesh=2 / nmeshvert=16 / nmeshpoly=12` 推导：
//!    `[2,2,36,12,12,48,16,16,48]`；
//! 3. 同一来源：`xml_sha256` / `qpos_bits` / `geom_xpos` / `geom_xmat` 与 M3 GPU 参考逐位一致；
//! 4. 真实 `validate_collision_inputs` 接受 M3 bins（32 项 + 9 项 sidecar + O7 值域）。
//!
//! 运行：`cargo test --locked --test g10_p5_mesh_m3_case_assembly`

#[path = "support/mesh_poly_case_support.rs"]
mod support;

use support::{
    INPUT_BINS, f32_bits, json_f32_bits, load_case, m3_cases_root, m3_manifest, m3_reference,
    sha256_hex,
};

const M3_CASES: [&str; 1] = ["mesh-mesh"];
/// M3 `(MESH, MESH)`：nmesh=2 / nmeshvert=16 / nmeshpoly=12 / nmeshpolyvert=48 /
/// nmeshpolymap=48 → 9 个 sidecar slice 长度（polyadr 逐 mesh、polyvert* 逐 polygon、
/// polymap* 逐 vertex）。
const M3_MESH_POLY_LENGTHS: [usize; 9] = [2, 2, 36, 12, 12, 48, 16, 16, 48];

#[test]
fn m3_same_source_matches_m3_reference() {
    let manifest = m3_manifest();
    let reference = m3_reference();
    let fixture_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/g10-p5-mesh-multicontact");

    let arrays_order: Vec<&str> = manifest["arrays_order"]
        .as_array()
        .expect("arrays_order")
        .iter()
        .map(|value| value.as_str().expect("数组名"))
        .collect();
    assert_eq!(
        arrays_order,
        INPUT_BINS.to_vec(),
        "M3 manifest 数组顺序须与 32 项口径一致"
    );

    for case in M3_CASES {
        let record = &manifest["cases"][case];
        let case_reference = &reference["cases"][case];

        // bins ↔ manifest：32 项 sha256/count 逐项一致。
        let arrays = record["arrays"].as_object().expect("case arrays");
        assert_eq!(arrays.len(), 32, "{case}: 应声明 32 个数组");
        let directory = m3_cases_root().join(case);
        for name in INPUT_BINS {
            let bytes = std::fs::read(directory.join(format!("{name}.bin")))
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
        // 9 个 slice 长度按 nmesh=2 / nmeshvert=16 / nmeshpoly=12 推导：
        // polynum/polyadr=2、polynormal=36（12 行 ×3）、polyvertadr/polyvertnum=12、
        // polyvert=48、polymapadr/polymapnum=16、polymap=48。
        let lengths: [usize; 9] = [
            arrays["mesh_polynum"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polyadr"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polynormal"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polyvertadr"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polyvertnum"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polyvert"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polymapadr"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polymapnum"]["count"].as_u64().unwrap() as usize,
            arrays["mesh_polymap"]["count"].as_u64().unwrap() as usize,
        ];
        assert_eq!(
            lengths, M3_MESH_POLY_LENGTHS,
            "{case}: mesh_poly 数组长度应为真实双 mesh 形状"
        );

        // 同一来源：xml_sha256「manifest == 现场重算 == M3 参考」。
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
            "{case}: xml_sha256 与 M3 参考不一致"
        );

        // `qpos_bits`：manifest 记录 == M3 参考（逐位）。
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
            "{case}: qpos_bits 与 M3 参考不一致"
        );

        // `geom_xpos` / `geom_xmat`：bins 位型 == M3 GPU 参考位型。
        let inputs = load_case(&directory);
        let reference_xpos = json_f32_bits(&case_reference["output"]["geom_xpos"]);
        let reference_xmat = json_f32_bits(&case_reference["output"]["geom_xmat"]);
        assert_eq!(
            f32_bits(&inputs.geom_xpos),
            reference_xpos,
            "{case}: geom_xpos 与 M3 参考不是逐位一致"
        );
        assert_eq!(
            f32_bits(&inputs.geom_xmat),
            reference_xmat,
            "{case}: geom_xmat 与 M3 参考不是逐位一致"
        );

        // manifest 内部交叉校验结论 + 18 条自检（含 adr==cumsum 与值域）必须 PASS。
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

#[test]
fn real_cpu_validation_accepts_m3_mesh_mesh_bins() {
    // O7（M3）：真实 M3 bins（双 mesh sidecar）必须通过真实 `validate_inputs`（含值域校验）。
    use mjwarp_rs::runtime::collision_gpu::validate_collision_inputs;

    for case in M3_CASES {
        let inputs = load_case(&m3_cases_root().join(case));
        assert!(
            inputs.mesh_poly_present(),
            "{case}: M3 真实 bins 应构造 sidecar（9 项 present）"
        );
        assert_eq!(
            inputs.mesh_poly_lengths(),
            M3_MESH_POLY_LENGTHS,
            "{case}: 装配后 9 个 slice 长度须与 bins 一致"
        );
        assert_eq!(inputs.geom_dataid, vec![0, 1], "{case}: geom_dataid");
        assert_eq!(
            validate_collision_inputs(&inputs.inputs()),
            Ok(()),
            "{case}: 真实 validate_inputs 应通过（32 项 + 9 项 sidecar + O7 值域）"
        );
    }
}
