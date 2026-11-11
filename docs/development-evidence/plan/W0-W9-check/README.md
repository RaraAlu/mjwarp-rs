# QA 复核：W0–W9 计划文档化（`fe72dcd1`，卡 B）

- 任务：`2026-11-11-render-plan-w0-w9-documentation-check`（owner QA）。复核对象：提交 **`fe72dcd1`** + 卡 `2026-11-11-render-plan-w0-w9-documentation`。零 GPU、只读复核。
- 基线：`2b6d9b56`（卡 B 登记/start）。

## 结论

**复核通过（0 阻断，1 低 + 2 观察）。** 文档化快照结构与首行声明到位、四项待裁决齐全且**无任何结论性表述**（结论词 0 命中）、写域仅 plan + `.normify`、内容源哈希与 manifest 自核正确、编码/门禁干净。

## 一、结构 / 声明 / 口径（✅）

- `docs/development-evidence/plan/W0-W9.md`：**14,120 B**，UTF-8、LF、无 BOM、CR=0、NUL=0。
- 首行即声明：`# W0–W9 执行落地计划（文档化快照，2026-11-11）` + 引用块「**文档化快照；裁决项未定，不构成结论**」，并写明落仓时点 **2026-11-11**、内容 owner Arch、落仓执行 Backend、卡 id、**可能过期 + 只追加 addenda 不重写**的更新规则。
- 结构齐备：§0 状态与口径 / §1 波次总表（W0–W9，列：条目·依赖·片数·GPU 窗·签名或裁决·完成判据）/ §2 待用户裁决区（2.1 U8/U7、2.2 ABI、2.3 eol、2.4 等价/发布，**四项齐全**）/ §3 硬约束（GPU 单窗+monitor 60/60、写锁单写者、高峰只读、判据不动、采集脚本抽模块排 G-L2 之后、证据纪律）/ §4 落仓时点可开工项（标明计划态、可能过期）/ §5 引用（W0 收口卡+提交、guards 哈希、aggregate、107 计数、P10、diag8、errata r2、z2vec 片 1）/ §6 风险 / §7 z2vec 两片验收清单 / §8 落仓与更新规则。
- 四项待裁决均为「待用户裁决/待用户签/待用户确认/待校准」口径，且每项显式写「只列对象、材料、影响面，不预设结论、不代用户裁决」。

## 二、结论性表述 grep（✅ 0 命中）

对全文（含 §1 表、§2、§7）逐词检索：

| 词 | 命中 |
| --- | --- |
| 已裁决 | 0 |
| 已签 | 0 |
| 已批 | 0 |
| 等价通过 | 0 |
| 发布通过 | 0 |
| 已通过 | 0 |
| 已发布 | 0 |
| 签名完成 | 0 |

「通过」全部 6 处均为**允许的事实陈述**：§0/§1/§2.4 的「G-L2 判据 A/B/C 在 attempt 4 canonical 窗口通过」（附 `5f1d436d`/`5b586e77`/卡 id）、§2.4 的否定句「本计划不预设其通过」、§7 的「unit 新增用例通过」（计划态验收项）——没有把任何未决裁决写成结论。

## 三、写域 / 证据 / 门禁（✅）

- 提交 8 files：`docs/development-evidence/plan/W0-W9.md` + `W0-W9.receipt.json` + `W0-W9.manifest.txt` + 5 个 `.normify` 卡/门禁记录；`git diff --name-only 2b6d9b56..fe72dcd1` **无** plan 目录与 `.normify` 之外的路径 ⇒ `src/`、`tests/`、`fixtures/`、`scripts/`、guards/冻结件零改动。
- `W0-W9.manifest.txt` 列 2 文件（md + receipt），我重算 sha256+bytes **全部匹配**，manifest 不自列（453 B）。
- 内容源哈希：receipt 声明 Arch 定稿 `w0-w9-plan-final.md` = `7649c8b1…`/11,380 B；我实测 target 文件 **11,380 B / `7649c8b1171684f71bb2860129722c95396c6c8f6f7ce323ec4a3fbba31c777c`** —— 逐位一致（该文件为工作材料，未入库）。
- receipt `task_id`/`verdict=PASS` 与卡 checks 的 JSON pointer 一致；卡状态 verified。
- 门禁（`.normify/runs/2026-11-11-render-plan-w0-w9-documentation/`）：fmt/clippy/clippy-cuda exit 0、unit **1,403/0/402**、exit 0。

## 四、事实核（✅ 除 1 条低项）

- **「W0–W9 此前从未落库（grep 无命中）」✅**：`git grep "W0–W9\|W0-W9"` 在本提交前的 `docs/` 中无命中；`git log -- docs/development-evidence/plan/` 除本次外无相关落仓提交。
- **§5 引用逐项核对 ✅**：W0 归档 `5f1d436d` + 卡 `…-attempt4` + QA `5b586e77`；guards `94478b3d…`/43,729 B、aggregate `08180107…`、107=12+93+2、12/12 `live==guard==stash`（与我 attempt4 复核一致）；P10 修复 `b0e372be` + QA `b84d5996`；诊断轮 #8 QA `11530f82` + 卡 `…-diag8-check`；errata r2 `84ed67d2`；z2vec 片 1 `19a68b49`；诊断证据卡 `c6ea92b8`（2026-11-11 00:07，R06 z2vec 只读诊断）——全部实存且指向正确。
- **波次表与定稿一致 ✅**：W0–W9 各行的条目/依赖/片数/窗数/签名裁决/完成判据与 `w0-w9-plan-final.md` §1 逐行一致（W5「用户暂缓」= 源稿 "deferred"）。
- **低 L1（事实表述不准确）**：§2.1 写「U8/U7 的精确条目 ID/定义……**仓内 docs 暂无该编号**」——实测仓内 docs **已有 U8/U7 指代**：`docs/development-evidence/g10-p5-mesh-cap20/README.md`（「U8/U7 维持 record-only」）、`g10-p5-mesh-cap20-check/README.md`、`mesh-polygon-gt16-gap/README.md`（标题即「mesh polygon 容量差距评估（U8/U7）」）。准确表述应为「有 U8/U7 指代但**无独立任务/条目 ID**，精确条目编号待 Chief 校准」（源稿同句，落仓沿用）。不影响裁决/结论，属记录面事实精度。

## 五、观察（不阻断）

- **O1（合规改写未声明）**：源稿 W0 行写「S-3 **已签**」，落仓版改为「S-3（用户签名，2026-11-11）」以规避卡 B 的 `forbid_claim` 词表（该卡要求全文不出现「已签」）；事实等价且是卡要求所致，但「按定稿原样落仓」的措辞因此不完全严格，建议 receipt 补一句「W0 签名措辞为合规改写」。
- **O2（快照过期属正常）**：§7 z2vec 清单仍为 `[ ]` 计划态、`render_flex.cu:47-53` 行号锚点随 P10 恢复已偏移（现近平行块 :47-56），与 §0/§7「计划态/可能过期、实际以实施卡与提交为准」声明一致；只读快照无碍，后续如需可 addenda 关联实际提交（片 1 `19a68b49`、片 2 记录面卡）。

## 六、口径

零 GPU、零窗口；不替用户/Chief 预判任何裁决；不宣称上游等价/对外发布/双平台；不改判据/参考/容差；本复核只核文档化快照的完整性/准确性/边界。
