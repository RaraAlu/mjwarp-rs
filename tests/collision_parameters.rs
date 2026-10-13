//! 独立批量的新宿主行为；不重跑旧转换或原生捕获目标。
use mjwarp_rs::{
    diagnostics::{InputError, NativeProbeError},
    io::NativePrimitiveGeometrySnapshot,
    model::{
        NativeModelInfo, ParameterBatch, PrimitiveGeometryHostBatch,
        PrimitiveGeometryParameter as Field, PrimitiveGeometryParameters,
    },
};

fn snapshot(n: usize) -> NativePrimitiveGeometrySnapshot {
    NativePrimitiveGeometrySnapshot {
        info: NativeModelInfo {
            schema: 1,
            native_version: NativeModelInfo::VERSION,
            pointer_bytes: 8,
            num_bytes: 8,
            index_bytes: 4,
            size_bytes: 8,
            native_model_bytes: 1,
            nbody: 1,
            ngeom: n as i64,
            ..Default::default()
        },
        geom_type: (0..n).map(|i| if i == 0 { 0 } else { 2 }).collect(),
        geom_bodyid: vec![0; n],
        geom_contype: vec![-1; n],
        geom_conaffinity: vec![i32::MIN; n],
        geom_condim: vec![3; n],
        geom_priority: vec![-2; n],
        geom_solmix: vec![0.5; n],
        geom_solref: vec![0.02; n * 2],
        geom_solimp: vec![0.9; n * 5],
        geom_size: vec![0.25; n * 3],
        geom_friction: vec![0.3; n * 3],
        geom_margin: vec![-0.0; n],
        geom_gap: vec![0.0; n],
    }
}

#[test]
fn default_and_partial_overrides_keep_shared_rows_and_integer_topology() {
    let base = snapshot(2)
        .into_host_batch(5, PrimitiveGeometryParameters::default())
        .unwrap();
    assert_eq!((base.worlds(), base.model().ngeom()), (5, 2));
    for w in 0..5 {
        let view = base.world(w).unwrap();
        let first = base.world(0).unwrap();
        for field in Field::ALL {
            assert_eq!(base.parameters().batches(field), 1);
            assert_eq!(view.values(field).as_ptr(), first.values(field).as_ptr());
            assert_eq!(view.geometry_values(field, 1).unwrap().len(), field.width());
        }
        assert_eq!(view.values(Field::Margin)[0].to_bits(), 0x8000_0000);
        assert_eq!(
            view.topology().geom_type.as_ptr(),
            first.topology().geom_type.as_ptr()
        );
        assert_eq!(
            view.topology().geom_bodyid.as_ptr(),
            first.topology().geom_bodyid.as_ptr()
        );
        assert_eq!(view.topology().geom_contype, [-1, -1]);
        assert_eq!(view.topology().geom_conaffinity, [i32::MIN, i32::MIN]);
        assert_eq!(view.topology().geom_condim, [3, 3]);
        assert_eq!(view.topology().geom_priority, [-2, -2]);
    }
    let mut parameters = PrimitiveGeometryParameters::default();
    parameters.set(
        Field::Gap,
        ParameterBatch::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]).unwrap(),
    );
    let partial = PrimitiveGeometryHostBatch::new(base.model().clone(), 5, parameters).unwrap();
    assert_eq!(partial.world(4).unwrap().values(Field::Gap), [1.0, 2.0]);
    assert_eq!(partial.world(3).unwrap().values(Field::Gap), [3.0, 4.0]);
    for field in Field::ALL.into_iter().filter(|f| *f != Field::Gap) {
        assert_eq!(partial.parameters().batches(field), 1);
        assert_eq!(
            partial.world(3).unwrap().values(field),
            base.world(3).unwrap().values(field)
        );
    }
}

#[test]
fn all_seven_fields_use_independent_periods_and_per_geometry_widths() {
    assert_eq!(Field::ALL.map(Field::width), [1, 2, 5, 3, 3, 1, 1]);
    let mut parameters = PrimitiveGeometryParameters::default();
    for (id, field) in Field::ALL.into_iter().enumerate() {
        let period = id + 1;
        let row = 2 * field.width();
        let values = (0..period)
            .flat_map(|r| (0..row).map(move |c| (id * 100 + r * 10 + c) as f32))
            .collect();
        parameters.set(field, ParameterBatch::new(period, row, values).unwrap());
    }
    // Five worlds deliberately do not divide periods 2/3/4/6/7.
    // Periods 6 and 7 exceed the number of exposed worlds.
    let batch = snapshot(2).into_host_batch(5, parameters).unwrap();
    for w in 0..5 {
        for (id, field) in Field::ALL.into_iter().enumerate() {
            assert_eq!(batch.parameters().batches(field), id + 1);
            let row = w % (id + 1);
            let expected: Vec<_> = (0..2 * field.width())
                .map(|c| (id * 100 + row * 10 + c) as f32)
                .collect();
            let view = batch.world(w).unwrap();
            assert_eq!(view.values(field), expected);
            assert_eq!(
                view.geometry_values(field, 1).unwrap(),
                &expected[field.width()..]
            );
        }
    }
}

#[test]
fn binding_rejects_every_wrong_row_width_and_accepts_empty_periodic_fields() {
    for field in Field::ALL {
        let mut parameters = PrimitiveGeometryParameters::default();
        let wrong = 2 * field.width() + 1;
        parameters.set(
            field,
            ParameterBatch::new(2, wrong, vec![0.0; 2 * wrong]).unwrap(),
        );
        assert!(matches!(snapshot(2).into_host_batch(5, parameters),
            Err(NativeProbeError::Input(InputError::LengthMismatch { field: name, .. })) if name == field.name()));
    }
    let mut parameters = PrimitiveGeometryParameters::default();
    for (id, field) in Field::ALL.into_iter().enumerate() {
        parameters.set(field, ParameterBatch::new(id + 1, 0, vec![]).unwrap());
    }
    let batch = snapshot(0).into_host_batch(5, parameters).unwrap();
    for field in Field::ALL {
        assert!(batch.world(4).unwrap().values(field).is_empty());
        assert!(matches!(
            batch.world(4).unwrap().geometry_values(field, 0),
            Err(InputError::InvalidIndex {
                field: "geometry",
                limit: 0,
                ..
            })
        ));
    }
}

#[test]
fn rejects_world_and_geometry_indices_and_invalid_batch_storage() {
    assert!(matches!(
        snapshot(2).into_host_batch(0, PrimitiveGeometryParameters::default()),
        Err(NativeProbeError::Input(InputError::InvalidDimension {
            field: "worlds"
        }))
    ));
    assert!(matches!(
        snapshot(2).into_host_batch(usize::MAX, PrimitiveGeometryParameters::default()),
        Err(NativeProbeError::Input(InputError::Overflow {
            field: "collision_worlds"
        }))
    ));
    let batch = snapshot(2)
        .into_host_batch(5, PrimitiveGeometryParameters::default())
        .unwrap();
    for w in [5, usize::MAX] {
        assert!(
            matches!(batch.world(w), Err(InputError::InvalidIndex { field: "world", index, limit: 5 }) if index == w)
        );
    }
    for g in [2, usize::MAX] {
        for field in Field::ALL {
            assert!(matches!(batch.world(0).unwrap().geometry_values(field, g),
                Err(InputError::InvalidIndex { field: "geometry", index, limit: 2 }) if index == g));
        }
    }
    assert!(ParameterBatch::new(0, 2, vec![]).is_err());
    assert!(ParameterBatch::new(2, 2, vec![0.0; 3]).is_err());
    assert!(ParameterBatch::new(i32::MAX as usize, 2, vec![]).is_err());
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        // The last row would not be used by a one-world batch, but still rejects.
        assert!(matches!(
            ParameterBatch::new(3, 2, vec![0.0, 0.0, 0.0, 0.0, 0.0, bad]),
            Err(InputError::NonFinite { index: 5, .. })
        ));
    }
}

#[test]
fn native_adapter_checks_abi_and_default_fields_before_parameter_binding() {
    let mut bad = snapshot(2);
    bad.info.native_version += 1;
    assert!(matches!(
        bad.into_host_batch(5, PrimitiveGeometryParameters::default()),
        Err(NativeProbeError::AbiMismatch)
    ));
    let mut bad = snapshot(2);
    bad.info.num_bytes = 4;
    assert!(matches!(
        bad.into_host_batch(5, PrimitiveGeometryParameters::default()),
        Err(NativeProbeError::AbiMismatch)
    ));
    let mut bad = snapshot(2);
    bad.geom_solref.pop();
    assert!(matches!(
        bad.into_host_batch(5, PrimitiveGeometryParameters::default()),
        Err(NativeProbeError::Input(InputError::LengthMismatch {
            field: "geom_solref",
            ..
        }))
    ));
    let mut bad = snapshot(2);
    bad.geom_margin[0] = f64::MAX;
    let mut parameters = PrimitiveGeometryParameters::default();
    parameters.set(
        Field::Margin,
        ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
    );
    assert!(matches!(
        bad.into_host_batch(5, parameters),
        Err(NativeProbeError::Input(InputError::ScalarOverflow {
            field: "geom_margin",
            index: 0
        }))
    ));
    let batch = snapshot(2)
        .into_host_batch(5, PrimitiveGeometryParameters::default())
        .unwrap();
    assert_eq!(
        batch.world(4).unwrap().values(Field::Friction)[0].to_bits(),
        0x3e99_999a
    );
    let cloned = batch.clone();
    drop(batch);
    assert_eq!(cloned.world(4).unwrap().values(Field::Solmix), [0.5, 0.5]);
}
