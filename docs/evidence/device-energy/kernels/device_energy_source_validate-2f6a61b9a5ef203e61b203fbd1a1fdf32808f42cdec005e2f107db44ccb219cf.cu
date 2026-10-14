#define ENERGY_NV 3

extern "C" __global__ void device_energy_source_validate(const int* meta,const float* q,const float* mq,float* status,
 unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=n==ENERGY_NV&&meta[0]==ENERGY_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&q[j]==-131072.0f&&mq[j]==-131072.0f&&q[4+1ull*n*worlds+j]==-131072.0f&&mq[4+1ull*n*worlds+j]==-131072.0f;
 for(unsigned j=0;j<n;++j)valid=valid&&isfinite(q[4+1ull*w*n+j])&&isfinite(mq[4+1ull*w*n+j]);
 status[4+w]=valid?1.0f:0.0f;
}
