//! G10@P5：产品 NVRTC 入口的纯 CPU 单测（**不创建 `CudaContext`、不跑 GPU**）。
//!
//! 覆盖：接触输出布局、设备源拼装（顺序/去本地 include/指纹）、产品入口唯一性、
//! 错误注入 seam（编译/设备构造失败都不 panic）。输入校验单测在 `validate_tests.rs`，
//! `CollisionArgs`/sidecar 的 ABI 偏移在 `abi_tests.rs`。

use std::error::Error;
use std::mem::{align_of, offset_of, size_of};

use cudarc::nvrtc::Ptx;

use super::abi::CollisionContactOut;
use super::error::CollisionGpuError;
use super::source::{
    DEVICE_SOURCES, PRODUCT_SOURCE, assemble_device_source, is_local_cu_include, source_sha256,
    strip_local_includes,
};
use super::{compile_options, compile_source_with, program_from_parts};

macro_rules! assert_offsets {
    ($structure:ty, $(($field:ident, $offset:expr)),* $(,)?) => {
        $(
            assert_eq!(
                offset_of!($structure, $field),
                $offset,
                "{}::{} 偏移不符",
                stringify!($structure),
                stringify!($field)
            );
        )*
    };
}

#[cfg(target_pointer_width = "64")]
#[test]
fn abi_contact_out_matches_contact_cu_layout() {
    assert_eq!(align_of::<CollisionContactOut>(), 8);
    assert_eq!(size_of::<CollisionContactOut>(), 136);
    assert_offsets!(
        CollisionContactOut,
        (dist, 0),
        (pos, 8),
        (frame, 16),
        (includemargin, 24),
        (friction, 32),
        (solref, 40),
        (solreffriction, 48),
        (solimp, 56),
        (dim, 64),
        (geom, 72),
        (efc_address, 80),
        (worldid, 88),
        (contact_type, 96),
        (geomcollisionid, 104),
        (adhesion, 112),
        (nacon, 120),
        (efc_address_width, 128),
    );
}

#[test]
fn assembles_all_device_sources_in_dependency_order() {
    let assembled = assemble_device_source();
    let mut previous = 0usize;
    for (name, text) in DEVICE_SOURCES {
        let stripped = strip_local_includes(text);
        let position = assembled
            .find(&stripped)
            .unwrap_or_else(|| panic!("设备源 {name} 未出现在拼装结果中"));
        assert!(position >= previous, "设备源 {name} 的拼接顺序不符依赖顺序");
        previous = position;
    }
    assert!(
        assembled.ends_with(PRODUCT_SOURCE),
        "product.cu 必须追加在合并源最后"
    );
}

#[test]
fn strips_only_local_cu_includes() {
    assert!(is_local_cu_include("#include \"hfield.cu\"\n"));
    assert!(is_local_cu_include("#include\"common.cu\""));
    assert!(!is_local_cu_include("#include <cuda_runtime.h>\n"));
    assert!(!is_local_cu_include("#include \"some_header.h\"\n"));
    assert!(!is_local_cu_include("// #include \"contact.cu\"\n"));
    for (name, text) in DEVICE_SOURCES {
        let stripped = strip_local_includes(text);
        assert!(
            !stripped
                .lines()
                .any(|line| line.contains("#include \"") && line.contains(".cu\"")),
            "{name} 仍残留本地 .cu include"
        );
        assert_eq!(
            stripped.contains("#pragma once"),
            text.contains("#pragma once"),
            "{name} 的 #pragma once 不应被删除"
        );
    }
    let assembled = assemble_device_source();
    assert!(
        !assembled
            .lines()
            .any(|line| line.contains("#include \"") && line.contains(".cu\"")),
        "拼装结果仍残留本地 .cu include"
    );
}

#[test]
fn assembly_is_deterministic_and_hashed() {
    let first = assemble_device_source();
    let second = assemble_device_source();
    assert_eq!(first, second, "拼装结果必须确定");
    let sha = source_sha256();
    assert_eq!(sha, source_sha256(), "sha256 必须确定");
    assert_eq!(sha.len(), 64, "sha256 须为 64 位小写十六进制");
    assert!(sha.bytes().all(|byte| byte.is_ascii_hexdigit()));
    println!(
        "device_source_len={} device_source_sha256={sha}",
        first.len()
    );
}

#[test]
fn product_source_exposes_three_extern_c_entries() {
    let product = PRODUCT_SOURCE;
    for entry in [super::CONTACT_ENTRY, super::HFIELD_ENTRY, super::SDF_ENTRY] {
        let declaration = format!("extern \"C\" __global__ void {entry}(");
        assert_eq!(
            product.matches(&declaration).count(),
            1,
            "{entry} 须有且仅有一处 extern \"C\" 定义"
        );
    }
    assert_eq!(super::CONTACT_ENTRY, "mjwarp_collision_contact");
    assert_eq!(super::HFIELD_ENTRY, "mjwarp_collision_hfield");
    assert_eq!(super::SDF_ENTRY, "mjwarp_collision_sdf");
    for host_symbol in ["int main(", "load_bin", "CaseMeta", "DeviceArrays"] {
        assert!(
            !product.contains(host_symbol),
            "product.cu 不应含探针 host 代码：{host_symbol}"
        );
    }
    assert_eq!(product.matches("struct mjwarp_collision_args").count(), 1);
    assert_eq!(product.matches("struct ProbeArgs").count(), 0);
    for old_entry in ["kernel_contact", "kernel_hfield", "kernel_sdf"] {
        assert!(
            !product.contains(&format!("void {old_entry}(")),
            "product.cu 不应保留探针旧入口：{old_entry}"
        );
    }
}

#[test]
fn error_display_covers_every_variant() {
    let cases = [
        (
            CollisionGpuError::NoDevice("d".to_owned()),
            "CUDA设备不可用：d",
        ),
        (
            CollisionGpuError::Compile("c".to_owned()),
            "NVRTC编译失败：c",
        ),
        (
            CollisionGpuError::Load("l".to_owned()),
            "设备模块加载失败：l",
        ),
        (CollisionGpuError::Launch("r".to_owned()), "内核执行失败：r"),
        (
            CollisionGpuError::InvalidArgument("参数"),
            "输入参数无效：参数",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        let dynamic: Box<dyn Error> = Box::new(error);
        assert!(!dynamic.to_string().is_empty());
    }
}

#[test]
fn compile_options_match_g5_probe_recipe() {
    let options = compile_options();
    assert_eq!(options.fmad, Some(true));
    assert_eq!(options.use_fast_math, Some(false));
    assert_eq!(
        options.options,
        vec![
            "--std=c++17".to_owned(),
            "--gpu-architecture=sm_89".to_owned()
        ]
    );
    assert!(options.include_paths.is_empty());
    assert_eq!(options.name.as_deref(), Some("mjwarp_collision_product.cu"));
}

#[test]
fn compile_failures_are_mapped_without_panicking() {
    let result = compile_source_with(|text, options| {
        assert!(text.contains("mjwarp_collision_contact"));
        assert_eq!(options.fmad, Some(true));
        Err("nvrtc: boom".to_owned())
    });
    assert_eq!(
        result.err(),
        Some(CollisionGpuError::Compile("nvrtc: boom".to_owned()))
    );

    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let panicked = compile_source_with(|_, _| -> Result<Ptx, String> { panic!("nvrtc ffi") });
    std::panic::set_hook(previous);
    assert_eq!(
        panicked.err(),
        Some(CollisionGpuError::Compile("NVRTC编译异常".to_owned()))
    );
}

#[test]
fn constructor_seam_maps_device_and_compile_errors() {
    let no_device = program_from_parts(Err("no cuda driver".into()), Ok(Ptx::from_src("// ptx")));
    assert_eq!(
        no_device.err(),
        Some(CollisionGpuError::NoDevice("no cuda driver".to_owned()))
    );
    let compile_first = program_from_parts(
        Err("unused device error".to_owned()),
        Err(CollisionGpuError::Compile("boom".to_owned())),
    );
    assert_eq!(
        compile_first.err(),
        Some(CollisionGpuError::Compile("boom".to_owned()))
    );
}
