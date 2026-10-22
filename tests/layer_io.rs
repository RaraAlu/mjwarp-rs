//! I1：数据交互层安全面的宿主与设备验证。
//! 宿主用例核对布局预检与转换；设备用例核对上传、回读、视图、
//! 完成回执、局部世界更新与错误传播。
//! 数据交互层不发射内核，不含`step`与`forward`调度。

use mjwarp_rs::diagnostics::InputError;
use mjwarp_rs::io::{
    StateReceipt, WorldView, WorldViewMut, convert_f64_to_f32_into, plan_field_elements,
    plan_world_range,
};
use mjwarp_rs::model::BatchLayout;

#[test]
fn plans_world_ranges_and_field_elements() {
    let layout = BatchLayout::new(3, 4, 4).unwrap();
    plan_field_elements(layout, 12).unwrap();
    assert!(matches!(
        plan_field_elements(layout, 11),
        Err(InputError::LengthMismatch {
            field: "field_elements",
            expected: 12,
            actual: 11
        })
    ));
    assert_eq!(plan_world_range(layout, 2, 4).unwrap(), 8..12);
    assert!(matches!(
        plan_world_range(layout, 3, 4),
        Err(InputError::InvalidIndex { field: "world", .. })
    ));
    assert!(matches!(
        plan_world_range(layout, 0, 3),
        Err(InputError::LengthMismatch {
            field: "world_elements",
            expected: 4,
            actual: 3
        })
    ));
}

#[test]
fn plans_empty_worlds_without_device_access() {
    let layout = BatchLayout::new(2, 0, 4).unwrap();
    plan_field_elements(layout, 0).unwrap();
    assert_eq!(plan_world_range(layout, 1, 0).unwrap(), 0..0);
    assert!(plan_world_range(layout, 2, 0).is_err());
}

#[test]
fn host_conversions_still_reject_invalid_input() {
    let mut target = [7.0; 3];
    assert!(matches!(
        convert_f64_to_f32_into(&[1.0, f64::NAN, 3.0], &mut target),
        Err(InputError::NonFinite {
            field: "source",
            index: 1
        })
    ));
    assert_eq!(target, [7.0; 3]);
    let mut overflow_target = [7.0; 2];
    assert!(matches!(
        convert_f64_to_f32_into(&[1.0, f64::MAX], &mut overflow_target),
        Err(InputError::ScalarOverflow {
            field: "source",
            index: 1
        })
    ));
    assert_eq!(overflow_target, [7.0; 2]);
    assert!(matches!(
        convert_f64_to_f32_into(&[1.0], &mut target),
        Err(InputError::LengthMismatch {
            field: "target",
            expected: 1,
            actual: 3
        })
    ));
}

#[test]
fn public_types_keep_their_lifetime_contract() {
    fn names() -> [&'static str; 3] {
        [
            std::any::type_name::<StateReceipt>(),
            std::any::type_name::<WorldView<'static, f32>>(),
            std::any::type_name::<WorldViewMut<'static, f32>>(),
        ]
    }
    let names = names();
    assert!(names[0].contains("StateReceipt"));
    assert!(names[1].contains("WorldView"));
    assert!(names[2].contains("WorldViewMut"));
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use mjwarp_rs::diagnostics::TransferError;
    use mjwarp_rs::io::DataExchange;
    use mjwarp_rs::model::{KinematicFields, KinematicModelInput};
    use mjwarp_rs::runtime::TransferSession;

    fn new_exchange() -> DataExchange {
        DataExchange::new(TransferSession::new(0).unwrap())
    }

    fn model_input() -> KinematicModelInput {
        let fields = KinematicFields {
            qpos0: vec![0.0],
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, 0],
            body_jntnum: vec![0, 1],
            body_pos: vec![0.0, 0.0, 1.0, 0.0, 0.0, 2.0],
            body_quat: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            jnt_type: vec![3],
            jnt_bodyid: vec![1],
            jnt_qposadr: vec![0],
            jnt_dofadr: vec![0],
            jnt_pos: vec![0.0, 0.0, 0.0],
            jnt_axis: vec![0.0, 0.0, 1.0],
        };
        KinematicModelInput::new(1, fields).unwrap()
    }

    #[test]
    #[ignore = "requires an NVIDIA GPU"]
    fn uploads_reads_and_views_through_execution_layer() {
        let mut exchange = new_exchange();
        assert_eq!(exchange.device(), 0);
        let layout = BatchLayout::new(2, 3, 8).unwrap();
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut field = exchange.upload(layout, &values).unwrap();
        assert_eq!(exchange.read_back(&field).unwrap(), values);
        // 只读视图。
        let view = exchange.world_view(&field, 1).unwrap();
        assert_eq!(view.world(), 1);
        assert_eq!(view.range(), 3..6);
        let mut world = [0.0; 3];
        view.read_into(&mut world).unwrap();
        assert_eq!(world, [4.0, 5.0, 6.0]);
        let _ = view;
        // 可写视图。
        let mut view = exchange.world_view_mut(&mut field, 0).unwrap();
        assert_eq!(view.range(), 0..3);
        view.write_from(&[7.0, 8.0, 9.0]).unwrap();
        let _ = view;
        // 局部世界写入与完成回执。
        let first = exchange
            .write_world(&mut field, 1, &[10.0, 11.0, 12.0])
            .unwrap();
        assert_eq!(first.device(), 0);
        assert_eq!(first.world(), Some(1));
        assert_eq!(first.generation(), 1);
        let copied = exchange.copy_world_within(&mut field, 0, 1).unwrap();
        assert_eq!(copied.generation(), 2);
        assert_eq!(
            exchange.read_back(&field).unwrap(),
            [10.0, 11.0, 12.0, 10.0, 11.0, 12.0]
        );
        // 字段间复制。
        let source = exchange.upload(layout, &[21.0; 6]).unwrap();
        let third = exchange.copy_world_from(&mut field, 1, &source, 0).unwrap();
        assert_eq!(third.generation(), 3);
        assert_eq!(
            exchange.read_back(&field).unwrap(),
            [10.0, 11.0, 12.0, 21.0, 21.0, 21.0]
        );
        assert_eq!(exchange.generation(), 3);
    }

    #[test]
    #[ignore = "requires an NVIDIA GPU"]
    fn imports_and_exports_models_through_execution_layer() {
        let mut exchange = new_exchange();
        let input = model_input();
        let model = exchange.import_kinematic(&input).unwrap();
        assert_eq!(model.topology(), [1, 2, 1, 1]);
        let back = exchange.export_kinematic(&model).unwrap();
        assert_eq!(back, input);
        assert_eq!(exchange.generation(), 0);
    }

    #[test]
    #[ignore = "requires an NVIDIA GPU"]
    fn propagates_errors_without_partial_writes() {
        let mut exchange = new_exchange();
        let layout = BatchLayout::new(2, 3, 8).unwrap();
        let mut field = exchange.upload(layout, &[1.0; 6]).unwrap();
        assert!(matches!(
            exchange.write_world(&mut field, 2, &[2.0; 3]),
            Err(TransferError::Input(InputError::InvalidIndex {
                field: "world",
                ..
            }))
        ));
        assert!(matches!(
            exchange.write_world(&mut field, 0, &[2.0; 2]),
            Err(TransferError::Input(InputError::LengthMismatch { .. }))
        ));
        assert!(exchange.world_view_mut(&mut field, 2).is_err());
        let mut view = exchange.world_view_mut(&mut field, 0).unwrap();
        assert!(view.write_from(&[2.0; 2]).is_err());
        let _ = view;
        assert_eq!(exchange.read_back(&field).unwrap(), [1.0; 6]);
        // 跨会话复制拒绝；错误归属执行层会话检查。
        let mut foreign = new_exchange();
        let source = foreign
            .upload(BatchLayout::new(1, 3, 8).unwrap(), &[9.0; 3])
            .unwrap();
        assert_eq!(
            exchange
                .copy_world_from(&mut field, 0, &source, 0)
                .unwrap_err(),
            TransferError::SessionMismatch
        );
        assert_eq!(exchange.read_back(&field).unwrap(), [1.0; 6]);
        assert_eq!(exchange.generation(), 0);
    }
}
