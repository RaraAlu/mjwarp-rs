//! 借用矩阵消费者单元测试。

use super::*;
use crate::model::SparseMassFields;

#[cfg(feature = "cuda-probe")]
mod gpu;
#[cfg(feature = "cuda-probe")]
mod launcher;

pub(super) fn input(n: usize, split: bool) -> SparseMassInput {
    let mid = n.div_ceil(2);
    let parents: Vec<i32> = (0..n)
        .map(|i| {
            if i == 0 || (split && i == mid) {
                -1
            } else {
                i as i32 - 1
            }
        })
        .collect();
    let mut rowadr = Vec::new();
    let mut rownnz = Vec::new();
    let mut cols = Vec::new();
    for i in 0..n {
        rowadr.push(cols.len() as i32);
        let mut row = Vec::new();
        let mut j = i as i32;
        while j >= 0 {
            row.push(j);
            j = parents[j as usize];
        }
        row.reverse();
        rownnz.push(row.len() as i32);
        cols.extend(row);
    }
    SparseMassInput::new(SparseMassFields {
        dof_parentid: parents,
        m_rowadr: rowadr,
        m_rownnz: rownnz,
        m_colind: cols,
        tree_dofadr: if split { vec![0, mid as i32] } else { vec![0] },
        tree_dofnum: if split {
            vec![mid as i32, (n - mid) as i32]
        } else {
            vec![n as i32]
        },
    })
    .unwrap()
}

#[cfg(feature = "cuda-probe")]
pub(super) fn model(i: &SparseMassInput) -> crate::model::AttachedModelInput {
    use crate::model::{
        AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
        KinematicModelInput,
    };
    let n = i.nv();
    let nb = n + 1;
    let parents = &i.fields().dof_parentid;
    let mut body_parentid = vec![0];
    body_parentid.extend(parents.iter().map(|&p| p + 1));
    let mut body_jntadr = vec![-1];
    body_jntadr.extend(0..n as i32);
    let mut body_jntnum = vec![0];
    body_jntnum.extend(vec![1; n]);
    let mut body_pos = vec![0.0; 3];
    body_pos.extend([0.08, 0.01, 0.02].repeat(n));
    let k = KinematicModelInput::new(
        n,
        KinematicFields {
            qpos0: vec![0.0; n],
            body_parentid,
            body_jntadr,
            body_jntnum,
            body_pos,
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(nb),
            jnt_type: vec![3; n],
            jnt_bodyid: (1..nb as i32).collect(),
            jnt_qposadr: (0..n as i32).collect(),
            jnt_dofadr: (0..n as i32).collect(),
            jnt_pos: vec![0.0; 3 * n],
            jnt_axis: [0.0, 0.0, 1.0].repeat(n),
        },
    )
    .unwrap();
    let mut body_ipos = vec![0.0; 3];
    body_ipos.extend([0.03, 0.01, 0.0].repeat(n));
    let mut body_mass = vec![0.0];
    body_mass.extend(vec![0.01; n]);
    let mut body_inertia = vec![0.0; 3];
    body_inertia.extend(vec![0.001; 3 * n]);
    let inertial = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos,
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(nb),
            body_mass,
            body_inertia,
            dof_bodyid: (1..nb as i32).collect(),
            dof_jntid: (0..n as i32).collect(),
            dof_parentid: parents.clone(),
            dof_armature: vec![2.0; n],
            dof_damping: vec![0.0; n],
        },
    )
    .unwrap();
    AttachedModelInput::new(inertial, AttachedFields::default()).unwrap()
}

#[test]
fn host_maps_original_append_rows_to_actual_lower_crb_addresses() {
    for n in [3, 33, 65] {
        let i = input(n, true);
        let old = metadata(&i).unwrap();
        let s = packed(n + 1, &i.fields().dof_parentid, &i).unwrap();
        let count = old[n] as usize;
        assert_eq!(s.meta[0], n as i32);
        assert_eq!(s.meta[1], count as i32);
        assert_eq!(&s.meta[2..2 + n + 1 + count], &old[..n + 1 + count]);
        for row in 0..n {
            for k in old[row] as usize..old[row + 1] as usize {
                let col = old[n + 1 + k] as usize;
                assert_eq!(
                    s.meta[2 + n + 1 + count + k] as usize,
                    row.max(col) * n + row.min(col)
                );
            }
        }
    }
}
#[test]
fn host_rejects_wrong_source_topology_and_guarded_source_strides() {
    let complete = input(3, true);
    let mut compact = complete.fields().clone();
    compact.m_rownnz = vec![1; 3];
    compact.m_rowadr = vec![0, 1, 2];
    compact.m_colind = vec![0, 1, 2];
    let compact = SparseMassInput::new(compact).unwrap();
    assert!(packed(4, &complete.fields().dof_parentid, &compact).is_err());
    let i = input(33, true);
    assert!(packed(34, &input(33, false).fields().dof_parentid, &i).is_err());
    assert!(packed(34, &[], &i).is_err());
    assert!(packed(0, &i.fields().dof_parentid, &i).is_err());
    assert!(packed(usize::MAX, &i.fields().dof_parentid, &i).is_err());
    for n in [3, 33, 65] {
        let i = input(n, true);
        let s = packed(n + 1, &i.fields().dof_parentid, &i).unwrap();
        let (m, v) = layouts(n, 10 * s.nbody + n * n, 257).unwrap();
        assert_eq!(m.total_elements(), 257 * (10 * (n + 1) + n * n));
        assert_eq!(v.total_elements(), 257 * n);
    }
}
