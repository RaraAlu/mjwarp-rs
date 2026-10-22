# W13 nvJitLink 动态绑定评审

任务：`2026-10-18-layers-m7-link`。
日期：2026-10-22。
依据：`docs/architecture.md`、`docs/unified-launcher.md`、`docs/module-contracts.md`。
本轮只评审，不实现、不加依赖、不改代码。
本轮结论不自动批准实现。

## 结论

批准自建 nvJitLink 动态绑定这条技术路线，但**有条件**。
条件未满足前不进入实施；实施另立交付任务。
不采用 `warp.dll` 或 Python 运行链作为产品绑定。
不新增 nvJitLink 的 C/C++ 组件，Rust 侧手写 `extern "C"` 与动态加载。
既有 MathDx 证据只作离线参考，不作为产品验收依据。

结论：**有条件批准评审路线，实施延后**。

## 事实基线

| 事实 | 证据 | 结论 |
| --- | --- | --- |
| cudarc 0.18.2 不含 nvJitLink 绑定 | 本地 crate 源码 `src/` 只有 driver、nvrtc、runtime、cublas、cudnn 等模块；无 jitlink 目录或 API | 需自建绑定 |
| CUDA 12.8.1 工具链未含 nvJitLink 组件 | `target/toolchains/cuda-12.8.1/bin` 只有 nvcc、nvrtc、cudart、ptxas 等；无 `nvJitLink_120_0.dll` | 需补组件或运行时依赖 |
| 工具链准备只拉四个组件 | `scripts/prepare-cuda-probe.ps1` 下载 `cuda_nvrtc`、`cuda_cudart`、`cuda_cccl`、`cuda_nvcc` | 未含 `cuda_nvjitlink` |
| 设计已登记缺失 | `docs/unified-launcher.md` 把 nvJitLink 列入「完全缺失」，底座写「自建 nvJitLink 动态绑定」，W13 依赖写「无」，原因写「cudarc 不含此库」 | 路线与设计一致 |
| 许可与部署仍列待核 | `docs/unified-launcher.md` 待决清单含「nvJitLink 的许可与部署方式」 | 本轮给出核对项，不做法律结论 |
| 仓库许可目录只有 Apache-2.0 | `LICENSES/Apache-2.0.txt` | CUDA 组件许可需另行登记 |

## 既有 MathDx 证据复核

证据目录：`docs/evidence/native-mathdx7-link/`（`audit.json`、`probe.txt`、`contract.json`）。
来源提交：`b5ef1ba`，日期 2026-10-14。
性质：离线工具证据，非产品链证据。

实际链路：

- 探针 `scripts/native-mathdx7-link.cpp` 用 `LoadLibraryExW` 加载 **pinned `warp.dll`**，经 `GetProcAddress` 取 `wp_cuda_compile_program` 入口。
- 链接输入是 LTOIR 与 FATBIN，类型枚举 `{3, 4}`，注释标为 nvJitLink 的 LTOIR/FATBIN 取值。
- 链接选项 `-dlto`、`-arch=sm_89`；产物为 CUBIN ELF，`linked_bytes` 2086544。
- `warp.dll` 来自 Python venv：`target/mjwarp-parallel/venv/Lib/site-packages/warp/bin/warp.dll`；头文件 `--include-path` 也指向该 venv 的 `warp/native`。
- `probe-windows-mathdx7-link.ps1` 先按 wheel `RECORD` 与 SHA-256 核对 `warp.dll`，再编译运行。

证据结论（可复核）：

| 项 | 值 |
| --- | --- |
| 平台 | Windows，RTX 4070 Ti SUPER，驱动 596.36，sm_89 |
| NVRTC | 12.9；CUDA 头 12.8.1 |
| 覆盖 | 6 批、8 次发射、25480 标量、下三角正零 10920、坏主元 2 次 |
| 容差 | U 绝对/相对 2e-4；重建 2e-6；max_abs 9.537e-7；max_reconstruction 1.748e-7 |
| 精度边界 | 共享输入与输出 f32；SDK 内部精度未接受；未对照冻结 MJWarp 7 维模板；未看 SASS |
| 边界 | 离线工具；不改产品源码；SDK 二进制不入库；不分发、不逆向、不接受新许可 |

复核判断：

- 该证据证明的是「Warp 封装后的 JIT 链接能力在冻结输入上可用」。
- 它不证明 Rust 产品链可直接绑定 nvJitLink，也不证明部署与许可已解决。
- 它依赖 Python venv 中的 `warp.dll` 与 Warp 头文件，不能进入产品构建或运行链；AGENTS 禁止 Python 构建或运行链。
- 证据中的 `linked_bytes`、CUBIN ELF 与链接选项可作为 W13 实施后的对照样本，但不作产品验收。

## 绑定方案评审

建议方案：Rust 侧动态加载 + 手写 C ABI。

- 加载：Windows 用 `LoadLibraryExW`（限定搜索目录），Linux 用 `dlopen`；句柄与版本查询集中在 `mjwarp.runtime.compiler`。
- 符号：按需 `GetProcAddress`/`dlsym` 取 `nvJitLinkCreate`、`nvJitLinkAddData`、`nvJitLinkComplete`、`nvJitLinkGetLinkedCubin`、`nvJitLinkDestroy` 等；缺失符号必须显式报错，不静默回退。
- 输入：LTOIR 类型 3、FATBIN 类型 4（与既有证据一致）；选项字符串 `-dlto`、`-arch=sm_XX`。
- 值类型：入口只用指针、`size_t`、`int` 与不透明句柄；不跨 FFI 传 Rust 所有权；错误码映射到 `TransferError`/`ProbeError` 风格。
- 构建：不新增 C/C++ 组件，不启用 bindgen 生成链；`Cargo.toml` 依赖变化属实施任务，本评审不引入。
- 线程与生命周期：句柄按调用创建销毁，或按上下文缓存并串行访问；不得跨线程共享未同步句柄。并发策略在实施任务冻结。
- 失败面：库缺失、版本不符、符号缺失、链接错误、产物为空、非 CUBIN ELF、超限输入全部覆盖。

版本约束（评审意见，待实施核对）：

- nvJitLink 主版本须与 CUDA 12.x 匹配；本机驱动 596.36 满足 CUDA 12.8 运行时。
- 工具链建议固定 `cuda_nvjitlink` 组件，按 `scripts/prepare-cuda-probe.ps1` 模式记录下载摘要与版本。
- 若运行期依赖用户已装 CUDA，须在启动时核对库版本并报错；两种部署二选一需产品决策。

许可与部署：

- nvJitLink 属 NVIDIA CUDA Toolkit 组件，许可为 CUDA Toolkit EULA；是否允许随二进制再分发须以 SDK 随附许可文本为准。
- 仓库不提交 SDK 二进制；如需随构建缓存落地，只进 `target/` 忽略目录。
- 若产品分发，`LICENSES/` 与第三方通知须登记相应许可文本与来源；本轮不做法律结论，只列条件。
- 现有 `LICENSES/` 只有 Apache-2.0，缺 CUDA 组件许可登记。

无 Python 边界：

- 产品构建与运行链不得调用 Python，也不得读取 venv 中的 `warp.dll` 或 Warp 头文件。
- 头文件来源固定为 CUDA SDK 版本；若未来需要 Warp 头文件，走 W15 的原样复制路线，与 Python 无关。
- 评审材料中的 Python 路径只属历史证据描述。

## 影响面

| 对象 | 影响 |
| --- | --- |
| `mjwarp.runtime.compiler` | 新增绑定与版本查询；模块状态从 planned 起改，但本轮不改代码 |
| W15 Warp 头文件 | 编译上游源码需要 LTO 链接能力；W13 是其前置之一 |
| MathDx LTO 路线 | 实施后才具备产品侧链接能力；L5 保留仓外，W13 完成后再评估 |
| W12 条件图节点 | 无直接影响 |
| 产品精度 | 绑定只决定链接与产物，不改 f32 路径、容差与内核字符串 |
| 构建与部署 | 新增运行时库依赖与许可登记；由实施任务单独引入 |
| Linux | 绑定需双平台；本轮不评审 Linux 部署 |

## 前置条件

实施任务启动前须满足：

1. 工具链固定 `cuda_nvjitlink` 组件版本与摘要，或明确运行期依赖用户 CUDA 库。
2. 许可文本与第三方通知进入 `LICENSES/` 或分发清单；明确可否再分发。
3. 冻结绑定 ABI：函数清单、签名、错误码、版本查询与拒绝面。
4. 冻结并发与生命周期策略。
5. 给出实施后对照样本：同一 LTOIR/FATBIN 输入在既有离线证据与新绑定下的产物摘要一致。
6. 无 Python 验证：构建与测试命令不出现 Python 解释器或 venv 路径。
7. Linux 另行评审，不与 Windows 同轮放行。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `cargo test --locked` | 通过，退出码 0 |
| `unchanged` 检查 | `src`、`native`、`include`、`tests`、`fixtures`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未改动 |
| `git status` | 仅 `?.claude/` 未跟踪 |

本轮未实现绑定。本轮未新增依赖。本轮未编译任何 CUDA 源。本轮未运行 GPU 测试。本轮未做许可法律结论。

## 依据

- `docs/architecture.md`（三层划分）。
- `docs/unified-launcher.md`（底座、W13、待决清单）。
- `docs/module-contracts.md`（runtime 职责）。
- `docs/evidence/native-mathdx7-link/{audit.json,probe.txt,contract.json}`。
- `scripts/native-mathdx7-link.cpp`、`scripts/probe-windows-mathdx7-link.ps1`。
- `scripts/prepare-cuda-probe.ps1` 与 `target/toolchains/cuda-12.8.1`。
- `Cargo.toml`、`Cargo.lock` 的 cudarc 0.18.2 依赖与本地 crate 源码。
- `LICENSES/`。
