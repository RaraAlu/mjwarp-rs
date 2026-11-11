# NP5-01a POL-SCAN 复核（QA）：`scripts/np-gate/policy_scan.py` 修复版入仓

- 复核对象：卡 `2026-11-11-np5-01a-policy-scan-landing`；内容提交 **`09b4094a`**（工具与证据随 `e5315d95` 登记提交一并落地）。
- 性质：只读复核（零 GPU）；本目录只写本卡证据。
- 结论：**复核通过（0 阻断，0 一般 + 0 低 + 1 观察）**。

## 1. 工具身份与内容

- `scripts/np-gate/policy_scan.py` = **4,559 B / `59765f71…`**，与 POL-SCAN receipt/selfcheck 及 RE-2 evidence 记的 sha/bytes **逐位一致**（我复算）。
- **归一化实现 ✅**：匹配前去掉空白/中点/连字符/下划线/斜杠/句点后再查核心词，核心词同样归一化（读码确认 + 实测）。
- **否定豁免 ✅**：`不得/未/不/无/非/禁止/not/no/never` 18 字符前瞻窗口；`不同/不仅/不但` 先从窗口剔除（修真实假阴性 bug）。
- **docstring ✅**：写明 18 字符窗口局限「护栏而非证明，须配人工复核」；头注「NOT in the 93 guards inputs；加入需用户重签」。

## 2. 实测（我独立跑）

- `--self-test` **12/12 PASS**。
- 变体命中：`位级·等价`/`位级 等价`/`位级-等价`/`位级_等价`/`位级/等价`/`位级.等价`/`位级等价`/`bit-level equivalent` → **8/8 命中**。
- 否定豁免：`不得写位级等价`、`未做位级测试、不提供位级保证`、`不同但结果相同` → **0 命中**。

## 3. 写域边界与证据

- 我核 `e5315d95`/`09b4094a` 两个提交的文件清单：仅 `scripts/np-gate/policy_scan.py` + docs evidence + `.normify`；**guards（`scripts/validate/**` 下 guards 输入）、freeze script、`fixtures/` 零改动**；receipt `guards_unchanged=true`、`freeze_script_untouched=true` 与实测一致。
- POL-SCAN `manifest.txt` **3/3** 我独立重算（README/receipt/logs，不自列）；receipt `self_test_total/passed=12/12`、`tree.head=949bf18b…`（父提交）；门禁 fmt/clippy/clippy-cuda exit 0、unit **1,406/0/402**。
- 证据目录 UTF-8/LF 无 BOM/CR/TAB/NUL。

## 4. 观察

- **O1**：12 条自测用例内嵌在工具文件里，而工具文件本身被排除在扫描口径外（「定义即被测物」）——合理，但更干净的做法是把用例放独立 test fixture，避免「扫描器不扫自己」的解读空间。

## 5. 非声明

本复核只确认工具入仓身份、语义、边界与自测；**不宣称**任何位级保证或 NP01 通过。该工具现在已是后续片 selfcheck 的唯一禁词口径（`FORBIDDEN_CORES`）；A-2 addenda 的「引文拆写」手法问题见 RE-2 复核卡 G2。
