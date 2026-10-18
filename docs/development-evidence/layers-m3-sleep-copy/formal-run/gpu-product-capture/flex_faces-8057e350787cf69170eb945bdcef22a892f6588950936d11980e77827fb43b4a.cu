
__device__ float face_dot(const float* a,const float* b) {
  return (a[0]*b[0]+a[1]*b[1])+a[2]*b[2];
}
__device__ void face_cross(float* r,const float* a,const float* b) {
  r[0]=a[1]*b[2]-a[2]*b[1];r[1]=a[2]*b[0]-a[0]*b[2];r[2]=a[0]*b[1]-a[1]*b[0];
}
__device__ void face_polar(float* q,const float F[3][3]) {
  q[0]=q[1]=q[2]=0.0f;q[3]=1.0f;
  for(int iter=0;iter<50;++iter) {
    // Warp quat_to_matrix rotates each basis vector without renormalizing.
    float R[3][3];
    for(int col=0;col<3;++col) {
      float v[3]={0,0,0};v[col]=1.0f;
      float c=2.0f*q[3]*q[3]-1.0f;
      float d=2.0f*(q[0]*v[0]+q[1]*v[1]+q[2]*v[2]);
      R[col][0]=v[0]*c+q[0]*d+(q[1]*v[2]-q[2]*v[1])*q[3]*2.0f;
      R[col][1]=v[1]*c+q[1]*d+(q[2]*v[0]-q[0]*v[2])*q[3]*2.0f;
      R[col][2]=v[2]*c+q[2]*d+(q[0]*v[1]-q[1]*v[0])*q[3]*2.0f;
    }
    float crosses[3][3];
    for(int c=0;c<3;++c) face_cross(crosses[c],R[c],F[c]);
    float omega[3];float denom=fabsf((face_dot(R[0],F[0])+face_dot(R[1],F[1]))+face_dot(R[2],F[2]))+1.0e-10f;
    for(int k=0;k<3;++k) omega[k]=((crosses[0][k]+crosses[1][k])+crosses[2][k])/denom;
    float w=sqrtf(face_dot(omega,omega));if(w<1.0e-6f) break;
    float rot[4];for(int k=0;k<3;++k) rot[k]=(omega[k]/w)*sinf(0.5f*w);rot[3]=cosf(0.5f*w);
    // Loaded frozen Warp SASS keeps the first product rounded and fuses all three sums.
    float next[4];
    next[0]=fmaf(-q[1],rot[2],fmaf(q[2],rot[1],fmaf(q[3],rot[0],__fmul_rn(q[0],rot[3]))));
    next[1]=fmaf(-q[2],rot[0],fmaf(q[0],rot[2],fmaf(q[3],rot[1],__fmul_rn(q[1],rot[3]))));
    next[2]=fmaf(-q[0],rot[1],fmaf(q[1],rot[0],fmaf(q[3],rot[2],__fmul_rn(q[2],rot[3]))));
    next[3]=rot[3]*q[3]-rot[0]*q[0]-rot[1]*q[1]-rot[2]*q[2];
    float n=sqrtf(((next[0]*next[0]+next[1]*next[1])+next[2]*next[2])+next[3]*next[3]);
    // Warp kEps is zero; nonpositive lengths return the identity.
    if(n>0.0f) {float inv_n=1.0f/n;for(int k=0;k<4;++k) q[k]=next[k]*inv_n;}
    else {q[0]=q[1]=q[2]=0.0f;q[3]=1.0f;}
  }
}
extern "C" __global__ void flex_faces(const int* m,const float* params,const float* nodes,
    float* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0];const float* nx=nodes+4+(unsigned long long)w*m[1];
  float* positions=result+4+(unsigned long long)w*31ull*nf;float* quats=positions+27*nf;
  for(int f=0;f<nf;++f) {
    const int* row=m+2+10*f;float t1[3]={0,0,0},t2[3]={0,0,0};
    for(int slot=0;slot<9;++slot) {
      int n=row[1+slot];float* out=positions+27*f+3*slot;
      if(n<0) {out[0]=out[1]=out[2]=0.0f;continue;}
      float g0=0.5f*(-1+2*(slot/2)),g1=0.5f*(-1+2*(slot%2));
      for(int k=0;k<3;++k) {out[k]=nx[3*n+k];t1[k]+=out[k]*g0;t2[k]+=out[k]*g1;}
    }
    float normal[3];face_cross(normal,t1,t2);float F[3][3];
    for(int k=0;k<3;++k) {
      if(row[0]==0) {F[0][k]=normal[k];F[1][k]=t1[k];F[2][k]=t2[k];}
      else if(row[0]==1) {F[0][k]=t2[k];F[1][k]=normal[k];F[2][k]=t1[k];}
      else {F[0][k]=t1[k];F[1][k]=t2[k];F[2][k]=normal[k];}
    }
    face_polar(quats+4*f,F);
  }
}
