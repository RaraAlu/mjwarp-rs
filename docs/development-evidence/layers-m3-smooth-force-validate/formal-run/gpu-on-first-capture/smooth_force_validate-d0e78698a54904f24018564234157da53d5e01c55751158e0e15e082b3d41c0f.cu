#define SMOOTH_SLEEP 1

extern "C" __global__ void smooth_force_validate(const int* meta,const float* forces,
 const int* sleep,float* status,unsigned nb,unsigned n,unsigned nt,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long total=1ull*worlds*n,mlen=1ull*nb+n+1ull*worlds*nt;bool valid=true;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+mlen+j]==-131072&&forces[j]==-131072.0f&&forces[4+4*total+j]==-131072.0f;
 const int* tree=meta+4;const int* body=tree+nb;const int* awake=body+n;unsigned long long stride=nt;
 #if SMOOTH_SLEEP
 if(sleep!=meta){unsigned long long sw=2ull*nt+3,sl=1ull*worlds*sw;for(unsigned j=0;j<4;++j)valid=valid&&sleep[j]==-131072&&sleep[4+sl+j]==-131072;awake=sleep+4+nt;stride=sw;}
 #endif
 for(unsigned b=0;b<nb;++b)if(tree[b]<-1||(tree[b]>=0&&(unsigned)tree[b]>=nt))valid=false;
 for(unsigned i=0;i<n;++i){int b=body[i];if(b<0||(unsigned)b>=nb){valid=false;continue;}
  int t=tree[b];if(t<-1||(t>=0&&(unsigned)t>=nt)){valid=false;continue;}
  #if SMOOTH_SLEEP
  if(t>=0&&awake[1ull*w*stride+t]==0)continue;
  #endif
  for(unsigned f=0;f<4;++f)valid=valid&&isfinite(forces[4+f*total+1ull*w*n+i]);
 }
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void smooth_force(const int* meta,const float* forces,
 const int* sleep,float* output,unsigned nb,unsigned n,unsigned nt,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,i=tid%n;const int* tree=meta+4;const int* body=tree+nb;const int* awake=body+n;
 #if SMOOTH_SLEEP
 unsigned long long stride=nt;if(sleep!=meta){awake=sleep+4+nt;stride=2ull*nt+3;}
 int t=tree[body[i]];if(t>=0&&awake[1ull*w*stride+t]==0){output[4+tid]=0.0f;return;}
 #endif
 const float* f=forces+4+tid;float acc=__fsub_rn(f[0],f[work]);
 acc=__fadd_rn(acc,f[2ull*work]);output[4+tid]=__fadd_rn(acc,f[3ull*work]);
}
