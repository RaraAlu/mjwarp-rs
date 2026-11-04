# NP01 E 指标 A2 QA 复核（只读）

任务：`2026-11-05-precision-np01-e-criterion-a2-check`（owner QA）。复核对象：`2026-11-05-precision-np01-e-criterion-a2`（verified；证据提交 `b9cb2993`）。日期：2026-11-05（空闲段）。
**边界：只读 + 只写本目录；零 GPU；不回改契约、A2 证据或任何已 verified 证据；问题只记录为观察。**

## 1. 结论

**PASS-with-observations（0 阻断）。** 契约只新增 E 指标小节（`+20/-0`）、既有 atol/rtol 表与实测 worst、`E_mjwarp`/`redline` 公式逐字未动；范围与样本逐字（CONVEX/SDF × dist/pos/frame、normal 并入 frame、97 标量）；判据为「实测 worst ≤ 已签 atol」且明确不用 E 比值；不宣称 native 口径；新旧措辞规则在案；签名记录与证据自核通过。
3 条低观察（O1 契约内历史句与新节并存、O2 派生模块指纹、O3 签名由 Chief 转述），均不影响 A2 定稿效力。

## 2. 逐项核对

### 2.1 只加定义（PASS）
- `git show --numstat b9cb2993 -- docs/numerical-precision-contract.md` = **20 added / 0 deleted**；删除行 0，无既有行被改。
- 新增小节 `### E指标定稿（NP01，用户2026-11-05拍板选项①）` 紧跟「接触字段容差分档」（原 §264 之后），契约总行数 585→605。
- 该文件在 A2 之后未被再次改动（`git log -1 -- <contract>` = `b9cb2993`）。

### 2.2 数字零改动（PASS）
脚本把 HEAD 与 `b9cb2993^` 逐行比对：
- atol/rtol 表 7 行（dist CONVEX `1e-6` / SDF `2e-6`；pos CONVEX/SDF `1e-6`；frame·normal CONVEX `2e-6` / SDF `5e-5`；结构字段 0）**逐字相同**；
- 实测 worst 列（9.31e-9 / 6.88e-7 / 1.86e-8 / 4.62e-7 / 8.19e-7 / 2.78e-5 / 0 失配）随表行相同；
- `t_i = atol_native + rtol_native * abs(native_i)`、`E_mjwarp = max_i(...)`、`redline = 1.05 * E_mjwarp` **三行逐字相同**。

### 2.3 范围与样本逐字（PASS）
新增节写死：**CONVEX / SDF 两路径 × 字段块 dist/pos/frame（normal 并入 frame）**、**同码跨后端 97 个标量**。
与依据指针 `docs/development-evidence/np01-e-recompute/README.md` §三一致：`same_code_backend_baseline`（97 标量）、**6/6 项实测 worst ≤ 已签 atol**、最紧余量 **1.80×（frame/SDF）**。

### 2.4 判据写法（PASS）
写成「**实测 worst ≤ 已签 atol**」的核对；明确**不使用 E 比值**并给出退化理由：同码基线作参考时 `E_mjwarp ≡ 0`、`1.05 * E_mjwarp` 退化、E 比值恒成立。

### 2.5 不宣称（PASS）
新增节写死：明确**不宣称 native 口径**；契约 native 侧「仅诊断」语义保留（2026-10-25 选项 B 不变）；本口径不用于 native 侧放行；不改变接触字段对上游 f32 的主判据；**不新增数值、不改已签 atol**；上一节 atol/rtol 表与实测 worst 保持原样。receipt `non_claims` 4 条 + `native_scope_claimed=false`。

### 2.6 措辞规则（PASS）
- 自 A2 起后续收口写「NP01 已核对；E 指标用户 2026-11-05 已签（同码跨后端核对，A2 `<提交号>`）」；
- A2 之前已 verified 的历史 summary「E 指标 1 项待用户签」保留为当时快照、不回改；
- 交叉核对：G10 收口 fix `afd5e6cc` 已按该规则写「已签（A2 `b9cb2993`）」。

### 2.7 签名记录与证据自核（PASS）
- 卡 note / 证据 README §1 / receipt `signature`：日期 **2026-11-05**、选项 **①「同码跨后端核对」**、级别 **signature-level**、**Chief 转达**（仓内无独立用户消息快照，与既有签名卡同口径处理）。
- `np01-e-criterion-a2/manifest.txt` 2 条（README `3261 B`、receipt `2517 B`）sha256/bytes **2/2 自核通过**，LF、无 BOM、首行 `#`。
- `b9cb2993` 6 个文件，`src/`、`tests/`、`fixtures/` **零改动**。

## 3. 观察清单（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（建议治理） | 契约 L260 既有「边界：**E尚未定稿**」与同文件新增「E指标定稿」并存，按字面冲突 | A2 明确「既有章节一律不动」，属历史快照；建议后续单独治理卡在 L260 加指向注记（不动 A2 证据） |
| O2 | 低 | `b9cb2993` 另带 `.normify/modules/mjwarp/index.md` 指纹更新 | normify 派生文件，非契约内容改动；如实登记 |
| O3 | 低（已在案） | 签名原文为 Chief 转述，仓内无独立用户消息快照 | 与既有签名卡同口径处理，已在 A2 卡 note 与 receipt 写明 |

## 4. 复现与证据

- 脚本：`scripts/np01_a2_check.py`（只读；契约 diff 增量、表/公式逐字比对、新增节 token、证据 manifest、receipt 字段、依据指针、零代码改动、G10 措辞交叉核对）。
- 日志：`logs/np01-a2-check.log`（`RESULT: PASS`，0 FAIL；repo 快照 `90bc1169`，复核对象 `b9cb2993`）。日志由 Python 直写 UTF-8/LF（本机 PowerShell `>` 重定向会写成 UTF-16LE+BOM+CRLF，已规避；脚本末尾含证据目录编码自审）。
- `manifest.txt` 4 条（README + facts.json + 脚本 + 日志），LF、无 BOM，首行 `#`。
