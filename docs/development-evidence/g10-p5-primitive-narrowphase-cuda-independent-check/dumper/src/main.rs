//! QA 独立 harness：用产品公开 API `CollisionGpuProgram` 跑 primitive case（及 launcher
//! 回归 case），把产品原始输出逐数组落盘。
//!
//! 目的：不依赖交付的 `tests/g10_p5_primitive_cuda.rs` 比较器——QA 自己 dump 原始输出，
//! 再由 `qa_primitive_check.py` 逐字节对照 `qa-upstream-gpu.json` 的上游 f32 GPU 参考。
//!
//! 每个 case 一次进程调用（`run_window.ps1` 逐 case 起进程），以便「任一 case 不一致立即停」。
//!
//! 运行前置：NVRTC 运行库在 PATH（`target/toolchains/cuda-12.8.1/bin`）+ NVIDIA GPU。
//! 用法：qa-primitive-dumper <cases-dir> <out-dir> <case> [kernel]
//!   kernel ∈ contact|hfield|sdf|primitive（省略时按 case 的 `meta.txt kind` 选）

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuProgram, CollisionKernel,
};

fn read_meta(dir: &Path) -> BTreeMap<String, String> {
    let text = fs::read_to_string(dir.join("meta.txt")).expect("meta.txt");
    text.lines()
        .filter_map(|line| line.split_once(' '))
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

fn meta_i32(meta: &BTreeMap<String, String>, key: &str) -> i32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("meta {key}: {e}"))
}

fn meta_f32(meta: &BTreeMap<String, String>, key: &str) -> f32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("meta {key}: {e}"))
}

fn meta_f32s<const N: usize>(meta: &BTreeMap<String, String>, key: &str) -> [f32; N] {
    let values: Vec<f32> = meta
        .get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .split_whitespace()
        .map(|v| v.parse().unwrap_or_else(|e| panic!("meta {key}: {e}")))
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

fn read_f32(dir: &Path, name: &str) -> Vec<f32> {
    let bytes = fs::read(dir.join(format!("{name}.bin"))).expect(name);
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn read_i32(dir: &Path, name: &str) -> Vec<i32> {
    let bytes = fs::read(dir.join(format!("{name}.bin"))).expect(name);
    bytes
        .chunks_exact(4)
        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn write_f32(path: &Path, values: &[f32]) {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    fs::write(path, bytes).expect(path.to_str().unwrap());
}

fn write_i32(path: &Path, values: &[i32]) {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    fs::write(path, bytes).expect(path.to_str().unwrap());
}

fn kernel_of(name: &str) -> CollisionKernel {
    match name {
        "contact" => CollisionKernel::Contact,
        "hfield" => CollisionKernel::Hfield,
        "sdf" => CollisionKernel::Sdf,
        "primitive" => CollisionKernel::Primitive,
        other => panic!("未知 kernel 覆盖值: {other:?}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 4,
        "usage: qa-primitive-dumper <cases-dir> <out-dir> <case> [kernel]"
    );
    let cases_dir = PathBuf::from(&args[1]);
    let out_dir = PathBuf::from(&args[2]);
    let case = args[3].as_str();
    fs::create_dir_all(&out_dir).expect("创建输出目录");

    let dir = cases_dir.join(case);
    let meta_map = read_meta(&dir);
    let kind = match args.get(4) {
        Some(name) => kernel_of(name),
        None => match meta_map.get("kind").map(String::as_str) {
            Some("contact") => CollisionKernel::Contact,
            Some("hfield") => CollisionKernel::Hfield,
            Some("sdf") => CollisionKernel::Sdf,
            other => panic!("未知 kind: {other:?}"),
        },
    };
    let meta = build_meta(&meta_map);
    let efc_width = meta.efc_width.max(1) as usize;

    let geom_xpos = read_f32(&dir, "geom_xpos");
    let geom_xmat = read_f32(&dir, "geom_xmat");
    let geom_size = read_f32(&dir, "geom_size");
    let geom_margin = read_f32(&dir, "geom_margin");
    let geom_gap = read_f32(&dir, "geom_gap");
    let geom_rbound = read_f32(&dir, "geom_rbound");
    let geom_aabb = read_f32(&dir, "geom_aabb");
    let geom_type = read_i32(&dir, "geom_type");
    let geom_dataid = read_i32(&dir, "geom_dataid");
    let mesh_vertadr = read_i32(&dir, "mesh_vertadr");
    let mesh_vertnum = read_i32(&dir, "mesh_vertnum");
    let mesh_graphadr = read_i32(&dir, "mesh_graphadr");
    let mesh_octadr = read_i32(&dir, "mesh_octadr");
    let mesh_vert = read_f32(&dir, "mesh_vert");
    let mesh_graph = read_i32(&dir, "mesh_graph");
    let hfield_size = read_f32(&dir, "hfield_size");
    let hfield_nrow = read_i32(&dir, "hfield_nrow");
    let hfield_ncol = read_i32(&dir, "hfield_ncol");
    let hfield_adr = read_i32(&dir, "hfield_adr");
    let hfield_data = read_f32(&dir, "hfield_data");
    let oct_aabb = read_f32(&dir, "oct_aabb");
    let oct_child = read_i32(&dir, "oct_child");
    let oct_coeff = read_f32(&dir, "oct_coeff");

    let inputs = CollisionGpuInputs {
        geom_xpos: &geom_xpos,
        geom_xmat: &geom_xmat,
        geom_size: &geom_size,
        geom_margin: &geom_margin,
        geom_gap: &geom_gap,
        geom_rbound: &geom_rbound,
        geom_aabb: &geom_aabb,
        geom_type: &geom_type,
        geom_dataid: &geom_dataid,
        mesh_vertadr: &mesh_vertadr,
        mesh_vertnum: &mesh_vertnum,
        mesh_graphadr: &mesh_graphadr,
        mesh_octadr: &mesh_octadr,
        mesh_vert: &mesh_vert,
        mesh_graph: &mesh_graph,
        hfield_size: &hfield_size,
        hfield_nrow: &hfield_nrow,
        hfield_ncol: &hfield_ncol,
        hfield_adr: &hfield_adr,
        hfield_data: &hfield_data,
        oct_aabb: &oct_aabb,
        oct_child: &oct_child,
        oct_coeff: &oct_coeff,
        meta,
    };

    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译并加载四个入口");
    println!("qa_dumper_program_ready kernel={kind:?}");

    let output = program
        .run(kind, &inputs)
        .unwrap_or_else(|e| panic!("{case} 运行失败：{e}"));

    let case_out = out_dir.join(case);
    fs::create_dir_all(&case_out).expect("创建 case 输出目录");
    let rows = output.nacon.max(0) as usize;
    println!(
        "qa_dumper_case case={case} kind={kind:?} nacon={} nccd={} rows={rows}",
        output.nacon, output.nccd
    );
    write_i32(&case_out.join("nacon.bin"), &[output.nacon]);
    write_i32(&case_out.join("nccd.bin"), &[output.nccd]);
    let overflow: Vec<i32> = output.overflow.iter().map(|v| *v as i32).collect();
    write_i32(&case_out.join("overflow.bin"), &overflow);
    write_f32(&case_out.join("dist.bin"), &output.dist[..rows.min(output.dist.len())]);
    let pos_len = (rows * 3).min(output.pos.len());
    write_f32(&case_out.join("pos.bin"), &output.pos[..pos_len]);
    let frame_len = (rows * 9).min(output.frame.len());
    write_f32(&case_out.join("frame.bin"), &output.frame[..frame_len]);
    write_f32(
        &case_out.join("includemargin.bin"),
        &output.includemargin[..rows.min(output.includemargin.len())],
    );
    write_f32(
        &case_out.join("friction.bin"),
        &output.friction[..(rows * 5).min(output.friction.len())],
    );
    write_f32(
        &case_out.join("solref.bin"),
        &output.solref[..(rows * 2).min(output.solref.len())],
    );
    write_f32(
        &case_out.join("solreffriction.bin"),
        &output.solreffriction[..(rows * 2).min(output.solreffriction.len())],
    );
    write_f32(
        &case_out.join("solimp.bin"),
        &output.solimp[..(rows * 5).min(output.solimp.len())],
    );
    write_f32(
        &case_out.join("adhesion.bin"),
        &output.adhesion[..rows.min(output.adhesion.len())],
    );
    write_i32(
        &case_out.join("dim.bin"),
        &output.dim[..rows.min(output.dim.len())],
    );
    write_i32(
        &case_out.join("geom.bin"),
        &output.geom[..(rows * 2).min(output.geom.len())],
    );
    write_i32(
        &case_out.join("efc_address.bin"),
        &output.efc_address[..(rows * efc_width).min(output.efc_address.len())],
    );
    write_i32(
        &case_out.join("worldid.bin"),
        &output.worldid[..rows.min(output.worldid.len())],
    );
    write_i32(
        &case_out.join("geomcollisionid.bin"),
        &output.geomcollisionid[..rows.min(output.geomcollisionid.len())],
    );
    write_i32(
        &case_out.join("type.bin"),
        &output.contact_type[..rows.min(output.contact_type.len())],
    );
    // 同时记录回读长度，便于比较器核对「数组实际容量」而非只看 nacon。
    let lengths = format!(
        "dist {}\npos {}\nframe {}\nincludemargin {}\nfriction {}\nsolref {}\nsolreffriction {}\nsolimp {}\nadhesion {}\ndim {}\ngeom {}\nefc_address {}\nworldid {}\ngeomcollisionid {}\ntype {}\noverflow {}\n",
        output.dist.len(),
        output.pos.len(),
        output.frame.len(),
        output.includemargin.len(),
        output.friction.len(),
        output.solref.len(),
        output.solreffriction.len(),
        output.solimp.len(),
        output.adhesion.len(),
        output.dim.len(),
        output.geom.len(),
        output.efc_address.len(),
        output.worldid.len(),
        output.geomcollisionid.len(),
        output.contact_type.len(),
        output.overflow.len(),
    );
    fs::write(case_out.join("array-lengths.txt"), lengths).expect("array-lengths");
    println!("qa_dumper_done case={case}");
}
