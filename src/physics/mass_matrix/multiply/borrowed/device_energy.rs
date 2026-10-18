//! 严格当前态Mqvel视图接入默认32线程动能。
//! GPU内成对复制，不采用宿主物理回传。
use super::{MassMatrixMultiplyPlan, MassMatrixMultiplyView, Spec};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::BatchLayout;
#[cfg(feature = "cuda-probe")]
use crate::physics::kinetic_energy::{
    KINETIC_METADATA_KIND, KineticEnergy32Kernel, KineticEnergyMetadataBuffer,
    KineticValidationKernel,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

struct EnergySpec {
    mass: Arc<Spec>,
}
#[derive(Clone, Copy)]
struct Layout {
    pair: BatchLayout,
    energy: BatchLayout,
}
fn layout(n: usize, worlds: usize) -> Result<Layout, InputError> {
    if !(1..=1024).contains(&n) || worlds == 0 || worlds > i32::MAX as usize {
        return Err(InputError::InvalidDimension {
            field: "device_energy_layout",
        });
    }
    let pair = BatchLayout::new(worlds, 2 * n, 4)?;
    let energy = BatchLayout::new(worlds, 2, 4)?;
    for count in [pair.total_elements(), energy.total_elements(), worlds] {
        count
            .checked_add(8)
            .and_then(|v| v.checked_mul(4))
            .ok_or(InputError::Overflow {
                field: "device_energy_guards",
            })?;
    }
    Ok(Layout { pair, energy })
}
fn binding(plan: &Arc<EnergySpec>, data: &Arc<EnergySpec>, mass: &Arc<Spec>) -> bool {
    Arc::ptr_eq(plan, data) && Arc::ptr_eq(&plan.mass, mass)
}
fn all_current(flags: &[bool]) -> Result<(), TransferError> {
    if flags.is_empty() || flags.iter().any(|&v| !v) {
        return Err(TransferError::StageNotReady {
            stage: "device_energy_all_current",
        });
    }
    Ok(())
}

#[cfg(feature = "cuda-probe")]
const SOURCE_VALIDATION_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::AddressBearing {
            proof: KINETIC_METADATA_KIND,
        },
    },
    SlotDesc {
        index: 1,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 2,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 3,
        kind: SlotKind::DevicePtr {
            elem: ElemType::F32,
        },
        access: Access::Write,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 4,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 5,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 6,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
    SlotDesc {
        index: 7,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    },
];

#[cfg(feature = "cuda-probe")]
struct DeviceEnergySourceValidationSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结校验ABI。
unsafe impl KernelAbi for DeviceEnergySourceValidationSpec {
    const NAME: &'static str = "device_energy_source_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &SOURCE_VALIDATION_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for DeviceEnergySourceValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (source, status) = source_validation_lengths(dims)?;
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 1,
            },
            SlotRequirement {
                slot: 1,
                min_elements: source,
            },
            SlotRequirement {
                slot: 2,
                min_elements: source,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        source_validation_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn source_validation_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [nv, worlds, reserved, work] = *dims;
    if !(1..=1024).contains(&nv)
        || worlds == 0
        || worlds > i32::MAX as u32
        || reserved != 0
        || work != worlds
    {
        return Err(LauncherError::InvalidArray("设备动能源校验维度无效"));
    }
    let source = u64::from(worlds)
        .checked_mul(u64::from(nv))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("device_energy_source_elements"))?;
    let status = u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("device_energy_status_elements"))?;
    Ok((source, status))
}

#[cfg(feature = "cuda-probe")]
fn source_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        LauncherError::StaleMetadata => InputError::InvalidTopology {
            field: "device_energy_source_metadata",
            index: 0,
            reason: "stale_proof",
        }
        .into(),
        _ => InputError::InvalidDimension {
            field: "device_energy_source_validate_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
struct DeviceEnergySourceValidationKernel {
    kernel: SynchronousKernel,
    nv: u32,
}

#[cfg(feature = "cuda-probe")]
impl DeviceEnergySourceValidationKernel {
    fn compile(session: &TransferSession, nv: u32) -> Result<Self, TransferError> {
        source_validation_lengths(&[nv, 1, 0, 1]).map_err(source_launch_error)?;
        let source = format!("#define ENERGY_NV {nv}\n{SOURCE_VALIDATE}");
        Ok(Self {
            kernel: SynchronousKernel::compile(session, &source, "device_energy_source_validate")?,
            nv,
        })
    }

    fn launch(
        &self,
        metadata: &KineticEnergyMetadataBuffer,
        q: &TransferBuffer<f32>,
        mq: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        if dims[0] != self.nv || metadata.nv() != self.nv {
            return Err(InputError::InvalidDimension {
                field: "device_energy_source_validate_static_nv",
            }
            .into());
        }
        let (source_len, status_len) =
            source_validation_lengths(&dims).map_err(source_launch_error)?;
        if metadata.len() != 1
            || u64::try_from(q.len()).ok() != Some(source_len)
            || u64::try_from(mq.len()).ok() != Some(source_len)
            || u64::try_from(status.len()).ok() != Some(status_len)
        {
            return Err(InputError::InvalidDimension {
                field: "device_energy_source_validate_abi",
            }
            .into());
        }
        let proof = metadata.proof(dims)?;
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.as_buffer().launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: q.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: mq.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        Kernel::<DeviceEnergySourceValidationSpec>::new()
            .and_then(|kernel| kernel.plan(&dims, &bindings, &[proof.binding(0)]))
            .map_err(source_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度。
        unsafe {
            self.kernel
                .launch(metadata.as_buffer(), q, mq, status, dims)
        }
    }
}

pub struct MassMatrixEnergyPlan<'plan, 'matrix> {
    multiply: &'plan MassMatrixMultiplyPlan<'matrix>,
    spec: Arc<EnergySpec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: KineticEnergyMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    source_validator: DeviceEnergySourceValidationKernel,
    #[cfg(feature = "cuda-probe")]
    scalar_validator: KineticValidationKernel,
    #[cfg(feature = "cuda-probe")]
    reducer: KineticEnergy32Kernel,
}
pub struct MassMatrixEnergyData {
    spec: Arc<EnergySpec>,
    layout: Layout,
    #[cfg(feature = "cuda-probe")]
    pair: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
/// 同时借用原乘积视图与能量数据，不认证未来状态。
/// 当前态证明仅覆盖动能；势能由调用方提供。
pub struct MassMatrixEnergyView<'energy, 'source> {
    input: &'energy MassMatrixMultiplyView<'source>,
    data: &'energy MassMatrixEnergyData,
}
#[derive(Clone, Debug)]
pub struct MassMatrixEnergySnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}
impl MassMatrixEnergySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    /// 返回历史[势能,动能]，不携带伪造状态标签。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl MassMatrixEnergyView<'_, '_> {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.input.worlds(), self.data.worlds());
        self.input.worlds()
    }
    pub fn snapshot(&self) -> Result<MassMatrixEnergySnapshot, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(MassMatrixEnergySnapshot {
                layout: self.data.layout.energy,
                values: super::read(
                    &self.data.current,
                    self.data.layout.energy.total_elements(),
                    "device_energy",
                )?,
            })
        }
    }
}
impl<'plan, 'matrix> MassMatrixEnergyPlan<'plan, 'matrix> {
    pub fn new(multiply: &'plan MassMatrixMultiplyPlan<'matrix>) -> Result<Self, TransferError> {
        let n = multiply.spec.nv;
        layout(n, 1)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let session = multiply.session.clone();
            Ok(Self {
                multiply,
                spec: Arc::new(EnergySpec {
                    mass: multiply.spec.clone(),
                }),
                metadata: KineticEnergyMetadataBuffer::upload(&session, n as u32)?,
                source_validator: DeviceEnergySourceValidationKernel::compile(&session, n as u32)?,
                scalar_validator: KineticValidationKernel::compile(&session, n as u32)?,
                reducer: KineticEnergy32Kernel::compile(&session, n as u32)?,
                session,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMatrixEnergyData, TransferError> {
        let l = layout(self.spec.mass.nv, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = l;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |count: usize, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            let mut current = alloc(l.energy.total_elements(), f32::NAN)?;
            for w in 0..worlds {
                current.write_range(4 + 2 * w, &[0.0])?;
            }
            Ok(MassMatrixEnergyData {
                spec: self.spec.clone(),
                layout: l,
                pair: alloc(l.pair.total_elements(), 0.0)?,
                current,
                staged: alloc(l.energy.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update<'energy, 'source>(
        &self,
        data: &'energy mut MassMatrixEnergyData,
        input: &'energy MassMatrixMultiplyView<'source>,
    ) -> Result<MassMatrixEnergyView<'energy, 'source>, TransferError> {
        if !binding(&self.spec, &data.spec, &input.data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        self.multiply.matrix.check_data(input.source)?;
        if input.worlds() != data.worlds() || input.computed.len() != data.worlds() {
            return Err(InputError::InvalidDimension {
                field: "device_energy_worlds",
            }
            .into());
        }
        all_current(&input.computed)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Recheck the actual source ticket; historical solver or energy
            // snapshots never substitute for the live matrix/source borrow.
            input.source.matrix_input()?;
            let n = self.spec.mass.nv;
            let worlds = data.worlds();
            if input.data.vector.len() != worlds * n + 8
                || input.data.current.len() != worlds * n + 8
            {
                return Err(TransferError::ModelMismatch);
            }
            let dims = [n as u32, worlds as u32, 0, worlds as u32];
            self.source_validator.launch(
                &self.metadata,
                &input.data.vector,
                &input.data.current,
                &mut data.status,
                dims,
            )?;
            status(&data.status, worlds)?;
            // These are explicit GPU-to-GPU copies, not a zero-copy pair and
            // never a download/upload round trip of physical values.
            for w in 0..worlds {
                data.pair
                    .copy_range_from(4 + 2 * w * n, &input.data.vector, 4 + w * n, n)?;
                data.pair.copy_range_from(
                    4 + (2 * w + 1) * n,
                    &input.data.current,
                    4 + w * n,
                    n,
                )?;
            }
            self.scalar_validator.launch(
                &self.metadata,
                &data.pair,
                &data.current,
                &mut data.status,
                dims,
            )?;
            status(&data.status, worlds)?;
            self.reducer.launch(
                &self.metadata,
                &data.pair,
                &data.current,
                &mut data.staged,
                dims,
            )?;
            super::read(
                &data.staged,
                data.layout.energy.total_elements(),
                "device_energy_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            Ok(MassMatrixEnergyView { input, data })
        }
    }
}
impl MassMatrixEnergyData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.layout.pair.worlds(), self.layout.energy.worlds());
        self.layout.energy.worlds()
    }
    pub fn write_potential(&mut self, values: &[f32]) -> Result<(), TransferError> {
        super::super::checked(values, self.worlds(), "device_energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            for (w, &v) in values.iter().enumerate() {
                self.current.write_range(4 + 2 * w, &[v])?;
            }
            Ok(())
        }
    }
    pub fn write_world_potential(&mut self, world: usize, value: f32) -> Result<(), TransferError> {
        self.layout.energy.world_elements(world)?;
        super::super::checked(&[value], 1, "device_energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.current.write_range(4 + 2 * world, &[value])
        }
    }
}
#[cfg(feature = "cuda-probe")]
fn status(buffer: &TransferBuffer<f32>, worlds: usize) -> Result<(), TransferError> {
    let values = super::read(buffer, worlds, "device_energy_status")?;
    if let Some(index) = values[4..4 + worlds].iter().position(|&v| v != 1.0) {
        return Err(InputError::InvalidTopology {
            field: "device_energy_source",
            index,
            reason: "nonfinite_guard_or_shape",
        }
        .into());
    }
    Ok(())
}
#[cfg(feature = "cuda-probe")]
const SOURCE_VALIDATE: &str = r#"
extern "C" __global__ void device_energy_source_validate(const int* meta,const float* q,const float* mq,float* status,
 unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=n==ENERGY_NV&&meta[0]==ENERGY_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&q[j]==-131072.0f&&mq[j]==-131072.0f&&q[4+1ull*n*worlds+j]==-131072.0f&&mq[4+1ull*n*worlds+j]==-131072.0f;
 for(unsigned j=0;j<n;++j)valid=valid&&isfinite(q[4+1ull*w*n+j])&&isfinite(mq[4+1ull*w*n+j]);
 status[4+w]=valid?1.0f:0.0f;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cuda-probe")]
    #[test]
    fn typed_source_validation_rejects_capacity_alias_session_and_stale_proof() {
        use crate::physics::kinetic_energy::KineticLaunchMetadata;
        use crate::runtime::launcher::{Checked, DeviceSpan, MetadataRevision};

        let dims = [65, 257, 0, 257];
        let (source_len, status_len) = source_validation_lengths(&dims).unwrap();
        let metadata_span = DeviceSpan::new(7, 0x1000, 4).unwrap();
        let revision = MetadataRevision::new();
        // SAFETY: 本测试绑定等价合成跨度。
        let metadata = unsafe {
            Checked::<KineticLaunchMetadata>::assume_uploaded(
                metadata_span,
                &[65],
                dims,
                revision.clone(),
            )
        }
        .unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata_span,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x10000, source_len * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x30000, source_len * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x50000, status_len * 4).unwrap(),
                elem: ElemType::F32,
            },
        ];
        let kernel = Kernel::<DeviceEnergySourceValidationSpec>::new().unwrap();
        let plan = kernel
            .plan(&dims, &bindings, &[metadata.binding(0)])
            .unwrap();
        assert_eq!(plan.dims().grid(), (2, 1, 1));
        assert_eq!(plan.dims().block(), (256, 1, 1));

        let mut capacity = bindings;
        capacity[1].span = DeviceSpan::new(7, 0x10000, source_len * 4 - 4).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &capacity, &[metadata.binding(0)]),
            Err(LauncherError::Capacity { slot: 1, .. })
        ));

        let mut alias = bindings;
        alias[3].span = alias[1].span;
        assert!(matches!(
            kernel.plan(&dims, &alias, &[metadata.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3,
            })
        ));

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x30000, source_len * 4).unwrap();
        assert!(matches!(
            kernel.plan(&dims, &foreign, &[metadata.binding(0)]),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));

        revision.invalidate();
        assert!(matches!(
            kernel.plan(&dims, &bindings, &[metadata.binding(0)]),
            Err(LauncherError::StaleMetadata)
        ));
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_source_validation_wrapper_binds_real_spans_and_metadata() {
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let validator = DeviceEnergySourceValidationKernel::compile(&session, 65).unwrap();
        let mut metadata = KineticEnergyMetadataBuffer::upload(&session, 65).unwrap();
        let foreign_metadata = KineticEnergyMetadataBuffer::upload(&foreign, 65).unwrap();
        let wrong_metadata = KineticEnergyMetadataBuffer::upload(&session, 64).unwrap();
        let dims = [65, 257, 0, 257];
        let mut source_values = vec![-131072.0; 65 * 257 + 8];
        let source_end = source_values.len() - 4;
        source_values[4..source_end].fill(0.25);
        let status_values = vec![-131072.0; 257 + 8];
        let q = session.upload(&source_values).unwrap();
        let mq = session.upload(&source_values).unwrap();
        let mut status = session.upload(&status_values).unwrap();

        validator
            .launch(&metadata, &q, &mq, &mut status, dims)
            .unwrap();
        let mut result = vec![0.0; status.len()];
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(&result[..4], &status_values[..4]);
        assert_eq!(
            &result[result.len() - 4..],
            &status_values[status_values.len() - 4..]
        );
        assert!(result[4..4 + 257].iter().all(|&value| value == 1.0));

        let short_q = session
            .upload(&source_values[..source_values.len() - 1])
            .unwrap();
        assert!(
            validator
                .launch(&metadata, &short_q, &mq, &mut status, dims)
                .is_err()
        );
        let foreign_q = foreign.upload(&source_values).unwrap();
        assert!(matches!(
            validator.launch(&metadata, &foreign_q, &mq, &mut status, dims),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            validator.launch(&foreign_metadata, &q, &mq, &mut status, dims),
            Err(TransferError::SessionMismatch)
        ));
        assert!(matches!(
            validator.launch(&wrong_metadata, &q, &mq, &mut status, dims),
            Err(TransferError::Input(InputError::InvalidDimension {
                field: "device_energy_source_validate_static_nv"
            }))
        ));
        assert!(matches!(
            validator.launch(&metadata, &q, &mq, &mut status, [64, 257, 0, 257]),
            Err(TransferError::Input(InputError::InvalidDimension {
                field: "device_energy_source_validate_static_nv"
            }))
        ));

        metadata.write_for_test(65).unwrap();
        assert!(matches!(
            validator.launch(&metadata, &q, &mq, &mut status, dims),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "kinetic_energy_metadata",
                reason: "stale_proof",
                ..
            }))
        ));
        let mut after = vec![0.0; status.len()];
        status.read_range_into(0, &mut after).unwrap();
        assert_eq!(result, after);
        metadata.restore_for_test().unwrap();
        validator
            .launch(&metadata, &q, &mq, &mut status, dims)
            .unwrap();
    }

    #[test]
    fn host_checks_pair_world_strides_and_original_scalar32_limits() {
        for n in [1, 3, 33, 65, 1024] {
            for w in [1, 2, 257] {
                let l = layout(n, w).unwrap();
                assert_eq!(l.pair.total_elements(), 2 * n * w);
                assert_eq!(l.energy.total_elements(), 2 * w);
                assert!(l.energy.world_elements(w).is_err());
            }
        }
        for (n, w) in [
            (0, 1),
            (1025, 1),
            (3, 0),
            (3, usize::MAX),
            (3, i32::MAX as usize + 1),
        ] {
            assert!(layout(n, w).is_err());
        }
    }
    #[test]
    fn host_requires_actual_owner_arcs_and_every_current_world() {
        let mass = Arc::new(Spec {
            nbody: 4,
            nv: 3,
            meta: vec![],
        });
        let plan = Arc::new(EnergySpec { mass: mass.clone() });
        let other = Arc::new(EnergySpec { mass: mass.clone() });
        let twin = Arc::new(Spec {
            nbody: 4,
            nv: 3,
            meta: vec![],
        });
        assert!(binding(&plan, &plan, &mass));
        assert!(!binding(&plan, &other, &mass));
        assert!(!binding(&plan, &plan, &twin));
        for flags in [
            &[][..],
            &[false][..],
            &[true, false][..],
            &[false, true][..],
        ] {
            assert!(all_current(flags).is_err());
        }
        all_current(&[true]).unwrap();
        all_current(&[true; 257]).unwrap();
    }
    #[cfg(feature = "cuda-probe")]
    fn raw(buffer: &TransferBuffer<f32>) -> Vec<f32> {
        let mut v = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut v).unwrap();
        v
    }
    #[cfg(feature = "cuda-probe")]
    fn bits(values: &[f32]) -> Vec<u32> {
        values.iter().map(|v| v.to_bits()).collect()
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new device-only liveM->Mqvel->energy connection, independent onehot exact bits; no old selector replay"]
    fn gpu_links_live_matrix_energy_with_exact_one_hot_and_device_pair_bits() {
        use super::super::tests::{input, model};
        use crate::physics::{MassMatrixPlan, MassMultiplyFormat};
        let session = TransferSession::new(0).unwrap();
        let start = std::time::Instant::now();
        let mut sources = 0;
        let mut products = 0;
        let mut batches = 0;
        let mut worlds_checked = 0;
        let mut pair_slots = 0;
        let mut capacity = 0;
        let mut last = None;
        for n in [3, 33, 65] {
            let i = input(n, true);
            let producer = MassMatrixPlan::new(&session, model(&i)).unwrap();
            let multiply = [
                MassMatrixMultiplyPlan::new(&producer, i.clone(), MassMultiplyFormat::Sparse)
                    .unwrap(),
                MassMatrixMultiplyPlan::new(&producer, i, MassMultiplyFormat::Dense).unwrap(),
            ];
            let plans = [
                MassMatrixEnergyPlan::new(&multiply[0]).unwrap(),
                MassMatrixEnergyPlan::new(&multiply[1]).unwrap(),
            ];
            for worlds in [1, 2, 257] {
                let mut source = producer.create_data(worlds).unwrap();
                let mut md = [
                    multiply[0].create_data(worlds).unwrap(),
                    multiply[1].create_data(worlds).unwrap(),
                ];
                let mut ed = [
                    plans[0].create_data(worlds).unwrap(),
                    plans[1].create_data(worlds).unwrap(),
                ];
                for state in 0..2 {
                    let pose: Vec<f32> = (0..worlds * n)
                        .map(|j| ((j + state) % 9) as f32 * 0.07 - 0.28)
                        .collect();
                    source.write_qpos(&pose).unwrap();
                    producer.update(&mut source).unwrap();
                    sources += 1;
                    let matrix = source.readback().unwrap();
                    if n == 3 && worlds == 2 && state == 0 {
                        for f in 0..2 {
                            let view = multiply[f]
                                .update(&mut md[f], &source, Some(&[true, true]))
                                .unwrap();
                            products += 1;
                            let before = bits(&raw(&ed[f].current));
                            assert!(plans[f].update(&mut ed[f], &view).is_err());
                            assert_eq!(before, bits(&raw(&ed[f].current)));
                        }
                    }
                    for vector in 0..2 {
                        let mut q = vec![0.0; worlds * n];
                        let hot: Vec<usize> =
                            (0..worlds).map(|w| (w + vector + state) % n).collect();
                        if vector == 1 {
                            for (w, &h) in hot.iter().enumerate() {
                                assert_ne!(h, (w + state) % n);
                            }
                        }
                        for w in 0..worlds {
                            q[w * n + hot[w]] = 1.0;
                        }
                        let p: Vec<f32> = (0..worlds)
                            .map(|w| {
                                if (w + state) % 7 == 0 {
                                    -0.0
                                } else {
                                    ((w + vector + state) % 19) as f32 * 0.25
                                }
                            })
                            .collect();
                        for f in 0..2 {
                            md[f].write_vector(&q).unwrap();
                            let mv = multiply[f].update(&mut md[f], &source, None).unwrap();
                            products += 1;
                            ed[f].write_potential(&p).unwrap();
                            let energy = plans[f].update(&mut ed[f], &mv).unwrap();
                            assert_eq!(energy.worlds(), worlds);
                            let out = energy.snapshot().unwrap();
                            assert_eq!(out.worlds(), worlds);
                            assert!(out.world(worlds).is_err());
                            let packed = raw(&energy.data.pair);
                            let mq = raw(&mv.data.current);
                            let srcq = raw(&mv.data.vector);
                            for w in 0..worlds {
                                assert_eq!(
                                    bits(&packed[4 + 2 * w * n..4 + (2 * w + 1) * n]),
                                    bits(&srcq[4 + w * n..4 + (w + 1) * n])
                                );
                                assert_eq!(
                                    bits(&packed[4 + (2 * w + 1) * n..4 + 2 * (w + 1) * n]),
                                    bits(&mq[4 + w * n..4 + (w + 1) * n])
                                );
                                pair_slots += 2 * n;
                                let diagonal = matrix.world(w).unwrap().matrix[hot[w] * n + hot[w]];
                                let word = diagonal.to_bits();
                                assert!(
                                    diagonal > 0.0
                                        && diagonal.is_normal()
                                        && ((word >> 23) & 255) > 1
                                );
                                // Independent binary-exponent certificate for
                                // onehot q: half the source diagonal, no energy
                                // candidate or Mqvel feeds this expected word.
                                let expected = word - 0x0080_0000;
                                assert_eq!(out.world(w).unwrap()[1].to_bits(), expected);
                                assert_eq!(out.world(w).unwrap()[0].to_bits(), p[w].to_bits());
                            }
                            let _ = energy;
                            if n == 3 && worlds == 2 && state == 0 && vector == 0 {
                                let mut wrong = plans[f].create_data(1).unwrap();
                                assert!(plans[f].update(&mut wrong, &mv).is_err());
                                let foreign = MassMatrixEnergyPlan::new(&multiply[f]).unwrap();
                                let before = bits(&raw(&ed[f].current));
                                assert!(foreign.update(&mut ed[f], &mv).is_err());
                                assert_eq!(before, bits(&raw(&ed[f].current)));
                                if f == 1 {
                                    let before = bits(&raw(&ed[0].current));
                                    assert!(plans[0].update(&mut ed[0], &mv).is_err());
                                    assert_eq!(before, bits(&raw(&ed[0].current)));
                                }
                            }
                            capacity = capacity.max(
                                4 * (ed[f].pair.len()
                                    + ed[f].current.len()
                                    + ed[f].staged.len()
                                    + ed[f].status.len()
                                    + plans[f].metadata.len()),
                            );
                            last = Some(out);
                            batches += 1;
                            worlds_checked += worlds;
                        }
                    }
                }
                if n == 3 && worlds == 2 {
                    for f in 0..2 {
                        md[f].write_world_vector(1, &[0.25; 3]).unwrap();
                        let mv = multiply[f]
                            .update(&mut md[f], &source, Some(&[true, false]))
                            .unwrap();
                        products += 1;
                        let before = bits(&raw(&ed[f].current));
                        assert!(plans[f].update(&mut ed[f], &mv).is_err());
                        assert_eq!(before, bits(&raw(&ed[f].current)));
                        drop(mv);
                        md[f].write_vector(&[-0.0; 6]).unwrap();
                        let mv = multiply[f].update(&mut md[f], &source, None).unwrap();
                        products += 1;
                        let out = plans[f]
                            .update(&mut ed[f], &mv)
                            .unwrap()
                            .snapshot()
                            .unwrap();
                        for w in 0..2 {
                            assert_eq!(out.world(w).unwrap()[1].to_bits(), 0x80000000);
                        }
                    }
                }
            }
        }
        drop(session);
        assert!(
            last.unwrap()
                .world(256)
                .unwrap()
                .iter()
                .all(|v| v.is_finite())
        );
        assert_eq!(
            (batches, worlds_checked, sources, products),
            (72, 6240, 18, 78)
        );
        assert_eq!(pair_slots, 420160);
        assert_eq!(capacity, 138912);
        println!(
            "device_energy batches={batches} worlds={worlds_checked} bit_difference=0 pair_bit_slots={pair_slots} negative_zero_worlds=4 source_dependency_updates={sources} Mqvel_dependency_updates={products} phase_ms={} energy_owned_capacity_bytes={capacity} pair_D2D_copies_per_update=2W M_host_feedback=0",
            start.elapsed().as_millis()
        );
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new device-energy private raw-source/pair/output guards and finite/overflow transactional faults"]
    fn gpu_rejects_corrupted_device_inputs_and_stale_source_without_partial_energy() {
        use super::super::tests::{input, model};
        use crate::physics::{MassMatrixPlan, MassMultiplyFormat};
        let s = TransferSession::new(0).unwrap();
        let i = input(3, true);
        let producer = MassMatrixPlan::new(&s, model(&i)).unwrap();
        let mut source = producer.create_data(2).unwrap();
        producer.update(&mut source).unwrap();
        let multiply =
            MassMatrixMultiplyPlan::new(&producer, i, MassMultiplyFormat::Sparse).unwrap();
        let mut md = multiply.create_data(2).unwrap();
        md.write_vector(&[0.5; 6]).unwrap();
        drop(multiply.update(&mut md, &source, None).unwrap());
        let mut plan = MassMatrixEnergyPlan::new(&multiply).unwrap();
        let mut data = plan.create_data(2).unwrap();
        // Test-only forged tickets let fault injection emulate corrupted GPU
        // buffers without rerunning a frozen producer or claiming that public
        // callers can construct/forge the sealed current-view provenance.
        fn ticket<'a>(
            source: &'a crate::physics::MassMatrixData,
            data: &'a super::super::MassMatrixMultiplyData,
        ) -> MassMatrixMultiplyView<'a> {
            MassMatrixMultiplyView {
                source,
                data,
                computed: vec![true; 2],
            }
        }
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        let mut faults = 0;
        for field in 0..4 {
            for edge in 0..8 {
                let b = match field {
                    0 => &mut data.pair,
                    1 => &mut data.current,
                    2 => &mut data.staged,
                    _ => &mut data.status,
                };
                let index = if edge < 4 { edge } else { b.len() - 8 + edge };
                b.write_range(index, &[99.0]).unwrap();
                let before = bits(&raw(&data.current));
                assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
                assert_eq!(before, bits(&raw(&data.current)));
                let b = match field {
                    0 => &mut data.pair,
                    1 => &mut data.current,
                    2 => &mut data.staged,
                    _ => &mut data.status,
                };
                b.write_range(index, &[-131072.0]).unwrap();
                let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
                faults += 1;
            }
        }
        for field in 0..2 {
            for edge in 0..8 {
                let b = if field == 0 {
                    &mut md.vector
                } else {
                    &mut md.current
                };
                let index = if edge < 4 { edge } else { b.len() - 8 + edge };
                b.write_range(index, &[99.0]).unwrap();
                let before = bits(&raw(&data.current));
                assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
                assert_eq!(before, bits(&raw(&data.current)));
                let b = if field == 0 {
                    &mut md.vector
                } else {
                    &mut md.current
                };
                b.write_range(index, &[-131072.0]).unwrap();
                let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
                faults += 1;
            }
        }
        for (field, index, bad) in [
            (0, 4, f32::NAN),
            (0, 9, f32::INFINITY),
            (1, 4, f32::NAN),
            (1, 9, f32::INFINITY),
        ] {
            let b = if field == 0 {
                &mut md.vector
            } else {
                &mut md.current
            };
            let old = raw(b)[index];
            b.write_range(index, &[bad]).unwrap();
            let before = bits(&raw(&data.current));
            assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
            assert_eq!(before, bits(&raw(&data.current)));
            let b = if field == 0 {
                &mut md.vector
            } else {
                &mut md.current
            };
            b.write_range(index, &[old]).unwrap();
            let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
            faults += 1;
        }
        plan.metadata.write_for_test(0).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        plan.metadata.restore_for_test().unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        faults += 1;
        let old = raw(&data.current)[4];
        data.current.write_range(4, &[f32::NAN]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        data.current.write_range(4, &[old]).unwrap();
        faults += 1;
        data.current.write_range(5, &[f32::NAN]).unwrap();
        data.pair.write_range(4, &[f32::NAN]).unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        let q = raw(&md.vector);
        let mq = raw(&md.current);
        md.vector.write_range(7, &[f32::MAX]).unwrap();
        md.current.write_range(7, &[f32::MAX]).unwrap();
        let before = bits(&raw(&data.current));
        assert!(plan.update(&mut data, &ticket(&source, &md)).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        md.vector.write_range(7, &q[7..8]).unwrap();
        md.current.write_range(7, &mq[7..8]).unwrap();
        let _ = plan.update(&mut data, &ticket(&source, &md)).unwrap();
        faults += 1;
        let before = bits(&raw(&data.current));
        assert!(data.write_potential(&[f32::NAN; 2]).is_err());
        assert!(data.write_world_potential(2, 1.0).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        source.write_qpos(&[0.0; 6]).unwrap();
        let stale = MassMatrixMultiplyView {
            source: &source,
            data: &md,
            computed: vec![true; 2],
        };
        assert!(plan.update(&mut data, &stale).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        faults += 1;
        assert_eq!(faults, 56);
        println!(
            "device_energy_private faults={faults} source_dependency_updates=1 Mqvel_dependency_updates=1 no_partial_publication=true cold_source_rejected=true"
        );
    }
}
