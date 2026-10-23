# MathDx 交付独立复核（2025a57 / e5f1559 / 4f362f2）

日期：2026-10-24。执行：QA。任务：`2026-10-24-mathdx-review-g01-rerun`。
复核对象（HEAD `4f362f2`）：

| 提交 | 内容 |
| --- | --- |
| `2025a57` | libmathdx 0.3.1 组件与许可登记（`2026-10-24-mathdx-libmathdx-prep`） |
| `e5f1559` | MathDx Solver LTO 绑定与冻结产物对照（`2026-10-24-mathdx-solver-lto`） |
| `4f362f2` | NVRTC 12.9.86 组件化 + MathDx 路径解耦（`2026-10-24-nvrtc-12-9-mathdx-component`） |

平台：Windows / RTX 4070 Ti SUPER / 驱动 596.36 / SM89；CUDA_PATH `target/toolchains/cuda-12.8.1`（NVJitLink 12.9.86），pinned NVRTC 12.9.86 `target/toolchains/cuda-12.9.1-nvrtc/`，libmathdx 0.3.1 `target/toolchains/libmathdx-0.3.1/`。

## 1. 结论（先给）

三块交付声明**全部成立**：数字、绑定方式、门控、许可、产品内核路径、拒绝面与三条位稳定边界均与证据一致；环境准备脚本 exit 0；探针在普通与 G01 环境均 2 过 0 败、四项冻结摘要一致；G01 debug/release 双模式通过。
发现 **2 条建议**（证据行数元数据陈旧，不阻断）：`receipt.json` 的 deps/ffi/tests/probe 行数字段与实际不符；README 接口节 `mathdx.rs` 行数（454）与实际（490）不符（回执 490 正确）。

## 2. 交付声明核对

| 声明 | 复核结果 |
| --- | --- |
| LTOIR **1,034,872 B** `98c01ce6…` | 成立（探针 JSON 逐字节摘要一致） |
| FATBIN **506,960 B** `c00d6412…` | 成立 |
| CUBIN **2,086,544 B** `de660b5d…` | 成立（复用 W13 nvJitLink，与冻结证据一致） |
| 设备输出 **104,528 B** `51e1a99b…`、**8 次发射** | 成立 |
| MathDx 绑定走 `mathdx64_0.dll` 描述符 API | 成立：手写 C ABI + `LoadLibrary`/`GetProcAddress`（`ffi.rs`），按 `warp.cu` 顺序 Size→BlockDim→Type→API→Function→…→GetLTOIR→GetUniversalFATBIN；无新增 C/C++ 文件（三个提交无 `.cu/.cpp/.c/.h`） |
| 无 Python/venv/warp.dll 调用 | 成立（仅注释声明；prepare 为 PS1） |
| `mathdx.rs`/`deps.rs` ≤500 行、仅 cuda-probe 门控 | 成立：实际 **490 / 81 行**；`mod mathdx` 在 `#[cfg(feature = "cuda-probe")]` 下 |
| SDK 二进制未入库（只在 `target/`） | 成立：`git ls-files` 无 `mathdx64_0.dll`/`nvrtc64_120_0.dll`/静态库；tracked 仅 `LICENSES/libmathdx-0.3.1/{LICENSE.txt,PROVENANCE.md}` |
| LICENSES 有许可原文与 provenance | 成立：LICENSE.txt 16,818 B、sha256 `8cfdcfcb…`（与 `component.json` 一致）；PROVENANCE.md 记录来源 URL/包大小/sha256/登记任务；`THIRD_PARTY_NOTICES.md` 已登记 |
| 现有产品内核仍用 NVRTC **12.8.93** | 成立：`component.json.nvrtc.product_kernel_path` 明示未全局升级；Cargo 默认特性不编译 mathdx |
| 拒绝面测试真实 | 成立：单测覆盖缺库/相对路径/缺符号（kernel32 实测）/版本不符/非法形状线程枚举符号/缺 pinned NVRTC；GPU 探针用非方阵 POTRF（`Size<7,8,1>`）触发真实描述符/编译失败并断言 `MathDx…` 错误日志（`probe.log` 含 6 条真实 NVRTC 编译错误） |
| 证据与门禁记录一致 | 成立：`.normify/runs/` 下 prep（fmt + unchanged/json/forbid_claim）、solver-lto、nvrtc-component 的 fmt/clippy/clippy-cuda 与 unit（480/0）记录齐全 |

三条位稳定边界如实写入（README「两个关键环境事实」+「三条位稳定边界」+ receipt `environment_facts`）：

1. libmathdx 0.3.1 会把进程工作目录写进 LTOIR（`D:\Rust\mjwarp-rs`→`9fd948c3…`、`C:\Windows\Temp`→`e75d9ea0…`、`C:\Rust\mjwarp-rs`→`98c01ce6…` 与冻结证据一致）；
2. LTOIR 字节由 NVRTC 版本决定（12.8.93→1,035,080 B 不匹配；12.9.86→1,034,872 B 匹配）；
3. 跨目录/跨机位位稳定不成立，冻结对照须固定工作目录 `C:\Rust\mjwarp-rs` 且用 pinned NVRTC 12.9.86；`MATHDX_EVIDENCE_CWD` 仅测试内切目录，**产品 API 不做 chdir**（复核 grep：`src/runtime/launcher/mathdx*` 无 `set_current_dir`，切目录只在 `tests/mathdx_solver_probe.rs`）。

## 3. 环境准备与探针抽查

- `target\toolchains\powershell-7.6.7\bin\pwsh.exe -NoProfile -File scripts\prepare-cuda-probe.ps1` → **exit 0**，输出 `MATHdx_ROOT`/`NVRTC_12_9_ROOT`/`CUDA_PATH`。
- 组件实测：`mathdx64_0.dll` 14,854,656 B `5ce7528e…`；`nvrtc64_120_0.dll` 89,832,960 B `0cc63349…`；`nvrtc-builtins64_129.dll` 7,217,664 B `c405226b…`（与 `component.json` 一致）。
- 探针（`cargo +1.99.0-… test --locked --features cuda-probe --test mathdx_solver_probe -- --ignored --test-threads=1 --nocapture`）：

| 环境 | 结果 | 四项冻结摘要 |
| --- | --- | --- |
| 普通（`CUDA_PATH=cuda-12.8.1` + PATH） | **2 过 0 败** | 全部一致；`call_nvrtc_version=[12,9]`、`evidence_cwd=C:\Rust\mjwarp-rs` |
| G01 环境变量集（+`MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL`） | **2 过 0 败** | 全部一致（摘要与普通环境逐字节相同；`device_phase_ms` 7.91/7.54 仅耗时差异） |

## 4. G01 双模式复验（HEAD `4f362f2`）

运行 HEAD `384842f`（源码 `4f362f2` 之上仅一条任务登记提交）。两模式 **14 阶段均 exit 0**：

| 阶段 | debug | release |
| --- | --- | --- |
| 运行目录 | `target/g01-acceptance/7095eee44fcf4d08a7423a20485a664b` | `target/g01-acceptance/95c7dc56d5e44843a1b132ce6b338bb3` |
| 墙钟 | 788s | 619s |
| host-msvc | **823 过** | **823 过** |
| all-probes | **352 过 / 0 败**（86 result 行） | **352 过 / 0 败**（86 result 行） |
| rustdoc（`-D warnings`） | 0 警告 | 0 警告 |
| 排除清单 | 11 项 | 11 项（两模式一致，无新增） |
| diff | 通过 | 通过 |

其余阶段两模式一致：fmt 通过、native 27、g01 7、native-g01 2、native-only-g01 1、resident 102、host-gnu 480、clippy-gnu/gnu-cuda/msvc 0 警告。**mathdx 探针 2 项被 all-probes 自动纳入并通过**（all-probes 350→352、result 行 85→86）。

## 5. 差异清单（建议级）

### 5.1 建议：`receipt.json` 行数字段陈旧

- 现象：`mathdx/deps.rs` 74（实际 **81**）、`mathdx/ffi.rs` 224（实际 **256**）、`mathdx/tests.rs` 136（实际 **138**）、`tests/mathdx_solver_probe.rs` 311（实际 **325**）；`mathdx.rs` 490 正确。
- 说明：所有 `blob` 与 HEAD 一致（内容未变，仅行数元数据在最后一次提交后未同步），≤500 行约束不受影响；建议后续修正行数字段（或只登记 blob）。

### 5.2 建议：README 接口节 `mathdx.rs` 行数不符

- `docs/development-evidence/mathdx-lto/README.md` 写 `mathdx.rs`「454 行」，实际 **490**（`receipt.json` 记录 490，正确）。建议同步。

## 6. 边界

- 只做复核与复跑，未改产品/测试/容差/fixtures/排除清单/G01 脚本；G01 结论与数字由本次复跑给出。
- Linux 未验收（归 `2026-10-24-linux-batch-retest`）；MathDx SDK 再分发边界未定；性能与完整上游等价未验收；U038-3D known-debt 仍显式排除。
- 未验证跨机位/跨目录位稳定（证据已如实声明不成立，需固定 CWD + pinned NVRTC）。
