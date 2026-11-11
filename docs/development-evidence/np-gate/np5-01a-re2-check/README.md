# NP5-01a RE-2 复核（QA）：A-2 归因更正/重扫 + A-2 设备级对照

- 复核对象：卡 `2026-11-11-np5-01a-re2-a2-device-level-policy-fix`（r2）；内容提交 **`949bf18b`**（旧卡 abandon `d1818d85`，干净点重建）；同批 **POL-SCAN `09b4094a`** 的工具落地另卡复核（本目录 §4 摘要）。
- 性质：只读复核（零 GPU、零窗）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，2 一般 + 1 低 + 2 观察）**——policy 修复与设备级映射主体正确；**一般项=A-2 addenda r2 非 append（旧字节被改写）+ 用「引文拆写」规避刚修好的扫描器**。

## 1. 第一节：policy_scan 修复与重扫（我逐项实测）

- **分隔符归一化 ✅**：我构造 `位级·等价`、`位级 等价`、`位级-等价`、`位级_等价`、`位级/等价`、`位级.等价`、`位级等价`、`bit-level equivalent` —— **8/8 全部命中**（旧版对带分隔符变体 0 命中的漏洞已关闭）。
- **否定豁免仍生效 ✅**：`不得写位级等价`、`未做位级测试、不提供位级保证`、`不同但结果相同` → **0 命中**；`不仅/不但` 不再被误当否定（工具自测第 12 条覆盖）。
- **自测 ✅**：`scripts/np-gate/policy_scan.py --self-test` 我独立跑 **12/12 PASS**。
- **重扫计数 ✅**：用修复后工具对 A-2 `evidence.json` → **5 命中**（= 已撤回的 5 处历史字节），对 A-2 `addenda.md` → **0 命中**，与 README/receipt/selfcheck 声明一致。
- **A-2 landing 字节未改 ✅**：`evidence.json`/`README.md`/`receipt.json`/`logs/selfcheck.txt` 在 `df5fc8aa`→`949bf18b` 逐 blob 未变；A-2 `manifest.txt` 我独立重算 **5/5**（含 addenda、不自列，`be9ffd46…`/717）与派工值一致。
- **归因更正 ✅ / 笔误处理 ⚠️**：新 addenda §2 明写历史「0 命中（否定形式豁免）」为**错误归因**、实为分隔符漏检，并更正 `append-orderr`→`append-only`——**但这两处是通过改写旧文件实现，不是 append**（见 G1）。

## 2. 第二节：A-2 设备级映射（3 行）

- **产品 PTX 身份我复算全对**：kinematics `262e8330…`/46,921、crb/mass_matrix `5e4fcd82…`/17,007（磁盘实测 sha+bytes 均匹配）；`.cu` 源在 `target/.../RE-2-draft/`；**PTX 实体未入仓**（RE-2 证据目录 0 个 `.ptx`，`ptx_landed=false`）。
- **冻结参考**：`smooth.ptx` `4179acd0…`/993,601、48 entries、**0 `.loc`**（我直接检查）⇒「no lineinfo」前提成立。
- **3 行分类**：`kinematics` 33 → `not-decidable`（`frozen-artifact-only`+`no-lineinfo`，理由：产品单 entry 内联 vs 参考多 entry 拆分、且 arch/NVRTC 不同）；`crb` 30 → `engine-semantic`（产品单 entry 合并树序累加+矩阵组装 vs 参考 `_crb_accumulate`+`_M`，带 `why_engine_semantic` 与 `bit_level_guarantee:false`，**明写不作数值等价结论**）；`mass_multiply` 18 → `not-decidable`（`no-reference-entry`）——分类与理由码合法。
- RE-2 `manifest.txt` **4/4** 我独立重算；门禁 1406/0/402；证据目录 UTF-8/LF 无 BOM/CR/NUL。

## 3. POL-SCAN（简要，另卡详核）

`scripts/np-gate/policy_scan.py` = **4,559 B / `59765f71…`** 已入仓，与 receipt/evidence 记的 sha/bytes 一致；12/12 自测通过；guards/freeze/fixtures 未触碰（我核 `e5315d95`/`09b4094a` 文件清单）；头注「NOT in the 93 guards inputs；加入需用户重签」、docstring 记录 18 字符否定窗口局限。详见 `np5-01a-policy-scan-check/`。

## 4. 发现

### 一般
- **G1 A-2 addenda r2 非 append（破坏前缀不变式）**：旧 `df5fc8aa` 版 2,030 B，新 4,179 B，**`new[:2030] != old`**（首行标题、§1 表头「原措辞（节选）」列、末段都被改写）。文件自称「历史版本见提交 `df5fc8aa`」，但没有版本文件（如 `addenda-v2.md`）+ supersedes 字段。按团队口径「改历史证据必须前缀不变；结构变更走新版本文件」——本处应走版本文件或恢复旧字节后纯追加。
- **G2 用「引文拆写」规避刚修好的扫描器**：新 addenda §2 自述「本版引文一律拆写（如「位级」+「等价」），不写出可被归一化匹配的连写形式」。这相当于把禁词扫描当对手：本次是引用历史原文，尚可理解，但**该手法一旦成为惯例，后续片可用同一手法写出真实位级断言而 0 命中**，扫描护栏失效。建议给扫描器加**显式引用/历史块豁免**（如 fenced quote + 必带「撤回/supersede」标记才豁免）或规定「引用必须人工复核」，而不是靠拆字。

### 低
- **L1 时序不精确**：RE-2 的 `evidence.json`/`receipt.json` 写 `policy_tool.landed: true`，但在 `949bf18b` 时 `scripts/np-gate/policy_scan.py` **尚不存在**（`git cat-file -e` 失败），工具由 `e5315d95`（POL-SCAN 卡）落地；终态 sha/bytes 一致，属记录时点问题，建议后续 addenda 注明落地卡。

### 观察
- **O1 范围表述**：RE-2 第二节含 **A-3 的 `mass_multiply` 18 站点**，而 A-2 只有 kinematics 33 + crb 30；标题「A-2 设备级」与内容范围不符，建议注明属 A-3 前瞻或移到 A-3/RE-3。
- **O2 粒度混用**：device **行级**分类「crb → engine-semantic」与 A-2 **逐站点 source 级**分类（crb 多数 identical）粒度不同，易被读成 crb 整行都是 engine-semantic；建议 README 明写「行级 device 结论 ≠ 逐站点 source 分类」。

## 5. 非声明

本复核对 policy 修复、重扫计数、设备级映射与证据卫生负责；**不宣称** NP01 通过/精度完成/位级等价/上游等价/双平台，不改判据/参考/容差。G1/G2 由 Chief/Backend 定处置（A-2 addenda 是否改走 addenda-v2 + supersedes、扫描器是否加引用豁免）。
