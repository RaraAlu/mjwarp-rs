//! 原生优先的字段块判定。
//! 本模块仅服务验收测试。
#![allow(dead_code)]

use serde_json::{Value, json};

// 用户于2026-10-11冻结5%余量。
// 它约束归一化误差，不约束输出值。
pub const MJWARP_RELATIVE_SLACK: f64 = 0.05;

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

/// 两路GPU结果共享原生基准。
/// 调用方先核对模型与状态。
#[derive(Clone, Debug)]
pub struct SecondaryReference {
    pub identity: String,
    pub conditions: String,
    pub values: Vec<f32>,
    pub relative_slack: Option<f64>,
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

    fn admission_label(self) -> &'static str {
        match self {
            Self::NativePassed => "原生放行",
            Self::MjwarpPassed => "二级放行",
            Self::Failed => "失败不放行",
            Self::Pending => "待验收不放行",
        }
    }
}

#[derive(Debug)]
pub struct Comparison {
    pub identity: String,
    pub count: usize,
    pub max_absolute: f64,
    pub max_normalized: Option<f64>,
    pub worst_normalized_index: Option<usize>,
    pub worst_index: Option<usize>,
    pub violations: Option<usize>,
    pub tolerance: Option<Tolerance>,
}

impl Comparison {
    fn json(&self) -> Value {
        json!({
            "identity": self.identity, "count": self.count,
            "max_absolute": self.max_absolute, "worst_index": self.worst_index,
            "max_normalized": self.max_normalized,
            "worst_normalized_index": self.worst_normalized_index,
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
    pub conditions: Option<String>,
    pub relative_slack: Option<f64>,
    pub redline: Option<f64>,
}

impl Decision {
    pub fn passed(&self) -> bool {
        matches!(self.status, Status::NativePassed | Status::MjwarpPassed)
    }

    pub fn json(&self) -> Value {
        let accepted_reference = match self.status {
            Status::NativePassed => Some("native_mujoco"),
            Status::MjwarpPassed => Some("mjwarp"),
            Status::Failed | Status::Pending => None,
        };
        // 放行标记仅描述本字段块。
        // 它不证明完整验收或精度改善。
        let label = match self.reason.as_str() {
            "native_error_not_worse_than_mjwarp" => "对照放行",
            "native_error_within_mjwarp_redline" => "红线内放行",
            _ => self.status.admission_label(),
        };
        json!({"policy": "native_error_v2", "status": self.status.name(), "reason": self.reason, "gate": "NP03",
            "np03_admitted": self.passed(), "admission_label": label,
            "accepted_reference": accepted_reference,
            "error_baseline": "native_mujoco", "conditions": self.conditions,
            "relative_slack": self.relative_slack, "redline": self.redline,
            "native": self.native.as_ref().map(Comparison::json),
            "mjwarp": self.mjwarp.as_ref().map(Comparison::json)})
    }
}

fn compare(actual: &[f32], reference: &Reference) -> Result<Comparison, &'static str> {
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
        identity: reference.identity.clone(),
        count: actual.len(),
        max_absolute: 0.0,
        max_normalized: reference.tolerance.map(|_| 0.0),
        worst_normalized_index: None,
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
            let normalized = if limit == 0.0 && error == 0.0 {
                0.0
            } else {
                error / limit
            };
            if result
                .max_normalized
                .is_some_and(|maximum| normalized > maximum)
            {
                result.worst_normalized_index = Some(index);
            }
            result.max_normalized = result
                .max_normalized
                .filter(|_| normalized.is_finite())
                .map(|maximum| maximum.max(normalized));
            if result.max_normalized.is_none() {
                result.worst_normalized_index = None;
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
    mjwarp: impl FnOnce() -> Result<Option<SecondaryReference>, String>,
) -> Decision {
    let mut decision = Decision {
        status: Status::Failed,
        reason: String::new(),
        native: None,
        mjwarp: None,
        conditions: None,
        relative_slack: None,
        redline: None,
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
    match compare(actual, &native) {
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
                    Ok(Some(reference)) => judge_secondary(&mut decision, &native, reference),
                },
            }
        }
    }
    decision
}

fn judge_secondary(decision: &mut Decision, native: &Reference, reference: SecondaryReference) {
    if reference.identity.is_empty() || reference.conditions.is_empty() {
        decision.reason = "missing_mjwarp_identity_or_conditions".into();
        return;
    }
    if let Some(slack) = reference.relative_slack
        && (!slack.is_finite() || slack < 0.0)
    {
        decision.reason = "invalid_mjwarp_redline".into();
        return;
    }
    decision.conditions = Some(reference.conditions);
    decision.relative_slack = reference.relative_slack;
    // 两路误差都相对同一原生数组。
    // 不再检查本库与MJWarp的距离。
    let mut comparison = match compare(&reference.values, native) {
        Ok(value) => value,
        Err(reason) => {
            decision.reason = format!("invalid_mjwarp_reference: {reason}");
            return;
        }
    };
    comparison.identity = reference.identity;
    let upstream_score = comparison.max_normalized;
    decision.mjwarp = Some(comparison);
    let (Some(actual_score), Some(upstream_score)) = (
        decision.native.as_ref().and_then(|c| c.max_normalized),
        upstream_score,
    ) else {
        decision.reason = "undefined_normalized_error".into();
        return;
    };
    if let Some(slack) = reference.relative_slack {
        let redline = upstream_score * (1.0 + slack);
        if !redline.is_finite() {
            decision.reason = "nonfinite_mjwarp_redline".into();
            return;
        }
        decision.redline = Some(redline);
    }
    if actual_score <= upstream_score {
        decision.status = Status::MjwarpPassed;
        decision.reason = "native_error_not_worse_than_mjwarp".into();
    } else if let Some(redline) = decision.redline {
        if actual_score <= redline {
            decision.status = Status::MjwarpPassed;
            decision.reason = "native_error_within_mjwarp_redline".into();
        } else {
            decision.reason = "native_error_exceeds_mjwarp_redline".into();
        }
    } else {
        decision.status = Status::Pending;
        decision.reason = "missing_mjwarp_redline".into();
    }
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
    compared_scalars: usize,
    details: Vec<Value>,
}

impl Audit {
    pub fn new(scope: impl Into<String>) -> Self {
        Self {
            scope: scope.into(),
            counts: [0; 4],
            compared_scalars: 0,
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
        // 空字段允许形状检查。
        // 汇总不能把空字段当数值证据。
        self.compared_scalars += decision.native.as_ref().map_or(0, |c| c.count);
        let maximum = decision.native.as_ref().map_or(0.0, |c| c.max_absolute);
        if decision.status != Status::NativePassed {
            let detail = json!({"context": context, "decision": decision.json()});
            eprintln!("NP03_DETAIL {detail}");
            self.details.push(detail);
        }
        maximum
    }

    pub fn report(&self) -> Value {
        let evaluated_blocks: usize = self.counts.iter().sum();
        let np03_admitted = self.compared_scalars > 0 && self.counts[2] == 0 && self.counts[3] == 0;
        let admission_label = if self.counts[2] > 0 {
            "失败不放行"
        } else if self.counts[3] > 0 {
            "待验收不放行"
        } else if self.compared_scalars == 0 {
            "未验收不放行"
        } else if self.counts[1] > 0 {
            "放行，含二级参考"
        } else {
            "原生放行"
        };
        json!({"schema": 2, "policy": "native_error_v2", "scope": self.scope, "gate": "NP03",
            "coverage": "recorded_blocks_only", "evaluated_blocks": evaluated_blocks,
            "compared_scalars": self.compared_scalars, "np03_admitted": np03_admitted,
            "admission_label": admission_label,
            "native_passed": self.counts[0], "mjwarp_passed": self.counts[1],
            "failed": self.counts[2], "pending": self.counts[3], "details": self.details})
    }

    pub fn finish(self) {
        let report = self.report();
        eprintln!("NP03_SUMMARY {report}");
        assert!(
            report["np03_admitted"] == true,
            "NP03 gate failed: {report}"
        );
    }
}
