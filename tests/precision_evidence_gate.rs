//! NP02 证据门禁（第一片）：已提交证据清单的**静态棘轮 + 指针可解析**断言。
//!
//! 口径：`docs/numerical-precision-contract.md` 的 NP02（精度证据覆盖率 100%、证据缺失 0）是目标，
//! 本测试不给出 NP02 通过结论；它只保证已登记的 Windows 单平台证据清单不被静默退化：
//!
//! 1. **计数棘轮**：`docs/development-evidence/np02-evidence/inventory.json` 的 `counts` 必须与
//!    本测试钉住的快照完全一致（100 条已实现路径 / 96 内核 / 3 冻结链 / 1 MathDx 路线 /
//!    100 有证据 / 0 缺失 / 100% 覆盖 / 89 源身份一致 / 9 漂移 / 713 回执 / 709 实体 / 32 缺口 …）。
//!    清单数字变化必须由人显式更新这里，不允许证据或缺口在无人注意时"消失"。
//! 2. **指针可解析**：每一行的 `source.path`（若非空）与 `artifact.evidence_paths` 每一项
//!    都必须指向仓内存在的文件；不允许把不存在的路径写成证据。
//! 3. **状态词表**：`evidence_state` 只能是 `verified` / `source-drift` / `derived` / `missing`
//!    （9 条已知漂移按用户 2026-10-24 裁决 B 归档，不重采不重基线；2026-11-04 的 12 个新增内核
//!    已由 `2026-11-04-precision-np02-new-kernel-evidence` 补齐回执并转 `verified`）；
//!    `compile.precision_gate` 只允许 `f32-physics` 或 `f32-render`，其中经 W15 头树编译的
//!    `aa_resolve` / `s7a_abi_probe` 必须是 `f32-render`。
//! 4. **上游 pin**：MJWarp `71da24d…`、Warp `ffa038b1…` 两个冻结提交必须写在清单里。
//!
//! 更新历史：
//! - 2026-11-04 `2026-11-03-precision-np02-receipt-rehash` 重算后同步快照
//!   （旧值 88/84/88/0/100%/9/701/650/32 存 `docs/development-evidence/np02-evidence/history/`）。
//! - 2026-11-04 `2026-11-04-precision-np02-new-kernel-evidence` 补 12 条 CPU-only NVRTC 回执后
//!   同步快照（88→100 有证据、12→0 缺失、88%→100% 覆盖、701→713 回执、697→709 实体、44→32 缺口；
//!   上一版清单留在 git 历史）。
//!
//! 本测试不覆盖：重新跑 np02 扫描工具（它需要 `target/mjwarp-parallel` 上游 checkout，
//! 仓库瘦身后不在仓；2026-11-04 曾人工+工具重算一次，见 `np02-evidence/rehash-2026-11-04/`）、
//! 回执文件哈希重算、PTX 实体哈希；也不宣称 NP02 通过、不宣称双平台。
//! 零 GPU、零网络、只读文件。

use std::fs;
use std::path::{Path, PathBuf};

const INVENTORY: &str = "docs/development-evidence/np02-evidence/inventory.json";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_inventory() -> serde_json::Value {
    let path = manifest_dir().join(INVENTORY);
    serde_json::from_str(
        &fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .expect("np02 inventory.json must be valid JSON")
}

fn assert_exists(root: &Path, relative: &str) {
    let path = root.join(relative);
    assert!(
        path.is_file(),
        "NP02 证据指针指向不存在的文件：{relative}（不允许把缺失路径写成证据）"
    );
}

#[test]
fn np02_counts_match_the_registered_snapshot() {
    let inventory = load_inventory();
    let counts = &inventory["counts"];
    let expected: [(&str, f64); 16] = [
        ("windows_implemented_paths", 100.0),
        ("kernel_rows", 96.0),
        ("frozen_chain_rows", 3.0),
        ("mathdx_route_rows", 1.0),
        ("evidence_present", 100.0),
        ("evidence_verified", 89.0),
        ("evidence_missing", 0.0),
        ("coverage_pct_evidence", 100.0),
        ("coverage_pct_source_identity", 89.0),
        ("source_identity_drift", 9.0),
        ("receipts_total", 713.0),
        ("receipts_file_hash_ok", 713.0),
        ("receipt_ptx_entities_verified", 709.0),
        ("gap_items", 32.0),
        ("unimplemented", 7.0),
        ("undetermined", 5.0),
    ];
    for (key, expected_value) in expected {
        let actual = counts[key]
            .as_f64()
            .unwrap_or_else(|| panic!("counts.{key} 缺失或非数字"));
        assert_eq!(
            actual, expected_value,
            "counts.{key} 由 {expected_value} 变为 {actual}：证据/缺口清单变化必须显式更新本快照并复核"
        );
    }
    assert_eq!(
        inventory["rows"].as_array().expect("rows").len(),
        100,
        "证据清单行数必须为 100"
    );
    assert_eq!(
        inventory["gaps"].as_array().expect("gaps").len(),
        32,
        "缺口项数必须为 32"
    );
}

#[test]
fn np02_evidence_pointers_resolve() {
    let root = manifest_dir();
    let inventory = load_inventory();
    let rows = inventory["rows"].as_array().expect("rows");
    let mut checked = 0usize;
    for row in rows {
        let name = row["name"].as_str().unwrap_or("<unnamed>");
        if let Some(source_path) = row["source"]["path"].as_str() {
            // 派生源行写的是说明文字（「（派生源，无独立 const）」），不是路径：只核对像路径的项。
            if !source_path.is_empty()
                && source_path.contains('/')
                && !source_path.starts_with('（')
            {
                assert_exists(&root, source_path);
                checked += 1;
            }
        }
        for evidence_path in row["artifact"]["evidence_paths"]
            .as_array()
            .unwrap_or(&Vec::new())
        {
            let relative = evidence_path
                .as_str()
                .unwrap_or_else(|| panic!("{name} 的 evidence_paths 含非字符串项"));
            assert_exists(&root, relative);
            checked += 1;
        }
    }
    assert!(
        checked >= 255,
        "可解析证据指针只数到 {checked}，少于已登记的 255（证据不得被静默移除）"
    );
}

#[test]
fn np02_new_kernel_rows_stay_verified() {
    // 2026-11-04 `2026-11-04-precision-np02-new-kernel-evidence` 补齐回执的 12 个内核：
    // 它们曾因「回执索引无该 entry」被判 missing（覆盖 88%）；此断言防止回执/指针被静默移除。
    const NEW_ROWS: [&str; 12] = [
        "build_rays",
        "mesh_bvh_ray",
        "scene_bvh_ray",
        "s7a_abi_probe",
        "aa_resolve",
        "sample_texture2d",
        "mjwarp_collision_boxbox",
        "mjwarp_collision_contact",
        "mjwarp_collision_contact_multi",
        "mjwarp_collision_hfield",
        "mjwarp_collision_primitive",
        "mjwarp_collision_sdf",
    ];
    let inventory = load_inventory();
    let rows = inventory["rows"].as_array().expect("rows");
    for name in NEW_ROWS {
        let row = rows
            .iter()
            .find(|row| row["name"] == name)
            .unwrap_or_else(|| panic!("{name} 行必须留在清单里"));
        assert_eq!(
            row["evidence_state"].as_str(),
            Some("verified"),
            "{name} 的 2026-11-04 回执必须保持 verified"
        );
        let receipts = row["artifact"]["receipts"].as_u64().unwrap_or(0);
        assert!(receipts >= 1, "{name} 必须至少 1 份回执");
        let ptx = row["artifact"]["ptx_sha256"]
            .as_array()
            .expect("ptx_sha256");
        assert!(!ptx.is_empty(), "{name} 必须有 ptx 摘要");
    }
}

#[test]
fn np02_states_and_pins_stay_on_the_registered_vocabulary() {
    let inventory = load_inventory();
    let rows = inventory["rows"].as_array().expect("rows");
    for row in rows {
        let name = row["name"].as_str().unwrap_or("<unnamed>");
        let state = row["evidence_state"].as_str().expect("evidence_state");
        assert!(
            matches!(state, "verified" | "source-drift" | "derived" | "missing"),
            "{name} 的 evidence_state={state} 不在登记词表（verified/source-drift/derived/missing）内"
        );
        let route = row["route"].as_str().expect("route");
        assert!(
            matches!(route, "HandWrittenCuda" | "FrozenPtx" | "MathDxLto"),
            "{name} 的 route={route} 不在登记词表内"
        );
        if route == "HandWrittenCuda" {
            let gate = row["compile"]["precision_gate"]
                .as_str()
                .unwrap_or_else(|| panic!("{name} 缺 compile.precision_gate"));
            assert!(
                matches!(gate, "f32-physics" | "f32-render"),
                "{name} 的精度门禁 = {gate}，不在登记词表（f32-physics / f32-render）内"
            );
            if matches!(name, "aa_resolve" | "s7a_abi_probe") {
                assert_eq!(
                    gate, "f32-render",
                    "{name} 经 W15 头树（builtin.h→svd.h）编译，门禁必须是 f32-render；\
                     两条 f64 死数据常量的定位与引用检查见 np02-evidence/new-kernels-2026-11/README.md"
                );
            } else {
                assert_eq!(gate, "f32-physics", "{name} 的精度门禁必须为 f32-physics");
            }
        }
    }
    let pins = &inventory["pins"];
    assert_eq!(
        pins["mjwarp"]["expected"].as_str(),
        Some("71da24d956378a87a703b6e1442b13aec0c4ac29"),
        "MJWarp pin 不得漂移"
    );
    assert_eq!(
        pins["warp"]["expected"].as_str(),
        Some("ffa038b1adf3927a8e893d4536f1a7562f17d749"),
        "Warp pin 不得漂移"
    );
    assert_eq!(
        inventory["platform"]["os"].as_str(),
        Some("Windows"),
        "本清单只覆盖 Windows 单平台，不得被改成双平台"
    );
}
