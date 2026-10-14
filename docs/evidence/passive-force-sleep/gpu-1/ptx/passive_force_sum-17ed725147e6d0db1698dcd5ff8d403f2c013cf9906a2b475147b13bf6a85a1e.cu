#define PF_NV 2

extern "C" __global__ void passive_force_clear(const int* meta,const float* p,const float* state,float* out,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=0.0f;out[4+(unsigned long long)work+tid]=0.0f;
}
extern "C" __global__ void passive_force_sum(const int* meta,const float* p,const float* parts,float* forces,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;forces[4+tid]=__fadd_rn(parts[4+tid],parts[4+1ull*worlds*nv+tid]);
}
