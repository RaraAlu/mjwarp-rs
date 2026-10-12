//! G03诊断专用；不改变产品输入或输出。
use super::*;
use crate::model::{InertialFields, KinematicFields, KinematicModelInput};
use serde_json::{Value, json};

fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}

fn model(v: &Value) -> InertialModelInput {
    let m = &v["model"];
    let ints = |key: &str| {
        m[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
            .collect()
    };
    let k = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: floats(m, "qpos0"),
            body_parentid: ints("body_parentid"),
            body_jntadr: ints("body_jntadr"),
            body_jntnum: ints("body_jntnum"),
            body_pos: floats(m, "body_pos"),
            body_quat: floats(m, "body_quat"),
            jnt_type: ints("jnt_type"),
            jnt_bodyid: ints("jnt_bodyid"),
            jnt_qposadr: ints("jnt_qposadr"),
            jnt_dofadr: ints("jnt_dofadr"),
            jnt_pos: floats(m, "jnt_pos"),
            jnt_axis: floats(m, "jnt_axis"),
        },
    )
    .unwrap();
    InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: floats(m, "body_ipos"),
            body_iquat: floats(m, "body_iquat"),
            body_mass: floats(m, "body_mass"),
            body_inertia: floats(m, "body_inertia"),
            dof_bodyid: ints("dof_bodyid"),
            dof_jntid: ints("dof_jntid"),
            dof_parentid: ints("dof_parentid"),
            dof_armature: floats(m, "dof_armature"),
            dof_damping: floats(m, "dof_damping"),
        },
    )
    .unwrap()
}

#[test]
#[ignore = "diagnostic only: requires frozen offline trace, NVIDIA driver and NVRTC"]
fn trace_failed_state_by_matrix_factor_and_solve() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::var_os("MJWARP_G03_TRACE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/g03-first-difference"));
    std::fs::create_dir_all(&out).unwrap();
    let upstream: Value = serde_json::from_str(include_str!(
        "../../fixtures/mjwarp-mass-solve/first-difference/upstream.json"
    ))
    .unwrap();
    let native: Value =
        serde_json::from_str(include_str!("../../fixtures/mass-solve/mixed-joints.json")).unwrap();
    let base: Value =
        serde_json::from_str(include_str!("../../fixtures/mass-matrix/mixed-joints.json")).unwrap();
    let m = model(&base);
    let s = TransferSession::new(0).unwrap();
    let q: Vec<_> = native["cases"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| floats(c, "qpos"))
        .collect();
    let rhs: Vec<_> = native["cases"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| floats(c, "rhs"))
        .collect();
    let fk = crate::physics::probe_kinematics(&s, &m, 8, &q).unwrap();
    let com = crate::physics::probe_com_position(&s, &m, 8, &q).unwrap();
    let (state, matrix) = crate::physics::mass_matrix::mass_matrix_device(&s, &m, 8, &q).unwrap();
    let n = m.kinematics().nv();
    let nb = m.kinematics().nbody();
    let layout = MassSolveLayout::new(8, n, 3).unwrap();
    let blocks = factor_blocks(&m.fields().dof_parentid);
    let solved = solve_device(&s, nb, layout, &state, &rhs, blocks.clone()).unwrap();
    let mut staged = vec![-131072.0; 8 * (10 * nb + n * n) + 8];
    for w in 0..8 {
        let start = 4 + w * (10 * nb + n * n);
        staged[start..start + 10 * nb].fill(0.0);
        staged[start + 10 * nb..start + 10 * nb + n * n]
            .copy_from_slice(&floats(&upstream["rows"][w], "matrix"));
    }
    let same_matrix =
        solve_device(&s, nb, layout, &s.upload(&staged).unwrap(), &rhs, blocks).unwrap();
    // 仅诊断内核跳过分解，复用产品回代段。
    // 产品入口不读取参考，也不采用此替换。
    let start = MASS_SOLVE_CUDA.find("  for(int block=0;").unwrap();
    let end = MASS_SOLVE_CUDA.find("  if(n==0) return;").unwrap();
    let mut source = MASS_SOLVE_CUDA.to_owned();
    source.replace_range(start..end, "  for(unsigned i=0;i<n*n;i++) ld[i]=mat[i];\n");
    for w in 0..8 {
        let start = 4 + w * (10 * nb + n * n) + 10 * nb;
        staged[start..start + n * n].copy_from_slice(&floats(&upstream["rows"][w], "factor"));
    }
    let kernel =
        crate::runtime::SynchronousKernel::compile(&s, &source, "rigid_mass_solve").unwrap();
    let metadata = s.upload(&[1, 0, n as i32, 1]).unwrap();
    let parameters = s.upload(&rhs).unwrap();
    let state = s.upload(&staged).unwrap();
    let mut values = vec![-131072.0; layout.guarded];
    let mut output = s.upload(&values).unwrap();
    // SAFETY: Same checked eight-world ABI; one 11-DOF Cholesky block.
    // Only the test replaces factorization. All buffers share the session.
    unsafe {
        kernel
            .launch(
                &metadata,
                &parameters,
                &state,
                &mut output,
                [nb as u32, n as u32, 3, 8],
            )
            .unwrap();
    }
    output.read_range_into(0, &mut values).unwrap();
    check_result(layout, &values).unwrap();
    let same_factor = MassSolveOutput {
        layout,
        values,
        blocks: factor_blocks(&m.fields().dof_parentid),
    };
    let mut rows = Vec::new();
    for w in 0..8 {
        let f = fk.world(w).unwrap();
        let c = com.world(w).unwrap();
        let a = matrix.world(w).unwrap();
        let x = solved.world(w).unwrap();
        rows.push(json!({"xpos":f.xpos,"xquat":f.xquat,"xmat":f.xmat,
            "xipos":f.xipos,"ximat":f.ximat,"xanchor":f.xanchor,"xaxis":f.xaxis,
            "subtree_com":c.subtree_com,"cinert":c.cinert,"cdof":c.cdof,
            "crb":a.crb,"matrix":a.matrix,"factor":x.ld,"solution":x.solution,
            "same_matrix_factor":same_matrix.world(w).unwrap().ld,
            "same_matrix_solution":same_matrix.world(w).unwrap().solution,
            "same_factor_solution":same_factor.world(w).unwrap().solution}));
    }
    std::fs::write(
        out.join("candidate.json"),
        serde_json::to_vec_pretty(&json!({"rows":rows})).unwrap(),
    )
    .unwrap();
}
