# 批次 2 证据索引：G07/G16/G20/G21/G23/A08/A10（7 条已交付，R01 顺延）

日期：2026-10-23。执行：Backend。批次范围：Chief 派单的 8 条 ready 路线图任务；
其中 7 条按「旧计划细化 → 冻结上游逐位/解析对照 + 原生 f64 一级判定 → 证据 →
gate/check/done」交付，`r01-p8` 按 Chief 裁决 A **顺延**（见 §2）。

所有交付均遵守：精度硬指标（全链 f32、不升精度/不放宽容差）、`cargo fmt/clippy -D
warnings/unit` 门禁、path-limited 提交、Normify 范围与回执、证据/回执双落盘。

## 1. 已交付 7 条

| # | 任务 | 提交 | 验证核心数字 | 未验收边界（摘要） |
| --- | --- | --- | --- | --- |
| 1 | `g07-p5` 宽相碰撞 | `12151dc`（登记 `5820fde`） | 47 用例候选 `(worldid,pair,pairid)` 与上游逐案相等；NXN/两类 SAP 集合一致；原生接触对 27/27 包含、边距 6/6 计数相等 | SLEEP/incremental、AABB/rbound/margin/gap 世界批量、窄相、GPU 内核 |
| 2 | `g16-p6` Euler 与 RK4 | `9692c92`（登记 `2ad0475`） | 单步 euler 位型 64/72、RK4 67/72，max abs 5.96e-08（FMA 末位，1e-6）；原生 1.19e-07/6.26e-06；合成 ODE 一/四阶收敛 | activation/mocap、阻尼隐式分支、forward 逐级求导、DISCRETE/IMPLICIT、GPU |
| 3 | `g20-p7` 逆向动力学 | `a10aea5`（登记 `074a736`） | qfrc_inverse vs 上游 1.91e-06；原生 `mj_inverse` 2.62e-06；正逆回代力平衡 2.62e-06（≤1e-5） | DISCRETE/IMPLICITFAST、inv_constraint、dense 分支、GJ/睡眠、GPU |
| 4 | `g21-p7` 平滑速度导数 | `cef3357`（登记 `90736a7`） | 多项式配置 18/18 元素逐位一致（max abs 0）；解析 vs 上游 f32 差分 1.55e-06、原生 f64 差分 4.67e-08 | 执行器/肌腱/流体导数、efm/隐式、世界批量、GPU |
| 5 | `g23-p7` 传感器 | `34bbd05`（登记 `0ba71e6`） | sensordata vs 上游 1.19e-07；原生 5.96e-08；cutoff 0.8→0.4、BALLQUAT/CLOCK/时序断言 | 其它传感器类型、sensor_acc/SENSOR_STAGE、世界批量、GPU |
| 6 | `a08-p10` 零拷贝优化 | `55a6cb8`（登记 `68f5529`） | 视图切片指针 == 原缓冲指针 + offset（零拷贝可判定）；跨流/未完成/代数/越界拒绝 5 类；5 测试过 | 真实 CUDA 事件与设备指针、设备分配/跨流并发、零拷贝 GPU 性能、A07 同步契约 |
| 7 | `a10-p10` 记录与性能工具 | `3757567`（登记 `e84620b`） | 阶段聚合/无头计时/状态与报告 JSON 往返/拒绝面 + 批次 2 公开 API 冒烟复核；5 测试过 | GPU 事件计时/显存、真实批次性能门槛（NP04）、CSV/二进制、跨平台性能 |

各任务证据目录：`docs/development-evidence/backlog-batch2/<task-id>/`（README +
authoring.json，A08/A10 无 authoring）；回执：`docs/development-evidence/changes/<task-id>.json`。

## 2. 顺延 1 条：`r01-p8` 渲染上下文（Chief 裁决 A）

- 任务保持 `proposed`、未启动；note 已登记阻塞原因（`ed7de14`）。
- 阻塞点：上游 `render_util.create_render_context` 依赖 BVH 构建/重拟合、网格与
  纹理资产上传、CUDA 渲染内核与相机/采样参数（约 30 个参数）；当前引擎未实现
  渲染/BVH 子系统。
- 处置：不做宿主侧布局探针（Chief 明确为假语义、不做）；顺延到渲染能力具备后
  （批次 3 或独立渲染计划）再按原验收「全部默认参数与生命周期」细化交付。
- 同步证据：`2026-10-11-roadmap-g28-p8`（批次 1）已把 BVH/RenderContext 登记为
  未实现，本顺延与其一致。

## 3. 批次门禁与状态（收口时实测）

- `cargo fmt --check`：通过；`cargo clippy --all-targets -- -D warnings`：通过。
- `cargo test --locked --lib`：**461 过 0 败**（A10 门禁记录）。
- `normify validate --fingerprints`：错误 0；工作区干净；每任务 gate/check/done
  均在 path-limited 提交前通过 `check --staged`。
- 精度：所有切片全链 f32；仅 A10 诊断统计使用 f64（不反馈产品物理）；未改动任何
  既有容差、断言、ABI 与排除清单；W13/Linux/g03-parity 未触碰。

## 4. 批次级未验收边界

- 本批 7 条均为「当前代码布局下最小可交付切片」，均未宣称完整矩阵族验收；
  每条 README 的 §未验收边界为权威清单。
- 上游余量：G07 窄相/接触、G16 activation 与隐式分支、G20 离散与约束求解、
  G21 其余导数项、G23 其余传感器与阶段调度、A08 真实设备零拷贝路径、A10 GPU
  性能测量，均待后续批次。
- R01 渲染上下文（含 R02–R10）整体顺延，未改架构与模块登记。
