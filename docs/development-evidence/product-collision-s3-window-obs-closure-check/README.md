# S3 窗口观察收口独立复核（QA，补正后最终版）

- 任务：`2026-10-27-product-collision-s3-window-obs-closure-check`（复核对象 `2026-10-27-product-collision-inputs-s3-window-observation-closure`）
- 复核链：`7d79596a`（登记）→ `6680dabe`（start）→ **`c65d84ac`（收口，8 文件）** → `71e78aae`（F2 补正，2 文件）→ **`ee5d0ca4`（F1 补正，7 文件）**
- 复核时 HEAD = `ee5d0ca4`；结论：**pass / 阻断 0 / 观察 2（2 建议，均已修）**
- 只读复核：未重开门、未开 GPU、未重跑实体、未改任何代码/判据/参考/容差/窗口数值

## 1. 总体判定（最终版）

收口片把上一轮 QA 的 7 条观察（O1–O7）全部处置到位；两条建议级观察 **F1 / F2 已由两个
`normify fix` 证据补正修掉**（`ee5d0ca4` / `71e78aae`），本轮已按补正后状态重新独立复算：**无阻断、
无未修观察、窗口数值不变量一项未变**。两个补正提交只碰证据与本任务记录，**0 代码路径**。

## 2. O1：stdout 证据副本编码修复（自验）

| 项 | 原始（`target/`，未入库只读） | 入库副本 |
| --- | --- | --- |
| 编码 | UTF-16LE 带 BOM（FF FE） | **UTF-8 无 BOM** |
| 行尾 | CRLF | **LF（CR=0）** |
| bytes | **81128** | **40380** |
| sha256 | **b7cffd45…** | **fb3e63e4…** |
| 行数 | **183**（解码后） | **183**；插入空行 0 |

- 我解码原始 UTF-16LE（去 BOM）后与入库副本**逐行比对 0 差异**；插入空行数 0（历史损坏副本 366 行已消失）。
- 原始文件 mtime 2026-10-27 17:04:04（窗口时点），sha/bytes 与收口 receipt 登记一致 → 未被后续改动；
  两个补正提交也未改该副本（sha 仍 `fb3e63e4…`）。
- 登记落地：window-receipt `evidence_format_notes.stdout_log_encoding`、receipt `/s3/evidence_format_notes`、
  窗口交付记录 `evidence_format.stdout_log_encoding`、README §10.4 四处都有原始/副本编码+sha+bytes。

## 3. O2 / O3：措辞与邻居采样

- **O2 四处一致**：window-receipt、receipt、README §10.2、窗口交付记录均含 `17:03:36.808`、
  `86 ms`、`±0.1 s`、stage2 下界 `17:03:37.658–38.443`、`≥0.9 s`，并保留「归因推断（未证实到具体分配点）」；
  峰值 4142 / seq 776 / `2026-10-27T17:03:36.7225390+08:00` 未变。
- **O3 邻居**：window-receipt（observation_OW1 与 budget peak）、receipt attempt1 三处均为
  `[3092,4142,3092]`，无 `[3092,3091]`/`[3092,4142,3091]` 残留。

## 4. F1（原观察，已修）：采样间隔区间改准

- 我从 `samples.csv`（窗口原件）独立重算：中位 **0.2759 s**（≈0.276 ✓）、min **0.2721 s**、max **0.2901 s**。
- 补正 `ee5d0ca4` 后，四处显示区间统一为 **`0.2721–0.2901`**（README §10.2、window-receipt、receipt、
  窗口交付记录），收口交付记录 milestone 与四处 `measured_range_s` 也统一为 4 位实测值
  `[0.2721, 0.2901]`（补正前为 3 位 `[0.272, 0.29]` + 4 位截断显示）。
- 复核：`0.2721` / `0.2901` 各 **15** 处命中；`git grep '0.2720|0.2900' -- docs/development-evidence`
  **0 命中**。观察状态：**已修**。

## 5. F2（原观察，已修）：例外清单 receipt.json 行数

- 补正 `71e78aae` 把窗口交付记录 `evidence_format.machine_readable_exceptions` 的
  `receipt.json (745 lines)` 改为 **`receipt.json (750 lines)`**；实际 `receipt.json` = **750 行（LF）**，
  与 window-receipt/README 一致。
- 复核：`\b745\b` 在 window-receipt/receipt/README/交付记录（`machine_readable_exceptions` 段）**0 命中**；
  `750` 到位。**假阳性保留说明**：README L376 `162 行 / 7453 B` 是 e2e target 的**字节数**，不是行数例外，
  已确认不是 F2 残留。观察状态：**已修**。

## 6. O5 / O6 / O7

- **O5 机器可读例外**：window-receipt / receipt `evidence_format_notes` 含 `receipt.json`(**750 行**)
  与 stdout 原始/副本编码声明；README §10.4 写 750；收口 receipt 例外列表齐。
- **O6**：`runner_outer_wall_s={"value_s":252.1,"derived":true,"basis":"无独立产物…推导值"}` 落地
  （window-receipt、窗口交付记录、README 均标 derived）；与我的判断一致。
- **O7**：统计口径写明「四份记录全文件字面计数 = 7（2+2+2+1；不是单文件 ≥3；不含本收口 receipt；
  文本用「预算·偏差」避免自增）」；我独立计数 `预算偏差` **7**、`在预算内` **0**。

## 7. 数值不变量（补正后重算，一项未变）

10 实体 exit 全 0；body_wall **238.480 s**；新 target 3 case `nacon=4=ref`、`bit_equal=12/12`、
dist/pos/frame ULP=**0/0/0**、geom `[1,0]/[1,0]/[0,1]`；S1 快照 **32/32×3**；9 回归日志全 `test result: ok.`；
stage2 **17** dump 全一致（record-only）；采样 **867** / baseline **3091** / peak **4142** /
delta **1051/1051/1055** / free_min **11920** / floor **9904** / util **31%** / streak **0** / residual 0 /
峰值排除后 3109、sustained 18；`sha-guards.txt` **133/133** 我逐条重新哈希全 match；窗后 10 exe **10/10**；
`attempt=1`、无 attempt2；零编译 deps **1812→1812**；runner `c7a387a4…`、冻结 10 exe、
`provenance.json` `3ea5c863…` 均未动。补正提交未改任何这些数值（diff 只含区间文本、行数与记录身份）。

## 8. 范围与边界

- `c65d84ac` 8 文件（收口证据 + 两份交付记录 + `.normify` 记录），**0 代码路径**；
  `71e78aae` 2 文件、`ee5d0ca4` 7 文件，**均为证据/任务记录**（trailer `Normify-Fixes:`）；
  `git diff 0c9497e4..HEAD` 与 `c65d84ac..HEAD` 的 `src/tests/native/include/fixtures/Cargo*/build.rs` **均 0 文件**。
- 收口任务记录 status=verified；`c65d84ac` trailer = `Normify-Task: …-s3-window-observation-closure`。
- 两份交付记录内所有 `{path,sha256,bytes,lines}` 条目我逐条复验 **0 不符**（补正后记录已同步新 sha/bytes）。
- 证据文本 UTF-8 无 BOM/LF；无「窗口通过」正面表述、6 条禁词 0 命中。

### 8.1 记录内历史文本（已记录，非证据声明）

- 收口任务卡 `.normify/changes/…-observation-closure.json` 的 `intent` / `acceptance` / `note`
  仍保留登记时的「0.2720–0.2900」原文（3 行），window 任务卡 addendum 的 reason 写「745→750」——
  都是**任务卡历史文本与补正说明**（Normify 记录为追加式，闭合记录不重写原文），不是证据声明，
  证据侧命中 0。本复核任务卡自身也保留旧值「receipt.json(719 行)」，同理不按卡面文字核，按实际 750 记。

## 9. 门禁与证据

- 本任务 checks 仅 `json`(receipt `/task_id`) + `forbid_claim`（6 条），**无 gate 项**：
  `normify gate` 返回「任务没有引用任何门禁」**exit 1 属预期**，receipt 已写明 gate 不适用。
  `normify check` **exit 0**。
- `verify_closure.py`（**80 项检查，0 problem，0 finding**）→ `verify_closure.json`；
  `consistency.py`（行数 / 收口 receipt `line_refs` / 交付记录一致性）→ `logs/line-counts-and-refs.txt`；
  `logs/o1-conversion.txt`、`logs/invariants.txt`、`logs/findings.txt`、`logs/scope-boundary.txt`、
  `logs/forbidden-terms.txt`；`manifest.json` 11 项 sha/bytes/lines。
- 复现：`python verify_closure.py`、`python consistency.py`（只读；读 `target/s3-product-wiring/` 原件与本仓库证据）。

## 10. 边界与不宣称

只读复核：未重开窗口、未开 GPU、未重跑任何实体、未改代码/判据/参考/容差。不宣称窗口结论、覆盖、精度、
等价或验收；本片只核「收口与两个证据补正是否把上一轮观察修好、且未改窗口数值」。
