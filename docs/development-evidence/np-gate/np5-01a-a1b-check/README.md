# NP5-01a A-1b 复核（QA）：mass_solve solve 半段 36 站点 source 级对照

- 复核对象：卡 `2026-11-11-np5-01a-a1b-mass-solve-solve-source-level`；内容提交 **`3f68a621`**（含 A-1a 四条裁定 addenda 与 A-1a manifest 回写）。
- 性质：只读复核（零 GPU、零窗、不改判据/参考/容差）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，1 一般 + 1 低 + 3 观察）**。

## 1. 结构/字段（我独立重算）

- **站点集合**：`evidence.json` 36 条，与 `A-1-split.json` A-1b 列表**逐 id 相等**（无缺、无多、无重复）；`site_count=36`、`pending_manual=0`；A-1a(40) ∪ A-1b(36) = **76**，交集 0，与拆分清单并集**完全相同**。
- **必填字段**：36/36 齐全；`type_mapping` 36/36 有 product/upstream/type_class、`double_in_product=false` 36/36。
- **分类直方图**（我自算）：`{n/a:21, not-decidable:1, identical:11, engine-semantic:3}` = 36，与 `evidence.json`/receipt/README 一致；`not_decidable_reasons` = `{no-lineinfo:1, frozen-artifact-only:1}`（均词表内）；`n/a` 21 条均有 `why_not_applicable`。
- **`engine-semantic` 3 条我逐条核**：`rigid_mass_solve:accumulation:7`（kind2 块内三级循环 vs 上游 level 扫描 + atomic）、`mixed_solve…:accumulation:3`（D 乘时机/分支结构：产品按 flag 预乘、上游统一乘 D）、`sparse_solve…:accumulation:0`（`flags[i]==-1 ? b[i] : D*b` 分支 vs 上游统一乘）—— 三条均带 `why_engine_semantic` 与 `bit_level_guarantee:false`，且 entry 明确两条乘 D 是**互斥分支**（非连乘），判定可接受。
- **锚点可解析性**：`product_anchor` 36/36 为 `src/**` 内 `file:line` 且行号在文件范围内；`reference_anchor` 全部可解析到上游 `smooth.py`（3314/3330/3332/3341/3346/3354/3360/3400/3415/3417/3419/3430/3457/3480/3486）或显式 `no-reference-entry`。
- **抽站点独立复核**：`cuda.rs:79/89/101/113/125`（tmp / x 指针 / tile 列扫描 `__frcp_rn` / scalar `value-=` / kind2 L^-T）、`resident.rs:68/94/97`（`split_sources` / saved 视图 / x 指针）、上游 `smooth.py:3314/3346/3360/3419/3480`（`_solve_LD_sparse_fused` / level 扫描 / `x_out *= D` / `value/L_in` 除法 / `wp.tile_cholesky_solve`）—— 与各条 expression/分类相符。
- **无通过类表述**：README+evidence 的「覆盖率/通过率/位级等价/字节相等/上游等价/双平台/NP01 通过」均在否定句/非声明中（逐处见日志）。

## 2. A-1a 裁定回写（本提交同时落地）

- `np5-01a-a1a/addenda.md`（2,347 B）与 Chief 修订裁定 R1–R4 逐条一致：R1 认可 engine-semantic；**R2 改判 recipe-difference**（value_equivalent/analytic + MathDx opt-in 注记 + 不记 pin-needed/不进 deferred）；R3 保持 not-decidable（不引入 `pending-a1`）+「A1 已具备、RE-1 升级」；R4 identical 前提补「收缩许可、未证位级」。
- **append-only 我以 blob 字节核对**：A-1a 的 `evidence.json`/`README.md`/`receipt.json`/`logs/selfcheck.txt` 在 `de8f44dc`→`3f68a621` **字节未变**；`manifest.txt` 重算纳入 addenda（5/5 全对）。
- `receipt.json` 记录 `a1a_rulings_recorded=true` 且 R1–R4 四个布尔全 true，与 addenda 一致。

## 3. 证据/manifest/门禁/边界

- `manifest.txt` **4/4** 全对（README/evidence/receipt/logs/selfcheck，不自列；我独立重算）。
- 输入 4 哈希我复算全对（同 A-1a）：`964dab77…`/`5321f312…`/`46282084…`/`ec5fc257…`；`tree.head=f5ded6c9…` 为该卡父提交。
- 门禁：fmt/clippy/clippy-cuda exit 0、unit **1,406 / 0 / 402**。
- 边界：提交 12 files = A-1b 证据 + A-1a addenda/manifest + `.normify`；无 `src/`/`tests/`/`fixtures/`/`scripts/` 改动。
- 编码：全部 UTF-8/LF、无 BOM/CR/TAB/NUL。

## 4. 发现

### 一般
- **G1 分类一致性（与 A-1a 同型，跨片）**：`resident_mass_substitute:accumulation:0` 记 `identical`，label 明写「**同 rigid_mass_solve 的 solve 半段（kind==0/1/2 三条代换路径）**」、notes 写「派生片算术文本与 rigid solve 半段逐字相同」；但同一文本的 **tile solve 分支**在 `rigid_mass_solve:accumulation:5` 记 **`not-decidable`**（无 lineinfo / frozen-artifact-only，device 级留 RE-1）。同一文本两判、`identical` 强于 `recipe-difference`，会掩盖该派生条目的遗留项。建议：改判 `not-decidable`，或 `identical` 仅限 kind0/kind2 半段 + tile 分支单列；A-1a 的 `resident_mass_factor_派生:accumulation:0` 同型，建议一并处理并同步派生计数说明。

### 低
- **L1 词表引用旧值**：README §1 写 `fcbec8f9…`/4,560 B，而 §5 与 receipt 用现值 `46282084…`/4,867 B（现值我复算正确）；同 A-1a L3，建议统一。

### 观察
- **O1**：README §4「4 个 host 驱动记 `host-path-no-ptx`、`_solve_LD_sparse_fused` 等记 `capture-scenario-miss`」实为 **RE-1 计划**；本片 evidence 的 `not_decidable_reasons` 只有 `no-lineinfo`/`frozen-artifact-only` 两类，没有这两类理由码。建议在 §4 明写「RE-1 将记」，避免被当作本片已记。
- **O2**：`resident_mass_substitute:accumulation:0` 的产品锚点用 `cuda.rs:91`（rigid 文本），而 README §2.2 写 resident 派生锚点在 `resident.rs:94-97`；口径差异已由 notes「文本逐字相同」解释，建议后续统一锚点约定（rigid 源 + resident 派生点二选一并标注）。
- **O3**：`output` 类 `identical` 的比较维度是「无窄化/加宽 + f32 直写」，与算术 `identical`（指令/表达式同形）粒度不同；建议在证据顶层注明该维度，避免混读。

## 4b. 处置（Chief 2026-11-11 裁定）

- **G1**（Chief 2026-11-11 裁定：**采纳改判**，与 A-1a 同型，**由 RE-1 卡 append-only 处置**。）
- 低项/观察项（由 RE-1 卡 append-only 处置，本卡按记录保留）：
  - L1 词表引用统一为 `46282084…`/4,867（与 A-1a L3 同修，由 RE-1 修正）

## 4b. 处置（Chief 2026-11-11 裁定）

- **G1**（Chief 2026-11-11 裁定：**采纳改判**，与 A-1a 同型，**由 RE-1 卡 append-only 处置**。）
- 低项/观察项（由 RE-1 卡 append-only 处置，本卡按记录保留）：
  - L1 词表引用统一为 `46282084…`/4,867（与 A-1a L3 同修，由 RE-1 修正）

## 5. 非声明

本复核对 source 级对照记录、addenda 落地与证据卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。G1 与 L1 待 Backend 下一片处理，处置方式由 Chief/Arch 定。
