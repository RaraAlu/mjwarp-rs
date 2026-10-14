#define ENERGY_NV 3

extern "C" __global__ void kinetic_energy_validate(const int* meta,const float* pair,
 const float* current,float* status,unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=meta[0]==ENERGY_NV&&n==ENERGY_NV;unsigned long long pl=2ull*n*worlds,el=2ull*worlds;
 for(unsigned j=0;j<4;++j)valid=valid&&pair[j]==-131072.0f&&pair[4+pl+j]==-131072.0f&&current[j]==-131072.0f&&current[4+el+j]==-131072.0f;
 for(unsigned j=0;j<2*n;++j)valid=valid&&isfinite(pair[4+2ull*n*w+j]);
 valid=valid&&isfinite(current[4+2ull*w]);status[4+w]=valid?1.0f:0.0f;
}
extern "C" __global__ void kinetic_energy_reduce32(const int* meta,const float* pair,
 const float* current,float* staged,unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x,lane=threadIdx.x;const float* q=pair+4+2ull*ENERGY_NV*w;const float* m=q+ENERGY_NV;
 float products[(ENERGY_NV+31)/32];
 #pragma unroll
 for(unsigned r=0;r<(ENERGY_NV+31)/32;++r){unsigned i=lane+32*r;float a=0.0f,b=0.0f;if(i<ENERGY_NV){a=q[i];b=m[i];}products[r]=a*b;}
 float sum=products[0];bool active=lane<ENERGY_NV;
 #pragma unroll
 for(unsigned r=1;r<(ENERGY_NV+31)/32;++r){if(lane+32*r>=ENERGY_NV)break;sum=sum+products[r];}
 unsigned mask=__ballot_sync(0xffffffffu,active);
 if(active){
  #pragma unroll
  for(unsigned offset=16;offset>0;offset/=2){float other=__shfl_down_sync(mask,sum,offset,32);
   if(mask==0xffffffffu||(mask&(1u<<((lane+offset)%32))))sum=sum+other;}
  if(lane==0){staged[4+2ull*w]=current[4+2ull*w];staged[5+2ull*w]=0.5f*sum;}
 }
}
