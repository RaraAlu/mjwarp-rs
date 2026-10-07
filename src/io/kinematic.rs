//! 十二项运动学字段的转换与上传。
//! 本子集不替代完整put_model。

use super::DeviceBatch;
use crate::{
    diagnostics::TransferError,
    model::{BatchLayout, KinematicFields, KinematicModelInput},
    runtime::{TransferElement, TransferSession, host_staging},
};

/// 独占只读GPU字段组。
/// 所有字段共享一组参数。
/// 上传完成后才返回本对象。
/// 不提供裸指针或可写缓冲。
///
/// ```compile_fail
/// use mjwarp_rs::{io::DeviceKinematicModel, model::KinematicModelInput,
///     runtime::TransferSession};
/// fn mutate(session: &TransferSession, input: &KinematicModelInput) {
///     let mut gpu = DeviceKinematicModel::upload(session, input).unwrap();
///     gpu.body_parentid.write_world(0, &[1]);
/// }
/// ```
pub struct DeviceKinematicModel {
    nv: usize,
    qpos0: DeviceBatch<f32>,
    body_parentid: DeviceBatch<i32>,
    body_jntadr: DeviceBatch<i32>,
    body_jntnum: DeviceBatch<i32>,
    body_pos: DeviceBatch<f32>,
    body_quat: DeviceBatch<f32>,
    jnt_type: DeviceBatch<i32>,
    jnt_bodyid: DeviceBatch<i32>,
    jnt_qposadr: DeviceBatch<i32>,
    jnt_dofadr: DeviceBatch<i32>,
    jnt_pos: DeviceBatch<f32>,
    jnt_axis: DeviceBatch<f32>,
}

impl DeviceKinematicModel {
    pub fn upload(
        session: &TransferSession,
        input: &KinematicModelInput,
    ) -> Result<Self, TransferError> {
        let f = input.fields();
        // Each synchronous transfer retains its session and waits for completion.
        // On failure, previously built fields drop without publishing a partial group.
        Ok(Self {
            nv: input.nv(),
            qpos0: upload_field(session, &f.qpos0)?,
            body_parentid: upload_field(session, &f.body_parentid)?,
            body_jntadr: upload_field(session, &f.body_jntadr)?,
            body_jntnum: upload_field(session, &f.body_jntnum)?,
            body_pos: upload_field(session, &f.body_pos)?,
            body_quat: upload_field(session, &f.body_quat)?,
            jnt_type: upload_field(session, &f.jnt_type)?,
            jnt_bodyid: upload_field(session, &f.jnt_bodyid)?,
            jnt_qposadr: upload_field(session, &f.jnt_qposadr)?,
            jnt_dofadr: upload_field(session, &f.jnt_dofadr)?,
            jnt_pos: upload_field(session, &f.jnt_pos)?,
            jnt_axis: upload_field(session, &f.jnt_axis)?,
        })
    }

    pub fn readback(&self) -> Result<KinematicModelInput, TransferError> {
        let fields = KinematicFields {
            qpos0: read_field(&self.qpos0)?,
            body_parentid: read_field(&self.body_parentid)?,
            body_jntadr: read_field(&self.body_jntadr)?,
            body_jntnum: read_field(&self.body_jntnum)?,
            body_pos: read_field(&self.body_pos)?,
            body_quat: read_field(&self.body_quat)?,
            jnt_type: read_field(&self.jnt_type)?,
            jnt_bodyid: read_field(&self.jnt_bodyid)?,
            jnt_qposadr: read_field(&self.jnt_qposadr)?,
            jnt_dofadr: read_field(&self.jnt_dofadr)?,
            jnt_pos: read_field(&self.jnt_pos)?,
            jnt_axis: read_field(&self.jnt_axis)?,
        };
        Ok(KinematicModelInput::new(self.nv, fields)?)
    }
}

fn upload_field<T: TransferElement>(
    session: &TransferSession,
    values: &[T],
) -> Result<DeviceBatch<T>, TransferError> {
    DeviceBatch::upload(
        session,
        BatchLayout::new(1, values.len(), size_of::<T>())?,
        values,
    )
}

fn read_field<T: TransferElement>(field: &DeviceBatch<T>) -> Result<Vec<T>, TransferError> {
    let mut values = host_staging(field.layout().total_elements())?;
    field.read_into(&mut values)?;
    Ok(values)
}

#[cfg(feature = "native-model-probe")]
pub use native::NativeKinematicSnapshot;

#[cfg(feature = "native-model-probe")]
mod native {
    use super::*;
    use crate::{
        diagnostics::{InputError, NativeProbeError},
        model::NativeModelInfo,
    };
    use std::ffi::c_void;

    /// 独占原生运动学字段。
    /// 标量保留原生f64精度。
    /// 转换入口执行严格检查。
    #[derive(Clone, Debug, PartialEq)]
    pub struct NativeKinematicSnapshot {
        pub info: NativeModelInfo,
        pub qpos0: Vec<f64>,
        pub body_parentid: Vec<i32>,
        pub body_jntadr: Vec<i32>,
        pub body_jntnum: Vec<i32>,
        pub body_pos: Vec<f64>,
        pub body_quat: Vec<f64>,
        pub jnt_type: Vec<i32>,
        pub jnt_bodyid: Vec<i32>,
        pub jnt_qposadr: Vec<i32>,
        pub jnt_dofadr: Vec<i32>,
        pub jnt_pos: Vec<f64>,
        pub jnt_axis: Vec<f64>,
    }

    #[repr(C)]
    struct NativeTargets {
        schema: u32,
        reserved: u32,
        nq: u64,
        nbody: u64,
        njnt: u64,
        qpos0: *mut f64,
        body_parentid: *mut i32,
        body_jntadr: *mut i32,
        body_jntnum: *mut i32,
        body_pos: *mut f64,
        body_quat: *mut f64,
        jnt_type: *mut i32,
        jnt_bodyid: *mut i32,
        jnt_qposadr: *mut i32,
        jnt_dofadr: *mut i32,
        jnt_pos: *mut f64,
        jnt_axis: *mut f64,
    }

    const _: () = {
        assert!(size_of::<NativeTargets>() == 128);
        assert!(align_of::<NativeTargets>() == 8);
        assert!(std::mem::offset_of!(NativeTargets, nq) == 8);
        assert!(std::mem::offset_of!(NativeTargets, qpos0) == 32);
        assert!(std::mem::offset_of!(NativeTargets, body_quat) == 72);
        assert!(std::mem::offset_of!(NativeTargets, jnt_axis) == 120);
    };

    unsafe extern "C" {
        fn mjwarp_native_copy_kinematic(owner: *const c_void, targets: *const NativeTargets)
        -> i32;
    }

    fn elements(count: i64, width: usize) -> Result<usize, InputError> {
        let count = usize::try_from(count).map_err(|_| InputError::Overflow {
            field: "kinematic_field",
        })?;
        let len = count.checked_mul(width).ok_or(InputError::Overflow {
            field: "kinematic_field",
        })?;
        BatchLayout::new(1, len, 8)?;
        Ok(len)
    }

    fn check_info(info: NativeModelInfo) -> Result<(), NativeProbeError> {
        info.validate()?;
        for (field, count) in [
            ("nq", info.nq),
            ("nv", info.nv),
            ("nbody", info.nbody),
            ("njnt", info.njnt),
        ] {
            if count > i64::from(i32::MAX) {
                return Err(InputError::Overflow { field }.into());
            }
        }
        Ok(())
    }

    impl NativeKinematicSnapshot {
        /// # Safety
        /// owner须拥有有效只读模型。
        /// 调用期间须保持其DLL存活。
        pub(in crate::io) unsafe fn capture(
            owner: *const c_void,
            info: NativeModelInfo,
        ) -> Result<Self, NativeProbeError> {
            use crate::io::native::staging;
            check_info(info)?;
            let mut s = Self {
                info,
                qpos0: staging(elements(info.nq, 1)?)?,
                body_parentid: staging(elements(info.nbody, 1)?)?,
                body_jntadr: staging(elements(info.nbody, 1)?)?,
                body_jntnum: staging(elements(info.nbody, 1)?)?,
                body_pos: staging(elements(info.nbody, 3)?)?,
                body_quat: staging(elements(info.nbody, 4)?)?,
                jnt_type: staging(elements(info.njnt, 1)?)?,
                jnt_bodyid: staging(elements(info.njnt, 1)?)?,
                jnt_qposadr: staging(elements(info.njnt, 1)?)?,
                jnt_dofadr: staging(elements(info.njnt, 1)?)?,
                jnt_pos: staging(elements(info.njnt, 3)?)?,
                jnt_axis: staging(elements(info.njnt, 3)?)?,
            };
            let targets = NativeTargets {
                schema: 1,
                reserved: 0,
                nq: info.nq as u64,
                nbody: info.nbody as u64,
                njnt: info.njnt as u64,
                qpos0: s.qpos0.as_mut_ptr(),
                body_parentid: s.body_parentid.as_mut_ptr(),
                body_jntadr: s.body_jntadr.as_mut_ptr(),
                body_jntnum: s.body_jntnum.as_mut_ptr(),
                body_pos: s.body_pos.as_mut_ptr(),
                body_quat: s.body_quat.as_mut_ptr(),
                jnt_type: s.jnt_type.as_mut_ptr(),
                jnt_bodyid: s.jnt_bodyid.as_mut_ptr(),
                jnt_qposadr: s.jnt_qposadr.as_mut_ptr(),
                jnt_dofadr: s.jnt_dofadr.as_mut_ptr(),
                jnt_pos: s.jnt_pos.as_mut_ptr(),
                jnt_axis: s.jnt_axis.as_mut_ptr(),
            };
            // SAFETY: caller retains the immutable model and DLL; all arrays are
            // owned, disjoint, aligned and capacity-checked. The bridge preflights
            // every native source and target before copying; empty arrays stay untouched.
            let status = unsafe { mjwarp_native_copy_kinematic(owner, &targets) };
            if status != 0 {
                return Err(NativeProbeError::Native {
                    stage: "copy_kinematic",
                    code: status as u32,
                });
            }
            Ok(s)
        }

        pub fn into_model(self) -> Result<KinematicModelInput, NativeProbeError> {
            let i = self.info;
            check_info(i)?;
            for (field, len, count, width) in [
                ("qpos0", self.qpos0.len(), i.nq, 1),
                ("body_parentid", self.body_parentid.len(), i.nbody, 1),
                ("body_jntadr", self.body_jntadr.len(), i.nbody, 1),
                ("body_jntnum", self.body_jntnum.len(), i.nbody, 1),
                ("body_pos", self.body_pos.len(), i.nbody, 3),
                ("body_quat", self.body_quat.len(), i.nbody, 4),
                ("jnt_type", self.jnt_type.len(), i.njnt, 1),
                ("jnt_bodyid", self.jnt_bodyid.len(), i.njnt, 1),
                ("jnt_qposadr", self.jnt_qposadr.len(), i.njnt, 1),
                ("jnt_dofadr", self.jnt_dofadr.len(), i.njnt, 1),
                ("jnt_pos", self.jnt_pos.len(), i.njnt, 3),
                ("jnt_axis", self.jnt_axis.len(), i.njnt, 3),
            ] {
                crate::io::check_length(field, elements(count, width)?, len)?;
            }
            let f = KinematicFields {
                qpos0: convert("qpos0", &self.qpos0)?,
                body_parentid: self.body_parentid,
                body_jntadr: self.body_jntadr,
                body_jntnum: self.body_jntnum,
                body_pos: convert("body_pos", &self.body_pos)?,
                body_quat: convert("body_quat", &self.body_quat)?,
                jnt_type: self.jnt_type,
                jnt_bodyid: self.jnt_bodyid,
                jnt_qposadr: self.jnt_qposadr,
                jnt_dofadr: self.jnt_dofadr,
                jnt_pos: convert("jnt_pos", &self.jnt_pos)?,
                jnt_axis: convert("jnt_axis", &self.jnt_axis)?,
            };
            Ok(KinematicModelInput::new(i.nv as usize, f)?)
        }
    }

    fn convert(field: &'static str, values: &[f64]) -> Result<Vec<f32>, NativeProbeError> {
        let mut target = crate::io::native::staging(values.len())?;
        crate::io::convert_f64_to_f32_into(values, &mut target).map_err(|error| match error {
            InputError::NonFinite { index, .. } => InputError::NonFinite { field, index },
            InputError::ScalarOverflow { index, .. } => InputError::ScalarOverflow { field, index },
            other => other,
        })?;
        Ok(target)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn rejects_expanded_capacity_overflow() {
            assert!(elements(i64::MAX, 4).is_err());
            assert!(elements(-1, 1).is_err());
            assert_eq!(elements(0, 3).unwrap(), 0);
        }
        #[test]
        fn labels_float_conversion_failures_and_preserves_bits() {
            for field in ["qpos0", "body_pos", "body_quat", "jnt_pos", "jnt_axis"] {
                assert_eq!(
                    convert(field, &[1.0, f64::MAX]),
                    Err(NativeProbeError::Input(InputError::ScalarOverflow {
                        field,
                        index: 1
                    }))
                );
                assert_eq!(
                    convert(field, &[1.0, f64::NAN]),
                    Err(NativeProbeError::Input(InputError::NonFinite {
                        field,
                        index: 1
                    }))
                );
            }
            let f = convert("body_pos", &[-0.0, f64::MIN_POSITIVE]).unwrap();
            assert_eq!(f[0].to_bits(), (-0.0_f32).to_bits());
            assert_eq!(f[1], 0.0);
        }
    }
}
