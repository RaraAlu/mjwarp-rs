# T4常驻源码锚点迁移

任务：`2026-10-21-t4-resident-source-anchors`。
基线提交：`7b13ce8`（父提交），平台 Rara，Windows x64 MSVC。
本任务只迁移测试源码锚点，不改产品行为。

## 结论

八处 `include_str!` 锚点已逐项核对。
七处 support 锚点改为「模块根加相关子模块」组合。
`tests/launcher_cache.rs` 的内核文件清单补齐三个子模块。
产品源码、fixtures、`docs/evidence` 与其他测试文件均未改动。
源码身份哈希已过期，重钉由 `2026-10-22-t4-source-hash-repin` 承接。

## 锚点核对

| 文件 | 原锚点语义 | 迁移 |
| --- | --- | --- |
| `tests/support/t4_field_internal.rs` | `RESIDENT`/`PHYSICS` 供 `FIELD_BATCH_CUDA` 与路由/函数体解析 | 改为 `include!` 注入组合锚点 |
| `tests/support/t4_packing_routes.rs` | `RESIDENT` 供 `pack_*` 函数体与 `cuda()` 模板解析；第一模块改用父模块 `RESIDENT` | 两处 `include!` 注入组合锚点，`source()` 对 `src/physics/resident.rs` 返回组合文本 |
| `tests/support/t4_rigid_capture.rs` | `RESIDENT` 供 `FIELD_BATCH_CUDA`；`PHYSICS` 供 `KINEMATICS_CUDA`/`COM_POSITION_CUDA`；`MASS_SOLVE` 供 `MASS_SOLVE_CUDA` | 三处改为组合锚点；`template()` 的 `unwrap` 改为带锚点名与证据路径的 `expect` 风格 `panic` |
| `tests/support/t4_rigid_internal.rs` | `RESIDENT`/`PHYSICS` 供函数体与内核模板解析 | 组合锚点 |
| `tests/support/t4_state_packing.rs` | `RESIDENT` 供打包结构体解析；`src/physics/mod.rs` 条目参与身份守卫 | 组合锚点 |
| `tests/support/t4_topology_frames.rs` | `RESIDENT` 供 `pack_model`/`pack_camlight` 函数体；`PHYSICS` 参与身份守卫 | 组合锚点 |
| `tests/support/t4_zero_quaternion.rs` | 测试内联读取 `PHYSICS`/`RESIDENT` 取内核前缀 | 组合锚点 |
| `tests/launcher_cache.rs` | `PHYSICS_FILES` 收集定义 L1 内核常量的物理文件 | 清单 22 项补到 25 项，见下 |

`tests/launcher_cache.rs` 补齐的三项：

- `src/physics/fluid_force/cuda.rs`（内核常量已从模块根迁入）。
- `src/physics/mass_solve/cuda.rs`（同上）。
- `src/physics/resident/pack_batches.rs`（`FIELD_BATCH_CUDA` 所在子模块）。

候选内核集合、`L1_DIGESTS` 匹配与断言均未改动。

## 组合口径

组合顺序固定为：模块根在前，随后按模块根 `mod` 声明顺序排列产品子模块。
文件之间以单个换行分隔。
组合不含 `#[cfg(test)]` 的 `tests` 子模块，口径为完整产品源码。

组合表达式存放在本目录：

- `resident.rs`：模块根加 16 个产品子模块。
- `physics.rs`：模块根加 4 个产品子模块。
- `mass-solve.rs`：模块根加 `cuda` 子模块，只服务 `MASS_SOLVE_CUDA` 解析。

测试通过 `include!` 注入表达式，不新增 `tests/` 文件，也不增长已超 500 行的文件。
组合值由下列脚本口径计算，供身份重钉任务引用。
摘要按 CRLF→LF 归一口径计算，与测试 `digest()` 一致。

## 组合 sha256

| 源码路径 | 口径 | 文件数 | 组合 sha256 |
|---|---|---|---|
| src/physics/resident.rs | 常驻运动学 | 17 | 919335fb11995e35dc834e8e3be8eb2d2ea6443afb5108b1d6e967e9d5c5aeaf |
| src/physics/mod.rs | 物理模块根 | 5 | 09da2b796e08fddabd93eba5c99833cadf1e1d0ebb32f46c730910a61dd6fe09 |
| src/physics/mass_solve.rs | 质量求解（内核解析与发射点锚点） | 3 | 63b1d4729e2e7f4b72f434ae95032362aa8553271a73da448d6689db528c3267 |

## 参与文件行数

| 文件 | 行数 |
|---|---|
| src/physics/resident.rs | 315 |
| src/physics/resident/checks.rs | 175 |
| src/physics/resident/data.rs | 483 |
| src/physics/resident/layout.rs | 118 |
| src/physics/resident/output.rs | 69 |
| src/physics/resident/pack.rs | 344 |
| src/physics/resident/pack_batches.rs | 220 |
| src/physics/resident/plan_accessors.rs | 105 |
| src/physics/resident/plan_constructors.rs | 340 |
| src/physics/resident/plan_contexts.rs | 157 |
| src/physics/resident/plan_data.rs | 256 |
| src/physics/resident/plan_subsets.rs | 470 |
| src/physics/resident/plan_update_flex.rs | 287 |
| src/physics/resident/plan_update_forward.rs | 231 |
| src/physics/resident/plan_update_rigid.rs | 127 |
| src/physics/resident/public.rs | 93 |
| src/physics/resident/sleep_copy.rs | 459 |
| src/physics/mod.rs | 331 |
| src/physics/com_position.rs | 247 |
| src/physics/device_checks.rs | 37 |
| src/physics/history.rs | 90 |
| src/physics/kinematics.rs | 342 |
| src/physics/mass_solve.rs | 143 |
| src/physics/mass_solve/kernel.rs | 100 |
| src/physics/mass_solve/cuda.rs | 120 |

## template() 诊断

`tests/support/t4_rigid_capture.rs` 的 `template()` 不再用裸 `unwrap`。
缺少锚点时报出锚点名与证据路径，模板未闭合时报出同一提示。
失败信息不改变断言与容差。

## 验证

| 编号 | 内容 | 结果 |
| --- | --- | --- |
| run-01 | `cargo test --locked --features cuda-probe --test t4_input_precision` | 锚点解析全部通过；8 项仍报旧源码身份漂移 |
| run-02 | `cargo test --locked --features cuda-probe --lib -- recapture` | 1 过 1 败；失败项为旧源码身份漂移（`t4-current-kernels.json` 的 `mod.rs` 摘要） |
| run-03 | `cargo test --locked --features cuda-probe --test launcher_cache` | 2 过 0 败（4 项 GPU 忽略） |
| check-04 | `cargo fmt --check` | 通过 |
| check-05 | `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings` | 通过 |
| check-06 | `normify gate` fmt/clippy-cuda/unit | 见门禁记录 |

## 未验收项

- 身份锚点 `product_sources.sha256` 未重钉，交由 `2026-10-22-t4-source-hash-repin`。
- 该重钉任务完成前，G01 全量回归预期为红。
- 未改动 `fixtures/`、`docs/evidence/`、`docs/development-evidence/changes/` 之外的证据文件。
- 未跑 G01、未宣称 NP01 至 NP04、精度与性能。

## 偏差记录

- 组合锚点表达式放在任务文件范围内的证据目录，由测试 `include!` 注入。
- 未新增 `tests/` 支撑文件，也未触碰范围外文件；等价于「模块根加子模块组合」要求。

## 身份重钉阶段的锚点修订

`2026-10-22-t4-source-hash-repin` 在重钉源码身份时补齐了拆分布局带来的锚点差异：

- 恢复拆分时丢失的两行纯注释（`// Descriptors hold …`、`// Wide pointer arithmetic …`），
  位置在 `src/physics/resident/pack_batches.rs` 的 `FIELD_BATCH_CUDA` 之前；不改任何代码。
- `src/physics/mass_solve.rs` 组合补入 `kernel.rs`，覆盖内核模板与其发射路由文本。
- 三个 precision 测试（camlight、spatial-tendon、tendon-wake）对拆分路径读取组合文本，
  不再直接读取模块根。
- `tests/support/t4_rigid_capture.rs` 的路由锚点随拆分从 `super::` 改为 `crate::physics::`。
- `tests/support/t4_rigid_internal.rs` 的历史整源码摘要按组合文本重钉。
- `tests/support/t4_state_packing.rs` 允许设备字段带 `pub(super) ` 前缀，仍逐字核对字段名与类型。
- `tests/t4_gravity_nonzero_product_chain.rs` 对拆分路径使用组合文本；该文件行数净不增。

所有断言、容差、映射条目保持不变。

