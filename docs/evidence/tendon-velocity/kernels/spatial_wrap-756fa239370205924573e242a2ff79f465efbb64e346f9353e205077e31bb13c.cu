
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
// 固定内侧初始长度的收缩顺序。
// 冻结上游先舍入x平方，再融合y平方。
// 短末段会放大1 ULP的长度差。
// 仅此处固定顺序，不改变通用n2。
__device__ float inside_norm(V2 a) {
  return sqrtf(__fmaf_rn(a.y,a.y,__fmul_rn(a.x,a.x)));
}
__device__ Wrap2 inside(V2 e0,V2 e1,float r) {
  float x0=e0.x,y0=e0.y,x1=e1.x,y1=e1.y;
  float l0=inside_norm(e0),l1=inside_norm(e1),dx=x1-x0,dy=y1-y0,dd=dx*dx+dy*dy;
  if(l0<=r || l1<=r || r<1e-15f || l0<1e-15f || l1<1e-15f) return no2();
  if(dd>1e-15f) {float a=-(dx*x0+dy*y0)/dd;
    if(a>0 && a<1 && n2({x0+dx*a,y0+dy*a})<=r) return no2();}
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
  float vx,vy,ang;
  if(x0*y1-y0*x1>0) {vx=x0/l0;vy=y0/l0;ang=as(z)-as(A*z);}
  else {vx=x1/l1;vy=y1/l1;ang=as(z)-as(B*z);}
  p={r*(cosf(ang)*vx-sinf(ang)*vy),r*(sinf(ang)*vx+cosf(ang)*vy)};
  return {0,p,p};
}
__device__ SV mt(const float* m,SV v) {
  return {m[0]*v.x+m[3]*v.y+m[6]*v.z,m[1]*v.x+m[4]*v.y+m[7]*v.z,m[2]*v.x+m[5]*v.y+m[8]*v.z};
}
__device__ SV mm(const float* m,SV v) {
  return {m[0]*v.x+m[1]*v.y+m[2]*v.z,m[3]*v.x+m[4]*v.y+m[5]*v.z,m[6]*v.x+m[7]*v.y+m[8]*v.z};
}
struct Wrap3 {float len;SV a,b,dir0,dir1;};
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
  Wrap2 w=valid && norm(sp)<r ? inside(e0,e1,r) : circle(e0,e1,s,r);
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


__device__ void put(float* p,SV v) {p[0]=v.x;p[1]=v.y;p[2]=v.z;}
__device__ SV point(const float* scratch,int i) {return sl(scratch+13*i+1);}
// Header: nt,nwrap,nnz,nbody,attached stride,site offset,ngeom,size period,size offset.
// Arrays: parent,root,dofadr,dofnum; tendonadr,num,rowadr,rownnz;
// wraptype,objid,body,sidesite; CSR columns.
// Private scratch: length, two contacts, two pre-rounding inside directions.
// Float result: lengths, CSR, compact wrap positions, private 13*nwrap scratch.
extern "C" __global__ void spatial_site(const int* m,const float* scales,
    const float* attached,float* result,unsigned nv,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[1],nnz=m[2],nb=m[3];
  const int* adr=m+9+4*nb;const int* num=adr+nt;
  const int* type=adr+4*nt;const int* site=type+nw;
  const int* side=type+3*nw;
  float* out=result+4+(unsigned long long)w*(nt+nnz+19ull*nw);
  float* points=out+nt+nnz;float* scratch=points+6*nw;
  for(int i=0;i<nt+nnz+19*nw;++i) out[i]=0.0f;
  const float* ax=attached+4+(unsigned long long)w*m[4];const float* sx=ax+m[5];
  for(int i=0;i<nw;++i) if(type[i]==3) {
    put(scratch+13*i+1,sl(sx+3*site[i]));
  }
  for(int i=0;i<nw;++i) if(type[i]==4 || type[i]==5) {
    int g=site[i];SV s=side[i]>=0 ? sl(sx+3*side[i]) : SV{1e10f,1e10f,1e10f};
    float radius=scales[m[8]+(unsigned long long)(w%m[7])*3*m[6]+3*g];
    Wrap3 r=wrap(point(scratch,i-1),point(scratch,i+1),sl(ax+3*g),ax+3*m[6]+9*g,radius,type[i],s);
    scratch[13*i]=r.len;put(scratch+13*i+1,r.a);put(scratch+13*i+4,r.b);
    put(scratch+13*i+7,r.dir0);put(scratch+13*i+10,r.dir1);
  }
  for(int t=0;t<nt;++t) for(int i=adr[t];i<adr[t]+num[t]-1;++i) {
    if(type[i]==3 && type[i+1]==3) out[t]+=norm(sub(point(scratch,i+1),point(scratch,i)))*scales[i];
    if(type[i]==4 || type[i]==5) {
      float len=scratch[13*i];
      if(len>=0) len=norm(sub(point(scratch,i),point(scratch,i-1)))+len
        +norm(sub(point(scratch,i+1),sl(scratch+13*i+4)));
      else len=norm(sub(point(scratch,i+1),point(scratch,i-1)));
      if(len!=0) out[t]+=len*scales[i];
    }
  }
  int cursor=0;
  for(int i=0;i<nw;++i) {
    if(type[i]==3) put(points+3*cursor++,point(scratch,i));
    else if(type[i]==2) put(points+3*cursor++,{0,0,0});
    else if(norm(point(scratch,i))<1e10f) {
      put(points+3*cursor++,point(scratch,i));put(points+3*cursor++,sl(scratch+13*i+4));
    }
  }
}
__device__ void chain(const int* m,const float* com,float* jac,int body,
    int row,int count,SV offset,SV vec,float scale) {
  int nb=m[3],nt=m[0],nw=m[1];
  const int* parent=m+9;const int* dofadr=parent+2*nb;const int* dofnum=dofadr+nb;
  const int* cols=m+9+4*nb+4*nt+4*nw;
  const float* cdof=com+14*nb;
  int ptr=count-1;
  while(body>0) {
    for(int k=dofnum[body]-1;k>=0;--k) {
      int dof=dofadr[body]+k;
      while(ptr>=0 && cols[row+ptr]>dof) --ptr;
      if(ptr>=0 && cols[row+ptr]==dof) {
        const float* c=cdof+6*dof;SV v=cross(sl(c),offset);
        float j=((c[3]+v.x)*vec.x+(c[4]+v.y)*vec.y+(c[5]+v.z)*vec.z)*scale;
        if(j!=0.0f) jac[row+ptr]+=j;
      }
    }
    body=parent[body];
  }
}
__device__ void directed_segment(const int* m,const float* com,float* jac,int row,int count,
    int b0,int b1,SV p0,SV p1,SV vec,float scale) {
  if(b0==b1) return;
  const int* root=m+9+m[3];
  chain(m,com,jac,b0,row,count,sub(p0,sl(com+m[3]+3*root[b0])),vec,-scale);
  chain(m,com,jac,b1,row,count,sub(p1,sl(com+m[3]+3*root[b1])),vec,scale);
}
__device__ void segment(const int* m,const float* com,float* jac,int row,int count,
    int b0,int b1,SV p0,SV p1,float scale) {
  SV dif=sub(p1,p0);
  directed_segment(m,com,jac,row,count,b0,b1,p0,p1,direction(dif,norm(dif)),scale);
}
extern "C" __global__ void spatial_moment(const int* m,const float* scales,
    const float* input,float* result,unsigned nv,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[1],nnz=m[2],nb=m[3];
  const int* adr=m+9+4*nb;const int* num=adr+nt;
  const int* row=num+nt;const int* count=row+nt;const int* type=count+nt;
  const int* body=type+2*nw;
  const float* com=input+4+(unsigned long long)w*(14ull*nb+6ull*nv);
  float* out=result+4+(unsigned long long)w*(nt+nnz+19ull*nw);
  float* jac=out+nt;const float* scratch=jac+nnz+6*nw;
  for(int j=0;j<nnz;++j) jac[j]=0.0f;
  for(int t=0;t<nt;++t) for(int i=adr[t];i<adr[t]+num[t]-1;++i) {
    if(type[i]==3 && type[i+1]==3)
      segment(m,com,jac,row[t],count[t],body[i],body[i+1],point(scratch,i),point(scratch,i+1),scales[i]);
    if(type[i]==4 || type[i]==5) {
      if(scratch[13*i]>=0) {
        SV d0=sl(scratch+13*i+7),d1=sl(scratch+13*i+10);
        if(dot(d0,d0)>0) {
          directed_segment(m,com,jac,row[t],count[t],body[i-1],body[i],point(scratch,i-1),point(scratch,i),d0,scales[i]);
          directed_segment(m,com,jac,row[t],count[t],body[i],body[i+1],sl(scratch+13*i+4),point(scratch,i+1),d1,scales[i]);
        } else {
          segment(m,com,jac,row[t],count[t],body[i-1],body[i],point(scratch,i-1),point(scratch,i),scales[i]);
          segment(m,com,jac,row[t],count[t],body[i],body[i+1],sl(scratch+13*i+4),point(scratch,i+1),scales[i]);
        }
      } else segment(m,com,jac,row[t],count[t],body[i-1],body[i+1],point(scratch,i-1),point(scratch,i+1),scales[i]);
    }
  }
}
// Mixed f32 input / i32 output retains the same eight-argument pointer ABI.
extern "C" __global__ void spatial_wrap(const int* m,const float* params,
    const float* state,int* result,unsigned nv,unsigned a,unsigned b,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds) return;
  int nt=m[0],nw=m[1],nnz=m[2],nb=m[3];const int* adr=m+9+4*nb;const int* num=adr+nt;
  const int* type=adr+4*nt;const int* id=type+nw;
  const float* scratch=state+4+(unsigned long long)w*(nt+nnz+19ull*nw)+nt+nnz+6*nw;
  int* out=result+4+(unsigned long long)w*(2ull*nt+2ull*nw);
  for(int i=0;i<2*nt+2*nw;++i) out[i]=0;
  int cursor=0;
  for(int t=0;t<nt;++t) {
    out[t]=cursor;
    for(int i=adr[t];i<adr[t]+num[t];++i) {
      if(type[i]==3 || type[i]==2) out[2*nt+cursor++]=type[i]==2?-2:-1;
      else if(norm(point(scratch,i))<1e10f) {out[2*nt+cursor++]=id[i];out[2*nt+cursor++]=id[i];}
    }
    out[nt+t]=cursor-out[t];
  }
}
