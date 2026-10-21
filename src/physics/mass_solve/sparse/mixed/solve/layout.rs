//! 混合求解布局与打包元数据。

use super::*;

pub(super) fn layout(worlds: usize, nv: usize, nr: usize) -> Result<BatchLayout, InputError> {
    let invalid = || InputError::InvalidDimension {
        field: "mixed_solve_layout",
    };
    if nv == 0
        || nr == 0
        || nr > i32::MAX as usize
        || worlds == 0
        || worlds.checked_mul(nr).is_none_or(|v| v > i32::MAX as usize)
    {
        return Err(invalid());
    }
    let layout = BatchLayout::new(worlds, nv.checked_mul(nr).ok_or_else(invalid)?, 4)?;
    BatchLayout::new(
        1,
        layout.total_elements().checked_add(8).ok_or_else(invalid)?,
        4,
    )?;
    Ok(layout)
}
#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn metadata(input: &SparseMassInput) -> Result<Vec<i32>, TransferError> {
    let n = input.nv();
    let offsets = input.qld_level_offsets();
    let small: Vec<_> = input
        .blocks()
        .iter()
        .filter(|b| b.route == SparseMassRoute::SmallCholesky)
        .collect();
    let length = 4usize
        .checked_add(2 * n)
        .and_then(|v| v.checked_add(offsets.len()))
        .and_then(|v| v.checked_add(3 * input.qld_all_updates().len()))
        .and_then(|v| v.checked_add(3 * small.len()))
        .filter(|&v| v <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "mixed_solve_metadata",
        })?;
    let mut packed = crate::runtime::host_staging::<i32>(length)?;
    packed.clear();
    packed.extend([
        (offsets.len() - 1) as i32,
        input.qld_block_total() as i32,
        input.nc() as i32,
        small.len() as i32,
    ]);
    packed.extend(input.qld_block_adr());
    let f = input.fields();
    let mut diagonal = vec![0i32; n];
    for block in input.blocks() {
        for i in 0..block.size {
            let dof = block.start + i;
            diagonal[dof] = if block.route == SparseMassRoute::SmallCholesky {
                input.qld_block_adr()[dof] + (i * block.size + i) as i32
            } else {
                (input.qld_block_total() + f.m_rowadr[dof] as usize + f.m_rownnz[dof] as usize - 1)
                    as i32
            };
        }
    }
    packed.extend(diagonal);
    packed.extend(offsets);
    packed.extend(input.qld_all_updates().iter().flatten());
    for block in small {
        packed.extend([
            block.start as i32,
            block.size as i32,
            input.qld_block_adr()[block.start],
        ]);
    }
    Ok(packed)
}
