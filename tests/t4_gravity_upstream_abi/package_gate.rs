//! 上游重力包结构校验。

use super::abi::*;
use super::support::*;
use super::*;

pub(super) fn validate_package(package: &Value) -> Result<(), GateError> {
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
        || requirements["receipt_fields"].as_array().map(Vec::len) != Some(38)
        || requirements["compile_options"]["requires_actual_receipt"] != true
        || requirements["device_output_required"] != true
        || requirements["full_chain_independent"] != false
    {
        return Err(GateError::Mismatch("artifact boundary"));
    }
    let pipeline = &package["generation_pipeline"];
    if package["warp_install"].as_array().map(Vec::len) != Some(11)
        || pipeline["python_codegen_entry"] != "warp._src.context.Module._run_codegen"
        || pipeline["native_compile_entry"] != "wp_cuda_compile_program"
        || pipeline["native_compile_binary_pinned"] != true
        || pipeline["non_python_codegen_export_present"] != false
        || pipeline["artifact_directory"] != "fixtures/gravity-upstream-abi/generated"
        || pipeline["python_execution_allowed"] != false
        || pipeline["offline_python_authoring_allowed"] != true
        || pipeline["runtime_dependency"] != "rust_cuda_driver_only"
    {
        return Err(GateError::Mismatch("generation pipeline"));
    }
    Ok(())
}
