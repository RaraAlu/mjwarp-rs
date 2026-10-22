//! 常驻CRB严格辅助。
//! 旧一次性探针保持独立。
#[cfg(feature = "cuda-probe")]
use self::kernels::{ResidentCrbAccumulateKernel, ResidentCrbInitKernel, ResidentCrbMatrixKernel};
#[cfg(feature = "cuda-probe")]
use self::launcher_abi::ResidentCrbStaticDims;
#[cfg(feature = "cuda-probe")]
use self::launcher_meta::resident_crb_launch_error;
#[cfg(feature = "cuda-probe")]
use self::metadata_buffer::ResidentCrbMetadataBuffer;
use super::{MassMatrixLayout, MassMatrixOutput};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::physics::{KinematicsData, KinematicsPlan};
use crate::runtime::TransferSession;
#[cfg(all(feature = "cuda-probe", test))]
use crate::runtime::launcher::DeviceSpan;
#[cfg(feature = "cuda-probe")]
use crate::runtime::launcher::{
    Access, BlockRule, Checked, CheckedBinding, ElemType, Kernel, KernelAbi, KernelSpec,
    LaunchDims, LauncherError, MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc,
    SlotKind, SlotRequirement, SlotRole,
};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 常驻模型、位置前置与质量内核。
/// 共享armature；不提供完整G02。
pub struct MassMatrixPlan {
    position: KinematicsPlan,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    depth: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: ResidentCrbMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    armature: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    init: ResidentCrbInitKernel,
    #[cfg(feature = "cuda-probe")]
    accumulate: ResidentCrbAccumulateKernel,
    #[cfg(feature = "cuda-probe")]
    matrix: ResidentCrbMatrixKernel,
}

/// 独占位置与矩阵设备状态。
pub struct MassMatrixData {
    position: KinematicsData,
    layout: MassMatrixLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

fn check_layout(worlds: usize, nb: usize, nv: usize) -> Result<MassMatrixLayout, InputError> {
    worlds
        .checked_mul(nb.max(nv))
        .filter(|&n| n <= (u32::MAX - 255) as usize)
        .ok_or(InputError::Overflow {
            field: "resident_mass_matrix_work_items",
        })?;
    let metadata = nb
        .checked_add(nv)
        .and_then(|n| n.checked_mul(2))
        .and_then(|n| n.checked_add(2))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "resident_mass_matrix_metadata",
        })?;
    BatchLayout::new(1, metadata, 4)?;
    MassMatrixLayout::new(worlds, nb, nv)
}

impl MassMatrixPlan {
    /// 只读取实际计划拓扑；不执行物理生产者。
    pub(super) fn multiply_context(
        &self,
    ) -> Result<(usize, usize, TransferSession, Vec<i32>), TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut parents = crate::runtime::host_staging::<i32>(self.nv)?;
            self.metadata
                .as_buffer()
                .read_range_into(2 + self.nbody + self.nv, &mut parents)?;
            Ok((self.nbody, self.nv, self.session.clone(), parents))
        }
    }
    pub(in crate::physics) fn check_data(
        &self,
        data: &MassMatrixData,
    ) -> Result<(), TransferError> {
        self.position.check_data(&data.position)
    }
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
    ) -> Result<Self, TransferError> {
        let rigid = model.rigid();
        let k = rigid.kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        check_layout(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let (metadata, armature, depth) = {
            let mut values = crate::runtime::host_staging::<i32>(2 + 2 * nbody + 2 * nv)?;
            values[0] = nbody as i32;
            values[1] = nv as i32;
            values[2..2 + nbody].copy_from_slice(&k.fields().body_parentid);
            values[2 + nbody..2 + nbody + nv].copy_from_slice(&rigid.fields().dof_bodyid);
            values[2 + nbody + nv..2 + nbody + 2 * nv]
                .copy_from_slice(&rigid.fields().dof_parentid);
            let start = 2 + nbody + 2 * nv;
            for b in 1..nbody {
                values[start + b] = values[start + k.fields().body_parentid[b] as usize] + 1;
            }
            let depth = *values[start..].iter().max().unwrap() as usize;
            let static_dims =
                ResidentCrbStaticDims::new(nbody, nv, depth).map_err(resident_crb_launch_error)?;
            let armature = &rigid.fields().dof_armature;
            (
                ResidentCrbMetadataBuffer::upload(session, values, static_dims)?,
                session.upload(if armature.is_empty() {
                    &[0.0]
                } else {
                    armature
                })?,
                depth,
            )
        };
        let position = KinematicsPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let (init, accumulate, matrix) = (
            ResidentCrbInitKernel::compile(session, CUDA)?,
            ResidentCrbAccumulateKernel::compile(session, CUDA)?,
            ResidentCrbMatrixKernel::compile(session, CUDA)?,
        );
        Ok(Self {
            position,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            depth,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            armature,
            #[cfg(feature = "cuda-probe")]
            init,
            #[cfg(feature = "cuda-probe")]
            accumulate,
            #[cfg(feature = "cuda-probe")]
            matrix,
        })
    }

    pub fn device(&self) -> usize {
        self.position.device()
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMatrixData, TransferError> {
        let layout = check_layout(worlds, self.nbody, self.nv)?;
        let position = self.position.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, position);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut output = crate::runtime::host_staging::<f32>(layout.guarded)?;
            output.fill(-131072.0);
            output[4..layout.guarded - 4].fill(f32::NAN);
            Ok(MassMatrixData {
                position,
                layout,
                ready: false,
                output: self.session.upload(&output)?,
            })
        }
    }

    pub fn update(&self, data: &mut MassMatrixData) -> Result<(), TransferError> {
        self.update_position(data)?;
        self.update_matrix(data)
    }
    pub fn update_position(&self, data: &mut MassMatrixData) -> Result<(), TransferError> {
        self.position.check_data(&data.position)?;
        data.ready = false;
        self.position.update_rigid(&mut data.position)?;
        self.position.update_com(&mut data.position)
    }
    /// 只消费已有质心与惯量。
    /// 不更新位置或空间速度。
    pub fn update_matrix(&self, data: &mut MassMatrixData) -> Result<(), TransferError> {
        self.position.check_data(&data.position)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = data.position.mass_matrix_input()?;
            let worlds = data.worlds() as u32;
            // 初始化只读质心输入，写互不相交的CRB前缀。
            self.init.launch(
                &self.metadata,
                &self.armature,
                input,
                &mut data.output,
                worlds,
            )?;
            for level in (1..=self.depth).rev() {
                self.accumulate.launch(
                    &self.metadata,
                    &self.armature,
                    input,
                    &mut data.output,
                    worlds,
                    level as u32,
                )?;
            }
            if self.nv != 0 {
                self.matrix.launch(
                    &self.metadata,
                    &self.armature,
                    input,
                    &mut data.output,
                    worlds,
                )?;
            }
            data.ready = true;
            Ok(())
        }
    }
}

impl MassMatrixData {
    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn state_values_for_test(&self) -> Result<Vec<f32>, TransferError> {
        self.position.state_values_for_test()
    }
    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn corrupt_output_for_test(
        &mut self,
        offset: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        self.output.write_range(offset, &[value])
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn matrix_input(&self) -> Result<&TransferBuffer<f32>, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "resident_mass_matrix",
            });
        }
        Ok(&self.output)
    }

    // 故障注入只供新缓存测试。
    // 它故意绕过票据，产品不导出此入口。
    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn poison_matrix_for_test(
        &mut self,
        world: usize,
    ) -> Result<(), TransferError> {
        let range = self.layout.output.world_elements(world)?;
        let values = vec![0.0f32; self.layout.nv * self.layout.nv];
        self.output
            .write_range(4 + range.start + 10 * self.layout.nbody, &values)
    }

    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.position.write_qpos(values);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.position.write_world_qpos(world, values);
        self.after_write(result)
    }
    pub fn readback(&self) -> Result<MassMatrixOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "resident_mass_matrix",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.position.check_mass_matrix_inputs()?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            crate::physics::check_device_values(&values, "resident_mass_matrix_output")?;
            Ok(MassMatrixOutput {
                layout: self.layout,
                values,
            })
        }
    }
}

// Adapted from frozen smooth.py crb/_M and math.py inert_vec (71da24d).
// Copyright 2025 The Newton Developers. Apache-2.0.
// See licenses/Apache-2.0.txt for the license and warranty disclaimer.
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
extern "C" __global__ void resident_crb_init(const int* m,const float* p,const float* state,
    float* output,unsigned nb,unsigned nv,unsigned unused,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  const float* ci=state+4+(unsigned long long)w*(14ull*nb+6ull*nv)+4ull*nb;
  float* crb=output+4+(unsigned long long)w*(10ull*nb+(unsigned long long)nv*nv);
  for(unsigned i=0;i<10*nb;++i) crb[i]=ci[i];
  for(unsigned i=0;i<nv*nv;++i) crb[10ull*nb+i]=0.0f;
}
extern "C" __global__ void resident_crb_accumulate(const int* m,const float* p,const float* state,
    float* output,unsigned nb,unsigned nv,unsigned level,unsigned count) {
  unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=count) return;
  unsigned b=tid%nb,w=tid/nb;const int* parent=m+2;const int* depth=m+2+nb+2ull*nv;
  if(parent[b]==0 || depth[b]!=(int)level) return;
  float* crb=output+4+(unsigned long long)w*(10ull*nb+(unsigned long long)nv*nv);
  for(int k=0;k<10;++k) atomicAdd(crb+10ull*parent[b]+k,crb[10ull*b+k]);
}
extern "C" __global__ void resident_crb_matrix(const int* m,const float* p,const float* state,
    float* output,unsigned nb,unsigned nv,unsigned unused,unsigned count) {
  unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=count) return;
  unsigned d=tid%nv,w=tid/nv;const int* body=m+2+nb;const int* parent=body+nv;
  const float* cd=state+4+(unsigned long long)w*(14ull*nb+6ull*nv)+14ull*nb;
  float* crb=output+4+(unsigned long long)w*(10ull*nb+(unsigned long long)nv*nv);
  float* mat=crb+10ull*nb;const float* i=crb+10ull*body[d];const float* v=cd+6ull*d;
  float first=fmaf(i[3],v[0],__fmul_rn(i[1],v[1]));
  float buf[6]={
    i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5],
    first+i[5]*v[2]+i[8]*v[3]-i[6]*v[5],
    i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4],
    i[8]*v[1]-i[7]*v[2]+i[9]*v[3],
    i[6]*v[2]-i[8]*v[0]+i[9]*v[4],
    i[7]*v[0]-i[6]*v[1]+i[9]*v[5]};
  mat[(unsigned long long)d*nv+d]=p[d];
  for(int j=(int)d;j>=0;j=parent[j]) {
    float value=fmaf(cd[6ull*j],buf[0],0.0f);
    for(int k=1;k<6;++k) value=fmaf(cd[6ull*j+k],buf[k],value);
    float* cell=mat+(unsigned long long)d*nv+j;
    atomicAdd(cell,value);
    if(j!=(int)d) mat[(unsigned long long)j*nv+d]=*cell;
  }
}
"#;

#[cfg(feature = "cuda-probe")]
#[path = "resident/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "resident/launcher_abi.rs"]
mod launcher_abi;
#[cfg(feature = "cuda-probe")]
#[path = "resident/launcher_meta.rs"]
mod launcher_meta;
#[cfg(feature = "cuda-probe")]
#[path = "resident/launcher_specs.rs"]
mod launcher_specs;
#[cfg(feature = "cuda-probe")]
#[path = "resident/metadata_buffer.rs"]
mod metadata_buffer;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "resident/tests/launcher.rs"]
mod tests_launcher;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_resident_parallel_work_and_zero_dof_layout() {
        assert!(check_layout((u32::MAX / 2) as usize, 3, 1).is_err());
        assert!(check_layout(1, usize::MAX, 0).is_err());
        assert!(check_layout(1, 1, usize::MAX).is_err());
        let l = check_layout(513, 1, 0).unwrap();
        assert_eq!(l.output.total_elements(), 5130);
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn checks_new_matrix_guards_and_atomic_subnormal_boundaries() {
        let session = TransferSession::new(0).unwrap();
        let model = crate::physics::tests::test_model(Some(3));
        let model =
            AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
        let plan = MassMatrixPlan::new(&session, model).unwrap();
        let mut data = plan.create_data(1).unwrap();
        plan.update(&mut data).unwrap();
        for index in [0, data.layout.guarded - 1] {
            data.output.write_range(index, &[0.0]).unwrap();
            assert!(data.readback().is_err());
            data.output.write_range(index, &[-131072.0]).unwrap();
        }
        for index in [4, 4 + 10 * data.layout.nbody] {
            data.output.write_range(index, &[f32::NAN]).unwrap();
            assert!(data.readback().is_err());
            plan.update_matrix(&mut data).unwrap();
        }
        data.readback().unwrap();
        // 合成两体单自由度夹具：八项拓扑元数据、34元质心跨度与21元CRB跨度。
        let metadata = ResidentCrbMetadataBuffer::upload(
            &session,
            vec![2, 1, 0, 0, 1, -1, 0, 1],
            ResidentCrbStaticDims::new(2, 1, 1).unwrap(),
        )
        .unwrap();
        for (mass, armature, expected) in [
            (f32::from_bits(1), 0.0, 0.0),
            (0.0, f32::from_bits(1), 0.0),
            (1.0, f32::from_bits(1), 1.0),
            (f32::from_bits(1), 1.0, 1.0),
        ] {
            let mut input = vec![0.0f32; 42];
            input[..4].fill(-131072.0);
            input[38..].fill(-131072.0);
            input[31] = mass;
            input[35] = 1.0;
            let input = session.upload(&input).unwrap();
            let params = session.upload(&[armature]).unwrap();
            let mut initial = vec![0.0f32; 29];
            initial[..4].fill(-131072.0);
            initial[25..].fill(-131072.0);
            let mut output = session.upload(&initial).unwrap();
            plan.init
                .launch(&metadata, &params, &input, &mut output, 1)
                .unwrap();
            plan.matrix
                .launch(&metadata, &params, &input, &mut output, 1)
                .unwrap();
            output.read_range_into(0, &mut initial).unwrap();
            crate::physics::check_device_values(&initial, "crb_atomic_boundary").unwrap();
            assert_eq!(initial[24].to_bits(), (expected as f32).to_bits());
        }
    }
}
