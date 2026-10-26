//! G10@P5 M0a：输入校验单测（纯 CPU，不建 context、不跑 GPU）。
//!
//! 覆盖 meta 范围、逐 slice 长度，以及 M0a mesh polygon sidecar 的写死语义：9 数组全空
//! （sidecar 不构造 / 行为不变）、部分空或长度不自洽（`InvalidArgument`）。

use super::abi::validate_inputs;
use super::{CollisionGpuError, CollisionGpuInputs, CollisionGpuMeta};

fn valid_meta() -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: 2,
        ngeom: 2,
        g1: 0,
        g2: 1,
        type1: 2,
        type2: 6,
        geomgeomid: 0,
        naccdmax: 8192,
        hfield_id: -1,
        oct_root: -1,
        noct: 0,
        tolerance: 1.0e-6,
        margin: 0.0,
        gap: 0.0,
        gjk_iterations: 35,
        epa_iterations: 35,
        condim: 3,
        pairid0: -1,
        pairid1: -1,
        friction: [1.0, 1.0, 0.005, 1.0e-4, 1.0e-4],
        solref: [0.02, 1.0],
        solreffriction: [0.0, 0.0],
        solimp: [0.9, 0.95, 0.001, 0.5, 2.0],
        adhesion: 0.0,
        sdf_initpoints: 40,
        sdf_iterations: 10,
        naconmax: 8192,
        efc_width: 4,
    }
}

/// 与 `valid_meta` 匹配的宿主输入（geom 数组按 `nworld*ngeom` 铺满，模型静态数组为空）。
struct SampleInputs {
    geom_xpos: Vec<f32>,
    geom_xmat: Vec<f32>,
    geom_size: Vec<f32>,
    geom_margin: Vec<f32>,
    geom_gap: Vec<f32>,
    geom_rbound: Vec<f32>,
    geom_aabb: Vec<f32>,
    geom_type: Vec<i32>,
    geom_dataid: Vec<i32>,
    empty_f32: Vec<f32>,
    empty_i32: Vec<i32>,
    meta: CollisionGpuMeta,
}

impl SampleInputs {
    fn new(meta: CollisionGpuMeta) -> Self {
        let nworld = meta.nworld as usize;
        let ngeom = meta.ngeom as usize;
        let mut geom_xmat = vec![0.0_f32; nworld * ngeom * 9];
        for rotation in geom_xmat.as_chunks_mut::<9>().0 {
            rotation[0] = 1.0;
            rotation[4] = 1.0;
            rotation[8] = 1.0;
        }
        Self {
            geom_xpos: vec![0.0; nworld * ngeom * 3],
            geom_xmat,
            geom_size: vec![0.1; ngeom * 3],
            geom_margin: vec![0.0; ngeom],
            geom_gap: vec![0.0; ngeom],
            geom_rbound: vec![0.1; ngeom],
            geom_aabb: vec![0.0; ngeom * 6],
            geom_type: vec![2; ngeom],
            geom_dataid: vec![-1; ngeom],
            empty_f32: Vec::new(),
            empty_i32: Vec::new(),
            meta,
        }
    }

    fn inputs(&self) -> CollisionGpuInputs<'_> {
        CollisionGpuInputs {
            geom_xpos: &self.geom_xpos,
            geom_xmat: &self.geom_xmat,
            geom_size: &self.geom_size,
            geom_margin: &self.geom_margin,
            geom_gap: &self.geom_gap,
            geom_rbound: &self.geom_rbound,
            geom_aabb: &self.geom_aabb,
            geom_type: &self.geom_type,
            geom_dataid: &self.geom_dataid,
            mesh_vertadr: &self.empty_i32,
            mesh_vertnum: &self.empty_i32,
            mesh_graphadr: &self.empty_i32,
            mesh_octadr: &self.empty_i32,
            mesh_vert: &self.empty_f32,
            mesh_graph: &self.empty_i32,
            // M0a sidecar：默认 9 项全空（不构造 sidecar / 功能不启用）。
            mesh_polynum: &self.empty_i32,
            mesh_polyadr: &self.empty_i32,
            mesh_polynormal: &self.empty_f32,
            mesh_polyvertadr: &self.empty_i32,
            mesh_polyvertnum: &self.empty_i32,
            mesh_polyvert: &self.empty_i32,
            mesh_polymapadr: &self.empty_i32,
            mesh_polymapnum: &self.empty_i32,
            mesh_polymap: &self.empty_i32,
            hfield_size: &self.empty_f32,
            hfield_nrow: &self.empty_i32,
            hfield_ncol: &self.empty_i32,
            hfield_adr: &self.empty_i32,
            hfield_data: &self.empty_f32,
            oct_aabb: &self.empty_f32,
            oct_child: &self.empty_i32,
            oct_coeff: &self.empty_f32,
            meta: self.meta,
        }
    }
}

#[test]
fn rejects_invalid_meta_but_accepts_matching_inputs() {
    let sample = SampleInputs::new(valid_meta());
    assert_eq!(validate_inputs(&sample.inputs()), Ok(()));
    let mut sample = sample;
    for (field, message) in [
        ("nworld", "nworld 必须为正"),
        ("ngeom", "ngeom 必须为正"),
        ("naconmax", "naconmax 必须为正"),
        ("efc_width", "efc_width 必须为正"),
        ("g1", "g1 超出 geom 范围"),
        ("g2", "g2 超出 geom 范围"),
    ] {
        let mut meta = valid_meta();
        match field {
            "nworld" => meta.nworld = 0,
            "ngeom" => meta.ngeom = 0,
            "naconmax" => meta.naconmax = 0,
            "efc_width" => meta.efc_width = 0,
            "g1" => meta.g1 = 2,
            _ => meta.g2 = -1,
        }
        sample.meta = meta;
        assert_eq!(
            validate_inputs(&sample.inputs()),
            Err(CollisionGpuError::InvalidArgument(message)),
            "{field} 非法值未被拒绝"
        );
    }
}

#[test]
fn rejects_mismatched_slice_lengths() {
    for (name, message) in [
        ("geom_xpos", "geom_xpos 长度须为 nworld*ngeom*3"),
        ("geom_xmat", "geom_xmat 长度须为 nworld*ngeom*9"),
        ("geom_size", "geom_size 长度须为 ngeom*3"),
        ("geom_margin", "geom_margin 长度须为 ngeom"),
        ("geom_gap", "geom_gap 长度须为 ngeom"),
        ("geom_rbound", "geom_rbound 长度须为 ngeom"),
        ("geom_aabb", "geom_aabb 长度须为 ngeom*6"),
        ("geom_type", "geom_type 长度须为 ngeom"),
        ("geom_dataid", "geom_dataid 长度须为 ngeom"),
    ] {
        let mut broken = SampleInputs::new(valid_meta());
        match name {
            "geom_xpos" => {
                broken.geom_xpos.pop();
            }
            "geom_xmat" => {
                broken.geom_xmat.pop();
            }
            "geom_size" => {
                broken.geom_size.pop();
            }
            "geom_margin" => {
                broken.geom_margin.pop();
            }
            "geom_gap" => {
                broken.geom_gap.pop();
            }
            "geom_rbound" => {
                broken.geom_rbound.pop();
            }
            "geom_aabb" => {
                broken.geom_aabb.pop();
            }
            "geom_type" => {
                broken.geom_type.pop();
            }
            _ => {
                broken.geom_dataid.pop();
            }
        }
        assert_eq!(
            validate_inputs(&broken.inputs()),
            Err(CollisionGpuError::InvalidArgument(message)),
            "{name} 长度不匹配未被拒绝"
        );
    }
}

#[test]
fn empty_mesh_poly_sidecar_is_null_and_valid() {
    // ① M0a：9 项全空 → 不构造 sidecar（`mesh_poly_args = null`），validate 通过、不 panic；
    // 非 mesh 组合（SPHERE-BOX）沿用同一空 sidecar 语义（发射级验证在 M2 GPU 窗口）。
    let mut meta = valid_meta();
    meta.type1 = 2;
    meta.type2 = 6;
    let sample = SampleInputs::new(meta);
    assert_eq!(validate_inputs(&sample.inputs()), Ok(()));
    let empty = sample.inputs();
    assert!(empty.mesh_polynum.is_empty() && empty.mesh_polymap.is_empty());
}

#[test]
fn rejects_partial_mesh_poly_sidecar() {
    let sample = SampleInputs::new(valid_meta());
    let base = sample.inputs();
    let polynum = vec![0_i32];
    let partial = CollisionGpuInputs {
        mesh_polynum: &polynum,
        ..base
    };
    assert_eq!(
        validate_inputs(&partial),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 输入不完整：9 个数组必须同时提供或同时为空"
        ))
    );
}

#[path = "validate_mesh_poly_tests.rs"]
mod mesh_poly_tests;
