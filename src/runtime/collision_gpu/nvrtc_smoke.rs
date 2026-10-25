//! CPU 侧 NVRTC 编译冒烟：只验证「产品设备源能被 NVRTC 编成含三个入口的 PTX」。
//!
//! **不创建 CUDA 上下文、不上传/发射/回读**，所以不需要 GPU；需要 NVRTC 运行库
//! （Windows：`target/toolchains/cuda-12.8.1/bin` 加入 `PATH` 后跑）。
//! 真正的三 case 对照在 `tests/g10_p5_nvrtc_launcher.rs`（ignored，需 GPU 窗口）。

use super::compile_program;

#[test]
#[ignore = "需要 NVRTC 运行库（CUDA toolkit bin 在 PATH）；不需要 GPU"]
fn nvrtc_compiles_product_device_source() {
    let ptx = compile_program().expect("NVRTC 编译产品设备源失败（错误串带 source_sha256）");
    let text = ptx.to_src();
    assert!(!text.is_empty(), "PTX 为空");
    for entry in [
        "mjwarp_collision_contact",
        "mjwarp_collision_hfield",
        "mjwarp_collision_sdf",
    ] {
        assert!(
            text.contains(entry),
            "PTX 缺少入口 {entry}（前 200 字节：{}）",
            &text[..text.len().min(200)]
        );
    }
    println!(
        "nvrtc_smoke_ok ptx_bytes={} entries=3 source_sha256={}",
        text.len(),
        super::source::source_sha256()
    );
}
