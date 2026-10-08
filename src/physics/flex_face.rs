//! GPU计算线性壳体面位姿。
//! 不计算弹性或碰撞。

use crate::{
    diagnostics::InputError,
    model::{BatchLayout, FlexFaceFields},
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct FlexFaceWorld<'a> {
    /// 每面九点，每点三个分量。
    pub face_xpos: &'a [f32],
    /// 每面四元数采用xyzw顺序。
    pub face_quat: &'a [f32],
}
#[derive(Clone, Debug)]
pub struct FlexFaceOutput {
    pub(super) layout: FlexFaceLayout,
    pub(super) values: Vec<f32>,
    pub(super) fields: Arc<FlexFaceFields>,
}
impl FlexFaceOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nflexface(&self) -> usize {
        self.layout.nf
    }
    pub fn fields(&self) -> &FlexFaceFields {
        &self.fields
    }
    pub fn world(&self, world: usize) -> Result<FlexFaceWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let values = &self.values[r.start + 4..r.end + 4];
        Ok(FlexFaceWorld {
            face_xpos: &values[..27 * self.layout.nf],
            face_quat: &values[27 * self.layout.nf..],
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct FlexFaceLayout {
    pub(super) output: BatchLayout,
    nf: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
}
impl FlexFaceLayout {
    pub(super) fn new(worlds: usize, nf: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize {
            return Err(InputError::InvalidDimension {
                field: "flex_face_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "flex_face_layout",
        };
        let width = nf
            .checked_mul(31)
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
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
            nf,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
    pub(super) fn is_empty(self) -> bool {
        self.nf == 0
    }
}

// Frozen smooth.py _flex_face_kinematics and support.py mat33_to_quat_polar.
// Apache-2.0, Copyright 2025 The Newton Developers.
// Header: face count, nodal result stride. Each row: normal axis, nine nodes.
// The private ABI retains all buffers and synchronizes before returning.
#[cfg(feature = "cuda-probe")]
pub(super) const FLEX_FACE_CUDA: &str = r#"
__device__ float face_dot(const float* a,const float* b) {
  return (a[0]*b[0]+a[1]*b[1])+a[2]*b[2];
}
__device__ void face_cross(float* r,const float* a,const float* b) {
  r[0]=a[1]*b[2]-a[2]*b[1];r[1]=a[2]*b[0]-a[0]*b[2];r[2]=a[0]*b[1]-a[1]*b[0];
}
__device__ void face_polar(float* q,const float F[3][3]) {
  q[0]=q[1]=q[2]=0.0f;q[3]=1.0f;
  for(int iter=0;iter<50;++iter) {
    float x=q[0],y=q[1],z=q[2],s=q[3];
    // Column-major rotation columns, matching Warp's xyzw convention.
    float R[3][3]={{1-2*(y*y+z*z),2*(x*y+s*z),2*(x*z-s*y)},
      {2*(x*y-s*z),1-2*(x*x+z*z),2*(y*z+s*x)},
      {2*(x*z+s*y),2*(y*z-s*x),1-2*(x*x+y*y)}};
    float crosses[3][3];
    for(int c=0;c<3;++c) face_cross(crosses[c],R[c],F[c]);
    float omega[3];float denom=fabsf((face_dot(R[0],F[0])+face_dot(R[1],F[1]))+face_dot(R[2],F[2]))+1.0e-10f;
    for(int k=0;k<3;++k) omega[k]=((crosses[0][k]+crosses[1][k])+crosses[2][k])/denom;
    float w=sqrtf(face_dot(omega,omega));if(w<1.0e-6f) break;
    float rot[4];for(int k=0;k<3;++k) rot[k]=(omega[k]/w)*sinf(0.5f*w);rot[3]=cosf(0.5f*w);
    float next[4];float cross[3];face_cross(cross,rot,q);
    for(int k=0;k<3;++k) next[k]=(rot[3]*q[k]+q[3]*rot[k])+cross[k];
    next[3]=rot[3]*q[3]-face_dot(rot,q);
    float n=sqrtf(((next[0]*next[0]+next[1]*next[1])+next[2]*next[2])+next[3]*next[3]);
    for(int k=0;k<4;++k) q[k]=next[k]/n;
  }
}
extern "C" __global__ void flex_faces(const int* m,const float* params,const float* nodes,
    float* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0];const float* nx=nodes+4+(unsigned long long)w*m[1];
  float* positions=result+4+(unsigned long long)w*31ull*nf;float* quats=positions+27*nf;
  for(int f=0;f<nf;++f) {
    const int* row=m+2+10*f;float t1[3]={0,0,0},t2[3]={0,0,0};
    for(int slot=0;slot<9;++slot) {
      int n=row[1+slot];float* out=positions+27*f+3*slot;
      if(n<0) {out[0]=out[1]=out[2]=0.0f;continue;}
      float g0=0.5f*(-1+2*(slot/2)),g1=0.5f*(-1+2*(slot%2));
      for(int k=0;k<3;++k) {out[k]=nx[3*n+k];t1[k]+=out[k]*g0;t2[k]+=out[k]*g1;}
    }
    float normal[3];face_cross(normal,t1,t2);float F[3][3];
    for(int k=0;k<3;++k) {
      if(row[0]==0) {F[0][k]=normal[k];F[1][k]=t1[k];F[2][k]=t2[k];}
      else if(row[0]==1) {F[0][k]=t2[k];F[1][k]=normal[k];F[2][k]=t1[k];}
      else {F[0][k]=t1[k];F[1][k]=t2[k];F[2][k]=normal[k];}
    }
    face_polar(quats+4*f,F);
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_face_layouts_empty_views_and_world_bounds() {
        for w in [0, u32::MAX as usize, usize::MAX] {
            assert!(FlexFaceLayout::new(w, 1).is_err());
        }
        for n in [usize::MAX, i32::MAX as usize] {
            assert!(FlexFaceLayout::new(1, n).is_err());
        }
        assert_eq!(
            FlexFaceLayout::new(513, 16)
                .unwrap()
                .output
                .elements_per_world(),
            496
        );
        let out = FlexFaceOutput {
            layout: FlexFaceLayout::new(2, 0).unwrap(),
            values: vec![0.0; 8],
            fields: Arc::new(FlexFaceFields::default()),
        };
        assert!(out.world(1).unwrap().face_xpos.is_empty());
        assert!(out.world(1).unwrap().face_quat.is_empty());
        assert!(out.world(2).is_err());
        assert!(out.layout.is_empty());
    }
}
