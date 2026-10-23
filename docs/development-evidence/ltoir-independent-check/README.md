# LTOIR 交付独立复核（提交 f9da799）

日期：2026-10-24。执行：QA。任务：`2026-10-24-ltoir-review-g01-rerun`。
复核对象：`2026-10-24-compile-chain-ltoir` 交付（`f9da799`，登记 `edebb97`），平台 Windows / RTX 4070 Ti SUPER / 驱动 596.36 / SM89。

## 1. 结论（先给）

LTOIR 交付声明**全部成立**：接口、门控、未改既有路径、证据数字与门禁记录经独立复跑一致。
发现 **2 条建议**（均为文档/声明准确性问题，不阻断）：① 证据标注行数与实际不符；② 「新 GPU 探针未接入 all-probes」的边界声明不准确——探针实际被 all-probes 自动纳入，并在本次 G01 两模式中 2/0 通过。

## 2. 交付范围与门控核对

| 检查项 | 结果 |
| --- | --- |
| 新增文件 | `src/runtime/launcher/artifact/ltoir.rs`（320 行 / 10,612 B）、`ltoir/tests.rs`（125 行）、`tests/compile_ltoir_probe.rs`（249 行） |
| 接线改动 | `artifact.rs` 仅 `mod ltoir;` + `pub use`；`launcher/mod.rs` 仅在 `#[cfg(feature = "cuda-probe")]` 的 `pub use` 中追加导出 |
| 仅 cuda-probe 门控 | `launcher/mod.rs` 的 `mod artifact;` 本身在 `#[cfg(feature = "cuda-probe")]` 下；新接口不参与默认特性编译（默认 `cargo test --locked` 480/0 通过） |
| 既有 PTX/CUBIN 路径 | `artifact/nvrtc.rs` 未被本提交改动（`git show --name-status` 未列出）；PTX/CUBIN 行为不变 |
| 内核字符串/容差/f32 | 改动仅 `src/runtime/launcher/*`（运行时编译链，cuda-probe 门控），未触碰 physics/内核字符串/容差/f32 |
| 无 C/C++/Python/venv | 提交无 `include/`、`native/`、`scripts/` 文件；源码与测试无 Python/venv 调用 |
| 模块归属 | 任务 `modules = {modify: [mjwarp.runtime]}`，与代码位置（`src/runtime/launcher/artifact/`）一致 |
| MathDx | 未链接 SDK；仅冻结选项含既有 `--define-macro=WP_ENABLE_MATHDX=1`（与 W13 相同，不是 SDK 接入） |

## 3. 独立复跑与数字对照

| 命令 | 结果 |
| --- | --- |
| `cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features cuda-probe --lib ltoir` | **8 过 0 败**（与证据一致） |
| `... --features cuda-probe --test compile_ltoir_probe -- --ignored --test-threads=1 --nocapture` | **2 过 0 败**（`CUDA_PATH=target/toolchains/cuda-12.8.1` 且 `bin` 入 PATH） |

本次实测探针 JSON 与证据逐项一致：

| 项 | 本次实测 | 证据登记 |
| --- | --- | --- |
| 调用 LTOIR 字节 / sha256 | 3632 / `1415be10…` | 3632 / `1415be10…` |
| CUBIN 字节 / sha256 | 2086544 / `1086284b…` | 2086544 / `1086284b…` |
| 设备输出字节 / sha256 | 104528 / `51e1a99b…` | 104528 / `51e1a99b…` |
| 发射次数 / 设备阶段耗时 | 8 / 6.92 ms（6.8 ms 级） | 8 / 6.8275 ms |
| NVRTC / nvJitLink 版本 | [12,8] / [12,9] | [12,8] / [12,9] |
| 重复编译 / 重复链接摘要一致 | true / true | true / true |

门禁记录（`.normify/runs/2026-10-24-compile-chain-ltoir/`）：fmt/clippy/clippy-cuda 均 exit 0、0 警告；unit 480 过 0 败（含新增 8 项 ltoir 单测）。

## 4. 边界声明核对

- 不接 MathDx SDK：成立（无 SDK 链接，仅冻结选项字符串）。
- Linux deferred：成立（本任务未跑 Linux，登记在 `2026-10-24-linux-batch-retest`）。
- G01 未在本任务复跑：成立（任务内无 G01 运行痕迹）；本次 QA 已另行复跑（见 `g01-fast-loop/README.md` §5.4）。
- T4 源码身份未漂移：成立——锚点组合（`t4-resident-anchors/physics.rs`）只含 `src/physics/{mod,com_position,device_checks,history,kinematics}.rs`，不含 `src/runtime/launcher`；无需重钉。

## 5. 差异清单（建议级）

### 5.1 建议：证据标注行数与实际文件不符

- 现象：`docs/development-evidence/compile-ltoir/README.md` 与 `receipt.json` 标注 `ltoir.rs` 288 行、`tests.rs` 123 行、`probe` 241 行；
  实际 HEAD 文件为 **320 / 125 / 249 行**（LF 行数，`git hash-object` 与回执登记 blob 一致，即内容未变、仅行数字段陈旧）。
- 影响：仅文档准确性，不影响功能/数字；建议后续修正行数标注（或改为登记 blob 不登记行数）。

### 5.2 建议：`新 GPU 探针未接入 all-probes` 的边界声明不准确

- 现象：README 与回执边界写「新 GPU 探针未接入 G01 脚本 all-probes 列表，需后续接线/QA 复跑决定」。
- 实际：G01 的 all-probes 阶段执行 `cargo test --all-features -- --ignored --test-threads=1`（覆盖全部测试 target，无显式白名单），
  因此 `compile_ltoir_probe` 的两项 ignored 测试**自动纳入**。本次 G01 两模式实测：all-probes 348→**350 过 0 败**、
  result 行 84→**85**，探针 2 项在两种模式下均通过（`all-probes.log` 中 `compile_ltoir` 命中 2 次）。
- 影响：不影响通过性，反而多出 2 项 GPU 端到端覆盖；但边界描述与脚本实际行为不符，建议在证据/回执中更正为「已自动纳入」。

## 6. 边界

- 只读复核 + 复跑 + 证据目录；未改产品/测试/容差/fixtures/编译链源码、未改排除清单或 G01 脚本。
- 未验证 Linux、MathDx SDK 接入或完整上游等价；G01 双模式复验见 `g01-fast-loop/README.md` §5.4。
- 复跑环境：Windows / RTX 4070 Ti SUPER / CUDA 12.8.1（`target/toolchains/cuda-12.8.1`）/ rustc 1.99.0-msvc。
