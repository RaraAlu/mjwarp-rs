//! 计划访问器。

use super::*;

impl KinematicsPlan {
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_kinematic_model(&self) -> &crate::model::KinematicModelInput {
        self.model.rigid().kinematics()
    }

    pub(in crate::physics) fn gravity_compensation_context(
        &self,
    ) -> Result<(TransferSession, Arc<AttachedModelInput>, ParameterBatch), TransferError> {
        let body_mass = self
            .parameters
            .get(KinematicsParameter::BodyMass)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| {
                ParameterBatch::new(
                    1,
                    self.model.rigid().kinematics().nbody(),
                    self.model.rigid().fields().body_mass.clone(),
                )
            })?;
        Ok((self.session.clone(), Arc::clone(&self.model), body_mass))
    }

    pub(in crate::physics) fn fluid_force_context(
        &self,
    ) -> Result<
        (
            TransferSession,
            Arc<AttachedModelInput>,
            ParameterBatch,
            ParameterBatch,
        ),
        TransferError,
    > {
        let nbody = self.model.rigid().kinematics().nbody();
        let inertial = self.model.rigid().fields();
        let body_mass = self
            .parameters
            .get(KinematicsParameter::BodyMass)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| ParameterBatch::new(1, nbody, inertial.body_mass.clone()))?;
        let body_inertia = self
            .parameters
            .get(KinematicsParameter::BodyInertia)
            .cloned()
            .map(Ok)
            .unwrap_or_else(|| ParameterBatch::new(1, 3 * nbody, inertial.body_inertia.clone()))?;
        Ok((
            self.session.clone(),
            Arc::clone(&self.model),
            body_mass,
            body_inertia,
        ))
    }

    pub fn flex_hessian_fields(&self) -> Option<&FlexHessianFields> {
        self.hessian_fields.as_deref()
    }
    pub fn flex_face_fields(&self) -> Option<&FlexFaceFields> {
        self.face_fields.as_deref()
    }
    pub fn flex_edge_fields(&self) -> Option<&FlexEdgeFields> {
        self.edge_fields.as_deref()
    }
    pub fn device(&self) -> usize {
        self.session.device()
    }

    pub fn parameters(&self) -> &KinematicsParameters {
        &self.parameters
    }

    pub fn nmocap(&self) -> usize {
        self.nmocap
    }

    pub fn body_mocapid(&self) -> &[i32] {
        &self.body_mocapid
    }

    pub fn camlight_parameters(&self) -> &CamLightParameters {
        &self.camlight_parameters
    }
    pub fn ncam(&self) -> usize {
        self.ncam
    }
    pub fn nlight(&self) -> usize {
        self.nlight
    }
    pub fn fixed_tendon_rows(&self) -> &FixedTendonRows {
        &self.fixed_tendon_rows
    }
    pub fn spatial_tendon_rows(&self) -> &SpatialTendonRows {
        &self.spatial_tendon_rows
    }
    pub fn tendon_rows(&self) -> Option<&TendonRows> {
        self.tendon_rows.as_deref()
    }
}
