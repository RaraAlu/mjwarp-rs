
extern "C" __global__ void resident_crb_init(const int* m,const float* p,const float* state,
    float* output,unsigned nb,unsigned nv,unsigned unused,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=nw) return;
  const float* ci=state+4+(unsigned long long)w*(14ull*nb+6ull*nv)+4ull*nb;
  float* crb=output+4+(unsigned long long)w*(10ull*nb+(unsigned long long)nv*nv);
  for(unsigned i=0;i<10*nb;++i) crb[i]=ci[i];
  for(unsigned i=0;i<nv*nv;++i) crb[10ull*nb+i]=0.0f;
}
extern "C" __global__ void resident_crb_accumulate(const int* m,const float* p,const float* state,
    float* output,unsigned nb,unsigned nv,unsigned level,unsigned count) {
  unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=count) return;
  unsigned b=tid%nb,w=tid/nb;const int* parent=m+2;const int* depth=m+2+nb+2ull*nv;
  if(parent[b]==0 || depth[b]!=(int)level) return;
  float* crb=output+4+(unsigned long long)w*(10ull*nb+(unsigned long long)nv*nv);
  for(int k=0;k<10;++k) atomicAdd(crb+10ull*parent[b]+k,crb[10ull*b+k]);
}
extern "C" __global__ void resident_crb_matrix(const int* m,const float* p,const float* state,
    float* output,unsigned nb,unsigned nv,unsigned unused,unsigned count) {
  unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=count) return;
  unsigned d=tid%nv,w=tid/nv;const int* body=m+2+nb;const int* parent=body+nv;
  const float* cd=state+4+(unsigned long long)w*(14ull*nb+6ull*nv)+14ull*nb;
  float* crb=output+4+(unsigned long long)w*(10ull*nb+(unsigned long long)nv*nv);
  float* mat=crb+10ull*nb;const float* i=crb+10ull*body[d];const float* v=cd+6ull*d;
  float first=fmaf(i[3],v[0],__fmul_rn(i[1],v[1]));
  float buf[6]={
    i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5],
    first+i[5]*v[2]+i[8]*v[3]-i[6]*v[5],
    i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4],
    i[8]*v[1]-i[7]*v[2]+i[9]*v[3],
    i[6]*v[2]-i[8]*v[0]+i[9]*v[4],
    i[7]*v[0]-i[6]*v[1]+i[9]*v[5]};
  mat[(unsigned long long)d*nv+d]=p[d];
  for(int j=(int)d;j>=0;j=parent[j]) {
    float value=fmaf(cd[6ull*j],buf[0],0.0f);
    for(int k=1;k<6;++k) value=fmaf(cd[6ull*j+k],buf[k],value);
    float* cell=mat+(unsigned long long)d*nv+j;
    atomicAdd(cell,value);
    if(j!=(int)d) mat[(unsigned long long)j*nv+d]=*cell;
  }
}
