# W14 共享内存设备上限查询修复（W14-F1）

任务：`2026-10-23-w14-shared-memory-device-limit`。
触发：QA 独立复核 `docs/development-evidence/w15-w14-independent-check/README.md` 的 W14-F1：
冻结要求「产品按设备属性核对共享内存上限」，原实现只有注入式 `SharedMemoryLimits` + 保守 48KiB，
全仓无 `cudaDeviceGetAttribute` 查询。

## 修复内容

- `src/runtime/launcher/launch.rs`
  - 新增 `DeviceSharedMemoryLimits`：经 cudarc driver 现有绑定调用 `cuDeviceGetAttribute`，读取
    `CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK` 与 `CU_DEVICE_ATTRIBUTE_MAX_SHARED_MEMORY_PER_BLOCK_OPTIN`；
    驱动不可用、属性缺失或数值非法时返回 `None`。
  - `SharedMemoryLimits::intersect(device)`：与注入上限取交集（注入更大时收紧到设备值，
    `opt_in` 不低于 `per_block`）。
  - `SharedMemoryLimits::for_device(device)`：查询成功时取「保守默认 ∩ 设备值」，查询失败保留保守默认（48KiB）。
  - 既有 `new`/`with_limits`/`plan_with_limits` 语义不变，不改变公开 ABI 行为。
- 测试（`tests/launcher-tile.rs`）
  - 宿主：`device_limits_intersect_with_injected_limits` 覆盖交集、保守默认收紧、opt_in clamp 与非法输入。
  - GPU（ignored）：`queries_real_device_shared_memory_limits` 真实查询设备属性，断言
    `per_block > 0`、`opt_in >= per_block`、`for_device` 等于「保守默认 ∩ 设备值」。
- QA 三条建议
  1. NVRTC 复现命令补 `target/toolchains/cuda-12.8.1/bin` 加入 `PATH` 的说明（W14/W15 README）。
  2. `headers.json.wheel_vs_clone` 从 7 个样本补成 36 个文件全量对比（36/36 全部与 wheel 不同）。
  3. W14 tile NVRTC 测试登记 `cuda_fp16.h` 摘要进产物键：`with_external_header` + 断言有/无该头时
     `ArtifactKey` 摘要不同。
- 未改容差、断言、冻结文档 `docs/development-evidence/layers-w14.md`。

## 验证

| 项 | 命令 | 结果 |
| --- | --- | --- |
| W14 宿主测试 | `cargo test --locked --features cuda-probe --test launcher-tile` | 4 过 0 败 2 ignored |
| 真实设备查询 | 同上 `-- --ignored`（需 NVIDIA 驱动） | 通过 |
| NVRTC tile + SDK 头摘要 | 同上 `-- --ignored`（前置：CUDA bin 入 `PATH`，设 `MJWARP_CUDA_INCLUDE`） | 通过：PTX 含入口名与 `.shared`；`cuda_fp16.h` 摘要进键断言通过 |
| 门禁 | `normify gate 2026-10-23-w14-shared-memory-device-limit` | fmt/clippy/clippy-cuda/unit 见 `.normify/runs/` |

复现前置：

```powershell
$env:PATH = "D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"  # NVRTC DLL
$env:MJWARP_CUDA_INCLUDE = "D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\include"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --test launcher-tile -- --ignored
```

## 边界

- 查询失败（无驱动/无设备）时保留保守默认，不阻断现有路径；产品内核仍全部 `shared_bytes=0`。
- 未启动 GPU 内核；未做 tile 装卸/清零 GPU 执行验证；未做 Linux/性能。
- 未改冻结文档 `layers-w14.md`、容差、断言、排除清单；W13/g03-parity 未动。
- 本任务同时把已关闭 U038 PTX 对齐任务的引擎 runs 记录随任务入库（非手改）。
