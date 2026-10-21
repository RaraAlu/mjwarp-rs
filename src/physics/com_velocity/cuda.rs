//! 空间速度内核源。

// Frozen smooth.py _comvel_branch and math.motion_cross (Apache-2.0).
// Copyright 2025 The Newton Developers.
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
pub(super) const COM_VELOCITY_CUDA: &str = r#"
__device__ void motion_cross(const float* a,const float* b,float* out) {
  out[0]=a[1]*b[2]-a[2]*b[1];out[1]=a[2]*b[0]-a[0]*b[2];out[2]=a[0]*b[1]-a[1]*b[0];
  out[3]=(a[1]*b[5]-a[2]*b[4])+(a[4]*b[2]-a[5]*b[1]);
  out[4]=(a[2]*b[3]-a[0]*b[5])+(a[5]*b[0]-a[3]*b[2]);
  out[5]=(a[0]*b[4]-a[1]*b[3])+(a[3]*b[1]-a[4]*b[0]);
}
extern "C" __global__ void com_velocity(const int* m,const float* velocities,
    const float* com,float* output,unsigned nb,unsigned nj,unsigned nv,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  const int* parent=m;const int* num=parent+nb;const int* adr=num+nb;
  const int* type=adr+nb;const int* dofadr=type+nj;
  const float* qvel=velocities+4+(unsigned long long)w*nv;
  const float* cdof=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+14ull*nb;
  float* cvel=output+4+(unsigned long long)w*(6ull*nb+6ull*nv);float* dot=cvel+6ull*nb;
  for(int k=0;k<6;++k) cvel[k]=0.0f;
  for(unsigned b=1;b<nb;++b) {
    float v[6];for(int k=0;k<6;++k) v[k]=cvel[6ull*parent[b]+k];
    for(int j=adr[b];j<adr[b]+num[b];++j) {
      int d=dofadr[j],t=type[j];
      if(t==0) {
        for(int i=0;i<3;++i) for(int k=0;k<6;++k) {
          v[k]+=cdof[6ull*(d+i)+k]*qvel[d+i];dot[6ull*(d+i)+k]=0.0f;
        }
        for(int i=3;i<6;++i) motion_cross(v,cdof+6ull*(d+i),dot+6ull*(d+i));
        for(int i=3;i<6;++i) for(int k=0;k<6;++k) v[k]+=cdof[6ull*(d+i)+k]*qvel[d+i];
      } else if(t==1) {
        for(int i=0;i<3;++i) motion_cross(v,cdof+6ull*(d+i),dot+6ull*(d+i));
        for(int i=0;i<3;++i) for(int k=0;k<6;++k) v[k]+=cdof[6ull*(d+i)+k]*qvel[d+i];
      } else {
        motion_cross(v,cdof+6ull*d,dot+6ull*d);
        for(int k=0;k<6;++k) v[k]+=cdof[6ull*d+k]*qvel[d];
      }
    }
    for(int k=0;k<6;++k) cvel[6ull*b+k]=v[k];
  }
}
"#;
