
extern "C" __global__ void tendon_values(const int* m,const int* spatial_ids,
    const float* state,float* result,unsigned kind,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nz=m[1],nw=m[2],ft=m[3],fz=m[4],st=m[5],sz=m[6],sw=m[7];
  const int* ids=spatial_ids+4+(unsigned long long)w*(2ull*st+2ull*sw);
  const float* in=state+4+(unsigned long long)w*(kind==0?ft+fz:st+sz+19ull*sw);
  float* out=result+4+(unsigned long long)w*(nt+nz+6ull*nw);
  if(kind==0) for(int i=0;i<nt+nz+6*nw;++i) out[i]=0.0f;
  int cursor=0;
  for(int t=0;t<nt;++t) {
    const int* row=m+8+5*t;int local=row[1];
    if(row[0]==(int)kind) {
      out[t]=in[local];
      for(int j=0;j<row[4];++j) out[nt+row[3]+j]=in[(kind==0?ft:st)+row[2]+j];
      if(kind==1) {
        int count=ids[st+local];
        for(int j=0;j<3*count;++j) out[nt+nz+3*cursor+j]=in[st+sz+3*ids[local]+j];
      }
    }
    if(row[0]==1) cursor+=ids[st+local];
  }
}
extern "C" __global__ void tendon_indices(const int* m,const float* unused,
    const int* state,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[2],st=m[5],sw=m[7];
  const int* in=state+4+(unsigned long long)w*(2ull*st+2ull*sw);
  int* out=result+4+(unsigned long long)w*(2ull*nt+2ull*nw);
  for(int i=0;i<2*nt+2*nw;++i) out[i]=0;
  int cursor=0;
  for(int t=0;t<nt;++t) {
    const int* row=m+8+5*t;
    // Fixed rows share the current compact cursor but add no points.
    out[t]=cursor;
    if(row[0]==0) continue;
    int local=row[1],count=in[st+local];out[nt+t]=count;
    for(int j=0;j<count;++j) out[2*nt+cursor+j]=in[2*st+in[local]+j];
    cursor+=count;
  }
}
