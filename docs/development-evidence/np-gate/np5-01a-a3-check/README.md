# NP5-01a A-3 复核（QA）：RE-3 L1/O1 处置 + A 批余量 com/mass_multiply 40 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-a3-com-mass-multiply-source-level`；内容提交 **`12156259`**（登记 `b3bbc29c`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**。

## 1. 第一节：RE-3 的 L1/O1 处置

- **L1 逐行字段补齐 ✅**：新增 `np5-01a-re3/rows-fields-v1.json`（11,733 B，`row_count: 24`）：
  - `engine-semantic` **15 行**：`why_engine_semantic`（= evidence basis 全文）+ `bit_level_guarantee: false`；
  - `not-decidable` **7 行**：`not_decidable_reasons[]` + `why_not_decidable` + `missing_evidence`；
  - `n/a` **2 行**：`why_not_applicable`。
  我逐行与 `evidence.json` 的 row/classification/sites 比对，**0 字段问题**；`source_evidence.sha256=f5988b7c…`/94,815 指向原文件，**原 evidence/README/receipt/selfcheck 四个 landing 文件字节未改**；`addenda.md`（2,003 B）说明补齐方式与 `supersedes: null`（原文件未改写）；RE-3 manifest 我独立重算 **6/6**（744 B）全对。
- **O1 口径更正 ✅**：`rows-fields-v1.not_decidable_reasons_split_measured` + addenda §2 明确 **`no-reference-entry` 4 行 47 站点**（tendon_velocity 7 + applied_force 12 + kinetic_energy 13 + sleep_wake 15）与 **`no-lineinfo`+`frozen-artifact-only` 3 行 26 站点**（tendon_wrap 9 + flex_edge 9 + flex_face 8）= **7 行 73 站点**，并明示派工摘要 40/33 有误——与 evidence 实测一致。

## 2. 第二节：A-3 40 站点（A 批最后一片）

- **站点集合 ✅**：40/40，与 `sites.json` 3 行并集**逐 id 相等**（com_position 14 / com_velocity 8 / mass_multiply 18），无缺/无多/无重复。
- **直方图 ✅**：`{identical 18, engine-semantic 3, n/a 19}`；**累计 443+40=483**；A 批 179（A-1 76 + A-2 63 + A-3 40）收口。
- **3 条 engine-semantic ✅**：`com_position:rigid_com_position:intermediate:4`（三重循环 vs `mat@diag@mat.T`）、`accumulation:0`（串行树序 vs `wp.atomic_add`）、`accumulation:2`（同 A-2 先例）——均带 `why_engine_semantic` + `bit_level_guarantee:false`，明写「不作数值等价结论」。
- **`mass_multiply` 18 条口径 ✅**：validate/borrowed_validate 的校验与 status 输出、视图绑定、rounding-note 共 13 条为 `n/a`（带 `why_not_applicable`），accumulation/output 共 5 条 `identical`；README 记录 RE-2 device 级 `no-reference-entry`（不重复判定）。
- **锚点 ✅**：40 条全部 `file:line` 可解析、**0 错误**（产品 `mass_matrix/multiply/**` 与上游 `support.py:_mul_m/_mul_m_dense`）。
- **证据/门禁 ✅**：policy_scan **0 命中**；manifest **4/4**（539 B）；selfcheck 30 项全 PASS；门禁 1,406/0/402；哈希与派工值一致（README `13085701…`/5,883、evidence `c9a9676f…`/53,532、receipt `c42dbbbd…`/4,224、manifest `337b3367…`/539、selfcheck `2392debb…`/1,843）；UTF-8/LF 无 BOM/CR/NUL；无覆盖率。

## 3. 发现

### 低
- **L1 A-3 条目缺 `kind`/`label` 字段**：A-3 的 40 条条目有 `id/product_anchor/reference_anchor/type_mapping/classification/basis/confidence/premise/evidence_pointer`，但**没有** `kind` 与 `label`（A-1a/A-1b/A-2/B-1/C-1..C-4 都有）；本片 selfcheck 的「kind match」只能从 id 段推导。建议补两字段或写明本片 schema 变化。

### 观察
- **O1 总站点数口径（重要）**：`rigid_com_position` 内核同时出现在 **`kinematics` 行（A-2，11 站点）** 与 **`com_position` 行（A-3，14 站点）**；我按 (kernel, kind, index) 去行前缀比对，**10 个 tail 在两个片中重复计数**（accumulation:0/1/2、intermediate:0–3、output:0–2）。因此 483（及计划中的 608）是「场表行×站点」口径，**去重后唯一表达式为 598**（608−10）。建议收尾 addenda 同时给出两个口径并注明来源（`sites.json` 的 `kinematics` 与 `com_position` 两行共享该内核站点），避免 608 被读成 608 个唯一表达式。
- **O2**：engine-semantic 的 why 措辞（「数学等价但实现与 fma 收缩形态不同，不作数值等价结论」）沿用 A-2 先例、可接受。

## 4. 非声明

本复核对 RE-3 L1/O1 闭环与 A-3 站点/分类/锚点/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
