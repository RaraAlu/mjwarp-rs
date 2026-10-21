use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn handles_coincident_reversed_static_and_same_body_edges() {
    let s = TransferSession::new(0).unwrap();
    let b = base();
    let bp = floats(&b["model"], "body_pos");
    let mut verts = Vec::from(&bp[3..6]);
    verts.extend([0.0; 3]);
    verts.extend([1.0, 0.0, 0.0]);
    verts.extend([bp[3], bp[4] + 2.0, bp[5]]);
    let f = FlexPositionFields {
        flex_interp: vec![0],
        flex_cellnum: vec![1; 3],
        flex_nodeadr: vec![0],
        flex_nodenum: vec![0],
        flex_vertadr: vec![0],
        flex_vertnum: vec![4],
        flex_centered: vec![false],
        flex_vertbodyid: vec![0, 1, 1, 0],
        flex_vert: verts,
        flex_vert0: vec![0.0; 12],
        ..Default::default()
    };
    let edges = FlexEdgeFields {
        flex_edgeadr: vec![0],
        flex_edgenum: vec![5],
        flex_edge: vec![0, 1, 3, 2, 2, 3, 1, 2, 0, 3],
        flexedge_j_rowadr: vec![0, 1, 2, 3, 0],
        flexedge_j_rownnz: vec![1, 1, 1, 1, 0],
        flexedge_j_colind: vec![0; 4],
    };
    let input = FlexPositionModelInput::new(tendons(&b), f)
        .unwrap()
        .with_edges(edges)
        .unwrap();
    let p = KinematicsPlan::with_flex_positions(
        &s,
        input,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut d = p.create_data(513).unwrap();
    let mut qvel = vec![0.0; 65];
    qvel[0] = 1.7;
    for w in 0..513 {
        d.write_world_qvel(w, &qvel).unwrap();
    }
    p.update(&mut d).unwrap();
    let out = d.readback().unwrap();
    for w in 0..513 {
        let f = out.flex_edges().unwrap().world(w).unwrap();
        assert_eq!(f.flexedge_length[0], 0.0);
        assert_eq!(f.flexedge_length[1], f.flexedge_length[2]);
        assert_eq!(f.flexedge_velocity[0], 0.0);
        // Reversing endpoints also reverses direction: length gradient stays equal.
        assert_eq!(f.flexedge_jacobian[1], f.flexedge_jacobian[2]);
        assert!(f.flexedge_jacobian[1].abs() > 0.01);
        assert!((f.flexedge_velocity[1] - 1.7 * f.flexedge_jacobian[1]).abs() < 1e-6);
        assert!(f.flexedge_jacobian[3].abs() < 1e-6);
        assert_eq!(f.flexedge_velocity[4], 0.0);
        assert!((f.flexedge_length[4] - 2.0).abs() < 1e-6);
    }
}
