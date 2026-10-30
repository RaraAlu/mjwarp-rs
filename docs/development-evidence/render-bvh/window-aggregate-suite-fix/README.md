# 窗口 runner：`guard_aggregate.suite` 从 guards 文件派生（R08-S3 QA O1；离线小修，未跑窗口）

任务：`2026-10-30-render-window-guard-aggregate-suite-hardcode`（backlog 卡，本片 start 落地）
背景：R08-S3 QA O1（同类问题 R02 时记为 B-O2/N12）——`run-window.ps1` 写 `guard_aggregate.json` 时把 `suite`
字段硬编码成 `r05-s3b-hfield-validate`，于是 R02/R08 等后续窗口的 aggregate 文件里 suite 名是错的（例：
R08-S3 的 aggregate 记 `r05-s3b-hfield-validate`，而 guards 文件 suite 为 `render-r08-rgb-hetero`）。
`aggregate_sha256` 与条目不受影响（QA 已复算 77 条 sha 串联一致），只是标签错。

## 1. 改动（一处逻辑，`scripts/validate/render-bvh/run-window.ps1`）

| 项 | 值 |
| --- | --- |
| diff | **+6 / −1**，只此一个文件 |
| 行数 / 字节 / sha256 | 386 → **391** / **18,491 B** / **`2a49503d68a73605c6cb426b57b7cccc38df96f788649ab47fb96839d6d1fe3b`** |
| 逻辑 | `$agg.suite` 由硬编码 `'r05-s3b-hfield-validate'` 改为 `$guard.suite`（`-GuardsFile`/默认 guards 文件的 `suite` 字段） |
| 缺省行为 | guards 文件没有 `suite` 字段时：保留旧值 `'r05-s3b-hfield-validate'`，并写一行 `AGGREGATE_SUITE_FALLBACK suite=... (guards file has no suite field; old behavior kept)` 到 `guard.log` |
| 未改 | `aggregate_sha256` 的算法/命名/条目语义不变（`aggregate_algorithm` 字符串、条目集合、排序、拼接、哈希全未动）；不碰任何 guards 文件；不改阈值/判据/实体表；ASCII 口径不变（非 ASCII 0）；PSParser 0 错 |

```diff
+$aggregateSuite = $guard.suite
+if (-not $aggregateSuite) {
+    $aggregateSuite = 'r05-s3b-hfield-validate'
+    Guard ("AGGREGATE_SUITE_FALLBACK suite=$aggregateSuite (guards file has no suite field; old behavior kept)")
+}
 $agg = [pscustomobject]@{
-    suite = 'r05-s3b-hfield-validate'
+    suite = $aggregateSuite
```

## 2. 离线自检（**未跑任何窗口、零 GPU、零设备执行**）

自检脚本：`materials/suite-derivation-check.ps1`（只读：解析 runner 文本 + 解析现有 guards 文件并重算 aggregate）。
日志：`logs/suite-derivation-check.txt`。执行方式：

```text
powershell -NoProfile -ExecutionPolicy Bypass -File docs\development-evidence\render-bvh\window-aggregate-suite-fix\materials\suite-derivation-check.ps1
```

结果（`SELF_CHECK_OK`，exit 0）：

| 检查 | 结果 |
| --- | --- |
| PSParser `run-window.ps1` | **parser_errors=0** |
| ASCII 口径 | **non_ascii_bytes=0** |
| 派生代码存在 | `$aggregateSuite = $guard.suite` ×1；`suite = $aggregateSuite` ×1；旧字面量恰好 1 次（仅 fallback） |
| `guards.json` | suite=`r05-s3b-hfield-validate`、73 条、aggregate 重算**一致** |
| `guards-r02-skybox.json` | suite=`r02-skybox-validate`、77 条、aggregate 重算**一致** |
| `guards-r08-rgb-hetero.json` | suite=`render-r08-rgb-hetero`、77 条、aggregate 重算**一致** |

aggregate 重算按 guards 文件 `aggregate_algorithm` 的语义：repo-relative path 排序 → 每条 sha256 hex 串无分隔拼接 → sha256；三份文件均与记录值逐位相同 ⇒ **aggregate 语义未被本改动影响**。

## 3. 边界与已知口径

- 只改 `run-window.ps1` 一处；**未跑任何窗口**、未读/写 `target/` 窗口目录、未动 R05/R02/R08 guards 与其历史 aggregate 文件（历史证据保留当时标签，不回改）。
- 新窗口（U4 等）开始后，`guard_aggregate.json` 的 `suite` 将等于所用 guards 文件的 `suite`；若某 guards 文件仍无 `suite` 字段，走 fallback 并在 `guard.log` 留一行。
- 不宣称任何窗口通过或数值结论；本片只是窗口工具的标签修正 + 离线自检。
