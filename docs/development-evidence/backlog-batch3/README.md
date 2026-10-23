# 批次 3 证据索引：G08/G18/G24（已交付 3 条，M10/A10 进行中）

日期：2026-10-23。执行：Backend。批次范围：Chief 派单的 5 条 ready 路线图任务
（`g08-p5`、`g18-p6`、`g24-p7`、`m10-p7`、`a10-p11`），逐条按「旧计划细化 →
冻结上游逐位/解析对照 + 原生 f64 一级判定 → 证据 → gate/check/done」交付。

所有交付均遵守：精度硬指标（全链 f32、不升精度/不放宽容差）、`cargo fmt/clippy -D
warnings/unit` 门禁、path-limited 提交、Normify 范围与回执、证据/回执双落盘。

## 1. 已交付 3 条

| # | 任务 | 提交 | 验证核心数字 | 未验收边界（摘要） |
| --- | --- | --- | --- | --- |
| 1 | `g08-p5` 基本几何碰撞 | `2269b04`（登记 `6ebe9ac`） | 接 G07 候选 5 条与上游逐条一致；接触数 5 = 上游 nacon = 原生 ncon 之和（世界 0:3、世界 1:2）；接触浮点字段 vs 冻结上游 max abs 7.45e-09（≤1e-6），整数逐位；原生 f64 `mj_collision` 5.96e-08（≤1e-5） | 其余几何对、convex/CCD/multiccd、flex、显式 pair、collision sensor 位、EFC/约束、GPU |
| 2 | `g18-p6` 离散积分 | `9239fa5`（登记 `88b3593`） | 单步位型 18/22，max abs 5.96e-08（≤1e-6）；原生 f64 5.96e-08（≤1e-5）；与 G16 `euler` 直接分支逐位等价；`discrete_acc` 恒等分支一致、DampedEuler 显式拒绝 | `(M+dt·diag(damping))` 有效度量求解、IMPLICITFAST、`_launch_discrete_free_gyro`、约束求解与一致性、`check_discrete`、GPU |
| 3 | `g24-p7` 历史与延迟 | `9cdf5de`（登记 `9d9e3e5`） | 冻结上游 wp.func 回放 4 场景缓冲字逐位一致；33 次读取 51 个值逐位一致（max abs 0）；4 个重置用例 + SO3+QUAT 逐位一致；原生 C f64 `mj_resetData` 2.682e-09（≤1e-6） | 三次插值、公共初始化入口、interval 运行期门控、`apply_sensor_delay`/`read_ctrl`/`read_sensor` forward 封装、`d.history` 全量分配与 registry、GPU |

各任务证据目录：`docs/development-evidence/backlog-batch3/<task-id>/`（README +
authoring.json）；回执：`docs/development-evidence/changes/<task-id>.json`。

## 2. 进行中 2 条

- `m10-p7`：Rust 回调阶段切片，按 Chief 边界只交可独立成立部分（回调契约类型/错误面/
  注册表），需要完整 forward/阶段调度才成立的部分停手报 Chief。
- `a10-p11`：记录与性能工具阶段切片，P11 含「全量矩阵与清洁部署 + 双平台准入」，
  核原验收后若超出当前引擎/平台能力则停手报 Chief，不硬做最小切片。
- 两条完成后的证据目录落 `docs/development-evidence/backlog-batch3/<task-id>/`。

## 3. 批次门禁与状态（收口时实测）

- `cargo fmt --check`：通过；`cargo clippy --all-targets -- -D warnings`：通过。
- `cargo test --locked`（gates.unit，全目标）：**470 过 0 败**（G24 门禁记录）。
- `normify validate --fingerprints`：错误 0；每任务 gate/check/done 均在 path-limited
  提交前通过 `check --staged`。
- 精度：所有切片全链 f32；未改动任何既有容差、断言、ABI 与排除清单；
  W13/Linux/g03-parity 未触碰。

## 4. 批次级未验收边界

- 本批 3 条已交付均为「当前代码布局下最小可交付切片」，均未宣称完整矩阵族验收；
  每条 README 的 §未验收边界为权威清单。
- 上游余量：G08 其余几何对/CCD/约束、G18 有效度量与约束求解、G24 三次插值与
  forward 调度，均待后续批次；M10/A10 见任务记录。

## 5. 目录约定

- 批次 1、2、3 各自独立证据目录：`backlog-batch1/`、`backlog-batch2/`、`backlog-batch3/`。
- 本目录由治理任务 `2026-10-23-batch3-evidence-index` 建立；`g08-p5`、`g18-p6`
  证据原误放 `backlog-batch2/`，已原样迁移至此（回执路径同步更新、文件内容与
  blob 不变，并已对两条已关闭任务用 `normify fix` 追加修正记录）。
