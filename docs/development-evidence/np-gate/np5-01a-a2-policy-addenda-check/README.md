# NP5-01a A2-POL 复核（QA）：A-2 政策违规 append-only 撤回 + 禁词表补强

- 复核对象：卡 `2026-11-11-np5-01a-a2-policy-addenda`；内容提交 **`df5fc8aa`**（登记 `8abab74a`）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，1 一般 + 1 低 + 2 观察）**——5 处撤回全覆盖、append-only 成立；**一般项=policy_scan 对带分隔符的变体漏检（可绕过）**。

## 1. 撤回覆盖与 append-only（我逐项核）

- **5 处撤回全覆盖 ✅**：`addenda.md` 逐条列出并改写，我按 evidence 原字节定位 5 处正面表述并核 `withdrawn` 清单一一对应：
  1. `crb:rigid_mass_matrix:accumulation:2.why_engine_semantic`（「结果位级等价」）
  2. `crb:rounding-note:1.label`（「位级等价」）
  3. `crb:rounding-note:1.product_anchor.expression`（「与冻结内核位级一致」）
  4. `kinematics:rounding-note:1.label`（「与冻结内核位级一致」）
  5. `kinematics:rounding-note:1.product_anchor.expression`（「与冻结内核位级一致」）
  —— 与我 A-2 复核的 G1 所述两处**是同一问题**；Backend 实际扫出并撤回 5 处（比我抽到的 2 处更全，含 3 处「位级一致」）。改写文「同一 f32 加法、次序不同；未做位级测试、不提供位级保证」符合要求。
- **A-2 append-only ✅**：`2cede9b6`→`df5fc8aa`，`evidence.json`(83,749)/`README.md`(3,379)/`receipt.json`(2,099)/`logs/selfcheck.txt`(358) **逐 blob 字节未改**；`addenda.md` = 2,030 B / `0ba40a0b…`（与派工值一致）；A-2 `manifest.txt` 我独立重算 **5 条全对**（含 addenda、不自列，`9109897e…`/661）且集合与目录相等。
- **分类与位级声明不变 ✅**：63 站点分类仍 `{n/a 32, identical 27, engine-semantic 4}`；evidence 内 `bit_level_guarantee` 仍全 `false`（字节未改）；receipt `classifications_unchanged=true`、`bit_level_guarantee_holds=true`、`withdrawn_statements=5`、`policy_scan_hits_in_addenda=0`。

## 2. policy_scan 语义（我构造用例实测）

- **否定豁免正确 ✅**：`不提供位级保证`、`不写位级等价` → 0 命中；`不同` → 无误报；`不仅`、`不但` → 正确命中（不作为否定前缀）。
- **英文变体 ✅**：`bit-level equivalent` 命中；`bitwise equal` 等列入清单；`evidence_schema.py` 已引用 `policy_scan`（`validate_evidence()` 接入）。
- 边界：提交仅 docs evidence + `.normify`；目录 UTF-8/LF 无 BOM/CR/NUL；门禁 fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**；`tree.head=8abab74a…` 为父提交。

## 3. 发现

### 一般（须修）
- **G1 禁词表可被分隔符绕过（实测）**：`policy_scan.FORBIDDEN` 是**无分隔符的精确串**（`位级等价`/`位级一致`…），实测
  - `「结果位级·等价」已撤回` → **0 命中（漏检）**；
  - `两边加法次序不同但结果位级·等价` → **0 命中（漏检）**；
  - `结果位级等价`（无点）→ 正确命中。
  而**本次 addenda 正是用带间隔号的「位级·等价」写原文**，才得到「0 命中」——`addenda.md` §末却把 0 命中归因于「否定形式豁免」，**归因错误**（0 命中是分隔符造成的漏检，不是豁免）。同理 `位级 等价`、`位级-等价`、`位级_等价` 都能绕过。`receipt.forbidden_phrases` 里写的 `位级·等价`/`位级·一致`（带点）与代码（无点）也不一致。
  建议：匹配前归一化分隔符（去 `·`、空白、`-`、`_`）或用正则 `位级[\s·\-_]?等价`；纠正 addenda 归因；`receipt.forbidden_phrases` 与代码对齐。**这正是你要求「policy_scan 禁词与否定豁免正确」的核心项，改完后续片才真正受保护。**

### 低
- **L1 笔误**：`np5-01a-a2/addenda.md` 标题「append-orderr」应为「append-only」。

### 观察
- **O1 工具未入仓**：`np5-01a-tools/policy_scan.py` 只在 `target/`，后续所有片都依赖它，但仓库无法复核该工具版本/口径（RE-0 的脚本是入仓的）；建议按写域批准后入仓（如 `docs/development-evidence/np-gate/tools/` 或 `scripts/np-gate/`）。
- **O2 否定窗口**：豁免窗口为短语前 18 字符；否定词太远会误报、太近会过度豁免（如「不得不…位级等价」），建议在工具 docstring 记录该局限，并在关键片上人工复核。

## 4. 非声明

本复核对撤回完整性、append-only 与扫描器语义负责；**不宣称**任何位级保证/NP01 通过，不改判据/参考/容差。G1 可由 Backend 在下一片把工具归一化 + 纠正归因；该修完后续片 selfcheck 才有实效。
