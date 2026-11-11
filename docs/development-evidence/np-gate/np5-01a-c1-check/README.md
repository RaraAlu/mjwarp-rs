# NP5-01a C-1 复核（QA）：B-1 低项闭环 + C 批 flex 族 52 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-c1-flex-source-level`；内容提交 **`2d3a17a4`**（登记 `db9d4739`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 2 观察）**。

## 1. 第一节：B-1 两低 + 观察闭环

- **① L1 肯定式声明 ✅**：`policy_scan.py` = **11,712 B / `1d3a88b9…`**；`--self-test` **24/24 PASS**（新增 4 条否定声明用例）。我另写构造用例实测：
  - 否定声明 → **命中（不豁免）**：`未撤回`、`不作废`、`没有撤回`、`无撤回`、`非作废`（均带足够远距 filler 避免 18 字符窗口干扰）；
  - 肯定声明 → **豁免**：`撤回声明`、`撤回(5)`、`supersedes:`、`作废声明`；
  - `--list-quote-blocks` 输出与 C-1 receipt `quote_block_inventory` 一致。
- **② L2 状态词表 ✅**：词表 addendum §6 = **6,771 B / `453c5c91…`**，给出 `no-reference-expression` vs `no-reference-entry` 的定义/使用场景/「不是什么」/判定顺序（前者=站点本身非可比较表达式，用于 `n/a`；后者=有语义点但参考侧无 entry/PTX，用于 `not-decidable` reasons）；C-1 README §2 同步；**B-1 的 21 条 `no-reference-expression` 用法与定义一致**（全部 n/a）。
- **③ 人工复核痕迹 ✅**：B-1 addenda §3 记录约定（`reviewed_by` 或 evidence receipt `quote_block_inventory` + `--list-quote-blocks` 核对）；C-1 receipt 给出清单：A-2 `addenda-v2.md` 行 29–38（closed/affirmative，`reviewed_by: backend-2026-11-11`）、行 49–50（文档说明块，非豁免块）——与工具输出一致。
- **④ B-1 append-only ✅**：B-1 新增 `addenda.md` = **2,676 B / `933cec55…`**；B-1 `README/evidence/receipt/selfcheck` 四个 landing 文件在 `bbd7c7cc`→本提交**逐 blob 字节未改**；B-1 manifest 重算 **5/5**（724 B，含 addenda）。

## 2. 第二节：C-1 flex 族 52 站点

- **站点集合 ✅**：52/52，与 `sites.json` 的 5 行并集**逐 id 相等**（flex_hessian 15 / flex_edge_passive 11 / flex_position 9 / flex_edge 9 / flex_face 8），无缺/无多/无重复。
- **直方图 ✅**：`{identical 32, n/a 18, engine-semantic 2}` = 52，与 evidence/receipt/README 一致；**累计 269**（40+36+63+78+52，交集为空）。
- **2 条 `engine-semantic` ✅**：`flex_hessian:accumulation:3`（同边 3×3 块叠加：单线程顺序累加 vs 上游 `wp.atomic_add`）、`:accumulation:4`（对角 = −Σ 边块：顺序累减 vs 原子加负块）——均带 `why_engine_semantic`（99/68 字）+ `bit_level_guarantee:false`，明写「f32 求和次序可能不同，不声明等价」，**未作数值等价结论**。
- **锚点 ✅**：我全量解析 **0 错误**（46 条 file:line + 6 条 `no-reference-expression`，后者均为 n/a 且符合新词表定义）；抽样行内容正确：`flex_hessian.rs:122/123/148/155` ↔ `passive.py:1071/1093/1769/1808`。
- **必填字段/类型映射 ✅**：52/52 齐全；`policy_scan` 对 C-1 evidence/README **0 命中**；manifest **4/4**（539 B）；selfcheck 17 项全 PASS；门禁 1,406/0/402；UTF-8/LF 无 BOM/CR/NUL；无覆盖率/通过率表述。
- **证据哈希与派工值一致 ✅**：evidence `5530f7d6…`/79,631、README `b2ee44e9…`/7,208、receipt `afb016fe…`/5,494、manifest `f7d2467b…`/539、selfcheck `00a016b3…`/1,120。

## 3. 发现

### 低
- **L1 声明否定词只覆盖中文**：`DECLARATION_NEGATORS` 仅中文（未/不/没/无/非/否…），我构造的英文否定声明块（`never supersede`、`no withdrawal`）仍被豁免（实测 0 命中，期望 1）。英文 `FORBIDDEN_CORES` 已有，建议声明否定词也补 `not/no/never`（或 docstring 明确仅支持中文声明）。

### 观察
- **O1**：`--list-quote-blocks` 对 A-2 引用块报 `reviewed:false`（块内无 `reviewed_by` 标记），人工痕迹实际保存在 C-1 receipt（符合「或 receipt 块清单」的约定）；建议在 docstring/README 写明该字段语义是「块内标记存在与否」，避免被读成「未复核」。
- **O2**：C-1 README 已记录 `flex_hessian_validate` 的 field-table 描述滞后（表写 `ok` bool/`status` f32，产品现写 i32 `flags`），按非缺陷处理；建议在下一张 errata 卡给该场表补一条更正注记，避免后续片继续引用旧描述。

## 4. 非声明

本复核对 B-1 低项闭环与 C-1 站点/分类/锚点/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
