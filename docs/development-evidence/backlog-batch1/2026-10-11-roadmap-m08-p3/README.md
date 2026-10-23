# M08@P3 局部重置阶段切片：世界掩码与关键帧重置

任务：`2026-10-11-roadmap-m08-p3`。日期：2026-10-23。执行：Backend（批次 1）。
矩阵：M08 局部重置（`世界掩码、关键帧` / `隔离、历史与时钟` / P3/P7）。

## 1. 阶段语义与细化口径

原图只有 M08@P3 粗切片，未给具体函数。按当前代码布局，本片取
架构计划 API `mjwarp_rs::io::state::reset_data`（U035）与
`reset_data_keyframe`（U036）的**宿主侧实现**，并附带最小世界批量状态
`WorldState { time, qpos, qvel }`（全 f32）。

P3 阶段只覆盖「状态、缓冲、重置」侧：不实现完整 Data、不发射 GPU 内核、
不接设备缓冲。验收口径未放宽：矩阵核心验收「隔离、历史与时钟」中，
本片交付**世界隔离**（布尔/整数掩码）与**时钟**（掩码内 `time=0`）；
历史（G24 `reset_history`）与其余 Data 字段如实列为未验收边界（§6）。

## 2. 实现

`src/io/state.rs`：

- `WorldState`：世界批量 `time`（nworld）、`qpos`（nworld×nq）、`qvel`（nworld×nv），
  `new`/`from_batches` 校验维度与长度。
- `reset_data(state, qpos0, mask)`：掩码内世界 `time=0`、`qpos=qpos0`、`qvel=0`；
  `ResetMask::{All, Worlds(&[bool]), Flags(&[i32])}` 对齐上游布尔/整数掩码
  （整数非零即重置）。
- `reset_data_keyframe(state, keyframes, selection)`：`KeyframeSelection::{All(i32),
  Worlds(&[i32])}`；全局整数索引越界报错（对齐上游 `ValueError`），逐世界数组中
  越界/负索引世界**不重置**（对齐上游逐世界语义）。
- 错误类型 `ResetError`：维度、长度、关键帧索引越界；`Display + Error`。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/local-reset/model.xml`（sha256 `5251f712…`，nq=2、nv=2、nkey=3） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_resetData` / `mj_resetDataKeyframe`（NP03 一级） |
| 上游参考 | MJWarp 冻结提交 `71da24d956378a87a703b6e1442b13aec0c4ac29`，Warp 1.15.0，`reset_data` / `reset_data_keyframe`（NP03 二级） |
| 生成脚本 | `scripts/authoring/author-m08-reset.ps1` + `author-m08-reset.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/local-reset/reset-cases.json`、本目录 `authoring.json` |
| 环境 | RTX 4070 Ti SUPER / 驱动 596.36；nworld=3、初始 time/qpos/qvel 非零 |

7 个冻结用例：`mask_none`、`mask_bool`（含不重置世界）、`mask_bool_all_false`、
`mask_flags`（整数非零）、`keyframe_all_1`（全局关键帧）、
`keyframe_worlds_mixed`（逐世界 `[0,2,-1]`，-1 不重置）、`keyframe_worlds_none`。

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（7 个用例） | **逐元素位型完全一致**（`time/qpos/qvel`，容差 0） |
| 上游 vs 原生 `mj_resetData`/`mj_resetDataKeyframe` | max abs **0**（NP03 一级通过） |
| 掩码隔离 | 未掩码世界 `time/qpos/qvel` 保持不变（用例 + 单元测试覆盖） |
| 逐世界关键帧越界 | 越界/负索引世界不重置，其余世界按 key 写入 |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：新增路径全部 f32（time/qpos/qvel/关键帧数组），无 f64 中间量。
- **NP02**：记录冻结上游提交、函数名（`reset_data`/`reset_data_keyframe`）、
  模型源 sha256、运行环境与用例清单。
- **NP03**：两级判定——先与原生 MuJoCo 参考比较（max abs 0），再与 MJWarp
  参考逐位比较（7/7 用例一致）。输入量化规则：原生 f64 → f32。
- **NP04**：**不适用/未测**——本片不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- 未实现完整 `Data`：`act`、`ctrl`、`qfrc_applied`、`xfrc_applied`、
  `mocap_pos/quat`、`qacc/qacc_warmstart`、`act_dot`、`sensordata`、
  `cfrc_ext`、接触/约束/能量/睡眠字段均未包含。
- 「历史」未交付：`reset_history`、`init_ctrl_history`/`init_sensor_history`
  依赖尚未实现的 G24 历史模块；关键帧的 `act/ctrl/mocap` 字段同样未包含。
- 无设备内核与上传路径（上游 reset 以内核实现）；不宣称完整 M08 或精度验收。
- NP04 未测。

## 7. 复现

```powershell
# 离线重新生成参考（不进入产品链，需要 pinned venv 与 GPU）
scripts/authoring/author-m08-reset.ps1

# 产品侧定向测试（只读冻结 fixture，不启动 Python）
cargo test --locked --lib state
```
