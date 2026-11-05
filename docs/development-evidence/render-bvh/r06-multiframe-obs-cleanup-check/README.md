# R06 多帧观察项治理 QA 只读复核（O2 + O6）

**结论：PASS-with-observations（0 阻断；2 条低观察）**

- 任务：`2026-11-06-render-r06-multiframe-obs-cleanup-check`（owner QA、P8-渲染与BVH、p60；QA 自登记，依赖观察项治理卡）
- 复核目标：HEAD **`d1653111`**（观察项治理卡 `verified`；登记 `c6988afd`、start `c965e48f`、实现 `d1653111`）
- 基线（改动前）：`1c5ea060`（上一次 QA 多帧收口复核证据提交）
- 复核方式：只读 + 独立复算（不采信执行方日志结论）；QA 脚本自检 **14/14 PASS**；CPU-only、零 GPU
- 本卡登记提交：`adee1031`

## 1. 结论与定级

治理卡的两处证据层改动按卡面 acceptance 执行无误：**仅动证据/文案层、冻结字节不变、manifest 重算自核、tests/fixtures/scripts/spec 零改动**；O6 禁词清零且语义保持。按 Chief 定级规则，2 条观察均只影响追溯/记账文案，**不影响判据与数值，记 observation，不阻断**：

| 编号 | 级别 | 区域 | 摘要 |
| --- | --- | --- | --- |
| O1 | 低 | 追溯路径叶名 | 按卡面 acceptance 改到 `receipt.json.option_a.chief_ruling`，容器层级已正确（修复卡顶层 `option_a`）；但该 receipt 中无 `chief_ruling` 叶（实际为 `chief_ruling_at` + `wording`），严格按指针追溯仍不可解析 |
| O2 | 低 | 记账时序 | 治理卡 `created_at` 写作 `2026-11-06T01:01:00.000Z`（值=本地时间却带 `Z`），比真实 UTC 晚 8h、形式上晚于 `closed_at`（`2026-11-05T17:02:28.285Z`）；仅记账口径 |

## 2. 逐项结果

### 2.1 两处修正只改证据/文案层，冻结字节不变（PASS）

- `git diff --name-only 1c5ea060..d1653111` = **恰 11 个文件**：治理卡 + `r06-flex-multiframe-entity-fix/{README.md,manifest.txt,receipt.json}` + `r06-flex-multiframe-entity/{manifest.txt,receipt.json}` + `r06-multiframe-obs-cleanup/{README.md,manifest.txt,receipt.json,artifacts/changed-files.json,logs/manifest-recompute.log}`。
- G-3 冻结字节：`scripts/validate/render-bvh/guards-r06-multiframe.json` raw sha256 **`64bfecdf…`/11,899 B**（与 G-3 复核记录逐字节一致）；父提交 `1c5ea060` 与 HEAD 的 blob 同为 `bf4b9302…`/11,696 B；`git diff 1c5ea060..d1653111 -- <guards>` 与 `git status -- <guards>` 均空。
- 复核执行方 `changed-files.json`：5 条前后 sha256/bytes 全部用 git blob（`head_before=c965e48f`）与工作树独立复算一致（`artifacts/changed-files-check.json`），guards 段前/后 raw sha 与实测一致。

### 2.2 O2 路径文案生效面（PASS，1 条低观察）

- `r06-flex-multiframe-entity-fix/receipt.json` 的 `option_a.ruling_trace` 与 `README.md` §4 均已写 **`receipt.json.option_a.chief_ruling`**（`artifacts/o2-o6-lines.txt`）。
- 两目录（entity + entity-fix）11 个文本证据文件内 **`criteria.option_a` 0 命中**；仓库范围内旧串 `receipt.json.criteria.option_a` 也已 0 命中。
- 结构交叉核对（`artifacts/path-resolution.json`）：修复卡 receipt **顶层** `option_a`（无 `criteria`）；实体卡 receipt 的 `criteria.option_a.chief_ruling` 存在；修复卡 `option_a` 内实际叶名为 `chief_ruling_at` 与 `wording` → 见观察 O1。

### 2.3 O6 禁词清零与语义保持（PASS）

- 实体卡 receipt `claims.statement` 前后对照（`artifacts/o2-o6-lines.txt`）：
  - before：`…不声明 R06 整族通过、动画通用性或与上游等价；…`
  - after：`…不声明 R06 整族通过、动画通用性，也不构成与上游实现的一致性结论；…`
- 两目录内 exact 禁词「上游等价」**0 命中**；相邻（间隔 ≤1 字）的「上游…等价」**0 命中**（`artifacts/forbidden-terms.json`）。
- 宽窗（间隔 ≤12 字）组合：**无新增**；唯一移除项即本次 O6 目标句（实体卡 receipt `claims.statement`），其余既存量均为否定式「与上游实现的等价结论」措辞，未改动。
- 四项否定语义逐一保留：R06 整族通过 / 动画通用性 / 上游一致性 / 验证窗结论；`claims.acceptance_conclusion` 仍 `none`。

### 2.4 manifest 重算自核（PASS）

- 独立重算（逐条 sha256 + bytes）：`r06-flex-multiframe-entity` **8/8**、`r06-flex-multiframe-entity-fix` **6/6**、`r06-multiframe-obs-cleanup` **4/4**；三份 manifest 首行均为 `#`、均不自列（`artifacts/manifests.json`）。
- 编码卫生（obs-cleanup 证据 5 文件 + 被改动的 5 个证据文件，共 10 个）：UTF-8 可解码、**无 BOM、裸 CR 0、TAB 0**（`artifacts/encoding.json`）。

### 2.5 未动项与边界（PASS）

- `unchanged tests/, fixtures/, scripts/, src/, .gitattributes/, r06-flex-multiframe-criterion/, r06-flex-multiframe-verify-g3/, r06-flex-multiframe-capture-g2/`：上述路径 `git diff 1c5ea060..d1653111` 全空（`artifacts/scope.json`）。
- O1/O3/O4/O5 与 guards eol **明确未做**：obs-cleanup README §3「其余 O1/O3/O4/O5 与 guards eol 不做（等下一次验证窗前随重冻一起动）」、receipt `boundaries.not_done = [O1,O3,O4,O5,guards eol]`、治理卡 acceptance #5 同口径；O1 backlog 卡 `2026-11-05-render-r06-multiframe-guards-eol-backlog` 未被本提交改动且 `status=backlog`（`artifacts/untouched.json`）。
- 零 GPU、CPU-only；未跑四门禁（无代码改动，执行方卡面 checks 为 unchanged/json/forbid_claim，口径成立）。

## 3. 观察清单（低，不阻断）

- **O1 追溯路径叶名**：卡面 acceptance 指定的目标串即 `receipt.json.option_a.chief_ruling`，执行方未偏离；但修复卡 receipt 的 `option_a` 下无 `chief_ruling` 叶（`chief_ruling_at`：日期；`wording`：裁决正文）。严格 JSON 指针追溯会失败，建议后续顺手把叶名精确为 `receipt.json.option_a.wording`（或在 receipt 增加 `chief_ruling` 别名）；仅文案，不影响判据。
- **O2 记账时序**：治理卡 `created_at = 2026-11-06T01:01:00.000Z` 与登记提交时间（`c6988afd` = 2026-11-06 01:01:08 +0800 = 17:01:08Z）不符——值写的是本地时间却带 `Z`，比真实 UTC 晚 8h，故形式上晚于自身 `closed_at`（`17:02:28.285Z`）。建议登记模板统一按 UTC 写；仅记账口径、不影响验收与判据（与既往 refit-fix 观察 O2 同类）。

## 4. 证据与自核

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_obs_cleanup_review.py` | 复核脚本（只读复算 14 项；可复跑） |
| `logs/verify-obs-cleanup.json` | 14 项检查的机器可读结果（PASS 14/14） |
| `logs/verify-obs-cleanup.log` | 脚本原始输出 |
| `artifacts/scope.json` | 变更面 11 文件 + 无关面零 diff |
| `artifacts/changed-files-check.json` | 5 个证据文件前后 sha256/bytes 独立复算 |
| `artifacts/guards.json` | guards raw / 父 blob / HEAD blob 三方一致 |
| `artifacts/manifests.json` | 三目录 manifest 重算（8/6/4）与编码 |
| `artifacts/forbidden-terms.json` | exact/相邻/宽窗禁词扫描与前后差集 |
| `artifacts/path-resolution.json` | O2 路径结构解析交叉核对 |
| `artifacts/o2-o6-lines.txt` | O2/O6 前后逐字对照 |
| `artifacts/encoding.json` | 10 个证据文件编码卫生 |
| `artifacts/untouched.json` | O1/O3/O4/O5 + guards eol 未动声明与 backlog 卡状态 |
| `README.md` / `receipt.json` | 本报告与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 5. 边界与非声明

- 只读复核 + 只写本卡证据目录；不改被复核卡/证据、既有证据、代码、fixtures、guards、`.gitattributes`。
- 本次复核**不产生新验收结论**，不重开任何已 `verified` 判定；不写 R06 整族通过 / 动画与几何一致性全广度覆盖 / 与上游实现的一致性结论 / 发布依据。

## 6. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe docs/development-evidence/render-bvh/r06-multiframe-obs-cleanup-check/scripts/verify_obs_cleanup_review.py
# 期望末行：SUMMARY PASS 14/14
```
