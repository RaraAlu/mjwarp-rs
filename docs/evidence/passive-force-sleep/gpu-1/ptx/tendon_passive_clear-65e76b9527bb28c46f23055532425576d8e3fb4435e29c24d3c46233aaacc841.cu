#define TP_NV 2
#define TP_NT 1
#define TP_NNZ 1
#define TP_STRIDE 8
#define TP_MAX_ROW 1
#define TP_PL 49
#define TP_S 1
#define TP_D 1
__device__ __constant__ unsigned TP_BATCH[5]={2,3,5,7,11};
__device__ __constant__ unsigned TP_OFF[5]={0,2,8,13,27};

__device__ float tp_poly(float linear,float a,float b,float x){float result=linear;result=result+a*x;result=result+(b*x)*x;return result;}
extern "C" __global__ void tendon_passive_validate(const int* meta,const float* p,const float* source,const float* velocity,float* status,unsigned nt,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=nt==TP_NT&&nv==TP_NV;
 for(unsigned k=0;k<4;++k)ok=ok&&meta[k]==-131072&&meta[4+2ull*TP_NT+TP_NNZ+k]==-131072&&p[k]==-131072.0f&&p[4+TP_PL+k]==-131072.0f&&source[k]==-131072.0f&&source[4+1ull*worlds*TP_STRIDE+k]==-131072.0f&&velocity[k]==-131072.0f&&velocity[4+1ull*worlds*TP_NT+k]==-131072.0f;
 for(unsigned i=0;i<TP_PL;++i)ok=ok&&isfinite(p[4+i]);
 for(unsigned t=0;t<TP_NT;++t){int start=meta[4+t],n=meta[4+TP_NT+t];if(start<0||n<0||start>TP_NNZ||n>TP_NNZ-start){ok=false;continue;}unsigned si=4+TP_OFF[0]+(w%TP_BATCH[0])*TP_NT+t,spi=4+TP_OFF[1]+2*((w%TP_BATCH[1])*TP_NT+t),di=4+TP_OFF[2]+(w%TP_BATCH[2])*TP_NT+t,dpi=4+TP_OFF[3]+2*((w%TP_BATCH[3])*TP_NT+t);bool has_s=!TP_S&&(p[si]!=0.0f||p[spi]!=0.0f||p[spi+1]!=0.0f),has_d=!TP_D&&(p[di]!=0.0f||p[dpi]!=0.0f||p[dpi+1]!=0.0f);if(n>0&&has_s)ok=ok&&isfinite(source[4+1ull*w*TP_STRIDE+t]);if(n>0&&has_d)ok=ok&&isfinite(velocity[4+1ull*w*TP_NT+t]);for(int i=0;i<n;++i){int c=meta[4+2ull*TP_NT+start+i];if(c<0||c>=TP_NV)ok=false;if(has_s||has_d)ok=ok&&isfinite(source[4+1ull*w*TP_STRIDE+TP_NT+start+i]);}}
 status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void tendon_passive_clear(const int* meta,const float* p,const float* source,const float* velocity,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<work)out[4+i]=0.0f;}
extern "C" __global__ void tendon_passive_compute(const int* meta,const float* p,const float* source,const float* velocity,float* out,unsigned nt,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned k=tid%TP_MAX_ROW,q=tid/TP_MAX_ROW,t=q%TP_NT,w=q/TP_NT;unsigned si=4+TP_OFF[0]+(w%TP_BATCH[0])*TP_NT+t,spi=4+TP_OFF[1]+2*((w%TP_BATCH[1])*TP_NT+t),di=4+TP_OFF[2]+(w%TP_BATCH[2])*TP_NT+t,dpi=4+TP_OFF[3]+2*((w%TP_BATCH[3])*TP_NT+t);float s=p[si],sa=p[spi],sb=p[spi+1],d=p[di],da=p[dpi],db=p[dpi+1];bool has_s=!TP_S&&(s!=0.0f||sa!=0.0f||sb!=0.0f),has_d=!TP_D&&(d!=0.0f||da!=0.0f||db!=0.0f);if(!has_s&&!has_d)return;int n=meta[4+TP_NT+t];if(k>=(unsigned)n)return;int sparse=meta[4+t]+k,col=meta[4+2ull*TP_NT+sparse];float J=source[4+1ull*w*TP_STRIDE+TP_NT+sparse];
 if(has_s){unsigned li=4+TP_OFF[4]+2*((w%TP_BATCH[4])*TP_NT+t);float length=source[4+1ull*w*TP_STRIDE+t],lower=p[li],upper=p[li+1],x=length>upper?length-upper:(length<lower?length-lower:0.0f);float force=-x*tp_poly(s,sa,sb,x);atomicAdd(&out[4+1ull*w*TP_NV+col],J*force);}
 if(has_d){float v=velocity[4+1ull*w*TP_NT+t];float force=-v*tp_poly(d,da,db,fabsf(v));atomicAdd(&out[4+1ull*worlds*TP_NV+1ull*w*TP_NV+col],J*force);}
}
