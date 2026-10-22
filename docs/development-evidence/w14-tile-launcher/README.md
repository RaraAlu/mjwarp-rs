# W14 tile 发射器落地记录

任务：`2026-10-23-w14-tile-launcher`。
依据：`docs/development-evidence/layers-w14.md`（tile 约束冻结）、`docs/development-evidence/layers-w15.md`（头文件原样复制）。
内核来源：**W15 头文件原样复制路线**（NVRTC + `include/warp-headers`），不引 nvJitLink/MathDx。
本任务不改产品内核、不改容差与断言。

## 依赖拆分（Chief 2026-10-23 指令）

- `depends_on` 由「m7-tile + W13 + W15」改为「m7-tile + W15」。
- 原因：W13/nvJitLink 只是 LTO 内核来源路线，且需用户对许可/部署裁决；tile 发射器实现（静态 `__shared__` + 发射校验）
  不依赖它。files 收窄到 `src/runtime/launcher`、`tests/launcher-tile.rs`、证据与 docs，不再含 `src/physics`。

## 约束实现点

| 冻结约束 | 实现 |
| --- | --- |
| 共享内存由内核源码静态声明 | `KernelAbi::SHARED_BYTES: u32 = 0`（静态 `__shared__` 字节数，按 4 字节对齐校验）；`AbiSignature` 记录并提供 `shared_bytes()` |
| 发射参数 `shared_mem_bytes` 为 0 | `LaunchDims::new` 对非零值返回 `Unsupported("动态共享内存未opt-in")`；产品路径仍全部传 0 |
| 动态共享内存本轮不启用 | 非零值必须 `LaunchDims::with_limits(..., dynamic_opt_in=true)`；未 opt-in 直接拒绝 |
| 共享内存上限按设备属性核对 | `SharedMemoryLimits { per_block, opt_in }`（注入式，默认 48KiB/48KiB，不沿用探针 128KiB）+ `DeviceSharedMemoryLimits::query` 经 cudarc 查询 `MAX_SHARED_MEMORY_PER_BLOCK`/`OPTIN`，`SharedMemoryLimits::intersect` 与注入值取交集、`for_device` 查询失败回退保守默认；超限返回 `InvalidArray("共享内存超过设备上限")`；`Kernel::plan_with_limits` 核对静态声明上限 |
| 静态与动态不可同时声明 | `plan_with_limits` 同时检查 `signature.shared_bytes()` 与 `launch_dims.shared_bytes()` |
| 块大小等于编译线程数，发射器不改写 | 新增 `BlockRule::Tile(u32)`；`AbiSignature` 校验 32 的倍数且 ≤1024；`LaunchDims::validate_rule` 要求 block 严格等于规则线程数 |
| 每 block 一个 tile 单元 | 由 `KernelSpec::launch_dims` 的 grid 语义决定，测试用 grid=世界数、block=(32,1,1) 覆盖 |
| 静态 `__shared__` 内核走 W15 头文件路线 | `NvrtcSource::with_headers_from_dir(include/warp-headers)` + `--include-path` 编译，最小 tile 内核见测试 |
| tile 数据 f32 | 测试内核共享缓冲 `float[49]`/`int[1]`；未引入 f64 路径 |

## 验证

| 项 | 命令/测试 | 结果 |
| --- | --- | --- |
| 块大小约束 | `cargo test --locked --features cuda-probe --test launcher-tile` 的 `tile_block_rule_must_equal_the_compiled_thread_count` | 通过：Tile(32) 计划成功；Tile(64) 内核配 block 32 被拒；Tile(0/48/2048) 声明被拒 |
| 共享内存请求拒绝面 | 同文件 `launch_dims_reject_dynamic_shared_memory_without_opt_in_and_over_limits` | 通过：非零未 opt-in 拒绝、64KiB 超默认上限拒绝、opt-in 内 64KiB 允许、超 opt-in 上限拒绝、非法 limits 拒绝 |
| 静态共享内存上限 | 同文件 `static_shared_memory_is_checked_against_device_limits` | 通过：静态 200B 在默认上限允许、64B 上限拒绝、SHARED_BYTES 非 4 倍数声明被拒 |
| 静态 `__shared__` 编译 | 同文件 `nvrtc_compiles_static_shared_tile_kernel_with_verbatim_warp_headers`（ignored；前置：`target/toolchains/cuda-12.8.1/bin` 加入 `PATH` + `MJWARP_CUDA_INCLUDE`） | 通过：W15 头文件 + 最小 tile 内核（`__shared__ float[49]`、`__syncthreads()`×3）编译出 PTX，含入口名与 `.shared` 声明；`cuda_fp16.h` 外部头摘要进产物键断言通过 |
| 既有逻辑回归 | `cargo test --locked --features cuda-probe --lib` | 447 过 0 败（127 ignored） |
| 门禁 | `normify gate 2026-10-23-w14-tile-launcher` | fmt/clippy/clippy-cuda/unit 见 `.normify/runs/` |

## 2026-10-23 复核修复（W14-F1）\n\nQA 独立复核指出「共享内存上限只有注入+保守默认、未接设备查询」。已补：\n`DeviceSharedMemoryLimits::query(device)` 经 cudarc driver 绑定调用 `cuDeviceGetAttribute` 读取\n`MAX_SHARED_MEMORY_PER_BLOCK` 与 `MAX_SHARED_MEMORY_PER_BLOCK_OPTIN`；`SharedMemoryLimits::intersect`\n与注入上限取交集（注入更大时收紧），`SharedMemoryLimits::for_device` 查询不可用时保留保守默认。\n测试：模拟设备值交集/收紧/回退（宿主）+ 真实设备查询（ignored GPU）均通过；W14 tile NVRTC 测试\n补 `cuda_fp16.h` 摘要进键断言；NVRTC 复现命令补 `target/toolchains/cuda-12.8.1/bin` 到 `PATH` 的前置说明。\n\n## 未验收边界

- 产品内核未切换：`src/physics` 未改动，`unchanged` 检查保护；tile 能力只落在发射器/ABI 层。
- 未启动 GPU 内核，未做 tile 装卸/清零的 GPU 执行验证（NVRTC 只生成 PTX）；GPU 内核级测试需后续任务与窗口。
- 动态共享内存未启用（需独立评审与 opt-in 策略）。
- 未引 nvJitLink/MathDx；未做 MathDx SDK 内部精度验收。
- 未做逐字段精度映射，上游源码产物仍不得进入物理配方；未跑 Linux/性能。
