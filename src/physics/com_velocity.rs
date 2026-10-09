//! 常驻G02空间速度子集。
//! 完整G02与等价阶段仍待实现。

use super::{KinematicsData, KinematicsPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 一次上传与编译的空间速度计划。
/// 复用严格常驻位置与质心阶段。
/// 拓扑与参数只共享一份模型。
/// 不计算RNE、休眠与柔体动力学。
pub struct ComVelocityPlan {
    position: KinematicsPlan,
    nbody: usize,
    njnt: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernel: SynchronousKernel,
}

/// 独占位置、速度与派生设备字段。
/// 相同尺寸不允许跨计划使用。
/// 合法状态写入废弃速度结果。
pub struct ComVelocityData {
    position: KinematicsData,
    layout: ComVelocityLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    qvel: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

/// 显式回读后的独立宿主快照。
/// 后续设备写入不改变快照。
#[derive(Clone, Debug)]
pub struct ComVelocitySnapshot {
    layout: ComVelocityLayout,
    values: Vec<f32>,
}

/// 单世界空间字段的只读视图。
/// 六维顺序先角向再线向。
#[derive(Clone, Copy, Debug)]
pub struct ComVelocityWorld<'a> {
    pub cvel: &'a [f32],
    pub cdof_dot: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct ComVelocityLayout {
    output: BatchLayout,
    qvel: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
    #[cfg(feature = "cuda-probe")]
    guarded_qvel: usize,
}

impl ComVelocityLayout {
    fn new(worlds: usize, nbody: usize, njnt: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "com_velocity_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "com_velocity_layout",
        };
        let width = nbody
            .checked_add(nv)
            .and_then(|n| n.checked_mul(6))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(3)
            .and_then(|n| njnt.checked_mul(2).and_then(|j| n.checked_add(j)))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let qvel = BatchLayout::new(worlds, nv, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        let guarded_qvel = qvel.total_elements().checked_add(8).ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        BatchLayout::new(1, guarded_qvel, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (guarded, guarded_qvel);
        Ok(Self {
            output,
            qvel,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
            #[cfg(feature = "cuda-probe")]
            guarded_qvel,
        })
    }
}

impl ComVelocityPlan {
    /// 检查模型后建立严格常驻底座。
    /// 默认位置采用既有qpos0。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
    ) -> Result<Self, TransferError> {
        let k = model.rigid().kinematics();
        let (nbody, njnt, nv) = (k.nbody(), k.njnt(), k.nv());
        ComVelocityLayout::new(1, nbody, njnt, nv)?;
        #[cfg(feature = "cuda-probe")]
        let metadata_values = {
            let f = k.fields();
            let mut values = crate::runtime::host_staging::<i32>(3 * nbody + 2 * njnt)?;
            let mut cursor = 0;
            for field in [
                &f.body_parentid,
                &f.body_jntnum,
                &f.body_jntadr,
                &f.jnt_type,
                &f.jnt_dofadr,
            ] {
                values[cursor..cursor + field.len()].copy_from_slice(field);
                cursor += field.len();
            }
            values
        };
        let position = KinematicsPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let metadata = session.upload(&metadata_values)?;
        #[cfg(feature = "cuda-probe")]
        let kernel = SynchronousKernel::compile(session, COM_VELOCITY_CUDA, "com_velocity")?;
        Ok(Self {
            position,
            nbody,
            njnt,
            nv,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            kernel,
        })
    }

    pub fn device(&self) -> usize {
        self.position.device()
    }

    /// 创建独立世界与零qvel。
    /// 初始速度结果保持未就绪。
    pub fn create_data(&self, worlds: usize) -> Result<ComVelocityData, TransferError> {
        let layout = ComVelocityLayout::new(worlds, self.nbody, self.njnt, self.nv)?;
        let position = self.position.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, position);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut velocities = crate::runtime::host_staging::<f32>(layout.guarded_qvel)?;
            velocities.fill(-131072.0);
            velocities[4..layout.guarded_qvel - 4].fill(0.0);
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(ComVelocityData {
                position,
                layout,
                ready: false,
                qvel: self.session.upload(&velocities)?,
                output: self.session.upload(&values)?,
            })
        }
    }

    /// GPU重算位置、质心与空间速度。
    /// 不上传模型或重新编译内核。
    /// 阶段间不回读宿主中间量。
    /// 完成仍需显式回读数值检查。
    pub fn update(&self, data: &mut ComVelocityData) -> Result<(), TransferError> {
        self.position.check_data(&data.position)?;
        data.ready = false;
        self.position.update_rigid(&mut data.position)?;
        self.position.update_com(&mut data.position)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds() as u32;
            // SAFETY: Validated topology bounds joint/DOF indices and parent order.
            // The checked layouts bound all guarded offsets. Every f32 buffer
            // belongs to this plan's session; one thread owns one output world.
            // COM remains on-device with stride 14*nbody+6*nv. The adapter waits
            // before any buffer or compiled module can drop or be reused.
            unsafe {
                self.kernel.launch(
                    &self.metadata,
                    &data.qvel,
                    data.position.com_device()?,
                    &mut data.output,
                    [self.nbody as u32, self.njnt as u32, self.nv as u32, worlds],
                )?;
            }
            data.ready = true;
            Ok(())
        }
    }
}

impl ComVelocityData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.position.write_qpos(qpos);
        self.after_position_write(result)
    }

    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.position.write_world_qpos(world, qpos);
        self.after_position_write(result)
    }

    fn after_position_write(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    /// 检查全部速度后再写入设备。
    /// 输入错误保留已有速度结果。
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        self.write_checked_qvel(0, self.layout.qvel.total_elements(), qvel)
    }

    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qvel.world_elements(world)?;
        self.write_checked_qvel(range.start, range.len(), qvel)
    }

    fn write_checked_qvel(
        &mut self,
        offset: usize,
        expected: usize,
        qvel: &[f32],
    ) -> Result<(), TransferError> {
        check_qvel(expected, qvel)?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        self.qvel.write_range(4 + offset, qvel)
    }

    /// 显式检查速度与输出守卫。
    /// 未就绪或非有限结果不发布。
    pub fn readback(&self) -> Result<ComVelocitySnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "com_velocity",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.position.check_com_device_outputs()?;
            let mut qvel = crate::runtime::host_staging::<f32>(self.layout.guarded_qvel)?;
            self.qvel.read_range_into(0, &mut qvel)?;
            super::check_device_values(&qvel, "com_velocity_qvel")?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            super::check_device_values(&values, "com_velocity_output")?;
            Ok(ComVelocitySnapshot {
                layout: self.layout,
                values,
            })
        }
    }
}

impl ComVelocitySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<ComVelocityWorld<'_>, InputError> {
        let range = self.layout.output.world_elements(world)?;
        let values = &self.values[4 + range.start..4 + range.end];
        Ok(ComVelocityWorld {
            cvel: &values[..6 * self.nbody()],
            cdof_dot: &values[6 * self.nbody()..],
        })
    }
}

fn check_qvel(expected: usize, qvel: &[f32]) -> Result<(), InputError> {
    if qvel.len() != expected {
        return Err(InputError::LengthMismatch {
            field: "com_velocity_qvel",
            expected,
            actual: qvel.len(),
        });
    }
    if let Some(index) = qvel.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: "com_velocity_qvel",
            index,
        });
    }
    Ok(())
}

// Frozen smooth.py _comvel_branch and math.motion_cross (Apache-2.0).
// Copyright 2025 The Newton Developers.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(feature = "cuda-probe")]
const COM_VELOCITY_CUDA: &str = r#"
__device__ void motion_cross(const float* a,const float* b,float* out) {
  out[0]=a[1]*b[2]-a[2]*b[1];out[1]=a[2]*b[0]-a[0]*b[2];out[2]=a[0]*b[1]-a[1]*b[0];
  out[3]=(a[1]*b[5]-a[2]*b[4])+(a[4]*b[2]-a[5]*b[1]);
  out[4]=(a[2]*b[3]-a[0]*b[5])+(a[5]*b[0]-a[3]*b[2]);
  out[5]=(a[0]*b[4]-a[1]*b[3])+(a[3]*b[1]-a[4]*b[0]);
}
extern "C" __global__ void com_velocity(const int* m,const float* velocities,
    const float* com,float* output,unsigned nb,unsigned nj,unsigned nv,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  const int* parent=m;const int* num=parent+nb;const int* adr=num+nb;
  const int* type=adr+nb;const int* dofadr=type+nj;
  const float* qvel=velocities+4+(unsigned long long)w*nv;
  const float* cdof=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+14ull*nb;
  float* cvel=output+4+(unsigned long long)w*(6ull*nb+6ull*nv);float* dot=cvel+6ull*nb;
  for(int k=0;k<6;++k) cvel[k]=0.0f;
  for(unsigned b=1;b<nb;++b) {
    float v[6];for(int k=0;k<6;++k) v[k]=cvel[6ull*parent[b]+k];
    for(int j=adr[b];j<adr[b]+num[b];++j) {
      int d=dofadr[j],t=type[j];
      if(t==0) {
        for(int i=0;i<3;++i) for(int k=0;k<6;++k) {
          v[k]+=cdof[6ull*(d+i)+k]*qvel[d+i];dot[6ull*(d+i)+k]=0.0f;
        }
        for(int i=3;i<6;++i) motion_cross(v,cdof+6ull*(d+i),dot+6ull*(d+i));
        for(int i=3;i<6;++i) for(int k=0;k<6;++k) v[k]+=cdof[6ull*(d+i)+k]*qvel[d+i];
      } else if(t==1) {
        for(int i=0;i<3;++i) motion_cross(v,cdof+6ull*(d+i),dot+6ull*(d+i));
        for(int i=0;i<3;++i) for(int k=0;k<6;++k) v[k]+=cdof[6ull*(d+i)+k]*qvel[d+i];
      } else {
        motion_cross(v,cdof+6ull*d,dot+6ull*d);
        for(int k=0;k<6;++k) v[k]+=cdof[6ull*d+k]*qvel[d];
      }
    }
    for(int k=0;k<6;++k) cvel[6ull*b+k]=v[k];
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn velocity_readback_rejects_guards_and_nonfinite_buffers() {
        let model = super::super::tests::test_model(Some(3));
        let nv = model.kinematics().nv();
        let model =
            AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
        let session = TransferSession::new(0).unwrap();
        let plan = ComVelocityPlan::new(&session, model).unwrap();
        let mut data = plan.create_data(1).unwrap();
        plan.update(&mut data).unwrap();
        data.readback().unwrap();
        data.output.write_range(0, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.output.write_range(0, &[-131072.0]).unwrap();
        data.output.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        plan.update(&mut data).unwrap();
        data.qvel.write_range(0, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.qvel.write_range(0, &[-131072.0]).unwrap();
        data.qvel.write_range(4, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        data.write_qvel(&vec![0.0; nv]).unwrap();
        plan.update(&mut data).unwrap();
        data.readback().unwrap();
    }

    #[test]
    fn rejects_velocity_layout_dimensions_and_overflow() {
        for (w, b, j, v) in [
            (0, 1, 0, 0),
            (1, 0, 0, 0),
            (u32::MAX as usize, 1, 0, 0),
            (1, usize::MAX, 0, 0),
            (1, 1, usize::MAX, 0),
            (1, 1, 0, usize::MAX),
        ] {
            assert!(ComVelocityLayout::new(w, b, j, v).is_err());
        }
        assert!(ComVelocityLayout::new(513, 1, 0, 0).is_ok());
    }

    #[test]
    fn checks_velocity_lengths_nonfinite_inputs_and_empty_world_views() {
        assert!(check_qvel(1, &[]).is_err());
        for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(check_qvel(1, &[x]).is_err());
        }
        assert!(check_qvel(0, &[]).is_ok());
        let layout = ComVelocityLayout::new(2, 1, 0, 0).unwrap();
        let out = ComVelocitySnapshot {
            layout,
            values: vec![0.0; 20],
        };
        assert_eq!(out.worlds(), 2);
        assert_eq!(out.world(1).unwrap().cvel.len(), 6);
        assert!(out.world(1).unwrap().cdof_dot.is_empty());
        assert!(out.world(2).is_err());
    }
}
