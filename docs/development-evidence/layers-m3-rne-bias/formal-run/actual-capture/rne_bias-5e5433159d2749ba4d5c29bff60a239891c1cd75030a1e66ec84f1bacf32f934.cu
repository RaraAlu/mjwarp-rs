
__device__ void inertia_vector(const float* i,const float* v,float* f) {
  f[0]=i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5];
  f[1]=i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5];
  f[2]=i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4];
  f[3]=i[8]*v[1]-i[7]*v[2]+i[9]*v[3];
  f[4]=i[6]*v[2]-i[8]*v[0]+i[9]*v[4];
  f[5]=i[7]*v[0]-i[6]*v[1]+i[9]*v[5];
}
__device__ void cross_force(const float* v,const float* f,float* out) {
  out[0]=(v[1]*f[2]-v[2]*f[1])+(v[4]*f[5]-v[5]*f[4]);
  out[1]=(v[2]*f[0]-v[0]*f[2])+(v[5]*f[3]-v[3]*f[5]);
  out[2]=(v[0]*f[1]-v[1]*f[0])+(v[3]*f[4]-v[4]*f[3]);
  out[3]=v[1]*f[5]-v[2]*f[4];out[4]=v[2]*f[3]-v[0]*f[5];out[5]=v[0]*f[4]-v[1]*f[3];
}
extern "C" __global__ void rne_bias(const int* m,const float* velocities,
    const float* com,const float* velocity,float* output,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  unsigned nb=m[0],nv=m[1];const int* parent=m+5;
  const int* num=parent+nb;const int* adr=num+nb;const int* body=adr+nb;
  const float* qvel=velocities+4+(unsigned long long)w*nv;
  const float* ci=com+4+(unsigned long long)w*(14ull*nb+6ull*nv)+4ull*nb;
  const float* cdof=ci+10ull*nb;
  const float* cvel=velocity+4+(unsigned long long)w*(6ull*nb+6ull*nv);
  const float* dot=cvel+6ull*nb;
  float* acc=output+4+(unsigned long long)w*(12ull*nb+nv);
  float* frc=acc+6ull*nb;float* bias=frc+6ull*nb;
  for(int k=0;k<6;++k) { acc[k]=k<3?0.0f:-__int_as_float(m[k-1]);frc[k]=0.0f; }
  for(unsigned b=1;b<nb;++b) {
    float* a=acc+6ull*b;
    for(int k=0;k<6;++k) a[k]=acc[6ull*parent[b]+k];
    for(int d=adr[b];d<adr[b]+num[b];++d)
      for(int k=0;k<6;++k) a[k]+=dot[6ull*d+k]*qvel[d];
    float iv[6],cross[6],v[6];for(int k=0;k<6;++k) v[k]=cvel[6ull*b+k];
    inertia_vector(ci+10ull*b,v,iv);cross_force(v,iv,cross);
    inertia_vector(ci+10ull*b,a,frc+6ull*b);
    for(int k=0;k<6;++k) frc[6ull*b+k]+=cross[k];
  }
  for(int b=int(nb)-1;b>0;--b)
    for(int k=0;k<6;++k) frc[6ull*parent[b]+k]+=frc[6ull*b+k];
  for(unsigned d=0;d<nv;++d) {
    float f=0.0f;for(int k=0;k<6;++k) f+=cdof[6ull*d+k]*frc[6ull*body[d]+k];bias[d]=f;
  }
}
