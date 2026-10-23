# 待办池分诊（124 条 backlog 四类评估）

任务：`2026-10-23-backlog-triage-2`。日期：2026-10-23。执行：QA。
范围：`.normify/changes/` 中 `status=backlog` 的 124 条任务（原排程 P0–P11 候选池，2026-10-11 roadmap 系列 + 3 条早期计划条目）。
本任务只做评估与 CLI 状态动作（abandon / promote），**不改产品代码、不 start 任何任务、不硬改 depends_on**。

## 1. 结论

| 类别 | 数量 | 处理 |
| --- | ---: | --- |
| a 已过时/被替代 | **1** | 已 abandon |
| b 已被现有工作覆盖 | **52** | 已 abandon（reason 引用覆盖它的 verified 任务/证据） |
| c 仍有效但需外部前提 | **13** | 保持 backlog，note 已含解锁条件 |
| d 现在可在 Windows 本地执行 | **43** | 已 promote（proposed，未 start；按批次 1–6 给出优先级） |
| keep 待用户/待证据（不确定） | **15** | 保持 backlog，逐条标原因 |

- 动作执行：**abandon 53 条**（a 1 + b 52）、**promote 43 条**（d）、保持 backlog **28 条**（c 13 + keep 15）。
- 分诊后 `normify status`：待办 28、阻塞 46、已验收 287、放弃 74、错误 0。
- **重要发现**：43 条 d 类 promote 后当前全部处于**阻塞**（依赖仍指向旧排程的 backlog/proposed 任务；其中 9 条直接依赖已 abandon 的兄弟任务，共 36 条悬空依赖）。分诊只做状态动作、未硬改依赖，**排程前需要一个依赖清理/重排任务**（见 §5）。
- a 类只有 1 条：本候选池没有大量“真正过时”条目；多数是“已覆盖”或“未实现但可做”。

## 2. 方法

- 数据源：`.normify/changes/*.json`（backlog 124 条）+ 当前任务状态（verified/abandoned）+ 仓内实现事实（`src/` 模块、`tests/`、`docs/`）。
- 判定顺序：
  1. 先查是否有同类 verified 任务/证据覆盖该族/切片 → b；
  2. 族在 `src/` 未实现且属本地可做（无外部硬件/许可前提）→ d；
  3. 需要 Linux RTX 5080、双平台冻结、其他厂商硬件等外部前提 → c；
  4. 旧计划被正式架构/实现取代 → a；
  5. 无法确认范围或需用户裁决 → keep（不强行归类）。
- **b 类依据**：每条在 `triage.json` 的 `evidence` 字段引用覆盖它的 verified 任务 id、证据文件或源码位置（例：G01→layers-m3-rigid-kinematics/G01 双模式验收；G03→g03-matrix-chain-alignment；M0x→layers-m1..m7；A06→launcher/graph.rs）。
- **d 类判定依据**：`src/` 中无对应实现（碰撞 G07–G11、约束/积分 G12–G18、逆动力学/传感器/历史/岛 G20–G25、查询 G28、渲染 R01–R10、执行器 G05、零拷贝 A08、性能工具 A10、局部重置 M08、回调 M10、常量 G27），且均为本地/Windows 单平台可做。
- **c 类判定**：Linux/双平台（D02、D03、D03-final、P0-06、P1、P11、A11）、跨厂商（A12×2）、NP 门禁（NP01–NP04，Windows 部分已交付但未验收、Linux 部分 deferred）。
- 逐条明细（id、标题、phase、依赖、分类、依据、证据、动作、当前状态）见 `triage.json`。

## 3. 各类清单摘要

- **a（1）**：`2026-10-06-plan-engine-architecture`（早期引擎计划架构草案，被 docs/architecture.md + launcher 实现取代）。
- **b（52）**：早期模型字段/阶段拆分条目 + P0/P1/P2/P3 门禁与 M01–M09 切片、A01–A09（已实现部分）、G01/G02/G03/G04/G06/G26 各阶段切片。完整 id 见 `triage.json.actions_executed.abandoned`。
- **c（13）**：`d02`、`d03`、`d03-final`、`p0-06`、`p1`、`p11`、`a11-p11`、`a12-p11`、`a12-p3`、`roadmap-np01`、`roadmap-np02`、`roadmap-np03`、`roadmap-np04`。解锁条件：Linux RTX 5080 + linux-acceptance（用户 2026-10-23 决定 deferred）；A12 另需其他厂商硬件/驱动（首版只要求 NVIDIA）。
- **d（43）**：完整 id 见 `triage.json.actions_executed.promoted`，按批次：
  - **批次 1（基础物理，建议先做）**：G27 常量计算（P2/P7）、M08 局部重置（P3/P7）、G05 执行器（P4/P7）、G28 查询（P7/P8）。
  - **批次 2（约束与积分）**：G12 约束构建、G13 摩擦锥、G14 CG、G15 Newton、G16 Euler/RK4、G17 隐式积分、G18 离散积分。
  - **批次 3（碰撞）**：G07 宽相、G08 基本几何、G09 凸体网格、G10 地形/SDF、G11 柔性碰撞。
  - **批次 4（复杂物理）**：G20 逆动力学、G21 平滑速度导数、G23 传感器、G24 历史与延迟、G25 岛与休眠。
  - **批次 5（渲染）**：R01–R10（含 M03@P8 资产转换）。
  - **批次 6（优化与工具）**：A08 零拷贝、A10 性能工具（P10/P11）、M10 Rust 回调。
- **keep（15）**：G19@P4/P6（分阶段推进语义待证据）、G22@P7（flex 动力学完整性待证据）、M11@P3/P11（错误状态全套范围待证据）、P4–P10 各阶段门禁（依赖未实现族，待批次后再评估）、P9-incremental（伴随实现范围待证据）。均保持 backlog 并标原因。

## 4. 批次建议与执行顺序

1. 先立**依赖清理任务**：把 43 条 promoted 任务的旧 `depends_on` 重排到当前真实状态（去掉指向已 abandon 的兄弟任务、把“已覆盖”的依赖指向 verified 替代、按批次建立真实前置）。**不建议在依赖清理前启动任何 promoted 任务**（当前 43 条全部阻塞）。
2. **批次 1** 建议先做（小、基础、可解锁后续）：G27 常量、M08 局部重置、G05 执行器、G28 查询。每个仍需按规范拆子任务、补 NP01/NP02 映射与上游参考 authoring（现有流程要求）。
3. **批次 2（约束与积分）** 是后续物理链的关键路径（碰撞、逆动力学都依赖它），建议紧随批次 1。
4. **批次 3（碰撞）** 体量最大，建议单独立项、先做只读预检（上游参考与 fixtures 可行性）。
5. **批次 4/5（复杂物理/渲染）** 按用户优先级再定；渲染族（R01–R10）体量最大、与物理链独立，可并行但需独立资源窗口。
6. **批次 6** 可穿插做（工具/优化不阻塞功能链）。

## 5. 悬空依赖（动作前后）

- 动作前：33 条 backlog 依赖指向已 verified/abandoned（见 `triage.json.dangling_dependencies`）。
- 动作后：**36 条**依赖指向已 abandon 的任务（`triage.json.post_action.dangling_after_action`），集中在 promoted 的 d 类（如 G07–G10→P4/P5 旧门禁、G27→P2-CONTRACT、M08→P3-BASE、M03@P8→P8-ENTRY）与 keep 门禁。
- 影响：43 条 promoted 任务在 `normify status` 中显示为**阻塞**（依赖链未验证）；这不是失败，是旧排程依赖未重排。
- 处理建议：另立「backlog 依赖重排」任务，只改 `.normify/changes/*` 的 depends_on（经 CLI 或按规范登记），不 start 任务；或按批次重排时一并处理。

## 6. 风险与限制

- 本分诊是**只读分析 + CLI 状态动作**：b 类按“族/切片已被现有工作覆盖”判断，未对每条做逐项代码级复核；若后续发现覆盖不完整，可按规范 reopen（`normify fix` 不可用于 abandoned，需重新登记）。
- c 类保持 backlog 且 note 已写解锁条件；用户恢复 Linux 环境后可重新评估。
- keep 类为“待用户/待证据”，未做任何动作；其中 P4–P10 门禁需在批次完成后重评。
- `normify status` 的“冲突 468”是大量候选任务文件范围重叠的排程层提示（错误 0），不影响校验。
- 未修改任何产品代码/测试/容差/fixtures/脚本；仅任务 JSON 与证据目录。

## 7. 证据索引

- `triage.json`：逐条 124 项的 id/标题/phase/依赖/分类/依据/证据/动作/当前状态，动作执行清单，动作后状态计数与悬空依赖。
- 本文件：方法、计数、批次建议、风险、后续。
