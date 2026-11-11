# A-2 政策违规撤回 addendum v2（版本化修正版）

- version: 2
- date: 2026-11-11
- supersedes: `addenda.md@0ba40a0bf907b465fd21b81cb064be04e57162f5346b75fb3a6a09b8a457ee62`（2,030 B）
- supersedes_reason: RE-2(r2) 曾把本 addendum 原地改写——旧 2,030 B 不是新 4,179 B 的前缀（标题/表头/末段被改），且只写「历史版本见提交 `df5fc8aa`」而无版本化文件。按 Chief 2026-11-11 裁定改为版本化：`addenda.md` 恢复 v1 原字节（历史字节不得静默改写），修正内容由本文件承载。
- quote_policy: 引用被撤回原文必须逐字；本文件只在 §2 的显式标记引用块内保留 v1 原文（含其原始分隔符写法），块外不使用任何写法复制或规避。

## 1. 撤回 5 处正面表述（改写后口径，无正面结论）

QA 对 A-2 复核（PASS）指出 `evidence.json` 存在正面「位级」与「等价/一致」连写表述，违反「不得写
正面位级结论」政策、并与本片 `bit_level_guarantee:false` 矛盾。本文件撤回以下 5 处（历史字节不改）：

| # | entry | 原措辞位置 | 撤回后写法 |
| --- | --- | --- | --- |
| 1 | `crb:rigid_mass_matrix:accumulation:2` | `why_engine_semantic` | 同一 f32 加法、次序不同；**未做位级测试、不提供位级保证**（`bit_level_guarantee:false`） |
| 2 | `crb:rounding-note:1` | `label` | 两边 f32 加法次序不同；**未做位级测试、不提供位级保证** |
| 3 | `crb:rounding-note:1` | `product_anchor.expression` | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |
| 4 | `kinematics:rounding-note:1` | `label` | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |
| 5 | `kinematics:rounding-note:1` | `product_anchor.expression` | 与冻结内核为**同一 source 级写法**；未做位级测试、不提供位级保证 |

声明：
- **63 站点分类不变**（n/a 32 / identical 27 / engine-semantic 4）；`identical` 仅表示 source 级表达式形态对应。
- `bit_level_guarantee:false` 仍成立；全片**不提供位级保证、未做位级测试**。
- A-2 `evidence.json`/`README.md`/`receipt.json` 为历史 landing 字节，不改；本文件与 manifest 为派生更新。

## 2. 被撤回原文（逐字引用，policy_scan 显式豁免块）

<!-- policy-scan:quote:start -->
撤回/supersede 声明：以下 5 处原文已被 §1 撤回、作废，仅作审计引用；本块内为 v1 记录的逐字节选（保留原始分隔符写法）：
| # | entry | 位置 | 逐字原文（节选） |
| --- | --- | --- | --- |
| 1 | `crb:rigid_mass_matrix:accumulation:2` | `why_engine_semantic` | 「同一 f32 加法、次序不同（表 rounding-note:1 已登记『两边加法次序不同但结果位级·等价』）」 |
| 2 | `crb:rounding-note:1` | `label` | 「两边 f32 加法次序不同但结果位级·等价」 |
| 3 | `crb:rounding-note:1` | `product_anchor.expression` | 「…与冻结内核位级·一致」 |
| 4 | `kinematics:rounding-note:1` | `label` | 「…与冻结内核位级·一致」 |
| 5 | `kinematics:rounding-note:1` | `product_anchor.expression` | 「…与冻结内核位级·一致」 |
<!-- policy-scan:quote:end -->

## 3. r2 归因更正 + policy_scan 修复（2026-11-11）

1. **归因更正**：历史版末段曾写「本文件自身扫描 0 命中（否定形式豁免）」——该归因**错误**。
   当时的 0 命中源于旧 `policy_scan` 只做无分隔符精确串匹配，而引文使用「位级」+分隔符+「等价/一致」写法，
   被漏检（**不是**否定豁免，且该结果不代表合规）。QA G1 已实测分隔符绕过：在「位级」与「等价/一致」
   之间加 中点/空格/连字符/下划线的变体均 0 命中。
2. **工具修复**：匹配前**归一化分隔符**（去掉空白/中点/连字符/下划线/斜杠/句点后查核心词，核心词同样
   归一化），否定豁免保留（不得/未/不/无/非/禁止/not/no/never），并修正「不同/不仅/不但」被误当否定的
   bug。18 字符否定窗口局限写进工具 docstring（近距否定可豁免、远距否定可能漏判；扫描器是护栏，须配人工复核）。
3. **G2 引用块豁免（本次新增）**：仅当文本位于带标记的 quote 块（`<!-- policy-scan:quote:start -->` …
   `<!-- policy-scan:quote:end -->`）且块内**含撤回/supersede/作废声明**时才豁免；未标记、无声明、
   未闭合、块外一律照常命中。引用必须**逐字原文**，禁止拆写/改写规避。构造自测新增 8 条（标记块=0、
   未标记≥1、无声明≥1、未闭合≥1、块外命中、拆写变体、英文 core），自测总数 **20/20 PASS**。
4. **工具版本化入仓**：`scripts/np-gate/policy_scan.py`，sha256
   `e28ddb899cd65c909a7677235b03e829bec5e44715bd5bebd119dddd8019e97b` / 8,013 B。工具头注保留
   「NOT in the 93 guards inputs；加入 guards 需用户重签」。
5. **重扫（修复后实跑）**：
   - A-2 `evidence.json`：**5 命中**（历史字节中 5 处已被 §1 撤回）。
   - `addenda.md`（v1 历史字节）：**7 命中**（历史分隔符写法；按 G1 已恢复为 v1 字节、不改历史，仅作审计）。
   - `addenda-v2.md`（本文件）：**0 命中**（§2 引用块豁免；块外无正面表述）。
   - A-2 `README.md`：**1 命中**，为否定式非声明句被 18 字符窗口漏豁免（非正面结论）；历史 landing
     字节不改，人工复核确认合规。
   - `receipt.json`：**0 命中**。
   - 清单口径：`FORBIDDEN_CORES` 为唯一口径；A2-POL receipt 的历史 `forbidden_phrases` 字段不再作为
     口径（本行 supersede；派生 manifest 已重算）。
6. **笔误更正**：历史版开首「append-orderr」应为「append-only」（历史字节不改，本行更正）。

## 4. L1 更正：`policy_tool.landed` 时点

RE-2 evidence 曾写 `policy_tool.landed: true`；该标志在 `949bf18b`（RE-2 主提交）时点不成立——
修复版工具由后续 POL-SCAN 卡落仓（`e5315d95` 工具入仓、`09b4094a` 收口）。本次在 RE-2 evidence
更正为 `landed_at: e5315d95`，历史字段保留并在 `landing_at_949bf18b` 子对象中标注 supersede。

## 5. 观察项（RE-2 复核，记录不处置）

- `mass_multiply` 18 站点属 **A-3** 范围（A 批剩余），RE-2 只记 `no-reference-entry`，不重复判定。
- 设备级**行级**分类与 source 级**逐站点**分类粒度不同：RE-2 `device_rows` 是按行的映射结论，
  逐站点 source 级证据在 `np5-01a-a2`；两者不互相替代，报告不写覆盖率。
