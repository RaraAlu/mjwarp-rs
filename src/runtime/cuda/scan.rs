//! 扫描层级缓冲与多级发射。

use std::sync::Arc;

use cudarc::driver::{CudaSlice, CudaStream, DeviceRepr};

use crate::diagnostics::ProbeError;
use crate::runtime::GUARD_ELEMENTS;
use crate::runtime::samples::{Samples, ScanSamples};

use super::GpuProgram;
use super::cuda_error;
use super::typed::{ValidateValues, download, launch};

pub(super) struct ScanLevel<T> {
    pub(super) sums: CudaSlice<T>,
    pub(super) prefix: CudaSlice<T>,
    totals_meta: CudaSlice<u32>,
    scan_meta: CudaSlice<u32>,
    offsets_meta: CudaSlice<u32>,
    elements: usize,
    preceding_elements: usize,
}

pub(super) fn allocate_scan<T: DeviceRepr + Copy>(
    transfer: &Arc<CudaStream>,
    samples: &Samples<T>,
) -> Result<Vec<ScanLevel<T>>, ProbeError> {
    let mut levels = Vec::new();
    let mut preceding = samples.input.len();
    for sample in &samples.scan_levels {
        let elements = sample.sums.len() - GUARD_ELEMENTS;
        let initial = vec![*sample.sums.last().unwrap(); sample.sums.len()];
        levels.push(ScanLevel {
            sums: transfer
                .clone_htod(&initial)
                .map_err(cuda_error("scan-allocate-sums"))?,
            prefix: transfer
                .clone_htod(&initial)
                .map_err(cuda_error("scan-allocate-prefix"))?,
            totals_meta: transfer
                .clone_htod(&[preceding as u32, elements as u32])
                .map_err(cuda_error("scan-totals-meta"))?,
            scan_meta: transfer
                .clone_htod(&[elements as u32, elements as u32])
                .map_err(cuda_error("scan-prefix-meta"))?,
            offsets_meta: transfer
                .clone_htod(&[elements as u32, preceding as u32])
                .map_err(cuda_error("scan-offsets-meta"))?,
            elements,
            preceding_elements: preceding,
        });
        preceding = elements;
    }
    Ok(levels)
}

pub(super) fn launch_program<T: DeviceRepr>(
    stream: &CudaStream,
    program: &GpuProgram,
    input: &CudaSlice<T>,
    output: &mut CudaSlice<T>,
    elements: usize,
    metadata: Option<&CudaSlice<u32>>,
    levels: &mut [ScanLevel<T>],
) -> Result<(), ProbeError> {
    launch(
        stream,
        &program.main.function,
        input,
        output,
        elements,
        metadata,
        program.main.shared_memory,
    )?;
    let Some(scan) = &program.scan else {
        return Ok(());
    };
    for depth in 0..levels.len() {
        let (before, remaining) = levels.split_at_mut(depth);
        let level = &mut remaining[0];
        let preceding = before.last().map_or(&*output, |previous| &previous.prefix);
        launch(
            stream,
            &scan.totals.function,
            preceding,
            &mut level.sums,
            level.elements,
            Some(&level.totals_meta),
            scan.totals.shared_memory,
        )?;
        launch(
            stream,
            &program.main.function,
            &level.sums,
            &mut level.prefix,
            level.elements,
            Some(&level.scan_meta),
            program.main.shared_memory,
        )?;
    }
    // 从最小层向下传播块间偏移。
    for depth in (0..levels.len()).rev() {
        let (before, remaining) = levels.split_at_mut(depth);
        let level = &remaining[0];
        let preceding = before
            .last_mut()
            .map_or(&mut *output, |previous| &mut previous.prefix);
        launch(
            stream,
            &scan.offsets.function,
            &level.prefix,
            preceding,
            level.preceding_elements,
            Some(&level.offsets_meta),
            scan.offsets.shared_memory,
        )?;
    }
    Ok(())
}

pub(super) fn check_scan<T: DeviceRepr>(
    transfer: &Arc<CudaStream>,
    execute: &CudaStream,
    levels: &[ScanLevel<T>],
    samples: &[ScanSamples<T>],
    validate: ValidateValues<T>,
) -> Result<(), ProbeError> {
    for (level, sample) in levels.iter().zip(samples) {
        validate(&sample.sums, &download(transfer, execute, &level.sums)?)?;
        validate(&sample.prefix, &download(transfer, execute, &level.prefix)?)?;
    }
    Ok(())
}
