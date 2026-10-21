use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn keeps_optional_empty_old_routes_and_tendon_wake_composition() {
    let s = TransferSession::new(0).unwrap();
    let b = base();
    let e = fixture();
    let mut inactive = e.clone();
    inactive["model"] = inactive["inactive_model"].clone();
    let inactive_plan = plan(&s, &b, &inactive);
    let mut inactive_data = inactive_plan.create_data(5).unwrap();
    inactive_plan.update(&mut inactive_data).unwrap();
    let inactive_out = inactive_data.readback().unwrap();
    for w in 0..5 {
        let f = inactive_out.flex_edges().unwrap().world(w).unwrap();
        compare(f.flexedge_length, &e["cases"][0]["flexedge_length"]);
        assert_eq!(f.flexedge_jacobian.len(), 10);
        assert!(f.flexedge_jacobian.iter().all(|&v| v == 0.0));
        assert!(f.flexedge_velocity.iter().all(|&v| v == 0.0));
    }
    let old = KinematicsPlan::with_flex_positions(
        &s,
        FlexPositionModelInput::new(tendons(&b), positions(&b)).unwrap(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut d = old.create_data(2).unwrap();
    old.update(&mut d).unwrap();
    assert!(d.readback().unwrap().flex_edges().is_none());
    assert!(d.write_world_qvel(0, &[0.0; 65]).is_err());
    let empty = FlexPositionModelInput::new(tendons(&b), FlexPositionFields::default())
        .unwrap()
        .with_edges(FlexEdgeFields::default())
        .unwrap();
    let p = KinematicsPlan::with_flex_positions(
        &s,
        empty,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut d = p.create_data(5).unwrap();
    d.write_qvel(&[0.0; 325]).unwrap();
    p.update(&mut d).unwrap();
    assert!(
        d.readback()
            .unwrap()
            .flex_edges()
            .unwrap()
            .world(4)
            .unwrap()
            .flexedge_length
            .is_empty()
    );
    let parents = ids(&b["model"], "body_parentid");
    let num = ids(&b["model"], "body_jntnum");
    let mut trees = vec![-1; parents.len()];
    let mut count = 0;
    for i in 1..trees.len() {
        trees[i] = trees[parents[i] as usize];
        if trees[i] < 0 && num[i] > 0 {
            trees[i] = count;
            count += 1;
        }
    }
    let wake = TendonWakeFields {
        body_treeid: trees,
        tendon_limited: vec![false; 2],
        tendon_range: ParameterBatch::new(1, 4, vec![-10.0, 10.0, -10.0, 10.0]).unwrap(),
        tendon_margin: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
        sleep_enabled: true,
        island_disabled: false,
    };
    let model = TendonWakeModelInput::with_flex_positions(input(&b, &e), wake).unwrap();
    assert!(model.flex_edges().is_some());
    let p = KinematicsPlan::with_tendon_wake(
        &s,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut d = p.create_data(5).unwrap();
    for w in 0..5 {
        write(&mut d, w, &e["cases"][w]);
    }
    p.update(&mut d).unwrap();
    let out = d.readback().unwrap();
    for w in 0..5 {
        native(&out, w, &e["cases"][w]);
        assert_eq!(
            out.sleep_trees().unwrap().world(w).unwrap().ntree_awake,
            count
        );
    }
}
