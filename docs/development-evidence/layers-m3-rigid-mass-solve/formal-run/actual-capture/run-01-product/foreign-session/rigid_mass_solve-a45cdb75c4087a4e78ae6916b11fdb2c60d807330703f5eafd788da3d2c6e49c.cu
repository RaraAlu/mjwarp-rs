
__device__ float tile_schur_update(float value,float a,float b) {
  float result;
  // Match the frozen MathDx PTX, including its permission to contract.
  // Explicit .rn on mul/sub would forbid the driver's FMA contraction.
  asm("{ .reg .f32 product; mul.f32 product,%2,%3; sub.f32 %0,%1,product; }"
      : "=f"(result) : "f"(value), "f"(a), "f"(b));
  return result;
}
extern "C" __global__ void rigid_mass_solve(const int* blocks,const float* rhs,
    const float* state,float* output,unsigned nb,unsigned n,unsigned nr,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const float* mat=state+4+(unsigned long long)w*(10*nb+n*n)+10*nb;
  float* status=output+4+(unsigned long long)w*(3+n*n+n+nr*n);
  float* ld=status+3; float* inv=ld+n*n; float* x=inv+n;
  status[0]=0.0f; status[1]=-1.0f; status[2]=0.0f;
  for(unsigned i=0;i<n*n;i++) ld[i]=0.0f;
  for(unsigned i=0;i<n;i++) inv[i]=0.0f;
  for(int block=0;block<blocks[0];block++) {
    int begin=blocks[1+3*block], end=begin+blocks[2+3*block], kind=blocks[3+3*block];
    if(kind==0) {
      float pivot=mat[begin*n+begin];
      if(!(pivot>0.0f) || !isfinite(pivot)) {
        status[0]=1.0f; status[1]=float(begin); status[2]=pivot; return;
      }
      inv[begin]=1.0f/pivot;
      if(!isfinite(inv[begin])) {
        status[0]=2.0f; status[1]=float(begin); status[2]=inv[begin]; return;
      }
    } else if(kind==1) {
      // Frozen MathDx tile PTX uses rsqrt.approx plus multiply.
      // Preserve the mul/sub contract for each Schur update.
      // The small scalar route (through six DOFs) retains sqrt/divide/FMA.
      bool tile=end-begin>6;
      for(int i=begin;i<end;i++) {
        float pivot=mat[i*n+i];
        for(int k=begin;k<i;k++) {
          float u=ld[k*n+i];
          pivot=tile ? tile_schur_update(pivot,u,u) : fmaf(-u,u,pivot);
        }
        if(!(pivot>0.0f) || !isfinite(pivot)) {
          status[0]=1.0f; status[1]=float(i); status[2]=pivot; return;
        }
        float reciprocal, diagonal;
        if(tile) {
          asm("rsqrt.approx.f32 %0, %1;" : "=f"(reciprocal) : "f"(pivot));
          diagonal=__fmul_rn(pivot,reciprocal);
        } else {
          diagonal=sqrtf(pivot);
          reciprocal=1.0f/diagonal;
        }
        ld[i*n+i]=diagonal;
        for(int j=i+1;j<end;j++) {
          float value=mat[i*n+j];
          for(int k=begin;k<i;k++) {
            float u=ld[k*n+i], v=ld[k*n+j];
            value=tile ? tile_schur_update(value,u,v) : fmaf(-u,v,value);
          }
          ld[i*n+j]=__fmul_rn(value,reciprocal);
        }
      }
    } else {
    for(int i=begin;i<end;i++) for(int j=begin;j<=i;j++) ld[i*n+j]=mat[i*n+j];
    for(int k=end-1;k>=begin;k--) {
    float pivot=ld[k*n+k];
    if(!(pivot>0.0f) || !isfinite(pivot)) {
      status[0]=1.0f; status[1]=float(k); status[2]=pivot; return;
    }
    float reciprocal=1.0f/pivot; inv[k]=reciprocal;
    if(!isfinite(reciprocal) || reciprocal>3.4028234663852886e38f) {
      status[0]=2.0f; status[1]=float(k); status[2]=reciprocal; return;
    }
    // Keep the original row until every preceding row update completes.
    for(int i=k-1;i>=begin;i--) {
      float tmp=ld[k*n+i]*reciprocal;
      for(int j=begin;j<=i;j++) ld[i*n+j]-=ld[k*n+j]*tmp;
    }
    for(int j=begin;j<k;j++) ld[k*n+j]*=reciprocal;
    }
    }
  }
  if(n==0) return;
  const float* b=rhs+(unsigned long long)w*nr*n;
  for(unsigned r=0;r<nr;r++) {
    float* v=x+r*n;
    for(unsigned i=0;i<n;i++) v[i]=b[r*n+i];
    for(int block=0;block<blocks[0];block++) {
      int begin=blocks[1+3*block], end=begin+blocks[2+3*block], kind=blocks[3+3*block];
      if(kind==0) {
        v[begin]*=inv[begin];
      } else if(kind==1) {
        for(int i=begin;i<end;i++) {
          float value=v[i];
          for(int j=begin;j<i;j++) value-=ld[j*n+i]*v[j];
          v[i]=value/ld[i*n+i];
        }
        for(int i=end-1;i>=begin;i--) {
          float value=v[i];
          for(int j=i+1;j<end;j++) value-=ld[i*n+j]*v[j];
          v[i]=value/ld[i*n+i];
        }
      } else {
    // 反向LDL块依次执行L^-T、D^-1与L^-1。
    for(int k=end-1;k>=begin;k--)
      for(int i=begin;i<k;i++) v[i]-=ld[k*n+i]*v[k];
    for(int i=begin;i<end;i++) v[i]*=inv[i];
    for(int i=begin;i<end;i++) {
      float sum=0.0f;
      for(int j=begin;j<i;j++) sum+=ld[i*n+j]*v[j];
      v[i]-=sum;
    }
      }
    }
  }
}
