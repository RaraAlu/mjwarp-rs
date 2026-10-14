
extern "C" __global__ void resident_mixed_validate(const int* meta,const float* source,
 const float* unused,float* status,unsigned nb,unsigned n,unsigned nc,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 unsigned long long stride=10ull*nb+1ull*n*n,total=stride*worlds;bool valid=true;
 for(unsigned j=0;j<4;++j)valid=valid && source[j]==-131072.0f && source[4+total+j]==-131072.0f;
 const float* value=source+4+1ull*w*stride;
 for(unsigned long long j=0;j<stride;++j)valid=valid && isfinite(value[j]);
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void resident_mixed_gather(const int* meta,const float* source,
 const float* unused,float* csr,unsigned nb,unsigned n,unsigned nc,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,row=tid%n;const int* rownnz=meta;const int* rowadr=meta+n;const int* colind=meta+2*n;
 const float* dense=source+4+1ull*w*(10ull*nb+1ull*n*n)+10ull*nb;
 float* output=csr+4+1ull*w*nc;
 for(int j=0;j<rownnz[row];++j){int a=rowadr[row]+j;output[a]=dense[1ull*row*n+colind[a]];}
}
