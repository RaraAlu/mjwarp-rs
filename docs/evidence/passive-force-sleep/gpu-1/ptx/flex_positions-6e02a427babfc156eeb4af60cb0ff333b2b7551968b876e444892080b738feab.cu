
// Frozen smooth.flex clears every (world, flex) before any position work.
// Private guarded i32 slots encode false=0 and true=1; this is not Warp's bool ABI.
extern "C" __global__ void flex_invalidate(const int* m,const float* params,
    const float* rigid,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0];int* flags=result+4+(unsigned long long)w*nf;
  for(int f=0;f<nf;++f) flags[f]=0;
}
__device__ void flex_transform(float* out,const float* local,const float* xp,
    const float* xm,int body,bool centered) {
  const float* p=xp+3*body;const float* r=xm+9*body;
  for(int k=0;k<3;++k) out[k]=centered?p[k]:
    ((r[3*k]*local[0]+r[3*k+1]*local[1])+r[3*k+2]*local[2])+p[k];
}
extern "C" __global__ void flex_positions(const int* m,const float* params,
    const float* rigid,float* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0],nn=m[1],nv=m[2],nb=m[3];
  const float* xp=rigid+4+(unsigned long long)w*m[4];const float* xm=xp+7*nb;
  const float* verts=params;const float* coords=verts+3*nv;const float* nodes=coords+3*nv;
  const int* nodebody=m+5+9*nf;const int* vertbody=nodebody+nn;
  float* nx=result+4+(unsigned long long)w*(3ull*nn+3ull*nv);float* vx=nx+3*nn;
  for(int f=0;f<nf;++f) {
    const int* row=m+5+9*f;
    for(int j=0;j<row[5];++j) {
      int i=row[4]+j;const float* local=nodes+3*i;
      bool zero=local[0]==0.0f&&local[1]==0.0f&&local[2]==0.0f;
      flex_transform(nx+3*i,local,xp,xm,nodebody[i],row[8]!=0||zero);
    }
  }
  for(int f=0;f<nf;++f) {
    const int* row=m+5+9*f;
    for(int j=0;j<row[7];++j) {
      int v=row[6]+j;
      if(row[0]==0) { flex_transform(vx+3*v,verts+3*v,xp,xm,vertbody[v],row[8]!=0);continue; }
      int cell[3];float local[3];
      for(int k=0;k<3;++k) {
        float coord=coords[3*v+k]*(float)row[1+k];
        cell[k]=max(0,min((int)coord,row[1+k]-1));
        local[k]=fminf(1.0f,fmaxf(0.0f,coord-(float)cell[k]));
      }
      float p[3]={0,0,0};int ny=row[2]+1,nz=row[3]+1;
      for(int i=0;i<2;++i) for(int j=0;j<2;++j) for(int k=0;k<2;++k) {
        float weight=(i?local[0]:1-local[0])*(j?local[1]:1-local[1])*(k?local[2]:1-local[2]);
        int n=row[4]+(cell[0]+i)*ny*nz+(cell[1]+j)*nz+cell[2]+k;
        for(int d=0;d<3;++d) p[d]+=weight*nx[3*n+d];
      }
      for(int d=0;d<3;++d) vx[3*v+d]=p[d];
    }
  }
}
