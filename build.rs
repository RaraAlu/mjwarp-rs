fn main() {
    println!("cargo:rerun-if-env-changed=MJWARP_MUJOCO_ROOT");
    #[cfg(feature = "native-model-probe")]
    build_native_probe();
}

#[cfg(feature = "native-model-probe")]
fn build_native_probe() {
    use std::{env, path::PathBuf};
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    assert!(
        (os == "windows" && target_env == "msvc") || (os == "linux" && target_env == "gnu"),
        "native-model-probe requires Windows MSVC or Linux GNU"
    );
    assert_eq!(
        env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
        "x86_64",
        "native-model-probe only supports x86_64"
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
    println!("cargo:rerun-if-changed=native/g01_fields.inc");
    println!("cargo:rerun-if-changed=include/mjwarp_native_probe.h");
    let mut build = cc::Build::new();
    build.cpp(true).std("c++17").warnings_into_errors(true);
    if os == "windows" {
        build.cpp_link_stdlib(None).static_crt(false);
    } else {
        println!("cargo:rustc-link-lib=dl");
    }
    build
        .include(include)
        .include("include")
        .file("native/model_probe.cpp")
        .compile("mjwarp_native_probe");
}
