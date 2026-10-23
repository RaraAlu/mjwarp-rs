
extern "C" __global__ void sparse_solve_validate(const int* meta,const float* rhs,
 const float* factor,float* status,unsigned n,unsigned nc,unsigned nr,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long fs=1ull*nc+n,rs=1ull*n*nr;
 bool f=true,r=true;
 for(unsigned j=0;j<4;++j) {
   f=f && factor[j]==-131072.0f && factor[4+fs*worlds+j]==-131072.0f;
   r=r && rhs[j]==-131072.0f && rhs[4+rs*worlds+j]==-131072.0f;
 }
 const float* L=factor+4+fs*w;const float* b=rhs+4+rs*w;
 for(unsigned long long j=0;j<fs;++j)f=f && isfinite(L[j]);
 const int* diag=meta+1+n;
 for(unsigned i=0;i<n;++i)f=f && L[diag[i]]>0.0f && L[nc+i]>0.0f;
 for(unsigned long long j=0;j<rs;++j)r=r && isfinite(b[j]);
 status[4+w]=(f?0.0f:1.0f)+(r?0.0f:2.0f);
}
extern "C" __global__ void sparse_solve_fused(const int* meta,const float* rhs,
 const float* factor,float* output,unsigned n,unsigned nc,unsigned nr,unsigned worlds) {
 unsigned group=blockIdx.x,w=group/nr,tid=threadIdx.x;
 const int* flags=meta+1;const int* offsets=flags+2*n;const int* updates=offsets+meta[0]+1;
 const float* L=factor+4+(unsigned long long)w*(1ull*nc+n);const float* D=L+nc;
 const float* b=rhs+4+(unsigned long long)group*n;float* x=output+4+(unsigned long long)group*n;
 for(unsigned i=tid;i<n;i+=128) x[i]=flags[i]==-1?b[i]:__fmul_rn(D[i],b[i]);
 __syncthreads();
 for(int level=meta[0]-1;level>=0;--level) {
   for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
     int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];
     float value=__fmul_rn(L[a],x[k]),ignored;
     asm volatile("atom.global.add.f32 %0, [%1], %2;" : "=f"(ignored) : "l"(x+i),"f"(-value) : "memory");
   }
   __syncthreads();
 }
 for(unsigned i=tid;i<n;i+=128)if(flags[i]==-1)x[i]=__fmul_rn(x[i],D[i]);
 __syncthreads();
 for(int level=0;level<meta[0];++level) {
   for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
     int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];
     float value=__fmul_rn(L[a],x[i]),ignored;
     asm volatile("atom.global.add.f32 %0, [%1], %2;" : "=f"(ignored) : "l"(x+k),"f"(-value) : "memory");
   }
   __syncthreads();
 }
}
