//! QA G2 independent harness: call the product GJK/EPA entry point (`ccd_pair`) on QA's own
//! re-captured upstream-f32 inputs and compare contacts against the upstream mujoco_warp f32
//! reference with QA's own comparator (ULP / scaled-ULP / frame absolute).
//!
//! Independent from the delivered `case_compare.rs`: own inputs JSON, own comparator, own stats.

use mjwarp_rs::physics::{CcdOptions, ContactOptions, Geom, MeshView, ccd_pair};
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;

const ROOT: &str = r"D:\Rust\mjwarp-rs";
const ULP_ONE: f32 = 1.192_092_9e-7;
const MESH_GEOM: i32 = 7;

#[derive(Clone, Copy, Debug, Default)]
struct Row {
    world: i32,
    id: i32,
    dist: f32,
    pos: [f32; 3],
    frame: [f32; 9],
}

#[derive(Default, Debug)]
struct CaseStats {
    reference_count: usize,
    product_count: usize,
    dist_zero: usize,
    dist_max_ulp: u32,
    pos_zero: usize,
    pos_max_scaled_ulp: f32,
    frame_zero: usize,
    frame_total: usize,
    frame_max_abs: f32,
    normal_max_abs: f32,
    id_mismatch: usize,
    pass: bool,
}

fn values_f32(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|item| item.as_f64().expect("number") as f32)
        .collect()
}

fn nested_f32(value: &Value) -> Vec<Vec<Vec<f32>>> {
    value
        .as_array()
        .expect("outer array")
        .iter()
        .map(|world| {
            world
                .as_array()
                .expect("world array")
                .iter()
                .map(values_f32)
                .collect()
        })
        .collect()
}

fn chunk3(flat: &[f32]) -> Vec<[f32; 3]> {
    flat.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn ulp_difference(left: f32, right: f32) -> u32 {
    if left == right {
        return 0;
    }
    if !left.is_finite()
        || !right.is_finite()
        || left.is_sign_positive() != right.is_sign_positive()
    {
        return u32::MAX;
    }
    left.to_bits().abs_diff(right.to_bits())
}

fn ulp_width(scale: f32) -> f32 {
    let magnitude = scale.abs();
    if magnitude == 0.0 || !magnitude.is_finite() {
        return f32::MIN_POSITIVE;
    }
    f32::from_bits(magnitude.to_bits() + 1) - magnitude
}

fn sort_contacts(contacts: &mut [Row]) {
    contacts.sort_by(|left, right| {
        left.dist
            .total_cmp(&right.dist)
            .then(left.pos[0].total_cmp(&right.pos[0]))
            .then(left.pos[1].total_cmp(&right.pos[1]))
            .then(left.pos[2].total_cmp(&right.pos[2]))
            .then(left.id.cmp(&right.id))
    });
}

fn flatten_numbers(value: &Value, out: &mut Vec<f32>) {
    match value {
        Value::Array(items) => {
            for item in items {
                flatten_numbers(item, out);
            }
        }
        Value::Number(number) => out.push(number.as_f64().expect("number") as f32),
        other => panic!("unexpected json value in numeric array: {other:?}"),
    }
}

fn flat_f32(value: &Value) -> Vec<f32> {
    let mut out = Vec::new();
    flatten_numbers(value, &mut out);
    out
}

fn reference_rows(case: &Value) -> Vec<Row> {
    let fields = &case["upstream"]["fields"];
    let display = case["upstream"]["count"].as_u64().expect("count") as usize;
    let worldids = flat_f32(&fields["worldid"]);
    let ids = flat_f32(&fields["geomcollisionid"]);
    let dists = flat_f32(&fields["dist"]);
    let pos = flat_f32(&fields["pos"]);
    let frame = flat_f32(&fields["frame"]);
    let mut rows = Vec::with_capacity(display);
    for index in 0..display {
        let mut frame_row = [0.0_f32; 9];
        frame_row.copy_from_slice(&frame[9 * index..9 * index + 9]);
        rows.push(Row {
            world: worldids[index] as i32,
            id: ids[index] as i32,
            dist: dists[index],
            pos: [pos[3 * index], pos[3 * index + 1], pos[3 * index + 2]],
            frame: frame_row,
        });
    }
    rows
}

fn pair_of(case: &Value) -> [i32; 2] {
    let geom = case["upstream"]["fields"]["geom"]
        .as_array()
        .expect("geom rows");
    let first = values_f32(&geom[0]);
    [first[0] as i32, first[1] as i32]
}

struct MeshData {
    vert: Vec<[f32; 3]>,
    polynormal: Vec<[f32; 3]>,
    polyvertadr: Vec<i32>,
    polyvertnum: Vec<i32>,
    polyvert: Vec<i32>,
    polymapadr: Vec<i32>,
    polymapnum: Vec<i32>,
    polymap: Vec<i32>,
    graph: Vec<i32>,
    polyadr: i32,
}

fn native_reference_rows(case: &Value) -> Vec<Row> {
    let reference = &case["native_reference"];
    let count = reference["count"].as_u64().expect("native count") as usize;
    let worldids = flat_f32(&reference["worldid"]);
    let ids = flat_f32(&reference["geomcollisionid"]);
    let dists = flat_f32(&reference["dist"]);
    let pos = flat_f32(&reference["pos"]);
    let frame = flat_f32(&reference["frame"]);
    let mut rows = Vec::with_capacity(count);
    for index in 0..count {
        let mut frame_row = [0.0_f32; 9];
        frame_row.copy_from_slice(&frame[9 * index..9 * index + 9]);
        rows.push(Row {
            world: worldids[index] as i32,
            id: ids[index] as i32,
            dist: dists[index],
            pos: [pos[3 * index], pos[3 * index + 1], pos[3 * index + 2]],
            frame: frame_row,
        });
    }
    rows
}

fn error_values(ours: &[Row], native: &[Row], worlds: usize) -> Vec<f32> {
    let mut values = Vec::new();
    for world in 0..worlds {
        let mut left: Vec<Row> = ours.iter().filter(|r| r.world == world as i32).copied().collect();
        let mut right: Vec<Row> = native.iter().filter(|r| r.world == world as i32).copied().collect();
        sort_contacts(&mut left);
        sort_contacts(&mut right);
        for (a, b) in left.iter().zip(right.iter()) {
            values.push((a.dist - b.dist).abs());
            for axis in 0..3 {
                values.push((a.pos[axis] - b.pos[axis]).abs());
            }
            for cell in 0..9 {
                values.push((a.frame[cell] - b.frame[cell]).abs());
            }
        }
    }
    values
}

fn summarize(mut values: Vec<f32>) -> (usize, f32, f32, f32, f32) {
    if values.is_empty() {
        return (0, 0.0, 0.0, 0.0, 0.0);
    }
    values.sort_by(f32::total_cmp);
    let count = values.len();
    let over = values.iter().filter(|v| **v > 1.0e-6).count() as f32 / count as f32;
    (
        count,
        values[0],
        values[count / 2],
        values[count - 1],
        over,
    )
}

fn mesh_data(case: &Value, dataid: i32) -> MeshData {
    let table = &case["mesh"][dataid as usize];
    MeshData {
        vert: chunk3(&values_f32(&table["vert"])),
        polynormal: chunk3(&values_f32(&table["polynormal"])),
        polyvertadr: values_f32(&table["polyvertadr"])
            .iter()
            .map(|v| *v as i32)
            .collect(),
        polyvertnum: values_f32(&table["polyvertnum"])
            .iter()
            .map(|v| *v as i32)
            .collect(),
        polyvert: values_f32(&table["polyvert"])
            .iter()
            .map(|v| *v as i32)
            .collect(),
        polymapadr: values_f32(&table["polymapadr"])
            .iter()
            .map(|v| *v as i32)
            .collect(),
        polymapnum: values_f32(&table["polymapnum"])
            .iter()
            .map(|v| *v as i32)
            .collect(),
        polymap: values_f32(&table["polymap"])
            .iter()
            .map(|v| *v as i32)
            .collect(),
        graph: values_f32(&table["graph"]).iter().map(|v| *v as i32).collect(),
        polyadr: table["polyadr"].as_i64().expect("polyadr") as i32,
    }
}

fn product_rows(case: &Value, poses_key: &str) -> Vec<Row> {
    let pair = pair_of(case);
    let geom_type: Vec<i32> = values_f32(&case["geom_type"]).iter().map(|v| *v as i32).collect();
    let geom_size = values_f32(&case["geom_size"]);
    if case.get(poses_key).is_none() {
        eprintln!(
            "DEBUG case keys: {:?}",
            case.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>())
        );
    }
    let positions = nested_f32(&case[poses_key]["geom_xpos"]);
    let rotations = nested_f32(&case[poses_key]["geom_xmat"]);
    let mut rows = Vec::new();
    for world in 0..positions.len() {
        let mut meshes: Vec<Option<MeshData>> = Vec::new();
        let mut views: Vec<Option<MeshView<'_>>> = Vec::new();
        for slot in 0..2 {
            let gid = pair[slot] as usize;
            if geom_type[gid] == MESH_GEOM {
                meshes.push(Some(mesh_data(case, case["geom_dataid"][gid].as_i64().unwrap() as i32)));
            } else {
                meshes.push(None);
            }
        }
        for mesh in &meshes {
            views.push(mesh.as_ref().map(|mesh| MeshView {
                vert: &mesh.vert,
                graph: &mesh.graph,
                polyadr: mesh.polyadr,
                polynormal: &mesh.polynormal,
                polyvertadr: &mesh.polyvertadr,
                polyvertnum: &mesh.polyvertnum,
                polyvert: &mesh.polyvert,
                polymapadr: &mesh.polymapadr,
                polymapnum: &mesh.polymapnum,
                polymap: &mesh.polymap,
            }));
        }
        let mut geoms = [Geom::simple([0.0; 3], [0.0; 9], [0.0; 3], 0.0); 2];
        for slot in 0..2 {
            let gid = pair[slot] as usize;
            let size = [geom_size[3 * gid], geom_size[3 * gid + 1], geom_size[3 * gid + 2]];
            let position = &positions[world][gid];
            let rotation = &rotations[world][gid];
            let mut rot = [0.0_f32; 9];
            rot.copy_from_slice(&rotation[..9]);
            let mut geom = Geom::simple(
                [position[0], position[1], position[2]],
                rot,
                size,
                0.0,
            );
            if let Some(view) = views[slot].as_ref() {
                geom.vertadr = 0;
                geom.vertnum = view.vert.len() as i32;
                geom.graphadr = -1;
                geom.mesh = Some(view);
            }
            geoms[slot] = geom;
        }
        let options = ContactOptions {
            ccd: CcdOptions {
                tolerance: 1.0e-6,
                cutoff: 0.0,
                gjk_iterations: 35,
                epa_iterations: 35,
            },
            margin: 0.0,
            gap: 0.0,
            collision_sensor: false,
            use_multiccd: true,
        };
        let output = ccd_pair(
            &geoms[0],
            &geoms[1],
            geom_type[pair[0] as usize],
            geom_type[pair[1] as usize],
            options,
        );
        for (index, contact) in output.contacts.iter().enumerate() {
            rows.push(Row {
                world: world as i32,
                id: index as i32,
                dist: contact.dist,
                pos: contact.pos,
                frame: contact.frame,
            });
        }
    }
    rows
}

fn compare(ours: &[Row], reference: &[Row]) -> CaseStats {
    let mut stats = CaseStats {
        reference_count: reference.len(),
        product_count: ours.len(),
        frame_total: reference.len() * 9,
        ..CaseStats::default()
    };
    if ours.len() != reference.len() {
        return stats;
    }
    let mut sorted_ours = ours.to_vec();
    let mut sorted_ref = reference.to_vec();
    sort_contacts(&mut sorted_ours);
    sort_contacts(&mut sorted_ref);
    for (expected, actual) in sorted_ref.iter().zip(sorted_ours.iter()) {
        if expected.id != actual.id {
            stats.id_mismatch += 1;
        }
        let dist_ulp = ulp_difference(expected.dist, actual.dist);
        stats.dist_max_ulp = stats.dist_max_ulp.max(dist_ulp);
        if dist_ulp == 0 {
            stats.dist_zero += 1;
        }
        let mut scale = 0.0_f32;
        let mut pos_abs = 0.0_f32;
        for axis in 0..3 {
            scale = scale.max(expected.pos[axis].abs()).max(actual.pos[axis].abs());
            pos_abs = pos_abs.max((expected.pos[axis] - actual.pos[axis]).abs());
        }
        if pos_abs == 0.0 {
            stats.pos_zero += 1;
        }
        stats.pos_max_scaled_ulp = stats.pos_max_scaled_ulp.max(pos_abs / ulp_width(scale));
        let mut frame_abs = 0.0_f32;
        for cell in 0..9 {
            let diff = (expected.frame[cell] - actual.frame[cell]).abs();
            frame_abs = frame_abs.max(diff);
            if diff == 0.0 {
                stats.frame_zero += 1;
            }
            if cell < 3 {
                stats.normal_max_abs = stats.normal_max_abs.max(diff);
            }
        }
        stats.frame_max_abs = stats.frame_max_abs.max(frame_abs);
    }
    stats.pass = stats.product_count == stats.reference_count
        && stats.id_mismatch == 0
        && stats.dist_max_ulp <= 4
        && stats.pos_max_scaled_ulp <= 4.0
        && stats.frame_max_abs <= 4.8e-7;
    stats
}

fn ordering_check(rows: &[Row], worlds: usize) -> Vec<usize> {
    let mut counts = vec![0_usize; worlds];
    for row in rows {
        if (row.world as usize) < worlds {
            counts[row.world as usize] += 1;
        }
    }
    counts
}

fn stats_json(stats: &CaseStats) -> Value {
    serde_json::json!({
        "reference_count": stats.reference_count,
        "product_count": stats.product_count,
        "dist_zero": stats.dist_zero,
        "dist_max_ulp": stats.dist_max_ulp,
        "pos_zero": stats.pos_zero,
        "pos_max_scaled_ulp": stats.pos_max_scaled_ulp,
        "frame_zero": stats.frame_zero,
        "frame_total": stats.frame_total,
        "frame_max_abs": stats.frame_max_abs,
        "normal_max_abs": stats.normal_max_abs,
        "normal_ulp_at_one": stats.normal_max_abs / ULP_ONE,
        "id_mismatch": stats.id_mismatch,
        "pass_criterion": stats.pass,
    })
}

fn main() {
    let inputs: Value = serde_json::from_str(
        &fs::read_to_string(format!(r"{ROOT}\target\qa-g2\qa-case-inputs.json")).expect("inputs"),
    )
    .expect("json");
    let mut report = serde_json::json!({"schema": 1, "kind": "qa-g2-independent-ours-vs-upstream-f32", "cases": {}});
    let mut all_pass = true;
    let mut e_ours_all: Vec<f32> = Vec::new();
    let mut e_mjwarp_all: Vec<f32> = Vec::new();
    for case in inputs["cases"].as_array().expect("cases") {
        let name = case["name"].as_str().expect("name");
        let reference = reference_rows(case);
        let worlds = case["nworld"].as_u64().expect("nworld") as usize;
        let ours_upstream = product_rows(case, "upstream_poses");
        let ours_native = product_rows(case, "native_poses");
        let stats_upstream = compare(&ours_upstream, &reference);
        let stats_native = compare(&ours_native, &reference);
        let native_reference = native_reference_rows(case);
        let e_ours = error_values(&ours_native, &native_reference, worlds);
        let e_mjwarp = error_values(&reference, &native_reference, worlds);
        let (e_ours_count, e_ours_min, e_ours_median, e_ours_max, e_ours_over) =
            summarize(e_ours.clone());
        let (e_mj_count, e_mj_min, e_mj_median, e_mj_max, e_mj_over) =
            summarize(e_mjwarp.clone());
        e_ours_all.extend(e_ours);
        e_mjwarp_all.extend(e_mjwarp);
        let ref_world_counts = ordering_check(&reference, worlds);
        let ours_world_counts = ordering_check(&ours_upstream, worlds);
        all_pass = all_pass && stats_upstream.pass;
        println!(
            "[judge] {:<16} count {}/{} | dist bit {}/{} max {} ULP | pos bit {}/{} max {:.2} ULP@scale | frame bit {}/{} max abs {:e} (normal {:.2} ULP@1) | id_mismatch {} | pass {}",
            name,
            stats_upstream.product_count,
            stats_upstream.reference_count,
            stats_upstream.dist_zero,
            stats_upstream.reference_count,
            stats_upstream.dist_max_ulp,
            stats_upstream.pos_zero,
            stats_upstream.reference_count,
            stats_upstream.pos_max_scaled_ulp,
            stats_upstream.frame_zero,
            stats_upstream.frame_total,
            stats_upstream.frame_max_abs,
            stats_upstream.normal_max_abs / ULP_ONE,
            stats_upstream.id_mismatch,
            stats_upstream.pass,
        );
        println!(
            "[native-input] {:<16} count {}/{} | dist max {} ULP | pos max {:.2} ULP@scale | frame max abs {:e}",
            name,
            stats_native.product_count,
            stats_native.reference_count,
            stats_native.dist_max_ulp,
            stats_native.pos_max_scaled_ulp,
            stats_native.frame_max_abs,
        );
        report["cases"][name] = serde_json::json!({
            "upstream_f32_input": stats_json(&stats_upstream),
            "native_f32_pose_input": stats_json(&stats_native),
            "reference_contacts_per_world": ref_world_counts,
            "product_contacts_per_world": ours_world_counts,
            "e_table_ours_vs_native": {
                "count": e_ours_count, "min": e_ours_min, "median": e_ours_median,
                "max": e_ours_max, "over_1e6_ratio": e_ours_over,
            },
            "e_table_mjwarp_vs_native": {
                "count": e_mj_count, "min": e_mj_min, "median": e_mj_median,
                "max": e_mj_max, "over_1e6_ratio": e_mj_over,
            },
        });
    }
    let (count, min, median, max, over) = summarize(e_ours_all);
    let (mj_count, mj_min, mj_median, mj_max, mj_over) = summarize(e_mjwarp_all);
    let seed_text = fs::read_to_string(format!(
        r"{ROOT}\docs\development-evidence\g10-p5-gjk-epa\native-error-seed.json"
    ))
    .expect("seed");
    let seed: Value = serde_json::from_str(&seed_text).expect("seed json");
    let seed_ours = &seed["fields"]["ours_vs_native"];
    let seed_mj = &seed["fields"]["mjwarp_vs_native"];
    report["native_error_seed_check"] = serde_json::json!({
        "ours_vs_native": {
            "qa": {"count": count, "min": min, "median": median, "max": max, "over_1e6_ratio": over},
            "seed": seed_ours,
            "count_matches": seed_ours["count"].as_u64() == Some(count as u64),
            "max_matches": (seed_ours["max"].as_f64().unwrap() as f32) == max,
            "over_matches": (seed_ours["over_1e6_ratio"].as_f64().unwrap() as f32) == over,
        },
        "mjwarp_vs_native": {
            "qa": {"count": mj_count, "min": mj_min, "median": mj_median, "max": mj_max,
                   "over_1e6_ratio": mj_over},
            "seed": seed_mj,
            "count_matches": seed_mj["count"].as_u64() == Some(mj_count as u64),
            "max_matches": (seed_mj["max"].as_f64().unwrap() as f32) == mj_max,
            "over_matches": (seed_mj["over_1e6_ratio"].as_f64().unwrap() as f32) == mj_over,
        },
    });
    println!(
        "[E] ours-vs-native  count {} min {:.3e} median {:.3e} max {:.3e} over {:.3} | mjwarp-vs-native count {} max {:.3e}",
        count, min, median, max, over, mj_count, mj_max
    );
    report["overall"] = serde_json::json!({"all_cases_pass_criterion": all_pass});
    // ccd phase selection is covered by the delivered unit tests; QA's numeric criterion is above.

    fs::write(
        format!(r"{ROOT}\target\qa-g2\harness-report.json"),
        serde_json::to_string_pretty(&report).expect("serialize"),
    )
    .expect("write report");
    println!("overall pass: {all_pass}");
    let _ = Ordering::Equal;
}
