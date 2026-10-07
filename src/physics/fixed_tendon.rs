//! 固定肌腱长度与稀疏力臂。
//! 不计算空间绕行或包裹点。

use crate::diagnostics::InputError;
use crate::model::{BatchLayout, FixedTendonRows};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct FixedTendonWorld<'a> {
    pub ten_length: &'a [f32],
    pub ten_jacobian: &'a [f32],
}

/// 独立拥有两项宿主结果。
/// CSR布局只读且跨世界共享。
#[derive(Clone, Debug)]
pub struct FixedTendonOutput {
    pub(super) layout: FixedTendonLayout,
    pub(super) values: Vec<f32>,
    pub(super) rows: Arc<FixedTendonRows>,
}
impl FixedTendonOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn rows(&self) -> &FixedTendonRows {
        &self.rows
    }
    pub fn world(&self, world: usize) -> Result<FixedTendonWorld<'_>, InputError> {
        let range = self.layout.output.world_elements(world)?;
        let values = &self.values[range.start + 4..range.end + 4];
        Ok(FixedTendonWorld {
            ten_length: &values[..self.layout.ntendon],
            ten_jacobian: &values[self.layout.ntendon..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FixedTendonLayout {
    pub(super) output: BatchLayout,
    pub(super) ntendon: usize,
    pub(super) nnz: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
}
impl FixedTendonLayout {
    pub(super) fn new(worlds: usize, ntendon: usize, nnz: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "fixed_tendon_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "fixed_tendon_layout",
        };
        let stride = ntendon
            .checked_add(nnz)
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
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
            ntendon,
            nnz,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
    pub(super) fn is_empty(self) -> bool {
        self.ntendon == 0 && self.nnz == 0
    }
}

// Adapted from mujoco_warp/_src/smooth.py::_joint_tendon at
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
pub(super) const FIXED_TENDON_CUDA: &str = r#"
extern "C" __global__ void fixed_tendon(const int* meta,const float* coefficients,
    const float* state,float* result,unsigned nq,unsigned nt,unsigned nnz,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* adr=meta+1;const int* num=adr+nt;
  const int* qadr=num+nt;const int* jslot=qadr+meta[0];
  const float* qpos=state+(unsigned long long)w*nq;
  float* length=result+4+(unsigned long long)w*((unsigned long long)nt+nnz);
  float* jac=length+nt;
  for(unsigned i=0;i<nnz;++i) jac[i]=0.0f;
  for(unsigned t=0;t<nt;++t) {
    float sum=0.0f;
    for(int i=adr[t];i<adr[t]+num[t];++i) {
      // Raw qpos, not qpos-qpos0. Each row rejects duplicate joint references.
      sum+=coefficients[i]*qpos[qadr[i]];
      jac[jslot[i]]=coefficients[i];
    }
    length[t]=sum;
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_capacity_before_device_allocation() {
        for worlds in [0, u32::MAX as usize, usize::MAX] {
            assert!(FixedTendonLayout::new(worlds, 1, 1).is_err());
        }
        for (nt, nnz) in [(usize::MAX, 1), (1, usize::MAX), (i32::MAX as usize, 1)] {
            assert!(FixedTendonLayout::new(1, nt, nnz).is_err());
        }
        assert!(FixedTendonLayout::new(513, 0, 0).unwrap().is_empty());
    }
    #[test]
    fn exposes_disjoint_world_views_and_checks_indices() {
        let layout = FixedTendonLayout::new(2, 2, 3).unwrap();
        let out = FixedTendonOutput {
            layout,
            values: (0..18).map(|i| i as f32).collect(),
            rows: Arc::new(FixedTendonRows::default()),
        };
        assert_eq!(out.world(0).unwrap().ten_length, [4.0, 5.0]);
        assert_eq!(out.world(1).unwrap().ten_jacobian, [11.0, 12.0, 13.0]);
        assert!(out.world(2).is_err());
    }
}
