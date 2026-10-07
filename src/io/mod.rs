//! 宿主转换与连续GPU字段交换。
//! 不提供完整模型上传入口。

use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
use crate::runtime::{TransferBuffer, TransferElement, TransferSession};

mod kinematic;
pub use kinematic::DeviceKinematicModel;
#[cfg(feature = "native-model-probe")]
pub use kinematic::NativeKinematicSnapshot;

/// 连续批量字段的GPU传输辅助。
/// 不解释物理字段或模型身份。
/// 不替代Model、Data或put_data。
///
/// ```no_run
/// use mjwarp_rs::{diagnostics::TransferError, io::DeviceBatch,
///     model::BatchLayout, runtime::TransferSession};
/// let session = TransferSession::new(0)?;
/// let layout = BatchLayout::new(2, 3, 4)?;
/// let mut field = DeviceBatch::upload(&session, layout, &[0.0_f32; 6])?;
/// field.write_world(1, &[1.0, 2.0, 3.0])?;
/// let mut world = [0.0; 3];
/// field.read_world_into(1, &mut world)?;
/// # Ok::<(), TransferError>(())
/// ```
pub struct DeviceBatch<T: TransferElement> {
    layout: BatchLayout,
    buffer: TransferBuffer<T>,
}

impl<T: TransferElement> DeviceBatch<T> {
    pub fn upload(
        session: &TransferSession,
        layout: BatchLayout,
        source: &[T],
    ) -> Result<Self, TransferError> {
        check_field_layout::<T>(layout, source.len())?;
        let buffer = session.upload(source)?;
        Ok(Self { layout, buffer })
    }

    pub fn layout(&self) -> BatchLayout {
        self.layout
    }

    pub fn read_into(&self, target: &mut [T]) -> Result<(), TransferError> {
        check_length("target", self.layout.total_elements(), target.len())?;
        self.buffer.read_range_into(0, target)
    }

    pub fn read_world_into(&self, world: usize, target: &mut [T]) -> Result<(), TransferError> {
        let range = self.layout.world_elements(world)?;
        check_length("target", range.len(), target.len())?;
        self.buffer.read_range_into(range.start, target)
    }

    pub fn write_world(&mut self, world: usize, source: &[T]) -> Result<(), TransferError> {
        let range = self.layout.world_elements(world)?;
        check_length("source", range.len(), source.len())?;
        self.buffer.write_range(range.start, source)
    }

    pub fn copy_world_from(
        &mut self,
        world: usize,
        source: &Self,
        source_world: usize,
    ) -> Result<(), TransferError> {
        let target = self.layout.world_elements(world)?;
        let input = source.layout.world_elements(source_world)?;
        check_length("source_world", target.len(), input.len())?;
        self.buffer
            .copy_range_from(target.start, &source.buffer, input.start, target.len())
    }

    pub fn copy_world_within(
        &mut self,
        world: usize,
        source_world: usize,
    ) -> Result<(), TransferError> {
        let target = self.layout.world_elements(world)?;
        let source = self.layout.world_elements(source_world)?;
        self.buffer
            .copy_range_within(target.start, source.start, target.len())
    }
}

/// 先执行严格宿主转换再上传。
/// 该入口不替代等价状态导入。
pub fn upload_f64_batch(
    session: &TransferSession,
    layout: BatchLayout,
    source: &[f64],
) -> Result<DeviceBatch<f32>, TransferError> {
    check_field_layout::<f32>(layout, source.len())?;
    let mut values = crate::runtime::host_staging::<f32>(source.len())?;
    convert_f64_to_f32_into(source, &mut values)?;
    DeviceBatch::upload(session, layout, &values)
}

fn check_field_layout<T: TransferElement>(
    layout: BatchLayout,
    elements: usize,
) -> Result<(), InputError> {
    if layout.element_bytes() != size_of::<T>() {
        return Err(InputError::InvalidDimension {
            field: "element_bytes",
        });
    }
    check_length("source", layout.total_elements(), elements)
}

/// 先检查全量输入，再转换。
/// 拒绝非有限值与f32溢出。
/// 允许舍入、下溢和负零。
/// 此严格辅助接口不替代put_data。
pub fn convert_f64_to_f32_into(source: &[f64], target: &mut [f32]) -> Result<(), InputError> {
    check_length("target", source.len(), target.len())?;
    for (index, &value) in source.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "source",
                index,
            });
        }
        if !(value as f32).is_finite() {
            return Err(InputError::ScalarOverflow {
                field: "source",
                index,
            });
        }
    }
    for (&value, output) in source.iter().zip(target) {
        *output = value as f32;
    }
    Ok(())
}

/// 复制一个世界的连续f32字段。
/// 保留全部位模式。
/// 任意输入错误都不改写目标。
pub fn copy_world_f32(
    layout: BatchLayout,
    world: usize,
    source: &[f32],
    target: &mut [f32],
) -> Result<(), InputError> {
    if layout.element_bytes() != size_of::<f32>() {
        return Err(InputError::InvalidDimension {
            field: "element_bytes",
        });
    }
    let range = layout.world_elements(world)?;
    check_length("source", layout.elements_per_world(), source.len())?;
    check_length("target", layout.total_elements(), target.len())?;
    target[range].copy_from_slice(source);
    Ok(())
}

fn check_length(field: &'static str, expected: usize, actual: usize) -> Result<(), InputError> {
    if expected != actual {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}

#[cfg(feature = "native-model-probe")]
pub use native::{NativeCoreSnapshot, NativeModelProbe};

#[cfg(feature = "native-model-probe")]
mod native {
    use crate::{
        diagnostics::{InputError, NativeProbeError},
        model::NativeModelInfo,
    };
    use std::{
        ffi::c_void, marker::PhantomData, os::windows::ffi::OsStrExt, path::Path, ptr::NonNull,
        rc::Rc,
    };

    unsafe extern "C" {
        fn mjwarp_native_open(
            path: *const u16,
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
    }

    impl Drop for NativeModelProbe {
        fn drop(&mut self) {
            // SAFETY: unique valid owner; close deletes model before unloading DLL.
            unsafe { mjwarp_native_close(self.owner.as_ptr()) };
        }
    }

    pub(super) fn staging<T: Copy + Default>(count: usize) -> Result<Vec<T>, NativeProbeError> {
        let bytes = count
            .checked_mul(size_of::<T>())
            .filter(|v| *v <= isize::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "native_snapshot",
            })?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| NativeProbeError::HostAllocation { bytes })?;
        values.resize(count, T::default());
        Ok(values)
    }

    fn dll_path(dll: &Path) -> Result<Vec<u16>, NativeProbeError> {
        let mut path: Vec<u16> = dll.as_os_str().encode_wide().collect();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_three_device_field_types_and_empty_layouts() {
        let layout = BatchLayout::new(3, 5, 4).unwrap();
        check_field_layout::<f32>(layout, 15).unwrap();
        check_field_layout::<i32>(layout, 15).unwrap();
        check_field_layout::<u32>(layout, 15).unwrap();
        check_field_layout::<f32>(BatchLayout::new(2, 0, 4).unwrap(), 0).unwrap();
    }

    #[test]
    fn rejects_device_field_size_and_length() {
        assert_eq!(
            check_field_layout::<f32>(BatchLayout::new(2, 3, 8).unwrap(), 6),
            Err(InputError::InvalidDimension {
                field: "element_bytes"
            })
        );
        assert_eq!(
            check_field_layout::<u32>(BatchLayout::new(2, 3, 4).unwrap(), 5),
            Err(InputError::LengthMismatch {
                field: "source",
                expected: 6,
                actual: 5
            })
        );
    }

    #[test]
    fn converts_with_rounding_underflow_and_negative_zero() {
        let source = [
            1.5,
            -0.0,
            f64::MIN_POSITIVE,
            f64::from(f32::MAX),
            16_777_217.0,
        ];
        let mut target = [9.0; 5];
        convert_f64_to_f32_into(&source, &mut target).unwrap();
        assert_eq!(target, [1.5, -0.0, 0.0, f32::MAX, 16_777_216.0]);
        assert_eq!(target[1].to_bits(), (-0.0_f32).to_bits());
        convert_f64_to_f32_into(&[], &mut []).unwrap();
    }

    #[test]
    fn rejects_non_finite_inputs_without_partial_writes() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut target = [7.0; 3];
            assert_eq!(
                convert_f64_to_f32_into(&[1.0, 2.0, invalid], &mut target),
                Err(InputError::NonFinite {
                    field: "source",
                    index: 2
                })
            );
            assert_eq!(target, [7.0; 3]);
        }
    }

    #[test]
    fn rejects_conversion_overflow_without_partial_writes() {
        for invalid in [f64::MAX, -f64::MAX] {
            let mut target = [7.0; 2];
            assert_eq!(
                convert_f64_to_f32_into(&[1.0, invalid], &mut target),
                Err(InputError::ScalarOverflow {
                    field: "source",
                    index: 1
                })
            );
            assert_eq!(target, [7.0; 2]);
        }
    }

    #[test]
    fn rejects_conversion_length_without_writes() {
        let mut target = [7.0];
        assert_eq!(
            convert_f64_to_f32_into(&[1.0, 2.0], &mut target),
            Err(InputError::LengthMismatch {
                field: "target",
                expected: 2,
                actual: 1
            })
        );
        assert_eq!(target, [7.0]);
    }

    #[test]
    fn copies_one_world_and_preserves_bits() {
        let layout = BatchLayout::new(3, 2, 4).unwrap();
        let source = [f32::from_bits(0x7fc0_0017), -0.0];
        let mut target = [7.0; 6];
        copy_world_f32(layout, 1, &source, &mut target).unwrap();
        assert_eq!(&target[..2], &[7.0; 2]);
        assert_eq!(&target[4..], &[7.0; 2]);
        assert_eq!(target[2].to_bits(), source[0].to_bits());
        assert_eq!(target[3].to_bits(), source[1].to_bits());
        copy_world_f32(BatchLayout::new(2, 0, 4).unwrap(), 1, &[], &mut []).unwrap();
    }

    #[test]
    fn rejects_copy_inputs_without_writes() {
        let mut target = [7.0; 4];
        for (layout, world, source) in [
            (BatchLayout::new(2, 2, 8).unwrap(), 0, &[1.0, 2.0][..]),
            (BatchLayout::new(2, 2, 4).unwrap(), 2, &[1.0, 2.0][..]),
            (BatchLayout::new(2, 2, 4).unwrap(), 1, &[1.0][..]),
            (BatchLayout::new(2, 3, 4).unwrap(), 1, &[1.0, 2.0, 3.0][..]),
        ] {
            assert!(copy_world_f32(layout, world, source, &mut target).is_err());
            assert_eq!(target, [7.0; 4]);
        }
    }
}
