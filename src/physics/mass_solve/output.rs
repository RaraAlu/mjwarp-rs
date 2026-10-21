//! 质量求解输出、世界视图与检查。

use super::*;

/// 每世界分解与求解的只读视图。
#[derive(Clone, Copy, Debug)]
pub struct MassSolveWorld<'a> {
    /// 兼容字段名；按blocks解释稠密行优先因子。
    /// 不再统一采用旧版反向LDL格式。
    pub ld: &'a [f32],
    /// 对角块存质量倒数；LDL块存D倒数。
    /// Cholesky块填零，不提供无语义倒数。
    pub diagonal_inverse: &'a [f32],
    pub blocks: &'a [MassFactorBlock],
    /// 多右端项按`[rhs,nv]`连续排列。
    pub solution: &'a [f32],
}

/// 独立拥有的同步宿主结果。
#[derive(Clone, Debug)]
pub struct MassSolveOutput {
    pub(super) layout: MassSolveLayout,
    pub(super) values: Vec<f32>,
    pub(super) blocks: Vec<MassFactorBlock>,
}
impl MassSolveOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn rhs_count(&self) -> usize {
        self.layout.rhs_count
    }
    pub fn world(&self, world: usize) -> Result<MassSolveWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 7..r.end + 4];
        let square = self.nv() * self.nv();
        Ok(MassSolveWorld {
            blocks: &self.blocks,
            ld: &v[..square],
            diagonal_inverse: &v[square..square + self.nv()],
            solution: &v[square + self.nv()..],
        })
    }
}
