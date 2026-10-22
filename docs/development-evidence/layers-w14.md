# W14 共享内存与 tile 发射评审

任务：`2026-10-18-layers-m7-tile`。
日期：2026-10-22。
依据：`docs/architecture.md`、`docs/unified-launcher.md`、`docs/module-contracts.md`。
本轮只评审，不改内核、不加依赖、不改代码。
本轮结论不自动批准实现。

## 结论

批准 W14 的 tile 约束冻结，但**实施延后**并另立交付任务。
tile 共享内存维持内核源码内静态声明，产品发射暂不启用动态共享内存。
线程参与按上游 tile 的 cooperative 语义冻结，发射器不得改块大小。
内核来源沿用 W13 与 W15 的结论，本评审不选路线。
既有 MathDx 证据只作离线参考，不作产品验收。

## 现状基线

| 事实 | 证据 | 结论 |
| --- | --- | --- |
| 新发射器已留共享内存位 | `src/runtime/launcher/launch.rs` 的 `LaunchDims { grid, block, shared_bytes }` | 位置已留，缺上限与 opt-in 校验 |
| `LaunchDims` 只校验网格与线程数 | 同文件 `new`：网格零维拒绝、线程数 ≤1024 | 无共享内存上限检查 |
| 线程块规则只有两种 | `src/runtime/launcher/abi.rs` 的 `BlockRule::{Fixed, WarpGrid}`；`WarpGrid` 固定 256 | 无法表达 tile 维度 |
| 旧同步发射写死参数 | `src/runtime/transfer/kernel/launch.rs` 与 `launchers_physics.rs` 全部 `shared_mem_bytes: 0`、块 256 | 当前产品内核不用动态共享内存 |
| 探针缓存有独立上限 | `src/runtime/cache.rs` 拒绝 `shared_memory > 128KiB`；原生 PTX 路线要求共享内存为 0 | 只服务探针，不适用于产品 tile |
| cubecl 探针含共享内存用例 | `src/runtime/cubecl.rs` 断言内核源码含 `__shared__`；`docs/cubecl-probe.md` 记录 128×1×1 线程块 | 仅工具证据 |
| 产品路径无 tile 内核 | `src/physics` 的 CUDA 字符串不含上游 tile 操作；`__shared__` 只出现在测试内核 | W14 是新增能力 |
| 设计已登记 W14 | `docs/unified-launcher.md`：W14 依赖 W04，原因「上游 tile 内核」；M7 阶段 W12 至 W15 各自另立任务 | 路线一致 |

## 上游 tile 证据

证据目录：`docs/evidence/native-mathdx7-link/`、`scripts/native-mathdx7-call.cu`。
来源：Warp `v1.15.0`（提交 `ffa038b1adf3927a8e893d4536f1a7562f17d749`）的 `tile_cholesky` MathDx 分支，照抄为离线内核。
性质：离线工具证据，非产品内核。

可复核事实：

- `__shared__ __align__(16) float tile[49]` 与 `__shared__ int info[1]`：共享内存由**内核源码静态声明**，不走发射参数。
- 32 线程块：设备函数名 `potrf_7_7_1_89_32_1_1_5_x_x_0` 含 7×7、sm_89、32 线程、批 1×1。
- 参与方式：全部 lane 用 `i = lane; i < 49; i += 32` 协作装载与写回；`__syncthreads()` 在装载后、设备调用后、清零后各一次，配对完整。
- 每 block 处理一个世界；grid 按世界数。
- 共享与公开数据为 f32；MathDx SDK 内部精度未接受。
- 链接与精度边界见 W13 评审。

W15 结论补充：`tile*.h` 属「条件复制」级，只在对应上游内核需要时原样复制；`tile_cholesky.h` 的 SHA-256 已记录。tile 内核若走上游源码路线，依赖 W15 头文件；若走离线链接产物，依赖 W13 绑定。

## tile ABI 冻结

| 项 | 冻结值 |
| --- | --- |
| 共享内存形态 | 内核源码内静态 `__shared__`；发射参数 `shared_mem_bytes` 为 0 |
| 动态共享内存 | 本轮不启用；如后续需要 `extern __shared__`，另立评审 |
| 参数形状 | 沿用 W04 冻结的 Warp ABI（`WarpArray`/`LaunchBounds`）与 L1 私有指针 ABI；tile 数据不跨参数槽传递 |
| 线程块 | `KernelAbi::BLOCK` 必须等于内核编译时的线程数；MathDx potrf 用 `Fixed(32)`，Warp 默认 tile 用 `WarpGrid(256)` |
| 每 block 工作单元 | 一个 tile 单元（矩阵或世界）；grid 按单元数 |
| 共享内存上限 | 产品按设备属性核对（默认每块上限与 opt-in 上限）；不沿用探针的 128KiB 常数 |
| 精度 | tile 缓冲、输入与输出全 f32；不得升 f64；SDK 内部精度未接受 |

补充约束：

- 发射器不得改写块大小；`LaunchDims` 与 `BlockRule` 不一致必须拒绝。
- `shared_bytes` 非零目前只允许在显式启用动态共享内存的任务中出现；本轮产品路径要求 0。
- 静态共享内存大小无法从 Rust 侧 ABI 声明推断，需在实施任务增加 ABI 常量或产物属性核对。
- `__syncthreads()` 的前提属于 CUDA 源码语义，Rust 侧无法静态验证，只能由测试覆盖。

## 线程参与约束

| 约束 | 依据 |
| --- | --- |
| 全部线程参与同一 tile 操作 | 上游 cooperative tile 语义；MathDx 分支全部 lane 参与装卸 |
| 屏障由全部线程到达 | 静态源码中三处 `__syncthreads()` 配对 |
| 块内线程数固定且与编译参数一致 | 设备函数名含 32；Warp 默认 256 |
| 每 block 只处理一个工作单元 | 离线内核按 `blockIdx.x` 取世界 |
| 失败路径不得留半个 tile | 装载、计算、写回都在同一 barrier 之后完成 |
| 线程数与共享内存同属 ABI 声明 | W04 的 `KernelAbi` 已声明线程块；共享内存声明需扩展 |

## 精度与内核来源

精度：

- tile 共享缓冲、输入矩阵与输出上三角都保持 f32；下三角正零由内核写 0。
- 不得为通过容差提升为 f64，也不得降低为 f16/TF32。
- MathDx SDK 内部精度仍未接受；即使 tile 外壳为 f32，也不代表 SDK 内部精度可验收。
- 产品精度验收仍按冻结 MJWarp 口径与 NP01 至 NP04 规则另行执行，本评审不作结论。

内核来源两条路线：

| 路线 | 依赖 | 说明 |
| --- | --- | --- |
| 上游源码编译 | W15 头文件原样复制 + W13 绑定 LTO 链接 | `tile*.h` 属条件复制级；源码头文件逐字节固定 |
| 冻结链接产物 | W13 绑定 + 冻结 LTO/FATBIN 摘要 | 需要产物摘要与版本记录；不重新编译 |

本评审不选路线；实施任务按 W13 与 W15 的结论择一，并在回执中写明。

## 风险与缓解

| 风险 | 缓解 |
| --- | --- |
| 共享内存超设备上限被驱动拒绝 | 发射前按设备属性核对；opt-in 单独评审 |
| 线程数与内核不符导致屏障死锁或未定义行为 | `BlockRule` 与编译参数双核对；GPU 测试覆盖拒绝面 |
| 静态共享内存大小不可静态推断 | 实施任务增加 ABI 常量或产物属性核对 |
| 动态共享内存被误启用 | 本轮明确不启用；非零值走独立评审 |
| tile 精度被 SDK 内部实现污染 | 沿用 W13 结论：SDK 内部精度未接受 |
| 上游头文件来源漂移 | W15 原样复制与逐文件摘要 |
| 探针上限常数被误用于产品 | 产品路径单独定义上限 |

## 影响面

| 对象 | 影响 |
| --- | --- |
| `mjwarp.runtime` 发射器 | 需扩展共享内存与线程参与校验；本轮不改 |
| `runtime::launcher` ABI | `KernelAbi` 可能新增共享内存声明；实施任务另立 |
| `mjwarp.runtime.compiler` | 内核来源与链接能力依赖 W13/W15 |
| W13 nvJitLink | tile 走 LTO 路线时依赖其结论 |
| W15 Warp 头文件 | tile 走源码路线时依赖 `tile*.h` 条件复制 |
| 产品精度 | 不改变现有内核、容差与 f32 路径 |
| 现有发射路径 | 全部 `shared_mem_bytes: 0` 保持；不影响已验收内核 |

## 前置条件

实施任务启动前须满足：

1. 冻结 `KernelAbi` 的共享内存声明方式（常量或产物属性）。
2. 冻结设备属性核对与动态共享内存 opt-in 策略。
3. 冻结 tile 线程参与检查（块大小、每 block 单元数、barrier 前提说明）。
4. 选定内核来源路线（W13 链接或 W15 源码编译）。
5. 给出 GPU 测试覆盖：共享内存超限拒绝、块大小不符拒绝、tile 装卸与清零、失败路径不留半个 tile、精度对照。
6. 精度对照只作补充证据，不豁免既有容差。
7. 不新增 Python 构建或运行链。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | 通过，退出码 0 |
| `unchanged` 检查 | `src`、`native`、`include`、`tests`、`fixtures`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未改动 |
| `git status` | 仅 `?.claude/` 未跟踪 |

本轮未新增内核。本轮未改发射器。本轮未新增依赖。本轮未运行 GPU 测试。本轮未作精度验收结论。

## 依据

- `docs/architecture.md`、`docs/unified-launcher.md`、`docs/module-contracts.md`。
- `docs/development-evidence/layers-w13.md`、`docs/development-evidence/layers-w15.md`。
- `docs/evidence/native-mathdx7-link/{audit.json,probe.txt,contract.json}`。
- `scripts/native-mathdx7-call.cu`、`scripts/native-mathdx7-link.cpp`。
- `src/runtime/launcher/{launch.rs,abi.rs}`、`src/runtime/transfer/kernel/{launch.rs,launchers_physics.rs}`。
- `src/runtime/cache.rs`、`src/runtime/cuda.rs`、`src/runtime/cubecl.rs`、`docs/cubecl-probe.md`。
