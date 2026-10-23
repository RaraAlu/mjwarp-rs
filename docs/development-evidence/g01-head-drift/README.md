# G01 当前 HEAD 双模式失败记录：T4 冻结捕获源组合漂移

日期：2026-10-23。执行：QA。任务：`2026-10-23-g01-head-drift-record`。
性质：**失败记录与归因，只记录不修**；未改断言、夹具、排除清单、产品与验收文档。

## 1. 结论（先给）

在最新 HEAD `b3aa8542c607b10e7708d8cf697d2d2b4ff7a132` 上重跑
`scripts/test-windows-g01.ps1 -AllFeatures -FullRegression`（debug）与 `... -Release`（release）：

- 两模式均在 **host-msvc 阶段**（`cargo test --locked --all-features`）失败；
  脚本按设计中止，`g01`/`native-g01`/`native-only-g01`/`resident`/`all-probes`/`host-gnu`/
  clippy/rustdoc/diff 阶段**未执行**，因此本 HEAD 上的 all-probes 数字与 11 项排除清单未复核。
- 失败为 `T4 冻结捕获身份审计`的**预期源码漂移**（新增模块合法改动 `src/physics/*`），
  不是产品数值回归：默认特性全量 `cargo test --locked` 在同 HEAD **480 过 0 败**；
  `--all-features --no-fail-fast` 枚举全仓库**仅此 1 条失败**。
- 本记录**不声明 G01 双模式通过**；恢复方式（重基线 pin 或显式排除）待用户裁决。

## 2. 运行与阶段数字

| 项 | debug | release |
| --- | --- | --- |
| 命令 | `scripts/test-windows-g01.ps1 -AllFeatures -FullRegression` | `... -AllFeatures -FullRegression -Release` |
| HEAD | `b3aa854`（2026-10-23 15:49:06 +0800） | 同左 |
| 运行目录 | `target/g01-acceptance/975f40f3f588433ba9fdfb05fc56ee85` | `target/g01-acceptance/0f65aed62f8b47f291fb2a5ede0eb1a9` |
| 本机日志 | `target/final-g01-check/debug.log` | `target/final-g01-check/release.log` |
| 运行时间 | 22:24:24–22:25:50 | 22:29:08–22:29:42 |
| fmt | 通过 | 通过 |
| native | 27 过 0 败 | 27 过 0 败 |
| host-msvc | **536 过 / 1 败 / 129 ignored** | **536 过 / 1 败 / 129 ignored** |
| 后续阶段 | 未执行（脚本中止于 host-msvc） | 未执行（脚本中止于 host-msvc） |

## 3. 失败 target 与 trace

- target：lib 单测（`--all-features`；debug 与 release 相同）
- 测试：`runtime::transfer::kernel::recapture::tests_host::preserves_current_sources_and_all_thirty_variants`
- 位置：`tests/support/t4_rigid_capture/tests_host.rs:10`（`lineage(&value).unwrap()`）
- 错误：`Err("unexpected lineage change")`
- 该测试断言：当前产品源码组合（`docs/development-evidence/t4-resident-anchors/physics.rs` 的
  `concat!`：`src/physics/{mod,com_position,device_checks,history,kinematics}.rs`）与冻结捕获 inventory
  `fixtures/precision/t4-current-kernels.json` 中登记的 `src/physics/mod.rs` 摘要一致。

## 4. 漂移数据

| 项 | 值 |
| --- | --- |
| 冻结 inventory 登记摘要 | `7eb6e7a3c49ea2812f9b7b2ac579d9ced635d7c790613723297ba33d2718079d` |
| HEAD 实测源组合 sha256 | `d7f6c7d19a17307d281f6e43c9866d0da67bfcbe9af4af3bad305d77ee3a187c` |
| inventory 重钉提交 | `08e7983f10f3f715657b68436e457e8518d3a2db`（2026-10-23 02:19:09，重钉 G26 后 T4 源码身份摘要） |
| 最后一次匹配的 HEAD | `6e8c79aae0d2d1ff83b14f8732d4060bf952b490`（2026-10-23 12:20:20；实测组合 = 冻结值，该 HEAD 上 G01 双模式全绿） |
| 首个破坏提交 | `70b36b7fa358302e02f900782525a479553e3258`（2026-10-23 13:15:09，G27@P2 交付） |
| 受影响文件 | `src/physics/mod.rs`、`src/physics/com_position.rs`、`src/physics/device_checks.rs`、`src/physics/history.rs`、`src/physics/kinematics.rs` |

`6e8c79a..HEAD` 触碰上述组合文件的提交（16 个，按时间）：

| 提交 | 时间 | 主题 |
| --- | --- | --- |
| `70b36b7` | 13:15 | G27@P2 固定常量 set_const_fixed |
| `9850ee0` | 13:28 | G05@P4 执行器力 fwd_actuation |
| `8764ced` | 13:34 | G28@P7 原语射线查询 ray/rays |
| `a1fa5b5` | 13:42 | G27@P7 qpos0 依赖常量与动态重算 |
| `e7a7859` | 13:52 | G05@P7 关节传动 transmission |
| `00b8b60` | 13:55 | G28@P8 多世界批量射线查询 |
| `12151dc` | 14:10 | G07@P5 宽相碰撞候选生成 |
| `9692c92` | 14:18 | G16@P6 Euler 与 RK4 状态推进 |
| `a10aea5` | 14:22 | G20@P7 逆向动力学 qfrc_inverse |
| `cef3357` | 14:26 | G21@P7 平滑速度导数 |
| `34bbd05` | 14:30 | G23@P7 传感器 |
| `2269b04` | 14:49 | G08@P5 基本几何窄相 |
| `9239fa5` | 14:53 | G18@P6 离散积分 |
| `9cdf5de` | 15:07 | G24@P7 历史与延迟 |
| `b9cd236` | 15:23 | M10@P7 Rust 回调契约 |
| `c757e3d` | 15:34 | G09@P5 plane-box 多接触 |

## 5. 归因与不声明

- 归因：该审计测试的前提是「当前产品源码与冻结捕获身份一致」；批次 1–4 的交付在
  `src/physics/mod.rs` 注册新模块、G24 还改动 `history.rs`，属**合法交付导致的预期源码漂移**。
  测试本身的语义未变，失败是冻结 pin 过期，不是产品数值回归。
- 非回归佐证（同 HEAD 实测）：
  - 默认特性 `cargo test --locked`：**480 过 0 败**；
  - `cargo test --locked --all-features --no-fail-fast`：全仓库仅上述 1 条失败，其余测试二进制全过；
  - `cargo fmt --check` / `cargo clippy --all-targets --locked -- -D warnings` 通过。
- 不声明：本 HEAD 上 G01 双模式验收不成立；all-probes/排除清单未在本 HEAD 复核
  （上次全绿为 `6e8c79a`：debug `fff94fdd2e7c4e489362480b369c454b`、release
  `f0bb413b3de5423d91a82484c8a9d5de`，all-probes 346/0、11 项文档化排除——仅对那个 HEAD 有效）。
- 恢复方式（待用户裁决，本任务不实施）：① 跑 T4 历史捕获重基线，重钉
  `fixtures/precision/t4-current-kernels.json` 与源组合锚点；② 或按既有先例把该测试纳入
  文档化排除（需改排除清单/脚本与验收文档，须批准）。

## 6. 边界

- 只新增本证据目录与回执；未改断言、夹具、排除清单、产品、脚本与 `docs/three-layer-acceptance.md`。
- 未修改 `fixtures/precision/t4-current-kernels.json` 与 `docs/development-evidence/t4-resident-anchors/physics.rs`。
- 工作区在重跑前后均为干净；本记录不构成任何 G01 通过声明。
