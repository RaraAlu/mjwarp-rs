//! 本地冻结安装与缓存拒绝测试。

use super::package_gate::*;
use super::support::*;
use super::*;

#[test]
#[ignore = "requires local frozen checkout, Warp 1.15 install and cache; no Python"]
pub(super) fn local_frozen_install_matches_record_and_rejects_smooth_cache() {
    let package = package().unwrap();
    validate_package(&package).unwrap();
    let head =
        fs::read_to_string(root().join("target/mjwarp-parallel/upstream/.git/HEAD")).unwrap();
    assert_eq!(head.trim(), REVISION);
    let source = read_stamped(
        package["source"]["path"].as_str().unwrap(),
        package["source"]["sha256"].as_str().unwrap(),
    )
    .unwrap();
    let source = String::from_utf8(source).unwrap();
    for anchor in [
        "def _gravity_force(",
        "worldid, bodyid, dofid = wp.tid()",
        "support.jac_dof(",
        "wp.atomic_add(qfrc_gravcomp_out[worldid], dofid, wp.dot(jac, force))",
    ] {
        assert!(source.contains(anchor));
    }
    let support = read_stamped(
        package["source_dependencies"][0]["path"].as_str().unwrap(),
        package["source_dependencies"][0]["sha256"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let support = String::from_utf8(support).unwrap();
    assert!(support.contains("def jac_dof("));
    assert!(support.contains("body_isdofancestor"));
    let metadata =
        fs::read_to_string(root().join(
            "target/mjwarp-parallel/venv/Lib/site-packages/warp_lang-1.15.0.dist-info/METADATA",
        ))
        .unwrap();
    assert!(metadata.lines().any(|line| line == "Name: warp-lang"));
    assert!(metadata.lines().any(|line| line == "Version: 1.15.0"));
    for stamp in package["warp_install"].as_array().unwrap() {
        read_stamped(
            stamp["path"].as_str().unwrap(),
            stamp["sha256"].as_str().unwrap(),
        )
        .unwrap();
    }
    let record =
        fs::read_to_string(root().join(
            "target/mjwarp-parallel/venv/Lib/site-packages/warp_lang-1.15.0.dist-info/RECORD",
        ))
        .unwrap();
    for row in [
        "warp/native/array.h,sha256=p7NuIXjNIDLYpdklCuoT44UrDnARJLhP-Ve2A8BpFmo,64821",
        "warp/native/builtin.h,sha256=memrF2mOh-yF10PUFiSfel8p9cuEGaKVVWA1C8Vku-Q,91555",
        "warp/_src/types.py,sha256=LqZJJffB5VHH0gtuRQkWQLVI1sDmuIJ1ufqGZXgEHvo,288528",
        "warp/_src/context.py,sha256=Q8hrZ-fJ75hsJgUJhTF-Bl6ZdZlAnp5Dn622ewem08Y,603204",
        "warp/_src/codegen.py,sha256=0Hp3jX91TddGINpcmywWG5j5YFvD3bAcWZB_d712oXA,288626",
        "warp/config.py,sha256=06MbAIKq23adrXRkgKupLmWLoaRdxaHoR5WZ6R8VPBg,18722",
        "warp/_src/build.py,sha256=M1T1DrcVTp4jOiuONdqMNRAoXiymFo0aM_8Re8jLd-o,24949",
        "warp/bin/warp.dll,sha256=fABus0inNZgQ9NABy9QUHMXW0WQfwpieTDcR6Z5mcdY,365392896",
        "warp/native/warp.cu,sha256=jCD_6mK1A61OVkNRmpEfOeqlwZSR5c21Ccl2ac_BGm0,207944",
    ] {
        assert!(record.lines().any(|actual| actual == row), "{row}");
    }
    let array = fs::read_to_string(
        root().join("target/mjwarp-parallel/venv/Lib/site-packages/warp/native/array.h"),
    )
    .unwrap();
    assert!(array.contains("uint16_t ndim;\n    uint16_t flags;"));
    let builtin = fs::read_to_string(
        root().join("target/mjwarp-parallel/venv/Lib/site-packages/warp/native/builtin.h"),
    )
    .unwrap();
    assert!(builtin.contains("template <int N> struct launch_bounds_t"));
    assert!(builtin.contains("int shape[N];\n    size_t size;\n    size_t coord_mult;"));
    let lock = fs::read_to_string(root().join("target/mjwarp-parallel/upstream/uv.lock"))
        .unwrap()
        .replace("\r\n", "\n");
    assert!(lock.contains("name = \"warp-lang\"\nversion = \"1.15.0\""));
    assert!(lock.contains(package["warp_windows_wheel_sha256"].as_str().unwrap()));

    let smooth = fs::read_to_string(root().join(
        "target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.cu",
    ))
    .unwrap();
    let entry = "_qfrc_bias_4085b6de_cuda_kernel_forward";
    let signature =
        signature_lines(&smooth, &format!("extern \"C\" __global__ void {entry}(")).unwrap();
    assert_eq!(
        compact(&signature),
        format!(
            "extern\"C\"__global__void{entry}(wp::launch_bounds_t<2>dim,wp::array_t<wp::int32>var_dof_bodyid,wp::array_t<wp::vec_t<6,wp::float32>>var_cdof_in,wp::array_t<wp::vec_t<6,wp::float32>>var_cfrc_int_in,wp::array_t<wp::float32>var_qfrc_bias_out)"
        )
    );
    let smooth_ptx = fs::read(
        root().join(
            "target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx",
        ),
    )
    .unwrap();
    let smooth_ptx = if smooth_ptx.last() == Some(&0) {
        &smooth_ptx[..smooth_ptx.len() - 1]
    } else {
        &smooth_ptx
    };
    let smooth_ptx = std::str::from_utf8(smooth_ptx).unwrap();
    assert!(
        smooth_ptx
            .lines()
            .any(|line| line.trim() == ".target sm_89")
    );
    assert!(
        smooth_ptx
            .lines()
            .any(|line| line.trim() == ".address_size 64")
    );
    let ptx_signature = signature_lines(smooth_ptx, &format!(".visible .entry {entry}(")).unwrap();
    let mut expected = vec![format!(".visible .entry {entry}(")];
    expected.push(format!(".param .align 8 .b8 {entry}_param_0[24],"));
    for index in 1..=4 {
        let comma = if index == 4 { "" } else { "," };
        expected.push(format!(
            ".param .align 8 .b8 {entry}_param_{index}[56]{comma}"
        ));
    }
    expected.push(")".into());
    assert!(
        ptx_signature
            .iter()
            .map(|line| line.trim())
            .eq(expected.iter().map(String::as_str))
    );

    let cache = root().join("target/mjwarp-parallel/warp-cache/1.15.0");
    let mut candidates = Vec::new();
    let mut stack = vec![cache];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("cu" | "ptx" | "meta" | "json")
            ) {
                candidates.push(path);
            }
        }
    }
    assert!(!candidates.is_empty());
    for path in candidates {
        let bytes = fs::read(&path).unwrap();
        assert!(
            !String::from_utf8_lossy(&bytes).contains("_gravity_force_"),
            "unregistered gravity artifact: {}",
            path.display()
        );
    }
}
