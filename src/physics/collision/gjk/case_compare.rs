//! G2 数值对照：GJK/EPA（`ccd`）对 pinned 上游参考（含输入对齐两栏）。

use super::case_data::{Case, UPSTREAM_REVISION, cases, floats, root};
use super::{CcdOptions, ContactOptions, Geom, MeshView, ccd_pair};
use serde_json::Value;
use std::fs;

/// G2 主参考：pinned mujoco_warp 3.15（warp 1.15.0）CPU 后端在同 f32 输入下的接触输出。
/// provenance 见 `docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json`。
pub(super) fn upstream_mjwarp_contacts(case: &Case) -> Vec<RefContact> {
    let path = root().join("docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json");
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).expect("mjwarp 参考应可读"))
        .expect("mjwarp 参考应为合法 JSON");
    assert_eq!(value["provenance"]["versions"]["warp"], "1.15.0");
    let output = &value["cases"][case.name.as_str()]["output"];
    let fields = &output["fields"];
    let count = output["count"].as_u64().expect("count") as usize;
    (0..count)
        .map(|index| RefContact {
            worldid: fields["worldid"][index][0].as_i64().expect("worldid") as i32,
            geomcollisionid: fields["geomcollisionid"][index][0]
                .as_i64()
                .expect("geomcollisionid") as i32,
            dist: fields["dist"][index][0].as_f64().expect("dist") as f32,
            pos: [
                fields["pos"][index][0].as_f64().expect("pos") as f32,
                fields["pos"][index][1].as_f64().expect("pos") as f32,
                fields["pos"][index][2].as_f64().expect("pos") as f32,
            ],
            frame: [
                fields["frame"][index][0].as_f64().expect("frame") as f32,
                fields["frame"][index][1].as_f64().expect("frame") as f32,
                fields["frame"][index][2].as_f64().expect("frame") as f32,
                fields["frame"][index][3].as_f64().expect("frame") as f32,
                fields["frame"][index][4].as_f64().expect("frame") as f32,
                fields["frame"][index][5].as_f64().expect("frame") as f32,
                fields["frame"][index][6].as_f64().expect("frame") as f32,
                fields["frame"][index][7].as_f64().expect("frame") as f32,
                fields["frame"][index][8].as_f64().expect("frame") as f32,
            ],
        })
        .collect()
}

pub(super) fn ccd_options() -> CcdOptions {
    // 上游 `m.opt.ccd_tolerance=1e-6`、`ccd_iterations=35`；非全 box-box 对时
    // `epa_iterations = m.opt.ccd_iterations`（builder 侧 `collision_convex.py`）。
    CcdOptions {
        tolerance: 1.0e-6,
        cutoff: 0.0,
        gjk_iterations: 35,
        epa_iterations: 35,
    }
}

#[derive(Clone, Copy)]
pub(super) struct RefContact {
    pub(super) worldid: i32,
    /// 上游 `contact_geomcollisionid`：同一 geom 对内第 i 条接触（`write_contact` 的 `id_`）。
    pub(super) geomcollisionid: i32,
    pub(super) dist: f32,
    pub(super) pos: [f32; 3],
    pub(super) frame: [f32; 9],
}

pub(super) fn reference_contacts(case: &Case) -> Vec<RefContact> {
    let text = fs::read_to_string(root().join(&case.reference)).expect("参考 JSON 应可读");
    let value: Value = serde_json::from_str(&text).expect("参考 JSON 应为合法 JSON");
    let output = if case.standalone {
        assert_eq!(value["provenance"]["upstream_revision"], UPSTREAM_REVISION);
        value["output"].clone()
    } else {
        assert_eq!(value["upstream_revision"], UPSTREAM_REVISION);
        value["collision_cases"]
            .as_array()
            .expect("collision_cases")
            .iter()
            .find(|entry| entry["name"] == case.name.as_str())
            .expect("应有该 case")["output"]
            .clone()
    };
    let count = output["count"].as_u64().expect("count") as usize;
    let dist = floats(&output["fields"]["dist"]["values"]);
    let pos = floats(&output["fields"]["pos"]["values"]);
    let frame = floats(&output["fields"]["frame"]["values"]);
    let worldid = floats(&output["fields"]["worldid"]["values"]);
    let collisionid = floats(&output["fields"]["geomcollisionid"]["values"]);
    assert_eq!(dist.len(), count);
    (0..count)
        .map(|index| RefContact {
            worldid: worldid[index] as i32,
            geomcollisionid: collisionid[index] as i32,
            dist: dist[index],
            pos: [pos[3 * index], pos[3 * index + 1], pos[3 * index + 2]],
            frame: {
                let mut out = [0.0_f32; 9];
                out.copy_from_slice(&frame[9 * index..9 * index + 9]);
                out
            },
        })
        .collect()
}

pub(super) fn mesh_view<'a>(case: &'a Case) -> Option<MeshView<'a>> {
    case.mesh.as_ref().map(|mesh| MeshView {
        vert: &mesh.vert,
        graph: &[],
        polyadr: 0,
        polynormal: &mesh.polynormal,
        polyvertadr: &mesh.polyvertadr,
        polyvertnum: &mesh.polyvertnum,
        polyvert: &mesh.polyvert,
        polymapadr: &mesh.polymapadr,
        polymapnum: &mesh.polymapnum,
        polymap: &mesh.polymap,
    })
}

/// `aligned=false` 用 mj_forward(f64→f32) 位姿；`true` 用 mjwarp f32 运动学口径。
pub(super) fn product_contacts(case: &Case, world: usize, aligned: bool) -> Vec<RefContact> {
    let pose = if aligned {
        &case.aligned[world]
    } else {
        &case.worlds[world]
    };
    let mesh = mesh_view(case);
    let mut geoms = [Geom::simple([0.0; 3], [0.0; 9], [0.0; 3], 0.0); 2];
    for (slot, geom) in geoms.iter_mut().enumerate() {
        let gid = case.pair[slot];
        *geom = Geom::simple(pose.pos[gid], pose.rot[gid], case.sizes[gid], 0.0);
        if let Some(view) = mesh.as_ref() {
            geom.vertadr = 0;
            geom.vertnum = view.vert.len() as i32;
            geom.graphadr = -1;
            geom.mesh = Some(view);
        }
    }
    let options = ContactOptions {
        ccd: ccd_options(),
        margin: 0.0,
        gap: 0.0,
        collision_sensor: false,
        use_multiccd: true,
    };
    let output = ccd_pair(
        &geoms[0],
        &geoms[1],
        case.geomtypes[case.pair[0]],
        case.geomtypes[case.pair[1]],
        options,
    );
    output
        .contacts
        .iter()
        .enumerate()
        .map(|(index, contact)| RefContact {
            worldid: world as i32,
            geomcollisionid: index as i32,
            dist: contact.dist,
            pos: contact.pos,
            frame: contact.frame,
        })
        .collect()
}

pub(super) fn sort_contacts(contacts: &mut [RefContact]) {
    contacts.sort_by(|left, right| {
        left.dist
            .total_cmp(&right.dist)
            .then(left.pos[0].total_cmp(&right.pos[0]))
            .then(left.pos[1].total_cmp(&right.pos[1]))
            .then(left.pos[2].total_cmp(&right.pos[2]))
            .then(left.geomcollisionid.cmp(&right.geomcollisionid))
    });
}

pub(super) fn ulp_difference(left: f32, right: f32) -> u32 {
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

pub(super) fn ulp_width(scale: f32) -> f32 {
    let magnitude = scale.abs();
    if magnitude == 0.0 || !magnitude.is_finite() {
        return f32::MIN_POSITIVE;
    }
    f32::from_bits(magnitude.to_bits() + 1) - magnitude
}

#[derive(Default)]
pub(super) struct CaseStats {
    pub(super) reference_count: usize,
    pub(super) product_count: usize,
    pub(super) dist_zero: usize,
    pub(super) dist_max_ulp: u32,
    pub(super) pos_max_scaled_ulp: f32,
    pub(super) pos_zero_contacts: usize,
    /// geomcollisionid 位等条数（重复接触点的定序键）。
    pub(super) id_zero: usize,
    pub(super) frame_zero: usize,
    pub(super) frame_max_abs: f32,
}

pub(super) fn compare_case(case: &Case, aligned: bool, upstream: bool) -> CaseStats {
    let reference = if upstream {
        upstream_mjwarp_contacts(case)
    } else {
        reference_contacts(case)
    };
    let mut stats = CaseStats {
        reference_count: reference.len(),
        ..CaseStats::default()
    };
    for world in 0..case.worlds.len() {
        let mut expected: Vec<RefContact> = reference
            .iter()
            .filter(|contact| contact.worldid == world as i32)
            .copied()
            .collect();
        let mut actual = product_contacts(case, world, aligned);
        sort_contacts(&mut expected);
        sort_contacts(&mut actual);
        stats.product_count += actual.len();
        assert_eq!(
            actual.len(),
            expected.len(),
            "{} world {}",
            case.name,
            world
        );
        for (expected, actual) in expected.iter().zip(actual.iter()) {
            assert_eq!(
                expected.geomcollisionid, actual.geomcollisionid,
                "{} world {} geomcollisionid 不一致",
                case.name, world
            );
            stats.id_zero += 1;
            let dist_ulp = ulp_difference(expected.dist, actual.dist);
            stats.dist_max_ulp = stats.dist_max_ulp.max(dist_ulp);
            if dist_ulp == 0 {
                stats.dist_zero += 1;
            }
            let (mut scale, mut pos_abs) = (0.0_f32, 0.0_f32);
            for axis in 0..3 {
                scale = scale
                    .max(expected.pos[axis].abs())
                    .max(actual.pos[axis].abs());
                pos_abs = pos_abs.max((expected.pos[axis] - actual.pos[axis]).abs());
            }
            if pos_abs == 0.0 {
                stats.pos_zero_contacts += 1;
            }
            stats.pos_max_scaled_ulp = stats.pos_max_scaled_ulp.max(pos_abs / ulp_width(scale));
            let mut frame_abs = 0.0_f32;
            for cell in 0..9 {
                let abs = (expected.frame[cell] - actual.frame[cell]).abs();
                frame_abs = frame_abs.max(abs);
                if abs == 0.0 {
                    stats.frame_zero += 1;
                }
            }
            stats.frame_max_abs = stats.frame_max_abs.max(frame_abs);
            if dist_ulp > 8 || pos_abs / ulp_width(scale) > 4.0 || frame_abs > 4.8e-7 {
                println!(
                    "  [{}] world {} out-of-budget: dist {:?} vs {:?} ({dist_ulp} ULP) | pos {:?} vs {:?} ({:.2} ULP@尺度) | frame abs {frame_abs:e}",
                    case.name,
                    world,
                    expected.dist,
                    actual.dist,
                    expected.pos,
                    actual.pos,
                    pos_abs / ulp_width(scale),
                );
            }
        }
    }
    stats
}

/// 打印一张 ULP 表（`label` = 输入口径）。
pub(super) fn print_table(label: &str, all: &[(&Case, CaseStats)]) {
    for (case, stats) in all {
        println!(
            "[{label}] {:<16} count {}/{} | dist 位等 {}/{} max {} ULP | pos 位等接触 {}/{} max {:.2} ULP@尺度 | frame 位等分量 {}/{} max abs {:e}",
            case.name,
            stats.product_count,
            stats.reference_count,
            stats.dist_zero,
            stats.reference_count,
            stats.dist_max_ulp,
            stats.pos_zero_contacts,
            stats.reference_count,
            stats.pos_max_scaled_ulp,
            stats.frame_zero,
            stats.reference_count * 9,
            stats.frame_max_abs,
        );
    }
}
/// 上游 mjwarp f32 运动学产出的世界位姿（直接作为 `ccd` 输入，纯合成、无 FK 复算）。
pub(super) fn upstream_poses(case: &Case, world: usize) -> Vec<([f32; 3], [f32; 9])> {
    let path = root().join("docs/development-evidence/g10-p5-gjk-epa/upstream-mjwarp-f32.json");
    let value: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("可读")).expect("合法 JSON");
    let output = &value["cases"][case.name.as_str()]["output"];
    let ngeom = case.geomtypes.len();
    (0..ngeom)
        .map(|gid| {
            let pos = &output["geom_xpos"][world][gid];
            let rot = &output["geom_xmat"][world][gid];
            (
                [
                    pos[0].as_f64().expect("pos") as f32,
                    pos[1].as_f64().expect("pos") as f32,
                    pos[2].as_f64().expect("pos") as f32,
                ],
                [
                    rot[0].as_f64().expect("rot") as f32,
                    rot[1].as_f64().expect("rot") as f32,
                    rot[2].as_f64().expect("rot") as f32,
                    rot[3].as_f64().expect("rot") as f32,
                    rot[4].as_f64().expect("rot") as f32,
                    rot[5].as_f64().expect("rot") as f32,
                    rot[6].as_f64().expect("rot") as f32,
                    rot[7].as_f64().expect("rot") as f32,
                    rot[8].as_f64().expect("rot") as f32,
                ],
            )
        })
        .collect()
}

/// 用上游 f32 世界位姿作为输入的接触计算（判据口径）。
pub(super) fn product_contacts_upstream(case: &Case, world: usize) -> Vec<RefContact> {
    let poses = upstream_poses(case, world);
    let mesh = mesh_view(case);
    let mut geoms = [Geom::simple([0.0; 3], [0.0; 9], [0.0; 3], 0.0); 2];
    for (slot, geom) in geoms.iter_mut().enumerate() {
        let gid = case.pair[slot];
        *geom = Geom::simple(poses[gid].0, poses[gid].1, case.sizes[gid], 0.0);
        if let Some(view) = mesh.as_ref() {
            geom.vertadr = 0;
            geom.vertnum = view.vert.len() as i32;
            geom.graphadr = -1;
            geom.mesh = Some(view);
        }
    }
    let options = ContactOptions {
        ccd: ccd_options(),
        margin: 0.0,
        gap: 0.0,
        collision_sensor: false,
        use_multiccd: true,
    };
    ccd_pair(
        &geoms[0],
        &geoms[1],
        case.geomtypes[case.pair[0]],
        case.geomtypes[case.pair[1]],
        options,
    )
    .contacts
    .iter()
    .enumerate()
    .map(|(index, contact)| RefContact {
        worldid: world as i32,
        geomcollisionid: index as i32,
        dist: contact.dist,
        pos: contact.pos,
        frame: contact.frame,
    })
    .collect()
}

/// `ours` vs 同配方上游 mjwarp f32（CPU 后端）的位级/ULP 对照（G2 数值通过判据）。
pub(super) fn compare_vs_mjwarp(case: &Case) -> CaseStats {
    let reference = upstream_mjwarp_contacts(case);
    let mut stats = CaseStats {
        reference_count: reference.len(),
        ..CaseStats::default()
    };
    for world in 0..case.worlds.len() {
        let mut expected: Vec<RefContact> = reference
            .iter()
            .filter(|contact| contact.worldid == world as i32)
            .copied()
            .collect();
        let mut actual = product_contacts_upstream(case, world);
        sort_contacts(&mut expected);
        sort_contacts(&mut actual);
        stats.product_count += actual.len();
        assert_eq!(actual.len(), expected.len(), "{} world {world}", case.name);
        for (expected, actual) in expected.iter().zip(actual.iter()) {
            let dist_ulp = ulp_difference(expected.dist, actual.dist);
            stats.dist_max_ulp = stats.dist_max_ulp.max(dist_ulp);
            if dist_ulp == 0 {
                stats.dist_zero += 1;
            }
            let (mut scale, mut pos_abs) = (0.0_f32, 0.0_f32);
            for axis in 0..3 {
                scale = scale
                    .max(expected.pos[axis].abs())
                    .max(actual.pos[axis].abs());
                pos_abs = pos_abs.max((expected.pos[axis] - actual.pos[axis]).abs());
            }
            if pos_abs == 0.0 {
                stats.pos_zero_contacts += 1;
            }
            stats.pos_max_scaled_ulp = stats.pos_max_scaled_ulp.max(pos_abs / ulp_width(scale));
            let mut frame_abs = 0.0_f32;
            for cell in 0..9 {
                let abs = (expected.frame[cell] - actual.frame[cell]).abs();
                frame_abs = frame_abs.max(abs);
                if abs == 0.0 {
                    stats.frame_zero += 1;
                }
            }
            stats.frame_max_abs = stats.frame_max_abs.max(frame_abs);
        }
    }
    stats
}

#[test]
pub(super) fn our_contacts_match_upstream_f32_within_4ulp() {
    let all = cases();
    for case in &all {
        let stats = compare_vs_mjwarp(case);
        println!(
            "[判据] {:<16} count {}/{} | dist 位等 {}/{} max {} ULP | pos 位等接触 {}/{} max {:.2} ULP@尺度 | frame 位等分量 {}/{} max abs {:e}",
            case.name,
            stats.product_count,
            stats.reference_count,
            stats.dist_zero,
            stats.reference_count,
            stats.dist_max_ulp,
            stats.pos_zero_contacts,
            stats.reference_count,
            stats.pos_max_scaled_ulp,
            stats.frame_zero,
            stats.reference_count * 9,
            stats.frame_max_abs,
        );
        assert_eq!(stats.product_count, stats.reference_count, "{}", case.name);
        // 4 ULP 硬判据见 `upstream_f32_within_4ulp`（本文件即正式门禁）。
    }
}
