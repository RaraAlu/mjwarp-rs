//! `inputs_from_snapshot` 单元测试（A2 容量 32/32 边界；测试专用，不进产品路径）。
//!
//! A2（2026-11-12）：容量 20/10 -> 32/32；in-range 24/24 与 32/32 边界接受，
//! over-32（face=33 / degree=33）由宿主容量门给出确定性错误；`.cu` 宏一致性由
//! `host_limits_match_device_macros` 守护。

use super::{CollisionGpuError, CollisionGpuMeta, CollisionInputsSnapshot, inputs_from_snapshot};
use crate::runtime::collision_gpu::types::{
    MESH_MULTICONTACT_DEGREE_MAX, MESH_MULTICONTACT_FACE_MAX, MESH_MULTICONTACT_REJECT_PATH,
    MeshCapacityDimension,
};
const FAN_CONE_24_OBJ: &str =
    include_str!("../../../fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj");
const MULTICONTACT_GEOM_CU: &str =
    include_str!("../../physics/collision/cuda/multicontact_geom.cu");
/// 解析 OBJ 的 `v`/`f`（只关心顶点数与逐面顶点号；索引转 0 基）。
fn parse_obj(text: &str) -> (usize, Vec<Vec<usize>>) {
    let mut vertices = 0;
    let mut polygons = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with("v ") {
            vertices += 1;
        } else if let Some(rest) = line.strip_prefix("f ") {
            polygons.push(rest.split_whitespace().map(face_index).collect());
        }
    }
    (vertices, polygons)
}
fn face_index(token: &str) -> usize {
    token
        .split('/')
        .next()
        .unwrap_or(token)
        .parse::<usize>()
        .expect("OBJ face index")
        - 1
}
/// 单 mesh 的最小合法 sidecar + geom 数组（够走通 `validate_inputs` 的形状/值域校验）。
struct Fixture {
    geom_xpos: Vec<f32>,
    geom_xmat: Vec<f32>,
    geom_size: Vec<f32>,
    geom_margin: Vec<f32>,
    geom_gap: Vec<f32>,
    geom_rbound: Vec<f32>,
    geom_aabb: Vec<f32>,
    geom_type: Vec<i32>,
    geom_dataid: Vec<i32>,
    mesh_vertadr: Vec<i32>,
    mesh_vertnum: Vec<i32>,
    mesh_vert: Vec<f32>,
    mesh_polynum: Vec<i32>,
    mesh_polyadr: Vec<i32>,
    mesh_polynormal: Vec<f32>,
    mesh_polyvertadr: Vec<i32>,
    mesh_polyvertnum: Vec<i32>,
    mesh_polyvert: Vec<i32>,
    mesh_polymapadr: Vec<i32>,
    mesh_polymapnum: Vec<i32>,
    mesh_polymap: Vec<i32>,
}
impl Fixture {
    fn new(vertices: usize, polygons: &[Vec<usize>]) -> Self {
        let mut polyvertadr = Vec::new();
        let mut polyvertnum = Vec::new();
        let mut polyvert = Vec::new();
        for face in polygons {
            polyvertadr.push(polyvert.len() as i32);
            polyvertnum.push(face.len() as i32);
            polyvert.extend(face.iter().map(|vertex| *vertex as i32));
        }
        let mut polymap_counts = vec![0usize; vertices];
        for vertex in polygons.iter().flatten() {
            polymap_counts[*vertex] += 1;
        }
        let mut polymapadr = Vec::new();
        let mut polymapnum = Vec::new();
        let mut polymap = Vec::new();
        for (vertex, count) in polymap_counts.iter().enumerate() {
            polymapadr.push(polymap.len() as i32);
            polymapnum.push(*count as i32);
            for (polygon, face) in polygons.iter().enumerate() {
                if face.contains(&vertex) {
                    polymap.push(polygon as i32);
                }
            }
        }
        Self {
            geom_xpos: vec![0.0; 3],
            geom_xmat: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            geom_size: vec![0.1; 3],
            geom_margin: vec![0.0],
            geom_gap: vec![0.0],
            geom_rbound: vec![0.1],
            geom_aabb: vec![0.0; 6],
            geom_type: vec![7],
            geom_dataid: vec![0],
            mesh_vertadr: vec![0],
            mesh_vertnum: vec![vertices as i32],
            mesh_vert: vec![0.0; vertices * 3],
            mesh_polynum: vec![polygons.len() as i32],
            mesh_polyadr: vec![0],
            mesh_polynormal: vec![0.0; polygons.len() * 3],
            mesh_polyvertadr: polyvertadr,
            mesh_polyvertnum: polyvertnum,
            mesh_polyvert: polyvert,
            mesh_polymapadr: polymapadr,
            mesh_polymapnum: polymapnum,
            mesh_polymap: polymap,
        }
    }
    /// 9 项 mesh_poly 全空：无 sidecar，产品行为与改动前一致。
    fn without_sidecar() -> Self {
        let mut fixture = Self::new(0, &[]);
        fixture.mesh_polynum.clear();
        fixture.mesh_polyadr.clear();
        fixture
    }
    fn snapshot(&self) -> CollisionInputsSnapshot<'_> {
        CollisionInputsSnapshot {
            geom_xpos: &self.geom_xpos,
            geom_xmat: &self.geom_xmat,
            geom_size: &self.geom_size,
            geom_margin: &self.geom_margin,
            geom_gap: &self.geom_gap,
            geom_rbound: &self.geom_rbound,
            geom_aabb: &self.geom_aabb,
            geom_type: &self.geom_type,
            geom_dataid: &self.geom_dataid,
            mesh_vertadr: &self.mesh_vertadr,
            mesh_vertnum: &self.mesh_vertnum,
            mesh_graphadr: &[],
            mesh_octadr: &[],
            mesh_vert: &self.mesh_vert,
            mesh_graph: &[],
            mesh_polynum: &self.mesh_polynum,
            mesh_polyadr: &self.mesh_polyadr,
            mesh_polynormal: &self.mesh_polynormal,
            mesh_polyvertadr: &self.mesh_polyvertadr,
            mesh_polyvertnum: &self.mesh_polyvertnum,
            mesh_polyvert: &self.mesh_polyvert,
            mesh_polymapadr: &self.mesh_polymapadr,
            mesh_polymapnum: &self.mesh_polymapnum,
            mesh_polymap: &self.mesh_polymap,
            hfield_size: &[],
            hfield_nrow: &[],
            hfield_ncol: &[],
            hfield_adr: &[],
            hfield_data: &[],
            oct_aabb: &[],
            oct_child: &[],
            oct_coeff: &[],
        }
    }
}
fn meta() -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: 1,
        ngeom: 1,
        naconmax: 1,
        efc_width: 1,
        ..CollisionGpuMeta::default()
    }
}
/// 断言产品入口显式拒绝；返回（维度, actual, limit）列表与 Display 文本。
fn reject(fixture: &Fixture) -> (Vec<(MeshCapacityDimension, i32, i32)>, String) {
    let error = inputs_from_snapshot(&fixture.snapshot(), meta()).expect_err("产品入口应拒绝");
    let text = error.to_string();
    let CollisionGpuError::MeshMulticontactCapacity {
        violations,
        reject_path,
    } = error
    else {
        panic!("期望容量拒绝");
    };
    assert_eq!(reject_path, MESH_MULTICONTACT_REJECT_PATH);
    let list = violations
        .iter()
        .map(|v| (v.dimension, v.actual, v.limit))
        .collect();
    (list, text)
}
/// A2（2026-11-12）：容量 32/32 后 fan-cone-24（24/24）在界内 → 产品入口接受。
#[test]
fn product_entry_accepts_fan_cone_24_within_capacity_32() {
    let (vertices, polygons) = parse_obj(FAN_CONE_24_OBJ);
    assert_eq!((vertices, polygons.len()), (25, 25), "fan-cone-24 几何");
    assert!(
        inputs_from_snapshot(&Fixture::new(vertices, &polygons).snapshot(), meta()).is_ok(),
        "24/24 在 32/32 容量内，产品入口必须接受"
    );
}
/// A2：33 顶点 polygon 超过 face 容量 → 确定性容量错误（limit=32）。
#[test]
fn product_entry_rejects_face_33() {
    let mut polygons: Vec<Vec<usize>> = vec![(0..33).collect()];
    polygons.push(vec![0, 1, 2]);
    let (violations, text) = reject(&Fixture::new(33, &polygons));
    assert_eq!(
        violations,
        vec![(MeshCapacityDimension::Face, 33, MESH_MULTICONTACT_FACE_MAX)]
    );
    for needle in ["face", "actual=33", "limit=32", "拒绝路径"] {
        assert!(text.contains(needle), "Display 缺 {needle}：{text}");
    }
}
/// A2：单顶点相邻 33 个 polygon 超过 degree 容量 → 确定性容量错误（limit=32）。
#[test]
fn product_entry_rejects_degree_33() {
    // 顶点 0 被 33 个三角形共享 → polymapnum[0]=33，其余顶点度 1。
    let polygons: Vec<Vec<usize>> = (0..33)
        .map(|index| vec![1 + index, 1 + (index + 1) % 33, 0])
        .collect();
    let (violations, _) = reject(&Fixture::new(34, &polygons));
    assert_eq!(
        violations,
        vec![(
            MeshCapacityDimension::Degree,
            33,
            MESH_MULTICONTACT_DEGREE_MAX
        )]
    );
}
#[test]
fn product_entry_accepts_face_32_degree_32_boundary() {
    // 32 边形 + 32 个共享顶点 32 的三角形：模型级 max face=32、max degree=32（恰为容量上限）。
    let mut polygons: Vec<Vec<usize>> = vec![(0..32).collect()];
    for index in 0..32 {
        polygons.push(vec![index, (index + 1) % 32, 32]);
    }
    assert!(inputs_from_snapshot(&Fixture::new(33, &polygons).snapshot(), meta()).is_ok());
}
#[test]
fn product_entry_accepts_without_mesh_sidecar() {
    assert!(inputs_from_snapshot(&Fixture::without_sidecar().snapshot(), meta()).is_ok());
}
#[test]
fn host_limits_match_device_macros() {
    fn base_macro(source: &str, name: &str) -> i32 {
        source
            .lines()
            .find_map(|line| {
                let mut parts = line.split_whitespace();
                (parts.next() == Some("#define") && parts.next() == Some(name))
                    .then(|| parts.next()?.parse::<i32>().ok())
                    .flatten()
            })
            .unwrap_or_else(|| panic!("multicontact_geom.cu 缺少 {name}"))
    }
    assert_eq!(
        base_macro(MULTICONTACT_GEOM_CU, "G5_MC_MESH_FACE_MAX"),
        MESH_MULTICONTACT_FACE_MAX
    );
    assert_eq!(
        base_macro(MULTICONTACT_GEOM_CU, "G5_MC_MESH_DEG_MAX"),
        MESH_MULTICONTACT_DEGREE_MAX
    );
}
