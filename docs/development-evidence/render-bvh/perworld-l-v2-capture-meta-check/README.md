# perworld L v2 采集脚本 §11.1 前置 QA 只读复核（含 9275e6e6 证据层修正后最终状态）

**结论：PASS（0 阻断，0 观察；3 条信息项，无需代码/证据返工）**

- 任务：`2026-11-07-render-perworld-l-v2-capture-meta-check`（owner QA、P8-渲染与BVH、p60；QA 自登记，依赖采集元数据卡）
- 复核目标：HEAD **`9275e6e6`**（实现卡 `verified`；登记 `262c034a` → start `d8597014` → 实现 `b0af1be4` → 证据层修正 `9275e6e6`），代码基线 `91d628c3`
- 复核方式：只读 + 独立复算（同树旧/新脚本控制变量对比、证据日志逐叶复跑比对、负例实测、模块自检重跑、行数/哈希/manifest 独立重算）；QA 脚本自检 **20/20 PASS**；CPU-only、零 GPU、不采集
- 本卡登记提交：`bf0b44a9`

## 1. 结论

四项 §11.1 前置与全部验收点成立：`command` / `captured_at` / `per_world_scope` / `reference_tag` 四条均按要求落地，默认行为不变（同树逐叶 diff 仅 6 个允许新增叶 + script sha/bytes，removed=0、legacy 107 不变），负例在 GPU 之前早失败且不建 out_dir，行数 499/230、四门禁 unit 1359/0、边界零改动、证据 16/16 自核。`9275e6e6` 的三条证据层修正（O1 计时统一为实测 0.079/0.078 s、O2「环境变量」笔误、O4 `command.argv` 路径写法已知项）已逐项核实。**无阻断、无观察。**

信息项（非观察、无需返工；供 S-2/窗口材料参考）：
- **N1 干跑不校验 env 格式**：dry-run 对非法 `MJWARP_CAPTURE_AT`（如 `not-a-time`）仍 exit 0、只回显 `{present, non_empty}`（按设计「不消费该 env」）；格式严格校验在采集路径 GPU 之前。建议 S-2 材料把「开窗前用 ISO8601 校验或采集路径前置」写成步骤，避免窗口内才暴露格式问题。
- **N2 `command.argv` 为重建值**：记录形式是 `[python 解释器, *sys.argv]`，不含解释器启动 flags（如 `-B`），且尾部纯空白项会被裁掉；与已记 O4（正/反斜杠路径写法差异）同属 provenance 文本口径，机器比对前需规范化。
- **N3 合成形状文件首行为非 JSON 标记**：`artifacts/synthetic-capture-metadata.json` 首行是 `SYNTHETIC METADATA SHAPE ONLY ...` 标记行，严格 JSON 解析需跳过首行；文件已明确标注 SYNTHETIC、非参考，按证据用途可接受。

## 2. 逐项结果

### 2.1 ① command（PASS）

- `capture_metadata()` 返回 `command = {argv（非空列表）, cwd（绝对路径）, rendered（`subprocess.list2cmdline`）}`，并随 `**provenance` 写入 `capture.json` 顶层；dry-run plan 由 `dry_run_echo()` 同口径回显；写后 `verify_capture_metadata()` 自核非空（空 argv/空 rendered → fail）。
- 接口级复现：直调模块得到完整 6 键元数据块并通过 `verify_capture_metadata`；空 argv、空 captured_at、旧 scope 文案三种负例均被显式拒绝（接口级 + 模块自检双重）。

### 2.2 ② captured_at（PASS）

- 只接受显式非空 ISO8601：缺失 / 空串 / 非 ISO8601 三种输入在**采集路径**均 **exit 1**，消息分别为 `is missing` / `is empty` / `not ISO8601`；**out_dir 未创建**；实测耗时 0.081 / 0.079 s 量级（证据日志 0.079 / 0.078 s），输出无 CUDA/warp device 字样 → 在 GPU 之前失败。
- 静态顺序核对：`provenance = ... capture_metadata(...)` 紧随其后即 `import warp as wp`，早于任何 device 创建与渲染。
- dry-run 不消费该 env：未设/合法/非法三种情况下 dry-run 均 exit 0，只回显 `{present, non_empty}`；plan 不含 `captured_at` 字段。

### 2.3 ③ per_world_scope（PASS）

- 保留 `covers`/`does_not_cover` 字段名；`covers` 含 **per-world 灯位姿 + per-world geom 位姿**，并写明 spec v2 §10.3 的 host-only 灯语义前提（`1-D` 灯数组、`worldid == 0`、`.cu unchanged`）；`does_not_cover` 列 AA×perworld、单发射方案 K、连续动画/大 world 数、上游等价。
- 旧口径字符串 `per-world light poses (device path indexes lights single-world)` 出现即被 `verify_capture_metadata` 拒绝（接口级复现 + 模块自检 `capture-metadata-negative-old-scope`、`scope-negative-old-wording`）。

### 2.4 ④ task_id / reference_tag（PASS）

- 选 **(b)**：新增 `reference_tag`（默认 = `--out-dir` 目录名，即 spec v2 §2.2 参考 tag；`--reference-tag` 可覆盖且覆盖生效）；`task_id` 与 `task_id_source = script-constant` 原样保留。
- 决策理由与前后值在 README §2 与 `artifacts/capture-meta.json`（`decision.chosen/why_not_parametrize_task_id/before/after` 齐全）。

### 2.5 ⑤ 默认行为（PASS）

- **同树控制变量**（旧脚本 sha `3e1d207c…`/497 行 临时候选真实路径运行后 `finally` 恢复、恢复 sha 校验=`46635cc3…`）：逐叶 diff = 新增 `/command/{argv,cwd,rendered}`、`/reference_tag`、`/captured_at_env/{present,non_empty}` 6 个叶 + 变更 `/recipe/script/sha256`、`/recipe/script/bytes`；**removed=0**、`legacy_manifest_entries` 两侧 **107**、`repeat` 两侧 **2**，其余字段全同。
- 被复核 before/after 日志按证据记录的 backslash 命令行复跑：**两侧逐叶完全一致**（0 added/removed/changed）。

### 2.6 ⑥ 行数 / 门禁 / 边界 / 证据（PASS）

- 行数（git blob 换行口径）：`scripts/capture_render_perworld.py` **499**（基线 497）、`scripts/perworld_capture_meta.py` **230（新）**，均 ≤500；sha/bytes 与 `artifacts/capture-meta.json` 逐项一致。
- 模块自检：独立重跑 `python -B scripts/perworld_capture_meta.py` → exit 0、**`SELFCHECK PASS 14/14`**（与证据日志一致）。
- 边界：`git diff 91d628c3..9275e6e6` 恰 **24 路径**（卡 + 4 gate runs + 17 证据 + 2 脚本）；`fixtures/`、`src/`、`tests/`、criterion v2.2、spec 目录**零 diff**；未采集、无参考写入、无 `nw3-light-hinge02040` 参考目录产生。
- 证据：manifest 首行 `#`、**16/16** 条目复算通过、不自列（17 文件）；17 文件 UTF-8/LF/无 BOM/裸 CR·TAB 0；两条负例日志带 `exit_code` 与 `out_dir_created: False`；`artifacts/default-dryrun-diff.json` unexpected=0 且 5 条差异分类与我复算一致；合成形状文件标注 SYNTHETIC 且 6 键齐全。
- 被复核卡 `status=verified`、`revision.before=262c034a…`、checks 含四门禁，`unit` gate **1359 passed / 0 failed / 400 ignored**；`9275e6e6` 修正面恰 8 文件（README/receipt/manifest + 卡 addenda + 4 gate runs），addenda 记录 O1/O2/O4 理由与 hash。

## 3. 观察清单

无（0 阻断、0 观察）。信息项见 §1 N1–N3。

## 4. 证据清单

| 路径 | 说明 |
| --- | --- |
| `scripts/verify_capture_meta_review.py` | 复核脚本（20 项只读复算，可复跑） |
| `logs/verify-capture-meta-review.json` | 20 项检查机器可读结果（PASS 20/20） |
| `logs/verify-capture-meta-review.log` | 脚本原始输出 |
| `artifacts/scope.json` | 变更面 24 路径 + 零越界探针 |
| `artifacts/line-counts.json` | 499/230 与基线 497、证据一致性 |
| `artifacts/module-selfcheck.json` | 模块自检重跑 14/14 |
| `artifacts/dryrun-echo.json` | dry-run 回显（command/tag/env 状态/覆盖） |
| `artifacts/negative-cases.json` | 三种非法 env 的 exit/耗时/out_dir/消息/无 GPU 字样 |
| `artifacts/capture-metadata-interface.json` | 直调模块的 6 键元数据块 + 3 条 verify 负例 |
| `artifacts/default-behavior-diff.json` | 同树旧/新脚本逐叶 diff（removed=0、legacy 107） |
| `artifacts/evidence-logs-reproduced.json` | 被复核 before/after 日志逐叶复跑比对 |
| `artifacts/evidence-selfcheck.json` | manifest 16/16（17 文件）、编码、diff/负例 artifact |
| `artifacts/fix-commit.json` | `9275e6e6` 修正面（O1 计时/O2 笔误/O4 已知项） |
| `artifacts/spec-alignment.json` | 与 spec v2 §11.1 四条静态对齐 |
| `artifacts/reviewed-card.json` | 被复核卡状态 + unit 1359/0 + addenda |
| `README.md` / `receipt.json` | 本报告与机器可读回执 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 5. 边界与非声明

- 只读复核 + 只写本卡证据目录；不采集、不改被复核脚本/证据、既有证据、代码、fixtures；零 GPU。
- 复核期间为控制变量曾临时把旧脚本内容写回真实路径并在同进程 `finally` 恢复（sha 校验一致），最终工作树 `scripts/` 与 `fixtures/` 零改动。
- 本复核不产生参考、不宣称采集已有效、不宣称 per-world 灯验收通过、不宣称与上游实现等价、不宣称任何渲染数值结论；G-L1/G-L2 仍按 spec v2 签名/GO 流程。

## 6. 复现

```powershell
cd D:\Rust\mjwarp-rs
target/mjwarp-parallel/venv/Scripts/python.exe -B docs/development-evidence/render-bvh/perworld-l-v2-capture-meta-check/scripts/verify_capture_meta_review.py
# 期望末行：SUMMARY PASS 20/20
```
