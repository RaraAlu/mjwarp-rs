# perworld L v2 采集脚本 §11.1 前置实现（command / captured_at / per_world_scope / reference_tag；零 GPU）

- 任务：`2026-11-07-render-perworld-l-v2-capture-meta`（owner Backend、p60、P8-渲染与BVH）
- 定位：**工具/元数据面**（非判据，不需要用户签名）；零 GPU、不采集、不写参考、不改判据/容差/已签 spec v2/criterion v2.2、不动既有 perworld 参考与冻结件。
- 上游依据：判据 spec v2（`2026-11-06-render-perworld-l-v2-light-criterion-spec-v2`，QA 最小复核 PASS `91d628c3`）§11.1；S-2 签名前四项必须落实。

## 1. 结论

四项 §11.1 前置已在采集脚本落地，默认行为不变（无参 dry-run 与改动前逐字段一致，unexpected=**0**）：

| # | 前置 | 实现 | 负例 |
| --- | --- | --- | --- |
| ① | `command` 字段 | `capture.json` 顶层新增 `command = {argv（逐字）, cwd, rendered}`；dry-run plan 同口径回显；写后 `verify_capture_metadata` 自核非空 | 空 argv 构造 → 显式失败（模块自检 case `command-negative-empty-argv`） |
| ② | `captured_at` 与 env | 只接受**显式非空环境变量** `MJWARP_CAPTURE_AT`（非空 ISO8601）；缺失/空串/非 ISO8601 → 显式报错退出；**不再写空串**；dry-run 不消费该 env、只回显状态 | 缺 env、空串 env 各一条（脚本级 exit 1 + 不落盘，**0.079 s / 0.078 s** 即失败，未进入 GPU；见负例日志实测）；非 ISO8601 在模块自检 |
| ③ | `per_world_scope` 文案 | 保留 `covers`/`does_not_cover` 字段名，内容改新覆盖口径：**per-world 灯位姿 + per-world geom 位姿**，并写明 §10.3 的 host-only 灯语义前提；`does_not_cover` 列 AA×perworld、单发射性能方案 K、连续动画/多 world 规模、与上游实现等价性等不变项 | 旧口径字符串 `per-world light poses (device path indexes lights single-world)` 出现即拒（模块自检 `capture-metadata-negative-old-scope`、`scope-negative-old-wording`） |
| ④ | `recipe.task_id` 旧卡 id | **选 (b) 新增 `reference_tag`**（保留 `task_id` 原样）：`reference_tag` 默认 = 参考目录名（spec v2 §2.2 tag 约定），CLI `--reference-tag` 可覆盖；`task_id` + `task_id_source=script-constant` 同时写入采集元数据 | 空目录名无法派生 → 显式失败；`--reference-tag` 为空/未传时默认派生（模块自检 `reference-tag-negative-empty-dir` 等） |

## 2. 决策 (b) 的理由（为何不改 task_id）

- `task_id` 的语义是「**脚本所属 authoring 任务**」这一 provenance 来源；把它参数化会让同一脚本产出不同 `task_id`，重新打开 §11.1 第 4 条要消除的 provenance 混淆。
- 新增 `reference_tag` 与 spec v2 §2.2 的 **tag 概念直接对齐**（tag = 参考目录名，如 `nw3-light-hinge02040`），默认由 `--out-dir` 目录名派生、无需新增必填参数，默认行为不变；CLI 可覆盖。
- 采集产物同时可追溯：`task_id` + `task_id_source`（脚本来源）与 `reference_tag`（实际参考身份）。
- 前后值（写入证据 `artifacts/capture-meta.json`）：before `{recipe.task_id: 2026-10-30-…, reference_tag: <absent>, captured_at: "", command: <absent>}` → after `{recipe.task_id: 不变, reference_tag: <参考目录名>, captured_at: <显式 ISO8601>, command: <argv+cwd+rendered>}`。

## 3. 默认行为不变（无参 dry-run 前后对照）

- `artifacts/default-dryrun-diff.json`：`unexpected = 0`；允许差异仅 5 项——新增 `/command`、`/reference_tag`、`/captured_at_env`（additive echo）与 `/recipe/script/sha256|bytes`（脚本被编辑）；`legacy_manifest_entries` 保持 **107 不变**。
- `logs/before-default-dryrun.json`（HEAD `91d628c3` 脚本）与 `logs/after-default-dryrun.json` 均留档；before-meta 记旧脚本 sha。
- **已知项（Arch 旁证 O4）**：`command.argv` 逐字记录调用 argv，不同 shell / 路径写法（正斜杠 vs 反斜杠等）会产生不同 `command` 文本；该字段用于 provenance 留痕，若日后要做机器比对需先规范化路径写法。

## 4. 行数 / 拆分（500 行棘轮）

| 文件 | 行数（git blob 换行口径） | 说明 |
| --- | --- | --- |
| `scripts/capture_render_perworld.py` | **497 → 499** | 只留 import + 调用；≤500 |
| `scripts/perworld_capture_meta.py` | **230（新）** | 职责：command 记录、captured_at env 严格校验、per_world_scope 文案、task_id/reference_tag 解析、写后自核、CPU 自检；≤500 |

## 5. 负例与自检（全部 CPU、零 GPU）

| 项 | 证据 | 结果 |
| --- | --- | --- |
| captured_at 缺 env | `logs/negative-captured-at-missing.log` | exit 1；out_dir 未创建；文案「is missing」 |
| captured_at 空串 env | `logs/negative-captured-at-empty.log` | exit 1；out_dir 未创建；文案「is empty」 |
| 四项模块自检（含 command 空 argv、旧 scope 文案、reference_tag 空目录名、capture 元数据写后自核等 14 case） | `logs/module-selfcheck.log` | **SELFCHECK PASS 14/14** |
| 合成 capture.json 形状（**SYNTHETIC**，非参考） | `artifacts/synthetic-capture-metadata.json` | 四项字段齐全、写后自核通过 |

## 6. 边界与非声明

- 零 GPU、不开窗、不采集、不写 `fixtures/` 参考；`unchanged src/`、`unchanged fixtures/`；四门禁全过。
- **非声明**：本卡只改采集工具/元数据面；不产参考、不调判据、不宣称采集已有效、不宣称 per-world 灯验收通过、不宣称与上游实现等价、不宣称任何渲染数值结论；采集窗（G-L1）与验证窗（G-L2）仍按 spec v2 的签名/GO 流程执行。

## 7. 证据清单

| 路径 | 说明 |
| --- | --- |
| `README.md` | 本文件 |
| `receipt.json` | 机器可读回执（`/task_id` 供 normify `json` 检查） |
| `artifacts/default-dryrun-diff.json` | 默认 dry-run 前后逐字段 diff（unexpected=0） |
| `artifacts/capture-meta.json` | 四项实现/决策/行数/负例汇总（含 (b) 理由与前后值） |
| `artifacts/synthetic-capture-metadata.json` | **合成** 采集元数据形状（非参考、无 GPU） |
| `artifacts/script.diff` | 主脚本改动 diff |
| `logs/before-default-dryrun.json` / `logs/after-default-dryrun.json` | 改动前后默认 dry-run 原始输出 |
| `logs/negative-captured-at-{missing,empty}.log` | 两条脚本级负例（exit code + 不落盘） |
| `logs/module-selfcheck.log` | 模块 14 case CPU 自检原始输出 |
| `logs/line-counts.txt` | 两个 .py 行数 |
| `manifest.txt` | 本目录 sha256 + bytes（首行 `#`，不自列） |

## 8. 复现

```powershell
# 默认 dry-run（与改动前对照）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/capture_render_perworld.py --dry-run `
    --out-dir fixtures/render-bvh/scene-a/perworld/prep-default-dryrun
# 四项 CPU 自检（14 case）
target/mjwarp-parallel/venv/Scripts/python.exe -B scripts/perworld_capture_meta.py
# 负例：缺 MJWARP_CAPTURE_AT 时采集路径必须早失败（不建 out_dir、不进 GPU）
```
