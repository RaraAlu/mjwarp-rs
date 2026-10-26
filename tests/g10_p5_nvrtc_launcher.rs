//! G10@P5：产品 NVRTC 入口的 GPU 窗口验收（默认为 ignored，本机不跑）。
//!
//! 用产品 API（`CollisionGpuProgram::new(0)` + `run`）对 3 个冻结 case 重跑 G5 探针输入，
//! 并与探针 `out1/` 逐字段比对：先位级比较并打印 bit_equal；非位级按 G5 既有判据断言
//! （`dist` 自身量级 ≤4 ULP、`ref == 0` 用 4 ULP@1.0；`pos` 按 `max_i |ref_i|` 的
//! 4·ULP(scale)；`frame` 按 scale=1.0 的 4 ULP=4.768e-7；参数/整数/`nacon`/`nccd`/
//! `overflow` 位级）。数组按 probe 口径取前 `nacon` 行。
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput, CollisionGpuProgram, CollisionKernel,
};

#[path = "support/launcher_compare_support.rs"]
mod launcher_compare;

use launcher_compare::{
    CaseReport, check_dist, check_exact, check_exact_f32, check_vectors, compare_case,
    g5_contact_order, permute_rows_f32, permute_rows_i32,
};
const CASES: [&str; 3] = ["hfield-sphere", "sdf-box", "ellipsoid-box"];

fn cases_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

fn read_bytes(directory: &Path, name: &str) -> Vec<u8> {
    let path = directory.join(format!("{name}.bin"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()))
}

fn read_f32(directory: &Path, name: &str) -> Vec<f32> {
    let bytes = read_bytes(directory, name);
    assert!(
        bytes.len().is_multiple_of(4),
        "{name}.bin 长度不是 4 的倍数"
    );
    let (chunks, _) = bytes.as_chunks::<4>();
    chunks
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

fn read_i32(directory: &Path, name: &str) -> Vec<i32> {
    let bytes = read_bytes(directory, name);
    assert!(
        bytes.len().is_multiple_of(4),
        "{name}.bin 长度不是 4 的倍数"
    );
    let (chunks, _) = bytes.as_chunks::<4>();
    chunks
        .iter()
        .map(|chunk| i32::from_le_bytes(*chunk))
        .collect()
}

fn read_efc_width(directory: &Path) -> usize {
    let path = directory.join("efc_width.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    text.trim()
        .parse()
        .unwrap_or_else(|error| panic!("{} 解析失败：{error}", path.display()))
}

fn read_meta(directory: &Path) -> BTreeMap<String, String> {
    let path = directory.join("meta.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    text.lines()
        .filter_map(|line| line.split_once(' '))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn meta_i32(meta: &BTreeMap<String, String>, key: &str) -> i32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("meta {key} 不是 i32：{error}"))
}

fn meta_f32(meta: &BTreeMap<String, String>, key: &str) -> f32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("meta {key} 不是 f32：{error}"))
}

fn meta_f32s<const N: usize>(meta: &BTreeMap<String, String>, key: &str) -> [f32; N] {
    let values: Vec<f32> = meta
        .get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .split_whitespace()
        .map(|value| {
            value
                .parse()
                .unwrap_or_else(|error| panic!("meta {key} 分量非 f32：{error}"))
        })
        .collect();
    assert_eq!(values.len(), N, "meta {key} 分量数不符");
    let mut array = [0.0; N];
    array.copy_from_slice(&values);
    array
}

fn build_meta(meta: &BTreeMap<String, String>) -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: meta_i32(meta, "nworld"),
        ngeom: meta_i32(meta, "ngeom"),
        g1: meta_i32(meta, "g1"),
        g2: meta_i32(meta, "g2"),
        type1: meta_i32(meta, "type1"),
        type2: meta_i32(meta, "type2"),
        geomgeomid: 0,
        naccdmax: meta_i32(meta, "naccdmax"),
        hfield_id: meta_i32(meta, "hfield_id"),
        oct_root: meta_i32(meta, "oct_root"),
        noct: meta_i32(meta, "noct"),
        tolerance: meta_f32(meta, "tolerance"),
        margin: meta_f32(meta, "margin"),
        gap: meta_f32(meta, "gap"),
        gjk_iterations: meta_i32(meta, "gjk_iterations"),
        epa_iterations: meta_i32(meta, "epa_iterations"),
        condim: meta_i32(meta, "condim"),
        pairid0: meta_i32(meta, "pairid0"),
        pairid1: meta_i32(meta, "pairid1"),
        friction: meta_f32s::<5>(meta, "friction"),
        solref: meta_f32s::<2>(meta, "solref"),
        solreffriction: meta_f32s::<2>(meta, "solreffriction"),
        solimp: meta_f32s::<5>(meta, "solimp"),
        adhesion: meta_f32(meta, "adhesion"),
        sdf_initpoints: meta_i32(meta, "sdf_initpoints"),
        sdf_iterations: meta_i32(meta, "sdf_iterations"),
        naconmax: meta_i32(meta, "naconmax"),
        efc_width: meta_i32(meta, "efc_width"),
    }
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn product_nvrtc_launcher_matches_g5_probe_out1() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let mut failures: Vec<String> = Vec::new();
    for case in CASES {
        let directory = cases_root().join(case);
        let meta_map = read_meta(&directory);
        let kind = match meta_map.get("kind").map(String::as_str) {
            Some("contact") => CollisionKernel::Contact,
            Some("hfield") => CollisionKernel::Hfield,
            Some("sdf") => CollisionKernel::Sdf,
            other => panic!("未知 kind：{other:?}"),
        };
        let meta = build_meta(&meta_map);

        macro_rules! load {
            ($($name:ident : $reader:ident),* $(,)?) => {
                $(let $name = $reader(&directory, stringify!($name));)*
            };
        }
        load!(
            geom_xpos: read_f32, geom_xmat: read_f32, geom_size: read_f32, geom_margin: read_f32,
            geom_gap: read_f32, geom_rbound: read_f32, geom_aabb: read_f32, geom_type: read_i32,
            geom_dataid: read_i32, mesh_vertadr: read_i32, mesh_vertnum: read_i32,
            mesh_graphadr: read_i32, mesh_octadr: read_i32, mesh_vert: read_f32,
            mesh_graph: read_i32, hfield_size: read_f32, hfield_nrow: read_i32,
            hfield_ncol: read_i32, hfield_adr: read_i32, hfield_data: read_f32,
            oct_aabb: read_f32, oct_child: read_i32, oct_coeff: read_f32,
        );
        macro_rules! inputs {
            ($meta:expr, $($field:ident),* $(,)?) => {
                CollisionGpuInputs {
                    $($field: &$field,)*
                    // M0a：G5 冻结 case 只有 23 项输入；9 个 mesh_poly 数组全空（sidecar 不构造）。
                    mesh_polynum: &[],
                    mesh_polyadr: &[],
                    mesh_polynormal: &[],
                    mesh_polyvertadr: &[],
                    mesh_polyvertnum: &[],
                    mesh_polyvert: &[],
                    mesh_polymapadr: &[],
                    mesh_polymapnum: &[],
                    mesh_polymap: &[],
                    meta: $meta,
                }
            };
        }
        #[rustfmt::skip]
        let inputs = inputs!(
            meta,
            geom_xpos, geom_xmat, geom_size, geom_margin, geom_gap, geom_rbound, geom_aabb,
            geom_type, geom_dataid, mesh_vertadr, mesh_vertnum, mesh_graphadr, mesh_octadr,
            mesh_vert, mesh_graph, hfield_size, hfield_nrow, hfield_ncol, hfield_adr,
            hfield_data, oct_aabb, oct_child, oct_coeff,
        );

        let output = program.run(kind, &inputs).unwrap_or_else(|error| {
            panic!("{case} 运行失败：{error}");
        });
        let report = compare_case(case, kind, &directory, &output);
        report.print();
        for failure in &report.failures {
            println!("FAIL {case}: {failure}");
        }
        let tagged = |failure: String| format!("{case}: {failure}");
        failures.extend(report.failures.into_iter().map(tagged));
    }
    assert!(
        failures.is_empty(),
        "产品 NVRTC 入口与 G5 探针 out1 不一致（{} 条）",
        failures.len()
    );
}

#[test]
fn world_order_normalization_turns_row_swap_into_zero_diff() {
    // Chief 条件 2：同一份 dump（仓库内 `ellipsoid-box/out1`）构造 attempt1 观测到的「两行整体互换」，
    // 旧口径（原始行序逐字段比）应复现 attempt1 的 14 条差异；新口径（G5 行序归一化后比）应为 0 条。
    let expected = cases_root().join("ellipsoid-box").join("out1");
    let efc_width = read_efc_width(&expected);
    let ref_worldid = read_i32(&expected, "worldid");
    let rows = ref_worldid.len();
    assert_eq!(rows, 2, "ellipsoid-box dump 应为 2 行");
    let swap_row = |row: usize| -> usize {
        match row {
            0 => 1,
            1 => 0,
            other => other,
        }
    };
    let swap_f = |values: &[f32], width: usize| -> Vec<f32> {
        let mut out = Vec::with_capacity(values.len());
        for row in 0..rows {
            let source = swap_row(row);
            out.extend_from_slice(&values[source * width..(source + 1) * width]);
        }
        out
    };
    let swap_i = |values: &[i32], width: usize| -> Vec<i32> {
        let mut out = Vec::with_capacity(values.len());
        for row in 0..rows {
            let source = swap_row(row);
            out.extend_from_slice(&values[source * width..(source + 1) * width]);
        }
        out
    };

    let ref_dist = read_f32(&expected, "dist");
    let ref_pos = read_f32(&expected, "pos");
    let ref_frame = read_f32(&expected, "frame");
    let ref_includemargin = read_f32(&expected, "includemargin");
    let ref_friction = read_f32(&expected, "friction");
    let ref_solref = read_f32(&expected, "solref");
    let ref_solreffriction = read_f32(&expected, "solreffriction");
    let ref_solimp = read_f32(&expected, "solimp");
    let ref_adhesion = read_f32(&expected, "adhesion");
    let ref_dim = read_i32(&expected, "dim");
    let ref_geom = read_i32(&expected, "geom");
    let ref_efc = read_i32(&expected, "efc_address");
    let ref_type = read_i32(&expected, "type");
    let ref_id = read_i32(&expected, "geomcollisionid");
    let ref_nacon = read_i32(&expected, "nacon");
    let ref_nccd = read_i32(&expected, "nccd");
    let ref_overflow: Vec<u32> = read_i32(&expected, "overflow")
        .iter()
        .map(|value| *value as u32)
        .collect();

    let dev = CollisionGpuOutput {
        nacon: ref_nacon[0],
        nccd: ref_nccd[0],
        overflow: vec![0_u32; ref_overflow.len()],
        dist: swap_f(&ref_dist, 1),
        pos: swap_f(&ref_pos, 3),
        frame: swap_f(&ref_frame, 9),
        includemargin: swap_f(&ref_includemargin, 1),
        friction: swap_f(&ref_friction, 5),
        solref: swap_f(&ref_solref, 2),
        solreffriction: swap_f(&ref_solreffriction, 2),
        solimp: swap_f(&ref_solimp, 5),
        adhesion: swap_f(&ref_adhesion, 1),
        dim: swap_i(&ref_dim, 1),
        geom: swap_i(&ref_geom, 2),
        efc_address: swap_i(&ref_efc, efc_width),
        worldid: swap_i(&ref_worldid, 1),
        contact_type: swap_i(&ref_type, 1),
        geomcollisionid: swap_i(&ref_id, 1),
    };

    // 旧口径：原始行序逐字段比（attempt1 的失败集合应被复现）。
    let mut old = CaseReport::new("ellipsoid-box", CollisionKernel::Contact, &dev);
    check_dist(&mut old, "dist", &dev.dist, &ref_dist);
    check_vectors(&mut old, "pos", &dev.pos, &ref_pos, 3, false);
    check_vectors(&mut old, "frame", &dev.frame, &ref_frame, 9, true);
    check_exact_f32(
        &mut old,
        "includemargin",
        &dev.includemargin,
        &ref_includemargin,
    );
    check_exact_f32(&mut old, "friction", &dev.friction, &ref_friction);
    check_exact_f32(&mut old, "solref", &dev.solref, &ref_solref);
    check_exact_f32(
        &mut old,
        "solreffriction",
        &dev.solreffriction,
        &ref_solreffriction,
    );
    check_exact_f32(&mut old, "solimp", &dev.solimp, &ref_solimp);
    check_exact_f32(&mut old, "adhesion", &dev.adhesion, &ref_adhesion);
    check_exact(&mut old, "dim", &dev.dim, &ref_dim);
    check_exact(&mut old, "geom", &dev.geom, &ref_geom);
    check_exact(&mut old, "efc_address", &dev.efc_address, &ref_efc);
    check_exact(&mut old, "worldid", &dev.worldid, &ref_worldid);
    check_exact(&mut old, "geomcollisionid", &dev.geomcollisionid, &ref_id);
    check_exact(&mut old, "contact_type", &dev.contact_type, &ref_type);

    // 新口径：G5 行序归一化（worldid 分组 + 组内 sort_contacts 键）后逐字段比。
    let device_order = g5_contact_order(&dev.worldid, &dev.dist, &dev.pos, &dev.geomcollisionid);
    let reference_order = g5_contact_order(&ref_worldid, &ref_dist, &ref_pos, &ref_id);
    let dev_f = |values: &[f32], width: usize| permute_rows_f32(values, width, &device_order);
    let ref_f = |values: &[f32], width: usize| permute_rows_f32(values, width, &reference_order);
    let dev_i = |values: &[i32], width: usize| permute_rows_i32(values, width, &device_order);
    let ref_i = |values: &[i32], width: usize| permute_rows_i32(values, width, &reference_order);

    let mut new = CaseReport::new("ellipsoid-box", CollisionKernel::Contact, &dev);
    check_dist(&mut new, "dist", &dev_f(&dev.dist, 1), &ref_f(&ref_dist, 1));
    check_vectors(
        &mut new,
        "pos",
        &dev_f(&dev.pos, 3),
        &ref_f(&ref_pos, 3),
        3,
        false,
    );
    check_vectors(
        &mut new,
        "frame",
        &dev_f(&dev.frame, 9),
        &ref_f(&ref_frame, 9),
        9,
        true,
    );
    check_exact_f32(
        &mut new,
        "includemargin",
        &dev_f(&dev.includemargin, 1),
        &ref_f(&ref_includemargin, 1),
    );
    check_exact_f32(
        &mut new,
        "friction",
        &dev_f(&dev.friction, 5),
        &ref_f(&ref_friction, 5),
    );
    check_exact_f32(
        &mut new,
        "solref",
        &dev_f(&dev.solref, 2),
        &ref_f(&ref_solref, 2),
    );
    check_exact_f32(
        &mut new,
        "solreffriction",
        &dev_f(&dev.solreffriction, 2),
        &ref_f(&ref_solreffriction, 2),
    );
    check_exact_f32(
        &mut new,
        "solimp",
        &dev_f(&dev.solimp, 5),
        &ref_f(&ref_solimp, 5),
    );
    check_exact_f32(
        &mut new,
        "adhesion",
        &dev_f(&dev.adhesion, 1),
        &ref_f(&ref_adhesion, 1),
    );
    check_exact(&mut new, "dim", &dev_i(&dev.dim, 1), &ref_i(&ref_dim, 1));
    check_exact(&mut new, "geom", &dev_i(&dev.geom, 2), &ref_i(&ref_geom, 2));
    check_exact(
        &mut new,
        "efc_address",
        &dev_i(&dev.efc_address, efc_width),
        &ref_i(&ref_efc, efc_width),
    );
    check_exact(
        &mut new,
        "worldid",
        &dev_i(&dev.worldid, 1),
        &ref_i(&ref_worldid, 1),
    );
    check_exact(
        &mut new,
        "geomcollisionid",
        &dev_i(&dev.geomcollisionid, 1),
        &ref_i(&ref_id, 1),
    );
    check_exact(
        &mut new,
        "contact_type",
        &dev_i(&dev.contact_type, 1),
        &ref_i(&ref_type, 1),
    );

    println!(
        "normalize_proof: raw_order_failures={} sorted_failures={} rows={rows} max_dist_ulp(old)={} max_pos_ulp(old)={} max_frame_ulp(old)={}",
        old.failures.len(),
        new.failures.len(),
        old.max_dist_ulp,
        old.max_pos_ulp,
        old.max_frame_ulp
    );
    assert_eq!(
        old.failures.len(),
        14,
        "旧口径应复现 attempt1 的 14 条差异，实际 {} 条：{:?}",
        old.failures.len(),
        old.failures
    );
    assert_eq!(
        new.failures.len(),
        0,
        "新口径应为 0 条差异，实际 {:?}",
        new.failures
    );
}
