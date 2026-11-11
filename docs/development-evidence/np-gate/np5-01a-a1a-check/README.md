# NP5-01a A-1a 复核（QA）：mass_solve factor 半段 40 站点 source 级对照

- 复核对象：卡 `2026-11-11-np5-01a-a1a-mass-solve-factor-source-level`；内容提交 **`de8f44dc`**；A-1a 裁定 addenda + manifest 回写 **`3f68a621`**。
- 性质：只读复核（零 GPU、零窗、不改判据/参考/容差）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，1 一般 + 3 低 + 4 观察）**。

## 1. 结构/字段（我独立重算，非引用 Backend 自检）

- **站点集合**：`evidence.json` 40 条，与 `A-1-split.json` A-1a 列表**逐 id 相等**（无缺、无多、无重复）；`site_count=40`、`pending_manual=0`。
- **必填字段**：40/40 含 `id/kind/label/product_anchor/reference_anchor/type_mapping/classification/basis/confidence/premise/evidence_pointer`；`type_mapping` 40/40 有 product/upstream/type_class，`double_in_product=false` **40/40**（与 README「全链 f32、产品 double=0」一致）。
- **分类直方图**（我自算）：`{n/a:22, engine-semantic:1, not-decidable:2, identical:12, pin-needed:1, recipe-difference:2}` = 40，与 `evidence.json.classifications_histogram`、receipt、README 一致。
- **分类附加字段**：`recipe-difference` 2 条均带 `value_equivalent:true` + `value_equivalence_basis:"analytic, not bit-tested"` + `product_form/reference_form/why_value_equivalent/bit_tested:false`；`not-decidable` 2 条均带 `not_decidable_reasons[]`（`no-reference-entry`/`capture-scenario-miss`，均在 6 类词表）+ `missing_evidence`；`n/a` 22 条均有 `why_not_applicable`；`pin-needed` 1 条有 `pin_spec`/`expected_magnitude`。
- **锚点可解析性**：`product_anchor` 40/40 为 `src/**` 内 `file:line`，行号均 ≤ 文件行数；`reference_anchor` **34 条**为 `file:line` 可解析（上游 `smooth.py`/`include/warp-headers/`），**6 条**显式 `status:"no-reference-entry"` 并带 reason。
- **抽 2–3 站点独立复核**（行内容逐条比对，日志 `logs/independent-check.txt`）：`cuda.rs:18/38/43/53/80`（status 指针 / `bool tile=end-begin>6;` / tile|scalar Schur / scalar sqrt / kind2 循环）、`smooth.py:1171/1191/1229/1236/1291`（`_qLD_acc` 除法 / `_qLDiag_div` 倒数 / small 标量分支 / `wp.sqrt` / `for tile in m.M_tiles` 选择）—— 与各条 `expression`、`classification`、`engine-semantic`/`pin-needed`/`not-decidable` 判定相符。
- **无通过类表述**：README+evidence 的「覆盖率/通过率/位级等价/字节相等/上游等价/双平台/NP01 通过」全部出现在**否定句或非声明**中（逐处列在日志）。

## 2. A-1a 裁定 addenda（`np5-01a-a1a/addenda.md`）与 Chief 修订裁定一致

- **R1** `intermediate:3` 认可 `engine-semantic`（同一数学解、不同分解路径、位级不保证、不得当 identical）✅
- **R2** `accumulation:3` **改判 `recipe-difference`** + `value_equivalent:true` + `analytic, not bit-tested` + product/reference form + why + `bit_tested:false`；注明 MathDx 为产品 opt-in、默认不走、若变默认需重评；不记 pin-needed、不进 deferred、NP5-04 只记 watch ✅
- **R3** `accumulation:0/1` **保持 `not-decidable`**（不引入 `pending-a1` 新词）+ note「A1 codegen 已具备、由 RE-1 升级为 device 级」✅
- **R4** `accumulation:2` identical 前提补「收缩许可、未证位级相等」✅
- **append-only**：`evidence.json`/`README.md`/`receipt.json`/`logs/selfcheck.txt` 在 `de8f44dc` 与 `3f68a621` 两个提交的 blob **字节完全相同**（我逐文件 `git cat-file` 比对）；`manifest.txt` 重算为派生更新（510→593 B，纳入 `addenda.md`）。

## 3. 证据/manifest/门禁/边界

- `manifest.txt` **5/5** 全对（README/addenda.md/evidence.json/logs/selfcheck.txt/receipt.json，不自列；我独立重算 sha256+bytes）。
- 输入 4 哈希我复算全对：field-table `964dab77…`、exec-spec `5321f312…`、vocab-addendum `46282084…`、A-1-split `ec5fc257…`。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406 passed / 0 failed / 402 ignored**。
- 边界：提交 12 files = A-1a 证据 + `.normify`（卡+4 gate 日志）+ A-1b 新增；**无 `src/`/`tests/`/`fixtures/`/`scripts/` 改动**、guards/freeze script 未动。
- 编码：全部 UTF-8/LF、无 BOM/裸 CR/TAB/NUL（blob 级核，非仅工作区）。

## 4. 发现（按严重级别）

### 一般
- **G1 分类一致性（与 A-1b 同型，跨片）**：`resident_mass_factor_派生:accumulation:0` 记 `identical`，但其覆盖文本明确含 **tile 分支**（field-table `mass_solve.json` 的 resident 组 `accumulations` 写「kind==0/1/2 factor 分支（tile 可收缩 Schur + rsqrt.approx；scalar sqrt/div/fmaf）」）；同一算术文本的 tile 分支在 `rigid_mass_solve:accumulation:0/1` 记 **`not-decidable`**（参考 unique 内核未 capture、无 lineinfo）。同一文本两判、且 `identical` 强于 `recipe-difference`，会让 12 条 identical 被误读为「无遗留项」。建议随 addenda 一并处理：改判 `not-decidable`，或 `identical` 仅限 scalar 半段 + tile 分支单列（并同步直方图/README/receipt/manifest 的派生说明）。

### 低
- **L1 数字笔误（事实性）**：`not-decidable` 的 `missing_evidence` 写「**2603 entries** 无 entry」；冻结 PTX 实际 **303 entries**（6×48 + 15），应为 303。
- **L2 待复核项计数不一致**：`README.md` §3 列 **4 条**待复核项，但 `evidence.json.requires_decision` 与 `receipt.requires_decision` 只有 **3 条**（缺 `accumulation:2` identical 前提项）；标题还写着「见 evidence.json.requires_decision」，读者会以为同源。
- **L3 词表引用两值**：README §1 与 `evidence.vocabulary_authority` 用旧值 `fcbec8f9…`/4,560 B，而 README §4 与 receipt 用现值 `46282084…`/4,867 B（同一工作材料两个 sha）；现值我复算正确，旧值应删或标注版本差异。

### 观察
- **O1**：12 条 `identical` 的 `basis` 均为「source-level（无位级测试）」——满足 Chief 要求；但词表把 `identical` 定义为「归一化指令流等价」，本片未做指令流比对，建议在证据顶层或后续片明确「source-identical」这一粒度，避免下游把 `identical` 当指令流/位级结论。
- **O2**：部分条目的单行锚点对应多语句表达式（如 `accumulation:2` 锚 `cuda.rs:53` 只覆盖 `diagonal=sqrtf(pivot)`，而 `pivot=fmaf`/`reciprocal=`/`__fmul_rn` 在 43/54/63；`small…:accumulation:1` 锚 `small/cuda.rs:32` 只覆盖 sqrt，rcp 在 34）；分类不受影响，建议后续片用 line range。
- **O3**：锚点行号是 **`.rs` 文件行**（产品 CUDA 嵌在 raw string 内），非 NVRTC program 行；PREWARM 记录 `program = rust − 1`，后续 device 级映射需写明该口径，避免与场表行号混用。
- **O4**：addenda 已把 `accumulation:3` 改判 `recipe-difference`，但 `evidence.json` 直方图仍记 `pin-needed 1`（append-only 设计使然）；建议 addenda 补一行「按修订裁定的派生计数（pin-needed 0 / recipe-difference 3）」，否则读者需自行换算。
- **O5**：40 条 `confidence` 全为 `medium`，未区分（如 tile not-decidable 与 scalar identical 的可信度不同），后续片可细化。

## 4b. 处置（Chief 2026-11-11 裁定）

- **G1**（Chief 2026-11-11 裁定：**采纳改判**——相关站点改 not-decidable 或拆 scalar identical + tile not-decidable，不得整条 identical；**由 RE-1 卡 append-only 处置**。）
- 低项/观察项（由 RE-1 卡 append-only 处置，本卡按记录保留）：
  - L1 「2603 entries」→ 303（由 RE-1 修正）
  - L2 requires_decision 补第 4 条（accumulation:2 identical 前提）（由 RE-1 修正）
  - L3 词表引用统一为 `46282084…`/4,867（由 RE-1 修正）

## 4b. 处置（Chief 2026-11-11 裁定）

- **G1**（Chief 2026-11-11 裁定：**采纳改判**——相关站点改 not-decidable 或拆 scalar identical + tile not-decidable，不得整条 identical；**由 RE-1 卡 append-only 处置**。）
- 低项/观察项（由 RE-1 卡 append-only 处置，本卡按记录保留）：
  - L1 「2603 entries」→ 303（由 RE-1 修正）
  - L2 requires_decision 补第 4 条（accumulation:2 identical 前提）（由 RE-1 修正）
  - L3 词表引用统一为 `46282084…`/4,867（由 RE-1 修正）

## 5. 非声明

本复核对 source 级对照记录与证据卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差；G1 与 L1–L3 待 Backend 在下一片（或 addenda v2）处理，处理方式由 Chief/Arch 定。
