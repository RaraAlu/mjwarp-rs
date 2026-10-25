//! G10@P5 前置片 G2：GJK/EPA（`ccd`）对 pinned 上游参考的数值对照。
//!
//! 输入 = 冻结上游 `71da24d956…` 的 MJCF 场景 + 参考 qpos 前向运动学后的 geom 世界位姿；
//! 对照 = `fixtures/g10-p5/references/*.json`（G0b 补采）与
//! `fixtures/mjwarp-complex-physics/reference.json`（d04 既有）的接触输出。
//!
//! 只覆盖上游 `MJ_COLLISION_TABLE` 走 `CONVEX`（ccd/GJK/EPA）的组合：
//! `cylinder-box`、`ellipsoid-box`、`mesh-mesh`、`convex-mesh`、`degenerate-mesh`。
//! 同批 G0b 参考里的 `sphere-box-*`/`capsule-box`/`box-box-margin` 实际走 `PRIMITIVE` 表，
//! `hfield-*`/`sdf-box` 归 G4/G3（见证据 README 的分类更正）。
//!
//! 判据沿用 G0b 结论：按 `worldid` 分组 + 每 world 内 `(dist, pos)` 稳定排序后比较。

use mjwarp_rs::physics::{CcdOptions, ContactOptions, Geom, MeshView, ccd_pair, geom_type};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const UPSTREAM_REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";

// 以下数据由 authoring 导出（mujoco 3.15.0 CPU 前向运动学 f64→f32 一次转换）；
// 产品测试不启动 Python。字段：pair 为上游 `nxn_geom_pair` 排序后的模型 geom 下标，
// worlds 为每世界的 geom_xpos / 行优先 geom_xmat。
const CASE_DATA: &str = r#"{"mujoco":"3.15.0","cases":[{"name":"cylinder-box","reference":"fixtures/g10-p5/references/cylinder-box.json","standalone":true,"pair":[1,0],"geoms":[{"type":6,"size":[0.10000000149011612,0.07999999821186066,0.05999999865889549]},{"type":5,"size":[0.07999999821186066,0.10000000149011612,0.0]}],"worlds":[{"pos":[[0.0,0.0,0.0],[0.03999999910593033,0.03999999910593033,0.10000000149011612]],"rot":[[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0],[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]]},{"pos":[[0.0,0.0,0.0],[0.04999999701976776,0.04999999701976776,0.10000000149011612]],"rot":[[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0],[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]]}]},{"name":"ellipsoid-box","reference":"fixtures/g10-p5/references/ellipsoid-box.json","standalone":true,"pair":[1,0],"geoms":[{"type":6,"size":[0.10000000149011612,0.07999999821186066,0.05999999865889549]},{"type":4,"size":[0.10000000149011612,0.05999999865889549,0.05000000074505806]}],"worlds":[{"pos":[[0.0,0.0,0.0],[0.03999999910593033,0.0,0.10000000149011612]],"rot":[[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0],[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]]},{"pos":[[0.0,0.0,0.0],[0.04999999701976776,0.0,0.10000000149011612]],"rot":[[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0],[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]]}]},{"name":"mesh-mesh","reference":"fixtures/g10-p5/references/mesh-mesh.json","standalone":true,"pair":[0,1],"geoms":[{"type":7,"size":[0.08660253882408142,0.1414213627576828,0.1632993221282959]},{"type":7,"size":[0.08660253882408142,0.1414213627576828,0.1632993221282959]}],"worlds":[{"pos":[[0.05000000074505806,0.05000000074505806,0.05000000074505806],[0.10000000149011612,0.10000000149011612,0.10000000149011612]],"rot":[[0.5773502588272095,-0.7071067690849304,0.40824830532073975,0.5773502588272095,0.0,-0.8164966106414795,0.5773502588272095,0.7071067690849304,0.40824830532073975],[0.5773502588272095,-0.7071067690849304,0.40824830532073975,0.5773502588272095,0.0,-0.8164966106414795,0.5773502588272095,0.7071067690849304,0.40824830532073975]]},{"pos":[[0.05000000074505806,0.05000000074505806,0.05000000074505806],[0.10999999940395355,0.10999999940395355,0.10000000149011612]],"rot":[[0.5773502588272095,-0.7071067690849304,0.40824830532073975,0.5773502588272095,0.0,-0.8164966106414795,0.5773502588272095,0.7071067690849304,0.40824830532073975],[0.5773502588272095,-0.7071067690849304,0.40824830532073975,0.5773502588272095,0.0,-0.8164966106414795,0.5773502588272095,0.7071067690849304,0.40824830532073975]]}],"mesh":{"vert":[-0.08660253882408142,-1.3877787807814457e-17,-6.938893903907228e-18,0.028867514804005623,-0.1414213627576828,0.08164966106414795,0.028867514804005623,-1.3877787807814457e-17,-0.1632993221282959,0.028867514804005623,0.1414213627576828,0.08164966106414795],"polynormal":[1.0,1.3492982063365808e-16,7.790176574276581e-17,-0.5773502588272095,-0.7071067690849304,-0.40824830532073975,-0.5773502588272095,0.7071067690849304,-0.40824830532073975,-0.5773502588272095,2.2216770511661084e-18,0.8164966106414795],"polyvertadr":[0,3,6,9],"polyvertnum":[3,3,3,3],"polyvert":[2,3,1,2,1,0,3,2,0,1,3,0],"polymapadr":[0,3,6,9],"polymapnum":[3,3,3,3],"polymap":[1,2,3,0,1,3,0,1,2,0,2,3]}},{"name":"convex-mesh","reference":"fixtures/mjwarp-complex-physics/reference.json","standalone":false,"pair":[1,0],"geoms":[{"type":7,"size":[0.08660253882408142,0.1414213627576828,0.1632993221282959]},{"type":6,"size":[0.07999999821186066,0.05999999865889549,0.05000000074505806]}],"worlds":[{"pos":[[-0.029999999329447746,-0.029999999329447746,0.05000000074505806],[0.03999999910593033,0.03999999910593033,0.03999999910593033]],"rot":[[0.5773502588272095,-0.7071067690849304,0.40824830532073975,0.5773502588272095,0.0,-0.8164966106414795,0.5773502588272095,0.7071067690849304,0.40824830532073975],[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]]},{"pos":[[-0.029999999329447746,-0.029999999329447746,0.05000000074505806],[0.04999999701976776,0.029999999329447746,0.03999999910593033]],"rot":[[0.5773502588272095,-0.7071067690849304,0.40824830532073975,0.5773502588272095,0.0,-0.8164966106414795,0.5773502588272095,0.7071067690849304,0.40824830532073975],[1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]]}],"mesh":{"vert":[-0.08660253882408142,-1.3877787807814457e-17,-6.938893903907228e-18,0.028867514804005623,-0.1414213627576828,0.08164966106414795,0.028867514804005623,-1.3877787807814457e-17,-0.1632993221282959,0.028867514804005623,0.1414213627576828,0.08164966106414795],"polynormal":[1.0,1.3492982063365808e-16,7.790176574276581e-17,-0.5773502588272095,-0.7071067690849304,-0.40824830532073975,-0.5773502588272095,0.7071067690849304,-0.40824830532073975,-0.5773502588272095,2.2216770511661084e-18,0.8164966106414795],"polyvertadr":[0,3,6,9],"polyvertnum":[3,3,3,3],"polyvert":[2,3,1,2,1,0,3,2,0,1,3,0],"polymapadr":[0,3,6,9],"polymapnum":[3,3,3,3],"polymap":[1,2,3,0,1,3,0,1,2,0,2,3]}},{"name":"degenerate-mesh","reference":"fixtures/mjwarp-complex-physics/reference.json","standalone":false,"pair":[0,1],"geoms":[{"type":7,"size":[0.03004816733300686,0.18442554771900177,0.24597066640853882]},{"type":7,"size":[0.03004816733300686,0.18442554771900177,0.24597066640853882]}],"worlds":[{"pos":[[-0.07286406308412552,-0.023445287719368935,-0.07747935503721237],[0.2104691118001938,-0.023445291444659233,-0.07747935503721237]],"rot":[[0.09023859351873398,-0.9502652883529663,0.2980819642543793,0.9947072863578796,0.07122993469238281,-0.07405228912830353,0.049136966466903687,0.30318668484687805,0.9516634941101074],[0.09023865312337875,-0.9502652883529663,0.2980819642543793,0.9947072863578796,0.07122998684644699,-0.07405231148004532,0.049136966466903687,0.30318668484687805,0.9516634941101074]]},{"pos":[[-0.07286406308412552,-0.023445287719368935,-0.07747935503721237],[0.2104691118001938,-0.023445291444659233,-0.07747935503721237]],"rot":[[0.09023859351873398,-0.9502652883529663,0.2980819642543793,0.9947072863578796,0.07122993469238281,-0.07405228912830353,0.049136966466903687,0.30318668484687805,0.9516634941101074],[0.09023865312337875,-0.9502652883529663,0.2980819642543793,0.9947072863578796,0.07122998684644699,-0.07405231148004532,0.049136966466903687,0.30318668484687805,0.9516634941101074]]}],"mesh":{"vert":[0.02811409719288349,-0.10028816014528275,-0.05529490485787392,0.0010772814275696874,0.18442554771900177,-0.14460472762584686,-0.03004816733300686,-0.09847100079059601,-0.08439502120018005,-0.014452625997364521,0.002027445239946246,0.24551062285900116,0.014270758256316185,-0.004071722272783518,0.24597066640853882,0.00954477209597826,-0.09646763652563095,-0.14404939115047455],"polynormal":[0.9786424040794373,0.029098734259605408,-0.20350021123886108,-0.9936192035675049,0.11205407977104187,0.012836298905313015,-0.010831007733941078,-0.999110996723175,-0.04074167460203171,-0.8324959874153137,-0.026189586147665977,-0.5534117221832275,0.18179075419902802,0.8879857659339905,0.42241379618644714,-0.17555741965770721,-0.9401118159294128,0.29218044877052307,-0.2030288428068161,-0.9339592456817627,0.29410773515701294,0.9949952960014343,0.09891811013221741,0.014128759503364563],"polyvertadr":[0,3,6,9,12,15,18,21],"polyvertnum":[3,3,3,3,3,3,3,3],"polyvert":[0,5,1,1,2,3,2,5,0,5,2,1,4,1,3,4,2,0,2,4,3,4,0,1],"polymapadr":[0,4,9,14,17,21],"polymapnum":[4,5,5,3,4,3],"polymap":[0,2,5,7,0,1,3,4,7,1,2,3,5,6,1,4,6,4,5,6,7,0,2,3]}}]}"#;

#[derive(Default)]
struct MeshData {
    vert: Vec<[f32; 3]>,
    polynormal: Vec<[f32; 3]>,
    polyvertadr: Vec<i32>,
    polyvertnum: Vec<i32>,
    polyvert: Vec<i32>,
    polymapadr: Vec<i32>,
    polymapnum: Vec<i32>,
    polymap: Vec<i32>,
}

struct WorldPose {
    pos: Vec<[f32; 3]>,
    rot: Vec<[f32; 9]>,
}

struct Case {
    name: String,
    reference: String,
    standalone: bool,
    pair: [usize; 2],
    geomtypes: Vec<i32>,
    sizes: Vec<[f32; 3]>,
    mesh: Option<MeshData>,
    worlds: Vec<WorldPose>,
}

fn floats(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .expect("数值数组")
        .iter()
        .map(|item| item.as_f64().expect("数值") as f32)
        .collect()
}

fn vec3(value: &Value) -> [f32; 3] {
    let values = floats(value);
    [values[0], values[1], values[2]]
}

fn vec9(value: &Value) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    out.copy_from_slice(&floats(value));
    out
}

fn cases() -> Vec<Case> {
    let value: Value = serde_json::from_str(CASE_DATA).expect("内嵌数据应为合法 JSON");
    value["cases"]
        .as_array()
        .expect("cases 数组")
        .iter()
        .map(|entry| {
            let mesh = entry.get("mesh").map(|mesh| MeshData {
                vert: floats(&mesh["vert"])
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect(),
                polynormal: floats(&mesh["polynormal"])
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect(),
                polyvertadr: floats(&mesh["polyvertadr"])
                    .iter()
                    .map(|&v| v as i32)
                    .collect(),
                polyvertnum: floats(&mesh["polyvertnum"])
                    .iter()
                    .map(|&v| v as i32)
                    .collect(),
                polyvert: floats(&mesh["polyvert"])
                    .iter()
                    .map(|&v| v as i32)
                    .collect(),
                polymapadr: floats(&mesh["polymapadr"])
                    .iter()
                    .map(|&v| v as i32)
                    .collect(),
                polymapnum: floats(&mesh["polymapnum"])
                    .iter()
                    .map(|&v| v as i32)
                    .collect(),
                polymap: floats(&mesh["polymap"]).iter().map(|&v| v as i32).collect(),
            });
            Case {
                name: entry["name"].as_str().expect("name").to_owned(),
                reference: entry["reference"].as_str().expect("reference").to_owned(),
                standalone: entry["standalone"].as_bool().expect("standalone"),
                pair: [
                    entry["pair"][0].as_u64().expect("pair0") as usize,
                    entry["pair"][1].as_u64().expect("pair1") as usize,
                ],
                geomtypes: entry["geoms"]
                    .as_array()
                    .expect("geoms")
                    .iter()
                    .map(|geom| geom["type"].as_i64().expect("type") as i32)
                    .collect(),
                sizes: entry["geoms"]
                    .as_array()
                    .expect("geoms")
                    .iter()
                    .map(|geom| vec3(&geom["size"]))
                    .collect(),
                mesh,
                worlds: entry["worlds"]
                    .as_array()
                    .expect("worlds")
                    .iter()
                    .map(|world| WorldPose {
                        pos: world["pos"]
                            .as_array()
                            .expect("pos")
                            .iter()
                            .map(vec3)
                            .collect(),
                        rot: world["rot"]
                            .as_array()
                            .expect("rot")
                            .iter()
                            .map(vec9)
                            .collect(),
                    })
                    .collect(),
            }
        })
        .collect()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ccd_options() -> CcdOptions {
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
struct RefContact {
    worldid: i32,
    dist: f32,
    pos: [f32; 3],
    frame: [f32; 9],
}

fn reference_contacts(case: &Case) -> Vec<RefContact> {
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
    assert_eq!(dist.len(), count);
    (0..count)
        .map(|index| RefContact {
            worldid: worldid[index] as i32,
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

fn mesh_view<'a>(case: &'a Case) -> Option<MeshView<'a>> {
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

fn product_contacts(case: &Case, world: usize) -> Vec<RefContact> {
    let mesh = mesh_view(case);
    let mut geoms = [Geom::simple([0.0; 3], [0.0; 9], [0.0; 3], 0.0); 2];
    for slot in 0..2 {
        let gid = case.pair[slot];
        geoms[slot] = Geom::simple(
            case.worlds[world].pos[gid],
            case.worlds[world].rot[gid],
            case.sizes[gid],
            0.0,
        );
        if let Some(view) = mesh.as_ref() {
            geoms[slot].vertadr = 0;
            geoms[slot].vertnum = view.vert.len() as i32;
            geoms[slot].graphadr = -1;
            geoms[slot].mesh = Some(view);
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
        .map(|contact| RefContact {
            worldid: world as i32,
            dist: contact.dist,
            pos: contact.pos,
            frame: contact.frame,
        })
        .collect()
}

fn sort_contacts(contacts: &mut [RefContact]) {
    contacts.sort_by(|left, right| {
        left.dist
            .total_cmp(&right.dist)
            .then(left.pos[0].total_cmp(&right.pos[0]))
            .then(left.pos[1].total_cmp(&right.pos[1]))
            .then(left.pos[2].total_cmp(&right.pos[2]))
    });
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

#[derive(Default)]
struct CaseStats {
    reference_count: usize,
    product_count: usize,
    dist_zero: usize,
    dist_max_ulp: u32,
    pos_max_scaled_ulp: f32,
    pos_zero_contacts: usize,
    frame_zero: usize,
    frame_max_abs: f32,
}

fn compare_case(case: &Case) -> CaseStats {
    let reference = reference_contacts(case);
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
        let mut actual = product_contacts(case, world);
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

#[test]
fn ccd_contacts_match_pinned_reference() {
    let all = cases();
    let stats = all
        .iter()
        .map(|case| (case, compare_case(case)))
        .collect::<Vec<_>>();
    for (case, stats) in &stats {
        println!(
            "case {:<16} count {}/{} | dist 位等 {}/{} max {} ULP | pos 位等接触 {}/{} max {:.2} ULP@尺度 | frame 位等分量 {}/{} max abs {:e}",
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
    for (case, stats) in &stats {
        assert_eq!(stats.product_count, stats.reference_count, "{}", case.name);
        // 记录预算（非物理容限放宽）：见证据 README 的 ULP 分布与超差判定。
        assert!(
            stats.dist_max_ulp <= 32,
            "{} dist {} ULP",
            case.name,
            stats.dist_max_ulp
        );
        assert!(
            stats.pos_max_scaled_ulp <= 8.0,
            "{} pos {:.2} ULP@尺度",
            case.name,
            stats.pos_max_scaled_ulp
        );
        assert!(
            stats.frame_max_abs <= 1.0e-6,
            "{} frame {:e}",
            case.name,
            stats.frame_max_abs
        );
    }
}

/// 场景分类守卫：本片只对照上游 `CONVEX` 表的组合。
#[test]
fn convex_cases_are_convex_pairs() {
    let convex = |value: i32| {
        matches!(
            value,
            geom_type::ELLIPSOID | geom_type::CYLINDER | geom_type::BOX | geom_type::MESH
        )
    };
    for case in &cases() {
        assert!(convex(case.geomtypes[case.pair[0]]));
        assert!(convex(case.geomtypes[case.pair[1]]));
    }
}

#[test]
fn geom_type_constants_match_upstream() {
    assert_eq!(geom_type::PLANE, 0);
    assert_eq!(geom_type::HFIELD, 1);
    assert_eq!(geom_type::SPHERE, 2);
    assert_eq!(geom_type::CAPSULE, 3);
    assert_eq!(geom_type::ELLIPSOID, 4);
    assert_eq!(geom_type::CYLINDER, 5);
    assert_eq!(geom_type::BOX, 6);
    assert_eq!(geom_type::MESH, 7);
    assert_eq!(geom_type::SDF, 8);
}
