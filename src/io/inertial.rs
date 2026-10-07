//! 惯性字段的原生转换与GPU上传。
//! 本子集不替代完整put_model。

use super::{
    DeviceBatch, DeviceKinematicModel,
    kinematic::{read_field, upload_field},
};
use crate::{
    diagnostics::TransferError,
    model::{InertialFields, InertialModelInput},
    runtime::TransferSession,
};

/// 独占二十一项只读GPU字段。
/// 上传完成后才发布本对象。
/// 所有字段共享一组参数。
/// 不暴露可写缓冲或设备地址。
///
/// ```compile_fail
/// use mjwarp_rs::io::DeviceInertialModel;
/// fn mutate(gpu: &mut DeviceInertialModel) {
///     gpu.body_mass.write_world(0, &[1.0]);
/// }
/// ```
pub struct DeviceInertialModel {
    kinematics: DeviceKinematicModel,
    body_ipos: DeviceBatch<f32>,
    body_iquat: DeviceBatch<f32>,
    body_mass: DeviceBatch<f32>,
    body_inertia: DeviceBatch<f32>,
    dof_bodyid: DeviceBatch<i32>,
    dof_jntid: DeviceBatch<i32>,
    dof_parentid: DeviceBatch<i32>,
    dof_armature: DeviceBatch<f32>,
    dof_damping: DeviceBatch<f32>,
}
impl DeviceInertialModel {
    pub fn upload(
        session: &TransferSession,
        input: &InertialModelInput,
    ) -> Result<Self, TransferError> {
        let f = input.fields();
        // Every field waits for completion and retains its session. On any failure
        // Rust drops constructed fields; no partially initialized group escapes.
        Ok(Self {
            kinematics: DeviceKinematicModel::upload(session, input.kinematics())?,
            body_ipos: upload_field(session, &f.body_ipos)?,
            body_iquat: upload_field(session, &f.body_iquat)?,
            body_mass: upload_field(session, &f.body_mass)?,
            body_inertia: upload_field(session, &f.body_inertia)?,
            dof_bodyid: upload_field(session, &f.dof_bodyid)?,
            dof_jntid: upload_field(session, &f.dof_jntid)?,
            dof_parentid: upload_field(session, &f.dof_parentid)?,
            dof_armature: upload_field(session, &f.dof_armature)?,
            dof_damping: upload_field(session, &f.dof_damping)?,
        })
    }
    pub fn readback(&self) -> Result<InertialModelInput, TransferError> {
        let kinematics = self.kinematics.readback()?;
        let fields = InertialFields {
            body_ipos: read_field(&self.body_ipos)?,
            body_iquat: read_field(&self.body_iquat)?,
            body_mass: read_field(&self.body_mass)?,
            body_inertia: read_field(&self.body_inertia)?,
            dof_bodyid: read_field(&self.dof_bodyid)?,
            dof_jntid: read_field(&self.dof_jntid)?,
            dof_parentid: read_field(&self.dof_parentid)?,
            dof_armature: read_field(&self.dof_armature)?,
            dof_damping: read_field(&self.dof_damping)?,
        };
        Ok(InertialModelInput::new(kinematics, fields)?)
    }
}

#[cfg(feature = "native-model-probe")]
pub use native::NativeInertialSnapshot;

#[cfg(feature = "native-model-probe")]
mod native {
    use super::*;
    use crate::{
        diagnostics::NativeProbeError,
        io::{
            NativeKinematicSnapshot,
            fields::native_staging as staging,
            kinematic::native::{check_info, convert, elements},
        },
    };
    use std::ffi::c_void;

    /// 独占二十一项原生字段。
    /// 原生标量保留f64精度。
    /// 转换执行严格有限值检查。
    #[derive(Clone, Debug, PartialEq)]
    pub struct NativeInertialSnapshot {
        pub kinematics: NativeKinematicSnapshot,
        pub body_ipos: Vec<f64>,
        pub body_iquat: Vec<f64>,
        pub body_mass: Vec<f64>,
        pub body_inertia: Vec<f64>,
        pub dof_bodyid: Vec<i32>,
        pub dof_jntid: Vec<i32>,
        pub dof_parentid: Vec<i32>,
        pub dof_armature: Vec<f64>,
        pub dof_damping: Vec<f64>,
    }

    #[repr(C)]
    struct NativeTargets {
        schema: u32,
        reserved: u32,
        nbody: u64,
        nv: u64,
        body_ipos: *mut f64,
        body_iquat: *mut f64,
        body_mass: *mut f64,
        body_inertia: *mut f64,
        dof_bodyid: *mut i32,
        dof_jntid: *mut i32,
        dof_parentid: *mut i32,
        dof_armature: *mut f64,
        dof_damping: *mut f64,
    }
    const _: () = {
        assert!(size_of::<NativeTargets>() == 96);
        assert!(align_of::<NativeTargets>() == 8);
        assert!(std::mem::offset_of!(NativeTargets, nbody) == 8);
        assert!(std::mem::offset_of!(NativeTargets, body_ipos) == 24);
        assert!(std::mem::offset_of!(NativeTargets, dof_bodyid) == 56);
        assert!(std::mem::offset_of!(NativeTargets, dof_damping) == 88);
    };
    unsafe extern "C" {
        fn mjwarp_native_copy_inertial(owner: *const c_void, targets: *const NativeTargets) -> i32;
    }
    impl NativeInertialSnapshot {
        /// # Safety
        /// owner须拥有快照对应模型。
        /// 调用期间须保持其DLL存活。
        pub(in crate::io) unsafe fn capture(
            owner: *const c_void,
            kinematics: NativeKinematicSnapshot,
        ) -> Result<Self, NativeProbeError> {
            let i = kinematics.info;
            check_info(i)?;
            let mut s = Self {
                kinematics,
                body_ipos: staging(elements(i.nbody, 3)?)?,
                body_iquat: staging(elements(i.nbody, 4)?)?,
                body_mass: staging(elements(i.nbody, 1)?)?,
                body_inertia: staging(elements(i.nbody, 3)?)?,
                dof_bodyid: staging(elements(i.nv, 1)?)?,
                dof_jntid: staging(elements(i.nv, 1)?)?,
                dof_parentid: staging(elements(i.nv, 1)?)?,
                dof_armature: staging(elements(i.nv, 1)?)?,
                dof_damping: staging(elements(i.nv, 1)?)?,
            };
            let targets = NativeTargets {
                schema: 1,
                reserved: 0,
                nbody: i.nbody as u64,
                nv: i.nv as u64,
                body_ipos: s.body_ipos.as_mut_ptr(),
                body_iquat: s.body_iquat.as_mut_ptr(),
                body_mass: s.body_mass.as_mut_ptr(),
                body_inertia: s.body_inertia.as_mut_ptr(),
                dof_bodyid: s.dof_bodyid.as_mut_ptr(),
                dof_jntid: s.dof_jntid.as_mut_ptr(),
                dof_parentid: s.dof_parentid.as_mut_ptr(),
                dof_armature: s.dof_armature.as_mut_ptr(),
                dof_damping: s.dof_damping.as_mut_ptr(),
            };
            // SAFETY: caller retains the immutable model and DLL. All targets are
            // owned, disjoint, aligned and capacity-checked. The bridge validates
            // all sources and targets before copying; zero-length arrays stay untouched.
            let status = unsafe { mjwarp_native_copy_inertial(owner, &targets) };
            if status != 0 {
                return Err(NativeProbeError::Native {
                    stage: "copy_inertial",
                    code: status as u32,
                });
            }
            Ok(s)
        }
        pub fn into_model(self) -> Result<InertialModelInput, NativeProbeError> {
            let i = self.kinematics.info;
            check_info(i)?;
            for (field, len, count, width) in [
                ("body_ipos", self.body_ipos.len(), i.nbody, 3),
                ("body_iquat", self.body_iquat.len(), i.nbody, 4),
                ("body_mass", self.body_mass.len(), i.nbody, 1),
                ("body_inertia", self.body_inertia.len(), i.nbody, 3),
                ("dof_bodyid", self.dof_bodyid.len(), i.nv, 1),
                ("dof_jntid", self.dof_jntid.len(), i.nv, 1),
                ("dof_parentid", self.dof_parentid.len(), i.nv, 1),
                ("dof_armature", self.dof_armature.len(), i.nv, 1),
                ("dof_damping", self.dof_damping.len(), i.nv, 1),
            ] {
                crate::io::check_length(field, elements(count, width)?, len)?;
            }
            let kinematics = self.kinematics.into_model()?;
            // Reject negative native parameters before f32 underflow can hide them.
            for (field, values) in [
                ("body_mass", &self.body_mass),
                ("body_inertia", &self.body_inertia),
                ("dof_armature", &self.dof_armature),
                ("dof_damping", &self.dof_damping),
            ] {
                for (index, &value) in values.iter().enumerate() {
                    if value.is_finite() && value < 0.0 {
                        return Err(
                            crate::diagnostics::InputError::NegativeValue { field, index }.into(),
                        );
                    }
                }
            }
            let fields = InertialFields {
                body_ipos: convert("body_ipos", &self.body_ipos)?,
                body_iquat: convert("body_iquat", &self.body_iquat)?,
                body_mass: convert("body_mass", &self.body_mass)?,
                body_inertia: convert("body_inertia", &self.body_inertia)?,
                dof_bodyid: self.dof_bodyid,
                dof_jntid: self.dof_jntid,
                dof_parentid: self.dof_parentid,
                dof_armature: convert("dof_armature", &self.dof_armature)?,
                dof_damping: convert("dof_damping", &self.dof_damping)?,
            };
            Ok(InertialModelInput::new(kinematics, fields)?)
        }
    }
}
