# W4-01A p8-entry 契约核对（无头渲染输入准入）（CT，0 窗只读；零 GPU）

- 任务：`2026-11-11-w4-01a-p8-entry-contract`（owner Backend）；卡片对象：`2026-10-11-roadmap-p8-entry`（**status=backlog / checks=0**）。
- 性质：**契约核对（CT）**——只把「卡面 acceptance + 兼容矩阵行 + 实施计划段 + 依赖 verified 状态」对齐，
  输出适用切片清单/覆盖对照/缺口；**不生成平台证据、不宣称准入通过**。

## 1. 依据与来源

- 卡面 acceptance（9 条）：见 `contract.json` 的 `umbrella_acceptance`。
- 兼容矩阵（`docs/compatibility-matrix.md`）：本片适用行见 §3（自 `resolved_sources.matrix_rows` 解析）。
- 实施计划（`docs/replication-plan.md`）：{"107-108": "P8 探针可在 P3 后先行；完整柔性渲染依赖 P7", "274": "P8 阶段行：批量图像、BVH、设备视图；前置 P2/P3，柔性项需 P7；首个可观察交付=图像布局与完成事件正确"}。
- 卡面声明来源：`[{"file": "docs/replication-plan.md", "start": 107, "end": 108}, {"file": "docs/replication-plan.md", "start": 273, "end": 273}]`。

## 2. 适用切片清单（W4-xx 编号）

| 片 | 内容 | 窗 |
| --- | --- | --- |
| W4-01A | 契约核对：适用切片 + 矩阵覆盖 + 缺口（本片，0 窗） | 0 |
| W4-01B | 双模式平台证据（Windows debug+release），窗口规则同 W0 | 1 |
| W4-01C | 证据卡 + QA 复核 | 0 |

## 3. 覆盖对照（矩阵行 × 卡面 acceptance）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件行 | 证据状态 |
| --- | --- | --- | --- | --- |
| R01 | 渲染上下文 | P8 | 394 | CT-only |
| R02 | RGB输出 | P8 | 395 | CT-only |
| R03 | 深度输出 | P8 | 396 | CT-only |
| R04 | 分割输出 | P8 | 397 | CT-only |
| R05 | 网格与地形 | P8 | 398 | CT-only |
| R06 | 柔性体渲染 | P8 | 399 | CT-only |
| R07 | 高斯泼溅 | P8 | 400 | CT-only |
| R08 | 异构多相机 | P8 | 401 | CT-only |
| R09 | 光照与阴影 | P8 | 402 | CT-only |
| R10 | 批量渲染调度 | P8/P10 | 403 | CT-only |
| A07 | 设备数据视图 | P8/P10 | 422 | CT-only |
| M11 | 错误状态 | 全阶段 | 382 | CT-only |
| A11 | 双平台发布 | 全阶段/P11 | 426 | CT-only |

- 卡面 acceptance 的对齐：`核对适用切片与矩阵`（§2/§3）、`先完成范围内NP门禁`（§4 NP 门禁）、
  `保留双模式平台证据`（片 B，未做）、`按适用范围检查M11`/`保存A11证据`（AD 时核对）。
- 依赖 verified 状态：`transfers`=verified；`m3`=verified；`m5`=verified；`wiring`=verified。

## 4. 缺口（5 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-NP | NP 门禁 | NP01/NP02 未验收 → 按卡面 acceptance「先完成范围内 NP 门禁」，片 B 不得开窗，结论只能记「等待 NP 门禁」 |
| G-ROLEM | 来源指针漂移 | 卡面 note 的来源指针 `docs/replication-plan.md:107-108,273` 中 273 当前为 P7 阶段行；P8 阶段行为第 274 行。语义不变，按阶段行语义解析（非内容缺口） |
| G-AD | 证据缺口 | 双模式平台证据（debug/release）与 A11/M11 适用范围检查未做（片 B/CT 后置），本片只有契约核对 |
| G-FLEX | 依赖缺口 | 完整柔性渲染依赖 P7（G11/G22 未验收）；无头基础渲染可先行的前提是「只开放无头基础渲染」 |
| G-R07 | 渲染 ABI | R07 splat/ABI 待用户签（r07-p8 proposed）；p8-entry 卡面不直接依赖 r07-p8，但 R07 矩阵行属 P8 范围，AD 前需处置 |

**NP 门禁状态：等待 NP 门禁**（NP01/NP02 未验收；NP5-00…04 仅为证据卡，NP 伞卡未 verified）→
本片结论只能到 CT，不构成准入通过。

## 5. 结论

CT 完成：适用切片（A/B/C）与矩阵覆盖已列，缺口 5 项（NP 门禁、来源指针漂移、AD 证据、柔性依赖、R07 ABI）；不构成准入通过，片 B 等待 NP 门禁。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`np_gate_status=等待 NP 门禁`/
  `admission_claim=false`、manifest 4 文件自核、policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录+`.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差、不改任何矩阵或计划文件。
- 不宣称 W4/P8 准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
