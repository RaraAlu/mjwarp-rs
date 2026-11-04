# `.gitattributes` 证据目录 `-text -diff` 规则 QA 复核（只读）

任务：`2026-11-05-repo-evidence-gitattributes-rule-check`（owner QA）。复核对象：执行卡 `2026-11-05-repo-evidence-gitattributes-rule`（verified）→ 提交 **`ce4a7d18`**；复核时 HEAD `b88f7964`。日期：2026-11-05（空闲段）。
**边界：只读 + 只写本目录；零 GPU；不回改 `.gitattributes`、执行卡证据或任何已 verified 证据。**

## 1. 结论

**PASS-with-observations（0 阻断）。** 四口径全部成立：规则只加 1 行且在末尾；无 renormalize、无 blob 重写；属性前后对照符合预期；**独立 fresh clone 复跑**与提交者证据一致（且我用全量 manifest 扩大了覆盖面）；`.ps1` 与 `docs/evidence/**` 行为不变；回滚方法与不 renormalize 理由在证据里写清。
4 条低观察（O1–O4），均不影响规则正确性与可回滚性。

## 2. 四口径核对

### 2.1 只加一行且在末尾（PASS）
- `git diff 48bc2840..ce4a7d18 -- .gitattributes`：**+1/−0**，唯一新增行 `docs/development-evidence/** -text -diff`。
- 行数 **322 → 323**，新行在**末尾**；前 322 行与父提交**逐行逐字节相同**。

### 2.2 无 renormalize / 无 blob 重写（PASS）
- `git diff-tree -r --name-status 48bc2840 ce4a7d18` 共 16 项：`.gitattributes`(M) + 执行卡(M) + `.normify/modules/mjwarp/index.md`(M，派生) + 4 个 runs(A) + 8 个新证据文件(A)；**此外没有任何 M 条目**。
- 抽样 10 个既有证据/代码文件，`48bc2840:path` 与 `ce4a7d18:path` 的 **blob id 10/10 相同**（含 g10-p5-summary/receipt.json、render-bvh README、criteria-purpose-audit manifest、`scripts/validate/repo-write-lock.ps1`、`src/lib.rs`）。
- 无 `--renormalize` 痕迹；提交前后工作树除本次复核自身文件外无翻动。

### 2.3 独立 fresh clone 复跑（PASS，与提交者一致）
方法与命令：`git -c core.autocrlf=true -c core.longpaths=true clone --local --no-checkout <repo> target/gitattributes-verify-qa` → checkout 目标提交 → `git rm -r --cached . && git reset --hard HEAD` 强制按当前属性全量重建 → **把 clone 里每一个 `docs/development-evidence/**/manifest.txt` 与提交 blob 逐字节比 sha256**（不采信提交者拷贝；`core.longpaths=true` 必需，否则 `docs/evidence` 下超长路径会让 checkout/reset 失败）。

| 状态 | clone HEAD | manifest 数 | 与 blob 一致 | 失配 | attr text=unspecified |
| --- | --- | --- | --- | --- | --- |
| 规则前 | `48bc2840` | 161 | 147 | **14**（全部 CRLF 改写） | 14 |
| 规则后 | `b88f7964` | 162 | **162** | **0** | **0** |

- 规则前 14 个失配均为**未被既有规则覆盖**的目录（如 `g10-p5-summary`、`g10-p5-mesh-cap20-check`、`np01-e-criterion-a2-(check)`、`mesh-polygon-gt16-*`、`write-lock`、`repo-slim-2026-11-02` 等，逐条见日志）；规则后 clone 工作树 `git status` 干净。
- **复跑提交者的 18 个样本**：与主工作树对照 **17/18 一致**，唯一失配 `docs/development-evidence/mesh-polygon-gt16-host-diagnostics/manifest.txt` —— 与提交者 `verify/clone-compare.json` 的记录一致。
- 结论一致：clone-after 100% 保字节；clone-before 有漂移（规则必要性成立）。

### 2.4 属性前后对照与既有规则不受影响（PASS）
`git check-attr text diff --source=<rev>` 抽样 8 个文件：

| 文件 | 规则前 | 规则后 |
| --- | --- | --- |
| `docs/development-evidence/g10-p5-summary/receipt.json` | unspecified / unspecified | **unset / unset** |
| `docs/development-evidence/g10-p5-summary-check/manifest.txt` | unspecified / unspecified | **unset / unset** |
| `docs/development-evidence/np01-e-criterion-a2/manifest.txt` | unspecified / unspecified | **unset / unset** |
| `docs/development-evidence/criteria-purpose-audit/manifest.txt` | unset / unset | unset / unset（既有规则，不变） |
| `docs/evidence/resident-crb/receipt.json` | unset / unset | unset / unset（`docs/evidence/**` 行为不变） |
| `scripts/validate/repo-write-lock.ps1` | unspecified / unspecified | unspecified / unspecified（`.ps1` 规则未动） |
| `src/lib.rs` | unspecified / unspecified | unspecified / unspecified |
| `fixtures/render-bvh/scene-a/model-312.json` | unspecified / unspecified | unspecified / unspecified |

既有 322 行逐字未动 → 既有 `*.ps1 text eol=lf`、`docs/evidence/**`、`render-bvh/**` 等规则定义与优先级不变；新规则只覆盖此前未覆盖的 `docs/development-evidence/**`。

### 2.5 回滚与不 renormalize 记录（PASS）
执行证据 README 写清：效果范围（`docs/development-evidence/**` 全部已跟踪文件）、不 renormalize 的理由（会重写 blob = 改既有证据字节、历史 manifest 失配）、回滚方法（删行即可；旧工作树漂移用 `git checkout -- <paths>` 从 blob 恢复，并给出 `git hash-object --no-filters` 判定法）。既有 blob 未变已由 2.2 独立证实。

### 2.6 边界（PASS）
`src/`、`tests/`、`fixtures/`、`docs/evidence/` 均未被该提交触碰；`.git/info/attributes` 未入库。

## 3. 观察清单（按严重度）

| # | 级别 | 内容 | 建议 |
| --- | --- | --- | --- |
| O1 | 低（文字准确性） | 执行证据 README 写 `.git/info/attributes`「**不存在/未创建**」，实际该文件**自 2026-10-30 已存在**（3 条 guards `-text` 规则；mtime 早于本卡，本卡未修改） | 文案改为「已存在、本卡未修改」；不影响规则效果 |
| O2 | 低（证据自核口径） | 执行证据 `manifest.txt` 头写 `repo-relative path`，但 8 条条目实际是**证据目录内相对路径**（按头注释从仓根解析会全部失败；按 base dir 解析 8/8 通过）；与其它证据目录的仓相对路径约定不一致 | 改头注释或改为仓相对路径 |
| O3 | 低（样本代表性） | 提交者 18 个样本中 17 个位于**既有 `-text` 规则已覆盖**的目录，故 clone-before 只暴露 1 个失配；我全量 161 个实测 **14 个**未覆盖目录失配（同方向，规则必要性更强） | 后续同类证据用全量或分层抽样，避免低估影响面 |
| O4 | 低（commit message） | `ce4a7d18` 提交正文含未展开的 `\n`/`\` 转义（可读性差） | 下次注意提交正文转义；不影响内容 |

## 4. 顺带复核（advisory，不属于本卡验收项）

- **R07 `658c3677`：PASS**。README 8621→8633 B、行数 81 不变；**CR=0 / TAB=0 / 无 BOM（LF-only）**；`transmittance`、`receipt.json` 两词与反引号均已正确恢复；§3 `shade_splats` 更正句与 `24244bc2` 逐字相同；`receipt.json` blob 未变；manifest 2/2 自核通过。
- **G10 cosmetic `b88f7964`：PASS-with-observation**。receipt 8591→8593 B，**唯一差异 = `qa_addenda` 行补回 2 空格缩进**（75 行不变），JSON 合法，`requirement_counts`/`requirements`/`uncovered` 逐字段不变，README 未动，manifest 2/2 自核通过。观察（低）：提交信息称「去尾部空行」，但两版文件尾部均为 `}\n`、无尾部空行可去——描述不准确，不影响内容。

## 5. 复现与证据

- `scripts/verify_rule.py`：规则 diff、属性前后、blob 抽样、提交范围、执行证据 manifest 8/8、`src/tests/fixtures/docs/evidence` 边界、R07/G10 顺带检查。
- `scripts/qa_clone_compare.py`：独立 fresh clone 对照（before/after 全量 manifest + 提交者 18 样本复跑 + `core.longpaths` 说明）；结果 JSON `logs/qa-clone-compare.json`。
- 日志：`logs/verify-rule.log`、`logs/qa-clone-compare.log`（均 `RESULT: PASS`；UTF-8/LF，由 Python 直写，规避本机 PowerShell `>` 的 UTF-16LE 重定向）。
- `manifest.txt` 7 条（README + facts + 2 脚本 + 3 日志），LF、无 BOM，首行 `#`，路径为仓相对。
