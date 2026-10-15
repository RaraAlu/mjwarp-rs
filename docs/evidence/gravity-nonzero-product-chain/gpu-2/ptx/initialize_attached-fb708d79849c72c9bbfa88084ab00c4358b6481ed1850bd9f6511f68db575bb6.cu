
#define MJWARP_FIELD_BATCHES
__device__ const float* field_parameter(const float* p,const int* desc,unsigned world,unsigned width) {
  return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;
}

#define MJWARP_MOCAP

struct V { float x,y,z; };
struct Q { float w,x,y,z; };
__device__ V load_v(const float* p) { return {p[0],p[1],p[2]}; }
__device__ Q load_q(const float* p) { return {p[0],p[1],p[2],p[3]}; }
__device__ void store_v(float* p,V v) { p[0]=v.x;p[1]=v.y;p[2]=v.z; }
__device__ void store_q(float* p,Q q) { p[0]=q.w;p[1]=q.x;p[2]=q.y;p[3]=q.z; }
__device__ V add(V a,V b) { return {a.x+b.x,a.y+b.y,a.z+b.z}; }
__device__ V scale(V a,float s) { return {a.x*s,a.y*s,a.z*s}; }
__device__ float dot_v(V a,V b) { return a.x*b.x+a.y*b.y+a.z*b.z; }
__device__ V cross_v(V a,V b) { return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x}; }
__device__ Q normalize(Q q) {
  float n=sqrtf(q.w*q.w+q.x*q.x+q.y*q.y+q.z*q.z);
#ifdef MJWARP_G01_EQUIVALENT
  if(n==0.0f) return {0,0,0,1};
#endif
  float s=1.0f/n;
  return {q.w*s,q.x*s,q.y*s,q.z*s};
}
__device__ Q multiply(Q a,Q b) {
  return {a.w*b.w-a.x*b.x-a.y*b.y-a.z*b.z,
          a.w*b.x+a.x*b.w+a.y*b.z-a.z*b.y,
          a.w*b.y-a.x*b.z+a.y*b.w+a.z*b.x,
          a.w*b.z+a.x*b.y-a.y*b.x+a.z*b.w};
}
__device__ V rotate(Q q,V v) {
  V u={q.x,q.y,q.z};
  V lhs=scale(scale(u,dot_v(u,v)),2.0f);
  float factor=q.w*q.w-dot_v(u,u);
  V r=add(lhs,scale(v,factor));
  return add(r,scale(cross_v(u,v),2.0f*q.w));
}
__device__ void matrix(float* m,Q q) {
  m[0]=q.w*q.w+q.x*q.x-q.y*q.y-q.z*q.z;
  m[1]=2*(q.x*q.y-q.w*q.z); m[2]=2*(q.x*q.z+q.w*q.y);
  m[3]=2*(q.x*q.y+q.w*q.z); m[4]=q.w*q.w-q.x*q.x+q.y*q.y-q.z*q.z;
  m[5]=2*(q.y*q.z-q.w*q.x); m[6]=2*(q.x*q.z-q.w*q.y);
  m[7]=2*(q.y*q.z+q.w*q.x); m[8]=q.w*q.w-q.x*q.x-q.y*q.y+q.z*q.z;
}
extern "C" __global__ void rigid_kinematics(const int* meta,const float* model,
    const float* state,float* result,unsigned nq,unsigned nb,unsigned nj,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* parent=meta; const int* adr=parent+nb; const int* num=adr+nb;
  const int* type=num+nb; const int* qa=type+nj;
#ifdef MJWARP_FIELD_BATCHES
  const int* desc=qa+nj;
#ifdef MJWARP_MOCAP
  unsigned nm=desc[0]; const int* mocap=desc+1;
  desc=mocap+nb;
  const float* mp=state+(unsigned long long)worlds*nq+(unsigned long long)w*3*nm;
  const float* mq=state+(unsigned long long)worlds*(nq+3*nm)+(unsigned long long)w*4*nm;
#endif
  const float* q0=field_parameter(model,desc,w,nq);
  const float* bp=field_parameter(model,desc+2,w,3*nb);
  const float* bq=field_parameter(model,desc+4,w,4*nb);
  const float* ip=field_parameter(model,desc+6,w,3*nb);
  const float* iq=field_parameter(model,desc+8,w,4*nb);
  const float* jp=field_parameter(model,desc+10,w,3*nj);
  const float* axis=field_parameter(model,desc+12,w,3*nj);
#else
  const float* q0=model; const float* bp=q0+nq; const float* bq=bp+3*nb;
  const float* ip=bq+4*nb; const float* iq=ip+3*nb;
  const float* jp=iq+4*nb; const float* axis=jp+3*nj;
#endif
  const float* q=state+(unsigned long long)w*nq;
  float* xp=result+4+(unsigned long long)w*(28*nb+6*nj);
  float* xq=xp+3*nb; float* xm=xq+4*nb; float* xi=xm+9*nb;
  float* im=xi+3*nb; float* anchor=im+9*nb; float* xa=anchor+3*nj;
  for(unsigned b=0;b<nb;++b) {
    V p={0,0,0}; Q r={1,0,0,0};
#ifdef MJWARP_G01_EQUIVALENT
    if(!b) { p=load_v(xp); r=load_q(xq); }
#endif
    int j=adr[b], count=num[b];
    if(b && count==1 && type[j]==0) {
      p=load_v(q+qa[j]); r=normalize(load_q(q+qa[j]+3));
      store_v(anchor+3*j,p); store_v(xa+3*j,load_v(axis+3*j));
    } else if(b) {
      Q pr=load_q(xq+4*parent[b]);
      V local_p=load_v(bp+3*b); Q local_q=load_q(bq+4*b);
#ifdef MJWARP_MOCAP
      if(mocap[b]>=0) { local_p=load_v(mp+3*mocap[b]); local_q=load_q(mq+4*mocap[b]); }
#endif
      p=add(load_v(xp+3*parent[b]),rotate(pr,local_p));
      r=multiply(pr,local_q);
      for(int k=0;k<count;++k,++j) {
        V local=load_v(jp+3*j), a=add(p,rotate(r,local));
        V ax=rotate(r,load_v(axis+3*j));
        if(type[j]==2) p=add(p,scale(ax,q[qa[j]]-q0[qa[j]]));
        else {
          Q delta;
          if(type[j]==1) delta=normalize(load_q(q+qa[j]));
          else {
            float h=0.5f*(q[qa[j]]-q0[qa[j]]), s=sinf(h);
            V t=scale(load_v(axis+3*j),s); delta={cosf(h),t.x,t.y,t.z};
          }
          r=multiply(r,delta); p=add(a,scale(rotate(r,local),-1));
        }
        store_v(anchor+3*j,a); store_v(xa+3*j,ax);
      }
      r=normalize(r);
    }
    store_v(xp+3*b,p); store_q(xq+4*b,r); matrix(xm+9*b,r);
    store_v(xi+3*b,add(p,rotate(r,load_v(ip+3*b))));
    matrix(im+9*b,multiply(r,load_q(iq+4*b)));
  }
}


__device__ void attached_frames(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned ng,unsigned ns,unsigned worlds,bool initialize) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const float* xp=body+4+(unsigned long long)w*meta[0];
  const float* xq=xp+3*nb;
#ifdef MJWARP_FIELD_BATCHES
  const int* desc=meta+1+ng+ns;
#ifdef MJWARP_MOCAP
  const int* static_geom=desc; desc+=ng;
#endif
  const float* gp=field_parameter(model,desc,w,3*ng);
  const float* gq=field_parameter(model,desc+2,w,4*ng);
  const float* sp=field_parameter(model,desc+4,w,3*ns);
  const float* sq=field_parameter(model,desc+6,w,4*ns);
#else
  const float* gp=model; const float* gq=gp+3*ng;
  const float* sp=gq+4*ng; const float* sq=sp+3*ns;
#endif
  float* gx=result+4+(unsigned long long)w*(12*ng+12*ns);
  float* gm=gx+3*ng; float* sx=gm+9*ng; float* sm=sx+3*ns;
  for(unsigned g=0;g<ng;++g) {
#ifdef MJWARP_MOCAP
    if(!initialize && static_geom[g]) continue;
#endif
    int b=meta[1+g]; Q q=load_q(xq+4*b);
    store_v(gx+3*g,add(load_v(xp+3*b),rotate(q,load_v(gp+3*g))));
    matrix(gm+9*g,multiply(q,load_q(gq+4*g)));
  }
  for(unsigned s=0;s<ns;++s) {
    int b=meta[1+ng+s]; Q q=load_q(xq+4*b);
    store_v(sx+3*s,add(load_v(xp+3*b),rotate(q,load_v(sp+3*s))));
    matrix(sm+9*s,multiply(q,load_q(sq+4*s)));
  }
}
extern "C" __global__ void attached_kinematics(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned ng,unsigned ns,unsigned worlds) {
  attached_frames(meta,model,body,result,nb,ng,ns,worlds,false);
}
#ifdef MJWARP_MOCAP
extern "C" __global__ void initialize_attached(const int* meta,const float* model,
    const float* body,float* result,unsigned nb,unsigned ng,unsigned ns,unsigned worlds) {
  attached_frames(meta,model,body,result,nb,ng,ns,worlds,true);
}
#endif
