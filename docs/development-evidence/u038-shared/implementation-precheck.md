# U038 共享椭球与同代状态实现预检

任务：`2026-10-22-next-u038-shared-2`（重登记）
基线：`516784f`（前置 `2026-10-22-layers-a1-lifecycle-wiring` `deeb259` + `2026-10-22-t4-source-hash-repin-4` 已完成）
上游预检：`design-precheck.md`（同目录，冻结验收与受阻点）
验收（与原任务逐字一致）：独立验证共享椭球路径；关节、肌腱及常驻状态同代可核；保留失效阻断、故障恢复与原容差。

## 1. 现状（基线事实）

- resident 已提供来源代际：`KinematicsData::state_version()`、`state_field_versions()`（qpos/qvel/mocap/sleep，A1 接线 `deeb259`），以及 `data_identity()` 与只读 `lifecycle()`。
- `JointPassiveData`（`src/physics/smooth_force/joint_passive/data.rs`）只持自有 qpos/qvel 副本，`write_qpos`/`write_qvel` 不记录来源代际；`TendonVelocityData::write_qvel`（`src/physics/tendon_velocity/data.rs`）同样只写历史 qvel 副本。
- `PassiveForcePlan::update`（`src/physics/passive_force/update.rs`）只核对：计划身份、世界数、肌腱/柔体/重力/流体共享源指针、`tendon_velocity.source_data()` 指针；**不核对副本代际**，调用方须自行同步（`docs/fluid-force.md` 已记录的缺口）。
- 共享椭球：`src/physics/fluid_force/tests/reference.rs` 已有独立椭球参考（宿主），但共享被动链测试只覆盖 box 输入（`body_fluid_ellipsoid` 全 false）。

## 2. 设计

### 2.1 来源代际快照（`src/model/data/versions.rs` 扩展）

新增 `SourceState`（只读、Copy）：
- `state_version: u64`、`fields: StateFieldVersions`、`identity: DataIdentity`。
- 由 `KinematicsData` 侧构造：`KinematicsData::source_state()` 返回当前快照（`data_identity` + `state_version` + `state_field_versions`）。
- 相等判定 `matches(&self, other) -> bool`（身份 + state_version）；字段版本用于按字段核对（qpos/qvel）。

### 2.2 生产者记录来源（joint_passive / tendon_velocity）

- `JointPassiveData` 增加 `source: Option<SourceState>`：
  - 新方法 `write_qpos_from(&mut self, source: &KinematicsData, values: &[f32])`、`write_qvel_from`、`write_world_*_from`：先核对源计划身份/世界数，写入成功后记录 `source.source_state()`（快照含本次写入对应的 qpos 或 qvel 字段版本）。
  - 既有 `write_qpos`/`write_qvel` 保持兼容：写入成功后清空 `source`（无来源声明，下游按未声明处理）。
  - 只读访问器 `source_state() -> Option<SourceState>`。
- `TendonVelocityData` 同样增加 `write_qvel_from(&mut self, source: &KinematicsData, values)` 与 `source_state()`；既有 `write_qvel` 清空声明。`TendonVelocityPlan::update(source, data)` 成功时记录来源快照（它读取 resident 当前 qvel），并在读取前核对声明与当前源一致；不一致返回 `StageNotReady { stage: "tendon_velocity_source_generation" }`。
- `JointPassivePlan::update_joint` 不额外接收 resident 源；代际核对放在 `PassiveForcePlan::update`（唯一同时持有三方来源的入口）。

### 2.3 同代拒绝（`PassiveForcePlan::update`）

在现有指针/世界数检查之后、任何设备写入之前追加：
- 取 `flex_source.source_state()` 为当前代际；
- `joint.source_state()` 若为 `Some`：要求 identity 匹配且 `qpos`/`qvel` 字段版本与当前源一致；否则拒绝 `StageNotReady { stage: "passive_force_joint_generation" }`；
- `tendon_velocity.data` 声明同理：要求 qvel 字段版本一致，否则 `StageNotReady { stage: "passive_force_tendon_generation" }`；
- 未声明来源（旧调用路径）保持原有行为（不新增拒绝面），兼容既有测试与公开路径。
- 拒绝先于 `data.ready = false` 与任何发射；恢复路径：重新以 `*_from` 写入当前源后再次 `update` 即可（故障恢复测试覆盖）。

### 2.4 共享椭球独立验证

- 复用 `fluid_force/tests/reference.rs` 的独立椭球几何参考（宿主 f64 自由参考，不反馈产品结果），构造含 `body_fluid_ellipsoid` 的共享被动链场景：
  - 独立参考测试（宿主）：椭球体积/浮力中心/力矩的几何量用解析公式核对，不调用产品内核；
  - GPU 定向测试：共享被动链在椭球输入下与独立参考比对（原容差、不改 fixtures），并配一对 box 输入回归；
  - 同代测试：先记录来源 → 正常 `update` 通过；改写 resident qpos（不重写副本）→ `update` 拒绝；重写副本（当前源）→ 通过；
  - 失效阻断：跨世界数/跨计划/跨会话既有拒绝面回归；故障恢复：拒绝对设备无写入、重同步后结果恢复。

## 3. 精度映射与约束

- 产品物理字段保持 f32；共享椭球公式来自冻结流体内核，不改内核字符串/PTX。
- 独立参考用 f64 宿主公式，仅作诊断，不反馈产品物理结果（NP 规则）。
- 容差、fixtures 与既有断言语义不变；不新增依赖；不引入 Python；新文件 <500 行，超长文件净增行数受控。
- 编译边界：无 `cuda-probe` 时空实现仍返回 `FeatureDisabled`；`SourceState` 在 `model::data` 提供，不依赖 runtime 特性。

## 4. 验证计划

1. `cargo test --locked`：新增宿主单测（SourceState 相等/字段版本、`*_from` 身份与长度错误路径、未声明兼容路径）。
2. GPU 定向（RTX 4070 Ti SUPER/596.36，每次 NVRTC 独立空目录）：
   - `cargo test --locked --features cuda-probe --lib passive_force_shared`（同代拒绝/恢复、box 回归）；
   - 新增椭球独立参考测试（`--lib fluid_force_ellipsoid` 或既有共享测试目标）；
   - 既有 `resident_*`/`passive_*` 定向回归 0 失败。
3. 门禁：fmt/clippy/clippy-cuda/unit + unchanged + json 回执 + forbid_claim。
4. 回执写清真实测试证据与未验收边界。

## 5. 未验收边界

- 本任务不重跑 G01 全量脚本、不跑 Linux/release；不宣称 NP01–NP04/精度/性能。
- 旧调用路径不声明来源，保持不拒绝；同代可核只覆盖新 `*_from` 路径（文档与回执写明）。
- 共享椭球只验证流体共享链的椭球几何与容差内数值一致性，不宣称完整流体物理验收。
