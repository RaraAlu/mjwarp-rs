//! 常驻快照字段访问器。

use super::*;

impl KinematicsSnapshot {
    /// 显式计算完成才提供矩阵。
    /// 位置刷新后返回None。
    pub fn flex_hessian(&self) -> Option<&FlexHessianOutput> {
        self.hessian.as_ref()
    }
    pub fn flex_faces(&self) -> Option<&FlexFaceOutput> {
        self.faces.as_ref()
    }
    pub fn flex_edges(&self) -> Option<&FlexEdgeOutput> {
        self.edges.as_ref()
    }
    pub fn sleep_trees(&self) -> Option<&SleepTreeOutput> {
        self.sleep.as_ref()
    }
    /// 柔体入口提供位置结果。
    /// 旧入口返回None。
    pub fn flex_positions(&self) -> Option<&FlexPositionOutput> {
        self.flex.as_ref()
    }
    /// 混合入口提供全局结果。
    /// 旧局部入口返回None。
    pub fn tendon(&self) -> Option<&TendonOutput> {
        self.tendon.as_ref()
    }
    pub fn spatial_tendon(&self) -> &SpatialTendonOutput {
        &self.spatial_tendon
    }
    pub fn fixed_tendon(&self) -> &FixedTendonOutput {
        &self.fixed_tendon
    }
    pub fn camlight(&self) -> &CamLightOutput {
        &self.camlight
    }
    pub fn rigid(&self) -> &KinematicsOutput {
        self.attached.rigid()
    }
    pub fn attached(&self) -> &AttachedKinematicsOutput {
        &self.attached
    }
    pub fn com(&self) -> &ComPositionOutput {
        &self.com
    }
}
