# 超长文件拆分记录

## 结论

- 61 个超过 500 行的代码文件已全部拆分，61 个切片任务与收口任务均已验收。
- 重扫 675 个受检代码文件（共 113985 行），当前最大单文件 491 行，超过 500 行的文件为 0。
- 拆分是纯移动：公开 API、ABI 布局、CUDA 内核字符串、f32 精度路径、容差与测试断言均未改动。
- 未验收项与债务见文末；不得据本文宣称 G01 全量、NP01 至 NP04、GPU 性能、Linux 或 release 验收。

## 拆分口径

- 阈值：500 行。拆分后每个文件目标不超过 480 行，留出格式化余量。
- 原文件保留为模块根，只留 `mod` 声明、共享类型定义与转导出；实现移入同名子目录。
- 三个 `mod.rs`（`src/physics/mod.rs`、`src/runtime/mod.rs`、`src/diagnostics/mod.rs`）按特例处理：子模块落在父目录的独立文件（如 `src/runtime/config.rs`），不新建 `mod/` 目录，保证模块路径与语义不变。
- 没有改动产品行为，没有放宽断言或容差，没有新增 Clippy 豁免。
- src 切片逐个用 `normify module source <模块id> --add <路径>` 登记新源码；tests 文件按项目约定不登记模块源码。

## 扫描方法与结果

扫描脚本：`docs/development-evidence/split-program/scan-oversized.ps1`。

- 受检扩展名：`rs`、`cu`、`c`、`h`、`cpp`、`sh`、`ps1`。
- 排除目录：`docs/`（证据）、`fixtures/`（样本）、`target/`（构建产物）、`.git/`。
- 输出：`docs/development-evidence/split-program/report.json`，`/over_limit_files` 为 0。

本次扫描（HEAD `8aac086` 之后、收口提交之前）：

| 指标 | 数值 |
| --- | --- |
| 受检文件数 | 675 |
| 总行数 | 113985 |
| 最大单文件 | 491 |
| 最大文件路径 | `tests/resident_fixed_tendon.rs` |
| 超过 500 行文件数 | 0 |

`tests/resident_fixed_tendon.rs` 不在 61 个拆分清单内，拆分前即低于 500 行。

## 拆分清单

行数为物理行（`Measure-Object -Line` 口径）。"拆分后根"指保留为模块根的原文件行数；"子模块"指该片提交新增的 `.rs` 文件数与其中最大行数。

| # | 文件 | 拆分前 | 拆分后根 | 子模块 | 最大子文件 |
| --- | --- | --- | --- | --- | --- |
| 1 | `src/physics/rne.rs` | 3903 | 172 | 15 | 391 |
| 2 | `src/physics/applied_force.rs` | 2682 | 274 | 11 | 415 |
| 3 | `src/physics/smooth_force.rs` | 2653 | 207 | 13 | 468 |
| 4 | `src/physics/mass_solve/sparse.rs` | 2636 | 184 | 9 | 436 |
| 5 | `src/physics/passive_force.rs` | 2555 | 134 | 9 | 474 |
| 6 | `src/physics/mass_solve/sparse/mixed/solve.rs` | 2392 | 114 | 17 | 227 |
| 7 | `src/physics/mass_solve/sparse/mixed/solve/resident.rs` | 2033 | 69 | 14 | 277 |
| 8 | `src/physics/fluid_force.rs` | 1956 | 168 | 14 | 365 |
| 9 | `src/physics/mass_solve.rs` | 1829 | 137 | 16 | 266 |
| 10 | `tests/t4_gravity_upstream_abi.rs` | 1715 | 50 | 9 | 460 |
| 11 | `src/runtime/transfer/kernel.rs` | 1684 | 166 | 11 | 440 |
| 12 | `src/physics/kinetic_energy.rs` | 1542 | 139 | 12 | 287 |
| 13 | `src/physics/mass_solve/sparse/dense/small.rs` | 1528 | 90 | 13 | 212 |
| 14 | `src/runtime/warp_kinematics.rs` | 1477 | 407 | 8 | 399 |
| 15 | `src/physics/rne_bias.rs` | 1449 | 264 | 10 | 251 |
| 16 | `src/physics/mass_solve/sparse/dense.rs` | 1434 | 143 | 10 | 270 |
| 17 | `src/physics/smooth_force/joint_passive.rs` | 1427 | 274 | 8 | 302 |
| 18 | `src/physics/mod.rs` | 1330 | 326 | 5 | 330 |
| 19 | `src/physics/mass_matrix.rs` | 1248 | 266 | 6 | 331 |
| 20 | `tests/support/t4_field_internal.rs` | 1186 | 158 | 6 | 292 |
| 21 | `src/physics/tendon_velocity.rs` | 1184 | 184 | 10 | 223 |
| 22 | `tests/support/t4_packing_routes.rs` | 1149 | 7 | 4 | 349 |
| 23 | `src/physics/mass_matrix/multiply/borrowed/device_energy.rs` | 1095 | 29 | 6 | 332 |
| 24 | `src/runtime/launcher/artifact.rs` | 1001 | 240 | 4 | 397 |
| 25 | `src/physics/gravity_compensation.rs` | 922 | 228 | 3 | 345 |
| 26 | `src/physics/mass_solve/sparse/mixed.rs` | 897 | 127 | 5 | 277 |
| 27 | `src/physics/mass_matrix/multiply/borrowed.rs` | 883 | 123 | 6 | 341 |
| 28 | `tests/g01.rs` | 872 | 20 | 8 | 194 |
| 29 | `src/runtime/cuda/external.rs` | 869 | 404 | 4 | 187 |
| 30 | `src/physics/com_velocity.rs` | 842 | 151 | 6 | 212 |
| 31 | `src/model/flex.rs` | 818 | 107 | 5 | 180 |
| 32 | `src/runtime/cuda.rs` | 806 | 241 | 4 | 218 |
| 33 | `tests/support/t4_rigid_capture.rs` | 799 | 111 | 5 | 286 |
| 34 | `src/runtime/warp_com.rs` | 773 | 158 | 5 | 223 |
| 35 | `src/physics/mass_matrix/multiply.rs` | 768 | 148 | 4 | 325 |
| 36 | `tests/t4_gravity_nonzero_product_chain.rs` | 757 | 339 | 2 | 318 |
| 37 | `tests/launcher_abi.rs` | 745 | 140 | 5 | 269 |
| 38 | `tests/resident_spatial_tendon.rs` | 685 | 212 | 4 | 154 |
| 39 | `src/runtime/warp_gravity.rs` | 680 | 156 | 6 | 155 |
| 40 | `src/runtime/mod.rs` | 676 | 101 | 4 | 202 |
| 41 | `tests/resident_camlight.rs` | 672 | 124 | 5 | 184 |
| 42 | `src/runtime/cuda/resources.rs` | 665 | 13 | 6 | 275 |
| 43 | `src/runtime/transfer.rs` | 656 | 81 | 3 | 211 |
| 44 | `tests/support/t4_rigid_internal.rs` | 656 | 81 | 6 | 201 |
| 45 | `tests/support/t4_zero_quaternion.rs` | 640 | 376 | 1 | 262 |
| 46 | `tests/t4_input_precision.rs` | 631 | 183 | 4 | 164 |
| 47 | `tests/support/t4_topology_frames.rs` | 626 | 208 | 5 | 132 |
| 48 | `src/runtime/cache.rs` | 605 | 252 | 1 | 317 |
| 49 | `tests/mass_solve_probe.rs` | 599 | 75 | 4 | 202 |
| 50 | `tests/resident_geom_tendon.rs` | 591 | 64 | 4 | 228 |
| 51 | `src/physics/mass_solve/sparse/resident/solve.rs` | 589 | 86 | 4 | 166 |
| 52 | `tests/support/t4_state_packing.rs` | 582 | 117 | 4 | 250 |
| 53 | `tests/resident_flex_edge.rs` | 559 | 200 | 5 | 103 |
| 54 | `src/physics/flex_edge_passive.rs` | 553 | 180 | 4 | 265 |
| 55 | `tests/support/t4_evidence.rs` | 550 | 290 | 3 | 144 |
| 56 | `src/physics/mass_solve/resident.rs` | 538 | 103 | 3 | 191 |
| 57 | `tests/support/t4_face_operators.rs` | 537 | 38 | 4 | 253 |
| 58 | `src/physics/tendon_passive.rs` | 528 | 166 | 4 | 249 |
| 59 | `src/model/camlight.rs` | 523 | 153 | 3 | 205 |
| 60 | `src/runtime/launcher/abi.rs` | 506 | 262 | 4 | 81 |
| 61 | `src/diagnostics/mod.rs` | 505 | 9 | 4 | 165 |

汇总：

| 指标 | 数值 |
| --- | --- |
| 拆分文件数 | 61（src 42、tests 19） |
| 拆分前总行数 | 68191 |
| 拆分后模块根合计 | 10029 |
| 新增子模块文件 | 397（合计 59075 行） |
| 拆分后总行数（根+子模块） | 69104 |
| 结构开销（模块声明、文件头、转导出） | 913 |
| 拆分后模块根最大行数 | 407（`src/runtime/warp_kinematics.rs`） |
| 拆分后新增子文件最大行数 | 474（`src/physics/passive_force.rs` 组内） |

## 模块源码登记

拆分前后 `.normify/modules/` 的 `[[source]]` 条目对比：

| 指标 | 数值 |
| --- | --- |
| 拆分前登记条目 | 125 |
| 当前登记条目 | 434 |
| 新增 | 309 |
| 移除 | 0 |

- 42 个 src 切片提交新增 309 个 `.rs`，与新增登记数一致，没有漏登。
- 19 个 tests 切片提交新增 88 个 `.rs`；tests 文件按项目约定不登记模块源码。
- `normify validate` 0 错误。

登记一致性检查（434 条登记 vs `src/` 下 433 个 `.rs`）：

- `src/physics/rne_bias.rs`（264 行模块根）未登记：拆分前同样未登记，属既有登记缺口，不是本次拆分引入。
- `src/physics/rne.cu`、`src/runtime/probe.ptx` 已登记但文件不存在：基线提交即如此，属历史预绑定条目。

## 验证

- 每片：`normify gate <切片id>`（fmt、clippy-cuda、unit 三项门禁）→ `normify check` → `normify done` → 提交，61 片逐片通过。
- 收口：`normify gate 2026-10-21-split-program-reconcile` 覆盖 fmt、clippy-cuda、unit。
- 全量测试：`cargo test --locked --features cuda-probe --no-fail-fast` 无失败目标（lib 311 过、120 ignored）。
- 拆分后 T4 源码身份哈希由 `2026-10-22-t4-source-hash-repin` 与 `2026-10-22-t4-source-hash-repin-2` 两轮重钉，锚点迁移由 `2026-10-21-t4-resident-source-anchors` 完成，三者均已验收。

## 未完成项与债务

### 开放项一：源码锚点分层

拆分后 T4 证据链的源码身份锚点从「单文件全文」改为「模块根 + 产品子模块组合」拼接。组合口径逐用例定义在 `tests/support/` 的锚点模块与迁移后的 `tests/support/t4_source_anchors.rs` 模板里。

- 现状：拆分后两轮重钉已让锚点与哈希一致，相关目标测试通过。
- 未完项：新增或移动子模块时必须同步更新锚点组合与模板；锚点分层尚未收敛成单一清单来源，仍有散落在多个 support 模块中的组合定义。

### 开放项二：可见性收敛

拆分让原本同文件的私有项跨子模块可见，为此做了等价放宽：`pub(super)` 放宽到 `pub(in crate::physics)`、`pub(in crate::runtime::cuda)` 或 `pub(crate)`；只放宽可见性，没有改动实现。

- 未完项：放宽项尚未逐项收敛到最小可见范围，也还没有可见性审计用例。

### 开放项三：fixtures 契约

`fixtures/precision/*.json`、`fixtures/gravity-upstream-abi/nonzero-product-chain-package.json` 与 `docs/evidence/t4-g03-retest/*/map.json` 内嵌源码身份哈希，与源码文本强绑定。

- 现状：两轮重钉后哈希与当前源码一致；拆分收口本身未改动 `fixtures/`。
- 未完项：任何后续对已锚定源码的编辑或格式化都会再次造成漂移，需要重钉流程；夹具哈希与源码的组合口径缺少自动化漂移告警，目前靠目标测试发现。

### 开放项四：G01 债务

G01 全量回归仍不通过，属既有债务，与本拆分程序无因果关系：

- gravity 冻结位证书 2 bit 差异：在基线 worktree（提交 `360718f`）上同样失败，判定为既有问题，登记 `2026-10-22-g01-numerics-audit`（proposed）。
- 报告环境变量与 PTX 捕获目录双语义冲突：`MJWARP_PHYSICS_PTX_DIR` 同时承担 PTX 捕获输出与报告路径两种语义。
- 门禁脚本历史过期：已由 `2026-10-22-g01-gate-script-sync`（已验收）同步 adapter/内侧/主 G01 的精确计数与标记，并给 all-probes 段补齐临时报告路径与 capture 目录环境变量；all-probes 剩余红项与根因登记为既有债务。
- 历史失败数据：两轮全量 G01 耗时 261s / 182s，显存峰值 5532→5756 MiB、5521→5772 MiB。

## 不得宣称的验收

拆分与收口只证明：文件行数达标、既有测试目标在拆分后仍通过、源码身份锚点已重钉。以下均未验收：

- 完整 G01 全量回归（含上面四项债务）。
- NP01 至 NP04 精度验收。
- GPU 性能、Linux 平台、release 配置验收。

## 证据

- 扫描脚本与结果：`docs/development-evidence/split-program/scan-oversized.ps1`、`docs/development-evidence/split-program/report.json`。
- 收口运行记录：`docs/development-evidence/split-program/README.md`。
- 任务变更记录：`.normify/changes/2026-10-21-split-*.json`、`.normify/changes/2026-10-21-split-program-reconcile.json`。
