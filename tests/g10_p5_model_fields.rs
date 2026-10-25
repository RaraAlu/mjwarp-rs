#![cfg(feature = "native-model-probe")]
//! G1：hfield/octree/`opt.sdf_*`逐字段位级parity。
//! 期望值来自G0b落盘`fixtures/g10-p5/octree-parity.json`（3.12与3.15全等）。
//! 只读冻结夹具；不启动Python、不使用GPU。

use mjwarp_rs::{
    io::NativeModelProbe,
    model::{VolumeModelInput, volume::VolumeFieldData},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const PARITY: &str = include_str!("../fixtures/g10-p5/octree-parity.json");

#[derive(Deserialize)]
struct Parity {
    schema: u32,
    models: BTreeMap<String, ModelParity>,
}

#[derive(Deserialize)]
struct ModelParity {
    counts_312: Counts,
    f64_fields: Vec<Expected>,
    f32_fields: Vec<Expected>,
    struct_fields: Vec<Expected>,
}

#[derive(Deserialize)]
struct Counts {
    nhfield: i64,
    nhfielddata: i64,
    nmesh: i64,
    noct: i64,
}

#[derive(Deserialize)]
struct Expected {
    name: String,
    dtype: String,
    count: u64,
    sha256_312: String,
    equal: bool,
}

fn load(name: &str) -> NativeModelProbe {
    let root = env!("CARGO_MANIFEST_DIR");
    let mjb = std::fs::read(format!("{root}/fixtures/g10-p5/mjb-312/{name}.mjb")).unwrap();
    let dll: std::path::PathBuf = std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into();
    // SAFETY: acceptance supplies a verified MuJoCo 3.12 DLL and the frozen MJB below.
    unsafe { NativeModelProbe::load_trusted(&dll, &mjb) }.unwrap()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn i32_bytes(values: &[i32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn f64_bytes(values: &[f64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn field_bytes(input: &VolumeModelInput, name: &str) -> Vec<u8> {
    match input.field(name).expect(name) {
        VolumeFieldData::Int(values) => i32_bytes(values),
        VolumeFieldData::Num(values) => f64_bytes(values),
        VolumeFieldData::Float(values) => f32_bytes(values),
        VolumeFieldData::Size(values) => values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
    }
}

/// 把本片的14个字段映射到G0b夹具里的字段名。
const EXPECTED_FIELDS: [&str; 12] = [
    "oct_aabb",
    "oct_coeff",
    "hfield_size",
    "hfield_data",
    "oct_depth",
    "oct_child",
    "mesh_octadr",
    "mesh_octnum",
    "hfield_nrow",
    "hfield_ncol",
    "hfield_adr",
    "opt_sdf",
];

fn check_field(input: &VolumeModelInput, expected: &Expected, name: &str) {
    assert!(expected.equal, "{name} 3.12与3.15未全等，夹具不可作期望");
    let bytes = if expected.name == "opt_sdf" {
        // opt_sdf = [sdf_iterations, sdf_initpoints]（原生字段顺序）。
        let mut bytes = i32_bytes(&[input.sdf_iterations().unwrap()]);
        bytes.extend(i32_bytes(&[input.sdf_initpoints().unwrap()]));
        bytes
    } else {
        assert_eq!(expected.name, name, "夹具字段名不一致");
        field_bytes(input, name)
    };
    let dtype = match input.field(name) {
        Some(VolumeFieldData::Num(_)) => "f64",
        Some(VolumeFieldData::Float(_)) => "f32",
        Some(VolumeFieldData::Int(_)) => "i32",
        Some(VolumeFieldData::Size(_)) => "i64",
        None => "missing",
    };
    let actual_dtype = if expected.name == "opt_sdf" {
        "i32"
    } else {
        dtype
    };
    assert_eq!(actual_dtype, expected.dtype, "{name} dtype");
    assert_eq!(
        bytes.len() as u64,
        expected.count
            * if expected.dtype == "f32" || expected.dtype == "i32" {
                4
            } else {
                8
            },
        "{name} 字节数"
    );
    let actual = digest(&bytes);
    assert_eq!(actual, expected.sha256_312, "{name} 位级sha256不一致");
}

fn check_model(name: &str, parity: &ModelParity) {
    let owner = load(name);
    let snapshot = owner.volume_snapshot().unwrap();
    assert_eq!(snapshot.info(), owner.info(), "{name} ABI身份");
    let counts = snapshot.counts();
    assert_eq!(
        counts.nhfield as i64, parity.counts_312.nhfield,
        "{name} nhfield"
    );
    assert_eq!(
        counts.nhfielddata as i64, parity.counts_312.nhfielddata,
        "{name} nhfielddata"
    );
    assert_eq!(counts.nmesh as i64, parity.counts_312.nmesh, "{name} nmesh");
    assert_eq!(counts.noct as i64, parity.counts_312.noct, "{name} noct");
    drop(owner);
    let input = snapshot.into_model().unwrap();
    assert_eq!(
        input.noct() as i64,
        parity.counts_312.noct,
        "{name} noct字段"
    );
    let mut checked = 0usize;
    for expected in parity
        .f64_fields
        .iter()
        .chain(&parity.f32_fields)
        .chain(&parity.struct_fields)
    {
        let Some(&field) = EXPECTED_FIELDS
            .iter()
            .find(|field| **field == expected.name)
        else {
            continue;
        };
        check_field(&input, expected, field);
        checked += 1;
    }
    assert_eq!(checked, EXPECTED_FIELDS.len(), "{name} 夹具字段覆盖");
    check_hfield_values(&input, name);
    check_octree_values(&input, name);
    check_single_conversion(&input, name);
}

/// hfield-box 的值级锚点（G0b夹具sha256已逐位钉住，这里再给可读数值）。
fn check_hfield_values(input: &VolumeModelInput, name: &str) {
    if name != "hfield-box" {
        assert!(input.hfield_data().unwrap().is_empty());
        assert!(input.hfield_size().unwrap().is_empty());
        return;
    }
    let data: Vec<u32> = input
        .hfield_data()
        .unwrap()
        .iter()
        .map(|value| value.to_bits())
        .collect();
    let expected: Vec<u32> = [0.0f32, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0]
        .iter()
        .map(|value| value.to_bits())
        .collect();
    assert_eq!(data, expected, "hfield_data位型");
    let size: Vec<u64> = input
        .hfield_size()
        .unwrap()
        .iter()
        .map(|value| value.to_bits())
        .collect();
    let expected: Vec<u64> = [1.0f64, 1.0, 0.1, 0.1]
        .iter()
        .map(|value| value.to_bits())
        .collect();
    assert_eq!(size, expected, "hfield_size位型");
    assert_eq!(input.ints("hfield_nrow").unwrap(), &[3]);
    assert_eq!(input.ints("hfield_ncol").unwrap(), &[3]);
    assert_eq!(input.ints("hfield_adr").unwrap(), &[0]);
    assert!(input.oct_aabb().unwrap().is_empty());
}

/// sdf-cylinder 的octree形状锚点。
fn check_octree_values(input: &VolumeModelInput, name: &str) {
    if name != "sdf-cylinder" {
        assert!(input.oct_aabb().unwrap().is_empty());
        assert!(input.oct_coeff().unwrap().is_empty());
        return;
    }
    assert_eq!(input.noct(), 63_497);
    assert_eq!(input.ints("oct_depth").unwrap().len(), 63_497);
    assert_eq!(input.ints("oct_child").unwrap().len(), 63_497 * 8);
    assert_eq!(input.oct_aabb().unwrap().len(), 63_497 * 6);
    assert_eq!(input.oct_coeff().unwrap().len(), 63_497 * 8);
    assert_eq!(input.ints("mesh_octadr").unwrap(), &[0]);
    assert_eq!(input.ints("mesh_octnum").unwrap(), &[63_497]);
    assert_eq!(input.noct_i32().unwrap(), 63_497);
}

/// f64→f32只做一次就近舍入：每个元素等于一次f64→f32，且再经f32→f64→f32保持。
fn check_single_conversion(input: &VolumeModelInput, name: &str) {
    let pairs = [
        (input.oct_aabb().unwrap(), input.oct_aabb_f32().unwrap()),
        (input.oct_coeff().unwrap(), input.oct_coeff_f32().unwrap()),
        (
            input.hfield_size().unwrap(),
            input.hfield_size_f32().unwrap(),
        ),
    ];
    for (source, converted) in pairs {
        assert_eq!(source.len(), converted.len(), "{name} 转换长度");
        for (&raw, &once) in source.iter().zip(&converted) {
            assert_eq!(once.to_bits(), (raw as f32).to_bits(), "{name} 一次转换");
            assert_eq!(f64::from(once) as f32, once, "{name} 二次量化改变了位型");
        }
    }
}

#[test]
fn parses_g0b_fixture_coverage() {
    let parity: Parity = serde_json::from_str(PARITY).unwrap();
    assert_eq!(parity.schema, 1);
    for name in ["hfield-box", "sdf-cylinder"] {
        let model = &parity.models[name];
        assert!(model.counts_312.noct >= 0);
        assert!(EXPECTED_FIELDS.iter().all(|field| !field.is_empty()));
    }
}

#[test]
#[ignore = "需要可信MuJoCo DLL"]
fn reads_volume_fields_bitwise_equal_to_g0b_parity() {
    let parity: Parity = serde_json::from_str(PARITY).unwrap();
    for name in ["hfield-box", "sdf-cylinder"] {
        check_model(name, &parity.models[name]);
    }
}
