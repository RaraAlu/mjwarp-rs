//! 常驻混合装配偏移与惯性测试。

use super::plan::topology;
use super::tests_support::*;

#[test]
fn assembled_offsets_preserve_native_fields_and_single_world_inertia() {
    let model = input::model(false);
    let k = model.rigid().kinematics();
    let f = model.rigid().fields();
    assert_eq!((k.nv(), k.nq(), k.nbody(), k.njnt()), (68, 68, 69, 68));
    assert_eq!(&k.fields().body_parentid[66..], &[0, 66, 0]);
    assert_eq!(&k.fields().jnt_bodyid[65..], &[66, 67, 68]);
    assert_eq!(&f.dof_bodyid[65..], &[66, 67, 68]);
    assert_eq!(&f.dof_jntid[65..], &[65, 66, 67]);
    assert_eq!(&f.dof_parentid[65..], &[-1, 65, -1]);
    assert_eq!(&f.body_mass[66..], &[2.0, 3.0, 4.0]);
    assert_eq!(f.body_mass[0], 0.0);
    assert_eq!(&f.body_inertia[..3], &[0.0; 3]);
    let refs = input::reference();
    assert_eq!(refs["cases"].as_array().unwrap().len(), 2);
    assert_eq!(input::qpos(0).len(), 68);
    assert_eq!(input::rhs(&refs, 0, 3).len(), 204);
}
#[test]
fn bridge_topology_rejects_different_nv_and_same_nv_wrong_ancestry() {
    let model = input::model(false);
    let sparse = input::input(false);
    topology(&model, &sparse).unwrap();
    assert!(topology(&model, &input::input(true)).is_err());
    let other = input::model(true);
    assert_eq!(other.rigid().kinematics().nv(), 68);
    assert!(topology(&other, &sparse).is_err());
}
