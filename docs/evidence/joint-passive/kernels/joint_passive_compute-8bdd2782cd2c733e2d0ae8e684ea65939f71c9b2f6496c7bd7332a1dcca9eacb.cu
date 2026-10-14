#define JP_NQ 13
#define JP_NV 11
#define JP_NJ 4
#define JP_PL 397
#define JP_S 0
#define JP_D 1
__device__ __constant__ unsigned JP_BATCH[5]={2,3,5,7,11};
__device__ __constant__ unsigned JP_OFF[5]={0,26,38,78,155};

__device__ float jp_poly(float linear,float a,float b,float x){float result=linear;result=result+a*x;result=result+(b*x)*x;return result;}
extern "C" __global__ void joint_passive_validate(const int* meta,const float* p,const float* state,float* status,unsigned nq,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool valid=nq==JP_NQ&&nv==JP_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+3ull*JP_NV+j]==-131072&&p[j]==-131072.0f&&p[4+JP_PL+j]==-131072.0f&&state[j]==-131072.0f&&state[4+1ull*worlds*(JP_NQ+JP_NV)+j]==-131072.0f;
 for(unsigned j=0;j<JP_PL;++j)valid=valid&&isfinite(p[4+j]);
 for(unsigned j=0;j<JP_NQ;++j)valid=valid&&isfinite(state[4+1ull*w*JP_NQ+j]);for(unsigned j=0;j<JP_NV;++j)valid=valid&&isfinite(state[4+1ull*worlds*JP_NQ+1ull*w*JP_NV+j]);
 for(unsigned i=0;i<JP_NV;++i){int kind=meta[4+i],q=meta[4+JP_NV+i],j=meta[4+2ull*JP_NV+i];if(kind<0||kind>3||q<0||q>=JP_NQ||j<0||j>=JP_NJ){valid=false;continue;}
  if(!JP_S&&kind<2){unsigned si=4+JP_OFF[1]+(w%JP_BATCH[1])*JP_NJ+j,pi=4+JP_OFF[2]+2*((w%JP_BATCH[2])*JP_NJ+j);if(p[si]!=0.0f||p[pi]!=0.0f||p[pi+1]!=0.0f)valid=false;}}
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void joint_passive_compute(const int* meta,const float* p,const float* state,float* out,unsigned nq,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/nv,i=tid%nv;int kind=meta[4+i],q=meta[4+JP_NV+i],j=meta[4+2ull*JP_NV+i];float spring=0.0f,damper=0.0f;
 if(!JP_D){unsigned di=4+JP_OFF[3]+(w%JP_BATCH[3])*JP_NV+i,pi=4+JP_OFF[4]+2*((w%JP_BATCH[4])*JP_NV+i);float d=p[di],a=p[pi],b=p[pi+1];if(d!=0.0f||a!=0.0f||b!=0.0f){float v=state[4+1ull*worlds*JP_NQ+1ull*w*JP_NV+i];damper=-v*jp_poly(d,a,b,fabsf(v));}}
 if(!JP_S&&kind>=2){unsigned si=4+JP_OFF[1]+(w%JP_BATCH[1])*JP_NJ+j,pi=4+JP_OFF[2]+2*((w%JP_BATCH[2])*JP_NJ+j);float s=p[si],a=p[pi],b=p[pi+1];if(s!=0.0f||a!=0.0f||b!=0.0f){float dif=state[4+1ull*w*JP_NQ+q]-p[4+JP_OFF[0]+(w%JP_BATCH[0])*JP_NQ+q];spring=-dif*jp_poly(s,a,b,dif);}}
 out[4+tid]=spring;out[4+1ull*worlds*nv+tid]=damper;
}
extern "C" __global__ void joint_passive_sum(const int* meta,const float* parts,const float* unused,float* forces,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;forces[4+tid]=__fadd_rn(parts[4+tid],parts[4+1ull*worlds*nv+tid]);}
