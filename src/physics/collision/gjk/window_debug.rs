//! G5 诊断（仅测试，诊断-only）：在宿主 `gjk` 内部导出与设备
//! `gpu-window/device_logic_probe.cpp` 相同的中间量，用于 `ellipsoid-box` witness 差异定位。
//!
//! 输入：`docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/ellipsoid-box/`
//! 运行：`cargo test --locked --features cuda-probe --lib window_debug -- --ignored --nocapture`
//! 该测试不参与产品语义、不进产品路径。
#![allow(clippy::chunks_exact_to_as_chunks)]

use super::epa::epa;
use super::geom_type;
use super::phase::gjk_phase;
use super::polytope_build::{polytope2, polytope3, polytope4};
use super::{Geom, Polytope};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn cases_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/ellipsoid-box")
}

fn read_meta(directory: &std::path::Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(directory.join("meta.txt"))
        .expect("meta")
        .lines()
        .filter_map(|line| line.split_once(' '))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn read_f32(directory: &std::path::Path, name: &str) -> Vec<f32> {
    let bytes = std::fs::read(directory.join(format!("{name}.bin"))).expect("bin");
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[test]
#[ignore = "诊断：需要 gpu-window/cases/ellipsoid-box 输入（prepare_cases.py 产物）"]
fn host_intermediates_for_ellipsoid_box() {
    let directory = cases_dir();
    let meta = read_meta(&directory);
    let key = |name: &str| -> &str { meta[name].trim() };
    let nworld: usize = key("nworld").parse().expect("nworld");
    let ngeom: usize = key("ngeom").parse().expect("ngeom");
    let g1: usize = key("g1").parse().expect("g1");
    let g2: usize = key("g2").parse().expect("g2");
    let type1: i32 = key("type1").parse().expect("type1");
    let type2: i32 = key("type2").parse().expect("type2");
    let tolerance: f32 = key("tolerance").parse().expect("tolerance");
    let margin: f32 = key("margin").parse().expect("margin");
    let gap: f32 = key("gap").parse().expect("gap");
    let gjk_iterations: usize = key("gjk_iterations").parse().expect("gjk iterations");
    let epa_iterations: usize = key("epa_iterations").parse().expect("epa iterations");

    let xpos = read_f32(&directory, "geom_xpos");
    let xmat = read_f32(&directory, "geom_xmat");
    let size = read_f32(&directory, "geom_size");

    for world in 0..nworld {
        let build = |index: usize| {
            let base = (world * ngeom + index) * 3;
            let rot_base = (world * ngeom + index) * 9;
            let mut rot = [0.0_f32; 9];
            rot.copy_from_slice(&xmat[rot_base..rot_base + 9]);
            Geom::simple(
                [xpos[base], xpos[base + 1], xpos[base + 2]],
                rot,
                [size[index * 3], size[index * 3 + 1], size[index * 3 + 2]],
                0.0,
            )
        };
        let mut geom1 = build(g1);
        let mut geom2 = build(g2);
        geom1.margin = margin;
        geom2.margin = margin;
        let origin = geom1.pos;
        geom1.pos = [0.0; 3];
        geom2.pos = [
            geom2.pos[0] - origin[0],
            geom2.pos[1] - origin[1],
            geom2.pos[2] - origin[2],
        ];

        let phase = gjk_phase(
            tolerance,
            gap,
            gjk_iterations,
            &geom1,
            &geom2,
            type1,
            type2,
            geom1.pos,
            geom2.pos,
        );
        println!(
            "world={world} phase needs_epa={} gjk_dist={:?} ncontact={} dim={}",
            phase.needs_epa, phase.dist, phase.ncontact, phase.result.dim
        );
        if !phase.needs_epa {
            continue;
        }
        // 复刻 epa_phase 的 polytope 构造，但直接调用 epa 拿 face_idx。
        let mut result = phase.result;
        let mut pt = Polytope::with_capacity(epa_iterations);
        pt.nface = 0;
        pt.nvert = 0;
        pt.nhorizon = 0;
        pt.status = 0;
        if result.dim == 2 {
            let (new_pt, new_result) = polytope2(
                pt,
                &result.simplex,
                &result.simplex1,
                &result.simplex2,
                &result.simplex_index1,
                &result.simplex_index2,
                &phase.geom1,
                &phase.geom2,
                type1,
                type2,
            );
            pt = new_pt;
            if pt.status == -1 {
                result.simplex = new_result.simplex;
                result.simplex1 = new_result.simplex1;
                result.simplex2 = new_result.simplex2;
                result.simplex_index1 = new_result.simplex_index1;
                result.simplex_index2 = new_result.simplex_index2;
                result.dim = 3;
            }
        } else if result.dim == 4 {
            let (new_pt, new_result) = polytope4(
                pt,
                &result.simplex,
                &result.simplex1,
                &result.simplex2,
                &result.simplex_index1,
                &result.simplex_index2,
            );
            pt = new_pt;
            if pt.status == -1 {
                result.simplex = new_result.simplex;
                result.simplex1 = new_result.simplex1;
                result.simplex2 = new_result.simplex2;
                result.simplex_index1 = new_result.simplex_index1;
                result.simplex_index2 = new_result.simplex_index2;
                result.dim = 3;
            }
        }
        if result.dim == 3 {
            pt = polytope3(
                pt,
                result.dist,
                &result.simplex,
                &result.simplex1,
                &result.simplex2,
                &result.simplex_index1,
                &result.simplex_index2,
                &phase.geom1,
                &phase.geom2,
                type1,
                type2,
            );
        }
        if pt.status != 0 {
            println!("world={world} epa skipped status={}", pt.status);
            continue;
        }
        let is_discrete = super::core::is_discrete(type1, type2, geom1.margin, geom2.margin);
        let epa_result = epa(
            pt,
            tolerance,
            epa_iterations,
            &phase.geom1,
            &phase.geom2,
            type1,
            type2,
            is_discrete,
        );
        println!(
            "world={world} epa face_idx={} epa_dist={:?} x1={:?} x2={:?} overflow={}",
            epa_result.face_idx, epa_result.dist, epa_result.x1, epa_result.x2, epa_result.overflow
        );
        let face_index = epa_result.face_idx;
        if face_index < 0 {
            continue;
        }
        let face_index = face_index as usize;
        let polytope = &epa_result.polytope;
        let lower = polytope.face_norm2[face_index].sqrt();
        let pr = polytope.face_pr[face_index];
        let normal = [pr[0] / lower, pr[1] / lower, pr[2] / lower];
        let face = [
            (polytope.face[face_index] & 0x3FF) as usize,
            ((polytope.face[face_index] >> 10) & 0x3FF) as usize,
            ((polytope.face[face_index] >> 20) & 0x3FF) as usize,
        ];
        let v1 = super::math::sub3(polytope.vert[2 * face[0]], polytope.vert[2 * face[0] + 1]);
        let v2 = super::math::sub3(polytope.vert[2 * face[1]], polytope.vert[2 * face[1] + 1]);
        let v3 = super::math::sub3(polytope.vert[2 * face[2]], polytope.vert[2 * face[2] + 1]);
        let weights = super::simplex::tri_affine_coord(v1, v2, v3, polytope.face_pr[face_index]);
        println!(
            "world={world} face lower={:?} normal={:?} verts=({},{},{}) w={:?} pr={:?}",
            lower, normal, face[0], face[1], face[2], weights, pr
        );
        println!(
            "world={world} face_v1={:?} face_v2={:?} face_v3={:?}",
            polytope.vert[2 * face[0]],
            polytope.vert[2 * face[1]],
            polytope.vert[2 * face[2]]
        );
        let final_dist = epa_result.dist + margin;
        let sum = super::math::add3(epa_result.x1, epa_result.x2);
        let pos = super::math::add3(super::math::scale3(sum, 0.5), origin);
        println!("world={world} witness final_dist={final_dist:?} pos={pos:?}");
    }
    let _ = (geom_type::BOX, geom_type::ELLIPSOID);
}
