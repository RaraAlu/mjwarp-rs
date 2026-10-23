# 产品编译链 LTOIR 输出接口

任务：`2026-10-24-compile-chain-ltoir`。日期：2026-10-24。
来源：W13 证据「未完成与影响」第一条（产品编译链尚无 LTOIR 输出接口，测试内自建 NVRTC 调用）。
平台：Windows，NVIDIA GeForce RTX 4070 Ti SUPER，驱动 596.36，SM89；NVRTC 12.8.93，nvJitLink 12.9.86。

## 结论

- 产品侧 LTOIR 编译接口已落地：输入源/源名/头文件/include/arch（50–129）与冻结选项，输出 LTOIR 字节、SHA-256 与实际 NVRTC 版本。
- 13 项 NVRTC 选项与顺序按 W13 证据冻结，并有单元测试锁定。
- 端到端接口：程序 LTOIR + 可选 LTOIR/FATBIN → 复用 W13 `NvJitLink::link_cubin` → CUBIN + 摘要；同一输入两次编译/链接摘要一致。
- 与 W13 证据逐项一致：调用 LTOIR 3632 B（`1415be10…`）、CUBIN 2086544 B（`1086284b…`）、设备输出 104528 B（`51e1a99b…`，8 次发射）。
- 不改既有 PTX/CUBIN 编译路径、内核字符串、容差与 f32；不接 MathDx SDK；不调用 warp.dll，不启动 Python。

## 接口（`src/runtime/launcher/artifact/ltoir.rs`，288 行）

| 对象 | 说明 |
| --- | --- |
| `LtoirInput::new` / `with_headers` | 源名、源、头文件集合、include 路径、arch 校验 |
| `compile_ltoir` | 输出 `Ltoir`：LTOIR 字节、`sha256`、实际 `nvrtc_version` |
| `link_ltoir` | 程序 LTOIR 在前、附加输入按调用方顺序（可含 FATBIN）→ `LtoirCubin`（CUBIN + `sha256`） |
| `ltoir_options` | 冻结 13 项选项与顺序；arch 只替换首项 `--gpu-architecture=sm_<arch>` |

- 头文件先落临时目录，编译选项追加 `--include-path`；返回前清理。
- 每次编译创建独立 NVRTC 程序并在返回前销毁；编译可跨线程并发。链接串行策略沿用 W13 实例内互斥。

## 拒绝面

| 情形 | 行为 | 覆盖 |
| --- | --- | --- |
| 空源、空源名、NUL、非法字符/超长名 | `InvalidSource` | 单元测试 |
| 非法 arch（<50 或 >129） | `Compiler`：架构超出范围 | 单元测试 |
| include 路径为空/含 NUL/超长 | `Compiler` | 单元测试 |
| 源码超限（>8 MiB） | `InvalidSource` | 单元测试 |
| NVRTC 不可用 | `Compiler`：NVRTC不可用（不回退） | 单元测试（分支映射） |
| 编译失败 | `Compiler` + 真实 NVRTC 日志 | GPU 探针（broken.cu） |
| 链接失败、空产物、非 CUBIN ELF | 复用 W13 拒绝面 | W13 单元 + GPU 测试 |

## 验证（本机实测）

| 命令 | 结果 |
| --- | --- |
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib ltoir` | **8 过 0 败**（`unit-tests.log`） |
| `... test --locked --features cuda-probe --test compile_ltoir_probe -- --ignored --test-threads=1` | **2 过 0 败**（`probe.log`） |

探针 JSON（2026-10-24 实跑）：

```json
{"call_ltoir_bytes":3632,"call_ltoir_sha256":"1415be10987abafe56520b7f4e525023702bbd7c475164641f774271c934c34b","cubin_bytes":2086544,"cubin_sha256":"1086284ba5d1bd525d7320faee63978f6db79f955599d02232128da783433fe1","device_output_bytes":104528,"device_output_sha256":"51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f","device_phase_ms":6.8275,"gpu_cc":[8,9],"launches":8,"nvjitlink_version":[12,9],"nvrtc_version":[12,8],"repeat_compile_identical":true,"repeat_link_identical":true}
```

对照来源：W13 证据 `docs/development-evidence/w13-nvjitlink/receipt.json` 与 2026-10-14 离线证据 `docs/evidence/native-mathdx7-link/`。
环境：`CUDA_PATH=target/toolchains/cuda-12.8.1`，`PATH` 追加其 `bin`；探针输入为 `target/` 下钉死字节（不入库）。

## 边界

- 不接 MathDx SDK（L5 仍在仓外，许可与部署待用户裁决）；不新增 C/C++ 组件；无 Python/venv/warp.dll。
- 既有 PTX 编译路径 `artifact/nvrtc.rs` 与内核字符串、容差、f32 未改；新接口仅 `cuda-probe` feature 下编译。
- Linux 绑定 deferred，并入 `2026-10-24-linux-batch-retest`。
- G01 双模式未在本任务复跑：新单元测试 8 项会在下次 all-features host-msvc 计入（804→812）；新 GPU 探针未接入 G01 脚本 all-probes 列表，是否接线由后续任务/QA 复跑决定。
- T4 源码身份：改动只在 `src/runtime/launcher`（cuda-probe 门控），不在 T4 锚点集合（physics/resident/mass-solve）内，未漂移，无需重钉。

## 复现

```powershell
# 单元测试
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib ltoir

# GPU 探针（需 CUDA_PATH 与 target/ 下离线输入）
$env:CUDA_PATH = (Join-Path $PWD 'target\toolchains\cuda-12.8.1')
$env:PATH = "$env:CUDA_PATH\bin;$env:PATH"
cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe `
  --test compile_ltoir_probe -- --ignored --test-threads=1 --nocapture
```
