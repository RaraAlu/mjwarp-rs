//! 空间速度单元测试。

#[cfg(feature = "cuda-probe")]
use crate::model::AttachedModelInput;
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferSession;

use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn velocity_readback_rejects_guards_and_nonfinite_buffers() {
    let model = super::super::tests::test_model(Some(3));
    let nv = model.kinematics().nv();
    let model = AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
    let session = TransferSession::new(0).unwrap();
    let plan = ComVelocityPlan::new(&session, model).unwrap();
    let mut data = plan.create_data(1).unwrap();
    plan.update(&mut data).unwrap();
    data.readback().unwrap();
    let write = |d: &mut ComVelocityData, which, offset, value| {
        let buffer = match which {
            0 => &mut d.rigid,
            1 => &mut d.com,
            _ => &mut d.output,
        };
        buffer.write_range(offset, &[value]).unwrap();
    };
    for which in 0..3 {
        let length = match which {
            0 => data.rigid.len(),
            1 => data.com.len(),
            _ => data.output.len(),
        };
        for offset in [0, length - 1] {
            write(&mut data, which, offset, 0.0);
            assert!(data.readback().is_err());
            write(&mut data, which, offset, -131072.0);
        }
        write(&mut data, which, 4, f32::NAN);
        assert!(data.readback().is_err());
        plan.update(&mut data).unwrap();
    }
    write(&mut data, 2, 4, f32::INFINITY);
    assert!(data.readback().is_err());
    plan.update(&mut data).unwrap();
    data.qvel.write_range(0, &[0.0]).unwrap();
    assert!(data.readback().is_err());
    data.qvel.write_range(0, &[-131072.0]).unwrap();
    data.qvel.write_range(4, &[f32::NAN]).unwrap();
    assert!(data.readback().is_err());
    data.write_qvel(&vec![0.0; nv]).unwrap();
    plan.update(&mut data).unwrap();
    data.readback().unwrap();
}

#[test]
fn rejects_velocity_layout_dimensions_and_overflow() {
    for (w, b, j, v) in [
        (0, 1, 0, 0),
        (1, 0, 0, 0),
        (u32::MAX as usize, 1, 0, 0),
        (1, usize::MAX, 0, 0),
        (1, 1, usize::MAX, 0),
        (1, 1, 0, usize::MAX),
    ] {
        assert!(ComVelocityLayout::new(w, b, j, v).is_err());
    }
    let layout = ComVelocityLayout::new(513, 1, 0, 0).unwrap();
    assert_eq!(layout.output.total_bytes(), 513 * 6 * size_of::<f32>());
    #[cfg(feature = "cuda-probe")]
    {
        assert_eq!(layout.guarded_rigid, 513 * 28 + 8);
        assert_eq!(layout.guarded_com, 513 * 14 + 8);
    }
}

#[test]
fn checks_velocity_lengths_nonfinite_inputs_and_empty_world_views() {
    assert!(check_qvel(1, &[]).is_err());
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(check_qvel(1, &[x]).is_err());
    }
    assert!(check_qvel(0, &[]).is_ok());
    let layout = ComVelocityLayout::new(2, 1, 0, 0).unwrap();
    let out = ComVelocitySnapshot {
        layout,
        values: vec![0.0; 20],
    };
    assert_eq!(out.worlds(), 2);
    assert_eq!(out.world(1).unwrap().cvel.len(), 6);
    assert!(out.world(1).unwrap().cdof_dot.is_empty());
    assert!(out.world(2).is_err());
}
