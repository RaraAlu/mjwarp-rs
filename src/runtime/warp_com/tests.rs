//! 冻结质心导入器单元测试。

use super::util::digest;
use super::validate::{validate_input, validate_raw_field};
use super::*;

#[test]
fn rejects_raw_bit_and_topology_drift() {
    let mut input: FrozenWarpComInput = serde_json::from_str(include_str!(
        "../../../fixtures/gravity-upstream-abi/independent-inputs/input.json"
    ))
    .unwrap();
    validate_input(&input).unwrap();
    input.body_tree_levels[0][0] = 2;
    assert!(validate_input(&input).is_err());

    let values = [0.0_f32, -1.0];
    let mut field = RawField {
        field: "test".into(),
        count: 2,
        little_endian_sha256: digest(
            &values
                .iter()
                .flat_map(|value| value.to_bits().to_le_bytes())
                .collect::<Vec<_>>(),
        ),
        bits: values
            .iter()
            .map(|value| format!("{:08x}", value.to_bits()))
            .collect(),
    };
    validate_raw_field(&field, "test", &values).unwrap();
    field.bits[1] = "00000000".into();
    assert!(validate_raw_field(&field, "test", &values).is_err());

    let non_finite = FrozenWarpComReadback {
        subtree_com: vec![f32::NAN],
        cdof: vec![0.0],
        subtree_com_bits: vec![f32::NAN.to_bits()],
        cdof_bits: vec![0],
    };
    assert_eq!(non_finite.subtree_com_bits(), &[f32::NAN.to_bits()]);
    assert!(non_finite.into_finite().is_err());
}
