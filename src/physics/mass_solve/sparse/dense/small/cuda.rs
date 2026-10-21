//! Small冻结CUDA源码。

#[cfg(feature = "cuda-probe")]
pub(super) const CUDA: &str = r#"
__device__ float small_schur(float value,float a,float b) {
 float result;
 asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
     : "=f"(result) : "f"(value),"f"(a),"f"(b));
 return result;
}
extern "C" __global__ void small_prefix_validate(const int* meta,const float* matrix,
 const float* unused,float* status,unsigned prefix,unsigned a,unsigned b,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 bool valid=true;unsigned long long total=(unsigned long long)prefix*worlds;
 for(unsigned j=0;j<4;++j)valid=valid && matrix[j]==-131072.0f && matrix[4+total+j]==-131072.0f;
 const float* m=matrix+4+(unsigned long long)w*prefix;
 for(unsigned j=0;j<prefix;++j)valid=valid && isfinite(m[j]);
 status[4+w]=valid?1.0f:0.0f;
}
template<int N> __device__ void small_factor(const int* meta,const float* matrix,float* output,
 unsigned prefix,unsigned first,unsigned count,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/count,block=first+tid%count;int address=meta[3*block+1];
 const float* m=matrix+4+(unsigned long long)w*prefix+address;
 float* U=output+4+(unsigned long long)w*prefix+address;
 for(int j=0;j<N*N;++j)U[j]=0.0f;
 for(int i=0;i<N;++i) {
   float pivot=m[i*N+i];
   for(int k=0;k<i;++k)pivot=small_schur(pivot,U[k*N+i],U[k*N+i]);
   if(!(pivot>0.0f) || !isfinite(pivot)){U[i*N+i]=pivot;return;}
   float diagonal,reciprocal;
   asm("sqrt.rn.f32 %0,%1;":"=f"(diagonal):"f"(pivot));
   U[i*N+i]=diagonal;
   asm("rcp.rn.f32 %0,%1;":"=f"(reciprocal):"f"(diagonal));
   for(int j=i+1;j<N;++j) {
     float value=m[i*N+j];
     for(int k=0;k<i;++k)value=small_schur(value,U[k*N+i],U[k*N+j]);
     float result;asm("mul.f32 %0,%1,%2;":"=f"(result):"f"(value),"f"(reciprocal));
     U[i*N+j]=result;
   }
 }
}
#define SMALL_ENTRY(N) extern "C" __global__ void small_prefix_factor_##N(const int* meta,const float* matrix,\
 const float* unused,float* output,unsigned prefix,unsigned first,unsigned count,unsigned work) {\
 small_factor<N>(meta,matrix,output,prefix,first,count,work);}
SMALL_ENTRY(2)
SMALL_ENTRY(3)
SMALL_ENTRY(4)
SMALL_ENTRY(5)
SMALL_ENTRY(6)
"#;
