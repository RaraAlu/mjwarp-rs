# NP5-02 addendum（NP5-ERRATA-E 卡，2026-11-11）：E 判据状态 append-only 更正

本文件为 **append-only 更正**：NP5-02 的 `README.md`/`evidence.json`/`receipt.json`/`logs/selfcheck.txt`
历史字节不改（`manifest.txt` 重算为 5 条；历史 4 文件 sha/bytes 与 errata 卡 receipt 的 `pre_change_sha256`
逐项一致）；现行口径以本 addendum 为准，历史表述只作当时快照保留。任务：`2026-11-11-np5-e-metric-errata`
（登记 `40b7f16a`）。

## 1. 更正依据

- 用户 2026-11-05 签署 **E 判据 A2（选项①「同码跨后端核对」，签名级）**：卡
  `2026-11-05-precision-np01-e-criterion-a2`、提交 `b9cb2993`，记录
  `docs/development-evidence/np01-e-criterion-a2/README.md`。
- 契约 `docs/numerical-precision-contract.md` L267-L272 与「### E指标定稿（NP01，用户2026-11-05拍板选项①）」：
  判据写法「**实测 worst ≤ 已签 atol**」，不用 E 比值；适用范围 CONVEX/SDF × 字段块 dist/pos/frame
  （normal 并入 frame），样本 **97 标量**；**明确不宣称 native 口径**（native 侧「仅诊断」保留）。
- 因此 NP5-02 原写的「E 指标未定稿（用户 2026-10-27）」为**过时口径**：「未定稿」在定义层已不成立；
  未完成的只是**全产品 30 路径的 E/atol 核对**（A2 覆盖范围仅上述 97 标量）。

## 2. 现行口径（取代历史表述）

- E 判据状态：**已签 A2（2026-11-05，选项①，97 标量，不宣称 native）**。
- **「NP01 精度偏离 0 项」仍未完成（结论不变）**，理由更新为：
  1. 全产品 30 路径的 E/atol 核对未完成（A2 已签范围仅 CONVEX/SDF × dist/pos/frame，97 标量）；
  2. 跨 recipe 位级判据前提不成立（RE-0 仅 G1-partial；B1 证明 host/device 是两套 recipe）；
  3. device 级覆盖为可映射子集，缺项为 not-decidable；未做全路径 device 采样；
  4. 7 未实现子系统与 Linux/双平台项仍阻塞（NP5-03/NP5-04 账本）。
- 强制未完成句「NP01 精度偏离 0 项未完成」**仍在文且继续成立**；**不宣言 NP01 通过**。

## 3. 逐处更正（历史字节保留在下列位置，不回改）

下表 9 条与 `np5-e-metric-errata/corrections.json` 的 `repo_evidence[]` 中 `file` 为 np5-02 的条目一一对应。

| # | 位置（历史文件:行） | 历史表述（当时快照，已过时） | 现行口径 |
| --- | --- | --- | --- |
| 1 | `evidence.json:4`（scope） | "E metric not finalized -> not a pass claim" | E 判据已签 A2（2026-11-05）；全产品核对未完成 -> 不作通过声明 |
| 2 | `evidence.json:7`（criterion.e_metric） | "not finalized (user 2026-10-27); E-dependent acceptance cannot be declared" | 已签 A2（2026-11-05，选项①；97 标量；实测 worst ≤ 已签 atol；不宣称 native）；全产品核对未完成，E 相关验收不得声明 |
| 3 | `evidence.json:948-952`（coverage.not_covered_or_blocked[0]） | item「E 指标定稿」/ status "blocked on user decision (2026-10-27)" | item「全产品 E/atol 核对」/ status「判据已签 A2（2026-11-05）；全产品 30 路径核对未完成」 |
| 4 | `evidence.json:980`（conclusion.reasons[0]） | 「E 指标未定稿（用户 2026-10-27），精度偏离的判据与容限无法冻结」 | 「A2 判据已签（2026-11-05），但全产品 30 路径 E/atol 核对未完成」 |
| 5 | `evidence.json:988`（non_claims[0]） | "does not claim NP01 pass or precision completion (E not finalized)" | "does not claim NP01 pass or precision completion (E criterion signed 2026-11-05; full-product check incomplete)" |
| 6 | `README.md:60`（§5 表 E 行） | 「E 指标定稿 / 阻塞在用户裁决（2026-10-27）」 | 「全产品 E/atol 核对 / 判据已签 A2（2026-11-05，选项①，97 标量，不宣称 native）；全产品核对未完成」 |
| 7 | `README.md:71`（§6 未完成理由） | 「E 指标未定稿、跨 recipe 位级判据前提不成立…」 | 「全产品 E/atol 核对未完成、跨 recipe 位级判据前提不成立…」（结论不变） |
| 8 | `receipt.json:40`（coverage_blocked[0]） | "E 指标定稿" | "全产品 E/atol 核对（A2 判据已签 2026-11-05）" |
| 9 | `receipt.json:255`（non_claims[0]） | "does not claim NP01 pass or precision completion (E not finalized)" | 同 #5 现行口径 |

## 4. 不变项

- 分类/结论/站点数据**一律不改**：`precision_deviation_zero_items_verified` 仍 `false`；类型偏离 0 项、
  608 站点（唯一 598）、33 张类型表、recipe 差 5 case、engine-semantic source 20 站点 + RE-3 204 站点行级、
  同 recipe 残差 4 case、known deviation 1 项、未覆盖/阻塞 5 项均按 NP5-02 原证据。
- `manifest.txt` 由 4 条重算为 5 条（新增本 addenda；历史 4 文件 sha/bytes 不变）。

## 5. 边界与非声明

append-only；不改判据/参考/容差/相机/recipe、不改历史字节；零 GPU、零窗；
不宣称 NP01 通过、不宣称 E 完成、不提供位级保证。

## 6. QA 复核低项处置（NP5-REVIEW-LOW 卡，2026-11-11；append-only，前文不改）

来源：QA 复核卡 `np5-02-np01-precision-deviation-audit-check`（复核对象 `ac840ff7`，PASS，2 低 + 1 观察）。
本节只更正表述，不改 `evidence.json`/`README.md`/`receipt.json` 历史字节，不改任何分类或数字；
任务 `2026-11-11-np5-review-low-items-addenda`。

### L1（§3 站点枚举与「3 站点」不符）——已更正

- 原文（README §3，历史字节不改）：「NP5-01a 最终口径 recipe-difference **3 站点**（A-1a `acc:3`、
  A-1b `acc:5`、RE-1 的 `acc:5` …）」。
- 问题：`A-1b acc:5` 与 `RE-1 acc:5` 是**同一站点** `mass_solve:rigid_mass_solve:accumulation:5`
  的两层分类（A-1b source 级 landing `not-decidable` → RE-1 device 级定 `recipe-difference`）；
  且 A-1a 的 3 个 recipe-difference 站点未列 id。
- 更正后口径（分层写）：
  - **A-1a source 级 recipe-difference 3 站点**（id 全列）：
    1. `mass_solve:sparse_mass_copy_sparse_mass_update_sparse_mass_diag:accumulation:2`（landing）
    2. `mass_solve:small_prefix_validate_small_prefix_factor_2..6:accumulation:1`（landing）
    3. `mass_solve:rigid_mass_solve:accumulation:3`（landing `pin-needed` → A-1a R2 改判 `recipe-difference`）
  - **A-1b `mass_solve:rigid_mass_solve:accumulation:5`**：source 级 landing `not-decidable`，
    由 RE-1 在 device 级定 `recipe-difference`——**同站点两层分类，不重复计入 source 级 3 站点**。
- 结论不变：recipe 差 5 case×3 字段 + 3 站点（A-1a source 级）单列；不宣称 NP01 通过。

### L2（§1 节号交叉引用错）——已更正

- 原文（README §1，历史字节不改）：「站点分类（landing 直方图合计）… （最终口径另见 §4）」。
- 更正：最终口径变化（`recipe-difference`/`pin-needed`）在 **§3**（recipe 差）说明；§4 是引擎语义差与
  实现残差。正确引用为「（最终口径另见 §3）」。

### 观察 O1（`type_class` 分组表述）——记录不修

- README §1 把 `n/a-view 7 / f32 编码 2 / int→f32 编码 1 / f64-to-f32 1`（=11）并入「其它 f32 派生」；
  其中 `n/a-view`、`f64-to-f32` 并非 f32 派生。总数 608 正确、无统计错误；README 历史字节不改。
  后续逐类对账以 evidence 的 11 个 `type_class` 机器可读值为准（不合并口径）。

### 编号口径

- 以上编号与 `np5-review-low-items-addenda/items.json` 的 `NP5-02-L1`/`NP5-02-L2`/`NP5-02-O1` 一一对应。
