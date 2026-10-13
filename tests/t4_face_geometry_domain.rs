//! 公开Model的线性面可达域。
//! 不执行位置、极分解或三角函数。
#![cfg(feature = "cuda-probe")]

use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, FlexFaceFields,
    FlexPositionFields, FlexPositionModelInput, InertialFields, InertialModelInput,
    KinematicFields, KinematicModelInput, MocapModelInput, TendonFields, TendonModelInput,
};

fn tendons() -> TendonModelInput {
    let kinematics = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![0.0; 3],
            body_quat: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let rigid = InertialModelInput::new(
        kinematics,
        InertialFields {
            body_ipos: vec![0.0; 3],
            body_iquat: vec![1.0, 0.0, 0.0, 0.0],
            body_mass: vec![0.0],
            body_inertia: vec![0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    let attached = AttachedModelInput::new(rigid, AttachedFields::default()).unwrap();
    let mocap = MocapModelInput::new(attached, vec![-1]).unwrap();
    let camlight = CamLightModelInput::new(mocap, CamLightFields::default()).unwrap();
    TendonModelInput::new(camlight, TendonFields::default()).unwrap()
}

fn positions(cells: [i32; 3]) -> FlexPositionFields {
    let nodes = (cells[0] + 1) * (cells[1] + 1) * (cells[2] + 1);
    FlexPositionFields {
        // 第一柔体占八个节点；第二柔体才提供面。
        flex_interp: vec![1, -1],
        flex_cellnum: [vec![1; 3], cells.to_vec()].concat(),
        flex_nodeadr: vec![0, 8],
        flex_nodenum: vec![8, nodes],
        flex_vertadr: vec![0, 1],
        flex_vertnum: vec![1, 1],
        flex_centered: vec![true; 2],
        flex_nodebodyid: vec![0; (nodes + 8) as usize],
        flex_vertbodyid: vec![-1; 2],
        flex_node: vec![0.0; 3 * (nodes + 8) as usize],
        flex_vert: vec![0.0; 6],
        flex_vert0: vec![0.5; 6],
    }
}

// 独立按六侧二维单元枚举；不调用产品私有face_nodes。
fn faces(cells: [i32; 3]) -> (FlexFaceFields, Vec<usize>) {
    let mut fields = FlexFaceFields::default();
    let mut sides = Vec::new();
    let mut local = 0;
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for side in 0..2 {
            sides.push(local as usize);
            for a in 0..cells[u] {
                for b in 0..cells[v] {
                    fields.flex_face_map.extend([1, local]);
                    for du in 0..2 {
                        for dv in 0..2 {
                            let mut point = [0; 3];
                            point[axis] = side * cells[axis];
                            point[u] = a + du;
                            point[v] = b + dv;
                            fields.flex_face.push(
                                8 + (point[0] * (cells[1] + 1) + point[1]) * (cells[2] + 1)
                                    + point[2],
                            );
                        }
                    }
                    fields.flex_face.extend([-1; 5]);
                    local += 1;
                }
            }
        }
    }
    (fields, sides)
}

fn input(cells: [i32; 3]) -> FlexPositionModelInput {
    FlexPositionModelInput::new(tendons(), positions(cells)).unwrap()
}

#[test]
fn accepts_all_six_sides_and_nonzero_global_offsets_on_64_grids() {
    let mut total = 0;
    let mut grids = 0;
    for x in 1..=4 {
        for y in 1..=4 {
            for z in 1..=4 {
                let (faces, sides) = faces([x, y, z]);
                let count = 2 * (x * y + x * z + y * z) as usize;
                assert_eq!(faces.nflexface(), count);
                assert_eq!(sides.len(), 6);
                let model = input([x, y, z]).with_faces(faces).unwrap();
                for row in model.faces().unwrap().flex_face.as_chunks::<9>().0 {
                    assert!(row[..4].iter().all(|&node| node >= 8));
                    assert_eq!(&row[4..], &[-1; 5]);
                }
                total += count;
                grids += 1;
            }
        }
    }
    assert_eq!((grids, total), (64, 2400));
}

#[test]
fn rejects_high_order_and_extra_or_missing_nodes_on_every_side() {
    let cells = [2, 3, 4];
    let (fields, sides) = faces(cells);
    let model = input(cells);
    let mut rejected = 0;
    for start in sides {
        for slot in 4..9 {
            let mut bad = fields.clone();
            bad.flex_face[9 * start + slot] = 8;
            assert!(model.clone().with_faces(bad).is_err());
            rejected += 1;
        }
        let mut bad = fields.clone();
        bad.flex_face[9 * start] = -1;
        assert!(model.clone().with_faces(bad).is_err());
        rejected += 1;
    }
    let mut quadratic = positions(cells);
    quadratic.flex_interp[1] = -2;
    assert!(FlexPositionModelInput::new(tendons(), quadratic).is_err());
    rejected += 1;
    assert_eq!(rejected, 37);
}

#[test]
fn rejects_outward_normal_reordering_and_noncanonical_sentinel() {
    let cells = [2, 3, 4];
    let (fields, sides) = faces(cells);
    let model = input(cells);
    // 两侧沿用同轴基；不能擅自翻转一侧来追求外法向。
    for side in [1, 3, 5] {
        let mut bad = fields.clone();
        let start = 9 * sides[side];
        bad.flex_face.swap(start + 1, start + 2);
        assert!(model.clone().with_faces(bad).is_err());
    }
    for side in sides {
        let mut bad = fields.clone();
        bad.flex_face[9 * side + 4] = -2;
        assert!(model.clone().with_faces(bad).is_err());
    }
}
