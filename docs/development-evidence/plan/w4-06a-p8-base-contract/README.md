# W4-06A p8-base 基础批量渲染准入核对（CT）（0 窗 CT，只读；零 GPU）

- 任务：`2026-11-11-w4-06a-p8-base-contract`（owner Backend）；伞卡：`2026-10-11-roadmap-p8-base`（**status=backlog / checks=0**）。
- 性质：**CT 契约核对**——卡面 acceptance、兼容矩阵行、replication-plan 阶段段、依赖 verified 状态对齐；不生成平台证据、不宣称准入。

## 1. 卡面 acceptance（8 条）

- 核对适用切片与矩阵。
- 先完成范围内NP门禁。
- 保留双模式平台证据。
- 按适用范围检查M11。
- 按适用范围保存A11证据。
- 记录真实测试与未测项。
- 先核对NP01与NP02。
- 再验收适用NP03与NP04。

## 2. 依赖 verified 状态（10 项）

| 依赖 | 卡 id | status |
| --- | --- | --- |
| `p8` | `2026-10-11-roadmap-r01-p8` | verified |
| `p8` | `2026-10-11-roadmap-r02-p8` | verified |
| `p8` | `2026-10-11-roadmap-r03-p8` | verified |
| `p8` | `2026-10-11-roadmap-r04-p8` | verified |
| `p8` | `2026-10-11-roadmap-r05-p8` | verified |
| `p8` | `2026-10-11-roadmap-r07-p8` | proposed |
| `p8` | `2026-10-11-roadmap-r08-p8` | verified |
| `p8` | `2026-10-11-roadmap-r09-p8` | verified |
| `transfers` | `2026-10-07-batched-device-transfers` | verified |
| `closeout` | `2026-11-04-render-r05-r09-r10-family-closeout` | verified |

## 3. 兼容矩阵行（13 行）

| 矩阵行 | 功能族 | 阶段 | 矩阵文件锚点 |
| --- | --- | --- | --- |
| R01 | 渲染上下文 | P8 | 第 394 行 |
| R02 | RGB输出 | P8 | 第 395 行 |
| R03 | 深度输出 | P8 | 第 396 行 |
| R04 | 分割输出 | P8 | 第 397 行 |
| R05 | 网格与地形 | P8 | 第 398 行 |
| R06 | 柔性体渲染 | P8 | 第 399 行 |
| R07 | 高斯泼溅 | P8 | 第 400 行 |
| R08 | 异构多相机 | P8 | 第 401 行 |
| R09 | 光照与阴影 | P8 | 第 402 行 |
| R10 | 批量渲染调度 | P8/P10 | 第 403 行 |
| A07 | 设备数据视图 | P8/P10 | 第 422 行 |
| M11 | 错误状态 | 全阶段 | 第 382 行 |
| A11 | 双平台发布 | 全阶段/P11 | 第 426 行 |

replication-plan 阶段段：{"274": "P8 阶段行：批量图像、BVH、设备视图；前置 P2/P3，柔性项需 P7；交付=图像布局与完成事件正确"}

## 4. 缺口（5 项）

| id | 类别 | 说明 |
| --- | --- | --- |
| G-R07 | 前置未验收 | r07-p8 仍 proposed/0（待 R07 ABI 81 槽 + S3b 用户签）；按 W4-B0/卡 I 只登记 blocked-by-r07-p8 事实、**不 waive、不写准入**；p8-base 的 8 条 r-p8 依赖中 7 条 verified（r01–r05/r08/r09）、r07 为 proposed |
| G-NP | NP 门禁 | NP01/NP02 全局未验收 → 卡面 acceptance「先完成范围内 NP 门禁」未满足，片 B 不得开窗 |
| G-AD | 证据缺口 | 双模式平台证据（Windows debug+release）未生成（片 B）；A11/M11 适用范围检查未做 |
| G-SRC | 来源指针漂移 | 卡面来源指针中的阶段行号漂移（P8 实际锚点=replication-plan 第 274 行）；按阶段行语义解析、非内容缺口（同 W8 CT 口径） |
| G-UMBRELLA | 伞卡占位 | p8-base 伞卡 status=backlog / checks=0，不可直接 start；本片为 CT 契约核对，不启动伞卡 |

**状态：等待 ABI 裁决 + NP 门禁** —— r07-p8 待 R07 ABI 用户签（W4-B0 只登记事实、未 waive）+ NP01/NP02 未验收 → 不写准入。

## 5. 结论

CT 完成：8 条 r-p8 依赖中 7 verified（r01–r05/r08/r09）、r07-p8 proposed 记「等待 ABI 裁决」；batched-device-transfers verified、r05-r09-r10 family closeout verified；缺口 5 项。不构成 p8-base 准入通过。

## 6. 验证

- selfcheck（`logs/selfcheck.txt`）：contract 字段/矩阵行/缺口计数/`admission_claim=false`、manifest 4 文件自核、
  policy_scan 0 命中、编码 UTF-8/LF 无 BOM、边界仅本目录 + `.normify`。
- 四门禁：fmt/clippy/clippy-cuda/unit。

## 7. 边界与非声明

- 零 GPU、零窗；只写本目录 + `.normify/`；不改卡面 acceptance/判据/参考/容差/矩阵/计划文件。
- 不宣称任何阶段准入通过、不宣称 NP01/NP02 通过、不宣称等价/双平台/发布。
