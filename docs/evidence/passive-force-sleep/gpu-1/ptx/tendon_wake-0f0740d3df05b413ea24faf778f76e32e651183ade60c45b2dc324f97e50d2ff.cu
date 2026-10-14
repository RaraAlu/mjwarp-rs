
extern "C" __global__ void sleep_copy(const int* m,const float* params,
    const int* state,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  unsigned long long width=2ull*m[0]+3,offset=4+(unsigned long long)w*width;
  for(unsigned long long i=0;i<width;++i) result[offset+i]=state[offset+i];
}
__device__ void tendon_wake_tree(int* asleep,int ntree,int tree,int wakeval) {
  int old=asleep[tree];
  if(old<0) { if(wakeval<old) asleep[tree]=wakeval;return; }
  int current=tree;
  for(int step=0;step<=ntree;++step) {
    int next=asleep[current];if(next<0||next>=ntree) break;
    asleep[current]=wakeval;current=next;if(current==tree) break;
  }
}
extern "C" __global__ void tendon_wake(const int* m,const float* params,
    const float* lengths,int* result,unsigned a,unsigned b,unsigned c,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int n=m[0],nt=m[1];if(!m[6]||nt==0) return;
  int* asleep=result+4+(unsigned long long)w*(2ull*n+3);int* awake=asleep+n;
  const int* adr=m+7;const int* num=adr+nt;const int* limited=num+nt;const int* trees=limited+nt;
  const float* range=params+(unsigned long long)(w%m[3])*2*nt;
  const float* margin=params+2ull*nt*m[3]+(unsigned long long)(w%m[4])*nt;
  const float* length=lengths+4+(unsigned long long)w*m[5];
  for(int t=0;t<nt;++t) {
    int any_awake=0,wakeval=-11;
    for(int i=0;i<num[t];++i) {
      int tree=trees[adr[t]+i];if(tree>=0&&awake[tree]==1) {
        any_awake=1;if(asleep[tree]<wakeval) wakeval=asleep[tree];
      }
    }
    if(any_awake&&limited[t]&&
       (length[t]-range[2*t]<margin[t]||range[2*t+1]-length[t]<margin[t])) {
      for(int i=0;i<num[t];++i) {
        int tree=trees[adr[t]+i];if(tree>=0&&awake[tree]==0) tendon_wake_tree(asleep,n,tree,wakeval);
      }
    }
  }
  int count=0;for(int i=0;i<n;++i) { awake[i]=asleep[i]<0;count+=awake[i]; }
  awake[n]=count;awake[n+1]=0;awake[n+2]=0;
}
