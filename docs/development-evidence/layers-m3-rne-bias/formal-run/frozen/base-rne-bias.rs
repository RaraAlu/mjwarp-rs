//! 常驻RNE偏置力子集。
//! 完整G02与qacc路径仍待实现。

use super::{ComVelocityData, ComVelocityPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

/// 一次上传与编译的偏置力计划。
/// 重力与模型参数共享一份。
/// 显式零重力关闭重力贡献。
/// 本子集固定采用flg_acc=false。
pub struct RneBiasPlan {
    velocity: ComVelocityPlan,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    kernel: SynchronousKernel,
}

/// 独占位置、速度与偏置力设备状态。
/// 合法写入废弃已有偏置力。
pub struct RneBiasData {
    velocity: ComVelocityData,
    layout: RneBiasLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    output: TransferBuffer<f32>,
}

/// 显式回读后的独立宿主快照。
#[derive(Clone, Debug)]
pub struct RneBiasSnapshot {
    layout: RneBiasLayout,
    values: Vec<f32>,
}

/// 单世界广义偏置力只读视图。
#[derive(Clone, Copy, Debug)]
pub struct RneBiasWorld<'a> {
    pub qfrc_bias: &'a [f32],
}

#[derive(Clone, Copy, Debug)]
struct RneBiasLayout {
    output: BatchLayout,
    #[cfg(feature = "cuda-probe")]
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl RneBiasLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "rne_bias_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "rne_bias_layout",
        };
        let width = nbody
            .checked_mul(12)
            .and_then(|n| n.checked_add(nv))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(3)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(5))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            #[cfg(feature = "cuda-probe")]
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

fn check_gravity(gravity: &[f32; 3]) -> Result<(), InputError> {
    if let Some(index) = gravity.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: "rne_bias_gravity",
            index,
        });
    }
    Ok(())
}

impl RneBiasPlan {
    /// 调用方显式提供共享重力。
    /// 本入口不转换完整原生Option。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        gravity: [f32; 3],
    ) -> Result<Self, TransferError> {
        check_gravity(&gravity)?;
        let k = model.rigid().kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        RneBiasLayout::new(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let values = {
            let f = k.fields();
            let mut m = crate::runtime::host_staging::<i32>(5 + 3 * nbody + nv)?;
            m[0] = nbody as i32;
            m[1] = nv as i32;
            for i in 0..3 {
                m[2 + i] = gravity[i].to_bits() as i32;
            }
            m[5..5 + nbody].copy_from_slice(&f.body_parentid);
            for b in 1..nbody {
                let start = f.body_jntadr[b] as usize;
                let count = f.body_jntnum[b] as usize;
                if count != 0 {
                    m[5 + 2 * nbody + b] = f.jnt_dofadr[start];
                    m[5 + nbody + b] = f.jnt_type[start..start + count]
                        .iter()
                        .map(|&t| match t {
                            0 => 6,
                            1 => 3,
                            _ => 1,
                        })
                        .sum();
                }
            }
            m[5 + 3 * nbody..].copy_from_slice(&model.rigid().fields().dof_bodyid);
            m
        };
        let velocity = ComVelocityPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let metadata = session.upload(&values)?;
        #[cfg(feature = "cuda-probe")]
        let kernel = SynchronousKernel::compile(session, RNE_BIAS_CUDA, "rne_bias")?;
        Ok(Self {
            velocity,
            nbody,
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
        self.velocity.device()
    }

    pub fn create_data(&self, worlds: usize) -> Result<RneBiasData, TransferError> {
        let layout = RneBiasLayout::new(worlds, self.nbody, self.nv)?;
        let velocity = self.velocity.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, velocity);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(RneBiasData {
                velocity,
                layout,
                ready: false,
                output: self.session.upload(&values)?,
            })
        }
    }

    /// GPU更新位置、质心、速度与偏置力。
    /// 阶段间不回读宿主中间量。
    pub fn update(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.update_velocity(data)?;
        self.update_bias(data)
    }

    /// 显式准备本子集前置阶段。
    /// 本调用废弃已有偏置力。
    pub fn update_velocity(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        self.velocity.update(&mut data.velocity)
    }

    /// 只读取已有空间速度与质心。
    /// 不隐式重算任何前置阶段。
    /// 本严格子集要求新鲜度。
    /// 它不替代完整等价rne入口。
    pub fn update_bias(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let inputs = data.velocity.rne_inputs()?;
            // SAFETY: Reuse the five-pointer spatial ABI with f32 fields.
            // Validated parent/DOF maps bound all accesses. Metadata stores
            // dimensions and finite gravity's IEEE f32 bits, decoded by CUDA.
            // Input strides are nv, 14*nbody+6*nv and 6*nbody+6*nv.
            // Output uses f32 with stride 12*nbody+nv, like frozen RNE.
            // All state/result buffers have four guards on each side.
            // Disjoint buffers retain one session; launch waits for completion.
            unsafe {
                self.kernel.launch_flex_edges(
                    &self.metadata,
                    inputs.qvel,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                    data.layout.output.worlds() as u32,
                )?;
            }
            data.ready = true;
            Ok(())
        }
    }
}

impl RneBiasData {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }

    fn after_write(&mut self, result: Result<(), TransferError>) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.ready = false;
        }
        result
    }

    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qpos(qpos);
        self.after_write(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qpos(world, qpos);
        self.after_write(result)
    }
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_qvel(qvel);
        self.after_write(result)
    }
    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let result = self.velocity.write_world_qvel(world, qvel);
        self.after_write(result)
    }

    /// 显式检查前置字段与全部守卫。
    /// 回读包含内部工作区检查。
    pub fn readback(&self) -> Result<RneBiasSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady { stage: "rne_bias" });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.velocity.readback()?;
            let mut values = crate::runtime::host_staging::<f32>(self.layout.guarded)?;
            self.output.read_range_into(0, &mut values)?;
            super::check_device_values(&values, "rne_bias_output")?;
            let mut bias = crate::runtime::host_staging::<f32>(self.worlds() * self.layout.nv)?;
            for w in 0..self.worlds() {
                let start =
                    4 + self.layout.output.world_elements(w)?.start + 12 * self.layout.nbody;
                for d in 0..self.layout.nv {
                    bias[w * self.layout.nv + d] = values[start + d];
                }
            }
            if let Some(index) = bias.iter().position(|v| !v.is_finite()) {
                return Err(InputError::NonFinite {
                    field: "rne_bias_output",
                    index,
                }
                .into());
            }
            Ok(RneBiasSnapshot {
                layout: self.layout,
                values: bias,
            })
        }
    }
}

impl RneBiasSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<RneBiasWorld<'_>, InputError> {
        self.layout.output.world_elements(world)?;
        let start = world * self.nv();
        Ok(RneBiasWorld {
            qfrc_bias: &self.values[start..start + self.nv()],
        })
    }
}

// Adapted from frozen smooth.py RNE and math.py spatial inertia/force algebra.
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
const RNE_BIAS_CUDA: &str = r#"
__device__ void inertia_vector(const float* i,const float* v,float* f) {
  f[0]=i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5];
  f[1]=i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5];
  f[2]=i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4];
  f[3]=i[8]*v[1]-i[7]*v[2]+i[9]*v[3];
  f[4]=i[6]*v[2]-i[8]*v[0]+i[9]*v[4];
  f[5]=i[7]*v[0]-i[6]*v[1]+i[9]*v[5];
}
__device__ void cross_force(const float* v,const float* f,float* out) {
  out[0]=(v[1]*f[2]-v[2]*f[1])+(v[4]*f[5]-v[5]*f[4]);
  out[1]=(v[2]*f[0]-v[0]*f[2])+(v[5]*f[3]-v[3]*f[5]);
  out[2]=(v[0]*f[1]-v[1]*f[0])+(v[3]*f[4]-v[4]*f[3]);
  out[3]=v[1]*f[5]-v[2]*f[4];out[4]=v[2]*f[3]-v[0]*f[5];out[5]=v[0]*f[4]-v[1]*f[3];
}
extern "C" __global__ void rne_bias(const int* m,const float* velocities,
    const float* com,const float* velocity,float* output,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  unsigned nb=m[0],nv=m[1];const int* parent=m+5;
  const int* num=parent+nb;const int* adr=num+nb;const int* body=adr+nb;
  const float* qvel=velocities+4+(unsigned long long)w*nv;
  const float* ci=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+4ull*nb;
  const float* cdof=ci+10ull*nb;
  const float* cvel=velocity+4+(unsigned long long)w*(6ull*nb+6ull*nv);
  const float* dot=cvel+6ull*nb;
  float* acc=output+4+(unsigned long long)w*(12ull*nb+nv);
  float* frc=acc+6ull*nb;float* bias=frc+6ull*nb;
  for(int k=0;k<6;++k) { acc[k]=k<3?0.0f:-__int_as_float(m[k-1]);frc[k]=0.0f; }
  for(unsigned b=1;b<nb;++b) {
    float* a=acc+6ull*b;
    for(int k=0;k<6;++k) a[k]=acc[6ull*parent[b]+k];
    for(int d=adr[b];d<adr[b]+num[b];++d)
      for(int k=0;k<6;++k) a[k]+=dot[6ull*d+k]*qvel[d];
    float iv[6],cross[6],v[6];for(int k=0;k<6;++k) v[k]=cvel[6ull*b+k];
    inertia_vector(ci+10ull*b,v,iv);cross_force(v,iv,cross);
    inertia_vector(ci+10ull*b,a,frc+6ull*b);
    for(int k=0;k<6;++k) frc[6ull*b+k]+=cross[k];
  }
  for(int b=int(nb)-1;b>0;--b)
    for(int k=0;k<6;++k) frc[6ull*parent[b]+k]+=frc[6ull*b+k];
  for(unsigned d=0;d<nv;++d) {
    float f=0.0f;for(int k=0;k<6;++k) f+=cdof[6ull*d+k]*frc[6ull*body[d]+k];bias[d]=f;
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bias_layout_dimensions_and_overflow() {
        for (w, b, v) in [
            (0, 1, 0),
            (1, 0, 0),
            (u32::MAX as usize, 1, 0),
            (1, usize::MAX, 0),
            (1, 1, usize::MAX),
        ] {
            assert!(RneBiasLayout::new(w, b, v).is_err());
        }
        let layout = RneBiasLayout::new(513, 1, 0).unwrap();
        assert_eq!(layout.output.total_bytes(), 513 * 12 * size_of::<f32>());
    }

    #[test]
    fn rejects_nonfinite_gravity_and_checks_empty_bias_views() {
        for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for i in 0..3 {
                let mut g = [0.0; 3];
                g[i] = v;
                assert!(check_gravity(&g).is_err());
            }
        }
        assert!(check_gravity(&[1.0, -2.0, 3.0]).is_ok());
        let out = RneBiasSnapshot {
            layout: RneBiasLayout::new(2, 1, 0).unwrap(),
            values: vec![],
        };
        assert_eq!(out.worlds(), 2);
        assert!(out.world(1).unwrap().qfrc_bias.is_empty());
        assert!(out.world(2).is_err());
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn bias_readback_rejects_guards_nonfinite_workspaces_and_results() {
        let m = super::super::tests::test_model(Some(3));
        let m = AttachedModelInput::new(m, crate::model::AttachedFields::default()).unwrap();
        let s = TransferSession::new(0).unwrap();
        let p = RneBiasPlan::new(&s, m, [0.0; 3]).unwrap();
        let mut d = p.create_data(1).unwrap();
        p.update(&mut d).unwrap();
        d.readback().unwrap();
        for i in [0, d.layout.guarded - 1] {
            d.output.write_range(i, &[0.0]).unwrap();
            assert!(d.readback().is_err());
            d.output.write_range(i, &[-131072.0]).unwrap();
        }
        for i in [4, 4 + 6 * d.layout.nbody, 4 + 12 * d.layout.nbody] {
            d.output.write_range(i, &[f32::NAN]).unwrap();
            assert!(d.readback().is_err());
            p.update_bias(&mut d).unwrap();
            d.readback().unwrap();
        }
        d.output
            .write_range(4 + 12 * d.layout.nbody, &[f32::INFINITY])
            .unwrap();
        assert!(d.readback().is_err());
        p.update_bias(&mut d).unwrap();
        d.readback().unwrap();
    }
}
