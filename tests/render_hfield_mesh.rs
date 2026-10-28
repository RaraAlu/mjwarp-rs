//! R05-S2 hfield 三角化单测（逐条钉住冻结上游 `bvh.py` 的顶点/索引顺序与合并规则）。
//!
//! 上游锚点（MJWarp 3.15，`71da24d`）：
//! - `bvh.py::_optimize_hfield_mesh` L453-578：`get_point_index` 去重顺序、
//!   非共面单格 `TL,TR,BR` + `TL,BR,BL`、共面贪心扩展（先宽后高）、合并矩形同序；
//! - `bvh.py::build_hfield_bvh` L581-622：`width/height` 公式、`half = 0.5*(pmax-pmin)`。
//!
//! 本文件只验证宿主几何（顶点/索引/半尺寸/BVH 构建）与 S2 槽语义
//! （blob 内 mesh 索引 / `u64::MAX` 哨兵 / 未注入拒绝），**零渲染数值结论**。

use mjwarp_rs::render::bvh::{BvhError, HfieldMeshInput, build_hfield_mesh, triangulate_hfield};
use mjwarp_rs::render::kernel::cast::{HfieldDevice, ModelRenderArrays};
use mjwarp_rs::render::{HfieldContextInput, HfieldInputs, resolve_hfield};
use mjwarp_rs::runtime::launcher::LauncherError;

const SIZE: [f32; 3] = [2.0, 4.0, 0.5];

fn input<'a>(nrow: i32, ncol: i32, data: &'a [f32]) -> HfieldMeshInput<'a> {
    HfieldMeshInput {
        nrow,
        ncol,
        size: SIZE,
        data,
    }
}

#[test]
fn flat_two_by_two_emits_two_triangles_in_upstream_order() {
    // 2×2 一个格：`width = 0.5*max(nc-1,1) = 0.5`、`height = 0.5`；
    // x = sx*(c/width-1) → c=0:-2、c=1:2；y = sy*(r/height-1) → r=0:-4、r=1:4。
    let data = [0.0f32; 4];
    let (points, indices) = triangulate_hfield(&input(2, 2, &data)).unwrap();
    assert_eq!(
        points,
        vec![
            [-2.0, -4.0, 0.0],
            [2.0, -4.0, 0.0],
            [-2.0, 4.0, 0.0],
            [2.0, 4.0, 0.0],
        ]
    );
    // L510-512 / L574-576：TL,TR,BR + TL,BR,BL。
    assert_eq!(indices, vec![0, 1, 3, 0, 3, 2]);
}

#[test]
fn non_planar_cell_emits_single_cell_mesh() {
    // z00+z11=1 vs z01+z10=0 → 非共面（L499-500）→ 单格两三角。
    let data = [0.0f32, 0.0, 0.0, 1.0];
    let (points, indices) = triangulate_hfield(&input(2, 2, &data)).unwrap();
    assert_eq!(points[3], [2.0, 4.0, 0.5]);
    assert_eq!(indices, vec![0, 1, 3, 0, 3, 2]);
}

#[test]
fn planar_ramp_merges_whole_grid_into_one_quad() {
    // data[r,c] = r + 2c 对任意格都共面（z00+z11 == z01+z10）且斜率一致 → 合并 2×2 格。
    let mut data = Vec::new();
    for r in 0..3 {
        for c in 0..3 {
            data.push((r + 2 * c) as f32);
        }
    }
    let (points, indices) = triangulate_hfield(&input(3, 3, &data)).unwrap();
    // width = height = 1.0；点顺序 TL,TR,BL,BR（L568-571）；z = data*sz2。
    assert_eq!(
        points,
        vec![
            [-2.0, -4.0, 0.0],
            [2.0, -4.0, 2.0],
            [-2.0, 4.0, 1.0],
            [2.0, 4.0, 3.0],
        ]
    );
    assert_eq!(indices, vec![0, 1, 3, 0, 3, 2]);
}

#[test]
fn greedy_merge_stops_at_first_non_planar_neighbour() {
    // 2×3：格 (0,0) 共面但 (0,1) 非共面 → (0,0) 单格合并；(0,1) 单格输出。
    // 顶点去重顺序：先 (0,0)(0,1)(1,0)(1,1)，再补 (0,2)(1,2)。
    let data = [0.0f32, 0.0, 0.0, 0.0, 0.0, 1.0];
    let (points, indices) = triangulate_hfield(&input(2, 3, &data)).unwrap();
    assert_eq!(points.len(), 6);
    assert_eq!(points[0], [-2.0, -4.0, 0.0]);
    assert_eq!(points[1], [0.0, -4.0, 0.0]);
    assert_eq!(points[2], [-2.0, 4.0, 0.0]);
    assert_eq!(points[3], [0.0, 4.0, 0.0]);
    assert_eq!(points[4], [2.0, -4.0, 0.0]);
    assert_eq!(points[5], [2.0, 4.0, 0.5]);
    assert_eq!(indices, vec![0, 1, 3, 0, 3, 2, 1, 4, 5, 1, 5, 3]);
}

#[test]
fn half_extents_use_point_min_max_and_nonzero_size_z() {
    let mut data = Vec::new();
    for r in 0..3 {
        for c in 0..3 {
            data.push((r + 2 * c) as f32);
        }
    }
    let mesh = build_hfield_mesh(&input(3, 3, &data)).unwrap();
    // x: 0.5*(2-(-2))=2；y: 0.5*(4-(-4))=4；z: 0.5*(3-0)=1.5（非零 size[2]）。
    assert_eq!(mesh.half(), [2.0, 4.0, 1.5]);
    assert_eq!(mesh.vertices().len(), 4);
    assert_eq!(mesh.bvh().triangle_count(), 2);
    assert_eq!(mesh.indices(), &[0, 1, 3, 0, 3, 2]);
}

#[test]
fn degenerate_and_malformed_inputs_are_readable_errors() {
    assert_eq!(
        triangulate_hfield(&input(0, 2, &[])),
        Err(BvhError::HfieldDimensions { nrow: 0, ncol: 2 })
    );
    assert_eq!(
        triangulate_hfield(&input(2, -1, &[])),
        Err(BvhError::HfieldDimensions { nrow: 2, ncol: -1 })
    );
    assert_eq!(
        triangulate_hfield(&input(2, 3, &[0.0, 0.0])),
        Err(BvhError::HfieldDataLength {
            expected: 6,
            actual: 2,
        })
    );
    // nrow<2 或 ncol<2：上游无格可遍历（`np.min` 空数组会失败），这里给可读错误。
    assert_eq!(
        triangulate_hfield(&input(1, 3, &[0.0, 0.0, 0.0])),
        Err(BvhError::HfieldEmptyMesh { nrow: 1, ncol: 3 })
    );
    assert_eq!(
        triangulate_hfield(&input(3, 1, &[0.0, 0.0, 0.0])),
        Err(BvhError::HfieldEmptyMesh { nrow: 3, ncol: 1 })
    );
}

#[test]
fn non_finite_height_maps_to_mesh_bvh_error() {
    let data = [0.0f32, f32::NAN, 0.0, 0.0];
    assert!(matches!(
        build_hfield_mesh(&input(2, 2, &data)),
        Err(BvhError::HfieldMesh(_))
    ));
}

// ---- S2 槽语义（blob 内 mesh 索引 / u64::MAX 哨兵 / 未注入拒绝；宿主侧，无 GPU） ----

fn owned_context(
    size: &[f32],
    nrow: &[i32],
    ncol: &[i32],
    adr: &[i32],
    data: &[f32],
) -> HfieldContextInput {
    let inputs = HfieldInputs {
        size: Some(size),
        nrow: Some(nrow),
        ncol: Some(ncol),
        adr: Some(adr),
        data: Some(data),
    };
    let params = resolve_hfield(&inputs).unwrap().unwrap();
    HfieldContextInput::from_params(&params).unwrap()
}

fn model(geom_type: Vec<i32>, geom_dataid: Vec<i32>) -> ModelRenderArrays {
    let ngeom = geom_type.len();
    ModelRenderArrays {
        geom_type,
        geom_rgba: vec![1.0f32; 4 * ngeom],
        geom_matid: vec![-1; ngeom],
        mat_rgba: vec![1.0f32; 4],
        geom_dataid,
    }
}

#[test]
fn hfield_device_appends_after_existing_meshes_and_uses_blob_indices() {
    let data = [0.0f32; 4];
    let context = owned_context(&[2.0, 4.0, 0.5, 0.0], &[2], &[2], &[0], &data);
    let model = model(vec![6, 1], vec![-1, 0]);
    let device = HfieldDevice::build(Some(&context), &model, 2)
        .unwrap()
        .unwrap();
    // mesh blob = 既有 2 个 mesh + hfield mesh 追加在末尾 → 槽值 2。
    assert_eq!(device.meshes().len(), 1);
    assert_eq!(device.bvh_ids(), &[2u64]);
    assert_eq!(device.meshes()[0].half(), [2.0, 4.0, 0.0]);
}

#[test]
fn hfield_device_fills_sentinel_for_unreferenced_hfields() {
    // nhfield=2，只引用 hfield id 1：id 0 槽位是 u64::MAX 哨兵。
    let data = [0.0f32; 4];
    let context = owned_context(
        &[2.0, 4.0, 0.5, 0.0, 2.0, 4.0, 0.5, 0.0],
        &[2, 2],
        &[2, 2],
        &[0, 4],
        &[data, data].concat(),
    );
    let model = model(vec![1], vec![1]);
    let device = HfieldDevice::build(Some(&context), &model, 3)
        .unwrap()
        .unwrap();
    assert_eq!(device.bvh_ids(), &[u64::MAX, 3u64]);
}

#[test]
fn hfield_device_requires_injection_when_model_has_hfield() {
    let model = model(vec![1], vec![0]);
    assert!(matches!(
        HfieldDevice::build(None, &model, 0),
        Err(LauncherError::Unsupported(_))
    ));
}

#[test]
fn hfield_device_is_none_without_hfield_geoms() {
    let data = [0.0f32; 4];
    let context = owned_context(&[2.0, 4.0, 0.5, 0.0], &[2], &[2], &[0], &data);
    let model = model(vec![6, 7], vec![-1, 0]);
    assert!(
        HfieldDevice::build(Some(&context), &model, 1)
            .unwrap()
            .is_none()
    );
}

#[test]
fn hfield_device_rejects_out_of_range_hfield_id() {
    let data = [0.0f32; 4];
    let context = owned_context(&[2.0, 4.0, 0.5, 0.0], &[2], &[2], &[0], &data);
    let model = model(vec![1], vec![7]);
    assert!(matches!(
        HfieldDevice::build(Some(&context), &model, 0),
        Err(LauncherError::InvalidArray(_))
    ));
}
