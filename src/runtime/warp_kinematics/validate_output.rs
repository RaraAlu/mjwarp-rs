//! 冻结输出与裸位校验。

use super::support::artifact;
use super::support::checked_product;
use super::support::digest;
use super::*;

pub(super) fn validate_output(
    output: &FrozenOutputFile,
    input: &FrozenWarpKinematicsInput,
) -> Result<(), ProbeError> {
    let world_body = checked_product(input.worlds, input.nbody)?;
    let world_joint = checked_product(input.worlds, input.njnt)?;
    if output.schema != 1
        || output.evidence_kind != "frozen-upstream-generated-qpos-kinematics-com-cdof"
        || output.upstream_revision != REVISION
        || output.warp_version != WARP_VERSION
        || output.input_sha256 != INPUT_SHA256
        || (
            output.worlds,
            output.nbody,
            output.njnt,
            output.nq,
            output.nv,
        ) != (input.worlds, input.nbody, input.njnt, input.nq, input.nv)
        || output.xpos.len() != checked_product(world_body, 3)?
        || output.xquat_wxyz.len() != checked_product(world_body, 4)?
        || output.xipos.len() != checked_product(world_body, 3)?
        || output.xmat.len() != checked_product(world_body, 9)?
        || output.xanchor.len() != checked_product(world_joint, 3)?
        || output.xaxis.len() != checked_product(world_joint, 3)?
        || output.subtree_com.len() != checked_product(world_body, 3)?
        || output.cdof.len() != checked_product(input.worlds * input.nv, 6)?
        || !output.legacy_static_fields_bit_identical
        || !output.legacy_com_cdof_bit_identical
        || output.candidate_kinematics_inputs
    {
        return Err(artifact("output", "shape or identity mismatch"));
    }
    Ok(())
}

pub(super) fn validate_raw(
    raw: &RawBitsFile,
    output: &FrozenOutputFile,
) -> Result<Vec<Vec<u32>>, ProbeError> {
    if raw.schema != 1
        || raw.provenance != "direct_upstream_device_readback_before_finite_or_exact_validation"
        || raw.upstream_revision != REVISION
        || raw.fields.len() != 8
    {
        return Err(artifact("raw bits", "identity mismatch"));
    }
    let expected = [
        ("xpos", output.xpos.as_slice()),
        ("xquat_wxyz", output.xquat_wxyz.as_slice()),
        ("xipos", output.xipos.as_slice()),
        ("xmat", output.xmat.as_slice()),
        ("xanchor", output.xanchor.as_slice()),
        ("xaxis", output.xaxis.as_slice()),
        ("subtree_com", output.subtree_com.as_slice()),
        ("cdof", output.cdof.as_slice()),
    ];
    raw.fields
        .iter()
        .zip(expected)
        .map(|(field, (name, values))| validate_raw_field(field, name, values))
        .collect()
}

pub(super) fn validate_raw_field(
    field: &RawField,
    name: &str,
    values: &[f32],
) -> Result<Vec<u32>, ProbeError> {
    if field.field != name || field.count != values.len() || field.bits.len() != values.len() {
        return Err(artifact("raw bits", "field shape mismatch"));
    }
    let bits = field
        .bits
        .iter()
        .map(|value| {
            u32::from_str_radix(value, 16).map_err(|_| artifact("raw bits", "invalid hex"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if bits
        .iter()
        .zip(values)
        .any(|(&actual, expected)| actual != expected.to_bits())
    {
        return Err(artifact("raw bits", "value mismatch"));
    }
    let bytes = bits
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>();
    if digest(&bytes) != field.little_endian_sha256 {
        return Err(artifact("raw bits", "byte hash mismatch"));
    }
    Ok(bits)
}
