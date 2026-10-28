//! R05-S1 地形 hfield 静态参数校验与只读视图的宿主单测（10 条）。
//!
//! 输入口径（裁决 A，2026-10-28）：`RenderFields`（116 项）没有 hfield 通道且
//! `src/model/render.rs` 是 500 行红线文件，故本片用独立只读输入 [`HfieldInputs`]；
//! 与卡面原签名 `resolve_hfield(&RenderFields)` 的语义等价清单见证据 README。
//!
//! 单份口径：`HfieldInputs` 不含 `nworld`，解析结果不随 world 数复制（R10 单份口径）。
//! 全部用例都在宿主执行，无 GPU、无窗口、无渲染数值结论。

use mjwarp_rs::render::{HfieldError, HfieldInputs, resolve_hfield};

/// 合法两 hfield 的输入：5×5（25 点）与 3×4（12 点），data 共 37。
fn two_valid_inputs<'a>(
    size: &'a [f32],
    nrow: &'a [i32],
    ncol: &'a [i32],
    adr: &'a [i32],
    data: &'a [f32],
) -> HfieldInputs<'a> {
    HfieldInputs {
        size: Some(size),
        nrow: Some(nrow),
        ncol: Some(ncol),
        adr: Some(adr),
        data: Some(data),
    }
}

/// 合法两 hfield 的数据束：`size`（8 个 = 4×2）、`nrow`、`ncol`、`adr`、`data`（37 点）。
type ValidData = (Vec<f32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<f32>);

fn two_valid_data() -> ValidData {
    let size = vec![1.0f32, 2.0, 0.1, 0.2, 3.0, 4.0, 0.5, 0.6];
    let nrow = vec![5i32, 3];
    let ncol = vec![5i32, 4];
    let adr = vec![0i32, 25];
    let data: Vec<f32> = (0..37).map(|index| index as f32).collect();
    (size, nrow, ncol, adr, data)
}

#[test]
fn accepts_two_valid_hfields() {
    let (size, nrow, ncol, adr, data) = two_valid_data();
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &adr, &data);
    let params = resolve_hfield(&inputs)
        .expect("合法两 hfield 不应报错")
        .expect("nhfield=2 应返回视图");
    assert_eq!(params.len(), 2);
    assert!(!params.is_empty());
    assert_eq!(params.hfields[0].size, [1.0, 2.0, 0.1, 0.2]);
    assert_eq!(params.hfields[0].nrow, 5);
    assert_eq!(params.hfields[0].ncol, 5);
    assert_eq!(params.hfields[0].adr, 0);
    assert_eq!(params.hfields[0].span, 25);
    assert_eq!(params.hfields[1].size, [3.0, 4.0, 0.5, 0.6]);
    assert_eq!(params.hfields[1].nrow, 3);
    assert_eq!(params.hfields[1].ncol, 4);
    assert_eq!(params.hfields[1].adr, 25);
    assert_eq!(params.hfields[1].span, 12);
    assert_eq!(params.data_of(0).expect("第 0 段"), &data[0..25]);
    assert_eq!(params.data_of(1).expect("第 1 段"), &data[25..37]);
    assert!(params.data_of(2).is_none(), "越界下标不应给数据");
}

#[test]
fn no_hfield_is_legal_and_returns_none() {
    let inputs = HfieldInputs::default();
    assert!(!inputs.any_present());
    assert!(!inputs.all_present());
    assert_eq!(inputs.first_missing(), Some("hfield_size"));
    assert_eq!(resolve_hfield(&inputs).expect("全缺失合法"), None);
}

#[test]
fn partial_fields_report_first_missing_field() {
    let (size, nrow, ncol, adr, data) = two_valid_data();
    // 缺 hfield_ncol：其余四项都在。
    let missing_ncol = HfieldInputs {
        size: Some(&size),
        nrow: Some(&nrow),
        ncol: None,
        adr: Some(&adr),
        data: Some(&data),
    };
    assert!(missing_ncol.any_present());
    assert!(!missing_ncol.all_present());
    assert_eq!(
        resolve_hfield(&missing_ncol),
        Err(HfieldError::MissingField {
            field: "hfield_ncol"
        })
    );
    // 缺 hfield_data。
    let missing_data = HfieldInputs {
        size: Some(&size),
        nrow: Some(&nrow),
        ncol: Some(&ncol),
        adr: Some(&adr),
        data: None,
    };
    assert_eq!(
        resolve_hfield(&missing_data),
        Err(HfieldError::MissingField {
            field: "hfield_data"
        })
    );
}

#[test]
fn size_shape_mismatch_is_rejected() {
    // 原卡「类型错配」在 HfieldInputs 里由形状/长度承担：size.len() 必须 == 4 × nhfield。
    let (_, nrow, ncol, adr, data) = two_valid_data();
    let bad_size = vec![1.0f32; 4]; // 2 个 hfield 需要 8 个
    let inputs = two_valid_inputs(&bad_size, &nrow, &ncol, &adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::LengthMismatch {
            field: "hfield_size",
            expected: 8,
            actual: 4,
        })
    );
}

#[test]
fn row_col_adr_lengths_must_match() {
    let (size, nrow, ncol, adr, data) = two_valid_data();
    // ncol 比 nrow 短。
    let short_ncol = &ncol[..1];
    let inputs = two_valid_inputs(&size, &nrow, short_ncol, &adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::LengthMismatch {
            field: "hfield_ncol",
            expected: 2,
            actual: 1,
        })
    );
    // adr 比 nrow 长。
    let long_adr = vec![0i32, 25, 37];
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &long_adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::LengthMismatch {
            field: "hfield_adr",
            expected: 2,
            actual: 3,
        })
    );
}

#[test]
fn adr_out_of_range_is_reported_with_index() {
    let (size, nrow, ncol, _, data) = two_valid_data();
    // 第 1 个 hfield：adr=26，span=12，end=38 > data.len()=37。
    let bad_adr = vec![0i32, 26];
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &bad_adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::DataRange {
            index: 1,
            field: "hfield_data",
            start: 26,
            end: 38,
            actual: 37,
        })
    );
    // 负 adr。
    let negative_adr = vec![0i32, -3];
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &negative_adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::DataRange {
            index: 1,
            field: "hfield_adr",
            start: -3,
            end: 0,
            actual: 37,
        })
    );
}

#[test]
fn non_positive_rows_or_cols_are_rejected() {
    let (size, _, _, adr, data) = two_valid_data();
    // nrow = 0。
    let zero_row = vec![0i32, 3];
    let ncol = vec![5i32, 4];
    let inputs = two_valid_inputs(&size, &zero_row, &ncol, &adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::NonPositiveRowsCols {
            index: 0,
            nrow: 0,
            ncol: 5,
        })
    );
    // ncol = -2。
    let nrow = vec![5i32, 3];
    let negative_col = vec![5i32, -2];
    let inputs = two_valid_inputs(&size, &nrow, &negative_col, &adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::NonPositiveRowsCols {
            index: 1,
            nrow: 3,
            ncol: -2,
        })
    );
}

#[test]
fn extreme_span_does_not_panic_and_reports_range() {
    // i32::MAX × i32::MAX ≈ 4.61e18 在 64 位 usize 上不溢出，但明显超出 data 长度；
    // 这里验证 checked 运算与界值检查不 panic、且给出可读 DataRange（含 index/期望/实际）。
    let size = vec![1.0f32; 4];
    let nrow = vec![i32::MAX];
    let ncol = vec![i32::MAX];
    let adr = vec![0i32];
    let data = vec![0.0f32; 1];
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::DataRange {
            index: 0,
            field: "hfield_data",
            start: 0,
            end: (i32::MAX as usize) * (i32::MAX as usize),
            actual: 1,
        })
    );
    // 极端 adr（i32::MAX）同样不 panic。
    let nrow = vec![1i32];
    let ncol = vec![1i32];
    let adr = vec![i32::MAX];
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &adr, &data);
    assert_eq!(
        resolve_hfield(&inputs),
        Err(HfieldError::DataRange {
            index: 0,
            field: "hfield_data",
            start: i64::from(i32::MAX),
            end: (i32::MAX as usize) + 1,
            actual: 1,
        })
    );
}

#[test]
fn empty_field_set_is_legal_and_returns_none() {
    // 全部 Some 但空切片 = nhfield=0：合法（等价于无 hfield），不构造空视图。
    let empty_size: [f32; 0] = [];
    let empty_i32: [i32; 0] = [];
    let empty_f32: [f32; 0] = [];
    let inputs = HfieldInputs {
        size: Some(&empty_size),
        nrow: Some(&empty_i32),
        ncol: Some(&empty_i32),
        adr: Some(&empty_i32),
        data: Some(&empty_f32),
    };
    assert!(inputs.any_present() && inputs.all_present());
    assert_eq!(resolve_hfield(&inputs).expect("空字段集合法的"), None);
}

#[test]
fn resolution_is_single_copy_and_stable() {
    // 单份口径：HfieldInputs 不含 nworld；视图数恒等于 nhfield（不按 world 复制），
    // 同一输入重复解析逐字段相同（HfieldParams 派生 PartialEq）。
    let (size, nrow, ncol, adr, data) = two_valid_data();
    let inputs = two_valid_inputs(&size, &nrow, &ncol, &adr, &data);
    let first = resolve_hfield(&inputs)
        .expect("合法输入不应报错")
        .expect("应返回视图");
    let second = resolve_hfield(&inputs)
        .expect("合法输入不应报错")
        .expect("应返回视图");
    assert_eq!(first, second);
    assert_eq!(first.hfields.len(), nrow.len());
    assert_eq!(first.data.len(), data.len());
    // 数据区段总和不随任何 world 数放大（视图 span 之和 = 25 + 12 = 37）。
    let total: usize = first.hfields.iter().map(|view| view.span).sum();
    assert_eq!(total, 37);
}
