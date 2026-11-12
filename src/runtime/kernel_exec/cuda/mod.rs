//! CUDA 后端适配层（P1-b）。
//!
//! 只实现 P1-a 的宿主面 trait（`Backend`/`Device`/`Allocator`/`Buffer`/`Queue`/`Module`/
//! `Diagnostics`）；设备内原语（`DeviceMath`/`Atomics`/`ThreadScope`/`ExecControl`/
//! `QueueCapture`）属书写面（P2），不在本层实现。
//!
//! 边界：**先包不改**——不接任何现有调用点，旧 `runtime::transfer`/`runtime::launcher`/
//! `runtime::cuda` 路径保持原样（双轨并存，迁移属 P2）；不碰 `.cu`/`*_CUDA`/recipe/PTX/ABI。
//! 本层零 GPU 门禁：默认测试不创建 CUDA context；GPU 等价测试一律 `#[ignore]`。

// S2 落地：backends/buffer/queue/module/diagnostics/errors/tests。
