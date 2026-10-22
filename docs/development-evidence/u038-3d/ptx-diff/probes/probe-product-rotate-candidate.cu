struct V { float x,y,z; };
struct Q { float w,x,y,z; };
__device__ V load_v(const float* p) { return {p[0],p[1],p[2]}; }
__device__ Q load_q(const float* p) { return {p[0],p[1],p[2],p[3]}; }
__device__ void store_v(float* p,V v) { p[0]=v.x;p[1]=v.y;p[2]=v.z; }
__device__ void store_q(float* p,Q q) { p[0]=q.w;p[1]=q.x;p[2]=q.y;p[3]=q.z; }
__device__ V add(V a,V b) { return {a.x+b.x,a.y+b.y,a.z+b.z}; }
__device__ V scale(V a,float s) { return {a.x*s,a.y*s,a.z*s}; }
__device__ float dot_v(V a,V b) { return a.x*b.x+a.y*b.y+a.z*b.z; }
__device__ V cross_v(V a,V b) { return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x}; }
__device__ Q normalize(Q q) {
  float n=sqrtf(q.w*q.w+q.x*q.x+q.y*q.y+q.z*q.z);
#ifdef MJWARP_G01_EQUIVALENT
  if(n==0.0f) return {0,0,0,1};
#endif
  float s=1.0f/n;
  return {q.w*s,q.x*s,q.y*s,q.z*s};
}
__device__ Q multiply(Q a,Q b) {
  return {a.w*b.w-a.x*b.x-a.y*b.y-a.z*b.z,
          a.w*b.x+a.x*b.w+a.y*b.z-a.z*b.y,
          a.w*b.y-a.x*b.z+a.y*b.w+a.z*b.x,
          a.w*b.z+a.x*b.y-a.y*b.x+a.z*b.w};
}
__device__ V rotate(Q q,V v) {
  V u={q.x,q.y,q.z};
  V lhs=scale(scale(u,dot_v(u,v)),2.0f);
  float factor=q.w*q.w-dot_v(u,u);
  V r=add(lhs,scale(v,factor));
  return add(r,scale(cross_v(u,v),2.0f*q.w));
}

__device__ V smul(float s,V a) { return {s*a.x,s*a.y,s*a.z}; }
__device__ V rotate_candidate(Q q,V v) {
  V u={q.x,q.y,q.z};
  float d1=dot_v(u,v);
  V lhs=smul(2.0f,smul(d1,u));
  float factor=q.w*q.w-dot_v(u,u);
  V r=add(lhs,smul(factor,v));
  V c=cross_v(u,v);
  float two_s=2.0f*q.w;
  return add(r,smul(two_s,c));
}

extern "C" __global__ void probe_rotate_candidate(const float* in,float* out) {
  Q q=load_q(in); V v=load_v(in+4);
  store_v(out,rotate_candidate(q,v));
}
