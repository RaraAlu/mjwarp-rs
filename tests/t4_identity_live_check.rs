//! T4 身份活体校验（QA 复核观察 4 的补齐片）。
//!
//! 目的：让 20 条 `fixtures/precision/t4-current-kernels.json::product_sources` 的身份漂移
//! 在**默认特性、纯 CPU** 的常规测试里直接暴露（此前 19/20 只覆盖在 `#[ignore]` 的 recapture
//! 测试内，默认门禁跑不到）。
//!
//! 口径：单文件 = `sha256(LF 归一化字节)`；`src/physics/mod.rs` = physics_core 组合摘要
//! （锚点 5 文件 LF 归一化后以 `\n` 连接）。期望值只来自 fixture，测试不复制任何期望值。
//!
//! 反向判别 seam：`T4_IDENTITY_LIVE_FIXTURE` 可指向临时 fixture 副本（测试专用），
//! 用于验证「篡改期望值 → 测试失败」；默认不设置时读仓库内 fixture。

#[path = "support/t4_identity_live.rs"]
mod live;

use std::collections::BTreeSet;

/// 20 条 `product_sources` 逐个与活体源码 LF 归一化 sha256 比对。
#[test]
fn live_product_sources_match_fixture() {
    let fixture = live::load_fixture();
    let fixture_label = live::display_path(&live::fixture_path());

    assert_eq!(
        fixture.len(),
        live::EXPECTED_PATHS.len(),
        "{} 的 product_sources 条数应为 {}，实际 {}",
        fixture_label,
        live::EXPECTED_PATHS.len(),
        fixture.len()
    );

    let fixture_paths: BTreeSet<&str> = fixture.iter().map(|entry| entry.path.as_str()).collect();
    let expected_paths: BTreeSet<&str> = live::EXPECTED_PATHS.iter().copied().collect();
    assert_eq!(
        fixture_paths.len(),
        fixture.len(),
        "{} 的 product_sources 存在重复路径",
        fixture_label
    );

    let missing: Vec<&str> = expected_paths.difference(&fixture_paths).copied().collect();
    let unexpected: Vec<&str> = fixture_paths.difference(&expected_paths).copied().collect();
    assert!(
        missing.is_empty() && unexpected.is_empty(),
        "T4 身份路径集合不完整（新增/删除源码或未重钉？）：缺少 {missing:?}，多出 {unexpected:?}"
    );

    let mismatches = live::verify_all(&fixture);
    assert!(
        mismatches.is_empty(),
        "T4 身份活体校验失败（{} 条差异，共 {} 条）：\n{}",
        mismatches.len(),
        fixture.len(),
        mismatches.join("\n")
    );
}

/// 自实现 SHA-256 与 LF 归一化的已知向量自检（防止支撑代码本身出错导致假绿/假红）。
#[test]
fn sha256_and_lf_normalization_match_known_vectors() {
    assert_eq!(
        live::sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        live::sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(live::normalize_lf(b"a\r\nb\rc\n"), b"a\nb\rc\n".to_vec());
    assert_eq!(live::normalize_lf(b"\r\n\r\n"), b"\n\n".to_vec());
}
