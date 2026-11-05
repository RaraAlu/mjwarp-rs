# scene-flex 文本夹具 eol=lf 治理 —— QA 只读复核

- 任务：`2026-11-05-repo-scene-flex-fixtures-eol-lf-check`（owner QA；**QA 自登记**，Chief 2026-11-05 夜指派；登记提交 `da220308`）
- 复核对象：治理卡 `2026-11-05-repo-scene-flex-fixtures-eol-lf`（**verified**）→ 规则落地 `cce3e2e3`（父提交 `a7c401b4`）+ 落库后 fresh clone 复核追加 `c7eb4737`
- 触发：QA G-2 采集窗复核观察 **O1**（`docs/development-evidence/render-bvh/r06-flex-multiframe-capture-g2-check/`）
- 边界：**只读复核 + 只写本目录**；零 GPU；不改 `.gitattributes`、fixtures、G-2/治理证据、脚本
- 方法：独立定位改动提交、比对 diff 与 blob id、`git check-attr`、本机工作树 sha 核对、**fresh clone（core.autocrlf=true）** 15 文件 sha/bytes 比对、父提交对照复现 O1；QA 自检 **20/20 PASS**

## 1. 结论

**PASS（0 阻断，0 观察）。** 规则只新增、位于 `*.npz -text -diff` 之前；**不 renormalize、不改任何既有 blob**（15 个夹具 blob id 父提交→HEAD 逐一相同）；fresh clone（autocrlf=true）后 **3 个文本夹具 sha256/bytes 与 G-2 `files-sha256.json` 逐条一致**（LF：8,948 / 3,805 / 1,729 B，无 CRLF）、**12 个 npz 逐字节不变**，合计 15/15；clone 干净、G-2 证据 manifest 仍 16/16；父提交对照下 3 文本 CRLF、sha 不一致（复现 O1，证明规则必要）。治理证据 manifest 11/11、编码/卫生与 diff 忠实性均通过。

## 2. 逐项核对（6 项口径 + 对照）

### 2.1 `.gitattributes` 改动（PASS）
- 改动提交 `cce3e2e3` 的 `.gitattributes` diff：**只新增 4 行、0 删除、既有行逐字未动**。4 行分解 = **1 行规则** `fixtures/render-bvh/scene-flex/** text eol=lf` + 2 行中文依据注释 + 1 空行（与卡面「附 1–2 行中文依据注释」一致；Chief 口径的「+1 行」即指这 1 行实质规则）。
- 规则在 `.gitattributes` 第 **288** 行，`*.npz -text -diff` 在第 **295** 行 → **规则在前、后匹配优先，npz 仍为 `-text -diff`**。
- `.gitattributes` UTF-8/无 BOM、无 TAB、无裸 CR、行尾 LF。

### 2.2 不 renormalize / 不改 blob（PASS）
- 15 个落库夹具在 **父提交 `a7c401b4` 与 HEAD 的 blob id 逐一相同**（`git rev-parse <rev>:<path>` 比对，0 变化）→ 无 renormalize、无 blob 重写。
- 改动提交范围 = `.gitattributes` + 卡 + `.normify/runs`/`.normify/modules`（工具自动刷新）+ 治理证据目录；**不含 `fixtures/`、`src/`、`tests/`、`scripts/`、冻结 spec**。

### 2.3 属性生效（PASS）
- `git check-attr text diff eol`：3 个文本夹具 = **`text: set` / `eol: lf`**；npz = **`text: unset` / `diff: unset`**。
  （npz 的 `eol: lf` 由子树规则继承，但 `text: unset` 表示不做转换；fresh clone 12/12 逐字节一致已实证。）

### 2.4 本机工作树（PASS）
- 15 个夹具 sha256/bytes 与 G-2 `artifacts/files-sha256.json` **15/15** 一致（工作树字节未因规则改动而变化）。

### 2.5 fresh clone（core.autocrlf=true）字节安全（PASS）
| 文件 | clone 后 bytes | CRLF | 与证据 sha/bytes |
| --- | --- | --- | --- |
| `capture.json` | 8,948 | 0 | ✅ |
| `cam-poses.json` | 3,805 | 0 | ✅ |
| `README.md` | 1,729 | 0 | ✅ |
| 12 个 npz | 逐字节不变 | — | ✅ 12/12 |
- 合计 **15/15** 匹配；clone 工作树 `git status --porcelain` 为空；G-2 证据目录 manifest 在 clone 中仍 **16/16** 自洽（既有 `-text` 规则不受影响）。

### 2.6 对照必要性（PASS）
- 同一 clone 以 autocrlf=true checkout **改动父提交 `a7c401b4`** 并把 15 个文件先删后物化：3 个文本夹具变 **CRLF**（9,240 / 3,967 / 1,745 B）且 sha 与记录不一致 → **复现 O1**；12 个 npz 仍一致（差异只来自文本夹具）。

### 2.7 治理证据完整性（PASS）
- 证据 `docs/development-evidence/repo-scene-flex-fixtures-eol-lf/`：manifest 首行 `#`、**11/11 自核**；全部文件 UTF-8/LF/无 BOM、裸 CR/TAB 0 命中。
- `artifacts/gitattributes.diff` 与实际 diff（default context）逐字一致；`artifacts/clone-postcommit.json` 中 verdict 布尔**全 true**；卡 verified、addenda 记录落库后复核（`c7eb4737`）。
- 复跑治理卡 `scripts/verify_postcommit_clone.py` → **exit 0**、`15_files_match=True / text_no_crlf=True / npz_binary=True / rule_present=True`（`logs/repro-postcommit-clone.log`）。

## 3. 复现与证据

- `scripts/verify_o1_review.py`：QA 独立复核脚本（A 规则/diff、B blob/scope、C attr、D 工作树、E fresh clone、F 父提交对照、G 治理证据）；零 GPU、只读、`sys.dont_write_bytecode`；clone 落 `target/o1-verify/clone`（不落仓）。
- `logs/verify-o1.log` / `logs/verify-o1.json`：**RESULT: PASS (20/20)**。
- `logs/repro-postcommit-clone.log`：复跑治理卡 `verify_postcommit_clone.py` 的原始输出（exit 0）。
- `facts.json`：机器可读事实（diff/blob/attr/clone/对照/治理证据）。
- `manifest.txt`：本目录 sha256 + bytes + 证据目录内相对路径（首行 `#`，不自列）。

## 4. 非声明

- 本复核只确认 **eol=lf 治理规则落地与 fresh clone 字节安全**；不改变任何采集/判据结论，不声明多帧验收/发布依据；O2/O3（G-2 证据措辞）仍按 Chief 口径记已知不返工。
