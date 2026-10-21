//! 混合因子计划实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
use crate::runtime::TransferSession;
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::merge::MixedFactorMergeKernel;
use super::*;

impl SparseMassMixedFactorPlan {
    pub fn new(session: &TransferSession, input: SparseMassInput) -> Result<Self, TransferError> {
        routes(&input)?;
        let small = SparseMassSmallPlan::new(session, input.clone())?;
        let sparse = SparseMassFactorPlan::from_input(session, input)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (small, sparse);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let metadata = session.upload(&[0i32])?;
            let merge = MixedFactorMergeKernel::compile(session, sparse.input())?;
            Ok(Self {
                sparse,
                small,
                session: session.clone(),
                metadata,
                merge,
            })
        }
    }
    pub fn input(&self) -> &SparseMassInput {
        self.sparse.input()
    }
    pub fn create_data(&self, worlds: usize) -> Result<SparseMassMixedFactorData, TransferError> {
        let layout = layout(self.input(), worlds)?;
        let sparse = self.sparse.create_data(worlds)?;
        let small = self.small.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, sparse, small);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.total_elements() + 8)?;
            values.fill(-131072.0);
            values[4..4 + layout.total_elements()].fill(f32::NAN);
            let mut matrix =
                crate::runtime::host_staging::<f32>(sparse.matrix_layout.total_elements() + 8)?;
            matrix.fill(-131072.0);
            matrix[4..4 + sparse.matrix_layout.total_elements()].fill(0.0);
            Ok(SparseMassMixedFactorData {
                sparse,
                small,
                layout,
                gathered: false,
                ready: false,
                matrix: self.session.upload(&matrix)?,
                output: self.session.upload(&values)?,
            })
        }
    }
    pub(super) fn check_data(&self, data: &SparseMassMixedFactorData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.sparse.input, &data.sparse.input) {
            return Err(TransferError::ModelMismatch);
        }
        Ok(())
    }
    pub fn update(&self, data: &mut SparseMassMixedFactorData) -> Result<(), TransferError> {
        self.update_gather(data)?;
        self.update_factor(data)
    }
    pub fn update_gather(&self, data: &mut SparseMassMixedFactorData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Freeze the main CSR once for both producers. Later main-buffer
            // changes cannot mix a new sparse tail with an old Small prefix.
            // Both copies include guards and finish before either factor stage.
            data.sparse
                .matrix
                .copy_range_from(0, &data.matrix, 0, data.matrix.len())?;
            data.small.copy_matrix_from(&data.sparse.matrix)?;
            self.small.update_gather(&mut data.small)?;
            data.gathered = true;
            Ok(())
        }
    }
    pub fn update_factor(&self, data: &mut SparseMassMixedFactorData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = false;
        if !data.gathered {
            return Err(TransferError::StageNotReady {
                stage: "mixed_factor_gather",
            });
        }
        self.small.update_factor(&mut data.small)?;
        self.sparse.update_factor(&mut data.sparse)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.merge.launch(
                &self.metadata,
                data.small.factor_buffer()?,
                &data.sparse.output,
                &mut data.output,
                data.layout.worlds(),
            )?;
            data.checked_values()?;
            data.ready = true;
            Ok(())
        }
    }
}
