//! site、滑轮与球柱空间肌腱子集。
//! 不计算肌腱速度或限位。

use crate::diagnostics::InputError;
use crate::model::{BatchLayout, SpatialTendonRows};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct SpatialTendonWorld<'a> {
    pub ten_length: &'a [f32],
    pub ten_jacobian: &'a [f32],
    pub ten_wrapadr: &'a [i32],
    pub ten_wrapnum: &'a [i32],
    /// 原生双点布局，保留未使用槽位。
    pub wrap_obj: &'a [i32],
    /// 原生双点布局，每点三标量。
    pub wrap_xpos: &'a [f32],
}

#[derive(Clone, Debug)]
pub struct SpatialTendonOutput {
    pub(super) layout: SpatialTendonLayout,
    pub(super) values: Vec<f32>,
    pub(super) indices: Vec<i32>,
    pub(super) rows: Arc<SpatialTendonRows>,
}
impl SpatialTendonOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn rows(&self) -> &SpatialTendonRows {
        &self.rows
    }
    pub fn world(&self, world: usize) -> Result<SpatialTendonWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let i = self.layout.indices.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let ids = &self.indices[i.start + 4..i.end + 4];
        let nt = self.layout.ntendon;
        let nnz = self.rows.nnz();
        Ok(SpatialTendonWorld {
            ten_length: &v[..nt],
            ten_jacobian: &v[nt..nt + nnz],
            wrap_xpos: &v[nt + nnz..nt + nnz + 6 * self.rows.nwrap()],
            ten_wrapadr: &ids[..nt],
            ten_wrapnum: &ids[nt..2 * nt],
            wrap_obj: &ids[2 * nt..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SpatialTendonLayout {
    pub(super) output: BatchLayout,
    pub(super) indices: BatchLayout,
    pub(super) ntendon: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded_indices: usize,
}
impl SpatialTendonLayout {
    pub(super) fn new(worlds: usize, nt: usize, nnz: usize, nw: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "spatial_tendon_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "spatial_tendon_layout",
        };
        // 每项保留六个公开坐标。
        // 内部另存七值和方向六分量。
        let floats = nw
            .checked_mul(19)
            .and_then(|n| n.checked_add(nt))
            .and_then(|n| n.checked_add(nnz))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let ints = nt
            .checked_add(nw)
            .and_then(|n| n.checked_mul(2))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let output = BatchLayout::new(worlds, floats, 4)?;
        let indices = BatchLayout::new(worlds, ints, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        let guarded_indices = indices
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        BatchLayout::new(1, guarded_indices, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (guarded, guarded_indices);
        Ok(Self {
            output,
            indices,
            ntendon: nt,
            #[cfg(feature = "cuda-probe")]
            guarded,
            #[cfg(feature = "cuda-probe")]
            guarded_indices,
        })
    }
    pub(super) fn is_empty(self) -> bool {
        self.output.elements_per_world() == 0
    }
}

// Adapted from mujoco_warp/_src/smooth.py::_spatial_site_tendon,
// _spatial_geom_tendon, _accumulate_jac_chain and _spatial_tendon_wrap,
// and math.py::normalize_with_norm.
// Frozen revision: 71da24d956378a87a703b6e1442b13aec0c4ac29.
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
pub(super) const SPATIAL_TENDON_CUDA: &str = r#"
__device__ void put(float* p,SV v) {p[0]=v.x;p[1]=v.y;p[2]=v.z;}
__device__ SV point(const float* scratch,int i) {return sl(scratch+13*i+1);}
// Header: nt,nwrap,nnz,nbody,attached stride,site offset,ngeom,size period,size offset.
// Arrays: parent,root,dofadr,dofnum; tendonadr,num,rowadr,rownnz;
// wraptype,objid,body,sidesite; CSR columns.
// Private scratch: length, two contacts, two pre-rounding inside directions.
// Float result: lengths, CSR, compact wrap positions, private 13*nwrap scratch.
extern "C" __global__ void spatial_site(const int* m,const float* scales,
    const float* attached,float* result,unsigned nv,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[1],nnz=m[2],nb=m[3];
  const int* adr=m+9+4*nb;const int* num=adr+nt;
  const int* type=adr+4*nt;const int* site=type+nw;
  const int* side=type+3*nw;
  float* out=result+4+(unsigned long long)w*(nt+nnz+19ull*nw);
  float* points=out+nt+nnz;float* scratch=points+6*nw;
  for(int i=0;i<nt+nnz+19*nw;++i) out[i]=0.0f;
  const float* ax=attached+4+(unsigned long long)w*m[4];const float* sx=ax+m[5];
  for(int i=0;i<nw;++i) if(type[i]==3) {
    put(scratch+13*i+1,sl(sx+3*site[i]));
  }
  for(int i=0;i<nw;++i) if(type[i]==4 || type[i]==5) {
    int g=site[i];SV s=side[i]>=0 ? sl(sx+3*side[i]) : SV{1e10f,1e10f,1e10f};
    float radius=scales[m[8]+(unsigned long long)(w%m[7])*3*m[6]+3*g];
    Wrap3 r=wrap(point(scratch,i-1),point(scratch,i+1),sl(ax+3*g),ax+3*m[6]+9*g,radius,type[i],s);
    scratch[13*i]=r.len;put(scratch+13*i+1,r.a);put(scratch+13*i+4,r.b);
    put(scratch+13*i+7,r.dir0);put(scratch+13*i+10,r.dir1);
  }
  for(int t=0;t<nt;++t) for(int i=adr[t];i<adr[t]+num[t]-1;++i) {
    if(type[i]==3 && type[i+1]==3) out[t]+=norm(sub(point(scratch,i+1),point(scratch,i)))*scales[i];
    if(type[i]==4 || type[i]==5) {
      float len=scratch[13*i];
      if(len>=0) len=norm(sub(point(scratch,i),point(scratch,i-1)))+len
        +norm(sub(point(scratch,i+1),sl(scratch+13*i+4)));
      else len=norm(sub(point(scratch,i+1),point(scratch,i-1)));
      if(len!=0) out[t]+=len*scales[i];
    }
  }
  int cursor=0;
  for(int i=0;i<nw;++i) {
    if(type[i]==3) put(points+3*cursor++,point(scratch,i));
    else if(type[i]==2) put(points+3*cursor++,{0,0,0});
    else if(norm(point(scratch,i))<1e10f) {
      put(points+3*cursor++,point(scratch,i));put(points+3*cursor++,sl(scratch+13*i+4));
    }
  }
}
__device__ void chain(const int* m,const float* com,float* jac,int body,
    int row,int count,SV offset,SV vec,float scale) {
  int nb=m[3],nt=m[0],nw=m[1];
  const int* parent=m+9;const int* dofadr=parent+2*nb;const int* dofnum=dofadr+nb;
  const int* cols=m+9+4*nb+4*nt+4*nw;
  const float* cdof=com+14*nb;
  int ptr=count-1;
  while(body>0) {
    for(int k=dofnum[body]-1;k>=0;--k) {
      int dof=dofadr[body]+k;
      while(ptr>=0 && cols[row+ptr]>dof) --ptr;
      if(ptr>=0 && cols[row+ptr]==dof) {
        const float* c=cdof+6*dof;SV v=cross(sl(c),offset);
        float j=((c[3]+v.x)*vec.x+(c[4]+v.y)*vec.y+(c[5]+v.z)*vec.z)*scale;
        if(j!=0.0f) jac[row+ptr]+=j;
      }
    }
    body=parent[body];
  }
}
__device__ void directed_segment(const int* m,const float* com,float* jac,int row,int count,
    int b0,int b1,SV p0,SV p1,SV vec,float scale) {
  if(b0==b1) return;
  const int* root=m+9+m[3];
  chain(m,com,jac,b0,row,count,sub(p0,sl(com+m[3]+3*root[b0])),vec,-scale);
  chain(m,com,jac,b1,row,count,sub(p1,sl(com+m[3]+3*root[b1])),vec,scale);
}
__device__ void segment(const int* m,const float* com,float* jac,int row,int count,
    int b0,int b1,SV p0,SV p1,float scale) {
  SV dif=sub(p1,p0);
  directed_segment(m,com,jac,row,count,b0,b1,p0,p1,direction(dif,norm(dif)),scale);
}
extern "C" __global__ void spatial_moment(const int* m,const float* scales,
    const float* input,float* result,unsigned nv,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[1],nnz=m[2],nb=m[3];
  const int* adr=m+9+4*nb;const int* num=adr+nt;
  const int* row=num+nt;const int* count=row+nt;const int* type=count+nt;
  const int* body=type+2*nw;
  const float* com=input+4+(unsigned long long)w*(14ull*nb+6ull*nv);
  float* out=result+4+(unsigned long long)w*(nt+nnz+19ull*nw);
  float* jac=out+nt;const float* scratch=jac+nnz+6*nw;
  for(int j=0;j<nnz;++j) jac[j]=0.0f;
  for(int t=0;t<nt;++t) for(int i=adr[t];i<adr[t]+num[t]-1;++i) {
    if(type[i]==3 && type[i+1]==3)
      segment(m,com,jac,row[t],count[t],body[i],body[i+1],point(scratch,i),point(scratch,i+1),scales[i]);
    if(type[i]==4 || type[i]==5) {
      if(scratch[13*i]>=0) {
        SV d0=sl(scratch+13*i+7),d1=sl(scratch+13*i+10);
        if(dot(d0,d0)>0) {
          directed_segment(m,com,jac,row[t],count[t],body[i-1],body[i],point(scratch,i-1),point(scratch,i),d0,scales[i]);
          directed_segment(m,com,jac,row[t],count[t],body[i],body[i+1],sl(scratch+13*i+4),point(scratch,i+1),d1,scales[i]);
        } else {
          segment(m,com,jac,row[t],count[t],body[i-1],body[i],point(scratch,i-1),point(scratch,i),scales[i]);
          segment(m,com,jac,row[t],count[t],body[i],body[i+1],sl(scratch+13*i+4),point(scratch,i+1),scales[i]);
        }
      } else segment(m,com,jac,row[t],count[t],body[i-1],body[i+1],point(scratch,i-1),point(scratch,i+1),scales[i]);
    }
  }
}
// Mixed f32 input / i32 output retains the same eight-argument pointer ABI.
extern "C" __global__ void spatial_wrap(const int* m,const float* params,
    const float* state,int* result,unsigned nv,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[1],nnz=m[2],nb=m[3];const int* adr=m+9+4*nb;const int* num=adr+nt;
  const int* type=adr+4*nt;const int* id=type+nw;
  const float* scratch=state+4+(unsigned long long)w*(nt+nnz+19ull*nw)+nt+nnz+6*nw;
  int* out=result+4+(unsigned long long)w*(2ull*nt+2ull*nw);
  for(int i=0;i<2*nt+2*nw;++i) out[i]=0;
  int cursor=0;
  for(int t=0;t<nt;++t) {
    out[t]=cursor;
    for(int i=adr[t];i<adr[t]+num[t];++i) {
      if(type[i]==3 || type[i]==2) out[2*nt+cursor++]=type[i]==2?-2:-1;
      else if(norm(point(scratch,i))<1e10f) {out[2*nt+cursor++]=id[i];out[2*nt+cursor++]=id[i];}
    }
    out[nt+t]=cursor-out[t];
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_both_guarded_layouts_before_allocating() {
        for worlds in [0, u32::MAX as usize, usize::MAX] {
            assert!(SpatialTendonLayout::new(worlds, 1, 1, 2).is_err());
        }
        for (nt, nnz, nw) in [
            (usize::MAX, 0, 0),
            (0, usize::MAX, 0),
            (0, 0, usize::MAX),
            (0, 0, i32::MAX as usize / 6 + 1),
        ] {
            assert!(SpatialTendonLayout::new(1, nt, nnz, nw).is_err());
        }
        let l = SpatialTendonLayout::new(513, 0, 0, 0).unwrap();
        assert!(l.is_empty());
        assert_eq!(l.indices.total_elements(), 0);
        let l = SpatialTendonLayout::new(2, 2, 3, 4).unwrap();
        assert_eq!(l.output.elements_per_world(), 2 + 3 + (6 + 13) * 4);
        assert_eq!(l.output.total_elements(), 162);
        assert_eq!(l.indices.elements_per_world(), 2 * (2 + 4));
        assert_eq!(l.indices.total_elements(), 24);
    }
    #[test]
    fn checks_empty_world_indices_and_views() {
        let layout = SpatialTendonLayout::new(2, 0, 0, 0).unwrap();
        let out = SpatialTendonOutput {
            layout,
            values: vec![-131072.0; 8],
            indices: vec![-131072; 8],
            rows: Arc::new(SpatialTendonRows::default()),
        };
        assert!(out.world(1).unwrap().wrap_xpos.is_empty());
        assert!(out.world(2).is_err());
    }
}
