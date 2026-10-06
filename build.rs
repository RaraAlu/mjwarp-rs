fn main() {
    println!("cargo:rerun-if-env-changed=MJWARP_MUJOCO_ROOT");
    #[cfg(feature = "native-model-probe")]
    build_native_probe();
}

#[cfg(feature = "native-model-probe")]
fn build_native_probe() {
    use std::{env, path::PathBuf};
    assert_eq!(
        env::var("CARGO_CFG_TARGET_OS").unwrap(),
        "windows",
        "native-model-probe only supports Windows"
    );
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
        "x86_64",
        "native-model-probe only supports x86_64"
    );
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ENV").unwrap(),
        "msvc",
        "native-model-probe currently requires the explicit Windows MSVC target"
    );
    let root = PathBuf::from(
        env::var_os("MJWARP_MUJOCO_ROOT")
            .expect("set MJWARP_MUJOCO_ROOT to the verified MuJoCo 3.12.0 package"),
    );
    let include = root.join("include");
    assert!(
        include.join("mujoco/mujoco.h").is_file(),
        "missing MuJoCo headers"
    );
    println!("cargo:rerun-if-changed={}", include.display());
    println!("cargo:rerun-if-changed=native/model_probe.cpp");
    println!("cargo:rerun-if-changed=include/mjwarp_native_probe.h");
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .cpp_link_stdlib(None)
        .static_crt(false)
        .warnings_into_errors(true)
        .include(include)
        .include("include")
        .file("native/model_probe.cpp")
        .compile("mjwarp_native_probe");
}
