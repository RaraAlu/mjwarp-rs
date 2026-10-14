#define FP_NV 6
#define FP_NE 5
#define FP_NF 3
#define FP_NNZ 24
#define FP_STRIDE 34
#define FP_S 0
#define FP_D 0

extern "C" __global__ void flex_edge_passive_validate(const int* m,const float* p,const float* source,float* status,unsigned ne,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=ne==FP_NE&&nv==FP_NV;
 const unsigned mend=4+4*FP_NE+FP_NNZ,pend=4+FP_NE+2*FP_NF;const unsigned long long send=4ull+(unsigned long long)worlds*FP_STRIDE;
 for(unsigned k=0;k<4;++k)ok=ok&&m[k]==-131072&&m[mend+k]==-131072&&p[k]==-131072.0f&&p[pend+k]==-131072.0f&&source[k]==-131072.0f&&source[send+k]==-131072.0f;
 const int* rowadr=m+4;const int* rownnz=rowadr+FP_NE;const int* colind=rownnz+FP_NE;const int* flexid=colind+FP_NNZ;const int* rigid=flexid+FP_NE;
 const float* length0=p+4;const float* stiffness=length0+FP_NE;const float* damping=stiffness+FP_NF;const float* edge=source+4+(unsigned long long)w*FP_STRIDE;
 for(unsigned e=0;e<FP_NE;++e){int adr=rowadr[e],n=rownnz[e],f=flexid[e];if(adr<0||n<0||adr>FP_NNZ||n>FP_NNZ-adr||f<0||f>=FP_NF){ok=false;continue;}if(rigid[e]!=0&&rigid[e]!=1)ok=false;if(rigid[e])continue;
  float s=0.0f,d=0.0f;if(!FP_S){s=stiffness[f];ok=ok&&isfinite(s);}if(!FP_D){d=damping[f];ok=ok&&isfinite(d);}if(s==0.0f&&d==0.0f)continue;if(n==0)continue;
  float fs=0.0f,fd=0.0f;if(s!=0.0f){float r=length0[e],x=edge[e];ok=ok&&isfinite(r)&&isfinite(x);fs=s*(r-x);ok=ok&&isfinite(fs);}if(d!=0.0f){float v=edge[FP_NE+e];ok=ok&&isfinite(v);fd=-d*v;ok=ok&&isfinite(fd);}if(fs==0.0f&&fd==0.0f)continue;
  for(int i=0;i<n;++i){int c=colind[adr+i];if(c<0||c>=FP_NV)ok=false;ok=ok&&isfinite(edge[2*FP_NE+adr+i]);}
 }
 status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void flex_edge_passive_clear(const int* m,const float* p,const float* source,float* out,unsigned a,unsigned b,unsigned worlds,unsigned work){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<work)out[4+i]=0.0f;}
extern "C" __global__ void flex_edge_passive_compute(const int* m,const float* p,const float* source,float* out,unsigned ne,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned e=tid%FP_NE,w=tid/FP_NE;const int* rowadr=m+4;const int* rownnz=rowadr+FP_NE;const int* colind=rownnz+FP_NE;const int* flexid=colind+FP_NNZ;const int* rigid=flexid+FP_NE;if(rigid[e])return;int f=flexid[e];const float* length0=p+4;const float* stiffness=length0+FP_NE;const float* damping=stiffness+FP_NF;float s=FP_S?0.0f:stiffness[f],d=FP_D?0.0f:damping[f];if(s==0.0f&&d==0.0f)return;int adr=rowadr[e],n=rownnz[e];if(n==0)return;const float* edge=source+4+(unsigned long long)w*FP_STRIDE;float fs=0.0f,fd=0.0f;if(s!=0.0f)fs=s*(length0[e]-edge[e]);if(d!=0.0f)fd=-d*edge[FP_NE+e];if(fs==0.0f&&fd==0.0f)return;for(int i=0;i<n;++i){int c=colind[adr+i];float j=edge[2*FP_NE+adr+i];if(fs!=0.0f)atomicAdd(&out[4+(unsigned long long)w*FP_NV+c],j*fs);if(fd!=0.0f)atomicAdd(&out[4+(unsigned long long)worlds*FP_NV+(unsigned long long)w*FP_NV+c],j*fd);}
}
