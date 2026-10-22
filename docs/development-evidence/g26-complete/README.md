# G26 能量全链与上游 energy_vel 参考

任务：`2026-10-16-next-g26-complete`（重登记提交 `1acdbe3`；任务在模块拆分后按实际目录重新登记范围，语义与验收未放宽）。
执行日期：2026-10-23；环境：Windows x86_64 MSVC、RTX 4070 Ti SUPER、驱动 596.36、CUDA 12.8.1。

## 结论

1. **全链调度已接通**：`physics::schedule_energy_chain` 在一次调用内完成常驻位置→CRB（含 M）→宿主 qvel 写入乘积缓冲→M·qvel→默认32线程动能，全程无宿主物理回传。
2. **状态标签来源已核实**：快照携带 A1 `SourceState`（数据身份 + 状态代际 + 字段版本）；`SourceState::token()` 固定 FNV-1a 派生可复算标签；`KineticEnergyVector::from_source` / `KineticEnergyPlan::new_from_source` 取代调用方自造整数标签（旧整数入口保留兼容）。位置更新前后来源代际不一致时整链以 `StageNotReady` 拒绝。
3. **独立同条件参考已补齐**：离线 authoring 生成冻结上游 MJWarp `energy_vel`（上游 `71da24d…`）参考，9 个场景 × 8 状态 × 2 个 f32 qvel 向量 = 144 条记录；天然校验（原生 MuJoCo 3.15 f64）全部通过。
4. **产品对比**：定向 GPU 全链测试 468 个能量槽 **0 失败**，最坏归一化误差 **9.99e-3**（< 1），Sparse/Dense 两种乘积格式与 worlds 1/5 均覆盖。

## 目标模块设计预检（启动阶段）

- 拆分后布局：动能子集在 `src/physics/kinetic_energy/`（plan/data/checks/metadata/buffer/kernels/cuda/tests），设备动能接线在 `src/physics/mass_matrix/multiply/borrowed/device_energy/`，CRB/M 生产者在 `src/physics/mass_matrix/resident.rs`。
- 现状缺口：`MassMatrixEnergyPlan::update` 需要调用方自行驱动 CRB 生产、乘积与能量三段；快照不携带来源；`KineticEnergyVector.state_id` 是调用方整数标签；无上游 `energy_vel` 同条件参考与非默认范围/NP 证据。
- 方案：新增独立 `schedule.rs` 整链自由函数（不引入自引用计划，避免生命周期重构）；`MassMatrixData::source_state()` 暴露只读来源；快照增加 `SourceState`；标签派生复用 A1 `SourceState::token()`；上游参考走已验证离线 authoring 框架（pinned 环境 + 上游 frozen checkout），产物进静态 fixture 与证据，不进产品链路。
- 风险：动 `src/physics/mod.rs`、`src/physics/mass_matrix.rs` 等导出会漂移 T4 源码身份（历史审计断言），需另立 repin（本任务只登记，不伪造历史）。
- 不改容差、断言、产品 ABI、门禁语义；不新增 CUDA 模板字符串，不改冻结内核正文。

## 上游参考与天然校验

生成器：`scripts/authoring/author-g26-energy-vel.ps1` → `author-g26-energy-vel.py`（离线证据工具、非产品链）。

- pinned 环境：Python 3.12.13 / mujoco 3.15.0 / warp-lang 1.15.0 / numpy 2.5.4 / mujoco-warp 3.15.0；启动前复核上游 HEAD=`71da24d956378a87a703b6e1442b13aec0c4ac29` 且 clean、75+ 安装 .py 与上游逐文件一致、warp.dll 与 pinned RECORD 一致。
- 场景（9 个）：`mixed-joints`(nv11)、`inertial-tree`(nv13)、`rotated-tree`(nv12)、`zero-dof`(nv0)、`massless-tree`(nv0)、`armature-chain`(nv3) 的质量矩阵夹具模型（qpos 取 `fixtures/mass-matrix` 的 8 个冻结 case），以及合成串行铰链 `synthetic-chain-{3,33,65}`（对齐既有 device-energy 单测场景，覆盖 32 线程块尾部/非默认 nv）。
- 每个状态两个 f32 精确向量：`qvel_j = ((3j + 5*state + 7*vector) % 21) * 0.125 - 1.25`；上游以 `fwd_position`（CRB 与 M）→`mul_m`→`energy_vel` 计算 `0.5 q^T M q`，两次运行逐位一致。
- 天然校验：同一宿主模型用原生 MuJoCo 3.15 f64（`mj_forward` + `mj_mulM`）复算，144 条全部在 ABS=REL=2e-5 内；各场景最坏归一化误差：mixed-joints 1.87e-2、inertial-tree 7.94e-3、rotated-tree 1.48e-2、armature-chain 1.32e-2、synthetic-3 2.43e-3、synthetic-33 6.57e-3、synthetic-65 2.12e-2；零自由度记录为 0。
- 与冻结夹具矩阵交叉：真实模型用 `fixtures/mass-matrix` cases 的 f64 `matrix` 复算能量，归一化误差与原生复算一致（最坏 1.87e-2）。
- 产物：
  - 产品静态夹具 `fixtures/kinetic-energy/upstream-energy-vel.json`（9 场景、每场景 16 条：qpos/qvel/energy 位串、稀疏 CSR 结构、上游提交与容差）；`fixtures/kinetic-energy/manifest.json` 含生成器 SHA256 与文件哈希。
  - 证据 `upstream-energy-vel.json`、`natural-validation.json`、`provenance.json`（版本、上游提交、输入 SHA256、脚本 SHA256、GPU/驱动、确定性）。

## 实现点

| 文件 | 变更 |
| --- | --- |
| `src/physics/mass_matrix/multiply/borrowed/device_energy/schedule.rs`（新） | `schedule_energy_chain`：CRB→M→M·qvel→动能整链；位置更新前后来源代际一致校验；返回携带来源的快照 |
| `src/physics/mass_matrix/multiply/borrowed/device_energy/plan.rs` | 快照新增 `source: SourceState` 与 `source_state()`；快照生成时取真实来源 |
| `src/physics/mass_matrix/resident.rs` | `MassMatrixData::source_state()` 只读访问器 |
| `src/model/data/versions.rs` | `SourceState::token()`（固定 FNV-1a 派生）+ 单测 |
| `src/physics/kinetic_energy.rs` / `plan.rs` | `KineticEnergyVector::from_source`、`KineticEnergyPlan::new_from_source`（旧整数标签入口不动） |
| `src/physics/{mod,mass_matrix,mass_matrix/multiply}*.rs` | 新入口再导出 |
| `tests/g26_energy_chain.rs`（新）+ `tests/support/g26_energy.rs`（新） | 静态夹具完整性测试（非 ignored，校验 manifest 哈希与 144 条记录结构）+ 定向 GPU 全链对比测试（ignored） |
| `docs/kinetic-energy.md`、`docs/device-energy.md` | G26 小节与入口说明 |
| `.normify/gates.toml` | 新增定向 GPU 门禁 `g26-energy` |

未改：容差、断言语义、产品 ABI、冻结 CUDA 模板字符串、旧测试选择器与旧失败记录、`native/` 与 `Cargo.*`。

## 验证结果

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 夹具完整性（宿主） | `cargo +msvc test --locked --all-features --test g26_energy_chain freezes_static_upstream_energy_reference_and_manifest_hashes` | 1 过 0 败 |
| 定向 GPU 全链 | `cargo +msvc test --locked --all-features --test g26_energy_chain -- --ignored --test-threads=1` | **1 过 0 败**；`scalars=468`、`failed_blocks=0`、`worst_normalized=9.985e-3`、elapsed 7253 ms；报告 `energy-chain-report.json` |
| 默认特性全量 | `cargo +msvc test --locked --no-fail-fast` | **406 过 0 败**（79 个结果行） |
| 全特性全量 | `cargo +msvc test --locked --all-features --no-fail-fast` | 701 过 0 败 347 ignored，但 **11 项 T4 历史源码身份审计失败**（详见下节，属预期 identity 漂移） |

GPU 测试覆盖：9 场景 × 2 格式 × 16 条 = 288 次全链调度（含 `mixed-joints`、`synthetic-chain-33` 的 worlds=5 复跑），零自由度模型在乘积/动能布局处显式拒绝（边界，不静默回退）。

## 已知债务与边界

- **T4 源码身份漂移**：`src/physics/mod.rs`、`src/physics/mass_matrix.rs` 等导出变化使 T4 历史审计记录的源码摘要过期，`--all-features` 下 11 项失败（`recapture::tests_host::preserves_current_sources_and_all_thirty_variants`、`g01` field/rigid/state-packing/topology 审计、`t4_tendon_wake_precision` 等）。按规范另立 T4 源码身份重钉任务处理；**不伪造历史摘要、不改审计断言**。重钉完成前不宣称 T4 矩阵全绿。
- 不宣称完整 U047/G02 传感器调度、完整 NP01–NP04 放行、Linux 与性能；零自由度能量未定义（上游算子按 nv 静态展开，产品全链显式拒绝）。
- 上游参考是「同条件同算子」参考，不是完整上游 forward；产品对比容差为冻结 ABS=REL=2e-5，未放宽。

## 证据索引

- `README.md`（本文件）：预检、做法、结果、边界。
- `provenance.json`：pinned 环境、上游提交/clean、输入与脚本 SHA256、GPU/驱动、两次运行一致。
- `upstream-energy-vel.json`：上游参考完整记录（与产品夹具同内容）。
- `natural-validation.json`：144 条上游 vs 原生 f64 校验明细。
- `energy-chain-report.json`：产品全链 vs 上游对比（槽数、失败、最坏归一化、最坏记录、耗时）。
- `authoring-run.log`：离线 authoring 运行日志（gitignored 源的副本）。
