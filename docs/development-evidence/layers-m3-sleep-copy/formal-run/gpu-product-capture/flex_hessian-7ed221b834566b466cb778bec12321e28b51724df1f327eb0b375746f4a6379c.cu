
__device__ void hessian_endpoints(int dim,int a,int* i,int* j) {
  const int tri[6]={1,2,2,0,0,1};
  const int tet[12]={0,1,1,2,2,0,2,3,0,3,1,3};
  *i=dim==2?tri[2*a]:tet[2*a];*j=dim==2?tri[2*a+1]:tet[2*a+1];
}
__device__ float hessian_sign(int v,int a,int dim) {
  int i,j;hessian_endpoints(dim,a,&i,&j);return v==i?1.0f:(v==j?-1.0f:0.0f);
}
extern "C" __global__ void flex_hessian(const int* m,const float* p,const float* pos,
    const float* edges,const int* valid,float* result,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0],nv=m[1],ne=m[2];
  const float* vx=pos+4+(unsigned long long)w*m[3]+m[8];
  const float* length=edges+4+(unsigned long long)w*m[4];
  const float* stiffness=p+ne;const int* flags=valid+4+(unsigned long long)w*nf;
  float* diag=result+4+(unsigned long long)w*(6ull*nv+9ull*ne);float* blocks=diag+6*nv;
  for(int f=0;f<nf;++f) {
    if(flags[f]==1) continue;
    const int* r=m+9+11*f;int dim=r[0],va=r[2],ea=r[4];
    for(int v=0;v<r[3];++v) for(int k=0;k<6;++k) diag[6*(va+v)+k]=0.0f;
    for(int e=0;e<r[5];++e) for(int k=0;k<9;++k) blocks[9*(ea+e)+k]=0.0f;
    if(!r[1]) continue;
    int n=dim==2?3:6;
    for(int el=0;el<r[7];++el) {
      const int* v=m+m[5]+r[8]+el*(dim+1);
      const int* ids=m+m[6]+r[9]+el*n;
      float x[18],elong[6],metric[36]={0},tension[6]={0};
      for(int a=0;a<n;++a) {
        int i,j;hessian_endpoints(dim,a,&i,&j);
        for(int k=0;k<3;++k) x[3*a+k]=vx[3*(va+v[i])+k]-vx[3*(va+v[j])+k];
        int e=ea+ids[a];elong[a]=length[e]*length[e]-p[e]*p[e];
      }
      int idx=0;const float* s=stiffness+r[10]+21*el;
      for(int a=0;a<n;++a) for(int b=a;b<n;++b) {metric[6*a+b]=metric[6*b+a]=s[idx++];}
      for(int a=0;a<n;++a) {
        for(int b=0;b<n;++b) tension[a]+=metric[6*a+b]*elong[b];
        tension[a]=fmaxf(tension[a],0.0f);
      }
      for(int e=0;e<n;++e) {
        int i,j;hessian_endpoints(dim,e,&i,&j);
        int edge=ea+ids[e];const int* endpoints=m+m[7]+2*edge;
        if(endpoints[0]!=v[i]) {int swap=i;i=j;j=swap;}
        float block[9]={0};
        for(int a=0;a<n;++a) {
          float sa=hessian_sign(i,a,dim);if(sa==0.0f) continue;
          for(int b=0;b<n;++b) {
            float sb=hessian_sign(j,b,dim);if(sb==0.0f) continue;
            float weight=2.0f*metric[6*a+b]*sa*sb;
            for(int row=0;row<3;++row) for(int col=0;col<3;++col)
              block[3*row+col]+=weight*x[3*a+row]*x[3*b+col];
          }
        }
        float geo=0.0f;
        for(int a=0;a<n;++a) geo+=tension[a]*hessian_sign(i,a,dim)*hessian_sign(j,a,dim);
        block[0]+=geo;block[4]+=geo;block[8]+=geo;
        for(int k=0;k<9;++k) blocks[9*edge+k]+=block[k];
      }
    }
    for(int e=0;e<r[5];++e) {
      int edge=ea+e;const int* endpoints=m+m[7]+2*edge;
      const float* b=blocks+9*edge;float* d0=diag+6*(va+endpoints[0]);float* d1=diag+6*(va+endpoints[1]);
      const int upper[6]={0,1,2,4,5,8},lower[6]={0,3,6,4,7,8};
      for(int k=0;k<6;++k) {d0[k]-=b[upper[k]];d1[k]-=b[lower[k]];}
    }
  }
}
extern "C" __global__ void flex_hessian_validate(const int* m,const float* p,const float* h,
    int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nf=m[0];int* flags=result+4+(unsigned long long)w*nf;
  for(int f=0;f<nf;++f) flags[f]=1;
}
