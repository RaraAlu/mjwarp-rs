# W4-04A r10-p10 批量渲染调度（P10）契约核对（CT，0 窗只读；零 GPU）

- 任务：`2026-11-11-w4-04a-r10-p10-contract`（owner Backend）；卡片对象：`2026-10-11-roadmap-r10-p10`（**status=proposed / checks=0**）。
- 性质：**契约核对（CT）**——只把「卡面 acceptance + 兼容矩阵行 + 实施计划段 + 依赖 verified 状态」对齐，
  输出适用切片清单/覆盖对照/缺口；**不生成平台证据、不宣称准入通过**。

## 1. 依据与来源

- 卡面 acceptance（8 条）：见 `contract.json` 的 `umbrella_acceptance`。
- 兼容矩阵（`docs/compatibility-matrix.md`）：本片适用行见 §3（自 `resolved_sources.matrix_rows` 解析）。
- 实施计划（`docs/replication-plan.md`）：{"276": "P10 阶段行：图重放、缓存失效、性能；前置 P6–P9"}。
- 卡面声明来源：`[{"file": "docs/compatibility-matrix.md", "start": 324, "end": 324}, {"file": "docs/replication-plan.md", "start": 265, "end": 276}]`。

## 2. 适用切片清单（W4-xx 编号）

| 片 | 内容 | 窗 |
| --- | --- | --- |
| W4-04A | 契约核对：独立缓冲 + 完成令牌 + P10 调度前提（本片，0 窗） | 0 |
| W4-04B | 批量调度实体验收（P10，Windows 单平台） | 1 |
| W4-04C | 证据卡 + QA 复核 | 0 |

## 3. 覆盖对照（矩阵行 × 卡面 acceptance）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件行 | 证据状态 |
| --- | --- | --- | --- | --- |
| R10 | 批量渲染调度 | P8/P10 | 403 | CT-only |
| A04 | 多流与事件 | P3/P10 | 419 | CT-only |
| A07 | 设备数据视图 | P8/P10 | 422 | CT-only |
| A09 | 通用调用方接口 | P9/P10 | 424 | CT-only |
| M11 | 错误状态 | 全阶段 | 382 | CT-only |

- 卡面 acceptance 的对齐：`核对适用切片与矩阵`（§2/§3）、`先完成范围内NP门禁`（§4 NP 门禁）、
  `保留双模式平台证据`（片 B，未做）、`按适用范围检查M11`/`保存A11证据`（AD 时核对）。
- 依赖 verified 状态：`closeout`=verified。

## 4. 缺口（4 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-NP | NP 门禁 | NP01/NP02 未验收 → 片 B 不得开窗，结论记等待 NP 门禁 |
| G-PRED | 前置未验收 | r10-p8 未验收（proposed/0）→ r10-p10 的前置链未满足；r05-r09-r10 family closeout 已 verified 可作输入引用 |
| G-SRC | 来源指针漂移 | 卡面来源 `docs/compatibility-matrix.md:324` 当前指向「完整对标的定义」段，不再指向 R10 行；R10 实际锚点=矩阵第 403 行；语义不变、按矩阵 ID 解析（非内容缺口） |
| G-CONTRACT | 契约缺口 | 独立缓冲/完成令牌契约断言未落证；P10 调度额外依赖执行图/缓存（A05/A06）与 A09 调用方契约，AD 时一并核对 |

**NP 门禁状态：等待 NP 门禁**（NP01/NP02 未验收；NP5-00…04 仅为证据卡，NP 伞卡未 verified）→
本片结论只能到 CT，不构成准入通过。

## 5. 结论

CT 完成：契约对象与矩阵覆盖已列，缺口 4 项；不构成准入通过；前置 r10-p8 未验收 + NP 门禁未满足。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`np_gate_status=等待 NP 门禁`/
  `admission_claim=false`、manifest 4 文件自核、policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录+`.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差、不改任何矩阵或计划文件。
- 不宣称 W4/P8 准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
