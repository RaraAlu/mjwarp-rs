//! 历史布局与刚体运动学探针。
//! 不实现完整物理阶段。

use std::ops::Range;

use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;

/// 七项GPU结果的只读世界视图。
/// 矩阵按行展开，四元数用wxyz。
#[derive(Clone, Copy, Debug)]
pub struct KinematicsWorld<'a> {
    pub xpos: &'a [f32],
    pub xquat: &'a [f32],
    pub xmat: &'a [f32],
    pub xipos: &'a [f32],
    pub ximat: &'a [f32],
    pub xanchor: &'a [f32],
    pub xaxis: &'a [f32],
}

/// 同步完成后的宿主结果。
/// 宿主不计算任何运动学。
#[derive(Clone, Debug)]
pub struct KinematicsOutput {
    layout: KinematicsLayout,
    values: Vec<f32>,
}

impl KinematicsOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn njnt(&self) -> usize {
        self.layout.njnt
    }
    pub fn world(&self, world: usize) -> Result<KinematicsWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let nb = self.nbody();
        let nj = self.njnt();
        Ok(KinematicsWorld {
            xpos: &v[..3 * nb],
            xquat: &v[3 * nb..7 * nb],
            xmat: &v[7 * nb..16 * nb],
            xipos: &v[16 * nb..19 * nb],
            ximat: &v[19 * nb..28 * nb],
            xanchor: &v[28 * nb..28 * nb + 3 * nj],
            xaxis: &v[28 * nb + 3 * nj..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct KinematicsLayout {
    output: BatchLayout,
    nbody: usize,
    njnt: usize,
    #[cfg(feature = "cuda-probe")]
    nq: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: usize,
    #[cfg(feature = "cuda-probe")]
    parameters: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl KinematicsLayout {
    fn new(worlds: usize, nq: usize, nbody: usize, njnt: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "kinematics_dimensions",
            });
        }
        fn packed(terms: &[(usize, usize)]) -> Result<usize, InputError> {
            let n = terms.iter().try_fold(0usize, |n, &(count, width)| {
                count
                    .checked_mul(width)
                    .and_then(|k| n.checked_add(k))
                    .ok_or(InputError::Overflow {
                        field: "kinematics_layout",
                    })
            })?;
            if n > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "kinematics_kernel_index",
                });
            }
            Ok(n)
        }
        let metadata = packed(&[(nbody, 3), (njnt, 2)])?;
        let parameters = packed(&[(nq, 1), (nbody, 14), (njnt, 6)])?;
        let stride = packed(&[(nbody, 28), (njnt, 6)])?;
        let output = BatchLayout::new(worlds, stride, 4)?;
        BatchLayout::new(worlds, nq, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "kinematics_guards",
            })?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, parameters, guarded);
        Ok(Self {
            output,
            nbody,
            njnt,
            #[cfg(feature = "cuda-probe")]
            nq,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            parameters,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

fn norm_squared(values: &[f32]) -> f64 {
    values.iter().map(|&x| f64::from(x).powi(2)).sum()
}

fn check_kinematics(
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsLayout, InputError> {
    let k = model.kinematics();
    let f = k.fields();
    let layout = KinematicsLayout::new(worlds, k.nq(), k.nbody(), k.njnt())?;
    let expected = worlds.checked_mul(k.nq()).ok_or(InputError::Overflow {
        field: "kinematics_qpos",
    })?;
    if qpos.len() != expected {
        return Err(InputError::LengthMismatch {
            field: "kinematics_qpos",
            expected,
            actual: qpos.len(),
        });
    }
    for (index, value) in qpos.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "kinematics_qpos",
                index,
            });
        }
    }
    // This strict helper does not silently reinterpret noncanonical world input.
    for (field, values, required) in [
        ("body_pos", &f.body_pos[..3], &[0.0, 0.0, 0.0][..]),
        ("body_quat", &f.body_quat[..4], &[1.0, 0.0, 0.0, 0.0][..]),
        (
            "body_ipos",
            &model.fields().body_ipos[..3],
            &[0.0, 0.0, 0.0][..],
        ),
        (
            "body_iquat",
            &model.fields().body_iquat[..4],
            &[1.0, 0.0, 0.0, 0.0][..],
        ),
    ] {
        if values != required {
            return Err(InputError::InvalidTopology {
                field,
                index: 0,
                reason: "noncanonical_world",
            });
        }
    }
    for (field, values, width) in [
        ("body_quat", &f.body_quat, 4),
        ("body_iquat", &model.fields().body_iquat, 4),
    ] {
        for (index, rotation) in values.chunks_exact(width).enumerate() {
            if (norm_squared(rotation) - 1.0).abs() > 2e-6 {
                return Err(InputError::InvalidTopology {
                    field,
                    index,
                    reason: "nonunit_model_rotation",
                });
            }
        }
    }
    for joint in 0..k.njnt() {
        let ty = f.jnt_type[joint];
        let a = f.jnt_qposadr[joint] as usize;
        if ty >= 2 {
            if (norm_squared(&f.jnt_axis[3 * joint..3 * joint + 3]) - 1.0).abs() > 2e-6 {
                return Err(InputError::InvalidTopology {
                    field: "jnt_axis",
                    index: joint,
                    reason: "nonunit_joint_axis",
                });
            }
        } else {
            let offset = a + if ty == 0 { 3 } else { 0 };
            for world in 0..worlds {
                let start = world * k.nq() + offset;
                let squared = norm_squared(&qpos[start..start + 4]);
                if !(1e-12..=1e12).contains(&squared) {
                    return Err(InputError::InvalidTopology {
                        field: "kinematics_qpos",
                        index: start,
                        reason: "unusable_state_quaternion",
                    });
                }
            }
        }
    }
    Ok(layout)
}

/// G01刚体子集的同步GPU探针。
/// 参数仅共享一份，状态按世界展开。
/// 支持四种关节与静态祖先。
/// 非零状态四元数在GPU归一化。
/// 位置用米，角度用弧度。
/// 只计算七项体与关节字段。
/// 不处理mocap、休眠或柔性体。
/// 不输出geom、site或相机姿态。
/// 不替代U018、U062等价入口。
/// 每次调用上传并同步回读。
/// 需要cuda-probe及NVRTC。
pub fn probe_kinematics(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsOutput, TransferError> {
    let layout = check_kinematics(model, worlds, qpos)?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (session, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        let f = model.kinematics().fields();
        let i = model.fields();
        let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
        let mut parameters = crate::runtime::host_staging::<f32>(layout.parameters)?;
        let mut cursor = 0;
        for field in [
            &f.body_parentid,
            &f.body_jntadr,
            &f.body_jntnum,
            &f.jnt_type,
            &f.jnt_qposadr,
        ] {
            metadata[cursor..cursor + field.len()].copy_from_slice(field);
            cursor += field.len();
        }
        cursor = 0;
        for field in [
            &f.qpos0,
            &f.body_pos,
            &f.body_quat,
            &i.body_ipos,
            &i.body_iquat,
            &f.jnt_pos,
            &f.jnt_axis,
        ] {
            parameters[cursor..cursor + field.len()].copy_from_slice(field);
            cursor += field.len();
        }
        let metadata = session.upload(&metadata)?;
        let parameters = session.upload(&parameters)?;
        // A real one-element allocation supplies a valid ABI pointer for nq=0.
        let state = session.upload(if qpos.is_empty() { &[0.0] } else { qpos })?;
        let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
        values.fill(-131072.0);
        values[4..layout.guarded - 4].fill(f32::NAN);
        let mut output = session.upload(&values)?;
        let kernel = crate::runtime::SynchronousKernel::compile(
            session,
            KINEMATICS_CUDA,
            "rigid_kinematics",
        )?;
        // SAFETY: Owned validated topology proves acyclic parent/joint/qpos
        // ranges. Checked packing matches the fixed CUDA ABI and every offset.
        // One thread owns each world and processes parents before children.
        // Guards fit the output allocation; session launch waits for completion.
        unsafe {
            kernel.launch(
                &metadata,
                &parameters,
                &state,
                &mut output,
                [
                    layout.nq as u32,
                    layout.nbody as u32,
                    layout.njnt as u32,
                    worlds as u32,
                ],
            )?;
        }
        output.read_range_into(0, &mut values)?;
        for index in (0..4).chain(layout.guarded - 4..layout.guarded) {
            if values[index] != -131072.0 {
                return Err(ProbeError::Mismatch {
                    index,
                    expected: -131072.0,
                    actual: values[index],
                }
                .into());
            }
        }
        for (index, value) in values[4..layout.guarded - 4].iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "kinematics_output",
                    index,
                }
                .into());
            }
        }
        Ok(KinematicsOutput { layout, values })
    }
}

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
__device__ Q normalize(Q q) {
  float s=1.0f/sqrtf(q.w*q.w+q.x*q.x+q.y*q.y+q.z*q.z);
  return {q.w*s,q.x*s,q.y*s,q.z*s};
}
__device__ Q multiply(Q a,Q b) {
  return {a.w*b.w-a.x*b.x-a.y*b.y-a.z*b.z,
          a.w*b.x+a.x*b.w+a.y*b.z-a.z*b.y,
          a.w*b.y-a.x*b.z+a.y*b.w+a.z*b.x,
          a.w*b.z+a.x*b.y-a.y*b.x+a.z*b.w};
}
__device__ V rotate(Q q,V v) {
  float d=q.x*v.x+q.y*v.y+q.z*v.z;
  float s=q.w*q.w-q.x*q.x-q.y*q.y-q.z*q.z;
  return {s*v.x+2*d*q.x+2*q.w*(q.y*v.z-q.z*v.y),
          s*v.y+2*d*q.y+2*q.w*(q.z*v.x-q.x*v.z),
          s*v.z+2*d*q.z+2*q.w*(q.x*v.y-q.y*v.x)};
}
__device__ void matrix(float* m,Q q) {
  m[0]=q.w*q.w+q.x*q.x-q.y*q.y-q.z*q.z;
  m[1]=2*(q.x*q.y-q.w*q.z); m[2]=2*(q.x*q.z+q.w*q.y);
  m[3]=2*(q.x*q.y+q.w*q.z); m[4]=q.w*q.w-q.x*q.x+q.y*q.y-q.z*q.z;
  m[5]=2*(q.y*q.z-q.w*q.x); m[6]=2*(q.x*q.z-q.w*q.y);
  m[7]=2*(q.y*q.z+q.w*q.x); m[8]=q.w*q.w-q.x*q.x-q.y*q.y+q.z*q.z;
}
extern "C" __global__ void rigid_kinematics(const int* meta,const float* model,
    const float* state,float* result,unsigned nq,unsigned nb,unsigned nj,unsigned worlds) {
  unsigned w=blockIdx.x*blockDim.x+threadIdx.x;
  if(w>=worlds) return;
  const int* parent=meta; const int* adr=parent+nb; const int* num=adr+nb;
  const int* type=num+nb; const int* qa=type+nj;
  const float* q0=model; const float* bp=q0+nq; const float* bq=bp+3*nb;
  const float* ip=bq+4*nb; const float* iq=ip+3*nb;
  const float* jp=iq+4*nb; const float* axis=jp+3*nj;
  const float* q=state+(unsigned long long)w*nq;
  float* xp=result+4+(unsigned long long)w*(28*nb+6*nj);
  float* xq=xp+3*nb; float* xm=xq+4*nb; float* xi=xm+9*nb;
  float* im=xi+3*nb; float* anchor=im+9*nb; float* xa=anchor+3*nj;
  for(unsigned b=0;b<nb;++b) {
    V p={0,0,0}; Q r={1,0,0,0};
    int j=adr[b], count=num[b];
    if(b && count==1 && type[j]==0) {
      p=load_v(q+qa[j]); r=normalize(load_q(q+qa[j]+3));
      store_v(anchor+3*j,p); store_v(xa+3*j,load_v(axis+3*j));
    } else if(b) {
      Q pr=load_q(xq+4*parent[b]);
      p=add(load_v(xp+3*parent[b]),rotate(pr,load_v(bp+3*b)));
      r=multiply(pr,load_q(bq+4*b));
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
    matrix(im+9*b,multiply(r,load_q(iq+4*b)));
  }
}
"#;

/// 连续历史布局 `[world, sample, channel]`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryLayout {
    batch: BatchLayout,
    samples: usize,
    channels: usize,
}

impl HistoryLayout {
    pub fn new(worlds: usize, samples: usize, channels: usize) -> Result<Self, InputError> {
        if samples == 0 {
            return Err(InputError::InvalidDimension { field: "samples" });
        }
        if channels == 0 {
            return Err(InputError::InvalidDimension { field: "channels" });
        }
        let elements = samples.checked_mul(channels).ok_or(InputError::Overflow {
            field: "history_layout",
        })?;
        Ok(Self {
            batch: BatchLayout::new(worlds, elements, size_of::<f32>())?,
            samples,
            channels,
        })
    }

    pub fn batch(self) -> BatchLayout {
        self.batch
    }

    pub fn samples(self) -> usize {
        self.samples
    }

    pub fn channels(self) -> usize {
        self.channels
    }

    pub fn sample_range(self, world: usize, sample: usize) -> Result<Range<usize>, InputError> {
        let world_range = self.batch.world_elements(world)?;
        if sample >= self.samples {
            return Err(InputError::InvalidIndex {
                field: "sample",
                index: sample,
                limit: self.samples,
            });
        }
        let start = world_range.start + sample * self.channels;
        Ok(start..start + self.channels)
    }

    /// 调用方显式提供原生最小间隔。
    /// 允许间隔恰好等于阈值。
    /// 不改写任何历史槽位。
    pub fn validate_times(self, times: &[f64], minimum_interval: f64) -> Result<(), InputError> {
        if !minimum_interval.is_finite() || minimum_interval <= 0.0 {
            return Err(InputError::InvalidDimension {
                field: "minimum_interval",
            });
        }
        if times.len() != self.samples {
            return Err(InputError::LengthMismatch {
                field: "history_times",
                expected: self.samples,
                actual: times.len(),
            });
        }
        for (index, time) in times.iter().enumerate() {
            if !time.is_finite() {
                return Err(InputError::NonFinite {
                    field: "history_times",
                    index,
                });
            }
        }
        for (index, pair) in times.windows(2).enumerate() {
            let interval = pair[1] - pair[0];
            if !interval.is_finite() || interval < minimum_interval {
                return Err(InputError::InvalidHistoryTime { index: index + 1 });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model(joint: Option<i32>) -> InertialModelInput {
        use crate::model::{InertialFields, KinematicFields, KinematicModelInput};
        let (nq, nv) = match joint {
            Some(0) => (7, 6),
            Some(1) => (4, 3),
            Some(_) => (1, 1),
            None => (0, 0),
        };
        let mut k = KinematicFields {
            qpos0: vec![0.0; nq],
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, if joint.is_some() { 0 } else { -1 }],
            body_jntnum: vec![0, i32::from(joint.is_some())],
            body_pos: vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            ..Default::default()
        };
        if let Some(ty) = joint {
            k.jnt_type.push(ty);
            k.jnt_bodyid.push(1);
            k.jnt_qposadr.push(0);
            k.jnt_dofadr.push(0);
            k.jnt_pos.extend([0.125, 0.25, 0.5]);
            k.jnt_axis.extend([0.0, 0.0, 1.0]);
            if ty < 2 {
                k.qpos0[if ty == 0 { 3 } else { 0 }] = 1.0;
            }
        }
        let i = InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            body_mass: vec![0.0, 1.0],
            body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            dof_bodyid: vec![1; nv],
            dof_jntid: vec![0; nv],
            dof_parentid: (0..nv).map(|n| n as i32 - 1).collect(),
            dof_armature: vec![0.0; nv],
            dof_damping: vec![0.0; nv],
        };
        InertialModelInput::new(KinematicModelInput::new(nv, k).unwrap(), i).unwrap()
    }

    fn edited(
        model: &InertialModelInput,
        edit: impl FnOnce(&mut crate::model::KinematicFields, &mut crate::model::InertialFields),
    ) -> InertialModelInput {
        let mut k = model.kinematics().fields().clone();
        let mut i = model.fields().clone();
        edit(&mut k, &mut i);
        InertialModelInput::new(
            crate::model::KinematicModelInput::new(model.kinematics().nv(), k).unwrap(),
            i,
        )
        .unwrap()
    }

    #[test]
    fn rejects_kinematics_dimensions_capacity_and_state_length() {
        // G01 subset: no full-stage acceptance.
        let m = test_model(Some(3));
        for worlds in [0, usize::MAX, u32::MAX as usize] {
            assert!(check_kinematics(&m, worlds, &[]).is_err());
        }
        for dimensions in [
            (1, usize::MAX, 2, 0),
            (1, 0, usize::MAX, 0),
            (1, 0, 1, usize::MAX),
            (1, 0, 0, 0),
        ] {
            assert!(
                KinematicsLayout::new(dimensions.0, dimensions.1, dimensions.2, dimensions.3)
                    .is_err()
            );
        }
        assert!(matches!(
            check_kinematics(&m, 2, &[0.0]),
            Err(InputError::LengthMismatch {
                expected: 2,
                actual: 1,
                ..
            })
        ));
        assert!(check_kinematics(&m, 2, &[0.0, 0.5]).is_ok());
    }

    #[test]
    fn rejects_nonfinite_state_in_every_world() {
        let m = test_model(Some(3));
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(matches!(
                check_kinematics(&m, 2, &[0.0, bad]),
                Err(InputError::NonFinite { index: 1, .. })
            ));
        }
    }

    #[test]
    fn rejects_noncanonical_world_and_nonunit_model_rotations() {
        let m = test_model(Some(3));
        for n in 0..6 {
            let changed = edited(&m, |k, i| match n {
                0 => k.body_pos[0] = 0.5,
                1 => k.body_quat[0] = -1.0,
                2 => i.body_ipos[0] = 0.5,
                3 => i.body_iquat[0] = -1.0,
                4 => k.body_quat[4] = 2.0,
                _ => i.body_iquat[4] = 0.0,
            });
            assert!(check_kinematics(&changed, 1, &[0.0]).is_err());
        }
    }

    #[test]
    fn requires_unit_slide_and_hinge_axes() {
        for ty in [2, 3] {
            let m = test_model(Some(ty));
            let changed = edited(&m, |k, _| k.jnt_axis.fill(0.0));
            assert!(matches!(
                check_kinematics(&changed, 1, &[0.0]),
                Err(InputError::InvalidTopology {
                    field: "jnt_axis",
                    ..
                })
            ));
        }
    }

    #[test]
    fn preserves_usable_free_and_ball_quaternions() {
        for ty in [0, 1] {
            let m = test_model(Some(ty));
            let mut q = m.kinematics().fields().qpos0.repeat(2);
            let a = m.kinematics().nq() + if ty == 0 { 3 } else { 0 };
            for bad in [0.0, 1e-8, 1e8] {
                q[a..a + 4].fill(0.0);
                q[a] = bad;
                assert!(
                    matches!(check_kinematics(&m,2,&q),Err(InputError::InvalidTopology {index,..}) if index==a)
                );
            }
            q[a..a + 4].copy_from_slice(&[-2.0, 1.0, 0.0, 0.0]);
            let original = q.clone();
            check_kinematics(&m, 2, &q).unwrap();
            assert_eq!(q, original);
        }
    }

    #[test]
    fn supports_static_zero_dof_shapes_and_checked_output_views() {
        let m = test_model(None);
        let layout = check_kinematics(&m, 2, &[]).unwrap();
        assert_eq!(layout.output.total_elements(), 112);
        let out = KinematicsOutput {
            layout,
            values: (0..120).map(|n| n as f32).collect(),
        };
        let w = out.world(1).unwrap();
        assert_eq!(w.xpos, &[60.0, 61.0, 62.0, 63.0, 64.0, 65.0]);
        assert_eq!(w.xquat.len(), 8);
        assert_eq!(w.xmat.len(), 18);
        assert_eq!(w.xipos.len(), 6);
        assert_eq!(w.ximat.len(), 18);
        assert!(w.xanchor.is_empty() && w.xaxis.is_empty());
        assert!(out.world(2).is_err());
    }

    #[test]
    fn locates_flattened_history_samples() {
        // MC-38/MC-39：仅检查宿主范围。
        let layout = HistoryLayout::new(2, 3, 4).unwrap();
        assert_eq!(layout.samples(), 3);
        assert_eq!(layout.channels(), 4);
        assert_eq!(layout.batch().total_bytes(), 96);
        assert_eq!(layout.sample_range(1, 2).unwrap(), 20..24);
    }

    #[test]
    fn rejects_history_dimensions_and_indices() {
        for dimensions in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (1, usize::MAX, 2)] {
            assert!(HistoryLayout::new(dimensions.0, dimensions.1, dimensions.2).is_err());
        }
        let layout = HistoryLayout::new(2, 3, 4).unwrap();
        assert!(layout.sample_range(2, 0).is_err());
        assert_eq!(
            layout.sample_range(0, 3),
            Err(InputError::InvalidIndex {
                field: "sample",
                index: 3,
                limit: 3
            })
        );
    }

    #[test]
    fn accepts_exact_minimum_interval_and_single_sample() {
        // MC-38：间隔不能小于原生阈值。
        let layout = HistoryLayout::new(2, 3, 1).unwrap();
        layout.validate_times(&[-0.25, 0.0, 0.5], 0.25).unwrap();
        HistoryLayout::new(1, 1, 1)
            .unwrap()
            .validate_times(&[0.0], 0.25)
            .unwrap();
    }

    #[test]
    fn rejects_short_duplicate_decreasing_and_overflowing_intervals() {
        let layout = HistoryLayout::new(1, 2, 1).unwrap();
        for times in [[0.0, 0.125], [1.0, 1.0], [1.0, 0.0], [-f64::MAX, f64::MAX]] {
            assert_eq!(
                layout.validate_times(&times, 0.25),
                Err(InputError::InvalidHistoryTime { index: 1 })
            );
        }
    }

    #[test]
    fn rejects_invalid_times_length_and_threshold() {
        let layout = HistoryLayout::new(1, 2, 1).unwrap();
        assert_eq!(
            layout.validate_times(&[0.0], 0.25),
            Err(InputError::LengthMismatch {
                field: "history_times",
                expected: 2,
                actual: 1
            })
        );
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                layout.validate_times(&[0.0, invalid], 0.25),
                Err(InputError::NonFinite {
                    field: "history_times",
                    index: 1
                })
            );
        }
        for threshold in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(layout.validate_times(&[0.0, 1.0], threshold).is_err());
        }
    }
}
