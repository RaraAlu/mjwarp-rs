# R06-S3b guards 扩条目 r2：位姿 provenance 纳入冻结（inputs 13→14 / total 15→16）

任务：`2026-11-04-render-r06-s3b-flex-guards-pose-entry-r2`（owner Backend）。依赖 D5 修复
`2026-11-04-render-r06-s3b-cam-pose-source-fix`（提交 `ab8da091`）：新增参考
`fixtures/render-bvh/scene-flex/reference/base/cam-poses.json` 并修改 `scripts/authoring/capture_render_flex.py`。
**本片不做真冻结**（`-Attempt` 只在 attempt 4 窗口内由窗口执行者现场跑）；这里只改冻结器 scope + guards 计划态，
并用 `-GuardsOnly` 静态自检证明结构自洽。零 GPU、`src/` 零 diff。

## 1. 冻结器修正（`scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1`，191→192 行，ASCII-only）

| 位置 | 旧 | 新 |
| --- | --- | --- |
| 头注释（行 6–7） | 13 read-only input entries（6 scene-flex + 5 reference + capture script + freeze script） | **14**（6 scene-flex + **6 reference** + capture script + freeze script） |
| `$inputPaths` | 13 条 | **14 条**：新增 `fixtures/render-bvh/scene-flex/reference/base/cam-poses.json` |
| 入口断言 | `Count -ne 13` / `expected 13 guard inputs` | `Count -ne 14` / `expected 14 guard inputs` |
| `-GuardsOnly` 断言 | `guard_inputs.Count -ne 13` | `guard_inputs.Count -ne 14` |
| 总条目断言 | `$total -ne 15` / `expected 15 frozen entries` | `$total -ne 16` / `expected 16 frozen entries` |
| `aggregate_algorithm` 文案 | all 15 entries | **all 16 entries** |

自检：PS 5.1 `Parser` AST **0 错误**；非 ASCII 字节 **0**；行数 **192 ≤ 200**（卡面限定）。

## 2. guards JSON 修正（`scripts/validate/render-bvh/guards-r06-s3b-flex.json`，计划态、exe 仍 pending）

| 项 | 旧 | 新 |
| --- | --- | --- |
| `counts.guard_inputs` / `counts.total` | 13 / 15 | **14 / 16** |
| `guard_inputs` 条目数 | 13 | **14**（新增 1 条 + 更新 2 条 sha/bytes） |
| 新增条目 | — | `reference/base/cam-poses.json`：bytes **3987**，sha256 **`9b6eabf2…`** |
| `scripts/authoring/capture_render_flex.py` | bytes 21866，sha `09c38236…` | bytes **25548**，sha **`7093d705…`** |
| `freeze-guards-r06-s3b-flex.ps1` 自身条目 | bytes 9195，sha `d052deae…` | bytes **9379**，sha **`056102ab…`** |
| `aggregate_algorithm` 文案 | 13 input entries | **14 input entries** |
| `aggregate_inputs_sha256` | `a46a4200…`（before） | **`ec6fa222…`（after）** |

分类：`input=13` + `guard_script=1` = 14；`frozen_exes` 2 条仍 `pending_freeze=true`（真冻结在 attempt 4 窗口内做，
exe 哈希以那一刻为准）。

## 3. 自检实测（`-GuardsOnly`，零 GPU、不跑 cargo）

```
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1 -GuardsOnly
GUARDS_STRUCTURE_OK suite=render-r06-s3b-flex exes_pending=2 inputs=14 inputs_sha256_ok=true aggregate_inputs_sha256=ec6fa22239c492c620a6d7dba9191b1d7f07b041ff4c27976b2de2a8f6998e77 classification=input=13,guard_script=1
```

`inputs_sha256_ok=true` 表示 14 条条目逐条 bytes+sha256 与磁盘一致、aggregate 重算匹配。

**attempt 4 窗口期望**（真冻结时由冻结器打印，不是本片实测）：

```
GUARDS_OK=16/16 suite=render-r06-s3b-flex exes=2 inputs=14 stash=target/render-bvh/r06-s3b-flex-window/frozen-exes ...
```

窗口材料里的守卫数字按 **16/16（exes=2, inputs=14）** 同步；attempt 4 仍需 Chief token。

## 4. 边界

- 只改冻结器 + guards JSON + 本证据；不动 `src/`、不改判据/容差/参考 npz、**不做真冻结**、零 GPU。
- 原卡 `2026-11-04-render-r06-s3b-flex-guards-pose-entry` 已 abandon（其 files 漏冻结器脚本，且 proposed 卡不可原地改 scope）。
