# QA G5 independent review: GPU window runner (own script, no delivery reuse).
# Discipline: baseline BEFORE window (this script records it first), one GPU task at a time,
# closing baseline + compute-apps check AFTER window.
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File qa_window_run.ps1
#   env QA_GPU_CONFIRMED=1 means Arch approved the window plan (baseline report sent first).

$ErrorActionPreference = "Stop"
$root = (Get-Location).Path
$evidence = Join-Path $root "docs\development-evidence\g10-p5-gpu-kernels-independent-check"
$work = Join-Path $evidence "qa-gpu"
$python = Join-Path $root "target\mjwarp-parallel\venv\Scripts\python.exe"
$probe = Join-Path $root "target\qa-g5\qa_probe.exe"
$log = Join-Path $work "window-log.txt"
$memory = Join-Path $work "window-memory.txt"

New-Item -ItemType Directory -Force -Path $work | Out-Null
$lines = @()
function Note([string]$text) {
  $stamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss.fff")
  $line = "[$stamp] $text"
  Write-Host $line
  $script:lines += $line
}
function GpuState([string]$label) {
  $gpu = & nvidia-smi --query-gpu=memory.used,memory.total,utilization.gpu --format=csv,noheader,nounits
  $apps = & nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader
  Note "$label gpu_used_total_util=$gpu"
  if ($apps) {
    foreach ($row in $apps) { Note "$label compute_app: $row" }
  } else {
    Note "$label compute_app: none"
  }
  return [pscustomobject]@{ gpu = $gpu; apps = $apps }
}

if ($env:QA_GPU_CONFIRMED -ne "1") {
  throw "QA_GPU_CONFIRMED not set: send the baseline report to Arch before opening the window"
}
if (-not (Test-Path $probe)) { throw "probe not built: $probe" }
if (-not (Test-Path $python)) { throw "python not found: $python" }

# ---------- pre-window baseline ----------
Note "=== pre-window baseline ==="
$before = GpuState "before"
$foreign = @()
if ($before.apps) {
  foreach ($row in $before.apps) {
    if ($row -notmatch "python.exe|qa_probe") { $foreign += $row }
  }
}
Note ("foreign_compute_apps_count=" + $foreign.Count)
Note ("wall_start=" + (Get-Date).ToString("o"))

Remove-Item -Force $memory -ErrorAction SilentlyContinue
$duration = 900
$sampler = Start-Job -ScriptBlock {
  param($out, $seconds)
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.Elapsed.TotalSeconds -lt $seconds) {
    $value = & nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits
    "$((Get-Date).ToString('HH:mm:ss.fff')) $value" | Add-Content -Path $out
    Start-Sleep -Milliseconds 500
  }
} -ArgumentList $memory, $duration

try {
  # ---------- 1) upstream f32 GPU capture (own script) + device input bins ----------
  Note "=== phase 1: own upstream f32 GPU capture + input bins ==="
  & $python (Join-Path $evidence "qa_prepare_capture.py") --device cuda 2>&1 | Tee-Object -FilePath (Join-Path $work "window-capture.log") | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "upstream capture failed: $LASTEXITCODE" }
  $stamp = Join-Path $evidence "qa-native-f64.json"
  Note ("upstream capture done; native json=" + (Test-Path $stamp))

  # ---------- 2) own device probe, two runs ----------
  $caseList = Join-Path $work "cases-all.txt"
  Get-ChildItem -Directory (Join-Path $work "cases") | Sort-Object Name | ForEach-Object { $_.Name } |
    Set-Content -Path $caseList -Encoding ascii
  $outRoot = Join-Path $work "out"
  New-Item -ItemType Directory -Force -Path $outRoot | Out-Null

  Note "=== phase 2: own device probe run 1 ==="
  & $probe (Join-Path $work "cases") $outRoot $caseList 2>&1 |
    Tee-Object -FilePath (Join-Path $work "window-probe-run1.log") | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "probe run1 failed: $LASTEXITCODE" }

  Note "=== phase 3: own device probe run 2 (determinism) ==="
  & $probe (Join-Path $work "cases") $outRoot $caseList run2 2>&1 |
    Tee-Object -FilePath (Join-Path $work "window-probe-run2.log") | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "probe run2 failed: $LASTEXITCODE" }
} finally {
  Stop-Job $sampler -ErrorAction SilentlyContinue | Out-Null
  Remove-Job $sampler -Force -ErrorAction SilentlyContinue | Out-Null
  Note "=== post-window ==="
  $after = GpuState "after"
  Note ("wall_end=" + (Get-Date).ToString("o"))
  $residual = @()
  if ($after.apps) {
    foreach ($row in $after.apps) {
      if ($row -match "qa_probe") { $residual += $row }
    }
  }
  Note ("residual_qa_probe_processes=" + $residual.Count)
  if (Test-Path $memory) {
    $values = Get-Content $memory | ForEach-Object { ($_ -split " ")[1] } | Where-Object { $_ -match "^\d+$" } | ForEach-Object { [int]$_ }
    if ($values.Count -gt 0) {
      Note ("memory_samples=" + $values.Count + " max_used_mib=" + ($values | Measure-Object -Maximum).Maximum +
            " min_used_mib=" + ($values | Measure-Object -Minimum).Minimum)
    } else {
      Note "memory_samples=0 (not captured)"
    }
  } else {
    Note "memory_samples=0 (not captured)"
  }
  $lines | Set-Content -Path $log -Encoding utf8
}
