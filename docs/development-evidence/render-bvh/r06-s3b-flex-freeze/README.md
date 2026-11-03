# R06-S3b⑤：flex 验证窗冻结器与守卫（只建脚本 + 静态自检，不真冻结）

任务：`2026-11-04-render-r06-s3b-flex-freeze-guards`（p75，owner Backend；depends S3b① capture r2）。
基线 HEAD：`9aa2a6ce`。本片**不跑 `-Attempt`、不做真冻结、不开窗、零 GPU、零 cargo 执行**（`-GuardsOnly` 只读结构）。

## 1. 交付面

| 文件 | 大小/行数 | 内容 |
| --- | --- | --- |
| `scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1` | 188 行（≤200）、9195 B | 冻结器：`-Attempt N` 真冻结（窗口卡用）/ `-GuardsOnly` 静态结构自检；ASCII-only、UTF-8 LF、无 BOM |
| `scripts/validate/render-bvh/guards-r06-s3b-flex.json` | 3794 B | 守卫清单元数据：suite `render-r06-s3b-flex`、15 条目（2 exe + 13 输入）、分类与 sha/bytes |
| `docs/development-evidence/render-bvh/r06-s3b-flex-freeze/` | 本目录 | README + `self-check.txt`（静态自检三项）+ `guards-summary.txt` + manifest |

## 2. 条目分类（15 条）

| 类别 | 数量 | 条目 |
| --- | --- | --- |
| `frozen_exes`（真冻结时解析+stash） | 2 | `mjwarp-rs`（产品 bin）、`render_flex_validate`（GPU 用例测试二进制，`--features cuda-probe`） |
| `guard_inputs.kind=input` | 12 | `scene-flex/` 6 文件（`scene.xml`、`model-{312,315}.json`、`scene-{312,315}.mjb`、`provenance.json`）+ `reference/base/` 5 文件（`rgb/depth/seg/state.npz`、`capture.json`）+ `scripts/authoring/capture_render_flex.py` |
| `guard_inputs.kind=guard_script` | 1 | 冻结器自身（`freeze-guards-r06-s3b-flex.ps1`） |

- 输入条目**逐条已带 `sha256 + bytes`**；13 条按 repo-relative path **ordinal 排序**后对 sha256 十六进制串做 SHA-256 得
  `aggregate_inputs_sha256 = a46a4200…`（跨机器与 locale 稳定，脚本用 `[StringComparer]::Ordinal`）。
- 2 个 exe 条目在本片是 **`pending_freeze: true`**（`sha256/bytes = null`），由开窗卡的真实冻结解析
  `cargo` 产物、拷贝 stash、填 sha/bytes 并把 `status` 从 `plan` 置 `frozen`，最后输出 `GUARDS_OK=<N>/<N>` + stash 计数。
  本片按卡面「不做真冻结」不解析/不冻结 exe，避免把陈旧 `target/` 二进制写进守卫。

## 3. 静态自检（`self-check.txt` 为机器可读原文）

- **PSParser**：`[System.Management.Automation.Language.Parser]::ParseFile` → `PSPARSE_ERRORS=0`（PS 5.1）。
- **编码/行数**：`nonascii=0`、`cr=0`、`bom=false`、`lines=188 ≤ 200`、`max_line_length=151`、`comment_lines=10`、`blank_lines=7`。
- **结构自检**：`-GuardsOnly` → `GUARDS_STRUCTURE_OK suite=render-r06-s3b-flex exes_pending=2 inputs=13
  inputs_sha256_ok=true aggregate_inputs_sha256=a46a4200… classification=input=12,guard_script=1`（不读真实二进制、不跑 cargo）。
- 未跑 `-Attempt`、未建 stash、未写任何 target 产物；`fixtures/`、`src/`、既有 R02/R08/SDF 守卫文件零改动。

## 4. 与 R08 先例的差异（`freeze-guards-r08-rgb-hetero.ps1` 165 行 / 77 条目）

| 维度 | R08 | R06-S3b⑤ |
| --- | --- | --- |
| 条目 | 9 exe + 68 输入 = 77 | 2 exe + 13 输入 = 15 |
| 真冻结 | authoring 窗口内解析 9 个 cargo 测试 exe 并 stash | **本片不做**：`-Attempt` 留给开窗卡（Chief GO 后） |
| 静态自检 | 无独立 `-GuardsOnly` | 新增 `-GuardsOnly`（结构 + 13 输入 sha/bytes 复核 + aggregate 复核） |
| 行数 | 165 | 188（≤200 卡面） |
| 分类 | 仅 exe/input | 新增 `kind` 字段区分 `input`/`guard_script`；exe 带 `pending_freeze` |

## 5. 命令

```
# 本片已执行的静态自检（零 cargo / 零 GPU / 零二进制读取）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1 -GuardsOnly

# 开窗卡真冻结（Chief GO 后；本片不执行）
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/freeze-guards-r06-s3b-flex.ps1 -Attempt 1
```

## 6. 门禁

- fmt / clippy / clippy-cuda / unit 全过；零 GPU；`fixtures/`、`src/render/kernel/cuda/` 零改动（`unchanged`）。
