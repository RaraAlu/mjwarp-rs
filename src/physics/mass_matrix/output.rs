//! 质量矩阵宿主视图、结果与布局。

use crate::diagnostics::InputError;
use crate::model::BatchLayout;

/// 同步结果的只读世界视图。
#[derive(Clone, Copy, Debug)]
pub struct MassMatrixWorld<'a> {
    /// 每体十项复合惯量。
    /// 打包顺序沿用cinert。
    pub crb: &'a [f32],
    /// 对称稠密矩阵，按行展开。
    /// 混合坐标不共享同一单位。
    pub matrix: &'a [f32],
}

/// GPU质量矩阵的宿主结果。
/// 结果不承诺正定或可逆。
#[derive(Clone, Debug)]
pub struct MassMatrixOutput {
    pub(super) layout: MassMatrixLayout,
    pub(super) values: Vec<f32>,
}
impl MassMatrixOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<MassMatrixWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let values = &self.values[r.start + 4..r.end + 4];
        Ok(MassMatrixWorld {
            crb: &values[..10 * self.nbody()],
            matrix: &values[10 * self.nbody()..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MassMatrixLayout {
    pub(super) output: BatchLayout,
    pub(super) nbody: usize,
    pub(super) nv: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) metadata: usize,
    #[cfg(feature = "cuda-probe")]
    pub(super) guarded: usize,
}
impl MassMatrixLayout {
    pub(super) fn new(worlds: usize, nbody: usize, nv: usize) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "mass_matrix_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "mass_matrix_layout",
        };
        let metadata = nv
            .checked_mul(2)
            .and_then(|n| n.checked_add(nbody))
            .ok_or_else(overflow)?;
        let stride = nbody
            .checked_mul(10)
            .and_then(|n| nv.checked_mul(nv).and_then(|m| n.checked_add(m)))
            .ok_or_else(overflow)?;
        if metadata > i32::MAX as usize || stride > i32::MAX as usize {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, guarded);
        Ok(Self {
            output,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}
