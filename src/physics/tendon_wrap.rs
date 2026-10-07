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
struct DV2 {double x,y;};
struct Inside2 {double len;DV2 a,b;};
__device__ double das(double x) {return asin(fmin(fmax(x,-1.0),1.0));}
__device__ Inside2 inside(DV2 e0,DV2 e1,double r) {
  // Newton在近切点处需要双精度。
  double x0=e0.x,y0=e0.y,x1=e1.x,y1=e1.y;
  double l0=hypot(x0,y0),l1=hypot(x1,y1),dx=x1-x0,dy=y1-y0,dd=dx*dx+dy*dy;
  Inside2 no={-1,{1e10,1e10},{1e10,1e10}};
  if(l0<=r || l1<=r || r<1e-15 || l0<1e-15 || l1<1e-15) return no;
  if(dd>1e-15) {double a=-(dx*x0+dy*y0)/dd;
    if(a>0 && a<1 && hypot(x0+dx*a,y0+dy*a)<=r) return no;}
  double n=hypot(x0+x1,y0+y1);
  DV2 p={n!=0?r*(x0+x1)/n:0,n!=0?r*(y0+y1)/n:0};Inside2 fallback={0,p,p};
  double A=r/l0,B=r/l1,aa=A*A,bb=B*B;
  double c=(l0*l0+l1*l1-dd)/(2*l0*l1);
  if(c<-1+1e-15) return {-1,p,p};
  if(c>1-1e-15) return fallback;
  double G=acos(fmin(fmax(c,-1.0),1.0)),z=1-1e-7;
  double f=das(A*z)+das(B*z)-2*das(z)+G;
  if(f>0) return fallback;
  int iter=0;
  while(iter<20 && fabs(f)>1e-6) {
    double zz=z*z;
    double df=A/fmax(1e-15,sqrt(1-zz*aa))+B/fmax(1e-15,sqrt(1-zz*bb))
      -2/fmax(1e-15,sqrt(1-zz));
    if(df>-1e-15) return fallback;
    double z1=z-f/df;if(z1>z) return fallback;
    z=z1;f=das(A*z)+das(B*z)-2*das(z)+G;
    if(f>1e-6) return fallback;
    ++iter;
  }
  if(iter>=20) return fallback;
  double vx,vy,ang;
  if(x0*y1-y0*x1>0) {vx=x0/l0;vy=y0/l0;ang=das(z)-das(A*z);}
  else {vx=x1/l1;vy=y1/l1;ang=das(z)-das(B*z);}
  p={r*(cos(ang)*vx-sin(ang)*vy),r*(sin(ang)*vx+cos(ang)*vy)};
  return {0,p,p};
}
__device__ SV mt(const float* m,SV v) {
  return {m[0]*v.x+m[3]*v.y+m[6]*v.z,m[1]*v.x+m[4]*v.y+m[7]*v.z,m[2]*v.x+m[5]*v.y+m[8]*v.z};
}
__device__ SV mm(const float* m,SV v) {
  return {m[0]*v.x+m[1]*v.y+m[2]*v.z,m[3]*v.x+m[4]*v.y+m[5]*v.z,m[6]*v.x+m[7]*v.y+m[8]*v.z};
}
struct DV {double x,y,z;};
__device__ DV dsub(DV a,DV b) {return {a.x-b.x,a.y-b.y,a.z-b.z};}
__device__ DV local(const float* m,SV x,SV pos) {
  double dx=(double)x.x-pos.x,dy=(double)x.y-pos.y,dz=(double)x.z-pos.z;
  return {m[0]*dx+m[3]*dy+m[6]*dz,m[1]*dx+m[4]*dy+m[7]*dz,m[2]*dx+m[5]*dy+m[8]*dz};
}
__device__ DV rotate(const float* m,DV v) {
  return {m[0]*v.x+m[1]*v.y+m[2]*v.z,m[3]*v.x+m[4]*v.y+m[5]*v.z,m[6]*v.x+m[7]*v.y+m[8]*v.z};
}
__device__ SV precise_direction(DV v) {
  double n=sqrt(v.x*v.x+v.y*v.y+v.z*v.z);
  return n<1e-15 ? SV{1,0,0} : SV{(float)(v.x/n),(float)(v.y/n),(float)(v.z/n)};
}
struct Wrap3 {float len;SV a,b,dir0,dir1;};
__device__ Wrap3 inside_wrap(SV x0,SV x1,SV pos,const float* mat,float r,int type,SV ax0,SV ax1) {
  DV p0=local(mat,x0,pos),p1=local(mat,x1,pos);
  DV2 e0={p0.x*ax0.x+p0.y*ax0.y+p0.z*ax0.z,p0.x*ax1.x+p0.y*ax1.y+p0.z*ax1.z};
  DV2 e1={p1.x*ax0.x+p1.y*ax0.y+p1.z*ax0.z,p1.x*ax1.x+p1.y*ax1.y+p1.z*ax1.z};
  Inside2 w=inside(e0,e1,r);
  if(w.len<0) return {-1,{1e10f,1e10f,1e10f},{1e10f,1e10f,1e10f},{0,0,0},{0,0,0}};
  DV a={ax0.x*w.a.x+ax1.x*w.a.y,ax0.y*w.a.x+ax1.y*w.a.y,ax0.z*w.a.x+ax1.z*w.a.y};
  DV b={ax0.x*w.b.x+ax1.x*w.b.y,ax0.y*w.b.x+ax1.y*w.b.y,ax0.z*w.b.x+ax1.z*w.b.y};
  if(type==5) {
    double l0=hypot(p0.x-a.x,p0.y-a.y),l1=hypot(p1.x-b.x,p1.y-b.y),total=l0+w.len+l1;
    double denominator=total!=0?total:1e-15;
    a.z=p0.z+(p1.z-p0.z)*l0/denominator;
    b.z=p0.z+(p1.z-p0.z)*(l0+w.len)/denominator;
    w.len=hypot(w.len,b.z-a.z);
  }
  // 先算方向，再舍入世界接触点。
  // 局部差分避开矩阵往返误差。
  SV dir0=precise_direction(rotate(mat,dsub(a,p0))),dir1=precise_direction(rotate(mat,dsub(p1,b)));
  a=rotate(mat,a);b=rotate(mat,b);
  return {(float)w.len,{(float)(a.x+pos.x),(float)(a.y+pos.y),(float)(a.z+pos.z)},
    {(float)(b.x+pos.x),(float)(b.y+pos.y),(float)(b.z+pos.z)},dir0,dir1};
}
__device__ Wrap3 wrap(SV x0,SV x1,SV pos,const float* mat,float r,int type,SV side) {
  SV p0=mt(mat,sub(x0,pos)),p1=mt(mat,sub(x1,pos));
  Wrap3 no={-1,{1e10f,1e10f,1e10f},{1e10f,1e10f,1e10f},{0,0,0},{0,0,0}};
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
  if(valid && norm(sp)<r) return inside_wrap(x0,x1,pos,mat,r,type,ax0,ax1);
  Wrap2 w=circle(e0,e1,s,r);
  if(w.len<0) return no;
  SV a=add(mul(ax0,w.a.x),mul(ax1,w.a.y)),b=add(mul(ax0,w.b.x),mul(ax1,w.b.y));
  if(type==5) {
    float l0=n2({p0.x-a.x,p0.y-a.y}),l1=n2({p1.x-b.x,p1.y-b.y});
    a.z=p0.z+(p1.z-p0.z)*sd(l0,l0+w.len+l1);
    b.z=p0.z+(p1.z-p0.z)*sd(l0+w.len,l0+w.len+l1);
    float h=fabsf(b.z-a.z);w.len=sqrtf(w.len*w.len+h*h);
  }
  return {w.len,add(mm(mat,a),pos),add(mm(mat,b),pos),{0,0,0},{0,0,0}};
}
"#;
