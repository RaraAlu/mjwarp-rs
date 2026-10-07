//! 历史布局与刚体GPU辅助探针。
//! 不实现完整物理阶段。

use std::ops::Range;

use crate::diagnostics::{InputError, ProbeError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;

mod mass_matrix;
pub use mass_matrix::{MassMatrixOutput, MassMatrixWorld, probe_mass_matrix};
mod mass_solve;
pub use mass_solve::{MassSolveOutput, MassSolveWorld, probe_mass_solve};

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
        kinematics_device::<f32>(session, model, layout, qpos)
            .map(|(_, values)| KinematicsOutput { layout, values })
    }
}

#[cfg(feature = "cuda-probe")]
fn kinematics_device<T: RigidScalar>(
    session: &TransferSession,
    model: &InertialModelInput,
    layout: KinematicsLayout,
    qpos: &[f32],
) -> Result<(crate::runtime::TransferBuffer<T>, Vec<T>), TransferError> {
    let f = model.kinematics().fields();
    let i = model.fields();
    let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
    let mut parameters = crate::runtime::host_staging::<T>(layout.parameters)?;
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
        pack_rigid(&mut parameters[cursor..cursor + field.len()], field);
        cursor += field.len();
    }
    let metadata = session.upload(&metadata)?;
    let parameters = session.upload(&parameters)?;
    // A real one-element allocation supplies a valid ABI pointer for nq=0.
    let state = upload_rigid::<T>(session, if qpos.is_empty() { &[0.0] } else { qpos })?;
    let mut values = crate::runtime::host_staging::<T>(layout.guarded)?;
    values.fill(T::from(-131072.0));
    values[4..layout.guarded - 4].fill(T::from(f32::NAN));
    let mut output = session.upload(&values)?;
    let kernel = crate::runtime::SynchronousKernel::compile(
        session,
        &rigid_source::<T>(KINEMATICS_CUDA),
        "rigid_kinematics",
    )?;
    // SAFETY: Owned validated topology proves acyclic parent/joint/qpos
    // ranges. Checked packing matches the fixed CUDA ABI and every offset.
    // RigidScalar and rigid_source select the same pointer width for all buffers.
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
                layout.output.worlds() as u32,
            ],
        )?;
    }
    output.read_range_into(0, &mut values)?;
    check_device_values(&values, "kinematics_output")?;
    Ok((output, values))
}

/// 四项质心结果的只读世界视图。
/// cdof按角向量、线向量排列。
#[derive(Clone, Copy, Debug)]
pub struct ComPositionWorld<'a> {
    /// 每体子树质量，单位kg。
    pub subtree_mass: &'a [f32],
    /// 每体世界坐标质心，单位m。
    pub subtree_com: &'a [f32],
    /// 每体根子树质心坐标惯量。
    pub cinert: &'a [f32],
    /// 每自由度空间运动映射。
    pub cdof: &'a [f32],
}

/// 同步GPU质心结果。
/// cinert每体包含十项。
/// 前六项为xx,yy,zz,xy,xz,yz。
/// 后四项为质量一阶矩与质量。
#[derive(Clone, Debug)]
pub struct ComPositionOutput {
    layout: ComPositionLayout,
    values: Vec<f32>,
}

impl ComPositionOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<ComPositionWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let nb = self.nbody();
        Ok(ComPositionWorld {
            subtree_mass: &v[..nb],
            subtree_com: &v[nb..4 * nb],
            cinert: &v[4 * nb..14 * nb],
            cdof: &v[14 * nb..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ComPositionLayout {
    output: BatchLayout,
    nbody: usize,
    nv: usize,
    #[cfg(feature = "cuda-probe")]
    metadata: usize,
    #[cfg(feature = "cuda-probe")]
    parameters: usize,
    #[cfg(feature = "cuda-probe")]
    guarded: usize,
}

impl ComPositionLayout {
    fn new(worlds: usize, nbody: usize, njnt: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "com_position_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "com_position_layout",
        };
        let metadata = njnt
            .checked_mul(3)
            .and_then(|n| n.checked_add(nbody))
            .ok_or_else(overflow)?;
        let parameters = nbody.checked_mul(4).ok_or_else(overflow)?;
        let stride = nbody
            .checked_mul(14)
            .and_then(|n| nv.checked_mul(6).and_then(|d| n.checked_add(d)))
            .ok_or_else(overflow)?;
        if [metadata, parameters, stride]
            .iter()
            .any(|&n| n > i32::MAX as usize)
        {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, parameters, guarded);
        Ok(Self {
            output,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            parameters,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

fn check_com_position(
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<(KinematicsLayout, ComPositionLayout), InputError> {
    let fk = check_kinematics(model, worlds, qpos)?;
    for (field, values) in [
        ("body_mass", &model.fields().body_mass[..1]),
        ("body_inertia", &model.fields().body_inertia[..3]),
    ] {
        if values.iter().any(|&v| v != 0.0) {
            return Err(InputError::InvalidTopology {
                field,
                index: 0,
                reason: "nonzero_world_inertia",
            });
        }
    }
    let k = model.kinematics();
    Ok((
        fk,
        ComPositionLayout::new(worlds, k.nbody(), k.njnt(), k.nv())?,
    ))
}

/// G01质心子集的同步GPU探针。
/// 复用设备中的刚体运动学结果。
/// GPU推导子树质量与根体索引。
/// 零质量子树质心保持零。
/// 不采用原生CPU的阈值回退。
/// 世界体质量与惯量必须为零。
/// 惯量单位为kg*m^2。
/// 不处理休眠、柔性体或mocap。
/// 不替代U057完整等价入口。
/// 每次调用上传并同步回读。
/// 需要cuda-probe及NVRTC。
pub fn probe_com_position(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<ComPositionOutput, TransferError> {
    let (fk, layout) = check_com_position(model, worlds, qpos)?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (session, fk, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        com_position_device::<f32>(session, model, fk, layout, qpos)
            .map(|(_, values)| ComPositionOutput { layout, values })
    }
}

#[cfg(feature = "cuda-probe")]
fn com_position_device<T: RigidScalar>(
    session: &TransferSession,
    model: &InertialModelInput,
    fk: KinematicsLayout,
    layout: ComPositionLayout,
    qpos: &[f32],
) -> Result<(crate::runtime::TransferBuffer<T>, Vec<T>), TransferError> {
    // The validated FK output remains allocated on the same GPU session.
    // Its host checkpoint proves finite values; no CPU transform feeds COM.
    let (state, checkpoint) = kinematics_device::<T>(session, model, fk, qpos)?;
    drop(checkpoint);
    let k = model.kinematics();
    let f = k.fields();
    let i = model.fields();
    let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
    let mut cursor = 0;
    for field in [&f.body_parentid, &f.jnt_type, &f.jnt_bodyid, &f.jnt_dofadr] {
        metadata[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    let mut parameters = crate::runtime::host_staging::<T>(layout.parameters)?;
    pack_rigid(&mut parameters[..k.nbody()], &i.body_mass);
    pack_rigid(&mut parameters[k.nbody()..], &i.body_inertia);
    let metadata = session.upload(&metadata)?;
    let parameters = session.upload(&parameters)?;
    let mut values = crate::runtime::host_staging::<T>(layout.guarded)?;
    values.fill(T::from(-131072.0));
    values[4..layout.guarded - 4].fill(T::from(f32::NAN));
    let mut output = session.upload(&values)?;
    let kernel = crate::runtime::SynchronousKernel::compile(
        session,
        &rigid_source::<T>(COM_POSITION_CUDA),
        "rigid_com_position",
    )?;
    // SAFETY: Checked layouts bound all packed offsets by i32::MAX.
    // RigidScalar and rigid_source keep shader pointers and buffers equal-width.
    // Validated topology proves parent order and joint/body/DOF addresses.
    // FK state has precisely 28*nbody+6*njnt elements per world plus guards.
    // Each thread owns one output world, with no cross-thread reads/writes.
    // The same-session adapter waits before any buffer or module can drop.
    unsafe {
        kernel.launch(
            &metadata,
            &parameters,
            &state,
            &mut output,
            [
                k.nbody() as u32,
                k.njnt() as u32,
                k.nv() as u32,
                layout.output.worlds() as u32,
            ],
        )?;
    }
    output.read_range_into(0, &mut values)?;
    check_device_values(&values, "com_position_output")?;
    Ok((output, values))
}

/// Keep every buffer and shader pointer type equal. Public rigid inputs and
/// results stay f32; only the new solver selects the internal f64 pipeline.
#[cfg(feature = "cuda-probe")]
trait RigidScalar: crate::runtime::TransferElement + From<f32> + Into<f64> {}
#[cfg(feature = "cuda-probe")]
impl RigidScalar for f32 {}
#[cfg(feature = "cuda-probe")]
impl RigidScalar for f64 {}

#[cfg(feature = "cuda-probe")]
fn pack_rigid<T: RigidScalar>(destination: &mut [T], source: &[f32]) {
    for (d, &s) in destination.iter_mut().zip(source) {
        *d = T::from(s);
    }
}

#[cfg(feature = "cuda-probe")]
fn upload_rigid<T: RigidScalar>(
    session: &TransferSession,
    source: &[f32],
) -> Result<crate::runtime::TransferBuffer<T>, TransferError> {
    let mut values = crate::runtime::host_staging::<T>(source.len())?;
    pack_rigid(&mut values, source);
    session.upload(&values)
}

#[cfg(feature = "cuda-probe")]
fn rigid_source<T: RigidScalar>(source: &str) -> std::borrow::Cow<'_, str> {
    // Only the private fixed shader constants use this scalar specialization.
    if size_of::<T>() == 4 {
        std::borrow::Cow::Borrowed(source)
    } else {
        std::borrow::Cow::Owned(
            source
                .replace("float", "double")
                .replace("sqrtf", "sqrt")
                .replace("sinf", "sin")
                .replace("cosf", "cos"),
        )
    }
}

#[cfg(any(feature = "cuda-probe", test))]
fn check_device_values<T: Copy + Into<f64>>(
    values: &[T],
    field: &'static str,
) -> Result<(), TransferError> {
    check_device_guards(values)?;
    for (index, value) in values[4..values.len() - 4].iter().enumerate() {
        if !(*value).into().is_finite() {
            return Err(InputError::NonFinite { field, index }.into());
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda-probe", test))]
fn check_device_guards<T: Copy + Into<f64>>(values: &[T]) -> Result<(), TransferError> {
    // Private callers supply a checked payload layout and two four-value guards.
    debug_assert!(values.len() >= 8);
    for index in (0..4).chain(values.len() - 4..values.len()) {
        if values[index].into() != -131072.0 {
            return Err(ProbeError::Mismatch {
                index,
                expected: -131072.0,
                actual: values[index].into() as f32,
            }
            .into());
        }
    }
    Ok(())
}

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
  const float* mass=p; const float* inertia=p+nb;
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

    pub(super) fn test_model(joint: Option<i32>) -> InertialModelInput {
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
    fn rejects_com_position_capacity_and_invalid_world_inertia() {
        for dims in [
            (0, 1, 0, 0),
            (1, 0, 0, 0),
            (usize::MAX, 1, 0, 0),
            (1, usize::MAX, 0, 0),
            (1, 1, usize::MAX, 0),
            (1, 1, 0, usize::MAX),
            (1, 1, 0, i32::MAX as usize),
        ] {
            assert!(ComPositionLayout::new(dims.0, dims.1, dims.2, dims.3).is_err());
        }
        let m = test_model(Some(3));
        for field in ["body_mass", "body_inertia"] {
            let changed = edited(&m, |_, i| {
                if field == "body_mass" {
                    i.body_mass[0] = 1.0;
                } else {
                    i.body_inertia[0] = 1.0;
                }
            });
            assert!(matches!(
                check_com_position(&changed, 1, &[0.0]),
                Err(InputError::InvalidTopology {
                    reason: "nonzero_world_inertia",
                    ..
                })
            ));
            // The narrower COM contract does not restrict the existing FK helper.
            assert!(check_kinematics(&changed, 1, &[0.0]).is_ok());
        }
    }

    #[test]
    fn com_position_reuses_kinematics_checks_before_device_work() {
        let m = test_model(Some(3));
        assert!(check_com_position(&m, 2, &[0.0]).is_err());
        assert!(check_com_position(&m, 2, &[0.0, f32::NAN]).is_err());
        let changed = edited(&m, |k, _| k.jnt_axis.fill(0.0));
        assert!(check_com_position(&changed, 1, &[0.0]).is_err());
        assert!(check_com_position(&m, 2, &[0.0, 0.5]).is_ok());
    }

    #[test]
    fn com_position_views_keep_world_body_and_dof_ranges_separate() {
        let layout = ComPositionLayout::new(2, 2, 1, 1).unwrap();
        let out = ComPositionOutput {
            layout,
            values: (0..76).map(|n| n as f32).collect(),
        };
        assert_eq!(out.worlds(), 2);
        assert_eq!(out.nbody(), 2);
        assert_eq!(out.nv(), 1);
        let w = out.world(1).unwrap();
        assert_eq!(w.subtree_mass, &[38.0, 39.0]);
        assert_eq!(w.subtree_com, &[40.0, 41.0, 42.0, 43.0, 44.0, 45.0]);
        assert_eq!(w.cinert.len(), 20);
        assert_eq!(w.cdof, &[66.0, 67.0, 68.0, 69.0, 70.0, 71.0]);
        assert!(out.world(2).is_err());
        let layout = ComPositionLayout::new(1, 1, 0, 0).unwrap();
        let out = ComPositionOutput {
            layout,
            values: vec![0.0; 22],
        };
        assert!(out.world(0).unwrap().cdof.is_empty());
    }

    #[test]
    fn rejects_both_device_guards_and_incomplete_finite_payloads() {
        let mut v = vec![-131072.0; 12];
        v[4..8].fill(0.0);
        check_device_values(&v, "probe_output").unwrap();
        for index in [0, 3, 8, 11] {
            let mut changed = v.clone();
            changed[index] = 0.0;
            assert!(matches!(
                check_device_values(&changed, "probe_output"),
                Err(TransferError::Backend(ProbeError::Mismatch { .. }))
            ));
        }
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut changed = v.clone();
            changed[5] = bad;
            assert!(matches!(
                check_device_values(&changed, "probe_output"),
                Err(TransferError::Input(InputError::NonFinite { index: 1, .. }))
            ));
        }
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
