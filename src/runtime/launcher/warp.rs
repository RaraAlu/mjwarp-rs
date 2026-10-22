//! L2 Warp ABI 发射检查与打包。
//! 冻结PTX入口只接受边界加数组描述。
//! 本模块只做宿主校验与打包；内核语义由L2既有证据负责。

use cudarc::driver::{CudaStream, LaunchConfig, PushKernelArg, sys};

use super::{KernelHandle, LaunchBounds2, LaunchBounds3, LauncherError, WarpArray, WarpElement};

/// L2冻结内核的固定块大小。
/// 三处L2发射沿用256线程规则。
pub const L2_BLOCK: u32 = 256;

/// 一个数组描述、其元素宽度与实际分配容量。
#[derive(Clone, Copy)]
pub struct WarpArg {
    array: WarpArrayValue,
    element_bytes: u64,
    capacity_bytes: u64,
}

#[derive(Clone, Copy)]
enum WarpArrayValue {
    F32(WarpArray<f32>),
    F64(WarpArray<f64>),
    I32(WarpArray<i32>),
    U32(WarpArray<u32>),
    U8(WarpArray<u8>),
}

impl WarpArg {
    pub fn f32(array: WarpArray<f32>, element_bytes: u64, capacity_bytes: u64) -> Self {
        Self {
            array: WarpArrayValue::F32(array),
            element_bytes,
            capacity_bytes,
        }
    }

    pub fn f64(array: WarpArray<f64>, element_bytes: u64, capacity_bytes: u64) -> Self {
        Self {
            array: WarpArrayValue::F64(array),
            element_bytes,
            capacity_bytes,
        }
    }

    pub fn i32(array: WarpArray<i32>, element_bytes: u64, capacity_bytes: u64) -> Self {
        Self {
            array: WarpArrayValue::I32(array),
            element_bytes,
            capacity_bytes,
        }
    }

    pub fn u32(array: WarpArray<u32>, element_bytes: u64, capacity_bytes: u64) -> Self {
        Self {
            array: WarpArrayValue::U32(array),
            element_bytes,
            capacity_bytes,
        }
    }

    pub fn u8(array: WarpArray<u8>, element_bytes: u64, capacity_bytes: u64) -> Self {
        Self {
            array: WarpArrayValue::U8(array),
            element_bytes,
            capacity_bytes,
        }
    }

    fn elem_bytes(&self) -> u64 {
        match self.array {
            WarpArrayValue::F32(_) => <f32 as WarpElement>::ELEM.bytes(),
            WarpArrayValue::F64(_) => <f64 as WarpElement>::ELEM.bytes(),
            WarpArrayValue::I32(_) => <i32 as WarpElement>::ELEM.bytes(),
            WarpArrayValue::U32(_) => <u32 as WarpElement>::ELEM.bytes(),
            WarpArrayValue::U8(_) => <u8 as WarpElement>::ELEM.bytes(),
        }
    }

    /// 逐项核对数组：指针、维度、形状、对齐、元素宽度与容量。
    pub(crate) fn check(&self, slot: usize) -> Result<(), LauncherError> {
        let elem = self.elem_bytes();
        let (data, ndim, shape_ok, shape, _) = match &self.array {
            WarpArrayValue::F32(array) => array.check_parts(),
            WarpArrayValue::F64(array) => array.check_parts(),
            WarpArrayValue::I32(array) => array.check_parts(),
            WarpArrayValue::U32(array) => array.check_parts(),
            WarpArrayValue::U8(array) => array.check_parts(),
        };
        if data == 0 {
            return Err(LauncherError::InvalidArray("数组数据指针为空"));
        }
        if !(1..=4).contains(&ndim) {
            return Err(LauncherError::InvalidArray("数组维度须为1至4"));
        }
        if !shape_ok {
            return Err(LauncherError::InvalidArray("数组形状非法"));
        }
        if data % elem != 0 {
            return Err(LauncherError::InvalidArray("数组指针未按元素对齐"));
        }
        if self.element_bytes < elem || !self.element_bytes.is_multiple_of(elem) {
            return Err(LauncherError::InvalidArray("数组元素宽度非法"));
        }
        let mut bytes = self.element_bytes;
        for &axis in shape {
            bytes = bytes
                .checked_mul(u64::try_from(axis).unwrap_or(u64::MAX))
                .ok_or(LauncherError::Overflow("warp_array_bytes"))?;
        }
        if bytes == 0 || bytes > self.capacity_bytes {
            return Err(LauncherError::Capacity {
                slot,
                required: bytes,
                actual: self.capacity_bytes,
            });
        }
        Ok(())
    }
}

/// L2发射边界：1至3维，与冻结上游一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarpBounds {
    Two(LaunchBounds2),
    Three(LaunchBounds3),
}

impl WarpBounds {
    pub fn size(&self) -> u64 {
        match self {
            Self::Two(bounds) => bounds.size(),
            Self::Three(bounds) => bounds.size(),
        }
    }
}

/// 一次L2发射的全部实参。
pub struct WarpLaunch<'a> {
    pub bounds: WarpBounds,
    pub args: &'a [WarpArg],
}

impl KernelHandle {
    /// 安全发射：先核对上下文、捕获、边界、数组、线程规则，再打包。
    /// 冻结L2只做同步诊断，不进入产品调度。
    pub fn launch_l2(
        &self,
        stream: &CudaStream,
        launch: &WarpLaunch<'_>,
    ) -> Result<(), LauncherError> {
        self.enqueue_l2(stream, launch)
    }

    /// M5异步发射：校验与打包同`launch_l2`，只在流上入队。
    /// 返回的完成凭证持有句柄（模块）与流，并可核验完成。
    /// 缓冲所有权仍由调用方持有；访问顺序由提交账本覆盖。
    pub fn submit_l2(
        &self,
        stream: &std::sync::Arc<CudaStream>,
        launch: &WarpLaunch<'_>,
    ) -> Result<L2Completion, LauncherError> {
        self.enqueue_l2(stream.as_ref(), launch)?;
        let event = stream
            .context()
            .new_event(None)
            .map_err(|error| LauncherError::Driver(format!("event: {error:?}")))?;
        event
            .record(stream.as_ref())
            .map_err(|error| LauncherError::Driver(format!("event-record: {error:?}")))?;
        Ok(L2Completion {
            handle: self.clone(),
            stream: std::sync::Arc::clone(stream),
            event,
        })
    }

    fn enqueue_l2(
        &self,
        stream: &CudaStream,
        launch: &WarpLaunch<'_>,
    ) -> Result<(), LauncherError> {
        if !std::sync::Arc::ptr_eq(self.context(), stream.context()) {
            return Err(LauncherError::ModuleContext);
        }
        match stream.capture_status() {
            Ok(sys::CUstreamCaptureStatus::CU_STREAM_CAPTURE_STATUS_NONE) => {}
            Ok(_) => return Err(LauncherError::Unsupported("捕获期间禁止L2发射")),
            Err(error) => return Err(LauncherError::Driver(format!("capture: {error:?}"))),
        }
        let size = launch.bounds.size();
        if size == 0 || size > u64::from(u32::MAX) {
            return Err(LauncherError::InvalidArray("发射边界为零或超出u32"));
        }
        if launch.args.is_empty() {
            return Err(LauncherError::InvalidSignature {
                abi: "warp_l2",
                reason: "至少一个数组实参",
            });
        }
        for (slot, arg) in launch.args.iter().enumerate() {
            arg.check(slot)?;
        }
        let grid = u32::try_from(size.div_ceil(u64::from(L2_BLOCK)))
            .map_err(|_| LauncherError::Overflow("warp_l2_grid"))?;
        if grid == 0 {
            return Err(LauncherError::InvalidArray("发射网格为空"));
        }
        let mut builder = stream.launch_builder(self.function());
        match &launch.bounds {
            WarpBounds::Two(bounds) => {
                builder.arg(bounds);
            }
            WarpBounds::Three(bounds) => {
                builder.arg(bounds);
            }
        }
        for arg in launch.args {
            match &arg.array {
                WarpArrayValue::F32(array) => {
                    builder.arg(array);
                }
                WarpArrayValue::F64(array) => {
                    builder.arg(array);
                }
                WarpArrayValue::I32(array) => {
                    builder.arg(array);
                }
                WarpArrayValue::U32(array) => {
                    builder.arg(array);
                }
                WarpArrayValue::U8(array) => {
                    builder.arg(array);
                }
            }
        }
        // SAFETY: 上下文、边界、数组与线程规则已核对。
        // 每线程访问范围由L2既有证据与输入校验限定。
        unsafe {
            builder
                .launch(LaunchConfig {
                    grid_dim: (grid, 1, 1),
                    block_dim: (L2_BLOCK, 1, 1),
                    shared_mem_bytes: 0,
                })
                .map_err(|error| LauncherError::Driver(format!("launch: {error:?}")))?;
        }
        Ok(())
    }
}

/// M5：一次异步L2发射的完成凭证。
/// 持有入口句柄（模块）、流与事件。
/// 查询与等待只观察完成，不释放调用方缓冲。
pub struct L2Completion {
    handle: KernelHandle,
    stream: std::sync::Arc<CudaStream>,
    event: cudarc::driver::CudaEvent,
}

impl L2Completion {
    pub fn handle(&self) -> &KernelHandle {
        &self.handle
    }

    pub fn stream(&self) -> &std::sync::Arc<CudaStream> {
        &self.stream
    }

    /// 非阻塞完成查询。
    pub fn is_complete(&self) -> bool {
        self.event.is_complete()
    }
}

impl super::Fence for L2Completion {
    fn query(&self) -> Result<bool, LauncherError> {
        Ok(self.event.is_complete())
    }

    fn wait(&self) -> Result<(), LauncherError> {
        self.event
            .synchronize()
            .map_err(|error| LauncherError::Driver(format!("event-wait: {error:?}")))
    }
}

/// 模块由句柄与缓存共同持有；隔离不释放模块。
impl super::RetainedResources for KernelHandle {
    fn quarantine(&self) {}
}

/// 构造单精度L2数组实参。
pub fn f32_arg(
    data: u64,
    shape: &[usize],
    element_bytes: u64,
    capacity_bytes: u64,
) -> Result<WarpArg, LauncherError> {
    let array = WarpArray::<f32>::with_element_bytes(data, shape, element_bytes)?;
    Ok(WarpArg::f32(array, element_bytes, capacity_bytes))
}

/// 构造整数L2数组实参。
pub fn i32_arg(data: u64, shape: &[usize], capacity_bytes: u64) -> Result<WarpArg, LauncherError> {
    let array = WarpArray::<i32>::with_element_bytes(data, shape, 4)?;
    Ok(WarpArg::i32(array, 4, capacity_bytes))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_args_reject_capacity_and_width_mismatch() {
        let array = WarpArray::<f32>::with_element_bytes(0x1000, &[2], 12).unwrap();
        assert!(WarpArg::f32(array, 12, 24).check(0).is_ok());
        assert!(matches!(
            WarpArg::f32(array, 12, 20).check(0),
            Err(LauncherError::Capacity { slot: 0, .. })
        ));
        assert!(matches!(
            WarpArg::f32(array, 0, 24).check(1),
            Err(LauncherError::InvalidArray(_))
        ));
        let misaligned = WarpArray::<f32>::with_element_bytes(0x1002, &[1], 4).unwrap();
        assert!(matches!(
            WarpArg::f32(misaligned, 4, 4).check(2),
            Err(LauncherError::InvalidArray(_))
        ));
    }
}
