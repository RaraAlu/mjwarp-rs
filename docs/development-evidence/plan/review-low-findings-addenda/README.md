# 复核低项 addenda 批（卡 K，2026-11-11；append-only、不改历史字节、零 GPU）

- 任务：`2026-11-11-review-low-findings-addenda`（owner Backend；p90；P8-渲染与BVH）。
- 依据：Chief 2026-11-11 12:56 指令；来源复核卡（均为只读复核、PASS，低项不阻断）。

## 1. 低项闭环清单（8 条）

| # | 低项 | 来源复核卡（复核对象） | 处置 | 落点 |
| --- | --- | --- | --- | --- |
| 1 | F-L1 `unregistered` 状态注解与词表匹配 | `np-gate/np5-00-freeze-check`（`875ae41d`） | **本批补说明 + 留 backlog**（账本逐字节相同、不就地改；拆字段/checker 断言留下一版账本） | `np-gate/README.md` §六 |
| 2 | F-L2 README `tree.head` 归属 | 同上 | **本批补**（更正为第一版卡登记提交 `f4628bd7`，r2 登记 `dec4e7d9`） | `np-gate/README.md` §六 |
| 3 | F-L3 账本 `task` 草稿字样 | 同上 | **本批补**（说明为生成时残留，账本字节不改） | `np-gate/README.md` §六 |
| 4 | J-L1 receipt 缺 `tree.head` | `np-gate/np5-01b-check`（`316b28dd`） | **本批补**（新增 `receipt-v2.json`，补 `tree.head=cead8bac…` + supersession；旧 receipt 不改） | `np-gate/np5-01b/receipt-v2.json` + README §七 |
| 5 | B-L1 §2.1「仓内 docs 暂无该编号」不准确 | `plan/W0-W9-check`（`fe72dcd1`） | **本批补**（已有 U8/U7 指代 3 处、无独立任务/条目 ID） | `plan/W0-W9.md` §9 |
| 6 | C-L1 `cam_res` 单帧/多帧措辞不准确 | `render-bvh/r06-flex-multiframe-recipe-params-fill-check`（`20015625`） | **本批补**（多帧 capture 无 `context.resolved` 段 → 实际 27 项，cam_res 名义第 28） | `…fill/README.md` §8 |
| 7 | C-L2 21 项新增断言缺非空转反例 | 同上 | **已在原证据闭环**（卡 E `8618e447`：新增键负例 `use_textures` + 既有键负例；`logs/coverage-21-mapping.txt`） | 引用，不改 |
| 8 | G-L1 计划文档「24 片 = 4+20」与表内 25 行不一致 | `plan/W4-B0-governance-check`（`279db5c8`） | **已在原证据闭环**（卡 J §9：25 片 = B0 4 + 实体 21，`ef78ba82…`/13,791 B） | 引用，不改 |

统计：**8 = 已在原证据闭环 2 + 本次补 5 + 留 backlog 1**（F-L1 的账本拆字段/checker 断言部分）。

## 2. append-only 证明（原文件历史字节不改）

| 文件 | 旧 sha256 / bytes | 新 sha256 / bytes | 说明 |
| --- | --- | --- | --- |
| `README.md` | `3f820851728ca91c…` / 9478 | `4cc553946c1898ff…` / 11576 | append-only |
| `np5-01b/README.md` | `af416071cdf4eb6e…` / 6108 | `94b39766072f7302…` / 6973 | append-only |
| `W0-W9.md` | `63caa15c4a38c6a4…` / 14120 | `749c1ceb959dc11c…` / 15202 | append-only |
| `r06-flex-multiframe-recipe-params-fill/README.md` | `3f7da3a1ce972e5e…` / 7940 | `143fa2174ec85ff8…` / 9392 | append-only |

- 每个改写文件均验证「新文件前 N 字节 == 旧文件全部字节」（`old_sha256 = sha256(new[:old_bytes])`）；JSON 结构变更走**新版本文件**（`receipt-v2.json`），旧 `receipt.json` 字节不动。
- 派生 `manifest.txt` 按新 sha 重算（manifest 不自列）；F 目录其余 7 条、J 目录旧 5 条、W0-W9 目录其他条目 sha 不变（见各 manifest）。

## 3. 本批改动文件

- `docs/development-evidence/np-gate/README.md`：9478 B → 11576 B（append-only，前缀不变）
- `docs/development-evidence/np-gate/np5-01b/README.md`：6108 B → 6973 B（append-only，前缀不变）
- `docs/development-evidence/plan/W0-W9.md`：14120 B → 15202 B（append-only，前缀不变）
- `docs/development-evidence/render-bvh/r06-flex-multiframe-recipe-params-fill/README.md`：7940 B → 9392 B（append-only，前缀不变）
- `docs/development-evidence/np-gate/np5-01b/receipt-v2.json`：新增（4800 B），旧 receipt 不改
- 4 个派生 `manifest` 重算（F/J/W0-W9/C-fill）
- 本目录 `README.md`/`receipt.json`/`manifest.txt`

## 4. 非声明

- 不宣称任何门禁通过/NP01/NP02 通过/上游等价/双平台；不改判据/参考/容差；不重跑窗口、不改账本字节、不改历史证据语义；零 GPU、零窗。
- 本批只做记录面更正：低项处置不改变任何已复核结论（各复核卡仍为 PASS，低项不阻断）。
