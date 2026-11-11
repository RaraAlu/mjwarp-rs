# W8-00 覆盖审计（0 窗只读）：阶段伞卡 × 兼容矩阵行 × 依赖 verified 状态 × 缺口总表

- 任务：`2026-11-11-w8-00-coverage-audit`（owner Backend）；基线 HEAD `f9eb64e1`；日期 2026-11-11。
- 性质：**只读覆盖审计**（零 GPU、零窗）；数据来源：板面 `.normify/changes/**`（1127 卡）、
  `docs/compatibility-matrix.md`（`e68c4456` / 30782 B，61 个矩阵行）、
  W8/W4 拆解工作材料（target，仅本机可复现）。
- **结论摘要**：全部阶段级伞卡 **`checks=0`，均不可 start**；W4 片 B 与 W8 AD 的**前置均未满足**
  （NP01/NP02 未验收、r07-p8 待 ABI、Linux/T4/U8U7 待裁）；本审计**不构成任何准入通过**，
  也不宣称 NP01/NP02 通过、等价、双平台或发布（P11 为用户级）。

## 1. 第一节：NP5-ERRATA-E QA 观察 O1/O2 处置（append-only）

- **O1（已落）**：`np5-e-metric-errata/addenda.md`（新）明确 `corrections.json.historical_snapshot`
  是**归一化摘要、非逐字引用**；逐字原文以历史文件字节为准。本目录 manifest **4→5** 条重算，
  errata 目录 4 个历史文件 sha/bytes 不变。
- **O2（记录不修）**：4 条 W5 working docs 更正位于 `target/`（不入仓），只能本机按 sha 复现；
  记录在 errata receipt `working_docs` 与本报告，无需改动。

## 2. 阶段 × 兼容矩阵行（61 行）

| 阶段 | 矩阵行（id） | 行数 |
| --- | --- | --- |
| P0/工具链 | （矩阵无该阶段独立行） | 0 |
| P4 | G01、G02、G03、G04、G05、G06、G19、G26 | 8 |
| P5 | G07、G08、G09、G10 | 4 |
| P6 | G03、G12、G13、G14、G15、G16、G17、G18、G19 | 9 |
| P7 | G05、G06、G11、G20、G21、G22、G23、G24、G25、G26、G27、G28、M03、M08、M09、M10 | 16 |
| P8 | G28、M03、R01、R02、R03、R04、R05、R06、R07、R08、R09、R10、A07 | 13 |
| P9 | M07、A09 | 2 |
| P10 | G25、M05、M09、M10、R10、A04、A05、A06、A07、A08、A09、A10 | 12 |
| P11 | A10、A11、A12 | 3 |

矩阵分段：{"GPU物理矩阵": 28, "模型与数据矩阵": 11, "渲染矩阵": 10, "运行时与数据接口矩阵": 12}；全阶段行（M11/A11）另计；矩阵中 P1/P2/P3 行（P1 3 + P2 5 + P3 8 = 16 行）不在 W8 阶段集，未列入本表（audit.json matrix.by_stage 保留全量）。
映射口径：矩阵「阶段」列 token（`P4`…`P11`、`全阶段`）；一行可跨阶段（如 G03 P4/P6、A11 全阶段/P11）。

## 3. 阶段伞卡 × 依赖 verified 状态

| 阶段 | 阶段级伞卡（status/checks） | 阶段切片卡 verified/总数 | 未验收依赖数 |
| --- | --- | --- | --- |
| P0/工具链 | `d02`=backlog/0；`d03`=backlog/0；`p0-06`=backlog/0；`d03-final`=backlog/0 | 0/0 | 3 |
| P4 | `p4`=backlog/0 | 0/1 | 1 |
| P5 | `p5`=backlog/0 | 3/4 | 1 |
| P6 | `p6`=backlog/0 | 2/8 | 10 |
| P7 | `p7`=backlog/0 | 6/10 | 9 |
| P8 | `p8`=backlog/0；`p8-base`=backlog/0；`p8-entry`=backlog/0 | 8/12 | 3 |
| P9 | `p9`=backlog/0；`p9-incremental`=backlog/0 | 0/0 | 1 |
| P10 | `p10`=backlog/0 | 2/5 | 4 |
| P11 | `p11`=backlog/0 | 0/3 | 35 |

合计：阶段级伞卡 **15**（其中 `checks=0` **15**）；
阶段切片卡 **43**（verified **21**）；
去重未验收依赖 **41** 个（proposed/backlog/abandoned 或不在板面）。

**判定口径**：先决条件**只认 verified**；`checks=0` 的伞卡不能直接 start（须先登记实体卡或由
verified 收口卡承担）——本报告全部伞卡均 `checks=0`。

## 4. 缺口总表（12 项）

| id | 类别 | 说明 | 状态 |
| --- | --- | --- | --- |
| W8-G1 | 伞卡占位 | 全部阶段级伞卡 checks=0（proposed/backlog），不可 start；实体片须先登记或由 verified 收口卡承担 | 记录（不可 start） |
| W8-G2 | NP 门禁 | NP01/NP02 未验收（NP5-00…04 只是证据卡，伞卡未 verified）→ W4 片 B 与 W8 AD 的 NP 前置未满足 | 阻塞 AD 与 W4 片 B |
| W8-G3 | 渲染 ABI | r07-p8（proposed,0）待 R07 ABI（81 槽 splat 拆分 + S3b 另签）用户签名 → 卡 r10-p8/p8-base 与 W4-03/06 片 B | 阻塞（用户签） |
| W8-G4 | Linux | 用户 2026-10-24 暂停 Linux：P0/工具链 4 卡（d02/d03/p0-06/d03-final）、W7、NP03/04 与 P11 双平台均 deferred | 阻塞（用户裁决） |
| W8-G5 | T4 重基线 | t4-historical-capture-rebaseline-decision A/B/C 未定 → W8-02 P4 AD 口径待裁 | 阻塞 P4 AD（用户裁决） |
| W8-G6 | W2 判据 | U8/U7 是否提升 pass/fail + A2 容量 20/10→32/32 未裁 → W2/W8 阶段完成判据口径待定（与 P5/P6 相关） | 阻塞（用户裁决） |
| W8-G7 | P11 发布 | P11 全量双平台发布准入 = 用户级发布声明；前置 NP01–NP04、双平台与清洁部署（A11 全阶段） | 用户级，不排窗 |
| W8-G8 | g10-p5 伞卡 | g10-p5 伞卡仍 proposed/0；可用 verified 的 `2026-11-04-g10-p5-family-closeout` 作 CT 依据（B0-2 已改指），AD 前仍需 NP 碰撞范围门禁 | 记录（有 verified 代理收口卡） |
| W8-G9 | 未验收依赖 | P6/P7/P11 组存在大量未验收依赖（proposed/backlog/abandoned），详见各阶段 unverified_deps | 依赖链阻塞 |
| W8-G10 | 窗口 | W8 AD 片需 GPU 窗（每阶段约 1 窗，窗口规则同 W0）；当前 0 窗、不排窗 | 不排窗 |
| W8-G11 | W4 片 B | W4 片 B（r10-p8 / p8-base / p8 的 GPU 证据片）受 r07-p8 + NP01/NP02 双重阻塞，不得提前开窗 | 阻塞（双前置） |
| W8-G12 | W8 AD 前置 | W8 AD 需 CT 无缺口 + 范围内 NP 门禁已过；当前两者均未满足 | 阻塞 |

## 5. W4 侧前置状态（核对）

- **7 张 W4 伞卡全部 `checks=0`**（r10-p8/r10-p10/m03-p8/m10-p10/p8-entry/p8-base/p8）——
  均不可直接 start；实体片 25 = B0 4 + 实体 21（7 卡 × 3 片，§9 已更正「24→25」）。
- **片 B 双重阻塞**：r07-p8（proposed/0，待 R07 ABI 81 槽用户签）+ **NP01/NP02 未验收**
  （NP5-00…04 为证据卡，NP 伞卡未 verified）→ 片 B 一律不碰、不得提前开窗。
- **现可派（0 窗）**：W4-01A p8-entry / W4-02A m03-p8 / W4-05A m10-p10 / W4-03A r10-p8 /
  W4-04A r10-p10 的契约核对（本队列后续 5 片）。

## 6. 结论

- 全部阶段级伞卡 `checks=0`，**不可 start**；W4 片 B 与 W8 AD 前置未满足；
  **不得写成准入通过**、不得宣称阶段 verified。
- 可立即做：各阶段 **CT（契约/覆盖核对，0 窗只读）**；**AD（准入证据）**需先满足
  CT 无缺口 + NP 门禁 + 窗口规则；P11 为用户级发布声明、不排窗。
- 需用户裁决：Linux 批次恢复、T4 重基线 A/B/C、U8/U7 判据提升、R07 ABI、P11 发布口径。

## 7. 验证

- selfcheck（`logs/selfcheck.txt`）：板面计数、矩阵 61 行、阶段映射、伞卡 `checks=0`、
  依赖未验收清单、缺 12 项、errata addenda 与 manifest 4→5、历史字节不变、policy_scan 0 命中。
- 编码 UTF-8/LF 无 BOM；manifest 4 文件（README/audit.json/receipt.json/logs/selfcheck.txt；不列自身）。

## 8. 边界与非声明

- 只读审计（零 GPU、零窗）；写域仅本目录 + `np-gate/np5-e-metric-errata/` + `.normify/`；
  不改任何卡、不改判据/参考/容差。
- 不宣称任何阶段准入通过/伞卡 verified、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
