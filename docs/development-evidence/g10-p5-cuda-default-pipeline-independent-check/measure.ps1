# G10@P5 CUDA default-pipeline (stage 1) independent measurements.
# QA owns this script; writes raw JSON summary to target/qa-default-pipeline/summary.json.
# NOTE: keep ASCII-only (PowerShell 5.1 reads .ps1 as ANSI).

param(
    [string]$Repo = 'D:\Rust\mjwarp-rs',
    [string]$OutDir = 'target\qa-default-pipeline'
)
$ErrorActionPreference = 'Continue'
$work = Join-Path $Repo $OutDir
New-Item -ItemType Directory -Force -Path $work | Out-Null
$log = Join-Path $work 'measure.log'
Remove-Item $log -ErrorAction SilentlyContinue

function Note([string]$text) {
    $line = "[{0}] {1}" -f (Get-Date).ToString('HH:mm:ss.fff'), $text
    Add-Content -Path $log -Value $line -Encoding UTF8
    Write-Output $line
}

function DirMiB([string]$path) {
    if (-not (Test-Path $path)) { return -1 }
    $sum = (Get-ChildItem $path -Recurse -File -ErrorAction SilentlyContinue |
        Measure-Object -Property Length -Sum).Sum
    if ($null -eq $sum) { return 0 }
    return [math]::Round($sum / 1048576.0, 1)
}

function RunCmd([string]$label, [string]$targetDir, [string[]]$arguments) {
    $full = Join-Path $Repo $targetDir
    if (Test-Path $full) { Remove-Item $full -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $full | Out-Null
    $env:CARGO_TARGET_DIR = $full
    $command = 'cargo ' + ($arguments -join ' ')
    Note "run label=$label target=$targetDir command=$command"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & cargo @arguments 2>&1
    $code = $LASTEXITCODE
    $sw.Stop()
    $seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    $text = ($output | ForEach-Object { [string]$_ }) -join "`n"
    Add-Content -Path $log -Value "--- output begin label=$label exit=$code seconds=$seconds ---" -Encoding UTF8
    Add-Content -Path $log -Value $text -Encoding UTF8
    Add-Content -Path $log -Value "--- output end label=$label ---" -Encoding UTF8
    Note "done label=$label exit=$code seconds=$seconds target_mib=$(DirMiB $full)"
    return @{
        label = $label
        target_dir = $targetDir
        command = $command
        exit = $code
        seconds = $seconds
        target_mib = (DirMiB $full)
        output_tail = (($output | Select-Object -Last 8) -join "`n")
    }
}

function RunCmdIncremental([string]$label, [string]$targetDir, [string[]]$arguments) {
    $full = Join-Path $Repo $targetDir
    $env:CARGO_TARGET_DIR = $full
    $command = 'cargo ' + ($arguments -join ' ')
    Note "run label=$label target=$targetDir command=$command (incremental, dir kept)"
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & cargo @arguments 2>&1
    $code = $LASTEXITCODE
    $sw.Stop()
    $seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    $text = ($output | ForEach-Object { [string]$_ }) -join "`n"
    Add-Content -Path $log -Value "--- output begin label=$label exit=$code seconds=$seconds ---" -Encoding UTF8
    Add-Content -Path $log -Value $text -Encoding UTF8
    Add-Content -Path $log -Value "--- output end label=$label ---" -Encoding UTF8
    $ok = ($text -match 'test result: ok\.')
    $summary = ($text -split "`n" | Where-Object { $_ -match 'test result:' }) -join ' | '
    $failLines = ($text -split "`n" | Where-Object { $_ -match 'FAILED|failures:' }) -join ' | '
    Note "done label=$label exit=$code seconds=$seconds result_lines=$($summary -replace "`n",' ')"
    return @{
        label = $label
        target_dir = $targetDir
        command = $command
        exit = $code
        seconds = $seconds
        target_mib = (DirMiB $full)
        test_result = $summary
        all_ok = $ok
        failure_lines = $failLines
        output_tail = (($output | Select-Object -Last 12) -join "`n")
    }
}

function ExeMiB([string]$targetDir, [string]$profile) {
    $path = Join-Path $Repo ($targetDir + '\' + $profile + '\mjwarp-rs.exe')
    if (-not (Test-Path $path)) { return -1 }
    return [math]::Round((Get-Item $path).Length / 1048576.0, 2)
}

$results = [ordered]@{}
Note 'qa=independent-measurements task=cuda-default-pipeline-stage1'
Note ("repo_head=" + (& git -C $Repo rev-parse HEAD).Trim())
Note ("cargo=" + ((& cargo --version) -join ' '))
Note ("rustc=" + ((& rustc --version) -join ' '))
Note ("fresh_target_root=" + (Join-Path $Repo $OutDir))
$nvccBefore = (@(Get-Command nvcc -ErrorAction SilentlyContinue | ForEach-Object { $_.Source }) -join ';')
Note "nvcc_before=$nvccBefore"

$r = RunCmd 'debug-default-build' 'target\qa-default-pipeline\t-default' @('build','--locked')
$r.exe_mib = ExeMiB 'target\qa-default-pipeline\t-default' 'debug'
$results['debug_default_build'] = $r

$r = RunCmdIncremental 'debug-default-test-norun' 'target\qa-default-pipeline\t-default' @('test','--locked','--lib','--no-run')
$results['debug_default_test_norun'] = $r

$r = RunCmdIncremental 'debug-default-test-run' 'target\qa-default-pipeline\t-default' @('test','--locked','--lib')
$results['debug_default_test_run'] = $r

$r = RunCmd 'debug-probe-build' 'target\qa-default-pipeline\t-probe' @('build','--locked','--features','cuda-probe')
$r.exe_mib = ExeMiB 'target\qa-default-pipeline\t-probe' 'debug'
$results['debug_probe_build'] = $r

$r = RunCmdIncremental 'debug-probe-test-norun' 'target\qa-default-pipeline\t-probe' @('test','--locked','--features','cuda-probe','--lib','--no-run')
$results['debug_probe_test_norun'] = $r

$r = RunCmdIncremental 'debug-probe-test-run' 'target\qa-default-pipeline\t-probe' @('test','--locked','--features','cuda-probe','--lib')
$results['debug_probe_test_run'] = $r

$r = RunCmd 'release-default-build' 'target\qa-default-pipeline\r-default' @('build','--locked','--release')
$r.exe_mib = ExeMiB 'target\qa-default-pipeline\r-default' 'release'
$results['release_default_build'] = $r

$r = RunCmd 'release-probe-build' 'target\qa-default-pipeline\r-probe' @('build','--locked','--release','--features','cuda-probe')
$r.exe_mib = ExeMiB 'target\qa-default-pipeline\r-probe' 'release'
$results['release_probe_build'] = $r

# no-CUDA environment simulation
$origPath = $env:PATH
$removedEnv = New-Object System.Collections.Generic.List[string]
$keptPath = @($origPath -split ';' | Where-Object { $_ -ne '' -and $_ -notmatch '(?i)cuda' })
$removedPath = @($origPath -split ';' | Where-Object { $_ -ne '' -and $_ -match '(?i)cuda' })
$env:PATH = $keptPath -join ';'
Get-ChildItem env: | Where-Object { $_.Name -match '(?i)^CUDA' } | ForEach-Object {
    $removedEnv.Add($_.Name)
    Remove-Item ("env:" + $_.Name) -ErrorAction SilentlyContinue
}
$nvccAfter = (@(Get-Command nvcc -ErrorAction SilentlyContinue | ForEach-Object { $_.Source }) -join ';')
$cudaEnvLeft = @(Get-ChildItem env: | Where-Object { $_.Name -match '(?i)^CUDA' } | ForEach-Object { $_.Name })
$noCudaEnv = @{
    removed_path_entries = $removedPath
    removed_path_count = $removedPath.Count
    kept_path_has_cargo = ($keptPath -join ';') -match 'cargo'
    removed_env_names = $removedEnv
    cuda_env_left = $cudaEnvLeft
    nvcc_after = $nvccAfter
}
Note ("no_cuda_env: removed_path_entries=" + ($removedPath -join '|'))
Note ("no_cuda_env: removed_env=" + ($removedEnv -join ','))
Note ("no_cuda_env: nvcc_after='" + $nvccAfter + "'")

$r = RunCmd 'nocuda-default-build' 'target\qa-default-pipeline\t-nocuda-default' @('build','--locked')
$r.exe_mib = ExeMiB 'target\qa-default-pipeline\t-nocuda-default' 'debug'
$results['nocuda_default_build'] = $r

$r = RunCmdIncremental 'nocuda-default-test-run' 'target\qa-default-pipeline\t-nocuda-default' @('test','--locked','--lib')
$results['nocuda_default_test_run'] = $r

$r = RunCmd 'nocuda-probe-build' 'target\qa-default-pipeline\t-nocuda-probe' @('build','--locked','--features','cuda-probe')
$r.exe_mib = ExeMiB 'target\qa-default-pipeline\t-nocuda-probe' 'debug'
$results['nocuda_probe_build'] = $r

$r = RunCmdIncremental 'nocuda-probe-test-norun' 'target\qa-default-pipeline\t-nocuda-probe' @('test','--locked','--features','cuda-probe','--lib','--no-run')
$results['nocuda_probe_test_norun'] = $r

$env:PATH = $origPath
foreach ($name in $removedEnv) {
    $value = [Environment]::GetEnvironmentVariable($name, 'Process')
    if ($null -ne $value) { Set-Item ("env:" + $name) $value }
}
Note 'restored PATH and env'

$payload = [ordered]@{
    task = '2026-10-25-g10-p5-cuda-default-pipeline-independent-check'
    stage = 'stage1-independent-measurements'
    machine = @{
        cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name)
        gpu = ((& nvidia-smi --query-gpu=name --format=csv,noheader) -join '')
        driver = ((& nvidia-smi --query-gpu=driver_version --format=csv,noheader) -join '')
        cargo = ((& cargo --version) -join ' ')
        rustc = ((& rustc --version) -join ' ')
        method = 'fresh CARGO_TARGET_DIR per config (dir deleted before build), serial single measurement; size = sum of all files under target dir (MiB=1048576B)'
    }
    head = (& git -C $Repo rev-parse HEAD).Trim()
    no_cuda_env = $noCudaEnv
    debug = @{
        default_build = $results['debug_default_build']
        default_test_norun = $results['debug_default_test_norun']
        default_test_run = $results['debug_default_test_run']
        probe_build = $results['debug_probe_build']
        probe_test_norun = $results['debug_probe_test_norun']
        probe_test_run = $results['debug_probe_test_run']
    }
    release = @{
        default_build = $results['release_default_build']
        probe_build = $results['release_probe_build']
    }
    no_cuda = @{
        default_build = $results['nocuda_default_build']
        default_test_run = $results['nocuda_default_test_run']
        probe_build = $results['nocuda_probe_build']
        probe_test_norun = $results['nocuda_probe_test_norun']
    }
}
$json = $payload | ConvertTo-Json -Depth 8
$jsonPath = Join-Path $work 'summary.json'
[System.IO.File]::WriteAllText($jsonPath, $json + "`n", (New-Object System.Text.UTF8Encoding($false)))
Note ("summary_written=" + $jsonPath)
Write-Output $json
