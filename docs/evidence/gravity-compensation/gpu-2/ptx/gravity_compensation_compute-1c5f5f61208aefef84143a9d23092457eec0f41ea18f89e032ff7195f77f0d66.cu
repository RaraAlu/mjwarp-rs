#define GC_NB 5
#define GC_NJ 4
#define GC_NV 11
#define GC_PL 36
#define GC_RS 164
#define GC_CS 136
#define GC_ENABLED 1
__device__ __constant__ unsigned GC_BATCH[3]={2,3,3};
__device__ __constant__ unsigned GC_OFF[3]={0,6,21};

__device__ const float* gc_field(const float* p,unsigned field,unsigned w,unsigned width){return p+4+GC_OFF[field]+(w%GC_BATCH[field])*width;}
__device__ bool gc_finite(float x){return isfinite(x);}
extern "C" __global__ void gravity_compensation_clear(const int* m,const float* p,const float* state,float* out,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=0.0f;
}
extern "C" __global__ void gravity_compensation_validate(const int* raw,const float* p,const float* rigid,const float* com,float* status,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;bool ok=true;
 const int* root=raw+4;const int* ancestor=root+GC_NB;const int* dof_jnt=ancestor+GC_NB*GC_NV;const int* act=dof_jnt+GC_NV;
 for(int i=0;i<4;i++)ok=ok&&raw[i]==-131072&&raw[4+GC_NB+GC_NB*GC_NV+GC_NV+GC_NJ+i]==-131072;
 for(int i=0;i<4;i++)ok=ok&&p[i]==-131072.0f&&p[4+GC_PL+i]==-131072.0f;
 for(int i=0;i<4;i++)ok=ok&&rigid[i]==-131072.0f&&rigid[4+(unsigned long long)worlds*GC_RS+i]==-131072.0f;
 for(int i=0;i<4;i++)ok=ok&&com[i]==-131072.0f&&com[4+(unsigned long long)worlds*GC_CS+i]==-131072.0f;
 for(unsigned b=0;b<GC_NB;b++)ok=ok&&root[b]>=0&&root[b]<(int)GC_NB;
 for(unsigned d=0;d<GC_NV;d++)ok=ok&&dof_jnt[d]>=0&&dof_jnt[d]<(int)GC_NJ;
 for(unsigned j=0;j<GC_NJ;j++)ok=ok&&(act[j]==0||act[j]==1);
 const float* gravity=gc_field(p,0,w,3);const float* mass=gc_field(p,1,w,GC_NB);const float* gravcomp=gc_field(p,2,w,GC_NB);
 const float* r=rigid+4+(unsigned long long)w*GC_RS;const float* xipos=r+16*GC_NB;
 const float* c=com+4+(unsigned long long)w*GC_CS;const float* subtree=c+GC_NB;const float* cdof=c+14*GC_NB;
 for(unsigned b=1;b<GC_NB;b++){float gc=gravcomp[b];ok=ok&&gc_finite(gc);if(gc==0.0f)continue;
  ok=ok&&gc_finite(gravity[0])&&gc_finite(gravity[1])&&gc_finite(gravity[2])&&gc_finite(mass[b]);
  int rb=root[b];bool root_ok=rb>=0&&rb<(int)GC_NB;ok=ok&&root_ok;
  if(root_ok)for(int a=0;a<3;a++)ok=ok&&gc_finite(xipos[3*b+a])&&gc_finite(subtree[3*rb+a]);
  for(unsigned d=0;d<GC_NV;d++){int affect=ancestor[b*GC_NV+d];ok=ok&&(affect==0||affect==1);if(affect)for(int a=0;a<6;a++)ok=ok&&gc_finite(cdof[6*d+a]);}
 }
 status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void gravity_compensation_compute(const int* raw,const float* p,const float* rigid,const float* com,float* out,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned d=tid%GC_NV;unsigned q=tid/GC_NV;unsigned b=1+q%(GC_NB-1);unsigned w=q/(GC_NB-1);
 const int* root=raw+4;const int* ancestor=root+GC_NB;
 const float* gravcomp=gc_field(p,2,w,GC_NB);float gc=gravcomp[b];if(gc==0.0f)return;
 const float* gravity=gc_field(p,0,w,3);const float* mass=gc_field(p,1,w,GC_NB);
 const float* r=rigid+4+(unsigned long long)w*GC_RS;const float* xipos=r+16*GC_NB;
 const float* c=com+4+(unsigned long long)w*GC_CS;const float* subtree=c+GC_NB;const float* cdof=c+14*GC_NB+6*d;
 float fx=(-gravity[0]*mass[b])*gc,fy=(-gravity[1]*mass[b])*gc,fz=(-gravity[2]*mass[b])*gc;
 float jx=0.0f,jy=0.0f,jz=0.0f;
 if(ancestor[b*GC_NV+d]){
  float ox=xipos[3*b]-subtree[3*root[b]],oy=xipos[3*b+1]-subtree[3*root[b]+1],oz=xipos[3*b+2]-subtree[3*root[b]+2];
  jx=cdof[3]+(cdof[1]*oz-cdof[2]*oy);jy=cdof[4]+(cdof[2]*ox-cdof[0]*oz);jz=cdof[5]+(cdof[0]*oy-cdof[1]*ox);
 }
 atomicAdd(out+4+(unsigned long long)w*GC_NV+d,jx*fx+jy*fy+jz*fz);
}
extern "C" __global__ void gravity_compensation_sum(const int* raw,const float* parts,const float* gravity,const float* unused,float* out,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 const int* root=raw+4;const int* dof_jnt=root+GC_NB+GC_NB*GC_NV;const int* act=dof_jnt+GC_NV;
 float value=__fadd_rn(parts[4+tid],parts[4+(unsigned long long)worlds*GC_NV+tid]);
 if(GC_ENABLED&&!act[dof_jnt[tid%GC_NV]])value=__fadd_rn(value,gravity[4+tid]);out[4+tid]=value;
}
