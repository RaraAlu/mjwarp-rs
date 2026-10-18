
extern "C" __global__ void applied_force_validate(const int* meta,const float* geometry,
 const float* ft,float* status,unsigned nb,unsigned n,unsigned worlds,unsigned work) {
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=true;unsigned long long ml=2ull*nb+n,gl=6ull*worlds*(nb+n),fl=6ull*worlds*nb;
 for(unsigned j=0;j<4;++j)valid=valid&&meta[j]==-131072&&meta[4+ml+j]==-131072&&geometry[j]==-131072.0f&&geometry[4+gl+j]==-131072.0f&&ft[j]==-131072.0f&&ft[4+fl+j]==-131072.0f;
 const int* parent=meta+4;const int* root=parent+nb;const int* body=root+nb;
 for(unsigned b=0;b<nb;++b){int p=parent[b],r=root[b];if((b==0&&p!=0)||(b>0&&(p<0||(unsigned)p>=b))||r<0||(unsigned)r>=nb){valid=false;continue;}
  int expected=b==0?0:(p==0?(int)b:root[p]);if(r!=expected)valid=false;}
 for(unsigned i=0;i<n;++i)if(body[i]<0||(unsigned)body[i]>=nb)valid=false;
 for(unsigned long long j=0;j<3ull*nb;++j)valid=valid&&isfinite(geometry[4+3ull*w*nb+j])&&isfinite(geometry[4+3ull*worlds*nb+3ull*w*nb+j]);
 for(unsigned long long j=0;j<6ull*n;++j)valid=valid&&isfinite(geometry[4+6ull*worlds*nb+6ull*w*n+j]);
 for(unsigned long long j=0;j<6ull*nb;++j)valid=valid&&isfinite(ft[4+6ull*w*nb+j]);status[4+w]=valid?1.0f:0.0f;
}
__device__ float force_dot6(const float* a,const float* b){float ret=0.0f;
 #pragma unroll
 for(unsigned i=0;i<6;++i)ret+=a[i]*b[i];return ret;}
__device__ float force_dot3(const float* a,const float* b){return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}
extern "C" __global__ void applied_force_delta(const int* meta,const float* geometry,
 const float* ft,float* output,unsigned nb,unsigned n,unsigned worlds,unsigned work) {
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/n,i=tid%n;
 const int* parent=meta+4;const int* root=parent+nb;const int* body=root+nb;
 const float* xpos=geometry+4+3ull*w*nb;const float* com=geometry+4+3ull*worlds*nb+3ull*w*nb;
 const float* cdof=geometry+4+6ull*worlds*nb+6ull*w*n+6ull*i;const float* force=ft+4+6ull*w*nb;
 float jac[6]={cdof[3],cdof[4],cdof[5],cdof[0],cdof[1],cdof[2]};float accum=0.0f;int dofbody=body[i];
 for(unsigned b=dofbody;b<nb;++b){const float* f=force+6ull*b;if(f[0]==0.0f&&f[1]==0.0f&&f[2]==0.0f&&f[3]==0.0f&&f[4]==0.0f&&f[5]==0.0f)continue;
  int p=b;while(p!=0&&p!=dofbody)p=parent[p];if(p==0)continue;
  const float* point=xpos+3ull*b;const float* center=com+3ull*(unsigned)root[b];
  float offset[3]={point[0]-center[0],point[1]-center[1],point[2]-center[2]};
  float cross[3]={cdof[1]*offset[2]-cdof[2]*offset[1],cdof[2]*offset[0]-cdof[0]*offset[2],cdof[0]*offset[1]-cdof[1]*offset[0]};
  accum+=force_dot6(jac,f)+force_dot3(cross,f);
 }output[4+tid]=accum;
}
extern "C" __global__ void applied_force_publish(const int* meta,const float* current,
 const float* delta,float* staged,unsigned n,unsigned worlds,unsigned add,unsigned work){
 unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=work)return;
 staged[4+i]=add?__fadd_rn(current[4+i],delta[4+i]):delta[4+i];
}
