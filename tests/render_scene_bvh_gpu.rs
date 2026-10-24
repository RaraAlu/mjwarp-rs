#![cfg(feature = "cuda-probe")]
//! 渲染/BVH S2b GPU 验收：CUDA 场景 BVH 遍历与语义射线命中逐条对照。
//!
//! 参考：`fixtures/render-bvh/scene-a/s2/reference_hits.json`（由离线脚本
//! `scripts/authoring/render-bvh/export_s2_hits.py` 从 S0b/S2 已冻结 npz 导出，
//! 存的是送进 MJWarp 的 f32 位姿与 f32 位型命中）。产品测试只读位型，不启 Python。
//!
//! 这些测试需要 GPU + NVRTC，默认 ignore；显式执行：
//!   cargo test --locked --features cuda-probe --test render_scene_bvh_gpu -- --ignored --test-threads=1

use mjwarp_rs::render::bvh::{
    Aabb, BvhError, LeafRayGeometry, RayHit, SceneBvhInput, SceneBvhQuery, build_scene_bvh,
    refit_bvh,
};
use mjwarp_rs::runtime::TransferSession;

#[path = "support/render_bvh/mod.rs"]
mod support;
use support::{ReferenceCase, leaf_bounds, leaf_ray_geometry, load_model, load_ray_reference};

const ULP_TOLERANCE: i64 = 4;
const NORMAL_TOLERANCE: f64 = 1.0e-4;

#[derive(Default)]
struct Comparison {
    compared: usize,
    hits: usize,
    misses: usize,
    skipped_mesh: usize,
    max_ulp: i64,
    max_angle: f64,
}

fn ulp_distance(actual: f32, expected: f32) -> i64 {
    let actual = i64::from(actual.to_bits());
    let expected = i64::from(expected.to_bits());
    (actual - expected).abs()
}

fn normal_angle(actual: [f32; 3], expected: [f32; 3]) -> f64 {
    // 用 f64 的 atan2(|cross|, dot) 度量单位向量夹角；直接 f32 dot+acos 在夹角
    // ~1e-7 rad 时会因 dot 舍入放大到 1e-3，不能反映真实角度。
    let a = actual.map(f64::from);
    let b = expected.map(f64::from);
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let cross_norm = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    cross_norm.atan2(dot)
}

fn compare(actual: &[RayHit], reference: &ReferenceCase, skip: &[usize]) -> Comparison {
    let mut stats = Comparison::default();
    for (index, hit) in actual.iter().enumerate() {
        if skip.contains(&index) {
            stats.skipped_mesh += 1;
            continue;
        }
        let expected_geomid = reference.expected_geomid[index];
        if expected_geomid < 0 {
            assert_eq!(
                hit.geom_id, -1,
                "{} ray {index}: 参考未命中，产品命中",
                reference.name
            );
            assert_eq!(
                hit.distance, -1.0,
                "{} ray {index}: 未命中的 distance 应为 -1",
                reference.name
            );
            stats.misses += 1;
        } else {
            assert_eq!(
                hit.geom_id, expected_geomid,
                "{} ray {index}: geomid 不一致（实际 {}，参考 {}）",
                reference.name, hit.geom_id, expected_geomid
            );
            let ulp = ulp_distance(hit.distance, reference.expected_dist[index]);
            assert!(
                ulp <= ULP_TOLERANCE,
                "{} ray {index}: distance {:#010x} vs {:#010x} 差 {ulp} ULP 超过 {ULP_TOLERANCE}",
                reference.name,
                hit.distance.to_bits(),
                reference.expected_dist[index].to_bits()
            );
            let angle = normal_angle(hit.normal, reference.expected_normal[index]);
            assert!(
                angle <= NORMAL_TOLERANCE,
                "{} ray {index} geom {}: normal {:?} vs {:?} 角度 {angle} 超过 {NORMAL_TOLERANCE}; distance {} vs {}",
                reference.name,
                hit.geom_id,
                hit.normal,
                reference.expected_normal[index],
                hit.distance,
                reference.expected_dist[index]
            );
            stats.max_ulp = stats.max_ulp.max(ulp);
            stats.max_angle = stats.max_angle.max(angle);
            stats.hits += 1;
        }
        stats.compared += 1;
    }
    stats
}

fn session() -> TransferSession {
    TransferSession::new(0).expect("S2b GPU 验收需要 NVIDIA 驱动与 cuda-probe")
}

fn query(session: &TransferSession) -> SceneBvhQuery {
    SceneBvhQuery::new(session).expect("scene.cu 应可在 NVRTC 下编译")
}

fn build_initial(
    bounds: &[Aabb],
    geom_ids: &[i32],
    worlds: usize,
    groups: &[u32],
) -> mjwarp_rs::render::bvh::SceneBvh {
    build_scene_bvh(SceneBvhInput {
        bounds,
        groups,
        geom_ids,
        worlds,
    })
    .expect("S2a 构建输入应被接受")
}

fn changed_geom_rows(actual: &[i32], expected: &[i32], skip: &[usize]) -> Vec<usize> {
    (0..actual.len())
        .filter(|index| !skip.contains(index) && actual[*index] != expected[*index])
        .collect()
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn ray_hits_match_reference_for_initial_and_refit() {
    let session = session();
    let query = query(&session);
    let model = load_model();
    let reference = load_ray_reference();
    let initial_reference = &reference.cases[0];
    let moved_reference = &reference.cases[1];
    let geom_ids: Vec<i32> = reference.enabled.clone();
    let groups = vec![0u32; reference.enabled.len()];

    let initial_bounds = leaf_bounds(&model, &reference, initial_reference);
    let mut bvh = build_initial(&initial_bounds, &geom_ids, 1, &groups);
    let initial_geometry = leaf_ray_geometry(&model, &reference, initial_reference);
    let initial_hits = query
        .query(0, &bvh, &initial_geometry, &reference.rays)
        .expect("initial query");
    let initial_stats = compare(&initial_hits, initial_reference, &reference.mesh_hit_rays);

    let moved_bounds = leaf_bounds(&model, &reference, moved_reference);
    refit_bvh(&mut bvh, &moved_bounds).expect("refit after move");
    let moved_geometry = leaf_ray_geometry(&model, &reference, moved_reference);
    let moved_hits = query
        .query(0, &bvh, &moved_geometry, &reference.rays)
        .expect("moved query");
    let moved_stats = compare(&moved_hits, moved_reference, &reference.mesh_hit_rays_moved);

    assert_eq!(initial_hits.len(), 384);
    assert_eq!(moved_hits.len(), 384);
    assert_eq!(
        initial_stats.misses + initial_stats.hits + initial_stats.skipped_mesh,
        384
    );
    assert_eq!(
        moved_stats.misses + moved_stats.hits + moved_stats.skipped_mesh,
        384
    );

    let mut skip: Vec<usize> = reference.mesh_hit_rays.clone();
    for ray in &reference.mesh_hit_rays_moved {
        if !skip.contains(ray) {
            skip.push(*ray);
        }
    }
    let reference_changed = changed_geom_rows(
        &initial_reference.expected_geomid,
        &moved_reference.expected_geomid,
        &skip,
    );
    let initial_geomids: Vec<i32> = initial_hits.iter().map(|hit| hit.geom_id).collect();
    let moved_geomids: Vec<i32> = moved_hits.iter().map(|hit| hit.geom_id).collect();
    let product_changed = changed_geom_rows(&initial_geomids, &moved_geomids, &skip);
    assert_eq!(
        product_changed, reference_changed,
        "refit 前后 geomid 变化集合必须与参考一致"
    );

    println!(
        "initial: compared={} hits={} misses={} skipped_mesh={} max_ulp={} max_normal_angle={:.3e}",
        initial_stats.compared,
        initial_stats.hits,
        initial_stats.misses,
        initial_stats.skipped_mesh,
        initial_stats.max_ulp,
        initial_stats.max_angle
    );
    println!(
        "moved:   compared={} hits={} misses={} skipped_mesh={} max_ulp={} max_normal_angle={:.3e}",
        moved_stats.compared,
        moved_stats.hits,
        moved_stats.misses,
        moved_stats.skipped_mesh,
        moved_stats.max_ulp,
        moved_stats.max_angle
    );
    println!("refit changed rays: {:?}", product_changed);
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn world_groups_do_not_cross_hit() {
    let session = session();
    let query = query(&session);
    let model = load_model();
    let reference = load_ray_reference();
    let initial_reference = &reference.cases[0];
    let leaf_count = reference.enabled.len();
    let single_bounds = leaf_bounds(&model, &reference, initial_reference);
    let single_geometry = leaf_ray_geometry(&model, &reference, initial_reference);
    let mut single_geom_ids: Vec<i32> = reference.enabled.clone();
    let single_hits = {
        let bvh = build_initial(&single_bounds, &single_geom_ids, 1, &vec![0u32; leaf_count]);
        query
            .query(0, &bvh, &single_geometry, &reference.rays)
            .expect("single query")
    };

    // world 1 复用同一几何，但 geom id 偏移 100；两组各自独立建树。
    let mut bounds = single_bounds.clone();
    bounds.extend_from_slice(&single_bounds);
    let mut groups = vec![0u32; leaf_count];
    groups.extend(vec![1u32; leaf_count]);
    single_geom_ids.extend(reference.enabled.iter().map(|geom_id| geom_id + 100));
    let mut geometry: Vec<LeafRayGeometry> = single_geometry.clone();
    geometry.extend_from_slice(&single_geometry);
    let bvh = build_initial(&bounds, &single_geom_ids, 2, &groups);

    let world0 = query
        .query(0, &bvh, &geometry, &reference.rays)
        .expect("world 0 query");
    let world1 = query
        .query(1, &bvh, &geometry, &reference.rays)
        .expect("world 1 query");
    assert_eq!(world0.len(), 384);
    assert_eq!(world1.len(), 384);

    let skip: Vec<usize> = reference.mesh_hit_rays.clone();
    for index in 0..384 {
        if skip.contains(&index) {
            continue;
        }
        assert_eq!(
            world0[index].geom_id, single_hits[index].geom_id,
            "world 0 应与单 world 结果一致"
        );
        assert!(
            world0[index].geom_id < 100,
            "world 0 不应命 world 1 的 geom"
        );
        if single_hits[index].geom_id >= 0 {
            assert_eq!(
                world1[index].geom_id,
                single_hits[index].geom_id + 100,
                "world 1 应命中自己分组、geom id 偏移 100"
            );
            assert_eq!(
                world1[index].distance.to_bits(),
                world0[index].distance.to_bits(),
                "相同几何下两 world 的 distance 位型应一致"
            );
        } else {
            assert_eq!(world1[index].geom_id, -1, "world 1 未命中应与 world 0 一致");
        }
    }
    println!("two-world query: 384 rays per world, 0 cross-world candidate");
}

#[test]
#[ignore = "需要 GPU + NVRTC"]
fn rejects_invalid_query_inputs() {
    let session = session();
    let query = query(&session);
    let model = load_model();
    let reference = load_ray_reference();
    let case = &reference.cases[0];
    let bounds = leaf_bounds(&model, &reference, case);
    let geometry = leaf_ray_geometry(&model, &reference, case);
    let geom_ids: Vec<i32> = reference.enabled.clone();
    let groups = vec![0u32; reference.enabled.len()];
    let bvh = build_initial(&bounds, &geom_ids, 1, &groups);

    assert!(
        query.query(0, &bvh, &geometry, &[]).is_err(),
        "空射线集应被拒绝"
    );
    assert!(
        query
            .query(1, &bvh, &geometry, &reference.rays[..1])
            .is_err(),
        "world 越界应被拒绝"
    );
    assert!(
        query
            .query(0, &bvh, &geometry[..3], &reference.rays[..1])
            .is_err(),
        "leaf 几何长度不符应被拒绝"
    );
    let mut hfield = geometry.clone();
    hfield[0].geom_type = 1;
    assert!(
        query.query(0, &bvh, &hfield, &reference.rays[..1]).is_err(),
        "hfield 类型应被拒绝"
    );

    let empty: [Aabb; 0] = [];
    assert!(matches!(
        build_scene_bvh(SceneBvhInput {
            bounds: &empty,
            groups: &[],
            geom_ids: &[],
            worlds: 1,
        }),
        Err(BvhError::EmptyScene)
    ));
    println!(
        "rejection surface: empty rays / world out of range / length mismatch / hfield rejected"
    );
}
