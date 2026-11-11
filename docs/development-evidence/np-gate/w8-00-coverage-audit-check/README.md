# W8-00 覆盖审计复核（QA）：阶段伞卡 × 兼容矩阵行 × 依赖 verified × 缺口总表

- 复核对象：卡 `2026-11-11-w8-00-coverage-audit`；内容提交 **`96b8d201`**（基线 `f9eb64e1`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**；矩阵 61 行、阶段/伞卡统计、W4 侧与
  缺口总表我独立复算全部吻合，结论旗标均为 false，未发现准入类表述。

## 1. 兼容矩阵（我从文档独立解析）

- `docs/compatibility-matrix.md` sha `e68c4456…`/30,782 B 与 audit `sources` 一致。
- 四段矩阵行数我按表格逐行解析：**GPU物理 28 + 模型与数据 11 + 渲染 10 + 运行时与数据接口 12 = 61**，
  与 `matrix.by_section` / `total_rows` 完全一致。
- **唯一行 id 集合**（G01–G28 / M01–M11 / R01–R10 / A01–A12 = 61 个）与 4 段行数一致，无重复/漏行。
- `matrix.by_stage` 我按「阶段」列重算（含 `P4/P6` 多阶段与 `全阶段`）——与 audit 的 11 个阶段键
  逐键集合相等（mismatches=[]）。

## 2. 阶段伞卡与依赖 verified 状态（独立对板面复核）

- 我从 `.normify/changes/*.json` 逐卡读取 status/checks，核对 `stage_summary` 里 **15 张阶段级卡**：
  **15/15 status 与 checks 全对**（全部 `checks=0`、状态均为 proposed/backlog ⇒ 不可 start）。
- 逐阶段统计我复算后与 audit 完全一致：**stage_level_cards 15、zero_checks 15、slice_cards 43、
  slice_verified 21、distinct_unverified_deps 41**（分阶段：P0/工具链 4、P4 1、P5 1、P6 1、P7 1、
  P8 3、P9 2、P10 1、P11 1）。
- 各阶段 `unverified_deps` 我逐条对板面核 status：**全部一致（bad=[]）**；分阶段计数之和 67 >
  去重 41 属依赖被多阶段共享，audit 的 totals 用去重口径（41），口径自洽。
- 结论旗标：`umbrella_startable=false`、`w4_slice_b_prereq_met=false`、`w8_ad_prereq_met=false`、
  `any_admission_claim=false`——与「只做覆盖对照、不构成准入」一致。

## 3. W4 侧与缺口总表

- **7 张 W4 伞卡**：我从板面逐卡核 status/checks，**7/7 全 `checks=0`**（proposed/backlog），
  与 `umbrella_zero_checks=7` 一致；`slices` 记 **25 = B0 4 + 实体 21**（§9 已有的 24→25 更正），
  与 `plan/W4-dependency-governance.md` §9 一致。
- **片 B 双重阻塞**（`r07-p8` proposed/0 待 ABI 用户签 + NP01/NP02 未验收）与
  `dispatchable_0_window` 五片（W4-01A/02A/05A/03A/04A）记录在位。
- **缺口总表 12 项**（W8-G1…G12）：G1 伞卡占位 / G2 NP 门禁 / G3 渲染 ABI / G4 Linux / G5 T4 重基线 /
  G6 W2 判据 / G7 P11 用户级 / G8 g10-p5 伞卡（有 verified 代理收口卡）/ G9 未验收依赖 /
  G10 窗口 / G11 W4 片 B / G12 W8 AD 前置——逐项与上述实测对应，无「已满足」误记。

## 4. 第一节（NP5-ERRATA-E O1/O2 处置）

- `np-gate/np5-e-metric-errata/addenda.md`（1,627 B、sha `5c6e06e0…`，与 receipt 记录一致）：
  **O1** 明确 `historical_snapshot` 是归一化摘要、非逐字引用（原文以历史文件字节为准）；
  **O2** 明确 W5 working docs 在 target 不入仓、只能按 sha 复现，记录不修——均为 append-only 注记。
- errata 目录 manifest **4→5**，我按行重算 5/5 sha/bytes 全对；**8 个历史文件字节未改**
  （np5-02 4 文件 vs `ac840ff7`、np5-04 4 文件 vs `82c3186a`，逐 `git cat-file blob` 比对）。

## 5. 证据 / 边界 / 门禁 / 编码

- `plan/w8-00-coverage-audit/` manifest **4/4**（README/audit.json/receipt.json/logs/selfcheck.txt）
  我按行重算全对；selfcheck **RESULT: PASS**（36 行）；policy_scan 3 文件 **0 命中**、
  `--self-test` **34/34**；该卡四门禁运行记录齐（unit 1,406/0/402）。
- 提交 `96b8d201` 只含 `.normify/**` + `plan/w8-00-coverage-audit/` + errata addenda/manifest；
  **无 src/tests/fixtures/scripts 改动、未改任何其它卡**；UTF-8/LF 无 BOM、无 NUL、无裸 CR。
- 哈希：audit README `00ba73c2…`/7,330、audit.json `e3e9f87c…`/38,349、receipt `976623e2…`/3,094、
  selfcheck `26d976aa…`/2,012、manifest `fed8b6e2…`/548（证据 manifest 4/4 自核见上）。

## 6. 发现

### 观察
- **O1 板面计数时点混用（轻微）**：`sources.board` 写 `cards_total=1127 / verified=936`——但按提交历史，
  `f9eb64e1` 基线为 **1126 卡 / 936 verified**，`acca3189` 登记本卡后为 **1127 / 936（in_progress）**，
  `96b8d201` 收口后为 **1127 / 937**。即 total 取「登记后」、verified 取「收口前」，两个数来自不同时点，
  直接算比例会差 1 卡。建议注明 snapshot 时点或「verified 不含本卡」。
- **O2 slice 计数未给 id 明细**：`stage_summary` 各阶段的 `slice_cards/slice_verified` 只给计数、
  未列 slice 卡 id（`stage_level_cards` 有 id），无法仅凭 audit.json 逐卡复核明细（本次我用板面
  命名/status 交叉核对通过）。建议后续审计补 slice id 列表或注明口径（如「阶段切片卡」的匹配规则）。

## 7. 非声明

本复核只对覆盖审计的**数字与口径**负责；**不代表**任何阶段准入、伞卡 verified、NP 通过；
不宣称上游等价、位级等价、双平台或发布；零 GPU、零窗、不改判据/参考/容差/历史字节。
