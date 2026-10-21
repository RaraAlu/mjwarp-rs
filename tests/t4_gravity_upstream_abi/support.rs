//! 重力ABI测试共用路径、摘要与数值工具。

use super::abi::*;
use super::artifact::validate_artifact_candidate;
use super::*;

pub(super) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn package() -> Result<Value, GateError> {
    if digest(PACKAGE.as_bytes()) != PACKAGE_SHA256 {
        return Err(GateError::Mismatch("package hash"));
    }
    serde_json::from_str(PACKAGE).map_err(|_| GateError::Parse("package"))
}

pub(super) fn read(path: &Path, what: &'static str) -> Result<Vec<u8>, GateError> {
    fs::read(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            GateError::Missing(what)
        } else {
            GateError::Io(what)
        }
    })
}

pub(super) fn read_stamped(path: &str, expected: &str) -> Result<Vec<u8>, GateError> {
    let bytes = read(&root().join(path), "stamped source")?;
    if digest(&bytes) != expected {
        return Err(GateError::Mismatch("source hash"));
    }
    Ok(bytes)
}

pub(super) fn usize_value(value: &Value, what: &'static str) -> Result<usize, GateError> {
    let value = value.as_u64().ok_or(GateError::Parse(what))?;
    usize::try_from(value).map_err(|_| GateError::Mismatch(what))
}

pub(super) fn f32_values(value: &Value, what: &'static str) -> Result<Vec<f32>, GateError> {
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

pub(super) fn max_absolute_finite(actual: &[f32], expected: &[f32]) -> Result<f32, GateError> {
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

pub(super) fn contiguous(
    data: u64,
    shape: &[usize],
    element_bytes: usize,
) -> Result<WarpArray, GateError> {
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

pub(super) fn stamp<'a>(
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

pub(super) fn safe_file(
    directory: &Path,
    name: &str,
    what: &'static str,
) -> Result<Vec<u8>, GateError> {
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

pub(super) fn strip_comments(source: &str) -> String {
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

pub(super) fn signature_lines(source: &str, opening: &str) -> Result<Vec<String>, GateError> {
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

pub(super) fn compact(lines: &[String]) -> String {
    lines
        .iter()
        .flat_map(|line| line.chars())
        .filter(|value| !value.is_ascii_whitespace())
        .collect()
}

pub(super) fn expected_cuda_signature(entry: &str) -> String {
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

pub(super) fn expected_ptx_signature(entry: &str) -> Vec<String> {
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

pub(super) fn admit_artifact(
    package: &Value,
    directory: &Path,
) -> Result<ArtifactIdentity, GateError> {
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

pub(super) struct Temp(pub(super) PathBuf);

impl Temp {
    pub(super) fn new(name: &str) -> Self {
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

pub(super) fn receipt(package: &Value) -> Value {
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
        "meta_file": "module.meta",
        "meta_sha256": "4a80327bc21a875a63b9cf3dfc43386ed40119012357560339f071ac50ea461c",
        "python_version": "3.12.13",
        "python_executable_sha256": "feeec36120097a03201a96ae17a0e2c1fba70dd6241cb4bfdd9e7c2a00bada77",
        "generator_file": "generate.py",
        "generator_sha256": "3f0348511e241a345cb3ae66b201cb02b1cef19d532b5980cd86a0e1700b0f2e",
        "warp_dll_sha256": "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6",
        "cuda_toolkit": "12.9",
        "ptx_compiler": "12.9.86",
        "driver_version": "596.36",
        "device": "NVIDIA GeForce RTX 4070 Ti SUPER",
        "generation_invocation_id": "exec-9a825267",
        "generation_log": "generation.log",
        "generation_log_sha256": "92626aabae26d638ad6d1c0e83461eac15a0d4c564afb9e3873dab9884578efa",
        "generation_execution_file": "generation-execution.json",
        "generation_execution_sha256": "8247ab2031fdccfd47d12ca8c3856150aed1c3d5539006cc65a0fd0f57edfd12",
        "compiler_provenance": "reconstructed_from_stdout_ptx_and_pinned_source_defaults",
        "authoring_only": true,
        "product_python_dependency": false,
    })
}

pub(super) fn synthetic_candidate(package: &Value) -> (Vec<u8>, Vec<u8>, Value) {
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

pub(super) fn write_candidate(directory: &Path, cu: &[u8], ptx: &[u8], receipt: &Value) {
    fs::write(directory.join("module.cu"), cu).unwrap();
    fs::write(directory.join("module.ptx"), ptx).unwrap();
    fs::write(
        directory.join("receipt.json"),
        serde_json::to_vec(receipt).unwrap(),
    )
    .unwrap();
}

pub(super) fn frozen_physical_input(package: &Value) -> FrozenWarpGravityInput {
    let model = &package["model"];
    let ints = |field: &str| {
        model[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
            .collect::<Vec<_>>()
    };
    let worlds = usize_value(&package["dimensions"]["worlds"], "worlds").unwrap();
    let nbody = usize_value(&package["dimensions"]["nbody"], "nbody").unwrap();
    let nv = usize_value(&package["dimensions"]["nv"], "nv").unwrap();
    let xipos = f32_values(&package["state"]["xipos_row"], "xipos row")
        .unwrap()
        .repeat(worlds);
    let (candidate_path, candidate_hash) = stamp(package, "input_sources", 0).unwrap();
    let candidate: Value =
        serde_json::from_slice(&read_stamped(candidate_path, candidate_hash).unwrap()).unwrap();
    let mut subtree_com = Vec::with_capacity(worlds * nbody * 3);
    let mut cdof = Vec::with_capacity(worlds * nv * 6);
    for row in candidate["candidate_com_diagnostics"].as_array().unwrap() {
        subtree_com.extend(f32_values(&row["subtree_com"], "subtree com").unwrap());
        cdof.extend(f32_values(&row["cdof"], "cdof").unwrap());
    }
    FrozenWarpGravityInput {
        worlds,
        nbody,
        nv,
        opt_gravity: f32_values(&model["opt_gravity"], "opt_gravity").unwrap(),
        body_parentid: ints("body_parentid"),
        body_rootid: ints("body_rootid"),
        body_mass: f32_values(&model["body_mass"], "body_mass").unwrap(),
        body_gravcomp: f32_values(&model["body_gravcomp"], "body_gravcomp").unwrap(),
        dof_bodyid: ints("dof_bodyid"),
        body_isdofancestor: ints("body_isdofancestor"),
        xipos,
        subtree_com,
        cdof,
    }
}

pub(super) fn analytic_expected(package: &Value) -> Vec<f32> {
    let (path, hash) = stamp(package, "input_sources", 1).unwrap();
    let contract: Value = serde_json::from_slice(&read_stamped(path, hash).unwrap()).unwrap();
    contract["expected"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| f32_values(row, "expected").unwrap())
        .collect()
}
