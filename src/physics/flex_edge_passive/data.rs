//! 柔体边被动力数据与快照视图。

use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;

impl FlexEdgePassiveData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
}

impl FlexEdgePassiveView<'_, '_> {
    pub fn snapshot(&self) -> Result<FlexEdgePassiveSnapshot, TransferError> {
        if !self.data.ready {
            return Err(TransferError::StageNotReady {
                stage: "flex_edge_passive",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.plan.source.flex_edge_passive_source(self.source)?;
            Ok(FlexEdgePassiveSnapshot {
                layout: self.data.layout,
                values: read(
                    &self.data.current,
                    2 * self.data.layout.total_elements(),
                    "flex_edge_passive",
                )?,
            })
        }
    }
}
