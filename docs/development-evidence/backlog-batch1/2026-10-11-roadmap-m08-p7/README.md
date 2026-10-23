# M08@P7 局部重置阶段切片：Data 字段扩展（ctrl/act/qfrc/xfrc/mocap + 关键帧附加字段）

任务：`2026-10-11-roadmap-m08-p7`。日期：2026-10-23。执行：Backend（批次 1 后继）。
矩阵：M08 局部重置（`世界掩码、关键帧` / `隔离、历史与时钟` / P3/P7）。

## 1. 阶段语义与细化口径

P3 切片（`e63b296`）已交付 time/qpos/qvel 的掩码与关键帧重置。本 P7 切片在
`io::state` 扩展计划 API `reset_data`/`reset_data_keyframe`（U035/U036；M08；P3/P7）的
Data 字段：`ctrl`（nu）、`act`（na）、`qfrc_applied`（nv）、`xfrc_applied`（nbody×6）、
`mocap_pos`（nmocap×3）与 `mocap_quat`（nmocap×4）。

语义逐项对齐上游：掩码内世界 **ctrl/act/qfrc_applied/xfrc_applied 置零**，
**mocap 复位为模型 mocap 身体的 body_pos/body_quat**；关键帧路径先按有效 key
世界执行 reset_data 语义（其余字段清零），再写入 `key_act/key_ctrl/key_mpos/key_mquat`
与 time/qpos/qvel。逐世界越界 key 不重置。

## 2. 实现

- `src/io/state/extended.rs`：`ExtendedLayout`/`ExtendedState`（九组 f32 世界批量字段，
  长度校验）、`ExtendedBatches`、`MocapDefaults`、`ExtendedKeyframes`（七组关键帧字段）、
  `reset_data_extended`、`reset_data_keyframe_extended`；复用 P3 的
  `ResetError`/`ResetMask`/`KeyframeSelection`。
- `src/io/state.rs` 与 `src/io/mod.rs` 导出；不破坏 P3 API。

## 3. 参考来源与生成方式

| 项 | 值 |
| --- | --- |
| 模型源 | `fixtures/local-reset/extended-model.xml`（sha256 `413e5728…`；nbody=4、nq=2、nv=2、nu=1、na=1、nmocap=1、nkey=2） |
| 原生参考 | MuJoCo 3.15.0（pinned venv）`mj_resetData`/`mj_resetDataKeyframe`（NP03 一级） |
| 上游参考 | MJWarp `71da24d956…`，Warp 1.15.0，`reset_data`/`reset_data_keyframe`（NP03 二级） |
| 生成脚本 | `scripts/authoring/author-m08-extended-reset.ps1` + `.py`（离线，产品测试不启动 Python） |
| 产物 | `fixtures/local-reset/extended-cases.json`、本目录 `authoring.json` |
| 用例 | 6 个：全量重置、布尔掩码、整数掩码、全局关键帧、逐世界 `[0,-1]`、逐世界 `[1,1]`；初始 9 组字段均非零 |

## 4. 对照结果（本次实测）

| 判定 | 结果 |
| --- | --- |
| 产品 vs 冻结上游（6 用例 × 9 字段） | **逐元素位型一致**（容差 0） |
| 产品 vs 原生 f64 一级判定 | max abs **0**（6 用例全部字段） |
| 掩码隔离 | 未掩码世界保留初始 ctrl/act/qfrc/xfrc/mocap 值 |
| keyframe 语义 | 有效 key 世界按关键帧写入并清零 qfrc/xfrc；越界 key 世界保持原状态 |
| mocap 复位 | 掩码/全量重置后 mocap_pos/quat = mocap 身体 pos/quat；关键帧路径写入 key_mpos/key_mquat |

## 5. 验收指标映射（NP01–NP04）

- **NP01**：九组字段全 f32，无 f64 中间量。
- **NP02**：记录冻结上游提交、两个上游函数名、模型源 sha256、环境与 6 用例 × 9 字段清单。
- **NP03**：两级判定——原生 `mj_resetData`/`mj_resetDataKeyframe` 先行（max abs 0），
  再与 MJWarp f32 逐位一致。
- **NP04**：**不适用/未测**——不发射内核、无批量对照与性能记录。

## 6. 未验收边界

- G24 历史：`reset_history`、`init_ctrl_history`/`init_sensor_history` 未实现。
- `eq_active`（依赖等式约束字段）、`qacc_warmstart`/`act_dot`/`sensordata`/`userdata`、
  能量/接触/约束/睡眠字段与完整 Data 均未包含。
- 无设备内核与上传路径；不宣称完整 M08 或精度验收通过；NP04 未测。

## 7. 复现

```powershell
scripts/authoring/author-m08-extended-reset.ps1
cargo test --locked --lib extended
```
