//! G09 plane-box 多接触/退化场景测试（独立文件以控制模块规模）。

use super::*;

fn plane_box_fixture() -> Value {
    serde_json::from_str(include_str!("../../../../../fixtures/plane-box/cases.json")).unwrap()
}

/// G09：plane-box 多接触/退化场景 vs 冻结上游与原生 f64 mj_collision。
#[test]
fn plane_box_contacts_match_frozen_upstream_and_native() {
    let fixture = plane_box_fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g09-p5"
    );
    let ngeom = fixture["narrowphase"]["ngeom"].as_u64().unwrap() as usize;
    let nworld = fixture["nworld"].as_u64().unwrap() as usize;
    let upstream_tolerance = fixture["tolerance"]["upstream_abs"].as_f64().unwrap();
    let native_tolerance = fixture["tolerance"]["native_abs"].as_f64().unwrap();

    let model = OwnedNarrowphase::new(&fixture["narrowphase"]);
    let mut positions = Vec::new();
    let mut matrices = Vec::new();
    for world in fixture["worlds"].as_array().unwrap() {
        positions.extend(vec3_flat(&world["geom_xpos"]));
        matrices.extend(vec9_flat(&world["geom_xmat"]));
    }
    let state = PrimitiveState {
        nworld,
        geom_xpos: &positions,
        geom_xmat: &matrices,
    };
    let candidate_value = &fixture["candidates"];
    let pairs = vec2_i32_list(&candidate_value["pairs"]);
    let pairids = vec2_i32_list(&candidate_value["pairids"]);
    let worldids = i32_list(&candidate_value["worldids"]);
    let candidates = BroadphaseOutput {
        ncollision: pairs.len(),
        pairs,
        pairids,
        worldids,
    };
    let contacts = primitive_narrowphase(&model.view(ngeom), &state, &candidates).unwrap();

    // 1) 冻结上游接触逐字段对照（按 (worldid, geom) 分组、组内保持生成顺序）。
    let fields = &fixture["contacts"]["fields"];
    let expected_count = fixture["contacts"]["count"].as_u64().unwrap() as usize;
    assert_eq!(contacts.len(), expected_count, "contact count");
    let expected_geom = vec2_i32_list(&fields["geom"]);
    let expected_worldid = i32_list(&fields["worldid"]);
    let expected_dist = f32_list(&fields["dist"]);
    let expected_pos = f32_list(&fields["pos"]);
    let expected_frame = f32_list(&fields["frame"]);
    let expected_includemargin = f32_list(&fields["includemargin"]);
    let expected_friction = f32_list(&fields["friction"]);
    let expected_solref = f32_list(&fields["solref"]);
    let expected_solimp = f32_list(&fields["solimp"]);
    let expected_adhesion = f32_list(&fields["adhesion"]);
    let expected_dim = i32_list(&fields["dim"]);
    let expected_type = i32_list(&fields["type"]);
    let mut expected_groups: BTreeMap<(i32, i32, i32), Vec<usize>> = BTreeMap::new();
    for index in 0..expected_count {
        expected_groups
            .entry(contact_key(expected_worldid[index], expected_geom[index]))
            .or_default()
            .push(index);
    }
    let mut product_groups: BTreeMap<(i32, i32, i32), Vec<usize>> = BTreeMap::new();
    for (index, contact) in contacts.iter().enumerate() {
        product_groups
            .entry(contact_key(contact.worldid, contact.geom))
            .or_default()
            .push(index);
    }
    assert_eq!(product_groups.len(), expected_groups.len(), "geom pair set");
    let mut contact_max = 0.0_f64;
    for (key, product_indices) in &product_groups {
        let expected_indices = expected_groups
            .get(key)
            .unwrap_or_else(|| panic!("product pair not in upstream: {key:?}"));
        assert_eq!(
            product_indices.len(),
            expected_indices.len(),
            "pair {key:?} contact count"
        );
        for (product_index, expected_index) in product_indices.iter().zip(expected_indices.iter()) {
            let contact = &contacts[*product_index];
            let index = *expected_index;
            contact_max = contact_max.max(max_abs(&[contact.dist], &[expected_dist[index]]));
            contact_max = contact_max.max(max_abs(
                &contact.pos,
                &expected_pos[index * 3..index * 3 + 3],
            ));
            contact_max = contact_max.max(max_abs(
                &contact.frame,
                &expected_frame[index * 9..index * 9 + 9],
            ));
            contact_max = contact_max.max(max_abs(
                &[contact.includemargin],
                &[expected_includemargin[index]],
            ));
            contact_max = contact_max.max(max_abs(
                &contact.friction,
                &expected_friction[index * 5..index * 5 + 5],
            ));
            contact_max = contact_max.max(max_abs(
                &contact.solref,
                &expected_solref[index * 2..index * 2 + 2],
            ));
            contact_max = contact_max.max(max_abs(
                &contact.solimp,
                &expected_solimp[index * 5..index * 5 + 5],
            ));
            contact_max =
                contact_max.max(max_abs(&[contact.adhesion], &[expected_adhesion[index]]));
            assert_eq!(contact.dim, expected_dim[index]);
            assert_eq!(contact.contact_type, expected_type[index]);
            for value in contact.friction {
                assert!(value >= MJ_MINMU);
            }
        }
    }
    assert!(
        contact_max <= upstream_tolerance,
        "plane-box upstream delta {contact_max:.3e}"
    );

    // 2) 世界接触数量（多接触退化：4/2/1/0/4/4/0）与原生 f64 mj_collision 一级判定。
    let native = fixture["native"]["worlds"].as_array().unwrap();
    assert_eq!(native.len(), nworld);
    let mut native_max = 0.0_f64;
    let mut world_counts = Vec::new();
    for (worldid, world) in native.iter().enumerate() {
        let native_contacts = &world["contacts"];
        let ncon = native_contacts["ncon"].as_u64().unwrap() as usize;
        world_counts.push(ncon);
        let mut product_rows: Vec<(f32, [f32; 3], [f32; 9])> = contacts
            .iter()
            .filter(|contact| contact.worldid as usize == worldid)
            .map(|contact| (contact.dist, contact.pos, contact.frame))
            .collect();
        assert_eq!(product_rows.len(), ncon, "world {worldid} contact count");
        let native_dist = f32_list(&native_contacts["dist"]);
        let native_pos = f32_list(&native_contacts["pos"]);
        let native_frame = f32_list(&native_contacts["frame"]);
        let mut native_rows: Vec<(f32, [f32; 3], [f32; 9])> = (0..ncon)
            .map(|index| {
                (
                    native_dist[index],
                    native_pos[index * 3..index * 3 + 3].try_into().unwrap(),
                    native_frame[index * 9..index * 9 + 9].try_into().unwrap(),
                )
            })
            .collect();
        product_rows.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1[0].total_cmp(&right.1[0]))
                .then_with(|| left.1[1].total_cmp(&right.1[1]))
                .then_with(|| left.1[2].total_cmp(&right.1[2]))
        });
        native_rows.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1[0].total_cmp(&right.1[0]))
                .then_with(|| left.1[1].total_cmp(&right.1[1]))
                .then_with(|| left.1[2].total_cmp(&right.1[2]))
        });
        for (product, native) in product_rows.iter().zip(native_rows.iter()) {
            native_max = native_max.max((f64::from(product.0) - f64::from(native.0)).abs());
            native_max = native_max.max(max_abs(&product.1, &native.1));
            native_max = native_max.max(max_abs(&product.2, &native.2));
        }
    }
    assert_eq!(
        world_counts,
        vec![4, 2, 1, 0, 4, 4, 0],
        "multi-contact 场景接触数"
    );
    assert!(
        native_max <= native_tolerance,
        "plane-box native delta {native_max:.3e}"
    );
    println!(
        "plane-box contacts={} upstream_max={contact_max:.3e} native_max={native_max:.3e} counts={world_counts:?}",
        contacts.len()
    );
}
