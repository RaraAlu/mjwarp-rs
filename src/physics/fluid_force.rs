//! 惯性盒与椭球流体被动力。

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
    pub geom_size: ParameterBatch,
    pub geom_type: Vec<i32>,
    pub geom_fluid: Vec<f32>,
    pub body_fluid_ellipsoid: Vec<bool>,
}

#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    body_mass: ParameterBatch,
    body_inertia: ParameterBatch,
    input: FluidForceInput,
    metadata: Vec<i32>,
    offsets: [usize; 6],
    fluid_offset: usize,
    parameter_len: usize,
    nb: usize,
    nj: usize,
    nv: usize,
    ng: usize,
}

fn spec(
    model: Arc<AttachedModelInput>,
    body_mass: ParameterBatch,
    body_inertia: ParameterBatch,
    input: FluidForceInput,
) -> Result<Spec, InputError> {
    let kinematic = model.rigid().kinematics();
    let inertial = model.rigid().fields();
    let (nb, nj, nv, ng) = (
        kinematic.nbody(),
        kinematic.njnt(),
        kinematic.nv(),
        model.ngeom(),
    );
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
        ("geom_size", input.geom_size.row_elements(), 3 * ng),
        ("geom_type", input.geom_type.len(), ng),
        ("geom_fluid", input.geom_fluid.len(), 12 * ng),
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
    if let Some(index) = input.geom_fluid.iter().position(|value| !value.is_finite()) {
        return Err(InputError::NonFinite {
            field: "geom_fluid",
            index,
        });
    }

    let fields = [
        &input.wind,
        &input.density,
        &input.viscosity,
        &body_mass,
        &body_inertia,
        &input.geom_size,
    ];
    let mut offsets = [0; 6];
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
    let fluid_offset = parameter_len;
    parameter_len = parameter_len
        .checked_add(input.geom_fluid.len())
        .filter(|&value| value <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "fluid_force_parameters",
        })?;

    let topology = kinematic.fields();
    let attached = model.fields();
    let mut roots = vec![0; nb];
    for (body, body_root) in roots.iter_mut().enumerate().skip(1) {
        let mut root = body;
        while topology.body_parentid[root] != 0 {
            root = topology.body_parentid[root] as usize;
        }
        *body_root = root as i32;
    }
    let metadata_len = nb
        .checked_mul(3)
        .and_then(|value| value.checked_add(nv))
        .and_then(|value| ng.checked_mul(2).and_then(|ng2| value.checked_add(ng2)))
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
    metadata.extend(&attached.geom_bodyid);
    metadata.extend(&input.geom_type);
    metadata.extend(
        input
            .body_fluid_ellipsoid
            .iter()
            .map(|&value| i32::from(value)),
    );
    metadata.extend([-131072; 4]);

    Ok(Spec {
        body_mass,
        body_inertia,
        input,
        metadata,
        offsets,
        fluid_offset,
        parameter_len,
        nb,
        nj,
        nv,
        ng,
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
        let spec = Arc::new(spec(model.clone(), body_mass, body_inertia, input)?);
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
                &spec.input.geom_size,
            ] {
                packed.extend(field.values());
            }
            packed.extend(&spec.input.geom_fluid);
            packed.extend([-131072.0; 4]);
            let batches = [
                spec.input.wind.batches(),
                spec.input.density.batches(),
                spec.input.viscosity.batches(),
                spec.body_mass.batches(),
                spec.body_inertia.batches(),
                spec.input.geom_size.batches(),
            ]
            .map(|value| value.to_string())
            .join(",");
            let offsets = spec.offsets.map(|value| value.to_string()).join(",");
            let rigid_stride = 28 * spec.nb + 6 * spec.nj;
            let com_stride = 14 * spec.nb + 6 * spec.nv;
            let velocity_stride = 6 * spec.nb + 6 * spec.nv;
            let code = format!(
                "#define FF_NB {}\n#define FF_NV {}\n#define FF_NG {}\n#define FF_HE {}\n#define FF_PL {}\n#define FF_GF {}\n#define FF_RS {}\n#define FF_CS {}\n#define FF_VS {}\n#define FF_AS {}\n__device__ __constant__ unsigned FF_BATCH[6]={{{batches}}};\n__device__ __constant__ unsigned FF_OFF[6]={{{offsets}}};\n{CUDA}",
                spec.nb,
                spec.nv,
                spec.ng,
                usize::from(spec.input.body_fluid_ellipsoid.iter().any(|&value| value)),
                spec.parameter_len,
                spec.fluid_offset,
                rigid_stride,
                com_stride,
                velocity_stride,
                12 * (spec.ng + model.nsite()),
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
        source: &'a mut ComVelocityData,
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
            let has_ellipsoid = self
                .spec
                .input
                .body_fluid_ellipsoid
                .iter()
                .any(|&value| value);
            if has_ellipsoid {
                plan.update_fluid_attached(source)?;
            }
            let inputs = plan.fluid_inputs(source, has_ellipsoid)?;
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
            let has_ellipsoid = self
                .spec
                .input
                .body_fluid_ellipsoid
                .iter()
                .any(|&value| value);
            let inputs = plan.fluid_inputs(source, resident, has_ellipsoid)?;
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
        let attached = inputs.attached.unwrap_or(&self.parameters);
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
                    attached,
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
                    attached,
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
                [&self.parameters, inputs.rigid, inputs.com, body, attached],
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
#define FF_FOUR_THIRDS_PI 4.18879032135009765625f
#define FF_EIGHT_FIFTEENTHS_PI 1.6755161285400390625f
#define FF_TWO_THIRDS 0.666666686534881591796875f
#define FF_THREE_PI 9.424777984619140625f
__device__ const float* ff_field(const float* p,unsigned field,unsigned w,unsigned width){return p+4+FF_OFF[field]+(w%FF_BATCH[field])*width;}
__device__ float ff_dot6(const float* a,const float* b){float ret=0.0f;
 #pragma unroll
 for(unsigned i=0;i<6;++i)ret+=a[i]*b[i];return ret;}
__device__ float ff_dot3(const float* a,const float* b){return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}
__device__ void ff_cross(const float* a,const float* b,float* out){out[0]=a[1]*b[2]-a[2]*b[1];out[1]=a[2]*b[0]-a[0]*b[2];out[2]=a[0]*b[1]-a[1]*b[0];}
__device__ float ff_pow4(float x){float sq=x*x;return sq*sq;}
__device__ void ff_mul(const float* m,const float* v,float* out){out[0]=m[0]*v[0]+m[1]*v[1]+m[2]*v[2];out[1]=m[3]*v[0]+m[4]*v[1]+m[5]*v[2];out[2]=m[6]*v[0]+m[7]*v[1]+m[8]*v[2];}
__device__ void ff_mul_t(const float* m,const float* v,float* out){out[0]=m[0]*v[0]+m[3]*v[1]+m[6]*v[2];out[1]=m[1]*v[0]+m[4]*v[1]+m[7]*v[2];out[2]=m[2]*v[0]+m[5]*v[1]+m[8]*v[2];}
__device__ void ff_semiaxes(const float* size,int type,float* out){out[0]=size[0];out[1]=size[1];out[2]=size[2];if(type==2){out[1]=size[0];out[2]=size[0];}else if(type==3){out[1]=size[0];out[2]=size[1]+size[0];}else if(type==5){out[1]=size[0];out[2]=size[1];}}
__device__ float ff_ellipsoid_moment(const float* size,unsigned dir){float d0=size[dir],d1=size[(dir+1)%3],d2=size[(dir+2)%3];return FF_EIGHT_FIFTEENTHS_PI*d0*ff_pow4(fmaxf(d1,d2));}
extern "C" __global__ void fluid_force_validate(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* attached,float* status,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;bool ok=true;
 const int* parent=raw+4;const int* root=parent+FF_NB;const int* body=root+FF_NB;const int* geom_body=body+FF_NV;const int* geom_type=geom_body+FF_NG;const int* ellipsoid=geom_type+FF_NG;
 for(unsigned i=0;i<4;++i)ok=ok&&raw[i]==-131072&&raw[4+3*FF_NB+FF_NV+2*FF_NG+i]==-131072&&p[i]==-131072.0f&&p[4+FF_PL+i]==-131072.0f;
 for(unsigned i=0;i<4;++i)ok=ok&&rigid[i]==-131072.0f&&rigid[4+(unsigned long long)worlds*FF_RS+i]==-131072.0f&&com[i]==-131072.0f&&com[4+(unsigned long long)worlds*FF_CS+i]==-131072.0f&&velocity[i]==-131072.0f&&velocity[4+(unsigned long long)worlds*FF_VS+i]==-131072.0f;
 if(FF_HE)for(unsigned i=0;i<4;++i)ok=ok&&attached[i]==-131072.0f&&attached[4+(unsigned long long)worlds*FF_AS+i]==-131072.0f;
 for(unsigned b=0;b<FF_NB;++b){int parentid=parent[b],rootid=root[b];bool parent_ok=(b==0&&parentid==0)||(b>0&&parentid>=0&&(unsigned)parentid<b);bool root_ok=rootid>=0&&(unsigned)rootid<FF_NB;if(!parent_ok||!root_ok){ok=false;continue;}int expected=b==0?0:(parentid==0?(int)b:root[parentid]);if(rootid!=expected||(ellipsoid[b]!=0&&ellipsoid[b]!=1))ok=false;}
 for(unsigned d=0;d<FF_NV;++d)if(body[d]<0||(unsigned)body[d]>=FF_NB)ok=false;
 if(!ok){status[4+w]=0.0f;return;}
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];const float* mass=ff_field(p,3,w,FF_NB);const float* inertia=ff_field(p,4,w,3*FF_NB);
 ok=ok&&isfinite(wind[0])&&isfinite(wind[1])&&isfinite(wind[2])&&isfinite(density)&&isfinite(viscosity);
 const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* ximat=r+19*FF_NB;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* cdof=c+14*FF_NB;const float* cvel=velocity+4+(unsigned long long)w*FF_VS;
 for(unsigned b=0;b<FF_NB;++b){ok=ok&&isfinite(mass[b])&&isfinite(inertia[3*b])&&isfinite(inertia[3*b+1])&&isfinite(inertia[3*b+2]);if(b==0||mass[b]<FF_MINVAL)continue;for(unsigned i=0;i<3;++i)ok=ok&&isfinite(xipos[3*b+i])&&isfinite(subtree[3*(unsigned)root[b]+i]);for(unsigned i=0;i<9;++i)ok=ok&&isfinite(ximat[9*b+i]);for(unsigned i=0;i<6;++i)ok=ok&&isfinite(cvel[6*b+i]);}
 const float* size=ff_field(p,5,w,3*FF_NG);const float* fluid=p+4+FF_GF;const float* frames=attached+4+(unsigned long long)w*FF_AS;const float* geom_xmat=frames+3*FF_NG;
 for(unsigned g=0;g<FF_NG;++g){if(geom_body[g]<0||(unsigned)geom_body[g]>=FF_NB)ok=false;for(unsigned i=0;i<3;++i){ok=ok&&isfinite(size[3*g+i]);if(FF_HE)ok=ok&&isfinite(frames[3*g+i]);}if(FF_HE)for(unsigned i=0;i<9;++i)ok=ok&&isfinite(geom_xmat[9*g+i]);for(unsigned i=0;i<12;++i)ok=ok&&isfinite(fluid[12*g+i]);}
 for(unsigned i=0;i<6*FF_NV;++i)ok=ok&&isfinite(cdof[i]);status[4+w]=ok?1.0f:0.0f;
}
extern "C" __global__ void fluid_force_body(const int* raw,const float* p,const float* rigid,const float* com,const float* velocity,const float* attached,float* output,unsigned nb,unsigned nv,unsigned worlds,unsigned work){
 unsigned tid=blockIdx.x*blockDim.x+threadIdx.x;if(tid>=work)return;unsigned b=tid%FF_NB,w=tid/FF_NB;float* out=output+4+6ull*tid;for(unsigned i=0;i<6;++i)out[i]=0.0f;if(b==0)return;
 const int* root=raw+4+FF_NB;const int* dof_body=root+FF_NB;const int* geom_body=dof_body+FF_NV;const int* geom_type=geom_body+FF_NG;const int* ellipsoid=geom_type+FF_NG;const float* mass=ff_field(p,3,w,FF_NB);const float body_mass=mass[b];if(body_mass<FF_MINVAL)return;
 const float* wind=ff_field(p,0,w,3);const float density=ff_field(p,1,w,1)[0];const float viscosity=ff_field(p,2,w,1)[0];bool has_viscosity=viscosity>0.0f,has_density=density>0.0f;
 const float* inertia=ff_field(p,4,w,3*FF_NB)+3*b;const float* r=rigid+4+(unsigned long long)w*FF_RS;const float* xipos=r+16*FF_NB;const float* rot=r+19*FF_NB+9*b;const float* c=com+4+(unsigned long long)w*FF_CS;const float* subtree=c+FF_NB;const float* v=velocity+4+(unsigned long long)w*FF_VS+6*b;
 float offset[3]={xipos[3*b]-subtree[3*(unsigned)root[b]],xipos[3*b+1]-subtree[3*(unsigned)root[b]+1],xipos[3*b+2]-subtree[3*(unsigned)root[b]+2]};
 float cross[3]={offset[1]*v[2]-offset[2]*v[1],offset[2]*v[0]-offset[0]*v[2],offset[0]*v[1]-offset[1]*v[0]};
 float lin[3]={v[3]-cross[0],v[4]-cross[1],v[5]-cross[2]};
 if(ellipsoid[b]){
  const float* frames=attached+4+(unsigned long long)w*FF_AS;const float* geom_xpos=frames;const float* geom_xmat=frames+3*FF_NG;const float* sizes=ff_field(p,5,w,3*FF_NG);const float* fluids=p+4+FF_GF;float force_global[3]={0.0f,0.0f,0.0f},torque_global[3]={0.0f,0.0f,0.0f};
  for(unsigned g=0;g<FF_NG;++g){if((unsigned)geom_body[g]!=b)continue;const float* fluid=fluids+12*g;float coef=fluid[0];if(coef<=0.0f)continue;const float* grot=geom_xmat+9*g;const float* gpos=geom_xpos+3*g;float semiaxes[3];ff_semiaxes(sizes+3*g,geom_type[g],semiaxes);
   float point_offset[3]={gpos[0]-xipos[3*b],gpos[1]-xipos[3*b+1],gpos[2]-xipos[3*b+2]},point_cross[3],lin_point[3];ff_cross(v,point_offset,point_cross);for(unsigned i=0;i<3;++i)lin_point[i]=lin[i]+point_cross[i];
   float lang[3],llin[3];ff_mul_t(grot,v,lang);ff_mul_t(grot,lin_point,llin);if(wind[0]!=0.0f||wind[1]!=0.0f||wind[2]!=0.0f){float local_wind[3];ff_mul_t(grot,wind,local_wind);for(unsigned i=0;i<3;++i)llin[i]-=local_wind[i];}
   float ltorque[3]={0.0f,0.0f,0.0f},lforce[3]={0.0f,0.0f,0.0f};
   if(has_density){float virtual_lin[3]={density*fluid[6]*llin[0],density*fluid[7]*llin[1],density*fluid[8]*llin[2]};float virtual_ang[3]={density*fluid[9]*lang[0],density*fluid[10]*lang[1],density*fluid[11]*lang[2]};float added_force[3],lin_cross[3],ang_cross[3];ff_cross(virtual_lin,lang,added_force);ff_cross(virtual_lin,llin,lin_cross);ff_cross(virtual_ang,lang,ang_cross);for(unsigned i=0;i<3;++i){lforce[i]+=added_force[i];ltorque[i]+=lin_cross[i]+ang_cross[i];}}
   float volume=FF_FOUR_THIRDS_PI*semiaxes[0]*semiaxes[1]*semiaxes[2];float dmax=fmaxf(fmaxf(semiaxes[0],semiaxes[1]),semiaxes[2]);float dmin=fminf(fminf(semiaxes[0],semiaxes[1]),semiaxes[2]);float dmid=semiaxes[0]+semiaxes[1]+semiaxes[2]-dmax-dmin;float amax=FF_PI*dmax*dmid;
   float lin_speed=sqrtf(ff_dot3(llin,llin));float inv_speed=lin_speed>FF_MINVAL?1.0f/lin_speed:0.0f;float unit[3]={llin[0]*inv_speed,llin[1]*inv_speed,llin[2]*inv_speed};float inv_dmax=dmax>FF_MINVAL?1.0f/dmax:0.0f;float scaled[3]={semiaxes[0]*inv_dmax,semiaxes[1]*inv_dmax,semiaxes[2]*inv_dmax};
   float magnus[3];ff_cross(lang,llin,magnus);float magnus_scale=fluid[5]*density*volume;for(unsigned i=0;i<3;++i)magnus[i]*=magnus_scale;
   float s12=scaled[1]*scaled[2],s20=scaled[2]*scaled[0],s01=scaled[0]*scaled[1];float s12_sq=s12*s12,s20_sq=s20*s20,s01_sq=s01*s01;float unit0_sq=unit[0]*unit[0],unit1_sq=unit[1]*unit[1],unit2_sq=unit[2]*unit[2];float proj_num=s12_sq*unit0_sq+s20_sq*unit1_sq+s01_sq*unit2_sq;float inv_proj=proj_num>FF_MINVAL?1.0f/proj_num:0.0f;float pa=s12_sq*inv_proj,pb=s20_sq*inv_proj,pc=s01_sq*inv_proj;float proj_denom=pa*pa*unit[0]*unit[0]+pb*pb*unit[1]*unit[1]+pc*pc*unit[2]*unit[2];float aproj=FF_PI*dmax*dmax*sqrtf(proj_num*proj_denom);float cos_alpha=proj_denom>FF_MINVAL?1.0f/proj_denom:0.0f;float norm[3]={pa*unit[0],pb*unit[1],pc*unit[2]};
   float kutta[3]={0.0f,0.0f,0.0f};if(has_density&&fluid[4]!=0.0f&&lin_speed>FF_MINVAL){float circ[3];ff_cross(norm,llin,circ);float circ_scale=fluid[4]*density*cos_alpha*aproj;for(unsigned i=0;i<3;++i)circ[i]*=circ_scale;ff_cross(circ,llin,kutta);}
   float eq_d=FF_TWO_THIRDS*(semiaxes[0]+semiaxes[1]+semiaxes[2]);float visc_force=FF_THREE_PI*eq_d;float visc_torque=FF_PI*eq_d*eq_d*eq_d;float imax=FF_EIGHT_FIFTEENTHS_PI*dmid*ff_pow4(dmax);float ii[3]={ff_ellipsoid_moment(semiaxes,0),ff_ellipsoid_moment(semiaxes,1),ff_ellipsoid_moment(semiaxes,2)};float mom[3]={lang[0]*(fluid[3]*ii[0]+fluid[2]*(imax-ii[0])),lang[1]*(fluid[3]*ii[1]+fluid[2]*(imax-ii[1])),lang[2]*(fluid[3]*ii[2]+fluid[2]*(imax-ii[2]))};float drag_lin=viscosity*visc_force+density*lin_speed*(aproj*fluid[1]+fluid[2]*(amax-aproj));float drag_ang=viscosity*visc_torque+density*sqrtf(ff_dot3(mom,mom));
   for(unsigned i=0;i<3;++i){ltorque[i]-=drag_ang*lang[i];lforce[i]+=magnus[i]+kutta[i]-drag_lin*llin[i];ltorque[i]*=coef;lforce[i]*=coef;}float world_force[3],world_torque[3];ff_mul(grot,lforce,world_force);ff_mul(grot,ltorque,world_torque);for(unsigned i=0;i<3;++i){force_global[i]+=world_force[i];torque_global[i]+=world_torque[i];}
  }
  for(unsigned i=0;i<3;++i){out[i]=force_global[i];out[3+i]=torque_global[i];}return;
 }
 if(!has_viscosity&&!has_density)return;
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
        AttachedModelInput::new(
            inertial,
            AttachedFields {
                geom_bodyid: vec![1, 1, 1, 2, 4],
                geom_pos: vec![
                    0.2, 0.0, 0.1, -0.15, 0.05, 0.0, 0.0, -0.2, 0.15, 0.1, 0.0, -0.1, 0.0, 0.0, 0.0,
                ],
                geom_quat: vec![
                    1.0, 0.0, 0.0, 0.0, 0.9238795, 0.0, 0.38268343, 0.0, 1.0, 0.0, 0.0, 0.0,
                    0.70710677, 0.0, 0.0, 0.70710677, 1.0, 0.0, 0.0, 0.0,
                ],
                ..Default::default()
            },
        )
        .unwrap()
    }

    fn simple_fluid() -> FluidForceInput {
        FluidForceInput {
            wind: ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.0, 0.5, -0.25, 0.125]).unwrap(),
            density: ParameterBatch::new(3, 1, vec![0.0, 0.5, 1.0]).unwrap(),
            viscosity: ParameterBatch::new(5, 1, vec![0.0, 0.125, 0.25, 0.375, 0.5]).unwrap(),
            geom_size: ParameterBatch::new(1, 0, Vec::new()).unwrap(),
            geom_type: Vec::new(),
            geom_fluid: Vec::new(),
            body_fluid_ellipsoid: vec![false; 3],
        }
    }

    fn fluid() -> FluidForceInput {
        let mut input = simple_fluid();
        input.body_fluid_ellipsoid.resize(5, false);
        input.geom_size = ParameterBatch::new(
            4,
            15,
            (0..4)
                .flat_map(|row| {
                    let delta = row as f32 * 0.015625;
                    [
                        0.2 + delta,
                        0.2,
                        0.2,
                        0.1,
                        0.25 + delta,
                        0.1,
                        0.3,
                        0.2,
                        0.15 + delta,
                        0.12,
                        0.22,
                        0.12,
                        0.18,
                        0.14,
                        0.1 + delta,
                    ]
                })
                .collect(),
        )
        .unwrap();
        input.geom_type = vec![2, 3, 4, 5, 6];
        input.geom_fluid = vec![0.0; 60];
        input
    }

    fn ellipsoid_fluid() -> FluidForceInput {
        let mut input = fluid();
        input.wind = ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.0, 0.25, -0.125, 0.0625]).unwrap();
        input.density = ParameterBatch::new(3, 1, vec![1.0, 0.5, 1.25]).unwrap();
        input.viscosity = ParameterBatch::new(5, 1, vec![0.2, 0.125, 0.25, 0.375, 0.5]).unwrap();
        input.body_fluid_ellipsoid = vec![false, true, true, false, true];
        for (geom, coefficient) in [1.0, 0.75, 0.0, -1.0, 1.0].into_iter().enumerate() {
            let field = &mut input.geom_fluid[12 * geom..12 * geom + 12];
            field.copy_from_slice(&[
                coefficient,
                0.8,
                0.35,
                0.2,
                0.45,
                0.3,
                0.04,
                0.06,
                0.08,
                0.02,
                0.03,
                0.05,
            ]);
            if coefficient == 0.0 {
                field[1..].fill(1.0e6);
            }
        }
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
        parameters.set(
            KinematicsParameter::GeomPos,
            ParameterBatch::new(
                3,
                15,
                (0..3)
                    .flat_map(|row| {
                        let delta = row as f32 * 0.03125;
                        [
                            0.2 + delta,
                            0.0,
                            0.1,
                            -0.15,
                            0.05 + delta,
                            0.0,
                            0.0,
                            -0.2,
                            0.15,
                            0.1,
                            0.0,
                            -0.1,
                            0.0,
                            0.0,
                            0.0,
                        ]
                    })
                    .collect(),
            )
            .unwrap(),
        );
        parameters.set(
            KinematicsParameter::GeomQuat,
            ParameterBatch::new(
                2,
                20,
                [
                    1.0, 0.0, 0.0, 0.0, 0.9238795, 0.0, 0.38268343, 0.0, 1.0, 0.0, 0.0, 0.0,
                    0.70710677, 0.0, 0.0, 0.70710677, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                    0.8660254, 0.0, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.70710677, 0.0, 0.0, 0.70710677,
                    1.0, 0.0, 0.0, 0.0,
                ]
                .to_vec(),
            )
            .unwrap(),
        );
        parameters
    }

    #[test]
    fn host_preserves_independent_periods_and_ellipsoid_inputs() {
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
        assert_eq!(simple.offsets, [0, 6, 9, 14, 35, 134]);
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
        assert_eq!(value.offsets, [0, 6, 9, 14, 49, 214]);
        assert_eq!(value.parameter_len, 334);
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
                value.input.geom_size.batches(),
            ],
            [2, 3, 5, 7, 11, 4]
        );
        let mut ellipsoid = ellipsoid_fluid();
        let accepted = spec(
            model.clone(),
            mass.clone(),
            inertia.clone(),
            ellipsoid.clone(),
        )
        .unwrap();
        assert_eq!(
            accepted.input.body_fluid_ellipsoid,
            [false, true, true, false, true]
        );
        ellipsoid.geom_fluid[0] = f32::NAN;
        assert!(matches!(
            spec(model.clone(), mass.clone(), inertia.clone(), ellipsoid),
            Err(InputError::NonFinite {
                field: "geom_fluid",
                index: 0
            })
        ));
        let mut invalid = fluid();
        invalid.geom_type.push(4);
        assert!(matches!(
            spec(model, mass, inertia, invalid),
            Err(InputError::LengthMismatch {
                field: "geom_type",
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

    #[test]
    fn ellipsoid_constants_and_projection_order_match_frozen_source() {
        assert_eq!(4.18879032135009765625f32.to_bits(), 0x4086_0a92);
        assert_eq!(1.6755161285400390625f32.to_bits(), 0x3fd6_7750);
        assert_eq!(0.666666686534881591796875f32.to_bits(), 0x3f2a_aaab);
        assert_eq!(9.424777984619140625f32.to_bits(), 0x4116_cbe4);
        for needle in [
            "float sq=x*x;return sq*sq",
            "float unit0_sq=unit[0]*unit[0]",
            "float proj_num=s12_sq*unit0_sq+s20_sq*unit1_sq+s01_sq*unit2_sq",
            "if(coef<=0.0f)continue",
            "if(has_density&&fluid[4]!=0.0f&&lin_speed>FF_MINVAL)",
            "force_global[i]+=world_force[i];torque_global[i]+=world_torque[i]",
        ] {
            assert!(CUDA.contains(needle), "missing frozen expression: {needle}");
        }
        assert!(!CUDA.contains("powf(dmax,4.0f)"));
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
        sources: (&[f32], &[f32], &[f32], Option<&[f32]>),
        mass: &[f32],
        inertia: &[f32],
        input: &FluidForceInput,
    ) -> (Vec<f32>, Vec<f32>) {
        let kinematic = model.rigid().kinematics();
        let inertial = model.rigid().fields();
        let (nb, nj, nv) = (kinematic.nbody(), kinematic.njnt(), kinematic.nv());
        let ng = model.ngeom();
        let rigid_stride = 28 * nb + 6 * nj;
        let com_stride = 14 * nb + 6 * nv;
        let velocity_stride = 6 * nb + 6 * nv;
        let attached_stride = 12 * (ng + model.nsite());
        let rigid = &sources.0[4 + world * rigid_stride..];
        let com = &sources.1[4 + world * com_stride..];
        let velocity = &sources.2[4 + world * velocity_stride..];
        let attached = sources
            .3
            .map(|values| &values[4 + world * attached_stride..]);
        let wind = input.wind.world(world);
        let density = input.density.world(world)[0];
        let viscosity = input.viscosity.world(world)[0];
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
            if input.body_fluid_ellipsoid[body] {
                let attached = attached.expect("ellipsoid reference requires attached frames");
                let geom_xpos = &attached[..3 * ng];
                let geom_xmat = &attached[3 * ng..12 * ng];
                let sizes = input.geom_size.world(world);
                let mut force_global = [0.0f32; 3];
                let mut torque_global = [0.0f32; 3];
                for geom in 0..ng {
                    if model.fields().geom_bodyid[geom] as usize != body {
                        continue;
                    }
                    let fluid = &input.geom_fluid[12 * geom..12 * geom + 12];
                    let coefficient = fluid[0];
                    if coefficient <= 0.0 {
                        continue;
                    }
                    let size = &sizes[3 * geom..3 * geom + 3];
                    let semiaxes = match input.geom_type[geom] {
                        2 => [size[0], size[0], size[0]],
                        3 => [size[0], size[0], size[1] + size[0]],
                        5 => [size[0], size[0], size[1]],
                        _ => [size[0], size[1], size[2]],
                    };
                    let rotation = &geom_xmat[9 * geom..9 * geom + 9];
                    let position = &geom_xpos[3 * geom..3 * geom + 3];
                    let point_offset = [
                        position[0] - xipos[3 * body],
                        position[1] - xipos[3 * body + 1],
                        position[2] - xipos[3 * body + 2],
                    ];
                    let point_cross = cross(angular, point_offset);
                    let linear_point = [
                        linear_com[0] + point_cross[0],
                        linear_com[1] + point_cross[1],
                        linear_com[2] + point_cross[2],
                    ];
                    let lang = mul_transpose(rotation, angular);
                    let mut llin = mul_transpose(rotation, linear_point);
                    if wind[0] != 0.0 || wind[1] != 0.0 || wind[2] != 0.0 {
                        let local_wind = mul_transpose(rotation, [wind[0], wind[1], wind[2]]);
                        for index in 0..3 {
                            llin[index] -= local_wind[index];
                        }
                    }
                    let mut local_force = [0.0f32; 3];
                    let mut local_torque = [0.0f32; 3];
                    if density > 0.0 {
                        let virtual_linear = [
                            density * fluid[6] * llin[0],
                            density * fluid[7] * llin[1],
                            density * fluid[8] * llin[2],
                        ];
                        let virtual_angular = [
                            density * fluid[9] * lang[0],
                            density * fluid[10] * lang[1],
                            density * fluid[11] * lang[2],
                        ];
                        let added_force = cross(virtual_linear, lang);
                        let linear_torque = cross(virtual_linear, llin);
                        let angular_torque = cross(virtual_angular, lang);
                        for index in 0..3 {
                            local_force[index] += added_force[index];
                            local_torque[index] += linear_torque[index] + angular_torque[index];
                        }
                    }
                    const FOUR_THIRDS_PI: f32 = 4.18879032135009765625;
                    const EIGHT_FIFTEENTHS_PI: f32 = 1.6755161285400390625;
                    const TWO_THIRDS: f32 = 0.666666686534881591796875;
                    const THREE_PI: f32 = 9.424777984619140625;
                    let volume = FOUR_THIRDS_PI * semiaxes[0] * semiaxes[1] * semiaxes[2];
                    let maximum = semiaxes[0].max(semiaxes[1]).max(semiaxes[2]);
                    let minimum = semiaxes[0].min(semiaxes[1]).min(semiaxes[2]);
                    let middle = semiaxes[0] + semiaxes[1] + semiaxes[2] - maximum - minimum;
                    let maximum_area = std::f32::consts::PI * maximum * middle;
                    let speed = (llin[0] * llin[0] + llin[1] * llin[1] + llin[2] * llin[2]).sqrt();
                    let inverse_speed = if speed > 1.0e-15 { 1.0 / speed } else { 0.0 };
                    let unit = [
                        llin[0] * inverse_speed,
                        llin[1] * inverse_speed,
                        llin[2] * inverse_speed,
                    ];
                    let inverse_maximum = if maximum > 1.0e-15 {
                        1.0 / maximum
                    } else {
                        0.0
                    };
                    let scaled = [
                        semiaxes[0] * inverse_maximum,
                        semiaxes[1] * inverse_maximum,
                        semiaxes[2] * inverse_maximum,
                    ];
                    let mut magnus = cross(lang, llin);
                    let magnus_scale = fluid[5] * density * volume;
                    for value in &mut magnus {
                        *value *= magnus_scale;
                    }
                    let s12 = scaled[1] * scaled[2];
                    let s20 = scaled[2] * scaled[0];
                    let s01 = scaled[0] * scaled[1];
                    let s12_sq = s12 * s12;
                    let s20_sq = s20 * s20;
                    let s01_sq = s01 * s01;
                    let unit_sq = [unit[0] * unit[0], unit[1] * unit[1], unit[2] * unit[2]];
                    let projection_numerator =
                        s12_sq * unit_sq[0] + s20_sq * unit_sq[1] + s01_sq * unit_sq[2];
                    let inverse_projection = if projection_numerator > 1.0e-15 {
                        1.0 / projection_numerator
                    } else {
                        0.0
                    };
                    let projection = [
                        s12_sq * inverse_projection,
                        s20_sq * inverse_projection,
                        s01_sq * inverse_projection,
                    ];
                    let projection_denominator = projection[0] * projection[0] * unit[0] * unit[0]
                        + projection[1] * projection[1] * unit[1] * unit[1]
                        + projection[2] * projection[2] * unit[2] * unit[2];
                    let projected_area = std::f32::consts::PI
                        * maximum
                        * maximum
                        * (projection_numerator * projection_denominator).sqrt();
                    let cosine = if projection_denominator > 1.0e-15 {
                        1.0 / projection_denominator
                    } else {
                        0.0
                    };
                    let normal = [
                        projection[0] * unit[0],
                        projection[1] * unit[1],
                        projection[2] * unit[2],
                    ];
                    let mut kutta = [0.0f32; 3];
                    if density > 0.0 && fluid[4] != 0.0 && speed > 1.0e-15 {
                        let mut circulation = cross(normal, llin);
                        let scale = fluid[4] * density * cosine * projected_area;
                        for value in &mut circulation {
                            *value *= scale;
                        }
                        kutta = cross(circulation, llin);
                    }
                    let equivalent_diameter =
                        TWO_THIRDS * (semiaxes[0] + semiaxes[1] + semiaxes[2]);
                    let linear_viscosity = THREE_PI * equivalent_diameter;
                    let angular_viscosity = std::f32::consts::PI
                        * equivalent_diameter
                        * equivalent_diameter
                        * equivalent_diameter;
                    let power4 = |value: f32| {
                        let square = value * value;
                        square * square
                    };
                    let maximum_moment = EIGHT_FIFTEENTHS_PI * middle * power4(maximum);
                    let moment = |direction: usize| {
                        let d0 = semiaxes[direction];
                        let d1 = semiaxes[(direction + 1) % 3];
                        let d2 = semiaxes[(direction + 2) % 3];
                        EIGHT_FIFTEENTHS_PI * d0 * power4(d1.max(d2))
                    };
                    let moments = [moment(0), moment(1), moment(2)];
                    let viscous_moment = [
                        lang[0]
                            * (fluid[3] * moments[0] + fluid[2] * (maximum_moment - moments[0])),
                        lang[1]
                            * (fluid[3] * moments[1] + fluid[2] * (maximum_moment - moments[1])),
                        lang[2]
                            * (fluid[3] * moments[2] + fluid[2] * (maximum_moment - moments[2])),
                    ];
                    let linear_drag = viscosity * linear_viscosity
                        + density
                            * speed
                            * (projected_area * fluid[1]
                                + fluid[2] * (maximum_area - projected_area));
                    let angular_drag = viscosity * angular_viscosity
                        + density
                            * (viscous_moment[0] * viscous_moment[0]
                                + viscous_moment[1] * viscous_moment[1]
                                + viscous_moment[2] * viscous_moment[2])
                                .sqrt();
                    for index in 0..3 {
                        local_torque[index] -= angular_drag * lang[index];
                        local_force[index] +=
                            magnus[index] + kutta[index] - linear_drag * llin[index];
                        local_torque[index] *= coefficient;
                        local_force[index] *= coefficient;
                    }
                    let force = mul(rotation, local_force);
                    let torque = mul(rotation, local_torque);
                    for index in 0..3 {
                        force_global[index] += force[index];
                        torque_global[index] += torque[index];
                    }
                }
                body_force[6 * body..6 * body + 3].copy_from_slice(&force_global);
                body_force[6 * body + 3..6 * body + 6].copy_from_slice(&torque_global);
                continue;
            }
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
        let simple_fields = simple_plan
            .fluid_snapshot_for_test(&simple_source, false)
            .unwrap();
        let mut simple_data = simple_fluid_plan.create_data(WORLDS).unwrap();
        let simple_output = simple_fluid_plan
            .update(&mut simple_source, &mut simple_data)
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
                (
                    &simple_fields.0,
                    &simple_fields.1,
                    &simple_fields.2,
                    simple_fields.3.as_deref(),
                ),
                simple_mass.world(world),
                simple_inertia.world(world),
                &simple_fluid,
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
        let source_fields = plan.fluid_snapshot_for_test(&source, false).unwrap();
        let mut data = fluid_plan.create_data(WORLDS).unwrap();
        let output = fluid_plan
            .update(&mut source, &mut data)
            .unwrap()
            .snapshot()
            .unwrap();

        let mass = kinematics.get(KinematicsParameter::BodyMass).unwrap();
        let inertia = kinematics.get(KinematicsParameter::BodyInertia).unwrap();
        for world in 0..WORLDS {
            let (expected_body, expected_qfrc) = reference(
                &complex_model,
                world,
                (
                    &source_fields.0,
                    &source_fields.1,
                    &source_fields.2,
                    source_fields.3.as_deref(),
                ),
                mass.world(world),
                inertia.world(world),
                &fluid_input,
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
            fluid_plan.update(&mut source, &mut data),
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
            "ellipsoid_case_in_this_test": false,
            "frozen_upstream_device_output": false,
            "elapsed_ms": started.elapsed().as_millis(),
            "performance_acceptance": false,
        });
        if let Some(path) = std::env::var_os("MJWARP_FLUID_FORCE_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }

    #[test]
    #[cfg(feature = "cuda-probe")]
    #[ignore = "new G04 ellipsoid fluid branch"]
    fn gpu_matches_independent_ellipsoid_reference() {
        const WORLDS: usize = 9;
        const TOLERANCE: f32 = 2.0e-6;
        let started = std::time::Instant::now();
        let session = crate::runtime::TransferSession::new(0).unwrap();
        let model = model();
        let parameters = kinematics_parameters();
        let plan =
            ComVelocityPlan::with_parameters(&session, model.clone(), parameters.clone()).unwrap();
        let input = ellipsoid_fluid();
        let fluid_plan = FluidForcePlan::new(&plan, input.clone()).unwrap();
        let mut source = plan.create_data(WORLDS).unwrap();
        let qpos = (0..WORLDS)
            .flat_map(|world| {
                let scale = world as f32;
                [0.07 * scale, -0.045 * scale]
            })
            .collect::<Vec<_>>();
        source.write_qpos(&qpos).unwrap();
        let qvel = (0..WORLDS)
            .flat_map(|world| {
                if world == 0 {
                    [0.0, 0.0]
                } else {
                    [0.12 + 0.025 * world as f32, -0.08 + 0.0175 * world as f32]
                }
            })
            .collect::<Vec<_>>();
        source.write_qvel(&qvel).unwrap();
        plan.update(&mut source).unwrap();
        let mut data = fluid_plan.create_data(WORLDS).unwrap();
        let output = fluid_plan
            .update(&mut source, &mut data)
            .unwrap()
            .snapshot()
            .unwrap();
        let fields = plan.fluid_snapshot_for_test(&source, true).unwrap();
        let mass = parameters.get(KinematicsParameter::BodyMass).unwrap();
        let inertia = parameters.get(KinematicsParameter::BodyInertia).unwrap();
        let mut compared = 0usize;
        let mut max_absolute = 0.0f32;
        let mut saw_nondegenerate = false;
        for world in 0..WORLDS {
            let (expected_body, expected_qfrc) = reference(
                &model,
                world,
                (&fields.0, &fields.1, &fields.2, fields.3.as_deref()),
                mass.world(world),
                inertia.world(world),
                &input,
            );
            let actual = output.world(world).unwrap();
            if world == 0 {
                assert_eq!(actual.body_force_torque, &[0.0; 30]);
                assert_eq!(actual.qfrc, &[0.0; 2]);
            }
            assert_eq!(&actual.body_force_torque[12..18], &[0.0; 6]);
            assert_eq!(&actual.body_force_torque[24..30], &[0.0; 6]);
            saw_nondegenerate |= actual.body_force_torque[6..12]
                .iter()
                .any(|value| *value != 0.0);
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
        assert!(saw_nondegenerate);
        assert_eq!(compared, 288);
        let report = serde_json::json!({
            "worlds": WORLDS,
            "compared_slots": compared,
            "absolute_tolerance": TOLERANCE,
            "max_absolute": max_absolute,
            "parameter_periods": {
                "wind": 2,
                "density": 3,
                "viscosity": 5,
                "body_mass": 7,
                "body_inertia": 11,
                "geom_size": 4,
                "geom_pos": 3,
                "geom_quat": 2
            },
            "active_geoms_on_one_body": 2,
            "zero_coefficient_skipped": true,
            "negative_coefficient_skipped": true,
            "low_mass_body_skipped": true,
            "zero_speed_boundary": true,
            "extra_lever_arm_torque": false,
            "frozen_upstream_device_output": false,
            "elapsed_ms": started.elapsed().as_millis(),
            "performance_acceptance": false
        });
        if let Some(path) = std::env::var_os("MJWARP_ELLIPSOID_FLUID_REPORT") {
            std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        println!("{report}");
    }
}
