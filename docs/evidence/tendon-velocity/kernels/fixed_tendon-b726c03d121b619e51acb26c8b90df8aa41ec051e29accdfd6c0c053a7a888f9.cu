
extern "C" __global__ void fixed_tendon(const int* meta,const float* coefficients,
    const float* state,float* result,unsigned nq,unsigned nt,unsigned nnz,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* adr=meta+1;const int* num=adr+nt;
  const int* qadr=num+nt;const int* jslot=qadr+meta[0];
  const float* qpos=state+(unsigned long long)w*nq;
  float* length=result+4+(unsigned long long)w*((unsigned long long)nt+nnz);
  float* jac=length+nt;
  for(unsigned i=0;i<nnz;++i) jac[i]=0.0f;
  for(unsigned t=0;t<nt;++t) {
    float sum=0.0f;
    for(int i=adr[t];i<adr[t]+num[t];++i) {
      // Raw qpos, not qpos-qpos0. Each row rejects duplicate joint references.
      sum+=coefficients[i]*qpos[qadr[i]];
      jac[jslot[i]]=coefficients[i];
    }
    length[t]=sum;
  }
}
