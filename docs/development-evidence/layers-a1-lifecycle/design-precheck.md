# A1 生命周期代际接线设计预检

任务：`2026-10-22-layers-a1-lifecycle-wiring`
基线：`c76665c`（预检期间 HEAD 只增加任务登记提交，产品源码未变）
上游依据：`docs/development-evidence/u038-shared/design-precheck.md`；W23 评审 `docs/development-evidence/layers-w23.md`（A1-4 只登记未接线、两套失效表示待统一）。

## 1. 现状证据

### 1.1 生命周期类型已存在，但产品路径不递增

- `src/model/data/lifecycle.rs:133` 定义 `DataLifecycle`，含 `state_version`、`derived_version`、`metadata_generation`；`state_written()`（179）、`metadata_written()`（198）、`check_metadata_generation()`（227）只有定义与单测使用。
- `src/physics/resident/plan_data.rs:139` 在 `create_data` 构造生命周期记录；`src/physics/resident/data.rs:18` 只读访问器 `lifecycle()`。
- `grep -n "state_written\|metadata_written" src/` 仅命中 `model/data/lifecycle.rs` 与 `tests/layer_data.rs`；`src/physics` 产品路径从未调用，即预检结论中的"代际源不存在"。
- 身份在 `create_data` 绑定：`ModelIdentity::of_model` + `LayoutIdentity::of_parts`（`plan_data.rs:139-150`）；`check_data`（254-258）按模型指针拒绝异计划。

### 1.2 常驻状态写路径

| 写入口 | 位置 | 设备缓冲 | 现有失效动作 |
| --- | --- | --- | --- |
| `write_qpos` / `write_world_qpos` | `data.rs:163/174` | `state` 前 `nq` 段 | `ready = ReadyStages::default()`（189） |
| `write_mocap` / `write_world_mocap` | `data.rs:207/220` | `state` 的 mocap 段（`plan_data.rs:108-128`） | `ready = default`（239） |
| `write_qvel` / `write_world_qvel` | `data.rs:71/75` | `edges.qvel`（`plan_data.rs:194-205`），与 `state` 分离 | `ready.edges = false`（108） |
| `write_world_sleep` | `data.rs:123` | `sleep.state` | `ready.sleep = false`（144） |

失败路径先校验长度/有限值/维度后写设备（例如 `write_checked_qvel` 87-107、`write_qpos` 163-170），错误时保留既有设备结果。

### 1.3 元数据代际

- 权威写入代际为 `MetadataRevision`（`src/runtime/launcher/checked.rs:25-47`），证明 `Checked` 在 `validate_for` 复核代际并返回 `StaleMetadata`（133-145）。
- 常驻元数据缓冲在计划构造期上传一次：`com_metadata_buffer.rs:42-70`、`attached_metadata_buffer.rs`、`rigid_kinematics/mod.rs:192-244`、`sleep_copy.rs:292-331` 等；`grep metadata.write_range` 的产品路径写入不存在，运行期改写只出现在 `#[cfg(test)]` 辅助。
- 结论：resident 运行期没有元数据写入点，`metadata_generation` 只能提供"构造代际 + 写入递增 API + 陈旧拒绝语义"，不在本任务伪造运行期递增。

## 2. 设计决定

### 2.1 新增共享契约（`src/model/data/versions.rs`，新文件）

- `StateField`：`Qpos`、`Qvel`、`Mocap`、`Sleep`。
- `StateFieldVersions`：四个 `u64` 字段版本，只读 `get`/`qpos`/`qvel`/`mocap`/`sleep`；内部 `next`/`set` 供 `DataLifecycle` 原子更新。
- 由 `model::data` 重导出；`DataLifecycle` 增加 `field_versions: StateFieldVersions`。
- `DataLifecycle::record_state_write(field)`：先做两个计数器的溢出检查，再一次性写入，返回新的 `state_version`；失败不改变任何计数器。
- `DataLifecycle::state_field_versions()` 只读返回快照。
- `DataLifecycle::record_metadata_write()` = 既有 `metadata_written()` 的语义入口（保留旧方法，兼容单测）。
- 生命周期文档（模块文档、`lifecycle.md`、`docs/module-contracts.md` A1.4）写明：状态写使派生失效并递增字段版本；元数据写使证明代际失效；身份不可变，跨身份/跨会话拒绝。

### 2.2 常驻接线（`src/physics/resident/data.rs`）

- `write_checked_qpos` → `record_state_write(Qpos)`；`write_checked_qvel` → `Qvel`；`write_checked_mocap` → `Mocap`；`write_world_sleep` → `Sleep`。均在设备写入成功后调用；`#[cfg(not(feature = "cuda-probe"))]` 错误分支不递增。
- `ReadyStages` 语义不变，不新增/不移除阶段旗帜。
- 新增只读访问器：`state_version()`、`state_field_versions()`、`metadata_generation()`、`data_identity()`（已有，保持）。全部 `&self`，不新增可变借用。
- `data.rs` 已 493 行：把 `impl KinematicsSnapshot`（450-493）移入新文件 `src/physics/resident/snapshot.rs`，`data.rs` 保持 <500；两个新文件都 <500。

### 2.3 验证与范围

- 宿主单测：`src/model/data/versions.rs`/`lifecycle.rs` 覆盖各字段递增、失败不递增、溢出、跨身份、跨会话；`tests/layer_data.rs` 增加契约级断言（不申请设备内存）。
- GPU 定向：`src/physics/resident/tests/reads.rs` 增加 1 个 `#[ignore]` 测试，验证 `create_data` 后为 0、四种写入分别递增、失败写入不递增、世界写路径递增；用 `scripts/test-windows-resident-kinematics.ps1 -AllFeatures` 跑既有 resident 定向回归（0 失败），新测试单独 `cargo test --features cuda-probe --lib lifecycle_generations -- --ignored` 执行。
- 门禁：fmt/clippy/clippy-cuda/unit + unchanged（fixtures/native/include/Cargo.*/rust-toolchain）+ json 回执 + forbid_claim。
- 精度映射：全部改动为宿主侧 u64 计数与只读访问器；不碰内核字符串/PTX/ABI/设备类型、不改 f32 产品路径、不新增依赖，NP01–NP04 状态不变。
- 已知漂移：`src/physics/resident` 产品源码变化会使 T4 固定源码身份（resident 组合哈希、precision 夹具、gravity 内联 contract）过期；按 Chief 指示落地后立即执行 `2026-10-22-t4-source-hash-repin-4` 才宣称矩阵全绿。

## 3. 未验收边界

- 本任务只提供来源代际与只读访问器；关节/肌腱记录来源版本、`passive_force` 同代拒绝与故障恢复在下游 `2026-10-22-next-u038-shared-2` 实现。
- resident 运行期无元数据写入点：`metadata_generation` 运行期保持构造代际，`record_metadata_write` 的递增语义由宿主单测覆盖，不在产品路径伪造递增。
- 不改 `ReadyStages` 与派生就绪语义；`derived_version` 不在本任务发布。
- T4 源码身份在 repin-4 完成前保持过期，不宣称矩阵全绿。
