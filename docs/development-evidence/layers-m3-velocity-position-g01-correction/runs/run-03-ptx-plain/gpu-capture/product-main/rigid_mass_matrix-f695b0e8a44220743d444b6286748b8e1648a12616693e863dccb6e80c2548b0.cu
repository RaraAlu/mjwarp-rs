
extern "C" __global__ void rigid_mass_matrix(const int* m,const float* p,
    const float* state,float* output,unsigned nb,unsigned nv,unsigned unused,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const int* parent=m; const int* body=m+nb; const int* dparent=body+nv;
  const float* s=state+4+(unsigned long long)w*(14*nb+6*nv);
  const float* ci=s+4*nb; const float* cd=ci+10*nb;
  float* crb=output+4+(unsigned long long)w*(10*nb+nv*nv);
  float* mat=crb+10*nb;
  for(unsigned a=0;a<10*nb;a++) crb[a]=ci[a];
  // Root subtrees use different COM frames. Never accumulate into world.
  for(int b=int(nb)-1;b>0;b--) if(parent[b]>0)
    for(int a=0;a<10;a++) crb[10*parent[b]+a]+=crb[10*b+a];
  for(unsigned a=0;a<nv*nv;a++) mat[a]=0.0f;
  for(unsigned d=0;d<nv;d++) {
    const float* i=crb+10*body[d]; const float* v=cd+6*d;
    float buf[6]={
      i[0]*v[0]+i[3]*v[1]+i[4]*v[2]-i[8]*v[4]+i[7]*v[5],
      i[3]*v[0]+i[1]*v[1]+i[5]*v[2]+i[8]*v[3]-i[6]*v[5],
      i[4]*v[0]+i[5]*v[1]+i[2]*v[2]-i[7]*v[3]+i[6]*v[4],
      i[8]*v[1]-i[7]*v[2]+i[9]*v[3],
      i[6]*v[2]-i[8]*v[0]+i[9]*v[4],
      i[7]*v[0]-i[6]*v[1]+i[9]*v[5]};
    for(int j=int(d);j>=0;j=dparent[j]) {
      // Frozen _M finishes wp.dot before adding the diagonal armature.
      // Seeding the dot with armature changes the f32 rounding sequence.
      float value=cd[6*j]*buf[0];
      for(int a=1;a<6;a++) value+=cd[6*j+a]*buf[a];
      if(j==int(d)) value=__fadd_rn(value,p[d]);
      mat[d*nv+j]=value; mat[j*nv+d]=value;
    }
  }
}
