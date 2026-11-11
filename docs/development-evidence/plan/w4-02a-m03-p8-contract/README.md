# W4-02A m03-p8 资产转换契约核对（CT，0 窗只读；零 GPU）

- 任务：`2026-11-11-w4-02a-m03-p8-contract`（owner Backend）；卡片对象：`2026-10-11-roadmap-m03-p8`（**status=proposed / checks=0**）。
- 性质：**契约核对（CT）**——只把「卡面 acceptance + 兼容矩阵行 + 实施计划段 + 依赖 verified 状态」对齐，
  输出适用切片清单/覆盖对照/缺口；**不生成平台证据、不宣称准入通过**。

## 1. 依据与来源

- 卡面 acceptance（8 条）：见 `contract.json` 的 `umbrella_acceptance`。
- 兼容矩阵（`docs/compatibility-matrix.md`）：本片适用行见 §3（自 `resolved_sources.matrix_rows` 解析）。
- 实施计划（`docs/replication-plan.md`）：{"268": "P2 阶段行：输入检查、字段转换、资产转换", "274": "P8 阶段行：批量图像、BVH、设备视图"}。
- 卡面声明来源：`[{"file": "docs/compatibility-matrix.md", "start": 295, "end": 295}, {"file": "docs/replication-plan.md", "start": 265, "end": 276}]`。

## 2. 适用切片清单（W4-xx 编号）

| 片 | 内容 | 窗 |
| --- | --- | --- |
| W4-02A | 契约核对：单位/路径/内容摘要口径 + 矩阵覆盖（本片，0 窗） | 0 |
| W4-02B | 单位/路径/摘要实现或核对（0 窗；若触发设备路径则 +1 窗） | 0 |
| W4-02C | 证据卡 + QA 复核 | 0 |

## 3. 覆盖对照（矩阵行 × 卡面 acceptance）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件行 | 证据状态 |
| --- | --- | --- | --- | --- |
| M03 | 资产转换 | P2/P7/P8 | 374 | CT-only |
| R01 | 渲染上下文 | P8 | 394 | CT-only |
| M11 | 错误状态 | 全阶段 | 382 | CT-only |

- 卡面 acceptance 的对齐：`核对适用切片与矩阵`（§2/§3）、`先完成范围内NP门禁`（§4 NP 门禁）、
  `保留双模式平台证据`（片 B，未做）、`按适用范围检查M11`/`保存A11证据`（AD 时核对）。
- 依赖 verified 状态：`p8`=verified。

## 4. 缺口（4 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-NP | NP 门禁 | NP01/NP02 未验收 → 卡面 acceptance「先完成范围内 NP 门禁」未满足，结论记等待 NP 门禁 |
| G-SRC | 来源指针漂移 | 卡面来源 `docs/compatibility-matrix.md:295` 当前指向「首轮样本集」表，不再指向 M03 行；M03 实际锚点=矩阵第 374 行（模型与数据矩阵）；语义按矩阵 ID M03 解析（非内容缺口） |
| G-CONTRACT | 契约缺口 | 单位/路径/内容摘要三项的具体断言与验收口径未落地（片 B），本片只列契约对象与覆盖对照 |
| G-R01 | 前置边界 | 资产路径确认依赖 R01 渲染上下文（verified 卡 r01-p8）；m03-p8 卡面 depends_on=r01-p8（verified），无未验收依赖 |

**NP 门禁状态：等待 NP 门禁**（NP01/NP02 未验收；NP5-00…04 仅为证据卡，NP 伞卡未 verified）→
本片结论只能到 CT，不构成准入通过。

## 5. 结论

CT 完成：资产转换单位/路径/摘要契约对象与矩阵覆盖已列，缺口 4 项；不构成准入通过，等待 NP 门禁。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`np_gate_status=等待 NP 门禁`/
  `admission_claim=false`、manifest 4 文件自核、policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录+`.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差、不改任何矩阵或计划文件。
- 不宣称 W4/P8 准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
