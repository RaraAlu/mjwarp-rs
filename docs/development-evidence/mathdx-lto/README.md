# MathDx Solver LTO 生成与冻结产物对照

任务：`2026-10-24-mathdx-solver-lto`；NVRTC 组件化与 G01 环境解耦：`2026-10-24-nvrtc-12-9-mathdx-component`。前置：`2026-10-24-mathdx-libmathdx-prep`（提交 `2025a57`，组件记录见同目录 `component.json`）。
平台：Windows，NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，SM89。
组件：`libmathdx` 0.3.1（cuda12、win32-x86_64）与 pinned `cuda_nvrtc` 12.9.86（CUDA 12.9.1 redist），摘要与落盘路径见 `component.json`。

## 结论

- Rust 侧手写 C ABI 动态加载 `mathdx64_0.dll`（Windows `__stdcall`，x64 同 C 调用约定），按 Warp 1.15.0 `warp.cu::wp_cuda_compile_solver` 的调用顺序生成 7 维 POTRF LTOIR 与 universal FATBIN。
- 与 2026-10-14 冻结证据逐字节一致：LTOIR **1,034,872 B** `98c01ce6…`、FATBIN **506,960 B** `c00d6412…`。
- 端到端复用 W13 nvJitLink：调用 LTOIR → MathDx LTOIR → FATBIN 链接为 CUBIN **2,086,544 B** `de660b5d…`（与冻结证据一致）；GPU **8 次发射**（6 批正定 + 2 次坏主元），设备输出 **104,528 B** `51e1a99b…` 一致。
- 同参数重复生成、重复链接摘要一致（确定性）。

## 两个关键环境事实（本机实测）

1. **工作目录进入 LTOIR**：同参数、同 libmathdx、同 NVRTC，仅更换进程工作目录即得到不同摘要：
   - `D:\Rust\mjwarp-rs` → `9fd948c3…`
   - `C:\Windows\Temp` → `e75d9ea0…`
   - `C:\Rust\mjwarp-rs`（2026-10-14 冻结证据的工作目录）→ `98c01ce6…`，与证据一致。
   因此逐字节对照必须在该冻结工作目录下生成。探针默认切到 `C:\Rust\mjwarp-rs`（`MATHDX_EVIDENCE_CWD` 可覆盖），结束或 panic 时恢复；该切换只在测试内，产品 API 不做 chdir，产物与运行工作目录绑定。
2. **NVRTC 版本决定 LTOIR 字节**：NVRTC 12.8.93 → LTOIR 1,035,080 B（不同摘要）；NVRTC 12.9.86 → 1,034,872 B，与证据一致。NVRTC 12.9.86 已按组件模式固定：`scripts/prepare-cuda-probe.ps1` 只解包到 `target/toolchains/cuda-12.9.1-nvrtc/`（CUDA 12.9.1 redist，压缩包 314,608,748 B，sha256 `1aa0644f…`；DLL 89,832,960 B `0cc63349…`，builtins 7,217,664 B `c405226b…`，见 `component.json` 的 `nvrtc` 节）。
   MathDx 绑定与探针通过显式目录（`load_from_dir_with_nvrtc`）或 `MATHDX_NVRTC_DIR` 解析该 pinned 组件，与 `CUDA_PATH` 解耦；缺组件显式报错，不回退到 12.8.93。现有产品内核编译路径仍用 12.8.93（不全局升级）。

## 接口（`src/runtime/launcher/mathdx.rs`，454 行）

| 对象 | 说明 |
| --- | --- |
| `MathDx::load_from_dir` / `load_from_env` / `load_file` | 只按绝对路径加载；依赖预加载顺序：`MATHDX_NVRTC_DIR`（显式）→ 库同目录 → `CUDA_PATH/bin` |
| `MathDx::load_from_dir_with_nvrtc` / `load_file_with_nvrtc` | 强制从显式 NVRTC bin 目录预加载 pinned 12.9.86（含 builtins）；缺组件显式报错，不回退 |
| `MathDx::version` | `mathdxGetVersionEx` 返回 0.3.1；`cusolverdxGetVersion`（组件版本 0.3.0）只作诊断，不作钉版 |
| `MathDx::generate_solver_lto` | 按 `SolverSpec` 生成 `SolverLto`（LTOIR/FATBIN 字节、SHA-256、libmathdx 版本） |
| `SolverSpec::potrf_7d_sm89` | 冻结 contract：M=N=7、K=1、function=POTRF(2)、side/diag 不设置、precision=F32(5)、row-major(1,1)、fill=UPPER(0)、32 线程、sm_89、符号 `potrf_7_7_1_89_32_1_1_5_x_x_0` |

- 描述符调用顺序与 `warp.cu` 一致：Size → BlockDim → Type → API → Function → [Side] → [Diag] → Execution → Precision → Arrangement → FillMode → SM → SymbolName → GetLTOIR → GetUniversalFATBIN。
- 依赖预加载：`mathdx64_0.dll` 静态依赖 `nvrtc64_120_0.dll`；加载前按绝对路径预加载 pinned NVRTC 与 `nvrtc-builtins64_129.dll`，使按名解析命中已加载模块，不修改进程 DLL 搜索目录。
- 并发与生命周期：`MathDx` 可跨线程共享，生成调用经互斥锁串行；描述符按调用创建，守卫 `Drop` 调 `cusolverdxDestroyDescriptor`；句柄不跨线程共享。

## 拒绝面

| 情形 | 行为 | 覆盖 |
| --- | --- | --- |
| DLL 缺失 / 相对路径 | `Linker`：库缺失 / 须为绝对路径 | 单元测试 |
| 符号缺失 | `Linker`：MathDx符号缺失 | 单元测试（kernel32.dll 实测） |
| 版本不符 | `Linker`：MathDx版本不符 | 单元测试 |
| 描述符创建/设置失败 | `Compiler`：MathDx…失败 + `commondxGetLastErrorStr` 真实日志 | GPU 探针（非方阵 POTRF 实测） |
| LTOIR/FATBIN 为空 | `Compiler`：MathDx LTOIR/FATBIN为空 | 代码路径 |
| 非法形状/线程/枚举/SM/符号 | `Compiler`：求解器规格无效 | 单元测试 |
| 链接错误、空产物、非 CUBIN ELF | 复用 W13 拒绝面 | W13 单元 + GPU 测试 |

## 验证（本机实测）

| 命令 | 结果 |
| --- | --- |
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib mathdx` | mathdx 模块 **11 项通过**（`unit-tests.log`；同过滤另匹配 2 项既有物理测试名） |
| 普通环境：`CUDA_PATH=target/toolchains/cuda-12.8.1`、`PATH` 加其 `bin`；探针从 pinned 组件解析 NVRTC | **2 过 0 败**（`probe.log`） |
| G01 环境变量集：`CUDA_PATH=target/toolchains/cuda-12.8.1`、`PATH` 加其 `bin`，并设 `MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL` | **2 过 0 败**（`probe-g01-env.log`；两项摘要相同，证明与 CUDA_PATH 解耦） |

探针 JSON（2026-10-24 实跑，工作目录 `C:\Rust\mjwarp-rs`；两环境一致）：

```json
{"mathdx_version":[0,3,1],"ltoir_bytes":1034872,"ltoir_sha256":"98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb","fatbin_bytes":506960,"fatbin_sha256":"c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f","repeat_generate_identical":true,"call_ltoir_bytes":3632,"call_ltoir_sha256":"a323db381e0793b03837c95179965eb90aeb44c110eae042f16ede1d587da953","call_nvrtc_version":[12,9],"cubin_bytes":2086544,"cubin_sha256":"de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae","repeat_link_identical":true,"device_output_bytes":104528,"device_output_sha256":"51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f","launches":8,"device_phase_ms":6.22,"gpu_cc":[8,9],"evidence_cwd":"C:\\Rust\\mjwarp-rs"}
```

- 说明：调用 LTOIR 由产品 `compile_ltoir` 在 pinned NVRTC 12.9.86 下生成（3,632 B，`a323db38…`；`call_nvrtc_version=[12,9]` 证明显式解析生效，即使 `CUDA_PATH` 指向 12.8.1）；它与冻结证据中的调用 LTOIR（W13 的 12.8 口径 `1415be10…`）不同，但链接后 CUBIN 与设备输出均与 2026-10-14 证据逐字节一致。
- 环境：`CUDA_PATH=target/toolchains/cuda-12.8.1`（nvJitLink 12.9.86 在其 bin）；pinned NVRTC 默认来自 `target/toolchains/cuda-12.9.1-nvrtc/bin`（`MATHDX_NVRTC_DIR` 可覆盖）。

## 边界与后续

- 不接产品物理配方/既有 PTX/cubin 路径/内核字符串/容差/f32；绑定仅在 `cuda-probe` feature 下编译。
- Linux 未验收，归 `2026-10-24-linux-batch-retest`；SDK 二进制与静态库只在 `target/`，再分发边界未定。
- **G01 环境已解耦并验证**：pinned NVRTC 12.9.86 由 `scripts/prepare-cuda-probe.ps1` 安装到 `target/toolchains/cuda-12.9.1-nvrtc/`，探针与绑定通过显式路径/`MATHDX_NVRTC_DIR` 解析，不依赖 `CUDA_PATH`（G01 当前为 12.8.1）。本任务已在普通环境与 G01 环境变量集下各跑一次探针，均 2 过 0 败且四项冻结摘要一致；未改 `scripts/test-windows-g01.ps1` 与排除清单。现有产品内核编译路径仍用 12.8.93，未全局升级。
- **三条位稳定边界**（供后续引用）：① libmathdx 0.3.1 会把进程工作目录写进 LTOIR，同参数不同 CWD 摘要不同；② LTOIR 字节由 NVRTC 版本决定（12.8.93→1,035,080 B 不匹配，12.9.86→1,034,872 B 匹配）；③ 因此跨目录/跨机位位稳定不成立，冻结字节对照须固定工作目录 `C:\Rust\mjwarp-rs`（`MATHDX_EVIDENCE_CWD` 可覆盖）并使用 pinned NVRTC 12.9.86。
- 产品 API 不做 chdir；探针切工作目录仅在测试内，结束/panic 时恢复。
- T4 源码身份：改动只在 `src/runtime/launcher`、`tests/` 与 `scripts/prepare-cuda-probe.ps1`（cuda-probe 门控），不在 T4 锚点集合（physics/resident/mass-solve）内，未漂移，无需重钉。

## 复现

```powershell
# 先准备组件（libmathdx + pinned NVRTC 12.9.86 + nvJitLink 12.9.86）
target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File scripts\prepare-cuda-probe.ps1

# 与 G01 相同的 CUDA 环境；pinned NVRTC 由探针从组件目录显式解析
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1'
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe `
  --test mathdx_solver_probe -- --ignored --test-threads=1 --nocapture
```
