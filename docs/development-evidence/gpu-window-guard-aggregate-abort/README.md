# GPU 窗口守卫：abort 判定改为消费采样器聚合（B2 开窗硬前置）

- 任务：`2026-10-26-gpu-window-guard-aggregate-abort`（owner Arch）。
- 触发：B1 attempt1 暴露守卫缺陷——外部任务在窗口内返回，采样记录显示 `used_peak=7071 MiB`、
  `free_min=8991 MiB`（< 10240 地板）、`util_max=98%`，而基于离散点的 `Assert-Guard` 未 abort。
  Chief 2026-10-26 要求：B2 起 abort 判定改为消费采样器聚合（peak delta / free_min），
  与离散采样无关。

## 1. 实现

`scripts/gpu-window-guard.ps1` 新增 `Assert-GuardAggregates`：

- 输入：`-State`（沿用 `New-GuardState` 的阈值字段）、`-CsvPath`（0.25 s 采样 CSV）或 `-Rows`（注入行，供 dry-run）、
  可选 `-OwnGpuStepActive`（有本任务 GPU 步时不判 util 连续段）。
- 计算：`used_peak`、`free_min`、`util_max`、`delta_peak = used_peak - baseline_used`、
  `util` 最长连续段（≥ `util_abort_percent`）、`limit = min(increment_limit_mib, total_mib/2)`。
- abort 规则（原因码）：
  - `agg_free_below_floor`：`free_min < free_floor_mib`；
  - `agg_increment_over_limit`：`delta_peak >= limit`；
  - `agg_sustained_util`：无 own step 且最长 util 连续段 ≥ `util_abort_consecutive`。
- 日志：`guard_aggregates samples=… baseline_used=… used_peak=… delta_peak=… limit=… free_min=… util_max=… util_streak=…`
  与 `guard_stop reason=agg_*`，格式与既有 `guard_sample`/`guard_stop` 一致。
- **阈值与既有 `Assert-Guard` 离散断言语义不变**（新增聚合层，不替换）。

## 2. 接线

B2 窗口 runner（`target/g10-p5-primitive-sphere-capsule/window/run-window-attempt1.ps1`，gitignored）在
`Stop-GuardSampler` 之后调用 `Assert-GuardAggregates -CsvPath samples.csv -OwnGpuStepActive`，
把结果写入 `facts.txt` 的 `guard_aggregate=` 行；聚合 abort 时窗口退出码 5，attempt 如实记 aborted、不拼接。

## 3. 自测（纯 CPU，无 CUDA context、无 GPU）

`pwsh -NoProfile -File scripts/gpu-window-guard-dryrun.ps1 -OutDir docs/development-evidence/gpu-window-guard-aggregate-abort/dryrun-out`

**ASSERTIONS total=23 pass=23 fail=0；DRYRUN RESULT: PASS**（原 19 条 + 新增 4 条聚合用例）：

| 用例 | 期望 | 结果 |
| --- | --- | --- |
| `aggregate_clean_window` | 无 abort，且 `used_peak=3367`/`free_min=12695`/`delta_peak=15`/`util_max=41` 精确匹配 | PASS |
| `aggregate_free_below_floor` | `agg_free_below_floor`（B1 attempt1 回归：used 7071 / free 8991 / util 98） | PASS |
| `aggregate_increment_over_limit` | `agg_increment_over_limit`（注入 `-TotalMib 4000` 使 limit=2000 可达） | PASS |
| `aggregate_sustained_util` | `agg_sustained_util`（util 30→85→90→88，无 own step） | PASS |

完整输出见 `dryrun-run.log`，各用例日志见 `dryrun-out/`（`injection-dryrun.log`、`injection-aggregate.log`、
`inject-*.log`、`samples.csv`、`sampler-dryrun.log`、`parser-dryrun.log`）。

## 4. 边界

- 不改阈值/日志格式语义；不宣称「守卫绝对可靠」。
- 不改窗口双签纪律；窗口内仍零编译，只跑预构建二进制。
- 本片只改 `scripts/`（+ 本证据目录），不触 `src/`/`tests/`，与 B2 在途实现无写路径重叠。
