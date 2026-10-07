//! GPU合并原生顺序的肌腱输出。
//! 不在宿主计算肌腱物理结果。

use crate::diagnostics::InputError;
use crate::model::{BatchLayout, TendonRows};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct TendonWorld<'a> {
    pub ten_length: &'a [f32],
    pub ten_jacobian: &'a [f32],
    pub ten_wrapadr: &'a [i32],
    pub ten_wrapnum: &'a [i32],
    pub wrap_obj: &'a [i32],
    pub wrap_xpos: &'a [f32],
}

/// 原生全局顺序的只读快照。
/// 未用包裹槽位保持零值。
#[derive(Clone, Debug)]
pub struct TendonOutput {
    pub(super) layout: TendonLayout,
    pub(super) values: Vec<f32>,
    pub(super) indices: Vec<i32>,
    pub(super) rows: Arc<TendonRows>,
}
impl TendonOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn rows(&self) -> &TendonRows {
        &self.rows
    }
    pub fn world(&self, world: usize) -> Result<TendonWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let i = self.layout.indices.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let ids = &self.indices[i.start + 4..i.end + 4];
        let nt = self.rows.ntendon();
        let nnz = self.rows.nnz();
        Ok(TendonWorld {
            ten_length: &v[..nt],
            ten_jacobian: &v[nt..nt + nnz],
            wrap_xpos: &v[nt + nnz..],
            ten_wrapadr: &ids[..nt],
            ten_wrapnum: &ids[nt..2 * nt],
            wrap_obj: &ids[2 * nt..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TendonLayout {
    pub(super) output: BatchLayout,
    pub(super) indices: BatchLayout,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded_indices: usize,
}
impl TendonLayout {
    pub(super) fn new(worlds: usize, nt: usize, nnz: usize, nw: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "tendon_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "tendon_layout",
        };
        let floats = nw
            .checked_mul(6)
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

// Header: global nt,nnz,nwrap; fixed nt,nnz; spatial nt,nnz,nwrap.
// Rows: kind, local tendon, local CSR offset, global CSR offset, count.
// Both launches read the spatial integer buffer. Fixed paths add no wrap points.
#[cfg(feature = "cuda-probe")]
pub(super) const TENDON_CUDA: &str = r#"
extern "C" __global__ void tendon_values(const int* m,const int* spatial_ids,
    const float* state,float* result,unsigned kind,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nz=m[1],nw=m[2],ft=m[3],fz=m[4],st=m[5],sz=m[6],sw=m[7];
  const int* ids=spatial_ids+4+(unsigned long long)w*(2ull*st+2ull*sw);
  const float* in=state+4+(unsigned long long)w*(kind==0?ft+fz:st+sz+19ull*sw);
  float* out=result+4+(unsigned long long)w*(nt+nz+6ull*nw);
  if(kind==0) for(int i=0;i<nt+nz+6*nw;++i) out[i]=0.0f;
  int cursor=0;
  for(int t=0;t<nt;++t) {
    const int* row=m+8+5*t;int local=row[1];
    if(row[0]==(int)kind) {
      out[t]=in[local];
      for(int j=0;j<row[4];++j) out[nt+row[3]+j]=in[(kind==0?ft:st)+row[2]+j];
      if(kind==1) {
        int count=ids[st+local];
        for(int j=0;j<3*count;++j) out[nt+nz+3*cursor+j]=in[st+sz+3*ids[local]+j];
      }
    }
    if(row[0]==1) cursor+=ids[st+local];
  }
}
extern "C" __global__ void tendon_indices(const int* m,const float* unused,
    const int* state,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[2],st=m[5],sw=m[7];
  const int* in=state+4+(unsigned long long)w*(2ull*st+2ull*sw);
  int* out=result+4+(unsigned long long)w*(2ull*nt+2ull*nw);
  for(int i=0;i<2*nt+2*nw;++i) out[i]=0;
  int cursor=0;
  for(int t=0;t<nt;++t) {
    const int* row=m+8+5*t;
    // Fixed rows share the current compact cursor but add no points.
    out[t]=cursor;
    if(row[0]==0) continue;
    int local=row[1],count=in[st+local];out[nt+t]=count;
    for(int j=0;j<count;++j) out[2*nt+cursor+j]=in[2*st+in[local]+j];
    cursor+=count;
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_global_capacities_and_empty_views() {
        for w in [0, u32::MAX as usize, usize::MAX] {
            assert!(TendonLayout::new(w, 1, 1, 1).is_err());
        }
        for (nt, nz, nw) in [
            (usize::MAX, 0, 0),
            (0, usize::MAX, 0),
            (0, 0, usize::MAX),
            (i32::MAX as usize, 0, 0),
        ] {
            assert!(TendonLayout::new(1, nt, nz, nw).is_err());
        }
        let out = TendonOutput {
            layout: TendonLayout::new(2, 0, 0, 0).unwrap(),
            values: vec![0.0; 8],
            indices: vec![0; 8],
            rows: Arc::new(TendonRows::default()),
        };
        assert!(out.world(1).unwrap().ten_length.is_empty());
        assert!(out.world(2).is_err());
        assert!(out.layout.is_empty());
        let l = TendonLayout::new(513, 3, 2, 4).unwrap();
        assert_eq!(l.output.elements_per_world(), 29);
        assert_eq!(l.indices.elements_per_world(), 14);
    }
}
