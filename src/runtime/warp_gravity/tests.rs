use super::bytes::validate_ptx;
use super::receipt::validate_input;
use super::signature::expected_ptx_signature;
use super::{ENTRY, FrozenWarpGravityInput, checked_product};

fn input() -> FrozenWarpGravityInput {
    FrozenWarpGravityInput {
        worlds: 1,
        nbody: 2,
        nv: 1,
        opt_gravity: vec![0.0, -1.0, 0.0],
        body_parentid: vec![0, 0],
        body_rootid: vec![0, 1],
        body_mass: vec![0.0, 1.0],
        body_gravcomp: vec![0.0, 1.0],
        dof_bodyid: vec![1],
        body_isdofancestor: vec![0, 1],
        xipos: vec![0.0; 6],
        subtree_com: vec![0.0; 6],
        cdof: vec![0.0; 6],
    }
}

#[test]
fn validates_exact_frozen_input_shapes() {
    validate_input(&input()).unwrap();

    let mut zero_body = input();
    zero_body.nbody = 1;
    assert!(validate_input(&zero_body).is_err());

    let mut non_finite = input();
    non_finite.cdof[0] = f32::NAN;
    assert!(validate_input(&non_finite).is_err());

    let mut bad_topology = input();
    bad_topology.dof_bodyid[0] = 2;
    assert!(validate_input(&bad_topology).is_err());
}

#[test]
fn rejects_overflow_and_embedded_ptx_nul() {
    assert!(checked_product(usize::MAX, 2).is_err());
    let mut ptx = format!(
        ".version 8.8\n.target sm_89\n.address_size 64\n{}\n",
        expected_ptx_signature(ENTRY).join("\n")
    )
    .into_bytes();
    ptx.extend_from_slice(b"\0x");
    assert!(validate_ptx(&ptx, ENTRY, 89).is_err());
}
