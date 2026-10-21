//! 混合求解冻结CUDA源码。

// Frozen smooth.py3314–3553; Apache-2.0, Copyright2025 The Newton Developers.
// Private ABI adds RHS grouping, not a different scalar or sparse algorithm.
#[cfg(feature = "cuda-probe")]
pub(super) const CUDA: &str = r#"
__device__ float mixed_schur(float value,float a,float b) {
 float result;asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
 : "=f"(result):"f"(value),"f"(a),"f"(b));return result;
}
template<int N> __device__ void mixed_small_solve(const float* F,const float* b,float* x,int start,int address) {
 for(int i=0;i<N;++i) {
  float value=b[start+i];for(int k=0;k<i;++k)value=mixed_schur(value,F[address+k*N+i],x[start+k]);
  x[start+i]=__fdiv_rn(value,F[address+i*N+i]);
 }
 for(int i=N-1;i>=0;--i) {
  float value=x[start+i];for(int k=i+1;k<N;++k)value=mixed_schur(value,F[address+i*N+k],x[start+k]);
  x[start+i]=__fdiv_rn(value,F[address+i*N+i]);
 }
}
extern "C" __global__ void mixed_solve_validate(const int* meta,const float* rhs,const float* factor,
 float* status,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long rs=1ull*n*nr;bool f=true,r=true;
 for(unsigned j=0;j<4;++j) {
  f=f && factor[j]==-131072.0f && factor[4+1ull*stride*worlds+j]==-131072.0f;
  r=r && rhs[j]==-131072.0f && rhs[4+rs*worlds+j]==-131072.0f;
 }
 const float* F=factor+4+1ull*w*stride;const int* diag=meta+4+n;
 for(unsigned j=0;j<stride;++j)f=f && isfinite(F[j]);
 for(unsigned i=0;i<n;++i)f=f && F[diag[i]]>0.0f && F[meta[1]+meta[2]+i]>0.0f;
 const float* b=rhs+4+rs*w;for(unsigned long long j=0;j<rs;++j)r=r && isfinite(b[j]);
 status[4+w]=(f?0.0f:1.0f)+(r?0.0f:2.0f);
}
extern "C" __global__ void mixed_solve_fused(const int* meta,const float* rhs,const float* factor,
 float* output,unsigned n,unsigned stride,unsigned nr,unsigned worlds) {
 unsigned group=blockIdx.x,w=group/nr,tid=threadIdx.x;
 const int* flags=meta+4;const int* offsets=flags+2*n;const int* updates=offsets+meta[0]+1;
 const int* small=updates+3*offsets[meta[0]];
 const float* F=factor+4+1ull*w*stride;const float* L=F+meta[1];const float* D=L+meta[2];
 const float* b=rhs+4+1ull*group*n;float* x=output+4+1ull*group*n;
 for(unsigned i=tid;i<n;i+=128) {
  if(flags[i]==-1)x[i]=b[i];else if(flags[i]==-2)x[i]=__fmul_rn(D[i],b[i]);
 }
 for(unsigned block=tid;block<(unsigned)meta[3];block+=128) {
  int start=small[3*block],size=small[3*block+1],a=small[3*block+2];
  switch(size) {
   case 2:mixed_small_solve<2>(F,b,x,start,a);break;
   case 3:mixed_small_solve<3>(F,b,x,start,a);break;
   case 4:mixed_small_solve<4>(F,b,x,start,a);break;
   case 5:mixed_small_solve<5>(F,b,x,start,a);break;
   case 6:mixed_small_solve<6>(F,b,x,start,a);break;
  }
 }
 __syncthreads();
 for(int level=meta[0]-1;level>=0;--level) {
  for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
   int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];float value=__fmul_rn(L[a],x[k]),ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;":"=f"(ignored):"l"(x+i),"f"(-value):"memory");
  }
  __syncthreads();
 }
 for(unsigned i=tid;i<n;i+=128)if(flags[i]==-1)x[i]=__fmul_rn(x[i],D[i]);
 __syncthreads();
 for(int level=0;level<meta[0];++level) {
  for(int u=offsets[level]+tid;u<offsets[level+1];u+=128) {
   int i=updates[3*u],k=updates[3*u+1],a=updates[3*u+2];float value=__fmul_rn(L[a],x[i]),ignored;
   asm volatile("atom.global.add.f32 %0, [%1], %2;":"=f"(ignored):"l"(x+k),"f"(-value):"memory");
  }
  __syncthreads();
 }
}
"#;
