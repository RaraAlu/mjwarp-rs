//! 常驻相机与光源的GPU子集。
//! 不渲染图像或创建窗口。

use crate::diagnostics::InputError;
use crate::model::BatchLayout;

#[derive(Clone, Copy, Debug)]
pub struct CamLightWorld<'a> {
    pub cam_xpos: &'a [f32],
    pub cam_xmat: &'a [f32],
    pub light_xpos: &'a [f32],
    pub light_xdir: &'a [f32],
}

/// 独立拥有四项宿主结果。
/// 矩阵按行展开。
#[derive(Clone, Debug)]
pub struct CamLightOutput {
    pub(super) layout: CamLightLayout,
    pub(super) values: Vec<f32>,
}

impl CamLightOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn ncam(&self) -> usize {
        self.layout.ncam
    }
    pub fn nlight(&self) -> usize {
        self.layout.nlight
    }
    pub fn world(&self, world: usize) -> Result<CamLightWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let nc = self.ncam();
        let nl = self.nlight();
        Ok(CamLightWorld {
            cam_xpos: &v[..3 * nc],
            cam_xmat: &v[3 * nc..12 * nc],
            light_xpos: &v[12 * nc..12 * nc + 3 * nl],
            light_xdir: &v[12 * nc + 3 * nl..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CamLightLayout {
    pub(super) output: BatchLayout,
    pub(super) ncam: usize,
    pub(super) nlight: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
}

impl CamLightLayout {
    pub(super) fn new(worlds: usize, ncam: usize, nlight: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "camlight_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "camlight_layout",
        };
        let stride = ncam
            .checked_mul(12)
            .and_then(|n| nlight.checked_mul(6).and_then(|l| n.checked_add(l)))
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
            ncam,
            nlight,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
    pub(super) fn is_empty(self) -> bool {
        self.ncam == 0 && self.nlight == 0
    }
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
// Vector normalization follows NVIDIA/warp v1.15.0 (Apache-2.0).
// Copyright (c) 2022 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
#[cfg(feature = "cuda-probe")]
pub(super) const CAMLIGHT_CUDA: &str = r#"
// Warp 1.15.0 builtin.h defines kEps as zero. Keep sqrt/division semantics.
__device__ V light_normalize(V v) {
  float len=sqrtf(v.x*v.x+v.y*v.y+v.z*v.z);
  return len>0.0f ? V{v.x/len,v.y/len,v.z/len} : V{0,0,0};
}
__device__ V light_cross(V a,V b) {
  return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x};
}
__device__ void look_at(float* mat,V pos,V target) {
  V z=light_normalize(add(pos,scale(target,-1.0f)));
  V x=light_normalize(light_cross(V{0,0,1},z));
  V y=light_normalize(light_cross(z,x));
  mat[0]=x.x;mat[1]=y.x;mat[2]=z.x;
  mat[3]=x.y;mat[4]=y.y;mat[5]=z.y;
  mat[6]=x.z;mat[7]=y.z;mat[8]=z.z;
}
extern "C" __global__ void camlight_rigid(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned nc,unsigned nl,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* cm=meta+2;const int* cb=cm+nc;const int* ct=cb+nc;
  const int* lm=ct+nc;const int* lb=lm+nl;const int* lt=lb+nl;
  const int* d=lt+nl;
  const float* cp=field_parameter(model,d,w,3*nc);
  const float* cq=field_parameter(model,d+2,w,4*nc);
  const float* cp0=field_parameter(model,d+6,w,3*nc);
  const float* cmat=field_parameter(model,d+8,w,9*nc);
  const float* lp=field_parameter(model,d+10,w,3*nl);
  const float* ld=field_parameter(model,d+12,w,3*nl);
  const float* lp0=field_parameter(model,d+16,w,3*nl);
  const float* ld0=field_parameter(model,d+18,w,3*nl);
  const float* xp=body+4+(unsigned long long)w*meta[0];
  const float* xq=xp+3*nb;
  float* cx=result+4+(unsigned long long)w*(12*nc+6*nl);
  float* mx=cx+3*nc;float* lx=mx+9*nc;float* dx=lx+3*nl;
  for(unsigned c=0;c<nc;++c) {
    int b=cb[c],mode=cm[c],t=ct[c]; Q q=load_q(xq+4*b);
    V pos=add(load_v(xp+3*b),rotate(q,load_v(cp+3*c)));
    if(mode==1) pos=add(load_v(xp+3*b),load_v(cp0+3*c));
    store_v(cx+3*c,pos);
    if(mode==1 || mode==2) {
      for(int j=0;j<9;++j) mx[9*c+j]=cmat[9*c+j];
    } else if(mode==3 && t>=0) look_at(mx+9*c,pos,load_v(xp+3*t));
    else matrix(mx+9*c,multiply(q,load_q(cq+4*c)));
  }
  for(unsigned l=0;l<nl;++l) {
    int b=lb[l],mode=lm[l],t=lt[l]; Q q=load_q(xq+4*b);
    V pos=add(load_v(xp+3*b),rotate(q,load_v(lp+3*l)));
    V dir=rotate(q,load_v(ld+3*l));
    bool invalid=(mode==3 || mode==4) && t<0;
    if(mode==1) pos=add(load_v(xp+3*b),load_v(lp0+3*l));
    if(mode==1 || mode==2) dir=load_v(ld0+3*l);
    else if(mode==3 && t>=0) dir=add(load_v(xp+3*t),scale(pos,-1.0f));
    store_v(lx+3*l,pos);
    store_v(dx+3*l,invalid ? dir : light_normalize(dir));
  }
}
// The first launch stores fixed positions. This launch only adjusts COM modes.
// Reading the same world's output is safe after the synchronous FK launch.
extern "C" __global__ void camlight_com(const int* meta,const float* model,
    const float* com,float* result,unsigned nb,unsigned nc,unsigned nl,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* cm=meta+2;const int* cb=cm+nc;const int* ct=cb+nc;
  const int* lm=ct+nc;const int* lb=lm+nl;const int* lt=lb+nl;
  const int* d=lt+nl;
  const float* cpc=field_parameter(model,d+4,w,3*nc);
  const float* lpc=field_parameter(model,d+14,w,3*nl);
  const float* sc=com+4+(unsigned long long)w*meta[1]+nb;
  float* cx=result+4+(unsigned long long)w*(12*nc+6*nl);
  float* mx=cx+3*nc;float* lx=mx+9*nc;float* dx=lx+3*nl;
  for(unsigned c=0;c<nc;++c) {
    if(cm[c]==2) store_v(cx+3*c,add(load_v(sc+3*cb[c]),load_v(cpc+3*c)));
    else if(cm[c]==4 && ct[c]>=0) look_at(mx+9*c,load_v(cx+3*c),load_v(sc+3*ct[c]));
  }
  for(unsigned l=0;l<nl;++l) {
    if(lm[l]==2) store_v(lx+3*l,add(load_v(sc+3*lb[l]),load_v(lpc+3*l)));
    else if(lm[l]==4 && lt[l]>=0)
      store_v(dx+3*l,light_normalize(add(load_v(sc+3*lt[l]),scale(load_v(lx+3*l),-1.0f))));
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_capacity_and_exposes_disjoint_world_views() {
        for (nc, nl) in [(2, 3), (0, 3), (2, 0), (0, 0)] {
            let layout = CamLightLayout::new(2, nc, nl).unwrap();
            let stride = 12 * nc + 6 * nl;
            let out = CamLightOutput {
                layout,
                values: (0..2 * stride + 8).map(|n| n as f32).collect(),
            };
            let w = out.world(1).unwrap();
            assert_eq!((out.worlds(), out.ncam(), out.nlight()), (2, nc, nl));
            assert_eq!(w.cam_xpos.len(), 3 * nc);
            assert_eq!(w.cam_xmat.len(), 9 * nc);
            assert_eq!(w.light_xpos.len(), 3 * nl);
            assert_eq!(w.light_xdir.len(), 3 * nl);
            assert_eq!(
                [w.cam_xpos, w.cam_xmat, w.light_xpos, w.light_xdir].concat(),
                (stride + 4..2 * stride + 4)
                    .map(|n| n as f32)
                    .collect::<Vec<_>>()
            );
            assert!(out.world(2).is_err());
        }
        for (w, nc, nl) in [
            (0, 0, 0),
            (u32::MAX as usize, 1, 1),
            (1, usize::MAX, 0),
            (1, 0, usize::MAX),
            (1, i32::MAX as usize / 12 + 1, 0),
            (1, 0, i32::MAX as usize / 6 + 1),
        ] {
            assert!(CamLightLayout::new(w, nc, nl).is_err());
        }
    }
}
