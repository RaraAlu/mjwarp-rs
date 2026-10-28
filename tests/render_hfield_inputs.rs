//! R05 前置 channel 适配器单测（方案 A，宿主）：`VolumeModelInput` → `VolumeHfieldInputs`
//! → `HfieldInputs` → `resolve_hfield` 全链路。
//!
//! 只验证**输入一致性**（与 G10 hfield 夹具同源数组、f64→f32 唯一转换点、不复制 data、
//! 单份无 world 维度）；**不作任何渲染数值结论**，无 GPU、无窗口。

use mjwarp_rs::model::volume::num_to_f32;
use mjwarp_rs::model::{VolumeFieldData, VolumeFields, VolumeModelCounts, VolumeModelInput};
use mjwarp_rs::render::HfieldError;
use mjwarp_rs::render::context::params::hfield_inputs::VolumeHfieldInputs;

/// `VOLUME_FIELDS` 顺序（14 项）：hfield 5 项 + noct/oct 4 项 + mesh 2 项 + opt 2 项。
fn empty_volume_data() -> Vec<VolumeFieldData> {
    vec![
        VolumeFieldData::Int(vec![]),   // hfield_nrow
        VolumeFieldData::Int(vec![]),   // hfield_ncol
        VolumeFieldData::Num(vec![]),   // hfield_size
        VolumeFieldData::Int(vec![]),   // hfield_adr
        VolumeFieldData::Float(vec![]), // hfield_data
        VolumeFieldData::Size(vec![0]), // noct
        VolumeFieldData::Int(vec![]),   // oct_depth
        VolumeFieldData::Int(vec![]),   // oct_child
        VolumeFieldData::Num(vec![]),   // oct_aabb
        VolumeFieldData::Num(vec![]),   // oct_coeff
        VolumeFieldData::Int(vec![0]),  // mesh_octadr（nmesh=1）
        VolumeFieldData::Int(vec![0]),  // mesh_octnum（nmesh=1）
        VolumeFieldData::Int(vec![10]), // opt_sdf_iterations
        VolumeFieldData::Int(vec![40]), // opt_sdf_initpoints
    ]
}

fn counts(nhfield: usize, nhfielddata: usize) -> VolumeModelCounts {
    VolumeModelCounts {
        nhfield,
        nhfielddata,
        noct: 0,
        nmesh: 1,
    }
}

/// 3×3 平缓 hfield（与 G10 `hfield-box` 夹具同源数组）。
fn flat_volume(nhfield: usize) -> VolumeModelInput {
    let nrow = vec![3i32];
    let ncol = vec![3i32];
    let size = vec![1.0f64, 1.0, 0.1, 0.1];
    let adr = vec![0i32];
    let data = vec![0.0f32, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0];
    let mut fields = empty_volume_data();
    fields[0] = VolumeFieldData::Int(nrow);
    fields[1] = VolumeFieldData::Int(ncol);
    fields[2] = VolumeFieldData::Num(size);
    fields[3] = VolumeFieldData::Int(adr);
    fields[4] = VolumeFieldData::Float(data);
    VolumeModelInput::new(counts(nhfield, 9), VolumeFields::new(fields)).expect("合法体积输入")
}

#[test]
fn adapts_volume_hfield_into_inputs() {
    let model = flat_volume(1);
    let adapter = VolumeHfieldInputs::from_volume(&model)
        .expect("适配不应报错")
        .expect("nhfield=1 应返回适配结果");
    let inputs = adapter.as_inputs();
    assert_eq!(inputs.size, Some(&[1.0f32, 1.0, 0.1, 0.1][..]));
    assert_eq!(inputs.nrow, Some(&[3i32][..]));
    assert_eq!(inputs.ncol, Some(&[3i32][..]));
    assert_eq!(inputs.adr, Some(&[0i32][..]));
    assert_eq!(
        inputs.data,
        Some(&[0.0f32, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0][..])
    );
}

#[test]
fn resolve_full_chain_matches_s1_views() {
    let model = flat_volume(1);
    let adapter = VolumeHfieldInputs::from_volume(&model).unwrap().unwrap();
    let params = adapter
        .resolve()
        .expect("全链路不应报错")
        .expect("应返回 hfield 视图");
    assert_eq!(params.len(), 1);
    let view = &params.hfields()[0];
    assert_eq!(view.size(), [1.0, 1.0, 0.1, 0.1]);
    assert_eq!(view.nrow(), 3);
    assert_eq!(view.ncol(), 3);
    assert_eq!(view.adr(), 0);
    assert_eq!(view.span(), 9);
    assert_eq!(
        params.data_of(0).unwrap(),
        &[0.0f32, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0]
    );
}

#[test]
fn no_hfield_returns_none() {
    let model = VolumeModelInput::new(counts(0, 0), VolumeFields::new(empty_volume_data()))
        .expect("无 hfield 的合法体积输入");
    assert_eq!(VolumeHfieldInputs::from_volume(&model).unwrap(), None);
}

#[test]
fn g10_fixture_arrays_match_volume_adapter() {
    let path = "fixtures/g10-p5/hfield-f32/hfield-box-cpu.json";
    let text = std::fs::read_to_string(path).expect("读取 G10 hfield 夹具");
    let value: serde_json::Value = serde_json::from_str(&text).expect("解析夹具 JSON");
    let arrays = &value["model"]["arrays"];
    let fixture_size: Vec<f64> = arrays["hfield_size"]
        .as_array()
        .expect("hfield_size 数组")
        .iter()
        .map(|item| item.as_f64().expect("f64"))
        .collect();
    let fixture_nrow: Vec<i32> = arrays["hfield_nrow"]
        .as_array()
        .expect("hfield_nrow 数组")
        .iter()
        .map(|item| item.as_i64().expect("i64") as i32)
        .collect();
    let fixture_ncol: Vec<i32> = arrays["hfield_ncol"]
        .as_array()
        .expect("hfield_ncol 数组")
        .iter()
        .map(|item| item.as_i64().expect("i64") as i32)
        .collect();
    let fixture_adr: Vec<i32> = arrays["hfield_adr"]
        .as_array()
        .expect("hfield_adr 数组")
        .iter()
        .map(|item| item.as_i64().expect("i64") as i32)
        .collect();
    let fixture_data: Vec<f32> = arrays["hfield_data"]
        .as_array()
        .expect("hfield_data 数组")
        .iter()
        .map(|item| item.as_f64().expect("f64") as f32)
        .collect();
    // 用夹具数组构造体积输入（其余字段取空/默认）。
    let mut fields = empty_volume_data();
    fields[0] = VolumeFieldData::Int(fixture_nrow.clone());
    fields[1] = VolumeFieldData::Int(fixture_ncol.clone());
    fields[2] = VolumeFieldData::Num(fixture_size.clone());
    fields[3] = VolumeFieldData::Int(fixture_adr.clone());
    fields[4] = VolumeFieldData::Float(fixture_data.clone());
    let model = VolumeModelInput::new(
        counts(fixture_nrow.len(), fixture_data.len()),
        VolumeFields::new(fields),
    )
    .expect("夹具数组应构成合法体积输入");
    let adapter = VolumeHfieldInputs::from_volume(&model).unwrap().unwrap();
    let inputs = adapter.as_inputs();
    // 输入一致性：size 经唯一 f64→f32 转换点；nrow/ncol/adr/data 与夹具同源数组逐位一致。
    assert_eq!(inputs.size.unwrap(), num_to_f32(&fixture_size));
    assert_eq!(inputs.nrow.unwrap(), fixture_nrow);
    assert_eq!(inputs.ncol.unwrap(), fixture_ncol);
    assert_eq!(inputs.adr.unwrap(), fixture_adr);
    assert_eq!(inputs.data.unwrap(), fixture_data);
    let params = adapter.resolve().unwrap().unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params.hfields()[0].span(), 3 * 3);
}

#[test]
fn size_conversion_is_single_f64_to_f32() {
    // 唯一转换点：f64 0.1 → f32 位型 0x3DCC_CCCD（与 model::volume::num_to_f32 / G10 一致）。
    let model = flat_volume(1);
    let adapter = VolumeHfieldInputs::from_volume(&model).unwrap().unwrap();
    let size = adapter.as_inputs().size.unwrap();
    assert_eq!(size[2].to_bits(), 0x3DCC_CCCD);
    assert_eq!(size[3].to_bits(), 0x3DCC_CCCD);
    assert_eq!(size, num_to_f32(&[1.0, 1.0, 0.1, 0.1]).as_slice());
}

#[test]
fn boundary_errors_are_rejected_by_volume_construction() {
    // 范围错误在 `VolumeModelInput::new` 即被拒（适配器不会见到非法数据）。
    let mut fields = empty_volume_data();
    fields[0] = VolumeFieldData::Int(vec![3]);
    fields[1] = VolumeFieldData::Int(vec![3]);
    fields[2] = VolumeFieldData::Num(vec![1.0, 1.0, 0.1, 0.1]);
    fields[3] = VolumeFieldData::Int(vec![1]); // adr=1，data 只有 9 个 → end=10 > 9
    fields[4] = VolumeFieldData::Float(vec![0.0; 9]);
    assert!(VolumeModelInput::new(counts(1, 9), VolumeFields::new(fields)).is_err());
    let mut zero_rows = empty_volume_data();
    zero_rows[0] = VolumeFieldData::Int(vec![0]);
    zero_rows[1] = VolumeFieldData::Int(vec![3]);
    zero_rows[2] = VolumeFieldData::Num(vec![1.0, 1.0, 0.1, 0.1]);
    zero_rows[3] = VolumeFieldData::Int(vec![0]);
    zero_rows[4] = VolumeFieldData::Float(vec![0.0; 9]);
    // 语义边界：体积通道只拒绝负数（`nrow/ncol/adr >= 0` 口径），`nrow=0` 可构造；
    // S1 校验层对 `<= 0` 拒绝——两层口径差异在此显式验证，不做静默改写。
    let model = VolumeModelInput::new(counts(1, 9), VolumeFields::new(zero_rows))
        .expect("体积层允许 nrow=0（仅拒绝负数）");
    let adapter = VolumeHfieldInputs::from_volume(&model).unwrap().unwrap();
    assert!(matches!(
        adapter.resolve(),
        Err(HfieldError::NonPositiveRowsCols {
            index: 0,
            nrow: 0,
            ncol: 3,
        })
    ));
}

#[test]
fn adapter_is_stable_and_copy_free() {
    let model = flat_volume(1);
    let first = VolumeHfieldInputs::from_volume(&model).unwrap().unwrap();
    let second = VolumeHfieldInputs::from_volume(&model).unwrap().unwrap();
    assert_eq!(first, second, "两次适配结果应逐字段相同");
    // 未复制：适配结果直接借用体积输入的高程数据缓冲。
    assert!(std::ptr::eq(
        first.as_inputs().data.unwrap().as_ptr(),
        model.hfield_data().unwrap().as_ptr()
    ));
    // 单份：适配接口无 `nworld` 维度（结构性），视图数恒等于 nhfield。
    assert_eq!(first.resolve().unwrap().unwrap().len(), 1);
}
