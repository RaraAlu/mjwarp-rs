// AUTO-GENERATED from probe_template.rs by target/qa-m0afix/make_probe.py -- do not hand-edit.
//
// `validate_mesh_poly_old` is the VERBATIM body of `abi.rs` at b50d93a6~1
// (only the function name is suffixed so both bodies can coexist in one file).
// `validate_mesh_poly` is the VERBATIM body of `abi.rs` at b50d93a6 (HEAD code tree).
// Arrays below are this QA's own dump of the pinned mjModel for the M1 cube fixture
// (target/qa-m0afix/dump_cube_poly.py -> cube-poly-model.json).
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum CollisionGpuError {
    InvalidArgument(&'static str),
}

struct CollisionGpuInputs<'a> {
    mesh_polynum: &'a [i32],
    mesh_polyadr: &'a [i32],
    mesh_polynormal: &'a [f32],
    mesh_polyvertadr: &'a [i32],
    mesh_polyvertnum: &'a [i32],
    mesh_polyvert: &'a [i32],
    mesh_polymapadr: &'a [i32],
    mesh_polymapnum: &'a [i32],
    mesh_polymap: &'a [i32],
}

// __OLD_FN__

// __NEW_FN__

fn real_cube() -> (
    Vec<i32>,
    Vec<i32>,
    Vec<f32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
) {
    // __ARRAY_DECLS__
    (
        polynum,
        polyadr,
        polynormal,
        polyvertadr,
        polyvertnum,
        polyvert,
        polymapadr,
        polymapnum,
        polymap,
    )
}

fn show(result: &Result<(), CollisionGpuError>) -> String {
    match result {
        Ok(()) => "Ok(())".to_string(),
        Err(CollisionGpuError::InvalidArgument(message)) => {
            format!("Err(InvalidArgument({message:?}))")
        }
    }
}

fn json_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    let base = real_cube();
    let mut rows: Vec<(String, String, String)> = Vec::new();

    macro_rules! run_case {
        ($name:expr, $polynum:expr, $polyadr:expr, $polynormal:expr, $pva:expr, $pvn:expr, $pv:expr, $pma:expr, $pmn:expr, $pm:expr) => {{
            let inputs = CollisionGpuInputs {
                mesh_polynum: $polynum,
                mesh_polyadr: $polyadr,
                mesh_polynormal: $polynormal,
                mesh_polyvertadr: $pva,
                mesh_polyvertnum: $pvn,
                mesh_polyvert: $pv,
                mesh_polymapadr: $pma,
                mesh_polymapnum: $pmn,
                mesh_polymap: $pm,
            };
            rows.push((
                $name.to_string(),
                show(&validate_mesh_poly_old(&inputs)),
                show(&validate_mesh_poly(&inputs)),
            ));
        }};
    }

    let (polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap) = &base;

    // 0) real cube arrays exactly as pinned mjModel stores them.
    run_case!(
        "real_cube",
        polynum,
        polyadr,
        polynormal,
        polyvertadr,
        polyvertnum,
        polyvert,
        polymapadr,
        polymapnum,
        polymap
    );

    // One-array mutations of the real data.
    let mut short_polymapnum = polymapnum.clone();
    let last = short_polymapnum.len() - 1;
    short_polymapnum[last] = 2;
    let mut short_polyvertnum = polyvertnum.clone();
    short_polyvertnum[0] = 3;
    let mut polynormal_len4 = polynormal.clone();
    polynormal_len4.truncate(4);
    let mut long_polyadr = polyadr.clone();
    long_polyadr.push(0);
    let mut short_polyvertadr = polyvertadr.clone();
    short_polyvertadr.pop();
    let mut short_polymapadr_ = polymapnum.clone();
    short_polymapadr_.pop();
    let mut shifted_polyadr = polyadr.clone();
    shifted_polyadr[0] = 1;
    let mut short_polyvert = polyvert.clone();
    short_polyvert.pop();
    let mut short_polymap = polymap.clone();
    short_polymap.pop();
    let neg_polynum: Vec<i32> = vec![-1];
    let sum_polynum: Vec<i32> = vec![5];
    let neg_polyvertadr: Vec<i32> = vec![-1, 4, 8, 12, 16, 20];
    let neg_polymapnum: Vec<i32> = vec![3, 3, 3, 3, 3, 3, 3, -1];
    let partial_polynum: Vec<i32> = vec![6];
    // Two-mesh layout (nmesh=2) built from the same real polygon tail: proves the rule
    // generalises beyond the single-mesh cube instead of being fitted to it.
    let two_polynum: Vec<i32> = vec![3, 3];
    let two_polyadr: Vec<i32> = vec![0, 3];
    let two_polynum_bad_sum: Vec<i32> = vec![2, 3];
    let two_polyadr_bad_range: Vec<i32> = vec![4, 3];
    let neg_polyadr: Vec<i32> = vec![-1];
    let neg_polyvertnum: Vec<i32> = vec![-4, 4, 4, 4, 4, 4];
    let neg_polymapadr: Vec<i32> = vec![-1, 3, 6, 9, 12, 15, 18, 21];

    run_case!("neg_polynum", &neg_polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("neg_polyvertadr", polynum, polyadr, polynormal, &neg_polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("neg_polymapnum", polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, &neg_polymapnum, polymap);
    run_case!("polynormal_not_multiple_of_3", polynum, polyadr, &polynormal_len4, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("per_mesh_length_mismatch", polynum, &long_polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("per_polygon_length_mismatch", polynum, polyadr, polynormal, &short_polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("per_vertex_length_mismatch", polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, &short_polymapadr_, polymap);
    run_case!("per_mesh_range_overflow", polynum, &shifted_polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("per_polygon_range_overflow", polynum, polyadr, polynormal, polyvertadr, polyvertnum, &short_polyvert, polymapadr, polymapnum, polymap);
    run_case!("per_vertex_range_overflow", polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, &short_polymap);
    run_case!("sum_polynum_mismatch", &sum_polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("sum_polyvertnum_mismatch", polynum, polyadr, polynormal, polyvertadr, &short_polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("sum_polymapnum_mismatch", polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, &short_polymapnum, polymap);
    run_case!("partial_sidecar", &partial_polynum, &[], &[], &[], &[], &[], &[], &[], &[]);
    run_case!("all_empty", &[], &[], &[], &[], &[], &[], &[], &[], &[]);
    run_case!("two_meshes_valid", &two_polynum, &two_polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("two_meshes_sum_mismatch", &two_polynum_bad_sum, &two_polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("two_meshes_range_overflow", &two_polynum, &two_polyadr_bad_range, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("neg_polyadr", polynum, &neg_polyadr, polynormal, polyvertadr, polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("neg_polyvertnum", polynum, polyadr, polynormal, polyvertadr, &neg_polyvertnum, polyvert, polymapadr, polymapnum, polymap);
    run_case!("neg_polymapadr", polynum, polyadr, polynormal, polyvertadr, polyvertnum, polyvert, &neg_polymapadr, polymapnum, polymap);

    let mut out = String::new();
    out.push_str("{\"probe\":\"qa-m0afix-old-vs-new-rule\",\"old_fn\":[");
    out.push_str("__OLD_RANGE_JSON__");
    out.push_str("],\"new_fn\":[");
    out.push_str("__NEW_RANGE_JSON__");
    out.push_str("],\"cases\":[");
    for (index, (name, old, new)) in rows.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"case\":\"{}\",\"old\":\"{}\",\"new\":\"{}\"}}",
            name,
            json_escape(old),
            json_escape(new)
        ));
    }
    out.push_str("]}\n");
    std::fs::write(
        "D:\\Rust\\mjwarp-rs\\target\\qa-m0afix\\old-new-rule-matrix.json",
        out,
    )
    .expect("write matrix");

    for (name, old, new) in &rows {
        println!("{name}|old={old}|new={new}");
    }
}
