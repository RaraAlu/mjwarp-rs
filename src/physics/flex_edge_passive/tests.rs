use super::*;

fn fields() -> Arc<FlexEdgeFields> {
    Arc::new(FlexEdgeFields {
        flex_edgeadr: vec![0, 2],
        flex_edgenum: vec![2, 1],
        flex_edge: vec![0, 1, 0, 1, 0, 1],
        flexedge_j_rowadr: vec![0, 2, 0],
        flexedge_j_rownnz: vec![2, 1, 0],
        flexedge_j_colind: vec![0, 2, 1],
    })
}

#[test]
fn host_checks_native_dimensions_and_derives_flex_ids() {
    let input = FlexEdgePassiveInput {
        length0: vec![1.0, 2.0, f32::NAN],
        stiffness: vec![3.0, 4.0],
        damping: vec![5.0, 6.0],
        rigid: vec![false, true, false],
    };
    let s = spec(3, fields(), input.clone()).unwrap();
    assert_eq!(s.flexid, [0, 0, 1]);
    assert_eq!((s.ne, s.nf, s.nnz, s.stride), (3, 2, 3, 9));
    for field in 0..4 {
        let mut bad = input.clone();
        match field {
            0 => {
                bad.length0.pop();
            }
            1 => {
                bad.stiffness.pop();
            }
            2 => {
                bad.damping.pop();
            }
            _ => {
                bad.rigid.pop();
            }
        }
        assert!(spec(3, fields(), bad).is_err());
    }
    assert!(layout(0, 3).is_err());
    assert!(layout(usize::MAX, 3).is_err());
}
