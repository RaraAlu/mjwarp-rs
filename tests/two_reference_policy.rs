//! 原生优先的误差红线真值表。
//! 本目标不需要GPU。
#[path = "support/two_reference.rs"]
mod policy;
use policy::{
    Audit, MJWARP_RELATIVE_SLACK, Reference, SecondaryReference, Status, Tolerance, evaluate,
    native_reference,
};
use serde_json::json;

fn native(values: &[f64]) -> Reference {
    native_reference("native:fixture:field", values.to_vec(), 0.125, 0.0)
}

fn secondary(values: &[f32]) -> SecondaryReference {
    SecondaryReference {
        identity: "mjwarp:fixture:field".into(),
        conditions: "verified:model:state:world:field".into(),
        values: values.to_vec(),
        relative_slack: Some(MJWARP_RELATIVE_SLACK),
    }
}

#[test]
fn native_pass_does_not_request_second_reference() {
    let d = evaluate(&[1.0], Some(native(&[1.0])), || panic!("must stay lazy"));
    assert_eq!(d.status, Status::NativePassed);
    assert!(d.mjwarp.is_none());
    assert_eq!(d.json()["admission_label"], "原生放行");
    assert_eq!(d.json()["accepted_reference"], "native_mujoco");
}

#[test]
fn better_native_accuracy_passes_even_far_from_mjwarp() {
    let d = evaluate(&[2.0], Some(native(&[0.0])), || Ok(Some(secondary(&[4.0]))));
    assert_eq!(d.status, Status::MjwarpPassed);
    let r = d.json();
    assert_eq!(r["admission_label"], "对照放行");
    assert_eq!(r["native"]["max_normalized"], 16.0);
    assert_eq!(r["mjwarp"]["max_normalized"], 32.0);
    assert_eq!(r["mjwarp"]["max_absolute"], 4.0);
    assert_eq!(r["error_baseline"], "native_mujoco");
    assert_eq!(r["np03_admitted"], true);
    assert_eq!(r["accepted_reference"], "mjwarp");
}

#[test]
fn equal_error_passes_without_claiming_native_tolerance_pass() {
    let d = evaluate(&[1.0], Some(native(&[0.0])), || {
        Ok(Some(secondary(&[-1.0])))
    });
    assert_eq!(d.status, Status::MjwarpPassed);
    assert_eq!(d.native.as_ref().unwrap().violations, Some(1));
    assert_eq!(d.json()["admission_label"], "对照放行");
}

#[test]
fn small_degradation_uses_the_frozen_five_percent_redline() {
    assert_eq!(MJWARP_RELATIVE_SLACK, 0.05);
    let d = evaluate(&[1.03125], Some(native(&[0.0])), || {
        Ok(Some(secondary(&[1.0])))
    });
    let r = d.json();
    assert!(d.passed());
    assert_eq!(r["native"]["max_normalized"], 8.25);
    assert_eq!(r["mjwarp"]["max_normalized"], 8.0);
    assert_eq!(r["redline"], 8.4);
    assert_eq!(r["relative_slack"], 0.05);
    assert_eq!(r["admission_label"], "红线内放行");
}

#[test]
fn redline_boundary_is_inclusive_and_next_float_fails() {
    // 160乘105%恰好等于168。
    for (actual, passed) in [
        (168.0_f32, true),
        (f32::from_bits(168.0_f32.to_bits() + 1), false),
    ] {
        let d = evaluate(&[actual], Some(native(&[0.0])), || {
            Ok(Some(secondary(&[160.0])))
        });
        assert_eq!(d.passed(), passed);
        if !passed {
            assert_eq!(d.reason, "native_error_exceeds_mjwarp_redline");
            assert_eq!(d.json()["admission_label"], "失败不放行");
        }
    }
}

#[test]
fn close_to_mjwarp_does_not_override_the_native_error_redline() {
    let d = evaluate(&[0.15625], Some(native(&[0.0])), || {
        Ok(Some(secondary(&[0.125])))
    });
    assert_eq!(d.status, Status::Failed);
    assert_eq!(d.reason, "native_error_exceeds_mjwarp_redline");
    assert!((0.15625_f32 - 0.125).abs() < 0.125);
}

#[test]
fn normalized_block_maximum_not_raw_absolute_maximum_controls_admission() {
    let r = native_reference("scaled", vec![0.0, 1000.0], 0.01, 0.01);
    let d = evaluate(&[0.03125, 1000.0], Some(r), || {
        Ok(Some(secondary(&[0.0, 1020.0])))
    });
    assert!(d.native.as_ref().unwrap().max_absolute < d.mjwarp.as_ref().unwrap().max_absolute);
    assert_eq!(d.status, Status::Failed);
}

#[test]
fn field_block_uses_one_common_baseline_without_reference_splicing() {
    let d = evaluate(&[1.0, 2.0], Some(native(&[1.0, 3.0])), || {
        Ok(Some(secondary(&[4.0, 2.0])))
    });
    assert!(d.passed());
    assert_eq!(d.native.as_ref().unwrap().max_normalized, Some(8.0));
    assert_eq!(d.mjwarp.as_ref().unwrap().max_normalized, Some(24.0));
}

#[test]
fn missing_secondary_reference_or_needed_redline_stays_pending() {
    let d = evaluate(&[1.0], Some(native(&[0.0])), || Ok(None));
    assert_eq!(d.status, Status::Pending);
    let mut r = secondary(&[1.0]);
    r.relative_slack = None;
    let d = evaluate(&[2.0], Some(native(&[0.0])), || Ok(Some(r)));
    assert_eq!(d.status, Status::Pending);
    assert_eq!(d.reason, "missing_mjwarp_redline");
    assert_eq!(d.json()["admission_label"], "待验收不放行");
    assert_eq!(d.json()["accepted_reference"], serde_json::Value::Null);
}

#[test]
fn improved_error_does_not_need_a_degradation_allowance() {
    let mut r = secondary(&[2.0]);
    r.relative_slack = None;
    let d = evaluate(&[1.0], Some(native(&[0.0])), || Ok(Some(r)));
    assert!(d.passed());
    assert_eq!(d.json()["admission_label"], "对照放行");
}

#[test]
fn missing_native_evidence_never_invokes_fallback() {
    let d = evaluate(&[1.0], None, || panic!("no fallback"));
    assert_eq!(d.status, Status::Pending);
    let mut r = native(&[1.0]);
    r.tolerance = None;
    let d = evaluate(&[1.0], Some(r), || panic!("no fallback"));
    assert_eq!(d.status, Status::Pending);
}

#[test]
fn nonfinite_values_and_native_shape_errors_never_fall_back() {
    for actual in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            evaluate(&[actual], Some(native(&[0.0])), || panic!("no fallback")).status,
            Status::Failed
        );
    }
    for values in [vec![], vec![f64::NAN], vec![f64::INFINITY]] {
        assert_eq!(
            evaluate(&[1.0], Some(native(&values)), || panic!("no fallback")).status,
            Status::Failed
        );
    }
}

#[test]
fn invalid_secondary_evidence_fails_closed() {
    assert_eq!(
        evaluate(&[1.0], Some(native(&[0.0])), || Err(
            "input_hash_mismatch".into()
        ))
        .status,
        Status::Failed
    );
    for values in [
        vec![],
        vec![f32::NAN],
        vec![f32::INFINITY],
        vec![f32::NEG_INFINITY],
    ] {
        assert_eq!(
            evaluate(&[1.0], Some(native(&[0.0])), || Ok(Some(secondary(
                &values
            ))))
            .status,
            Status::Failed
        );
    }
    for clear_identity in [false, true] {
        let mut r = secondary(&[2.0]);
        if clear_identity {
            r.identity.clear();
        } else {
            r.conditions.clear();
        }
        let d = evaluate(&[1.0], Some(native(&[0.0])), || Ok(Some(r)));
        assert_eq!(d.status, Status::Failed);
        assert_eq!(d.reason, "missing_mjwarp_identity_or_conditions");
    }
}

#[test]
fn invalid_redlines_fail_even_when_candidate_is_better() {
    for slack in [-0.01, f64::NAN, f64::INFINITY, f64::MAX] {
        let mut r = secondary(&[2.0]);
        r.relative_slack = Some(slack);
        let d = evaluate(&[1.0], Some(native(&[0.0])), || Ok(Some(r)));
        assert_eq!(d.status, Status::Failed);
    }
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
        let mut r = native(&[2.0]);
        r.tolerance = Some(t);
        assert_eq!(
            evaluate(&[1.0], Some(r), || panic!("no fallback")).status,
            Status::Failed
        );
    }
}

#[test]
fn tolerance_boundary_signed_zero_and_empty_shapes_remain_valid() {
    let r = native_reference("boundary", vec![1.0], 0.125, 0.125);
    assert_eq!(
        evaluate(&[1.25], Some(r), || panic!("no fallback")).status,
        Status::NativePassed
    );
    let r = native_reference("zero", vec![0.0], 0.0, 0.0);
    assert!(evaluate(&[-0.0], Some(r), || panic!("no fallback")).passed());
    let d = evaluate(&[], Some(native(&[])), || panic!("no fallback"));
    assert_eq!(d.native.unwrap().count, 0);
}

#[test]
fn undefined_normalization_never_admits_infinity_against_infinity() {
    let r = native_reference("zero-limit", vec![0.0], 0.0, 0.0);
    let d = evaluate(&[1.0], Some(r), || Ok(Some(secondary(&[2.0]))));
    assert_eq!(d.status, Status::Failed);
    assert_eq!(d.reason, "undefined_normalized_error");
    assert_eq!(d.native.unwrap().max_normalized, None);
}

#[test]
fn empty_audit_and_only_empty_fields_do_not_admit_numerical_acceptance() {
    for record_empty in [false, true] {
        let mut audit = Audit::new("no-numerical-evidence");
        if record_empty {
            audit.record(
                json!({"field":"empty"}),
                evaluate(&[], Some(native(&[])), || panic!("no fallback")),
            );
        }
        let r = audit.report();
        assert_eq!(r["evaluated_blocks"], usize::from(record_empty));
        assert_eq!(r["compared_scalars"], 0);
        assert_eq!(r["np03_admitted"], false);
        assert_eq!(r["admission_label"], "未验收不放行");
        assert!(std::panic::catch_unwind(|| audit.finish()).is_err());
    }
}

#[test]
fn audit_admits_all_three_pass_reasons_and_discloses_scope() {
    let mut audit = Audit::new("three-pass-reasons");
    for d in [
        evaluate(&[0.0], Some(native(&[0.0])), || panic!("no fallback")),
        evaluate(&[1.0], Some(native(&[0.0])), || Ok(Some(secondary(&[2.0])))),
        evaluate(&[1.03125], Some(native(&[0.0])), || {
            Ok(Some(secondary(&[1.0])))
        }),
    ] {
        audit.record(json!({"field":"synthetic"}), d);
    }
    let r = audit.report();
    assert_eq!(r["schema"], 2);
    assert_eq!(r["policy"], "native_error_v2");
    assert_eq!(r["coverage"], "recorded_blocks_only");
    assert_eq!(r["native_passed"], 1);
    assert_eq!(r["mjwarp_passed"], 2);
    assert_eq!(r["compared_scalars"], 3);
    assert_eq!(r["np03_admitted"], true);
    assert_eq!(r["details"][0]["decision"]["admission_label"], "对照放行");
    assert_eq!(r["details"][1]["decision"]["admission_label"], "红线内放行");
    audit.finish();
}

#[test]
fn failed_or_pending_blocks_prevent_aggregate_admission() {
    for with_failure in [false, true] {
        let mut audit = Audit::new("blocked-mixed");
        audit.record(
            json!({"field":"native"}),
            evaluate(&[0.0], Some(native(&[0.0])), || panic!("no fallback")),
        );
        audit.record(
            json!({"field":"pending"}),
            evaluate(&[1.0], Some(native(&[0.0])), || Ok(None)),
        );
        if with_failure {
            audit.record(
                json!({"field":"redline"}),
                evaluate(&[2.0], Some(native(&[0.0])), || Ok(Some(secondary(&[1.0])))),
            );
        }
        let r = audit.report();
        assert_eq!(r["np03_admitted"], false);
        assert_eq!(r["pending"], 1);
        assert_eq!(r["failed"], usize::from(with_failure));
        assert_eq!(
            r["admission_label"],
            if with_failure {
                "失败不放行"
            } else {
                "待验收不放行"
            }
        );
        assert!(std::panic::catch_unwind(|| audit.finish()).is_err());
    }
}
