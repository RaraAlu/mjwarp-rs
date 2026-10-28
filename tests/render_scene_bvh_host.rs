//! 渲染/BVH S2a 宿主侧验收：primitive AABB 位等、构建/重拟合语义与拒绝面。
//!
//! 参考来自离线 authoring 捕获（MJWarp 3.15，`scripts/authoring/render-bvh/capture_s2a_bounds.py`），
//! fixture 里存的是送进 MJWarp kernel 的 f32 位姿与产出的 f32 位型。产品测试只读
//! 这些位型，不启动 Python、不需要 GPU。模型输入用 S1 已验收的 `model-312.json`。

use std::collections::BTreeMap;

use mjwarp_rs::render::bvh::{
    Aabb, BvhError, LeafGeometry, NodeKind, SceneBvh, SceneBvhInput, build_scene_bvh,
    compute_leaf_aabb, mesh_half_extents, refit_bvh,
};

const REFERENCE: &str = include_str!("../fixtures/render-bvh/scene-a/s2a/bounds.json");
const MODEL_312: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");

#[derive(Debug)]
struct Case {
    name: String,
    xpos: Vec<[f32; 3]>,
    xmat: Vec<[[f32; 3]; 3]>,
    lower: Vec<[f32; 3]>,
    upper: Vec<[f32; 3]>,
}

struct Reference {
    enabled: Vec<i32>,
    geom_type: Vec<i32>,
    geom_dataid: Vec<i32>,
    geom_size_bits: Vec<u32>,
    mesh_bounds_bits: BTreeMap<i32, [u32; 3]>,
    cases: Vec<Case>,
}

struct ModelInputs {
    geom_type: Vec<i32>,
    geom_dataid: Vec<i32>,
    geom_size: Vec<f64>,
    mesh_vertices: Vec<Vec<[f32; 3]>>,
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

fn u32s(value: &serde_json::Value) -> Vec<u32> {
    numbers(value).iter().map(|item| *item as u32).collect()
}

fn bits_f32(bits: &[u32], offset: usize, count: usize) -> Vec<f32> {
    bits[offset..offset + count]
        .iter()
        .map(|bits| f32::from_bits(*bits))
        .collect()
}

fn all_f32(bits: &[u32]) -> Vec<f32> {
    bits.iter().map(|bits| f32::from_bits(*bits)).collect()
}

fn rows3(values: &[f32]) -> Vec<[f32; 3]> {
    values
        .chunks(3)
        .map(|row| [row[0], row[1], row[2]])
        .collect()
}

fn mats3(bits: &[u32]) -> Vec<[[f32; 3]; 3]> {
    bits.chunks(9)
        .map(|rows| {
            let row = |index: usize| {
                [
                    f32::from_bits(rows[index]),
                    f32::from_bits(rows[index + 1]),
                    f32::from_bits(rows[index + 2]),
                ]
            };
            [row(0), row(3), row(6)]
        })
        .collect()
}

fn bits_of(values: [f32; 3]) -> [u32; 3] {
    values.map(f32::to_bits)
}

fn load_reference() -> Reference {
    let document: serde_json::Value = serde_json::from_str(REFERENCE).unwrap();
    let model = &document["model"];
    let mut mesh_bounds_bits = BTreeMap::new();
    for (key, value) in model["mesh_bounds_size_bits"].as_object().unwrap() {
        let bits = u32s(value);
        mesh_bounds_bits.insert(key.parse().unwrap(), [bits[0], bits[1], bits[2]]);
    }
    let cases = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| Case {
            name: case["name"].as_str().unwrap().to_string(),
            xpos: rows3(&all_f32(&u32s(&case["inputs"]["geom_xpos_bits"]))),
            xmat: mats3(&u32s(&case["inputs"]["geom_xmat_bits"])),
            lower: rows3(&all_f32(&u32s(&case["lower_bits"]))),
            upper: rows3(&all_f32(&u32s(&case["upper_bits"]))),
        })
        .collect();
    Reference {
        enabled: ints(&document["context"]["enabled_geom_ids"]),
        geom_type: ints(&model["geom_type"]),
        geom_dataid: ints(&model["geom_dataid"]),
        geom_size_bits: u32s(&model["geom_size_bits"]),
        mesh_bounds_bits,
        cases,
    }
}

fn load_model() -> ModelInputs {
    let document: serde_json::Value = serde_json::from_str(MODEL_312).unwrap();
    let arrays = &document["arrays"];
    let mesh_vert = numbers(&arrays["mesh_vert"]["values"]);
    let mesh_vertadr = ints(&arrays["mesh_vertadr"]["values"]);
    let mesh_vertnum = ints(&arrays["mesh_vertnum"]["values"]);
    let mesh_vertices = (0..mesh_vertadr.len())
        .map(|mesh| {
            let start = mesh_vertadr[mesh] as usize * 3;
            let count = mesh_vertnum[mesh] as usize * 3;
            mesh_vert[start..start + count]
                .chunks(3)
                .map(|vertex| [vertex[0] as f32, vertex[1] as f32, vertex[2] as f32])
                .collect()
        })
        .collect();
    ModelInputs {
        geom_type: ints(&arrays["geom_type"]["values"]),
        geom_dataid: ints(&arrays["geom_dataid"]["values"]),
        geom_size: numbers(&arrays["geom_size"]["values"]),
        mesh_vertices,
    }
}

fn case<'a>(reference: &'a Reference, name: &str) -> &'a Case {
    reference
        .cases
        .iter()
        .find(|case| case.name == name)
        .unwrap()
}

fn mesh_half_from_model(model: &ModelInputs) -> BTreeMap<i32, [f32; 3]> {
    model
        .mesh_vertices
        .iter()
        .enumerate()
        .map(|(mesh, vertices)| (mesh as i32, mesh_half_extents(vertices).unwrap()))
        .collect()
}

fn geom_geometry(
    model: &ModelInputs,
    reference: &Reference,
    geom_id: i32,
    mesh_half: &BTreeMap<i32, [f32; 3]>,
) -> LeafGeometry {
    let geom = geom_id as usize;
    let geom_type = model.geom_type[geom];
    let mut size = bits_f32(&reference.geom_size_bits, geom * 3, 3);
    if matches!(geom_type, 7 | 8) {
        let mesh_id = model.geom_dataid[geom];
        size = if mesh_id >= 0 {
            mesh_half[&mesh_id].to_vec()
        } else {
            vec![0.0; 3]
        };
    }
    LeafGeometry::from_geom_type(geom_type, [size[0], size[1], size[2]]).unwrap()
}

fn reference_leaf_bounds(case: &Case) -> Vec<Aabb> {
    case.lower
        .iter()
        .zip(case.upper.iter())
        .map(|(lower, upper)| Aabb {
            lower: *lower,
            upper: *upper,
        })
        .collect()
}

/// 语义不变量：遍历每个 group 的树恰好覆盖其全部 leaf，内部节点包围盒等于子节点并集。
fn assert_tree_invariants(bvh: &SceneBvh, bounds: &[Aabb]) {
    for world in 0..bvh.worlds() {
        let nodes = bvh.nodes();
        let mut stack = vec![bvh.group_root(world).unwrap()];
        let mut leaves = Vec::new();
        while let Some(index) = stack.pop() {
            let node = nodes[index as usize];
            match node.kind {
                NodeKind::Leaf { leaf } => leaves.push(leaf),
                NodeKind::Internal { left, right } => {
                    assert_eq!(
                        node.bounds,
                        nodes[left as usize]
                            .bounds
                            .union(nodes[right as usize].bounds),
                        "内部节点包围盒应等于子节点并集"
                    );
                    stack.push(left);
                    stack.push(right);
                }
            }
        }
        leaves.sort_unstable();
        let expected: Vec<u32> = (0..bounds.len() as u32)
            .filter(|leaf| bvh.leaf_group(*leaf) == Some(world as u32))
            .collect();
        assert_eq!(leaves, expected, "每个 leaf 必须恰好被访问一次");
    }
}

#[test]
fn model_inputs_match_reference_f32_bits() {
    let model = load_model();
    let reference = load_reference();
    assert_eq!(model.geom_type, reference.geom_type);
    assert_eq!(model.geom_dataid, reference.geom_dataid);
    assert_eq!(
        reference.enabled,
        (0..model.geom_type.len() as i32).collect::<Vec<_>>()
    );
    for (geom, size) in model.geom_size.iter().enumerate() {
        assert_eq!(
            (*size as f32).to_bits(),
            f32::from_bits(reference.geom_size_bits[geom]).to_bits(),
            "geom_size[{geom}] f32 位型与 3.15 参考不一致"
        );
    }
    for (mesh, half) in mesh_half_from_model(&model) {
        let expected = reference.mesh_bounds_bits[&mesh];
        for axis in 0..3 {
            assert_eq!(
                half[axis].to_bits(),
                expected[axis],
                "mesh[{mesh}] bounds[{axis}] 位型与 3.15 参考不一致"
            );
        }
    }
}

#[test]
fn leaf_aabbs_match_reference_bit_for_bit() {
    let model = load_model();
    let reference = load_reference();
    let mesh_half = mesh_half_from_model(&model);
    let mut checked = 0usize;
    for case in &reference.cases {
        for (slot, geom_id) in reference.enabled.iter().enumerate() {
            let geometry = geom_geometry(&model, &reference, *geom_id, &mesh_half);
            let actual = compute_leaf_aabb(&geometry, case.xpos[slot], case.xmat[slot]).unwrap();
            assert_eq!(
                (bits_of(actual.lower), bits_of(actual.upper)),
                (bits_of(case.lower[slot]), bits_of(case.upper[slot])),
                "{} geom {geom_id} leaf AABB 位型与冻结参考不一致",
                case.name
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 56, "7 类 leaf × 8 个位姿");
}

#[test]
fn build_and_refit_preserve_group_semantics() {
    let model = load_model();
    let reference = load_reference();
    let initial_bounds = reference_leaf_bounds(case(&reference, "initial"));
    let moved_bounds = reference_leaf_bounds(case(&reference, "moved-base"));
    let groups = vec![0u32; initial_bounds.len()];
    let geom_ids: Vec<i32> = reference.enabled.to_vec();
    let mut bvh = build_scene_bvh(SceneBvhInput {
        bounds: &initial_bounds,
        groups: &groups,
        geom_ids: &geom_ids,
        worlds: 1,
    })
    .unwrap();
    let initial_union = initial_bounds.iter().copied().reduce(Aabb::union).unwrap();
    assert_eq!(bvh.group_bounds(0).unwrap(), initial_union);
    assert_tree_invariants(&bvh, &initial_bounds);

    refit_bvh(&mut bvh, &moved_bounds).unwrap();
    assert_eq!(bvh.leaf_bounds(), &moved_bounds);
    assert_eq!(
        bvh.group_bounds(0).unwrap(),
        moved_bounds.iter().copied().reduce(Aabb::union).unwrap()
    );
    assert_tree_invariants(&bvh, &moved_bounds);

    let changed: Vec<usize> = (0..initial_bounds.len())
        .filter(|leaf| initial_bounds[*leaf] != moved_bounds[*leaf])
        .collect();
    assert_eq!(
        changed,
        vec![1, 2, 3, 4, 5, 6],
        "floor(geom 0)静态，其余 leaf 随 base/tri body 位移改变"
    );
    assert_eq!(model.geom_type[0], 0, "geom0 必须是 plane");
}

#[test]
fn world_groups_do_not_cross_hit() {
    let reference = load_reference();
    let bounds = reference_leaf_bounds(case(&reference, "initial"));
    let mut duplicated = bounds.clone();
    duplicated.extend_from_slice(&bounds);
    let mut groups = vec![0u32; bounds.len()];
    groups.extend(vec![1u32; bounds.len()]);
    let geom_ids: Vec<i32> = reference
        .enabled
        .iter()
        .chain(reference.enabled.iter())
        .copied()
        .collect();
    let bvh = build_scene_bvh(SceneBvhInput {
        bounds: &duplicated,
        groups: &groups,
        geom_ids: &geom_ids,
        worlds: 2,
    })
    .unwrap();
    let center = bounds.iter().copied().reduce(Aabb::union).unwrap().center();
    let floor = bounds[0];
    let mut hits = 0usize;
    for index in 0..32 {
        let theta = index as f32 * (std::f32::consts::TAU / 32.0);
        let radius = 0.5 + (index % 5) as f32 * 0.5;
        let origin = [
            center[0] + radius * theta.cos(),
            center[1] + radius * theta.sin(),
            floor.upper[2] + 5.0,
        ];
        let direction = [
            center[0] - origin[0],
            center[1] - origin[1],
            center[2] - origin[2],
        ];
        let mut per_world = Vec::new();
        for world in 0..2 {
            let candidates = bvh.ray_leaf_candidates(world, origin, direction).unwrap();
            if !candidates.is_empty() {
                hits += 1;
            }
            let mut geom_hits = Vec::new();
            for candidate in &candidates {
                assert_eq!(
                    bvh.leaf_group(*candidate),
                    Some(world as u32),
                    "候选 leaf 跨 world"
                );
                geom_hits.push(bvh.leaf_geom_id(*candidate).unwrap());
            }
            geom_hits.sort_unstable();
            per_world.push(geom_hits);
        }
        assert_eq!(
            per_world[0], per_world[1],
            "两个 world 的候选 geom 集合应一致"
        );
    }
    assert!(hits >= 60, "射线集应能实际命中（至少 60/64 次）");
    assert!(matches!(
        bvh.ray_leaf_candidates(2, center, [0.0, 0.0, -1.0]),
        Err(BvhError::WorldOutOfRange {
            world: 2,
            worlds: 2
        })
    ));
}

#[test]
fn rejects_empty_out_of_range_and_invalid_inputs() {
    let reference = load_reference();
    let bounds = reference_leaf_bounds(case(&reference, "initial"));
    let groups = vec![0u32; bounds.len()];
    let geom_ids: Vec<i32> = reference.enabled.clone();
    let attempt = |bounds: &[Aabb], groups: &[u32], geom_ids: &[i32], worlds: usize| {
        build_scene_bvh(SceneBvhInput {
            bounds,
            groups,
            geom_ids,
            worlds,
        })
    };
    let empty: [Aabb; 0] = [];
    assert!(matches!(
        attempt(&empty, &[], &[], 1),
        Err(BvhError::EmptyScene)
    ));
    assert!(matches!(
        attempt(&bounds, &[0], &geom_ids, 1),
        Err(BvhError::LengthMismatch {
            field: "groups",
            ..
        })
    ));
    assert!(matches!(
        attempt(&bounds, &groups, &geom_ids, 0),
        Err(BvhError::InvalidWorldCount { worlds: 0 })
    ));
    let mut out_of_range = groups.clone();
    out_of_range[3] = 5;
    assert!(matches!(
        attempt(&bounds, &out_of_range, &geom_ids, 1),
        Err(BvhError::GroupOutOfRange {
            leaf: 3,
            group: 5,
            worlds: 1
        })
    ));
    assert!(matches!(
        attempt(&bounds, &groups, &geom_ids, 2),
        Err(BvhError::EmptyWorld { world: 1 })
    ));
    let mut invalid = bounds.clone();
    invalid[2] = Aabb {
        lower: [1.0; 3],
        upper: [-1.0; 3],
    };
    assert!(matches!(
        attempt(&invalid, &groups, &geom_ids, 1),
        Err(BvhError::InvalidAabb { leaf: 2 })
    ));

    let mut bvh = attempt(&bounds, &groups, &geom_ids, 1).unwrap();
    assert!(matches!(
        refit_bvh(&mut bvh, &bounds[..3]),
        Err(BvhError::LengthMismatch {
            field: "bounds",
            ..
        })
    ));
    assert!(matches!(
        refit_bvh(&mut bvh, &invalid),
        Err(BvhError::InvalidAabb { leaf: 2 })
    ));

    // R05-S2：gtype 1（hfield）已支持；未支持面改用 gtype 9。
    assert_eq!(
        LeafGeometry::from_geom_type(9, [1.0, 1.0, 1.0]),
        Err(BvhError::UnsupportedGeomType { geom_type: 9 })
    );
    let plane = LeafGeometry::from_geom_type(0, [1.0, 1.0, 1.0]).unwrap();
    assert!(matches!(
        compute_leaf_aabb(
            &plane,
            [f32::NAN, 0.0, 0.0],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        ),
        Err(BvhError::NonFinite {
            field: "geom_xpos",
            index: 0
        })
    ));
    assert_eq!(mesh_half_extents(&[]), Err(BvhError::MissingMeshBounds));
}
