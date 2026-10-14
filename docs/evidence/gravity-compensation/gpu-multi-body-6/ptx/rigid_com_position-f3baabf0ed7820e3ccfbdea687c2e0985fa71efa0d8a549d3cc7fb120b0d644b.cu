
#define MJWARP_FIELD_BATCHES
__device__ const float* field_parameter(const float* p,const int* desc,unsigned world,unsigned width) {
  return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;
}


extern "C" __global__ void rigid_com_position(const int* m,const float* p,
    const float* state,float* output,unsigned nb,unsigned nj,unsigned nv,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const int* parent=m; const int* type=m+nb; const int* body=type+nj;
  const int* adr=body+nj;
#ifdef MJWARP_FIELD_BATCHES
  const int* desc=adr+nj;
  const float* mass=field_parameter(p,desc,w,nb);
  const float* inertia=field_parameter(p,desc+2,w,3*nb);
#else
  const float* mass=p; const float* inertia=p+nb;
#endif
  const float* s=state+4+(unsigned long long)w*(28*nb+6*nj);
  const float* xmat=s+7*nb; const float* ipos=s+16*nb;
  const float* imat=s+19*nb; const float* anchor=s+28*nb;
  const float* axis=anchor+3*nj;
  float* sm=output+4+(unsigned long long)w*(14*nb+6*nv);
  float* com=sm+nb; float* ci=com+3*nb; float* cd=ci+10*nb;
  for(unsigned b=0;b<nb;b++) {
    sm[b]=mass[b];
    for(int a=0;a<3;a++) com[3*b+a]=mass[b]*ipos[3*b+a];
  }
  for(int b=int(nb)-1;b>0;b--) {
    int up=parent[b]; sm[up]+=sm[b];
    for(int a=0;a<3;a++) com[3*up+a]+=com[3*b+a];
  }
  for(unsigned b=0;b<nb;b++) if(sm[b]!=0.0f)
    for(int a=0;a<3;a++) com[3*b+a]/=sm[b];
  for(unsigned b=0;b<nb;b++) {
    unsigned root=b; while(parent[root]!=0) root=parent[root];
    float x=ipos[3*b]-com[3*root], y=ipos[3*b+1]-com[3*root+1];
    float z=ipos[3*b+2]-com[3*root+2], mb=mass[b];
    const float* r=imat+9*b; const float* d=inertia+3*b;
    float t[9];
    for(int row=0;row<3;row++) for(int col=0;col<3;col++) {
      float v=0.0f;
      for(int a=0;a<3;a++) v+=r[3*row+a]*d[a]*r[3*col+a];
      t[3*row+col]=v;
    }
    float* c=ci+10*b;
    c[0]=t[0]+mb*(y*y+z*z); c[1]=t[4]+mb*(x*x+z*z);
    c[2]=t[8]+mb*(x*x+y*y); c[3]=t[1]-mb*x*y;
    c[4]=t[2]-mb*x*z; c[5]=t[5]-mb*y*z;
    c[6]=mb*x; c[7]=mb*y; c[8]=mb*z; c[9]=mb;
  }
  for(unsigned j=0;j<nj;j++) {
    unsigned b=body[j],root=b; while(parent[root]!=0) root=parent[root];
    float off[3]; for(int a=0;a<3;a++) off[a]=com[3*root+a]-anchor[3*j+a];
    unsigned d=adr[j]; int ty=type[j];
    if(ty==0) {
      for(int a=0;a<3;a++) {
        float* c=cd+6*(d+a);
        for(int k=0;k<6;k++) c[k]=0.0f;
        c[3+a]=1.0f;
      }
      d+=3;
    }
    int count=(ty<=1)?3:1;
    for(int a=0;a<count;a++) {
      float ax[3]; for(int k=0;k<3;k++) ax[k]=(ty<=1)?xmat[9*b+3*k+a]:axis[3*j+k];
      float* c=cd+6*(d+a);
      if(ty==2) {
        for(int k=0;k<3;k++) { c[k]=0.0f; c[3+k]=ax[k]; }
      } else {
        for(int k=0;k<3;k++) c[k]=ax[k];
        c[3]=ax[1]*off[2]-ax[2]*off[1];
        c[4]=ax[2]*off[0]-ax[0]*off[2];
        c[5]=ax[0]*off[1]-ax[1]*off[0];
      }
    }
  }
}
