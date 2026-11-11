# NP5-01a 收尾 addendum（NP5-REVIEW-LOW 卡，2026-11-11）：双口径方法限定 + 片数口径更正

本文件为 append-only：收尾报告的 `README.md`/`receipt.json`/`logs/selfcheck.txt` 历史字节不改；
`manifest.txt` 3→4 条（新增本 addenda）。任务：`2026-11-11-np5-review-low-items-addenda`。

## L1（双口径去重方法必须带限定）——已加注

- 原表述（README §2 / receipt `dual_count.method`，历史字节不改）：「按 `(kernel, kind, index)`
  去掉行前缀后比对」。
- **限定（生效口径）**：去重**仅适用于算术表达式站点**——即 id 为 4 段 `row:kernel:kind:index`
  且 `kind ∈ {intermediate, accumulation, output}` 的站点；**row 级 `rounding-note:index` 等
  `rounding-note` 条目（共 67 条，含 49 条 3 段 row 级 id 与 18 条带 kernel 的 4 段 id）按行独立、
  不参与表达式去重**。
- 精确复算（本 addendum 实测，基于 11 片 `evidence.json` 的 608 条 entries）：
  - 算术表达式站点 **541** 条（intermediate 230 / accumulation 132 / output 179）；
    按 `(kernel, kind, index)` 去重后 **531**（去 10 个 `rigid_com_position` 重复 tail：
    `acc:0/1/2`、`int:0–3`、`out:0–2`）；
  - `rounding-note` 条目 **67** 条，按行独立保留；
  - **531 + 67 = 598 = 608 − 10** ✓（结论不变）。
- 反例（为何必须带限定）：若把 3 段 row 级 id 也按「去行前缀」处理，会产生 **15 个共享 key**
  = 10 个 `rigid_com_position` 算术 tail + 5 个 `rounding-note:0..4`（分别跨 15/14/12/6/2 行同名）；
  此时严格集合去重得 **554**、按「每个共享 key 计一次重复」得 **593**（QA 复核数）——均 ≠ 598。
- 对外汇报口径：「**608 行×站点；算术表达式去重后 598（rounding-note 按行独立）**」；
  不得把 608 读成唯一表达式数。

## L2（片数口径）——已更正

- 原表述（收尾卡提交说明/标题）：「12 片 608 站点」。
- 更正口径：**11 个站点片**（A-1a/A-1b/A-2/A-3/B-1/C-1/C-2/C-3/C-4/D-1/D-2，合计 608）
  + **4 个设备证据片**（RE-0/RE-1/RE-2/RE-3）+ **5 张治理卡**（K/L/A2-POL/POL-SCAN/QA-ADD）。
  以 receipt `slice_count=11` 与 README §1 的 11 个站点片为准；历史提交说明不改（历史字节不可改写）。

## 边界与非声明

append-only；结论仍为 **608 = 598 + 10**；零 GPU、零窗；不改判据/参考/容差、不改历史字节；
不宣称 NP01 通过、不宣称精度完成/位级保证；编号与 `np5-review-low-items-addenda/items.json` 的
`NP5-01a-CLOSING-L1`/`NP5-01a-CLOSING-L2` 一一对应。
