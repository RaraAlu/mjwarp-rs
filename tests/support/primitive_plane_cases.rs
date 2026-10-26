//! G10@P5 B1：plane 族对照测试的 case 表与 mesh 数据（`g10_p5_primitive_plane_cuda.rs`
//! 以 `#[path]` 引入；不进入产品路径、不改变任何判据/期望）。
//!
//! 抽出原因：M0a sidecar 给每个 `CollisionGpuInputs` 补 9 个 `&[]` 后主文件超 500 行 size gate；
//! 本拆分是 **test-only**，case 数据/字段值一字未改。

pub(crate) struct PlaneCase {
    pub(crate) name: &'static str,
    /// 对照 GPU 参考的 case 名（B0 JSON 的 key）
    pub(crate) object_size: [f32; 3],
    pub(crate) object_type: i32,
    /// 是否 (PLANE, MESH)：mesh 顶点（`model.mesh_vert`，4 顶点 < 10 走穷举分支）
    pub(crate) mesh_vertices: &'static [[f32; 3]],
}

/// MuJoCo 的 `model.mesh_vert`（`fixtures/g10-p5-primitive/tetra.obj` 经 `mesh_quat`
/// 旋转到 mesh 局部系后的 4 个顶点，f32；用 pinned venv 的 mujoco 3.15 从同一 fixture
/// dump，非 `tetra.obj` 的原始坐标）。设备侧 mesh 顶点取的就是这份数据，故必须用它。
const TETRA: [[f32; 3]; 4] = [
    [-0.004_515_995, 0.05, 0.042_259_388],
    [-0.004_515_995, -0.05, 0.042_259_388],
    [-0.041_932_628, 0.0, -0.050_476_775],
    [0.050_964_616, 0.0, -0.034_041_997],
];

pub(crate) const CASES: [PlaneCase; 6] = [
    PlaneCase {
        name: "plane-sphere",
        object_size: [0.1, 0.0, 0.0],
        object_type: 2,
        mesh_vertices: &[],
    },
    PlaneCase {
        name: "plane-capsule",
        object_size: [0.05, 0.12, 0.0],
        object_type: 3,
        mesh_vertices: &[],
    },
    PlaneCase {
        name: "plane-ellipsoid",
        object_size: [0.1, 0.07, 0.05],
        object_type: 4,
        mesh_vertices: &[],
    },
    PlaneCase {
        name: "plane-cylinder",
        object_size: [0.06, 0.1, 0.0],
        object_type: 5,
        mesh_vertices: &[],
    },
    PlaneCase {
        name: "plane-box",
        object_size: [0.1, 0.08, 0.07],
        object_type: 6,
        mesh_vertices: &[],
    },
    PlaneCase {
        name: "plane-mesh",
        object_size: [0.0, 0.0, 0.0],
        object_type: 7,
        mesh_vertices: &TETRA,
    },
];
