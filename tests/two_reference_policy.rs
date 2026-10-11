//! 两级判定真值表，不需要GPU。
#[path = "support/two_reference.rs"]
mod policy;
use policy::{Reference, Status, Tolerance, evaluate, native_reference};

fn reference(values: &[f64]) -> Reference {
    native_reference("fixture:sha256:field", values.to_vec(), 0.01, 0.01)
}

#[test]
fn native_pass_does_not_request_second_reference() {
    let decision = evaluate(&[1.0], Some(reference(&[1.0])), || panic!("must stay lazy"));
    assert_eq!(decision.status, Status::NativePassed);
    assert!(decision.mjwarp.is_none());
}

#[test]
fn second_reference_pass_has_equal_admission() {
    let decision = evaluate(&[1.0], Some(reference(&[2.0])), || {
        Ok(Some(reference(&[1.0])))
    });
    assert_eq!(decision.status, Status::MjwarpPassed);
    assert!(decision.passed());
    assert_eq!(decision.native.unwrap().violations, Some(1));
}

#[test]
fn neither_reference_passes_a_mixed_element_block() {
    let decision = evaluate(&[1.0, 2.0], Some(reference(&[1.0, 3.0])), || {
        Ok(Some(reference(&[4.0, 2.0])))
    });
    assert_eq!(decision.status, Status::Failed);
    assert_eq!(decision.reason, "both_references_exceeded");
}

#[test]
fn missing_secondary_reference_or_tolerance_stays_pending() {
    let missing = evaluate(&[1.0], Some(reference(&[2.0])), || Ok(None));
    assert_eq!(missing.status, Status::Pending);
    assert!(!missing.passed());
    let mut second = reference(&[1.0]);
    second.tolerance = None;
    let missing = evaluate(&[1.0], Some(reference(&[2.0])), || Ok(Some(second)));
    assert_eq!(missing.status, Status::Pending);
    assert_eq!(missing.reason, "missing_mjwarp_tolerance");
}

#[test]
fn missing_native_evidence_never_invokes_fallback() {
    let missing = evaluate(&[1.0], None, || panic!("must stay lazy"));
    assert_eq!(missing.status, Status::Pending);
    let mut native = reference(&[1.0]);
    native.tolerance = None;
    let missing = evaluate(&[1.0], Some(native), || panic!("must stay lazy"));
    assert_eq!(missing.status, Status::Pending);
}

#[test]
fn nonfinite_values_and_shape_errors_never_fall_back() {
    for values in [vec![f32::NAN], vec![f32::INFINITY], vec![f32::NEG_INFINITY]] {
        assert_eq!(
            evaluate(&values, Some(reference(&[1.0])), || panic!("no fallback")).status,
            Status::Failed
        );
    }
    for values in [vec![], vec![f64::NAN], vec![f64::INFINITY]] {
        assert_eq!(
            evaluate(&[1.0], Some(reference(&values)), || panic!("no fallback")).status,
            Status::Failed
        );
    }
}

#[test]
fn invalid_secondary_identity_or_shape_fails_closed() {
    assert_eq!(
        evaluate(&[1.0], Some(reference(&[2.0])), || Err(
            "input hash mismatch".into()
        ))
        .status,
        Status::Failed
    );
    assert_eq!(
        evaluate(&[1.0], Some(reference(&[2.0])), || Ok(Some(reference(&[])))).status,
        Status::Failed
    );
    let mut second = reference(&[1.0]);
    second.identity.clear();
    assert_eq!(
        evaluate(&[1.0], Some(reference(&[2.0])), || Ok(Some(second))).status,
        Status::Failed
    );
}

#[test]
fn tolerance_validation_rejects_negative_nonfinite_and_overflow() {
    for t in [
        Tolerance {
            absolute: -1.0,
            relative: 0.0,
        },
        Tolerance {
            absolute: f64::NAN,
            relative: 0.0,
        },
        Tolerance {
            absolute: 0.0,
            relative: f64::INFINITY,
        },
        Tolerance {
            absolute: f64::MAX,
            relative: f64::MAX,
        },
    ] {
        let mut native = reference(&[2.0]);
        native.tolerance = Some(t);
        assert_eq!(
            evaluate(&[1.0], Some(native), || panic!("no fallback")).status,
            Status::Failed
        );
    }
}

#[test]
fn tolerance_boundary_zero_and_empty_blocks_remain_well_defined() {
    let native = native_reference("boundary", vec![1.0], 0.125, 0.125);
    assert_eq!(
        evaluate(&[1.25], Some(native), || panic!("no fallback")).status,
        Status::NativePassed
    );
    assert_eq!(
        evaluate(
            &[-0.0],
            Some(native_reference("zero", vec![0.0], 0.0, 0.0)),
            || panic!("no fallback")
        )
        .status,
        Status::NativePassed
    );
    let empty = evaluate(&[], Some(reference(&[])), || panic!("no fallback"));
    assert_eq!(empty.native.unwrap().count, 0);
}

#[test]
fn invalid_secondary_numeric_data_cannot_turn_into_pending() {
    for values in [vec![f64::NAN], vec![f64::INFINITY]] {
        let mut secondary = reference(&values);
        secondary.tolerance = None;
        let decision = evaluate(&[1.0], Some(reference(&[2.0])), || Ok(Some(secondary)));
        assert_eq!(decision.status, Status::Failed);
    }
}

#[test]
fn report_gate_rejects_pending_and_failed_but_accepts_either_pass() {
    for status in [
        Status::NativePassed,
        Status::MjwarpPassed,
        Status::Pending,
        Status::Failed,
    ] {
        let mut audit = policy::Audit::new("synthetic-policy-test");
        audit.record(
            serde_json::json!({"field": "test"}),
            policy::Decision {
                status,
                reason: "unit test".into(),
                native: None,
                mjwarp: None,
            },
        );
        let result = std::panic::catch_unwind(|| audit.finish());
        assert_eq!(
            result.is_ok(),
            matches!(status, Status::NativePassed | Status::MjwarpPassed)
        );
    }
}
