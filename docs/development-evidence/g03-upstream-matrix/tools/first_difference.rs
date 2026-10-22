//! Offline first-difference localization: upstream frozen chain vs product probes.
//! Analysis-only tool; never part of the Rust product.
use mjwarp_rs::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
use mjwarp_rs::physics::{probe_com_position, probe_kinematics, probe_mass_matrix};
use mjwarp_rs::runtime::TransferSession;
use serde_json::{Value, json};
use std::fs;

fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key].as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}
fn integers(v: &Value, key: &str) -> Vec<i32> {
    v[key].as_array().unwrap().iter().map(|x| i32::try_from(x.as_i64().unwrap()).unwrap()).collect()
}
fn model(v: &Value) -> InertialModelInput {
    let m = &v["model"];
    let k = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: floats(m, "qpos0"),
            body_parentid: integers(m, "body_parentid"),
            body_jntadr: integers(m, "body_jntadr"),
            body_jntnum: integers(m, "body_jntnum"),
            body_pos: floats(m, "body_pos"),
            body_quat: floats(m, "body_quat"),
            jnt_type: integers(m, "jnt_type"),
            jnt_bodyid: integers(m, "jnt_bodyid"),
            jnt_qposadr: integers(m, "jnt_qposadr"),
            jnt_dofadr: integers(m, "jnt_dofadr"),
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
            dof_bodyid: integers(m, "dof_bodyid"),
            dof_jntid: integers(m, "dof_jntid"),
            dof_parentid: integers(m, "dof_parentid"),
            dof_armature: floats(m, "dof_armature"),
            dof_damping: floats(m, "dof_damping"),
        },
    )
    .unwrap()
}

#[derive(Default)]
struct Diff {
    compared: usize,
    different: usize,
    first_world: Option<usize>,
    first_index: Option<usize>,
    first_expected: Option<f32>,
    first_actual: Option<f32>,
}

fn compare(upstream: &[f32], local: &[f32], worlds: usize, diff: &mut Diff) {
    if worlds == 0 || upstream.is_empty() {
        return;
    }
    let per_world = upstream.len() / worlds;
    for world in 0..worlds {
        for i in 0..per_world {
            let at = world * per_world + i;
            if at >= upstream.len() || at >= local.len() {
                continue;
            }
            diff.compared += 1;
            if upstream[at].to_bits() != local[at].to_bits() {
                diff.different += 1;
                if diff.first_world.is_none() {
                    diff.first_world = Some(world);
                    diff.first_index = Some(i);
                    diff.first_expected = Some(upstream[at]);
                    diff.first_actual = Some(local[at]);
                }
            }
        }
    }
}

fn record(fields: &mut Vec<Value>, first_stage: &mut Option<&'static str>, stage: &'static str, field: &str, upstream: &[f32], local: &[f32], worlds: usize) {
    let mut diff = Diff::default();
    compare(upstream, local, worlds, &mut diff);
    if diff.different > 0 && first_stage.is_none() {
        *first_stage = Some(stage);
    }
    fields.push(json!({
        "stage": stage,
        "field": field,
        "compared": diff.compared,
        "different": diff.different,
        "first_world": diff.first_world,
        "first_index": diff.first_index,
        "first_expected": diff.first_expected,
        "first_actual": diff.first_actual,
    }));
}

fn snapshot(upstream: &Value, field: &str) -> Vec<f32> {
    upstream["snapshots"][field]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}

fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("repo root"));
    let upstream_path = std::path::PathBuf::from(std::env::args().nth(2).expect("upstream-matrix.json"));
    let output_path = std::path::PathBuf::from(std::env::args().nth(3).expect("output json"));
    let matrix_fixture: Value = serde_json::from_str(
        &fs::read_to_string(root.join("fixtures/mass-matrix/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let solve_fixture: Value = serde_json::from_str(
        &fs::read_to_string(root.join("fixtures/mass-solve/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let upstream: Value = serde_json::from_str(&fs::read_to_string(&upstream_path).unwrap()).unwrap();
    let worlds = 8usize;
    let m = model(&matrix_fixture);
    let cases = solve_fixture["cases"].as_array().unwrap();
    let q: Vec<f32> = (0..worlds).flat_map(|w| floats(&cases[w % 8], "qpos")).collect();
    let session = TransferSession::new(0).unwrap();
    let kin = probe_kinematics(&session, &m, worlds, &q).unwrap();
    let com = probe_com_position(&session, &m, worlds, &q).unwrap();
    let mm = probe_mass_matrix(&session, &m, worlds, &q).unwrap();

    let mut fields: Vec<Value> = Vec::new();
    let mut first_stage: Option<&'static str> = None;

    macro_rules! kin_vec {
        ($field:ident) => {{
            let mut out = Vec::new();
            for w in 0..worlds {
                out.extend_from_slice(kin.world(w).unwrap().$field);
            }
            out
        }};
    }
    for (field, upstream_field) in [
        ("xpos", "xpos"),
        ("xquat", "xquat"),
        ("xmat", "xmat"),
        ("xipos", "xipos"),
        ("ximat", "ximat"),
        ("xanchor", "xanchor"),
        ("xaxis", "xaxis"),
    ] {
        let local = match field {
            "xpos" => kin_vec!(xpos),
            "xquat" => kin_vec!(xquat),
            "xmat" => kin_vec!(xmat),
            "xipos" => kin_vec!(xipos),
            "ximat" => kin_vec!(ximat),
            "xanchor" => kin_vec!(xanchor),
            _ => kin_vec!(xaxis),
        };
        let up = snapshot(&upstream, upstream_field);
        record(&mut fields, &mut first_stage, "kinematics", field, &up, &local, worlds);
    }

    let mut subtree_com = Vec::new();
    let mut cdof = Vec::new();
    let mut cinert = Vec::new();
    let mut crb = Vec::new();
    let mut matrix = Vec::new();
    for w in 0..worlds {
        let cw = com.world(w).unwrap();
        subtree_com.extend_from_slice(cw.subtree_com);
        cdof.extend_from_slice(cw.cdof);
        cinert.extend_from_slice(cw.cinert);
        let mw = mm.world(w).unwrap();
        crb.extend_from_slice(mw.crb);
        matrix.extend_from_slice(mw.matrix);
    }
    for (field, local) in [
        ("subtree_com", &subtree_com),
        ("cdof", &cdof),
        ("cinert", &cinert),
    ] {
        let up = snapshot(&upstream, field);
        record(&mut fields, &mut first_stage, "com_position", field, &up, local, worlds);
    }
    let up_crb = snapshot(&upstream, "crb");
    record(&mut fields, &mut first_stage, "mass_matrix", "crb", &up_crb, &crb, worlds);

    let upstream_dense: Vec<f32> = upstream["dense_blocks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|world| world.as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32))
        .collect();
    record(&mut fields, &mut first_stage, "mass_matrix", "matrix_dense_block", &upstream_dense, &matrix, worlds);

    let summary = json!({
        "schema": 1,
        "kind": "g03-chain-first-difference",
        "offline_only": true,
        "product_chain_reads": false,
        "worlds": worlds,
        "stages_in_order": ["kinematics", "com_position", "mass_matrix"],
        "first_stage_with_difference": first_stage,
        "fields": fields,
    });
    fs::write(&output_path, serde_json::to_string_pretty(&summary).unwrap()).unwrap();
    println!("FIRST_DIFF_OK first_stage={:?}", first_stage);
}
