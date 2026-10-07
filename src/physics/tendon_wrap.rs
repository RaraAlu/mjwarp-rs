//! 冻结球柱绕行的内部CUDA公式。
//! 产品链不运行宿主绕行计算。

// Adapted from mujoco_warp/_src/util_misc.py, frozen revision
// 71da24d956378a87a703b6e1442b13aec0c4ac29.
// Copyright 2025 The Newton Developers
// Copyright (c) 2022 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// Clamped inverse trig follows NVIDIA Warp 1.15.0 (Apache-2.0).
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
pub(super) const TENDON_WRAP_CUDA: &str = r#"
struct SV { float x,y,z; };
struct V2 { float x,y; };
__device__ SV sl(const float* p) {return {p[0],p[1],p[2]};}
__device__ SV add(SV a,SV b) {return {a.x+b.x,a.y+b.y,a.z+b.z};}
__device__ SV sub(SV a,SV b) {return {a.x-b.x,a.y-b.y,a.z-b.z};}
__device__ SV mul(SV a,float s) {return {a.x*s,a.y*s,a.z*s};}
__device__ float dot(SV a,SV b) {return a.x*b.x+a.y*b.y+a.z*b.z;}
__device__ SV cross(SV a,SV b) {return {a.y*b.z-a.z*b.y,a.z*b.x-a.x*b.z,a.x*b.y-a.y*b.x};}
__device__ float norm(SV a) {return sqrtf(dot(a,a));}
__device__ SV unit(SV a) {float n=norm(a);return n!=0 ? SV{a.x/n,a.y/n,a.z/n} : a;}
__device__ SV direction(SV a,float n) {return n<1e-15f ? SV{1,0,0} : SV{a.x/n,a.y/n,a.z/n};}
__device__ V2 a2(V2 a,V2 b) {return {a.x+b.x,a.y+b.y};}
__device__ V2 s2(V2 a,V2 b) {return {a.x-b.x,a.y-b.y};}
__device__ V2 m2(V2 a,float s) {return {a.x*s,a.y*s};}
__device__ float d2(V2 a,V2 b) {return a.x*b.x+a.y*b.y;}
__device__ float n2(V2 a) {return sqrtf(d2(a,a));}
__device__ V2 u2(V2 a) {float n=n2(a);return n!=0 ? V2{a.x/n,a.y/n} : a;}
__device__ float sd(float x,float y) {return x/(y!=0.0f ? y : 1e-15f);}
__device__ float ac(float x) {return acosf(fminf(fmaxf(x,-1.0f),1.0f));}
__device__ float as(float x) {return asinf(fminf(fmaxf(x,-1.0f),1.0f));}
__device__ bool intersect(V2 p1,V2 p2,V2 p3,V2 p4) {
  float det=(p4.y-p3.y)*(p2.x-p1.x)-(p4.x-p3.x)*(p2.y-p1.y);
  if(fabsf(det)<1e-15f) return false;
  float a=((p4.x-p3.x)*(p1.y-p3.y)-(p4.y-p3.y)*(p1.x-p3.x))/det;
  float b=((p2.x-p1.x)*(p1.y-p3.y)-(p2.y-p1.y)*(p1.x-p3.x))/det;
  return a>=0 && a<=1 && b>=0 && b<=1;
}
struct Wrap2 {float len;V2 a,b;};
__device__ Wrap2 no2() {return {-1,{1e10f,1e10f},{1e10f,1e10f}};}
__device__ Wrap2 circle(V2 e0,V2 e1,V2 side,float r) {
  bool valid=n2(side)<1e10f;
  float l0=d2(e0,e0),l1=d2(e1,e1),rr=r*r;
  if(l0<rr || l1<rr || r<1e-15f) return no2();
  V2 dif=s2(e1,e0);float dd=d2(dif,dif);if(dd<1e-15f) return no2();
  float a=fminf(fmaxf(-d2(dif,e0)/dd,0.0f),1.0f);
  V2 tmp=a2(m2(dif,a),e0);
  if(d2(tmp,tmp)>rr && (!valid || d2(side,tmp)>=0)) return no2();
  float s0=sqrtf(l0-rr),s1=sqrtf(l1-rr);
  V2 p00={sd(e0.x*rr+r*e0.y*s0,l0),sd(e0.y*rr-r*e0.x*s0,l0)};
  V2 p01={sd(e1.x*rr-r*e1.y*s1,l1),sd(e1.y*rr+r*e1.x*s1,l1)};
  V2 p10={sd(e0.x*rr-r*e0.y*s0,l0),sd(e0.y*rr+r*e0.x*s0,l0)};
  V2 p11={sd(e1.x*rr+r*e1.y*s1,l1),sd(e1.y*rr-r*e1.x*s1,l1)};
  float g0,g1;
  if(valid) {g0=d2(u2(a2(p00,p01)),side);g1=d2(u2(a2(p10,p11)),side);}
  else {tmp=s2(p00,p01);g0=-d2(tmp,tmp);tmp=s2(p10,p11);g1=-d2(tmp,tmp);}
  if(intersect(e0,p00,e1,p01)) g0=-10000.0f;
  if(intersect(e0,p10,e1,p11)) g1=-10000.0f;
  int ind=g0>g1 ? 0 : 1;V2 p0=ind==0?p00:p10,p1=ind==0?p01:p11;
  if(intersect(e0,p0,e1,p1)) return no2();
  float angle=ac(d2(u2(p0),u2(p1)));
  float cr=p0.y*p1.x-p0.x*p1.y;
  if((cr>0 && ind!=0) || (cr<0 && ind==0)) angle=2.0f*3.14159265358979323846f-angle;
  return {r*angle,p0,p1};
}
__device__ Wrap2 inside(V2 e0,V2 e1,float r) {
  float l0=n2(e0),l1=n2(e1);V2 dif=s2(e1,e0);float dd=d2(dif,dif);
  if(l0<=r || l1<=r || r<1e-15f || l0<1e-15f || l1<1e-15f) return no2();
  if(dd>1e-15f) {float a=-d2(dif,e0)/dd;
    if(a>0 && a<1 && n2(a2(e0,m2(dif,a)))<=r) return no2();}
  V2 p=m2(u2(m2(a2(e0,e1),0.5f)),r);Wrap2 fallback={0,p,p};
  float A=sd(r,l0),B=sd(r,l1),aa=A*A,bb=B*B;
  float c=sd(l0*l0+l1*l1-dd,2.0f*l0*l1);
  if(c<-1.0f+1e-15f) return {-1,p,p};
  if(c>1.0f-1e-15f) return fallback;
  float G=ac(c),z=1.0f-1e-7f;
  float f=as(A*z)+as(B*z)-2.0f*as(z)+G;
  if(f>0) return fallback;
  int iter=0;
  while(iter<20 && fabsf(f)>1e-6f) {
    float zz=z*z;
    float df=A/fmaxf(1e-15f,sqrtf(1.0f-zz*aa))+B/fmaxf(1e-15f,sqrtf(1.0f-zz*bb))
      -2.0f/fmaxf(1e-15f,sqrtf(1.0f-zz));
    if(df>-1e-15f) return fallback;
    float z1=z-sd(f,df);if(z1>z) return fallback;
    z=z1;f=as(A*z)+as(B*z)-2.0f*as(z)+G;
    if(f>1e-6f) return fallback;
    ++iter;
  }
  if(iter>=20) return fallback;
  V2 v;float ang;
  if(e0.x*e1.y-e0.y*e1.x>0) {v=e0;ang=as(z)-as(A*z);}
  else {v=e1;ang=as(z)-as(B*z);}
  v=u2(v);p={r*(cosf(ang)*v.x-sinf(ang)*v.y),r*(sinf(ang)*v.x+cosf(ang)*v.y)};
  return {0,p,p};
}
__device__ SV mt(const float* m,SV v) {
  return {m[0]*v.x+m[3]*v.y+m[6]*v.z,m[1]*v.x+m[4]*v.y+m[7]*v.z,m[2]*v.x+m[5]*v.y+m[8]*v.z};
}
__device__ SV mm(const float* m,SV v) {
  return {m[0]*v.x+m[1]*v.y+m[2]*v.z,m[3]*v.x+m[4]*v.y+m[5]*v.z,m[6]*v.x+m[7]*v.y+m[8]*v.z};
}
struct Wrap3 {float len;SV a,b;};
__device__ Wrap3 wrap(SV x0,SV x1,SV pos,const float* mat,float r,int type,SV side) {
  SV p0=mt(mat,sub(x0,pos)),p1=mt(mat,sub(x1,pos));
  Wrap3 no={-1,{1e10f,1e10f,1e10f},{1e10f,1e10f,1e10f}};
  if(norm(p0)<1e-15f || norm(p1)<1e-15f) return no;
  SV ax0,ax1;
  if(type==4) {
    ax0=unit(p0);SV normal=cross(p0,p1);float n=norm(normal);normal=unit(normal);
    if(n<1e-15f) {
      int i=0;float x=fabsf(ax0.x),y=fabsf(ax0.y),z=fabsf(ax0.z);
      if(y>x && y>z) i=1;if(z>x && z>y) i=2;
      ax1={i==0?0.0f:1.0f,i==1?0.0f:1.0f,i==2?0.0f:1.0f};normal=unit(cross(ax0,ax1));
    }
    ax1=unit(cross(normal,ax0));
  } else {ax0={1,0,0};ax1={0,1,0};}
  V2 e0={dot(p0,ax0),dot(p0,ax1)},e1={dot(p1,ax0),dot(p1,ax1)};
  bool valid=norm(side)<1e10f;SV sp={0,0,0};V2 s={1e10f,1e10f};
  if(valid) {sp=mt(mat,sub(side,pos));s=m2(u2({dot(sp,ax0),dot(sp,ax1)}),r);}
  Wrap2 w=valid && norm(sp)<r ? inside(e0,e1,r) : circle(e0,e1,s,r);
  if(w.len<0) return no;
  SV a=add(mul(ax0,w.a.x),mul(ax1,w.a.y)),b=add(mul(ax0,w.b.x),mul(ax1,w.b.y));
  if(type==5) {
    float l0=n2({p0.x-a.x,p0.y-a.y}),l1=n2({p1.x-b.x,p1.y-b.y});
    a.z=p0.z+(p1.z-p0.z)*sd(l0,l0+w.len+l1);
    b.z=p0.z+(p1.z-p0.z)*sd(l0+w.len,l0+w.len+l1);
    float h=fabsf(b.z-a.z);w.len=sqrtf(w.len*w.len+h*h);
  }
  return {w.len,add(mm(mat,a),pos),add(mm(mat,b),pos)};
}
"#;
