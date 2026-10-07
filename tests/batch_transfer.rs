#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    diagnostics::{InputError, ProbeError, TransferError},
    io::{DeviceBatch, upload_f64_batch},
    model::BatchLayout,
    runtime::TransferSession,
};

fn session() -> TransferSession {
    if std::env::var_os("MJWARP_TRANSFER_DRIVER_ONLY").as_deref() == Some(std::ffi::OsStr::new("1"))
    {
        // SAFETY: 只检查NVRTC库可见性。
        // 测试不向加载器传入用户指针。
        assert!(!unsafe { cudarc::nvrtc::sys::is_culib_present() });
        println!("driver_only: NVRTC invisible");
    }
    TransferSession::new(0).unwrap()
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn roundtrips_four_types_after_host_and_session_drop() {
    let session = session();
    assert_eq!(session.device(), 0);
    let floats = [
        0x8000_0000,
        0x7fc0_0017,
        0x7f80_0000,
        0xff80_0000,
        1,
        0x3fc0_0000,
    ];
    let mut host = floats.map(f32::from_bits);
    let buffer = session.upload(&host).unwrap();
    host.fill(7.0);
    let signed = session.upload(&[i32::MIN, -1, 0, i32::MAX]).unwrap();
    let unsigned = session.upload(&[0_u32, 16_777_217, u32::MAX]).unwrap();
    let double_bits = [
        0x8000_0000_0000_0000,
        0x7ff8_0000_0000_0017,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        1,
        0x3ff0_0000_0000_0001,
    ];
    let doubles = session.upload(&double_bits.map(f64::from_bits)).unwrap();
    drop(session);
    assert_eq!(buffer.len(), 6);
    assert_eq!(buffer.byte_len(), 24);
    assert!(!buffer.is_empty());
    let mut actual = [0.0; 6];
    buffer.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual.map(f32::to_bits), floats);
    let mut actual = [0; 4];
    signed.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, [i32::MIN, -1, 0, i32::MAX]);
    let mut actual = [0; 3];
    unsigned.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, [0, 16_777_217, u32::MAX]);
    let mut actual = [0.0f64; 6];
    doubles.read_range_into(0, &mut actual).unwrap();
    assert_eq!(doubles.byte_len(), 48);
    assert_eq!(actual.map(f64::to_bits), double_bits);
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn updates_worlds_across_empty_and_multiworld_matrix() {
    let session = session();
    // M07局部字段检查，不验收完整Data。
    for worlds in [1, 3, 64] {
        for elements in [0, 1, 5, 129, 257, 4097] {
            let layout = BatchLayout::new(worlds, elements, 4).unwrap();
            let mut expected: Vec<f32> = (0..layout.total_elements())
                .map(|i| i as f32 * 0.25)
                .collect();
            let mut field = DeviceBatch::upload(&session, layout, &expected).unwrap();
            let update = vec![-7.25; elements];
            field.write_world(worlds - 1, &update).unwrap();
            expected[layout.world_elements(worlds - 1).unwrap()].copy_from_slice(&update);
            field.copy_world_within(0, worlds - 1).unwrap();
            expected[layout.world_elements(0).unwrap()].copy_from_slice(&update);
            field.copy_world_within(worlds - 1, 0).unwrap();
            let mut actual = vec![0.0; expected.len()];
            field.read_into(&mut actual).unwrap();
            assert_eq!(actual, expected, "{worlds}/{elements}");
            let mut world = vec![0.0; elements];
            field.read_world_into(worlds - 1, &mut world).unwrap();
            assert_eq!(world, update);
        }
    }
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn copies_between_independent_fields_and_session_clones() {
    let session = session();
    let clone = session.clone();
    let source_values: Vec<i32> = (-7..8).collect();
    let source =
        DeviceBatch::upload(&session, BatchLayout::new(3, 5, 4).unwrap(), &source_values).unwrap();
    let mut target =
        DeviceBatch::upload(&clone, BatchLayout::new(2, 5, 4).unwrap(), &[99; 10]).unwrap();
    target.copy_world_from(1, &source, 2).unwrap();
    let mut actual = [0; 10];
    target.read_into(&mut actual).unwrap();
    assert_eq!(&actual[..5], &[99; 5]);
    assert_eq!(&actual[5..], &source_values[10..]);
    let mut original = [0; 15];
    source.read_into(&mut original).unwrap();
    assert_eq!(original.as_slice(), source_values);
    let precise = [1.0 + 2.0f64.powi(-40), -1.0, 3.0, 4.0];
    let input =
        DeviceBatch::upload(&session, BatchLayout::new(2, 2, 8).unwrap(), &precise).unwrap();
    let mut output =
        DeviceBatch::upload(&clone, BatchLayout::new(2, 2, 8).unwrap(), &[0.0f64; 4]).unwrap();
    output.copy_world_from(1, &input, 0).unwrap();
    output.copy_world_within(0, 1).unwrap();
    output.write_world(1, &precise[2..]).unwrap();
    let mut actual = [0.0f64; 4];
    output.read_into(&mut actual).unwrap();
    assert_eq!(actual.map(f64::to_bits), precise.map(f64::to_bits));
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn rejects_input_errors_without_host_or_device_writes() {
    let session = session();
    let layout = BatchLayout::new(2, 3, 4).unwrap();
    let mut field = DeviceBatch::upload(&session, layout, &[1_u32, 2, 3, 4, 5, 6]).unwrap();
    assert!(field.write_world(2, &[7; 3]).is_err());
    assert!(field.write_world(1, &[7; 2]).is_err());
    assert!(field.copy_world_within(0, 2).is_err());
    let source =
        DeviceBatch::upload(&session, BatchLayout::new(1, 2, 4).unwrap(), &[7; 2]).unwrap();
    assert!(field.copy_world_from(1, &source, 0).is_err());
    let mut target = [99; 2];
    assert!(field.read_world_into(1, &mut target).is_err());
    assert!(field.read_into(&mut target).is_err());
    assert_eq!(target, [99; 2]);
    let mut actual = [0; 6];
    field.read_into(&mut actual).unwrap();
    assert_eq!(actual, [1, 2, 3, 4, 5, 6]);
    assert!(matches!(
        DeviceBatch::upload(&session, BatchLayout::new(1, 1, 8).unwrap(), &[7_u32]),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "element_bytes"
        }))
    ));
    let mut buffer = session.upload(&[1_u32, 2, 3, 4, 5, 6]).unwrap();
    assert!(buffer.write_range(usize::MAX, &[7]).is_err());
    let mut target = [99; 2];
    assert!(buffer.read_range_into(5, &mut target).is_err());
    assert_eq!(target, [99; 2]);
    assert_eq!(
        buffer.copy_range_within(2, 1, 3),
        Err(TransferError::OverlappingCopy)
    );
    buffer.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, [1, 2, 3, 4, 5, 6]);
    buffer.copy_range_within(0, 0, 6).unwrap();
    buffer.copy_range_within(0, 3, 3).unwrap();
    buffer.copy_range_within(3, 0, 3).unwrap();
    buffer.read_range_into(0, &mut actual).unwrap();
    assert_eq!(actual, [4, 5, 6, 4, 5, 6]);
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn rejects_independent_sessions_before_copying() {
    let first = session();
    let second = session();
    let layout = BatchLayout::new(1, 3, 4).unwrap();
    let source = DeviceBatch::upload(&first, layout, &[7_u32; 3]).unwrap();
    let mut target = DeviceBatch::upload(&second, layout, &[99; 3]).unwrap();
    assert_eq!(
        target.copy_world_from(0, &source, 0),
        Err(TransferError::SessionMismatch)
    );
    let mut actual = [0; 3];
    target.read_into(&mut actual).unwrap();
    assert_eq!(actual, [99; 3]);
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn converts_f64_before_upload_and_rejects_strict_inputs() {
    let session = session();
    let layout = BatchLayout::new(2, 2, 4).unwrap();
    let field = upload_f64_batch(
        &session,
        layout,
        &[1.5, -0.0, 16_777_217.0, f64::MIN_POSITIVE],
    )
    .unwrap();
    let mut actual = [0.0; 4];
    field.read_into(&mut actual).unwrap();
    assert_eq!(actual, [1.5, -0.0, 16_777_216.0, 0.0]);
    assert_eq!(actual[1].to_bits(), (-0.0_f32).to_bits());
    for invalid in [f64::NAN, f64::INFINITY, f64::MAX] {
        assert!(upload_f64_batch(&session, layout, &[1.0, 2.0, 3.0, invalid]).is_err());
    }
    assert!(upload_f64_batch(&session, layout, &[1.0]).is_err());
    field.read_into(&mut actual).unwrap();
    assert_eq!(actual, [1.5, -0.0, 16_777_216.0, 0.0]);
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn handles_zero_length_fields_and_endpoint_ranges() {
    let session = session();
    let mut buffer = session.upload::<u32>(&[]).unwrap();
    assert!(buffer.is_empty());
    assert_eq!(buffer.byte_len(), 0);
    buffer.read_range_into(0, &mut []).unwrap();
    buffer.write_range(0, &[]).unwrap();
    buffer.copy_range_within(0, 0, 0).unwrap();
    assert!(buffer.write_range(1, &[]).is_err());
    let mut field =
        DeviceBatch::upload(&session, BatchLayout::new(2, 0, 4).unwrap(), &[] as &[f32]).unwrap();
    field.read_world_into(1, &mut []).unwrap();
    field.write_world(1, &[]).unwrap();
    field.copy_world_within(0, 1).unwrap();
    assert!(field.read_world_into(2, &mut []).is_err());
    let mut buffer = session.upload(&[1_u32, 2, 3]).unwrap();
    buffer.read_range_into(3, &mut []).unwrap();
    buffer.write_range(3, &[]).unwrap();
    buffer.copy_range_within(3, 0, 0).unwrap();
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn transfers_buffer_ownership_to_another_thread() {
    let session = session();
    let buffer = session.upload(&[1_u32, 2, 3, 4]).unwrap();
    drop(session);
    std::thread::spawn(move || {
        let mut actual = [0; 4];
        buffer.read_range_into(0, &mut actual).unwrap();
        assert_eq!(actual, [1, 2, 3, 4]);
    })
    .join()
    .unwrap();
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn serializes_copies_from_concurrent_session_clones() {
    let session = session();
    let mut threads = Vec::new();
    for index in 0..4_u32 {
        let session = session.clone();
        threads.push(std::thread::spawn(move || {
            let mut buffer = session.upload(&[index; 257]).unwrap();
            buffer.write_range(128, &[index + 4; 129]).unwrap();
            let mut actual = [0; 257];
            buffer.read_range_into(0, &mut actual).unwrap();
            assert_eq!(&actual[..128], &[index; 128]);
            assert_eq!(&actual[128..], &[index + 4; 129]);
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
#[ignore = "需要NVIDIA驱动；不回退CPU"]
fn rejects_invalid_device_without_cpu_fallback() {
    assert!(matches!(
        TransferSession::new(usize::MAX),
        Err(TransferError::Backend(ProbeError::InvalidDevice { .. }))
    ));
}
