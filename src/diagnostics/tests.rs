use super::*;

#[test]
fn preserves_resident_model_and_stage_errors() {
    use std::error::Error;
    assert_eq!(
        TransferError::ModelMismatch.to_string(),
        "状态不属于当前设备模型"
    );
    let error = TransferError::StageNotReady { stage: "com" };
    assert_eq!(error.to_string(), "辅助阶段尚未更新：com");
    assert!(error.source().is_none());
    assert!(TransferError::ModelMismatch.source().is_none());
}

#[test]
fn preserves_factorization_world_dof_value_and_source() {
    use std::error::Error;
    let error = TransferError::InvalidPivot {
        world: 2,
        dof: 3,
        value: -0.25,
    };
    assert_eq!(error.to_string(), "分解主元无效：世界2；自由度3；值-0.25");
    assert!(error.source().is_none());
    let value = -1.0 - 2.0f64.powi(-40);
    let error = TransferError::InvalidPivot {
        world: 2,
        dof: 3,
        value,
    };
    let TransferError::InvalidPivot { value: actual, .. } = error else {
        unreachable!()
    };
    assert_eq!(actual.to_bits(), value.to_bits());
    assert_ne!(actual, f64::from(value as f32));
}

#[test]
fn preserves_native_error_stage_version_and_source() {
    use std::error::Error;
    assert_eq!(
        NativeProbeError::Native {
            stage: "load_library",
            code: 126
        }
        .to_string(),
        "原生失败：load_library；错误码126"
    );
    let error = NativeProbeError::from(InputError::InvalidDimension { field: "nbody" });
    assert_eq!(error.source().unwrap().to_string(), "维度无效：nbody");
    assert!(NativeProbeError::AbiMismatch.source().is_none());
    assert!(
        NativeProbeError::VersionMismatch {
            expected: 3012000,
            actual: 123
        }
        .to_string()
        .contains("实际123")
    );
}

#[test]
fn preserves_transfer_error_sources_and_ranges() {
    use std::error::Error;
    let input = InputError::InvalidRange {
        field: "buffer",
        offset: 3,
        elements: 2,
        capacity: 4,
    };
    let error = TransferError::from(input.clone());
    assert_eq!(error, TransferError::Input(input));
    assert!(error.source().unwrap().to_string().contains("偏移3"));
    let error = TransferError::from(ProbeError::Cuda {
        stage: "copy",
        code: 700,
    });
    assert_eq!(
        error.source().unwrap().to_string(),
        "CUDA失败：copy；错误码700"
    );
    assert!(TransferError::Quarantined.source().is_none());
}

#[test]
fn preserves_input_field_and_index() {
    let error = InputError::InvalidIndex {
        field: "world",
        index: 3,
        limit: 2,
    };
    assert_eq!(error.to_string(), "索引越界：world[3]；上限2");
    let error = InputError::NonFinite {
        field: "state",
        index: 7,
    };
    assert_eq!(error.to_string(), "数值非有限：state[7]");
    let error = InputError::InvalidTopology {
        field: "jnt_bodyid",
        index: 2,
        reason: "joint_owner_mismatch",
    };
    assert_eq!(
        error.to_string(),
        "拓扑无效：jnt_bodyid[2]；joint_owner_mismatch"
    );
}

#[test]
fn preserves_cuda_stage_and_code() {
    let error = ProbeError::Cuda {
        stage: "load",
        code: 200,
    };
    assert_eq!(error.to_string(), "CUDA失败：load；错误码200");
}

#[test]
fn preserves_large_integer_mismatches_without_rounding() {
    let error = ProbeError::IntegerMismatch {
        index: 0,
        expected: 16_777_217,
        actual: 16_777_216,
    };
    assert_eq!(
        error.to_string(),
        "整数输出[0]错误：期望16777217，实际16777216"
    );
}
