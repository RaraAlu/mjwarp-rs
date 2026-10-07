//! G02刚体质量矩阵辅助探针。
//! 不实现分解、求解或完整阶段。
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;

/// 同步结果的只读世界视图。
#[derive(Clone, Copy, Debug)]
pub struct MassMatrixWorld<'a> {
    /// 每体十项复合惯量。
    /// 打包顺序沿用cinert。
    pub crb: &'a [f32],
    /// 对称稠密矩阵，按行展开。
    /// 混合坐标不共享同一单位。
    pub matrix: &'a [f32],
}

/// GPU质量矩阵的宿主结果。
/// 结果不承诺正定或可逆。
#[derive(Clone, Debug)]
pub struct MassMatrixOutput {
    layout: MassMatrixLayout,
    values: Vec<f32>,
}
impl MassMatrixOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<MassMatrixWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let values = &self.values[r.start + 4..r.end + 4];
        Ok(MassMatrixWorld {
            crb: &values[..10 * self.nbody()],
            matrix: &values[10 * self.nbody()..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MassMatrixLayout {
    output: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}
impl MassMatrixLayout {
    fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "mass_matrix_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "mass_matrix_layout",
        };
        let metadata = nv
            .checked_mul(2)
            .and_then(|n| n.checked_add(nbody))
            .ok_or_else(overflow)?;
        let stride = nbody
            .checked_mul(10)
            .and_then(|n| nv.checked_mul(nv).and_then(|m| n.checked_add(m)))
            .ok_or_else(overflow)?;
        if metadata > i32::MAX as usize || stride > i32::MAX as usize {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, guarded);
        Ok(Self {
            output,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

/// G02刚体子集的同步GPU探针。
/// GPU复用质心阶段设备结果。
/// GPU累加复合惯量并填充矩阵。
/// 自由度armature只加到对角线。
/// 参数仅共享一份模型。
/// 支持静态祖先与零自由度。
/// 不叠加肌腱或执行器惯量。
/// 不处理休眠、mocap或柔性体。
/// 不提供稀疏布局、分解或求解。
/// 不替代完整crb与G02入口。
/// 需要cuda-probe及NVRTC。
pub fn probe_mass_matrix(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<MassMatrixOutput, TransferError> {
    let (fk, com) = super::check_com_position(model, worlds, qpos)?;
    let k = model.kinematics();
    let layout = MassMatrixLayout::new(worlds, k.nbody(), k.nv())?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (session, fk, com, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        let (state, checkpoint) = super::com_position_device(session, model, fk, com, qpos)?;
        drop(checkpoint);
        let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
        let mut cursor = 0;
        for field in [
            &k.fields().body_parentid,
            &model.fields().dof_bodyid,
            &model.fields().dof_parentid,
        ] {
            metadata[cursor..cursor + field.len()].copy_from_slice(field);
            cursor += field.len();
        }
        let metadata = session.upload(&metadata)?;
        // A real pointer preserves the fixed ABI for the zero-DOF case.
        let armature = &model.fields().dof_armature;
        let parameters = session.upload(if armature.is_empty() {
            &[0.0]
        } else {
            armature
        })?;
        let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
        values.fill(-131072.0);
        values[4..layout.guarded - 4].fill(f32::NAN);
        let mut output = session.upload(&values)?;
        let kernel = crate::runtime::SynchronousKernel::compile(
            session,
            MASS_MATRIX_CUDA,
            "rigid_mass_matrix",
        )?;
        // SAFETY: Checked packing bounds all per-world indices by i32::MAX.
        // Validated body and DOF trees strictly precede children; owners fit nb.
        // State is the same-session COM buffer with 14*nb+6*nv values/world.
        // Each thread owns one world; the synchronized adapter retains buffers.
        unsafe {
            kernel.launch(
                &metadata,
                &parameters,
                &state,
                &mut output,
                [k.nbody() as u32, k.nv() as u32, 0, worlds as u32],
            )?;
        }
        output.read_range_into(0, &mut values)?;
        super::check_device_values(&values, "mass_matrix_output")?;
        Ok(MassMatrixOutput { layout, values })
    }
}

// Adapted from frozen smooth.py crb and math.py inert_vec at
// 71da24d956378a87a703b6e1442b13aec0c4ac29.
// Copyright 2025 The Newton Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(feature = "cuda-probe")]
const MASS_MATRIX_CUDA: &str = r#"
extern "C" __global__ void rigid_mass_matrix(const int* m,const float* p,
    const float* state,float* output,unsigned nb,unsigned nv,unsigned unused,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const int* parent=m; const int* body=m+nb; const int* dparent=body+nv;
  const float* s=state+4+(unsigned long long)w*(14*nb+6*nv);
  const float* ci=s+4*nb; const float* cd=ci+10*nb;
  float* crb=output+4+(unsigned long long)w*(10*nb+nv*nv);
  float* mat=crb+10*nb;
  for(unsigned a=0;a<10*nb;a++) crb[a]=ci[a];
  // Root subtrees use different COM frames. Never accumulate into world.
  for(int b=int(nb)-1;b>0;b--) if(parent[b]>0)
    for(int a=0;a<10;a++) crb[10*parent[b]+a]+=crb[10*b+a];
  for(unsigned a=0;a<nv*nv;a++) mat[a]=0.0f;
  for(unsigned d=0;d<nv;d++) {
    const float* i=crb+10*body[d]; const float* v=cd+6*d;
    float buf[6]={
      i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5],
      i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5],
      i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4],
      i[8]*v[1]-i[7]*v[2]+i[9]*v[3],
      i[6]*v[2]-i[8]*v[0]+i[9]*v[4],
      i[7]*v[0]-i[6]*v[1]+i[9]*v[5]};
    for(int j=int(d);j>=0;j=dparent[j]) {
      float value=(j==int(d))?p[d]:0.0f;
      for(int a=0;a<6;a++) value+=cd[6*j+a]*buf[a];
      mat[d*nv+j]=value; mat[j*nv+d]=value;
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_mass_matrix_dimensions_square_capacity_and_kernel_indices() {
        for dims in [
            (0, 1, 0),
            (1, 0, 0),
            (usize::MAX, 1, 0),
            (1, usize::MAX, 0),
            (1, 1, usize::MAX),
            (1, 1, 46341),
            (1, i32::MAX as usize, 1),
        ] {
            assert!(MassMatrixLayout::new(dims.0, dims.1, dims.2).is_err());
        }
        assert!(MassMatrixLayout::new(1, 1, 46340).is_ok());
    }
    #[test]
    fn mass_matrix_views_separate_worlds_inertias_and_dense_rows() {
        let layout = MassMatrixLayout::new(2, 2, 3).unwrap();
        let out = MassMatrixOutput {
            layout,
            values: (0..66).map(|n| n as f32).collect(),
        };
        assert_eq!(out.worlds(), 2);
        assert_eq!(out.nbody(), 2);
        assert_eq!(out.nv(), 3);
        let w = out.world(1).unwrap();
        assert_eq!(w.crb.len(), 20);
        assert_eq!(w.crb[0], 33.0);
        assert_eq!(
            w.matrix,
            &[53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0]
        );
        assert!(out.world(2).is_err());
        let out = MassMatrixOutput {
            layout: MassMatrixLayout::new(1, 1, 0).unwrap(),
            values: vec![0.0; 18],
        };
        assert!(out.world(0).unwrap().matrix.is_empty());
    }
    #[test]
    fn preflight_keeps_existing_com_and_state_rejections() {
        let m = super::super::tests::test_model(Some(3));
        assert!(super::super::check_com_position(&m, 2, &[0.0]).is_err());
        assert!(super::super::check_com_position(&m, 2, &[0.0, f32::NAN]).is_err());
        assert!(super::super::check_com_position(&m, 2, &[0.0, 0.5]).is_ok());
    }
}
