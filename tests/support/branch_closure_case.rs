//! S4 · UNC-3 分支收口：branch 验收组 9 case + limits 记录组 2 case 的输入定义（test-only）。
//!
//! 数据来源（只读）：
//! - fixture：S3 落盘的 13 个文件（sha256 清单见本文件 `FIXTURE_SHA256`，由 CPU 测试复核）；
//! - 参考：S3 `upstream-branch-closure-gpu.json` / `upstream-mesh-limits-gpu.json` 的逐 case
//!   `qpos` / `geom_xpos` / `geom_xmat`（target 运行时读取 JSON，不运行 Python/muJoCo）；
//! - mesh 拓扑：`branch_closure_mesh_data.rs`（pinned mujoco 3.15 解析同一批 fixture 的
//!   `mjModel` 字段；authoring-only 生成，生成脚本未入库，证据记录其与 fixture 的同源性）。
//!
//! 本文件不进入产品路径、不改任何判据/容差/参考/期望值。

use crate::branch_closure_mesh_data::{MeshData, U6_MESH, U7_MESH, U8_MESH};
use mjwarp_rs::runtime::collision_gpu::{CollisionGpuInputs, CollisionGpuMeta};

/// 设备入口：plane 族走 `Primitive`；(MESH, MESH)/(MESH, BOX) 走 `ContactMulti`。
#[derive(Debug)]
pub(crate) enum Kernel {
    Primitive,
    ContactMulti,
}

pub(crate) struct Case {
    pub(crate) name: &'static str,
    pub(crate) kernel: Kernel,
    /// 调用方类型序（与 S3 捕获 `geom=[0,1]` 一致；`type1`/`type2` 逐 case 写死）。
    pub(crate) geom_type: [i32; 2],
    /// 两个 geom 的 `geom_size`（来自 S3 fixture 的 `mjModel.geom_size`，f32 逐值）。
    pub(crate) geom_size: [[f32; 3]; 2],
    pub(crate) geom_dataid: [i32; 2],
    /// mesh sidecar 数据（None = 9 个 mesh_poly 数组全空、sidecar 不构造）。
    pub(crate) mesh: Option<&'static MeshData>,
}

/// branch 验收组（U1/U3/U4/U5/U6）：进 pass/fail 分母。
pub(crate) const BRANCH_CASES: [Case; 9] = [
    Case {
        name: "plane-cylinder-degenerate",
        kernel: Kernel::Primitive,
        geom_type: [0, 5],
        geom_size: [[1.0, 1.0, 0.1], [0.06, 0.1, 0.0]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-box-corner-face-down",
        kernel: Kernel::Primitive,
        geom_type: [0, 6],
        geom_size: [[1.0, 1.0, 0.1], [0.1, 0.08, 0.07]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-box-corner-face-up",
        kernel: Kernel::Primitive,
        geom_type: [0, 6],
        geom_size: [[1.0, 1.0, 0.1], [0.1, 0.08, 0.07]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-box-corner-tilt-x",
        kernel: Kernel::Primitive,
        geom_type: [0, 6],
        geom_size: [[1.0, 1.0, 0.1], [0.1, 0.08, 0.07]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-box-corner-tilt-y",
        kernel: Kernel::Primitive,
        geom_type: [0, 6],
        geom_size: [[1.0, 1.0, 0.1], [0.1, 0.08, 0.07]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-box-corner-single",
        kernel: Kernel::Primitive,
        geom_type: [0, 6],
        geom_size: [[1.0, 1.0, 0.1], [0.1, 0.08, 0.07]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-capsule-lying",
        kernel: Kernel::Primitive,
        geom_type: [0, 3],
        geom_size: [[1.0, 1.0, 0.1], [0.05, 0.12, 0.0]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "plane-cylinder-deep-tilt",
        kernel: Kernel::Primitive,
        geom_type: [0, 5],
        geom_size: [[1.0, 1.0, 0.1], [0.06, 0.1, 0.0]],
        geom_dataid: [-1, -1],
        mesh: None,
    },
    Case {
        name: "mesh-mesh-two-assets",
        kernel: Kernel::ContactMulti,
        geom_type: [7, 7],
        geom_size: [[0.1, 0.1, 0.1], [0.050964616, 0.05, 0.050476774]],
        geom_dataid: [0, 1],
        mesh: Some(&U6_MESH),
    },
];

/// limits 记录组（U7/U8）：**record-only**，不参与 pass/fail 断言。
pub(crate) const LIMIT_CASES: [Case; 2] = [
    Case {
        name: "mesh-limits-cone-box",
        kernel: Kernel::ContactMulti,
        geom_type: [7, 6],
        geom_size: [[0.075, 0.08, 0.08], [0.05, 0.05, 0.05]],
        geom_dataid: [0, -1],
        mesh: Some(&U7_MESH),
    },
    Case {
        name: "mesh-limits-prism-box",
        kernel: Kernel::ContactMulti,
        geom_type: [7, 6],
        geom_size: [[0.05, 0.08, 0.08], [0.05, 0.05, 0.05]],
        geom_dataid: [0, -1],
        mesh: Some(&U8_MESH),
    },
];

/// S3 签名集合里的 13 个 fixture（路径 + sha256；CPU 测试逐项复核）。
pub(crate) const FIXTURE_SHA256: [(&str, &str); 15] = [
    (
        "fixtures/g10-p5-primitive-branches/plane-cylinder-degenerate.xml",
        "393ff1af9811b9badf6e9c13e707fa9a843e2b1d6f3face4f98b2cc7c6157583",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-box-corner-face-down.xml",
        "b158ce87c6fd68b79306823cd23f0e778482c425b5f617fcfbbd67fba9b22cab",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-box-corner-face-up.xml",
        "0223d2a925fde62ca114b57b3baf797b40654de25d74f943e9d9f716ed7c70b7",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-box-corner-tilt-x.xml",
        "2d9d360c6c87b53fd7d7a03d8889bdceb947c15982d2062052218efe4dbe3e8f",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-box-corner-tilt-y.xml",
        "159df60fc97aa53beb2df325fdebcc9327d22bbdef354f4d01bece8e460be8c5",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-box-corner-single.xml",
        "fc9f710c9accb679f65b98d6484caeaab91d2e05c5f7c3f59f03bd04691ecc1c",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-capsule-lying.xml",
        "9d43b44e8cd53a214bf43aa4958980e764ad8f87c08c74cd7d788879b58cf5c2",
    ),
    (
        "fixtures/g10-p5-primitive-branches/plane-cylinder-deep-tilt.xml",
        "9801a8869fbb67eb2ca401956b66e1620d3e1b92f849ac6754262905fcfc6d84",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml",
        "1d71151364f4ba910308c80a42817f7a9b7fe843e42afb8396a3b5277f1b3bfb",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/fan-cone-24.obj",
        "4bc34a51a60d76469a68c64e6132cfd1f5713072488ea8deefd44a4ca517f73b",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/prism-20.obj",
        "a4477d3b080479e7356ac14bcd67c33272b2771eadd40b14fe3f40360e92cee4",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/mesh-limits-cone-box.xml",
        "6ce3552f2cab48fdbbb66a6d397790d18748dcb0a5a87b91186b5742e297f731",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/mesh-limits-prism-box.xml",
        "1bee35cfa3378d96d18317ff75bbf4af0561f7bd71915574e2ea596bc8d4f3b5",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/fan-cone-33.obj",
        "589080586fac53cca91ff6d1355b4e92c95b4a83bd512caf4ced33546444ee0b",
    ),
    (
        "fixtures/g10-p5-mesh-multicontact/prism-33.obj",
        "bb1b05c276b3f78a0bf680ff64682a4abad72233b880f1ed19c14f060677c401",
    ),
];

// ---- 设备输入装配（自 target 纯搬移，保持测试可读性且 target <=500 行）----

/// 参考 JSON 的 `geom_xpos`/`geom_xmat`（形状 `[nworld][ngeom][3/9]`）→ f32 平铺。
fn flatten_geom(value: &serde_json::Value) -> Vec<f32> {
    let mut out = Vec::new();
    for world in value.as_array().expect("nworld") {
        for geom in world.as_array().expect("ngeom") {
            for cell in geom.as_array().expect("cells") {
                out.push(cell.as_f64().expect("f32") as f32);
            }
        }
    }
    out
}

fn build_meta(case: &Case) -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: 1,
        ngeom: 2,
        g1: 0,
        g2: 1,
        type1: case.geom_type[0],
        type2: case.geom_type[1],
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

/// 一个 case 的 32 项设备输入（拥有所有权；`inputs()` 借出 slice）。
pub(crate) struct CaseInputs {
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
    mesh_graphadr: Vec<i32>,
    mesh_octadr: Vec<i32>,
    mesh_vert: Vec<f32>,
    mesh_graph: Vec<i32>,
    mesh_polynum: Vec<i32>,
    mesh_polyadr: Vec<i32>,
    mesh_polynormal: Vec<f32>,
    mesh_polyvertadr: Vec<i32>,
    mesh_polyvertnum: Vec<i32>,
    mesh_polyvert: Vec<i32>,
    mesh_polymapadr: Vec<i32>,
    mesh_polymapnum: Vec<i32>,
    mesh_polymap: Vec<i32>,
    meta: CollisionGpuMeta,
}

impl CaseInputs {
    /// 按 S3 参考 JSON 的 `output.geom_xpos`/`geom_xmat` + case 表 + mesh 拓扑装配 32 项输入。
    pub(crate) fn build(case: &Case, reference: &serde_json::Value) -> Self {
        let mut geom_size = Vec::with_capacity(6);
        for size in &case.geom_size {
            geom_size.extend_from_slice(size);
        }
        let mut inputs = Self {
            geom_xpos: flatten_geom(&reference["output"]["geom_xpos"]),
            geom_xmat: flatten_geom(&reference["output"]["geom_xmat"]),
            geom_size,
            geom_margin: vec![0.0; 2],
            geom_gap: vec![0.0; 2],
            geom_rbound: vec![0.0; 2],
            geom_aabb: vec![0.0; 12],
            geom_type: case.geom_type.to_vec(),
            geom_dataid: case.geom_dataid.to_vec(),
            mesh_vertadr: Vec::new(),
            mesh_vertnum: Vec::new(),
            mesh_graphadr: Vec::new(),
            mesh_octadr: Vec::new(),
            mesh_vert: Vec::new(),
            mesh_graph: Vec::new(),
            mesh_polynum: Vec::new(),
            mesh_polyadr: Vec::new(),
            mesh_polynormal: Vec::new(),
            mesh_polyvertadr: Vec::new(),
            mesh_polyvertnum: Vec::new(),
            mesh_polyvert: Vec::new(),
            mesh_polymapadr: Vec::new(),
            mesh_polymapnum: Vec::new(),
            mesh_polymap: Vec::new(),
            meta: build_meta(case),
        };
        if let Some(mesh) = case.mesh {
            inputs.mesh_vertadr = mesh.vertadr.to_vec();
            inputs.mesh_vertnum = mesh.vertnum.to_vec();
            inputs.mesh_graphadr = mesh.graphadr.to_vec();
            inputs.mesh_octadr = vec![-1; mesh.vertadr.len()];
            inputs.mesh_vert = mesh.vert.to_vec();
            inputs.mesh_graph = mesh.graph.to_vec();
            inputs.mesh_polynum = mesh.polynum.to_vec();
            inputs.mesh_polyadr = mesh.polyadr.to_vec();
            inputs.mesh_polynormal = mesh.polynormal.to_vec();
            inputs.mesh_polyvertadr = mesh.polyvertadr.to_vec();
            inputs.mesh_polyvertnum = mesh.polyvertnum.to_vec();
            inputs.mesh_polyvert = mesh.polyvert.to_vec();
            inputs.mesh_polymapadr = mesh.polymapadr.to_vec();
            inputs.mesh_polymapnum = mesh.polymapnum.to_vec();
            inputs.mesh_polymap = mesh.polymap.to_vec();
        }
        inputs
    }

    pub(crate) fn inputs(&self) -> CollisionGpuInputs<'_> {
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
            mesh_vertadr: &self.mesh_vertadr,
            mesh_vertnum: &self.mesh_vertnum,
            mesh_graphadr: &self.mesh_graphadr,
            mesh_octadr: &self.mesh_octadr,
            mesh_vert: &self.mesh_vert,
            mesh_graph: &self.mesh_graph,
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
            meta: self.meta,
        }
    }
}
