//! 路线枚举、产物键稳定性与拒绝面测试；真实 MathDx 管线在 GPU 探针内。

use super::*;
use crate::runtime::launcher::{ArtifactKey, FrozenPtx};

const CALL_SOURCE: &str = "extern \"C\" __global__ void route_kernel() { }";

fn source(text: &'static str) -> NvrtcSource {
    NvrtcSource::new("route-test", text).unwrap()
}

fn mathdx_call() -> LtoirInput<'static> {
    LtoirInput::new("native-mathdx7-call.cu", CALL_SOURCE, 89).unwrap()
}

fn fake_cubin() -> Vec<u8> {
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"\x7fELF");
    bytes[4] = 2;
    bytes[5] = 1;
    bytes[18..20].copy_from_slice(&190u16.to_le_bytes());
    bytes
}

fn sample_artifact() -> KernelArtifact {
    let call = mathdx_call();
    let solver = SolverSpec::potrf_7d_sm89();
    let key = RouteKey::for_mathdx_lto(&call, &solver, (12, 9), (12, 9), (0, 3, 1)).unwrap();
    let call_bytes = b"call-ltoir".to_vec();
    let solver_ltoir = b"solver-ltoir".to_vec();
    let solver_fatbin = b"solver-fatbin".to_vec();
    let cubin = fake_cubin();
    let metadata = KernelArtifactMetadata {
        route: KernelSourceRoute::MathDxLto,
        key_sha256: key.sha256(),
        source_sha256: key.source_sha256().to_owned(),
        header_sha256: key.header_sha256().to_owned(),
        arch: key.arch(),
        nvrtc_version: (12, 9),
        nvjitlink_version: (12, 9),
        libmathdx_version: (0, 3, 1),
        call_ltoir_bytes: call_bytes.len(),
        call_ltoir_sha256: sha256_hex(&call_bytes),
        solver_ltoir_bytes: solver_ltoir.len(),
        solver_ltoir_sha256: sha256_hex(&solver_ltoir),
        solver_fatbin_bytes: solver_fatbin.len(),
        solver_fatbin_sha256: sha256_hex(&solver_fatbin),
        cubin_bytes: cubin.len(),
        cubin_sha256: sha256_hex(&cubin),
    };
    KernelArtifact {
        key,
        metadata,
        call_ltoir: call_bytes,
        solver_ltoir,
        solver_fatbin,
        cubin,
    }
}

#[test]
fn four_routes_are_enumerable_and_disabled_by_default() {
    assert_eq!(KernelSourceRoute::ALL.len(), 4);
    let mut names: Vec<&str> = KernelSourceRoute::ALL.iter().map(|r| r.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 4);
    for route in KernelSourceRoute::ALL {
        assert!(
            !route.enabled_by_default(),
            "路线{}默认启用",
            route.as_str()
        );
        assert!(!route.as_str().is_empty());
    }
}

#[test]
fn route_opt_in_is_required_and_checked() {
    let error = require_route_enabled(RouteOptIn::none(), KernelSourceRoute::MathDxLto)
        .unwrap_err()
        .to_string();
    assert!(error.contains("路线未启用"), "错误面不符：{error}");
    assert!(error.contains("MathDxLto"), "错误面不符：{error}");
    assert!(
        require_route_enabled(
            RouteOptIn::enable(KernelSourceRoute::MathDxLto),
            KernelSourceRoute::MathDxLto
        )
        .is_ok()
    );
    let error = require_route_enabled(
        RouteOptIn::enable(KernelSourceRoute::HandWrittenCuda),
        KernelSourceRoute::MathDxLto,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("路线不符"), "错误面不符：{error}");
}

#[test]
fn nvrtc_version_gate_rejects_other_versions() {
    assert!(require_mathdx_nvrtc((12, 9)).is_ok());
    for version in [(12, 8), (12, 10), (11, 9), (13, 0)] {
        let error = require_mathdx_nvrtc(version).unwrap_err().to_string();
        assert!(error.contains("NVRTC版本不符"), "错误面不符：{error}");
        assert!(error.contains("不回退"), "错误面不符：{error}");
    }
}

#[test]
fn cuda_route_keys_are_stable_and_factor_sensitive() {
    let src = source(CALL_SOURCE);
    let physics = CompileRecipe::WARP_1_15_PHYSICS;
    let base =
        RouteKey::for_cuda_route(KernelSourceRoute::HandWrittenCuda, &src, &physics, (12, 8))
            .unwrap();
    let again =
        RouteKey::for_cuda_route(KernelSourceRoute::HandWrittenCuda, &src, &physics, (12, 8))
            .unwrap();
    assert_eq!(base, again);
    assert_eq!(base.sha256(), again.sha256());
    assert_eq!(base.route(), KernelSourceRoute::HandWrittenCuda);
    assert_eq!(base.nvrtc(), Some((12, 8)));

    let changed_source = RouteKey::for_cuda_route(
        KernelSourceRoute::HandWrittenCuda,
        &source("extern \"C\" __global__ void route_kernel_2() { }"),
        &physics,
        (12, 8),
    )
    .unwrap();
    assert_ne!(base.sha256(), changed_source.sha256(), "源变化键未变");

    let diagnostic = RouteKey::for_cuda_route(
        KernelSourceRoute::HandWrittenCuda,
        &src,
        &CompileRecipe::WARP_1_15_DIAGNOSTIC,
        (12, 8),
    )
    .unwrap();
    assert_ne!(base.sha256(), diagnostic.sha256(), "配方变化键未变");

    let upstream =
        RouteKey::for_cuda_route(KernelSourceRoute::UpstreamSource, &src, &physics, (12, 8))
            .unwrap();
    assert_ne!(base.sha256(), upstream.sha256(), "路线变化键未变");

    let other_nvrtc =
        RouteKey::for_cuda_route(KernelSourceRoute::HandWrittenCuda, &src, &physics, (11, 8))
            .unwrap();
    assert_ne!(base.sha256(), other_nvrtc.sha256(), "NVRTC版本变化键未变");
}

#[test]
fn cuda_route_rejects_non_cuda_routes() {
    let src = source(CALL_SOURCE);
    for route in [KernelSourceRoute::FrozenPtx, KernelSourceRoute::MathDxLto] {
        assert!(
            RouteKey::for_cuda_route(route, &src, &CompileRecipe::WARP_1_15_PHYSICS, (12, 8))
                .is_err()
        );
    }
}

#[test]
fn headers_and_option_order_change_key() {
    let header_a = HeaderFile::new("warp/a.h", "#pragma once\nA\n").unwrap();
    let header_b = HeaderFile::new("warp/b.h", "#pragma once\nB\n").unwrap();
    let source_ab = source(CALL_SOURCE)
        .with_header(header_a.clone())
        .unwrap()
        .with_header(header_b.clone())
        .unwrap();
    let source_ba = source(CALL_SOURCE)
        .with_header(header_b)
        .unwrap()
        .with_header(header_a)
        .unwrap();
    let recipe = CompileRecipe::WARP_1_15_PHYSICS;
    let ab = RouteKey::for_cuda_route(
        KernelSourceRoute::UpstreamSource,
        &source_ab,
        &recipe,
        (12, 8),
    )
    .unwrap();
    let ba = RouteKey::for_cuda_route(
        KernelSourceRoute::UpstreamSource,
        &source_ba,
        &recipe,
        (12, 8),
    )
    .unwrap();
    assert_ne!(ab.sha256(), ba.sha256(), "头文件顺序变化键未变");

    let changed = RouteKey::for_cuda_route(
        KernelSourceRoute::UpstreamSource,
        &source(CALL_SOURCE)
            .with_header(HeaderFile::new("warp/a.h", "#pragma once\nCHANGED\n").unwrap())
            .unwrap(),
        &recipe,
        (12, 8),
    )
    .unwrap();
    assert_ne!(ab.sha256(), changed.sha256(), "头文件内容变化键未变");

    let option_a = RouteKey::new(
        KernelSourceRoute::HandWrittenCuda,
        sha256_hex(b"source"),
        sha256_hex(&[]),
        Vec::new(),
        vec!["--a".to_owned(), "--b".to_owned()],
        80,
        None,
        None,
        Some((12, 8)),
        None,
        None,
    )
    .unwrap();
    let option_b = RouteKey::new(
        KernelSourceRoute::HandWrittenCuda,
        sha256_hex(b"source"),
        sha256_hex(&[]),
        Vec::new(),
        vec!["--b".to_owned(), "--a".to_owned()],
        80,
        None,
        None,
        Some((12, 8)),
        None,
        None,
    )
    .unwrap();
    assert_ne!(option_a.sha256(), option_b.sha256(), "选项顺序变化键未变");
}

#[test]
fn generic_key_rejects_invalid_digests() {
    let not_hex = "z".repeat(64);
    assert!(
        RouteKey::new(
            KernelSourceRoute::HandWrittenCuda,
            not_hex,
            sha256_hex(&[]),
            Vec::new(),
            Vec::new(),
            80,
            None,
            None,
            Some((12, 8)),
            None,
            None,
        )
        .is_err()
    );
}

#[test]
fn frozen_ptx_route_key_is_stable() {
    let text = ".version 8.0\n.target sm_80\n";
    let sha = sha256_hex(text.as_bytes());
    let frozen = FrozenPtx::from_ptx("frozen-route", text, &sha).unwrap();
    let key = RouteKey::for_frozen_ptx(&frozen).unwrap();
    assert_eq!(key.route(), KernelSourceRoute::FrozenPtx);
    assert_eq!(
        key.sha256(),
        RouteKey::for_frozen_ptx(&frozen).unwrap().sha256()
    );
    assert_eq!(key.arch(), 0);
}

#[test]
fn mathdx_route_key_tracks_versions_and_solver() {
    let call = mathdx_call();
    let solver = SolverSpec::potrf_7d_sm89();
    let base = RouteKey::for_mathdx_lto(&call, &solver, (12, 9), (12, 9), (0, 3, 1)).unwrap();
    let again = RouteKey::for_mathdx_lto(&call, &solver, (12, 9), (12, 9), (0, 3, 1)).unwrap();
    assert_eq!(base, again);
    assert_eq!(base.route(), KernelSourceRoute::MathDxLto);
    assert_eq!(base.options().len(), 13);
    assert_eq!(base.options()[0], "--gpu-architecture=sm_89");
    assert_eq!(base.nvrtc(), Some((12, 9)));
    assert_eq!(base.nvjitlink(), Some((12, 9)));
    assert_eq!(base.libmathdx(), Some((0, 3, 1)));

    for changed in [
        RouteKey::for_mathdx_lto(&call, &solver, (12, 8), (12, 9), (0, 3, 1)).unwrap(),
        RouteKey::for_mathdx_lto(&call, &solver, (12, 9), (12, 8), (0, 3, 1)).unwrap(),
        RouteKey::for_mathdx_lto(&call, &solver, (12, 9), (12, 9), (0, 3, 0)).unwrap(),
        {
            let mut other = solver;
            other.m = 8;
            RouteKey::for_mathdx_lto(&call, &other, (12, 9), (12, 9), (0, 3, 1)).unwrap()
        },
    ] {
        assert_ne!(base.sha256(), changed.sha256(), "因子变化键未变");
    }
}

#[test]
fn mathdx_route_rejects_arch_mismatch() {
    let call = LtoirInput::new("call.cu", CALL_SOURCE, 90).unwrap();
    let solver = SolverSpec::potrf_7d_sm89();
    let error = RouteKey::for_mathdx_lto(&call, &solver, (12, 9), (12, 9), (0, 3, 1))
        .unwrap_err()
        .to_string();
    assert!(error.contains("架构不一致"), "错误面不符：{error}");
}

#[test]
fn artifact_verify_accepts_consistent_and_rejects_tampering() {
    let good = sample_artifact();
    good.verify().unwrap();
    assert_eq!(good.metadata().cubin_sha256, sha256_hex(good.cubin()));

    let mut tampered = sample_artifact();
    tampered.cubin[6] ^= 0xff;
    let error = tampered.verify().unwrap_err().to_string();
    assert!(error.contains("产物键与内容不一致"), "错误面不符：{error}");

    let mut empty = sample_artifact();
    empty.cubin.clear();
    assert!(empty.verify().unwrap_err().to_string().contains("空产物"));

    let mut not_elf = sample_artifact();
    not_elf.cubin = vec![0u8; 64];
    not_elf.metadata.cubin_bytes = not_elf.cubin.len();
    not_elf.metadata.cubin_sha256 = sha256_hex(&not_elf.cubin);
    assert!(
        not_elf
            .verify()
            .unwrap_err()
            .to_string()
            .contains("不是CUBIN ELF")
    );

    let mut bad_key = sample_artifact();
    bad_key.metadata.key_sha256 = "0".repeat(64);
    assert!(bad_key.verify().unwrap_err().to_string().contains("键摘要"));

    let mut bad_route = sample_artifact();
    bad_route.metadata.route = KernelSourceRoute::HandWrittenCuda;
    assert!(bad_route.verify().unwrap_err().to_string().contains("路线"));
}

#[test]
fn existing_recipe_artifact_key_baseline_is_locked() {
    // 本任务不改 artifact 管线；该基线锁定既有配方产物键行为。
    let key = ArtifactKey::new(
        &source(CALL_SOURCE),
        &CompileRecipe::WARP_1_15_PHYSICS,
        (12, 8),
    );
    println!("existing_recipe_key_sha256={}", key.sha256());
    assert_eq!(key.nvrtc(), (12, 8));
    assert_eq!(key.recipe(), CompileRecipe::WARP_1_15_PHYSICS.fingerprint());
    assert_eq!(
        key.sha256(),
        "b515f1ebfbf8127e13e3e76b737cf30625bb2cd8662213a5a7cc6f665dc16b29"
    );
}
