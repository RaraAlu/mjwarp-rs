# W8-02 CT：P4 准入契约核对（0 窗 CT，只读；零 GPU）

- 任务：`2026-11-11-w8-02-p4-ct`（owner Backend）；阶段：**P4**。
- 性质：**CT 契约核对**——伞卡 acceptance、兼容矩阵行、replication-plan 阶段段、依赖 verified 状态对齐；不生成平台证据、不宣称准入。

## 1. 伞卡

- `2026-10-11-roadmap-p4` status=backlog checks=0（acceptance 8 条）

## 2. 阶段卡与依赖 verified 状态

### P4

- 阶段级伞卡：`p4`=backlog/0
- 阶段切片卡：verified **0/1**；未验收依赖 **1** 个


## 3. 兼容矩阵行（8 行）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 |
| --- | --- | --- | --- |
| G01 | 运动学 | P4 | 第 330 行 |
| G02 | 空间动力学 | P4 | 第 331 行 |
| G03 | 质量矩阵求解 | P4/P6 | 第 332 行 |
| G04 | 被动力 | P4 | 第 333 行 |
| G05 | 执行器 | P4/P7 | 第 334 行 |
| G06 | 肌腱与传动 | P4/P7 | 第 335 行 |
| G19 | 分阶段推进 | P4/P6 | 第 348 行 |
| G26 | 能量与外力 | P4/P7 | 第 355 行 |

replication-plan 阶段段：{"270": "P4 阶段行：空间数学、运动学、质量矩阵；前置 P2/P3；交付=无接触单步与解析解一致"}

## 4. 缺口（5 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-T4 | 外部裁决 | T4 重基线 A/B/C（`t4-historical-capture-rebaseline-decision`）未裁 → W8-02 P4 的 AD 口径待裁，不写准入 |
| G-NP | NP 门禁 | NP01/NP02 全局未验收 → P4 AD 的 NP 前置不满足 |
| G-DEPS | 依赖未验收 | p4 的 20 项依赖中含 g19-p4（backlog/0）等未验收项；g05-p4 已 verified 可作输入（以板面为准） |
| G-GATE | 矩阵覆盖 | P4 矩阵行 G01–G06/G19/G26 的逐行证据分散在大量历史片（部分 abandoned/verified），并非全部有当前验收证据；不得用历史子集顶替实际产物（d05） |
| G-AD | 证据缺口 | P4 阶段 AD（Windows debug+release 双模式）未生成；按模板 AD 需 CT 无缺口 + NP 门禁 + 窗口 |

**状态：等待 T4 重基线裁决 + NP 门禁** —— AD 记「等待 T4 重基线裁决 + NP 门禁」；本片只做 CT，不宣称准入。

## 5. 结论

CT 完成：P4 伞卡 acceptance、矩阵 8 行、依赖 verified 状态与 replication-plan P4 段已对齐；缺口 5 项（T4 裁决、NP 门禁、依赖未验收、矩阵覆盖、AD 证据）。AD 等待 T4 重基线裁决 + NP 门禁，不宣称准入。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`admission_claim=false`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称任何阶段准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
