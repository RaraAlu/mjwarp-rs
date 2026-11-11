# QA 复核：复核低项 addenda 批（`4353796d`，卡 K）

- 任务：`2026-11-11-review-low-findings-addenda-check`（owner QA）。复核对象：提交 **`4353796d`** + 卡 `2026-11-11-review-low-findings-addenda`。零 GPU、只读复核。
- 基线：`f5b05b7c`（卡 K 登记/start），HEAD 前值 `0cb007db`。

## 结论

**复核通过（0 阻断，1 低 + 2 观察）。** 4 个历史文件均为**纯追加**（`old_sha256 = sha256(new[:old_bytes])` 我逐文件验证成立）、旧字节未动；JSON 结构变更走**新版本文件** `receipt-v2.json`（补 `tree.head=cead8bac…` + supersedes `c4856857…`，旧 `receipt.json` 字节不变）；5 个派生 manifest 全部按新 sha 重算且其余条目不变；8 条低项分类 = **已在原证据闭环 2 + 本次补 5 + 留 backlog 1**，与批 README/receipt 一致；写域仅 `docs/development-evidence/**` + `.normify/**`，账本/历史语义未改，零 GPU、零窗。

## 一、append-only 与旧字节（✅ 我逐文件验）

| 文件 | 旧 sha256 / bytes | 新 sha256 / bytes | 前缀校验 |
| --- | --- | --- | --- |
| `np-gate/README.md` | `3f820851…` / 9,478 | **我实测 `4cc55394…` / 11,576** | prefix == 旧长 ✅ |
| `np-gate/np5-01b/README.md` | `af416071…` / 6,108 | **我实测 `94b39766…` / 6,973** | prefix == 旧长 ✅ |
| `plan/W0-W9.md` | `63caa15c…` / 14,120 | **我实测 `749c1ceb…` / 15,202** | prefix == 旧长 ✅ |
| `r06-flex-multiframe-recipe-params-fill/README.md` | `3f7da3a1…` / 7,940 | **我实测 `143fa217…` / 9,392** | prefix == 旧长 ✅ |

- **JSON 结构变更合规**：新增 `np5-01b/receipt-v2.json`（**我实测 `da81edf4…` / 4,800 B**），`tree.head=cead8bac154a7dc4fa033a6738d96579202a4922`、`head_source`（主提交 `316b28dd` 的父；四门禁与 existence-check 在该 HEAD 运行）、`main_commit`、`supersedes`（旧 receipt `c4856857…` / 6,317 B）；旧 `receipt.json` 我实测 sha/bytes **未变**（`c4856857…`/6,317）。
- **历史语义未改写**：账本 `np-gate-ledger.json`、`implemented-subsystems.json`、`unregistered-units.json`、`existence-check.txt` 均无 diff（我核 `git diff --name-status` 空）。

## 二、manifest 重算（✅ 我独立重算）

| 目录 | manifest 行数 | 我的重算 |
| --- | --- | --- |
| `docs/development-evidence/np-gate` | 8（README 新 sha + 7 条不变，含 2 条 repo-relative `scripts/np-gate/*.py`） | bad=0 |
| `docs/development-evidence/np-gate/np5-01b` | 6（README 新、`receipt-v2.json` 新增、旧 receipt + 2 JSON + log 不变） | bad=0 |
| `docs/development-evidence/plan`（`W0-W9.manifest.txt`） | 2 | bad=0 |
| `docs/development-evidence/plan/review-low-findings-addenda` | 2（README + receipt，不自列） | bad=0 |
| `docs/development-evidence/render-bvh/r06-flex-multiframe-recipe-params-fill` | 7（README 新 + 6 条不变） | bad=0 |

## 三、8 低项分类与落点（✅ 逐条核）

| # | 低项 | 处置 | 落点与我的核验 |
| --- | --- | --- | --- |
| 1 | F-L1 unregistered 状态注解/词表匹配 | **说明 + 留 backlog** | `np-gate/README.md` §六（括号前裸值入词表、括号内理由；下一版账本拆字段）+ checker 断言 backlog —— 说明已消除误读，见低 L1/观察 O1 |
| 2 | F-L2 README tree.head 归属 | **本次补** | §六：`f4628bd7` 是首版卡 F 登记提交、r2 为 `dec4e7d9`（我卡 F 复核 L2 原文一致） |
| 3 | F-L3 账本 task 草稿字样 | **本次补** | §六：生成时残留、按逐字节未改（我卡 F 复核 L3 一致） |
| 4 | J-L1 receipt 缺 tree.head | **本次补** | `receipt-v2.json` + README §七 + manifest 6/6（我卡 J 复核 L1 闭环） |
| 5 | B-L1 §2.1 编号指代 | **本次补** | `W0-W9.md` 新增 §9：已有 U8/U7 指代 3 处（g10-p5-mesh-cap20、其 -check、mesh-polygon-gt16-gap）但无独立任务/条目 ID（我卡 B 复核 L1 一致）；并补 O1 合规改写声明 |
| 6 | C-L1 cam_res 口径 | **本次补** | fill README 新增 §8：单帧 `render_flex_validate.rs:421-422` 是显式对照、多帧 capture 无 `context.resolved` 段 → **实际逐项断言 27 项**、cam_res 名义第 28（与我卡 C 的 L1 一致）；并如实注明 tests 源注释旧措辞留在后续测试面卡 |
| 7 | C-L2 21 项非空转 | **原证据已闭环** | 引用卡 E（`8618e447`，证据 `d091b1e7`）与 `coverage-21-mapping.txt` |
| 8 | G-L1 24→25 片 | **原证据已闭环** | 引用卡 J §9（`ef78ba82…`/13,791 B） |

- 批证据 `plan/review-low-findings-addenda/`：README 4,255 B、receipt **`8d15deea…`/5,040 B**（`lows_total=8 / closed_in_original=2 / fixed_this_batch=5 / backlog=1`，8=2+5+1 ✅）、manifest 2/2。

## 四、边界 / 编码 / 门禁（✅）

- 提交 17 files = 4 个 append 文件 + 4 个 manifest + `receipt-v2.json` + 批证据 3 文件 + 卡 K + 4 门禁记录；`git diff 4353796d^..4353796d` **无** `src/`/`tests/`/`fixtures/`/`scripts/` 改动、guards/freeze script 未动、不改判据/参考/容差；零 GPU、零窗。
- 改动/新增文档全 UTF-8/LF、无 BOM/CR/NUL；门禁 fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 无通过类声明：批 README §4 非声明 + 各 addendum 均明确「不宣称门禁通过/NP01/NP02 通过/等价/双平台」。

## 五、低项 / 观察

- **低 L1（backlog 计数口径）**：批 receipt `backlog=1`（仅 F-L1），但 C-L1 的 addendum 又留了一个 follow-up——**`tests/support/render_bvh/multiframe_recipe.rs` 仍含旧 cam_res 措辞**（我实测第 8/51/131/168 行仍写「单帧验收亦为隐式比对」「cam_res 隐式」），K 说明「属 tests 写域、留后续测试面小卡（backlog）」。该 follow-up 未计入 backlog 统计、也未登记卡 → 建议在 receipt/README 记为 backlog=2（或补登一张 tiny 测试面卡）。
- **观察 O1**：F-L1 的「说明 + 留 backlog」只解决误读，账本字段拆分/checker 断言仍需**下一版账本**做（建议与 NP5-01a 下一轮或账本 v2 一并）。
- **观察 O2（正向）**：本批用 `old_sha256 = sha256(new[:old_bytes])` 作 append-only 证明、JSON 走版本文件、manifest 只重算派生行，方法可复用于后续 errata/addenda 批。

## 六、口径

不宣称门禁/NP01/NP02 通过、不宣称上游等价/双平台；不改判据/参考/容差；不改账本字节与历史语义；零 GPU、零窗。
