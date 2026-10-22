# G01 release 双模式验收证据

任务：`2026-10-22-g01-release-mode-evidence`
日期：2026-10-22
源码基线：`ca94aa0`（产品源码；本任务只改文档）
结论：**release 下已执行的阶段与 debug 一致，全部通过；all-probes 在同一已知债务处中止；release 不宣称 G01 全量通过。**

## 一、release 全脚本运行

命令（脚本未修改）：

```powershell
scripts/test-windows-g01.ps1 -AllFeatures -FullRegression -Release
```

- 运行目录：`target/g01-acceptance/f7c4de41f1024372b74e0b60119824ef`
- 本机日志：`target/g01-release-final.log`（gitignored）
- 墙钟：约 421 秒（17:33:29 → 17:40:30）
- 结果：**在 all-probes 阶段中止**（与 debug 相同的已知债务），退出码 1。

| 阶段 | 结果 | 说明 |
| --- | --- | --- |
| fmt | 通过（exit 0） | checks.json 记录通过，日志为空 |
| native | **27 过** | release 原生探针 |
| host-msvc | **709 过** | `cargo test --all-features --release` |
| g01 | **7 过** | `--release` |
| native-g01 | **2 过** | `--release` |
| native-only-g01 | **1 过** | `--release` |
| resident | **102 过** | `--release`，约 134 秒 |
| all-probes | lib **124 过 0 败**（461 filtered）；阶段整体**失败** | 见下 |

all-probes 失败明细（与 debug 完全一致，非 release 新问题）：

- 3 个 `mass_solve_probe` 测试失败：`gpu::gpu_factors_and_multiple_rhs_match_independent_native_solves`、
  `gpu::rhs_counts_one_two_three_and_five_reuse_each_world_factor`、
  `native::native_snapshots_feed_gpu_solves_without_native_physics_at_runtime`。
- NP03 判定失败块数：factors-and-rhs **2/240**、native-snapshots **2/240**、rhs-counts **6/152**
  （case4/world4 等，数值与 debug 相同；属 3 项既有准入缺口，跟踪任务
  `2026-10-22-g01-g03-np03-admission-debt`，本任务不动）。
- 即 release 下 W-D4 重力位证书项同样已转绿（lib 124 过 0 败）。

## 二、脚本中止后未执行阶段：release 运行上下文补跑

按 `scripts/test-windows-g01.ps1` 第 112-128 行的**同一定义**逐项执行（host-gnu/clippy-gnu/
clippy-gnu-cuda/rustdoc 脚本本身不加 `--release`；clippy-msvc 在 Release 模式加 `--release`）：

| 阶段 | 命令 | 结果 | 耗时 |
| --- | --- | --- | --- |
| host-gnu | `cargo test --locked` | 退出 0，**404 过 0 败**（78 个测试二进制，门槛 ≥200） | 16.3s |
| clippy-gnu | `cargo clippy --locked --all-targets -- -D warnings` | 退出 0，0 警告 | 1.8s |
| clippy-gnu-cuda | `cargo clippy --locked --all-targets --features cuda-probe -- -D warnings` | 退出 0，0 警告 | 4.9s |
| clippy-msvc | `cargo +1.99.0-x86_64-pc-windows-msvc clippy --locked --all-targets --release --all-features -- -D warnings` | 退出 0，0 警告 | 17.2s |
| rustdoc | `RUSTDOCFLAGS=-D warnings cargo +1.99.0-x86_64-pc-windows-msvc doc --locked --all-features --no-deps` | 退出 0，0 警告 | 4.1s |
| diff | `git diff --check` | 退出 0，无空白错误（仅 Git LF/CRLF 提示） | 0.05s |

原始日志（gitignored）：`target/g01-release-refresh/<stage>.log`，汇总 `summary.json`。

## 三、release 与 debug 对比

| 项 | debug（9bd3489f…） | release（f7c4de41…） |
| --- | --- | --- |
| native | 27 过 | 27 过 |
| host-msvc | 709 过 | 709 过 |
| g01 | 7 过 | 7 过 |
| native-g01 | 2 过 | 2 过 |
| native-only-g01 | 1 过 | 1 过 |
| resident | 102 过 | 102 过 |
| all-probes lib | 124 过 0 败 | 124 过 0 败 |
| all-probes 中止原因 | mass_solve_probe 3 项已知缺口 | **相同** |
| 未执行阶段补跑 | 六阶段通过 | 六阶段通过 |

**未发现 release 专属的新失败**（没有 release 优化/精度相关的新问题）；差异仅在耗时（release 编译/测试整体约 7 分钟）。

## 四、边界与声明

- release 不能宣称 G01 全量通过：all-probes 在 3 项既有准入缺口处失败；
- 未重复运行 debug 已完成的全脚本，未改动测试、容差、fixtures、脚本或产品源码；
- 未宣称 release 精度/性能验收或双平台准入通过；
- Linux 仍未启动；
- all-probes 的 3 项缺口保持原状，等待用户对 G03 路线的裁决。
