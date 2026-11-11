# W8-06 CT：P11 发布准入契约核对（CT only，不写发布声明）（0 窗 CT，只读；零 GPU）

- 任务：`2026-11-11-w8-06-p11-ct`（owner Backend）；阶段：**P11**。
- 性质：**CT 契约核对**——伞卡 acceptance、兼容矩阵行、replication-plan 阶段段、依赖 verified 状态对齐；不生成平台证据、不宣称准入。

## 1. 伞卡

- `2026-10-11-roadmap-p11` status=backlog checks=0（acceptance 12 条）
- `2026-10-11-roadmap-a11-p11` status=backlog checks=0（acceptance 6 条）
- `2026-10-11-roadmap-a12-p11` status=backlog checks=0（acceptance 8 条）
- `2026-10-11-roadmap-m11-p11` status=backlog checks=0（acceptance 8 条）

## 2. 阶段卡与依赖 verified 状态

### P11

- 阶段级伞卡：`p11`=backlog/0
- 阶段切片卡：verified **0/3**；未验收依赖 **35** 个


## 3. 兼容矩阵行（4 行）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 |
| --- | --- | --- | --- |
| A10 | 记录与性能工具 | P10/P11 | 第 425 行 |
| A11 | 双平台发布 | 全阶段/P11 | 第 426 行 |
| A12 | 跨厂商预留 | P3/P11 | 第 427 行 |
| M11 | 错误状态 | 全阶段 | 第 382 行 |

replication-plan 阶段段：{"277": "P11 阶段行：全量矩阵与清洁部署；前置=前述全部阶段；交付=双平台通过全部正式准入"}

## 4. 缺口（5 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-USER | 用户级发布 | P11 = 全量双平台发布准入 = **用户级发布声明**；Bot 不得自行认定、本片**不写任何发布声明** |
| G-NP | NP 门禁 | NP01–NP04 未验收（NP03/04 依赖 Linux 且 deferred）；NP01/02 未通过前不得提 P11 发布声明 |
| G-LINUX | 双平台 | Linux 被用户 2026-10-24 暂停（W7 deferred）→ 双平台前提不成立 |
| G-DEPS | 依赖未验收 | p11 依赖 98 项，含大量 proposed/backlog/abandoned；a11/a12/m11-p11 均 backlog/0 |
| G-AD | 证据缺口 | P11 AD（双平台全量准入证据）未生成且当前不可生成（Linux 暂停）；本片只做 CT 契约核对 |

**状态：用户级（CT only，不写发布声明）** —— P11 发布声明为用户级；当前 NP/Linux/双平台前提不满足，本片不写、不暗示任何发布结论。

## 5. 结论

CT 完成：P11 伞卡 4 张（p11/a11/a12/m11-p11）全 backlog/0、依赖 98 项未验收、矩阵 A10/A11/A12 映射在案；缺口 5 项。P11 发布声明为用户级，本片**不写任何发布声明**。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`admission_claim=false`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称任何阶段准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布（尤指 P11 发布声明）。
