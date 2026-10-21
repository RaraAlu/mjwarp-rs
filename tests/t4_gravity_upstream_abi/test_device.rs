//! 本地设备启动确认Warp重力ABI测试。

use super::abi::*;
use super::package_gate::*;
use super::support::*;
use super::*;

#[cfg(windows)]
#[test]
#[ignore = "requires an SM89 GPU, NVIDIA driver, NVRTC and frozen Warp 1.15 headers"]
pub(super) fn local_device_launch_confirms_warp_gravity_abi() {
    use std::time::Instant;

    let package = package().unwrap();
    validate_package(&package).unwrap();
    let evidence_directory = std::env::var_os("MJWARP_GRAVITY_ABI_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(directory) = &evidence_directory {
        fs::create_dir_all(directory).unwrap();
        fs::write(directory.join("probe.cu"), DEVICE_PROBE_SOURCE).unwrap();
    }
    let native = root().join("target/mjwarp-parallel/venv/Lib/site-packages/warp/native");
    for index in 0..2 {
        let (path, hash) = stamp(&package, "warp_install", index).unwrap();
        read_stamped(path, hash).unwrap();
    }
    let context = CudaContext::new(0).unwrap();
    context.bind_to_thread().unwrap();
    let major = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)
        .unwrap();
    let minor = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)
        .unwrap();
    assert_eq!((major, minor), (8, 9));
    let mut driver_version = 0;
    // SAFETY: 驱动只写一个有效i32。
    unsafe { driver_sys::cuDriverGetVersion(&mut driver_version) }
        .result()
        .unwrap();
    let (mut nvrtc_major, mut nvrtc_minor) = (0, 0);
    // SAFETY: 编译器只写两个有效i32。
    unsafe { nvrtc_sys::nvrtcVersion(&mut nvrtc_major, &mut nvrtc_minor) }
        .result()
        .unwrap();
    let raw_options = vec![
        "--std=c++17".into(),
        "--define-macro=NDEBUG".into(),
        "--undefine-macro=WP_VERIFY_FP".into(),
        "--define-macro=WP_ENABLE_MATHDX=0".into(),
        "--device-as-default-execution-space".into(),
        "--extra-device-vectorization".into(),
        "--restrict".into(),
        "--diag-suppress=177,550".into(),
    ];
    let options = CompileOptions {
        fmad: Some(true),
        use_fast_math: Some(false),
        include_paths: vec![native.to_string_lossy().into_owned()],
        arch: Some("compute_89"),
        options: raw_options.clone(),
        name: Some("warp_gravity_abi.cu".into()),
        ..Default::default()
    };
    if let Some(directory) = &evidence_directory {
        let config = serde_json::json!({
            "schema": 1,
            "evidence_kind": "local-nvrtc-nonphysical-abi-launch",
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "device_compute_capability": [major, minor],
            "driver_version": driver_version,
            "nvrtc_version": [nvrtc_major, nvrtc_minor],
            "compile_options": {
                "arch": "compute_89",
                "fmad": true,
                "fast_math": false,
                "include_path": native,
                "options": raw_options,
            },
            "source_sha256": digest(DEVICE_PROBE_SOURCE.as_bytes()),
            "array_header_sha256": package["warp_install"][0]["sha256"],
            "builtin_header_sha256": package["warp_install"][1]["sha256"],
            "descriptor_only": true,
            "frozen_physical_input_uploaded": false,
            "source_transcribed_gravity_bridge": true,
            "upstream_generated_gravity_kernel": false,
            "equivalence_eligible": false
        });
        fs::write(
            directory.join("compile-options.json"),
            serde_json::to_vec_pretty(&config).unwrap(),
        )
        .unwrap();
    }
    let compile_start = Instant::now();
    let ptx = match compile_ptx_with_opts(DEVICE_PROBE_SOURCE, options) {
        Ok(ptx) => ptx,
        Err(error) => {
            if let Some(directory) = &evidence_directory {
                fs::write(directory.join("compile-error.log"), format!("{error:?}\n")).unwrap();
            }
            panic!("Warp ABI probe compilation failed: {error:?}");
        }
    };
    let compile_millis = compile_start.elapsed().as_millis();
    let ptx_source = ptx.to_src();
    if let Some(directory) = &evidence_directory {
        fs::write(directory.join("probe.ptx"), &ptx_source).unwrap();
    }
    assert!(
        ptx_source
            .lines()
            .any(|line| line.trim() == ".target sm_89")
    );
    assert!(
        ptx_source
            .lines()
            .any(|line| line.trim() == ".address_size 64")
    );
    let ptx_signature = signature_lines(
        &ptx_source,
        &format!(".visible .entry {DEVICE_PROBE_ENTRY}("),
    )
    .unwrap();
    assert!(
        ptx_signature
            .iter()
            .map(|line| line.trim())
            .eq(expected_ptx_signature(DEVICE_PROBE_ENTRY)
                .iter()
                .map(String::as_str))
    );

    let module = context.load_module(ptx).unwrap();
    let function = module.load_function(DEVICE_PROBE_ENTRY).unwrap();
    let stream = context.default_stream();
    let buffers = (0..10)
        .map(|_| stream.alloc_zeros::<u8>(64).unwrap())
        .collect::<Vec<_>>();
    let mut output = stream.alloc_zeros::<u64>(84).unwrap();
    let pointers = buffers
        .iter()
        .map(|buffer| buffer.device_ptr(&stream))
        .collect::<Vec<_>>();
    let (output_pointer, output_record) = output.device_ptr_mut(&stream);
    let specs: [(&[usize], usize); 11] = [
        (&[2], 12),
        (&[5], 4),
        (&[5], 4),
        (&[3, 5], 4),
        (&[5, 5], 4),
        (&[4], 4),
        (&[5, 4], 4),
        (&[7, 5], 12),
        (&[7, 5], 12),
        (&[7, 4], 24),
        (&[7, 4], 4),
    ];
    let mut descriptors = Vec::with_capacity(11);
    for (index, (shape, bytes)) in specs.into_iter().enumerate() {
        let pointer = if index == 10 {
            output_pointer
        } else {
            pointers[index].0
        };
        descriptors.push(contiguous(pointer, shape, bytes).unwrap());
    }
    let bounds = WarpBounds3 {
        shape: [7, 4, 4],
        padding: 0,
        size: 112,
        coord_mult: 1,
    };
    let launch_start = Instant::now();
    let mut launch = stream.launch_builder(&function);
    launch.arg(&bounds);
    for descriptor in &descriptors {
        launch.arg(descriptor);
    }
    // SAFETY: 单线程只写84个u64。
    // 十一个描述符保持设备分配存活。
    unsafe {
        launch
            .launch(LaunchConfig {
                grid_dim: (1, 1, 1),
                block_dim: (1, 1, 1),
                shared_mem_bytes: 0,
            })
            .unwrap();
    }
    drop(output_record);
    drop(pointers);
    let actual = stream.clone_dtoh(&output).unwrap();
    let launch_micros = launch_start.elapsed().as_micros();

    let mut expected = vec![
        size_of::<WarpArray>() as u64,
        std::mem::align_of::<WarpArray>() as u64,
        std::mem::offset_of!(WarpArray, data) as u64,
        std::mem::offset_of!(WarpArray, grad) as u64,
        std::mem::offset_of!(WarpArray, shape) as u64,
        std::mem::offset_of!(WarpArray, strides) as u64,
        std::mem::offset_of!(WarpArray, ndim) as u64,
        std::mem::offset_of!(WarpArray, flags) as u64,
        size_of::<WarpBounds3>() as u64,
        std::mem::align_of::<WarpBounds3>() as u64,
        std::mem::offset_of!(WarpBounds3, shape) as u64,
        std::mem::offset_of!(WarpBounds3, size) as u64,
        std::mem::offset_of!(WarpBounds3, coord_mult) as u64,
        7,
        4,
        4,
        112,
        1,
    ];
    for descriptor in &descriptors {
        expected.extend([
            descriptor.data,
            descriptor.shape[0] as u64,
            descriptor.shape[1] as u64,
            descriptor.strides[0] as u64,
            descriptor.strides[1] as u64,
            u64::from(descriptor.ndim),
        ]);
    }
    if let Some(directory) = &evidence_directory {
        let report = serde_json::json!({
            "schema": 1,
            "evidence_kind": "local-nvrtc-nonphysical-abi-launch",
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "device_compute_capability": [major, minor],
            "driver_version": driver_version,
            "nvrtc_version": [nvrtc_major, nvrtc_minor],
            "compile_options": {
                "std": "c++17",
                "target_arch": "compute_89",
                "fmad": true,
                "fast_math": false,
                "defines": ["WP_NO_BFLOAT16", "WP_TILE_BLOCK_DIM=256", "WP_NO_CRT"],
                "include": "frozen-warp-1.15-native"
            },
            "launch": {"grid": [1, 1, 1], "block": [1, 1, 1], "shared_memory": 0},
            "source_sha256": digest(DEVICE_PROBE_SOURCE.as_bytes()),
            "ptx_sha256": digest(ptx_source.as_bytes()),
            "array_size": actual[0],
            "array_alignment": actual[1],
            "bounds3_size": actual[8],
            "bounds3_alignment": actual[9],
            "launch_shape": actual[13..16],
            "launch_size": actual[16],
            "launch_coord_mult": actual[17],
            "descriptor_records": actual[18..],
            "rust_device_match": actual == expected,
            "actual": &actual,
            "expected": &expected,
            "compile_millis": compile_millis,
            "launch_and_readback_micros": launch_micros,
            "descriptor_only": true,
            "frozen_physical_input_uploaded": false,
            "physical_gravity_kernel": false,
            "source_transcribed_gravity_bridge": false,
            "upstream_generated_gravity_kernel": false,
            "equivalence_eligible": false
        });
        fs::write(
            directory.join("run.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(actual, expected);

    let gravity_signature = signature_lines(
        &ptx_source,
        &format!(".visible .entry {DEVICE_GRAVITY_ENTRY}("),
    )
    .unwrap();
    assert!(
        gravity_signature
            .iter()
            .map(|line| line.trim())
            .eq(expected_ptx_signature(DEVICE_GRAVITY_ENTRY)
                .iter()
                .map(String::as_str))
    );
    let model = &package["model"];
    let int_values = |field: &str| {
        model[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
            .collect::<Vec<_>>()
    };
    let gravity = f32_values(&model["opt_gravity"], "opt_gravity").unwrap();
    let parent = int_values("body_parentid");
    let roots = int_values("body_rootid");
    let mass = f32_values(&model["body_mass"], "body_mass").unwrap();
    let gravcomp = f32_values(&model["body_gravcomp"], "body_gravcomp").unwrap();
    let dof_body = int_values("dof_bodyid");
    let ancestors = int_values("body_isdofancestor");
    let xipos = f32_values(&package["state"]["xipos_row"], "xipos row")
        .unwrap()
        .repeat(7);
    let (candidate_path, candidate_hash) = stamp(&package, "input_sources", 0).unwrap();
    let candidate: Value =
        serde_json::from_slice(&read_stamped(candidate_path, candidate_hash).unwrap()).unwrap();
    let mut subtree_com = Vec::with_capacity(7 * 5 * 3);
    let mut cdof = Vec::with_capacity(7 * 4 * 6);
    for row in candidate["candidate_com_diagnostics"].as_array().unwrap() {
        subtree_com.extend(f32_values(&row["subtree_com"], "subtree com").unwrap());
        cdof.extend(f32_values(&row["cdof"], "cdof").unwrap());
    }
    let (contract_path, contract_hash) = stamp(&package, "input_sources", 1).unwrap();
    let contract: Value =
        serde_json::from_slice(&read_stamped(contract_path, contract_hash).unwrap()).unwrap();
    let expected_output = contract["expected"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| f32_values(row, "expected").unwrap())
        .collect::<Vec<_>>();

    let gravity_device = stream.clone_htod(&gravity).unwrap();
    let parent_device = stream.clone_htod(&parent).unwrap();
    let roots_device = stream.clone_htod(&roots).unwrap();
    let mass_device = stream.clone_htod(&mass).unwrap();
    let gravcomp_device = stream.clone_htod(&gravcomp).unwrap();
    let dof_device = stream.clone_htod(&dof_body).unwrap();
    let ancestor_device = stream.clone_htod(&ancestors).unwrap();
    let xipos_device = stream.clone_htod(&xipos).unwrap();
    let subtree_device = stream.clone_htod(&subtree_com).unwrap();
    let cdof_device = stream.clone_htod(&cdof).unwrap();
    let mut gravity_output = stream.alloc_zeros::<f32>(7 * 4).unwrap();
    let (gravity_pointer, gravity_record) = gravity_device.device_ptr(&stream);
    let (parent_pointer, parent_record) = parent_device.device_ptr(&stream);
    let (roots_pointer, roots_record) = roots_device.device_ptr(&stream);
    let (mass_pointer, mass_record) = mass_device.device_ptr(&stream);
    let (gravcomp_pointer, gravcomp_record) = gravcomp_device.device_ptr(&stream);
    let (dof_pointer, dof_record) = dof_device.device_ptr(&stream);
    let (ancestor_pointer, ancestor_record) = ancestor_device.device_ptr(&stream);
    let (xipos_pointer, xipos_record) = xipos_device.device_ptr(&stream);
    let (subtree_pointer, subtree_record) = subtree_device.device_ptr(&stream);
    let (cdof_pointer, cdof_record) = cdof_device.device_ptr(&stream);
    let (gravity_output_pointer, gravity_output_record) = gravity_output.device_ptr_mut(&stream);
    let physical_descriptors = [
        contiguous(gravity_pointer, &[2], 12).unwrap(),
        contiguous(parent_pointer, &[5], 4).unwrap(),
        contiguous(roots_pointer, &[5], 4).unwrap(),
        contiguous(mass_pointer, &[3, 5], 4).unwrap(),
        contiguous(gravcomp_pointer, &[5, 5], 4).unwrap(),
        contiguous(dof_pointer, &[4], 4).unwrap(),
        contiguous(ancestor_pointer, &[5, 4], 4).unwrap(),
        contiguous(xipos_pointer, &[7, 5], 12).unwrap(),
        contiguous(subtree_pointer, &[7, 5], 12).unwrap(),
        contiguous(cdof_pointer, &[7, 4], 24).unwrap(),
        contiguous(gravity_output_pointer, &[7, 4], 4).unwrap(),
    ];
    let gravity_function = module.load_function(DEVICE_GRAVITY_ENTRY).unwrap();
    let physical_start = Instant::now();
    let mut physical_launch = stream.launch_builder(&gravity_function);
    physical_launch.arg(&bounds);
    for descriptor in &physical_descriptors {
        physical_launch.arg(descriptor);
    }
    // SAFETY: 描述符覆盖全部冻结输入。
    // 一百一十二线程只写二十八槽。
    unsafe {
        physical_launch
            .launch(LaunchConfig {
                grid_dim: (1, 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
            .unwrap();
    }
    drop(gravity_output_record);
    drop(cdof_record);
    drop(subtree_record);
    drop(xipos_record);
    drop(ancestor_record);
    drop(dof_record);
    drop(gravcomp_record);
    drop(mass_record);
    drop(roots_record);
    drop(parent_record);
    drop(gravity_record);
    let physical_output = stream.clone_dtoh(&gravity_output).unwrap();
    let physical_micros = physical_start.elapsed().as_micros();
    let physical_bits = physical_output
        .iter()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect::<Vec<_>>();
    if let Some(directory) = &evidence_directory {
        let raw = serde_json::json!({
            "schema": 1,
            "provenance": "direct_device_readback_before_finite_validation",
            "upstream_revision": REVISION,
            "package_sha256": PACKAGE_SHA256,
            "source_sha256": digest(DEVICE_PROBE_SOURCE.as_bytes()),
            "ptx_sha256": digest(ptx_source.as_bytes()),
            "actual_bits": &physical_bits,
            "count": physical_bits.len()
        });
        fs::write(
            directory.join("physical-raw.json"),
            serde_json::to_vec_pretty(&raw).unwrap(),
        )
        .unwrap();
    }
    let max_absolute = max_absolute_finite(&physical_output, &expected_output).unwrap();
    let bit_differences = physical_output
        .iter()
        .zip(&expected_output)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count();
    if let Some(directory) = &evidence_directory {
        let report = serde_json::json!({
            "schema": 1,
            "evidence_kind": "local-nvrtc-source-transcribed-gravity-launch",
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "entry": DEVICE_GRAVITY_ENTRY,
            "source_sha256": digest(DEVICE_PROBE_SOURCE.as_bytes()),
            "ptx_sha256": digest(ptx_source.as_bytes()),
            "package_sha256": PACKAGE_SHA256,
            "launch": {"shape": [7, 4, 4], "grid": [1, 1, 1], "block": [256, 1, 1]},
            "input_elements": {
                "opt_gravity_f32": gravity.len(),
                "body_parentid_i32": parent.len(),
                "body_rootid_i32": roots.len(),
                "body_mass_f32": mass.len(),
                "body_gravcomp_f32": gravcomp.len(),
                "dof_bodyid_i32": dof_body.len(),
                "body_isdofancestor_i32": ancestors.len(),
                "xipos_f32": xipos.len(),
                "subtree_com_f32": subtree_com.len(),
                "cdof_f32": cdof.len(),
                "qfrc_gravcomp_f32": physical_output.len()
            },
            "actual": &physical_output,
            "actual_bits": &physical_bits,
            "expected": &expected_output,
            "expected_kind": "analytic_certificate_not_upstream_device_output",
            "tolerance": 0.000002,
            "max_absolute": max_absolute,
            "bit_differences": bit_differences,
            "launch_and_readback_micros": physical_micros,
            "descriptor_only": false,
            "frozen_physical_input_uploaded": true,
            "physical_gravity_kernel": true,
            "source_transcribed_gravity_bridge": true,
            "upstream_generated_gravity_kernel": false,
            "candidate_derived_inputs": true,
            "full_chain_independent": false,
            "equivalence_eligible": false
        });
        fs::write(
            directory.join("physical-run.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(physical_output.len(), expected_output.len());
    assert!(max_absolute <= 0.000002);
}
