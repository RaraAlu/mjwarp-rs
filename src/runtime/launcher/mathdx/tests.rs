//! MathDx 绑定的输入校验、版本核对与拒绝面测试。
//! 真实描述符调用在`tests/mathdx_solver_probe.rs`的 ignored GPU 探针内。

use super::*;
use std::path::Path;

#[test]
fn version_check_is_pinned() {
    super::check_version(MATHDX_VERSION).unwrap();
    for version in [(0, 3, 0), (0, 4, 0), (1, 0, 0), (0, 2, 9)] {
        assert!(
            super::check_version(version).is_err(),
            "应拒绝版本{version:?}"
        );
    }
}

#[test]
fn frozen_potrf_spec_matches_contract() {
    let spec = SolverSpec::potrf_7d_sm89();
    assert_eq!((spec.m, spec.n, spec.k), (7, 7, 1));
    assert_eq!(spec.function, 2);
    assert_eq!((spec.side, spec.diag), (-1, -1));
    assert_eq!(spec.precision, 5);
    assert_eq!((spec.arrangement_a, spec.arrangement_b), (1, 1));
    assert_eq!(spec.fill_mode, 0);
    assert_eq!(spec.num_threads, 32);
    assert_eq!(spec.arch, 89);
    assert_eq!(spec.symbol, "potrf_7_7_1_89_32_1_1_5_x_x_0");
    spec.validate().unwrap();
}

#[test]
fn rejects_invalid_shapes() {
    for value in [0, -1, MAX_DIM + 1] {
        let mut spec = SolverSpec::potrf_7d_sm89();
        spec.m = value;
        assert!(spec.validate().is_err(), "应拒绝M={value}");
        let mut spec = SolverSpec::potrf_7d_sm89();
        spec.n = value;
        assert!(spec.validate().is_err(), "应拒绝N={value}");
        let mut spec = SolverSpec::potrf_7d_sm89();
        spec.k = value;
        assert!(spec.validate().is_err(), "应拒绝K={value}");
    }
}

#[test]
fn rejects_invalid_threads_and_arch() {
    for value in [0, -32, 33, MAX_THREADS + 32] {
        let mut spec = SolverSpec::potrf_7d_sm89();
        spec.num_threads = value;
        assert!(spec.validate().is_err(), "应拒绝线程数{value}");
    }
    for value in [49, 130] {
        let mut spec = SolverSpec::potrf_7d_sm89();
        spec.arch = value;
        assert!(spec.validate().is_err(), "应拒绝sm_{value}");
    }
}

#[test]
fn rejects_invalid_enums() {
    fn reject(change: impl FnOnce(&mut SolverSpec<'_>)) {
        let mut spec = SolverSpec::potrf_7d_sm89();
        change(&mut spec);
        assert!(spec.validate().is_err());
    }
    reject(|spec| spec.function = 15);
    reject(|spec| spec.precision = 15);
    reject(|spec| spec.arrangement_a = 2);
    reject(|spec| spec.arrangement_b = 2);
    reject(|spec| spec.fill_mode = 2);
    reject(|spec| spec.side = 2);
    reject(|spec| spec.diag = 2);
}

#[test]
fn rejects_invalid_symbols() {
    for symbol in ["", "a b", "a/b", "a\0b"] {
        let mut spec = SolverSpec::potrf_7d_sm89();
        spec.symbol = symbol;
        assert!(spec.validate().is_err(), "应拒绝符号{symbol:?}");
    }
    let long = "a".repeat(129);
    let mut spec = SolverSpec::potrf_7d_sm89();
    spec.symbol = &long;
    assert!(spec.validate().is_err());
}

#[test]
fn relative_path_is_rejected() {
    let error = MathDx::load_from_dir(Path::new("relative-not-absolute")).unwrap_err();
    assert!(
        error.to_string().contains("绝对路径"),
        "错误面不符：{error}"
    );
}

#[test]
fn missing_library_is_rejected() {
    let directory = std::env::temp_dir().join("mjwarp-mathdx-missing-directory");
    let error = MathDx::load_from_dir(&directory).unwrap_err();
    assert!(error.to_string().contains("库缺失"), "错误面不符：{error}");
}

#[cfg(windows)]
#[test]
fn missing_symbol_is_rejected() {
    let system_root = std::env::var_os("SystemRoot").expect("SystemRoot");
    let kernel32 = Path::new(&system_root)
        .join("System32")
        .join("kernel32.dll");
    let error = MathDx::load_file(&kernel32).unwrap_err();
    assert!(
        error.to_string().contains("符号缺失"),
        "错误面不符：{error}"
    );
}

#[test]
fn status_text_covers_known_codes() {
    assert_eq!(super::status_text(0), "成功");
    for code in 1..=4 {
        assert_ne!(super::status_text(code), "未知状态");
    }
    assert_eq!(super::status_text(99), "未知状态");
}
