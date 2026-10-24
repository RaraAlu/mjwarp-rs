//! 渲染/BVH S3 验收：网格三角 BVH 构建/重拟合与网格射线语义命中逐条对照。
//!
//! 参考：`fixtures/render-bvh/scene-a/s3/mesh_query.json`，由离线脚本
//! `scripts/authoring/render-bvh/capture_s3_mesh.py` 在 GPU 上调用冻结上游
//! `ray_mesh_with_bvh`（渲染路径同一函数）捕获：设备侧 f32 geom 位姿、逐射线
//! max_t、distance/normal/u/v/face 位型。产品测试只读位型，不启动 Python。
//!
//! 主机测试默认执行；GPU 用例默认 ignore，显式执行：
//! cargo test --locked --features cuda-probe --test render_mesh_bvh -- --ignored --test-threads=1

#[cfg(feature = "cuda-probe")]
use mjwarp_rs::render::bvh::MeshBvhInput;
use mjwarp_rs::render::bvh::{MeshBvh, vertices_from_f64};

#[path = "support/render_bvh/mesh.rs"]
mod support;
#[cfg(feature = "cuda-probe")]
use support::MeshFixture;
use support::{MeshCase, load, model_312_mesh_vert};

#[cfg(feature = "cuda-probe")]
const ULP_TOLERANCE: i64 = 4;
#[cfg(feature = "cuda-probe")]
const NORMAL_TOLERANCE: f64 = 1.0e-4;

#[cfg(feature = "cuda-probe")]
fn ulp_distance(actual: f32, expected: f32) -> i64 {
    if actual == expected {
        return 0;
    }
    let actual = i64::from(actual.to_bits());
    let expected = i64::from(expected.to_bits());
    (actual - expected).abs()
}

#[cfg(feature = "cuda-probe")]
fn normal_angle(actual: [f32; 3], expected: [f32; 3]) -> f64 {
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

#[cfg(feature = "cuda-probe")]
fn mesh_of(fixture: &MeshFixture, case: &MeshCase) -> MeshBvh {
    fixture.case_mesh(case)
}

#[test]
fn fixture_vertices_match_model_312_conversion() {
    let fixture = load();
    assert_eq!(fixture.task_id, "2026-10-24-render-bvh-s3");
    let model_312 = vertices_from_f64(&model_312_mesh_vert()).expect("model-312 mesh_vert");
    let bits = |values: &[[f32; 3]]| -> Vec<u32> {
        values
            .iter()
            .flat_map(|vertex| vertex.iter().map(|value| value.to_bits()))
            .collect()
    };
    assert_eq!(
        bits(&model_312),
        bits(&fixture.model_312_vertices),
        "3.12 mesh_vert f64 -> f32 应与夹具记录一致"
    );
    assert_eq!(
        bits(&fixture.initial_vertices),
        bits(&fixture.model_312_vertices),
        "夹具初始顶点应等于 3.12 模型顶点"
    );
    assert_ne!(
        bits(&fixture.initial_vertices),
        bits(&fixture.dynamic_vertices)
    );
    assert!(fixture.indices.len().is_multiple_of(3));
    assert_eq!(fixture.origins.len(), fixture.tags.len());
}

#[test]
fn mesh_bvh_builds_refits_and_candidates_cover_reference_hits() {
    let fixture = load();
    let triangles = fixture.indices.len() / 3;
    let mut mesh = fixture.initial_mesh();
    assert_eq!(mesh.triangle_count(), triangles);
    let mut order = mesh.leaf_order().to_vec();
    order.sort_unstable();
    assert_eq!(order, (0..triangles as u32).collect::<Vec<u32>>());
    assert!(mesh.nodes().len() < 2 * triangles);
    let candidates_for = |mesh: &MeshBvh, index: usize, case: &MeshCase| {
        let (origin, direction) = support::world_to_local(
            fixture.origins[index],
            fixture.directions[index],
            case.xpos,
            case.xmat,
        );
        mesh.ray_candidates(origin, direction)
    };
    // 命中三角形的 AABB 必须被宿主候选遍历覆盖（只对设计射线断言，避免擦边数值争议）。
    let initial = fixture.case("initial");
    for (index, tag) in fixture.tags.iter().enumerate() {
        if matches!(tag.as_str(), "front" | "back" | "axis") && initial.hit[index] {
            let candidates = candidates_for(&mesh, index, initial);
            assert!(
                candidates.contains(&(initial.face[index] as u32)),
                "tag {tag} 射线 {index} 的命中面 {} 未被宿主候选覆盖：{candidates:?}",
                initial.face[index]
            );
        }
    }
    // 动态顶点更新走 set_points + refit，不重建树。
    mesh.set_points(&fixture.dynamic_vertices)
        .expect("动态顶点");
    mesh.refit();
    assert_eq!(mesh.vertices().len(), fixture.initial_vertices.len());
    let dynamic = fixture.case("dynamic");
    for (index, tag) in fixture.tags.iter().enumerate() {
        if matches!(tag.as_str(), "front" | "back" | "axis") && dynamic.hit[index] {
            let candidates = candidates_for(&mesh, index, dynamic);
            assert!(
                candidates.contains(&(dynamic.face[index] as u32)),
                "动态 refit 后 tag {tag} 射线 {index} 的命中面 {} 未被候选覆盖",
                dynamic.face[index]
            );
        }
    }
}

#[test]
fn fixture_cross_checks_and_static_replay_are_clean() {
    let fixture = load();
    assert!(fixture.cross_check_all_true, "S2b 交叉校验必须全部成立");
    assert_eq!(fixture.scene_degenerate_woop_queries, 0, "f64 兜底为 0");
    assert_eq!(fixture.fallback_decision_changes, 0, "f64 兜底不能改变判定");
    assert!(fixture.determinism_identical, "两次捕获必须逐位一致");
    assert_eq!(fixture.case("initial").geom_id, 3);
    assert_eq!(fixture.case("moved").geom_id, 103);
    assert!(fixture.case("initial").cull_backfaces);
    assert!(!fixture.case("no_cull").cull_backfaces);
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::render::bvh::{MeshBvhQuery, MeshRayGeometry, MeshRayHit, Ray};
    use mjwarp_rs::runtime::TransferSession;

    #[derive(Default)]
    struct Comparison {
        compared: usize,
        hits: usize,
        misses: usize,
        max_distance_ulp: i64,
        max_uv_ulp: i64,
        max_normal_angle: f64,
    }

    fn session() -> TransferSession {
        TransferSession::new(0).expect("S3 GPU 验收需要 NVIDIA 驱动与 cuda-probe")
    }

    fn query(session: &TransferSession) -> MeshBvhQuery {
        MeshBvhQuery::new(session).expect("mesh.cu 应可在 NVRTC 下编译")
    }

    fn geometry(case: &MeshCase) -> MeshRayGeometry {
        MeshRayGeometry {
            geom_id: case.geom_id,
            xpos: case.xpos,
            xmat: case.xmat,
        }
    }

    fn rays(fixture: &MeshFixture) -> Vec<Ray> {
        fixture
            .origins
            .iter()
            .zip(fixture.directions.iter())
            .map(|(origin, direction)| Ray {
                origin: *origin,
                direction: *direction,
            })
            .collect()
    }

    fn compare(actual: &[MeshRayHit], case: &MeshCase) -> Comparison {
        assert_eq!(actual.len(), case.hit.len(), "{} 查询条数", case.name);
        let mut stats = Comparison {
            compared: actual.len(),
            ..Default::default()
        };
        for (index, hit) in actual.iter().enumerate() {
            if !case.hit[index] {
                assert_eq!(hit.geom_id, -1, "{} 射线 {index} 应为未命中", case.name);
                assert_eq!(hit.distance, -1.0, "{} 射线 {index} 距离应为 -1", case.name);
                assert_eq!(hit.face, -1, "{} 射线 {index} face 应为 -1", case.name);
                stats.misses += 1;
                continue;
            }
            assert_eq!(
                hit.geom_id, case.geom_id,
                "{} 射线 {index} geomid",
                case.name
            );
            assert_eq!(
                hit.face, case.face[index],
                "{} 射线 {index} face",
                case.name
            );
            let distance = ulp_distance(hit.distance, case.distance[index]);
            assert!(
                distance <= ULP_TOLERANCE,
                "{} 射线 {index} distance 差 {distance} ULP 超过 {ULP_TOLERANCE}",
                case.name
            );
            let u = ulp_distance(hit.u, case.u[index]);
            let v = ulp_distance(hit.v, case.v[index]);
            assert!(
                u <= ULP_TOLERANCE && v <= ULP_TOLERANCE,
                "{} 射线 {index} u/v 差 {u}/{v} ULP 超过 {ULP_TOLERANCE}",
                case.name
            );
            let angle = normal_angle(hit.normal, case.normal[index]);
            assert!(
                angle <= NORMAL_TOLERANCE,
                "{} 射线 {index} normal 角度 {angle:.3e} 超过 {NORMAL_TOLERANCE}",
                case.name
            );
            stats.hits += 1;
            stats.max_distance_ulp = stats.max_distance_ulp.max(distance);
            stats.max_uv_ulp = stats.max_uv_ulp.max(u).max(v);
            stats.max_normal_angle = stats.max_normal_angle.max(angle);
        }
        stats
    }

    fn report(name: &str, stats: &Comparison) {
        println!(
            "{name}: compared={} hits={} misses={} max_distance_ulp={} max_uv_ulp={} max_normal_angle={:.3e}",
            stats.compared,
            stats.hits,
            stats.misses,
            stats.max_distance_ulp,
            stats.max_uv_ulp,
            stats.max_normal_angle
        );
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn mesh_query_matches_reference_for_every_case() {
        let session = session();
        let query = query(&session);
        let fixture = load();
        let rays = rays(&fixture);
        for name in ["initial", "moved", "no_cull", "dynamic", "dynamic_moved"] {
            let case = fixture.case(name);
            let mesh = mesh_of(&fixture, case);
            let hits = query
                .query(
                    &mesh,
                    &geometry(case),
                    &rays,
                    &case.max_t,
                    case.cull_backfaces,
                )
                .expect("网格查询应成功");
            report(name, &compare(&hits, case));
        }
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn set_points_refit_changes_hits_like_reference() {
        let session = session();
        let query = query(&session);
        let fixture = load();
        let rays = rays(&fixture);
        let initial = fixture.case("initial");
        let dynamic = fixture.case("dynamic");
        let initial_hits = query
            .query(
                &fixture.initial_mesh(),
                &geometry(initial),
                &rays,
                &initial.max_t,
                true,
            )
            .expect("initial query");
        let dynamic_hits = query
            .query(
                &mesh_of(&fixture, dynamic),
                &geometry(dynamic),
                &rays,
                &dynamic.max_t,
                true,
            )
            .expect("dynamic query");
        let changed = |actual: &[MeshRayHit]| -> Vec<usize> {
            actual
                .iter()
                .enumerate()
                .filter(|(index, hit)| {
                    hit.geom_id != initial_hits[*index].geom_id
                        || hit.face != initial_hits[*index].face
                        || (hit.geom_id >= 0
                            && ulp_distance(hit.distance, initial_hits[*index].distance) > 4)
                })
                .map(|(index, _)| index)
                .collect()
        };
        let reference_changed = |left: &MeshCase, right: &MeshCase| -> Vec<usize> {
            (0..left.hit.len())
                .filter(|index| {
                    left.hit[*index] != right.hit[*index]
                        || left.face[*index] != right.face[*index]
                        || (left.hit[*index]
                            && ulp_distance(left.distance[*index], right.distance[*index]) > 4)
                })
                .collect()
        };
        assert_eq!(changed(&dynamic_hits), reference_changed(dynamic, initial));
        report("dynamic", &compare(&dynamic_hits, dynamic));
        println!("set_points changed rays: {:?}", changed(&dynamic_hits));
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn tight_max_t_cases_match_reference() {
        let session = session();
        let query = query(&session);
        let fixture = load();
        let rays = rays(&fixture);
        for name in ["tight_above", "tight_below"] {
            let case = fixture.case(name);
            let hits = query
                .query(
                    &fixture.initial_mesh(),
                    &geometry(case),
                    &rays,
                    &case.max_t,
                    true,
                )
                .expect("tight query");
            report(name, &compare(&hits, case));
            for (index, hit) in hits.iter().enumerate() {
                if hit.geom_id >= 0 {
                    assert!(
                        hit.distance < case.max_t[index],
                        "{name} 射线 {index} 命中距离 {} 必须严格小于 max_t {}",
                        hit.distance,
                        case.max_t[index]
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn two_worlds_do_not_cross() {
        let session = session();
        let query = query(&session);
        let fixture = load();
        let rays = rays(&fixture);
        let initial = fixture.case("initial");
        let moved = fixture.case("moved");
        let mesh = fixture.initial_mesh();
        let world0_first = query
            .query(&mesh, &geometry(initial), &rays, &initial.max_t, true)
            .expect("world0 first");
        let world1 = query
            .query(&mesh, &geometry(moved), &rays, &moved.max_t, true)
            .expect("world1");
        let world0_second = query
            .query(&mesh, &geometry(initial), &rays, &initial.max_t, true)
            .expect("world0 second");
        assert_eq!(
            world0_first, world0_second,
            "网格查询必须无状态，不串 world"
        );
        report("world0", &compare(&world0_first, initial));
        report("world1", &compare(&world1, moved));
        for hit in &world0_first {
            assert!(hit.geom_id == -1 || hit.geom_id == initial.geom_id);
        }
        for hit in &world1 {
            assert!(hit.geom_id == -1 || hit.geom_id == moved.geom_id);
        }
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn degenerate_mesh_query_matches_reference() {
        let session = session();
        let query = query(&session);
        let fixture = load();
        let degenerate = &fixture.degenerate;
        let mesh = MeshBvh::build(MeshBvhInput {
            vertices: &degenerate.vertices,
            indices: &degenerate.indices,
        })
        .expect("退化网格构建不应 panic");
        let case = MeshCase {
            name: "degenerate".to_string(),
            vertices: "initial".to_string(),
            geom_id: degenerate.geom_id,
            cull_backfaces: degenerate.cull_backfaces,
            xpos: degenerate.xpos,
            xmat: degenerate.xmat,
            max_t: degenerate.max_t.clone(),
            hit: degenerate.hit.clone(),
            distance: degenerate.distance.clone(),
            normal: degenerate.normal.clone(),
            u: degenerate.u.clone(),
            v: degenerate.v.clone(),
            face: degenerate.face.clone(),
        };
        let rays: Vec<Ray> = degenerate
            .origins
            .iter()
            .zip(degenerate.directions.iter())
            .map(|(origin, direction)| Ray {
                origin: *origin,
                direction: *direction,
            })
            .collect();
        let hits = query
            .query(
                &mesh,
                &geometry(&case),
                &rays,
                &case.max_t,
                case.cull_backfaces,
            )
            .expect("退化网格查询");
        report("degenerate", &compare(&hits, &case));
    }

    #[test]
    #[ignore = "需要 GPU + NVRTC"]
    fn rejects_invalid_query_inputs() {
        let session = session();
        let query = query(&session);
        let fixture = load();
        let case = fixture.case("initial");
        let mesh = fixture.initial_mesh();
        let rays = rays(&fixture);
        assert!(
            query.query(&mesh, &geometry(case), &[], &[], true).is_err(),
            "空射线集应被拒绝"
        );
        assert!(
            query
                .query(&mesh, &geometry(case), &rays[..1], &[], true)
                .is_err(),
            "max_t 长度不符应被拒绝"
        );
        assert!(
            query
                .query(&mesh, &geometry(case), &rays[..1], &[-1.0], true)
                .is_err(),
            "负 max_t 应被拒绝"
        );
        assert!(
            query
                .query(&mesh, &geometry(case), &rays[..1], &[f32::NAN], true)
                .is_err(),
            "NaN max_t 应被拒绝"
        );
        let mut broken = geometry(case);
        broken.xpos[0] = f32::INFINITY;
        assert!(
            query
                .query(&mesh, &broken, &rays[..1], &[1.0], true)
                .is_err(),
            "非有限位姿应被拒绝"
        );
        let mut negative = geometry(case);
        negative.geom_id = -1;
        assert!(
            query
                .query(&mesh, &negative, &rays[..1], &[1.0], true)
                .is_err(),
            "负 geom_id 应被拒绝"
        );
        println!("rejection surface: empty rays / max_t / pose / geom_id");
    }
}
