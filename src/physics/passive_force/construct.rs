//! 被动合力计划构造与共享源绑定。

use super::*;

impl<'a, 'v, 'p, 'f> PassiveForcePlan<'a, 'v, 'p, 'f> {
    pub fn new(
        joint: &'a JointPassivePlan,
        tendon: &'a TendonPassivePlan<'v, 'p>,
        flex_edge: &'a FlexEdgePassivePlan<'f>,
    ) -> Result<Self, TransferError> {
        Self::build(joint, tendon, flex_edge, None, None)
    }

    pub fn with_gravity_compensation(
        joint: &'a JointPassivePlan,
        tendon: &'a TendonPassivePlan<'v, 'p>,
        flex_edge: &'a FlexEdgePassivePlan<'f>,
        parameters: GravityCompensationParameters,
        options: GravityCompensationOptions,
    ) -> Result<Self, TransferError> {
        let gravity =
            GravityCompensationPlan::new(flex_edge.shared_source_plan(), parameters, options)?;
        Self::build(joint, tendon, flex_edge, Some(gravity), None)
    }

    pub fn with_fluid_force(
        joint: &'a JointPassivePlan,
        tendon: &'a TendonPassivePlan<'v, 'p>,
        flex_edge: &'a FlexEdgePassivePlan<'f>,
        input: FluidForceInput,
    ) -> Result<Self, TransferError> {
        let fluid = FluidForcePlan::from_resident(flex_edge.shared_source_plan(), input)?;
        Self::build(joint, tendon, flex_edge, None, Some(fluid))
    }

    pub fn with_gravity_and_fluid(
        joint: &'a JointPassivePlan,
        tendon: &'a TendonPassivePlan<'v, 'p>,
        flex_edge: &'a FlexEdgePassivePlan<'f>,
        gravity_parameters: GravityCompensationParameters,
        gravity_options: GravityCompensationOptions,
        fluid_input: FluidForceInput,
    ) -> Result<Self, TransferError> {
        let source = flex_edge.shared_source_plan();
        let gravity = GravityCompensationPlan::new(source, gravity_parameters, gravity_options)?;
        let fluid = FluidForcePlan::from_resident(source, fluid_input)?;
        Self::build(joint, tendon, flex_edge, Some(gravity), Some(fluid))
    }

    fn build(
        joint: &'a JointPassivePlan,
        tendon: &'a TendonPassivePlan<'v, 'p>,
        flex_edge: &'a FlexEdgePassivePlan<'f>,
        gravity: Option<GravityCompensationPlan<'f>>,
        fluid: Option<FluidForcePlan<'f>>,
    ) -> Result<Self, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (joint, tendon, flex_edge, gravity, fluid);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let (session, smooth_input, nv, joint_options) = joint.shared_passive_context();
            let (tendon_session, tendon_nv, tendon_options) = tendon.shared_passive_context();
            let (flex_session, flex_nv, flex_options) = flex_edge.shared_passive_context();
            let same_options = joint_options.disable_spring == tendon_options.disable_spring
                && joint_options.disable_spring == flex_options.disable_spring
                && joint_options.disable_damper == tendon_options.disable_damper
                && joint_options.disable_damper == flex_options.disable_damper;
            if nv != tendon_nv
                || nv != flex_nv
                || !same_options
                || !flex_edge.matches_shared_model(joint.shared_passive_model())
                || !session.same_session(&tendon_session)
                || !session.same_session(&flex_session)
                || fluid.as_ref().is_some_and(|fluid| {
                    !fluid.accepts_resident_source(flex_edge.shared_source_plan())
                })
            {
                return Err(TransferError::ModelMismatch);
            }
            let (smooth_input, smooth_options) =
                if let Some((ntree, body_treeid, sleep_enabled, island_disabled)) =
                    flex_edge.shared_sleep_context()
                {
                    (
                        SmoothForceInput::new(
                            ntree,
                            body_treeid.to_vec(),
                            smooth_input.dof_bodyid().to_vec(),
                        )?,
                        SmoothForceOptions {
                            sleep_enabled,
                            island_disabled,
                        },
                    )
                } else {
                    (smooth_input, SmoothForceOptions::default())
                };
            let smooth = SmoothForcePlan::new(&session, smooth_input, smooth_options)?;
            let metadata = session.upload(&[-131072; 8])?;
            let dummy = session.upload(&[-131072.0; 8])?;
            let code = format!("#define PF_NV {nv}\n{CUDA}");
            Ok(Self {
                joint,
                tendon,
                flex_edge,
                gravity,
                fluid,
                identity: Arc::new(()),
                smooth,
                nv,
                disable_spring: joint_options.disable_spring,
                disable_damper: joint_options.disable_damper,
                metadata,
                dummy,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "passive_force_clear")?,
                    SynchronousKernel::compile(&session, &code, "passive_force_sum")?,
                    SynchronousKernel::compile(&session, &code, "passive_force_clear_gravity")?,
                    SynchronousKernel::compile(&session, &code, "passive_force_add_fluid")?,
                ],
                session,
            })
        }
    }
}
