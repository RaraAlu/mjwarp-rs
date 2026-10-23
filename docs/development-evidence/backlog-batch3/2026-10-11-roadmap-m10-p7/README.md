# M10@P7 Rust 回调契约切片：类型、时序、状态与同步契约

任务：`2026-10-11-roadmap-m10-p7`。日期：2026-10-23。执行：Backend（批次 3）。
矩阵：M10 Rust 回调（`类型回调与内核钩子` / `时序、状态与同步契约` / P7）。

## 1. 阶段语义与细化口径

按 Chief 边界，M10@P7 只交**可独立成立的回调契约部分**：冻结上游
`mujoco_warp._src.types.Callback` 的七个槽位、各槽位的阶段/写入字段/门控/状态前提，
以及上游 `forward`/`step` 中的调用时序与同步契约，并给出**不执行**的调用计划。
本片**不是**阶段回调：不实现 `forward`/`step`/`step1`/`step2` 调度与回调执行，
不用宿主回调桩冒充阶段回调。

数值等价对本片不适用（无物理数值计算），故改为两级冻结参考：
① 上游 dataclass 字段顺序/默认值与 `types.Stage` 数值（authoring 导出的
`fixtures/callback/contract.json`）；② 上游调用点 `file:line` 映射（下表）。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/callback.rs` | `CallbackKind`（7 槽位）、`CallbackContract`（阶段/写入字段/门控/USER 前提）、`Callback` 登记表与 `plan`（不执行调用计划）、`CallbackSyncContract`、`CallbackError` |
| `src/physics/callback/tests.rs` | 冻结字段/Stage 对照、时序计划、ACTUATION 门控、sensor 三阶段、登记错误面、写入契约表、同步契约 |
| `src/physics/mod.rs` | `pub use callback::{…}` 导出（既有模块行为不变） |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 上游参考 | MJWarp `71da24d956…`（Warp 1.15.0）：`types.Callback`、`types.Stage`、`forward.py`/`passive.py`/`sensor.py`/`collision_driver.py` 调用点 |
| 生成脚本 | `scripts/authoring/author-m10-callback.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/callback/contract.json`、本目录 `authoring.json` |
| 冻结内容 | `callback_fields=["passive","control","act_dyn","act_gain","act_bias","sensor","contactfilter"]`、默认值全 None、`Stage.POS/VEL/ACC = 1/2/3`（与 MuJoCo `mjtStage` 交叉核对） |

### 3.1 槽位 → 上游调用点/时序/状态映射

| 槽位 | 上游调用点 | 阶段 | 写入 `Data` 字段 | 门控/状态前提 |
| --- | --- | --- | --- | --- |
| `contactfilter` | `collision_driver.py:983-984`（`fwd_position` 内） | Collision(0) | `contact` | 无 |
| `sensor`（POS） | `sensor.py:963-964`（`sensor_pos`，`Stage.POS`） | SensorPos(1) | `sensordata` | 每次 forward 触发 |
| `passive` | `passive.py:3695-3696`（`forward.py:1439` 的 `fwd_velocity`） | Passive(2) | `qfrc_passive` | 无 |
| `sensor`（VEL） | `sensor.py:1509-1510`（`sensor_vel`，`Stage.VEL`） | SensorVel(3) | `sensordata` | 每次 forward 触发 |
| `control` | `forward.py:2080-2082`（`forward`；`step1/step2` 同序） | Control(4) | `ctrl` | `DisableBit.ACTUATION` 未置位 |
| `act_dyn` | `forward.py:1916-1917`（`_actuator_force`） | Actuation(5) | `act_dot` | 执行器 `dyntype=USER`（`DynType.USER`） |
| `act_gain` | `forward.py:1918-1919`（`_actuator_force`） | Actuation(5) | `actuator_force` | 执行器 `gaintype=USER`（`GainType.USER`） |
| `act_bias` | `forward.py:1920-1921`（`_actuator_force`） | Actuation(5) | `actuator_force` | 执行器 `biastype=USER`（`BiasType.USER`） |
| `sensor`（ACC） | `sensor.py:2847-2848`（`sensor_acc`，`Stage.ACC`） | SensorAcc(6) | `sensordata` | `solver.solve` 之后触发 |

时序依据：`forward.py:2060-2095`（`fwd_position` → `sensor_pos` → `fwd_velocity` →
`sensor_vel` → control → `fwd_actuation` → `fwd_acceleration` → `solver.solve` →
`sensor_acc`）；两阶段 API 同序见 `forward.py:2120-2150`。`types.Stage` 定义见
`types.py:241-247`。`inverse.py:182/200` 的 sensor 调用不在本片范围。

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 槽位字段顺序/默认值 | Rust `CallbackKind::ALL` 与冻结上游 `dataclasses.fields(Callback)` 逐项一致；7 个默认值均为 None |
| `Stage` 数值 | `SensorPos/SensorVel/SensorAcc` = **1/2/3**，与冻结 `types.Stage` 及 MuJoCo `mjtStage` 一致 |
| 调用时序计划 | 全槽位安装时计划 = contactfilter(Collision) → sensor(POS) → passive → sensor(VEL) → control → act_dyn → act_gain → act_bias → sensor(ACC)，阶段严格非降序 |
| 门控 | `CallbackGates{actuation_enabled=false}` 时 control 调用被跳过；sensor 仍三次触发 |
| 状态契约 | 7 槽位写入字段/门控/USER 类型前提与上游 docstring 与调用点一致 |
| 同步契约 | 前/后均为流序要求、`allows_host_readback=false`（禁止静默回退 CPU）；host 读回须调用方显式完成事件 |

复跑：`cargo test --locked --lib callback` → **7 过 0 败**；全量 `cargo test --locked`
通过（门禁记录为准）；`cargo fmt --check` 与 `cargo clippy --all-targets --locked -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：不适用——本片为契约切片，不含物理数值路径，无 f32/f64 精度映射；
  未引入任何数值计算或精度提升。
- **NP02**：记录冻结上游提交、`types.Callback`/`Stage` 字段与数值、调用点
  `file:line`、环境与 fixture 生成脚本。
- **NP03**：不适用——无物理数值对照，改用「冻结上游 dataclass 字段/Stage 数值 +
  调用点映射」两级契约对照（如实登记为契约等价，不冒充数值等价）。
- **NP04**：不适用——不发射内核、不做性能测量（`resources` 为空）。

## 6. 未验收边界

- 不实现 `forward`/`step`/`step1`/`step2` 阶段调度，不执行任何回调；本片不是阶段回调。
- 不实现回调钩子函数本体、设备内核、状态租约与同步原语接线（A08 的
  lease/completion 契约仅作同步语义引用，不在本片接线）。
- 不覆盖 `inverse.py` 路径的回调调用、sensor 延迟/interval 与 contactfilter 的
  全量运行语义、执行器 USER 类型的完整数值路径、GPU 内核与图执行。
- 不宣称完整 M10 或 P7 验收；契约等价不等于产品数值验收。

## 7. 复现

```powershell
scripts/authoring/author-m10-callback.ps1
cargo test --locked --lib callback -- --nocapture
```
