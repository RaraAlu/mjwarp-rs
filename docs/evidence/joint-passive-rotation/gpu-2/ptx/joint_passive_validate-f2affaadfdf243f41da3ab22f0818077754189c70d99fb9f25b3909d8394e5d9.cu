#define JP_NQ 13
#define JP_NV 11
#define JP_NJ 4
#define JP_PL 214
#define JP_S 1
#define JP_D 0
__device__ __constant__ unsigned JP_BATCH[5]={13,1,1,1,1};
__device__ __constant__ unsigned JP_OFF[5]={0,169,173,181,192};

__device__ float jp_poly(float linear,float a,float b,float x){float result=linear;result=result+a*x;result=result+(b*x)*x;return result;}
__device__ float jp_len3(float x,float y,float z){return sqrtf((x*x+y*y)+z*z);}
__device__ void jp_quat_sub(const float* qa,const float* qb,float* out){
 float n=sqrtf(((qa[0]*qa[0]+qa[1]*qa[1])+qa[2]*qa[2])+qa[3]*qa[3]);
 float a0,a1,a2,a3;
 if(n>0.0f){float inv=1.0f/n;a0=qa[0]*inv;a1=qa[1]*inv;a2=qa[2]*inv;a3=qa[3]*inv;}
 else{a0=0.0f;a1=0.0f;a2=0.0f;a3=1.0f;}
 float b0=qb[0],b1=-qb[1],b2=-qb[2],b3=-qb[3];
 float d0=((b0*a0-b1*a1)-b2*a2)-b3*a3;
 float d1=((b0*a1+b1*a0)+b2*a3)-b3*a2;
 float d2=((b0*a2-b1*a3)+b2*a0)+b3*a1;
 float d3=((b0*a3+b1*a2)-b2*a1)+b3*a0;
 float sine=jp_len3(d1,d2,d3);
 if(sine==0.0f){out[0]=0.0f;out[1]=0.0f;out[2]=0.0f;return;}
 float speed=2.0f*atan2f(sine,d0);
 if(speed>3.14159265358979323846f)speed-=6.28318530717958647692f;
 out[0]=(d1*speed)/sine;out[1]=(d2*speed)/sine;out[2]=(d3*speed)/sine;
}
extern "C" __global__ void joint_passive_validate(const int* meta,const float* p,const float* state,float* status,unsigned nq,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool valid=nq==JP_NQ&&nv==JP_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+4ull*JP_NV+j]==-131072&&p[j]==-131072.0f&&p[4+JP_PL+j]==-131072.0f&&state[j]==-131072.0f&&state[4+1ull*worlds*(JP_NQ+JP_NV)+j]==-131072.0f;
 for(unsigned j=0;j<JP_PL;++j)valid=valid&&isfinite(p[4+j]);
 for(unsigned j=0;j<JP_NQ;++j)valid=valid&&isfinite(state[4+1ull*w*JP_NQ+j]);for(unsigned j=0;j<JP_NV;++j)valid=valid&&isfinite(state[4+1ull*worlds*JP_NQ+1ull*w*JP_NV+j]);
 for(unsigned i=0;i<JP_NV;++i){int kind=meta[4+i],q=meta[4+JP_NV+i],j=meta[4+2ull*JP_NV+i],local=meta[4+3ull*JP_NV+i];unsigned qw=kind==0?7u:(kind==1?4u:1u),dw=kind==0?6u:(kind==1?3u:1u);if(kind<0||kind>3||q<0||j<0||j>=JP_NJ||local<0||(unsigned)local>=dw||qw>nq||(unsigned)q>nq-qw){valid=false;continue;}}
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void joint_passive_compute(const int* meta,const float* p,const float* state,float* out,unsigned nq,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/nv,i=tid%nv;int kind=meta[4+i],q=meta[4+JP_NV+i],j=meta[4+2ull*JP_NV+i],local=meta[4+3ull*JP_NV+i];float spring=0.0f,damper=0.0f;
 if(!JP_D){unsigned di=4+JP_OFF[3]+(w%JP_BATCH[3])*JP_NV+i,pi=4+JP_OFF[4]+2*((w%JP_BATCH[4])*JP_NV+i);float d=p[di],a=p[pi],b=p[pi+1];if(d!=0.0f||a!=0.0f||b!=0.0f){float v=state[4+1ull*worlds*JP_NQ+1ull*w*JP_NV+i];damper=-v*jp_poly(d,a,b,fabsf(v));}}
 if(!JP_S){unsigned si=4+JP_OFF[1]+(w%JP_BATCH[1])*JP_NJ+j,pi=4+JP_OFF[2]+2*((w%JP_BATCH[2])*JP_NJ+j);float s=p[si],a=p[pi],b=p[pi+1];if(s!=0.0f||a!=0.0f||b!=0.0f){const float* qs=state+4+1ull*w*JP_NQ+q;const float* qr=p+4+JP_OFF[0]+(w%JP_BATCH[0])*JP_NQ+q;
   if(kind==0&&local<3){float x=qs[0]-qr[0],y=qs[1]-qr[1],z=qs[2]-qr[2],r=jp_len3(x,y,z),k=jp_poly(s,a,b,r);float dif=local==0?x:(local==1?y:z);spring=-k*dif;}
   else if(kind<2){unsigned qo=kind==0?3u:0u;float dif[3];jp_quat_sub(qs+qo,qr+qo,dif);float r=jp_len3(dif[0],dif[1],dif[2]),k=jp_poly(s,a,b,r);spring=-k*dif[kind==0?local-3:local];}
   else{float dif=qs[0]-qr[0];spring=-dif*jp_poly(s,a,b,dif);}}}
 out[4+tid]=spring;out[4+1ull*worlds*nv+tid]=damper;
}
extern "C" __global__ void joint_passive_sum(const int* meta,const float* parts,const float* unused,float* forces,unsigned ignored,unsigned nv,unsigned worlds,unsigned work){unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;forces[4+tid]=__fadd_rn(parts[4+tid],parts[4+1ull*worlds*nv+tid]);}
