
__device__ bool edge_ancestor(int b,int owner,const int* parent) {
  while(b>0) {if(b==owner) return true;b=parent[b];}return false;
}
extern "C" __global__ void flex_edges(const int* m,const float* qvel,
    const float* com,const float* flex,float* out,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nb=m[0],nv=m[1],ne=m[2],nnz=m[3],nvert=m[6];
  const int* rows=m+7;const int* cols=rows+4*ne;const int* vb=cols+nnz;
  const int* parent=vb+nvert;const int* root=parent+nb;const int* db=root+nb;
  const float* cp=com+4+(unsigned long long)w*m[4];const float* sc=cp+nb;
  const float* cdof=cp+14*nb;const float* vx=flex+4+(unsigned long long)w*m[5]+m[5]-3*nvert;
  const float* qv=qvel+4+(unsigned long long)w*nv;
  float* len=out+4+(unsigned long long)w*(2ull*ne+nnz);float* vel=len+ne;float* jac=vel+ne;
  for(int i=0;i<nnz;++i) jac[i]=0.0f;
  for(int e=0;e<ne;++e) {
    const int* row=rows+4*e;int v1=row[0],v2=row[1],adr=row[2],num=row[3];
    float dir[3];for(int k=0;k<3;++k) dir[k]=vx[3*v2+k]-vx[3*v1+k];
    float norm=sqrtf((dir[0]*dir[0]+dir[1]*dir[1])+dir[2]*dir[2]);len[e]=norm;
    if(norm!=0.0f) for(int k=0;k<3;++k) dir[k]/=norm;
    vel[e]=0.0f;int b1=vb[v1],b2=vb[v2];if(b1<0||b2<0||num==0) continue;
    float off[2][3];int bodies[2]={b1,b2};
    for(int s=0;s<2;++s) for(int k=0;k<3;++k)
      off[s][k]=vx[3*(s?v2:v1)+k]-sc[3*root[bodies[s]]+k];
    for(int i=0;i<num;++i) {
      int d=cols[adr+i];const float* cd=cdof+6*d;float j=0.0f;
      for(int s=0;s<2;++s) if(edge_ancestor(bodies[s],db[d],parent)) {
        const float* o=off[s];float p[3]={cd[3]+(cd[1]*o[2]-cd[2]*o[1]),
          cd[4]+(cd[2]*o[0]-cd[0]*o[2]),cd[5]+(cd[0]*o[1]-cd[1]*o[0])};
        float dot=(p[0]*dir[0]+p[1]*dir[1])+p[2]*dir[2];j+=s?dot:-dot;
      }
      jac[adr+i]=j;vel[e]+=j*qv[d];
    }
  }
}
