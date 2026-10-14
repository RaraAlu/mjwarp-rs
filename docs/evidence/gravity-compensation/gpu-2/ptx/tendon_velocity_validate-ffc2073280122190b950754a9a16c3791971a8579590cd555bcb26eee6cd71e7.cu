#define TV_NV 11
#define TV_NT 1
#define TV_NNZ 1
#define TV_STRIDE 8

extern "C" __global__ void tendon_velocity_validate(const int* meta,const float* jac,const float* v,float* status,unsigned a,unsigned b,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 for(unsigned k=0;k<4;++k)ok=ok&&meta[k]==-131072&&meta[4+2ull*TV_NT+TV_NNZ+k]==-131072&&jac[k]==-131072.0f&&jac[4+1ull*worlds*TV_STRIDE+k]==-131072.0f&&v[k]==-131072.0f&&v[4+1ull*worlds*TV_NV+k]==-131072.0f;
 for(unsigned t=0;t<TV_NT;++t){int start=meta[4+t],n=meta[4+TV_NT+t];if(start<0||n<0||start>TV_NNZ||n>TV_NNZ-start){ok=false;continue;}for(int i=0;i<n;++i){int c=meta[4+2ull*TV_NT+start+i];if(c<0||c>=TV_NV)ok=false;}}
 for(unsigned i=0;i<TV_NNZ;++i)ok=ok&&isfinite(jac[4+1ull*w*TV_STRIDE+TV_NT+i]);
 for(unsigned i=0;i<TV_NV;++i)ok=ok&&isfinite(v[4+1ull*w*TV_NV+i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void tendon_velocity_compute(const int* meta,const float* jac,const float* v,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/TV_NT,t=tid%TV_NT;int start=meta[4+t],n=meta[4+TV_NT+t];float velocity=0.0f;
 for(int i=0;i<n;++i){int sparse=start+i;float J=jac[4+1ull*w*TV_STRIDE+TV_NT+sparse];if(J!=0.0f){int col=meta[4+2ull*TV_NT+sparse];velocity+=J*v[4+1ull*w*TV_NV+col];}}
 out[4+tid]=velocity;
}
