# 历史验收与待续范围复审

任务：`2026-10-12-legacy-scope-review`。
日期：2026-10-22。
依据：`docs/architecture.md`、`docs/compatibility-matrix.md`、`docs/implementation-plan.md`、`docs/project-positioning.md`、`docs/README.md`、`docs/physics-stage-contracts.md`、`docs/windows-kinematic-fields.md`。
本轮只做人工范围复审：不改代码、不改测试、不重做已交付实现、不关闭任何历史变更。

## 结论

三项历史变更保留 backlog，原始验收、note 与证据逐字保留。
旧时点「全部 planned」「17 个原文件不变」「174 项旧 API」等属于当时快照，不作为当前回退依据。
字段与物理历史清单仍含完整模型与完整阶段契约；已落地子集不替代完整契约。
当前剩余实施范围按现有排程收尾，不重复登记任务，不自动激活或关闭旧项。

复审对象：

| 变更 | 状态 | 时点 | 本轮处置 |
| --- | --- | --- | --- |
| `2026-10-06-plan-engine-architecture` | backlog | 2026-10-06 | 保留；仅作历史快照归档候选 |
| `2026-10-07-kinematic-model-fields` | backlog | 2026-10-07 | 保留；剩余实施走现有排程 |
| `2026-10-07-physics-stage-plan-separation` | backlog | 2026-10-07 | 保留；完整物理阶段仍开放 |
| `2026-10-12-development-schedule-alignment` | verified | 2026-10-12 | 保留；其结论为本复审输入 |

## 当前事实快照

| 指标 | 数值 | 来源 |
| --- | --- | --- |
| 模块数 | 60 | `normify status` |
| 公开 API 条目 | 381 | `.normify/modules` 的 `[[apis]]` 统计 |
| 模块源码条目 | 885 | 同上 `[[source]]` 统计 |
| `planned` 标签模块 | 13 | 模块 front matter |
| 已验收任务 | 215 | `normify status` |
| 待办（backlog）任务 | 124 | `normify status` |
| 就绪 / 阻塞 / 放弃 | 10 / 17 / 5 | `normify status` |
| 结构校验 | 0 错误 | `normify validate` |

矩阵状态（`docs/compatibility-matrix.md` 头部）：

- 完整 G01 入口已实现，Windows/Linux 双模式历史验收保留，最新完整复测各 483 过 0 败。
- NP03 失败与待验收归零；NP01、NP02 仍缺全路径证据；NP04 仍缺公平稳态性能对照。
- 完整 T4 仍开放；其余完整项目仍待实现。

## 逐项复审

### 1. 初始化引擎计划架构（2026-10-06）

原始验收（时点快照）：

1. 根下保留七个模块。
2. 全部模块保持 planned。
3. 登记 99 项冻结公开契约。
4. 叶子 API 声明完整。
5. 依赖保留既有方向且无环。
6. 结构校验错误归零。
7. 每层容器提供布局。
8. 仓库 17 个原文件保持不变。
9. 仅报告真实结构证据。

区分：

| 原始条目 | 时点含义 | 当前状态 | 归类 |
| --- | --- | --- | --- |
| 七个模块 | 初始根结构 | 七根仍在，叶子扩展为 60 模块 | 快照，已被超越 |
| 全部模块 planned | 初始化时无实现 | 13 个模块仍 planned，其余已交付子集或原型 | 快照，不得据此回退 |
| 99 项冻结公开契约 | 初始契约数 | 当前 381 条 API 条目 | 快照，契约已扩展 |
| 叶子 API 声明完整 | 初始声明范围 | 当前叶子声明随实现扩展 | 快照 |
| 依赖无环 | 结构约束 | 仍有效 | 持续约束 |
| 结构校验零错误 | 当时结果 | 当前仍 0 错误 | 持续结果 |
| 每层容器布局 | 初始布局要求 | 已落地并扩展 | 已交付 |
| 17 个原文件不变 | 当时「不修改源码」的边界 | 后续实现已改变这些文件 | 快照，不构成回退依据 |
| 仅报告真实证据 | 报告口径 | 仍有效 | 持续约束 |

当前剩余工作：无独立实施项。该项是「初始化计划架构」的时点记录，其目标已由后续 215 项验收工作超越。
收尾路径：由人工决定归档方式；本轮保留 backlog 与原始记录，不重做、不伪造关闭。

### 2. 运动学字段转换与上传（2026-10-07）

原始验收（时点快照）：

1. 保留原 16 节点、七模块与 174 项旧 API 路径；model/io 下沉四个叶子并迁移 API；产品与新叶子 planned，传输叶子 active。
2. 四种关节及 qpos/dof 宽度；12 项连续字段与全部校验；只读且不跑物理。
3. 可选 MSVC 只读桥接；12 字段快照在模型释放后有效；Rust/C++ 核验 DTO；原 4 字段接口兼容。
4. 整组 GPU 上传后返回只读句柄；失败不公开部分模型；零关节合法；无 CPU 回退。
5. 保留两份旧 MJB；独立样本与哈希；产品链不调用 Python 或 MJCF 编译器；Windows debug/release 检查。
6. fmt、测试、clippy/doc、全量 Windows 回归；Normify 刷新与提交；Linux 与完整物理未执行。

区分：

| 原始条目 | 时点含义 | 当前状态 | 归类 |
| --- | --- | --- | --- |
| 12 项连续运动学字段 | 子集交付范围 | 已交付；后续有惯性、mocap、相机光源等增量 | 已交付，不重做 |
| 四种关节与 qpos/dof 宽度 | 子集范围 | 已交付并由后续任务扩展 | 已交付 |
| 原生只读桥接与 DTO | P2 子集 | 已交付；`native/model_probe.cpp` 与头文件保留 | 已交付 |
| 原 16 节点 / 七模块 / 174 API | 当时结构 | 当前 60 模块、381 API | 快照，不得据此回退 |
| 产品与新叶子 planned、传输叶子 active | 当时状态 | 多个叶子已交付子集；传输层为原型 | 快照 |
| 两份旧 MJB 与独立样本 | 证据要求 | `fixtures/native-probe/` 保留 | 持续证据 |
| 全量 Windows 回归 | 当时门禁 | 已执行并有历史证据 | 已交付 |
| Linux 与完整物理未执行 | 当时边界 | 仍为剩余项 | 当前剩余 |
| 完整 Model/put_model、身份、周期参数 | 当时未验收 | 仍未验收 | 当前剩余 |

当前剩余（真实契约，非重复登记）：

- 完整 Model 字段全集与 `put_model` 等价。
- 模型身份与周期参数完整契约。
- 完整模型输入路径的 Windows/Linux 一致验收。
- 与冻结上游的完整等价，不得以已交付子集替代。

收尾路径：沿用现有 `roadmap-*` 与 `next-*` 排程推进；本复审不新建重复任务。

### 3. 物理阶段计划与矩阵叶子分离（2026-10-07）

原始验收（时点快照）：

1. 保留既有 API 路径与语义，并迁移 IO 契约引用。
2. 核心保留真实源码证据，完整等价契约保持计划。
3. 完整阶段通过验收后，才能激活核心叶子。

区分：

| 原始条目 | 时点含义 | 当前状态 | 归类 |
| --- | --- | --- | --- |
| 保留既有 API 路径与语义 | 66 项物理契约 | 已保留；后续新增大量子集入口 | 持续约束 |
| 核心保留真实源码证据 | 计划核心的最小证据 | 物理核心仍 planned，多个子集独立交付 | 快照＋持续 |
| 完整阶段验收后才激活核心 | 激活门槛 | 门槛仍有效；核心未激活 | 持续约束 |

当前剩余：

- 完整物理阶段等价（G01 之外的全阶段）与 NP01 至 NP04 验收。
- G02 完整、G03/G04 完整；G01 剩余债务包括位证书口径、all-probes 剩余红项、PTX 捕获目录双语义、门禁脚本残留。
- Linux、release、完整 T4 与部署验收。
- M7 系列 W12 至 W15、W23 的实施任务（评审已完成 W13、W14、W15 路线，实施另立）。

收尾路径：现役候选任务（`2026-10-18-layers-m3`、`2026-10-18-layers-m7-link` 后继、`2026-10-18-linux-toolchain-validation` 等）与 roadmap 系列；本复审不激活核心叶子、不关闭本项。

## 旧 planned 快照口径

以下约束只描述历史时点，不构成回退或复刻要求：

- 「全部模块保持 planned」：初始化时点的状态，当前已有大量已交付子集与原型模块。
- 「仓库 17 个原文件保持不变」：当时「不修改源码」的边界，后续实现已改变这些文件。
- 「174 项旧 API 路径」「99 项冻结公开契约」：当时计数；当前 API 条目 381，旧路径仍保留。
- 「16 节点 / 七模块」：当时结构；当前 60 模块。
- 「产品与新叶子 planned、传输叶子 active」：当时分工状态。

同时保留以下持续约束：

- 不删除既有 API 路径与语义。
- 不因旧快照回退已交付模块。
- 子集实现不替代完整模型或完整阶段契约。
- 结构校验、依赖无环、真实证据报告继续强制。

## 保留的原始验收与证据

- 三项变更的原始 JSON（`.normify/changes/2026-10-06-plan-engine-architecture.json`、`2026-10-07-kinematic-model-fields.json`、`2026-10-07-physics-stage-plan-separation.json`）逐字保留：status、acceptance、note、依赖不变。
- `2026-10-12-development-schedule-alignment` 的结论保留：原 43 份 verified 记录字节不变，验收与模块清单保留。
- 历史验收证据目录 `docs/development-evidence/`、报告 `docs/windows-kinematic-fields.md`、`docs/windows-inertial-fields.md`、`docs/physics-stage-contracts.md`、`docs/compatibility-matrix.md` 不改动。
- 已验收任务 215 项保持 verified，不重开、不重跑。

## 经确认的剩余实施范围（登记用，不新建任务）

| 范围 | 依据 | 收尾路径 |
| --- | --- | --- |
| 完整 Model 字段与 put_model、身份与周期参数 | 运动学字段变更的未验收部分；模块 planned | 现有 roadmap 与 next 排程 |
| 完整物理阶段与 NP01 至 NP04 | 物理阶段变更的激活门槛；矩阵状态 | roadmap 系列与 M7 实施任务 |
| G02、G03、G04 完整契约 | 矩阵与阶段契约 | 现役 layers 任务与后续切片 |
| G01 剩余债务 | 位证书口径、all-probes 红项、PTX 目录双语义 | 后继任务（另立） |
| Linux、release、T4、部署 | 矩阵与兼容性边界 | `linux-toolchain-validation` 等 |
| M7 W12 至 W15、W23 实施 | 三层设计与评审结论 | 各自另立交付任务 |

约束：以上范围不重做已交付实现；不以此清单自动激活或关闭任何旧变更；不代替正式验收。

## 本轮验证

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | 通过 |
| `unchanged` 检查 | `src`、`tests`、`native`、`include`、`fixtures`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml` 未改动 |
| `normify validate` | 0 错误 |
| 三项历史变更状态 | backlog，未关闭、未修改 |
| `git status` | 仅 `?.claude/` 未跟踪 |

本轮未运行 GPU 或 Linux 测试。本轮未新增产品实现。本轮未改动任何历史验收记录。

## 证据

- 复审报告：本文件。
- 交付回执：`docs/development-evidence/changes/2026-10-12-legacy-scope-review.json`。
- 依据文档：`docs/architecture.md`、`docs/compatibility-matrix.md`、`docs/implementation-plan.md`、`docs/project-positioning.md`、`docs/physics-stage-contracts.md`、`docs/windows-kinematic-fields.md`。
