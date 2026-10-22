# A1 生命周期代际接线证据

任务：`2026-10-22-layers-a1-lifecycle-wiring`
基线：`75664b4`（设计预检提交）
设计预检：`design-precheck.md`（同目录）

## 变更

| 文件 | 变更 |
| --- | --- |
| `src/model/data/versions.rs`（新） | `StateField`（qpos/qvel/mocap/sleep）与 `StateFieldVersions` 只读版本快照；版本递增与溢出拒绝 |
| `src/model/data/lifecycle.rs` | `DataLifecycle` 增加字段版本；新增 `record_state_write`、`state_field_versions`、`record_metadata_write`；模块与类型文档写明语义 |
| `src/model/data/mod.rs` | 重导出 `StateField`、`StateFieldVersions` |
| `src/physics/resident/data.rs` | 四个状态写路径成功后调用 `record_state_write`；新增只读访问器 `state_version`、`state_field_versions`、`metadata_generation`；`KinematicsSnapshot` 访问器移出 |
| `src/physics/resident/snapshot.rs`（新） | 承接 `KinematicsSnapshot` 只读访问器 |
| `src/physics/resident.rs` | 注册 `snapshot` 子模块；导入字段版本类型 |
| `src/physics/resident/tests/reads.rs` | 新增 GPU 定向测试：成功写递增、失败写不递增、四字段独立版本、身份与元数据代际不变量 |
| `tests/layer_data.rs` | 新增宿主契约测试：字段版本递增、元数据代际陈旧拒绝 |
| `docs/module-contracts.md` | A1.4 生命周期契约补充字段版本与代际不变量 |

## 语义

- 状态写入：`write_qpos`/`write_world_qpos` → qpos；`write_qvel`/`write_world_qvel` → qvel；`write_mocap`/`write_world_mocap` → mocap；`write_world_sleep` → sleep。
- 每次成功写入同时递增 `state_version` 与对应字段版本；先校验两个计数器再一次性写入，失败不递增。
- 失败路径（长度不符、非有限值、维度错误、特性未启用）在任何计数器变动前返回。
- 元数据：`record_metadata_write` 递增 `metadata_generation`，旧代际 `check_metadata_generation` 返回 `StaleMetadata`。常驻元数据缓冲在计划构造期上传、运行期不可改写，因此运行期 `metadata_generation()` 保持构造代际；该不变量写入生命周期契约与模块说明。
- 身份：构造期绑定且不可变；`check_identity`/`check_session` 的拒绝语义不变。

## 验证

- `cargo fmt --check`：通过（门禁 fmt 1159ms）。
- `cargo test --locked`：通过（门禁 unit 398 过 0 败，16449ms）。
- `cargo clippy --all-targets --locked -- -D warnings`：通过（3225ms）。
- `cargo clippy --all-targets --locked --features cuda-probe -- -D warnings`：通过（8517ms）。
- `cargo test --locked --features cuda-probe --lib lifecycle_generations_advance_only_on_successful_state_writes -- --ignored --test-threads=1`：1 过 0 败（1.20s，RTX 4070 Ti SUPER / 驱动 596.36）。
- `scripts/test-windows-resident-kinematics.ps1 -AllFeatures`：退出码 0，全部子项通过（RTX 4070 Ti SUPER / 驱动 596.36 / rustc 1.99.0）；共享参考对比 1245 世界，最大绝对误差 7.99e-6（容差 2e-5）；参数对比 3102 世界，最大绝对误差 1.144e-5；报告见 `resident-report.json`（原始目录 `target/resident-kinematics/3dc0561e3f924a96ade20141e28e5b18`）。
- 门禁：`normify gate` 记录 fmt/clippy/clippy-cuda/unit 全过。

## 未验收边界

- 本任务只提供来源代际与只读访问器；关节/肌腱记录来源版本、`passive_force` 同代拒绝与故障恢复在下游 `2026-10-22-next-u038-shared-2` 实现。
- 不改原生 ABI、内核字符串/PTX、数值容差、fixtures 与 Cargo 依赖；NP01–NP04 状态不变。
- `src/physics/resident` 产品源码变化使 T4 固定源码身份过期，由 `2026-10-22-t4-source-hash-repin-4` 重钉后再宣称矩阵全绿。
- 未重跑 Linux 验收（无 Linux 机器）。
