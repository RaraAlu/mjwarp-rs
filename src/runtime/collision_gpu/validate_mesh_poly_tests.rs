//! M3/O7：mesh polygon sidecar 的 `validate_mesh_poly` 负例/正例（自 `validate_tests.rs` 纯搬移）。
//!
//! 仅为满足单文件 500 行上限：仅模块挂接（`#[path] mod`）、imports 与文件位置变化；
//! 测试名、断言文本与数据块保持不变。
//!
//! 覆盖：真实 cube 形状正例、9 类形状/区间/负值负例、sum 恒等式负例、O7 值域 6 条负例 + 正例。

use super::super::abi::validate_inputs;
use super::super::{CollisionGpuError, CollisionGpuInputs};
use super::{SampleInputs, valid_meta};

/// 真实 cube 的 9 项 mesh polygon 数组（与 pinned `mujoco 3.15` 的 mjModel 逐值一致，
/// 形状按 pinned `mujoco_warp/_src/types.py` L1329-1337/L1864-1872：
/// `nmesh=1 / nmeshpoly=6 / nmeshvert=8 / nmeshpolyvert=24 / nmeshpolymap=24`）。
/// M3/O7：同时带逐 mesh 的 `mesh_vertadr`/`mesh_vertnum`（值域校验的前提）。
struct CubeMeshPoly {
    vertadr: Vec<i32>,
    vertnum: Vec<i32>,
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
            vertadr: vec![0],
            vertnum: vec![8],
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
            mesh_vertadr: &self.vertadr,
            mesh_vertnum: &self.vertnum,
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

#[test]
fn rejects_mesh_poly_value_domain_violations() {
    // O7（M3）：`polyvert`/`polymap` 元素在设备侧是「逐 geom 偏移 + 局部索引」的索引，
    // 必须在 CPU 校验层拒绝越界值（M0a 之前只校验长度/区间/求和）。
    let sample = SampleInputs::new(valid_meta());
    let base = sample.inputs();
    let cube = CubeMeshPoly::cube();

    // ① mesh_vertadr/mesh_vertnum 长度须为 nmesh（逐 mesh 前提）。
    let mut broken = CubeMeshPoly::cube();
    broken.vertadr.clear();
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_vertadr/mesh_vertnum 长度须为 nmesh（逐 mesh）"
        ))
    );

    // ② 逐 mesh 顶点区间越界（vertadr+vertnum > nmeshvert）。
    let mut broken = CubeMeshPoly::cube();
    broken.vertadr = vec![1];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_vertadr/mesh_vertnum 区间超出 mesh 顶点数"
        ))
    );

    // ③ 负的 vertnum。
    let mut broken = CubeMeshPoly::cube();
    broken.vertnum = vec![-1];
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_vertadr/mesh_vertnum 不得为负"
        ))
    );

    // ④ polyvert 元素超出所属 mesh 顶点数（8 号顶点不存在）。
    let mut broken = CubeMeshPoly::cube();
    broken.polyvert[0] = 8;
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polyvert 元素超出所属 mesh 的顶点数"
        ))
    );

    // ⑤ polymap 元素超出所属 mesh polygon 数（第 6 个 polygon 不存在）。
    let mut broken = CubeMeshPoly::cube();
    broken.polymap[0] = 6;
    assert_eq!(
        validate_inputs(&broken.over(base)),
        Err(CollisionGpuError::InvalidArgument(
            "mesh_polymap 元素超出所属 mesh 的 polygon 数"
        ))
    );

    // ⑥ mesh geom 的 dataid 超出 sidecar 覆盖的 mesh 数。
    let mesh_types = vec![7_i32, 2];
    let mesh_dataid = vec![1_i32, -1];
    let inputs = CollisionGpuInputs {
        geom_type: &mesh_types,
        geom_dataid: &mesh_dataid,
        ..cube.over(base)
    };
    assert_eq!(
        validate_inputs(&inputs),
        Err(CollisionGpuError::InvalidArgument(
            "mesh geom 的 geom_dataid 超出 mesh 编号范围"
        ))
    );

    // ⑦ 本身自洽（真实 cube + 逐 mesh vertadr/vertnum）必须通过。
    assert_eq!(validate_inputs(&cube.over(base)), Ok(()));
}
