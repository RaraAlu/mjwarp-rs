//! G2 契约框架读数（E 表）与 native 探索项：不作 G2 判据，供 G6 `native_error_v2` 引用。

use super::case_compare::{
    RefContact, compare_case, compare_vs_mjwarp, product_contacts, reference_contacts,
    sort_contacts, upstream_mjwarp_contacts,
};
use super::case_data::{Case, cases, root};

/// E 表的一行分量（`ours` 或 `mjwarp` 对 native 的绝对差）。
#[derive(Default)]
struct ErrorBuckets {
    values: Vec<f32>,
}

impl ErrorBuckets {
    fn push(&mut self, value: f32) {
        self.values.push(value);
    }

    fn summary(&self) -> (f32, f32, f32, f32) {
        if self.values.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut sorted = self.values.clone();
        sorted.sort_by(f32::total_cmp);
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = sorted[sorted.len() / 2];
        let over =
            sorted.iter().filter(|&&value| value > 1.0e-6).count() as f32 / sorted.len() as f32;
        (min, median, max, over)
    }
}

/// `ours vs native` 与 `mjwarp vs native` 的 E 表（契约框架读数，不作判据）。
fn error_tables(case: &Case) -> (ErrorBuckets, ErrorBuckets) {
    let mut ours = ErrorBuckets::default();
    let mut mjwarp = ErrorBuckets::default();
    let native = reference_contacts(case);
    let mj = upstream_mjwarp_contacts(case);
    for world in 0..case.worlds.len() {
        let mut native_rows: Vec<RefContact> = native
            .iter()
            .filter(|contact| contact.worldid == world as i32)
            .copied()
            .collect();
        let mut mj_rows: Vec<RefContact> = mj
            .iter()
            .filter(|contact| contact.worldid == world as i32)
            .copied()
            .collect();
        let mut product = product_contacts(case, world, false);
        sort_contacts(&mut native_rows);
        sort_contacts(&mut mj_rows);
        sort_contacts(&mut product);
        for ((native, product), mj) in native_rows.iter().zip(product.iter()).zip(mj_rows.iter()) {
            ours.push((product.dist - native.dist).abs());
            mjwarp.push((mj.dist - native.dist).abs());
            for axis in 0..3 {
                ours.push((product.pos[axis] - native.pos[axis]).abs());
                mjwarp.push((mj.pos[axis] - native.pos[axis]).abs());
            }
            for cell in 0..9 {
                ours.push((product.frame[cell] - native.frame[cell]).abs());
                mjwarp.push((mj.frame[cell] - native.frame[cell]).abs());
            }
        }
    }
    (ours, mjwarp)
}

/// 按田野口径导出 G6 种子数据（`G2_DUMP_NATIVE_ERROR=1` 时写证据目录）。
fn maybe_dump_error_seed(cases: &[Case]) {
    if std::env::var("G2_DUMP_NATIVE_ERROR").is_err() {
        return;
    }
    let mut payload = String::from(
        "{\"schema\":1,\"task_id\":\"2026-10-25-g10-p5-gjk-epa\",\"kind\":\"g2-native-error-seed\",\"note\":\"契约框架读数（ours/mjwarp 对 native f64 绝对差；不作 G2 判据，供 G6 native 容差候选）\",\"fields\":{",
    );
    let (mut ours, mut mjwarp) = (ErrorBuckets::default(), ErrorBuckets::default());
    for case in cases {
        let (case_ours, case_mj) = error_tables(case);
        ours.values.extend(case_ours.values);
        mjwarp.values.extend(case_mj.values);
    }
    let (min, median, max, over) = ours.summary();
    payload += &format!(
        "\"ours_vs_native\":{{\"count\":{},\"min\":{min:e},\"median\":{median:e},\"max\":{max:e},\"over_1e6_ratio\":{over}}},",
        ours.values.len()
    );
    let (min, median, max, over) = mjwarp.summary();
    payload += &format!(
        "\"mjwarp_vs_native\":{{\"count\":{},\"min\":{min:e},\"median\":{median:e},\"max\":{max:e},\"over_1e6_ratio\":{over}}}",
        mjwarp.values.len()
    );
    payload += "}}";
    let path = root().join("docs/development-evidence/g10-p5-gjk-epa/native-error-seed.json");
    std::fs::write(&path, payload).expect("写出 native E 种子数据");
    println!("wrote {}", path.display());
}

#[test]
fn our_contacts_match_upstream_f32_within_4ulp() {
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

/// 契约框架读数（不作判据）：`ours vs native` 与 `mjwarp vs native` 的 E 分布，G6 种子。
#[test]
fn native_error_seed_is_recorded() {
    let all = cases();
    for case in &all {
        let (ours, mjwarp) = error_tables(case);
        let (min, median, max, over) = ours.summary();
        println!(
            "[E] {:<16} ours-vs-native  min {min:e} median {median:e} max {max:e} >1e-6 {:.3}",
            case.name, over
        );
        let (min, median, max, over) = mjwarp.summary();
        println!(
            "[E] {:<16} mj-vs-native    min {min:e} median {median:e} max {max:e} >1e-6 {:.3}",
            case.name, over
        );
    }
    maybe_dump_error_seed(&all);
}

/// 正式判据：ours vs 同配方上游 f32（CPU 后端，输入直接取上游 f32 运动学位姿）全部分量 ≤4 ULP。
#[test]
fn upstream_f32_within_4ulp() {
    for case in &cases() {
        let stats = compare_vs_mjwarp(case);
        assert_eq!(stats.product_count, stats.reference_count, "{}", case.name);
        assert!(
            stats.dist_max_ulp <= 4,
            "{} dist {} ULP",
            case.name,
            stats.dist_max_ulp
        );
        assert!(
            stats.pos_max_scaled_ulp <= 4.0,
            "{} pos {:.2}",
            case.name,
            stats.pos_max_scaled_ulp
        );
        assert!(
            stats.frame_max_abs <= 4.8e-7,
            "{} frame {:e}",
            case.name,
            stats.frame_max_abs
        );
    }
}

/// 探索项（不作门禁）：native 位级不可达（f64 参考），仅记录。
#[test]
#[ignore = "探索项：native 是 f64，f32 实现不可能位级一致，不作门禁"]
fn native_bitwise_exploration() {
    for case in &cases() {
        let stats = compare_case(case, false, false);
        assert_eq!(stats.product_count, stats.reference_count);
        assert!(stats.dist_max_ulp <= 4);
    }
}
