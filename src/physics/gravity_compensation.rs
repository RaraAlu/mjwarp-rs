//! 重力补偿设备子集。
//! 它借用常驻位姿与质心。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout, ParameterBatch};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer, TransferSession};
use std::sync::Arc;

use super::{KinematicsData, KinematicsPlan};

#[derive(Clone, Debug)]
pub struct GravityCompensationParameters {
    pub gravity: ParameterBatch,
    pub body_gravcomp: ParameterBatch,
    pub jnt_actgravcomp: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GravityCompensationOptions {
    pub disable_gravity: bool,
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    body_mass: ParameterBatch,
    parameters: GravityCompensationParameters,
    metadata: Vec<i32>,
    offsets: [usize; 3],
    parameter_len: usize,
    active: bool,
    nb: usize,
    nj: usize,
    nv: usize,
}

fn spec(
    model: Arc<AttachedModelInput>,
    body_mass: ParameterBatch,
    parameters: GravityCompensationParameters,
    options: GravityCompensationOptions,
) -> Result<Spec, InputError> {
    let k = model.rigid().kinematics();
    let (nb, nj, nv) = (k.nbody(), k.njnt(), k.nv());
    if nb < 2 || nv == 0 {
        return Err(InputError::InvalidDimension {
            field: "gravity_compensation_dimensions",
        });
    }
    for (field, actual, expected) in [
        ("opt_gravity", parameters.gravity.row_elements(), 3),
        ("body_mass", body_mass.row_elements(), nb),
        ("body_gravcomp", parameters.body_gravcomp.row_elements(), nb),
        ("jnt_actgravcomp", parameters.jnt_actgravcomp.len(), nj),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    for (index, &value) in parameters.jnt_actgravcomp.iter().enumerate() {
        if value != 0 && value != 1 {
            return Err(InputError::InvalidTopology {
                field: "jnt_actgravcomp",
                index,
                reason: "noncanonical_boolean",
            });
        }
    }
    let fields = [&parameters.gravity, &body_mass, &parameters.body_gravcomp];
    let mut offsets = [0; 3];
    let mut parameter_len = 0usize;
    for (index, field) in fields.iter().enumerate() {
        offsets[index] = parameter_len;
        parameter_len = parameter_len
            .checked_add(field.values().len())
            .filter(|&value| value <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_parameters",
            })?;
    }
    let topology = k.fields();
    let inertial = model.rigid().fields();
    let ancestry_len = nb.checked_mul(nv).ok_or(InputError::Overflow {
        field: "gravity_compensation_metadata",
    })?;
    let metadata_len = nb
        .checked_add(ancestry_len)
        .and_then(|value| value.checked_add(nv))
        .and_then(|value| value.checked_add(nj))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(nb)
        .map_err(|_| InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    roots.resize(nb, 0);
    for body in 1..nb {
        let mut root = body;
        while topology.body_parentid[root] != 0 {
            root = topology.body_parentid[root] as usize;
        }
        roots[body] = root as i32;
    }
    let mut ancestry = Vec::new();
    ancestry
        .try_reserve_exact(ancestry_len)
        .map_err(|_| InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    ancestry.resize(ancestry_len, 0);
    for body in 1..nb {
        for dof in 0..nv {
            let target = inertial.dof_bodyid[dof] as usize;
            let mut cursor = body;
            loop {
                if cursor == target {
                    ancestry[body * nv + dof] = 1;
                    break;
                }
                if cursor == 0 {
                    break;
                }
                cursor = topology.body_parentid[cursor] as usize;
            }
        }
    }
    let mut metadata = Vec::new();
    metadata
        .try_reserve_exact(metadata_len)
        .map_err(|_| InputError::Overflow {
            field: "gravity_compensation_metadata",
        })?;
    metadata.extend([-131072; 4]);
    metadata.extend(roots);
    metadata.extend(ancestry);
    metadata.extend(&inertial.dof_jntid);
    metadata.extend(&parameters.jnt_actgravcomp);
    metadata.extend([-131072; 4]);
    let active = !options.disable_gravity
        && parameters
            .body_gravcomp
            .values()
            .chunks_exact(nb)
            .any(|row| row[1..].iter().any(|&value| value != 0.0));
    Ok(Spec {
        body_mass,
        parameters,
        metadata,
        offsets,
        parameter_len,
        active,
        nb,
        nj,
        nv,
    })
}

fn layout(worlds: usize, nv: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || worlds
            .checked_mul(nv)
            .is_none_or(|value| value > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "gravity_compensation_work",
        });
    }
    let layout = BatchLayout::new(worlds, nv, 4)?;
    BatchLayout::new(
        1,
        layout
            .total_elements()
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_output",
            })?,
        4,
    )?;
    Ok(layout)
}

pub struct GravityCompensationPlan<'p> {
    source: &'p KinematicsPlan,
    spec: Arc<Spec>,
    options: GravityCompensationOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 4],
}

pub struct GravityCompensationData {
    spec: Arc<Spec>,
    layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

pub struct GravityCompensationView<'a> {
    data: &'a GravityCompensationData,
    _source: &'a KinematicsData,
}

#[derive(Clone, Debug)]
pub struct GravityCompensationSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl GravityCompensationSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.values[4 + range.start..4 + range.end])
    }
}

impl<'p> GravityCompensationPlan<'p> {
    pub fn new(
        source: &'p KinematicsPlan,
        parameters: GravityCompensationParameters,
        options: GravityCompensationOptions,
    ) -> Result<Self, TransferError> {
        let (session, model, body_mass) = source.gravity_compensation_context()?;
        let spec = Arc::new(spec(model, body_mass, parameters, options)?);
        layout(1, spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, spec);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut packed = vec![-131072.0; 4];
            for field in [
                &spec.parameters.gravity,
                &spec.body_mass,
                &spec.parameters.body_gravcomp,
            ] {
                packed.extend(field.values());
            }
            packed.extend([-131072.0; 4]);
            let batches = [
                spec.parameters.gravity.batches(),
                spec.body_mass.batches(),
                spec.parameters.body_gravcomp.batches(),
            ]
            .map(|value| value.to_string())
            .join(",");
            let offsets = spec.offsets.map(|value| value.to_string()).join(",");
            let rigid_stride = spec.nb * 28 + spec.nj * 6;
            let com_stride = spec.nb * 14 + spec.nv * 6;
            let code = format!(
                "#define GC_NB {}\n#define GC_NJ {}\n#define GC_NV {}\n#define GC_PL {}\n#define GC_RS {}\n#define GC_CS {}\n#define GC_ENABLED {}\n__device__ __constant__ unsigned GC_BATCH[3]={{{batches}}};\n__device__ __constant__ unsigned GC_OFF[3]={{{offsets}}};\n{CUDA}",
                spec.nb,
                spec.nj,
                spec.nv,
                spec.parameter_len,
                rigid_stride,
                com_stride,
                u32::from(!options.disable_gravity),
            );
            Ok(Self {
                source,
                spec: spec.clone(),
                options,
                metadata: session.upload(&spec.metadata)?,
                parameters: session.upload(&packed)?,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_validate")?,
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_clear")?,
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_compute")?,
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_sum")?,
                ],
                session,
            })
        }
    }

    pub fn options(&self) -> GravityCompensationOptions {
        self.options
    }

    pub fn create_data(&self, worlds: usize) -> Result<GravityCompensationData, TransferError> {
        let layout = layout(worlds, self.spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, value: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(value);
                self.session.upload(&values)
            };
            Ok(GravityCompensationData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                current: allocate(layout.total_elements(), f32::NAN)?,
                staged: allocate(layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    pub fn update<'a>(
        &'a self,
        source: &'a KinematicsData,
        data: &'a mut GravityCompensationData,
    ) -> Result<GravityCompensationView<'a>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec)
            || data.worlds() != source.worlds()
            || self.source.check_data(source).is_err()
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            self.write(source, &mut data.staged, &mut data.status, worlds)?;
            read_output(
                &data.staged,
                data.layout.total_elements(),
                "gravity_compensation_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(GravityCompensationView {
                data,
                _source: source,
            })
        }
    }

    pub(super) fn accepts_source(&self, source: &KinematicsData) -> bool {
        source.worlds() > 0 && self.source.check_data(source).is_ok()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn test_write_root(&mut self, body: usize, value: i32) -> Result<(), TransferError> {
        self.metadata.write_range(4 + body, &[value])
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn test_write_ancestor(
        &mut self,
        body: usize,
        dof: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.metadata
            .write_range(4 + self.spec.nb + body * self.spec.nv + dof, &[value])
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn test_write_dof_joint(
        &mut self,
        dof: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.metadata.write_range(
            4 + self.spec.nb + self.spec.nb * self.spec.nv + dof,
            &[value],
        )
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn write_shared(
        &self,
        source: &KinematicsData,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if source.worlds() != worlds
            || self.source.check_data(source).is_err()
            || target.len() != worlds * self.spec.nv + 8
            || status.len() != worlds + 8
        {
            return Err(TransferError::ModelMismatch);
        }
        self.write(source, target, status, worlds)
    }

    #[cfg(feature = "cuda-probe")]
    fn write(
        &self,
        source: &KinematicsData,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let slots = worlds
            .checked_mul(self.spec.nv)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_work",
            })?;
        let mut guards = [0.0; 8];
        target.read_range_into(0, &mut guards[..4])?;
        target.read_range_into(target.len() - 4, &mut guards[4..])?;
        crate::physics::check_device_guards(&guards)?;
        // SAFETY: 每线程独占一个输出槽。
        unsafe {
            self.kernels[1].launch(
                &self.metadata,
                &self.parameters,
                &self.parameters,
                target,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    slots as u32,
                ],
            )?;
        }
        if !self.spec.active {
            return Ok(());
        }
        let (rigid, com) = self.source.gravity_compensation_source(source)?;
        // SAFETY: 校验线程只写本世界状态。
        unsafe {
            self.kernels[0].launch_gravity_compensation(
                &self.metadata,
                &self.parameters,
                rigid,
                com,
                status,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    worlds as u32,
                ],
            )?;
        }
        let checked = read_output(status, worlds, "gravity_compensation_status")?;
        if let Some(index) = checked[4..4 + worlds]
            .iter()
            .position(|&value| value != 1.0)
        {
            return Err(InputError::InvalidTopology {
                field: "gravity_compensation_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        let work = worlds
            .checked_mul(self.spec.nb - 1)
            .and_then(|value| value.checked_mul(self.spec.nv))
            .filter(|&value| value <= (u32::MAX - 255) as usize)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_work",
            })?;
        // SAFETY: 每线程处理一个刚体自由度对。
        // 共享自由度继续使用设备原子加。
        unsafe {
            self.kernels[2].launch_gravity_compensation(
                &self.metadata,
                &self.parameters,
                rigid,
                com,
                target,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    work as u32,
                ],
            )
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn sum_shared(
        &self,
        parts: &TransferBuffer<f32>,
        gravity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let work = worlds
            .checked_mul(self.spec.nv)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_sum",
            })?;
        // SAFETY: 每线程独占一个最终被动力槽。
        unsafe {
            self.kernels[3].launch_gravity_compensation(
                &self.metadata,
                parts,
                gravity,
                gravity,
                output,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    work as u32,
                ],
            )
        }
    }
}

impl GravityCompensationData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    fn snapshot(&self) -> Result<GravityCompensationSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "gravity_compensation",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(GravityCompensationSnapshot {
                layout: self.layout,
                values: read_output(
                    &self.current,
                    self.layout.total_elements(),
                    "gravity_compensation_output",
                )?,
            })
        }
    }
}

impl GravityCompensationView<'_> {
    pub fn snapshot(&self) -> Result<GravityCompensationSnapshot, TransferError> {
        self.data.snapshot()
    }
}

#[cfg(feature = "cuda-probe")]
fn read_output(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, field)?;
    Ok(values)
}

// 冻结来源：passive.py与support.py。
// 基线：71da24d，许可：Apache-2.0。
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
__device__ const float* gc_field(const float* p,unsigned field,unsigned w,unsigned width){return p+4+GC_OFF[field]+(w%GC_BATCH[field])*width;}
__device__ bool gc_finite(float x){return isfinite(x);}
extern "C" __global__ void gravity_compensation_clear(const int* m,const float* p,const float* state,float* out,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;out[4+tid]=0.0f;
}
extern "C" __global__ void gravity_compensation_validate(const int* raw,const float* p,const float* rigid,const float* com,float* status,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=worlds)return;bool ok=true;
 const int* root=raw+4;const int* ancestor=root+GC_NB;const int* dof_jnt=ancestor+GC_NB*GC_NV;const int* act=dof_jnt+GC_NV;
 for(int i=0;i<4;i++)ok=ok&&raw[i]==-131072&&raw[4+GC_NB+GC_NB*GC_NV+GC_NV+GC_NJ+i]==-131072;
 for(int i=0;i<4;i++)ok=ok&&p[i]==-131072.0f&&p[4+GC_PL+i]==-131072.0f;
 for(int i=0;i<4;i++)ok=ok&&rigid[i]==-131072.0f&&rigid[4+(unsigned long long)worlds*GC_RS+i]==-131072.0f;
 for(int i=0;i<4;i++)ok=ok&&com[i]==-131072.0f&&com[4+(unsigned long long)worlds*GC_CS+i]==-131072.0f;
 for(unsigned b=0;b<GC_NB;b++)ok=ok&&root[b]>=0&&root[b]<(int)GC_NB;
 for(unsigned d=0;d<GC_NV;d++)ok=ok&&dof_jnt[d]>=0&&dof_jnt[d]<(int)GC_NJ;
 for(unsigned j=0;j<GC_NJ;j++)ok=ok&&(act[j]==0||act[j]==1);
 const float* gravity=gc_field(p,0,w,3);const float* mass=gc_field(p,1,w,GC_NB);const float* gravcomp=gc_field(p,2,w,GC_NB);
 const float* r=rigid+4+(unsigned long long)w*GC_RS;const float* xipos=r+16*GC_NB;
 const float* c=com+4+(unsigned long long)w*GC_CS;const float* subtree=c+GC_NB;const float* cdof=c+14*GC_NB;
 for(unsigned b=1;b<GC_NB;b++){float gc=gravcomp[b];ok=ok&&gc_finite(gc);if(gc==0.0f)continue;
  ok=ok&&gc_finite(gravity[0])&&gc_finite(gravity[1])&&gc_finite(gravity[2])&&gc_finite(mass[b]);
  int rb=root[b];bool root_ok=rb>=0&&rb<(int)GC_NB;ok=ok&&root_ok;
  if(root_ok)for(int a=0;a<3;a++)ok=ok&&gc_finite(xipos[3*b+a])&&gc_finite(subtree[3*rb+a]);
  for(unsigned d=0;d<GC_NV;d++){int affect=ancestor[b*GC_NV+d];ok=ok&&(affect==0||affect==1);if(affect)for(int a=0;a<6;a++)ok=ok&&gc_finite(cdof[6*d+a]);}
 }
 status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void gravity_compensation_compute(const int* raw,const float* p,const float* rigid,const float* com,float* out,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 unsigned d=tid%GC_NV;unsigned q=tid/GC_NV;unsigned b=1+q%(GC_NB-1);unsigned w=q/(GC_NB-1);
 const int* root=raw+4;const int* ancestor=root+GC_NB;
 const float* gravcomp=gc_field(p,2,w,GC_NB);float gc=gravcomp[b];if(gc==0.0f)return;
 const float* gravity=gc_field(p,0,w,3);const float* mass=gc_field(p,1,w,GC_NB);
 const float* r=rigid+4+(unsigned long long)w*GC_RS;const float* xipos=r+16*GC_NB;
 const float* c=com+4+(unsigned long long)w*GC_CS;const float* subtree=c+GC_NB;const float* cdof=c+14*GC_NB+6*d;
 float fx=(-gravity[0]*mass[b])*gc,fy=(-gravity[1]*mass[b])*gc,fz=(-gravity[2]*mass[b])*gc;
 float jx=0.0f,jy=0.0f,jz=0.0f;
 if(ancestor[b*GC_NV+d]){
  float ox=xipos[3*b]-subtree[3*root[b]],oy=xipos[3*b+1]-subtree[3*root[b]+1],oz=xipos[3*b+2]-subtree[3*root[b]+2];
  jx=cdof[3]+(cdof[1]*oz-cdof[2]*oy);jy=cdof[4]+(cdof[2]*ox-cdof[0]*oz);jz=cdof[5]+(cdof[0]*oy-cdof[1]*ox);
 }
 atomicAdd(out+4+(unsigned long long)w*GC_NV+d,jx*fx+jy*fy+jz*fz);
}
extern "C" __global__ void gravity_compensation_sum(const int* raw,const float* parts,const float* gravity,const float* unused,float* out,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;
 const int* root=raw+4;const int* dof_jnt=root+GC_NB+GC_NB*GC_NV;const int* act=dof_jnt+GC_NV;
 float value=__fadd_rn(parts[4+tid],parts[4+(unsigned long long)worlds*GC_NV+tid]);
 if(GC_ENABLED&&!act[dof_jnt[tid%GC_NV]])value=__fadd_rn(value,gravity[4+tid]);out[4+tid]=value;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
        KinematicModelInput,
    };

    fn source_model() -> AttachedModelInput {
        let kinematic = KinematicModelInput::new(
            1,
            KinematicFields {
                qpos0: vec![0.0],
                body_parentid: vec![0, 0],
                body_jntadr: vec![-1, 0],
                body_jntnum: vec![0, 1],
                body_pos: vec![0.0; 6],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
                jnt_type: vec![2],
                jnt_bodyid: vec![1],
                jnt_qposadr: vec![0],
                jnt_dofadr: vec![0],
                jnt_pos: vec![0.0; 3],
                jnt_axis: vec![1.0, 0.0, 0.0],
            },
        )
        .unwrap();
        let inertial = InertialModelInput::new(
            kinematic,
            InertialFields {
                body_ipos: vec![0.0; 6],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
                body_mass: vec![0.0, 1.0],
                body_inertia: vec![0.0; 6],
                dof_bodyid: vec![1],
                dof_jntid: vec![0],
                dof_parentid: vec![-1],
                dof_armature: vec![0.0],
                dof_damping: vec![0.0],
            },
        )
        .unwrap();
        AttachedModelInput::new(inertial, AttachedFields::default()).unwrap()
    }

    #[test]
    fn host_derives_roots_ancestry_and_independent_periods() {
        let model = Arc::new(source_model());
        let params = GravityCompensationParameters {
            gravity: ParameterBatch::new(2, 3, vec![0.0; 6]).unwrap(),
            body_gravcomp: ParameterBatch::new(5, 2, vec![0.5; 10]).unwrap(),
            jnt_actgravcomp: vec![1],
        };
        let body_mass = ParameterBatch::new(3, 2, vec![1.0; 6]).unwrap();
        let value = spec(
            model,
            body_mass,
            params,
            GravityCompensationOptions::default(),
        )
        .unwrap();
        assert_eq!(value.offsets, [0, 6, 12]);
        assert_eq!(value.parameter_len, 22);
        assert_eq!(&value.metadata[4..6], &[0, 1]);
        assert_eq!(&value.metadata[6..8], &[0, 1]);
        assert!(value.active);
        assert!(layout(257, 1).is_ok());
        assert!(layout(0, 1).is_err());
        assert!(layout(1, usize::MAX).is_err());
    }

    #[test]
    fn host_rejects_shapes_and_noncanonical_actuator_flags() {
        let model = Arc::new(source_model());
        let body_mass = ParameterBatch::new(1, 2, vec![0.0, 1.0]).unwrap();
        for bad in [
            GravityCompensationParameters {
                gravity: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
                body_gravcomp: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
                jnt_actgravcomp: vec![0],
            },
            GravityCompensationParameters {
                gravity: ParameterBatch::new(1, 3, vec![0.0; 3]).unwrap(),
                body_gravcomp: ParameterBatch::new(1, 1, vec![0.0]).unwrap(),
                jnt_actgravcomp: vec![0],
            },
            GravityCompensationParameters {
                gravity: ParameterBatch::new(1, 3, vec![0.0; 3]).unwrap(),
                body_gravcomp: ParameterBatch::new(1, 2, vec![0.0; 2]).unwrap(),
                jnt_actgravcomp: vec![2],
            },
        ] {
            assert!(
                spec(
                    model.clone(),
                    body_mass.clone(),
                    bad,
                    GravityCompensationOptions::default(),
                )
                .is_err()
            );
        }
    }
}
