//! 惯性盒流体被动力设备子集。
//! 椭球流体模型仍显式拒绝。

use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::com_velocity::FluidInputs;
use super::com_velocity::{ResidentComVelocityData, ResidentComVelocityPlan};
use super::{ComVelocityData, ComVelocityPlan, KinematicsData, KinematicsPlan};
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{AttachedModelInput, BatchLayout, ParameterBatch};
use crate::runtime::TransferSession;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};

#[derive(Clone, Debug)]
pub struct FluidForceInput {
    pub wind: ParameterBatch,
    pub density: ParameterBatch,
    pub viscosity: ParameterBatch,
    pub body_fluid_ellipsoid: Vec<bool>,
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    body_mass: ParameterBatch,
    body_inertia: ParameterBatch,
    input: FluidForceInput,
    metadata: Vec<i32>,
    offsets: [usize; 5],
    parameter_len: usize,
    nb: usize,
    nj: usize,
    nv: usize,
}

fn spec(
    model: Arc<AttachedModelInput>,
    body_mass: ParameterBatch,
    body_inertia: ParameterBatch,
    input: FluidForceInput,
) -> Result<Spec, InputError> {
    let kinematic = model.rigid().kinematics();
    let inertial = model.rigid().fields();
    let (nb, nj, nv) = (kinematic.nbody(), kinematic.njnt(), kinematic.nv());
    if nb < 2 || nv == 0 {
        return Err(InputError::InvalidDimension {
            field: "fluid_force_dimensions",
        });
    }
    for (field, actual, expected) in [
        ("opt_wind", input.wind.row_elements(), 3),
        ("opt_density", input.density.row_elements(), 1),
        ("opt_viscosity", input.viscosity.row_elements(), 1),
        ("body_mass", body_mass.row_elements(), nb),
        ("body_inertia", body_inertia.row_elements(), 3 * nb),
        ("body_fluid_ellipsoid", input.body_fluid_ellipsoid.len(), nb),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    if let Some(index) = input.body_fluid_ellipsoid.iter().position(|&value| value) {
        return Err(InputError::InvalidTopology {
            field: "body_fluid_ellipsoid",
            index,
            reason: "ellipsoid_fluid_not_implemented",
        });
    }

    let fields = [
        &input.wind,
        &input.density,
        &input.viscosity,
        &body_mass,
        &body_inertia,
    ];
    let mut offsets = [0; 5];
    let mut parameter_len = 0usize;
    for (index, field) in fields.iter().enumerate() {
        offsets[index] = parameter_len;
        parameter_len = parameter_len
            .checked_add(field.values().len())
            .filter(|&value| value <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "fluid_force_parameters",
            })?;
    }

    let topology = kinematic.fields();
    let mut roots = vec![0; nb];
    for (body, body_root) in roots.iter_mut().enumerate().skip(1) {
        let mut root = body;
        while topology.body_parentid[root] != 0 {
            root = topology.body_parentid[root] as usize;
        }
        *body_root = root as i32;
    }
    let metadata_len = nb
        .checked_mul(2)
        .and_then(|value| value.checked_add(nv))
        .and_then(|value| value.checked_add(8))
        .filter(|&value| value <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "fluid_force_metadata",
        })?;
    let mut metadata = Vec::new();
    metadata
        .try_reserve_exact(metadata_len)
        .map_err(|_| InputError::Overflow {
            field: "fluid_force_metadata",
        })?;
    metadata.extend([-131072; 4]);
    metadata.extend(&topology.body_parentid);
    metadata.extend(roots);
    metadata.extend(&inertial.dof_bodyid);
    metadata.extend([-131072; 4]);

    Ok(Spec {
        body_mass,
        body_inertia,
        input,
        metadata,
        offsets,
        parameter_len,
        nb,
        nj,
        nv,
    })
}

#[derive(Clone, Copy, Debug)]
struct Layout {
    body: BatchLayout,
    qfrc: BatchLayout,
}

fn layout(worlds: usize, nb: usize, nv: usize) -> Result<Layout, InputError> {
    if worlds == 0
        || worlds
            .checked_mul(nb.max(nv))
            .is_none_or(|value| value > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "fluid_force_work",
        });
    }
    let body = BatchLayout::new(
        worlds,
        nb.checked_mul(6).ok_or(InputError::Overflow {
            field: "fluid_force_layout",
        })?,
        4,
    )?;
    let qfrc = BatchLayout::new(worlds, nv, 4)?;
    for count in [body.total_elements(), qfrc.total_elements(), worlds] {
        BatchLayout::new(
            1,
            count.checked_add(8).ok_or(InputError::Overflow {
                field: "fluid_force_layout",
            })?,
            4,
        )?;
    }
    Ok(Layout { body, qfrc })
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
fn clear_work(value: usize) -> Result<u32, InputError> {
    u32::try_from(value)
        .ok()
        .filter(|&value| value > 0 && value <= u32::MAX - 255)
        .ok_or(InputError::Overflow {
            field: "fluid_force_clear_work",
        })
}

pub struct FluidForcePlan<'p> {
    source: FluidForceSource<'p>,
    spec: Arc<Spec>,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: TransferBuffer<i32>,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: [SynchronousKernel; 4],
}

pub struct FluidForceData {
    spec: Arc<Spec>,
    layout: Layout,
    ready: bool,
    resident_velocity: Option<ResidentComVelocityData>,
    #[cfg(feature = "cuda-probe")]
    body: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    qfrc: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

enum FluidForceSource<'p> {
    Independent(&'p ComVelocityPlan),
    Resident(ResidentComVelocityPlan<'p>),
}

pub struct FluidForceView<'a> {
    data: &'a FluidForceData,
    _source: &'a ComVelocityData,
}

#[derive(Clone, Copy, Debug)]
pub struct FluidForceWorld<'a> {
    pub body_force_torque: &'a [f32],
    pub qfrc: &'a [f32],
}

#[derive(Clone, Debug)]
pub struct FluidForceSnapshot {
    layout: Layout,
    body: Vec<f32>,
    qfrc: Vec<f32>,
}

impl FluidForceSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.qfrc.worlds()
    }

    pub fn world(&self, world: usize) -> Result<FluidForceWorld<'_>, InputError> {
        let body = self.layout.body.world_elements(world)?;
        let qfrc = self.layout.qfrc.world_elements(world)?;
        Ok(FluidForceWorld {
            body_force_torque: &self.body[4 + body.start..4 + body.end],
            qfrc: &self.qfrc[4 + qfrc.start..4 + qfrc.end],
        })
    }
}

impl<'p> FluidForcePlan<'p> {
    pub fn new(source: &'p ComVelocityPlan, input: FluidForceInput) -> Result<Self, TransferError> {
        let (session, model, body_mass, body_inertia) = source.fluid_force_context()?;
        Self::build(
            FluidForceSource::Independent(source),
            session,
            model,
            body_mass,
            body_inertia,
            input,
        )
    }

    pub(super) fn from_resident(
        source: &'p KinematicsPlan,
        input: FluidForceInput,
    ) -> Result<Self, TransferError> {
        let resident = ResidentComVelocityPlan::new(source)?;
        let (session, model, body_mass, body_inertia) = source.fluid_force_context()?;
        Self::build(
            FluidForceSource::Resident(resident),
            session,
            model,
            body_mass,
            body_inertia,
            input,
        )
    }

    fn build(
        source: FluidForceSource<'p>,
        session: TransferSession,
        model: Arc<AttachedModelInput>,
        body_mass: ParameterBatch,
        body_inertia: ParameterBatch,
        input: FluidForceInput,
    ) -> Result<Self, TransferError> {
        let spec = Arc::new(spec(model, body_mass, body_inertia, input)?);
        layout(1, spec.nb, spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (source, session, spec);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut packed = vec![-131072.0; 4];
            for field in [
                &spec.input.wind,
                &spec.input.density,
                &spec.input.viscosity,
                &spec.body_mass,
                &spec.body_inertia,
            ] {
                packed.extend(field.values());
            }
            packed.extend([-131072.0; 4]);
            let batches = [
                spec.input.wind.batches(),
                spec.input.density.batches(),
                spec.input.viscosity.batches(),
                spec.body_mass.batches(),
                spec.body_inertia.batches(),
            ]
            .map(|value| value.to_string())
            .join(",");
            let offsets = spec.offsets.map(|value| value.to_string()).join(",");
            let rigid_stride = 28 * spec.nb + 6 * spec.nj;
            let com_stride = 14 * spec.nb + 6 * spec.nv;
            let velocity_stride = 6 * spec.nb + 6 * spec.nv;
            let code = format!(
                "#define FF_NB {}\n#define FF_NV {}\n#define FF_PL {}\n#define FF_RS {}\n#define FF_CS {}\n#define FF_VS {}\n__device__ __constant__ unsigned FF_BATCH[5]={{{batches}}};\n__device__ __constant__ unsigned FF_OFF[5]={{{offsets}}};\n{CUDA}",
                spec.nb, spec.nv, spec.parameter_len, rigid_stride, com_stride, velocity_stride,
            );
            Ok(Self {
                source,
                spec: spec.clone(),
                metadata: session.upload(&spec.metadata)?,
                parameters: session.upload(&packed)?,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "fluid_force_validate")?,
                    SynchronousKernel::compile(&session, &code, "fluid_force_body")?,
                    SynchronousKernel::compile(&session, &code, "fluid_force_project")?,
                    SynchronousKernel::compile(&session, &code, "fluid_force_clear")?,
                ],
                session,
            })
        }
    }

    pub fn create_data(&self, worlds: usize) -> Result<FluidForceData, TransferError> {
        let layout = layout(worlds, self.spec.nb, self.spec.nv)?;
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
            Ok(FluidForceData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                resident_velocity: match &self.source {
                    FluidForceSource::Independent(_) => None,
                    FluidForceSource::Resident(source) => Some(source.create_data(worlds)?),
                },
                body: allocate(layout.body.total_elements(), f32::NAN)?,
                qfrc: allocate(layout.qfrc.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    fn corrupt_metadata_for_test(&mut self, index: usize, value: i32) -> Result<(), TransferError> {
        self.metadata.write_range(index, &[value])
    }

    pub fn update<'a>(
        &'a self,
        source: &'a ComVelocityData,
        data: &'a mut FluidForceData,
    ) -> Result<FluidForceView<'a>, TransferError> {
        let FluidForceSource::Independent(plan) = &self.source else {
            return Err(TransferError::ModelMismatch);
        };
        if !Arc::ptr_eq(&self.spec, &data.spec)
            || data.worlds() != source.worlds()
            || plan.check_data(source).is_err()
            || data.resident_velocity.is_some()
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
            let inputs = source.fluid_inputs()?;
            self.launch(inputs, data)?;
            data.ready = true;
            Ok(FluidForceView {
                data,
                _source: source,
            })
        }
    }

    pub(super) fn accepts_resident_source(&self, source: &KinematicsPlan) -> bool {
        matches!(&self.source, FluidForceSource::Resident(plan) if plan.accepts_source(source))
    }

    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    pub(super) fn update_resident(
        &self,
        source: &KinematicsData,
        data: &mut FluidForceData,
    ) -> Result<(), TransferError> {
        let FluidForceSource::Resident(plan) = &self.source else {
            return Err(TransferError::ModelMismatch);
        };
        if !Arc::ptr_eq(&self.spec, &data.spec) || data.worlds() != source.worlds() {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        let resident = data
            .resident_velocity
            .as_mut()
            .ok_or(TransferError::ModelMismatch)?;
        plan.update(source, resident)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let inputs = resident.fluid_inputs()?;
            let layout = data.layout;
            self.launch_buffers(
                inputs,
                layout,
                &mut data.body,
                &mut data.qfrc,
                &mut data.status,
            )?;
            data.ready = true;
            Ok(())
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn shared_qfrc<'a>(
        &self,
        data: &'a FluidForceData,
        worlds: usize,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) || data.worlds() != worlds {
            return Err(TransferError::ModelMismatch);
        }
        if !data.ready {
            return Err(TransferError::StageNotReady {
                stage: "fluid_force",
            });
        }
        Ok(&data.qfrc)
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn clear_shared(
        &self,
        data: &mut FluidForceData,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if !matches!(&self.source, FluidForceSource::Resident(_))
            || !Arc::ptr_eq(&self.spec, &data.spec)
            || data.worlds() != worlds
            || data.resident_velocity.is_none()
        {
            return Err(TransferError::ModelMismatch);
        }
        let layout = data.layout;
        let body_work = clear_work(layout.body.total_elements())?;
        let qfrc_work = clear_work(layout.qfrc.total_elements())?;
        let status_work = clear_work(worlds)?;
        data.ready = false;
        data.resident_velocity
            .as_mut()
            .ok_or(TransferError::ModelMismatch)?
            .invalidate();
        check_guards(&data.body)?;
        check_guards(&data.qfrc)?;
        check_guards(&data.status)?;
        for (output, work) in [
            (&mut data.body, body_work),
            (&mut data.qfrc, qfrc_work),
            (&mut data.status, status_work),
        ] {
            // SAFETY: 每线程清零一个独占槽。
            // 五个参数缓冲只满足私有ABI。
            unsafe {
                self.kernels[3].launch_fluid_force(
                    &self.metadata,
                    [
                        &self.parameters,
                        &self.parameters,
                        &self.parameters,
                        &self.parameters,
                        &self.parameters,
                    ],
                    output,
                    [
                        self.spec.nb as u32,
                        self.spec.nv as u32,
                        worlds as u32,
                        work,
                    ],
                )?;
            }
        }
        read_output(&data.body, layout.body.total_elements(), "fluid_force_body")?;
        read_output(&data.qfrc, layout.qfrc.total_elements(), "qfrc_fluid")?;
        read_output(&data.status, worlds, "fluid_force_status")?;
        data.ready = true;
        Ok(())
    }

    #[cfg(feature = "cuda-probe")]
    fn launch(
        &self,
        inputs: FluidInputs<'_>,
        data: &mut FluidForceData,
    ) -> Result<(), TransferError> {
        let layout = data.layout;
        self.launch_buffers(
            inputs,
            layout,
            &mut data.body,
            &mut data.qfrc,
            &mut data.status,
        )
    }

    #[cfg(feature = "cuda-probe")]
    fn launch_buffers(
        &self,
        inputs: FluidInputs<'_>,
        layout: Layout,
        body: &mut TransferBuffer<f32>,
        qfrc: &mut TransferBuffer<f32>,
        status_buffer: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        check_guards(body)?;
        check_guards(qfrc)?;
        let worlds = layout.qfrc.worlds();
        let body_work = worlds
            .checked_mul(self.spec.nb)
            .ok_or(InputError::Overflow {
                field: "fluid_force_work",
            })?;
        let qfrc_work = worlds
            .checked_mul(self.spec.nv)
            .ok_or(InputError::Overflow {
                field: "fluid_force_work",
            })?;
        // SAFETY: 输入来自同一已就绪速度票据。
        // 验证核先检查守卫、映射与参数。
        unsafe {
            self.kernels[0].launch_fluid_force(
                &self.metadata,
                [
                    &self.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.velocity,
                    &self.parameters,
                ],
                status_buffer,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    worlds as u32,
                ],
            )?;
        }
        let status = read_output(status_buffer, worlds, "fluid_force_status")?;
        if let Some(index) = status[4..4 + worlds].iter().position(|&value| value != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "fluid_force_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        // SAFETY: 每线程独占一个刚体空间力槽。
        unsafe {
            self.kernels[1].launch_fluid_force(
                &self.metadata,
                [
                    &self.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.velocity,
                    &self.parameters,
                ],
                body,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    body_work as u32,
                ],
            )?;
        }
        read_output(body, layout.body.total_elements(), "fluid_force_body")?;
        // SAFETY: 每线程按bodyid升序投影自由度。
        unsafe {
            self.kernels[2].launch_fluid_force(
                &self.metadata,
                [
                    &self.parameters,
                    inputs.rigid,
                    inputs.com,
                    body,
                    &self.parameters,
                ],
                qfrc,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    qfrc_work as u32,
                ],
            )?;
        }
        read_output(qfrc, layout.qfrc.total_elements(), "qfrc_fluid")?;
        Ok(())
    }
}

impl FluidForceData {
    pub fn worlds(&self) -> usize {
        self.layout.qfrc.worlds()
    }

    pub(super) fn snapshot(&self) -> Result<FluidForceSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "fluid_force",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(FluidForceSnapshot {
                layout: self.layout,
                body: read_output(
                    &self.body,
                    self.layout.body.total_elements(),
                    "fluid_force_body",
                )?,
                qfrc: read_output(&self.qfrc, self.layout.qfrc.total_elements(), "qfrc_fluid")?,
            })
        }
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(super) fn corrupt_body_guard_for_test(
        &mut self,
        index: usize,
        value: f32,
    ) -> Result<(), TransferError> {
        self.body.write_range(index, &[value])
    }
}

impl FluidForceView<'_> {
    pub fn snapshot(&self) -> Result<FluidForceSnapshot, TransferError> {
        self.data.snapshot()
    }
}

#[cfg(feature = "cuda-probe")]
fn check_guards(buffer: &TransferBuffer<f32>) -> Result<(), TransferError> {
    let mut values = [0.0; 8];
    buffer.read_range_into(0, &mut values[..4])?;
    buffer.read_range_into(buffer.len() - 4, &mut values[4..])?;
    super::check_device_guards(&values)
}

#[cfg(feature = "cuda-probe")]
fn read_output(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    super::check_device_values(&values, field)?;
    Ok(values)
}

// 冻结来源：passive.py::_fluid_force与support.py::apply_ft。
// 基线：71da24d，许可：Apache-2.0。
#[cfg(feature = "cuda-probe")]
const CUDA: &str = r#"
#define FF_MINVAL 1.0e-15f
#define FF_PI 3.14159265358979323846f
__device__ const float* ff_field(const float* p,unsigned field,unsigned w,unsigned width){return p+4+FF_OFF[field]+(w%FF_BATCH[field])*width;}
__device__ float ff_dot6(const float* a,const float* b){float ret=0.0f;
 #pragma unroll
 for(unsigned i=0;i<6;++i)ret+=a[i]*b[i];return ret;}
__device__ float ff_dot3(const float* a,const float* b){return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}
extern "C" __global__ void fluid_force_validate(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* unused,float* status,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 const int* parent=raw+4;const int* root=parent+FF_NB;const int* body=root+FF_NB;
 for(unsigned i=0;i<4;++i)ok=ok&&raw[i]==-131072&&raw[4+2*FF_NB+FF_NV+i]==-131072&&p[i]==-131072.0f&&p[4+FF_PL+i]==-131072.0f;
 for(unsigned i=0;i<4;++i)ok=ok&&rigid[i]==-131072.0f&&rigid[4+(unsigned long long)worlds*FF_RS+i]==-131072.0f&&com[i]==-131072.0f&&com[4+(unsigned long long)worlds*FF_CS+i]==-131072.0f&&velocity[i]==-131072.0f&&velocity[4+(unsigned long long)worlds*FF_VS+i]==-131072.0f;
 for(unsigned b=0;b<FF_NB;++b){int parentid=parent[b],rootid=root[b];bool parent_ok=(b==0&&parentid==0)||(b>0&&parentid>=0&&(unsigned)parentid<b);bool root_ok=rootid>=0&&(unsigned)rootid<FF_NB;if(!parent_ok||!root_ok){ok=false;continue;}int expected=b==0?0:(parentid==0?(int)b:root[parentid]);if(rootid!=expected)ok=false;}
 for(unsigned d=0;d<FF_NV;++d)if(body[d]<0||(unsigned)body[d]>=FF_NB)ok=false;
 if(!ok){status[4+w]=0.0f;return;}
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];const float* mass=ff_field(p,3,w,FF_NB);const float* inertia=ff_field(p,4,w,3*FF_NB);
 ok=ok&&isfinite(wind[0])&&isfinite(wind[1])&&isfinite(wind[2])&&isfinite(density)&&isfinite(viscosity);
 const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* ximat=r+19*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB;const float* cvel=velocity+4+(unsigned long long)w*FF_VS;
 for(unsigned b=0;b<FF_NB;++b){ok=ok&&isfinite(mass[b])&&isfinite(inertia[3*b])&&isfinite(inertia[3*b+1])&&isfinite(inertia[3*b+2]);if(b==0||mass[b]<FF_MINVAL)continue;for(unsigned i=0;i<3;++i)ok=ok&&isfinite(xipos[3*b+i])&&isfinite(subtree[3*(unsigned)root[b]+i]);for(unsigned i=0;i<9;++i)ok=ok&&isfinite(ximat[9*b+i]);for(unsigned i=0;i<6;++i)ok=ok&&isfinite(cvel[6*b+i]);}
 for(unsigned i=0;i<6*FF_NV;++i)ok=ok&&isfinite(cdof[i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void fluid_force_body(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* unused,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned b=tid%FF_NB,w=tid/FF_NB;float* out=output+4+6ull*tid;for(unsigned i=0;i<6;++i)out[i]=0.0f;if(b==0)return;
 const int* root=raw+4+FF_NB;const float* mass=ff_field(p,3,w,FF_NB);const float body_mass=mass[b];if(body_mass<FF_MINVAL)return;
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];bool has_viscosity=viscosity>0.0f,has_density=density>0.0f;if(!has_viscosity&&!has_density)return;
 const float* inertia=ff_field(p,4,w,3*FF_NB)+3*b;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* rot=r+19*FF_NB+9*b;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* v=velocity+4+(unsigned long long)w*FF_VS+6*b;
 float offset[3]={xipos[3*b]-subtree[3*(unsigned)root[b]],xipos[3*b+1]-subtree[3*(unsigned)root[b]+1],xipos[3*b+2]-subtree[3*(unsigned)root[b]+2]};
 float cross[3]={offset[1]*v[2]-offset[2]*v[1],offset[2]*v[0]-offset[0]*v[2],offset[0]*v[1]-offset[1]*v[0]};
 float lin[3]={v[3]-cross[0],v[4]-cross[1],v[5]-cross[2]};
 float lang[3]={rot[0]*v[0]+rot[3]*v[1]+rot[6]*v[2],rot[1]*v[0]+rot[4]*v[1]+rot[7]*v[2],rot[2]*v[0]+rot[5]*v[1]+rot[8]*v[2]};
 float llin[3]={rot[0]*lin[0]+rot[3]*lin[1]+rot[6]*lin[2],rot[1]*lin[0]+rot[4]*lin[1]+rot[7]*lin[2],rot[2]*lin[0]+rot[5]*lin[1]+rot[8]*lin[2]};
 if(wind[0]!=0.0f||wind[1]!=0.0f||wind[2]!=0.0f){llin[0]-=rot[0]*wind[0]+rot[3]*wind[1]+rot[6]*wind[2];llin[1]-=rot[1]*wind[0]+rot[4]*wind[1]+rot[7]*wind[2];llin[2]-=rot[2]*wind[0]+rot[5]*wind[1]+rot[8]*wind[2];}
 float scl=6.0f/body_mass;float box0=sqrtf(fmaxf(FF_MINVAL,inertia[1]+inertia[2]-inertia[0])*scl);float box1=sqrtf(fmaxf(FF_MINVAL,inertia[0]+inertia[2]-inertia[1])*scl);float box2=sqrtf(fmaxf(FF_MINVAL,inertia[0]+inertia[1]-inertia[2])*scl);
 float ltorque[3]={0.0f,0.0f,0.0f},lforce[3]={0.0f,0.0f,0.0f};
 if(has_viscosity){float diam=(box0+box1+box2)/3.0f;float diam_pow3=powf(diam,3.0f);for(unsigned i=0;i<3;++i){ltorque[i]=((-lang[i]*diam_pow3)*FF_PI)*viscosity;lforce[i]=(((-3.0f*llin[i])*diam)*FF_PI)*viscosity;}}
 if(has_density){lforce[0]-=0.5f*density*box1*box2*fabsf(llin[0])*llin[0];lforce[1]-=0.5f*density*box0*box2*fabsf(llin[1])*llin[1];lforce[2]-=0.5f*density*box0*box1*fabsf(llin[2])*llin[2];scl=density/64.0f;float box0_pow4=powf(box0,4.0f),box1_pow4=powf(box1,4.0f),box2_pow4=powf(box2,4.0f);ltorque[0]-=box0*(box1_pow4+box2_pow4)*fabsf(lang[0])*lang[0]*scl;ltorque[1]-=box1*(box0_pow4+box2_pow4)*fabsf(lang[1])*lang[1]*scl;ltorque[2]-=box2*(box0_pow4+box1_pow4)*fabsf(lang[2])*lang[2]*scl;}
 out[0]=rot[0]*lforce[0]+rot[1]*lforce[1]+rot[2]*lforce[2];out[1]=rot[3]*lforce[0]+rot[4]*lforce[1]+rot[5]*lforce[2];out[2]=rot[6]*lforce[0]+rot[7]*lforce[1]+rot[8]*lforce[2];out[3]=rot[0]*ltorque[0]+rot[1]*ltorque[1]+rot[2]*ltorque[2];out[4]=rot[3]*ltorque[0]+rot[4]*ltorque[1]+rot[5]*ltorque[2];out[5]=rot[6]*ltorque[0]+rot[7]*ltorque[1]+rot[8]*ltorque[2];
}
extern "C" __global__ void fluid_force_project(const int* raw,const float* p,const float* rigid,const float* com,const float* body_force,const float* unused,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned w=tid/FF_NV,d=tid%FF_NV;const int* parent=raw+4;const int* root=parent+FF_NB;const int* dof_body=root+FF_NB;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB+6*d;const float* force=body_force+4+6ull*w*FF_NB;float jac[6]={cdof[3],cdof[4],cdof[5],cdof[0],cdof[1],cdof[2]};float accum=0.0f;int dofbody=dof_body[d];
 for(unsigned b=(unsigned)dofbody;b<FF_NB;++b){const float* f=force+6*b;if(f[0]==0.0f&&f[1]==0.0f&&f[2]==0.0f&&f[3]==0.0f&&f[4]==0.0f&&f[5]==0.0f)continue;int cursor=(int)b;while(cursor!=0&&cursor!=dofbody)cursor=parent[cursor];if(cursor==0)continue;const float* point=xipos+3*b;const float* center=subtree+3*(unsigned)root[b];float offset[3]={point[0]-center[0],point[1]-center[1],point[2]-center[2]};float cross[3]={cdof[1]*offset[2]-cdof[2]*offset[1],cdof[2]*offset[0]-cdof[0]*offset[2],cdof[0]*offset[1]-cdof[1]*offset[0]};accum+=ff_dot6(jac,f)+ff_dot3(cross,f);}
 output[4+tid]=accum;
}
extern "C" __global__ void fluid_force_clear(const int* raw,const float* p,const float* first,const float* second,const float* third,const float* fourth,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;output[4+tid]=0.0f;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
        KinematicsParameter, KinematicsParameters,
    };

    #[test]
    fn checks_fluid_clear_work_bounds() {
        assert_eq!(clear_work(1).unwrap(), 1);
        assert_eq!(
            clear_work((u32::MAX - 255) as usize).unwrap(),
            u32::MAX - 255
        );
        for value in [0, (u32::MAX - 254) as usize, usize::MAX] {
            assert!(matches!(
                clear_work(value),
                Err(InputError::Overflow {
                    field: "fluid_force_clear_work"
                })
            ));
        }
    }

    fn simple_model() -> AttachedModelInput {
        let kinematic = KinematicModelInput::new(
            1,
            KinematicFields {
                qpos0: vec![0.0],
                body_parentid: vec![0, 0, 1],
                body_jntadr: vec![-1, 0, -1],
                body_jntnum: vec![0, 1, 0],
                body_pos: vec![0.0; 9],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
                jnt_type: vec![3],
                jnt_bodyid: vec![1],
                jnt_qposadr: vec![0],
                jnt_dofadr: vec![0],
                jnt_pos: vec![0.0; 3],
                jnt_axis: vec![0.0, 0.0, 1.0],
            },
        )
        .unwrap();
        let inertial = InertialModelInput::new(
            kinematic,
            InertialFields {
                body_ipos: vec![0.0; 9],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(3),
                body_mass: vec![0.0, 6.0, 0.0],
                body_inertia: vec![0.0, 0.0, 0.0, 5.0, 4.0, 3.0, 0.0, 0.0, 0.0],
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

    fn simple_kinematics_parameters() -> KinematicsParameters {
        let mut parameters = KinematicsParameters::default();
        let mass = (0..7)
            .flat_map(|row| [0.0, 6.0 + 0.5 * (row % 3) as f32, 0.0])
            .collect();
        let inertia = (0..11)
            .flat_map(|row| {
                let delta = 0.125 * (row % 4) as f32;
                [0.0, 0.0, 0.0, 5.0 + delta, 4.0, 3.0, 0.0, 0.0, 0.0]
            })
            .collect();
        parameters.set(
            KinematicsParameter::BodyMass,
            ParameterBatch::new(7, 3, mass).unwrap(),
        );
        parameters.set(
            KinematicsParameter::BodyInertia,
            ParameterBatch::new(11, 9, inertia).unwrap(),
        );
        parameters
    }

    fn model() -> AttachedModelInput {
        let kinematic = KinematicModelInput::new(
            2,
            KinematicFields {
                qpos0: vec![0.0; 2],
                body_parentid: vec![0, 0, 1, 2, 1],
                body_jntadr: vec![-1, 0, 1, -1, -1],
                body_jntnum: vec![0, 1, 1, 0, 0],
                body_pos: vec![
                    0.0, 0.0, 0.0, 0.4, -0.2, 0.3, 0.8, 0.1, -0.15, 0.35, -0.25, 0.2, -0.3, 0.45,
                    0.1,
                ],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(5),
                jnt_type: vec![3, 3],
                jnt_bodyid: vec![1, 2],
                jnt_qposadr: vec![0, 1],
                jnt_dofadr: vec![0, 1],
                jnt_pos: vec![0.1, 0.0, 0.0, 0.0, 0.15, -0.05],
                jnt_axis: vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            },
        )
        .unwrap();
        let inertial = InertialModelInput::new(
            kinematic,
            InertialFields {
                body_ipos: vec![
                    0.0, 0.0, 0.0, 0.2, 0.05, 0.0, -0.1, 0.15, 0.05, 0.05, -0.1, 0.12, 0.0, 0.0,
                    0.0,
                ],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(5),
                body_mass: vec![0.0, 6.0, 4.0, 2.0, 0.0],
                body_inertia: vec![
                    0.0, 0.0, 0.0, 5.0, 4.0, 3.0, 3.0, 2.5, 2.0, 2.0, 1.5, 1.0, 0.0, 0.0, 0.0,
                ],
                dof_bodyid: vec![1, 2],
                dof_jntid: vec![0, 1],
                dof_parentid: vec![-1, 0],
                dof_armature: vec![0.0; 2],
                dof_damping: vec![0.0; 2],
            },
        )
        .unwrap();
        AttachedModelInput::new(inertial, AttachedFields::default()).unwrap()
    }

    fn simple_fluid() -> FluidForceInput {
        FluidForceInput {
            wind: ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.0, 0.5, -0.25, 0.125]).unwrap(),
            density: ParameterBatch::new(3, 1, vec![0.0, 0.5, 1.0]).unwrap(),
            viscosity: ParameterBatch::new(5, 1, vec![0.0, 0.125, 0.25, 0.375, 0.5]).unwrap(),
            body_fluid_ellipsoid: vec![false; 3],
        }
    }

    fn fluid() -> FluidForceInput {
        let mut input = simple_fluid();
        input.body_fluid_ellipsoid.resize(5, false);
        input
    }

    fn kinematics_parameters() -> KinematicsParameters {
        let mut parameters = KinematicsParameters::default();
        let mass = (0..7)
            .flat_map(|row| {
                [
                    0.0,
                    6.0 + 0.5 * (row % 3) as f32,
                    4.0 + 0.25 * ((row + 1) % 3) as f32,
                    2.0 + 0.125 * (row % 4) as f32,
                    0.0,
                ]
            })
            .collect();
        let inertia = (0..11)
            .flat_map(|row| {
                let delta = 0.125 * (row % 4) as f32;
                [
                    0.0,
                    0.0,
                    0.0,
                    5.0 + delta,
                    4.0,
                    3.0,
                    3.0,
                    2.5 + 0.5 * delta,
                    2.0,
                    2.0,
                    1.5,
                    1.0 + 0.25 * delta,
                    0.0,
                    0.0,
                    0.0,
                ]
            })
            .collect();
        parameters.set(
            KinematicsParameter::BodyMass,
            ParameterBatch::new(7, 5, mass).unwrap(),
        );
        parameters.set(
            KinematicsParameter::BodyInertia,
            ParameterBatch::new(11, 15, inertia).unwrap(),
        );
        parameters
    }

    #[test]
    fn host_preserves_independent_periods_and_rejects_ellipsoid_branch() {
        let simple_parameters = simple_kinematics_parameters();
        let simple = spec(
            Arc::new(simple_model()),
            simple_parameters
                .get(KinematicsParameter::BodyMass)
                .unwrap()
                .clone(),
            simple_parameters
                .get(KinematicsParameter::BodyInertia)
                .unwrap()
                .clone(),
            simple_fluid(),
        )
        .unwrap();
        assert_eq!(simple.offsets, [0, 6, 9, 14, 35]);
        assert_eq!(simple.parameter_len, 134);
        assert_eq!(&simple.metadata[4..7], &[0, 0, 1]);
        assert_eq!(&simple.metadata[7..10], &[0, 1, 1]);
        assert_eq!(&simple.metadata[10..11], &[1]);

        let model = Arc::new(model());
        let parameters = kinematics_parameters();
        let mass = parameters
            .get(KinematicsParameter::BodyMass)
            .unwrap()
            .clone();
        let inertia = parameters
            .get(KinematicsParameter::BodyInertia)
            .unwrap()
            .clone();
        let value = spec(model.clone(), mass.clone(), inertia.clone(), fluid()).unwrap();
        assert_eq!(value.offsets, [0, 6, 9, 14, 49]);
        assert_eq!(value.parameter_len, 214);
        assert_eq!(&value.metadata[4..9], &[0, 0, 1, 2, 1]);
        assert_eq!(&value.metadata[9..14], &[0, 1, 1, 1, 1]);
        assert_eq!(&value.metadata[14..16], &[1, 2]);
        assert_eq!(
            [
                value.input.wind.batches(),
                value.input.density.batches(),
                value.input.viscosity.batches(),
                value.body_mass.batches(),
                value.body_inertia.batches(),
            ],
            [2, 3, 5, 7, 11]
        );
        let mut ellipsoid = fluid();
        ellipsoid.body_fluid_ellipsoid[1] = true;
        assert!(matches!(
            spec(model, mass, inertia, ellipsoid),
            Err(InputError::InvalidTopology {
                field: "body_fluid_ellipsoid",
                index: 1,
                ..
            })
        ));
        assert!(layout(13, 5, 2).is_ok());
        assert!(layout(0, 5, 2).is_err());
        assert!(layout(1, usize::MAX, 1).is_err());
    }

    #[test]
    fn viscosity_torque_keeps_frozen_left_associative_rounding() {
        let scale = 6.0f32 / 6.0;
        let box0 = ((4.0f32 + 3.0 - 5.0).max(1.0e-15) * scale).sqrt();
        let box1 = ((5.0f32 + 3.0 - 4.0).max(1.0e-15) * scale).sqrt();
        let box2 = ((5.0f32 + 4.0 - 3.0).max(1.0e-15) * scale).sqrt();
        let diameter = (box0 + box1 + box2) / 3.0;
        let diameter_pow3 = diameter.powf(3.0);
        let left_associative = ((-0.2f32 * diameter_pow3) * std::f32::consts::PI) * 0.375;
        let regrouped = -0.2f32 * ((diameter_pow3 * std::f32::consts::PI) * 0.375);
        assert_eq!(left_associative.to_bits(), 0xbfe1_33de);
        assert_eq!(regrouped.to_bits(), 0xbfe1_33df);
    }

    #[cfg(feature = "cuda-probe")]
    fn mul(rotation: &[f32], value: [f32; 3]) -> [f32; 3] {
        [
            rotation[0] * value[0] + rotation[1] * value[1] + rotation[2] * value[2],
            rotation[3] * value[0] + rotation[4] * value[1] + rotation[5] * value[2],
            rotation[6] * value[0] + rotation[7] * value[1] + rotation[8] * value[2],
        ]
    }

    #[cfg(feature = "cuda-probe")]
    fn mul_transpose(rotation: &[f32], value: [f32; 3]) -> [f32; 3] {
        [
            rotation[0] * value[0] + rotation[3] * value[1] + rotation[6] * value[2],
            rotation[1] * value[0] + rotation[4] * value[1] + rotation[7] * value[2],
            rotation[2] * value[0] + rotation[5] * value[1] + rotation[8] * value[2],
        ]
    }

    #[cfg(feature = "cuda-probe")]
    fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
        [
            left[1] * right[2] - left[2] * right[1],
            left[2] * right[0] - left[0] * right[2],
            left[0] * right[1] - left[1] * right[0],
        ]
    }

    #[cfg(feature = "cuda-probe")]
    fn reference(
        model: &AttachedModelInput,
        world: usize,
        sources: (&[f32], &[f32], &[f32]),
        mass: &[f32],
        inertia: &[f32],
        wind: &[f32],
        density: f32,
        viscosity: f32,
    ) -> (Vec<f32>, Vec<f32>) {
        let kinematic = model.rigid().kinematics();
        let inertial = model.rigid().fields();
        let (nb, nj, nv) = (kinematic.nbody(), kinematic.njnt(), kinematic.nv());
        let rigid_stride = 28 * nb + 6 * nj;
        let com_stride = 14 * nb + 6 * nv;
        let velocity_stride = 6 * nb + 6 * nv;
        let rigid = &sources.0[4 + world * rigid_stride..];
        let com = &sources.1[4 + world * com_stride..];
        let velocity = &sources.2[4 + world * velocity_stride..];
        let xipos = &rigid[16 * nb..19 * nb];
        let ximat = &rigid[19 * nb..28 * nb];
        let subtree_com = &com[nb..4 * nb];
        let cdof = &com[14 * nb..14 * nb + 6 * nv];
        let cvel = &velocity[..6 * nb];
        let parent = &kinematic.fields().body_parentid;
        let mut roots = vec![0usize; nb];
        for body in 1..nb {
            let mut root = body;
            while parent[root] != 0 {
                root = parent[root] as usize;
            }
            roots[body] = root;
        }
        let mut body_force = vec![0.0f32; 6 * nb];
        for body in 1..nb {
            if mass[body] < 1.0e-15 {
                continue;
            }
            let rotation = &ximat[9 * body..9 * body + 9];
            let angular = [cvel[6 * body], cvel[6 * body + 1], cvel[6 * body + 2]];
            let linear = [cvel[6 * body + 3], cvel[6 * body + 4], cvel[6 * body + 5]];
            let offset = [
                xipos[3 * body] - subtree_com[3 * roots[body]],
                xipos[3 * body + 1] - subtree_com[3 * roots[body] + 1],
                xipos[3 * body + 2] - subtree_com[3 * roots[body] + 2],
            ];
            let offset_cross = cross(offset, angular);
            let linear_com = [
                linear[0] - offset_cross[0],
                linear[1] - offset_cross[1],
                linear[2] - offset_cross[2],
            ];
            let lang = mul_transpose(rotation, angular);
            let mut llin = mul_transpose(rotation, linear_com);
            if wind[0] != 0.0 || wind[1] != 0.0 || wind[2] != 0.0 {
                let local_wind = mul_transpose(rotation, [wind[0], wind[1], wind[2]]);
                for index in 0..3 {
                    llin[index] -= local_wind[index];
                }
            }
            let body_inertia = &inertia[3 * body..3 * body + 3];
            let scale = 6.0 / mass[body];
            let box0 =
                ((body_inertia[1] + body_inertia[2] - body_inertia[0]).max(1.0e-15) * scale).sqrt();
            let box1 =
                ((body_inertia[0] + body_inertia[2] - body_inertia[1]).max(1.0e-15) * scale).sqrt();
            let box2 =
                ((body_inertia[0] + body_inertia[1] - body_inertia[2]).max(1.0e-15) * scale).sqrt();
            let mut local_force = [0.0f32; 3];
            let mut local_torque = [0.0f32; 3];
            if viscosity > 0.0 {
                let diam = (box0 + box1 + box2) / 3.0;
                let diam_pow3 = diam.powf(3.0);
                for index in 0..3 {
                    local_torque[index] =
                        ((-lang[index] * diam_pow3) * std::f32::consts::PI) * viscosity;
                    local_force[index] =
                        (((-3.0 * llin[index]) * diam) * std::f32::consts::PI) * viscosity;
                }
            }
            if density > 0.0 {
                local_force[0] -= 0.5 * density * box1 * box2 * llin[0].abs() * llin[0];
                local_force[1] -= 0.5 * density * box0 * box2 * llin[1].abs() * llin[1];
                local_force[2] -= 0.5 * density * box0 * box1 * llin[2].abs() * llin[2];
                let scale = density / 64.0;
                let powers = [box0.powf(4.0), box1.powf(4.0), box2.powf(4.0)];
                local_torque[0] -= box0 * (powers[1] + powers[2]) * lang[0].abs() * lang[0] * scale;
                local_torque[1] -= box1 * (powers[0] + powers[2]) * lang[1].abs() * lang[1] * scale;
                local_torque[2] -= box2 * (powers[0] + powers[1]) * lang[2].abs() * lang[2] * scale;
            }
            let force = mul(rotation, local_force);
            let torque = mul(rotation, local_torque);
            body_force[6 * body..6 * body + 3].copy_from_slice(&force);
            body_force[6 * body + 3..6 * body + 6].copy_from_slice(&torque);
        }

        let mut qfrc = vec![0.0f32; nv];
        for dof in 0..nv {
            let vector = &cdof[6 * dof..6 * dof + 6];
            let jacobian = [
                vector[3], vector[4], vector[5], vector[0], vector[1], vector[2],
            ];
            let dof_body = inertial.dof_bodyid[dof];
            let mut accum = 0.0f32;
            for body in dof_body as usize..nb {
                let force = &body_force[6 * body..6 * body + 6];
                if force.iter().all(|&value| value == 0.0) {
                    continue;
                }
                let mut cursor = body as i32;
                while cursor != 0 && cursor != dof_body {
                    cursor = parent[cursor as usize];
                }
                if cursor == 0 {
                    continue;
                }
                let offset = [
                    xipos[3 * body] - subtree_com[3 * roots[body]],
                    xipos[3 * body + 1] - subtree_com[3 * roots[body] + 1],
                    xipos[3 * body + 2] - subtree_com[3 * roots[body] + 2],
                ];
                let cross_term = cross([vector[0], vector[1], vector[2]], offset);
                let mut dot6 = 0.0f32;
                for index in 0..6 {
                    dot6 += jacobian[index] * force[index];
                }
                let dot3 =
                    cross_term[0] * force[0] + cross_term[1] * force[1] + cross_term[2] * force[2];
                accum += dot6 + dot3;
            }
            qfrc[dof] = accum;
        }
        (body_force, qfrc)
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new G04 inertia-box fluid force and apply_ft projection"]
    fn gpu_matches_independent_inertia_box_reference() {
        const WORLDS: usize = 13;
        const TOLERANCE: f32 = 2.0e-6;
        let started = std::time::Instant::now();
        let session = crate::runtime::TransferSession::new(0).unwrap();

        let simple_kinematics = simple_kinematics_parameters();
        let simple_model = simple_model();
        let simple_plan = ComVelocityPlan::with_parameters(
            &session,
            simple_model.clone(),
            simple_kinematics.clone(),
        )
        .unwrap();
        let simple_fluid = simple_fluid();
        let simple_fluid_plan = FluidForcePlan::new(&simple_plan, simple_fluid.clone()).unwrap();
        let mut simple_source = simple_plan.create_data(WORLDS).unwrap();
        simple_source.write_qpos(&vec![0.0; WORLDS]).unwrap();
        let simple_qvel = (0..WORLDS)
            .map(|world| (world as f32 - 6.0) * 0.125)
            .collect::<Vec<_>>();
        simple_source.write_qvel(&simple_qvel).unwrap();
        simple_plan.update(&mut simple_source).unwrap();
        let simple_fields = simple_source.fluid_snapshot_for_test().unwrap();
        let mut simple_data = simple_fluid_plan.create_data(WORLDS).unwrap();
        let simple_output = simple_fluid_plan
            .update(&simple_source, &mut simple_data)
            .unwrap()
            .snapshot()
            .unwrap();
        let simple_mass = simple_kinematics
            .get(KinematicsParameter::BodyMass)
            .unwrap();
        let simple_inertia = simple_kinematics
            .get(KinematicsParameter::BodyInertia)
            .unwrap();
        let mut compared = 0usize;
        let mut max_absolute = 0.0f32;
        for world in 0..WORLDS {
            let (expected_body, expected_qfrc) = reference(
                &simple_model,
                world,
                (&simple_fields.0, &simple_fields.1, &simple_fields.2),
                simple_mass.world(world),
                simple_inertia.world(world),
                simple_fluid.wind.world(world),
                simple_fluid.density.world(world)[0],
                simple_fluid.viscosity.world(world)[0],
            );
            let actual = simple_output.world(world).unwrap();
            assert_eq!(&actual.body_force_torque[..6], &[0.0; 6]);
            assert_eq!(&actual.body_force_torque[12..18], &[0.0; 6]);
            for (index, (&actual, &expected)) in actual.body_force_torque[6..12]
                .iter()
                .zip(expected_body[6..12].iter())
                .enumerate()
            {
                let absolute = (actual - expected).abs();
                max_absolute = max_absolute.max(absolute);
                assert!(
                    absolute <= TOLERANCE,
                    "simple world={world} body_component={index} actual={actual} expected={expected}"
                );
                compared += 1;
            }
            let absolute = (actual.qfrc[0] - expected_qfrc[0]).abs();
            max_absolute = max_absolute.max(absolute);
            assert!(
                absolute <= TOLERANCE,
                "simple world={world} qfrc actual={} expected={}",
                actual.qfrc[0],
                expected_qfrc[0]
            );
            compared += 1;
        }
        assert_eq!(compared, 91);

        let kinematics = kinematics_parameters();
        let complex_model = model();
        let plan =
            ComVelocityPlan::with_parameters(&session, complex_model.clone(), kinematics.clone())
                .unwrap();
        let fluid_input = fluid();
        let mut fluid_plan = FluidForcePlan::new(&plan, fluid_input.clone()).unwrap();
        let mut source = plan.create_data(WORLDS).unwrap();
        let qpos = (0..WORLDS)
            .flat_map(|world| [(world as f32 - 6.0) * 0.11, (3.0 - world as f32) * 0.07])
            .collect::<Vec<_>>();
        source.write_qpos(&qpos).unwrap();
        let qvel = (0..WORLDS)
            .flat_map(|world| [0.2 + world as f32 * 0.03, -0.15 + world as f32 * 0.025])
            .collect::<Vec<_>>();
        source.write_qvel(&qvel).unwrap();
        plan.update(&mut source).unwrap();
        let source_fields = source.fluid_snapshot_for_test().unwrap();
        let mut data = fluid_plan.create_data(WORLDS).unwrap();
        let output = fluid_plan
            .update(&source, &mut data)
            .unwrap()
            .snapshot()
            .unwrap();

        let mass = kinematics.get(KinematicsParameter::BodyMass).unwrap();
        let inertia = kinematics.get(KinematicsParameter::BodyInertia).unwrap();
        for world in 0..WORLDS {
            let (expected_body, expected_qfrc) = reference(
                &complex_model,
                world,
                (&source_fields.0, &source_fields.1, &source_fields.2),
                mass.world(world),
                inertia.world(world),
                fluid_input.wind.world(world),
                fluid_input.density.world(world)[0],
                fluid_input.viscosity.world(world)[0],
            );
            let actual = output.world(world).unwrap();
            assert_eq!(&actual.body_force_torque[..6], &[0.0; 6]);
            assert_eq!(&actual.body_force_torque[24..30], &[0.0; 6]);
            for (index, (&actual, &expected)) in actual
                .body_force_torque
                .iter()
                .zip(expected_body.iter())
                .enumerate()
            {
                let absolute = (actual - expected).abs();
                max_absolute = max_absolute.max(absolute);
                assert!(
                    absolute <= TOLERANCE,
                    "world={world} body_component={index} actual={actual} expected={expected}"
                );
                compared += 1;
            }
            for (index, (&actual, &expected)) in
                actual.qfrc.iter().zip(expected_qfrc.iter()).enumerate()
            {
                let absolute = (actual - expected).abs();
                max_absolute = max_absolute.max(absolute);
                assert!(
                    absolute <= TOLERANCE,
                    "world={world} qfrc={index} actual={actual} expected={expected}"
                );
                compared += 1;
            }
        }
        assert_eq!(compared, 507);
        assert!(output.world(WORLDS).is_err());
        fluid_plan.corrupt_metadata_for_test(11, i32::MAX).unwrap();
        assert!(matches!(
            fluid_plan.update(&source, &mut data),
            Err(TransferError::Input(InputError::InvalidTopology {
                field: "fluid_force_source",
                reason: "guard_mapping_or_nonfinite",
                ..
            }))
        ));
        let report = serde_json::json!({
            "worlds": WORLDS,
            "compared_slots": compared,
            "preserved_simple_slots": 91,
            "expanded_slots": 416,
            "absolute_tolerance": TOLERANCE,
            "max_absolute": max_absolute,
            "parameter_periods": [2, 3, 5, 7, 11],
            "mass_below_minval_body": 2,
            "wind_short_circuit_worlds": 7,
            "nonidentity_rotation": true,
            "nonzero_com_offset": true,
            "multi_body_projection": true,
            "invalid_root_rejected": true,
            "ellipsoid_supported": false,
            "frozen_upstream_device_output": false,
            "elapsed_ms": started.elapsed().as_millis(),
            "performance_acceptance": false,
        });
        if let Some(path) = std::env::var_os("MJWARP_FLUID_FORCE_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }
}
