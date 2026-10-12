$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = 'C:\Rust\mjwarp-rs'
Set-Location $root
$directory = $PSScriptRoot
$ledger = Join-Path $directory 'calls.json'
if (Test-Path -LiteralPath $ledger) { throw '本轮已经启动；禁止再次执行' }
$env:MJWARP_MUJOCO_ROOT = Join-Path $root 'target/toolchains/mujoco-3.12.0/package'
$env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin/mujoco.dll'
$env:CUDA_PATH = Join-Path $root 'target/toolchains/cuda-12.8.1'
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
$manifest = Get-Content fixtures/g01/manifest.json -Raw | ConvertFrom-Json
$dllHash = (Get-FileHash $env:MJWARP_MUJOCO_DLL).Hash.ToLowerInvariant()
if ($dllHash -ne $manifest.dll_sha256) { throw '原生DLL哈希不符' }
$metadata = Get-Content (Join-Path $directory 'cargo-metadata.json') -Raw | ConvertFrom-Json
$allTargets = @($metadata.packages[0].targets | Where-Object { $_.kind -contains 'test' } | ForEach-Object name)
$targets = @($allTargets | Where-Object { $_ -notin @('frame393_trace', 'native_model_probe') })
$old = Get-Content target/t4-redline-retest/environment.json -Raw | ConvertFrom-Json
if (@(Compare-Object @($old.productTargets) $targets).Count) { throw '产品目标清单发生变化，须重新核对' }
$diagnostic = 'physics::mass_solve::first_difference::trace_failed_state_by_matrix_factor_and_solve'
$script:calls = @()
$prior = [ordered]@{
    g01 = @{debug=3;release=3}
    resident_geom_tendon = @{debug=3;release=3}
    mass_matrix_probe = @{debug=3;release=3}
    mass_solve_probe = @{debug=5;release=4}
    mass_solve_unit = @{debug=5;release=4}
    frame393_trace = @{debug=3;release=3;additional=0}
    g03_first_difference = @{debug=3;release=0;additional=0}
}
[ordered]@{
    gitRevision = (& git rev-parse HEAD)
    workingTreeDirty = [bool](& git status --porcelain)
    recordedAtUtc = [DateTime]::UtcNow.ToString('o')
    authorization = '2026-10-12 user: one additional full debug and release validation, preserving coverage and tolerances'
    policy = 'native_error_v2'; relativeSlack = 0.05
    productTargets = $targets
    nativeTarget = 'native_model_probe via native wrapper, regular plus restricted runtime'
    excludedDiagnostics = @(@{target='frame393_trace';tests=2;reason='Capped diagnostic, not product acceptance'}, @{test=$diagnostic;tests=1;reason='Capped G03 isolation diagnostic, not product acceptance'})
    priorCallsSinceLimitInstruction = $prior
    priorEvidence = @('target/t4-redline-retest/calls.json','target/g03-block-reference/calls.json','target/g03-first-difference/calls.json','docs/windows-g03-first-difference.md')
    gpu = @(& nvidia-smi --query-gpu=name,uuid,driver_version,memory.total,memory.used --format=csv,noheader)
    rustcMsvc = (& rustc +stable-x86_64-pc-windows-msvc --version)
    rustcGnu = (& rustc --version)
    nativeDllSha256 = $dllHash; upstreamRevision = $manifest.upstream_revision
    nvrtcLibraries = @(Get-ChildItem (Join-Path $env:CUDA_PATH 'bin') -Filter 'nvrtc*.dll' | ForEach-Object { @{name=$_.Name;sha256=(Get-FileHash $_.FullName).Hash.ToLowerInvariant()} })
    originalWrapperInvoked = $false
    reason = 'Reuse the prior nonduplicating full product plan. Preserve all acceptance targets and add current regression tests. Do not retry capped standalone diagnostics.'
} | ConvertTo-Json -Depth 12 | Set-Content (Join-Path $directory 'environment.json') -Encoding utf8

function Invoke-Once {
    param([string]$Name, [string]$Description, [scriptblock]$Command)
    if (@($script:calls | Where-Object name -eq $Name).Count) { throw "禁止重试：$Name" }
    $entry = [ordered]@{name=$Name;invocation=1;command=$Description;status='started';startedUtc=[DateTime]::UtcNow.ToString('o');log=(Join-Path $directory "$Name.log");gpuBefore=@(& nvidia-smi --query-gpu=memory.used --format=csv,noheader)}
    $script:calls += $entry
    ConvertTo-Json -InputObject $script:calls -Depth 12 | Set-Content $ledger -Encoding utf8
    Write-Host "START $Name"
    $global:LASTEXITCODE = 0
    $output = @(); $errorText = $null
    $timer = [Diagnostics.Stopwatch]::StartNew()
    try { $output = @(& $Command *>&1); $code = $LASTEXITCODE }
    catch { $code=1; $errorText=$_.Exception.Message; $output += $_ }
    $timer.Stop()
    $output | ForEach-Object { $_.ToString() } | Set-Content $entry.log -Encoding utf8
    $text = ($output | ForEach-Object { $_.ToString() }) -join "`n"
    $counts = @([regex]::Matches($text, 'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;') | ForEach-Object {
        @{passed=[int]$_.Groups[1].Value;failed=[int]$_.Groups[2].Value;ignored=[int]$_.Groups[3].Value;filtered=[int]$_.Groups[5].Value}
    })
    $entry.status='finished'; $entry.exitCode=$code; $entry.error=$errorText
    foreach($key in @('passed','failed','ignored','filtered')) { $entry[$key]=[int](($counts|ForEach-Object { $_[$key] }|Measure-Object -Sum).Sum) }
    $entry.testBinaries=$counts.Count; $entry.elapsedSeconds=$timer.Elapsed.TotalSeconds
    $entry.gpuAfter=@(& nvidia-smi --query-gpu=memory.used --format=csv,noheader)
    ConvertTo-Json -InputObject $script:calls -Depth 12 | Set-Content $ledger -Encoding utf8
    $summaries = @([regex]::Matches($text,'NP03_SUMMARY (\{[^\r\n]+\})') | ForEach-Object {
        $v=$_.Groups[1].Value|ConvertFrom-Json
        if ($v.scope -match '^G0[13]-') { $v }
    })
    ConvertTo-Json -InputObject $summaries -Depth 26 | Set-Content (Join-Path $directory "$Name-np03.json") -Encoding utf8
    Write-Host "END $Name exit=$code passed=$($entry.passed) failed=$($entry.failed) ignored=$($entry.ignored) filtered=$($entry.filtered)"
    if ($code -ne 0) { $output | Select-Object -Last 12 | ForEach-Object { Write-Host $_ } }
}
function Invoke-CargoOnce {
    param([string]$Name, [string[]]$CargoArgs)
    if ($CargoArgs -contains 'frame393_trace') { throw '禁止独立诊断重跑' }
    Invoke-Once $Name ('cargo ' + ($CargoArgs -join ' ')) { & cargo @CargoArgs }
}

Invoke-CargoOnce 'fmt' @('fmt','--check')
foreach($mode in @('debug','release')) {
    $release = $mode -eq 'release'
    $env:MJWARP_PHYSICS_PTX_DIR = Join-Path $directory "$mode-ptx"
    Invoke-Once "$mode-native" "scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime -Release:$release" { & scripts/test-windows-native.ps1 -AllFeatures -RestrictedRuntime -Release:$release }
    if (-not $env:MJWARP_NATIVE_MOCKS) { throw '原生环境缺失' }
    $nativeReport = Get-Content (Join-Path $env:MJWARP_NATIVE_MOCKS 'report.json') -Raw | ConvertFrom-Json
    if ($nativeReport.passed -ne 27 -or $nativeReport.restrictedPassed -ne 27) { throw '原生覆盖不完整' }
    $nativeReport | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $directory "$mode-native-report.json") -Encoding utf8
    $env:MJWARP_NATIVE_LIFETIME_LOG = Join-Path $directory "$mode-lifetime.log"
    $env:MJWARP_PARALLEL_REPORT = Join-Path $directory "$mode-parallel.json"
    $base = @('+stable-x86_64-pc-windows-msvc','test','--locked','--all-features','--no-fail-fast')
    if($release) { $base += '--release' }
    $selected = @('--lib','--bins')
    foreach($target in $targets) { $selected += @('--test',$target) }
    Invoke-CargoOnce "$mode-products" ($base+$selected+@('--','--include-ignored','--test-threads=1','--nocapture','--skip',$diagnostic))
    Invoke-CargoOnce "$mode-doc-tests" ($base+@('--doc','--','--test-threads=1'))
    $nativeOnly = @('+stable-x86_64-pc-windows-msvc','test','--locked','--features','native-model-probe','--test','native_g01')
    if($release) { $nativeOnly += '--release' }
    Invoke-CargoOnce "$mode-native-only-g01" ($nativeOnly+@('--','--include-ignored','--test-threads=1'))
    $lint = @('+stable-x86_64-pc-windows-msvc','clippy','--locked','--all-features','--all-targets')
    if($release) { $lint += '--release' }
    Invoke-CargoOnce "$mode-clippy-msvc" ($lint+@('--','-D','warnings'))
}
$gnu = @('test','--locked','--no-fail-fast','--lib','--bins')
foreach($target in $allTargets | Where-Object { $_ -ne 'frame393_trace' }) { $gnu += @('--test',$target) }
Invoke-CargoOnce 'gnu-host' ($gnu+@('--','--test-threads=1'))
Invoke-CargoOnce 'gnu-doc-tests' @('test','--locked','--doc','--','--test-threads=1')
Invoke-CargoOnce 'gnu-clippy' @('clippy','--locked','--all-targets','--','-D','warnings')
Invoke-CargoOnce 'gnu-cuda-clippy' @('clippy','--locked','--all-targets','--features','cuda-probe','--','-D','warnings')
$oldFlags=$env:RUSTDOCFLAGS
try { $env:RUSTDOCFLAGS='-D warnings'; Invoke-CargoOnce 'rustdoc' @('+stable-x86_64-pc-windows-msvc','doc','--locked','--all-features','--no-deps') }
finally { $env:RUSTDOCFLAGS=$oldFlags }
Invoke-Once 'diff' 'git diff --check' { & git diff --check }
Write-Host "T4_EVIDENCE=$directory"
if (@($script:calls|Where-Object { $_.exitCode -ne 0 }).Count) { exit 1 }
