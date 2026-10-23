//! G28@P8 多世界批量射线查询测试：冻结上游参考与校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../fixtures/ray-query/batch-cases.json"
    ))
    .unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

fn i32_list(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect()
}

#[test]
fn batch_rays_match_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g28-p8"
    );
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let ngeom = value["model"]["ngeom"].as_u64().unwrap() as usize;
    let worlds = value["worlds"].as_array().unwrap();
    assert_eq!(worlds.len(), nworld);
    let mut pos = Vec::new();
    let mut mat = Vec::new();
    for world in worlds {
        pos.extend(f32_list(&world["geom_xpos"]));
        mat.extend(f32_list(&world["geom_xmat"]));
    }
    let geom_type = value["model_arrays"]["geom_type"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect::<Vec<_>>();
    let size = value["model_arrays"]["geom_size"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(f32_list)
        .collect::<Vec<_>>();
    let rays: Vec<Ray> = value["rays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| Ray {
            pnt: f32_list(&item["pnt"]).try_into().unwrap(),
            vec: f32_list(&item["vec"]).try_into().unwrap(),
        })
        .collect();
    let hits = rays_batch(
        WorldGeoms {
            nworld,
            ngeom,
            geom_type: &geom_type,
            size: &size,
            pos: &pos,
            mat: &mat,
        },
        &rays,
    )
    .unwrap();
    assert_eq!(hits.len(), nworld);
    let tolerance = value["tolerance"]["dist_abs"].as_f64().unwrap() as f32;
    let normal_tolerance = value["tolerance"]["normal_abs"].as_f64().unwrap() as f32;
    let mut miss_count = 0;
    for (world_index, world) in worlds.iter().enumerate() {
        let expected = world["hits"].as_array().unwrap();
        assert_eq!(hits[world_index].len(), expected.len());
        for (ray_index, (hit, expected)) in
            hits[world_index].iter().zip(expected.iter()).enumerate()
        {
            let expected_dist = expected["dist"].as_f64().unwrap() as f32;
            let expected_geom = expected["geom_id"].as_i64().unwrap() as i32;
            assert_eq!(
                hit.geom_id(),
                expected_geom,
                "world {world_index} ray {ray_index} geom_id"
            );
            assert!(
                (hit.dist() - expected_dist).abs() <= tolerance,
                "world {world_index} ray {ray_index} dist product={} expected={expected_dist}",
                hit.dist()
            );
            let expected_normal = f32_list(&expected["normal"]);
            for (component, target) in hit.normal().iter().zip(expected_normal.iter()) {
                assert!(
                    (component - target).abs() <= normal_tolerance,
                    "world {world_index} ray {ray_index} normal product={component} expected={target}"
                );
            }
            if expected_geom < 0 {
                miss_count += 1;
                assert_eq!(hit.dist(), -1.0);
                assert_eq!(hit.normal(), [0.0; 3]);
            }
        }
    }
    assert!(miss_count >= 2, "batch must cover no-hit rays");
}

#[test]
fn rejects_invalid_world_shapes() {
    let value = fixture();
    let nworld = value["nworld"].as_u64().unwrap() as usize;
    let ngeom = value["model"]["ngeom"].as_u64().unwrap() as usize;
    let geom_type = i32_list(&value["model_arrays"]["geom_type"]);
    let size = value["model_arrays"]["geom_size"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(f32_list)
        .collect::<Vec<_>>();
    let empty: [f32; 0] = [];
    let rays = [Ray {
        pnt: [0.0, 0.0, 0.5],
        vec: [1.0, 0.0, 0.0],
    }];
    let error = rays_batch(
        WorldGeoms {
            nworld,
            ngeom,
            geom_type: &geom_type,
            size: &size,
            pos: &[],
            mat: &[],
        },
        &rays,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        QueryError::InvalidLength { field: "pos", .. }
    ));
    let error = rays_batch(
        WorldGeoms {
            nworld: 0,
            ngeom,
            geom_type: &geom_type,
            size: &size,
            pos: &empty,
            mat: &empty,
        },
        &rays,
    )
    .unwrap_err();
    assert!(matches!(error, QueryError::DegenerateSize { .. }));
}
