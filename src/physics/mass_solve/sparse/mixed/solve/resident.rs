//! GPU CRB到三路混合分解/求解的常驻桥接。
//! 调用方只写qpos/RHS；物理CSR始终留在设备上。
use super::{SparseMassMixedSolveData, SparseMassMixedSolveOutput, SparseMassMixedSolvePlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, SparseMassInput};
use crate::physics::{MassMatrixData, MassMatrixOutput, MassMatrixPlan};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, ElemType, Kernel, KernelAbi, KernelSpec, LaunchDims, LauncherError,
    SlotBinding, SlotDesc, SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

#[cfg(feature = "cuda-probe")]
const RESIDENT_MIXED_VALIDATION_SLOTS: [SlotDesc; 8] = [
    SlotDesc {
        index: 0,
        kind: SlotKind::DevicePtr {
            elem: ElemType::I32,
        },
        access: Access::Read,
        role: SlotRole::Value,
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
struct ResidentMixedValidationSpec;

#[cfg(feature = "cuda-probe")]
// SAFETY: 声明匹配冻结常驻混合校验ABI。
unsafe impl KernelAbi for ResidentMixedValidationSpec {
    const NAME: &'static str = "resident_mixed_validate";
    const BLOCK: BlockRule = BlockRule::Fixed(256);
    const SLOTS: &'static [SlotDesc] = &RESIDENT_MIXED_VALIDATION_SLOTS;
}

#[cfg(feature = "cuda-probe")]
impl KernelSpec for ResidentMixedValidationSpec {
    type Dims = [u32; 4];

    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        let (source, status) = resident_mixed_validation_lengths(dims)?;
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
                min_elements: 1,
            },
            SlotRequirement {
                slot: 3,
                min_elements: status,
            },
        ])
    }

    fn launch_dims(dims: &Self::Dims) -> Result<LaunchDims, LauncherError> {
        resident_mixed_validation_lengths(dims)?;
        LaunchDims::new((dims[3].div_ceil(256), 1, 1), (256, 1, 1), 0)
    }
}

#[cfg(feature = "cuda-probe")]
fn resident_mixed_validation_lengths(dims: &[u32; 4]) -> Result<(u64, u64), LauncherError> {
    let [nb, n, _nc, worlds] = *dims;
    if worlds == 0 || worlds > u32::MAX - 255 {
        return Err(LauncherError::InvalidArray("常驻混合校验维度无效"));
    }
    let body = 10u64
        .checked_mul(u64::from(nb))
        .ok_or(LauncherError::Overflow("resident_mixed_validation_body"))?;
    let dense = u64::from(n)
        .checked_mul(u64::from(n))
        .ok_or(LauncherError::Overflow("resident_mixed_validation_dense"))?;
    let stride = body
        .checked_add(dense)
        .ok_or(LauncherError::Overflow("resident_mixed_validation_stride"))?;
    let source = stride
        .checked_mul(u64::from(worlds))
        .and_then(|value| value.checked_add(8))
        .ok_or(LauncherError::Overflow("resident_mixed_validation_source"))?;
    let status = u64::from(worlds)
        .checked_add(8)
        .ok_or(LauncherError::Overflow("resident_mixed_validation_status"))?;
    Ok((source, status))
}

#[cfg(feature = "cuda-probe")]
fn resident_mixed_validation_plan(
    expected: [u32; 3],
    dims: &[u32; 4],
    bindings: &[SlotBinding; 4],
) -> Result<(), LauncherError> {
    if dims[..3] != expected {
        return Err(LauncherError::InvalidArray("常驻混合校验静态维度不匹配"));
    }
    let (source, status) = resident_mixed_validation_lengths(dims)?;
    let source_bytes = source.checked_mul(4).ok_or(LauncherError::Overflow(
        "resident_mixed_validation_source_bytes",
    ))?;
    let status_bytes = status.checked_mul(4).ok_or(LauncherError::Overflow(
        "resident_mixed_validation_status_bytes",
    ))?;
    if bindings[0].span.bytes() < 4
        || bindings[1].span.bytes() != source_bytes
        || bindings[2].span.bytes() < 4
        || bindings[3].span.bytes() != status_bytes
    {
        return Err(LauncherError::InvalidArray("常驻混合校验跨度无效"));
    }
    Kernel::<ResidentMixedValidationSpec>::new()?.plan(dims, bindings, &[])?;
    Ok(())
}

#[cfg(feature = "cuda-probe")]
fn resident_mixed_validation_launch_error(error: LauncherError) -> TransferError {
    match error {
        LauncherError::ContextMismatch { .. } => TransferError::SessionMismatch,
        LauncherError::Alias { .. } => TransferError::OverlappingCopy,
        _ => InputError::InvalidDimension {
            field: "resident_mixed_validation_abi",
        }
        .into(),
    }
}

#[cfg(feature = "cuda-probe")]
fn resident_mixed_validation_dimension(
    value: usize,
    field: &'static str,
) -> Result<u32, TransferError> {
    u32::try_from(value).map_err(|_| InputError::Overflow { field }.into())
}

#[cfg(feature = "cuda-probe")]
struct ResidentMixedValidationKernel {
    kernel: SynchronousKernel,
    nb: u32,
    n: u32,
    nc: u32,
}

#[cfg(feature = "cuda-probe")]
impl ResidentMixedValidationKernel {
    fn compile(
        session: &TransferSession,
        nb: usize,
        n: usize,
        nc: usize,
    ) -> Result<Self, TransferError> {
        let nb = resident_mixed_validation_dimension(nb, "resident_mixed_validation_nb")?;
        let n = resident_mixed_validation_dimension(n, "resident_mixed_validation_n")?;
        let nc = resident_mixed_validation_dimension(nc, "resident_mixed_validation_nc")?;
        resident_mixed_validation_lengths(&[nb, n, nc, 1])
            .map_err(resident_mixed_validation_launch_error)?;
        Ok(Self {
            kernel: SynchronousKernel::compile(session, CUDA, "resident_mixed_validate")?,
            nb,
            n,
            nc,
        })
    }

    fn launch(
        &self,
        metadata: &TransferBuffer<i32>,
        source: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let worlds =
            resident_mixed_validation_dimension(worlds, "resident_mixed_validation_worlds")?;
        self.launch_dims(
            metadata,
            source,
            unused,
            status,
            [self.nb, self.n, self.nc, worlds],
        )
    }

    fn launch_dims(
        &self,
        metadata: &TransferBuffer<i32>,
        source: &TransferBuffer<f32>,
        unused: &TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        dims: [u32; 4],
    ) -> Result<(), TransferError> {
        let bindings = [
            SlotBinding {
                slot: 0,
                span: metadata.launcher_span()?,
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: source.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: unused.launcher_span()?,
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: status.launcher_span()?,
                elem: ElemType::F32,
            },
        ];
        resident_mixed_validation_plan([self.nb, self.n, self.nc], &dims, &bindings)
            .map_err(resident_mixed_validation_launch_error)?;
        // SAFETY: 类型化计划核对真实跨度和固定ABI。
        unsafe { self.kernel.launch(metadata, source, unused, status, dims) }
    }
}

pub struct SparseMassMixedResidentPlan {
    matrix: MassMatrixPlan,
    solve: SparseMassMixedSolvePlan,
    #[cfg(feature = "cuda-probe")]
    nbody: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    validation: ResidentMixedValidationKernel,
    #[cfg(feature = "cuda-probe")]
    gather: SynchronousKernel,
}
pub struct SparseMassMixedResidentData {
    matrix: MassMatrixData,
    solve: SparseMassMixedSolveData,
    gathered: bool,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
fn topology(model: &AttachedModelInput, input: &SparseMassInput) -> Result<(), InputError> {
    if model.rigid().kinematics().nv() != input.nv()
        || model.rigid().fields().dof_parentid != input.fields().dof_parentid
    {
        return Err(InputError::InvalidTopology {
            field: "resident_mixed_topology",
            index: 0,
            reason: "CSR_and_inertial_dof_topology_must_match",
        });
    }
    Ok(())
}
impl SparseMassMixedResidentPlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        topology(&model, &input)?;
        #[cfg(feature = "cuda-probe")]
        let nbody = model.rigid().kinematics().nbody();
        // The mixed constructor rejects Tile before a physics producer exists.
        let solve = SparseMassMixedSolvePlan::new(session, input)?;
        let matrix = MassMatrixPlan::new(session, model)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (solve, matrix);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let f = solve.input().fields();
            let mut meta = f.m_rownnz.clone();
            meta.extend(&f.m_rowadr);
            meta.extend(&f.m_colind);
            if meta.len() > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "resident_mixed_metadata",
                }
                .into());
            }
            let metadata = session.upload(&meta)?;
            let validation = ResidentMixedValidationKernel::compile(
                session,
                nbody,
                solve.input().nv(),
                solve.input().nc(),
            )?;
            let gather = SynchronousKernel::compile(session, CUDA, "resident_mixed_gather")?;
            Ok(Self {
                matrix,
                solve,
                nbody,
                metadata,
                validation,
                gather,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.solve.input()
    }
    pub fn create_data(
        &self,
        worlds: usize,
        rhs_count: usize,
    ) -> Result<SparseMassMixedResidentData, TransferError> {
        let matrix = self.matrix.create_data(worlds)?;
        let solve = self.solve.create_data(worlds, rhs_count)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix, solve);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut status = crate::runtime::host_staging::<f32>(worlds + 8)?;
            status.fill(-131072.0);
            status[4..4 + worlds].fill(f32::NAN);
            Ok(SparseMassMixedResidentData {
                matrix,
                solve,
                gathered: false,
                status: self.solve.mass.session.upload(&status)?,
            })
        }
    }
    fn check_data(&self, data: &SparseMassMixedResidentData) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        self.solve.mass.check_data(&data.solve.mass)
    }
    pub fn update(&self, data: &mut SparseMassMixedResidentData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)?;
        self.update_gather(data)?;
        self.update_factor(data)?;
        self.update_solve(data)
    }
    pub fn update_position(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        self.matrix.update_position(&mut data.matrix)
    }
    pub fn update_matrix(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        self.matrix.update_matrix(&mut data.matrix)
    }
    pub fn update_gather(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = data.matrix.matrix_input()?;
            let n = self.input().nv() as u32;
            let nc = self.input().nc() as u32;
            let world_count = data.worlds();
            let worlds = world_count as u32;
            self.validation.launch(
                &self.metadata,
                source,
                source,
                &mut data.status,
                world_count,
            )?;
            let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "resident_mixed_source_status")?;
            for (world, &value) in status[4..4 + data.worlds()].iter().enumerate() {
                if value != 1.0 {
                    return Err(InputError::InvalidTopology {
                        field: "resident_mixed_source",
                        index: world,
                        reason: "invalid_CRB_or_dense_matrix",
                    }
                    .into());
                }
            }
            // SAFETY: Validated CSR rows partition nc, columns fit n, and the
            // disjoint primary M payload is worlds*nc with persistent guards.
            unsafe {
                self.gather.launch(
                    &self.metadata,
                    source,
                    source,
                    &mut data.solve.mass.matrix,
                    [self.nbody as u32, n, nc, worlds * n],
                )?;
            }
            let mut guards = [0.0f32; 8];
            data.solve
                .mass
                .matrix
                .read_range_into(0, &mut guards[..4])?;
            data.solve.mass.matrix.read_range_into(
                4 + data.solve.mass.sparse.matrix_layout.total_elements(),
                &mut guards[4..],
            )?;
            crate::physics::check_device_guards(&guards)?;
            // This captures the GPU-produced primary CSR once for both actual
            // factor producers. No host physical readback is uploaded again.
            self.solve.update_gather(&mut data.solve)?;
            data.gathered = true;
            Ok(())
        }
    }
    pub fn update_factor(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "resident_mixed_gather",
            });
        }
        self.solve.update_factor(&mut data.solve)
    }
    pub fn update_solve(
        &self,
        data: &mut SparseMassMixedResidentData,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "resident_mixed_gather",
            });
        }
        self.solve.update_solve(&mut data.solve)
    }
}
impl SparseMassMixedResidentData {
    pub fn worlds(&self) -> usize {
        self.matrix.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.solve.rhs_count()
    }
    fn invalidate(&mut self) {
        self.gathered = false;
        self.solve.ready = false;
        self.solve.mass.invalidate();
    }
    fn after_qpos(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate();
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_qpos(values);
        self.after_qpos(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_world_qpos(world, values);
        self.after_qpos(result)
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.solve.write_rhs(values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        self.solve.write_world_rhs(world, values)
    }
    /// 仅读取完成合并因子；陈旧CRB不决定缓存解寿命。
    pub fn readback(&self) -> Result<SparseMassMixedSolveOutput, TransferError> {
        self.solve.readback()
    }
    /// 独立诊断当前GPU矩阵；不改变求解缓存。
    pub fn readback_matrix(&self) -> Result<MassMatrixOutput, TransferError> {
        self.matrix.readback()
    }
}
// CSR selection reuses the frozen source/stride/copy semantics. No arithmetic
// or upstream precision change; full CRB and matrix source is validated first.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void resident_mixed_validate(const int* meta,const float* source,
 const float* unused,float* status,unsigned nb,unsigned n,unsigned nc,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long stride=10ull*nb+1ull*n*n,total=stride*worlds;bool valid=true;
 for(unsigned j=0;j<4;++j)valid=valid && source[j]==-131072.0f && source[4+total+j]==-131072.0f;
 const float* value=source+4+1ull*w*stride;
 for(unsigned long long j=0;j<stride;++j)valid=valid && isfinite(value[j]);
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void resident_mixed_gather(const int* meta,const float* source,
 const float* unused,float* csr,unsigned nb,unsigned n,unsigned nc,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,row=tid%n;const int* rownnz=meta;const int* rowadr=meta+n;const int* colind=meta+2*n;
 const float* dense=source+4+1ull*w*(10ull*nb+1ull*n*n)+10ull*nb;
 float* output=csr+4+1ull*w*nc;
 for(int j=0;j<rownnz[row];++j){int a=rowadr[row]+j;output[a]=dense[1ull*row*n+colind[a]];}
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate as engine;
    mod input {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/resident_mixed_mass.rs"
        ));
    }
    #[test]
    fn assembled_offsets_preserve_native_fields_and_single_world_inertia() {
        let model = input::model(false);
        let k = model.rigid().kinematics();
        let f = model.rigid().fields();
        assert_eq!((k.nv(), k.nq(), k.nbody(), k.njnt()), (68, 68, 69, 68));
        assert_eq!(&k.fields().body_parentid[66..], &[0, 66, 0]);
        assert_eq!(&k.fields().jnt_bodyid[65..], &[66, 67, 68]);
        assert_eq!(&f.dof_bodyid[65..], &[66, 67, 68]);
        assert_eq!(&f.dof_jntid[65..], &[65, 66, 67]);
        assert_eq!(&f.dof_parentid[65..], &[-1, 65, -1]);
        assert_eq!(&f.body_mass[66..], &[2.0, 3.0, 4.0]);
        assert_eq!(f.body_mass[0], 0.0);
        assert_eq!(&f.body_inertia[..3], &[0.0; 3]);
        let refs = input::reference();
        assert_eq!(refs["cases"].as_array().unwrap().len(), 2);
        assert_eq!(input::qpos(0).len(), 68);
        assert_eq!(input::rhs(&refs, 0, 3).len(), 204);
    }
    #[test]
    fn bridge_topology_rejects_different_nv_and_same_nv_wrong_ancestry() {
        let model = input::model(false);
        let sparse = input::input(false);
        topology(&model, &sparse).unwrap();
        assert!(topology(&model, &input::input(true)).is_err());
        let other = input::model(true);
        assert_eq!(other.rigid().kinematics().nv(), 68);
        assert!(topology(&other, &sparse).is_err());
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    fn typed_resident_mixed_validation_plan_freezes_exact_contract() {
        use crate::runtime::launcher::DeviceSpan;

        let expected = [69, 68, 2149];
        let dims = [expected[0], expected[1], expected[2], 257];
        let (source_len, status_len) = resident_mixed_validation_lengths(&dims).unwrap();
        let bindings = [
            SlotBinding {
                slot: 0,
                span: DeviceSpan::new(7, 0x1000, 4).unwrap(),
                elem: ElemType::I32,
            },
            SlotBinding {
                slot: 1,
                span: DeviceSpan::new(7, 0x20_0000, source_len * 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 2,
                span: DeviceSpan::new(7, 0x80_0000, 4).unwrap(),
                elem: ElemType::F32,
            },
            SlotBinding {
                slot: 3,
                span: DeviceSpan::new(7, 0x90_0000, status_len * 4).unwrap(),
                elem: ElemType::F32,
            },
        ];
        resident_mixed_validation_plan(expected, &dims, &bindings).unwrap();
        let launch = ResidentMixedValidationSpec::launch_dims(&dims).unwrap();
        assert_eq!(launch.grid(), (2, 1, 1));
        assert_eq!(launch.block(), (256, 1, 1));

        for (slot, bytes) in [(1, source_len * 4 - 4), (3, status_len * 4 - 4)] {
            let mut short = bindings;
            short[slot].span = DeviceSpan::new(7, short[slot].span.addr(), bytes).unwrap();
            assert!(resident_mixed_validation_plan(expected, &dims, &short).is_err());
        }
        for (slot, bytes) in [(1, source_len * 4 + 4), (3, status_len * 4 + 4)] {
            let mut long = bindings;
            long[slot].span = DeviceSpan::new(7, long[slot].span.addr(), bytes).unwrap();
            assert!(resident_mixed_validation_plan(expected, &dims, &long).is_err());
        }

        let mut foreign = bindings;
        foreign[2].span = DeviceSpan::new(8, 0x80_0000, 4).unwrap();
        assert!(matches!(
            resident_mixed_validation_plan(expected, &dims, &foreign),
            Err(LauncherError::ContextMismatch { slot: 2 })
        ));

        let mut alias = bindings;
        alias[3].span = DeviceSpan::new(7, bindings[1].span.addr(), status_len * 4).unwrap();
        assert!(matches!(
            resident_mixed_validation_plan(expected, &dims, &alias),
            Err(LauncherError::Alias {
                first: 1,
                second: 3,
            })
        ));

        assert!(
            resident_mixed_validation_plan(
                expected,
                &[expected[0] + 1, expected[1], expected[2], 257],
                &bindings,
            )
            .is_err()
        );
        assert!(matches!(
            resident_mixed_validation_lengths(&[u32::MAX, u32::MAX, 0, 2]),
            Err(LauncherError::Overflow(_))
        ));
        assert!(ResidentMixedValidationSpec::launch_dims(&[1, 1, 0, u32::MAX - 254,]).is_err());
        if usize::BITS > 32 {
            assert!(
                resident_mixed_validation_dimension(usize::MAX, "resident_mixed_validation_test")
                    .is_err()
            );
        }
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn gpu_resident_mixed_validation_wrapper_rejects_before_launch() {
        let session = TransferSession::new(0).unwrap();
        let foreign = TransferSession::new(0).unwrap();
        let (nb, n, nc, worlds) = (2usize, 3usize, 5usize, 257usize);
        let dims = [nb as u32, n as u32, nc as u32, worlds as u32];
        let (source_len, status_len) = resident_mixed_validation_lengths(&dims).unwrap();
        let mut source_values = vec![-131072.0f32; source_len as usize];
        let source_end = source_values.len() - 4;
        source_values[4..source_end].fill(0.25);
        let status_values = vec![-131072.0f32; status_len as usize];
        let mut metadata = session.upload(&[17i32]).unwrap();
        let mut unused = session.upload(&[7.0f32]).unwrap();
        let source = session.upload(&source_values).unwrap();
        let mut status = session.upload(&status_values).unwrap();
        let kernel = ResidentMixedValidationKernel::compile(&session, nb, n, nc).unwrap();

        kernel
            .launch(&metadata, &source, &unused, &mut status, worlds)
            .unwrap();
        let mut result = vec![0.0; status.len()];
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(&result[..4], &status_values[..4]);
        assert_eq!(
            &result[result.len() - 4..],
            &status_values[status_values.len() - 4..]
        );
        assert!(result[4..4 + worlds].iter().all(|&value| value == 1.0));

        metadata.write_range(0, &[-19]).unwrap();
        unused.write_range(0, &[f32::NAN]).unwrap();
        kernel
            .launch(&metadata, &source, &unused, &mut status, worlds)
            .unwrap();
        let stable = result.clone();
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);

        let short_source = session
            .upload(&source_values[..source_values.len() - 1])
            .unwrap();
        assert!(
            kernel
                .launch(&metadata, &short_source, &unused, &mut status, worlds)
                .is_err()
        );
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);

        let mut short_status = session
            .upload(&status_values[..status_values.len() - 1])
            .unwrap();
        assert!(
            kernel
                .launch(&metadata, &source, &unused, &mut short_status, worlds)
                .is_err()
        );
        let mut short_after = vec![0.0; short_status.len()];
        short_status.read_range_into(0, &mut short_after).unwrap();
        assert_eq!(short_after, status_values[..status_values.len() - 1]);

        let foreign_unused = foreign.upload(&[7.0f32]).unwrap();
        assert!(matches!(
            kernel.launch(&metadata, &source, &foreign_unused, &mut status, worlds),
            Err(TransferError::SessionMismatch)
        ));
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);

        assert!(
            kernel
                .launch_dims(
                    &metadata,
                    &source,
                    &unused,
                    &mut status,
                    [dims[0] + 1, dims[1], dims[2], dims[3]],
                )
                .is_err()
        );
        status.read_range_into(0, &mut result).unwrap();
        assert_eq!(result, stable);

        let mut source_after = vec![0.0; source.len()];
        source.read_range_into(0, &mut source_after).unwrap();
        assert_eq!(
            source_after
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            source_values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
    }
    #[cfg(feature = "cuda-probe")]
    fn close(a: f32, b: f64, tolerance: f64) -> f64 {
        let error = (f64::from(a) - b).abs();
        assert!(a.is_finite() && error <= tolerance + tolerance * b.abs());
        error
    }
    #[cfg(feature = "cuda-probe")]
    fn compare_solution(
        out: &SparseMassMixedSolveOutput,
        w: usize,
        v: &serde_json::Value,
        sample: usize,
    ) -> (usize, f64, f64) {
        let x = out.world(w).unwrap();
        let nr = out.rhs_count();
        let rhs = input::rhs(v, sample, nr);
        let mut max = 0.0f64;
        for (i, &a) in x.solution.iter().enumerate() {
            let b = v["cases"][sample]["solution"][(i / 68 % 3) * 68 + i % 68]
                .as_f64()
                .unwrap();
            max = max.max(close(a, b, 2e-4));
            assert_eq!(x.rhs[i].to_bits(), rhs[i].to_bits());
        }
        let mut worst = 0.0f64;
        for r in 0..nr {
            for block in out.input().blocks() {
                let mut an = 0.0f64;
                let mut xn = 0.0f64;
                let mut bn = 0.0f64;
                let mut rn = 0.0f64;
                for i in block.start..block.start + block.size {
                    let mut ax = 0.0f64;
                    let mut row = 0.0f64;
                    for j in block.start..block.start + block.size {
                        let m = v["cases"][sample]["matrix"][68 * i + j].as_f64().unwrap() as f32;
                        ax += f64::from(m) * f64::from(x.solution[68 * r + j]);
                        row += f64::from(m).abs();
                    }
                    an = an.max(row);
                    xn = xn.max(f64::from(x.solution[68 * r + i]).abs());
                    bn = bn.max(f64::from(rhs[68 * r + i]).abs());
                    rn = rn.max((ax - f64::from(rhs[68 * r + i])).abs());
                }
                let error = rn / (an * xn + bn).max(1e-30);
                assert!(error <= 2e-5);
                worst = worst.max(error);
            }
        }
        (nr * 68, max, worst)
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new qpos/CRB/CSR/three-route solve bridge numerical purpose"]
    fn gpu_complete_bridge_copies_actual_dense_bits_and_keeps_original_field_tolerances() {
        let v = input::reference();
        let native: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sparse-mass-factor/native.json"
        )))
        .unwrap();
        let session = TransferSession::new(0).unwrap();
        let plan =
            SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
                .unwrap();
        let start = std::time::Instant::now();
        let mut matrix_fields = 0;
        let mut csr_slots = 0;
        let mut factor_slots = 0;
        let mut solution_slots = 0;
        let mut max_matrix = 0.0f64;
        let mut max_factor = 0.0f64;
        let mut max_solution = 0.0f64;
        let mut max_residual = 0.0f64;
        let mut known_bytes = 0;
        let mut batches = 0;
        for sample in 0..2 {
            for (worlds, nr) in [(1, 3), (2, 3), (257, 3), (2, 1), (2, 2), (2, 5)] {
                let mut data = plan.create_data(worlds, nr).unwrap();
                data.write_qpos(&input::qpos(sample).repeat(worlds))
                    .unwrap();
                data.write_rhs(&input::rhs(&v, sample, nr).repeat(worlds))
                    .unwrap();
                plan.update_position(&mut data).unwrap();
                plan.update_matrix(&mut data).unwrap();
                plan.update_gather(&mut data).unwrap();
                let dense = data.readback_matrix().unwrap();
                let mut csr = vec![0.0; data.solve.mass.matrix.len()];
                data.solve.mass.matrix.read_range_into(0, &mut csr).unwrap();
                engine::physics::check_device_values(&csr, "bridge_test_CSR").unwrap();
                let f = plan.input().fields();
                for w in 0..worlds {
                    let source = dense.world(w).unwrap();
                    for (i, &a) in source.crb.iter().chain(source.matrix).enumerate() {
                        let expected = if i < 690 {
                            v["cases"][sample]["crb"][i].as_f64().unwrap()
                        } else {
                            v["cases"][sample]["matrix"][i - 690].as_f64().unwrap()
                        };
                        max_matrix = max_matrix.max(close(a, expected, 2e-5));
                        matrix_fields += 1;
                    }
                    for row in 0..68 {
                        for slot in
                            f.m_rowadr[row] as usize..(f.m_rowadr[row] + f.m_rownnz[row]) as usize
                        {
                            assert_eq!(
                                csr[4 + w * 2149 + slot].to_bits(),
                                source.matrix[68 * row + f.m_colind[slot] as usize].to_bits()
                            );
                            csr_slots += 1;
                        }
                    }
                }
                plan.update_factor(&mut data).unwrap();
                plan.update_solve(&mut data).unwrap();
                let out = data.readback().unwrap();
                for w in 0..worlds {
                    let factors = out.world(w).unwrap().factor;
                    let a = v["cases"][sample]["matrix"][68 * 65 + 65].as_f64().unwrap() as f32;
                    let b = v["cases"][sample]["matrix"][68 * 65 + 66].as_f64().unwrap() as f32;
                    let d = v["cases"][sample]["matrix"][68 * 66 + 66].as_f64().unwrap() as f32;
                    // Unique positive 2x2 U, independent immutable native M only.
                    let u = f64::from(a).sqrt();
                    let off = f64::from(b) / u;
                    let upper = [u, off, 0.0, (f64::from(d) - off * off).sqrt()];
                    for (&x, &y) in factors.dense_upper.iter().zip(&upper) {
                        max_factor = max_factor.max(close(x, y, 2e-4));
                        factor_slots += 1;
                    }
                    assert_eq!(factors.dense_upper[2].to_bits(), 0);
                    for i in 0..2149 {
                        let expected = if i < 2145 {
                            native["models"][0]["cases"][sample]["factor"][i]
                                .as_f64()
                                .unwrap()
                        } else {
                            f64::from(csr[4 + w * 2149 + i])
                        };
                        max_factor =
                            max_factor.max(close(factors.sparse_factors[i], expected, 2e-4));
                        factor_slots += 1;
                    }
                    for i in 0..68 {
                        let expected = if i < 65 {
                            native["models"][0]["cases"][sample]["diagonal_inverse"][i]
                                .as_f64()
                                .unwrap()
                        } else {
                            f64::from(
                                1.0f32
                                    / csr[4
                                        + w * 2149
                                        + (f.m_rowadr[i] + f.m_rownnz[i] - 1) as usize],
                            )
                        };
                        max_factor =
                            max_factor.max(close(factors.diagonal_inverse[i], expected, 2e-4));
                        factor_slots += 1;
                    }
                    let (count, a, r) = compare_solution(&out, w, &v, sample);
                    solution_slots += count;
                    max_solution = max_solution.max(a);
                    max_residual = max_residual.max(r);
                }
                // Exact capacities of the documented subset, not total/peak VRAM.
                let i = plan.input();
                let mixed_meta =
                    2 + 2 * i.nv() + 3 * i.qld_all_updates().len() + i.qld_block_total() + 3 + 1;
                let solve_meta = 4
                    + 2 * i.nv()
                    + i.qld_level_offsets().len()
                    + 3 * i.qld_all_updates().len()
                    + 3;
                let mixed = 4
                    * (worlds
                        * (5 * i.nc()
                            + 2 * i.nv()
                            + 3 * i.qld_block_total()
                            + 2
                            + 2 * nr * i.nv())
                        + 88
                        + mixed_meta
                        + solve_meta);
                let source = 4 * (worlds * 5314 + 8);
                let producer_meta = 4 * (2 * (69 + 68) + 2 + 68);
                let bridge = 4 * (2 * 68 + 2149 + worlds + 8);
                known_bytes = known_bytes.max(mixed + source + producer_meta + bridge);
                batches += 1;
            }
        }
        assert_eq!(
            (
                batches,
                matrix_fields,
                csr_slots,
                factor_slots,
                solution_slots
            ),
            (12, 2827048, 1143268, 1181572, 108256)
        );
        let report = serde_json::json!({"batches":batches,"CRB_M_fields":matrix_fields,"CSR_bit_slots":csr_slots,"factor_slots":factor_slots,
            "solution_slots":solution_slots,"max_CRB_M_abs":max_matrix,"max_factor_abs":max_factor,"max_solution_abs":max_solution,
            "max_tree_residual":max_residual,"phase_ms":start.elapsed().as_millis(),"known_owned_buffer_subset_bytes":known_bytes,
            "excluded_buffer_accounting":"KinematicsPlan/Data and related model subsets","complete_native_model":false,"full_G03_NP":false});
        let path =
            std::path::PathBuf::from(std::env::var_os("MJWARP_RESIDENT_MIXED_REPORT").unwrap());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        println!("{report}");
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new qpos generation/RHS factor-reuse bridge lifecycle purpose"]
    fn gpu_bridge_qpos_changes_invalidate_every_stage_and_rhs_only_reuses_completed_factor() {
        let v = input::reference();
        let session = TransferSession::new(0).unwrap();
        let plan =
            SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
                .unwrap();
        let other =
            SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
                .unwrap();
        let mut data = plan.create_data(257, 3).unwrap();
        let q = input::qpos(0);
        let rhs = input::rhs(&v, 0, 3);
        assert!(plan.update_matrix(&mut data).is_err());
        assert!(plan.update_gather(&mut data).is_err());
        assert!(data.readback().is_err());
        data.write_qpos(&q.repeat(257)).unwrap();
        data.write_rhs(&rhs.repeat(257)).unwrap();
        plan.update(&mut data).unwrap();
        let tokens = |d: &SparseMassMixedResidentData| {
            let state = d.matrix.state_values_for_test().unwrap();
            let mut rhs = vec![0.0; d.solve.rhs.len()];
            d.solve.rhs.read_range_into(0, &mut rhs).unwrap();
            let mut factor = vec![0.0; d.solve.mass.output.len()];
            d.solve.mass.output.read_range_into(0, &mut factor).unwrap();
            (
                state.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                rhs.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                factor.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            )
        };
        let unchanged = tokens(&data);
        assert!(matches!(
            other.update(&mut data),
            Err(TransferError::ModelMismatch)
        ));
        assert!(data.readback().is_ok());
        assert_eq!(tokens(&data), unchanged);
        for (w, values) in [(257, &q[..]), (0, &q[..67])] {
            assert!(data.write_world_qpos(w, values).is_err());
            assert!(data.readback().is_ok());
            assert_eq!(tokens(&data), unchanged);
        }
        let mut bad = q.clone();
        bad[0] = f32::NAN;
        assert!(data.write_world_qpos(0, &bad).is_err());
        assert!(data.readback().is_ok());
        assert_eq!(tokens(&data), unchanged);
        for (w, values) in [(257, &rhs[..]), (0, &rhs[..rhs.len() - 1])] {
            assert!(data.write_world_rhs(w, values).is_err());
            assert!(data.readback().is_ok());
            assert_eq!(tokens(&data), unchanged);
        }
        let mut bad_rhs = rhs.clone();
        bad_rhs[0] = f32::NAN;
        assert!(data.write_world_rhs(0, &bad_rhs).is_err());
        assert!(data.readback().is_ok());
        assert_eq!(tokens(&data), unchanged);
        let old = data.matrix.state_values_for_test().unwrap();
        data.write_world_qpos(256, &input::qpos(1)).unwrap();
        let next = data.matrix.state_values_for_test().unwrap();
        // qpos状态没有结果缓冲的四槽守卫。
        assert_eq!(old.len(), 257 * 68);
        assert_eq!(next.len(), old.len());
        for i in (0..256 * 68).chain(257 * 68..old.len()) {
            assert_eq!(old[i].to_bits(), next[i].to_bits());
        }
        assert_eq!(
            next[256 * 68..257 * 68]
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            input::qpos(1)
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(plan.update_solve(&mut data).is_err());
        data.write_world_rhs(256, &input::rhs(&v, 1, 3)).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        compare_solution(&out, 0, &v, 0);
        compare_solution(&out, 256, &v, 1);
        let mut factors = vec![0.0; data.solve.mass.output.len()];
        data.solve
            .mass
            .output
            .read_range_into(0, &mut factors)
            .unwrap();
        let raw_rhs = data.solve.rhs.len();
        let mut before = vec![0.0; raw_rhs];
        data.solve.rhs.read_range_into(0, &mut before).unwrap();
        data.write_world_rhs(256, &input::rhs(&v, 0, 3)).unwrap();
        assert!(data.readback().is_err());
        plan.update_solve(&mut data).unwrap();
        let mut after = vec![0.0; factors.len()];
        data.solve
            .mass
            .output
            .read_range_into(0, &mut after)
            .unwrap();
        assert_eq!(
            factors.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            after.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        let mut rhs_after = vec![0.0; raw_rhs];
        data.solve.rhs.read_range_into(0, &mut rhs_after).unwrap();
        for i in (0..4 + 256 * 204).chain(4 + 257 * 204..before.len()) {
            assert_eq!(before[i].to_bits(), rhs_after[i].to_bits());
        }
        data.write_world_rhs(256, &input::rhs(&v, 1, 3)).unwrap();
        plan.update_solve(&mut data).unwrap();
        let snapshot = data.readback().unwrap();
        drop(data);
        drop(plan);
        drop(other);
        drop(session);
        compare_solution(&snapshot, 256, &v, 1);
        assert!(snapshot.world(257).is_err());
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA/NVRTC; resident mixed validation product path purpose"]
    fn gpu_resident_mixed_validation_product_path_stops_publication() {
        let start = std::time::Instant::now();
        let v = input::reference();
        let session = TransferSession::new(0).unwrap();
        let plan =
            SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
                .unwrap();
        let mut data = plan.create_data(2, 3).unwrap();
        data.write_qpos(&input::qpos(0).repeat(2)).unwrap();
        data.write_rhs(&input::rhs(&v, 0, 3).repeat(2)).unwrap();
        plan.update(&mut data).unwrap();

        let bits = |buffer: &TransferBuffer<f32>| {
            let mut values = vec![0.0; buffer.len()];
            buffer.read_range_into(0, &mut values).unwrap();
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        };
        let stable_csr = bits(&data.solve.mass.matrix);
        let source_tail = data.matrix.matrix_input().unwrap().len() - 4;
        let unselected_dense = 4 + 690 + 67;
        let assert_bits_unchanged = |label: &str, fault: &str, before: &[u32], after: &[u32]| {
            assert_eq!(before.len(), after.len());
            if let Some((index, (&expected, &actual))) = before
                .iter()
                .zip(after)
                .enumerate()
                .find(|(_, (expected, actual))| expected != actual)
            {
                panic!(
                    "{label} changed during rejected {fault} at {index}: {expected} -> {actual}"
                );
            }
        };
        let mut recompute_drift = Vec::new();

        for (fault, offset, bad) in [
            ("head_guard", 0, 0.0),
            ("tail_guard", source_tail, 0.0),
            ("nonfinite_crb", 4, f32::NAN),
            ("nonfinite_unselected_dense", unselected_dense, f32::NAN),
        ] {
            let stable_factor = bits(&data.solve.mass.output);
            let stable_solution = bits(&data.solve.output);
            let mut original = [0.0];
            data.matrix
                .matrix_input()
                .unwrap()
                .read_range_into(offset, &mut original)
                .unwrap();
            data.matrix.corrupt_output_for_test(offset, bad).unwrap();

            assert!(plan.update_gather(&mut data).is_err());
            assert!(!data.gathered);
            assert!(!data.solve.ready);
            assert!(!data.solve.mass.ready);
            assert!(plan.update_factor(&mut data).is_err());
            assert!(plan.update_solve(&mut data).is_err());
            assert!(data.readback().is_err());
            assert_bits_unchanged("CSR", fault, &stable_csr, &bits(&data.solve.mass.matrix));
            assert_bits_unchanged(
                "factor",
                fault,
                &stable_factor,
                &bits(&data.solve.mass.output),
            );
            assert_bits_unchanged(
                "solution",
                fault,
                &stable_solution,
                &bits(&data.solve.output),
            );

            data.matrix
                .corrupt_output_for_test(offset, original[0])
                .unwrap();
            plan.update_gather(&mut data).unwrap();
            assert_bits_unchanged(
                "restored CSR",
                fault,
                &stable_csr,
                &bits(&data.solve.mass.matrix),
            );
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
            let restored = data.readback().unwrap();
            compare_solution(&restored, 0, &v, 0);
            compare_solution(&restored, 1, &v, 0);
            let recovered_factor = bits(&data.solve.mass.output);
            let recovered_solution = bits(&data.solve.output);
            recompute_drift.push((
                fault,
                stable_factor
                    .iter()
                    .zip(&recovered_factor)
                    .filter(|(before, after)| before != after)
                    .count(),
                stable_solution
                    .iter()
                    .zip(&recovered_solution)
                    .filter(|(before, after)| before != after)
                    .count(),
            ));
        }
        println!(
            "resident_mixed_validation_product worlds=2 faults=4 recovered=true recompute_drift={recompute_drift:?} phase_ms={}",
            start.elapsed().as_millis()
        );
    }
    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new bridge source/status/primary guard and cached CRB lifetime purpose"]
    fn cache_fault_bridge_rejects_full_source_and_target_guards_without_partial_publication() {
        let v = input::reference();
        let session = TransferSession::new(0).unwrap();
        let plan =
            SparseMassMixedResidentPlan::new(&session, input::model(false), input::input(false))
                .unwrap();
        let mut data = plan.create_data(2, 3).unwrap();
        data.write_qpos(&input::qpos(0).repeat(2)).unwrap();
        data.write_rhs(&input::rhs(&v, 0, 3).repeat(2)).unwrap();
        plan.update(&mut data).unwrap();
        // This cross-tree upper-triangular slot never enters the CSR. A cached
        // solve may ignore it; fresh bridge capture must validate and reject it.
        let unselected = 4 + 690 + 67;
        let mut saved = [0.0];
        data.matrix
            .matrix_input()
            .unwrap()
            .read_range_into(unselected, &mut saved)
            .unwrap();
        data.matrix
            .corrupt_output_for_test(unselected, f32::NAN)
            .unwrap();
        plan.update_solve(&mut data).unwrap();
        let cached = data.readback().unwrap();
        compare_solution(&cached, 0, &v, 0);
        assert!(data.readback_matrix().is_err());
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.readback().is_err());
        data.matrix
            .corrupt_output_for_test(unselected, saved[0])
            .unwrap();
        plan.update_gather(&mut data).unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
        for (offset, bad) in [
            (0, 0.0),
            (4 + 2 * 5314, 0.0),
            (4, f32::NAN),
            (4 + 690 + 67, f32::NAN),
        ] {
            let mut original = [0.0];
            data.matrix
                .matrix_input()
                .unwrap()
                .read_range_into(offset, &mut original)
                .unwrap();
            data.matrix.corrupt_output_for_test(offset, bad).unwrap();
            assert!(plan.update_gather(&mut data).is_err());
            assert!(!data.gathered);
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.matrix
                .corrupt_output_for_test(offset, original[0])
                .unwrap();
            plan.update_gather(&mut data).unwrap();
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        for offset in [0, 4 + 2 * 2149] {
            data.solve.mass.matrix.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_gather(&mut data).is_err());
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.solve
                .mass
                .matrix
                .write_range(offset, &[-131072.0])
                .unwrap();
            plan.update_gather(&mut data).unwrap();
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        for offset in [0, data.worlds() + 4] {
            data.status.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_gather(&mut data).is_err());
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.status.write_range(offset, &[-131072.0]).unwrap();
            plan.update_gather(&mut data).unwrap();
            plan.update_factor(&mut data).unwrap();
            plan.update_solve(&mut data).unwrap();
        }
        let out = data.readback().unwrap();
        compare_solution(&out, 0, &v, 0);
        compare_solution(&out, 1, &v, 0);
    }
}
