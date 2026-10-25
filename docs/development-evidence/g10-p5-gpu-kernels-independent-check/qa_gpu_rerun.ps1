# QA G5 independent review: probe-only rerun under the >=1 GiB peak-increment guard.
# Same content as the approved window (device probe over the 22 case dirs, two runs);
# no upstream capture is repeated. Guard: if sampled (used - baseline) >= 1024 MiB the
# remaining phases are aborted immediately and reported (no "hard run to completion").
#
# Usage: env QA_GPU_CONFIRMED=1 ; powershell -File qa_gpu_rerun.ps1

$ErrorActionPreference = "Stop"
$root = (Get-Location).Path
$evidence = Join-Path $root "docs\development-evidence\g10-p5-gpu-kernels-independent-check"
$work = Join-Path $evidence "qa-gpu"
$probe = Join-Path $root "target\qa-g5\qa_probe.exe"
$log = Join-Path $work "window-probe-rerun-log.txt"
$memory = Join-Path $work "window-probe-rerun-memory.txt"
$alertFile = Join-Path $work "window-probe-rerun-alert.txt"

if ($env:QA_GPU_CONFIRMED -ne "1") { throw "QA_GPU_CONFIRMED not set" }
if (-not (Test-Path $probe)) { throw "probe missing: $probe" }

$lines = @()
function Note([string]$text) {
  $line = "[" + (Get-Date).ToString("yyyy-MM-dd HH:mm:ss.fff") + "] " + $text
  Write-Host $line
  $script:lines += $line
}
function GpuUsed { [int](& nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits) }

Remove-Item -Force $memory, $alertFile -ErrorAction SilentlyContinue
$baseline = GpuUsed
Note "baseline_used_mib=$baseline"
Note ("compute_apps_before=" + ((& nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader) -join " | "))
Note ("wall_start=" + (Get-Date).ToString("o"))

$sampler = Start-Job -ScriptBlock {
  param($out, $alert, $baseline, $threshold, $seconds)
  $total = [int](& nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits)
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.Elapsed.TotalSeconds -lt $seconds) {
    $used = [int](& nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits)
    $delta = $used - $baseline
    "$((Get-Date).ToString('HH:mm:ss.fff')) $used $delta" | Add-Content -Path $out
    if ((($delta -ge $threshold) -or ($used -gt 0.5 * $total)) -and -not (Test-Path $alert)) {
      "alert delta=$delta used=$used baseline=$baseline at $(Get-Date -Format 'o')" | Set-Content -Path $alert
    }
    Start-Sleep -Milliseconds 200
  }
} -ArgumentList $memory, $alertFile, $baseline, 3072, 600

$aborted = $false
try {
  $caseList = Join-Path $work "cases-all.txt"
  $outRoot = Join-Path $work "out"
  Note "=== rerun probe run 1 (fixed witness max_horizon) ==="
  & $probe (Join-Path $work "cases") $outRoot $caseList 2>&1 |
    Tee-Object -FilePath (Join-Path $work "window-probe-rerun1.log") | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "probe run1 failed: $LASTEXITCODE" }
  if (Test-Path $alertFile) {
    Note ("GUARD TRIGGERED after run1: " + (Get-Content $alertFile -Raw))
    $aborted = $true
  } else {
    Note "guard_after_run1=ok (total delta < 3072 MiB and <=50% device)"
  }
  if (-not $aborted) {
    Note "=== rerun probe run 2 (determinism) ==="
    & $probe (Join-Path $work "cases") $outRoot $caseList run2 2>&1 |
      Tee-Object -FilePath (Join-Path $work "window-probe-rerun2.log") | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "probe run2 failed: $LASTEXITCODE" }
    if (Test-Path $alertFile) {
      Note ("GUARD TRIGGERED after run2: " + (Get-Content $alertFile -Raw))
      $aborted = $true
    } else {
      Note "guard_after_run2=ok (total delta < 3072 MiB and <=50% device)"
    }
  }
} finally {
  Stop-Job $sampler -ErrorAction SilentlyContinue | Out-Null
  Remove-Job $sampler -Force -ErrorAction SilentlyContinue | Out-Null
  $after = GpuUsed
  Note "after_used_mib=$after"
  Note ("compute_apps_after=" + ((& nvidia-smi --query-compute-apps=pid,process_name --format=csv,noheader) -join " | "))
  Note ("wall_end=" + (Get-Date).ToString("o"))
  $residual = (& nvidia-smi --query-compute-apps=process_name --format=csv,noheader) |
    Where-Object { $_ -match "qa_probe" }
  Note ("residual_qa_probe=" + ($residual | Measure-Object).Count)
  if (Test-Path $memory) {
    $rows = Get-Content $memory | ForEach-Object { ($_ -split "\s+") | Where-Object { $_ -ne "" } }
    $deltas = $rows | ForEach-Object { [int]$_[2] }
    $peak = ($deltas | Measure-Object -Maximum).Maximum
    $peakIndex = [array]::IndexOf($deltas, $peak)
    Note ("samples=" + $rows.Count + " peak_delta_mib=" + $peak +
          " peak_used_mib=" + $rows[$peakIndex][1] +
          " peak_time=" + $rows[$peakIndex][0] + " baseline_mib=" + $baseline)
  }
  Note ("guard_threshold_mib=3072_or_50pct_device guard_triggered=" + $aborted)
  $lines | Set-Content -Path $log -Encoding utf8
  if ($aborted) { throw "guard triggered (>=3072 MiB or >50% device); stopped remaining phases (reported)" }
}
