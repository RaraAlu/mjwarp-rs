# G05@P4 执行器阶段切片：fwd_actuation 执行器力

任务：`2026-10-11-roadmap-g05-p4`。日期：2026-10-23。执行：Backend（批次 1）。
矩阵：G05 执行器（`增益、偏置、动态、传动` / `限幅、激活状态、控制输入` / P4/P7）。

## 1. 阶段语义与细化口径

原图只有 G05@P4 粗切片，未给具体函数。按当前代码布局，本片取
架构计划 API `mjwarp_rs::physics::forward::fwd_actuation`（U017；G05；P4/P7）
的**宿主侧 joint 传动子集**：ctrl 限幅、dyntype none/integrator/filter/filterexact
的激活状态推进与限幅、gaintype fixed/affine、biastype none/affine、力限幅，
输出 `act_dot`、`actuator_force` 与按 CSR moment 累加的 `qfrc_actuator`。

P4 阶段只覆盖「执行器力」侧：transmission（U068，G06）与执行器模型字段
转换不在本片，`actuator_length`/`actuator_velocity`/moment CSR 作为**冻结输入**；
不发射 GPU 内核、不实现完整 forward。验收口径未放宽：矩阵核心验收
「限幅、激活状态、控制输入」逐项交付并有冻结上游逐位对照；muscle/dcmotor/pid/user
与 tendon/site/body 传动显式拒绝并登记为边界（§6）。

## 2. 实现（`src/physics/actuator.rs`）

- `ActuatorModel`：dyntype/gaintype/biastype/ctrladr/ctrlnum/actadr/actnum、
  dynprm/gainprm/biasprm（每执行器 10 槽）、actrange/forcerange/ctrlrange（2 槽）、
  actlimited/actearly/forcelimited/ctrllimited、timestep、disable_clampctrl。
- `ActuationData`：世界批量 ctrl/act、冻结 actuator_length/actuator_velocity、
  moment CSR（rownnz/rowadr/colind/moment）。
- 计算顺序对齐上游 `_actuator_force`：ctrl 提取与限幅 → 激活分支（`act_dot`
  与 `next_act` 提前激活/限幅）→ gain/bias → force 限幅 → 逐执行器按 moment
  行累加 `qfrc_actuator`。
- 类型常量对齐 `mjtDyn/mjtGain/mjtBias`；不支持的取值返回
  `ActuationError::Unsupported`（不静默回退）。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/actuation/model.xml`（sha256 `f6d484da…4b078`；nq=2、nv=2、nu=3、na=1、nactuator=3、dt=0.002） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_fwdPosition`→`mj_fwdVelocity`→`mj_fwdActuation`（NP03 一级） |
| 上游参考 | MJWarp 冻结提交 `71da24d956378a87a703b6e1442b13aec0c4ac29`，Warp 1.15.0，`transmission`→`fwd_velocity`→`fwd_actuation`（NP03 二级） |
| 生成脚本 | `scripts/authoring/author-g05-actuation.ps1` + `.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/actuation/actuation-cases.json`、本目录 `authoring.json` |
| 初始状态 | nworld=2；ctrl `[[0.4,-0.6,0.9],[2.0,0.3,-1.5]]`、act `[[0.5],[-0.7]]`（第二世界 ctrl 触发限幅） |

执行器覆盖：`servo`（fixed 增益 + affine 偏置 + ctrl/force 限幅）、
`motor`（fixed 增益、none 偏置）、`filtered`（affine 增益 + filter 动态 +
actearly + act 限幅）；moment 宽度 3。

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游 | `act_dot`(2)、`actuator_force`(6)、`qfrc_actuator`(4) **逐元素位型一致**（容差 0） |
| 产品 vs 原生 f64 一级判定 | max abs **9.54e-08** ≤ 冻结 1e-05 |
| 限幅路径 | ctrl=2.0 被限到 1.0；force 限幅在 [-5,5] 内生效；act 限幅 [-2,2] 生效 |
| 显式拒绝 | dyntype≥4 / gaintype≥2 / biastype≥2 / actnum≠1 / moment 行越界均报错 |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（模型参数、冻结输入、act_dot/force/qfrc），无 f64 中间量。
- **NP02**：记录冻结上游提交、三个上游函数名、模型源 sha256、运行环境与初始状态。
- **NP03**：两级判定——原生 f64 阶段链先行（max abs 9.54e-08），再与 MJWarp f32
  逐位比较（三组输出全一致）。
- **NP04**：**不适用/未测**——本片不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- 未实现 transmission（U068/G06）、执行器模型字段转换与设备内核；冻结输入
  由 authoring 产出，产品链未接入真实 MjModel→device 路径。
- 未支持：dyntype muscle/dcmotor/pid/user，gaintype/biastype muscle/dcmotor/so3/pid/user，
  tendon/site/body 传动，多 act 槽执行器（actnum>1），DCMOTOR 分支。
- 未覆盖：`actuator_acc0`、`actuator_lengthrange`、肌肉参数解析、动态更新与 forward 调度。
- 不宣称完整 G05 或精度验收通过；NP04 未测。

## 7. 复现

```powershell
# 离线重新生成参考（不进入产品链，需要 pinned venv 与 GPU）
scripts/authoring/author-g05-actuation.ps1

# 产品侧定向测试（只读冻结 fixture，不启动 Python）
cargo test --locked --lib actuator
```
