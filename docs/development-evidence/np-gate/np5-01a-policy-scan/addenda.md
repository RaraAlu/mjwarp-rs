# POL-SCAN addendum：工具版本化更新（B-1 卡，2026-11-11）

- supersedes: `scripts/np-gate/policy_scan.py@59765f71f273e469d42923523fd405509ec22520b48b75f4348153dac8b527ee`
  （4,559 B，本目录 receipt 记录的 landing 版本）
- 原因：RE-2 复核 G2 要求新增**显式引用块豁免**；工具在 B-1 卡第一节更新。
- 新版本：sha256 `e28ddb899cd65c909a7677235b03e829bec5e44715bd5bebd119dddd8019e97b` / `8013` B；`--self-test` **20/20 PASS**（原 12 条 + 新增 8 条引用块用例）。
- 变更：`quote_exempt_regions()` —— 仅当文本位于闭合的 `<!-- policy-scan:quote:start/end -->`
  标记块内、且块内**含撤回/supersede/作废声明**时豁免；未标记、无声明、未闭合、块外一律照常命中；
  引用必须逐字原文，禁止拆写/改写规避。docstring 写明标记用法、逐字要求与人工复核职责；
  分隔符归一化与 18 字符否定窗口口径不变。
- 边界：未触碰 guards/freeze script/既有采集脚本；`guards_touched:false` 仍成立。
- 本目录 `README.md`/`receipt.json`/`logs/selfcheck.txt` 为历史 landing 字节，不改；本 addenda 与
  manifest 为派生更新。
