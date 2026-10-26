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

/// 真实 cube 的 9 项 mesh polygon 数组（与 pinned `mujoco 3.15` 的 mjModel 逐值一致，
/// 形状按 pinned `mujoco_warp/_src/types.py` L1329-1337/L1864-1872：
/// `nmesh=1 / nmeshpoly=6 / nmeshvert=8 / nmeshpolyvert=24 / nmeshpolymap=24`）。
struct CubeMeshPoly {
    polynum: Vec<i32>,
    polyadr: Vec<i32>,
    polynormal: Vec<f32>,
    polyvertadr: Vec<i32>,
    polyvertnum: Vec<i32>,
    polyvert: Vec<i32>,
    polymapadr: Vec<i32>,
    polymapnum: Vec<i32>,
    polymap: Vec<i32>,
}

impl CubeMeshPoly {
    fn cube() -> Self {
        Self {
            polynum: vec![6],
            polyadr: vec![0],
            polynormal: vec![
                1.0, 0.0, -0.0, 0.0, 0.0, -1.0, 0.0, -0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, -0.0,
                0.0, -1.0, 0.0,
            ],
            polyvertadr: vec![0, 4, 8, 12, 16, 20],
            polyvertnum: vec![4, 4, 4, 4, 4, 4],
            polyvert: vec![
                2, 6, 5, 1, 3, 2, 1, 0, 5, 6, 7, 4, 6, 2, 3, 7, 7, 3, 0, 4, 1, 5, 4, 0,
            ],
            polymapadr: vec![0, 3, 6, 9, 12, 15, 18, 21],
            polymapnum: vec![3, 3, 3, 3, 3, 3, 3, 3],
            polymap: vec![
                1, 4, 5, 0, 1, 5, 0, 1, 3, 1, 3, 4, 2, 4, 5, 0, 2, 5, 0, 2, 3, 2, 3, 4,
            ],
        }
    }

    fn over<'a>(&'a self, base: CollisionGpuInputs<'a>) -> CollisionGpuInputs<'a> {
        CollisionGpuInputs {
            mesh_polynum: &self.polynum,
            mesh_polyadr: &self.polyadr,
            mesh_polynormal: &self.polynormal,
            mesh_polyvertadr: &self.polyvertadr,
            mesh_polyvertnum: &self.polyvertnum,
            mesh_polyvert: &self.polyvert,
            mesh_polymapadr: &self.polymapadr,
            mesh_polymapnum: &self.polymapnum,
            mesh_polymap: &self.polymap,
            ..base
        }
    }
}

#[test]
fn accepts_real_cube_mesh_poly_shapes() {
    // M0a 修正：真实数组形状是 polynum/polyadr=(nmesh,)、polyvertadr/polyvertnum=(nmeshpoly,)、
    // polymapadr/polymapnum=(nmeshvert,)；此用例是「旧规则（逐 mesh 等长）会误拒」的回归。
    let sample = SampleInputs::new(valid_meta());
    let cube = CubeMeshPoly::cube();
    assert_eq!(validate_inputs(&cube.over(sample.inputs())), Ok(()));
    let inputs = cube.over(sample.inputs());
    assert_eq!(inputs.mesh_polynum.len(), 1);
    assert_eq!(inputs.mesh_polyvertadr.len(), 6);
    assert_eq!(inputs.mesh_polymapadr.len(), 8);
}

#[test]
fn rejects_inconsistent_mesh_poly_sidecar() {
    let sample = SampleInputs::new(valid_meta());
    let base = sample.inputs();
    let cube = CubeMeshPoly::cube();

    // ① polynormal 非 3 的倍数。
    let mut broken = CubeMeshPoly::cube();
    broken.polynormal = vec![0.0_f32; 4];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polynormal 长度须为 3 的倍数"
        ))
    );

    // ② 逐 mesh 数组长度不一致（polynum/polyadr）。
    let mut broken = CubeMeshPoly::cube();
    broken.polyadr = vec![0, 0];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 逐 mesh 数组长度须一致（polynum/polyadr）"
        ))
    );

    // ③ 逐 polygon 数组长度须为 nmeshpoly（polyvertadr/polyvertnum）。
    let mut broken = CubeMeshPoly::cube();
    broken.polyvertadr.pop();
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 逐 polygon 数组长度须为 nmeshpoly（polyvertadr/polyvertnum）"
        ))
    );

    // ④ 逐 vertex 数组长度须一致（polymapadr/polymapnum）。
    let mut broken = CubeMeshPoly::cube();
    broken.polymapnum.pop();
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 逐 vertex 数组长度须一致（polymapadr/polymapnum）"
        ))
    );

    // ⑤ 区间越界：逐 mesh polyadr+polynum 超出 nmeshpoly。
    let mut broken = CubeMeshPoly::cube();
    broken.polyadr = vec![1];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_poly 区间超出 mesh_polynormal 行数"
        ))
    );

    // ⑥ 区间越界：逐 polygon polyvertadr+polyvertnum 超出 mesh_polyvert。
    let mut broken = CubeMeshPoly::cube();
    broken.polyvert.pop();
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument("mesh_polyvert 区间越界"))
    );

    // ⑦ 区间越界：逐 vertex polymapadr+polymapnum 超出 mesh_polymap。
    let mut broken = CubeMeshPoly::cube();
    broken.polymap.pop();
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument("mesh_polymap 区间越界"))
    );

    // ⑧ 负值拒绝。
    let mut broken = CubeMeshPoly::cube();
    broken.polynum = vec![-1];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polyadr/mesh_polynum 不得为负"
        ))
    );

    // ⑨ 本身自洽（照抄真实 cube）应通过，防止负例把正常数据也拒掉。
    assert_eq!(validate_inputs(&cube.over(base)), Ok(()));
}

#[test]
fn rejects_inconsistent_mesh_poly_sums() {
    let sample = SampleInputs::new(valid_meta());
    let base = sample.inputs();

    // ① sum(polynum) 与 polynormal 行数不一致。
    let mut broken = CubeMeshPoly::cube();
    broken.polynum = vec![5];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polynum 之和须等于 mesh_polynormal 行数"
        ))
    );

    // ② sum(polyvertnum) 与 mesh_polyvert 长度不一致。
    let mut broken = CubeMeshPoly::cube();
    broken.polyvertnum = vec![3, 4, 4, 4, 4, 4];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polyvertnum 之和须等于 mesh_polyvert 长度"
        ))
    );

    // ③ sum(polymapnum) 与 mesh_polymap 长度不一致。
    let mut broken = CubeMeshPoly::cube();
    broken.polymapnum = vec![3, 3, 3, 3, 3, 3, 3, 2];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polymapnum 之和须等于 mesh_polymap 长度"
        ))
    );
}
