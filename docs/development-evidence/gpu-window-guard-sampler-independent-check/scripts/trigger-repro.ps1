# QA trigger reproduction for the guard sampler fix (CPU only, read-only nvidia-smi)
$ErrorActionPreference = 'Continue'
$Root = 'D:\Rust\mjwarp-rs'
$Out = Join-Path $Root 'target\qa-guard\trigger'
New-Item -ItemType Directory -Force -Path $Out | Out-Null
$lines = New-Object System.Collections.ArrayList

# ---------- 1. pre-fix parser defect (as in parent commit 627b989d) ----------
$line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu --format=csv,noheader 2>&1 | Select-Object -First 1)
[void]$lines.Add("live_line=$line")
$oldInside = {
    param($raw)
    $parts = @(($raw -split ',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -match '^\d+' })
    if ($parts.Count -lt 3) { throw "GUARD_ABORT: cannot parse nvidia-smi memory/util line: $raw" }
    return @{ used = [int]$parts[0]; free = [int]$parts[1]; util = [int]$parts[2] }
}
$oldError = ''
$oldType = 'none'
try { $r = & $oldInside ([string]$line); [void]$lines.Add("old_parser_result=OK used=$($r.used)") } catch {
    $oldError = $_.Exception.Message
    $oldType = if ($oldError -match 'Int32') { 'Int32_conversion_exception' } else { 'other' }
    [void]$lines.Add("old_parser_result=THROW type=$oldType")
    [void]$lines.Add("old_parser_error=$($oldError -replace "`r|`n", ' ')")
}
[void]$lines.Add("old_parser_defect_reproduced=$($oldType -eq 'Int32_conversion_exception')")

# ---------- 2. old-style Start-Job sampler with a relative CSV path (samples=0 mechanism) ----------
$jobDir = Join-Path $Out 'startjob'
New-Item -ItemType Directory -Force -Path $jobDir | Out-Null
Set-Location $jobDir
$csvRel = 'samples.csv'
"time,used,free,util,temp" | Out-File -LiteralPath (Join-Path $jobDir $csvRel) -Encoding ascii
$homeCsv = Join-Path $env:USERPROFILE $csvRel
$homeRowsBefore = if (Test-Path -LiteralPath $homeCsv) { @(Get-Content -LiteralPath $homeCsv).Count } else { 0 }
$job = Start-Job -ScriptBlock {
    param($f)
    $seq = 0
    while ($true) {
        $line = (& nvidia-smi --query-gpu=memory.used,memory.free,utilization.gpu,temperature.gpu --format=csv,noheader,nounits 2>$null | Select-Object -First 1)
        if ($line) { $seq++; Add-Content -LiteralPath $f -Value ("{0},{1}" -f (Get-Date -Format 'o'), $line) -Encoding ascii }
        Start-Sleep -Milliseconds 250
    }
} -ArgumentList $csvRel
Start-Sleep -Seconds 2
Stop-Job -Job $job -ErrorAction SilentlyContinue
Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
$expectedRows = @(Import-Csv -LiteralPath (Join-Path $jobDir $csvRel) -ErrorAction SilentlyContinue).Count
$homeRowsAfter = if (Test-Path -LiteralPath $homeCsv) { @(Get-Content -LiteralPath $homeCsv).Count } else { 0 }
[void]$lines.Add("startjob_expected_csv_rows=$expectedRows (header-only expected: 0 rows)")
[void]$lines.Add("startjob_home_csv=$homeCsv exists=$(Test-Path -LiteralPath $homeCsv) rows_before=$homeRowsBefore rows_after=$homeRowsAfter")
[void]$lines.Add("startjob_relative_path_misplaced_rows=$([bool](Test-Path -LiteralPath $homeCsv) -and $homeRowsAfter -gt $homeRowsBefore)")

# ---------- 3. stage-2 attempt4 raw artifacts (machine-local) ----------
$attempt4 = Join-Path $Root 'target\stage2-impl\window\attempt4'
$facts = Join-Path $attempt4 'facts.txt'
$csv4 = Join-Path $attempt4 'samples.csv'
if (Test-Path -LiteralPath $facts) {
    $factText = [IO.File]::ReadAllText($facts)
    $samplesLine = (@($factText -split "`n" | Where-Object { $_ -match '^samples=' }) -join '')
    [void]$lines.Add("stage2_attempt4_facts_samples_line=$samplesLine")
    [void]$lines.Add("stage2_attempt4_facts_contains_samples0=$($factText -match 'samples=0')")
} else { [void]$lines.Add('stage2_attempt4_facts=MISSING') }
if (Test-Path -LiteralPath $csv4) {
    $csv4Lines = [IO.File]::ReadAllLines($csv4)
    [void]$lines.Add("stage2_attempt4_samples_csv_bytes=$((Get-Item -LiteralPath $csv4).Length) lines=$($csv4Lines.Count) header=$($csv4Lines[0])")
} else { [void]$lines.Add('stage2_attempt4_samples_csv=MISSING') }

# ---------- 4. capture-phase [int] abort record (committed evidence) ----------
$captureReadme = Join-Path $Root 'docs\development-evidence\g10-p5-g2-gpu-reference-capture\README.md'
if (Test-Path -LiteralPath $captureReadme) {
    $cap = [IO.File]::ReadAllText($captureReadme)
    $termOverride = -join (0x5C40, 0x90E8, 0x8986, 0x76D6 | ForEach-Object { [char]$_ })
    $termIntConv = -join (0x7C7B, 0x578B, 0x8F6C, 0x6362, 0x5931, 0x8D25 | ForEach-Object { [char]$_ })
    $termAttempts = -join (0x7A97, 0x53E3, 0x5C1D, 0x8BD5, 0x8BB0, 0x5F55 | ForEach-Object { [char]$_ })
    [void]$lines.Add("capture_record_local_override_documented=$($cap.Contains($termOverride))")
    [void]$lines.Add("capture_record_int_conversion_abort_documented=$($cap.Contains($termIntConv))")
    [void]$lines.Add("capture_record_attempts_section_documented=$($cap.Contains($termAttempts))")
} else { [void]$lines.Add('capture_record_readme=MISSING') }

Set-Location $Root
$logPath = Join-Path $Out 'trigger-repro.txt'
$lines | Set-Content -LiteralPath $logPath -Encoding utf8
$lines | ForEach-Object { Write-Output $_ }
Write-Output 'TRIGGER REPRO DONE'
