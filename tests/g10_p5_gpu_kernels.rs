//! G10@P5 G5：设备内核 GPU 对照入口（独立 nvcc 探针输出 vs 上游 CUDA 参考）。
//!
//! 前置（GPU 窗口内，见证据 `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/`）：
//! 1. `prepare_cases.py --device cuda` → `cases/<case>/`（输入 + 本窗口参考绑定）；
//! 2. `build_probe.ps1` → `target/g10-p5-g5/probe.exe`（同一批 device 源、fmad=true）；
//! 3. 探针每 case 跑两次：`probe.exe cases/<case> 2` → `out1|out2/`。
//!
//! 判据与全部比对逻辑见证据目录 `gpu-window/gpu_compare.rs` 与 `gpu-window/witness_compare.rs`
//! （Arch 2026-10-25 裁决：frame 按 scale=1.0 的 4 ULP 绝对差，dist 按自身量级 ULP，
//! pos/x1/x2 按位置向量的 scale=max_i|ref_i| 取 4·ULP(scale)，参数/整数位级；
//! witness 对照对直调上游 `collision_gjk.ccd` 的 dist/x1/x2）。
//!
//! 运行（GPU 窗口结束后，CPU 侧）：
//! `cargo test --locked --features cuda-probe --test g10_p5_gpu_kernels -- --ignored --test-threads=1`

#![cfg(feature = "cuda-probe")]
// 证据对照测试：字节解析/索引循环刻意直白；按需豁免对应 clippy 提示。
#![allow(clippy::chunks_exact_to_as_chunks)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_range_loop)]

#[path = "../docs/development-evidence/g10-p5-gpu-kernels/gpu-window/gpu_compare.rs"]
mod gpu_compare;
#[path = "../docs/development-evidence/g10-p5-gpu-kernels/gpu-window/witness_compare.rs"]
mod witness_compare;

use serde_json::Value;

#[test]
#[ignore = "需要先在本机 GPU 窗口跑 prepare_cases.py --device cuda + build_probe.ps1 + probe.exe"]
fn device_contact_arrays_match_upstream_cuda_reference() {
    let (result, failures) = gpu_compare::run_contact_comparison();
    let path = gpu_compare::root()
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/result.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&result).expect("json"))
        .expect("写 result.json");
    println!("wrote {}", path.display());
    assert!(
        failures.is_empty(),
        "设备对照未全部通过（未通过 case: {failures:?}）；明细见 {}",
        path.display()
    );
}

/// witness 对照（Arch 裁决 1）：`cylinder-box`/`box-box-margin`/`mesh-mesh` 三例
/// 设备 `g5_ccd`（`witness_probe.exe`）vs 上游 CUDA 直调 `collision_gjk.ccd`
/// （`capture_witness_ccd.py --device cuda`）：dist/x1/x2 ≤4 ULP、ncontact 位级。
#[test]
#[ignore = "需要 GPU 窗口跑 witness_probe.exe + capture_witness_ccd.py --device cuda"]
fn device_ccd_witness_matches_upstream_ccd() {
    let (result, failures) = witness_compare::run_witness_comparison();
    let path = witness_compare::root()
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/witness-result.json");
    std::fs::write(&path, serde_json::to_vec_pretty(&result).expect("json"))
        .expect("写 witness-result.json");
    println!("wrote {}", path.display());
    assert!(
        failures.is_empty(),
        "witness 对照未全部通过（未通过 case: {failures:?}）；明细见 {}",
        path.display()
    );
}

/// ellipsoid-box 三方对照之 ①：宿主 `ccd_pair`（G2，已对上游 CPU ≤4 ULP）
/// 用**同一探针输入**重跑并与上游 CUDA fixture（③）比较。纯 CPU。
#[test]
#[ignore = "需要 gpu-window/cases/ellipsoid-box 输入（prepare_cases.py 产物）"]
fn ellipsoid_box_host_ccd_pair_vs_cuda_reference() {
    use mjwarp_rs::physics::{CcdOptions, ContactOptions, Geom, ccd_pair};

    let directory = gpu_compare::cases_dir().join("ellipsoid-box");
    let read = |name: &str| -> Vec<f32> {
        let bytes = std::fs::read(directory.join(format!("{name}.bin"))).expect("bin");
        bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    };
    let meta: std::collections::BTreeMap<String, String> =
        std::fs::read_to_string(directory.join("meta.txt"))
            .expect("meta")
            .lines()
            .filter_map(|line| line.split_once(' '))
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect();
    let meta_f32 = |key: &str| -> f32 { meta[key].trim().parse().expect("f32 meta") };
    let meta_i32 = |key: &str| -> i32 { meta[key].trim().parse().expect("i32 meta") };

    let nworld = meta_i32("nworld") as usize;
    let ngeom = meta_i32("ngeom") as usize;
    let g1 = meta_i32("g1") as usize;
    let g2 = meta_i32("g2") as usize;
    let xpos = read("geom_xpos");
    let xmat = read("geom_xmat");
    let size = read("geom_size");
    let margin = read("geom_margin");
    let options = ContactOptions {
        ccd: CcdOptions {
            tolerance: meta_f32("tolerance"),
            cutoff: meta_f32("gap"),
            gjk_iterations: meta_i32("gjk_iterations") as usize,
            epa_iterations: meta_i32("epa_iterations") as usize,
        },
        margin: meta_f32("margin"),
        gap: meta_f32("gap"),
        collision_sensor: false,
        use_multiccd: true,
    };
    let reference_text = std::fs::read_to_string(
        gpu_compare::root().join("fixtures/g10-p5/references/ellipsoid-box.json"),
    )
    .expect("ref");
    let reference: Value = serde_json::from_str(&reference_text).expect("ref json");
    let ref_count = reference["output"]["count"].as_u64().expect("count") as usize;
    let ref_values = |name: &str| -> Vec<f32> {
        reference["output"]["fields"][name]["values"]
            .as_array()
            .expect("values")
            .iter()
            .map(|item| item.as_f64().expect("f32") as f32)
            .collect()
    };
    let ref_width = |name: &str| -> usize {
        if ref_count == 0 {
            0
        } else {
            ref_values(name).len() / ref_count
        }
    };

    let mut max_dist_ulp = 0_u32;
    let mut max_pos_abs = 0.0_f32;
    let mut max_frame_abs = 0.0_f32;
    let mut hits = 0_usize;
    let mut host_vs_cpu_dist = 0.0_f32;
    let mut host_vs_cpu_pos = 0.0_f32;
    let mut host_vs_cpu_frame = 0.0_f32;
    for world in 0..nworld {
        let geom = |index: usize| {
            let base = (world * ngeom + index) * 3;
            let rot_base = (world * ngeom + index) * 9;
            let mut rot = [0.0_f32; 9];
            rot.copy_from_slice(&xmat[rot_base..rot_base + 9]);
            Geom::simple(
                [xpos[base], xpos[base + 1], xpos[base + 2]],
                rot,
                [size[index * 3], size[index * 3 + 1], size[index * 3 + 2]],
                margin[index],
            )
        };
        let output = ccd_pair(
            &geom(g1),
            &geom(g2),
            meta_i32("type1"),
            meta_i32("type2"),
            options,
        );
        for contact in &output.contacts {
            // 在参考里按 worldid + dist/pos 位型找最近行（本 case 每 world 一个接触）。
            let dist_width = ref_width("dist");
            let pos_width = ref_width("pos");
            let frame_width = ref_width("frame");
            let mut best: Option<usize> = None;
            let mut best_diff = f32::MAX;
            for index in 0..ref_count {
                let ref_dist = ref_values("dist")[index * dist_width];
                let diff = (ref_dist - contact.dist).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best = Some(index);
                }
            }
            let Some(index) = best else { continue };
            hits += 1;
            let ref_dist = ref_values("dist")[index * dist_width];
            let ulp = {
                let key = |value: f32| -> i64 {
                    let bits = value.to_bits() as i32;
                    if bits < 0 {
                        0x8000_0000_i64 - i64::from(bits)
                    } else {
                        i64::from(bits)
                    }
                };
                (key(contact.dist) - key(ref_dist)).unsigned_abs() as u32
            };
            max_dist_ulp = max_dist_ulp.max(ulp);
            let ref_pos = &ref_values("pos")[index * pos_width..(index + 1) * pos_width];
            for axis in 0..3 {
                max_pos_abs = max_pos_abs.max((contact.pos[axis] - ref_pos[axis]).abs());
            }
            let ref_frame = &ref_values("frame")[index * frame_width..(index + 1) * frame_width];
            for cell in 0..9 {
                max_frame_abs = max_frame_abs.max((contact.frame[cell] - ref_frame[cell]).abs());
            }
        }
    }
    println!(
        "host ccd_pair ellipsoid-box: hits={hits} dist_max_ulp={max_dist_ulp} \
         pos_max_abs={max_pos_abs:.3e} frame_max_abs={max_frame_abs:.3e}"
    );
    assert_eq!(hits, nworld, "每 world 至少一个接触");
    assert!(max_dist_ulp <= 4, "宿主 dist ULP={max_dist_ulp} 超过 4");
    assert!(
        max_pos_abs <= 1.0e-6,
        "宿主 pos 绝对差 {max_pos_abs} 超过 1e-6"
    );
    // 与 CUDA fixture 的 frame 差 = 上游 CPU/GPU 自身差（证据 ellipsoid-box-cpu-capture.json:
    // 8.19e-7），宿主目标是上游 CPU；位级确认见下方 CPU fixture 对比。
    assert!(
        max_frame_abs <= 1.0e-6,
        "宿主 frame 绝对差 {max_frame_abs} 超过上游 CPU/GPU 自身差量级 1e-6"
    );

    // 与上游 CPU fixture 位级对比（②）：宿主应与上游 CPU 一致。
    let cpu_text = std::fs::read_to_string(gpu_compare::root().join(
        "docs/development-evidence/g10-p5-gpu-kernels/gpu-window/ellipsoid-box-cpu-capture.json",
    ))
    .expect("cpu capture");
    let cpu: Value = serde_json::from_str(&cpu_text).expect("cpu json");
    let cpu_field = |name: &str| -> Vec<f32> {
        cpu["cpu_contact_fields"][name]
            .as_array()
            .expect("cpu field")
            .iter()
            .map(|item| item.as_f64().expect("f32") as f32)
            .collect()
    };
    let cpu_dist = cpu_field("dist");
    let cpu_pos = cpu_field("pos");
    let cpu_frame = cpu_field("frame");
    let cpu_count = cpu_dist.len();

    for world in 0..nworld {
        let geom = |index: usize| {
            let base = (world * ngeom + index) * 3;
            let rot_base = (world * ngeom + index) * 9;
            let mut rot = [0.0_f32; 9];
            rot.copy_from_slice(&xmat[rot_base..rot_base + 9]);
            Geom::simple(
                [xpos[base], xpos[base + 1], xpos[base + 2]],
                rot,
                [size[index * 3], size[index * 3 + 1], size[index * 3 + 2]],
                margin[index],
            )
        };
        let output = ccd_pair(
            &geom(g1),
            &geom(g2),
            meta_i32("type1"),
            meta_i32("type2"),
            options,
        );
        for contact in &output.contacts {
            // 两个 world 的 dist 可能相同：按 pos 最近匹配行。
            let mut best = 0_usize;
            let mut best_diff = f32::MAX;
            for index in 0..cpu_count {
                let mut diff = 0.0_f32;
                for axis in 0..3 {
                    diff += (cpu_pos[index * 3 + axis] - contact.pos[axis]).abs();
                }
                if diff < best_diff {
                    best_diff = diff;
                    best = index;
                }
            }

            host_vs_cpu_dist = host_vs_cpu_dist.max((contact.dist - cpu_dist[best]).abs());
            for axis in 0..3 {
                host_vs_cpu_pos =
                    host_vs_cpu_pos.max((contact.pos[axis] - cpu_pos[best * 3 + axis]).abs());
            }
            for cell in 0..9 {
                host_vs_cpu_frame =
                    host_vs_cpu_frame.max((contact.frame[cell] - cpu_frame[best * 9 + cell]).abs());
            }
        }
    }
    println!(
        "host ccd_pair vs upstream CPU fixture: dist_max_abs={host_vs_cpu_dist:.3e} \
         pos_max_abs={host_vs_cpu_pos:.3e} frame_max_abs={host_vs_cpu_frame:.3e}"
    );
    // 宿主与上游 CPU 的差应在 CPU/CUDA 自身差量级（dist 1.86e-9 / pos 7.45e-9 / frame 8.19e-7）。
    assert!(
        host_vs_cpu_dist <= 1.0e-7,
        "宿主 vs CPU dist 差 {host_vs_cpu_dist}"
    );
    assert!(
        host_vs_cpu_pos <= 1.0e-6,
        "宿主 vs CPU pos 差 {host_vs_cpu_pos}"
    );
    assert!(
        host_vs_cpu_frame <= 2.0e-6,
        "宿主 vs CPU frame 差 {host_vs_cpu_frame}"
    );
}
