//! Warp 发射维度按值布局。

use super::LauncherError;

macro_rules! launch_bounds {
    ($(#[$doc:meta])* $name:ident, $dims:literal, $($padding:ident)?) => {
        $(#[$doc])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            pub(super) shape: [i32; $dims],
            $(pub(super) $padding: u32,)?
            pub(super) size: u64,
            pub(super) coord_mult: u64,
        }

        impl $name {
            /// 各维须非零且落入i32；总数须落入u32发射计数。
            pub fn new(shape: [usize; $dims]) -> Result<Self, LauncherError> {
                let mut dims = [0i32; $dims];
                let mut size: u64 = 1;
                for (axis, &extent) in shape.iter().enumerate() {
                    if extent == 0 {
                        return Err(LauncherError::InvalidArray("发射维度为零"));
                    }
                    dims[axis] = i32::try_from(extent)
                        .map_err(|_| LauncherError::Overflow("launch_bounds_shape"))?;
                    size = size
                        .checked_mul(extent as u64)
                        .ok_or(LauncherError::Overflow("launch_bounds_size"))?;
                }
                if size > u64::from(u32::MAX) {
                    return Err(LauncherError::Overflow("launch_bounds_size"));
                }
                Ok(Self {
                    shape: dims,
                    $($padding: 0,)?
                    size,
                    // 沿用L2已核验取值。
                    coord_mult: 1,
                })
            }

            pub fn size(&self) -> u64 {
                self.size
            }

            pub fn shape(&self) -> [i32; $dims] {
                self.shape
            }
        }
    };
}

launch_bounds!(
    /// `launch_bounds_t<1>`，显式填充至8字节对齐。
    LaunchBounds1, 1, padding
);
launch_bounds!(
    /// `launch_bounds_t<2>`，与L2`WarpBounds2`一致。
    LaunchBounds2, 2,
);
launch_bounds!(
    /// `launch_bounds_t<3>`，与L2`WarpBounds3`一致。
    LaunchBounds3, 3, padding
);
