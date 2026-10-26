//! stage-2 等价性窗口输入侧的**G2 静态输入合成**（被 `tests/stage2_equivalence_window.rs` 以 `#[path]` 引入）。
//!
//! 只覆盖 `convex-mesh`/`degenerate-mesh` 两例：它们没有 G5 冻结设备输入 bin，复用
//! `src/physics/collision/gjk/case_data.rs` 的 G2 静态输入（commit 内数据，不新捕获），
//! 合成设备上传数组 + 宿主 `ccd_pair` 输入，保证两侧输入逐位相同。

#![allow(dead_code)]
#![allow(clippy::chunks_exact_to_as_chunks)]

use mjwarp_rs::runtime::collision_gpu::CollisionGpuMeta;

use crate::cases::{OwnedInputs, bins_dir, build_meta, read_f32, read_meta, root};

/// G2 静态输入（`case_data.rs` 的 `CASE_DATA`）：运行时只读该源文件并解析，不改写。
pub fn g2_case_data() -> serde_json::Value {
    let source = std::fs::read_to_string(root().join("src/physics/collision/gjk/case_data.rs"))
        .expect("读 case_data.rs");
    let start = source.find("r#\"").expect("CASE_DATA 原始串起始") + 3;
    let end = source[start..].find("\"#;").expect("CASE_DATA 原始串结束") + start;
    serde_json::from_str(&source[start..end]).expect("解析 G2 CASE_DATA")
}

fn g2_case<'a>(data: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    data["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|case| case["name"] == name)
        .unwrap_or_else(|| panic!("G2 静态输入缺少 {name}"))
}

fn f32_at(value: &serde_json::Value) -> f32 {
    value.as_f64().expect("f32") as f32
}

fn vec3(value: &serde_json::Value) -> [f32; 3] {
    let array = value.as_array().expect("vec3");
    [f32_at(&array[0]), f32_at(&array[1]), f32_at(&array[2])]
}

fn vec9(value: &serde_json::Value) -> [f32; 9] {
    let array = value.as_array().expect("vec9");
    let mut out = [0.0_f32; 9];
    for (slot, value) in out.iter_mut().zip(array) {
        *slot = f32_at(value);
    }
    out
}

/// G2 静态输入的 mesh 拓扑（host `MeshView` 用）。`mesh-mesh`/`convex-mesh` 共享同一
/// 4 顶点三角网格；`degenerate-mesh` 为 8 顶点网格。
pub struct MeshTopology {
    pub vert: Vec<[f32; 3]>,
    pub polynormal: Vec<[f32; 3]>,
    pub polyvertadr: Vec<i32>,
    pub polyvertnum: Vec<i32>,
    pub polyvert: Vec<i32>,
    pub polymapadr: Vec<i32>,
    pub polymapnum: Vec<i32>,
    pub polymap: Vec<i32>,
}

fn mesh_topology(data: &serde_json::Value, case: &str) -> MeshTopology {
    let mesh = &g2_case(data, case)["mesh"];
    let triples = |key: &str| -> Vec<[f32; 3]> {
        mesh[key]
            .as_array()
            .expect(key)
            .chunks_exact(3)
            .map(|chunk| vec3(&serde_json::Value::Array(chunk.to_vec())))
            .collect()
    };
    let ints = |key: &str| -> Vec<i32> {
        mesh[key]
            .as_array()
            .expect(key)
            .iter()
            .map(|value| value.as_i64().expect("i32") as i32)
            .collect()
    };
    MeshTopology {
        vert: triples("vert"),
        polynormal: triples("polynormal"),
        polyvertadr: ints("polyvertadr"),
        polyvertnum: ints("polyvertnum"),
        polyvert: ints("polyvert"),
        polymapadr: ints("polymapadr"),
        polymapnum: ints("polymapnum"),
        polymap: ints("polymap"),
    }
}

/// mesh case 的 host 拓扑（`mesh-mesh` 复用同名 G2 case 的 mesh；两例 synth case 用自身 mesh）。
pub fn host_mesh_topology(case: &str) -> Option<MeshTopology> {
    let mesh_case = match case {
        "mesh-mesh" => "mesh-mesh",
        "convex-mesh" => "convex-mesh",
        "degenerate-mesh" => "degenerate-mesh",
        _ => return None,
    };
    Some(mesh_topology(&g2_case_data(), mesh_case))
}

/// 宿主 `ccd_pair` 所需的 `Geom` 两类输入（来自与设备完全相同的 floats）。
pub struct HostPairInputs {
    pub meta: CollisionGpuMeta,
    pub pos: Vec<[f32; 3]>,
    pub rot: Vec<[f32; 9]>,
    pub size: [[f32; 3]; 2],
}

/// 从 G5 bin 取宿主输入（`pos`/`rot` 为每个 world 的两个 geom）。
pub fn host_pair_inputs_from_bins(case: &str) -> HostPairInputs {
    let directory = bins_dir(case);
    let meta = build_meta(&read_meta(&directory));
    let xpos = read_f32(&directory, "geom_xpos");
    let xmat = read_f32(&directory, "geom_xmat");
    let size_values = read_f32(&directory, "geom_size");
    let ngeom = meta.ngeom as usize;
    let nworld = meta.nworld as usize;
    let mut pos = Vec::with_capacity(2 * nworld);
    let mut rot = Vec::with_capacity(2 * nworld);
    for world in 0..nworld {
        for slot in 0..2 {
            let geom = if slot == 0 { meta.g1 } else { meta.g2 } as usize;
            let base = (world * ngeom + geom) * 3;
            pos.push([xpos[base], xpos[base + 1], xpos[base + 2]]);
            let rot_base = (world * ngeom + geom) * 9;
            let mut matrix = [0.0_f32; 9];
            matrix.copy_from_slice(&xmat[rot_base..rot_base + 9]);
            rot.push(matrix);
        }
    }
    let size = |geom: i32| -> [f32; 3] {
        let base = geom as usize * 3;
        [
            size_values[base],
            size_values[base + 1],
            size_values[base + 2],
        ]
    };
    HostPairInputs {
        meta,
        pos,
        rot,
        size: [size(meta.g1), size(meta.g2)],
    }
}

/// 从 G2 静态输入取宿主/设备输入（`convex-mesh`/`degenerate-mesh`；用 `aligned` 位姿，
/// 与 G5 设备输入口径同为 mjwarp f32 运动学）。
pub fn g2_pair_inputs(case: &str) -> (HostPairInputs, Vec<[f32; 3]>) {
    let data = g2_case_data();
    let entry = g2_case(&data, case);
    let pair = entry["pair"].as_array().expect("pair");
    let g1 = pair[0].as_u64().expect("pair[0]") as usize;
    let g2 = pair[1].as_u64().expect("pair[1]") as usize;
    let geomtypes: Vec<i32> = entry["geoms"]
        .as_array()
        .expect("geoms")
        .iter()
        .map(|geom| geom["type"].as_i64().expect("type") as i32)
        .collect();
    let sizes: Vec<[f32; 3]> = entry["geoms"]
        .as_array()
        .expect("geoms")
        .iter()
        .map(|geom| vec3(&geom["size"]))
        .collect();
    let worlds = entry["aligned"].as_array().expect("aligned");
    let nworld = worlds.len();
    let mut pos = Vec::with_capacity(2 * nworld);
    let mut rot = Vec::with_capacity(2 * nworld);
    for world in worlds {
        for geom in [g1, g2] {
            pos.push(vec3(&world["pos"][geom]));
            rot.push(vec9(&world["rot"][geom]));
        }
    }
    let topology = host_mesh_topology(case);
    let vert = topology.map(|mesh| mesh.vert).unwrap_or_default();
    let meta = CollisionGpuMeta {
        nworld: nworld as i32,
        ngeom: geomtypes.len() as i32,
        g1: g1 as i32,
        g2: g2 as i32,
        type1: geomtypes[g1],
        type2: geomtypes[g2],
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
    };
    let input = HostPairInputs {
        meta,
        pos,
        rot,
        size: [sizes[g1], sizes[g2]],
    };
    (input, vert)
}

/// `convex-mesh`/`degenerate-mesh` 的设备上传数组：由 G2 静态输入合成；
/// mesh 顶点数 4/8 < 10，命中宿主/设备 `support_mesh` 的穷举分支（无需 graph 数据）。
pub fn device_inputs_from_g2(case: &str) -> OwnedInputs {
    let (host, vert) = g2_pair_inputs(case);
    let meta = host.meta;
    let ngeom = meta.ngeom as usize;
    let nworld = meta.nworld as usize;
    let mut geom_xpos = Vec::with_capacity(nworld * ngeom * 3);
    let mut geom_xmat = Vec::with_capacity(nworld * ngeom * 9);
    let mut geom_size = vec![0.0_f32; ngeom * 3];
    let mut geom_type = vec![0_i32; ngeom];
    let data = g2_case_data();
    let entry = g2_case(&data, case);
    for (geom, value) in entry["geoms"].as_array().expect("geoms").iter().enumerate() {
        let size = vec3(&value["size"]);
        geom_size[geom * 3..geom * 3 + 3].copy_from_slice(&size);
        geom_type[geom] = value["type"].as_i64().expect("type") as i32;
    }
    for world in 0..nworld {
        for geom in 0..ngeom {
            let slot = if geom as i32 == meta.g1 {
                0
            } else if geom as i32 == meta.g2 {
                1
            } else {
                geom
            };
            geom_xpos.extend_from_slice(&host.pos[world * 2 + slot]);
            geom_xmat.extend_from_slice(&host.rot[world * 2 + slot]);
        }
    }
    // geom_aabb（center, half-size）与 rbound：由尺寸/顶点做静态上界，本对照的 ccd 路径不读。
    let mut geom_aabb = Vec::with_capacity(ngeom * 6);
    let mut geom_rbound = vec![0.0_f32; ngeom];
    for geom in 0..ngeom {
        let size = &geom_size[geom * 3..geom * 3 + 3];
        let half = if geom_type[geom] == 7 {
            let mut half = [0.0_f32; 3];
            for axis in 0..3 {
                let (mut min, mut max) = (f32::MAX, f32::MIN);
                for vertex in &vert {
                    min = min.min(vertex[axis]);
                    max = max.max(vertex[axis]);
                }
                half[axis] = 0.5 * (max - min);
            }
            half
        } else {
            [size[0], size[1], size[2]]
        };
        geom_aabb.extend_from_slice(&[0.0, 0.0, 0.0, half[0], half[1], half[2]]);
        geom_rbound[geom] = (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt();
    }
    let mesh_vert: Vec<f32> = vert.iter().flat_map(|vertex| *vertex).collect();
    let has_mesh = !mesh_vert.is_empty();
    OwnedInputs {
        meta,
        geom_xpos,
        geom_xmat,
        geom_size,
        geom_margin: vec![0.0; ngeom],
        geom_gap: vec![0.0; ngeom],
        geom_rbound,
        geom_aabb,
        geom_type,
        geom_dataid: vec![if has_mesh { 0 } else { -1 }; ngeom],
        mesh_vertadr: vec![0],
        mesh_vertnum: vec![vert.len() as i32],
        mesh_graphadr: vec![-1],
        mesh_octadr: vec![-1],
        mesh_vert,
        mesh_graph: Vec::new(),
        hfield_size: Vec::new(),
        hfield_nrow: Vec::new(),
        hfield_ncol: Vec::new(),
        hfield_adr: Vec::new(),
        hfield_data: Vec::new(),
        oct_aabb: Vec::new(),
        oct_child: Vec::new(),
        oct_coeff: Vec::new(),
    }
}
