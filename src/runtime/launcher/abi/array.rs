//! Warp 数组按值布局。

use super::*;

#[repr(C)]
pub struct WarpArray<T> {
    pub(super) data: u64,
    pub(super) grad: u64,
    pub(super) shape: [i32; 4],
    pub(super) strides: [i32; 4],
    pub(super) ndim: u16,
    pub(super) flags: u16,
    pub(super) padding: u32,
    marker: PhantomData<T>,
}

impl<T> Clone for WarpArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WarpArray<T> {}

impl<T> WarpArray<T> {
    pub fn data(&self) -> u64 {
        self.data
    }

    pub fn ndim(&self) -> usize {
        self.ndim as usize
    }

    pub fn shape(&self) -> &[i32] {
        &self.shape[..self.ndim as usize]
    }

    pub fn strides(&self) -> &[i32] {
        &self.strides[..self.ndim as usize]
    }
}

impl<T> std::fmt::Debug for WarpArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WarpArray")
            .field("data", &self.data)
            .field("shape", &self.shape())
            .field("strides", &self.strides())
            .finish()
    }
}

impl<T: WarpElement> WarpArray<T> {
    /// 行主序连续数组；无梯度缓冲。
    /// 形状与步长须落入i32，与Warp头文件一致。
    pub fn contiguous(data: u64, shape: &[usize]) -> Result<Self, LauncherError> {
        if data == 0 {
            return Err(LauncherError::InvalidArray("数据指针为空"));
        }
        if !(1..=4).contains(&shape.len()) || shape.contains(&0) {
            return Err(LauncherError::InvalidArray("形状须为1至4维且各维非零"));
        }
        let mut array = Self {
            data,
            grad: 0,
            shape: [0; 4],
            strides: [0; 4],
            ndim: shape.len() as u16,
            flags: 0,
            padding: 0,
            marker: PhantomData,
        };
        let mut stride = T::ELEM.bytes();
        for axis in (0..shape.len()).rev() {
            array.shape[axis] = i32::try_from(shape[axis])
                .map_err(|_| LauncherError::Overflow("warp_array_shape"))?;
            array.strides[axis] =
                i32::try_from(stride).map_err(|_| LauncherError::Overflow("warp_array_stride"))?;
            stride = stride
                .checked_mul(shape[axis] as u64)
                .ok_or(LauncherError::Overflow("warp_array_bytes"))?;
        }
        Ok(array)
    }

    /// 连续数组覆盖的字节数。
    pub fn byte_len(&self) -> u64 {
        self.shape()
            .iter()
            .fold(T::ELEM.bytes(), |total, &axis| total * axis as u64)
    }

    /// 元素宽度显式给定的行主序连续数组。
    /// 供L2的向量与矩阵数组复用同一描述符定义。
    pub fn with_element_bytes(
        data: u64,
        shape: &[usize],
        element_bytes: u64,
    ) -> Result<Self, LauncherError> {
        if data == 0 {
            return Err(LauncherError::InvalidArray("数据指针为空"));
        }
        if element_bytes == 0 {
            return Err(LauncherError::InvalidArray("元素宽度为零"));
        }
        if !(1..=4).contains(&shape.len()) || shape.contains(&0) {
            return Err(LauncherError::InvalidArray("形状须为1至4维且各维非零"));
        }
        let mut array = Self {
            data,
            grad: 0,
            shape: [0; 4],
            strides: [0; 4],
            ndim: shape.len() as u16,
            flags: 0,
            padding: 0,
            marker: PhantomData,
        };
        let mut stride = element_bytes;
        for axis in (0..shape.len()).rev() {
            array.shape[axis] = i32::try_from(shape[axis])
                .map_err(|_| LauncherError::Overflow("warp_array_shape"))?;
            array.strides[axis] =
                i32::try_from(stride).map_err(|_| LauncherError::Overflow("warp_array_stride"))?;
            stride = stride
                .checked_mul(shape[axis] as u64)
                .ok_or(LauncherError::Overflow("warp_array_bytes"))?;
        }
        Ok(array)
    }

    /// 供发射前逐项核对的最低限度只读视图。
    #[cfg(feature = "cuda-probe")]
    pub(crate) fn check_parts(&self) -> (u64, usize, bool, &[i32], u64) {
        let ndim = self.ndim as usize;
        let shape = self.shape();
        let shape_ok = (1..=4).contains(&ndim) && shape.iter().all(|&axis| axis > 0);
        (self.data, ndim, shape_ok, shape, self.byte_len())
    }
}
