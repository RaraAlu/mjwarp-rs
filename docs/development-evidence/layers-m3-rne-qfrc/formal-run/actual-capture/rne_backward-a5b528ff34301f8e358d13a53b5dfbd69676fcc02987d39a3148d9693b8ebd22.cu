// Adapted from frozen MJWarp smooth.py and math.py, revision 71da24d.
// Copyright 2025 The Newton Developers. Apache-2.0.
// See licenses/Apache-2.0.txt for the license and warranty disclaimer.
__device__ void rne_inert_vec(const float* i,const float* v,float* f) {
  f[0]=i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5];
  f[1]=i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5];
  f[2]=i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4];
  f[3]=i[8]*v[1]-i[7]*v[2]+i[9]*v[3];
  f[4]=i[6]*v[2]-i[8]*v[0]+i[9]*v[4];
  f[5]=i[7]*v[0]-i[6]*v[1]+i[9]*v[5];
}
__device__ void rne_cross_force(const float* v,const float* f,float* out) {
  out[0]=(v[1]*f[2]-v[2]*f[1])+(v[4]*f[5]-v[5]*f[4]);
  out[1]=(v[2]*f[0]-v[0]*f[2])+(v[5]*f[3]-v[3]*f[5]);
  out[2]=(v[0]*f[1]-v[1]*f[0])+(v[3]*f[4]-v[4]*f[3]);
  out[3]=v[1]*f[5]-v[2]*f[4];
  out[4]=v[2]*f[3]-v[0]*f[5];
  out[5]=v[0]*f[4]-v[1]*f[3];
}
__device__ void rne_inert_acc(const float* i,const float* a,float* f) {
  rne_inert_vec(i,a,f);
  // Frozen _cfrc rounds i3*a0, then fuses i1*a1. The cvel call differs.
  float first=fmaf(i[1],a[1],__fmul_rn(i[3],a[0]));
  f[1]=first+i[5]*a[2]+i[8]*a[3]-i[6]*a[5];
}
extern "C" __global__ void rne_cacc(const int* m,const float* v,const float* qa,
    const float* com,const float* velocity,float* out,unsigned flag,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  unsigned nb=m[0],nv=m[1];const int* parent=m+5;const int* num=parent+nb;const int* adr=num+nb;
  const float* qvel=v+4+(unsigned long long)w*nv;
  const float* qacc=qa+4+(unsigned long long)w*nv;
  const float* cdof=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+14ull*nb;
  const float* dot=velocity+4+(unsigned long long)w*(6ull*nb+6ull*nv)+6ull*nb;
  float* acc=out+4+(unsigned long long)w*(12ull*nb+nv);
  for(int k=0;k<6;++k) acc[k]=k<3?0.0f:-__int_as_float(m[k-1]);
  for(unsigned b=1;b<nb;++b) {
    float a[6];for(int k=0;k<6;++k) a[k]=acc[6ull*parent[b]+k];
    for(int d=adr[b];d<adr[b]+num[b];++d) {
      for(int k=0;k<6;++k) a[k]+=dot[6ull*d+k]*qvel[d];
      if(flag) for(int k=0;k<6;++k) a[k]+=cdof[6ull*d+k]*qacc[d];
    }
    for(int k=0;k<6;++k) acc[6ull*b+k]=a[k];
  }
}
extern "C" __global__ void rne_cfrc(const int* m,const float* com,const float* velocity,
    float* out,unsigned x,unsigned y,unsigned z,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  unsigned nb=m[0],nv=m[1];
  const float* ci=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+4ull*nb;
  const float* cv=velocity+4+(unsigned long long)w*(6ull*nb+6ull*nv);
  float* acc=out+4+(unsigned long long)w*(12ull*nb+nv);float* frc=acc+6ull*nb;
  for(int k=0;k<6;++k) frc[k]=0.0f;
  for(unsigned b=1;b<nb;++b) {
    float iv[6],cross[6];rne_inert_vec(ci+10ull*b,cv+6ull*b,iv);
    rne_cross_force(cv+6ull*b,iv,cross);rne_inert_acc(ci+10ull*b,acc+6ull*b,frc+6ull*b);
    for(int k=0;k<6;++k) frc[6ull*b+k]+=cross[k];
  }
}
extern "C" __global__ void rne_backward(const int* m,const float* unused1,const float* unused2,
    float* out,unsigned nb_arg,unsigned level,unsigned z,unsigned count) {
  unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=count) return;
  unsigned nb=m[0],nv=m[1],b=tid%nb,w=tid/nb;
  const int* parent=m+5;const int* depth=m+5+3ull*nb+nv;
  if(b==0 || depth[b]!=(int)level) return;
  float* frc=out+4+(unsigned long long)w*(12ull*nb+nv)+6ull*nb;
  for(int k=0;k<6;++k) atomicAdd(frc+6ull*parent[b]+k,frc[6ull*b+k]);
}
extern "C" __global__ void rne_qfrc(const int* m,const float* com,const float* unused,
    float* out,unsigned x,unsigned y,unsigned z,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  unsigned nb=m[0],nv=m[1];const int* body=m+5+3ull*nb;
  const float* cdof=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+14ull*nb;
  float* frc=out+4+(unsigned long long)w*(12ull*nb+nv)+6ull*nb;float* bias=frc+6ull*nb;
  for(unsigned d=0;d<nv;++d) {
    float value=0.0f;for(int k=0;k<6;++k) value+=cdof[6ull*d+k]*frc[6ull*body[d]+k];
    bias[d]=value;
  }
}
