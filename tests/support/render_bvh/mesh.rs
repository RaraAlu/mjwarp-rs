//! S3 网格 BVH 夹具解析（纯主机，不启动 Python、不依赖 cuda feature）。
//!
//! 输入：`fixtures/render-bvh/scene-a/s3/mesh_query.json`（authoring 脚本
//! `scripts/authoring/render-bvh/capture_s3_mesh.py` 在 GPU 上捕获的参考）。
//! 夹具存的是设备侧 f32 位型：初始/动态顶点、geom 位姿、逐射线 max_t 与
//! distance/normal/u/v/face。产品测试只读位型，命中判定在设备侧做。
#![allow(dead_code)]

use mjwarp_rs::render::bvh::{MeshBvh, MeshBvhInput};

pub const MESH_QUERY: &str =
    include_str!("../../../fixtures/render-bvh/scene-a/s3/mesh_query.json");
pub const MODEL_312: &str = include_str!("../../../fixtures/render-bvh/scene-a/model-312.json");

/// 单个用例的参考结果。
#[derive(Clone, Debug)]
pub struct MeshCase {
    pub name: String,
    /// 该用例使用的顶点集：`initial` 或 `dynamic`。
    pub vertices: String,
    pub geom_id: i32,
    pub cull_backfaces: bool,
    pub xpos: [f32; 3],
    pub xmat: [f32; 9],
    pub max_t: Vec<f32>,
    pub hit: Vec<bool>,
    pub distance: Vec<f32>,
    pub normal: Vec<[f32; 3]>,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub face: Vec<i32>,
}

/// 退化三角形网格的参考结果（自建夹具，与 scene-a 无关）。
pub struct DegenerateCase {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub origins: Vec<[f32; 3]>,
    pub directions: Vec<[f32; 3]>,
    pub max_t: Vec<f32>,
    pub geom_id: i32,
    pub cull_backfaces: bool,
    pub xpos: [f32; 3],
    pub xmat: [f32; 9],
    pub hit: Vec<bool>,
    pub distance: Vec<f32>,
    pub normal: Vec<[f32; 3]>,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub face: Vec<i32>,
}

/// S3 网格参考夹具。
pub struct MeshFixture {
    pub task_id: String,
    pub initial_vertices: Vec<[f32; 3]>,
    pub dynamic_vertices: Vec<[f32; 3]>,
    pub model_312_vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub s0b_count: usize,
    pub origins: Vec<[f32; 3]>,
    pub directions: Vec<[f32; 3]>,
    pub tags: Vec<String>,
    pub cases: Vec<MeshCase>,
    pub degenerate: DegenerateCase,
    pub cross_check_all_true: bool,
    /// 场景射线触发上游 f64 兜底分支的次数（应为 0，产品内核是 f32-only）。
    pub scene_degenerate_woop_queries: u64,
    /// 退化三角网格用例触发该分支的次数（允许 >0，只要判定不变）。
    pub degenerate_mesh_woop_queries: u64,
    /// f64 兜底相对纯 f32 计算的判定变化次数（必须为 0）。
    pub fallback_decision_changes: u64,
    pub determinism_identical: bool,
}

impl MeshFixture {
    /// 按名字取用例（`initial`/`moved`/`dynamic`/`tight_above`/...）。
    pub fn case(&self, name: &str) -> &MeshCase {
        self.cases
            .iter()
            .find(|case| case.name == name)
            .unwrap_or_else(|| panic!("夹具缺少用例 {name}"))
    }

    /// 用指定顶点集构建宿主三角 BVH。
    pub fn mesh(&self, vertices: &str) -> MeshBvh {
        let points = match vertices {
            "initial" => &self.initial_vertices,
            "dynamic" => &self.dynamic_vertices,
            other => panic!("未知顶点集 {other}"),
        };
        MeshBvh::build(MeshBvhInput {
            vertices: points,
            indices: &self.indices,
        })
        .expect("夹具顶点/索引应被接受")
    }

    /// 初始顶点集的三角 BVH。
    pub fn initial_mesh(&self) -> MeshBvh {
        self.mesh("initial")
    }

    /// 用例对应的宿主三角 BVH（必要时先 `set_points` + `refit`，走动态更新路径）。
    pub fn case_mesh(&self, case: &MeshCase) -> MeshBvh {
        let mut mesh = self.initial_mesh();
        if case.vertices == "dynamic" {
            mesh.set_points(&self.dynamic_vertices)
                .expect("动态顶点应被接受");
            mesh.refit();
        }
        mesh
    }
}

/// 把世界系射线映射到网格局部系（镜射上游 `_ray_map`：lpnt = R^T (p - pos)，lvec = R^T v）。
pub fn world_to_local(
    origin: [f32; 3],
    direction: [f32; 3],
    xpos: [f32; 3],
    xmat: [f32; 9],
) -> ([f32; 3], [f32; 3]) {
    let difference = [
        origin[0] - xpos[0],
        origin[1] - xpos[1],
        origin[2] - xpos[2],
    ];
    let rotate = |vector: [f32; 3]| -> [f32; 3] {
        [
            xmat[0] * vector[0] + xmat[3] * vector[1] + xmat[6] * vector[2],
            xmat[1] * vector[0] + xmat[4] * vector[1] + xmat[7] * vector[2],
            xmat[2] * vector[0] + xmat[5] * vector[1] + xmat[8] * vector[2],
        ]
    };
    (rotate(difference), rotate(direction))
}

fn numbers(value: &serde_json::Value) -> Vec<f64> {
    value
        .as_array()
        .expect("期望 JSON 数组")
        .iter()
        .map(|item| item.as_f64().expect("期望数值"))
        .collect()
}

fn ints(value: &serde_json::Value) -> Vec<i32> {
    numbers(value).iter().map(|item| *item as i32).collect()
}

fn u32s(value: &serde_json::Value) -> Vec<u32> {
    numbers(value).iter().map(|item| *item as u32).collect()
}

fn f32s(value: &serde_json::Value) -> Vec<f32> {
    u32s(value)
        .iter()
        .map(|bits| f32::from_bits(*bits))
        .collect()
}

fn points(value: &serde_json::Value) -> Vec<[f32; 3]> {
    let values = f32s(value);
    values
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect()
}

fn vector3(value: &serde_json::Value) -> [f32; 3] {
    let values = f32s(value);
    [values[0], values[1], values[2]]
}

fn matrix9(value: &serde_json::Value) -> [f32; 9] {
    let values = f32s(value);
    let mut matrix = [0.0f32; 9];
    matrix.copy_from_slice(&values[..9]);
    matrix
}

fn bools(value: &serde_json::Value) -> Vec<bool> {
    value
        .as_array()
        .expect("期望 JSON 数组")
        .iter()
        .map(|item| item.as_u64().expect("期望 0/1") != 0)
        .collect()
}

fn grouped(value: &serde_json::Value) -> Vec<[f32; 3]> {
    let values = f32s(value);
    let mut grouped = Vec::with_capacity(values.len() / 3);
    for chunk in values.chunks(3) {
        grouped.push([chunk[0], chunk[1], chunk[2]]);
    }
    grouped
}

fn mesh_case(document: &serde_json::Value, name: &str, case: &serde_json::Value) -> MeshCase {
    MeshCase {
        name: name.to_string(),
        vertices: case["vertices"]
            .as_str()
            .expect("vertices 标记")
            .to_string(),
        geom_id: case["geom_id"].as_i64().expect("geom_id") as i32,
        cull_backfaces: case["cull_backfaces"].as_u64().expect("cull_backfaces") != 0,
        xpos: vector3(&case["geom_xpos_bits"]),
        xmat: matrix9(&case["geom_xmat_bits"]),
        max_t: f32s(&case["max_t_bits"]),
        hit: bools(&case["hit"]),
        distance: f32s(&case["distance_bits"]),
        normal: grouped(&case["normal_bits"]),
        u: f32s(&case["u_bits"]),
        v: f32s(&case["v_bits"]),
        face: ints(&case["face"]),
    }
    .check(document)
}

impl MeshCase {
    fn check(self, document: &serde_json::Value) -> Self {
        let ray_count = document["rays"]["count"].as_u64().expect("ray count") as usize;
        for field in [
            ("max_t", self.max_t.len()),
            ("hit", self.hit.len()),
            ("distance", self.distance.len()),
            ("normal", self.normal.len()),
            ("u", self.u.len()),
            ("v", self.v.len()),
            ("face", self.face.len()),
        ] {
            assert_eq!(
                field.1, ray_count,
                "用例 {} 的 {} 长度不符",
                self.name, field.0
            );
        }
        self
    }
}

/// 加载夹具；JSON 结构不符会 panic（夹具是入库产物）。
pub fn load() -> MeshFixture {
    let document: serde_json::Value = serde_json::from_str(MESH_QUERY).expect("S3 夹具 JSON");
    let mesh = &document["mesh"];
    let rays = &document["rays"];
    let mut cases: Vec<MeshCase> = document["cases"]
        .as_object()
        .expect("cases 对象")
        .iter()
        .map(|(name, case)| mesh_case(&document, name, case))
        .collect();
    cases.sort_by(|left, right| left.name.cmp(&right.name));
    let degenerate = &document["degenerate"];
    let cross_check_all_true = document["cross_check_s2b"]
        .as_object()
        .expect("cross_check_s2b")
        .values()
        .filter(|value| value.is_boolean())
        .all(|value| value.as_bool().unwrap_or(false));
    let stats = &document["stats"];
    let scene_degenerate_woop_queries = stats["degenerate_woop_queries_scene"]
        .as_u64()
        .expect("degenerate_woop_queries_scene");
    let degenerate_mesh_woop_queries = stats["degenerate_woop_queries_degenerate_mesh"]
        .as_u64()
        .expect("degenerate_woop_queries_degenerate_mesh");
    let fallback_decision_changes = stats["fallback_decision_changes"]
        .as_u64()
        .expect("fallback_decision_changes");
    MeshFixture {
        task_id: document["task_id"].as_str().expect("task_id").to_string(),
        initial_vertices: points(&mesh["vertex_bits_initial"]),
        dynamic_vertices: points(&mesh["vertex_bits_dynamic"]),
        model_312_vertices: points(&mesh["model_312_vertex_bits"]),
        indices: u32s(&mesh["indices"]),
        s0b_count: rays["s0b_count"].as_u64().expect("s0b_count") as usize,
        origins: points(&rays["origin_bits"]),
        directions: points(&rays["direction_bits"]),
        tags: rays["tags"]
            .as_array()
            .expect("tags")
            .iter()
            .map(|tag| tag.as_str().expect("tag").to_string())
            .collect(),
        cases,
        degenerate: DegenerateCase {
            vertices: points(&degenerate["vertices_bits"]),
            indices: u32s(&degenerate["indices"]),
            origins: points(&degenerate["origin_bits"]),
            directions: points(&degenerate["direction_bits"]),
            max_t: f32s(&degenerate["max_t_bits"]),
            geom_id: degenerate["geom_id"].as_i64().expect("geom_id") as i32,
            cull_backfaces: degenerate["cull_backfaces"].as_u64().expect("cull") != 0,
            xpos: vector3(&degenerate["geom_xpos_bits"]),
            xmat: matrix9(&degenerate["geom_xmat_bits"]),
            hit: bools(&degenerate["hit"]),
            distance: f32s(&degenerate["distance_bits"]),
            normal: grouped(&degenerate["normal_bits"]),
            u: f32s(&degenerate["u_bits"]),
            v: f32s(&degenerate["v_bits"]),
            face: ints(&degenerate["face"]),
        },
        cross_check_all_true,
        scene_degenerate_woop_queries,
        degenerate_mesh_woop_queries,
        fallback_decision_changes,
        determinism_identical: document["determinism"]["bitwise_identical"]
            .as_bool()
            .expect("determinism"),
    }
}

/// `model-312.json` 里的 `mesh_vert`（f64，局部系顶点数组）。
pub fn model_312_mesh_vert() -> Vec<f64> {
    let document: serde_json::Value = serde_json::from_str(MODEL_312).expect("model-312 JSON");
    numbers(&document["arrays"]["mesh_vert"]["values"])
}
