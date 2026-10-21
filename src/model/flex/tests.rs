//! 柔体位置与边面字段单元测试。

use super::*;

fn fields() -> FlexPositionFields {
    FlexPositionFields {
        flex_interp: vec![0, 1],
        flex_cellnum: vec![1; 6],
        flex_nodeadr: vec![-1, 0],
        flex_nodenum: vec![0, 8],
        flex_vertadr: vec![0, 1],
        flex_vertnum: vec![1, 1],
        flex_centered: vec![true, true],
        flex_nodebodyid: vec![0; 8],
        flex_vertbodyid: vec![0, -1],
        flex_node: vec![0.0; 24],
        flex_vert: vec![0.0; 6],
        flex_vert0: vec![0.5; 6],
    }
}
#[test]
fn accepts_direct_linear_centered_and_empty_fields() {
    let mut f = fields();
    f.validate(1).unwrap();
    f.flex_centered.fill(false);
    f.flex_node[0] = -0.0;
    f.flex_vert[0] = 0.2;
    f.validate(1).unwrap();
    assert_eq!(f.flex_node[0].to_bits(), (-0.0f32).to_bits());
    FlexPositionFields::default().validate(1).unwrap();
}
#[test]
fn rejects_every_position_field_length_and_nonfinite_value() {
    for i in 0..9 {
        let mut f = fields();
        match i {
            0 => {
                f.flex_cellnum.pop();
            }
            1 => {
                f.flex_nodeadr.pop();
            }
            2 => {
                f.flex_nodenum.pop();
            }
            3 => {
                f.flex_vertadr.pop();
            }
            4 => {
                f.flex_vertnum.pop();
            }
            5 => {
                f.flex_centered.pop();
            }
            6 => {
                f.flex_node.pop();
            }
            7 => {
                f.flex_vert.pop();
            }
            _ => {
                f.flex_vert0.pop();
            }
        }
        assert!(
            matches!(f.validate(1), Err(InputError::LengthMismatch { .. })),
            "field {i}"
        );
    }
    for i in 0..3 {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut f = fields();
            let v = match i {
                0 => &mut f.flex_node,
                1 => &mut f.flex_vert,
                _ => &mut f.flex_vert0,
            };
            let last = v.len() - 1;
            v[last] = bad;
            assert!(matches!(f.validate(1),Err(InputError::NonFinite{index,..}) if index==last));
        }
    }
}
#[test]
fn rejects_unsafe_ranges_bodies_and_centered_offsets() {
    for i in 0..13 {
        let mut f = fields();
        match i {
            0 => f.flex_vertadr[1] = 0,
            1 => f.flex_vertadr[1] = 2,
            2 => f.flex_vertnum[0] = -1,
            3 => f.flex_vertnum[1] = i32::MAX,
            4 => f.flex_nodeadr[1] = -1,
            5 => f.flex_nodenum[1] = -1,
            6 => f.flex_nodenum[1] = 9,
            7 => f.flex_nodebodyid[7] = 1,
            8 => f.flex_nodebodyid[0] = -1,
            9 => f.flex_vertbodyid[0] = -1,
            10 => f.flex_vertbodyid[1] = -2,
            11 => f.flex_vert[0] = 1.0,
            _ => f.flex_node[23] = 1.0,
        }
        assert!(
            matches!(f.validate(1), Err(InputError::InvalidTopology { .. })),
            "case {i}"
        );
    }
    let mut f = fields();
    f.flex_interp[1] = 0;
    assert!(matches!(
        f.validate(1),
        Err(InputError::InvalidTopology {
            field: "flex_nodenum",
            ..
        })
    ));
    let mut f = fields();
    f.flex_vertnum[1] = 0;
    assert!(f.validate(1).is_err());
    let mut f = fields();
    f.flex_interp.clear();
    assert!(f.validate(1).is_err());
    let mut f = fields();
    f.flex_nodebodyid.push(0);
    f.flex_node.extend([0.0; 3]);
    assert!(f.validate(1).is_err());
    let mut f = fields();
    f.flex_vertbodyid.push(0);
    f.flex_vert.extend([0.0; 3]);
    f.flex_vert0.extend([0.0; 3]);
    assert!(f.validate(1).is_err());
}
#[test]
fn rejects_high_order_and_unsafe_linear_cells() {
    for mode in [-2, 2, i32::MAX] {
        let mut f = fields();
        f.flex_interp[1] = mode;
        assert!(f.validate(1).is_err());
    }
    for cell in [-1, 0, 2, i32::MAX] {
        let mut f = fields();
        f.flex_cellnum[3] = cell;
        assert!(f.validate(1).is_err());
    }
    for coord in [f32::MIN, f32::MAX, 2147483648.0] {
        let mut f = fields();
        f.flex_vert0[5] = coord;
        assert!(f.validate(1).is_err());
    }
    for coord in [-0.1, 0.0, 1.0, 1.1] {
        let mut f = fields();
        f.flex_vert0[5] = coord;
        f.validate(1).unwrap();
    }
}
#[test]
fn accepts_linear_shell_and_checks_canonical_faces() {
    let mut p = fields();
    p.flex_interp[1] = -1;
    p.validate(1).unwrap();
    let mut f = FlexFaceFields::default();
    for local in 0..6 {
        f.flex_face_map.extend([1, local]);
        f.flex_face.extend(face_nodes(&[1, 1, 1], local as usize).1);
        f.flex_face.extend([-1; 5]);
    }
    f.validate(&p).unwrap();
    for i in 0..f.flex_face.len() {
        let mut bad = f.clone();
        bad.flex_face[i] += 1;
        assert!(bad.validate(&p).is_err());
    }
    for i in 0..f.flex_face_map.len() {
        let mut bad = f.clone();
        bad.flex_face_map[i] += 1;
        assert!(bad.validate(&p).is_err());
    }
    FlexFaceFields::default()
        .validate(&FlexPositionFields::default())
        .unwrap();
    assert!(FlexFaceFields::default().validate(&p).is_err());
}
