//! GPU计算柔体节点与顶点位置。
//! 不计算边、弹性或碰撞。

use crate::{diagnostics::InputError, model::BatchLayout};

#[derive(Clone, Copy, Debug)]
pub struct FlexPositionWorld<'a> {
    pub flexnode_xpos: &'a [f32],
    pub flexvert_xpos: &'a [f32],
    /// 位置更新会清除全部标志。
    /// 本辅助不计算Hessian矩阵。
    pub flex_hessian_valid: &'a [bool],
}
#[derive(Clone, Debug)]
pub struct FlexPositionOutput {
    pub(super) layout: FlexPositionLayout,
    pub(super) values: Vec<f32>,
    pub(super) validity: Vec<bool>,
}
impl FlexPositionOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nflexnode(&self) -> usize {
        self.layout.nn
    }
    pub fn nflex(&self) -> usize {
        self.layout.validity.elements_per_world()
    }
    pub fn nflexvert(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<FlexPositionWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        Ok(FlexPositionWorld {
            flexnode_xpos: &v[..3 * self.layout.nn],
            flexvert_xpos: &v[3 * self.layout.nn..],
            flex_hessian_valid: &self.validity[self.layout.validity.world_elements(world)?],
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct FlexPositionLayout {
    pub(super) output: BatchLayout,
    pub(super) validity: BatchLayout,
    nn: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded_validity: usize,
}
impl FlexPositionLayout {
    pub(super) fn new(worlds: usize, nf: usize, nn: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "flex_position_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "flex_position_layout",
        };
        let width = nn
            .checked_add(nv)
            .and_then(|n| n.checked_mul(3))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        if nf > i32::MAX as usize {
            return Err(overflow());
        }
        // Use checked i32 device slots, not Rust bool device representations.
        let validity = BatchLayout::new(worlds, nf, 4)?;
        let guarded_validity = validity
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded_validity, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (guarded, guarded_validity);
        Ok(Self {
            output,
            validity,
            nn,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
            #[cfg(feature = "cuda-probe")]
            guarded_validity,
        })
    }
    pub(super) fn is_empty(self) -> bool {
        self.output.elements_per_world() == 0 && self.validity.elements_per_world() == 0
    }
}

// Frozen smooth.py: _flex_nodes and _flex_vertices (Apache-2.0).
// Header: nf, nn, nv, nb, rigid stride. Rows have nine integers.
// One thread owns a world: node writes precede vertex interpolation.
#[cfg(feature = "cuda-probe")]
pub(super) const FLEX_POSITION_CUDA: &str = r#"
// Frozen smooth.flex clears every (world, flex) before any position work.
// Private guarded i32 slots encode false=0 and true=1; this is not Warp's bool ABI.
extern "C" __global__ void flex_invalidate(const int* m,const float* params,
    const float* rigid,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0];int* flags=result+4+(unsigned long long)w*nf;
  for(int f=0;f<nf;++f) flags[f]=0;
}
__device__ void flex_transform(float* out,const float* local,const float* xp,
    const float* xm,int body,bool centered) {
  const float* p=xp+3*body;const float* r=xm+9*body;
  for(int k=0;k<3;++k) out[k]=centered?p[k]:
    ((r[3*k]*local[0]+r[3*k+1]*local[1])+r[3*k+2]*local[2])+p[k];
}
extern "C" __global__ void flex_positions(const int* m,const float* params,
    const float* rigid,float* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0],nn=m[1],nv=m[2],nb=m[3];
  const float* xp=rigid+4+(unsigned long long)w*m[4];const float* xm=xp+7*nb;
  const float* verts=params;const float* coords=verts+3*nv;const float* nodes=coords+3*nv;
  const int* nodebody=m+5+9*nf;const int* vertbody=nodebody+nn;
  float* nx=result+4+(unsigned long long)w*(3ull*nn+3ull*nv);float* vx=nx+3*nn;
  for(int f=0;f<nf;++f) {
    const int* row=m+5+9*f;
    for(int j=0;j<row[5];++j) {
      int i=row[4]+j;const float* local=nodes+3*i;
      bool zero=local[0]==0.0f&&local[1]==0.0f&&local[2]==0.0f;
      flex_transform(nx+3*i,local,xp,xm,nodebody[i],row[8]!=0||zero);
    }
  }
  for(int f=0;f<nf;++f) {
    const int* row=m+5+9*f;
    for(int j=0;j<row[7];++j) {
      int v=row[6]+j;
      if(row[0]==0) { flex_transform(vx+3*v,verts+3*v,xp,xm,vertbody[v],row[8]!=0);continue; }
      int cell[3];float local[3];
      for(int k=0;k<3;++k) {
        float coord=coords[3*v+k]*(float)row[1+k];
        cell[k]=max(0,min((int)coord,row[1+k]-1));
        local[k]=fminf(1.0f,fmaxf(0.0f,coord-(float)cell[k]));
      }
      float p[3]={0,0,0};int ny=row[2]+1,nz=row[3]+1;
      for(int i=0;i<2;++i) for(int j=0;j<2;++j) for(int k=0;k<2;++k) {
        float weight=(i?local[0]:1-local[0])*(j?local[1]:1-local[1])*(k?local[2]:1-local[2]);
        int n=row[4]+(cell[0]+i)*ny*nz+(cell[1]+j)*nz+cell[2]+k;
        for(int d=0;d<3;++d) p[d]+=weight*nx[3*n+d];
      }
      for(int d=0;d<3;++d) vx[3*v+d]=p[d];
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_flex_position_layouts_and_views() {
        for w in [0, u32::MAX as usize, usize::MAX] {
            assert!(FlexPositionLayout::new(w, 1, 1, 1).is_err());
        }
        for (n, v) in [(usize::MAX, 0), (0, usize::MAX), (i32::MAX as usize, 1)] {
            assert!(FlexPositionLayout::new(1, 1, n, v).is_err());
        }
        let l = FlexPositionLayout::new(513, 4, 8, 3).unwrap();
        assert_eq!(l.output.elements_per_world(), 33);
        assert_eq!(l.validity.total_elements(), 2052);
        let out = FlexPositionOutput {
            layout: FlexPositionLayout::new(2, 0, 0, 0).unwrap(),
            values: vec![0.0; 8],
            validity: vec![],
        };
        assert!(out.world(1).unwrap().flexvert_xpos.is_empty());
        assert!(out.world(2).is_err());
        assert!(out.layout.is_empty());
    }
    #[test]
    fn checks_hessian_validity_layout_and_bool_views() {
        for nf in [i32::MAX as usize + 1, usize::MAX] {
            assert!(FlexPositionLayout::new(1, nf, 0, 0).is_err());
        }
        assert!(
            FlexPositionLayout::new((u32::MAX - 255) as usize, i32::MAX as usize, 0, 0).is_err()
        );
        let out = FlexPositionOutput {
            layout: FlexPositionLayout::new(2, 3, 0, 0).unwrap(),
            values: vec![0.0; 8],
            validity: vec![false, true, false, true, false, true],
        };
        assert_eq!(out.nflex(), 3);
        assert_eq!(
            out.world(0).unwrap().flex_hessian_valid,
            &[false, true, false]
        );
        assert_eq!(
            out.world(1).unwrap().flex_hessian_valid,
            &[true, false, true]
        );
        assert!(out.world(2).is_err());
        assert!(!out.layout.is_empty());
    }
}
