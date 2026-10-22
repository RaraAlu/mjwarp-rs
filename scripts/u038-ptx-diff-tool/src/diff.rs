//! 逐表达式对照：融合计数、指令种类序列、差异分类。

use crate::ptx::{Counts, Entry, Kind};

#[derive(Clone, Debug)]
pub struct Comparison {
    pub diff_type: &'static str,
    pub upstream_counts: Counts,
    pub product_counts: Counts,
    pub upstream_kinds: Vec<Kind>,
    pub product_kinds: Vec<Kind>,
    pub notes: Vec<String>,
}

pub fn counts(entry: &Entry) -> Counts {
    let arith = entry.arith();
    Counts::of(&arith)
}

pub fn kinds(entry: &Entry) -> Vec<Kind> {
    entry.kinds()
}

/// 分类规则（保守）：
/// - 各算子计数完全相同 => 一致（指令调度位置可不相同，收缩量一致）；
/// - 否则若“未融合乘法数 / 未融合加法数 / 其他算子数”两侧相同 => 仅收缩；
/// - 否则 => 顺序或代数不同（需人工复核，附计数）。
pub fn compare(upstream: &Entry, product: &Entry) -> Comparison {
    let up_kinds = kinds(upstream);
    let prod_kinds = kinds(product);
    let up_counts = Counts::of(&upstream.arith());
    let prod_counts = Counts::of(&product.arith());
    let mut notes = Vec::new();
    let same_counts = up_counts == prod_counts;
    let same_unfused = up_counts.unfused_mul() == prod_counts.unfused_mul()
        && up_counts.unfused_add() == prod_counts.unfused_add()
        && up_counts.sqrt == prod_counts.sqrt
        && up_counts.rcp == prod_counts.rcp
        && up_counts.div == prod_counts.div
        && up_counts.neg == prod_counts.neg;
    let diff_type = if same_counts {
        "一致"
    } else if same_unfused {
        "仅收缩"
    } else {
        "顺序或代数不同"
    };
    if same_counts && up_kinds != prod_kinds {
        notes.push("counts identical; only non-fused instruction scheduling differs".to_string());
    }
    if !same_unfused {
        notes.push(format!(
            "unfused totals differ: mul {} vs {}, add {} vs {}, sqrt {} vs {}, rcp {} vs {}, neg {} vs {}",
            up_counts.unfused_mul(),
            prod_counts.unfused_mul(),
            up_counts.unfused_add(),
            prod_counts.unfused_add(),
            up_counts.sqrt,
            prod_counts.sqrt,
            up_counts.rcp,
            prod_counts.rcp,
            up_counts.neg,
            prod_counts.neg
        ));
    }
    if !same_counts && same_unfused {
        notes.push(format!(
            "fusion placement differs: fma {} vs {}",
            up_counts.fma, prod_counts.fma
        ));
    }
    Comparison {
        diff_type,
        upstream_counts: up_counts,
        product_counts: prod_counts,
        upstream_kinds: up_kinds,
        product_kinds: prod_kinds,
        notes,
    }
}

/// 在完整内核的 f32 算术种类序列里查找探针序列的连续出现位置。
pub fn occurrences(kernel: &[Kind], probe: &[Kind]) -> Vec<usize> {
    if probe.is_empty() || probe.len() > kernel.len() {
        return Vec::new();
    }
    let mut found = Vec::new();
    let mut index = 0usize;
    while index + probe.len() <= kernel.len() {
        if kernel[index..index + probe.len()] == *probe {
            found.push(index);
            index += probe.len();
        } else {
            index += 1;
        }
    }
    found
}

/// 指令种类序列的可读形式。
pub fn kind_sequence(kinds: &[Kind]) -> String {
    kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(",")
}
