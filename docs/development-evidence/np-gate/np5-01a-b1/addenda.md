# B-1 addendum（C-1 卡第一节，2026-11-11）：L1/L2/人工复核痕迹闭环

B-1 复核 PASS（0 阻断），2 低建议 + 1 观察并入 C-1 卡第一节处置；本节为 append-only 记录
（B-1 的 `README.md`/`evidence.json`/`receipt.json`/`logs/selfcheck.txt` 字节不改）。

## 1. 低 L1：扫描器声明否定漏洞（已修）

- 问题：引用块豁免的「撤回/supersede/作废」原为裸子串匹配，块内写「未撤回」「不作废」仍会豁免
  （QA 实测 0 命中）。
- 修复：声明识别改为**肯定式**——声明词前（去分隔符后）紧邻 未/不/没/无/非/否/没有/无法/不能/不会/
  尚未/并未/不予/未予 时不构成声明、不豁免；固定句式如 `撤回(5)`、`supersedes:` 仍算。
- 自测：12→**24/24 PASS**（新增 4 条：未撤回/不作废/没有撤回 不豁免、肯定式声明豁免）。
- 工具：`scripts/np-gate/policy_scan.py` 新版 sha256 `1d3a88b97c2eda55004307141b847e70cc80c858080f2fc80e5938e2fad780ff` / `11712` B（B-1 落库时
  `e28ddb899cd65c909a7677235b03e829bec5e44715bd5bebd119dddd8019e97b` / 8,013 B、20/20 为历史值）。

## 2. 低 L2：`reference_anchor.status` 词表文档化（已补）

- `no-reference-expression`：站点本身不是可比较的算术表达式（声明/缓冲/视图/纯拷贝/状态码/
  整型索引/零初始化/prose），或上游在该点无对应表达式；用于 source 级 `n/a` 站点的 reference_anchor。
- `no-reference-entry`：有语义对应点但参考侧无 entry/PTX；用于 `not-decidable` 的
  `not_decidable_reasons[]`。二者同名但字段层级不同，不得互相替代。
- 落点：`np5-01a-vocab-addendum.md` §6（工作材料，新 sha `453c5c9157341fa28ecb009c50f2ff560b6f43e15f593ed9b7a75f27dd5b1b26` / `6771` B）+
  C-1 `README.md` §2；B-1 实测 n/a 50 条中 21 条使用 `no-reference-expression`（其余 n/a 有上游锚点）。

## 3. 观察：quote 块人工复核痕迹（已立约定）

- 每个 `policy-scan:quote:start/end` 块须留人工核对痕迹：块内 `reviewed_by: <name>-<date>`（首选），
  或在证据 receipt 的 `quote_block_inventory` 中列块（file/行范围/复核人）。扫描器新增
  `--list-quote-blocks FILE...` 供 QA 核对；豁免内容盲，逐字原文仍须人工确认。
- 现存唯一豁免块：`np-gate/np5-01a-a2/addenda-v2.md` §2（行 29–38），复核人 `backend-2026-11-11`，
  清单与说明见 C-1 receipt `quote_block_inventory`（A-2 addenda-v2 字节不改）。

## 4. B-1 状态

B-1 证据历史字节不变；本 addenda 与 `manifest.txt` 重算为派生更新。B-1 分类/计数/结论不变。
