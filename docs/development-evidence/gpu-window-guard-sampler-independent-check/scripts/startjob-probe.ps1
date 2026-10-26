# Deeper probe: why did Start-Job sampling produce 0 rows here? (CPU only)
$ErrorActionPreference = 'Continue'
$Root = 'D:\Rust\mjwarp-rs'
$Out = Join-Path $Root 'target\qa-guard\trigger\startjob-probe2'
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$lines = New-Object System.Collections.ArrayList
function Add-Line([string]$s) { [void]$lines.Add($s); Write-Output $s }

# 1) job that only writes the clock (no nvidia-smi)
$clockCsv = Join-Path $Out 'clock.csv'
$job = Start-Job -ScriptBlock {
    param($f)
    "t" | Out-File -LiteralPath $f -Encoding ascii
    for ($i = 0; $i -lt 8; $i++) { Add-Content -LiteralPath $f -Value (Get-Date -Format o) -Encoding ascii; Start-Sleep -Milliseconds 200 }
} -ArgumentList $clockCsv
Start-Sleep -Seconds 2
$clockState = $job.State
Stop-Job -Job $job -ErrorAction SilentlyContinue; Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
$clockRows = if (Test-Path -LiteralPath $clockCsv) { (Get-Content -LiteralPath $clockCsv).Count } else { -1 }
Add-Line "job_clock_only state=$clockState rows=$clockRows (expect header+8 when job loop executes)"

# 2) job that calls nvidia-smi once and records exit code + output
$smiOut = Join-Path $Out 'smi-result.txt'
$job2 = Start-Job -ScriptBlock {
    param($f)
    $o = & nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits 2>&1
    $code = $LASTEXITCODE
    "exit=$code output=$($o -join ' | ')" | Out-File -LiteralPath $f -Encoding ascii
} -ArgumentList $smiOut
Start-Sleep -Seconds 2
$smiState = $job2.State
$job2Out = @(Receive-Job -Job $job2 -ErrorAction SilentlyContinue -Keep 2>&1 | ForEach-Object { [string]$_ }) | Select-Object -First 5
Stop-Job -Job $job2 -ErrorAction SilentlyContinue; Remove-Job -Job $job2 -Force -ErrorAction SilentlyContinue
$smiFile = if (Test-Path -LiteralPath $smiOut) { (Get-Content -LiteralPath $smiOut -Raw).Trim() } else { 'MISSING' }
Add-Line "job_nvidia_smi_once state=$smiState file=$smiFile job_output=$($job2Out -join ' | ')"

# 3) job with the exact old sampler loop but write to a debug file each iteration
$loopCsv = Join-Path $Out 'loop.csv'
$job3 = Start-Job -ScriptBlock {
    param($f)
    "t" | Out-File -LiteralPath $f -Encoding ascii
    $seq = 0
    while ($true) {
        $seq++
        $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null | Select-Object -First 1)
        Add-Content -LiteralPath $f -Value ("iter=$seq smi='$line'") -Encoding ascii
        Start-Sleep -Milliseconds 250
    }
} -ArgumentList $loopCsv
Start-Sleep -Seconds 2
$loopState = $job3.State
$job3Out = @(Receive-Job -Job $job3 -ErrorAction SilentlyContinue -Keep 2>&1 | ForEach-Object { [string]$_ }) | Select-Object -First 5
Stop-Job -Job $job3 -ErrorAction SilentlyContinue; Remove-Job -Job $job3 -Force -ErrorAction SilentlyContinue
$loopRows = if (Test-Path -LiteralPath $loopCsv) { (Get-Content -LiteralPath $loopCsv).Count } else { -1 }
$loopHead = if (Test-Path -LiteralPath $loopCsv) { (Get-Content -LiteralPath $loopCsv | Select-Object -First 4) -join ' || ' } else { 'MISSING' }
Add-Line "job_old_loop_with_debug state=$loopState rows=$loopRows head=$loopHead job_output=$($job3Out -join ' | ')"

$lines | Set-Content -LiteralPath (Join-Path $Out 'probe2.txt') -Encoding utf8
Add-Line 'PROBE2 DONE'
