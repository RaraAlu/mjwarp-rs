# MathDx Solver LTO 生成与冻结产物对照

任务：`2026-10-24-mathdx-solver-lto`。前置：`2026-10-24-mathdx-libmathdx-prep`（提交 `2025a57`，组件记录见同目录 `component.json`）。
平台：Windows，NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，SM89。
组件：`libmathdx` 0.3.1（cuda12、win32-x86_64），`mathdx64_0.dll` 见 `component.json`。

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
2. **NVRTC 版本决定 LTOIR 字节**：NVRTC 12.8.93 → LTOIR 1,035,080 B（不同摘要）；NVRTC 12.9.86 → 1,034,872 B，与证据一致。本机 12.9.86 来自 W13 诊断目录 `target/w13/diag-12.9.86`（CUDA 12.9.1 redist `cuda_nvrtc` 12.9.86），尚未纳入 `scripts/prepare-cuda-probe.ps1`（见边界）。

## 接口（`src/runtime/launcher/mathdx.rs`，454 行）

| 对象 | 说明 |
| --- | --- |
| `MathDx::load_from_dir` / `load_from_env` / `load_file` | 只按绝对路径加载；`load_from_env` 读 `MATHDX_PATH`（安装根/bin） |
| `MathDx::version` | `mathdxGetVersionEx` 返回 0.3.1；`cusolverdxGetVersion`（组件版本 0.3.0）只作诊断，不作钉版 |
| `MathDx::generate_solver_lto` | 按 `SolverSpec` 生成 `SolverLto`（LTOIR/FATBIN 字节、SHA-256、libmathdx 版本） |
| `SolverSpec::potrf_7d_sm89` | 冻结 contract：M=N=7、K=1、function=POTRF(2)、side/diag 不设置、precision=F32(5)、row-major(1,1)、fill=UPPER(0)、32 线程、sm_89、符号 `potrf_7_7_1_89_32_1_1_5_x_x_0` |

- 描述符调用顺序与 `warp.cu` 一致：Size → BlockDim → Type → API → Function → [Side] → [Diag] → Execution → Precision → Arrangement → FillMode → SM → SymbolName → GetLTOIR → GetUniversalFATBIN。
- 依赖预加载：`mathdx64_0.dll` 静态依赖 `nvrtc64_120_0.dll`；加载前从显式 `CUDA_PATH/bin` 或库同目录按绝对路径预加载，使按名解析命中已加载模块，不修改进程 DLL 搜索目录。
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
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib mathdx` | mathdx 模块 **10 项通过**（`unit-tests.log`；同过滤另匹配 2 项既有物理测试名） |
| `... test --locked --features cuda-probe --test mathdx_solver_probe -- --ignored --test-threads=1 --nocapture` | **2 过 0 败**（`probe.log`） |

探针 JSON（2026-10-24 实跑，工作目录 `C:\Rust\mjwarp-rs`）：

```json
{"mathdx_version":[0,3,1],"ltoir_bytes":1034872,"ltoir_sha256":"98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb","fatbin_bytes":506960,"fatbin_sha256":"c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f","repeat_generate_identical":true,"call_ltoir_bytes":3632,"call_ltoir_sha256":"a323db381e0793b03837c95179965eb90aeb44c110eae042f16ede1d587da953","cubin_bytes":2086544,"cubin_sha256":"de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae","repeat_link_identical":true,"device_output_bytes":104528,"device_output_sha256":"51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f","launches":8,"device_phase_ms":6.6907,"gpu_cc":[8,9],"evidence_cwd":"C:\\Rust\\mjwarp-rs"}
```

- 说明：调用 LTOIR 由产品 `compile_ltoir` 在 NVRTC 12.9.86 下生成（3,632 B，`a323db38…`）；它与冻结证据中的调用 LTOIR（W13 的 12.8 口径 `1415be10…`）不同，但链接后 CUBIN 与设备输出均与 2026-10-14 证据逐字节一致（W13 诊断也显示 12.8/12.9 调用的 CUBIN 相同）。
- 环境：`CUDA_PATH` 指向含 NVRTC 12.9.86 与 nvJitLink 12.9.86 的 `bin` 目录；`PATH` 追加其 `bin`（及 12.8.1 `bin` 兜底）。

## 边界与后续

- 不接产品物理配方/既有 PTX/cubin 路径/内核字符串/容差/f32；绑定仅在 `cuda-probe` feature 下编译。
- Linux 未验收，归 `2026-10-24-linux-batch-retest`；SDK 二进制与静态库只在 `target/`，再分发边界未定。
- **G01 接线缺口**：新 ignored 探针会被 G01 all-probes 的 `--ignored` 自动纳入，而 G01 当前 `CUDA_PATH` 为 12.8.1（NVRTC 12.8.93），冻结字节对照会失败。需 QA 复跑时排除该探针，或先完成下条接线；不得静默跳过。
- **建议后续小任务**：把 `cuda_nvrtc` 12.9.86 纳入 `scripts/prepare-cuda-probe.ps1` 的固定组件并让 MathDx 绑定/探针显式选择该 NVRTC；同时确定 G01 all-probes 的接线或排除口径。
- T4 源码身份：改动只在 `src/runtime/launcher` 与 `tests/`（`cuda-probe` 门控），不在 T4 锚点集合（physics/resident/mass-solve）内，未漂移，无需重钉。

## 复现

```powershell
# CUDA_PATH 指向同时含 nvrtc 12.9.86 与 nvJitLink 12.9.86 的 bin 目录；
# 本机证据运行使用 target/mathdx-prep/nvrtc129（诊断目录，不入库）。
$env:CUDA_PATH = 'D:\Rust\mjwarp-rs\target\mathdx-prep\nvrtc129'
$env:PATH = "$env:CUDA_PATH\bin;D:\Rust\mjwarp-rs\target\toolchains\cuda-12.8.1\bin;$env:PATH"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe `
  --test mathdx_solver_probe -- --ignored --test-threads=1 --nocapture
```
