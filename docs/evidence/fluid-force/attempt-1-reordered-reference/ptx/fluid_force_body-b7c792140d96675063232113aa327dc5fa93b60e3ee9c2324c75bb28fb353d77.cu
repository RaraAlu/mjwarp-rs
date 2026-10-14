#define FF_NB 3
#define FF_NV 1
#define FF_PL 134
#define FF_RS 90
#define FF_CS 48
#define FF_VS 24
__device__ __constant__ unsigned FF_BATCH[5]={2,3,5,7,11};
__device__ __constant__ unsigned FF_OFF[5]={0,6,9,14,35};

#define FF_MINVAL 1.0e-15f
#define FF_PI 3.14159265358979323846f
__device__ const float* ff_field(const float* p,unsigned field,unsigned w,unsigned width){return p+4+FF_OFF[field]+(w%FF_BATCH[field])*width;}
__device__ float ff_dot6(const float* a,const float* b){float ret=0.0f;
 #pragma unroll
 for(unsigned i=0;i<6;++i)ret+=a[i]*b[i];return ret;}
__device__ float ff_dot3(const float* a,const float* b){return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}
extern "C" __global__ void fluid_force_validate(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* unused,float* status,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 const int* parent=raw+4;const int* root=parent+FF_NB;const int* body=root+FF_NB;
 for(unsigned i=0;i<4;++i)ok=ok&&raw[i]==-131072&&raw[4+2*FF_NB+FF_NV+i]==-131072&&p[i]==-131072.0f&&p[4+FF_PL+i]==-131072.0f;
 for(unsigned i=0;i<4;++i)ok=ok&&rigid[i]==-131072.0f&&rigid[4+(unsigned long long)worlds*FF_RS+i]==-131072.0f&&com[i]==-131072.0f&&com[4+(unsigned long long)worlds*FF_CS+i]==-131072.0f&&velocity[i]==-131072.0f&&velocity[4+(unsigned long long)worlds*FF_VS+i]==-131072.0f;
 for(unsigned b=0;b<FF_NB;++b){int parentid=parent[b],rootid=root[b];ok=ok&&((b==0&&parentid==0)||(b>0&&parentid>=0&&(unsigned)parentid<b))&&rootid>=0&&(unsigned)rootid<FF_NB;if(ok){int expected=b==0?0:(parentid==0?(int)b:root[parentid]);ok=ok&&rootid==expected;}}
 for(unsigned d=0;d<FF_NV;++d)ok=ok&&body[d]>=0&&(unsigned)body[d]<FF_NB;
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];const float* mass=ff_field(p,3,w,FF_NB);const float* inertia=ff_field(p,4,w,3*FF_NB);
 ok=ok&&isfinite(wind[0])&&isfinite(wind[1])&&isfinite(wind[2])&&isfinite(density)&&isfinite(viscosity);
 const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* ximat=r+19*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB;const float* cvel=velocity+4+(unsigned long long)w*FF_VS;
 for(unsigned b=0;b<FF_NB;++b){ok=ok&&isfinite(mass[b])&&isfinite(inertia[3*b])&&isfinite(inertia[3*b+1])&&isfinite(inertia[3*b+2]);if(b==0||mass[b]<FF_MINVAL)continue;for(unsigned i=0;i<3;++i)ok=ok&&isfinite(xipos[3*b+i])&&isfinite(subtree[3*(unsigned)root[b]+i]);for(unsigned i=0;i<9;++i)ok=ok&&isfinite(ximat[9*b+i]);for(unsigned i=0;i<6;++i)ok=ok&&isfinite(cvel[6*b+i]);}
 for(unsigned i=0;i<6*FF_NV;++i)ok=ok&&isfinite(cdof[i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void fluid_force_body(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* unused,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned b=tid%FF_NB,w=tid/FF_NB;float* out=output+4+6ull*tid;for(unsigned i=0;i<6;++i)out[i]=0.0f;if(b==0)return;
 const int* root=raw+4+FF_NB;const float* mass=ff_field(p,3,w,FF_NB);const float body_mass=mass[b];if(body_mass<FF_MINVAL)return;
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];bool has_viscosity=viscosity>0.0f,has_density=density>0.0f;if(!has_viscosity&&!has_density)return;
 const float* inertia=ff_field(p,4,w,3*FF_NB)+3*b;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* rot=r+19*FF_NB+9*b;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* v=velocity+4+(unsigned long long)w*FF_VS+6*b;
 float offset[3]={xipos[3*b]-subtree[3*(unsigned)root[b]],xipos[3*b+1]-subtree[3*(unsigned)root[b]+1],xipos[3*b+2]-subtree[3*(unsigned)root[b]+2]};
 float cross[3]={offset[1]*v[2]-offset[2]*v[1],offset[2]*v[0]-offset[0]*v[2],offset[0]*v[1]-offset[1]*v[0]};
 float lin[3]={v[3]-cross[0],v[4]-cross[1],v[5]-cross[2]};
 float lang[3]={rot[0]*v[0]+rot[3]*v[1]+rot[6]*v[2],rot[1]*v[0]+rot[4]*v[1]+rot[7]*v[2],rot[2]*v[0]+rot[5]*v[1]+rot[8]*v[2]};
 float llin[3]={rot[0]*lin[0]+rot[3]*lin[1]+rot[6]*lin[2],rot[1]*lin[0]+rot[4]*lin[1]+rot[7]*lin[2],rot[2]*lin[0]+rot[5]*lin[1]+rot[8]*lin[2]};
 if(wind[0]!=0.0f||wind[1]!=0.0f||wind[2]!=0.0f){llin[0]-=rot[0]*wind[0]+rot[3]*wind[1]+rot[6]*wind[2];llin[1]-=rot[1]*wind[0]+rot[4]*wind[1]+rot[7]*wind[2];llin[2]-=rot[2]*wind[0]+rot[5]*wind[1]+rot[8]*wind[2];}
 float scl=6.0f/body_mass;float box0=sqrtf(fmaxf(FF_MINVAL,inertia[1]+inertia[2]-inertia[0])*scl);float box1=sqrtf(fmaxf(FF_MINVAL,inertia[0]+inertia[2]-inertia[1])*scl);float box2=sqrtf(fmaxf(FF_MINVAL,inertia[0]+inertia[1]-inertia[2])*scl);
 float ltorque[3]={0.0f,0.0f,0.0f},lforce[3]={0.0f,0.0f,0.0f};
 if(has_viscosity){float diam=(box0+box1+box2)/3.0f;float angular=powf(diam,3.0f)*FF_PI*viscosity;for(unsigned i=0;i<3;++i){ltorque[i]=-lang[i]*angular;lforce[i]=-3.0f*llin[i]*diam*FF_PI*viscosity;}}
 if(has_density){lforce[0]-=0.5f*density*box1*box2*fabsf(llin[0])*llin[0];lforce[1]-=0.5f*density*box0*box2*fabsf(llin[1])*llin[1];lforce[2]-=0.5f*density*box0*box1*fabsf(llin[2])*llin[2];scl=density/64.0f;float box0_pow4=powf(box0,4.0f),box1_pow4=powf(box1,4.0f),box2_pow4=powf(box2,4.0f);ltorque[0]-=box0*(box1_pow4+box2_pow4)*fabsf(lang[0])*lang[0]*scl;ltorque[1]-=box1*(box0_pow4+box2_pow4)*fabsf(lang[1])*lang[1]*scl;ltorque[2]-=box2*(box0_pow4+box1_pow4)*fabsf(lang[2])*lang[2]*scl;}
 out[0]=rot[0]*lforce[0]+rot[1]*lforce[1]+rot[2]*lforce[2];out[1]=rot[3]*lforce[0]+rot[4]*lforce[1]+rot[5]*lforce[2];out[2]=rot[6]*lforce[0]+rot[7]*lforce[1]+rot[8]*lforce[2];out[3]=rot[0]*ltorque[0]+rot[1]*ltorque[1]+rot[2]*ltorque[2];out[4]=rot[3]*ltorque[0]+rot[4]*ltorque[1]+rot[5]*ltorque[2];out[5]=rot[6]*ltorque[0]+rot[7]*ltorque[1]+rot[8]*ltorque[2];
}
extern "C" __global__ void fluid_force_project(const int* raw,const float* p,const float* rigid,const float* com,const float* body_force,const float* unused,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/FF_NV,d=tid%FF_NV;const int* parent=raw+4;const int* root=parent+FF_NB;const int* dof_body=root+FF_NB;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB+6*d;const float* force=body_force+4+6ull*w*FF_NB;float jac[6]={cdof[3],cdof[4],cdof[5],cdof[0],cdof[1],cdof[2]};float accum=0.0f;int dofbody=dof_body[d];
 for(unsigned b=(unsigned)dofbody;b<FF_NB;++b){const float* f=force+6*b;if(f[0]==0.0f&&f[1]==0.0f&&f[2]==0.0f&&f[3]==0.0f&&f[4]==0.0f&&f[5]==0.0f)continue;int cursor=(int)b;while(cursor!=0&&cursor!=dofbody)cursor=parent[cursor];if(cursor==0)continue;const float* point=xipos+3*b;const float* center=subtree+3*(unsigned)root[b];float offset[3]={point[0]-center[0],point[1]-center[1],point[2]-center[2]};float cross[3]={cdof[1]*offset[2]-cdof[2]*offset[1],cdof[2]*offset[0]-cdof[0]*offset[2],cdof[0]*offset[1]-cdof[1]*offset[0]};accum+=ff_dot6(jac,f)+ff_dot3(cross,f);}
 output[4+tid]=accum;
}
