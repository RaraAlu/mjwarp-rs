# NP5-04 addendum（NP5-ERRATA-E 卡，2026-11-11）：E 判据状态 append-only 更正

本文件为 **append-only 更正**：NP5-04 的 `README.md`/`evidence.json`/`receipt.json`/`logs/selfcheck.txt`
历史字节不改（`manifest.txt` 重算为 5 条；历史 4 文件 sha/bytes 与 errata 卡 receipt 的 `pre_change_sha256`
逐项一致）；现行口径以本 addendum 为准。任务：`2026-11-11-np5-e-metric-errata`（登记 `40b7f16a`）。

## 1. 更正依据

- 用户 2026-11-05 签署 **E 判据 A2（选项①「同码跨后端核对」，签名级）**：卡
  `2026-11-05-precision-np01-e-criterion-a2`、提交 `b9cb2993`；契约
  `docs/numerical-precision-contract.md` L267-L272 与「### E指标定稿（NP01，用户2026-11-05拍板选项①）」。
- 口径：判据「实测 worst ≤ 已签 atol」，不用 E 比值；范围 CONVEX/SDF × dist/pos/frame（normal 并入
  frame），**97 标量**；**不宣称 native**。

## 2. 现行口径

- NP01 状态：**未验收**（结论不变）——类型偏离 0 项已达成；精度偏离 0 项未完成，理由是 **A2 判据已签、
  全产品 30 路径 E/atol 核对未完成**，加上跨 recipe 位级判据前提不成立、device 级仅可映射子集、
  7 未实现子系统与 Linux 双平台阻塞。
- E 裁决点性质变化：**E 判据定义已签 A2（2026-11-05）**；仍待用户裁决的是「是否确认 A2 即
  NP01 判据定稿 + 是否把 E/atol 核对扩到全产品 30 路径」（材料包 05 选项 A/B/C 的范围部分）。
  裁决点**总数仍为 7**（`e-metric` 保留，只更新主题/阻塞描述）。
- `any_gate_passed` 仍 `false`；NP02 状态、32/32 处置、4 未实现子系统 blocked 全部不变。

## 3. 逐处更正（历史字节保留在下列位置，不回改）

下表 7 条与 `np5-e-metric-errata/corrections.json` 的 `repo_evidence[]` 中 `file` 为 np5-04 的条目一一对应。

| # | 位置（历史文件:行） | 历史表述（当时快照，已过时） | 现行口径 |
| --- | --- | --- | --- |
| 1 | `evidence.json:16`（np01.status） | 「…整体「NP01 精度偏离 0 项」未完成（E 未定稿）」 | 「…未完成（A2 判据已签 2026-11-05；全产品 E/atol 核对未完成）」 |
| 2 | `evidence.json:90`（watch-np04 items[1]） | 「…若 MathDx 变默认路线需在 E 定稿后重评」 | 「…需在全产品 E/atol 核对后重评（A2 判据已签 2026-11-05）」 |
| 3 | `evidence.json:133`（user_decision_points[e-metric].topic） | 「E 指标定稿（2026-10-27 口径：G6 首步用产品输出对 native f64 重算）」 | 「确认 A2（2026-11-05，选项①，97 标量、不宣称 native）为 NP01 E 判据定稿 + 是否扩全产品 30 路径 E/atol 核对（材料包 05 选项 A/B/C）」 |
| 4 | `evidence.json:151`（conclusion.np01_status） | 「未验收（…精度偏离 0 项未完成，E 未定稿）」 | 「未验收（…精度偏离 0 项未完成——A2 判据已签 2026-11-05、全产品 E/atol 核对未完成）」 |
| 5 | `README.md:15`（§1 表·精度偏离行） | 「未完成（E 指标未定稿；跨 recipe 位级前提不成立；device 可映射子集）」 | 「未完成（A2 判据已签 2026-11-05；全产品 E/atol 核对未完成；跨 recipe 位级前提不成立；device 可映射子集）」 |
| 6 | `README.md:49`（§4 表 e-metric 行） | 「E 指标定稿 / 2026-10-27 口径（G6 首步产品输出对 native f64 重算） / 用户裁决」 | 「E 判据确认与扩范围 / A2 已于 2026-11-05 签署（选项①，97 标量）；待确认定稿 + 是否扩全产品 30 路径核对 / 用户裁决」 |
| 7 | `receipt.json:6`（np01_status） | 「未验收（…精度偏离 0 项未完成，E 未定稿）」 | 「未验收（…A2 判据已签 2026-11-05、全产品 E/atol 核对未完成）」 |

## 4. 不变项

- 分类/结论/数据不改：`any_gate_passed` 仍 `false`、`precision_deviation_zero_items_verified` 仍 `false`、
  NP02 inventory 102/98、32 缺口处置 9/2/3/4/6/2/2/4、4 未实现子系统 blocked、7 项残余阻塞、
  7 个裁决点 id 集合（`U8/U7`、`ABI`、`eol`、`equivalence-release`、`e-metric`、
  `unimplemented-4-schedule`、`scene_bvh_ray-drift`）均不变。
- `manifest.txt` 由 4 条重算为 5 条（新增本 addenda；历史 4 文件 sha/bytes 不变）。

## 5. 边界与非声明

append-only；零 GPU、零窗；不改判据/参考/容差、不改历史字节；
不宣称 NP01 通过、不宣称 NP02 通过、不宣称上游等价/双平台、不提供位级保证。
