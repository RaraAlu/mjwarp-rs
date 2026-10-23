# G23@P7 传感器切片：sensor_pos / sensor_vel 的类型、时序与单位

任务：`2026-10-11-roadmap-g23-p7`。日期：2026-10-23。执行：Backend（批次 2）。
矩阵：G23 传感器（`位置、速度、加速度阶段` / `类型、时序与单位` / P7）。

## 1. 阶段语义与细化口径

旧计划 G23@P7 只有粗切片（位置/速度/加速度阶段；验收「类型、时序与单位」）。
按当前代码布局，本片交付 `sensor_pos`/`sensor_vel` 的四个类型子集：

| 传感器 | 阶段 | 取值 | 单位 |
| --- | --- | --- | --- |
| `JOINTPOS`（mjtSensor 9） | 位置 | `qpos[jnt_qposadr[objid]]` | 关节空间（hinge/slide 弧度或长度） |
| `JOINTVEL`（10） | 速度 | `qvel[jnt_dofadr[objid]]` | 关节速度 |
| `BALLQUAT`（18，datatype 3） | 位置 | `qpos[jnt_qposadr..+4]`（wxyz 4 分量） | 单位四元数 |
| `CLOCK`（45） | 位置 | `time` | 秒 |

写出遵循上游 `_write_scalar`/`_write_vector`：`sensor_adr` 偏移、`cutoff` 截断
（`REAL` → clamp(±cutoff)，`POSITIVE` → min(cutoff)，`QUATERNION` 不截断）；
不同阶段只写本阶段类型，其余保持 0（对齐上游分阶段调度）。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/physics/sensor.rs` | `SensorLayout`、`sensor_pos`（JOINTPOS/BALLQUAT/CLOCK）、`sensor_vel`（JOINTVEL）、cutoff/数据类型写出与校验 |
| `src/physics/sensor/tests.rs` | 冻结上游 sensordata、原生 f64 一级判定、时序/单位与 cutoff、校验面 |

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/sensor/model.xml`（sha256 `f32407bd…`；铰链 + 球关节 + 滑块，nq=6/nv=5/nsensor=6/nsensordata=9；`slide_pos` cutoff=0.4） |
| 用例 | 3 组状态（default / rotated / cutoff），分别记录 `sensor_pos` 与 `sensor_vel` 两阶段快照 |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0：`sensor.sensor_pos`/`sensor.sensor_vel`（kinematics 后调用） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）f64 `mj_kinematics` + `mj_sensorPos`/`mj_sensorVel` |
| 生成脚本 | `scripts/authoring/author-g23-sensor.ps1` + `.py`（离线；产品测试不启动 Python） |
| 产物 | `fixtures/sensor/cases.json`、本目录 `authoring.json` |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游 | `sensor_pos`/`sensor_vel` sensordata max abs **1.19e-07** ≤ 1e-6（单 ulp 舍入） |
| 产品 vs 原生 f64 一级判定 | max abs **5.96e-08** ≤ 1e-5 |
| 时序/单位 | 位置阶段不写 `JOINTVEL`（保持 0）、速度阶段不写位置类传感器；`BALLQUAT` 写 4 分量；`CLOCK` 写 `time` |
| cutoff 截断 | `slide_pos` qpos=0.8、cutoff=0.4 → 输出 **0.4**（上游/原生一致） |
| 布局 | 6 个传感器 adr `[0,1,2,3,4,8]`、datatype `[REAL,REAL,REAL,REAL,QUATERNION(3),REAL]` |

复跑：`cargo test --locked --lib sensor` → **2 过 0 败**；`cargo fmt --check` 与
`cargo clippy --all-targets -- -D warnings` 通过。

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全 f32（qpos/qvel/time 输入、cutoff 截断、sensordata 输出）。
- **NP02**：记录冻结上游提交、`sensor_pos`/`sensor_vel` 函数名、模型 sha256、sensor/joint 布局与常量、环境。
- **NP03**：两级判定——原生 f64 `mj_sensorPos`/`mj_sensorVel` 先行（5.96e-08），再与 MJWarp f32 参考对照（1.19e-07）。
- **NP04**：**不适用/未测**——不发射内核、无批量耗时与显存记录。

## 6. 未验收边界

- 未实现 contact/rangefinder/framepos 等其它传感器、tendon/actuator 传感器、
  磁力计/相机投影/geomfromto 特例。
- 未实现 `sensor_acc` 阶段与 SENSOR_STAGE 调度（位置/速度/加速度阶段的组合执行）。
- 未实现世界批量广播语义、GPU 内核与图执行。
- 不宣称完整 G23 或 P7 验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-g23-sensor.ps1
cargo test --locked --lib sensor -- --nocapture
```
