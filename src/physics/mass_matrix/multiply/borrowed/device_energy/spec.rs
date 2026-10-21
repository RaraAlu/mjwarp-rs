use super::*;

pub(super) struct EnergySpec {
    pub(super) mass: Arc<Spec>,
}
#[derive(Clone, Copy)]
pub(super) struct Layout {
    pub(super) pair: BatchLayout,
    pub(super) energy: BatchLayout,
}
pub(super) fn layout(n: usize, worlds: usize) -> Result<Layout, InputError> {
    if !(1..=1024).contains(&n) || worlds == 0 || worlds > i32::MAX as usize {
        return Err(InputError::InvalidDimension {
            field: "device_energy_layout",
        });
    }
    let pair = BatchLayout::new(worlds, 2 * n, 4)?;
    let energy = BatchLayout::new(worlds, 2, 4)?;
    for count in [pair.total_elements(), energy.total_elements(), worlds] {
        count
            .checked_add(8)
            .and_then(|v| v.checked_mul(4))
            .ok_or(InputError::Overflow {
                field: "device_energy_guards",
            })?;
    }
    Ok(Layout { pair, energy })
}
pub(super) fn binding(plan: &Arc<EnergySpec>, data: &Arc<EnergySpec>, mass: &Arc<Spec>) -> bool {
    Arc::ptr_eq(plan, data) && Arc::ptr_eq(&plan.mass, mass)
}
pub(super) fn all_current(flags: &[bool]) -> Result<(), TransferError> {
    if flags.is_empty() || flags.iter().any(|&v| !v) {
        return Err(TransferError::StageNotReady {
            stage: "device_energy_all_current",
        });
    }
    Ok(())
}
