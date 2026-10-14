//! 常驻CRB直接采集为CSR，再复用稀疏分解。
//! 宿主只读取诊断状态，不上传计算后的物理输入。
use super::{
    SparseMassFactorData, SparseMassFactorOutput, SparseMassFactorPlan, SparseMassFactorWorld,
};
use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout, SparseMassInput};
use crate::physics::{MassMatrixData, MassMatrixOutput, MassMatrixPlan, MassMatrixWorld};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

mod solve;
pub use solve::{
    SparseMassSolveData, SparseMassSolveOutput, SparseMassSolvePlan, SparseMassSolveWorld,
};

pub struct SparseMassPlan {
    matrix: MassMatrixPlan,
    factor: SparseMassFactorPlan,
    #[cfg(feature = "cuda-probe")]
    nbody: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 2],
}
pub struct SparseMassData {
    matrix: MassMatrixData,
    factor: SparseMassFactorData,
    gathered: bool,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}
#[derive(Clone, Debug)]
pub struct SparseMassOutput {
    matrix: MassMatrixOutput,
    factor: SparseMassFactorOutput,
    csr_layout: BatchLayout,
    csr: Vec<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct SparseMassWorld<'a> {
    pub matrix: MassMatrixWorld<'a>,
    pub factor: SparseMassFactorWorld<'a>,
    pub csr_matrix: &'a [f32],
}
impl SparseMassOutput {
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn worlds(&self) -> usize {
        self.matrix.worlds()
    }
    pub fn world(&self, world: usize) -> Result<SparseMassWorld<'_>, InputError> {
        let r = self.csr_layout.world_elements(world)?;
        Ok(SparseMassWorld {
            matrix: self.matrix.world(world)?,
            factor: self.factor.world(world)?,
            csr_matrix: &self.csr[4 + r.start..4 + r.end],
        })
    }
}
impl SparseMassPlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        input: SparseMassInput,
    ) -> Result<Self, TransferError> {
        let rigid = model.rigid();
        if rigid.kinematics().nv() != input.nv()
            || rigid.fields().dof_parentid != input.fields().dof_parentid
        {
            return Err(InputError::InvalidTopology {
                field: "sparse_mass_model",
                index: 0,
                reason: "CSR_and_inertial_dof_topology_must_match",
            }
            .into());
        }
        #[cfg(feature = "cuda-probe")]
        let nbody = rigid.kinematics().nbody();
        let factor = SparseMassFactorPlan::new(session, input)?;
        let matrix = MassMatrixPlan::new(session, model)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (factor, matrix);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let f = factor.input().fields();
            let mut metadata = f.m_rownnz.clone();
            metadata.extend(&f.m_rowadr);
            metadata.extend(&f.m_colind);
            if metadata.is_empty() {
                metadata.push(0);
            }
            let metadata = session.upload(&metadata)?;
            let kernels = [
                SynchronousKernel::compile(session, CUDA, "resident_sparse_validate")?,
                SynchronousKernel::compile(session, CUDA, "resident_sparse_gather")?,
            ];
            Ok(Self {
                matrix,
                factor,
                nbody,
                metadata,
                session: session.clone(),
                kernels,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.factor.input()
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassData, TransferError> {
        let matrix = self.matrix.create_data(worlds)?;
        let factor = self.factor.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (matrix, factor);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut status = crate::runtime::host_staging::<f32>(worlds + 8)?;
            status.fill(-131072.0);
            status[4..4 + worlds].fill(f32::NAN);
            Ok(SparseMassData {
                matrix,
                factor,
                gathered: false,
                status: self.session.upload(&status)?,
            })
        }
    }
    fn check_data(&self, data: &SparseMassData) -> Result<(), TransferError> {
        self.matrix.check_data(&data.matrix)?;
        if !std::sync::Arc::ptr_eq(&self.factor.input, &data.factor.input) {
            return Err(TransferError::ModelMismatch);
        }
        Ok(())
    }
    pub fn update(&self, data: &mut SparseMassData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)?;
        self.update_gather(data)?;
        self.update_factor(data)
    }
    pub fn update_position(&self, data: &mut SparseMassData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        self.matrix.update_position(&mut data.matrix)
    }
    pub fn update_matrix(&self, data: &mut SparseMassData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        self.matrix.update_matrix(&mut data.matrix)
    }
    /// GPU先验证整个源缓冲，再逐行复制真实CSR。
    pub fn update_gather(&self, data: &mut SparseMassData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = data.matrix.matrix_input()?;
            let worlds = data.worlds() as u32;
            let n = self.input().nv() as u32;
            let nc = self.input().nc() as u32;
            // SAFETY: Same-plan CRB stride is10*nb+n*n; source has4+4 guards.
            // Each validation thread owns one status slot inside worlds+8.
            // No source buffer is mutable and all launches finish synchronously.
            unsafe {
                self.kernels[0].launch(
                    &self.metadata,
                    source,
                    source,
                    &mut data.status,
                    [self.nbody as u32, n, nc, worlds],
                )?;
            }
            let mut status = crate::runtime::host_staging::<f32>(worlds as usize + 8)?;
            data.status.read_range_into(0, &mut status)?;
            crate::physics::check_device_values(&status, "sparse_mass_source_status")?;
            for (index, &actual) in status[4..4 + worlds as usize].iter().enumerate() {
                if actual != 1.0 {
                    return Err(ProbeError::Mismatch {
                        index,
                        expected: 1.0,
                        actual,
                    }
                    .into());
                }
            }
            if n != 0 {
                // SAFETY: Checked CSR columns fit n, rows partition nC and work
                // count worlds*n is bounded by the factor data layout. Gather
                // writes only the disjoint CSR payload, preserving both guards.
                unsafe {
                    self.kernels[1].launch(
                        &self.metadata,
                        source,
                        source,
                        &mut data.factor.matrix,
                        [self.nbody as u32, n, nc, worlds * n],
                    )?;
                }
            }
            // 只回读目标守卫；物理输入仍留在设备上。
            let mut guards = [0.0f32; 8];
            data.factor.matrix.read_range_into(0, &mut guards[..4])?;
            data.factor.matrix.read_range_into(
                data.factor.matrix_layout.total_elements() + 4,
                &mut guards[4..],
            )?;
            crate::physics::check_device_guards(&guards)?;
            data.gathered = true;
            Ok(())
        }
    }
    pub fn update_factor(&self, data: &mut SparseMassData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "resident_sparse_gather",
            });
        }
        self.factor.update_factor(&mut data.factor)
    }
}
impl SparseMassData {
    pub fn worlds(&self) -> usize {
        self.matrix.worlds()
    }
    fn invalidate(&mut self) {
        self.gathered = false;
        self.factor.ready = false;
    }
    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate();
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_qpos(values);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_world_qpos(world, values);
        self.after_write(result)
    }
    pub fn readback(&self) -> Result<SparseMassOutput, TransferError> {
        if !self.gathered {
            return Err(TransferError::StageNotReady {
                stage: "resident_sparse_gather",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let matrix = self.matrix.readback()?;
            let factor = self.factor.readback()?;
            let mut csr = crate::runtime::host_staging::<f32>(
                self.factor.matrix_layout.total_elements() + 8,
            )?;
            self.factor.matrix.read_range_into(0, &mut csr)?;
            crate::physics::check_device_values(&csr, "resident_sparse_matrix")?;
            Ok(SparseMassOutput {
                matrix,
                factor,
                csr_layout: self.factor.matrix_layout,
                csr,
            })
        }
    }
}

#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void resident_sparse_validate(const int* meta,const float* source,
 const float* unused,float* status,unsigned nb,unsigned n,unsigned nc,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long stride=10ull*nb+1ull*n*n,total=stride*worlds;
 bool valid=true;
 for(unsigned j=0;j<4;++j)valid=valid && source[j]==-131072.0f && source[4+total+j]==-131072.0f;
 const float* value=source+4+(unsigned long long)w*stride;
 for(unsigned long long j=0;j<stride;++j)valid=valid && isfinite(value[j]);
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void resident_sparse_gather(const int* meta,const float* source,
 const float* unused,float* csr,unsigned nb,unsigned n,unsigned nc,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,row=tid%n;const int* rownnz=meta;const int* rowadr=meta+n;const int* colind=meta+2*n;
 const float* dense=source+4+(unsigned long long)w*(10ull*nb+1ull*n*n)+10ull*nb;
 float* output=csr+4+(unsigned long long)w*nc;
 for(int j=0;j<rownnz[row];++j) {int a=rowadr[row]+j;output[a]=dense[(unsigned long long)row*n+colind[a]];}
}
"#;

#[cfg(all(test, feature = "cuda-probe"))]
mod tests {
    use super::*;
    use crate as engine;
    mod input {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/sparse_mass_model.rs"
        ));
    }
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC; new bridge source-guard fault purpose"]
    fn rejects_source_guards_unselected_nonfinite_and_status_corruption() {
        let refs = input::reference();
        let m = &refs["models"][0];
        let model = input::model(m);
        let n = model.rigid().kinematics().nv();
        let nb = model.rigid().kinematics().nbody();
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassPlan::new(&session, model, input::sparse(m)).unwrap();
        let mut data = plan.create_data(2).unwrap();
        data.write_qpos(&input::floats(&m["cases"][0], "qpos").repeat(2))
            .unwrap();
        plan.update(&mut data).unwrap();
        let before = data.readback().unwrap();
        for offset in [0, 4 + 2 * (10 * nb + n * n)] {
            data.matrix.corrupt_output_for_test(offset, 0.0).unwrap();
            assert!(plan.update_gather(&mut data).is_err());
            assert!(!data.gathered);
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.matrix
                .corrupt_output_for_test(offset, -131072.0)
                .unwrap();
            plan.update_gather(&mut data).unwrap();
            plan.update_factor(&mut data).unwrap();
        }
        // 上三角的这个值不进入CSR；源验证仍须拒绝它。
        let offset = 4 + 10 * nb + 1;
        let original = before.world(0).unwrap().matrix.matrix[1];
        data.matrix
            .corrupt_output_for_test(offset, f32::NAN)
            .unwrap();
        assert!(plan.update_gather(&mut data).is_err());
        assert!(data.readback().is_err());
        data.matrix
            .corrupt_output_for_test(offset, original)
            .unwrap();
        data.status.write_range(0, &[0.0]).unwrap();
        assert!(plan.update_gather(&mut data).is_err());
        data.status.write_range(0, &[-131072.0]).unwrap();
        for offset in [0, data.factor.matrix_layout.total_elements() + 4] {
            data.factor.matrix.write_range(offset, &[0.0]).unwrap();
            assert!(plan.update_gather(&mut data).is_err());
            assert!(plan.update_factor(&mut data).is_err());
            assert!(data.readback().is_err());
            data.factor
                .matrix
                .write_range(offset, &[-131072.0])
                .unwrap();
        }
        plan.update_gather(&mut data).unwrap();
        plan.update_factor(&mut data).unwrap();
        let mut values = vec![0.0; data.factor.matrix_layout.total_elements() + 8];
        data.factor.matrix.read_range_into(0, &mut values).unwrap();
        for index in (0..4).chain(values.len() - 4..values.len()) {
            assert_eq!(values[index], -131072.0);
        }
        assert!(data.readback().is_ok());
    }
}
