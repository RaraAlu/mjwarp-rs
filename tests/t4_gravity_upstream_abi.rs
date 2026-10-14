//! 冻结重力核的Warp原生ABI与输入门禁。
//! 本目标不生成上游产物，也不启动Python。
#![cfg(feature = "cuda-probe")]

use cudarc::driver::DeviceRepr;
#[cfg(windows)]
use cudarc::{
    driver::{
        CudaContext, DevicePtr, DevicePtrMut, LaunchConfig, PushKernelArg, sys as driver_sys,
    },
    nvrtc::{CompileOptions, compile_ptx_with_opts, sys as nvrtc_sys},
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, mem::size_of, path::Path, path::PathBuf};

const PACKAGE: &str = include_str!("../fixtures/gravity-upstream-abi/package.json");
const PACKAGE_SHA256: &str = "4fafa7e6b76cc6ac77b81983b6a1bd2b2b2f92f7ba6622921d673a6f8b6acbc4";
const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
const WARP_VERSION: &str = "1.15.0";
#[cfg(windows)]
const DEVICE_PROBE_SOURCE: &str = include_str!("native/warp_gravity_abi.cu");
#[cfg(windows)]
const DEVICE_PROBE_ENTRY: &str = "_gravity_force_abi_probe_cuda_kernel_forward";
#[cfg(windows)]
const DEVICE_GRAVITY_ENTRY: &str = "_gravity_force_bridge_cuda_kernel_forward";

// Warp 1.15 array_t按值传参。
// 显式尾填充固定全部参数字节。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WarpArray {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
}

// launch_bounds_t<3>含四字节对齐槽。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WarpBounds3 {
    shape: [i32; 3],
    padding: u32,
    size: u64,
    coord_mult: u64,
}

// SAFETY: 两个类型只含固定宽度值。
// repr(C)和原生探针绑定冻结头文件。
unsafe impl DeviceRepr for WarpArray {}
// SAFETY: 该布局逐字段匹配launch_bounds_t<3>。
unsafe impl DeviceRepr for WarpBounds3 {}

#[derive(Debug, PartialEq, Eq)]
enum GateError {
    Missing(&'static str),
    Io(&'static str),
    Parse(&'static str),
    Mismatch(&'static str),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactReceipt {
    schema: u32,
    evidence_kind: String,
    upstream_revision: String,
    warp_version: String,
    module: String,
    passive_source_sha256: String,
    support_source_sha256: String,
    codegen_source_sha256: String,
    target_arch: u32,
    mode: String,
    fast_math: bool,
    fuse_fp: bool,
    entry: String,
    launch_rank: usize,
    array_arguments: usize,
    cu_file: String,
    ptx_file: String,
    cu_sha256: String,
    ptx_sha256: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ArtifactIdentity {
    evidence_kind: String,
    entry: String,
    cu_sha256: String,
    ptx_sha256: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn package() -> Result<Value, GateError> {
    if digest(PACKAGE.as_bytes()) != PACKAGE_SHA256 {
        return Err(GateError::Mismatch("package hash"));
    }
    serde_json::from_str(PACKAGE).map_err(|_| GateError::Parse("package"))
}

fn read(path: &Path, what: &'static str) -> Result<Vec<u8>, GateError> {
    fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            GateError::Missing(what)
        } else {
            GateError::Io(what)
        }
    })
}

fn read_stamped(path: &str, expected: &str) -> Result<Vec<u8>, GateError> {
    let bytes = read(&root().join(path), "stamped source")?;
    if digest(&bytes) != expected {
        return Err(GateError::Mismatch("source hash"));
    }
    Ok(bytes)
}

fn usize_value(value: &Value, what: &'static str) -> Result<usize, GateError> {
    let value = value.as_u64().ok_or(GateError::Parse(what))?;
    usize::try_from(value).map_err(|_| GateError::Mismatch(what))
}

fn f32_values(value: &Value, what: &'static str) -> Result<Vec<f32>, GateError> {
    value
        .as_array()
        .ok_or(GateError::Parse(what))?
        .iter()
        .map(|value| {
            let original = value.as_f64().ok_or(GateError::Parse(what))?;
            let narrowed = original as f32;
            if !narrowed.is_finite() || f64::from(narrowed) != original {
                return Err(GateError::Mismatch(what));
            }
            Ok(narrowed)
        })
        .collect()
}

fn max_absolute_finite(actual: &[f32], expected: &[f32]) -> Result<f32, GateError> {
    if actual.len() != expected.len() {
        return Err(GateError::Mismatch("output length"));
    }
    let mut maximum = 0.0_f32;
    for (&actual, &expected) in actual.iter().zip(expected) {
        if !actual.is_finite() || !expected.is_finite() {
            return Err(GateError::Mismatch("non-finite output"));
        }
        maximum = maximum.max((actual - expected).abs());
    }
    Ok(maximum)
}

fn contiguous(data: u64, shape: &[usize], element_bytes: usize) -> Result<WarpArray, GateError> {
    if data == 0 || !(1..=4).contains(&shape.len()) || element_bytes == 0 {
        return Err(GateError::Mismatch("array dimensions"));
    }
    if shape.contains(&0) {
        return Err(GateError::Mismatch("zero array dimension"));
    }
    let mut array = WarpArray {
        data,
        grad: 0,
        shape: [0; 4],
        strides: [0; 4],
        ndim: shape.len() as u16,
        flags: 0,
        padding: 0,
    };
    let mut stride = element_bytes;
    for axis in (0..shape.len()).rev() {
        array.shape[axis] =
            i32::try_from(shape[axis]).map_err(|_| GateError::Mismatch("shape overflow"))?;
        array.strides[axis] =
            i32::try_from(stride).map_err(|_| GateError::Mismatch("stride overflow"))?;
        stride = stride
            .checked_mul(shape[axis])
            .ok_or(GateError::Mismatch("stride overflow"))?;
    }
    Ok(array)
}

fn stamp<'a>(
    package: &'a Value,
    group: &str,
    index: usize,
) -> Result<(&'a str, &'a str), GateError> {
    let stamp = package[group]
        .as_array()
        .and_then(|values| values.get(index))
        .ok_or(GateError::Parse("stamp"))?;
    Ok((
        stamp["path"]
            .as_str()
            .ok_or(GateError::Parse("stamp path"))?,
        stamp["sha256"]
            .as_str()
            .ok_or(GateError::Parse("stamp hash"))?,
    ))
}

fn validate_package(package: &Value) -> Result<(), GateError> {
    if package["schema"] != 1
        || package["upstream_revision"] != REVISION
        || package["warp_version"] != WARP_VERSION
        || package["source"]["symbol"] != "_gravity_force"
    {
        return Err(GateError::Mismatch("frozen identity"));
    }
    let dependencies = package["source_dependencies"]
        .as_array()
        .ok_or(GateError::Parse("source dependencies"))?;
    if dependencies.len() != 1 || dependencies[0]["symbols"] != serde_json::json!(["jac_dof"]) {
        return Err(GateError::Mismatch("support source"));
    }
    let dimensions = &package["dimensions"];
    let worlds = usize_value(&dimensions["worlds"], "worlds")?;
    let nbody = usize_value(&dimensions["nbody"], "nbody")?;
    let nv = usize_value(&dimensions["nv"], "nv")?;
    if (worlds, nbody, nv) != (7, 5, 4)
        || dimensions["launch_shape"] != serde_json::json!([7, 4, 4])
    {
        return Err(GateError::Mismatch("dimensions"));
    }
    let model = &package["model"];
    for (field, length) in [
        ("opt_gravity", 2 * 3),
        ("body_parentid", nbody),
        ("body_rootid", nbody),
        ("body_mass", 3 * nbody),
        ("body_gravcomp", 5 * nbody),
        ("dof_bodyid", nv),
        ("body_isdofancestor", nbody * nv),
    ] {
        if model[field].as_array().map(Vec::len) != Some(length) {
            return Err(GateError::Mismatch("model field"));
        }
    }
    f32_values(&model["opt_gravity"], "opt_gravity")?;
    f32_values(&model["body_mass"], "body_mass")?;
    f32_values(&model["body_gravcomp"], "body_gravcomp")?;
    let state = &package["state"];
    if state["candidate_derived_inputs"] != true
        || state["full_chain_independent"] != false
        || state["xipos_broadcast_rows"] != worlds
        || f32_values(&state["xipos_row"], "xipos row")?.len() != 3 * nbody
    {
        return Err(GateError::Mismatch("state boundary"));
    }
    for index in 0..2 {
        let (path, hash) = stamp(package, "input_sources", index)?;
        read_stamped(path, hash)?;
    }
    let (report_path, report_hash) = stamp(package, "input_sources", 0)?;
    let report: Value = serde_json::from_slice(&read_stamped(report_path, report_hash)?)
        .map_err(|_| GateError::Parse("candidate report"))?;
    let rows = report["candidate_com_diagnostics"]
        .as_array()
        .ok_or(GateError::Parse("candidate rows"))?;
    if rows.len() != worlds {
        return Err(GateError::Mismatch("candidate worlds"));
    }
    for (world, row) in rows.iter().enumerate() {
        if row["world"] != world
            || f32_values(&row["subtree_com"], "subtree com")?.len() != 3 * nbody
            || f32_values(&row["cdof"], "cdof")?.len() != 6 * nv
        {
            return Err(GateError::Mismatch("candidate row"));
        }
    }
    let (expected_path, expected_hash) = stamp(package, "input_sources", 1)?;
    if package["expected_path"] != expected_path {
        return Err(GateError::Mismatch("expected path"));
    }
    let contract: Value = serde_json::from_slice(&read_stamped(expected_path, expected_hash)?)
        .map_err(|_| GateError::Parse("analytic contract"))?;
    if contract["worlds"] != worlds
        || contract["expected"].as_array().map(Vec::len) != Some(worlds)
        || contract["boundaries"]["frozen_upstream_device_output"] != false
        || package["expected_kind"] != "analytic_certificate_not_upstream_device_output"
    {
        return Err(GateError::Mismatch("analytic boundary"));
    }
    let requirements = &package["artifact_requirements"];
    if requirements["launch_rank"] != 3
        || requirements["array_arguments"] != 11
        || requirements["receipt_fields"].as_array().map(Vec::len) != Some(19)
        || requirements["compile_options"]["requires_actual_receipt"] != true
    {
        return Err(GateError::Mismatch("artifact boundary"));
    }
    let pipeline = &package["generation_pipeline"];
    if package["warp_install"].as_array().map(Vec::len) != Some(11)
        || pipeline["python_codegen_entry"] != "warp._src.context.Module._run_codegen"
        || pipeline["native_compile_entry"] != "wp_cuda_compile_program"
        || pipeline["native_compile_binary_pinned"] != true
        || pipeline["non_python_codegen_export_present"] != false
        || pipeline["missing_artifact"] != "frozen passive module emitted CUDA"
        || pipeline["python_execution_allowed"] != false
    {
        return Err(GateError::Mismatch("generation pipeline"));
    }
    Ok(())
}

fn safe_file(directory: &Path, name: &str, what: &'static str) -> Result<Vec<u8>, GateError> {
    let path = Path::new(name);
    if path.file_name().and_then(|value| value.to_str()) != Some(name) {
        return Err(GateError::Mismatch("artifact filename"));
    }
    let bytes = read(&directory.join(path), what)?;
    if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
        return Err(GateError::Mismatch("artifact size"));
    }
    Ok(bytes)
}

fn strip_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut block = false;
    while index < bytes.len() {
        if block {
            if bytes[index..].starts_with(b"*/") {
                block = false;
                output.extend_from_slice(b"  ");
                index += 2;
            } else {
                output.push(if bytes[index] == b'\n' { b'\n' } else { b' ' });
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            block = true;
            output.extend_from_slice(b"  ");
            index += 2;
        } else if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                output.push(b' ');
                index += 1;
            }
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).unwrap()
}

fn signature_lines(source: &str, opening: &str) -> Result<Vec<String>, GateError> {
    let clean = strip_comments(source);
    let lines = clean.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.trim() == opening)
        .ok_or(GateError::Mismatch("entry missing"))?;
    let relative = lines[start..]
        .iter()
        .skip(1)
        .position(|line| line.trim_end().ends_with(')'))
        .map(|value| value + 1)
        .ok_or(GateError::Mismatch("entry signature"))?;
    Ok(lines[start..=start + relative]
        .iter()
        .map(|line| (*line).to_owned())
        .collect())
}

fn compact(lines: &[String]) -> String {
    lines
        .iter()
        .flat_map(|line| line.chars())
        .filter(|value| !value.is_ascii_whitespace())
        .collect()
}

fn expected_cuda_signature(entry: &str) -> String {
    let arrays = [
        "wp::array_t<wp::vec_t<3,wp::float32>>var_opt_gravity",
        "wp::array_t<wp::int32>var_body_parentid",
        "wp::array_t<wp::int32>var_body_rootid",
        "wp::array_t<wp::float32>var_body_mass",
        "wp::array_t<wp::float32>var_body_gravcomp",
        "wp::array_t<wp::int32>var_dof_bodyid",
        "wp::array_t<wp::int32>var_body_isdofancestor",
        "wp::array_t<wp::vec_t<3,wp::float32>>var_xipos_in",
        "wp::array_t<wp::vec_t<3,wp::float32>>var_subtree_com_in",
        "wp::array_t<wp::vec_t<6,wp::float32>>var_cdof_in",
        "wp::array_t<wp::float32>var_qfrc_gravcomp_out",
    ];
    format!(
        "extern\"C\"__global__void{entry}(wp::launch_bounds_t<3>dim,{})",
        arrays.join(",")
    )
}

fn expected_ptx_signature(entry: &str) -> Vec<String> {
    let mut lines = vec![format!(".visible .entry {entry}(")];
    lines.push(format!(".param .align 8 .b8 {entry}_param_0[32],"));
    for index in 1..=11 {
        let comma = if index == 11 { "" } else { "," };
        lines.push(format!(
            ".param .align 8 .b8 {entry}_param_{index}[56]{comma}"
        ));
    }
    lines.push(")".into());
    lines
}

fn validate_artifact_candidate(
    package: &Value,
    directory: &Path,
) -> Result<ArtifactIdentity, GateError> {
    validate_package(package)?;
    let receipt_bytes = read(&directory.join("receipt.json"), "artifact receipt")?;
    let receipt: ArtifactReceipt =
        serde_json::from_slice(&receipt_bytes).map_err(|_| GateError::Parse("artifact receipt"))?;
    let requirements = &package["artifact_requirements"];
    let options = &requirements["compile_options"];
    if receipt.schema != 1
        || receipt.upstream_revision != REVISION
        || receipt.warp_version != WARP_VERSION
        || receipt.module != requirements["module"]
        || receipt.passive_source_sha256 != package["source"]["sha256"]
        || receipt.support_source_sha256 != package["source_dependencies"][0]["sha256"]
        || receipt.codegen_source_sha256 != package["warp_install"][4]["sha256"]
        || receipt.target_arch != options["target_arch"]
        || receipt.mode != options["mode"]
        || receipt.fast_math != options["fast_math"]
        || receipt.fuse_fp != options["fuse_fp"]
        || receipt.launch_rank != 3
        || receipt.array_arguments != 11
    {
        return Err(GateError::Mismatch("artifact receipt identity"));
    }
    let prefix = requirements["kernel_prefix"]
        .as_str()
        .ok_or(GateError::Parse("kernel prefix"))?;
    let suffix = requirements["kernel_suffix"]
        .as_str()
        .ok_or(GateError::Parse("kernel suffix"))?;
    if !receipt.entry.starts_with(prefix) || !receipt.entry.ends_with(suffix) {
        return Err(GateError::Mismatch("artifact entry"));
    }
    let cu = safe_file(directory, &receipt.cu_file, "artifact cu")?;
    let ptx = safe_file(directory, &receipt.ptx_file, "artifact ptx")?;
    if digest(&cu) != receipt.cu_sha256 || digest(&ptx) != receipt.ptx_sha256 {
        return Err(GateError::Mismatch("artifact hash"));
    }
    let cu = std::str::from_utf8(&cu).map_err(|_| GateError::Parse("artifact cu"))?;
    let cu_signature = signature_lines(
        cu,
        &format!("extern \"C\" __global__ void {}(", receipt.entry),
    )?;
    if compact(&cu_signature) != expected_cuda_signature(&receipt.entry) {
        return Err(GateError::Mismatch("cuda abi"));
    }
    let ptx = if ptx.last() == Some(&0) {
        &ptx[..ptx.len() - 1]
    } else {
        &ptx
    };
    let ptx = std::str::from_utf8(ptx).map_err(|_| GateError::Parse("artifact ptx"))?;
    let target = format!(".target sm_{}", receipt.target_arch);
    if !ptx.lines().any(|line| line.trim() == target) {
        return Err(GateError::Mismatch("ptx target"));
    }
    if !ptx.lines().any(|line| line.trim() == ".address_size 64") {
        return Err(GateError::Mismatch("ptx address size"));
    }
    let ptx_signature = signature_lines(ptx, &format!(".visible .entry {}(", receipt.entry))?;
    let expected = expected_ptx_signature(&receipt.entry);
    if ptx_signature
        .iter()
        .map(|line| line.trim())
        .ne(expected.iter().map(String::as_str))
    {
        return Err(GateError::Mismatch("ptx abi"));
    }
    Ok(ArtifactIdentity {
        evidence_kind: receipt.evidence_kind,
        entry: receipt.entry,
        cu_sha256: receipt.cu_sha256,
        ptx_sha256: receipt.ptx_sha256,
    })
}

fn admit_artifact(package: &Value, directory: &Path) -> Result<ArtifactIdentity, GateError> {
    let identity = validate_artifact_candidate(package, directory)?;
    let requirements = &package["artifact_requirements"];
    if requirements["actual_artifact_present"] != true {
        return Err(GateError::Mismatch("artifact not registered"));
    }
    if requirements["equivalence_eligible"] != true {
        return Err(GateError::Mismatch("artifact not eligible"));
    }
    if identity.evidence_kind != "frozen-generated" {
        return Err(GateError::Mismatch("artifact evidence kind"));
    }
    Ok(identity)
}

struct Temp(PathBuf);

impl Temp {
    fn new(name: &str) -> Self {
        let path = root().join("target").join(format!(
            "t4-gravity-abi-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn receipt(package: &Value) -> Value {
    serde_json::json!({
        "schema": 1,
        "evidence_kind": "synthetic-non-physical",
        "upstream_revision": REVISION,
        "warp_version": WARP_VERSION,
        "module": "mujoco_warp._src.passive",
        "passive_source_sha256": package["source"]["sha256"],
        "support_source_sha256": package["source_dependencies"][0]["sha256"],
        "codegen_source_sha256": package["warp_install"][4]["sha256"],
        "target_arch": 89,
        "mode": "release",
        "fast_math": false,
        "fuse_fp": true,
        "entry": "_gravity_force_untrusted_cuda_kernel_forward",
        "launch_rank": 3,
        "array_arguments": 11,
        "cu_file": "module.cu",
        "ptx_file": "module.ptx",
        "cu_sha256": "",
        "ptx_sha256": "",
    })
}

fn synthetic_candidate(package: &Value) -> (Vec<u8>, Vec<u8>, Value) {
    let entry = "_gravity_force_untrusted_cuda_kernel_forward";
    let cu = format!(
        "extern \"C\" __global__ void {entry}(\n\
         wp::launch_bounds_t<3> dim,\n\
         wp::array_t<wp::vec_t<3, wp::float32>> var_opt_gravity,\n\
         wp::array_t<wp::int32> var_body_parentid,\n\
         wp::array_t<wp::int32> var_body_rootid,\n\
         wp::array_t<wp::float32> var_body_mass,\n\
         wp::array_t<wp::float32> var_body_gravcomp,\n\
         wp::array_t<wp::int32> var_dof_bodyid,\n\
         wp::array_t<wp::int32> var_body_isdofancestor,\n\
         wp::array_t<wp::vec_t<3, wp::float32>> var_xipos_in,\n\
         wp::array_t<wp::vec_t<3, wp::float32>> var_subtree_com_in,\n\
         wp::array_t<wp::vec_t<6, wp::float32>> var_cdof_in,\n\
         wp::array_t<wp::float32> var_qfrc_gravcomp_out)\n{{}}\n"
    )
    .into_bytes();
    let mut ptx = String::from(".version 8.8\n.target sm_89\n.address_size 64\n");
    ptx.push_str(&expected_ptx_signature(entry).join("\n"));
    ptx.push_str("\n{}\n");
    let ptx = ptx.into_bytes();
    let mut receipt = receipt(package);
    receipt["cu_sha256"] = Value::String(digest(&cu));
    receipt["ptx_sha256"] = Value::String(digest(&ptx));
    (cu, ptx, receipt)
}

fn write_candidate(directory: &Path, cu: &[u8], ptx: &[u8], receipt: &Value) {
    fs::write(directory.join("module.cu"), cu).unwrap();
    fs::write(directory.join("module.ptx"), ptx).unwrap();
    fs::write(
        directory.join("receipt.json"),
        serde_json::to_vec(receipt).unwrap(),
    )
    .unwrap();
}

#[test]
fn warp_abi_layout_and_gravity_descriptors_are_exact() {
    assert_eq!(size_of::<WarpArray>(), 56);
    assert_eq!(std::mem::align_of::<WarpArray>(), 8);
    assert_eq!(std::mem::offset_of!(WarpArray, shape), 16);
    assert_eq!(std::mem::offset_of!(WarpArray, strides), 32);
    assert_eq!(std::mem::offset_of!(WarpArray, ndim), 48);
    assert_eq!(std::mem::offset_of!(WarpArray, flags), 50);
    assert_eq!(size_of::<WarpBounds3>(), 32);
    assert_eq!(std::mem::align_of::<WarpBounds3>(), 8);
    assert_eq!(std::mem::offset_of!(WarpBounds3, size), 16);
    assert_eq!(std::mem::offset_of!(WarpBounds3, coord_mult), 24);

    let package = package().unwrap();
    validate_package(&package).unwrap();
    let bounds = WarpBounds3 {
        shape: [7, 4, 4],
        padding: 0,
        size: 112,
        coord_mult: 1,
    };
    assert_eq!(bounds.size, 7 * 4 * 4);
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
    let arrays = specs
        .into_iter()
        .enumerate()
        .map(|(index, (shape, bytes))| {
            contiguous(0x1000 + index as u64 * 0x1000, shape, bytes).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(arrays[0].strides, [12, 0, 0, 0]);
    assert_eq!(arrays[3].strides, [20, 4, 0, 0]);
    assert_eq!(arrays[7].strides, [60, 12, 0, 0]);
    assert_eq!(arrays[9].strides, [96, 24, 0, 0]);
    assert_eq!(arrays[10].strides, [16, 4, 0, 0]);
    assert!(contiguous(0, &[1], 4).is_err());
    assert!(contiguous(1, &[0], 4).is_err());
    assert!(contiguous(1, &[1, 1, 1, 1, 1], 4).is_err());
    assert_eq!(
        max_absolute_finite(&[f32::NAN], &[0.0]),
        Err(GateError::Mismatch("non-finite output"))
    );
    assert_eq!(
        max_absolute_finite(&[0.0], &[f32::INFINITY]),
        Err(GateError::Mismatch("non-finite output"))
    );
}

#[test]
fn frozen_input_and_artifact_gate_reject_drift() {
    let package = package().unwrap();
    validate_package(&package).unwrap();
    assert_eq!(
        package["artifact_requirements"]["actual_artifact_present"],
        false
    );
    assert_eq!(
        package["artifact_requirements"]["equivalence_eligible"],
        false
    );
    let missing = Temp::new("missing");
    assert_eq!(
        admit_artifact(&package, &missing.0),
        Err(GateError::Missing("artifact receipt"))
    );
    for (pointer, replacement) in [
        ("/upstream_revision", Value::String("wrong".into())),
        ("/warp_version", Value::String("1.17.0".into())),
        ("/dimensions/nv", Value::from(5)),
        ("/state/full_chain_independent", Value::Bool(true)),
    ] {
        let mut changed = package.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            validate_package(&changed).is_err(),
            "accepted drift: {pointer}"
        );
    }

    for (name, pointer, replacement) in [
        (
            "wrong-version",
            "/warp_version",
            Value::String("1.17.0".into()),
        ),
        ("wrong-rank", "/launch_rank", Value::from(2)),
        ("missing-array", "/array_arguments", Value::from(10)),
    ] {
        let temp = Temp::new(name);
        let mut value = receipt(&package);
        *value.pointer_mut(pointer).unwrap() = replacement;
        fs::write(
            temp.0.join("receipt.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_artifact_candidate(&package, &temp.0),
            Err(GateError::Mismatch("artifact receipt identity"))
        );
    }

    let valid = Temp::new("synthetic-valid");
    let (cu, ptx, value) = synthetic_candidate(&package);
    write_candidate(&valid.0, &cu, &ptx, &value);
    let identity = validate_artifact_candidate(&package, &valid.0).unwrap();
    assert_eq!(identity.evidence_kind, "synthetic-non-physical");
    assert_eq!(
        admit_artifact(&package, &valid.0),
        Err(GateError::Mismatch("artifact not registered"))
    );

    let changed = Temp::new("changed-byte");
    let mut changed_cu = cu.clone();
    changed_cu[0] ^= 1;
    write_candidate(&changed.0, &changed_cu, &ptx, &value);
    assert_eq!(
        validate_artifact_candidate(&package, &changed.0),
        Err(GateError::Mismatch("artifact hash"))
    );

    for (name, cu_edit, ptx_edit, expected) in [
        (
            "bounds-size",
            None,
            Some(("_param_0[32]", "_param_0[24]")),
            GateError::Mismatch("ptx abi"),
        ),
        (
            "array-type",
            Some((
                "wp::array_t<wp::float32> var_body_mass",
                "wp::array_t<wp::int32> var_body_mass",
            )),
            None,
            GateError::Mismatch("cuda abi"),
        ),
        (
            "array-order",
            Some(("var_body_parentid", "var_body_rootid_changed")),
            None,
            GateError::Mismatch("cuda abi"),
        ),
        (
            "wrong-target",
            None,
            Some((".target sm_89", ".target sm_90")),
            GateError::Mismatch("ptx target"),
        ),
        (
            "wrong-address-size",
            None,
            Some((".address_size 64", ".address_size 32")),
            GateError::Mismatch("ptx address size"),
        ),
        (
            "missing-parameter",
            None,
            Some((
                ".param .align 8 .b8 _gravity_force_untrusted_cuda_kernel_forward_param_11[56]\n",
                "",
            )),
            GateError::Mismatch("ptx abi"),
        ),
        (
            "comment-entry",
            Some((
                "extern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
                "// extern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
            )),
            None,
            GateError::Mismatch("entry missing"),
        ),
        (
            "block-comment-entry",
            Some((
                "extern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
                "/*\nextern \"C\" __global__ void _gravity_force_untrusted_cuda_kernel_forward(",
            )),
            None,
            GateError::Mismatch("entry missing"),
        ),
    ] {
        let temp = Temp::new(name);
        let mut changed_cu = String::from_utf8(cu.clone()).unwrap();
        let mut changed_ptx = String::from_utf8(ptx.clone()).unwrap();
        if let Some((from, to)) = cu_edit {
            changed_cu = changed_cu.replacen(from, to, 1);
        }
        if let Some((from, to)) = ptx_edit {
            changed_ptx = changed_ptx.replacen(from, to, 1);
        }
        let mut changed_receipt = value.clone();
        changed_receipt["cu_sha256"] = Value::String(digest(changed_cu.as_bytes()));
        changed_receipt["ptx_sha256"] = Value::String(digest(changed_ptx.as_bytes()));
        write_candidate(
            &temp.0,
            changed_cu.as_bytes(),
            changed_ptx.as_bytes(),
            &changed_receipt,
        );
        assert_eq!(
            validate_artifact_candidate(&package, &temp.0),
            Err(expected)
        );
    }
}

#[test]
#[ignore = "requires local frozen checkout, Warp 1.15 install and cache; no Python"]
fn local_frozen_install_matches_record_and_rejects_smooth_cache() {
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

#[cfg(windows)]
#[test]
#[ignore = "requires an SM89 GPU, NVIDIA driver, NVRTC and frozen Warp 1.15 headers"]
fn local_device_launch_confirms_warp_gravity_abi() {
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
