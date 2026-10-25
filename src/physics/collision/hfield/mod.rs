//! G4 hfield 宿主窄相（对齐 pinned `mujoco_warp/_src/collision_convex.py` 的
//! `_hfield_filter` 与 `ccd_hfield_kernel` 子集）。
//! 只读宿主实现：不实装 GPU 内核、不分配设备缓冲。
//! 数据来自 G1 的 `hfield_size/nrow/ncol/adr/data` 字段。

mod driver;
mod filter;
#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use driver::{HfieldContact, HfieldInput, HfieldOutput, MJ_MAXCONPAIR, hfield_pair_contacts};
#[allow(unused_imports)]
pub use filter::HfieldBounds;

/// hfield 模型数据（G1 字段上传口径，f32）。
#[derive(Clone, Debug, PartialEq)]
pub struct HfieldData {
    /// `(x, y, z_top, z_bottom)`。
    pub size: [f32; 4],
    pub nrow: i32,
    pub ncol: i32,
    /// `hfield_data` 起始下标。
    pub adr: i32,
    /// 高程数据（`nhfielddata` 个 f32）。
    pub data: Vec<f32>,
}

impl HfieldData {
    /// 构造并校验行列数、数据容量与地址范围。
    pub fn new(size: [f32; 4], nrow: i32, ncol: i32, adr: i32, data: Vec<f32>) -> Option<Self> {
        if nrow < 2 || ncol < 2 || adr < 0 {
            return None;
        }
        let cells = usize::try_from(nrow).ok()? * usize::try_from(ncol).ok()?;
        let end = usize::try_from(adr).ok()?.checked_add(cells)?;
        (end <= data.len() && size[0] > 0.0 && size[1] > 0.0).then_some(Self {
            size,
            nrow,
            ncol,
            adr,
            data,
        })
    }

    /// 第 `row` 行 `col` 列的高程（不做越界检查，调用方保证在子网格内）。
    pub(crate) fn height(&self, row: i32, col: i32) -> f32 {
        let index = self.adr as usize + (row as usize) * self.ncol as usize + col as usize;
        self.data[index]
    }
}
