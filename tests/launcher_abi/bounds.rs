//! 启动边界与PTX首参数冻结测试。

use mjwarp_rs::runtime::launcher::{LaunchBounds1, LaunchBounds2, LaunchBounds3, WarpArray};

#[path = "../support/launcher_bounds_evidence.rs"]
mod bounds_evidence;
use bounds_evidence::{PtxParam, SAMPLES};

fn bounds_layout(dims: u8) -> (u64, u64) {
    use std::mem::{align_of, size_of};
    match dims {
        1 => (
            align_of::<LaunchBounds1>() as u64,
            size_of::<LaunchBounds1>() as u64,
        ),
        2 => (
            align_of::<LaunchBounds2>() as u64,
            size_of::<LaunchBounds2>() as u64,
        ),
        3 => (
            align_of::<LaunchBounds3>() as u64,
            size_of::<LaunchBounds3>() as u64,
        ),
        _ => panic!("unsupported launch bounds dims {dims}"),
    }
}

#[test]
fn frozen_launch_bounds_match_ptx_first_parameter() {
    let one_dimensional = SAMPLES.iter().filter(|s| s.dims == 1).count();
    assert_eq!(one_dimensional, 3, "1维样本数量不得为零或漂移");
    for sample in SAMPLES {
        // 摘要在cuda-probe下另行核对。
        for digest in [sample.cu_sha256, sample.ptx_sha256] {
            assert!(digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()));
        }
        let cu = String::from_utf8(bounds_evidence::read(sample.cu)).unwrap();
        let ptx = String::from_utf8(bounds_evidence::read(sample.ptx)).unwrap();
        assert_eq!(
            bounds_evidence::cu_bounds_dims(&cu, sample.entry),
            Some(sample.dims),
            "{}",
            sample.entry
        );
        let params = bounds_evidence::ptx_params(&ptx, sample.entry)
            .unwrap_or_else(|| panic!("{}: 无法解析PTX参数", sample.entry));
        let (align, bytes) = bounds_layout(sample.dims);
        assert_eq!(
            params[0],
            PtxParam::Bytes { align, bytes },
            "{}",
            sample.entry
        );
        // 其余按字节声明的参数均为56字节Warp数组。
        let array = std::mem::size_of::<WarpArray<f32>>() as u64;
        for param in &params[1..] {
            if let PtxParam::Bytes { align, bytes } = *param {
                assert_eq!((align, bytes), (8, array), "{}", sample.entry);
            }
        }
    }
    // 重力入口回执登记11个数组参数。
    let ptx = String::from_utf8(bounds_evidence::read(SAMPLES[4].ptx)).unwrap();
    let gravity = bounds_evidence::ptx_params(&ptx, SAMPLES[4].entry).unwrap();
    let arrays = gravity
        .iter()
        .filter(|p| matches!(p, PtxParam::Bytes { bytes: 56, .. }))
        .count();
    assert_eq!(arrays, 11);
}

#[cfg(feature = "cuda-probe")]
#[test]
fn frozen_bounds_samples_keep_recorded_digests() {
    use sha2::{Digest, Sha256};
    let digest = |bytes: Vec<u8>| format!("{:x}", Sha256::digest(bytes));
    for sample in SAMPLES {
        assert_eq!(digest(bounds_evidence::read(sample.cu)), sample.cu_sha256);
        assert_eq!(digest(bounds_evidence::read(sample.ptx)), sample.ptx_sha256);
    }
    let receipt = String::from_utf8(bounds_evidence::read(
        "fixtures/gravity-upstream-abi/generated/receipt.json",
    ))
    .unwrap();
    assert!(receipt.contains(SAMPLES[0].cu_sha256));
    assert!(receipt.contains(SAMPLES[0].ptx_sha256));
}
