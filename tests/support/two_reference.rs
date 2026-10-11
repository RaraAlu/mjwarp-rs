//! 原生优先的字段块判定。
//! 本模块仅服务验收测试。
#![allow(dead_code)]

use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
pub struct Tolerance {
    pub absolute: f64,
    pub relative: f64,
}

#[derive(Clone, Debug)]
pub struct Reference {
    pub identity: String,
    pub values: Vec<f64>,
    pub tolerance: Option<Tolerance>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    NativePassed,
    MjwarpPassed,
    Failed,
    Pending,
}

impl Status {
    fn name(self) -> &'static str {
        match self {
            Self::NativePassed => "native_passed",
            Self::MjwarpPassed => "mjwarp_passed",
            Self::Failed => "failed",
            Self::Pending => "pending",
        }
    }
}

#[derive(Debug)]
pub struct Comparison {
    pub identity: String,
    pub count: usize,
    pub max_absolute: f64,
    pub worst_index: Option<usize>,
    pub violations: Option<usize>,
    pub tolerance: Option<Tolerance>,
}

impl Comparison {
    fn json(&self) -> Value {
        json!({
            "identity": self.identity, "count": self.count,
            "max_absolute": self.max_absolute, "worst_index": self.worst_index,
            "violations": self.violations,
            "tolerance": self.tolerance.map(|t| json!({"absolute": t.absolute, "relative": t.relative}))
        })
    }
}

#[derive(Debug)]
pub struct Decision {
    pub status: Status,
    pub reason: String,
    pub native: Option<Comparison>,
    pub mjwarp: Option<Comparison>,
}

impl Decision {
    pub fn passed(&self) -> bool {
        matches!(self.status, Status::NativePassed | Status::MjwarpPassed)
    }

    pub fn json(&self) -> Value {
        json!({"status": self.status.name(), "reason": self.reason,
            "native": self.native.as_ref().map(Comparison::json),
            "mjwarp": self.mjwarp.as_ref().map(Comparison::json)})
    }
}

fn compare(actual: &[f32], reference: Reference) -> Result<Comparison, &'static str> {
    if reference.identity.is_empty() {
        return Err("missing_reference_identity");
    }
    if actual.len() != reference.values.len() {
        return Err("reference_length_mismatch");
    }
    if reference.values.iter().any(|v| !v.is_finite()) {
        return Err("nonfinite_reference");
    }
    if let Some(t) = reference.tolerance
        && (!t.absolute.is_finite()
            || !t.relative.is_finite()
            || t.absolute < 0.0
            || t.relative < 0.0)
    {
        return Err("invalid_tolerance");
    }
    let mut result = Comparison {
        identity: reference.identity,
        count: actual.len(),
        max_absolute: 0.0,
        worst_index: None,
        violations: reference.tolerance.map(|_| 0),
        tolerance: reference.tolerance,
    };
    for (index, (&a, &e)) in actual.iter().zip(&reference.values).enumerate() {
        let error = (f64::from(a) - e).abs();
        if !error.is_finite() {
            return Err("nonfinite_error");
        }
        if error > result.max_absolute {
            result.max_absolute = error;
            result.worst_index = Some(index);
        }
        if let Some(t) = result.tolerance {
            let limit = t.absolute + t.relative * e.abs();
            if !limit.is_finite() {
                return Err("nonfinite_tolerance_limit");
            }
            if error > limit {
                *result.violations.as_mut().unwrap() += 1;
            }
        }
    }
    Ok(result)
}

/// 仅在原生字段块超差后取第二参考。
/// 提供方负责核对模型、状态与哈希。
pub fn evaluate(
    actual: &[f32],
    native: Option<Reference>,
    mjwarp: impl FnOnce() -> Result<Option<Reference>, String>,
) -> Decision {
    let mut decision = Decision {
        status: Status::Failed,
        reason: String::new(),
        native: None,
        mjwarp: None,
    };
    if actual.iter().any(|v| !v.is_finite()) {
        decision.reason = "nonfinite_actual".into();
        return decision;
    }
    let Some(native) = native else {
        decision.status = Status::Pending;
        decision.reason = "missing_native_reference".into();
        return decision;
    };
    match compare(actual, native) {
        Err(reason) => decision.reason = reason.into(),
        Ok(comparison) => {
            let violations = comparison.violations;
            decision.native = Some(comparison);
            match violations {
                None => {
                    decision.status = Status::Pending;
                    decision.reason = "missing_native_tolerance".into();
                }
                Some(0) => {
                    decision.status = Status::NativePassed;
                    decision.reason = "native_within_tolerance".into();
                }
                Some(_) => match mjwarp() {
                    Err(reason) => decision.reason = format!("invalid_mjwarp_reference: {reason}"),
                    Ok(None) => {
                        decision.status = Status::Pending;
                        decision.reason = "missing_mjwarp_reference".into();
                    }
                    Ok(Some(reference)) => match compare(actual, reference) {
                        Err(reason) => decision.reason = reason.into(),
                        Ok(comparison) => {
                            let violations = comparison.violations;
                            decision.mjwarp = Some(comparison);
                            (decision.status, decision.reason) = match violations {
                                None => (Status::Pending, "missing_mjwarp_tolerance".into()),
                                Some(0) => (Status::MjwarpPassed, "mjwarp_within_tolerance".into()),
                                Some(_) => (Status::Failed, "both_references_exceeded".into()),
                            };
                        }
                    },
                },
            }
        }
    }
    decision
}

pub fn native_reference(
    identity: impl Into<String>,
    values: Vec<f64>,
    absolute: f64,
    relative: f64,
) -> Reference {
    Reference {
        identity: identity.into(),
        values,
        tolerance: Some(Tolerance { absolute, relative }),
    }
}

/// 原生已通过的路径无需准备第二参考。
/// 若将来超差，则明确阻止验收。
pub fn assert_native(
    identity: &str,
    actual: &[f32],
    expected: Vec<f64>,
    absolute: f64,
    relative: f64,
) -> f64 {
    let decision = evaluate(
        actual,
        Some(native_reference(identity, expected, absolute, relative)),
        || Ok(None),
    );
    assert!(decision.passed(), "NP03 {}", decision.json());
    decision.native.unwrap().max_absolute
}

/// 汇总后再失败，避免只记录首个超差。
pub struct Audit {
    scope: String,
    counts: [usize; 4],
    details: Vec<Value>,
}

impl Audit {
    pub fn new(scope: impl Into<String>) -> Self {
        Self {
            scope: scope.into(),
            counts: [0; 4],
            details: Vec::new(),
        }
    }

    pub fn record(&mut self, context: Value, decision: Decision) -> f64 {
        self.counts[match decision.status {
            Status::NativePassed => 0,
            Status::MjwarpPassed => 1,
            Status::Failed => 2,
            Status::Pending => 3,
        }] += 1;
        let maximum = decision.native.as_ref().map_or(0.0, |c| c.max_absolute);
        if decision.status != Status::NativePassed {
            let detail = json!({"context": context, "decision": decision.json()});
            eprintln!("NP03_DETAIL {detail}");
            self.details.push(detail);
        }
        maximum
    }

    pub fn finish(self) {
        let report = json!({"schema": 1, "scope": self.scope,
            "native_passed": self.counts[0], "mjwarp_passed": self.counts[1],
            "failed": self.counts[2], "pending": self.counts[3], "details": self.details});
        eprintln!("NP03_SUMMARY {report}");
        assert!(
            self.counts[2] == 0 && self.counts[3] == 0,
            "NP03 gate failed: {report}"
        );
    }
}
