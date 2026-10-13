//! 原生独占快照的宿主批量适配。
//! 参数覆盖保持f32，不调用原生物理。
use super::NativePrimitiveGeometrySnapshot;
use crate::{
    diagnostics::NativeProbeError,
    model::{PrimitiveGeometryHostBatch, PrimitiveGeometryParameters},
};

impl NativePrimitiveGeometrySnapshot {
    /// 先执行原ABI检查与一次量化。
    /// 结果独占默认模型及字段覆盖。
    pub fn into_host_batch(
        self,
        worlds: usize,
        parameters: PrimitiveGeometryParameters,
    ) -> Result<PrimitiveGeometryHostBatch, NativeProbeError> {
        Ok(PrimitiveGeometryHostBatch::new(
            self.into_input()?,
            worlds,
            parameters,
        )?)
    }
}
