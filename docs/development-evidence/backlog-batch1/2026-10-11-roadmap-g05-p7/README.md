# G05@P7 执行器阶段切片：关节传动 transmission（端到端到 fwd_actuation）

任务：`2026-10-11-roadmap-g05-p7`。日期：2026-10-23。执行：Backend（批次 1 后继）。
矩阵：G05 执行器（`增益、偏置、动态、传动` / `限幅、激活状态、控制输入` / P4/P7）。

## 1. 阶段语义与细化口径

P4 切片（`9850ee0`）已交付执行器力 `fwd_actuation`（输入为冻结的 length/velocity/moment）。
本 P7 切片交付其上游环节：计划 API `mjwarp_rs::physics::actuator::transmission`
（U068；G05/G06；P4/P7）的 **hinge/slide 关节传动**宿主侧实现，并与 P4 的
`fwd_actuation` 组合成 transmission→fwd_velocity（moment·qvel）→fwd_actuation
端到端链。

关节传动语义逐项对齐上游 `_transmission`：`actuator_length = qpos[jnt_qposadr] * gear[0]`；
每执行器 moment 行 `nnz=1、colind=jnt_dofadr、moment=gear[0]`；
`actuator_velocity = moment·qvel`（上游 `_actuator_velocity` 单行累加）。

## 2. 实现

- `src/physics/actuator/transmission.rs`：`JointActuatorModel`（nv/trntype/trnid/gear/
  jnt_type/qposadr/dofadr）、`JointTransmissionData`（nworld/nq/qpos/qvel）、
  `transmission()` 输出 `actuator_length/actuator_velocity` 与逐世界 CSR moment。
- 显式拒绝：非 JOINT 传动（tendon/site/body/JOINTINPARENT）、free/ball 等非 hinge/slide
  关节、索引越界（复用 `ActuationError`，不静默回退）。
- 输出可直接喂给 `fwd_actuation`（P4 切片）；行地址取确定前缀（每执行器 nnz=1）。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/actuation/model.xml`（sha256 `f6d484da…`；nq=2、nv=2、nu=3、na=1、nactuator=3；hinge + slide） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_fwdPosition`→`mj_fwdVelocity`→`mj_fwdActuation`（NP03 一级） |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0，`transmission`→`fwd_velocity`→`fwd_actuation`（NP03 二级） |
| 生成脚本 | `scripts/authoring/author-g05-transmission.ps1` + `.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/actuation/transmission-cases.json`、本目录 `authoring.json` |
| 初始状态 | nworld=2；qpos `[[0.2,-0.4],[0.7,0.1]]`、qvel `[[0.5,-0.2],[-0.3,0.8]]`、ctrl/act 同 P4 夹具 |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 传动 vs 冻结上游 | `actuator_length`/`actuator_velocity` 逐元素位型一致；每执行器 moment (`colind`, `moment`) 逐位一致（rowadr 为确定前缀，上游行地址可能不同） |
| 端到端 vs 上游 | `act_dot`/`actuator_force`/`qfrc_actuator` 逐元素位型一致 |
| 产品 vs 原生 f64 一级判定 | max abs **9.54e-08** ≤ 冻结 1e-05（length/velocity/act_dot/force/qfrc 合计） |
| 拒绝面 | JOINTINPARENT 传动、ball 关节显式报错 |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（qpos/qvel/gear/输出），无 f64 中间量。
- **NP02**：记录冻结上游提交、三个上游函数名、模型源 sha256、环境与初始状态。
- **NP03**：两级判定——原生 f64 阶段链先行（9.54e-08 ≤ 1e-5），再与 MJWarp f32 逐位一致。
- **NP04**：**不适用/未测**——不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- 未实现 free/ball/JOINTINPARENT 关节传动、tendon/site/body 传动与
  transmission-body-moment（`nacttrnbody`）。
- 未实现设备内核、完整 forward 调度与图捕获；执行器模型字段转换仍属 M 族边界。
- 不宣称完整 G05 或精度验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g05-transmission.ps1
cargo test --locked --lib transmission
```
