// Test-only assembly of two frozen independent-root DTOs, not native compilation.
use super::engine;
use engine::model::{
    AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
    KinematicModelInput, SparseMassInput,
};
use serde_json::Value;
mod large {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/sparse_mass_model.rs"
    ));
}
pub fn reference() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/resident-mixed-mass/reference.json"
    )))
    .unwrap()
}
fn small() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/mass-matrix/armature-chain.json"
    )))
    .unwrap()
}
fn small_model() -> InertialModelInput {
    let v = small();
    let f = &v["model"];
    let ints = |key: &str| {
        f[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i32)
            .collect()
    };
    let floats = |key: &str| {
        f[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect()
    };
    let k = KinematicModelInput::new(
        3,
        KinematicFields {
            qpos0: floats("qpos0"),
            body_parentid: ints("body_parentid"),
            body_jntadr: ints("body_jntadr"),
            body_jntnum: ints("body_jntnum"),
            body_pos: floats("body_pos"),
            body_quat: floats("body_quat"),
            jnt_type: ints("jnt_type"),
            jnt_bodyid: ints("jnt_bodyid"),
            jnt_qposadr: ints("jnt_qposadr"),
            jnt_dofadr: ints("jnt_dofadr"),
            jnt_pos: floats("jnt_pos"),
            jnt_axis: floats("jnt_axis"),
        },
    )
    .unwrap();
    InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: floats("body_ipos"),
            body_iquat: floats("body_iquat"),
            body_mass: floats("body_mass"),
            body_inertia: floats("body_inertia"),
            dof_bodyid: ints("dof_bodyid"),
            dof_jntid: ints("dof_jntid"),
            dof_parentid: ints("dof_parentid"),
            dof_armature: floats("dof_armature"),
            dof_damping: floats("dof_damping"),
        },
    )
    .unwrap()
}
pub fn model(connect_roots: bool) -> AttachedModelInput {
    let base = large::reference();
    let big = large::model(&base["models"][0]);
    let big = big.rigid();
    let small = small_model();
    let mut k = big.kinematics().fields().clone();
    let b = small.kinematics().fields();
    let bo = (big.kinematics().nbody() - 1) as i32;
    let jo = big.kinematics().njnt() as i32;
    let qo = big.kinematics().nq() as i32;
    let vo = big.kinematics().nv() as i32;
    k.qpos0.extend(&b.qpos0);
    k.body_parentid.extend(
        b.body_parentid[1..]
            .iter()
            .map(|&p| if p == 0 { 0 } else { p + bo }),
    );
    k.body_jntadr.extend(
        b.body_jntadr[1..]
            .iter()
            .map(|&p| if p < 0 { -1 } else { p + jo }),
    );
    k.body_jntnum.extend(&b.body_jntnum[1..]);
    k.body_pos.extend(&b.body_pos[3..]);
    k.body_quat.extend(&b.body_quat[4..]);
    k.jnt_type.extend(&b.jnt_type);
    k.jnt_bodyid.extend(b.jnt_bodyid.iter().map(|p| p + bo));
    k.jnt_qposadr.extend(b.jnt_qposadr.iter().map(|p| p + qo));
    k.jnt_dofadr.extend(b.jnt_dofadr.iter().map(|p| p + vo));
    k.jnt_pos.extend(&b.jnt_pos);
    k.jnt_axis.extend(&b.jnt_axis);
    let mut r = big.fields().clone();
    let f = small.fields();
    r.body_ipos.extend(&f.body_ipos[3..]);
    r.body_iquat.extend(&f.body_iquat[4..]);
    r.body_mass.extend(&f.body_mass[1..]);
    r.body_inertia.extend(&f.body_inertia[3..]);
    r.dof_bodyid.extend(f.dof_bodyid.iter().map(|p| p + bo));
    r.dof_jntid.extend(f.dof_jntid.iter().map(|p| p + jo));
    r.dof_parentid.extend(
        f.dof_parentid
            .iter()
            .map(|&p| if p < 0 { -1 } else { p + vo }),
    );
    r.dof_armature.extend(&f.dof_armature);
    r.dof_damping.extend(&f.dof_damping);
    if connect_roots {
        k.body_parentid[66] = 65;
        r.dof_parentid[65] = 64;
    }
    AttachedModelInput::new(
        InertialModelInput::new(KinematicModelInput::new(68, k).unwrap(), r).unwrap(),
        AttachedFields::default(),
    )
    .unwrap()
}
pub fn input(remove_compact: bool) -> SparseMassInput {
    let base = large::reference();
    let mut f = large::sparse(&base["models"][0]).fields().clone();
    f.dof_parentid.extend([-1, 65, -1]);
    f.m_rownnz.extend([1, 2, 1]);
    f.m_rowadr.extend([2145, 2146, 2148]);
    f.m_colind.extend([65, 65, 66, 67]);
    f.tree_dofadr.extend([65, 67]);
    f.tree_dofnum.extend([2, 1]);
    if remove_compact {
        f.dof_parentid.pop();
        f.m_rownnz.pop();
        f.m_rowadr.pop();
        f.m_colind.pop();
        f.tree_dofadr.pop();
        f.tree_dofnum.pop();
    }
    SparseMassInput::new(f).unwrap()
}
pub fn qpos(sample: usize) -> Vec<f32> {
    let base = large::reference();
    let mut p = large::floats(&base["models"][0]["cases"][sample], "qpos");
    p.extend(
        small()["cases"][sample]["qpos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32),
    );
    p
}
pub fn rhs(v: &Value, sample: usize, nr: usize) -> Vec<f32> {
    (0..nr * 68)
        .map(|i| {
            v["cases"][sample]["rhs"][(i / 68 % 3) * 68 + i % 68]
                .as_f64()
                .unwrap() as f32
        })
        .collect()
}
