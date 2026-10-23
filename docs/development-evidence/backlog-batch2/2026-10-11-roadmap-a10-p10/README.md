# A10@P10 记录与性能工具切片：Rust 无头测速与状态导出

任务：`2026-10-11-roadmap-a10-p10`。日期：2026-10-23。执行：Backend（批次 2，末片）。
矩阵：A10 记录与性能工具（`Rust无头测速与状态导出` / P10/P11）。

## 1. 条款语义与细化口径

A10 验收为「Rust无头测速与状态导出」。按当前代码布局，本片交付宿主侧
`runtime::tooling`：

- **无头测速**：`BenchmarkRecorder` 归纳各阶段样本（纳秒整数）并产出
  count/total/min/max/mean/median 报告；`measure` 在调用方闭包上做宿主侧计时
  （零迭代拒绝）。不启动 Python、不开窗口、不发射 GPU 内核。
- **状态导出**：`StateExport`（`time`/`nworld`/`qpos`/`qvel`，f32 物理字段）
  导出确定性 JSON；`export_report_json` 导出测速报告；f32 用最短往返格式，
  `serde_json` 解析回读后 qpos/qvel 位型一致；非有限值/空世界/长度不匹配拒绝。
- **末片复核**：`batch_two_public_apis_still_work` 冒烟复核批次 2 此前全部切片
  的公开 API（G07/G16/G20/G21/G23/A08），不重跑物理参考。

## 2. 实现

| 文件 | 内容 |
| --- | --- |
| `src/runtime/tooling/mod.rs` | `BenchmarkRecorder`/`PhaseSamples`/`PhaseSummary`/`BenchmarkReport`、`measure`、`StateExport`、JSON 导出与错误面 |
| `src/runtime/tooling/tests.rs` | 记录/报告聚合、无头计时、状态 JSON 往返、报告 JSON、错误面、批次 2 API 复核 |
| `src/runtime/mod.rs` | 新增 `pub mod tooling;`（113→114 行） |

## 3. 验证（本次实测）

| 判定 | 结果 |
| --- | --- |
| 阶段聚合 | 3 相样本 `[100,300,200]` → count 3 / total 600 / min 100 / max 300 / mean 200 / median 200；阶段按首次出现顺序 |
| 无头计时 | `measure(3, …)` 执行 3 次且返回纳秒 > 0；`measure(0, …)` 返回 `ZeroIterations` |
| 状态导出 | JSON 经 `serde_json` 回读：`time` 位型一致、`nworld` 正确、qpos/qvel f32 位型一致 |
| 报告导出 | JSON 解析：`total_samples=4`、`total_ns=65`、阶段名/计数/min/max 正确 |
| 拒绝面 | 空阶段名、非有限 time/qpos/qvel、`nworld=0`、qpos/qvel 长度不匹配 |
| 批次 2 复核 | G07 过滤位/类型、G16 `quat_integrate`/`next_velocity`、G20 `mul_m`、G21 `poly_force_deriv`、G23 `sensor_pos`、A08 `ZeroCopyView` 指针相等 全部通过 |

复跑：`cargo test --locked --lib tooling` → **5 过 0 败**；`cargo fmt --check` 与
`cargo clippy --all-targets -- -D warnings` 通过。

## 4. 验收指标映射（NP01–NP04）

- **NP01**：物理状态字段全 f32；mean/median 仅为诊断统计（f64，不反馈产品物理）。
- **NP02**：记录无头/无 Python/无 GPU 内核的测速与导出契约、JSON 结构与拒绝面。
- **NP03**：不适用——无物理数值对照。
- **NP04**：不适用——本片不做 GPU 性能测量，`resources` 置空不占用 GPU 窗口；
  真实批次性能门槛与显存统计未验收。

## 5. 未验收边界

- 未实现 GPU 事件计时、CUDA 图耗时与显存统计。
- 未做真实仿真批次的性能门槛测量（需独占 GPU 的 NP04 测量；本片不占 GPU）。
- 未实现 CSV/二进制导出格式与 T4/Linux 跨平台性能对照。
- 不宣称完整 A10 或性能验收；未改 `src/main.rs`（492 行）与其它运行时路径。

## 6. 复现

```powershell
cargo test --locked --lib tooling -- --nocapture
```
