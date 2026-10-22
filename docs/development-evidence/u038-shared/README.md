# U038 共享椭球与同代状态证据

任务：`2026-10-22-next-u038-shared-2`（重登记，验收与原任务逐字一致）
基线：`75c659a`（实现预检提交；前置 `deeb259` A1 代际接线、`516784f` T4 源码重钉）
预检：`design-precheck.md`、`implementation-precheck.md`（同目录）

## 变更

| 文件 | 变更 |
| --- | --- |
| `src/model/data/versions.rs` | 新增 `SourceState`（身份 + 状态代际 + 字段版本）与 `same_identity`/`same_field_generation`；宿主单测覆盖字段代际比较与异身份拒绝 |
| `src/model/data/mod.rs` | 重导出 `SourceState` |
| `src/physics/resident/data.rs` | `KinematicsData::source_state()` 只读来源快照 |
| `src/physics/smooth_force/joint_passive.rs` / `data.rs` / `plan.rs` | `JointPassiveData` 增加来源声明；新增 `write_qpos_from`/`write_qvel_from`/`write_world_*_from` 记录来源；旧 `write_*` 成功后清空声明；`source_state()` 只读访问 |
| `src/physics/tendon_velocity.rs` / `data.rs` / `plan.rs` / `view.rs` | `TendonVelocityData` 增加输入声明与已发布代际；`write_qvel_from`/`write_world_qvel_from`；`update` 入口核对声明与当前源 qvel 代际，成功后发布当前源快照；旧 `write_*` 清空两个声明 |
| `src/physics/passive_force.rs` / `data.rs` / `update.rs` | 新增 `check_shared_source_generation`：关节副本 qpos/qvel 与肌腱输出 qvel 必须与常驻源一致，不一致分阶段返回 `StageNotReady`，先于任何设备写入；旧路径（`None`）保持兼容不拒绝 |
| `src/physics/passive_force/tests/fluid.rs` | 共享链 GPU 测试改用 `*_from` 声明来源，报告 `full_chain_same_generation=true`；文件行数保持 480 未越限 |
| `src/physics/passive_force/tests/generation.rs`（新，已登记模块源码） | 同代拒绝/重同步恢复/旧入口兼容 GPU 测试：关节副本过期拒绝、肌腱历史声明过期拒绝、重同步恢复、旧 `write_*` 不声明不拒绝；报告 `shared-generation-report.json` |
| `src/physics/fluid_force/tests/gpu_ellipsoid.rs` | 新增共享常驻椭球 GPU 测试：`FluidForcePlan::from_resident` 输出对 `tests/reference.rs` 独立参考，容差 2e-6 不变 |
| `docs/fluid-force.md` | 更新共享椭球实测与同代可核边界 |

## 语义

- 声明来源：`*_from` 写入成功后记录 `source_state()`（身份 + state_version + 字段版本），写入失败不记录。
- 兼容：旧 `write_*` 入口保持签名与行为，成功后清空来源声明，不新增拒绝面；旧路径仍由调用方同步。
- 同代核对：`PassiveForcePlan::update` 在指针/世界数检查之后、`data.ready=false` 与任何发射之前核对；不一致返回 `StageNotReady { stage: "passive_force_joint_generation" }` 或 `"passive_force_tendon_generation"`；上一代已发布结果保持原样，不产生部分更新；以当前源重写副本后同一次更新恢复。
- 肌腱：`TendonVelocityPlan::update` 入口核对输入声明与当前源 qvel 代际，成功后把当前源快照记为已发布代际。
- 容差、fixtures、内核字符串/PTX 与产品物理公式不变。

## 验证（rustc 1.99.0 MSVC，RTX 4070 Ti SUPER/596.36，CUDA 12.8.1，MuJoCo 3.12.0）

| 编号 | 内容 | 结果 |
| --- | --- | --- |
| host-01 | `SourceState` 字段代际与异身份单测 | 2 过 0 败 |
| host-02 | 同代拒绝/重同步/旧入口兼容单测 | 4 过 0 败 |
| gpu-01 | `gpu_wires_resident_fluid_into_shared_passive_order` | 1 过 0 败（6.05s）；报告 `passive-fluid-report.json`：`full_chain_same_generation=true` |
| gpu-02 | `gpu_shared_resident_ellipsoid_matches_independent_reference` | 1 过 0 败（1.80s）；9 世界/288 槽，最大绝对误差 4.77e-7（容差 2e-6），报告 `shared-ellipsoid-report.json` |
| gpu-03 | `gpu_shared_source_generation_rejects_stale_copies_and_recovers` | 1 过 0 败（1.31s）；报告 `shared-generation-report.json`：关节代际拒绝、肌腱输入代际拒绝、重同步恢复、旧入口兼容全为 true |
| gpu-04 | 模块定向：`passive_force::`、`fluid_force::`、`tendon_velocity::`、`joint_passive::` | 全部 0 败 |
| gpu-05 | `physics::resident::tests_` 22 过 0 败（25.25s） | 通过 |
| gate | fmt/clippy/clippy-cuda/unit | 见 `normify gate` 记录 |

## 未验收边界

- 旧 `write_*` 入口保持兼容但不做代际核对；同代可核只覆盖声明来源的 `*_from` 路径（文档与回执写明）。
- 共享椭球只验证椭球几何与流体分量在冻结容差内一致，不宣称完整流体物理或完整 G04/U038 验收。
- 未跑 G01 全量脚本、Linux 与 release；未宣称 NP01–NP04、精度、性能验收。
- 不修改内核字符串/PTX、数值容差与 fixtures。
