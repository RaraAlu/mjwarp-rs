#define FF_NB 3
#define FF_NV 2
#define FF_NG 0
#define FF_HE 0
#define FF_PL 26
#define FF_GF 26
#define FF_RS 96
#define FF_CS 54
#define FF_VS 30
#define FF_AS 0
__device__ __constant__ unsigned FF_BATCH[6]={2,3,5,1,1,1};
__device__ __constant__ unsigned FF_OFF[6]={0,6,9,14,17,26};

#define FF_MINVAL 1.0e-15f
#define FF_PI 3.14159265358979323846f
#define FF_FOUR_THIRDS_PI 4.18879032135009765625f
#define FF_EIGHT_FIFTEENTHS_PI 1.6755161285400390625f
#define FF_TWO_THIRDS 0.666666686534881591796875f
#define FF_THREE_PI 9.424777984619140625f
__device__ const float* ff_field(const float* p,unsigned field,unsigned w,unsigned width){return p+4+FF_OFF[field]+(w%FF_BATCH[field])*width;}
__device__ float ff_dot6(const float* a,const float* b){float ret=0.0f;
 #pragma unroll
 for(unsigned i=0;i<6;++i)ret+=a[i]*b[i];return ret;}
__device__ float ff_dot3(const float* a,const float* b){return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}
__device__ void ff_cross(const float* a,const float* b,float* out){out[0]=a[1]*b[2]-a[2]*b[1];out[1]=a[2]*b[0]-a[0]*b[2];out[2]=a[0]*b[1]-a[1]*b[0];}
__device__ float ff_pow4(float x){float sq=x*x;return sq*sq;}
__device__ void ff_mul(const float* m,const float* v,float* out){out[0]=m[0]*v[0]+m[1]*v[1]+m[2]*v[2];out[1]=m[3]*v[0]+m[4]*v[1]+m[5]*v[2];out[2]=m[6]*v[0]+m[7]*v[1]+m[8]*v[2];}
__device__ void ff_mul_t(const float* m,const float* v,float* out){out[0]=m[0]*v[0]+m[3]*v[1]+m[6]*v[2];out[1]=m[1]*v[0]+m[4]*v[1]+m[7]*v[2];out[2]=m[2]*v[0]+m[5]*v[1]+m[8]*v[2];}
__device__ void ff_semiaxes(const float* size,int type,float* out){out[0]=size[0];out[1]=size[1];out[2]=size[2];if(type==2){out[1]=size[0];out[2]=size[0];}else if(type==3){out[1]=size[0];out[2]=size[1]+size[0];}else if(type==5){out[1]=size[0];out[2]=size[1];}}
__device__ float ff_ellipsoid_moment(const float* size,unsigned dir){float d0=size[dir],d1=size[(dir+1)%3],d2=size[(dir+2)%3];return FF_EIGHT_FIFTEENTHS_PI*d0*ff_pow4(fmaxf(d1,d2));}
extern "C" __global__ void fluid_force_validate(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* attached,float* status,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 const int* parent=raw+4;const int* root=parent+FF_NB;const int* body=root+FF_NB;const int* geom_body=body+FF_NV;const int* geom_type=geom_body+FF_NG;const int* ellipsoid=geom_type+FF_NG;
 for(unsigned i=0;i<4;++i)ok=ok&&raw[i]==-131072&&raw[4+3*FF_NB+FF_NV+2*FF_NG+i]==-131072&&p[i]==-131072.0f&&p[4+FF_PL+i]==-131072.0f;
 for(unsigned i=0;i<4;++i)ok=ok&&rigid[i]==-131072.0f&&rigid[4+(unsigned long long)worlds*FF_RS+i]==-131072.0f&&com[i]==-131072.0f&&com[4+(unsigned long long)worlds*FF_CS+i]==-131072.0f&&velocity[i]==-131072.0f&&velocity[4+(unsigned long long)worlds*FF_VS+i]==-131072.0f;
 if(FF_HE)for(unsigned i=0;i<4;++i)ok=ok&&attached[i]==-131072.0f&&attached[4+(unsigned long long)worlds*FF_AS+i]==-131072.0f;
 for(unsigned b=0;b<FF_NB;++b){int parentid=parent[b],rootid=root[b];bool parent_ok=(b==0&&parentid==0)||(b>0&&parentid>=0&&(unsigned)parentid<b);bool root_ok=rootid>=0&&(unsigned)rootid<FF_NB;if(!parent_ok||!root_ok){ok=false;continue;}int expected=b==0?0:(parentid==0?(int)b:root[parentid]);if(rootid!=expected||(ellipsoid[b]!=0&&ellipsoid[b]!=1))ok=false;}
 for(unsigned d=0;d<FF_NV;++d)if(body[d]<0||(unsigned)body[d]>=FF_NB)ok=false;
 if(!ok){status[4+w]=0.0f;return;}
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];const float* mass=ff_field(p,3,w,FF_NB);const float* inertia=ff_field(p,4,w,3*FF_NB);
 ok=ok&&isfinite(wind[0])&&isfinite(wind[1])&&isfinite(wind[2])&&isfinite(density)&&isfinite(viscosity);
 const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* ximat=r+19*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB;const float* cvel=velocity+4+(unsigned long long)w*FF_VS;
 for(unsigned b=0;b<FF_NB;++b){ok=ok&&isfinite(mass[b])&&isfinite(inertia[3*b])&&isfinite(inertia[3*b+1])&&isfinite(inertia[3*b+2]);if(b==0||mass[b]<FF_MINVAL)continue;for(unsigned i=0;i<3;++i)ok=ok&&isfinite(xipos[3*b+i])&&isfinite(subtree[3*(unsigned)root[b]+i]);for(unsigned i=0;i<9;++i)ok=ok&&isfinite(ximat[9*b+i]);for(unsigned i=0;i<6;++i)ok=ok&&isfinite(cvel[6*b+i]);}
 const float* size=ff_field(p,5,w,3*FF_NG);const float* fluid=p+4+FF_GF;const float* frames=attached+4+(unsigned long long)w*FF_AS;const float* geom_xmat=frames+3*FF_NG;
 for(unsigned g=0;g<FF_NG;++g){if(geom_body[g]<0||(unsigned)geom_body[g]>=FF_NB)ok=false;for(unsigned i=0;i<3;++i){ok=ok&&isfinite(size[3*g+i]);if(FF_HE)ok=ok&&isfinite(frames[3*g+i]);}if(FF_HE)for(unsigned i=0;i<9;++i)ok=ok&&isfinite(geom_xmat[9*g+i]);for(unsigned i=0;i<12;++i)ok=ok&&isfinite(fluid[12*g+i]);}
 for(unsigned i=0;i<6*FF_NV;++i)ok=ok&&isfinite(cdof[i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void fluid_force_body(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* attached,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned b=tid%FF_NB,w=tid/FF_NB;float* out=output+4+6ull*tid;for(unsigned i=0;i<6;++i)out[i]=0.0f;if(b==0)return;
 const int* root=raw+4+FF_NB;const int* dof_body=root+FF_NB;const int* geom_body=dof_body+FF_NV;const int* geom_type=geom_body+FF_NG;const int* ellipsoid=geom_type+FF_NG;const float* mass=ff_field(p,3,w,FF_NB);const float body_mass=mass[b];if(body_mass<FF_MINVAL)return;
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];bool has_viscosity=viscosity>0.0f,has_density=density>0.0f;
 const float* inertia=ff_field(p,4,w,3*FF_NB)+3*b;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* rot=r+19*FF_NB+9*b;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* v=velocity+4+(unsigned long long)w*FF_VS+6*b;
 float offset[3]={xipos[3*b]-subtree[3*(unsigned)root[b]],xipos[3*b+1]-subtree[3*(unsigned)root[b]+1],xipos[3*b+2]-subtree[3*(unsigned)root[b]+2]};
 float cross[3]={offset[1]*v[2]-offset[2]*v[1],offset[2]*v[0]-offset[0]*v[2],offset[0]*v[1]-offset[1]*v[0]};
 float lin[3]={v[3]-cross[0],v[4]-cross[1],v[5]-cross[2]};
 if(ellipsoid[b]){
  const float* frames=attached+4+(unsigned long long)w*FF_AS;const float* geom_xpos=frames;const float* geom_xmat=frames+3*FF_NG;const float* sizes=ff_field(p,5,w,3*FF_NG);const float* fluids=p+4+FF_GF;float force_global[3]={0.0f,0.0f,0.0f},torque_global[3]={0.0f,0.0f,0.0f};
  for(unsigned g=0;g<FF_NG;++g){if((unsigned)geom_body[g]!=b)continue;const float* fluid=fluids+12*g;float coef=fluid[0];if(coef<=0.0f)continue;const float* grot=geom_xmat+9*g;const float* gpos=geom_xpos+3*g;float semiaxes[3];ff_semiaxes(sizes+3*g,geom_type[g],semiaxes);
   float point_offset[3]={gpos[0]-xipos[3*b],gpos[1]-xipos[3*b+1],gpos[2]-xipos[3*b+2]},point_cross[3],lin_point[3];ff_cross(v,point_offset,point_cross);for(unsigned i=0;i<3;++i)lin_point[i]=lin[i]+point_cross[i];
   float lang[3],llin[3];ff_mul_t(grot,v,lang);ff_mul_t(grot,lin_point,llin);if(wind[0]!=0.0f||wind[1]!=0.0f||wind[2]!=0.0f){float local_wind[3];ff_mul_t(grot,wind,local_wind);for(unsigned i=0;i<3;++i)llin[i]-=local_wind[i];}
   float ltorque[3]={0.0f,0.0f,0.0f},lforce[3]={0.0f,0.0f,0.0f};
   if(has_density){float virtual_lin[3]={density*fluid[6]*llin[0],density*fluid[7]*llin[1],density*fluid[8]*llin[2]};float virtual_ang[3]={density*fluid[9]*lang[0],density*fluid[10]*lang[1],density*fluid[11]*lang[2]};float added_force[3],lin_cross[3],ang_cross[3];ff_cross(virtual_lin,lang,added_force);ff_cross(virtual_lin,llin,lin_cross);ff_cross(virtual_ang,lang,ang_cross);for(unsigned i=0;i<3;++i){lforce[i]+=added_force[i];ltorque[i]+=lin_cross[i]+ang_cross[i];}}
   float volume=FF_FOUR_THIRDS_PI*semiaxes[0]*semiaxes[1]*semiaxes[2];float dmax=fmaxf(fmaxf(semiaxes[0],semiaxes[1]),semiaxes[2]);float dmin=fminf(fminf(semiaxes[0],semiaxes[1]),semiaxes[2]);float dmid=semiaxes[0]+semiaxes[1]+semiaxes[2]-dmax-dmin;float amax=FF_PI*dmax*dmid;
   float lin_speed=sqrtf(ff_dot3(llin,llin));float inv_speed=lin_speed>FF_MINVAL?1.0f/lin_speed:0.0f;float unit[3]={llin[0]*inv_speed,llin[1]*inv_speed,llin[2]*inv_speed};float inv_dmax=dmax>FF_MINVAL?1.0f/dmax:0.0f;float scaled[3]={semiaxes[0]*inv_dmax,semiaxes[1]*inv_dmax,semiaxes[2]*inv_dmax};
   float magnus[3];ff_cross(lang,llin,magnus);float magnus_scale=fluid[5]*density*volume;for(unsigned i=0;i<3;++i)magnus[i]*=magnus_scale;
   float s12=scaled[1]*scaled[2],s20=scaled[2]*scaled[0],s01=scaled[0]*scaled[1];float s12_sq=s12*s12,s20_sq=s20*s20,s01_sq=s01*s01;float unit0_sq=unit[0]*unit[0],unit1_sq=unit[1]*unit[1],unit2_sq=unit[2]*unit[2];float proj_num=s12_sq*unit0_sq+s20_sq*unit1_sq+s01_sq*unit2_sq;float inv_proj=proj_num>FF_MINVAL?1.0f/proj_num:0.0f;float pa=s12_sq*inv_proj,pb=s20_sq*inv_proj,pc=s01_sq*inv_proj;float proj_denom=pa*pa*unit[0]*unit[0]+pb*pb*unit[1]*unit[1]+pc*pc*unit[2]*unit[2];float aproj=FF_PI*dmax*dmax*sqrtf(proj_num*proj_denom);float cos_alpha=proj_denom>FF_MINVAL?1.0f/proj_denom:0.0f;float norm[3]={pa*unit[0],pb*unit[1],pc*unit[2]};
   float kutta[3]={0.0f,0.0f,0.0f};if(has_density&&fluid[4]!=0.0f&&lin_speed>FF_MINVAL){float circ[3];ff_cross(norm,llin,circ);float circ_scale=fluid[4]*density*cos_alpha*aproj;for(unsigned i=0;i<3;++i)circ[i]*=circ_scale;ff_cross(circ,llin,kutta);}
   float eq_d=FF_TWO_THIRDS*(semiaxes[0]+semiaxes[1]+semiaxes[2]);float visc_force=FF_THREE_PI*eq_d;float visc_torque=FF_PI*eq_d*eq_d*eq_d;float imax=FF_EIGHT_FIFTEENTHS_PI*dmid*ff_pow4(dmax);float ii[3]={ff_ellipsoid_moment(semiaxes,0),ff_ellipsoid_moment(semiaxes,1),ff_ellipsoid_moment(semiaxes,2)};float mom[3]={lang[0]*(fluid[3]*ii[0]+fluid[2]*(imax-ii[0])),lang[1]*(fluid[3]*ii[1]+fluid[2]*(imax-ii[1])),lang[2]*(fluid[3]*ii[2]+fluid[2]*(imax-ii[2]))};float drag_lin=viscosity*visc_force+density*lin_speed*(aproj*fluid[1]+fluid[2]*(amax-aproj));float drag_ang=viscosity*visc_torque+density*sqrtf(ff_dot3(mom,mom));
   for(unsigned i=0;i<3;++i){ltorque[i]-=drag_ang*lang[i];lforce[i]+=magnus[i]+kutta[i]-drag_lin*llin[i];ltorque[i]*=coef;lforce[i]*=coef;}float world_force[3],world_torque[3];ff_mul(grot,lforce,world_force);ff_mul(grot,ltorque,world_torque);for(unsigned i=0;i<3;++i){force_global[i]+=world_force[i];torque_global[i]+=world_torque[i];}
  }
  for(unsigned i=0;i<3;++i){out[i]=force_global[i];out[3+i]=torque_global[i];}return;
 }
 if(!has_viscosity&&!has_density)return;
 float lang[3]={rot[0]*v[0]+rot[3]*v[1]+rot[6]*v[2],rot[1]*v[0]+rot[4]*v[1]+rot[7]*v[2],rot[2]*v[0]+rot[5]*v[1]+rot[8]*v[2]};
 float llin[3]={rot[0]*lin[0]+rot[3]*lin[1]+rot[6]*lin[2],rot[1]*lin[0]+rot[4]*lin[1]+rot[7]*lin[2],rot[2]*lin[0]+rot[5]*lin[1]+rot[8]*lin[2]};
 if(wind[0]!=0.0f||wind[1]!=0.0f||wind[2]!=0.0f){llin[0]-=rot[0]*wind[0]+rot[3]*wind[1]+rot[6]*wind[2];llin[1]-=rot[1]*wind[0]+rot[4]*wind[1]+rot[7]*wind[2];llin[2]-=rot[2]*wind[0]+rot[5]*wind[1]+rot[8]*wind[2];}
 float scl=6.0f/body_mass;float box0=sqrtf(fmaxf(FF_MINVAL,inertia[1]+inertia[2]-inertia[0])*scl);float box1=sqrtf(fmaxf(FF_MINVAL,inertia[0]+inertia[2]-inertia[1])*scl);float box2=sqrtf(fmaxf(FF_MINVAL,inertia[0]+inertia[1]-inertia[2])*scl);
 float ltorque[3]={0.0f,0.0f,0.0f},lforce[3]={0.0f,0.0f,0.0f};
 if(has_viscosity){float diam=(box0+box1+box2)/3.0f;float diam_pow3=powf(diam,3.0f);for(unsigned i=0;i<3;++i){ltorque[i]=((-lang[i]*diam_pow3)*FF_PI)*viscosity;lforce[i]=(((-3.0f*llin[i])*diam)*FF_PI)*viscosity;}}
 if(has_density){lforce[0]-=0.5f*density*box1*box2*fabsf(llin[0])*llin[0];lforce[1]-=0.5f*density*box0*box2*fabsf(llin[1])*llin[1];lforce[2]-=0.5f*density*box0*box1*fabsf(llin[2])*llin[2];scl=density/64.0f;float box0_pow4=powf(box0,4.0f),box1_pow4=powf(box1,4.0f),box2_pow4=powf(box2,4.0f);ltorque[0]-=box0*(box1_pow4+box2_pow4)*fabsf(lang[0])*lang[0]*scl;ltorque[1]-=box1*(box0_pow4+box2_pow4)*fabsf(lang[1])*lang[1]*scl;ltorque[2]-=box2*(box0_pow4+box1_pow4)*fabsf(lang[2])*lang[2]*scl;}
 out[0]=rot[0]*lforce[0]+rot[1]*lforce[1]+rot[2]*lforce[2];out[1]=rot[3]*lforce[0]+rot[4]*lforce[1]+rot[5]*lforce[2];out[2]=rot[6]*lforce[0]+rot[7]*lforce[1]+rot[8]*lforce[2];out[3]=rot[0]*ltorque[0]+rot[1]*ltorque[1]+rot[2]*ltorque[2];out[4]=rot[3]*ltorque[0]+rot[4]*ltorque[1]+rot[5]*ltorque[2];out[5]=rot[6]*ltorque[0]+rot[7]*ltorque[1]+rot[8]*ltorque[2];
}
extern "C" __global__ void fluid_force_project(const int* raw,const float* p,const float* rigid,const float* com,const float* body_force,const float* unused,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/FF_NV,d=tid%FF_NV;const int* parent=raw+4;const int* root=parent+FF_NB;const int* dof_body=root+FF_NB;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB+6*d;const float* force=body_force+4+6ull*w*FF_NB;float jac[6]={cdof[3],cdof[4],cdof[5],cdof[0],cdof[1],cdof[2]};float accum=0.0f;int dofbody=dof_body[d];
 for(unsigned b=(unsigned)dofbody;b<FF_NB;++b){const float* f=force+6*b;if(f[0]==0.0f&&f[1]==0.0f&&f[2]==0.0f&&f[3]==0.0f&&f[4]==0.0f&&f[5]==0.0f)continue;int cursor=(int)b;while(cursor!=0&&cursor!=dofbody)cursor=parent[cursor];if(cursor==0)continue;const float* point=xipos+3*b;const float* center=subtree+3*(unsigned)root[b];float offset[3]={point[0]-center[0],point[1]-center[1],point[2]-center[2]};float cross[3]={cdof[1]*offset[2]-cdof[2]*offset[1],cdof[2]*offset[0]-cdof[0]*offset[2],cdof[0]*offset[1]-cdof[1]*offset[0]};accum+=ff_dot6(jac,f)+ff_dot3(cross,f);}
 output[4+tid]=accum;
}
extern "C" __global__ void fluid_force_clear(const int* raw,const float* p,const float* first,const float* second,const float* third,const float* fourth,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;output[4+tid]=0.0f;
}
