# D6 工具缺陷修复：monitor 默认 CSV 目录不存在导致收尾崩溃

任务：`2026-11-04-render-bvh-monitor-default-csv-dir-fix`（owner Arch）。纯 CPU、零 GPU 任务；**与 attempt4 的判据结论无关**（attempt4 实体失败是设备侧 CUDA 700，判据未评估）。

## 1. 缺陷（attempt4 首跑实际踩中）

`scripts/validate/render-bvh/monitor-until-clean.ps1` 默认 `-OutCsv target/s3b-validation-window/monitor-pre-window.csv`。采样循环结束后无条件执行：

```powershell
$path = Join-Path (Get-Location).Path $OutCsv
[IO.File]::WriteAllText($path, ...)
```

若父目录 `target/s3b-validation-window/` 不存在（本仓瘦身后该目录不再存在），`WriteAllText` 抛 `DirectoryNotFoundException`，**已采集的样本全部丢失，且 samples/streak 汇总行也不打印**。attempt4 首跑 12:14:37–12:19 采满 60 个样本后即因此崩溃（facts 见 `r06-s3b-flex-window/attempts/attempt-4-facts.md` §2）。

复现（修复前）：

```
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/validate/render-bvh/monitor-until-clean.ps1 -Required 1 -MaxMinutes 1
# → Exception calling "WriteAllText" ... Could not find a part of the path 'D:\Rust\mjwarp-rs\target\s3b-validation-window\monitor-pre-window.csv'
```

## 2. 修复（最小改动，只碰写盘前路径处理）

在写盘前解析父目录并在不存在时创建，路径非法时给出可读错误：

```powershell
$path = Join-Path (Get-Location).Path $OutCsv
$dir = Split-Path -Parent $path
if ([string]::IsNullOrWhiteSpace($dir)) { throw "monitor: invalid -OutCsv '$OutCsv' (no parent directory)" }
if (-not (Test-Path -LiteralPath $dir)) {
    try { [IO.Directory]::CreateDirectory($dir) | Out-Null }
    catch { throw "monitor: cannot create output directory '$dir': $($_.Exception.Message)" }
}
[IO.File]::WriteAllText($path, (($lines -join "`n") + "`n"), $utf8)
```

**未改动**：采样循环、`Required`/`MaxUtil`/`free>=10240` clean 判据、compute/build 过滤、`exit 20`（未达标）语义、汇总行格式。

## 3. 自证（CPU-only，nvidia-smi 只读）

**nested 目录不存在 → 自动创建并落盘（exit 0）**：

```
> monitor-until-clean.ps1 -Required 1 -MaxMinutes 1 -OutCsv "target/monitor-selftest-nested/dir/monitor.csv"
monitor samples=1 longest_clean=1 required=1 ready=True start=2026-11-04T12:32:46.412+08:00 end=2026-11-04T12:32:46.412+08:00 util_min=7 util_max=7 free_min=14281
EXIT=0
```

CSV（2 行 = 表头 + 1 样本）：

```
ts,used_mib,free_mib,util_percent,temp_c,compute_raw,compute_filtered,build,clean,streak
2026-11-04T12:32:46.412+08:00,1781,14281,7,28,33,0,0,True,1
```

**非法路径 → 可读错误、不再抛原始异常**：

```
> monitor-until-clean.ps1 -Required 1 -MaxMinutes 1 -OutCsv "Z:\no-such-drive-xyz/monitor.csv"
monitor: cannot create output directory 'D:\Rust\mjwarp-rs\Z:\no-such-drive-xyz': Exception calling "CreateDirectory" ... "The given path's format is not supported."
EXIT=1
```

## 4. 边界

- 只改 `monitor-until-clean.ps1` 与该证据；不动 `src/`、guards、参考、attempt4 原记录，不重跑窗口、不改任何判据/容差。
- 本修复不改变任何「clean/floor/达标」语义，只保证采到的样本一定落盘、失败时错误可读。
- 后续窗口（attempt5 等）可直接使用默认 `-OutCsv`，或继续显式传证据目录路径。

## 5. 闸门

fmt / clippy / clippy-cuda / unit 全过。
