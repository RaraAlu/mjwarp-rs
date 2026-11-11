# NP5-01a 收尾 addenda 复核（QA）：12/11 片、双口径 608/598、errata、L manifest、累积观察

- 复核对象：卡 `2026-11-11-np5-01a-closing-addenda`；内容提交 **`5a5f0d9c`**（登记 `113a9cfe`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 2 低 + 1 观察）**。

## 1. 分片清单与 608 核对（我逐片从 evidence 复算）

- 11 个站点片站点数：a1a **40** / a1b **36** / a2 **63** / a3 **40** / b1 **78** / c1 **52** / c2 **68** / c3 **43** / c4 **63** / d1 **88** / d2 **37** = **608**，逐片与 receipt `slices[]`、README §1 一致。
- 批次合计：**A179**（40+36+63+40） + **B78** + **C226**（52+68+43+63） + **D125**（88+37） = **608** ✓；2 个附录单元（`src/math/fma.rs`、`render_light_dir_normalize`）单列 `appendix_unit_count=2`、**不计入 608**，与「场表行×站点」口径一致。
- 设备级/治理片：RE-0/RE-1/RE-2/RE-3 与 K/L/A2-POL/POL-SCAN/QA-ADD 的提交号在 receipt `re_slices`/`governance_cards` 与 README §1 均对得上。

## 2. 双口径 608 / 598 / 10（1 低）

- receipt/README：`np5_01a_sites=608`、`np5_01a_unique_expressions=598`、`duplicate_tails=10`；来源 `rigid_com_position` 在 kinematics 行（A-2 11 站点）与 com_position 行（A-3 14 站点）双登记；10 个 tail = `acc:0/1/2`、`int:0–3`、`out:0–2`；cross-ref `np-gate/np5-01a-d1/receipt.json#section1.o1_dual_count`。
- **可复算性提示（L1）**：按 README/receipt 字面方法「(kernel,kind,index) 去行前缀后去重」对**全部 608 条**执行，会得到 **15 个共享 tail**（另有 5 个 `rounding-note:*` 在 a1a/a1b/a2/a3/c2/c4/d1/d2 之间同名）⇒ 唯一数变为 **593**。**598 只有在「rounding-note 为按行独立说明条目、不参与表达式去重」的口径下成立**。建议在 README §2 与 receipt `dual_count.method` 明写去重范围（仅 `kind ∈ {intermediate, accumulation, output}` 的算术站点），否则他人按字面复算会对不上（不影响 608/598 的实质结论）。

## 3. A-1 派生计数 / errata / L manifest / 累积观察

- **A-1 派生计数 ✅**：landing（A-1a 40：22/12/2/2/1/1；A-1b 36：21/11/3/1）与 final（A-1a：22/11/3/3/1；A-1b：21/10/3/2）两套直方图我逐项复算：总计 landing = 43/23/2/3/1/4 = **76**、final = 43/21/3/5/4 = **76**；改判来源（R2 `acc:3` pin-needed→recipe-difference、QA-ADD G1 两派生站点 identical→not-decidable、R3 保持 not-decidable）与各片 addenda 一致；历史 evidence 字节未改。
- **errata ✅**：`errata-0001`（`flex_hessian_validate` field-table 滞后）入册，来源 C-2/C-3/C-4，`present_in_c2_evidence=true`，field-table 历史不改。
- **L manifest ✅（我独立重算）**：`r06-flex-multiframe-recipe-params-comment-wording/manifest.txt` **6 行**（README + 4 logs + receipt），sha256/bytes **全对、集合相等** —— 卡 L 的「6/6」口径成立（此前 README 写 7/7 的计数低项由此在收尾确认为 6/6）。
- **累积观察 ✅**：15 条观察均记录/标记闭环或入 errata；完成声明带完整非声明（不宣称 NP01 通过/E 完成/上游等价/双平台/位级保证）。

## 4. 证据/边界

- closing `manifest.txt` **3/3**（487 B，我独立重算）、selfcheck **20 项全 PASS**、门禁 1,406/0/402；`policy_scan` 对 closing README/receipt **0 命中**；UTF-8/LF 无 BOM/CR/NUL。
- 边界：本提交仅 `np5-01a-closing/**` + `.normify/**`；收尾报告为 append-only 聚合，不改任何历史片字节（各片 manifest 仍指向 landing 字节，我抽查 A-1a/A-1b/RE-3 等 landing 文件未变）。
- 哈希：README `f9a6db17…`/8,501、receipt `6d296a80…`/10,471、manifest `33b06f3c…`/487、selfcheck `2350832a…`/1,262。

## 5. 发现

### 低
- **L1 去重方法表述不完整**：见 §2——字面复算 593 vs 声明 598；建议限定「仅对算术表达式站点去重；rounding-note 按行独立、不参与去重」。
- **L2 片数口径不一致**：提交说明/卡片标题写「**12 片** 608 站点」，而 receipt `slice_count=11`、README §1 列 **11 个站点片**（另有 4 个设备级证据片、5 张治理片）。建议统一为「11 站点片（+4 设备证据片 +5 治理片）」，避免总数口径混淆。

### 观察
- **O1**：附录单元不计入 608、A-1 派生计量、errata-0001、L manifest 6/6、15 条累积观察均已核；收尾报告的完成声明只写「已全部分片收口」并带非声明，符合口径（不宣称 NP01 通过/位级保证）。

## 6. 非声明

本复核对收尾报告的计数、口径、errata 与卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
