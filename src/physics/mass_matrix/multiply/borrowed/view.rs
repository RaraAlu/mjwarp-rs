//! 借用矩阵只读视图实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};

use super::super::MassMultiplyOutput;
use super::*;

impl MassMatrixMultiplyView<'_> {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.source.worlds(), self.data.worlds());
        self.source.worlds()
    }
    /// 只标记本次计算的世界；skip缓存不认证当前态。
    pub fn is_current(&self, world: usize) -> Result<bool, InputError> {
        self.data.vector_layout.world_elements(world)?;
        Ok(self.computed[world])
    }
    /// 完整当前态消费者必须先通过此门禁。
    pub fn fresh_snapshot(&self) -> Result<MassMultiplyOutput, TransferError> {
        if self.computed.iter().any(|&v| !v) {
            return Err(TransferError::StageNotReady {
                stage: "borrowed_mul_m_all_current",
            });
        }
        self.snapshot()
    }
    /// 返回历史结果；跳过槽不代表当前矩阵或向量。
    pub fn snapshot(&self) -> Result<MassMultiplyOutput, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(MassMultiplyOutput {
                layout: self.data.vector_layout,
                values: read(
                    &self.data.current,
                    self.data.vector_layout.total_elements(),
                    "borrowed_Mqvel",
                )?,
            })
        }
    }
}
