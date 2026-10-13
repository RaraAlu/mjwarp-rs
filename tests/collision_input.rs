//! G08宿主字段小原型；不执行物理或设备代码。
use mjwarp_rs::{
    diagnostics::{InputError, NativeProbeError},
    io::NativePrimitiveGeometrySnapshot,
    model::{NativeModelInfo, PrimitiveGeometryFields, PrimitiveGeometryInput},
};

fn snapshot() -> NativePrimitiveGeometrySnapshot {
    NativePrimitiveGeometrySnapshot {
        info: NativeModelInfo {
            schema: 1,
            native_version: NativeModelInfo::VERSION,
            pointer_bytes: 8,
            num_bytes: 8,
            index_bytes: 4,
            size_bytes: 8,
            native_model_bytes: 5175,
            nq: 7,
            nv: 6,
            nbody: 2,
            njnt: 1,
            ngeom: 2,
            ..Default::default()
        },
        geom_type: vec![0, 2],
        geom_bodyid: vec![0, 1],
        geom_contype: vec![-1, i32::MIN],
        geom_conaffinity: vec![i32::MIN, -1],
        geom_condim: vec![3, 6],
        geom_priority: vec![-1, 4],
        geom_solmix: vec![1.0, 0.5],
        geom_solref: vec![0.02, 1.0, 0.02, 1.0],
        geom_solimp: vec![0.9; 10],
        geom_size: vec![2.0, 2.0, 0.125, 0.25, 0.0, 0.0],
        geom_friction: vec![0.3; 6],
        geom_margin: vec![0.125; 2],
        geom_gap: vec![0.0; 2],
    }
}

#[test]
fn owns_thirteen_fields_and_preserves_quantization_and_signed_masks() {
    let mut native = snapshot();
    native.geom_gap = vec![-0.0, f64::MIN_POSITIVE];
    let input = native.into_input().unwrap();
    assert_eq!((input.nbody(), input.ngeom()), (2, 2));
    let f = input.fields();
    assert_eq!(f.geom_type, [0, 2]);
    assert_eq!(f.geom_bodyid, [0, 1]);
    assert_eq!(f.geom_contype, [-1, i32::MIN]);
    assert_eq!(f.geom_conaffinity, [i32::MIN, -1]);
    assert_eq!(f.geom_condim, [3, 6]);
    assert_eq!(f.geom_priority, [-1, 4]);
    assert_eq!(f.geom_solmix, [1.0, 0.5]);
    assert_eq!(f.geom_solref, [0.02_f32, 1.0, 0.02, 1.0]);
    assert_eq!(f.geom_solimp, [0.9_f32; 10]);
    assert_eq!(f.geom_size, [2.0, 2.0, 0.125, 0.25, 0.0, 0.0]);
    assert_eq!(f.geom_friction[0].to_bits(), 0x3e99_999a);
    assert_eq!(f.geom_margin, [0.125; 2]);
    assert_eq!(
        f.geom_gap.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        [0x8000_0000, 0]
    );
}

#[test]
fn rejects_each_native_field_width_before_conversion() {
    macro_rules! shortened {
        ($($field:ident),+ $(,)?) => { $(
            let mut value = snapshot();
            value.$field.pop();
            assert!(matches!(value.into_input(), Err(NativeProbeError::Input(
                InputError::LengthMismatch { field: stringify!($field), .. }
            ))));
        )+ };
    }
    shortened!(
        geom_type,
        geom_bodyid,
        geom_contype,
        geom_conaffinity,
        geom_condim,
        geom_priority,
        geom_solmix,
        geom_solref,
        geom_solimp,
        geom_size,
        geom_friction,
        geom_margin,
        geom_gap
    );
    let mut value = snapshot();
    value.geom_gap[0] = f64::NAN;
    value.geom_size.pop();
    assert!(matches!(
        value.into_input(),
        Err(NativeProbeError::Input(InputError::LengthMismatch {
            field: "geom_size",
            ..
        }))
    ));
}

#[test]
fn rejects_nonfinite_and_overflow_in_every_native_float_field() {
    macro_rules! invalid {
        ($($field:ident),+ $(,)?) => { $(
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
                let mut value = snapshot();
                value.$field[1] = bad;
                let error = value.into_input().unwrap_err();
                if bad.is_finite() {
                    assert!(matches!(error, NativeProbeError::Input(InputError::ScalarOverflow {
                        field: stringify!($field), index: 1
                    })));
                } else {
                    assert!(matches!(error, NativeProbeError::Input(InputError::NonFinite {
                        field: stringify!($field), index: 1
                    })));
                }
            }
        )+ };
    }
    invalid!(
        geom_solmix,
        geom_solref,
        geom_solimp,
        geom_size,
        geom_friction,
        geom_margin,
        geom_gap
    );
}

#[test]
fn checks_abi_counts_geometry_kind_body_and_contact_dimension() {
    let mut value = snapshot();
    value.info.native_version += 1;
    assert!(matches!(
        value.into_input(),
        Err(NativeProbeError::AbiMismatch)
    ));
    let mut value = snapshot();
    value.info.ngeom = -1;
    assert!(matches!(
        value.into_input(),
        Err(NativeProbeError::Input(InputError::InvalidDimension {
            field: "ngeom"
        }))
    ));
    let mut value = snapshot();
    value.info.ngeom = i64::MAX;
    assert!(value.into_input().is_err());
    let mut value = snapshot();
    value.info.ngeom = i64::from(i32::MAX);
    assert!(matches!(
        value.into_input(),
        Err(NativeProbeError::Input(InputError::Overflow {
            field: "primitive_geometry"
        }))
    ));
    for bad in [-1, 1, 3, 8] {
        let mut value = snapshot();
        value.geom_type[1] = bad;
        assert!(matches!(
            value.into_input(),
            Err(NativeProbeError::Input(InputError::InvalidTopology {
                field: "geom_type",
                ..
            }))
        ));
    }
    for bad in [-1, 2] {
        let mut value = snapshot();
        value.geom_bodyid[1] = bad;
        assert!(matches!(
            value.into_input(),
            Err(NativeProbeError::Input(InputError::InvalidTopology {
                field: "geom_bodyid",
                ..
            }))
        ));
    }
    for bad in [0, 2, 5, 7] {
        let mut value = snapshot();
        value.geom_condim[1] = bad;
        assert!(matches!(
            value.into_input(),
            Err(NativeProbeError::Input(InputError::InvalidTopology {
                field: "geom_condim",
                ..
            }))
        ));
    }
    for dim in [1, 3, 4, 6] {
        let mut value = snapshot();
        value.geom_condim.fill(dim);
        assert!(value.into_input().is_ok());
    }
}

#[test]
fn accepts_empty_geometry_and_rechecks_direct_float_inputs() {
    let empty = NativePrimitiveGeometrySnapshot {
        info: NativeModelInfo {
            ngeom: 0,
            ..snapshot().info
        },
        ..Default::default()
    };
    assert_eq!(empty.into_input().unwrap().ngeom(), 0);
    assert!(matches!(
        PrimitiveGeometryInput::new(0, PrimitiveGeometryFields::default()),
        Err(InputError::InvalidDimension { field: "nbody" })
    ));
    assert!(matches!(
        PrimitiveGeometryInput::new(i32::MAX as usize + 1, PrimitiveGeometryFields::default()),
        Err(InputError::Overflow { .. })
    ));
    let fields = snapshot().into_input().unwrap().fields().clone();
    macro_rules! invalid {
        ($($field:ident),+ $(,)?) => { $(
            let mut value = fields.clone(); value.$field[0] = f32::NAN;
            assert!(matches!(PrimitiveGeometryInput::new(2, value), Err(InputError::NonFinite { field: stringify!($field), index: 0 })));
        )+ };
    }
    invalid!(
        geom_solmix,
        geom_solref,
        geom_solimp,
        geom_size,
        geom_friction,
        geom_margin,
        geom_gap
    );
    let mut value = fields;
    value.geom_solimp.pop();
    assert!(matches!(
        PrimitiveGeometryInput::new(2, value),
        Err(InputError::LengthMismatch {
            field: "geom_solimp",
            ..
        })
    ));
}
