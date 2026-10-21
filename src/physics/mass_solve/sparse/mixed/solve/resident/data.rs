//! 常驻混合数据写入、失效与回读。

use super::*;

impl SparseMassMixedResidentData {
    pub fn worlds(&self) -> usize {
        self.matrix.worlds()
    }
    pub fn rhs_count(&self) -> usize {
        self.solve.rhs_count()
    }
    pub(super) fn invalidate(&mut self) {
        self.gathered = false;
        self.solve.ready = false;
        self.solve.mass.invalidate();
    }
    pub(super) fn after_qpos(
        &mut self,
        result: Result<(), TransferError>,
    ) -> Result<(), TransferError> {
        if !matches!(result, Err(TransferError::Input(_))) {
            self.invalidate();
        }
        result
    }
    pub fn write_qpos(&mut self, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_qpos(values);
        self.after_qpos(result)
    }
    pub fn write_world_qpos(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let result = self.matrix.write_world_qpos(world, values);
        self.after_qpos(result)
    }
    pub fn write_rhs(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.solve.write_rhs(values)
    }
    pub fn write_world_rhs(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        self.solve.write_world_rhs(world, values)
    }
    /// 仅读取完成合并因子；陈旧CRB不决定缓存解寿命。
    pub fn readback(&self) -> Result<SparseMassMixedSolveOutput, TransferError> {
        self.solve.readback()
    }
    /// 独立诊断当前GPU矩阵；不改变求解缓存。
    pub fn readback_matrix(&self) -> Result<MassMatrixOutput, TransferError> {
        self.matrix.readback()
    }
}
