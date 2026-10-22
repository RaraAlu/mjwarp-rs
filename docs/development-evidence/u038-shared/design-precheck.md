# U038 共享椭球与同代状态：设计预检

任务：`2026-10-16-next-u038-shared`。
日期：2026-10-22。
范围：共享椭球路径独立验证、关节/肌腱/常驻状态同代可核、保留失效阻断与故障恢复、原容差不动。
状态：预检发现同代校验需要任务范围外的 `src/physics/resident`（及 A1 生命周期写入接线）；已停手，未改产品源码，等待范围重登记。

## 验收与实现映射（预检结论）

| 验收项 | 预检结论 | 所需改动 |
| --- | --- | --- |
| 共享椭球路径独立验证 | 可在当前范围实现：`fluid_force/tests/reference.rs` 已含独立椭球参考，`passive_force/tests/fluid.rs` 现有共享测试只覆盖 box 输入（`fluid_input` 的 `body_fluid_ellipsoid` 全 false） | 在 `src/physics/passive_force/tests/` 增加共享椭球 GPU 测试：共享计划输入椭球 `FluidForceInput`，先跑无流体基线，再验证共享组装、流体系数与冻结顺序；比较用独立参考，不复用候选输出 |
| 关节、肌腱及常驻状态同代可核 | **受阻**：`JointPassiveData` 只有 `input: Arc<Input>`、`worlds` 与 `ready`，没有来源代际；`TendonVelocityView` 只核对源指针；`KinematicsData.lifecycle()` 的 `state_version`/`metadata_generation` 在产品路径从未递增（grep 仅在 `model::data::lifecycle` 定义与单测出现），A1-4 只登记所有权、未接线 | 需要 resident 写路径递增代际并暴露只读代际；joint/tendon 写入记录代际；`PassiveForcePlan::update` 校验三方一致并在不一致时拒绝 |
| 保留失效阻断与故障恢复 | 可在当前范围实现：在共享更新前做同代校验，配套故障注入测试（改关节副本/改源后更新拒绝，重建后恢复） | `src/physics/passive_force/tests/` 新测试 |
| 原容差不动 | 不新增容差；独立参考比较沿用原 2e-6 与位型对照口径 | 不改容差 |

## 同代校验的阻塞点（精确）

1. 代际源不存在：`DataLifecycle` 的 `state_version`（`src/model/data/lifecycle.rs`）是私有字段，`state_written()` 从未在 `src/physics` 产品路径调用；`KinematicsData::lifecycle()` 只读。
2. 关节/肌腱副本没有代际字段：`JointPassiveData`（`src/physics/smooth_force/joint_passive.rs:168`）与 `TendonVelocityView`（`src/physics/tendon_velocity`) 均无来源状态版本；`PassiveForcePlan::update`（`src/physics/passive_force/update.rs:9`）只核对指针身份、世界数与就绪态，这正是 `docs/fluid-force.md` 记录的“关节状态仍需外部同步、代码不核对副本值一致、完整同代状态仍未实现”。
3. 精确内容校验同样受阻：`KinematicsData` 的 qpos/qvel 设备缓冲与其访问器是 `pub(super)`/私有（`src/physics/resident/layout.rs`、`data.rs`），`readback()` 只暴露派生输出快照，不暴露原始 qpos/qvel；在 `passive_force` 内无法读取源 qpos/qvel 做逐位副本比较。
4. 因此“同代可核”无法在不改 `src/physics/resident`（以及 A1 生命周期写入路径）的前提下实现；用纯测试或裸标签代替不满足“别用测试够用代替”的要求。

## 建议的范围重登记

在现有范围（`src/physics/passive_force`、`src/physics/fluid_force`、`src/physics/smooth_force`、`src/physics/tendon_velocity`）之外增加：

```text
src/physics/resident
src/model/data
```

模块增加：`mjwarp.physics.resident`、`mjwarp.model.data`（按实际模块 id）。
验收语义不变，仍只关闭本节点；不改容差、不宣称 NP。

实现顺序（重登记后）：

1. resident：在 `write_qpos`/`write_qvel`/世界写路径递增 `DataLifecycle::state_written`，提供只读 `SourceStateVersion`（或等价的 `state_version` 访问）。
2. joint_passive/tendon_velocity：写入 API 记录来源版本；`TendonVelocityView` 暴露数据版本。
3. passive_force：`update` 校验关节/肌腱/常驻三方版本一致，不一致返回可定位错误；不改变原容差。
4. 测试：共享椭球独立参考 GPU 测试；同代错误拒绝与恢复测试；保留既有失效阻断/故障恢复测试。
5. 证据：`docs/development-evidence/u038-shared/` 记录独立参考、同代拒绝面与未验收边界。

## 本轮无产品改动

本轮只做设计预检与范围登记（`ccdc853`、`3604dac`），未改 `src/`，未运行 GPU 测试，未声明任何验收通过。
