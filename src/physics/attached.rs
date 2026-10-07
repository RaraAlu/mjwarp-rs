//! G01几何与site位姿辅助探针。
//! 不实现完整运动学阶段。

use super::{KinematicsOutput, check_kinematics};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout};
use crate::runtime::TransferSession;

/// 四项附着位姿的只读世界视图。
/// 位置用米，矩阵按行展开。
#[derive(Clone, Copy, Debug)]
pub struct AttachedKinematicsWorld<'a> {
    pub geom_xpos: &'a [f32],
    pub geom_xmat: &'a [f32],
    pub site_xpos: &'a [f32],
    pub site_xmat: &'a [f32],
}

/// 同步GPU刚体与附着结果。
/// 结果独立拥有宿主缓冲。
#[derive(Clone, Debug)]
pub struct AttachedKinematicsOutput {
    pub(super) rigid: KinematicsOutput,
    pub(super) layout: AttachedLayout,
    pub(super) values: Vec<f32>,
}

impl AttachedKinematicsOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn ngeom(&self) -> usize {
        self.layout.ngeom
    }
    pub fn nsite(&self) -> usize {
        self.layout.nsite
    }
    pub fn rigid(&self) -> &KinematicsOutput {
        &self.rigid
    }
    pub fn world(&self, world: usize) -> Result<AttachedKinematicsWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let ng = self.ngeom();
        let ns = self.nsite();
        Ok(AttachedKinematicsWorld {
            geom_xpos: &v[..3 * ng],
            geom_xmat: &v[3 * ng..12 * ng],
            site_xpos: &v[12 * ng..12 * ng + 3 * ns],
            site_xmat: &v[12 * ng + 3 * ns..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AttachedLayout {
    pub(super) output: BatchLayout,
    ngeom: usize,
    nsite: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: usize,
    #[cfg(feature = "cuda-probe")]
    parameters: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
}

impl AttachedLayout {
    pub(super) fn new(worlds: usize, ngeom: usize, nsite: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "attached_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "attached_layout",
        };
        let count = ngeom.checked_add(nsite).ok_or_else(overflow)?;
        let stride = count.checked_mul(12).ok_or_else(overflow)?;
        if stride > i32::MAX as usize {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = guarded;
        Ok(Self {
            output,
            ngeom,
            nsite,
            #[cfg(feature = "cuda-probe")]
            metadata: count + 1,
            #[cfg(feature = "cuda-probe")]
            parameters: (7 * count).max(1),
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

/// GPU计算几何与site世界位姿。
/// 复用同一设备中的刚体结果。
/// 包含世界体与静态子树附着。
/// 空附着集合仍计算刚体结果。
/// 每次调用重新计算所有附着。
/// 不复刻静态geom的缓存副作用。
/// 不处理mocap、相机或休眠。
/// 不替代U018与U062等价入口。
/// 需要cuda-probe与NVRTC。
pub fn probe_attached_kinematics(
    session: &TransferSession,
    model: &AttachedModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<AttachedKinematicsOutput, TransferError> {
    let fk = check_kinematics(model.rigid(), worlds, qpos)?;
    let layout = AttachedLayout::new(worlds, model.ngeom(), model.nsite())?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (session, fk, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        let (body, rigid_values) =
            super::kinematics_device::<f32>(session, model.rigid(), fk, qpos)?;
        let device = upload_attached_model(session, model, fk, layout)?;
        let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
        values.fill(-131072.0);
        values[4..layout.guarded - 4].fill(f32::NAN);
        let mut output = session.upload(&values)?;
        let source = format!("{}\n{}", super::KINEMATICS_CUDA, ATTACHED_CUDA);
        let kernel =
            crate::runtime::SynchronousKernel::compile(session, &source, "attached_kinematics")?;
        // SAFETY: Validated owned fields prove every body ID and parameter offset.
        // Checked layouts bound all per-world strides to i32; world offsets use u64.
        // The first metadata word is the checked FK stride, including joint fields.
        // All pointers use same-session allocations and the fixed f32 ABI.
        // Empty sets retain real metadata/parameter/guard buffers with no reads.
        // Each thread owns one world's output; launch waits before any owner drops.
        unsafe {
            kernel.launch(
                &device.metadata,
                &device.parameters,
                &body,
                &mut output,
                [
                    fk.nbody as u32,
                    model.ngeom() as u32,
                    model.nsite() as u32,
                    worlds as u32,
                ],
            )?;
        }
        output.read_range_into(0, &mut values)?;
        super::check_device_values(&values, "attached_kinematics_output")?;
        Ok(AttachedKinematicsOutput {
            rigid: KinematicsOutput {
                layout: fk,
                values: rigid_values,
            },
            layout,
            values,
        })
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn upload_attached_model(
    session: &TransferSession,
    model: &AttachedModelInput,
    fk: super::KinematicsLayout,
    layout: AttachedLayout,
) -> Result<super::DeviceModel<f32>, TransferError> {
    let f = model.fields();
    let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
    metadata[0] = fk.output.elements_per_world() as i32;
    metadata[1..1 + model.ngeom()].copy_from_slice(&f.geom_bodyid);
    metadata[1 + model.ngeom()..].copy_from_slice(&f.site_bodyid);
    let mut parameters = crate::runtime::host_staging::<f32>(layout.parameters)?;
    let mut cursor = 0;
    for field in [&f.geom_pos, &f.geom_quat, &f.site_pos, &f.site_quat] {
        parameters[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    Ok(super::DeviceModel {
        metadata: session.upload(&metadata)?,
        parameters: session.upload(&parameters)?,
    })
}

// Adapted from mujoco_warp/_src/smooth.py at
// 71da24d956378a87a703b6e1442b13aec0c4ac29.
// Copyright 2025 The Newton Developers
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
pub(super) const ATTACHED_CUDA: &str = r#"
__device__ void attached_frames(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned ng,unsigned ns,unsigned worlds,bool initialize) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const float* xp=body+4+(unsigned long long)w*meta[0];
  const float* xq=xp+3*nb;
#ifdef MJWARP_FIELD_BATCHES
  const int* desc=meta+1+ng+ns;
#ifdef MJWARP_MOCAP
  const int* static_geom=desc; desc+=ng;
#endif
  const float* gp=field_parameter(model,desc,w,3*ng);
  const float* gq=field_parameter(model,desc+2,w,4*ng);
  const float* sp=field_parameter(model,desc+4,w,3*ns);
  const float* sq=field_parameter(model,desc+6,w,4*ns);
#else
  const float* gp=model; const float* gq=gp+3*ng;
  const float* sp=gq+4*ng; const float* sq=sp+3*ns;
#endif
  float* gx=result+4+(unsigned long long)w*(12*ng+12*ns);
  float* gm=gx+3*ng; float* sx=gm+9*ng; float* sm=sx+3*ns;
  for(unsigned g=0;g<ng;++g) {
#ifdef MJWARP_MOCAP
    if(!initialize && static_geom[g]) continue;
#endif
    int b=meta[1+g]; Q q=load_q(xq+4*b);
    store_v(gx+3*g,add(load_v(xp+3*b),rotate(q,load_v(gp+3*g))));
    matrix(gm+9*g,multiply(q,load_q(gq+4*g)));
  }
  for(unsigned s=0;s<ns;++s) {
    int b=meta[1+ng+s]; Q q=load_q(xq+4*b);
    store_v(sx+3*s,add(load_v(xp+3*b),rotate(q,load_v(sp+3*s))));
    matrix(sm+9*s,multiply(q,load_q(sq+4*s)));
  }
}
extern "C" __global__ void attached_kinematics(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned ng,unsigned ns,unsigned worlds) {
  attached_frames(meta,model,body,result,nb,ng,ns,worlds,false);
}
#ifdef MJWARP_MOCAP
extern "C" __global__ void initialize_attached(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned ng,unsigned ns,unsigned worlds) {
  attached_frames(meta,model,body,result,nb,ng,ns,worlds,true);
}
#endif
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_dimensions_and_kernel_capacity() {
        for worlds in [0, u32::MAX as usize] {
            assert!(matches!(
                AttachedLayout::new(worlds, 1, 1),
                Err(InputError::InvalidDimension { .. })
            ));
        }
        for (ng, ns) in [(usize::MAX, 1), (i32::MAX as usize / 12 + 1, 0)] {
            assert!(matches!(
                AttachedLayout::new(1, ng, ns),
                Err(InputError::Overflow { .. })
            ));
        }
        assert!(AttachedLayout::new(1, 0, 0).is_ok());
        assert!(AttachedLayout::new(513, 2, 3).is_ok());
    }

    #[test]
    fn exposes_disjoint_world_fields_and_checked_empty_views() {
        for (ng, ns) in [(2, 3), (0, 3), (2, 0), (0, 0)] {
            let layout = AttachedLayout::new(2, ng, ns).unwrap();
            let stride = 12 * (ng + ns);
            let out = AttachedKinematicsOutput {
                rigid: KinematicsOutput {
                    layout: super::super::KinematicsLayout::new(2, 0, 1, 0).unwrap(),
                    values: vec![0.0; 64],
                },
                layout,
                values: (0..2 * stride + 8).map(|n| n as f32).collect(),
            };
            assert_eq!((out.worlds(), out.ngeom(), out.nsite()), (2, ng, ns));
            assert_eq!(out.rigid().worlds(), 2);
            let w = out.world(1).unwrap();
            assert_eq!(w.geom_xpos.len(), 3 * ng);
            assert_eq!(w.geom_xmat.len(), 9 * ng);
            assert_eq!(w.site_xpos.len(), 3 * ns);
            assert_eq!(w.site_xmat.len(), 9 * ns);
            let flattened: Vec<_> = [w.geom_xpos, w.geom_xmat, w.site_xpos, w.site_xmat].concat();
            assert_eq!(
                flattened,
                (stride + 4..2 * stride + 4)
                    .map(|n| n as f32)
                    .collect::<Vec<_>>()
            );
            assert!(out.world(2).is_err());
        }
    }
}
