//! G01运动学与GPU辅助阶段。
//! 其余完整物理阶段仍待实现。

mod actuator;
pub use actuator::{
    ActuationData, ActuationError, ActuationOutput, ActuatorModel, JOINT_HINGE, JOINT_SLIDE,
    JointActuatorModel, JointTransmission, JointTransmissionData, TRN_JOINT, fwd_actuation,
    transmission,
};
mod collision;
/// G3 宿主 SDF 数学（相对旋转修正后）公开给集成测试/后续片：配对窄相与 octree 体积。
pub use collision::sdf::{SdfContact, SdfPair, SdfShape, SdfType, VolumeData, sdf_pair_contacts};
pub use collision::{
    BroadphaseError, BroadphaseFilter, BroadphaseModel, BroadphaseOptions, BroadphaseOutput,
    BroadphaseState, BroadphaseType, CcdContact, CcdOptions, CcdOutput, CcdResult, ContactOptions,
    GEOM_BOX, GEOM_MESH, GEOM_PLANE, GEOM_SPHERE, Geom, GeomContact, GeomPair, MJ_MINMU, MJ_MINVAL,
    MeshView, NarrowphaseError, PrimitiveModel, PrimitiveState, ccd, ccd_pair, geom_type,
    make_frame, nxn_broadphase, plane_box, plane_sphere, primitive_narrowphase, sap_broadphase,
    sphere_sphere,
};
// G10@P5：碰撞设备源码片段，供 NVRTC 产品入口按依赖顺序拼装（`cuda-probe` 下可用）。
#[cfg(feature = "cuda-probe")]
pub(crate) use collision::cuda::{
    CCD_CUDA, COMMON_CUDA, CONTACT_CUDA, EPA_CUDA, GEOM_CUDA, GJK_CORE_CUDA, HFIELD_CUDA,
    POLYTOPE_BUILD_CUDA, POLYTOPE_CUDA, SDF_MATH_CUDA, SDF_NARROWPHASE_CUDA, SDF_OCTREE_CUDA,
    SIMPLEX_CUDA, SUPPORT_CUDA,
};
mod constants;
pub use constants::{
    FixedConstants, FixedConstantsInput, QposDependentConstants, QposDependentInput, set_const_0,
    set_const_fixed,
};
mod query;
pub use query::{
    GeomPrimitive, QueryError, Ray, RayHit, WorldGeoms, ray, ray_geom, rays, rays_batch,
};
mod integrator;
pub use integrator::{
    DiscreteAccBranch, IntegratorError, IntegratorState, IntegratorStep, JOINT_BALL, JOINT_FREE,
    JointLayout, Rk4Stage, advance, discrete, discrete_acc, euler, next_position, next_velocity,
    quat_integrate, rungekutta4,
};
mod derivative;
pub use derivative::{DerivativeError, DerivativeLayout, deriv_smooth_vel, poly_force_deriv};
mod inverse;
pub use inverse::{InverseError, SparseMass, inverse_dynamics, mul_m};
mod kinetic_energy;
pub use kinetic_energy::{
    KineticEnergyData, KineticEnergyOutput, KineticEnergyPlan, KineticEnergyVector,
};
mod applied_force;
pub use applied_force::{
    AppliedForceData, AppliedForceGeometry, AppliedForceOutput, AppliedForcePlan,
};
mod smooth_force;
pub use smooth_force::{
    JointPassiveData, JointPassiveOptions, JointPassiveOutput, JointPassiveParameters,
    JointPassivePlan,
};
pub use smooth_force::{
    SmoothForceData, SmoothForceField, SmoothForceOptions, SmoothForceOutput, SmoothForcePlan,
};

mod mass_matrix;
pub use mass_matrix::schedule_energy_chain;
pub use mass_matrix::{
    MassMatrixData, MassMatrixOutput, MassMatrixPlan, MassMatrixWorld, probe_mass_matrix,
};
pub use mass_matrix::{
    MassMatrixEnergyData, MassMatrixEnergyPlan, MassMatrixEnergySnapshot, MassMatrixEnergyView,
};
pub use mass_matrix::{MassMatrixMultiplyData, MassMatrixMultiplyPlan, MassMatrixMultiplyView};
pub use mass_matrix::{MassMultiplyData, MassMultiplyFormat, MassMultiplyOutput, MassMultiplyPlan};
mod mass_solve;
pub use mass_solve::{
    MassFactorBlock, MassFactorKind, MassSolveData, MassSolveOutput, MassSolvePlan, MassSolveWorld,
    SparseMassData, SparseMassDenseData, SparseMassDenseOutput, SparseMassDensePlan,
    SparseMassFactorData, SparseMassFactorOutput, SparseMassFactorPlan, SparseMassFactorWorld,
    SparseMassMixedFactorData, SparseMassMixedFactorOutput, SparseMassMixedFactorPlan,
    SparseMassMixedFactorWorld, SparseMassMixedResidentData, SparseMassMixedResidentPlan,
    SparseMassMixedSolveData, SparseMassMixedSolveOutput, SparseMassMixedSolvePlan,
    SparseMassMixedSolveWorld, SparseMassOutput, SparseMassPlan, SparseMassSmallData,
    SparseMassSmallOutput, SparseMassSmallPlan, SparseMassSolveData, SparseMassSolveOutput,
    SparseMassSolvePlan, SparseMassSolveWorld, SparseMassWorld, probe_mass_solve,
};
mod attached;
pub use attached::{AttachedKinematicsOutput, AttachedKinematicsWorld, probe_attached_kinematics};
mod resident;
#[cfg(any(feature = "cuda-probe", test))]
mod rigid_kinematics;
pub use resident::{
    G01SleepState, G01State, KinematicsData, KinematicsPlan, KinematicsSnapshot, camlight, com_pos,
    flex, fwd_kinematics, kinematics, tendon,
};
mod com_velocity;
pub use com_velocity::{ComVelocityData, ComVelocityPlan, ComVelocitySnapshot, ComVelocityWorld};
mod rne_bias;
pub use rne_bias::{RneBiasData, RneBiasPlan, RneBiasSnapshot, RneBiasWorld};
mod rne;
pub use rne::{RneData, RnePlan, RneSnapshot, RneWorld};
mod camlight;
pub use camlight::{CamLightOutput, CamLightWorld};
mod fixed_tendon;
pub use fixed_tendon::{FixedTendonOutput, FixedTendonWorld};
mod tendon_velocity;
pub use tendon_velocity::{
    TendonVelocityData, TendonVelocityPlan, TendonVelocitySnapshot, TendonVelocityView,
};
mod tendon_passive;
pub use tendon_passive::{
    TendonPassiveData, TendonPassiveOptions, TendonPassiveParameters, TendonPassivePlan,
    TendonPassiveSnapshot, TendonPassiveView,
};
mod flex_edge_passive;
pub use flex_edge_passive::{
    FlexEdgePassiveData, FlexEdgePassiveInput, FlexEdgePassiveOptions, FlexEdgePassivePlan,
    FlexEdgePassiveSnapshot, FlexEdgePassiveView,
};
mod gravity_compensation;
pub use gravity_compensation::{
    GravityCompensationData, GravityCompensationOptions, GravityCompensationParameters,
    GravityCompensationPlan, GravityCompensationSnapshot, GravityCompensationView,
};
mod fluid_force;
pub use fluid_force::{
    FluidForceData, FluidForceInput, FluidForcePlan, FluidForceSnapshot, FluidForceView,
    FluidForceWorld,
};
mod passive_force;
pub use passive_force::{
    ClearedPassiveForceView, PassiveForceData, PassiveForcePlan, PassiveForceSnapshot,
    PassiveForceView, SleepClearedPassiveForceView,
};
mod sensor;
pub use sensor::{SensorError, SensorLayout, sensor_pos, sensor_vel};
mod spatial_tendon;
#[cfg(feature = "cuda-probe")]
mod tendon_wrap;
pub use spatial_tendon::{SpatialTendonOutput, SpatialTendonWorld};
mod tendon;
pub use tendon::{TendonOutput, TendonWorld};
mod flex;
pub use flex::{FlexPositionOutput, FlexPositionWorld};
mod flex_edge;
pub use flex_edge::{FlexEdgeOutput, FlexEdgeWorld};
mod flex_face;
pub use flex_face::{FlexFaceOutput, FlexFaceWorld};
mod flex_hessian;
pub use flex_hessian::{FlexHessianOutput, FlexHessianWorld};
mod sleep;
pub use sleep::{SleepTreeOutput, SleepTreeWorld};

// 子模块：纯移动自原单文件，行为不变。
mod callback;
mod com_position;
mod device_checks;
mod history;
mod kinematics;
pub use callback::{
    CALLBACK_SYNC_CONTRACT, Callback, CallbackContract, CallbackError, CallbackGate, CallbackGates,
    CallbackInvocation, CallbackKind, CallbackStage, CallbackSyncContract, CallbackTarget,
    SyncRequirement, UserTypeField,
};
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) use com_position::com_position_device;
pub(in crate::physics) use com_position::{ComPositionLayout, check_com_position};
pub use com_position::{ComPositionOutput, ComPositionWorld, probe_com_position};
#[cfg(any(feature = "cuda-probe", test))]
pub(in crate::physics) use device_checks::{check_device_guards, check_device_values};
pub use history::{
    DelayBufferLayout, HISTORY_EPSILON, HistoryBuffer, HistoryError, HistoryInterp, HistoryLayout,
};
#[cfg(feature = "cuda-probe")]
pub(in crate::physics) use kinematics::kinematics_device;
pub(in crate::physics) use kinematics::{KinematicsLayout, check_kinematics, norm_squared};
pub use kinematics::{KinematicsOutput, KinematicsWorld, probe_kinematics};
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

// Adapted from the frozen smooth.py com_pos algebra. Apache-2.0 attribution
// and license terms below apply to both bounded rigid CUDA probes.
#[cfg(feature = "cuda-probe")]
const COM_POSITION_CUDA: &str = r#"
extern "C" __global__ void rigid_com_position(const int* m,const float* p,
    const float* state,float* output,unsigned nb,unsigned nj,unsigned nv,unsigned nw) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=nw) return;
  const int* parent=m; const int* type=m+nb; const int* body=type+nj;
  const int* adr=body+nj;
#ifdef MJWARP_FIELD_BATCHES
  const int* desc=adr+nj;
  const float* mass=field_parameter(p,desc,w,nb);
  const float* inertia=field_parameter(p,desc+2,w,3*nb);
#else
  const float* mass=p; const float* inertia=p+nb;
#endif
  const float* s=state+4+(unsigned long long)w*(28*nb+6*nj);
  const float* xmat=s+7*nb; const float* ipos=s+16*nb;
  const float* imat=s+19*nb; const float* anchor=s+28*nb;
  const float* axis=anchor+3*nj;
  float* sm=output+4+(unsigned long long)w*(14*nb+6*nv);
  float* com=sm+nb; float* ci=com+3*nb; float* cd=ci+10*nb;
  for(unsigned b=0;b<nb;b++) {
    sm[b]=mass[b];
    for(int a=0;a<3;a++) com[3*b+a]=mass[b]*ipos[3*b+a];
  }
  for(int b=int(nb)-1;b>0;b--) {
    int up=parent[b]; sm[up]+=sm[b];
    for(int a=0;a<3;a++) com[3*up+a]+=com[3*b+a];
  }
  for(unsigned b=0;b<nb;b++) if(sm[b]!=0.0f)
    for(int a=0;a<3;a++) com[3*b+a]/=sm[b];
  for(unsigned b=0;b<nb;b++) {
    unsigned root=b; while(parent[root]!=0) root=parent[root];
    float x=ipos[3*b]-com[3*root], y=ipos[3*b+1]-com[3*root+1];
    float z=ipos[3*b+2]-com[3*root+2], mb=mass[b];
    const float* r=imat+9*b; const float* d=inertia+3*b;
    float t[9];
    for(int row=0;row<3;row++) for(int col=0;col<3;col++) {
      float v=0.0f;
      for(int a=0;a<3;a++) v+=r[3*row+a]*d[a]*r[3*col+a];
      t[3*row+col]=v;
    }
    float* c=ci+10*b;
    c[0]=t[0]+mb*(y*y+z*z); c[1]=t[4]+mb*(x*x+z*z);
    c[2]=t[8]+mb*(x*x+y*y); c[3]=t[1]-mb*x*y;
    c[4]=t[2]-mb*x*z; c[5]=t[5]-mb*y*z;
    c[6]=mb*x; c[7]=mb*y; c[8]=mb*z; c[9]=mb;
  }
  for(unsigned j=0;j<nj;j++) {
    unsigned b=body[j],root=b; while(parent[root]!=0) root=parent[root];
    float off[3]; for(int a=0;a<3;a++) off[a]=com[3*root+a]-anchor[3*j+a];
    unsigned d=adr[j]; int ty=type[j];
    if(ty==0) {
      for(int a=0;a<3;a++) {
        float* c=cd+6*(d+a);
        for(int k=0;k<6;k++) c[k]=0.0f;
        c[3+a]=1.0f;
      }
      d+=3;
    }
    int count=(ty<=1)?3:1;
    for(int a=0;a<count;a++) {
      float ax[3]; for(int k=0;k<3;k++) ax[k]=(ty<=1)?xmat[9*b+3*k+a]:axis[3*j+k];
      float* c=cd+6*(d+a);
      if(ty==2) {
        for(int k=0;k<3;k++) { c[k]=0.0f; c[3+k]=ax[k]; }
      } else {
        for(int k=0;k<3;k++) c[k]=ax[k];
        c[3]=ax[1]*off[2]-ax[2]*off[1];
        c[4]=ax[2]*off[0]-ax[0]*off[2];
        c[5]=ax[0]*off[1]-ax[1]*off[0];
      }
    }
  }
}
"#;

// CUDA expressions follow the frozen Warp wxyz/row-major math conventions.
// The sequential body schedule is intentionally a correctness probe, not the
// upstream branch schedule or a selected production backend.
// Adapted from mujoco_warp/_src/smooth.py and math.py at
// 71da24d956378a87a703b6e1442b13aec0c4ac29.
// Copyright 2025 The Newton Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[cfg(feature = "cuda-probe")]
const KINEMATICS_CUDA: &str = r#"
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
// Frozen Warp math.quat_to_mat emits separate mul.f32/add.f32/sub.f32 (no
// explicit FMA); the driver JIT decides contraction on that exact sequence.
// Emitting the same unqualified PTX operations reproduces its rounding.
__device__ float umul(float a,float b) { float r; asm("mul.f32 %0,%1,%2;" : "=f"(r) : "f"(a),"f"(b)); return r; }
__device__ float uadd(float a,float b) { float r; asm("add.f32 %0,%1,%2;" : "=f"(r) : "f"(a),"f"(b)); return r; }
__device__ float usub(float a,float b) { float r; asm("sub.f32 %0,%1,%2;" : "=f"(r) : "f"(a),"f"(b)); return r; }
__device__ float ufma(float a,float b,float c) { float r; asm("fma.rn.f32 %0,%1,%2,%3;" : "=f"(r) : "f"(a),"f"(b),"f"(c)); return r; }
__device__ void matrix_from(float* m,float w,float x,float y,float z) {
  float q00=umul(w,w), q01=umul(w,x), q02=umul(w,y), q03=umul(w,z);
  float q11=umul(x,x), q12=umul(x,y), q13=umul(x,z);
  float q22=umul(y,y), q23=umul(y,z), q33=umul(z,z);
  m[0]=usub(usub(uadd(q00,q11),q22),q33);
  float t=usub(q12,q03); m[1]=uadd(t,t);
  t=uadd(q02,q13); m[2]=uadd(t,t);
  t=uadd(q12,q03); m[3]=uadd(t,t);
  m[4]=usub(uadd(usub(q00,q11),q22),q33);
  t=usub(q23,q01); m[5]=uadd(t,t);
  t=usub(q13,q02); m[6]=uadd(t,t);
  t=uadd(q01,q23); m[7]=uadd(t,t);
  t=usub(usub(q00,q11),q22); m[8]=uadd(t,q33);
}
__device__ void matrix(float* m,Q q) { matrix_from(m,q.w,q.x,q.y,q.z); }
// The inertial-frame kernel keeps the natural (compiler-contracted) form: it
// matches the frozen kernel bit-for-bit, unlike the body-matrix sequence above.
__device__ void matrix_natural(float* m,Q q) {
  m[0]=q.w*q.w+q.x*q.x-q.y*q.y-q.z*q.z;
  m[1]=2*(q.x*q.y-q.w*q.z); m[2]=2*(q.x*q.z+q.w*q.y);
  m[3]=2*(q.x*q.y+q.w*q.z); m[4]=q.w*q.w-q.x*q.x+q.y*q.y-q.z*q.z;
  m[5]=2*(q.y*q.z-q.w*q.x); m[6]=2*(q.x*q.z-q.w*q.y);
  m[7]=2*(q.y*q.z+q.w*q.x); m[8]=q.w*q.w-q.x*q.x-q.y*q.y+q.z*q.z;
}
// Frozen Warp mul_quat inside _compute_body_inertial_frames fuses a subset of
// the terms (PTX fma.rn) and leaves the rest as mul/sub; transcribe that exact
// operation order for the inertial-frame matrix.
__device__ void matrix_inertial(float* m,Q a,Q b) {
  float rw=usub(usub(usub(umul(a.w,b.w),umul(a.x,b.x)),umul(a.y,b.y)),umul(a.z,b.z));
  float rx=usub(ufma(a.y,b.z,ufma(a.x,b.w,umul(a.w,b.x))),umul(a.z,b.y));
  float ry=ufma(a.z,b.x,ufma(a.y,b.w,usub(umul(a.w,b.y),umul(a.x,b.z))));
  float rz=ufma(a.z,b.w,usub(ufma(a.x,b.y,umul(a.w,b.z)),umul(a.y,b.x)));
  matrix_from(m,rw,rx,ry,rz);
}
extern "C" __global__ void rigid_kinematics(const int* meta,const float* model,
    const float* state,float* result,unsigned nq,unsigned nb,unsigned nj,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* parent=meta; const int* adr=parent+nb; const int* num=adr+nb;
  const int* type=num+nb; const int* qa=type+nj;
#ifdef MJWARP_FIELD_BATCHES
  const int* desc=qa+nj;
#ifdef MJWARP_MOCAP
  unsigned nm=desc[0]; const int* mocap=desc+1;
  desc=mocap+nb;
  const float* mp=state+(unsigned long long)worlds*nq+(unsigned long long)w*3*nm;
  const float* mq=state+(unsigned long long)worlds*(nq+3*nm)+(unsigned long long)w*4*nm;
#endif
  const float* q0=field_parameter(model,desc,w,nq);
  const float* bp=field_parameter(model,desc+2,w,3*nb);
  const float* bq=field_parameter(model,desc+4,w,4*nb);
  const float* ip=field_parameter(model,desc+6,w,3*nb);
  const float* iq=field_parameter(model,desc+8,w,4*nb);
  const float* jp=field_parameter(model,desc+10,w,3*nj);
  const float* axis=field_parameter(model,desc+12,w,3*nj);
#else
  const float* q0=model; const float* bp=q0+nq; const float* bq=bp+3*nb;
  const float* ip=bq+4*nb; const float* iq=ip+3*nb;
  const float* jp=iq+4*nb; const float* axis=jp+3*nj;
#endif
  const float* q=state+(unsigned long long)w*nq;
  float* xp=result+4+(unsigned long long)w*(28*nb+6*nj);
  float* xq=xp+3*nb; float* xm=xq+4*nb; float* xi=xm+9*nb;
  float* im=xi+3*nb; float* anchor=im+9*nb; float* xa=anchor+3*nj;
  for(unsigned b=0;b<nb;++b) {
    V p={0,0,0}; Q r={1,0,0,0};
#ifdef MJWARP_G01_EQUIVALENT
    if(!b) { p=load_v(xp); r=load_q(xq); }
#endif
    int j=adr[b], count=num[b];
    if(b && count==1 && type[j]==0) {
      p=load_v(q+qa[j]); r=normalize(load_q(q+qa[j]+3));
      store_v(anchor+3*j,p); store_v(xa+3*j,load_v(axis+3*j));
    } else if(b) {
      Q pr=load_q(xq+4*parent[b]);
      V local_p=load_v(bp+3*b); Q local_q=load_q(bq+4*b);
#ifdef MJWARP_MOCAP
      if(mocap[b]>=0) { local_p=load_v(mp+3*mocap[b]); local_q=load_q(mq+4*mocap[b]); }
#endif
      p=add(load_v(xp+3*parent[b]),rotate(pr,local_p));
      r=multiply(pr,local_q);
      for(int k=0;k<count;++k,++j) {
        V local=load_v(jp+3*j), a=add(p,rotate(r,local));
        V ax=rotate(r,load_v(axis+3*j));
        if(type[j]==2) p=add(p,scale(ax,q[qa[j]]-q0[qa[j]]));
        else {
          Q delta;
          if(type[j]==1) delta=normalize(load_q(q+qa[j]));
          else {
            float h=0.5f*(q[qa[j]]-q0[qa[j]]), s=sinf(h);
            V t=scale(load_v(axis+3*j),s); delta={cosf(h),t.x,t.y,t.z};
          }
          r=multiply(r,delta); p=add(a,scale(rotate(r,local),-1));
        }
        store_v(anchor+3*j,a); store_v(xa+3*j,ax);
      }
      r=normalize(r);
    }
    store_v(xp+3*b,p); store_q(xq+4*b,r); matrix(xm+9*b,r);
    store_v(xi+3*b,add(p,rotate(r,load_v(ip+3*b))));
    matrix_natural(im+9*b,multiply(r,load_q(iq+4*b)));
  }
}
"#;
