//! S2b GPU 测试的共享夹具解析（纯主机，不启动 Python）。
//!
//! 输入：
//! - `fixtures/render-bvh/scene-a/model-312.json`（S1 已验收的 3.12 渲染模型输入）；
//! - `fixtures/render-bvh/scene-a/s2/reference_hits.json`（离线脚本
//!   `scripts/authoring/render-bvh/export_s2_hits.py` 由 S0b/S2 npz 导出的 f32 位型参考）。
#![allow(dead_code)]

use mjwarp_rs::render::bvh::{
    LeafGeometry, LeafRayGeometry, Ray, compute_leaf_aabb, mesh_half_extents,
};

pub const MODEL_312: &str = include_str!("../../../fixtures/render-bvh/scene-a/model-312.json");
pub const REFERENCE_HITS: &str =
    include_str!("../../../fixtures/render-bvh/scene-a/s2/reference_hits.json");

pub struct ReferenceCase {
    pub name: String,
    pub xpos: Vec<[f32; 3]>,
    pub xmat: Vec<[[f32; 3]; 3]>,
    pub expected_dist: Vec<f32>,
    pub expected_geomid: Vec<i32>,
    pub expected_normal: Vec<[f32; 3]>,
}

pub struct RayReference {
    pub enabled: Vec<i32>,
    pub rays: Vec<Ray>,
    pub cases: Vec<ReferenceCase>,
    pub mesh_hit_rays: Vec<usize>,
    pub mesh_hit_rays_moved: Vec<usize>,
}

pub struct ModelInputs {
    pub geom_type: Vec<i32>,
    pub geom_dataid: Vec<i32>,
    pub geom_size: Vec<f64>,
    pub mesh_half: Vec<[f32; 3]>,
}

fn numbers(value: &serde_json::Value) -> Vec<f64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap())
        .collect()
}

fn ints(value: &serde_json::Value) -> Vec<i32> {
    numbers(value).iter().map(|item| *item as i32).collect()
}

fn bits_f32(bits: &[u32], offset: usize, count: usize) -> Vec<f32> {
    bits[offset..offset + count]
        .iter()
        .map(|bits| f32::from_bits(*bits))
        .collect()
}

fn u32s(value: &serde_json::Value) -> Vec<u32> {
    numbers(value).iter().map(|item| *item as u32).collect()
}

pub fn load_model() -> ModelInputs {
    let document: serde_json::Value = serde_json::from_str(MODEL_312).unwrap();
    let arrays = &document["arrays"];
    let mesh_vert = numbers(&arrays["mesh_vert"]["values"]);
    let mesh_vertadr = ints(&arrays["mesh_vertadr"]["values"]);
    let mesh_vertnum = ints(&arrays["mesh_vertnum"]["values"]);
    let mesh_half = (0..mesh_vertadr.len())
        .map(|mesh| {
            let start = mesh_vertadr[mesh] as usize * 3;
            let count = mesh_vertnum[mesh] as usize * 3;
            let vertices: Vec<[f32; 3]> = mesh_vert[start..start + count]
                .chunks(3)
                .map(|vertex| [vertex[0] as f32, vertex[1] as f32, vertex[2] as f32])
                .collect();
            mesh_half_extents(&vertices).unwrap()
        })
        .collect();
    ModelInputs {
        geom_type: ints(&arrays["geom_type"]["values"]),
        geom_dataid: ints(&arrays["geom_dataid"]["values"]),
        geom_size: numbers(&arrays["geom_size"]["values"]),
        mesh_half,
    }
}

pub fn load_ray_reference() -> RayReference {
    let document: serde_json::Value = serde_json::from_str(REFERENCE_HITS).unwrap();
    let ray_count = document["ray_count"].as_u64().unwrap() as usize;
    let origin = bits_f32(&u32s(&document["rays"]["origin_bits"]), 0, ray_count * 3);
    let direction = bits_f32(&u32s(&document["rays"]["direction_bits"]), 0, ray_count * 3);
    let rays = (0..ray_count)
        .map(|index| Ray {
            origin: [
                origin[3 * index],
                origin[3 * index + 1],
                origin[3 * index + 2],
            ],
            direction: [
                direction[3 * index],
                direction[3 * index + 1],
                direction[3 * index + 2],
            ],
        })
        .collect();
    let mut cases = Vec::new();
    for name in ["initial", "moved"] {
        let case = &document["cases"][name];
        let xpos = bits_f32(&u32s(&case["geom_xpos_bits"]), 0, 21);
        let xmat = bits_f32(&u32s(&case["geom_xmat_bits"]), 0, 63);
        let dist = bits_f32(&u32s(&case["dist_bits"]), 0, ray_count);
        let normal = bits_f32(&u32s(&case["normal_bits"]), 0, ray_count * 3);
        let geomid = ints(&case["geomid"]);
        let xpos = (0..7)
            .map(|index| [xpos[3 * index], xpos[3 * index + 1], xpos[3 * index + 2]])
            .collect();
        let xmat = (0..7)
            .map(|index| {
                let base = 9 * index;
                [
                    [xmat[base], xmat[base + 1], xmat[base + 2]],
                    [xmat[base + 3], xmat[base + 4], xmat[base + 5]],
                    [xmat[base + 6], xmat[base + 7], xmat[base + 8]],
                ]
            })
            .collect();
        let normal = (0..ray_count)
            .map(|index| {
                [
                    normal[3 * index],
                    normal[3 * index + 1],
                    normal[3 * index + 2],
                ]
            })
            .collect();
        cases.push(ReferenceCase {
            name: name.to_string(),
            xpos,
            xmat,
            expected_dist: dist,
            expected_geomid: geomid,
            expected_normal: normal,
        });
    }
    RayReference {
        enabled: ints(&document["enabled_geom_ids"]),
        rays,
        cases,
        mesh_hit_rays: document["stats"]["mesh_hit_rays"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect(),
        mesh_hit_rays_moved: document["stats"]["mesh_hit_rays_moved"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap() as usize)
            .collect(),
    }
}

/// 每个 leaf 的 S2a 几何输入（`LeafGeometry`）。
pub fn leaf_geometry(model: &ModelInputs, reference: &RayReference) -> Vec<LeafGeometry> {
    reference
        .enabled
        .iter()
        .map(|geom_id| {
            let geom = *geom_id as usize;
            let geom_type = model.geom_type[geom];
            let mut size = [
                model.geom_size[3 * geom] as f32,
                model.geom_size[3 * geom + 1] as f32,
                model.geom_size[3 * geom + 2] as f32,
            ];
            if matches!(geom_type, 7 | 8) {
                let mesh_id = model.geom_dataid[geom];
                if mesh_id >= 0 {
                    size = model.mesh_half[mesh_id as usize];
                } else {
                    size = [0.0; 3];
                }
            }
            LeafGeometry::from_geom_type(geom_type, size).unwrap()
        })
        .collect()
}

pub fn leaf_ray_geometry(
    model: &ModelInputs,
    reference: &RayReference,
    case: &ReferenceCase,
) -> Vec<LeafRayGeometry> {
    reference
        .enabled
        .iter()
        .enumerate()
        .map(|(slot, geom_id)| {
            let geom = *geom_id as usize;
            let geom_type = model.geom_type[geom];
            let mut size = [
                model.geom_size[3 * geom] as f32,
                model.geom_size[3 * geom + 1] as f32,
                model.geom_size[3 * geom + 2] as f32,
            ];
            if matches!(geom_type, 7 | 8) {
                let mesh_id = model.geom_dataid[geom];
                if mesh_id >= 0 {
                    size = model.mesh_half[mesh_id as usize];
                } else {
                    size = [0.0; 3];
                }
            }
            let rotation = case.xmat[slot];
            LeafRayGeometry {
                geom_type,
                size,
                xpos: case.xpos[slot],
                xmat: [
                    rotation[0][0],
                    rotation[0][1],
                    rotation[0][2],
                    rotation[1][0],
                    rotation[1][1],
                    rotation[1][2],
                    rotation[2][0],
                    rotation[2][1],
                    rotation[2][2],
                ],
            }
        })
        .collect()
}

pub fn leaf_bounds(
    model: &ModelInputs,
    reference: &RayReference,
    case: &ReferenceCase,
) -> Vec<mjwarp_rs::render::bvh::Aabb> {
    leaf_geometry(model, reference)
        .iter()
        .enumerate()
        .map(|(slot, geometry)| {
            compute_leaf_aabb(geometry, case.xpos[slot], case.xmat[slot]).unwrap()
        })
        .collect()
}
