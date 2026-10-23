//! LTOIR 接口的输入校验、冻结选项与摘要稳定性测试。
//! 真实 NVRTC 编译在`tests/compile_ltoir_probe.rs`的 ignored GPU 探针内。

use super::super::HeaderFile;
use super::*;
use crate::runtime::launcher::LauncherError;

const FROZEN_OPTIONS: [&str; LTOIR_OPTION_COUNT] = [
    "--gpu-architecture=sm_89",
    "--std=c++17",
    "--Ofast-compile=0",
    "--define-macro=NDEBUG",
    "--undefine-macro=WP_VERIFY_FP",
    "--define-macro=WP_ENABLE_MATHDX=1",
    "--fmad=true",
    "--device-as-default-execution-space",
    "--extra-device-vectorization",
    "--restrict",
    "--diag-suppress=177,550",
    "-dlto",
    "--relocatable-device-code=true",
];

fn expected_options(arch: u32) -> Vec<String> {
    let mut options: Vec<String> = FROZEN_OPTIONS.iter().map(|s| (*s).to_owned()).collect();
    options[0] = format!("--gpu-architecture=sm_{arch}");
    options
}

#[test]
fn options_are_frozen_and_ordered() {
    let options = ltoir_options(89).unwrap();
    assert_eq!(options.len(), LTOIR_OPTION_COUNT);
    assert_eq!(options, expected_options(89));
}

#[test]
fn options_arch_bounds_and_substitution() {
    assert_eq!(ltoir_options(MIN_ARCH).unwrap(), expected_options(MIN_ARCH));
    assert_eq!(ltoir_options(MAX_ARCH).unwrap(), expected_options(MAX_ARCH));
    assert!(ltoir_options(MIN_ARCH - 1).is_err());
    assert!(ltoir_options(MAX_ARCH + 1).is_err());
}

#[test]
fn digest_is_stable() {
    assert_eq!(
        super::digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        super::digest(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn rejects_invalid_names() {
    for name in ["", "a b", "a\0b", "a/b"] {
        assert!(
            LtoirInput::new(name, "void kernel() {}", 89).is_err(),
            "应拒绝源名{name:?}"
        );
    }
    assert!(LtoirInput::new(&"a".repeat(129), "void kernel() {}", 89).is_err());
    assert!(LtoirInput::new(&"a".repeat(128), "void kernel() {}", 89).is_ok());
}

#[test]
fn rejects_invalid_sources() {
    assert!(LtoirInput::new("ok.cu", "", 89).is_err());
    assert!(LtoirInput::new("ok.cu", "void kernel() {}\0", 89).is_err());
    let oversized = "a".repeat(MAX_LTOIR_SOURCE_BYTES + 1);
    assert!(LtoirInput::new("ok.cu", &oversized, 89).is_err());
}

#[test]
fn rejects_invalid_arch_and_include_paths() {
    assert!(LtoirInput::new("ok.cu", "void kernel() {}", 49).is_err());
    assert!(LtoirInput::new("ok.cu", "void kernel() {}", 130).is_err());

    let header = HeaderFile::new("m/head.h", "#pragma once\n").unwrap();
    let bad_paths = [String::new(), "x\0y".to_owned(), "a".repeat(1025)];
    for path in &bad_paths {
        assert!(
            LtoirInput::with_headers(
                "ok.cu",
                "void kernel() {}",
                std::slice::from_ref(&header),
                std::slice::from_ref(path),
                89
            )
            .is_err(),
            "应拒绝 include 路径{path:?}"
        );
    }
}

#[test]
fn accepts_valid_headers_and_include_paths() {
    let header = HeaderFile::new("m/head.h", "#pragma once\n").unwrap();
    let paths = [String::from("C:/cuda/include")];
    let input = LtoirInput::with_headers(
        "ok.cu",
        "void kernel() {}",
        std::slice::from_ref(&header),
        &paths,
        89,
    )
    .unwrap();
    assert_eq!(input.name(), "ok.cu");
    assert_eq!(input.source(), "void kernel() {}");
    assert_eq!(input.headers().len(), 1);
    assert_eq!(input.include_paths(), paths);
    assert_eq!(input.arch(), 89);
}

#[test]
fn nvrtc_unavailable_is_explicit() {
    assert_eq!(
        ensure_nvrtc(false).unwrap_err(),
        LauncherError::Compiler("NVRTC不可用".into())
    );
    assert!(ensure_nvrtc(true).is_ok());
}
