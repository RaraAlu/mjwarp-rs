//! 空间速度布局校验。

use crate::diagnostics::InputError;
use crate::model::BatchLayout;

use super::ComVelocityLayout;

impl ComVelocityLayout {
    pub(super) fn new(
        worlds: usize,
        nbody: usize,
        njnt: usize,
        nv: usize,
    ) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "com_velocity_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "com_velocity_layout",
        };
        let width = nbody
            .checked_add(nv)
            .and_then(|n| n.checked_mul(6))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let metadata = nbody
            .checked_mul(3)
            .and_then(|n| njnt.checked_mul(2).and_then(|j| n.checked_add(j)))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, metadata, 4)?;
        let output = BatchLayout::new(worlds, width, 4)?;
        let qvel = BatchLayout::new(worlds, nv, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        let guarded_qvel = qvel.total_elements().checked_add(8).ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        BatchLayout::new(1, guarded_qvel, 4)?;
        let rigid_width = nbody
            .checked_mul(28)
            .and_then(|n| njnt.checked_mul(6).and_then(|j| n.checked_add(j)))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let com_width = nbody
            .checked_mul(14)
            .and_then(|n| nv.checked_mul(6).and_then(|v| n.checked_add(v)))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let guarded_rigid = BatchLayout::new(worlds, rigid_width, 4)?
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        let guarded_com = BatchLayout::new(worlds, com_width, 4)?
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded_rigid, 4)?;
        BatchLayout::new(1, guarded_com, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (guarded, guarded_qvel, guarded_rigid, guarded_com);
        Ok(Self {
            output,
            qvel,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            guarded,
            #[cfg(feature = "cuda-probe")]
            guarded_qvel,
            #[cfg(feature = "cuda-probe")]
            guarded_rigid,
            #[cfg(feature = "cuda-probe")]
            guarded_com,
        })
    }
}
