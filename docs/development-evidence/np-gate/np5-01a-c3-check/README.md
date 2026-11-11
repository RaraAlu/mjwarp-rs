# NP5-01a C-3 复核（QA）：C-2 两观察处置 + C 批 rne/gravity 43 站点 source 级

- 复核对象：卡 `2026-11-11-np5-01a-c3-rne-gravity-source-level`；内容提交 **`37a21e59`**（登记 `ec60c7c2`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 1 低 + 1 观察）**。

## 1. 第一节：C-2 两观察处置

- **① 对称声明词 ✅**：`policy_scan.py` = **14,885 B / `8753685b…`**；`--self-test` **31/31 PASS**。`QUOTE_DECLARATIONS` 含中：撤回/作废/撤销/收回，英：supersede/withdraw/withdrawn/withdrawal/retract/retracted。我构造实测：
  - **豁免（0 命中）**：`withdrawn:`、`withdrawal:`、`retracted:`、`withdraw this:`、`撤回：`；
  - **命中（1）**：`not withdrawn:`、`未撤回：`（否定词 + 词边界正确）。
- **② 否定 veto ✅（块级）**：`quoteblocks` 对 `withdrawn: never supersede` 块给出 `affirmative_declaration=False`；工具自测的否决用例通过。
- **③ docstring 词表 ✅**：4,017 字符 docstring 列出全部中英声明词与否定词表（我逐词检索确认）。
- **④ errata 转发 ✅**：`errata-0001`（flex_hessian_validate field-table 滞后）已从 C-2 转发到 C-3 `errata_pending`（`o2_errata_forwarded=true`，status=forwarded to closing addenda errata list），field-table 历史未改。

## 2. 第二节：C-3 rne/gravity 43 站点

- **站点集合 ✅**：43/43，与 `sites.json` 的 3 行并集**逐 id 相等**（rne_acc 18 / gravity_compensation 13 / rne_bias 12），无缺/无多/无重复。
- **直方图 ✅**：`{identical 29, n/a 13, engine-semantic 1}` = 43；**累计 380**（337+43，交集为空）。
- **engine-semantic 1 条 ✅**：`rne_bias:rne_bias:accumulation:1`（子树力向父汇聚：产品按 body 逆序单线程确定性累加 vs 上游 `body_tree` 逆层并行 `wp.atomic_add`），带 `why_engine_semantic` + `bit_level_guarantee:false`，明写「f32 求和次序可能不同，不声明等价」，**未作数值等价结论**。
- **锚点 ✅**：我全量解析 **0 错误**（40 条 file:line + 3 条 `no-reference-expression`，均 n/a 带 reason）；抽样行内容正确：`rne.cu:35/37/39` ↔ `smooth.py:1331/1366/1372`（`cacc_out` 初始化、`local_cacc` 取父、`cdof_dot*qvel` 累加）、`rne_bias.rs:274` ↔ `smooth.py:1444`（`wp.atomic_add`）。
- **必填字段/类型映射 ✅**：43/43 齐全；`policy_scan` 对 C-3 evidence/README **0 命中**；manifest **4/4**（539 B）；selfcheck 16 项全 PASS；门禁 1,406/0/402；UTF-8/LF 无 BOM/CR/NUL；无覆盖率/通过率表述。
- **证据哈希与派工值一致 ✅**：evidence `3f9a7c86…`/63,172、README `57c39e88…`/5,661、receipt `529130c2…`/4,312、manifest `8a9f8791…`/539、selfcheck `4b724b4b…`/1,088。

## 3. 发现

### 低
- **L1 veto 与 18 字符否定窗口的交互（假阴性边界）**：块级 veto 生效后，块内断言若紧邻否定声明（落入 18 字符否定窗口），仍会被**否定窗口**豁免。实测：
  - `{S}\nwithdrawn: never supersede\n结果位级等价\n{E}` → **0 命中**（block veto 为真、但仍被 `never` 窗口豁免）；
  - 同文本在否定声明与断言之间插入 24 字符 filler → **1 命中**。
  工具自测只覆盖「英文断言 + `result ` 前缀」布局（否定词恰好落在 18 字符窗口外），测不到本布局。建议：对 **veto 块**内的断言禁用否定窗口豁免（或直接在 veto 块内照主题断言计数），并补一条「断言紧邻否定声明」的自测。

### 观察
- **O1**：其余行为符合预期——声明词表、否定词边界、块级 veto、豁免需闭合标记块等均已按口径实现；18 字符窗口本身是已在 docstring 声明的启发式局限（护栏 + 人工复核）。

## 4. 非声明

本复核对 C-2 观察处置与 C-3 站点/分类/锚点/卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。
