//! G03正定刚体质量矩阵辅助。
//! 不替代完整分解与求解入口。

use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;

mod resident;
pub use resident::{MassSolveData, MassSolvePlan};
mod sparse;
pub use sparse::{
    SparseMassData, SparseMassFactorData, SparseMassFactorOutput, SparseMassFactorPlan,
    SparseMassFactorWorld, SparseMassOutput, SparseMassPlan, SparseMassSolveData,
    SparseMassSolveOutput, SparseMassSolvePlan, SparseMassSolveWorld, SparseMassWorld,
};

/// 每个独立运动树的因子格式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MassFactorKind {
    /// 单自由度只存质量倒数，因子区域为零。
    Diagonal,
    /// 上三角U满足M = U^T U；倒数区域为零。
    Cholesky,
    /// 下三角L与对角D满足M = L^T D L。
    ReverseLdl,
}

/// 自由度区间共用同一因子格式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MassFactorBlock {
    pub start: usize,
    pub size: usize,
    pub kind: MassFactorKind,
}

#[cfg(any(feature = "cuda-probe", test))]
fn factor_blocks(parents: &[i32]) -> Vec<MassFactorBlock> {
    let mut blocks = Vec::<MassFactorBlock>::new();
    for (dof, &parent) in parents.iter().enumerate() {
        if parent < 0 {
            blocks.push(MassFactorBlock {
                start: dof,
                size: 0,
                kind: MassFactorKind::Diagonal,
            });
        }
        // 模型校验保证先祖编号更小。
        // 后续区间检查再拒绝交错树。
        blocks.last_mut().expect("checked DOF tree").size += 1;
    }
    for block in &mut blocks {
        block.kind = match block.size {
            1 => MassFactorKind::Diagonal,
            2..=64 => MassFactorKind::Cholesky,
            _ => MassFactorKind::ReverseLdl,
        };
    }
    blocks
}

#[cfg(any(feature = "cuda-probe", test))]
fn check_block_intervals(parents: &[i32], blocks: &[MassFactorBlock]) -> Result<(), InputError> {
    for block in blocks {
        if parents[block.start..block.start + block.size]
            .iter()
            .any(|&p| p >= 0 && (p as usize) < block.start)
        {
            return Err(InputError::InvalidDimension {
                field: "mass_solve_interleaved_trees",
            });
        }
    }
    Ok(())
}

/// 每世界分解与求解的只读视图。
#[derive(Clone, Copy, Debug)]
pub struct MassSolveWorld<'a> {
    /// 兼容字段名；按blocks解释稠密行优先因子。
    /// 不再统一采用旧版反向LDL格式。
    pub ld: &'a [f32],
    /// 对角块存质量倒数；LDL块存D倒数。
    /// Cholesky块填零，不提供无语义倒数。
    pub diagonal_inverse: &'a [f32],
    pub blocks: &'a [MassFactorBlock],
    /// 多右端项按`[rhs,nv]`连续排列。
    pub solution: &'a [f32],
}

/// 独立拥有的同步宿主结果。
#[derive(Clone, Debug)]
pub struct MassSolveOutput {
    layout: MassSolveLayout,
    values: Vec<f32>,
    blocks: Vec<MassFactorBlock>,
}
impl MassSolveOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn rhs_count(&self) -> usize {
        self.layout.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<MassSolveWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 7..r.end + 4];
        let square = self.nv() * self.nv();
        Ok(MassSolveWorld {
            blocks: &self.blocks,
            ld: &v[..square],
            diagonal_inverse: &v[square..square + self.nv()],
            solution: &v[square + self.nv()..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MassSolveLayout {
    output: BatchLayout,
    rhs: BatchLayout,
    nv: usize,
    rhs_count: usize,
    #[cfg(any(feature = "cuda-probe", test))]
    guarded: usize,
}
impl MassSolveLayout {
    fn new(worlds: usize, nv: usize, rhs_count: usize) -> Result<Self, InputError> {
        if worlds == 0
            || worlds > (u32::MAX - 255) as usize
            || rhs_count == 0
            || rhs_count > i32::MAX as usize
        {
            return Err(InputError::InvalidDimension {
                field: "mass_solve_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "mass_solve_layout",
        };
        let rhs_stride = nv.checked_mul(rhs_count).ok_or_else(overflow)?;
        let stride = nv
            .checked_mul(nv)
            .and_then(|n| n.checked_add(nv))
            .and_then(|n| n.checked_add(rhs_stride))
            .and_then(|n| n.checked_add(3))
            .ok_or_else(overflow)?;
        if stride > i32::MAX as usize {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let rhs = BatchLayout::new(worlds, rhs_stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        Ok(Self {
            output,
            rhs,
            nv,
            rhs_count,
            #[cfg(any(feature = "cuda-probe", test))]
            guarded,
        })
    }
    fn check_rhs(&self, rhs: &[f32]) -> Result<(), InputError> {
        if rhs.len() != self.rhs.total_elements() {
            return Err(InputError::LengthMismatch {
                field: "mass_solve_rhs",
                expected: self.rhs.total_elements(),
                actual: rhs.len(),
            });
        }
        for (index, &v) in rhs.iter().enumerate() {
            if !v.is_finite() {
                return Err(InputError::NonFinite {
                    field: "mass_solve_rhs",
                    index,
                });
            }
        }
        Ok(())
    }
}

/// 同步GPU分块因子与多右端项求解。
/// rhs采用`[world,rhs,nv]`布局。
/// GPU复用已检查的质量矩阵。
/// 主元必须严格为正且有限。
/// 接口不修补主元或回退CPU。
/// 零自由度允许空向量结果。
/// 单个失败世界阻止整个结果发布。
/// 单自由度采用倒数，至64维采用Cholesky。
/// 更大树保留稠密反向LDL辅助。
/// 不复制上游tile调度或多自由度紧凑优化。
/// 不替代U060、U065与完整G03。
pub fn probe_mass_solve(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
    rhs_count: usize,
    rhs: &[f32],
) -> Result<MassSolveOutput, TransferError> {
    // Complete input checks precede any device submission.
    super::check_com_position(model, worlds, qpos)?;
    let layout = MassSolveLayout::new(worlds, model.kinematics().nv(), rhs_count)?;
    layout.check_rhs(rhs)?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = session;
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        let blocks = factor_blocks(&model.fields().dof_parentid);
        check_block_intervals(&model.fields().dof_parentid, &blocks)?;
        let (state, checkpoint) =
            super::mass_matrix::mass_matrix_device(session, model, worlds, qpos)?;
        drop(checkpoint);
        solve_device(
            session,
            model.kinematics().nbody(),
            layout,
            &state,
            rhs,
            blocks,
        )
    }
}

#[cfg(feature = "cuda-probe")]
fn solve_device(
    session: &TransferSession,
    nbody: usize,
    layout: MassSolveLayout,
    state: &crate::runtime::TransferBuffer<f32>,
    rhs: &[f32],
    blocks: Vec<MassFactorBlock>,
) -> Result<MassSolveOutput, TransferError> {
    let mut packed = vec![blocks.len() as i32];
    for block in &blocks {
        let kind = match block.kind {
            MassFactorKind::Diagonal => 0,
            MassFactorKind::Cholesky => 1,
            MassFactorKind::ReverseLdl => 2,
        };
        packed.extend([block.start as i32, block.size as i32, kind]);
    }
    let metadata = session.upload(&packed)?;
    // Preserve a real pointer for the empty zero-DOF vector.
    let parameters = session.upload(if rhs.is_empty() { &[0.0] } else { rhs })?;
    let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
    values.fill(-131072.0);
    values[4..layout.guarded - 4].fill(f32::NAN);
    let mut output = session.upload(&values)?;
    let kernel =
        crate::runtime::SynchronousKernel::compile(session, MASS_SOLVE_CUDA, "rigid_mass_solve")?;
    // SAFETY: Layouts bound every per-world offset and nv*nv by i32::MAX.
    // The fixed shader and all physical buffers use frozen f32.
    // State comes from the same-session checked mass matrix with 10*nb+nv*nv
    // values/world and two four-element guards. Each thread owns one world.
    // RHS has exactly worlds*nrhs*nv values, or a real zero-DOF placeholder.
    // Metadata partitions nv into disjoint checked tree intervals.
    // The fixed-ABI adapter synchronizes and retains all submitted resources.
    unsafe {
        kernel.launch(
            &metadata,
            &parameters,
            state,
            &mut output,
            [
                nbody as u32,
                layout.nv as u32,
                layout.rhs_count as u32,
                layout.output.worlds() as u32,
            ],
        )?;
    }
    output.read_range_into(0, &mut values)?;
    check_result(layout, &values)?;
    Ok(MassSolveOutput {
        layout,
        values,
        blocks,
    })
}

#[cfg(any(feature = "cuda-probe", test))]
fn check_result<T: Copy + Into<f64>>(
    layout: MassSolveLayout,
    values: &[T],
) -> Result<(), TransferError> {
    // Failure statuses may precede uninitialized payload. Check guards first,
    // then decode explicit statuses before requiring the whole payload finite.
    super::check_device_guards(values)?;
    for world in 0..layout.output.worlds() {
        let r = layout.output.world_elements(world)?;
        let s: [f64; 3] = [
            values[r.start + 4].into(),
            values[r.start + 5].into(),
            values[r.start + 6].into(),
        ];
        if s[0] == 0.0 && s[1] == -1.0 && s[2] == 0.0 {
            continue;
        }
        let dof = s[1] as usize;
        if !s[1].is_finite() || s[1] != dof as f64 || dof >= layout.nv {
            return Err(ProbeError::InvalidArgument("GPU求解状态无效").into());
        }
        if s[0] == 1.0 {
            return Err(TransferError::InvalidPivot {
                world,
                dof,
                value: s[2],
            });
        }
        if s[0] == 2.0 && (!s[2].is_finite() || s[2] > f64::from(f32::MAX)) {
            return Err(InputError::NonFinite {
                field: "mass_solve_diagonal_inverse",
                index: world * layout.nv + dof,
            }
            .into());
        }
        return Err(ProbeError::InvalidArgument("GPU求解状态无效").into());
    }
    super::check_device_values(values, "mass_solve_output")
}

// 冻结smooth.py提供Cholesky与反向LDL公式。
// 本辅助按树分块；每线程处理一个世界。
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
const MASS_SOLVE_CUDA: &str = r#"
__device__ float tile_schur_update(float value,float a,float b) {
  float result;
  // Match the frozen MathDx PTX, including its permission to contract.
  // Explicit .rn on mul/sub would forbid the driver's FMA contraction.
  asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
      : "=f"(result) : "f"(value), "f"(a), "f"(b));
  return result;
}
extern "C" __global__ void rigid_mass_solve(const int* blocks,const float* rhs,
    const float* state,float* output,unsigned nb,unsigned n,unsigned nr,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const float* mat=state+4+(unsigned long long)w*(10*nb+n*n)+10*nb;
  float* status=output+4+(unsigned long long)w*(3+n*n+n+nr*n);
  float* ld=status+3; float* inv=ld+n*n; float* x=inv+n;
  status[0]=0.0f; status[1]=-1.0f; status[2]=0.0f;
  for(unsigned i=0;i<n*n;i++) ld[i]=0.0f;
  for(unsigned i=0;i<n;i++) inv[i]=0.0f;
  for(int block=0;block<blocks[0];block++) {
    int begin=blocks[1+3*block], end=begin+blocks[2+3*block], kind=blocks[3+3*block];
    if(kind==0) {
      float pivot=mat[begin*n+begin];
      if(!(pivot>0.0f) || !isfinite(pivot)) {
        status[0]=1.0f; status[1]=float(begin); status[2]=pivot; return;
      }
      inv[begin]=1.0f/pivot;
      if(!isfinite(inv[begin])) {
        status[0]=2.0f; status[1]=float(begin); status[2]=inv[begin]; return;
      }
    } else if(kind==1) {
      // Frozen MathDx tile PTX uses rsqrt.approx plus multiply.
      // Preserve the mul/sub contract for each Schur update.
      // The small scalar route (through six DOFs) retains sqrt/divide/FMA.
      bool tile=end-begin>6;
      for(int i=begin;i<end;i++) {
        float pivot=mat[i*n+i];
        for(int k=begin;k<i;k++) {
          float u=ld[k*n+i];
          pivot=tile ? tile_schur_update(pivot,u,u) : fmaf(-u,u,pivot);
        }
        if(!(pivot>0.0f) || !isfinite(pivot)) {
          status[0]=1.0f; status[1]=float(i); status[2]=pivot; return;
        }
        float reciprocal, diagonal;
        if(tile) {
          asm("rsqrt.approx.f32 %0, %1;" : "=f"(reciprocal) : "f"(pivot));
          diagonal=__fmul_rn(pivot,reciprocal);
        } else {
          diagonal=sqrtf(pivot);
          reciprocal=1.0f/diagonal;
        }
        ld[i*n+i]=diagonal;
        for(int j=i+1;j<end;j++) {
          float value=mat[i*n+j];
          for(int k=begin;k<i;k++) {
            float u=ld[k*n+i], v=ld[k*n+j];
            value=tile ? tile_schur_update(value,u,v) : fmaf(-u,v,value);
          }
          ld[i*n+j]=__fmul_rn(value,reciprocal);
        }
      }
    } else {
    for(int i=begin;i<end;i++) for(int j=begin;j<=i;j++) ld[i*n+j]=mat[i*n+j];
    for(int k=end-1;k>=begin;k--) {
    float pivot=ld[k*n+k];
    if(!(pivot>0.0f) || !isfinite(pivot)) {
      status[0]=1.0f; status[1]=float(k); status[2]=pivot; return;
    }
    float reciprocal=1.0f/pivot; inv[k]=reciprocal;
    if(!isfinite(reciprocal) || reciprocal>3.4028234663852886e38f) {
      status[0]=2.0f; status[1]=float(k); status[2]=reciprocal; return;
    }
    // Keep the original row until every preceding row update completes.
    for(int i=k-1;i>=begin;i--) {
      float tmp=ld[k*n+i]*reciprocal;
      for(int j=begin;j<=i;j++) ld[i*n+j]-=ld[k*n+j]*tmp;
    }
    for(int j=begin;j<k;j++) ld[k*n+j]*=reciprocal;
    }
    }
  }
  if(n==0) return;
  const float* b=rhs+(unsigned long long)w*nr*n;
  for(unsigned r=0;r<nr;r++) {
    float* v=x+r*n;
    for(unsigned i=0;i<n;i++) v[i]=b[r*n+i];
    for(int block=0;block<blocks[0];block++) {
      int begin=blocks[1+3*block], end=begin+blocks[2+3*block], kind=blocks[3+3*block];
      if(kind==0) {
        v[begin]*=inv[begin];
      } else if(kind==1) {
        for(int i=begin;i<end;i++) {
          float value=v[i];
          for(int j=begin;j<i;j++) value-=ld[j*n+i]*v[j];
          v[i]=value/ld[i*n+i];
        }
        for(int i=end-1;i>=begin;i--) {
          float value=v[i];
          for(int j=i+1;j<end;j++) value-=ld[i*n+j]*v[j];
          v[i]=value/ld[i*n+i];
        }
      } else {
    // 反向LDL块依次执行L^-T、D^-1与L^-1。
    for(int k=end-1;k>=begin;k--)
      for(int i=begin;i<k;i++) v[i]-=ld[k*n+i]*v[k];
    for(int i=begin;i<end;i++) v[i]*=inv[i];
    for(int i=begin;i<end;i++) {
      float sum=0.0f;
      for(int j=begin;j<i;j++) sum+=ld[i*n+j]*v[j];
      v[i]-=sum;
    }
      }
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_solve_dimensions_square_and_rhs_capacity() {
        for (w, n, r) in [
            (0, 1, 1),
            (usize::MAX, 1, 1),
            (1, 1, 0),
            (1, 1, usize::MAX),
            (1, usize::MAX, 1),
            (1, 46341, 1),
            (1, 1, i32::MAX as usize),
        ] {
            assert!(MassSolveLayout::new(w, n, r).is_err());
        }
        assert!(MassSolveLayout::new(1, 0, 3).is_ok());
        assert!(MassSolveLayout::new(1, 46340, 1).is_err());
        assert!(MassSolveLayout::new(1, 46339, 1).is_ok());
        let layout = MassSolveLayout::new(513, 2, 3).unwrap();
        assert_eq!(layout.output.total_bytes(), 513 * 15 * size_of::<f32>());
        assert_eq!(layout.rhs.total_bytes(), 513 * 6 * size_of::<f32>());
    }
    #[test]
    fn rejects_rhs_length_and_nonfinite_values() {
        let l = MassSolveLayout::new(2, 2, 3).unwrap();
        assert!(matches!(
            l.check_rhs(&[0.0]),
            Err(InputError::LengthMismatch { expected: 12, .. })
        ));
        let mut rhs = [0.0; 12];
        rhs[7] = f32::INFINITY;
        assert_eq!(
            l.check_rhs(&rhs),
            Err(InputError::NonFinite {
                field: "mass_solve_rhs",
                index: 7
            })
        );
        assert!(l.check_rhs(&[0.0; 12]).is_ok());
    }
    #[test]
    fn solve_views_skip_status_and_separate_worlds_rhs_and_factors() {
        let l = MassSolveLayout::new(2, 2, 3).unwrap();
        let out = MassSolveOutput {
            layout: l,
            values: (0..l.guarded).map(|n| n as f32).collect(),
            blocks: factor_blocks(&[-1, 0]),
        };
        assert_eq!(out.worlds(), 2);
        assert_eq!(out.nv(), 2);
        assert_eq!(out.rhs_count(), 3);
        let w = out.world(1).unwrap();
        assert_eq!(w.ld, &[22.0, 23.0, 24.0, 25.0]);
        assert_eq!(w.diagonal_inverse, &[26.0, 27.0]);
        assert_eq!(w.solution, &[28.0, 29.0, 30.0, 31.0, 32.0, 33.0]);
        assert!(out.world(2).is_err());
        let l = MassSolveLayout::new(1, 0, 3).unwrap();
        let out = MassSolveOutput {
            layout: l,
            values: vec![0.0; l.guarded],
            blocks: vec![],
        };
        let w = out.world(0).unwrap();
        assert!(w.ld.is_empty() && w.diagonal_inverse.is_empty() && w.solution.is_empty());
    }
    #[test]
    fn statuses_precede_partial_payload_but_never_skip_guards() {
        let l = MassSolveLayout::new(2, 2, 1).unwrap();
        let mut v = vec![f32::NAN; l.guarded];
        v[..4].fill(-131072.0);
        v[l.guarded - 4..].fill(-131072.0);
        v[4..7].copy_from_slice(&[0.0, -1.0, 0.0]);
        let start = 4 + l.output.elements_per_world();
        v[start..start + 3].copy_from_slice(&[1.0, 0.0, -3.0]);
        assert_eq!(
            check_result(l, &v),
            Err(TransferError::InvalidPivot {
                world: 1,
                dof: 0,
                value: -3.0
            })
        );
        v[0] = 0.0;
        assert!(matches!(
            check_result(l, &v),
            Err(TransferError::Backend(ProbeError::Mismatch {
                index: 0,
                ..
            }))
        ));
    }
    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn device_factorization_rejects_indefinite_world_and_recovers() {
        let s = TransferSession::new(0).unwrap();
        let l = MassSolveLayout::new(3, 2, 1).unwrap();
        let mut state = vec![-131072.0; 50];
        for w in 0..3 {
            state[4 + 14 * w..4 + 14 * w + 10].fill(0.0);
            state[14 + 14 * w..18 + 14 * w].copy_from_slice(&[2.0, 0.0, 0.0, 3.0]);
        }
        state[28..32].copy_from_slice(&[1.0, 2.0, 2.0, 1.0]);
        let b = s.upload(&state).unwrap();
        assert_eq!(
            solve_device(&s, 1, l, &b, &[1.0; 6], reverse_test_block()).unwrap_err(),
            TransferError::InvalidPivot {
                world: 1,
                dof: 0,
                value: -3.0
            }
        );
        for pivot in [-1.0 - 2.0f32.powi(-23), 0.0, f32::INFINITY, f32::NAN] {
            state[28..32].copy_from_slice(&[pivot, 0.0, 0.0, 3.0]);
            let b = s.upload(&state).unwrap();
            let TransferError::InvalidPivot { world, dof, value } =
                solve_device(&s, 1, l, &b, &[1.0; 6], reverse_test_block()).unwrap_err()
            else {
                panic!("expected invalid pivot")
            };
            assert_eq!((world, dof), (1, 0));
            if pivot.is_nan() {
                assert!(value.is_nan());
            } else {
                assert_eq!(value.to_bits(), f64::from(pivot).to_bits());
            }
        }
        state[28..32].copy_from_slice(&[2.0, 0.0, 0.0, 3.0]);
        let b = s.upload(&state).unwrap();
        let out = solve_device(&s, 1, l, &b, &[1.0; 6], reverse_test_block()).unwrap();
        assert_eq!(out.world(2).unwrap().solution, &[0.5, 1.0 / 3.0]);
    }
    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn device_reciprocal_and_solution_overflow_do_not_poison_session() {
        let s = TransferSession::new(0).unwrap();
        let l = MassSolveLayout::new(1, 1, 1).unwrap();
        let mut state = vec![-131072.0; 19];
        state[4..14].fill(0.0);
        state[14] = 1e-39;
        let b = s.upload(&state).unwrap();
        assert!(matches!(
            solve_device(&s, 1, l, &b, &[1.0], factor_blocks(&[-1])),
            Err(TransferError::Input(InputError::NonFinite {
                field: "mass_solve_diagonal_inverse",
                index: 0
            }))
        ));
        state[14] = 1e-30;
        let b = s.upload(&state).unwrap();
        assert!(matches!(
            solve_device(&s, 1, l, &b, &[1e20], factor_blocks(&[-1])),
            Err(TransferError::Input(InputError::NonFinite {
                field: "mass_solve_output",
                ..
            }))
        ));
        state[14] = 2.0;
        let b = s.upload(&state).unwrap();
        assert_eq!(
            solve_device(&s, 1, l, &b, &[1.0], factor_blocks(&[-1]))
                .unwrap()
                .world(0)
                .unwrap()
                .solution,
            &[0.5]
        );
    }

    #[cfg(feature = "cuda-probe")]
    fn reverse_test_block() -> Vec<MassFactorBlock> {
        vec![MassFactorBlock {
            start: 0,
            size: 2,
            kind: MassFactorKind::ReverseLdl,
        }]
    }

    #[test]
    fn partitions_independent_trees_and_freezes_factor_formats() {
        assert!(factor_blocks(&[]).is_empty());
        let blocks = factor_blocks(&[-1, 0, -1]);
        assert!(check_block_intervals(&[-1, 0, -1], &blocks).is_ok());
        let interleaved = [-1, -1, 0];
        assert!(check_block_intervals(&interleaved, &factor_blocks(&interleaved)).is_err());
        assert_eq!(
            blocks,
            vec![
                MassFactorBlock {
                    start: 0,
                    size: 2,
                    kind: MassFactorKind::Cholesky
                },
                MassFactorBlock {
                    start: 2,
                    size: 1,
                    kind: MassFactorKind::Diagonal
                },
            ]
        );
        for (size, kind) in [
            (64, MassFactorKind::Cholesky),
            (65, MassFactorKind::ReverseLdl),
        ] {
            let parents: Vec<i32> = (0..size).map(|i| i - 1).collect();
            assert_eq!(factor_blocks(&parents)[0].kind, kind);
        }
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn device_cholesky_rejects_bad_pivots_and_recovers() {
        let s = TransferSession::new(0).unwrap();
        let l = MassSolveLayout::new(1, 2, 1).unwrap();
        let mut state = vec![-131072.0; 22];
        state[4..14].fill(0.0);
        for pivot in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            state[14..18].copy_from_slice(&[pivot, 0.0, 0.0, 3.0]);
            let b = s.upload(&state).unwrap();
            assert!(matches!(
                solve_device(&s, 1, l, &b, &[1.0; 2], factor_blocks(&[-1, 0])),
                Err(TransferError::InvalidPivot {
                    world: 0,
                    dof: 0,
                    ..
                })
            ));
        }
        state[14..18].copy_from_slice(&[1.0, 2.0, 2.0, 1.0]);
        let b = s.upload(&state).unwrap();
        assert_eq!(
            solve_device(&s, 1, l, &b, &[1.0; 2], factor_blocks(&[-1, 0])).unwrap_err(),
            TransferError::InvalidPivot {
                world: 0,
                dof: 1,
                value: -3.0
            }
        );
        state[14..18].copy_from_slice(&[4.0, 2.0, 2.0, 5.0]);
        let b = s.upload(&state).unwrap();
        let out = solve_device(&s, 1, l, &b, &[8.0, 12.0], factor_blocks(&[-1, 0])).unwrap();
        let w = out.world(0).unwrap();
        assert_eq!(w.ld, &[2.0, 1.0, 0.0, 2.0]);
        assert_eq!(w.diagonal_inverse, &[0.0, 0.0]);
        assert_eq!(w.solution, &[1.0, 2.0]);
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    fn first_difference_evidence_binds_inputs_and_upstream() {
        use sha2::{Digest, Sha256};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mjwarp-mass-solve/first-difference/manifest.json"
        ))
        .unwrap();
        for (path, hash) in manifest["files"].as_object().unwrap() {
            assert_eq!(
                format!(
                    "{:x}",
                    Sha256::digest(std::fs::read(root.join(path)).unwrap())
                ),
                hash.as_str().unwrap(),
                "{path}"
            );
        }
        let upstream: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mjwarp-mass-solve/first-difference/upstream.json"
        ))
        .unwrap();
        let baseline: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mjwarp-mass-solve/first-difference/baseline.json"
        ))
        .unwrap();
        assert_eq!(
            upstream["revision"],
            "71da24d956378a87a703b6e1442b13aec0c4ac29"
        );
        assert_eq!(upstream["worlds"], 8);
        assert_eq!(upstream["rhs_count"], 3);
        for field in [
            "xpos",
            "xquat",
            "xmat",
            "xipos",
            "ximat",
            "xanchor",
            "xaxis",
            "subtree_com",
            "cinert",
            "cdof",
            "crb",
            "matrix",
        ] {
            let bits = |v: &serde_json::Value| {
                v.as_array()
                    .unwrap()
                    .iter()
                    .map(|x| (x.as_f64().unwrap() as f32).to_bits())
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                bits(&baseline["rows"][1][field]),
                bits(&upstream["rows"][1][field]),
                "{field}"
            );
        }
        assert_ne!(
            baseline["rows"][1]["factor"][0],
            upstream["rows"][1]["factor"][0]
        );
    }

    #[cfg(feature = "cuda-probe")]
    #[test]
    #[ignore = "requires NVIDIA driver and NVRTC"]
    fn device_tile_factor_matches_frozen_matrix_before_back_substitution() {
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../fixtures/mjwarp-mass-solve/first-difference/upstream.json"
        ))
        .unwrap();
        let rows = reference["rows"].as_array().unwrap();
        let s = TransferSession::new(0).unwrap();
        let n = 11;
        let stride = 10 + n * n;
        let mut state = vec![-131072.0; 8 + rows.len() * stride];
        for (w, row) in rows.iter().enumerate() {
            let start = 4 + w * stride;
            state[start..start + 10].fill(0.0);
            for (out, v) in state[start + 10..start + stride]
                .iter_mut()
                .zip(row["matrix"].as_array().unwrap())
            {
                *out = v.as_f64().unwrap() as f32;
            }
        }
        let parents: Vec<_> = (0..n as i32).map(|i| i - 1).collect();
        let layout = MassSolveLayout::new(rows.len(), n, 1).unwrap();
        let out = solve_device(
            &s,
            1,
            layout,
            &s.upload(&state).unwrap(),
            &vec![0.0; rows.len() * n],
            factor_blocks(&parents),
        )
        .unwrap();
        for (w, row) in rows.iter().enumerate() {
            let factor = out.world(w).unwrap().ld;
            for (i, (a, b)) in factor
                .iter()
                .zip(row["factor"].as_array().unwrap())
                .enumerate()
            {
                let b = b.as_f64().unwrap() as f32;
                // Same-matrix trace bounds the observed MathDx rounding only.
                // It does not replace the full native-first solution gates.
                assert!(
                    a.to_bits().abs_diff(b.to_bits()) <= 1,
                    "same-matrix factor world={w} index={i}: {a} != {b}"
                );
            }
            assert!(out.world(w).unwrap().solution.iter().all(|&x| x == 0.0));
        }
        assert_eq!(
            out.world(1).unwrap().ld[0].to_bits(),
            3.741_657_3_f32.to_bits()
        );

        // Exercise invalid tile pivots, then recover on the same session.
        let layout = MassSolveLayout::new(1, 7, 1).unwrap();
        let parents: Vec<_> = (0..7).map(|i| i - 1).collect();
        let mut state = vec![-131072.0; 67];
        state[4..63].fill(0.0);
        for i in 0..7 {
            state[14 + i * 8] = 4.0;
        }
        for pivot in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            state[14] = pivot;
            assert!(matches!(
                solve_device(
                    &s,
                    1,
                    layout,
                    &s.upload(&state).unwrap(),
                    &[1.0; 7],
                    factor_blocks(&parents)
                ),
                Err(TransferError::InvalidPivot { dof: 0, .. })
            ));
        }
        state[14] = 4.0;
        let out = solve_device(
            &s,
            1,
            layout,
            &s.upload(&state).unwrap(),
            &[1.0; 7],
            factor_blocks(&parents),
        )
        .unwrap();
        assert_eq!(out.world(0).unwrap().solution, &[0.25; 7]);
    }
}

#[cfg(all(test, feature = "cuda-probe"))]
#[path = "../../tests/support/g03_first_difference.rs"]
mod first_difference;
