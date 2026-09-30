//! 宿主转换与连续GPU字段交换。
//! 不提供完整模型上传入口。

mod fields;
#[cfg(feature = "native-model-probe")]
mod flex;
#[cfg(feature = "native-model-probe")]
mod g01;
#[cfg(feature = "native-model-probe")]
use fields::check_length;
pub use fields::{DeviceBatch, convert_f64_to_f32_into, copy_world_f32, upload_f64_batch};
#[cfg(feature = "native-model-probe")]
pub use flex::NativeFlexPositionSnapshot;
#[cfg(feature = "native-model-probe")]
pub use g01::NativeG01Snapshot;

mod inertial;
mod kinematic;
pub use inertial::DeviceInertialModel;
#[cfg(feature = "native-model-probe")]
pub use inertial::NativeInertialSnapshot;
pub use kinematic::DeviceKinematicModel;
#[cfg(feature = "native-model-probe")]
pub use kinematic::NativeKinematicSnapshot;

#[cfg(feature = "native-model-probe")]
pub use native::{NativeCoreSnapshot, NativeModelProbe};

#[cfg(feature = "native-model-probe")]
mod native {
    use super::fields::native_staging as staging;
    use crate::{
        diagnostics::{InputError, NativeProbeError},
        model::NativeModelInfo,
    };
    #[cfg(target_os = "linux")]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(target_os = "windows")]
    use std::os::windows::ffi::OsStrExt;
    use std::{ffi::c_void, marker::PhantomData, path::Path, ptr::NonNull, rc::Rc};

    #[cfg(target_os = "windows")]
    type NativePathChar = u16;
    #[cfg(target_os = "linux")]
    type NativePathChar = std::ffi::c_char;

    unsafe extern "C" {
        fn mjwarp_native_open(
            path: *const NativePathChar,
            mjb: *const c_void,
            bytes: i32,
            owner: *mut *mut c_void,
            info: *mut NativeModelInfo,
            detail: *mut u32,
        ) -> i32;
        fn mjwarp_native_close(owner: *mut c_void);
        fn mjwarp_native_copy(
            owner: *const c_void,
            qpos0: *mut f64,
            nq: u64,
            mass: *mut f64,
            parent: *mut i32,
            nbody: u64,
            joint: *mut i32,
            njnt: u64,
        ) -> i32;
    }

    /// 独占原生模型与DLL。
    /// 本探针只允许单线程调用。
    /// 不接收或导出裸模型指针。
    pub struct NativeModelProbe {
        owner: NonNull<c_void>,
        info: NativeModelInfo,
        _thread: PhantomData<Rc<()>>,
    }

    /// 四项代表字段的独占快照。
    /// 不表示完整ModelInput。
    #[derive(Clone, Debug, PartialEq)]
    pub struct NativeCoreSnapshot {
        pub info: NativeModelInfo,
        /// 广义坐标；单位随关节变化。
        pub qpos0: Vec<f64>,
        /// 体质量；单位kg。
        pub body_mass: Vec<f64>,
        /// 父体索引；包含世界体。
        pub body_parentid: Vec<i32>,
        /// 原生关节枚举值。
        pub jnt_type: Vec<i32>,
    }

    impl NativeModelProbe {
        /// 读取可信DLL与已编译MJB。
        /// 本入口不编译MJCF。
        ///
        /// # Safety
        /// 调用者须信任DLL及其依赖。
        /// DLL须遵循候选原生ABI。
        /// 调用者须保证原生解析安全。
        /// 正常输入须来自可信制备流程。
        /// 头部检查不构成恶意输入隔离。
        /// 原生库可能终止进程。
        pub unsafe fn load_trusted(dll: &Path, mjb: &[u8]) -> Result<Self, NativeProbeError> {
            check_mjb(mjb)?;
            let path = dll_path(dll)?;
            let bytes =
                i32::try_from(mjb.len()).map_err(|_| InputError::Overflow { field: "mjb" })?;
            let mut raw = std::ptr::null_mut();
            let mut info = NativeModelInfo::default();
            let mut detail = 0;
            // SAFETY: path is absolute, NUL-terminated and live; MJB is trusted.
            // All output pointers reference initialized, correctly aligned DTOs.
            let status = unsafe {
                mjwarp_native_open(
                    path.as_ptr(),
                    mjb.as_ptr().cast(),
                    bytes,
                    &mut raw,
                    &mut info,
                    &mut detail,
                )
            };
            native_status(status, detail)?;
            let owner = NonNull::new(raw).ok_or(NativeProbeError::AbiMismatch)?;
            let probe = Self {
                owner,
                info,
                _thread: PhantomData,
            };
            probe.info.validate()?; // Drop closes native resources on validation failure.
            Ok(probe)
        }

        pub fn info(&self) -> NativeModelInfo {
            self.info
        }

        /// 先检查全部目标长度。
        /// 输入错误不改写任何目标。
        pub fn copy_core_into(
            &self,
            qpos0: &mut [f64],
            body_mass: &mut [f64],
            body_parentid: &mut [i32],
            jnt_type: &mut [i32],
        ) -> Result<(), NativeProbeError> {
            for (field, expected, actual) in [
                ("qpos0", self.info.nq, qpos0.len()),
                ("body_mass", self.info.nbody, body_mass.len()),
                ("body_parentid", self.info.nbody, body_parentid.len()),
                ("jnt_type", self.info.njnt, jnt_type.len()),
            ] {
                super::check_length(field, expected as usize, actual)?;
            }
            // SAFETY: owned immutable model and live DLL; disjoint Rust slices
            // have exact validated lengths. C++ prechecks before any memcpy.
            let status = unsafe {
                mjwarp_native_copy(
                    self.owner.as_ptr(),
                    qpos0.as_mut_ptr(),
                    qpos0.len() as u64,
                    body_mass.as_mut_ptr(),
                    body_parentid.as_mut_ptr(),
                    body_mass.len() as u64,
                    jnt_type.as_mut_ptr(),
                    jnt_type.len() as u64,
                )
            };
            native_status(status, 0)
        }

        pub fn snapshot(&self) -> Result<NativeCoreSnapshot, NativeProbeError> {
            let mut snapshot = NativeCoreSnapshot {
                info: self.info,
                qpos0: staging(self.info.nq as usize)?,
                body_mass: staging(self.info.nbody as usize)?,
                body_parentid: staging(self.info.nbody as usize)?,
                jnt_type: staging(self.info.njnt as usize)?,
            };
            self.copy_core_into(
                &mut snapshot.qpos0,
                &mut snapshot.body_mass,
                &mut snapshot.body_parentid,
                &mut snapshot.jnt_type,
            )?;
            Ok(snapshot)
        }

        /// 复制十二项运动学字段。
        /// 快照不借用模型或DLL。
        pub fn kinematic_snapshot(
            &self,
        ) -> Result<super::NativeKinematicSnapshot, NativeProbeError> {
            // SAFETY: the unique model and its DLL remain live during this call.
            // No API mutates the model; capture owns all disjoint output arrays.
            unsafe { super::NativeKinematicSnapshot::capture(self.owner.as_ptr(), self.info) }
        }

        /// 复制二十一项模型字段。
        /// 快照不借用模型或DLL。
        pub fn inertial_snapshot(&self) -> Result<super::NativeInertialSnapshot, NativeProbeError> {
            let kinematics = self.kinematic_snapshot()?;
            // SAFETY: this thread-confined owner keeps its immutable model and DLL
            // live through both copies. No public API can mutate the native model.
            unsafe { super::NativeInertialSnapshot::capture(self.owner.as_ptr(), kinematics) }
        }

        /// 复制十二项柔体位置字段。
        /// 快照不借用模型或DLL。
        pub fn flex_position_snapshot(
            &self,
        ) -> Result<super::NativeFlexPositionSnapshot, NativeProbeError> {
            // SAFETY: this thread-confined owner retains its immutable model and
            // DLL; capture preflights counts and owns all disjoint target arrays.
            unsafe { super::NativeFlexPositionSnapshot::capture(self.owner.as_ptr(), self.info) }
        }

        /// 复制同一模型的全部G01字段。
        /// 快照不借用原生模型或DLL。
        pub fn g01_snapshot(&self) -> Result<super::NativeG01Snapshot, NativeProbeError> {
            let inertial = self.inertial_snapshot()?;
            let positions = self.flex_position_snapshot()?;
            // SAFETY: Self retains the immutable owner and DLL on this thread.
            // Each snapshot owns separate buffers; the bridge checks its field ABI.
            unsafe { super::NativeG01Snapshot::capture(self.owner.as_ptr(), inertial, positions) }
        }
    }

    impl Drop for NativeModelProbe {
        fn drop(&mut self) {
            // SAFETY: unique valid owner; close deletes model before unloading DLL.
            unsafe { mjwarp_native_close(self.owner.as_ptr()) };
        }
    }

    fn dll_path(dll: &Path) -> Result<Vec<NativePathChar>, NativeProbeError> {
        #[cfg(target_os = "windows")]
        let mut path: Vec<u16> = dll.as_os_str().encode_wide().collect();
        #[cfg(target_os = "linux")]
        let mut path: Vec<NativePathChar> = dll
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|&byte| byte as NativePathChar)
            .collect();
        if !dll.is_absolute() || path.contains(&0) {
            return Err(NativeProbeError::InvalidPath);
        }
        path.push(0);
        Ok(path)
    }

    fn check_mjb(mjb: &[u8]) -> Result<(), NativeProbeError> {
        if mjb.len() < 20 {
            return Err(NativeProbeError::InvalidMjbHeader);
        }
        let read = |offset| i32::from_le_bytes(mjb[offset..offset + 4].try_into().unwrap());
        if read(0) != 54321 || read(4) != 8 || read(8) <= 0 || read(16) <= 0 {
            return Err(NativeProbeError::InvalidMjbHeader);
        }
        if read(12) != NativeModelInfo::VERSION {
            return Err(NativeProbeError::VersionMismatch {
                expected: NativeModelInfo::VERSION,
                actual: read(12),
            });
        }
        Ok(())
    }

    fn native_status(status: i32, detail: u32) -> Result<(), NativeProbeError> {
        let stage = match status {
            0 => return Ok(()),
            2 => {
                return Err(NativeProbeError::VersionMismatch {
                    expected: NativeModelInfo::VERSION,
                    actual: detail as i32,
                });
            }
            1 => "load_library",
            3 => "mj_version",
            4 => "mj_loadModelBuffer",
            5 => "mj_deleteModel",
            6 => "load_mjb",
            7 => "allocate_owner",
            8 => "copy_core",
            _ => "bridge_status",
        };
        Err(NativeProbeError::Native {
            stage,
            code: if detail == 0 { status as u32 } else { detail },
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn rejects_untrusted_header_shapes_before_native_calls() {
            for bytes in [vec![], vec![0; 19], vec![0; 20]] {
                assert_eq!(check_mjb(&bytes), Err(NativeProbeError::InvalidMjbHeader));
            }
            let mut header = [54321_i32, 8, 84, NativeModelInfo::VERSION, 1]
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .collect::<Vec<_>>();
            check_mjb(&header).unwrap();
            header[12..16].copy_from_slice(&123_i32.to_le_bytes());
            assert!(matches!(
                check_mjb(&header),
                Err(NativeProbeError::VersionMismatch { actual: 123, .. })
            ));
        }
        #[test]
        fn rejects_relative_and_nul_dll_paths() {
            assert_eq!(
                dll_path(Path::new("mujoco.dll")),
                Err(NativeProbeError::InvalidPath)
            );
            assert_eq!(
                dll_path(Path::new("C:\\bad\0.dll")),
                Err(NativeProbeError::InvalidPath)
            );
            assert_eq!(
                dll_path(Path::new("/bad\0.so")),
                Err(NativeProbeError::InvalidPath)
            );
        }
        #[cfg(target_os = "linux")]
        #[test]
        fn preserves_non_utf8_linux_library_paths() {
            let bytes = b"/trusted/\xff/mujoco.so";
            let path = Path::new(std::ffi::OsStr::from_bytes(bytes));
            let encoded = dll_path(path).unwrap();
            assert_eq!(encoded.last(), Some(&0));
            assert_eq!(
                encoded[..encoded.len() - 1]
                    .iter()
                    .map(|&byte| byte as u8)
                    .collect::<Vec<_>>(),
                bytes
            );
        }
        #[test]
        fn checks_snapshot_capacity_and_native_status() {
            assert!(staging::<f64>(usize::MAX).is_err());
            assert!(staging::<i32>(0).unwrap().is_empty());
            assert!(matches!(
                native_status(4, 0),
                Err(NativeProbeError::Native {
                    stage: "mj_loadModelBuffer",
                    ..
                })
            ));
        }
    }
}
