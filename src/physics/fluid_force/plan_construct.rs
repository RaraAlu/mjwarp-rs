//! 流体计划构造与独立源绑定。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::spec::spec;
use super::*;

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

    pub(in crate::physics) fn from_resident(
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
}
