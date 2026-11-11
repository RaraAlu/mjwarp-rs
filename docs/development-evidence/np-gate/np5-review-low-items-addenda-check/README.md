# NP5-REVIEW-LOW 复核（QA）：NP5 复核低项 append-only addenda（NP5-02 L1/L2、NP5-03 L3、收尾 L1/L2）

- 复核对象：卡 `2026-11-11-np5-review-low-items-addenda`；内容提交 **`f9eb64e1`**。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 2 观察）**；五处低项全部按 append-only 处置到位，
  收尾 L1 的 593 反例与 608=598+10 限定经我从 11 片 evidence 独立复算**逐数吻合**。

## 1. 五处低项的处置（逐项独立核）

### 收尾 L1（双口径去重方法限定）—— 我从 608 条 entries 独立复算

- 先按 id 段解析 608 条（4 段 559 + 3 段 49）：**算术表达式站点 541**（intermediate 230 /
  accumulation 132 / output 179）——与 addenda 一致。
- 按 `(kernel, kind, index)` 去重：**531**，移除的正是 **10 个 `rigid_com_position` tail**
  （`acc:0/1/2`、`int:0–3`、`out:0–2`，各跨 `kinematics`/`com_position` 两行各一次）。
- `rounding-note` **67** 条 = 49 条 3 段 row 级 + 18 条带 kernel 的 4 段；按行独立不参与去重。
- **531 + 67 = 598 = 608 − 10** ✓；反例复算：把 3 段 row 级 id 也去行前缀后产生 **15 个共享 key**
  （10 个算术 tail + `rounding-note:0..4`，分别跨 **15/14/12/6/2** 行），严格集合去重 **554**、
  按「每个共享 key 计一次重复」**593**（均 ≠ 598）——addenda 的三组数字全部对上。
- 结论仍为 608 = 598 + 10，且明确「不得把 608 读成唯一表达式数」。

### 收尾 L2（片数口径）—— 已对齐
- addenda 写明 **11 个站点片 + 4 个设备证据片 + 5 张治理卡**，并指明以 receipt `slice_count=11`
  与 README §1 的 11 个站点片为准；历史提交说明不改（历史字节不可改写）——与 `slice_count=11` 一致。

### NP5-02 L1（recipe-difference 站点分层）—— 已按我建议分层+列 id
- addenda §6 全列 A-1a source 级 3 站点 id：`sparse_mass_copy_sparse_mass_update_sparse_mass_diag:accumulation:2`、
  `small_prefix_validate_small_prefix_factor_2..6:accumulation:1`、`rigid_mass_solve:accumulation:3`；
  并把 `rigid_mass_solve:accumulation:5`（A-1b landing `not-decidable` → RE-1 device 级 `recipe-difference`）
  单列为「同站点两层、不重复计入 source 级 3 站点」——与我复核结论一致。

### NP5-02 L2（§4→§3 交叉引用）—— 已更正，写明最终口径变化在 §3。
### NP5-03 L3（`unimplemented=7` vs `remaining=4`）—— 已加注
- 新增 `np5-03/addenda.md`：7 为历史快照（3 条已 closed-implemented），现行剩余 **4**（卡 H blocked、
  不计覆盖），以 `unimplemented_remaining=4` 为准；不删历史清单、不改计数。

## 2. append-only / 字节 / manifest / 扫描 / 门禁

- **历史文件逐字节未改**：closing 3 文件 vs `5a5f0d9c`（README `f9a6db17…`/8,501、receipt
  `6d296a80…`/10,471、selfcheck `2350832a…`/1,262）；np5-02 4 文件 vs `ac840ff7`；np5-03 4 文件 vs
  `d46f435f`——我逐文件 `git cat-file blob` 比对，**11/11 UNCHANGED**。
- **manifest 重算**：closing 3→4、np5-03 4→5、np5-02 5（addenda 追加），我按行重算 sha/bytes 与文件集合
  **全部相等**；表头均注明本次 addenda 与「历史文件 unchanged」。
- **items.json 9 条**：`NP5-02-L1/L2/O1`、`NP5-03-L3`、`NP5-01a-CLOSING-L1/L2`、`D-2-O1/O2`、
  `W4-GOV-§3-TYPO`；处置分布 `fixed_append_only 5 / recorded_no_action 3 / already_corrected 1`，
  与卡 acceptance 逐条对应；每条含 source/issue/disposition/target。
- **W4 §3 笔误**：`docs/development-evidence/plan/W4-dependency-governance.md`（13,791 B）§9 已 append-only
  更正「24 片 = B0 4 + 实体 20」→「实体 7 卡 × 3 片 = 21，总数 25」，并注明原 §3 为算术笔误；
  §9 由提交 `316b28dd` 引入——我 `git log -S` 核实。
- **扫描/门禁/编码**：policy_scan 对 3 个 addenda + items/README/receipt 6 文件 **0 命中**、
  `--self-test` **34/34 PASS**（我重跑）；该卡四门禁 fmt/clippy/clippy-cuda passed、unit **1,406/0/402**；
  6 个新/改文件 UTF-8/LF 无 BOM、无 NUL、无裸 CR。
- 提交 `f9eb64e1` 仅 `.normify/**` + 3 个既有证据目录（addenda+manifest）+ 新卡证据目录；
  **无 src/tests/fixtures/scripts 改动**。

## 3. 发现

### 观察
- **O1 ERRATA-E O1 注记不在本卡提交**：`np5-e-metric-errata/addenda.md`（归一化摘要注记）实际由
  **W8-00 提交 `96b8d201`** 引入，本卡 `f9eb64e1` 未含；内容是 append-only 注记（历史字节不改、
  指明 `historical_snapshot` 仅用于指路）——口径正确，但按派工它属于本卡复核清单，记一条指针：
  该项在 W8-00 第一节覆盖，我复核 W8-00 时会再核（本卡不重复计缺陷）。
- **O2 W4 §3 更正随附在 NP5-01b 提交里**：§9 更正由 `316b28dd`（NP5-01b 卡提交）引入而非独立治理提交；
  §9 正文写清了更正原因与算术，可追溯，但提交归属略分散，建议后续 append-only 更正尽量随同名治理卡提交。

## 4. 非声明

append-only、零 GPU、零窗；不改判据/参考/容差、不改历史字节；不宣称 NP01/NP02 通过、精度完成、
上游等价/位级等价/双平台。
