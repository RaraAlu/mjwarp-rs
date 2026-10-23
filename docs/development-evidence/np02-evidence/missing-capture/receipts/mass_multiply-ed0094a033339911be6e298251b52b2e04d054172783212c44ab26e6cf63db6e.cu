#define MASS_NV 86
#define MASS_DENSE 1

extern "C" __global__ void mass_multiply_validate(const int* meta,const float* M,
 const float* vec,float* status,unsigned n,unsigned mc,unsigned so,unsigned worlds) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;
 bool valid=true;for(unsigned j=0;j<4;++j)valid=valid&&M[j]==-131072.0f&&M[4+1ull*worlds*mc+j]==-131072.0f&&vec[j]==-131072.0f&&vec[4+1ull*worlds*n+j]==-131072.0f;
 if(!meta[so+w]){for(unsigned j=0;j<mc;++j)valid=valid&&isfinite(M[4+1ull*w*mc+j]);for(unsigned j=0;j<n;++j)valid=valid&&isfinite(vec[4+1ull*w*n+j]);}
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void mass_multiply(const int* meta,const float* M,
 const float* vec,float* result,unsigned n,unsigned mc,unsigned so,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned w=tid/n,row=tid%n;if(meta[so+w])return;
 const float* matrix=M+4+1ull*w*mc;const float* v=vec+4+1ull*w*n;float acc=0.0f;
 #if MASS_DENSE
 #pragma unroll
 for(unsigned j=0;j<MASS_NV;++j)acc=acc+matrix[row*MASS_NV+j]*v[j];
 #else
 const int* col=meta+n+1;const int* adr=col+meta[n];
 for(int k=meta[row];k<meta[row+1];++k)acc=acc+matrix[adr[k]]*v[col[k]];
 #endif
 result[4+tid]=acc;
}
