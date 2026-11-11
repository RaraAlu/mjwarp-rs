# W8-05 CT：P0/工具链准入契约核对（0 窗 CT，只读；零 GPU）

- 任务：`2026-11-11-w8-05-p0-toolchain-ct`（owner Backend）；阶段：**P0/工具链**。
- 性质：**CT 契约核对**——伞卡 acceptance、兼容矩阵行、replication-plan 阶段段、依赖 verified 状态对齐；不生成平台证据、不宣称准入。

## 1. 伞卡

- `2026-10-11-roadmap-p0-06` status=backlog checks=0（acceptance 4 条）
- `2026-10-11-roadmap-d02` status=backlog checks=0（acceptance 5 条）
- `2026-10-11-roadmap-d03` status=backlog checks=0（acceptance 4 条）
- `2026-10-11-roadmap-d03-final` status=backlog checks=0（acceptance 5 条）

## 2. 阶段卡与依赖 verified 状态

### P0/工具链

- 阶段级伞卡：`d02`=backlog/0；`d03`=backlog/0；`p0-06`=backlog/0；`d03-final`=backlog/0
- 阶段切片卡：verified **0/0**；未验收依赖 **3** 个


## 3. 兼容矩阵行（2 行）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 |
| --- | --- | --- | --- |
| M11 | 错误状态 | 全阶段 | 第 382 行 |
| A11 | 双平台发布 | 全阶段/P11 | 第 426 行 |

replication-plan 阶段段：{"126": "P0-06 任务行：冻结双平台环境；Windows 探针已执行、Linux 等待推送；交付=分别验收两种系统"}

## 4. 缺口（4 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-LINUX | 外部裁决 | 用户 2026-10-24 暂停 Linux：`2026-10-24-linux-batch-retest` 为 backlog，D02/D03/P0-06/D03-final 全部依赖它 → 按用户决定记阻塞，不排窗、不开工 |
| G-DEPS | 依赖未验收 | d02/d03/p0-06/d03-final 均 backlog/0；依赖链含 linux-batch-retest、toolchain/ABI 卡等未验收项 |
| G-MATRIX | 矩阵覆盖 | 兼容矩阵无 P0 独立行；工具链/发布相关为 M11（全阶段）与 A11（全阶段/P11），本片只登记映射，不构成准入 |
| G-AD | 证据缺口 | 工具链阶段 AD（双平台环境证据）被 Linux 暂停阻塞，当前无法生成 |

**状态：等待用户裁决（Linux 暂停）** —— Linux 批次被用户 2026-10-24 暂停 → P0/工具链全部阻塞；只做 CT 登记，不排窗。

## 5. 结论

CT 完成：P0/工具链 4 卡（d02/d03/p0-06/d03-final）全部 backlog/0，依赖 `linux-batch-retest`（用户暂停）；矩阵映射登记为 M11/A11（无 P0 独立行）。按用户决定记阻塞，不宣称准入、不排窗。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`admission_claim=false`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称任何阶段准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
