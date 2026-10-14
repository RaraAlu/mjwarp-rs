#define BORROW_NV 3
#define BORROW_NB 4
#define BORROW_COUNT 5
#define BORROW_DENSE 1

extern "C" __global__ void borrowed_mass_validate(const int* meta,const float* source,const float* vec,float* status,
 unsigned nb,unsigned n,unsigned so,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 unsigned long long stride=10ull*nb+1ull*n*n;bool valid=nb==BORROW_NB&&n==BORROW_NV&&meta[4]==BORROW_NV&&meta[5]==BORROW_COUNT;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[so+work+j]==-131072&&source[j]==-131072.0f&&source[4+stride*work+j]==-131072.0f&&vec[j]==-131072.0f&&vec[4+1ull*work*n+j]==-131072.0f;
 const int* row=meta+6;const int* col=row+BORROW_NV+1;const int* adr=col+BORROW_COUNT;
 valid=valid&&row[0]==0&&row[BORROW_NV]==BORROW_COUNT;
 for(unsigned r=0;r<BORROW_NV;++r){int a=row[r],b=row[r+1];if(a<0||a>b||b>BORROW_COUNT){valid=false;continue;}
  for(int k=a;k<b;++k){int c=col[k];if(c<0||c>=BORROW_NV){valid=false;continue;}unsigned expected=(r>(unsigned)c?r:(unsigned)c)*BORROW_NV+(r<(unsigned)c?r:(unsigned)c);if(adr[k]!=(int)expected)valid=false;}}
 if(!meta[so+w]){for(unsigned long long j=0;j<stride;++j)valid=valid&&isfinite(source[4+stride*w+j]);for(unsigned j=0;j<n;++j)valid=valid&&isfinite(vec[4+1ull*w*n+j]);}
 status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void borrowed_mass_multiply(const int* meta,const float* source,const float* vec,float* result,
 unsigned nb,unsigned n,unsigned so,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/n,r=tid%n;if(meta[so+w])return;
 const float* M=source+4+(10ull*nb+1ull*n*n)*w+10ull*nb;const float* v=vec+4+1ull*w*n;float acc=0.0f;
 #if BORROW_DENSE
 #pragma unroll
 for(unsigned c=0;c<BORROW_NV;++c)acc=acc+M[1ull*r*BORROW_NV+c]*v[c];
 #else
 const int* row=meta+6;const int* col=row+BORROW_NV+1;const int* adr=col+BORROW_COUNT;
 for(int k=row[r];k<row[r+1];++k)acc=acc+M[adr[k]]*v[col[k]];
 #endif
 result[4+tid]=acc;
}
