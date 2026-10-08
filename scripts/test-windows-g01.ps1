[CmdletBinding()]
param([switch]$AllFeatures, [switch]$Release, [switch]$FullRegression)
$ErrorActionPreference = 'Stop'
if ($FullRegression -and -not $AllFeatures) { throw '完整回归需要AllFeatures' }
$root = Split-Path -Parent $PSScriptRoot
$directory = Join-Path $root ('target\g01-acceptance\' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $directory | Out-Null
$manifest = Get-Content -LiteralPath (Join-Path $root 'fixtures\g01\manifest.json') -Raw | ConvertFrom-Json
foreach ($file in $manifest.files) {
    $path = Join-Path $root "fixtures\g01\$($file.path)"
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $file.sha256) { throw "样本哈希不符：$($file.path)" }
}
$env:MJWARP_MUJOCO_ROOT = Join-Path $root 'target\toolchains\mujoco-3.12.0\package'
$env:MJWARP_MUJOCO_DLL = Join-Path $env:MJWARP_MUJOCO_ROOT 'bin\mujoco.dll'
$dllHash = (Get-FileHash -LiteralPath $env:MJWARP_MUJOCO_DLL -Algorithm SHA256).Hash.ToLowerInvariant()
if ($dllHash -ne $manifest.dll_sha256) { throw '原生DLL哈希不符' }
$env:CUDA_PATH = Join-Path $root 'target\toolchains\cuda-12.8.1'
if (-not (Test-Path -LiteralPath (Join-Path $env:CUDA_PATH 'bin'))) { throw '请先准备CUDA工具链' }
$env:PATH = (Join-Path $env:CUDA_PATH 'bin') + ';' + $env:PATH
$base = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked')
[string[]]$feature = if ($AllFeatures) { @('--all-features') } else { @('--features', 'native-model-probe,cuda-probe') }
$base += $feature
if ($Release) { $base += '--release' }
$script:g01Checks = @()
function Invoke-G01Check {
    param([string]$Name, [scriptblock]$Command, [int]$Expected = -1, [int]$Minimum = 0)
    $log = Join-Path $directory "$Name.log"
    $global:LASTEXITCODE = 0
    $output = & $Command *>&1
    $code = $LASTEXITCODE
    $output | Set-Content -LiteralPath $log -Encoding utf8
    if ($code -ne 0) { $output | Select-Object -Last 60 | Write-Host; throw "$Name 失败" }
    $counts = @([regex]::Matches(($output -join "`n"), 'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;') | ForEach-Object {
        [ordered]@{ passed = [int]$_.Groups[1].Value; failed = [int]$_.Groups[2].Value; ignored = [int]$_.Groups[3].Value }
    })
    if ($Name -eq 'resident') {
        $evidence = [regex]::Match(($output -join "`n"), 'RESIDENT_KINEMATICS_EVIDENCE=([^\r\n]+)')
        if (-not $evidence.Success) { throw '常驻回归证据缺失' }
        $script:residentEvidence = $evidence.Groups[1].Value.Trim()
        $report = Get-Content -LiteralPath (Join-Path $script:residentEvidence 'report.json') -Raw | ConvertFrom-Json
        $n = [int](($report.PSObject.Properties | Where-Object { $_.Name -match 'Passed$' } |
            ForEach-Object { [int]$_.Value } | Measure-Object -Sum).Sum)
        $counts = @([ordered]@{ passed = $n; failed = 0; ignored = 0 })
    }
    $passed = [int](($counts | ForEach-Object { $_.passed } | Measure-Object -Sum).Sum)
    $failed = [int](($counts | ForEach-Object { $_.failed } | Measure-Object -Sum).Sum)
    if ($failed -ne 0 -or $passed -lt $Minimum -or ($Expected -ge 0 -and $passed -ne $Expected)) {
        throw "$Name 测试计数无效：$passed"
    }
    $script:g01Checks += [ordered]@{ name = $Name; exitCode = $code; passed = $passed; counts = $counts; log = $log }
    $script:g01Checks | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $directory 'checks.json') -Encoding utf8
    Write-Host "$Name passed=$passed"
}
Push-Location $root
try {
    Invoke-G01Check 'fmt' { cargo fmt --check }
    Invoke-G01Check 'native' {
        if ($AllFeatures) { & scripts/test-windows-native.ps1 -AllFeatures -Release:$Release -RestrictedRuntime }
        else { & scripts/test-windows-native.ps1 -Gpu -Release:$Release -RestrictedRuntime }
    } -Expected 27
    $nativeDirectory = $env:MJWARP_NATIVE_MOCKS
    $nativeReport = Get-Content -LiteralPath (Join-Path $nativeDirectory 'report.json') -Raw | ConvertFrom-Json
    if ($nativeReport.restrictedPassed -ne 27) { throw '受限原生验收计数无效' }
    Invoke-G01Check 'host-msvc' { cargo @base } -Minimum 1
    Invoke-G01Check 'g01' { cargo @base --test g01 -- --include-ignored --test-threads=1 --nocapture } -Expected 7
    Invoke-G01Check 'native-g01' { cargo @base --test native_g01 -- --include-ignored --test-threads=1 } -Expected 2
    Invoke-G01Check 'native-only-g01' {
        $args = @('+stable-x86_64-pc-windows-msvc', 'test', '--locked', '--features', 'native-model-probe', '--test', 'native_g01')
        if ($Release) { $args += '--release' }
        cargo @args -- --include-ignored --test-threads=1
    } -Expected 1
    Invoke-G01Check 'resident' { & scripts/test-windows-resident-kinematics.ps1 -AllFeatures:$AllFeatures -Release:$Release } -Expected 98
    if ($FullRegression) {
        Invoke-G01Check 'all-probes' { cargo @base -- --ignored --test-threads=1 --nocapture } -Minimum 169
        Invoke-G01Check 'host-gnu' { cargo test --locked } -Minimum 200
        Invoke-G01Check 'clippy-gnu' { cargo clippy --locked --all-targets -- -D warnings }
        Invoke-G01Check 'clippy-gnu-cuda' { cargo clippy --locked --all-targets --features cuda-probe -- -D warnings }
    }
    Invoke-G01Check 'clippy-msvc' {
        $args = @('+stable-x86_64-pc-windows-msvc', 'clippy', '--locked', '--all-targets') + $feature
        if ($Release) { $args += '--release' }
        cargo @args -- -D warnings
    }
    Invoke-G01Check 'rustdoc' {
        $oldFlags = $env:RUSTDOCFLAGS
        try {
            $env:RUSTDOCFLAGS = '-D warnings'
            cargo +stable-x86_64-pc-windows-msvc doc --locked @feature --no-deps
        } finally { $env:RUSTDOCFLAGS = $oldFlags }
    }
    Invoke-G01Check 'diff' { git diff --check }
    $text = Get-Content -LiteralPath (Join-Path $directory 'g01.log') -Raw
    $stats = [regex]::Match($text, 'G01-complete states=(\d+) scalars=(\d+) max_abs_error=([0-9.eE+-]+)')
    $raw = [regex]::Match($text, 'G01 raw scalars=(\d+) max_abs_error=([0-9.eE+-]+)')
    if (-not $stats.Success -or [int]$stats.Groups[1].Value -ne 2084 -or
        [int]$stats.Groups[2].Value -ne 7704548 -or -not $raw.Success -or [int]$raw.Groups[1].Value -ne 50512) {
        throw 'G01浮点比较计数无效'
    }
    $gpu = & nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'GPU信息查询失败' }
    [ordered]@{
        platform = 'Windows x86_64 MSVC'
        gpu = @($gpu)
        rustc = (& rustc +stable-x86_64-pc-windows-msvc --version)
        gitRevision = (& git rev-parse HEAD)
        workingTreeDirty = [bool](& git status --porcelain)
        recordedAtUtc = [DateTime]::UtcNow.ToString('o')
        upstreamRevision = $manifest.upstream_revision
        feature = $feature
        release = [bool]$Release
        fullRegression = [bool]$FullRegression
        completeStates = 2084
        completeScalars = 7704548
        maxAbsoluteError = [double]::Parse($stats.Groups[3].Value, [Globalization.CultureInfo]::InvariantCulture)
        rawScalars = 50512
        rawMaxAbsoluteError = [double]::Parse($raw.Groups[2].Value, [Globalization.CultureInfo]::InvariantCulture)
        nativePassed = $nativeReport.passed
        nativeRestrictedPassed = $nativeReport.restrictedPassed
        nativeEvidence = $nativeDirectory
        residentEvidence = $script:residentEvidence
        checks = $script:g01Checks
        dllSha256 = $dllHash
        fixtureManifest = 'fixtures/g01/manifest.json'
        scriptInvokesNativePhysics = $false
        scriptInvokesModelCompiler = $false
        scriptInvokesPython = $false
        linuxVerified = $false
    } | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $directory 'report.json') -Encoding utf8
} finally { Pop-Location }
Write-Host "G01_EVIDENCE=$directory"
