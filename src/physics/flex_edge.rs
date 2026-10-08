//! GPU计算柔体边运动学。
//! 不计算面、弹性与碰撞。
use crate::model::FlexEdgeFields;
use crate::{diagnostics::InputError, model::BatchLayout};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct FlexEdgeWorld<'a> {
    pub flexedge_length: &'a [f32],
    pub flexedge_velocity: &'a [f32],
    pub flexedge_jacobian: &'a [f32],
}
#[derive(Clone, Debug)]
pub struct FlexEdgeOutput {
    pub(super) layout: FlexEdgeLayout,
    pub(super) values: Vec<f32>,
    pub(super) fields: Arc<FlexEdgeFields>,
}
impl FlexEdgeOutput {
    pub fn fields(&self) -> &FlexEdgeFields {
        &self.fields
    }
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nflexedge(&self) -> usize {
        self.layout.ne
    }
    pub fn nnz(&self) -> usize {
        self.layout.output.elements_per_world() - 2 * self.layout.ne
    }
    pub fn world(&self, world: usize) -> Result<FlexEdgeWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let ne = self.layout.ne;
        Ok(FlexEdgeWorld {
            flexedge_length: &v[..ne],
            flexedge_velocity: &v[ne..2 * ne],
            flexedge_jacobian: &v[2 * ne..],
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct FlexEdgeLayout {
    pub(super) output: BatchLayout,
    pub(super) qvel: BatchLayout,
    ne: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded_qvel: usize,
}
impl FlexEdgeLayout {
    pub(super) fn new(worlds: usize, ne: usize, nnz: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "flex_edge_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "flex_edge_layout",
        };
        let width = ne
            .checked_mul(2)
            .and_then(|n| n.checked_add(nnz))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        if nv > i32::MAX as usize {
            return Err(overflow());
        }
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
            ne,
            #[cfg(feature = "cuda-probe")]
            guarded,
            #[cfg(feature = "cuda-probe")]
            guarded_qvel,
        })
    }
    pub(super) fn is_empty(self) -> bool {
        self.output.elements_per_world() == 0
    }
}

// Frozen smooth.py _flex_edges and math.normalize_with_norm (Apache-2.0).
// One thread owns a world and writes each disjoint checked sparse row.
#[cfg(feature = "cuda-probe")]
pub(super) const FLEX_EDGE_CUDA: &str = r#"
__device__ bool edge_ancestor(int b,int owner,const int* parent) {
  while(b>0) {if(b==owner) return true;b=parent[b];}return false;
}
extern "C" __global__ void flex_edges(const int* m,const float* qvel,
    const float* com,const float* flex,float* out,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nb=m[0],nv=m[1],ne=m[2],nnz=m[3],nvert=m[6];
  const int* rows=m+7;const int* cols=rows+4*ne;const int* vb=cols+nnz;
  const int* parent=vb+nvert;const int* root=parent+nb;const int* db=root+nb;
  const float* cp=com+4+(unsigned long long)w*m[4];const float* sc=cp+nb;
  const float* cdof=cp+14*nb;const float* vx=flex+4+(unsigned long long)w*m[5]+m[5]-3*nvert;
  const float* qv=qvel+4+(unsigned long long)w*nv;
  float* len=out+4+(unsigned long long)w*(2ull*ne+nnz);float* vel=len+ne;float* jac=vel+ne;
  for(int i=0;i<nnz;++i) jac[i]=0.0f;
  for(int e=0;e<ne;++e) {
    const int* row=rows+4*e;int v1=row[0],v2=row[1],adr=row[2],num=row[3];
    float dir[3];for(int k=0;k<3;++k) dir[k]=vx[3*v2+k]-vx[3*v1+k];
    float norm=sqrtf((dir[0]*dir[0]+dir[1]*dir[1])+dir[2]*dir[2]);len[e]=norm;
    if(norm!=0.0f) for(int k=0;k<3;++k) dir[k]/=norm;
    vel[e]=0.0f;int b1=vb[v1],b2=vb[v2];if(b1<0||b2<0||num==0) continue;
    float off[2][3];int bodies[2]={b1,b2};
    for(int s=0;s<2;++s) for(int k=0;k<3;++k)
      off[s][k]=vx[3*(s?v2:v1)+k]-sc[3*root[bodies[s]]+k];
    for(int i=0;i<num;++i) {
      int d=cols[adr+i];const float* cd=cdof+6*d;float j=0.0f;
      for(int s=0;s<2;++s) if(edge_ancestor(bodies[s],db[d],parent)) {
        const float* o=off[s];float p[3]={cd[3]+(cd[1]*o[2]-cd[2]*o[1]),
          cd[4]+(cd[2]*o[0]-cd[0]*o[2]),cd[5]+(cd[0]*o[1]-cd[1]*o[0])};
        float dot=(p[0]*dir[0]+p[1]*dir[1])+p[2]*dir[2];j+=s?dot:-dot;
      }
      jac[adr+i]=j;vel[e]+=j*qv[d];
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_edge_layout_overflow_empty_views_and_world_bounds() {
        for w in [0, u32::MAX as usize, usize::MAX] {
            assert!(FlexEdgeLayout::new(w, 1, 1, 1).is_err());
        }
        for (e, j, v) in [(usize::MAX, 0, 0), (0, usize::MAX, 0), (0, 0, usize::MAX)] {
            assert!(FlexEdgeLayout::new(1, e, j, v).is_err());
        }
        let l = FlexEdgeLayout::new(513, 2, 3, 4).unwrap();
        assert_eq!(l.output.elements_per_world(), 7);
        let l = FlexEdgeLayout::new(2, 0, 0, 0).unwrap();
        assert!(l.is_empty());
        let o = FlexEdgeOutput {
            layout: l,
            values: vec![0.0; 8],
            fields: Arc::new(FlexEdgeFields::default()),
        };
        assert!(o.world(1).unwrap().flexedge_length.is_empty());
        assert!(o.world(2).is_err());
    }
}
