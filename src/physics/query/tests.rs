//! G28 原语射线查询测试：冻结上游参考、批量等价与拒绝面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/ray-query/ray-cases.json")).unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

fn build_geoms(value: &Value) -> Vec<GeomPrimitive> {
    let arrays = &value["model_arrays"];
    let data = &value["frozen_data"];
    let geom_type: Vec<i32> = arrays["geom_type"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect();
    let geom_size = arrays["geom_size"].as_array().unwrap();
    let geom_xpos = data["geom_xpos"].as_array().unwrap();
    let geom_xmat = data["geom_xmat"].as_array().unwrap();
    (0..geom_type.len())
        .map(|index| GeomPrimitive {
            geom_type: geom_type[index],
            pos: f32_list(&geom_xpos[index]).try_into().unwrap(),
            size: f32_list(&geom_size[index]).try_into().unwrap(),
            mat: f32_list(&geom_xmat[index]).try_into().unwrap(),
        })
        .collect()
}

fn build_rays(value: &Value) -> Vec<Ray> {
    value["rays"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| Ray {
            pnt: f32_list(&item["pnt"]).try_into().unwrap(),
            vec: f32_list(&item["vec"]).try_into().unwrap(),
        })
        .collect()
}

#[test]
fn primitive_rays_match_frozen_upstream_reference() {
    let value = fixture();
    assert_eq!(
        value["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g28-p7"
    );
    let geom_list = build_geoms(&value);
    let query_list = build_rays(&value);
    let hits = crate::physics::rays(&geom_list, &query_list).unwrap();
    let expected = value["rays"].as_array().unwrap();
    assert_eq!(hits.len(), expected.len());
    let tolerance = value["tolerance"]["dist_abs"].as_f64().unwrap() as f32;
    let normal_tolerance = value["tolerance"]["normal_abs"].as_f64().unwrap() as f32;
    let mut hit_geoms = Vec::new();
    for (index, (hit, expected)) in hits.iter().zip(expected.iter()).enumerate() {
        let expected_dist = expected["dist"].as_f64().unwrap() as f32;
        let expected_geom = expected["geom_id"].as_i64().unwrap() as i32;
        assert_eq!(hit.geom_id(), expected_geom, "ray {index} geom_id");
        assert!(
            (hit.dist() - expected_dist).abs() <= tolerance,
            "ray {index} dist product={} expected={expected_dist}",
            hit.dist()
        );
        let expected_normal = f32_list(&expected["normal"]);
        for (component, target) in hit.normal().iter().zip(expected_normal.iter()) {
            assert!(
                (component - target).abs() <= normal_tolerance,
                "ray {index} normal component product={component} expected={target}"
            );
        }
        if expected_geom < 0 {
            assert_eq!(hit.dist(), -1.0);
            assert_eq!(hit.normal(), [0.0; 3]);
        } else {
            hit_geoms.push(expected_geom);
        }
    }
    let mut covered = hit_geoms;
    covered.sort_unstable();
    covered.dedup();
    assert_eq!(
        covered,
        vec![0, 1, 2, 3, 4, 5],
        "six primitive types covered"
    );
    assert_eq!(hits.iter().filter(|hit| hit.geom_id() < 0).count(), 2);
}

#[test]
fn batch_rays_equal_single_ray_queries() {
    let value = fixture();
    let geom_list = build_geoms(&value);
    let query_list = build_rays(&value);
    let batch = crate::physics::rays(&geom_list, &query_list).unwrap();
    for (index, query) in query_list.iter().enumerate() {
        assert_eq!(
            crate::physics::ray(&geom_list, *query).unwrap(),
            batch[index],
            "ray {index}"
        );
    }
}

#[test]
fn nearest_hit_wins_on_multi_geom_ray() {
    let value = fixture();
    let geom_list = build_geoms(&value);
    let query_list = build_rays(&value);
    let hit = crate::physics::ray(&geom_list, query_list[8]).unwrap();
    assert_eq!(hit.geom_id(), 2);
    assert!((hit.dist() - 0.8).abs() <= 1e-6);
}

#[test]
fn rejects_unsupported_and_degenerate_geoms() {
    let value = fixture();
    let query = build_rays(&value)[0];
    let mut geom_list = build_geoms(&value);
    geom_list[0].geom_type = 7; // mjGEOM_MESH
    assert!(matches!(
        crate::physics::ray(&geom_list, query),
        Err(QueryError::UnsupportedGeom {
            index: 0,
            geom_type: 7
        })
    ));
    let mut geom_list = build_geoms(&value);
    geom_list[1].size[0] = 0.0;
    assert!(matches!(
        crate::physics::ray(&geom_list, query),
        Err(QueryError::DegenerateSize { index: 1 })
    ));
}
