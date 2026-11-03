# R06-S3b② 窗口材料 QA 复核（对象 `87b8fbb9`）

- 任务：`2026-11-04-render-r06-s3b-flex-window-material-check`（QA；只读、零 GPU、不进窗口、未冻结）
- 被复核对象：`2026-11-04-render-r06-s3b-flex-window-material` 提交 `87b8fbb9`（10 文件；材料目录 `docs/development-evidence/render-bvh/r06-s3b-flex-validate/`）
- 参考数据：`fixtures/render-bvh/scene-flex/reference/base/`（采集卡 `71ec7c83`）
- 结论：**PASS-with-observations**（0 阻断 / 0 严重；2 条建议级、3 条观察级；材料事实与判据全部成立，缺口如实，可进入「补前置 → 开窗」流程）

## 1. 复核点逐条

| # | 复核点 | 结果 | 依据（复算见 `qa-recheck.json`） |
| --- | --- | --- | --- |
| 1 | 事实一致性：参考 sha256 | 成立 | 独立重算 5 个文件：`rgb 5ed50287…`、`depth c3ee4adc…`、`seg f3b00fa8…`、`state 7f2fff50…`、`capture.json 4713e2c7…`；与 `capture.json.files_sha256`、`entity-table.reference_source.files_sha256`、`provenance.files_sha256`/`capture_json_sha256` **四方一致**。 |
| 1 | 事实一致性：flex 关键量 | 成立 | 逐项对 `capture.json.flex`：`nflex=2`、`flex_dim=[1,1]`、`flex_radius=[0.02,0.04]`、`vertadr=[0,4]`、`edgeadr=[0,3]`、`edgenum=[3,1]`、`elemadr=[0,3]`、`shellnum=[2,2]`、`rgba=[[0.9,0.5,0.1,1],[0.2,0.4,0.9,1]]`、`bvh_nflexgeom=4`、`flex_geom_flexid=[0,0,0,1]`、`flex_geom_edgeid=[0,1,2,3]` —— 13/13 全对。 |
| 1 | **`cam-flex` 654 个 `(flexid,9)` 像素独立复算** | 成立 | 直接从 `seg.npz` 复算（不读产品代码、不读材料断言）：`seg_cam2` **654** 像素，其中 `(0,9)=403`、`(1,9)=251`，flexid 0/1 均出现；`seg_cam0=0`、`seg_cam1=0`。`provenance.reference.flex_coverage["cam2_cam-flex"]=654` 与之吻合（相机映射 cam2↔cam-flex 由 provenance 标明）。 |
| 2 | 判据零放宽 | 成立 | 与家族先例逐词对照：`perworld-s1-window-material/materials/entity-table.json` 的 `rgb: mismatched=0 / max_byte_diff=0 / beyond_1_lsb=0 (bitwise)`、`depth: mismatched==0 AND max_ulp<=4 AND beyond_ulp==0`；`r02-skybox-s3-validate/README.md` 同款「位级优先 + max_ulp<=4/beyond_ulp==0」。S3b 材料用词与二者一致，未出现「≈/close enough/允许 N 像素例外」等近似措辞；seg 为精确相等；two-run sha 要求存在，且参考侧 `two_captures_bitwise_identical=true`。 |
| 3 | 可执行性 | 基本成立（有 1 条前置缺口，见观察 1） | `monitor-until-clean.ps1` 存在（参数 `-Required/-MaxMinutes/-MaxUtil`，默认 `MaxMinutes=12`、`MaxUtil=40`；**无** `-OpenUtilLimit`，与 plan「不带」一致）；`run-window.ps1` 存在（`-Attempt`、`-GuardsOnly`、`-OpenFreeMiB`(默认 10240)、`-BudgetMiB`(3072)）；`window-execution-policy.md`、`scene.xml`、`normalize-evidence-eol.ps1` 均在。**`scripts/validate/check-evidence-bom.ps1` 当前不存在**（见观察 1）。 |
| 3 | 缺口如实 | 成立 | P2（设备用例 + 生产调用点）、P3（冻结器 + guards）明确 `missing`，P4（本复核）`pending`；`freeze-guards-r06-s3b-flex.ps1` 实查确实不存在，与 P3 一致；README §5 明写「A/B 未收口前不得开窗」，未把缺口写成已完成。 |
| 4 | 纪律面 | 成立 | plan §0/§2/§7：唯一执行会话 + Chief 直接 GO、群内帖仅 record-only；失败即停、不重试、不重冻「跑到过」、不放宽阈值、attempt 原文留档；冻结 `-Attempt 1`；EOL（先 `-DryRun` 再实跑）与 BOM 检查已写进 §1/§7。 |
| 5 | 越界 | 成立 | 对象提交 10 个文件全部在 `docs/development-evidence/render-bvh/r06-s3b-flex-validate/`、`.normify/runs/`、`.normify/changes/`；**`src/`、`fixtures/` 改动 0**；README §8 与 `receipt.json.non_claims` 明确不宣称 flex 渲染可用/不宣称 R06 通过，`nbuffer` 版本差异明确未放宽、不进本窗口判据。 |

## 2. 观察与建议（不阻塞）——下列 5 条均已由 `ff681521` 收口，见 §5

1. **【建议】BOM 审计脚本尚未落库，但 plan 已把它当既有命令**：`materials/window-plan.txt` §1 与 `receipt.pre_window_hygiene` 都写 `scripts/validate/check-evidence-bom.ps1`（基线 128 non-`.ps1` WARN + 4 manifest 违规），但该脚本当前**不存在**（治理卡 `2026-11-04-governance-evidence-bom-audit` 仍 `proposed`）。建议在 prerequisites 补 **P7「BOM 审计脚本已落库并通过」**，否则 §1 第 3 条在开窗时无命令可跑；本复核卡落库的同时我会把该脚本与基线证据交付。
2. **【建议】floor 与执行脚本阈值未显式衔接**：`run-window.ps1` 的 `-OpenFreeMiB` 默认 **10240**，`monitor-until-clean.ps1` 的干净判据也硬编码 `free >= 10240`；按材料 `gpu-baseline.txt`（free 14772）套公式 `floor = max(14772-3072, signed) = 11700`，若照 plan 现命令原样跑，实际生效的是**更松**的 10240。建议 plan 明确写 `-OpenFreeMiB <当次 floor>`（冻结/守卫两处都传），并在 §3 补「`free < floor` 由执行者逐点人工核对（monitor 脚本不含 floor 参数）」。
3. **【观察】先例数字口径**：`gpu-baseline.txt` 写「R02 precedent: 12858 - 3072 = 9786」，而 R02 实际记录是 `r02-skybox-s3-validate/attempt2/facts.txt` 的 `floor_mib=9795`（基线 `free_min=12858`）。公式值与签署值不是同一个数，建议改写为「R02 基线 12858、当次 floor 9795」，避免开窗时误用 9786。
4. **【观察】字段命名**：`entity-table.entities[].flex_coverage_reference.object_id_9_pixels` 实际语义是「seg 第二通道 == 9 的像素数（第一通道 = flexid）」，不是 `object_id == 9`。建议改名（如 `flex_seg_flexid_9_pixels`）或在旁注明，避免执行者按 object_id 过滤。
5. **【观察】时长预期**：plan 写 monitor「5–10 min（硬顶 30）」，而脚本默认 `-MaxMinutes=12`、plan 显式传 30 —— 已显式覆盖，仅提示「硬顶」是脚本参数而非脚本内置上限。

## 3. 复核方式与边界

- 全程只读：读 `capture.json`/`seg.npz`/`state.npz`/`provenance.json`/材料文件 + `git show` 检查对象范围；**未跑 GPU、未开窗、未冻结、未改任何文件**。
- 未重跑四门禁（材料卡 `87b8fbb9` 自带的 runs 记录可查）；本复核卡自身按卡面跑 fmt/clippy/clippy-cuda/unit。
- 不宣称 flex 渲染可用、不宣称 R06/NP01/NP02 通过；不改变判据/容差/参考。

## 5. 观察收口记录（材料 fix `ff681521`，`Normify-Fixes: …-window-material-r2`）

材料卡按本复核 §2 的 5 条观察做了 fix，QA 逐条核对 worktree/HEAD 均为已修复状态：

| # | 观察 | 收口证据（`ff681521` 后） |
| --- | --- | --- |
| 1 | BOM 审计脚本未落库却被当既有命令 | `materials/entity-table.json` 新增 **P7「BOM 审计脚本 check-evidence-bom.ps1 已落库并通过（2026-11-04-governance-evidence-bom-audit）」**（status missing，注明脚本未落库前不得引用）；README §4 前提表第 7 行「❌ 缺（卡 proposed；未落库前本材料不得引用该命令）」；plan §1 改为「P7 前提」。 |
| 2 | floor 与 `-OpenFreeMiB`/monitor 硬编码 10240 未衔接 | plan §2 新增 NOTE：`run-window.ps1 -OpenFreeMiB` 默认 10240、monitor 干净判据硬编码 free≥10240 **且无 floor 参数**；据此要求 ① 运行器显式传 `-OpenFreeMiB <当次三点重算的 floor>`、② monitor 期间**人工逐点核对 free ≥ floor，低于即停**；14772→11700 标为示例、须按开窗当次重算。 |
| 3 | `gpu-baseline.txt` 把 R02 公式值当签署值 | 改为「R02 precedent: baseline 12858 - 3072 = 9786 **by formula**, signed floor for that window was **9795** — the signed value, not the formula value, governs」。 |
| 4 | `object_id_9_pixels` 字段语义误导 | 改名 **`seg_channel2_eq_9_pixels`** 并注明「seg 第二通道==9、第一通道=flexid；654 = (0,9) 403 + (1,9) 251」（三处实体均改）。 |
| 5 | monitor `-MaxMinutes` 默认值未说明 | plan 注明「`-MaxMinutes` defaults to 12; this plan passes 30 explicitly（"hard cap 30" is that argument, not a built-in limit）」。 |

顺带材料卡同笔更新：**P2 标 ✅ `0a3058ee`**（S3b③ harness 已收口）、P4 标「read-only PASS-with-observations（证据落库中）」；四门禁重跑 unit 1314/0。

因此本复核结论维持 **PASS-with-observations**，且**观察项全部闭合**；P7 的真实闭合取决于本复核队列里的 BOM 审计卡（`2026-11-04-governance-evidence-bom-audit`）落库——该卡交付后材料 P7 即可转 ✅。

### 5.1 后续闭合提交（2026-11-04，QA 已核对）

| 提交 | 内容 | QA 核对结果 |
| --- | --- | --- |
| `ff681521` | 材料卡按本复核 §2 的 5 条观察收口（P7 前置、floor/`-OpenFreeMiB` 衔接、R02 floor 9795、`seg_channel2_eq_9_pixels` 改名、`-MaxMinutes` 注明） | 已逐条核对，见 §5 表 |
| `df7dad06` | **P7 闭合**：标 ✅ `f8a0fa75`（脚本 `scripts/validate/check-evidence-bom.ps1`，基线 128 non-`.ps1` WARN + 4 manifest 违规），并记录 BOM 漂移史（S3b③ 新文件 `r06-s3b-flex-harness/compile.log` 曾使计数到 129，已由 `d034eb43` 剥离回 128）；plan §0 剩余前置只剩 **P3 冻结器**（新卡 `2026-11-04-render-r06-s3b-flex-freeze-guards`，`f1ab11a3`，owner Backend） | 已核对：材料 README §4 第 7 行为 ✅ `f8a0fa75`；plan §0「P7 … -> DONE (f8a0fa75)」「Remaining before window: P3 …」；`r06-s3b-flex-harness/compile.log` 首 3 字节 `63 61 72`（无 BOM，剥离已生效） |

因此本复核提出的「可执行性」缺口（BOM 脚本 + floor 衔接）已全部闭合；窗口仍受 P3（冻结器）约束，属 S3b④/⑤ 开窗流程，不在本复核面。

## 4. 文件

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本文件：三态结论、复核点逐条、观察清单 |
| `qa-recheck.json` | 独立复算原始数据（5 个 sha 四方一致、13 项 flex 事实、654 像素逐个 flexid 计数、判据先例原文、脚本存在性/参数、对象范围、gpu-baseline 原文） |
| `manifest.txt` | 本目录文件 sha256 + bytes（不含自身） |
