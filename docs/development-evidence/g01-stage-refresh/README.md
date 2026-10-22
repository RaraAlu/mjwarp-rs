# G01 未执行阶段补跑证据（W-D4 之后）

任务：`2026-10-22-g01-stage-evidence-refresh`
源码：`49fd4d9f9b249432f1b5273da558f6316f70af43` 之后（本任务只改文档，无源码/脚本/测试改动）
日期：2026-10-22

## 背景

W-D4 证书改为 f32 逐步口径并重生成后，full-debug 运行
（`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`，
目录 `target/g01-acceptance/9bd3489f5b4542409f104f5feac1c4fb`）通过
fmt/native/host-msvc/g01/native-g01/native-only-g01/resident，all-probes lib
124 过 0 败，但阶段在 `mass_solve_probe` 的 3 项既有 G03 门禁缺口处中止，
脚本后半段没有执行。

本任务按脚本第 112-128 行的同一命令与环境，单独补跑 host-gnu、clippy-gnu、
clippy-gnu-cuda、clippy-msvc、rustdoc、diff 六个阶段。未修改脚本。

## 环境

- Windows x86_64 MSVC；Rust 1.99.0（GNU 默认工具链 + 显式 `+1.99.0-x86_64-pc-windows-msvc`）
- CUDA 12.8.1：`target/toolchains/cuda-12.8.1`（PATH 前置 `bin`）
- MuJoCo 3.12.0：`target/toolchains/mujoco-3.12.0/package`（`MJWARP_MUJOCO_ROOT`/`MJWARP_MUJOCO_DLL`）
- GPU：RTX 4070 Ti SUPER，驱动 596.36

## 结果

| 阶段 | 命令 | 结果 | 耗时 |
| --- | --- | --- | --- |
| host-gnu | `cargo test --locked` | 退出 0，**404 过 0 败**（78 个测试二进制，门槛 ≥200） | 21.0s |
| clippy-gnu | `cargo clippy --locked --all-targets -- -D warnings` | 退出 0，0 警告 | 2.8s |
| clippy-gnu-cuda | `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | 退出 0，0 警告 | 7.6s |
| clippy-msvc | `cargo +1.99.0-x86_64-pc-windows-msvc clippy --locked --all-targets --all-features -- -D warnings` | 退出 0，0 警告 | 22.5s |
| rustdoc | `RUSTDOCFLAGS=-D warnings cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --all-features --no-deps` | 退出 0，0 警告 | 4.5s |
| diff | `git diff --check` | 退出 0，无空白错误（仅 Git 的 LF/CRLF 提示） | 0.06s |

逐阶段原始计数见 `stage-refresh.json`；日志（gitignored）在
`target/g01-stage-refresh/<stage>.log`，汇总在 `target/g01-stage-refresh/summary.json`。

## 结论与边界

- 六个未执行阶段均通过，当前源码没有新增格式、lint、文档链接或空白问题。
- 未重复运行已通过的 fmt/native/host-msvc/g01/native-g01/native-only-g01/resident；未重跑 all-probes。
- all-probes 现状：lib 124 过 0 败（W-D4 项已转绿），阶段整体仍因
  `mass_solve_probe` 3 项既有 G03 门禁缺口失败；已登记
  `2026-10-22-g01-g03-np03-admission-debt`（proposed，仅跟踪）。
- 不宣称完整 G01 通过；release 与 Linux 未跑。
