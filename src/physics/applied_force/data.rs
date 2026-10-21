//! 数据缓冲、回读与检查。

use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn read(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut v)?;
    crate::physics::check_device_values(&v, field)?;
    Ok(v)
}
#[cfg(feature = "cuda-probe")]
pub(super) fn guards(buffer: &TransferBuffer<f32>) -> Result<(), TransferError> {
    let mut v = [0.0; 8];
    buffer.read_range_into(0, &mut v[..4])?;
    buffer.read_range_into(buffer.len() - 4, &mut v[4..])?;
    crate::physics::check_device_guards(&v)
}
pub(super) fn check(
    values: &[f32],
    expected: usize,
    field: &'static str,
) -> Result<(), InputError> {
    if values.len() != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual: values.len(),
        });
    }
    if let Some(index) = values.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite { field, index });
    }
    Ok(())
}
impl AppliedForceData {
    pub fn worlds(&self) -> usize {
        self.layout.vector.worlds()
    }
    pub fn write_geometry(
        &mut self,
        field: AppliedForceGeometry,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let i = field.index();
        self.geom(field, 0, self.layout.geometry[i].total_elements(), values)
    }
    pub fn write_world_geometry(
        &mut self,
        field: AppliedForceGeometry,
        world: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        let r = self.layout.geometry[field.index()].world_elements(world)?;
        self.geom(field, r.start, r.len(), values)
    }
    pub(super) fn geom(
        &mut self,
        field: AppliedForceGeometry,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check(values, expected, field.name())?;
        let a = 4 + self.layout.offsets[field.index()] + offset;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = a;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.geometry.write_range(a, values)
        }
    }
    pub fn write_ft(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(false, 0, self.layout.ft.total_elements(), values)
    }
    pub fn write_world_ft(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.ft.world_elements(world)?;
        self.write(false, r.start, r.len(), values)
    }
    pub fn write_qfrc(&mut self, values: &[f32]) -> Result<(), TransferError> {
        self.write(true, 0, self.layout.vector.total_elements(), values)
    }
    pub fn write_world_qfrc(&mut self, world: usize, values: &[f32]) -> Result<(), TransferError> {
        let r = self.layout.vector.world_elements(world)?;
        self.write(true, r.start, r.len(), values)
    }
    pub(super) fn write(
        &mut self,
        qfrc: bool,
        offset: usize,
        expected: usize,
        values: &[f32],
    ) -> Result<(), TransferError> {
        check(values, expected, if qfrc { "qfrc" } else { "xfrc_applied" })?;
        self.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if qfrc {
                self.current.write_range(4 + offset, values)
            } else {
                self.ft.write_range(4 + offset, values)
            }
        }
    }
    pub fn readback(&self) -> Result<AppliedForceOutput, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "xfrc_accumulate",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(AppliedForceOutput {
                layout: self.layout.vector,
                values: read(&self.current, self.layout.vector.total_elements(), "qfrc")?,
            })
        }
    }
}
